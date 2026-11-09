//! Identity: handle/DID validation and resolution to a PDS endpoint.
//!
//! Chain (proven live by `experiments/03-post-builder/REPORT.md` §1):
//! DNS TXT `_atproto.<handle>` → HTTPS well-known
//! `https://<handle>/.well-known/atproto-did` → DID document
//! (`plc.directory` for `did:plc`, `/.well-known/did.json` for `did:web`)
//! → `#atproto_pds` service endpoint. Input may also be a full DID, which
//! skips the handle steps.

use std::sync::LazyLock;

use serde::Deserialize;

use crate::dns;
use crate::{Error, XrpcClient};

/// A syntactically valid handle: dotted labels of `[a-zA-Z0-9-]`, each 1–63
/// chars with no leading/trailing `-`, at least one dot, ≤253 chars total
/// (atproto handle syntax, practical subset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle(String);

/// A syntactically valid DID: `did:<method>:<id>` with lowercase-alnum
/// method and `[A-Za-z0-9._%-]` id (DID core syntax, practical subset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Did(String);

static HANDLE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$").expect("handle regex")
});

static DID_RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^did:[a-z0-9]+:[A-Za-z0-9._%-]+$").expect("did regex"));

impl Handle {
    pub fn parse(s: &str) -> Result<Self, Error> {
        let s = s.trim().trim_start_matches('@');
        if s.len() <= 253 && HANDLE_RE.is_match(s) {
            Ok(Handle(s.to_string()))
        } else {
            Err(Error::BadInput(format!("invalid handle {s:?}")))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Did {
    pub fn parse(s: &str) -> Result<Self, Error> {
        let s = s.trim();
        if s.len() <= 253 && DID_RE.is_match(s) {
            Ok(Did(s.to_string()))
        } else {
            Err(Error::BadInput(format!("invalid DID {s:?}")))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A resolved identity: DID plus the PDS that serves it.
#[derive(Debug, Clone)]
pub struct ResolvedIdentity {
    pub did: Did,
    pub pds_endpoint: String,
    /// When the input was a handle: whether the DID document's `alsoKnownAs`
    /// contains `at://<handle>` (bidirectional verification, experiment 03
    /// step 4). `None` when the input was already a DID.
    pub handle_binding_verified: Option<bool>,
}

/// Identity resolution over HTTP (+ DNS TXT for handles).
#[derive(Clone)]
pub struct Resolver {
    http: reqwest::Client,
    /// PLC directory base for `did:plc` documents.
    plc_directory: String,
}

impl std::fmt::Debug for Resolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolver").field("plc_directory", &self.plc_directory).finish_non_exhaustive()
    }
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent(concat!("leafbud/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("reqwest client builds"),
            plc_directory: "https://plc.directory".into(),
        }
    }

    /// Resolve a handle or a full DID to its PDS endpoint.
    pub async fn resolve(&self, input: &str) -> Result<ResolvedIdentity, Error> {
        if let Ok(did) = Did::parse(input) {
            return self.resolve_did(&did).await;
        }
        let handle = Handle::parse(input).map_err(|_| Error::BadInput(format!("not a handle or DID: {input:?}")))?;
        self.resolve_handle(&handle).await
    }

    /// Handle → DID (DNS TXT, well-known fallback) → DID document → PDS.
    pub async fn resolve_handle(&self, handle: &Handle) -> Result<ResolvedIdentity, Error> {
        let did = match self.handle_to_did(handle).await? {
            Some(did) => did,
            None => {
                return Err(Error::Identity {
                    input: handle.to_string(),
                    reason: "neither DNS TXT _atproto.{handle} nor the HTTPS well-known file yielded a DID".into(),
                });
            }
        };
        let (mut resolved, doc) = self.resolve_did_with_doc(&did).await?;
        resolved.handle_binding_verified = Some(doc.has_handle(handle));
        Ok(resolved)
    }

    /// DID → DID document → PDS endpoint.
    pub async fn resolve_did(&self, did: &Did) -> Result<ResolvedIdentity, Error> {
        Ok(self.resolve_did_with_doc(did).await?.0)
    }

    /// DID → (resolved identity, parsed document). The document powers the
    /// handle-binding verification when the input was a handle.
    async fn resolve_did_with_doc(&self, did: &Did) -> Result<(ResolvedIdentity, DidDocument), Error> {
        let doc = self.did_document(did).await?;
        let endpoint = doc.pds_endpoint().ok_or_else(|| Error::Identity {
            input: did.to_string(),
            reason: "no #atproto_pds (AtprotoPersonalDataServer) service in the DID document".into(),
        })?;
        Ok((
            ResolvedIdentity {
                did: did.clone(),
                pds_endpoint: endpoint,
                handle_binding_verified: None,
            },
            doc,
        ))
    }

    /// Handle → DID via DNS TXT, then the HTTPS well-known file. `Ok(None)`
    /// means both methods ran without yielding a DID.
    async fn handle_to_did(&self, handle: &Handle) -> Result<Option<Did>, Error> {
        if let Some(did) = dns::resolve_handle_txt(handle.as_str()).await? {
            tracing::debug!(handle = %handle, "resolved via DNS TXT");
            return Did::parse(&did).map(Some);
        }
        // Well-known: https://<handle>/.well-known/atproto-did contains
        // `did=...` (bsky.social handles resolve this way).
        let url = format!("https://{}/.well-known/atproto-did", handle.as_str());
        tracing::debug!(url = %url, "fetching well-known atproto-did");
        let response = self.http.get(&url).send().await?;
        let status = response.status();
        if !status.is_success() {
            tracing::debug!(url = %url, status = %status.as_u16(), "well-known fetch failed");
            return Ok(None);
        }
        let body = response.text().await?;
        // The well-known body is the DID, optionally `did=`-prefixed
        // (bsky.social serves the bare form; both are spec-legal).
        let did = body
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .map(|l| l.strip_prefix("did=").unwrap_or(l))
            .filter(|s| s.starts_with("did:"))
            .ok_or_else(|| Error::Identity {
                input: handle.to_string(),
                reason: format!("well-known response has no DID (HTTP {status})"),
            })?;
        Did::parse(did).map(Some)
    }

    /// Fetch and parse the DID document for a DID.
    async fn did_document(&self, did: &Did) -> Result<DidDocument, Error> {
        let s = did.as_str();
        let url = if s.starts_with("did:plc:") {
            format!("{}/{}", self.plc_directory, s)
        } else if let Some(rest) = s.strip_prefix("did:web:") {
            // Method id is a domain (a port is encoded as '%3A'; any path
            // after a first '/' joins as real path segments). The common
            // case is a plain domain.
            let (host, path) = match rest.split_once('/') {
                Some((h, p)) => (h, format!("/{p}")),
                None => (rest, String::new()),
            };
            let host = host.replace("%3A", ":");
            format!("https://{host}{path}/.well-known/did.json")
        } else {
            return Err(Error::Identity {
                input: s.to_string(),
                reason: "unsupported DID method (plc and web only; webvh needs a log verifier)".into(),
            });
        };
        tracing::debug!(url = %url, "fetching DID document");
        let response = self.http.get(&url).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Identity {
                input: s.to_string(),
                reason: format!("DID document fetch returned HTTP {status}"),
            });
        }
        let doc: DidDocument = serde_json::from_str(&response.text().await?).map_err(|e| Error::Identity {
            input: s.to_string(),
            reason: format!("DID document decode: {e}"),
        })?;
        if doc.id != s {
            return Err(Error::Identity {
                input: s.to_string(),
                reason: format!("DID document id {:?} does not match the requested DID", doc.id),
            });
        }
        Ok(doc)
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

/// The subset of a DID document leafbud needs.
#[derive(Debug, Clone, Deserialize)]
pub struct DidDocument {
    /// DID-core key: the subject, named `id` (not `did`).
    pub id: String,
    #[serde(default, rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    #[serde(default)]
    pub service: Vec<DidService>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DidService {
    pub id: String,
    #[serde(rename = "type")]
    pub service_type: String,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

impl DidDocument {
    /// The `#atproto_pds` service endpoint (`AtprotoPersonalDataServer`).
    pub fn pds_endpoint(&self) -> Option<String> {
        self.service
            .iter()
            .find(|s| s.id.ends_with("#atproto_pds") || s.service_type == "AtprotoPersonalDataServer")
            .map(|s| s.service_endpoint.trim_end_matches('/').to_string())
    }

    /// Whether `alsoKnownAs` contains `at://<handle>` (bidirectional
    /// verification of the handle→DID binding).
    pub fn has_handle(&self, handle: &Handle) -> bool {
        let want = format!("at://{}", handle.as_str());
        self.also_known_as.iter().any(|aka| aka.eq_ignore_ascii_case(&want))
    }
}

/// Convenience: resolve a handle/DID to an [`XrpcClient`] bound to its PDS.
pub async fn resolve_xrpc_client(resolver: &Resolver, input: &str) -> Result<(ResolvedIdentity, XrpcClient), Error> {
    let identity = resolver.resolve(input).await?;
    let client = XrpcClient::new(&identity.pds_endpoint)?;
    Ok((identity, client))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_acceptance() {
        for ok in [
            "jauntywunderkind.bsky.social",
            "example.com",
            "sub.domain.example.com",
            "a-b.c-d.example",
            "@handle.at", // leading @ trimmed like the web apps do
        ] {
            Handle::parse(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
        for bad in [
            "",
            "no-dots",
            "-leading.example",
            "trailing-.example",
            "a..b",
            "under_score.example",
            "a b.example",
            &format!("{}.example", "x".repeat(64)),
        ] {
            assert!(Handle::parse(bad).is_err(), "{bad} should be rejected");
        }
        assert_eq!(Handle::parse("@jauntywunderkind.bsky.social").unwrap().as_str(), "jauntywunderkind.bsky.social");
    }

    #[test]
    fn did_acceptance() {
        for ok in [
            "did:plc:zjbq26wybii5ojoypkso2mso",
            "did:web:example.com",
            "did:web:example.com%3A8443",
            "did:key:z6Mkabc",
        ] {
            Did::parse(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
        for bad in ["", "did:plc:", "did:plc", ":plc:x", "did:PLC:upper", "did:plc:has space"] {
            assert!(Did::parse(bad).is_err(), "{bad} should be rejected");
        }
    }

    /// The DID document shape from experiment 03 step 4.
    #[test]
    fn did_document_pds_extraction() {
        let doc: DidDocument = serde_json::from_str(
            r##"{
              "@context": ["https://www.w3.org/ns/did/v1"],
              "id": "did:plc:zjbq26wybii5ojoypkso2mso",
              "alsoKnownAs": ["at://jauntywunderkind.bsky.social"],
              "verificationMethod": [{"id": "did:plc:x#atproto", "type": "Multikey", "publicKeyMultibase": "zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme"}],
              "service": [
                {"id": "#chat", "type": "BskyChat", "serviceEndpoint": "https://chat.example"},
                {"id": "#atproto_pds", "type": "AtprotoPersonalDataServer",
                 "serviceEndpoint": "https://puffball.us-east.host.bsky.network/"}
              ]
            }"##,
        )
        .unwrap();
        assert_eq!(doc.id, "did:plc:zjbq26wybii5ojoypkso2mso");
        assert_eq!(doc.pds_endpoint().as_deref(), Some("https://puffball.us-east.host.bsky.network"));
        assert!(doc.has_handle(&Handle::parse("jauntywunderkind.bsky.social").unwrap()));
        assert!(!doc.has_handle(&Handle::parse("other.bsky.social").unwrap()));
    }

    #[test]
    fn did_document_without_pds_service() {
        let doc: DidDocument =
            serde_json::from_str(r##"{"id": "did:web:example.com", "service": [{"id": "#other", "type": "Other", "serviceEndpoint": "https://x"}]}"##).unwrap();
        assert!(doc.pds_endpoint().is_none());
    }

    #[test]
    fn well_known_body_parse_shapes() {
        // Production serves the bare DID; the spec also allows `did=`.
        let bare = "did:plc:zjbq26wybii5ojoypkso2mso\n";
        let prefixed = "did=did:plc:zjbq26wybii5ojoypkso2mso\r\n";
        for body in [bare, prefixed] {
            let did = body
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .map(|l| l.strip_prefix("did=").unwrap_or(l))
                .filter(|s| s.starts_with("did:"))
                .unwrap();
            assert_eq!(Did::parse(did).unwrap().as_str(), "did:plc:zjbq26wybii5ojoypkso2mso");
        }
    }
}
