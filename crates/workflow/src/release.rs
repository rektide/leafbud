//! Independent, read-only current-public observation. No receipt adoption,
//! auth store, native actions, release attempts or stored historical bodies.
use crate::{editor_state::Change, import};
use anyhow::{Context, Result, ensure};
use leafbud_atproto::{AtUri, Record, Resolver, XrpcClient};
use leafbud_document::{Document, RELEASE_PROFILE};
use leafbud_records::projection::{self, Projection};
use serde::Serialize;

#[derive(Clone, Debug, Default)]
pub struct PublicReader {
    resolver: Resolver,
    endpoint: Option<XrpcClient>,
}

impl PublicReader {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        Ok(match endpoint {
            Some(endpoint) => Self::with_endpoint(XrpcClient::new(endpoint)?),
            None => Self::default(),
        })
    }
    /// Explicit endpoint selection for callers/configured PDS reads. The repo
    /// remains the binding URI's actual DID, never a configured account/handle.
    pub fn with_endpoint(endpoint: XrpcClient) -> Self {
        Self {
            resolver: Resolver::new(),
            endpoint: Some(endpoint),
        }
    }

    pub async fn fetch(&self, uri: &str) -> std::result::Result<Record, FetchError> {
        leafbud_document::validate_document_uri(uri).map_err(|_| FetchError::Mismatch)?;
        let uri_parts = AtUri::parse(uri).map_err(|_| FetchError::Mismatch)?;
        let resolved;
        let client = if let Some(endpoint) = &self.endpoint {
            endpoint
        } else {
            let identity = self.resolver.resolve(&uri_parts.authority).await.map_err(|_| FetchError::Unavailable)?;
            resolved = XrpcClient::new(&identity.pds_endpoint).map_err(|_| FetchError::Unavailable)?;
            &resolved
        };
        let record = client
            .get_record(&uri_parts.authority, &uri_parts.collection, &uri_parts.rkey)
            .await
            .map_err(|error| match error {
                leafbud_atproto::Error::Xrpc { code, .. } if code == "RecordNotFound" => FetchError::Missing,
                _ => FetchError::Unavailable,
            })?
            .ok_or(FetchError::Missing)?;
        if record.uri != uri || record.value["$type"] != uri_parts.collection {
            return Err(FetchError::Mismatch);
        }
        if record.cid.as_deref().is_some_and(|cid| leafbud_document::validate_record_cid(cid).is_err()) {
            return Err(FetchError::Mismatch);
        }
        Ok(record)
    }

    /// Separate stronger acquisition for future publication settlement. Ordinary
    /// status/diff intentionally retain their reported/syntax-only evidence.
    /// Still NOT authorship, time, repo-signature, semantic/linkage or delivery proof.
    pub async fn fetch_content_verified(&self, uri: &str) -> std::result::Result<ContentVerifiedRecord, ContentFetchError> {
        let record = self.fetch(uri).await.map_err(ContentFetchError::Fetch)?;
        let cid = record.cid.as_deref().ok_or(ContentFetchError::MissingCid)?;
        let content = leafbud_atproto::content::verify_record_content(&record.value, cid).map_err(ContentFetchError::Content)?;
        Ok(ContentVerifiedRecord { record, content })
    }

    pub async fn comparison(&self, document: &Document) -> Result<(Projection, Projection)> {
        let binding = document.state.binding.as_ref().context("public binding missing")?;
        let uri = binding.uri.as_deref().context("document is not published")?;
        let record = self
            .fetch(uri)
            .await
            .map_err(|error| anyhow::anyhow!("public read {}: no comparison produced", error.as_str()))?;
        // URI/type/CID acquisition still verifies the exact selected record.
        // A changed supported site/publication field is useful diff content,
        // not permission to rebind. Status reports that linkage mismatch;
        // read-only comparison may show it without accepting either endpoint.
        Ok((projection::working(document)?, projection::from_record(&record.value, document)?))
    }
}

/// The record cannot be mutated independently of its content-address witness.
pub struct ContentVerifiedRecord {
    record: Record,
    content: leafbud_atproto::content::VerifiedContent,
}
impl ContentVerifiedRecord {
    pub fn record(&self) -> &Record {
        &self.record
    }
    pub fn content(&self) -> &leafbud_atproto::content::VerifiedContent {
        &self.content
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ContentFetchError {
    #[error("public acquisition failed: {0:?}")]
    Fetch(FetchError),
    #[error("public endpoint returned no record CID")]
    MissingCid,
    #[error("record content addressing: {0}")]
    Content(leafbud_atproto::content::Error),
}

#[derive(Debug)]
pub enum FetchError {
    Missing,
    Unavailable,
    Mismatch,
}
impl FetchError {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Missing => "record-missing",
            Self::Unavailable => "unavailable",
            Self::Mismatch => "identity-or-CID-mismatch",
        }
    }
}

