//! XRPC over reqwest: a stateless client bound to one PDS endpoint.
//!
//! Methods cover leafspring's read surface plus the app-password session
//! lifecycle. Writes (`putRecord`, `uploadBlob`) are stage 4 — see the
//! crate docs. The client holds no secrets; authenticated calls take the
//! [`Session`] explicitly, and neither the session JWTs nor any password
//! ever reach logs or error text.

use serde::Deserialize;
use serde_json::Value;

use crate::{Error, RawRecord, validate_nsid, validate_rkey};

/// An app-password session (`com.atproto.server.createSession` /
/// `refreshSession`). JWTs are secrets: `Debug` shows only the identity,
/// and no API in this crate logs or embeds the token strings.
#[derive(Clone)]
pub struct Session {
    pub access_jwt: String,
    pub refresh_jwt: String,
    pub did: String,
    pub handle: String,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("did", &self.did)
            .field("handle", &self.handle)
            .finish_non_exhaustive() // never the JWTs
    }
}

impl Session {
    fn from_wire(wire: SessionWire) -> Self {
        Session {
            access_jwt: wire.access_jwt,
            refresh_jwt: wire.refresh_jwt,
            did: wire.did,
            handle: wire.handle,
        }
    }
}

#[derive(Deserialize)]
struct SessionWire {
    #[serde(rename = "accessJwt")]
    access_jwt: String,
    #[serde(rename = "refreshJwt")]
    refresh_jwt: String,
    did: String,
    handle: String,
}

#[derive(Deserialize)]
struct UploadBlobWire {
    blob: Value,
}

/// One page of `com.atproto.repo.listRecords`.
#[derive(Debug, Clone, Deserialize)]
pub struct ListRecordsPage {
    pub records: Vec<RawRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// A `getRecord` result.
#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    pub uri: String,
    pub cid: Option<String>,
    pub value: Value,
}

/// A `putRecord` result: the record's at-uri and its new cid.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PutRecordOutput {
    pub uri: String,
    pub cid: String,
}

/// XRPC client bound to one PDS base URL.
///
/// `Debug` prints only the base URL (mirrors leafspring-drafts' client).
#[derive(Clone)]
pub struct XrpcClient {
    http: reqwest::Client,
    base_url: String,
}

impl std::fmt::Debug for XrpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XrpcClient").field("base_url", &self.base_url).finish_non_exhaustive()
    }
}

impl XrpcClient {
    pub fn new(pds_base_url: &str) -> Result<Self, Error> {
        let base = pds_base_url.trim().trim_end_matches('/');
        let parsed = reqwest::Url::parse(base).map_err(|_| Error::BadInput(format!("invalid PDS url {base:?}")))?;
        if parsed.scheme() != "https"
            && parsed.host_str() != Some("localhost")
            && !base.starts_with("http://127.0.0.1")
            && !base.starts_with("http://localhost")
        {
            return Err(Error::BadInput(format!("PDS url must be https (got {base:?})")));
        }
        let http = reqwest::Client::builder()
            .user_agent(concat!("leafspring/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            http,
            base_url: base.to_string(),
        })
    }

    fn url_for(&self, method: &str) -> String {
        format!("{}/xrpc/{method}", self.base_url)
    }

    /// GET query endpoint; non-2xx maps to [`Error::HttpStatus`] or
    /// [`Error::Xrpc`] when the body carries `{error, message}`.
    async fn query(&self, method: &str, params: &[(&str, String)]) -> Result<Value, Error> {
        let url = self.url_for(method);
        tracing::debug!(method = %method, "XRPC query");
        let response = self.http.get(&url).query(params).send().await?;
        Self::decode(method, response).await
    }

    /// POST procedure endpoint with a JSON body and optional bearer token.
    async fn procedure(&self, method: &str, body: &Value, bearer: Option<&str>) -> Result<Value, Error> {
        let url = self.url_for(method);
        tracing::debug!(method = %method, "XRPC procedure");
        let mut request = self.http.post(&url).json(body);
        if let Some(token) = bearer {
            request = request.bearer_auth(token);
        }
        let response = request.send().await?;
        Self::decode(method, response).await
    }

