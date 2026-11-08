//! Read-only Leaflet web client: `/home` discovery and `get_leaflet_data`.
//!
//! Promotes the extraction protocol proven in experiments 01/02, not the old
//! sidecar lifecycle. No filesystem configuration, MDX, editor decoding or writes.
//! Raw per-Leaflet observations survive alongside the deliberately partial view.

use std::time::Duration;

use reqwest::{Url, header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

mod home;
mod pull;
pub use home::{DiscoveredLeaflet, Discovery, Identity, parse_home};
pub use pull::{PullSnapshot, parse_pull};

pub const DEFAULT_BASE_URL: &str = "https://leaflet.pub";

/// Diagnostics intentionally exclude response bodies, credentials and input URLs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed full editor pull: {0}")]
    Pull(&'static str),
    #[error("Leaflet transport failed")]
    Transport(#[source] reqwest::Error),
    #[error("{operation} returned HTTP {status}")]
    Http { operation: &'static str, status: u16 },
    #[error("base URL must be an http(s) origin without credentials, path, query or fragment")]
    BadBaseUrl,
    #[error("auth token is not a valid cookie value")]
    BadToken,
    #[error("document AT-URI resolution is not supported yet; use a native editor URL or permission-token UUID")]
    UnsupportedAtUri,
    #[error("locator must be a permission-token UUID or a same-origin native editor URL (/<uuid>)")]
    BadLocator,
    #[error("/home has no identity payload: anonymous/expired authentication or changed page shape")]
    NoHomePayload,
    #[error("malformed /home flight payload: {0}")]
    Home(&'static str),
    #[error("get_leaflet_data: token not found or not accessible to this session")]
    NotFound,
    #[error("get_leaflet_data RPC failed (result status {status:?}); response body withheld")]
    Rpc { status: Option<u16> },
    #[error("malformed get_leaflet_data response: {0}")]
    Response(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeafletState {
    Draft,
    Published,
}

impl LeafletState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
        }
    }
}

/// Only the fields used for inspection are typed. All source data, including
/// unknown rights/linkage fields and nested records, remain in `LeafletData::raw`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub id: Uuid,
    pub root_entity: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub permission_token_rights: Vec<Right>,
    pub leaflets_in_publications: Vec<Linkage>,
    pub leaflets_to_documents: Vec<StandaloneLink>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Right {
    pub token: Option<String>,
    pub entity_set: Option<String>,
    pub read: Option<bool>,
    pub write: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Linkage {
    pub doc: Option<String>,
    pub publication: Option<String>,
    pub title: Option<String>,
    pub scheduled_publish_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StandaloneLink {
    pub document: Option<String>,
}

impl Metadata {
    /// Publication association without a published `doc` is still a draft.
    pub fn state(&self) -> LeafletState {
        if self.leaflets_in_publications.iter().any(|link| link.doc.is_some()) || self.leaflets_to_documents.iter().any(|link| link.document.is_some()) {
            LeafletState::Published
        } else {
            LeafletState::Draft
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LeafletData {
    pub metadata: Metadata,
    pub raw: Value,
}

impl LeafletData {
    fn from_raw(raw: Value) -> Result<Self, Error> {
        let metadata = serde_json::from_value(raw.clone()).map_err(|_| Error::Response("invalid metadata/rights/linkage shape"))?;
        Ok(Self { metadata, raw })
    }

    /// Native metadata can be associated through sibling permission tokens.
    /// Inspect only the verified RPC's one-level entity-set token joins.
    pub fn association_rows(&self) -> Vec<&Value> {
        let mut rows = vec![&self.raw];
        for right in self.raw["permission_token_rights"].as_array().into_iter().flatten() {
            rows.extend(right["entity_sets"]["permission_tokens"].as_array().into_iter().flatten());
        }
        rows
    }

    pub fn observed_state(&self) -> LeafletState {
        if self.association_rows().iter().any(|row| {
            row["leaflets_in_publications"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|link| !link["doc"].is_null())
                || row["leaflets_to_documents"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|link| !link["document"].is_null())
        }) {
            LeafletState::Published
        } else {
            LeafletState::Draft
        }
    }
}

/// Redirects are disabled so an auth cookie cannot follow a server-directed
/// change of origin. Native editor URLs are resolved only against this origin.
#[derive(Clone)]
pub struct LeafletClient {
    http: reqwest::Client,
    base: Url,
}

impl std::fmt::Debug for LeafletClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeafletClient").field("base", &self.base).finish_non_exhaustive()
    }
}

impl LeafletClient {
    pub fn new(base_url: &str, auth_token: Option<&str>) -> Result<Self, Error> {
        let base = Url::parse(base_url.trim()).map_err(|_| Error::BadBaseUrl)?;
        if !matches!(base.scheme(), "https" | "http")
            || base.host_str().is_none()
            || !base.username().is_empty()
            || base.password().is_some()
            || base.path() != "/"
            || base.query().is_some()
            || base.fragment().is_some()
        {
            return Err(Error::BadBaseUrl);
        }
        let mut headers = header::HeaderMap::new();
        if let Some(token) = auth_token {
            // RFC 6265 cookie-octet: reject separators, whitespace and injection.
            if token.is_empty() || !token.bytes().all(|b| matches!(b, 0x21 | 0x23..=0x2b | 0x2d..=0x3a | 0x3c..=0x5b | 0x5d..=0x7e)) {
                return Err(Error::BadToken);
            }
            let mut cookie = header::HeaderValue::from_str(&format!("auth_token={token}")).map_err(|_| Error::BadToken)?;
            cookie.set_sensitive(true);
            headers.insert(header::COOKIE, cookie);
        }
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent(concat!("springleaf/", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(transport)?;
        Ok(Self { http, base })
    }

    pub fn base_url(&self) -> &str {
        self.base.as_str().trim_end_matches('/')
    }

    pub fn editor_url(&self, id: Uuid) -> String {
        format!("{}/{id}", self.base_url())
    }

    /// Query/fragment on a native editor URL do not select a different token.
    /// Foreign origins, userinfo and extra path segments are rejected, never fetched.
    pub fn resolve_locator(&self, locator: &str) -> Result<Uuid, Error> {
        if locator.starts_with("at://") {
            return Err(Error::UnsupportedAtUri);
        }
        if let Ok(id) = Uuid::parse_str(locator)
            && locator.len() == 36
        {
            return Ok(id);
        }
        let url = Url::parse(locator).map_err(|_| Error::BadLocator)?;
        if url.origin() != self.base.origin() || !url.username().is_empty() || url.password().is_some() {
            return Err(Error::BadLocator);
        }
        let path = url.path().strip_prefix('/').ok_or(Error::BadLocator)?;
        if path.len() != 36 {
            return Err(Error::BadLocator);
        }
        Uuid::parse_str(path).map_err(|_| Error::BadLocator)
    }

    pub async fn list(&self) -> Result<Discovery, Error> {
        let response = self.http.get(self.base.join("home").expect("static path")).send().await.map_err(transport)?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Http {
                operation: "GET /home",
                status: status.as_u16(),
            });
        }
        parse_home(&response.text().await.map_err(transport)?)
    }

    pub async fn get(&self, id: Uuid) -> Result<LeafletData, Error> {
        let response = self
            .http
            .post(self.base.join("api/rpc/get_leaflet_data").expect("static path"))
            .json(&serde_json::json!({"token_id": id}))
            .send()
            .await
            .map_err(transport)?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Http {
                operation: "POST get_leaflet_data",
                status: status.as_u16(),
            });
        }
        let body = response.text().await.map_err(transport)?;
        let data = parse_rpc(&body)?;
        if data.metadata.id != id {
            return Err(Error::Response("returned token differs from requested token"));
        }
        Ok(data)
    }
}

fn transport(error: reqwest::Error) -> Error {
    Error::Transport(error.without_url())
}

/// HTTP success alone is insufficient: PostgREST errors ride `result.error`.
pub fn parse_rpc(body: &str) -> Result<LeafletData, Error> {
    let value: Value = serde_json::from_str(body).map_err(|_| Error::Response("invalid JSON"))?;
    let result = value.get("result").and_then(Value::as_object).ok_or(Error::Response("missing result object"))?;
    let status = result.get("status").and_then(Value::as_u64).and_then(|v| u16::try_from(v).ok());
    let error = result.get("error").ok_or(Error::Response("missing result.error"))?;
    if !error.is_null() || status.is_some_and(|s| !(200..300).contains(&s)) {
        if error.get("code").and_then(Value::as_str) == Some("PGRST116") || status == Some(406) {
            return Err(Error::NotFound);
        }
        return Err(Error::Rpc { status });
    }
    let raw = result
        .get("data")
        .filter(|v| v.is_object())
        .ok_or(Error::Response("missing result.data object"))?
        .clone();
    LeafletData::from_raw(raw)
}

#[cfg(test)]
mod tests;

// Reuse the workspace's proven loopback responder; test-only, no runtime edge.
#[cfg(test)]
#[path = "../../../atproto/src/testserver.rs"]
mod testserver;