fn verify_linkage(document: &Document, record: &Record) -> Result<()> {
    let binding = document.state.binding.as_ref().context("binding unavailable")?;
    let uri = AtUri::parse(&record.uri)?;
    if uri.collection == "pub.leaflet.document" {
        ensure!(
            record.value["author"] == uri.authority && record.value.get("publication").and_then(serde_json::Value::as_str) == binding.publication.as_deref(),
            "legacy public author/publication differs from established native binding"
        );
        return Ok(());
    }
    let site = binding
        .publication
        .clone()
        .unwrap_or_else(|| format!("https://leaflet.pub/p/{}", uri.authority));
    ensure!(record.value["site"] == site, "public site/publication differs from established native binding");
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct Evidence {
    pub uri: Option<String>,
    pub endpoint_scope: &'static str,
    pub profile: &'static str,
    pub state: &'static str,
    pub baseline: &'static str,
    pub input: Change,
    pub output: Change,
    pub freshness: &'static str,
    pub input_projection: &'static str,
    pub output_projection: &'static str,
    pub input_hash: Option<String>,
    pub observed_cid: Option<String>,
    pub observed_at: Option<String>,
    pub endpoints_equal: Option<bool>,
    pub cid_evidence: &'static str,
}

impl Default for Evidence {
    fn default() -> Self {
        Self {
            uri: None,
            endpoint_scope: "not-requested",
            profile: RELEASE_PROFILE,
            state: "unknown",
            baseline: "unknown",
            input: Change::Unknown,
            output: Change::Unknown,
            freshness: "not-requested",
            input_projection: "unknown",
            output_projection: "not-requested",
            input_hash: None,
            observed_cid: None,
            observed_at: None,
            endpoints_equal: None,
            cid_evidence: "not-requested",
        }
    }
}

impl Evidence {
    pub(crate) fn source_raced(&mut self) {
        self.state = "stale-local-source";
        self.freshness = "stale-local-source";
        self.baseline = "stale-source";
        self.input = Change::Unknown;
        self.output = Change::Unknown;
        self.input_hash = None;
        self.endpoints_equal = None;
    }
}

pub async fn observe(document: &Document, public: Option<&PublicReader>, offline: bool) -> Evidence {
    let mut evidence = Evidence::default();
    if document.state.state != "published" {
        evidence.state = "not-published";
        return evidence;
    }
    evidence.uri = document.state.binding.as_ref().and_then(|binding| binding.uri.clone());
    let baseline = document.state.released.as_ref();
    evidence.baseline = if baseline.is_some() { "confirmed-input-output" } else { "unknown-imported" };
    let input = projection::working(document);
    evidence.input_projection = if input.is_ok() { "supported" } else { "unsupported" };
    evidence.input_hash = input.as_ref().ok().map(|input| import::hash(&input.value));
    evidence.input = match (baseline, &evidence.input_hash) {
        (Some(baseline), Some(hash)) if &baseline.input_hash == hash => Change::Unchanged,
        (Some(_), Some(_)) => Change::Changed,
        _ => Change::Unknown,
    };
    let Some(public) = public else {
        evidence.freshness = if offline { "offline" } else { "not-requested" };
        evidence.state = if evidence.input == Change::Changed {
            "input-changed-output-unknown"
        } else {
            "unknown"
        };
        return evidence;
    };
    evidence.endpoint_scope = if public.endpoint.is_some() {
        "explicit-endpoint"
    } else {
        "bound-owner-DID-resolution"
    };
    let Some(uri) = document.state.binding.as_ref().and_then(|binding| binding.uri.as_deref()) else {
        evidence.state = "binding-unavailable";
        return evidence;
    };
    evidence.observed_at = Some(jiff::Timestamp::now().to_string());
    let record = match public.fetch(uri).await {
        Ok(record) => record,
        Err(error) => {
            evidence.state = error.as_str();
            if matches!(error, FetchError::Missing) && baseline.is_some() {
                evidence.output = Change::Changed;
            }
            evidence.freshness = if matches!(error, FetchError::Missing) {
                "checked-missing"
            } else {
                "unavailable"
            };
            return evidence;
        }
    };
    evidence.freshness = "current";
    evidence.cid_evidence = if record.cid.is_some() {
        "PDS-reported-CID; syntax-validated, not rehashed"
    } else {
        "PDS-returned-no-CID"
    };
    evidence.observed_cid = record.cid.clone();
    evidence.output = match (baseline, &record.cid) {
        (Some(baseline), Some(cid)) if &baseline.cid == cid => Change::Unchanged,
        (Some(_), Some(_)) => Change::Changed,
        _ => Change::Unknown,
    };
    if verify_linkage(document, &record).is_err() {
        evidence.state = "linkage-mismatch";
        evidence.output_projection = "mismatch";
        return evidence;
    }
    let output = projection::from_record(&record.value, document);
    evidence.output_projection = if output.is_ok() { "supported" } else { "unsupported" };
    evidence.endpoints_equal = match (input, output) {
        (Ok(input), Ok(output)) => Some(input.value == output.value),
        _ => None,
    };
    evidence.state = if evidence.endpoints_equal.is_none() {
        "unsupported-projection"
    } else if baseline.is_none() {
        if evidence.endpoints_equal == Some(true) {
            "equal-untracked"
        } else {
            "different-untracked"
        }
    } else if evidence.endpoints_equal == Some(true) && (evidence.input == Change::Changed || evidence.output == Change::Changed) {
        "converged-unaccepted"
    } else {
        match (evidence.input, evidence.output, evidence.endpoints_equal) {
            (Change::Unchanged, Change::Unchanged, Some(true)) => "clean",
            (Change::Changed, Change::Unchanged, _) => "input-changed",
            (Change::Unchanged, Change::Changed, _) => "output-changed",
            (Change::Changed, Change::Changed, _) => "both-changed",
            (Change::Unchanged, Change::Unchanged, Some(false)) => "inconsistent-evidence",
            _ => "unknown",
        }
    };
    evidence
}

pub(crate) fn view(projection: &Projection) -> Result<String> {
    let rendered = leafbud_mdx::render_comparison(&projection.document)?;
    // The existing renderer owns the exact body presentation. Its generated
    // header is replaced with the projection's complete public metadata, not
    // a second prose converter or a source-file editing operation.
    let (_, body) = rendered.split_once("\n---\n").context("generated comparison header missing")?;
    Ok(format!("---\n{}\n---\n{body}", serde_json::to_string_pretty(&projection.metadata)?))
}

#[cfg(test)]
mod tests;