    /// Shared response handling: HTTP status + XRPC error body decode.
    async fn decode(method: &str, response: reqwest::Response) -> Result<Value, Error> {
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            // XRPC errors ride a JSON body {error, message}.
            if let Ok(v) = serde_json::from_str::<Value>(&text)
                && let Some(code) = v.get("error").and_then(Value::as_str)
            {
                let message = v.get("message").and_then(Value::as_str).unwrap_or_default();
                return Err(Error::Xrpc {
                    method: method.into(),
                    code: code.into(),
                    message: message.into(),
                });
            }
            return Err(Error::HttpStatus {
                method: method.into(),
                status: status.as_u16(),
            });
        }
        serde_json::from_str(&text).map_err(|e| Error::Decode(method.into(), e))
    }

    /// `com.atproto.server.createSession` with an app password.
    ///
    /// The password is used once for this request and dropped; it never
    /// appears in logs or errors (the request body is not part of any
    /// error path in this crate).
    pub async fn create_session(&self, identifier: &str, app_password: &str) -> Result<Session, Error> {
        let body = serde_json::json!({ "identifier": identifier, "password": app_password });
        let value = self.procedure("com.atproto.server.createSession", &body, None).await?;
        let wire: SessionWire = serde_json::from_value(value).map_err(|e| Error::Decode("com.atproto.server.createSession".into(), e))?;
        Ok(Session::from_wire(wire))
    }

    /// `com.atproto.server.refreshSession` with the refresh JWT.
    pub async fn refresh_session(&self, refresh_jwt: &str) -> Result<Session, Error> {
        let value = self
            .procedure("com.atproto.server.refreshSession", &serde_json::json!({}), Some(refresh_jwt))
            .await?;
        let wire: SessionWire = serde_json::from_value(value).map_err(|e| Error::Decode("com.atproto.server.refreshSession".into(), e))?;
        Ok(Session::from_wire(wire))
    }

    /// `com.atproto.repo.putRecord` — upsert a record at an explicit
    /// rkey (`putRecord`, never `createRecord`, per the design doc).
    /// Returns the record's uri and cid. The bearer token travels in the
    /// Authorization header only; nothing about the session is logged.
    pub async fn put_record(&self, session: &Session, repo: &str, collection: &str, rkey: &str, record: &Value) -> Result<PutRecordOutput, Error> {
        validate_nsid(collection)?;
        validate_rkey(rkey)?;
        let body = serde_json::json!({ "repo": repo, "collection": collection, "rkey": rkey, "record": record });
        let value = self.procedure("com.atproto.repo.putRecord", &body, Some(&session.access_jwt)).await?;
        serde_json::from_value(value).map_err(|e| Error::Decode("com.atproto.repo.putRecord".into(), e))
    }

    /// `com.atproto.repo.uploadBlob` — stream raw bytes with the given
    /// content type; returns the `blob` ref object (a `BlobRef`-shaped
    /// JSON value — this crate stays lexicon-agnostic). Downstream use
    /// (leaflet blocks, publication icons) embeds the returned value.
    pub async fn upload_blob(&self, session: &Session, bytes: Vec<u8>, content_type: &str) -> Result<Value, Error> {
        if !content_type.contains('/') || content_type.len() > 255 {
            return Err(Error::BadInput(format!("invalid content type {content_type:?}")));
        }
        let url = self.url_for("com.atproto.repo.uploadBlob");
        tracing::debug!(method = "com.atproto.repo.uploadBlob", bytes = bytes.len(), "XRPC upload");
        let response = self
            .http
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .bearer_auth(&session.access_jwt)
            .body(bytes)
            .send()
            .await?;
        let value = Self::decode("com.atproto.repo.uploadBlob", response).await?;
        let wire: UploadBlobWire = serde_json::from_value(value).map_err(|e| Error::Decode("com.atproto.repo.uploadBlob".into(), e))?;
        Ok(wire.blob)
    }

    /// `com.atproto.identity.resolveHandle` against this endpoint (usually
    /// an appview or a bsky.social host). Cross-check helper — the identity
    /// chain in [`crate::identity`] is authoritative for leafspring.
    pub async fn resolve_handle(&self, handle: &str) -> Result<String, Error> {
        let value = self.query("com.atproto.identity.resolveHandle", &[("handle", handle.to_string())]).await?;
        value.get("did").and_then(Value::as_str).map(str::to_string).ok_or_else(|| Error::Identity {
            input: handle.to_string(),
            reason: "resolveHandle response missing did".into(),
        })
    }

    /// `com.atproto.repo.listRecords` (public read, one page, ≤100).
    pub async fn list_records(&self, repo: &str, collection: &str, limit: u8, cursor: Option<&str>) -> Result<ListRecordsPage, Error> {
        validate_nsid(collection)?;
        let mut params = vec![("repo", repo.to_string()), ("collection", collection.to_string()), ("limit", limit.to_string())];
        if let Some(c) = cursor {
            params.push(("cursor", c.to_string()));
        }
        let value = self.query("com.atproto.repo.listRecords", &params).await?;
        serde_json::from_value(value).map_err(|e| Error::Decode("com.atproto.repo.listRecords".into(), e))
    }

    /// `listRecords` with the cursor followed to exhaustion.
    pub async fn list_all_records(&self, repo: &str, collection: &str) -> Result<Vec<RawRecord>, Error> {
        let mut out = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let page = self.list_records(repo, collection, 100, cursor.as_deref()).await?;
            out.extend(page.records);
            match page.cursor {
                Some(next) if !next.is_empty() => cursor = Some(next),
                _ => return Ok(out),
            }
        }
    }

    /// `com.atproto.repo.getRecord` (public read). `Ok(None)` when the
    /// record does not exist (mapped from the XRPC not-found error).
    pub async fn get_record(&self, repo: &str, collection: &str, rkey: &str) -> Result<Option<Record>, Error> {
        validate_nsid(collection)?;
        let value = self
            .query(
                "com.atproto.repo.getRecord",
                &[("repo", repo.to_string()), ("collection", collection.to_string()), ("rkey", rkey.to_string())],
            )
            .await;
        match value {
            Ok(v) => {
                let record: Record = serde_json::from_value(v).map_err(|e| Error::Decode("com.atproto.repo.getRecord".into(), e))?;
                Ok(Some(record))
            }
            Err(Error::Xrpc { code, message, .. }) if code == "RecordNotFound" || code == "InvalidRequest" && message.contains("ould not locate") => Ok(None),
            Err(other) => Err(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    const CREATE_SESSION_OK: &str = r#"{
        "accessJwt": "eyJhbGciOiJIUzI1NiJ9.access-secret",
        "refreshJwt": "eyJhbGciOiJIUzI1NiJ9.refresh-secret",
        "handle": "jauntywunderkind.bsky.social",
        "did": "did:plc:zjbq26wybii5ojoypkso2mso",
        "email": "redacted@example.com",
        "active": true
    }"#;

    #[test]
    fn session_decode_and_debug_redaction() {
        let wire: SessionWire = serde_json::from_str(CREATE_SESSION_OK).unwrap();
        let session = Session::from_wire(wire);
        assert_eq!(session.did, "did:plc:zjbq26wybii5ojoypkso2mso");
        assert_eq!(session.handle, "jauntywunderkind.bsky.social");
        let dbg = format!("{session:?}");
        assert!(!dbg.contains("access-secret"), "{dbg}");
        assert!(!dbg.contains("refresh-secret"), "{dbg}");
        assert!(dbg.contains("jauntywunderkind.bsky.social"), "{dbg}");
    }

    #[test]
    fn list_records_page_decode() {
        let body = json!({
            "cursor": "3molsbwkh522o",
            "records": [
                {"uri": "at://did:plc:x/site.standard.publication/3molsbwkh522o",
                 "cid": "bafyreicid",
                 "value": {"$type": "site.standard.publication", "name": "Technautical", "url": "https://technautical.leaflet.pub"}},
                {"uri": "at://did:plc:x/site.standard.publication/skyreader-links",
                 "cid": "bafyreiother",
                 "value": {"$type": "site.standard.publication", "name": "links", "url": "https://linkblogs.skyreader.app/did:plc:x/", "skyreaderLinkblog": {}}}
            ]
        });
        let page: ListRecordsPage = serde_json::from_value(body).unwrap();
        assert_eq!(page.records.len(), 2);
        assert_eq!(page.cursor.as_deref(), Some("3molsbwkh522o"));
        assert_eq!(page.records[0].value["name"], json!("Technautical"));
        assert_eq!(page.records[1].cid, "bafyreiother");
    }

    #[test]
    fn list_records_page_without_cursor() {
        let page: ListRecordsPage = serde_json::from_value(json!({
            "records": [{"uri": "at://did:plc:x/pub.leaflet.publication/legacy1", "cid": "bafy", "value": {"$type": "pub.leaflet.publication", "name": "legacy"}}]
        }))
        .unwrap();
        assert!(page.cursor.is_none());
        assert_eq!(page.records.len(), 1);
    }

    #[test]
    fn get_record_not_found_mapping() {
        // The PDS reports record-not-found as a 400 XRPC error.
        let not_found = json!({"error": "RecordNotFound", "message": "Could not locate record at://did:plc:x/site.standard.document/missing"}).to_string();
        let value: Value = serde_json::from_str(&not_found).unwrap();
        let code = value.get("error").and_then(Value::as_str).unwrap();
        let message = value.get("message").and_then(Value::as_str).unwrap();
        let matches = code == "RecordNotFound" || code == "InvalidRequest" && message.contains("ould not locate");
        assert!(matches, "not-found shape must map to None");
    }

    #[test]
    fn client_debug_never_leaks_base_auth() {
        let client = XrpcClient::new("https://puffball.us-east.host.bsky.network").unwrap();
        let dbg = format!("{client:?}");
        assert!(dbg.contains("puffball"), "{dbg}");
    }

    #[test]
    fn client_rejects_bad_urls() {
        assert!(XrpcClient::new("not a url").is_err());
        // http is only tolerated for loopback testing.
        assert!(XrpcClient::new("http://example.com").is_err());
        assert!(XrpcClient::new("http://127.0.0.1:2583").is_ok());
        assert!(XrpcClient::new("https://pds.example/").unwrap().url_for("x") == *"https://pds.example/xrpc/x");
    }

    #[test]
    fn rkey_acceptance() {
        for ok in ["leafspring-protocol-probe", "3molsbwkh522o", "a.b_c~d:e-f", "x"] {
            crate::validate_rkey(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
        for bad in ["", "has space", "a/b", "a/b/c", "üñî"] {
            assert!(crate::validate_rkey(bad).is_err(), "{bad} should be rejected");
        }
    }

    // ── wire tests against the local test server ──────────────────────────

    use crate::testserver::TestServer;

    fn session() -> Session {
        Session {
            access_jwt: "test-access-jwt".into(),
            refresh_jwt: "test-refresh-jwt".into(),
            did: "did:plc:zjbq26wybii5ojoypkso2mso".into(),
            handle: "jauntywunderkind.bsky.social".into(),
        }
    }

    #[tokio::test]
    async fn put_record_roundtrip_on_the_wire() {
        let server = TestServer::spawn(Arc::new(|_req| {
            (200, r#"{"uri":"at://did:plc:x/site.standard.document/my-post","cid":"bafyreitest"}"#.into())
        }))
        .await;
        let client = XrpcClient::new(&server.base_url).unwrap();
        let record = serde_json::json!({"$type": "site.standard.document", "title": "T", "publishedAt": "2026-11-07T00:00:00Z", "site": "at://did:plc:x/site.standard.publication/3molsbwkh522o"});
        let out = client
            .put_record(&session(), "did:plc:x", "site.standard.document", "my-post", &record)
            .await
            .expect("put succeeds");
        assert_eq!(out.uri, "at://did:plc:x/site.standard.document/my-post");
        assert_eq!(out.cid, "bafyreitest");
        let requests = server.requests();
        assert_eq!(requests.len(), 1, "one connection, one request");
        let req = &requests[0];
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/xrpc/com.atproto.repo.putRecord");
        assert_eq!(req.header("authorization"), Some("Bearer test-access-jwt"));
        let body = req.body_json();
        assert_eq!(body["repo"], "did:plc:x");
        assert_eq!(body["collection"], "site.standard.document");
        assert_eq!(body["rkey"], "my-post");
        assert_eq!(body["record"], record, "record travels verbatim");
    }

    #[tokio::test]
    async fn put_record_maps_xrpc_errors_and_rejects_bad_rkeys() {
        let server = TestServer::spawn(Arc::new(|_req| {
            (400, r#"{"error":"InvalidSwapError","message":"commit was at another cid"}"#.into())
        }))
        .await;
        let client = XrpcClient::new(&server.base_url).unwrap();
        let err = client
            .put_record(&session(), "did:plc:x", "site.standard.document", "my-post", &serde_json::json!({}))
            .await
            .unwrap_err();
        match err {
            Error::Xrpc { method, code, message } => {
                assert_eq!(method, "com.atproto.repo.putRecord");
                assert_eq!(code, "InvalidSwapError");
                assert_eq!(message, "commit was at another cid");
            }
            other => panic!("expected Xrpc error, got {other:?}"),
        }
        // Bad rkey/collection never reach the wire.
        assert!(matches!(
            client
                .put_record(&session(), "did:plc:x", "site.standard.document", "a/b", &serde_json::json!({}))
                .await,
            Err(Error::BadInput(_))
        ));
        assert_eq!(server.requests().len(), 1, "only the errored request happened");
    }

    #[tokio::test]
    async fn upload_blob_sends_raw_body_with_content_type() {
        let server = TestServer::spawn(Arc::new(|_req| {
            (
                200,
                r#"{"blob":{"$type":"blob","ref":{"$link":"bafyreiblob"},"mimeType":"image/png","size":8}}"#.into(),
            )
        }))
        .await;
        let client = XrpcClient::new(&server.base_url).unwrap();
        let blob = client
            .upload_blob(&session(), vec![0x89, b'P', b'N', b'G', 0, 0, 0, 0], "image/png")
            .await
            .expect("upload succeeds");
        assert_eq!(blob["$type"], "blob");
        assert_eq!(blob["ref"]["$link"], "bafyreiblob");
        assert_eq!(blob["size"], 8);
        let requests = server.requests();
        assert_eq!(requests[0].path, "/xrpc/com.atproto.repo.uploadBlob");
        assert_eq!(requests[0].header("content-type"), Some("image/png"));
        assert_eq!(requests[0].header("authorization"), Some("Bearer test-access-jwt"));
        assert_eq!(requests[0].body, vec![0x89, b'P', b'N', b'G', 0, 0, 0, 0], "raw bytes, not a JSON wrapper");
        // Bad content types rejected before the wire.
        assert!(client.upload_blob(&session(), vec![0], "png").await.is_err());
        assert_eq!(server.requests().len(), 1);
    }

    #[tokio::test]
    async fn refresh_session_flows_through_the_wire() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_for_task = hits.clone();
        let server = TestServer::spawn(Arc::new(move |req| {
            if req.path == "/xrpc/com.atproto.server.refreshSession" {
                hits_for_task.fetch_add(1, Ordering::Relaxed);
                (
                    200,
                    r#"{"accessJwt":"fresh-access","refreshJwt":"fresh-refresh","did":"did:plc:x","handle":"h.example"}"#.into(),
                )
            } else {
                (404, r#"{"error":"NotFound"}"#.into())
            }
        }))
        .await;
        let client = XrpcClient::new(&server.base_url).unwrap();
        let session = client.refresh_session("old-refresh-jwt").await.expect("refresh succeeds");
        assert_eq!(session.access_jwt, "fresh-access");
        assert_eq!(session.did, "did:plc:x");
        assert_eq!(hits.load(Ordering::Relaxed), 1);
        let req = &server.requests()[0];
        assert_eq!(
            req.header("authorization"),
            Some("Bearer old-refresh-jwt"),
            "refresh JWT rides the Authorization header"
        );
    }
}
