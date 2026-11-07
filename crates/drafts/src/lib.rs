//! springleaf-drafts — the leaflet.pub web surface for springleaf.
//!
//! Cookie-authenticated reads of `/home` (Next.js RSC flight payload) and the
//! `/api/rpc/get_leaflet_data` enrichment RPC, implementing the extraction
//! algorithm and v1 output schema proven by
//! `experiments/01-draft-list/REPORT.md`:
//!
//! - concatenate the `self.__next_f.push([1,"…"])` chunk strings into the
//!   flight stream,
//! - locate `"permission_token_on_homepage":`, brace-match backwards to the
//!   enclosing identity object, decode it (raw-decode style),
//! - merge `contributor_leaflets` by token id,
//! - derive `published | in_publication | draft` from the
//!   `leaflets_in_publications[]` linkage (`doc != null` ⇒ published),
//! - cross-check the DOM `href="/<uuid>"` card set against the payload id set
//!   (`href_payload_mismatch` is an early alarm that the page shape changed),
//! - sort by `created_at` descending (row value, uuidv7-decode fallback,
//!   nulls last) — lexical UUID order is chronological only among v7s, and
//!   legacy v4 tokens must not be dropped or misplaced.
//!
//! The auth token is the `auth_token` cookie value supplied by the caller; it
//! is never logged, traced, or embedded in error messages.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod extract;
pub mod facts;
pub mod fractional;
pub mod pull;
pub mod yjs;

/// Default production base URL.
pub const DEFAULT_BASE_URL: &str = "https://leaflet.pub";

/// Canonical public host used for `lish` URL derivation (REPORT §2.2);
/// custom-domain publications override it via the publication record `url`.
const LISH_HOST: &str = "https://leaflet.pub";

static PUSH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)self\.__next_f\.push\((\[.*?\])\)</script>").expect("push regex"));

static UUID_HREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"href="/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})""#).expect("uuid href regex"));

static PUB_ATURI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^at://([^/]+)/site\.standard\.publication/([^/]+)$").expect("pub at-uri regex"));

static DOC_ATURI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^at://([^/]+)/site\.standard\.document/([^/]+)$").expect("doc at-uri regex"));

/// Route-ish first path segments that are never token ids. Kept for parity
/// with the experiment-01 prototype; a uuid-shaped href cannot actually match
/// any of these (REPORT §1.1).
const KNOWN_ROUTES: &[&str] = &[
    "reader",
    "home",
    "manual",
    "manifest.json",
    "favicon.ico",
    "apple-icon.png",
    "lish",
    "api",
    "_next",
    "icon",
    "createPub",
];

/// Errors. None of these ever embed the auth token value.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport error")]
    Transport(#[from] reqwest::Error),
    #[error("GET {url} returned HTTP {status}")]
    HttpStatus { url: String, status: u16 },
    #[error("invalid base url {0:?}")]
    BadBaseUrl(String),
    #[error("auth token is not valid as a cookie value")]
    BadToken,
    #[error("rpc {rpc} error {code}: {message}")]
    Rpc { rpc: String, code: String, message: String },
    #[error("unknown leaflet token id {0}")]
    UnknownDraft(String),
    #[error("no `permission_token_on_homepage` in the /home flight payload (page shape changed, or the request was anonymous: no valid auth token)")]
    NoHomePayload,
    #[error("flight payload parse failed: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("unexpected response from {rpc}: {detail}")]
    BadResponse { rpc: String, detail: String },
    #[error("structural error in the draft fact graph: {0}")]
    Structure(String),
}

// ---------------------------------------------------------------------------
// Wire shapes (deserialized subsets; unknown members are ignored)
// ---------------------------------------------------------------------------

/// A `/home` row of `permission_token_on_homepage[]` or
/// `contributor_leaflets[]` (REPORT §2.1).
#[derive(Debug, Deserialize)]
struct HomeRow {
    archived: Option<serde_json::Value>,
    created_at: Option<String>,
    permission_tokens: PermissionToken,
}

#[derive(Debug, Deserialize)]
struct PermissionToken {
    id: String,
    title: Option<String>,
    description: Option<String>,
    root_entity: Option<String>,
    #[serde(default)]
    permission_token_rights: Vec<PermissionRight>,
    #[serde(default)]
    leaflets_in_publications: Vec<Linkage>,
}

#[derive(Debug, Deserialize)]
struct PermissionRight {
    read: Option<bool>,
    write: Option<bool>,
    token: Option<String>,
}

/// One `leaflets_in_publications[]` entry. Identical shape in the home
/// payload and the `get_leaflet_data` RPC (REPORT §3).
#[derive(Debug, Clone, Deserialize)]
pub struct Linkage {
    pub doc: Option<String>,
    pub title: Option<String>,
    pub publication: Option<String>,
    pub documents: Option<LinkageDocument>,
    pub publications: Option<LinkagePublication>,
    pub scheduled_publish_at: Option<String>,
    pub scheduled_publish_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkageDocument {
    pub data: Option<DocumentRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentRecord {
    pub path: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkagePublication {
    pub record: Option<PublicationRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublicationRecord {
    pub name: Option<String>,
    pub url: Option<String>,
}

/// The identity object decoded from the flight stream (subset; REPORT §2).
#[derive(Debug, Deserialize)]
struct IdentityData {
    atp_did: Option<String>,
    #[serde(default)]
    bsky_profiles: Option<BskyProfile>,
    fetched_at: Option<i64>,
    #[serde(default)]
    permission_token_on_homepage: Vec<HomeRow>,
    #[serde(default)]
    contributor_leaflets: Vec<HomeRow>,
}

#[derive(Debug, Deserialize)]
struct BskyProfile {
    did: Option<String>,
    handle: Option<String>,
}

/// `get_leaflet_data` result row (REPORT §3). Members kept as raw JSON are
/// documented shapes this stage renders verbatim (`drafts show --json`) or
/// defers (stage 3+).
#[derive(Debug, Deserialize)]
pub struct LeafletData {
    pub id: Option<String>,
    pub root_entity: Option<String>,
    pub blocked_by_admin: Option<serde_json::Value>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub permission_token_rights: Vec<RpcRight>,
    #[serde(default)]
    pub custom_domain_routes: Vec<serde_json::Value>,
    #[serde(default)]
    pub leaflets_in_publications: Vec<Linkage>,
    #[serde(default)]
    pub leaflets_to_documents: Vec<serde_json::Value>,
    #[serde(default)]
    pub publications: Vec<serde_json::Value>,
    pub publication_scheduled_posts: Option<serde_json::Value>,
    pub reply_drafts: Option<serde_json::Value>,
}

/// A `permission_token_rights[]` row as returned by the RPC (adds
/// `entity_set`/`entity_sets` detail over the home shape).
#[derive(Debug, Deserialize)]
pub struct RpcRight {
    pub read: Option<bool>,
    pub write: Option<bool>,
    pub token: Option<String>,
    pub entity_set: Option<String>,
}

// ---------------------------------------------------------------------------
// Output schema v1 (experiments/01-draft-list/REPORT.md §5)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Identity {
    pub did: Option<String>,
    pub handle: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CreatedAtSource {
    #[serde(rename = "row")]
    Row,
    #[serde(rename = "uuidv7")]
    Uuidv7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LeafletState {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "in_publication")]
    InPublication,
    #[serde(rename = "published")]
    Published,
}

impl LeafletState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeafletState::Draft => "draft",
            LeafletState::InPublication => "in_publication",
            LeafletState::Published => "published",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rights {
    pub read: Option<bool>,
    pub write: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicationRef {
    pub uri: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Leaflet {
    pub token_id: String,
    pub created_at: Option<String>,
    pub created_at_source: Option<CreatedAtSource>,
    pub uuid_version: u8,
    pub title: Option<String>,
    pub description: Option<String>,
    pub root_entity: Option<String>,
    pub archived: bool,
    pub on_homepage: bool,
    pub contributor: bool,
    pub rights: Option<Rights>,
    pub state: LeafletState,
    pub publication: Option<PublicationRef>,
    pub published_doc_uri: Option<String>,
    pub published_at: Option<String>,
    pub published_url: Option<String>,
    /// Present only after `--fetch` enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by_admin: Option<serde_json::Value>,
    /// Present only after `--fetch` enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_drafts: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeafletList {
    pub version: u8,
    pub source: String,
    pub fetched_at: Option<i64>,
    pub identity: Identity,
    pub count: usize,
    pub href_payload_mismatch: Vec<String>,
    pub leaflets: Vec<Leaflet>,
}

// ---------------------------------------------------------------------------
// Flight-stream parsing (REPORT §2)
// ---------------------------------------------------------------------------

/// Concatenate the `self.__next_f.push([1,"…"])` chunk strings into the RSC
/// flight stream (plain JSON lines).
pub fn flight_stream(html: &str) -> Result<String, Error> {
    let mut out = String::new();
    for caps in PUSH_RE.captures_iter(html) {
        // caps[1] is the JSON array literal `[1,"…"]`.
        let arr: serde_json::Value = serde_json::from_str(&caps[1])?;
        match arr.get(1).and_then(|v| v.as_str()) {
            Some(fragment) => out.push_str(fragment),
            None => {
                return Err(Error::BadResponse {
                    rpc: "flight stream".into(),
                    detail: "a __next_f.push chunk is not [1,<string>]".into(),
                });
            }
        }
    }
    Ok(out)
}

/// Slice of the whole JSON object enclosing the first `"<key>":` occurrence in
/// `stream`, found by brace-matching backwards from the key and forwards from
/// the object opening (raw-decode style; REPORT §2). String literals are
/// skipped in both directions so braces inside values cannot miscount.
pub fn enclosing_object<'a>(stream: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\":");
    let key_pos = stream.find(&needle)?;
    let start = find_object_start(stream, key_pos)?;
    let end = find_object_end(stream, start)?;
    Some(&stream[start..=end])
}

/// Walk backwards from `from` to the `{` that opens the object enclosing
/// position `from`, skipping string literals.
fn find_object_start(stream: &str, from: usize) -> Option<usize> {
    let bytes = stream.as_bytes();
    let mut depth: i64 = 0;
    let mut j = from;
    while j > 0 {
        j -= 1;
        match bytes[j] {
            b'"' => {
                // Skip the entire string literal: find its opening quote,
                // honoring backslash escapes.
                loop {
                    if j == 0 {
                        break;
                    }
                    j -= 1;
                    if bytes[j] == b'"' {
                        let mut backslashes = 0;
                        let mut k = j;
                        while k > 0 && bytes[k - 1] == b'\\' {
                            backslashes += 1;
                            k -= 1;
                        }
                        if backslashes % 2 == 0 {
                            break; // unescaped quote: real string boundary
                        }
                    }
                }
            }
            b'}' | b']' => depth += 1,
            b'{' if depth == 0 => return Some(j),
            b'{' | b'[' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// Walk forwards from the `{` at `start` to its matching `}`, skipping string
/// literals and escapes.
fn find_object_end(stream: &str, start: usize) -> Option<usize> {
    let bytes = stream.as_bytes();
    let mut depth: i64 = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else {
                match b {
                    b'\\' => escaped = true,
                    b'"' => in_string = false,
                    _ => {}
                }
            }
        } else {
            match b {
                b'"' => in_string = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Decode the identity object from a flight stream; `None` when the payload
/// carries no `permission_token_on_homepage` key (anonymous surface).
fn identity_data(stream: &str) -> Result<Option<IdentityData>, Error> {
    match enclosing_object(stream, "permission_token_on_homepage") {
        None => Ok(None),
        Some(slice) => Ok(Some(serde_json::from_str(slice)?)),
    }
}

/// Identity visible in a `/home` response body: `None` when the flight
/// payload carries no identity object (logged-out / anonymous surface).
pub fn identity_from_home(html: &str) -> Result<Option<Identity>, Error> {
    let stream = flight_stream(html)?;
    Ok(identity_data(&stream)?.map(|ident| {
        let profile = ident.bsky_profiles.as_ref();
        Identity {
            did: profile.and_then(|p| p.did.clone()).or(ident.atp_did.clone()),
            handle: profile.and_then(|p| p.handle.clone()),
        }
    }))
}

// ---------------------------------------------------------------------------
// Draft-list extraction (REPORT §4–5)
// ---------------------------------------------------------------------------

/// Token ids from `href="/<uuid>"` DOM cards: uuid-shaped bare first path
/// segments only (REPORT §1.1). Contributor leaflets get no href cards.
pub fn href_token_ids(html: &str) -> Vec<String> {
    UUID_HREF_RE
        .captures_iter(html)
        .map(|caps| caps[1].to_string())
        .filter(|id| {
            let first_segment = id.split('-').next().unwrap_or("");
            !id.starts_with("lish") && !KNOWN_ROUTES.contains(&first_segment)
        })
        .collect()
}

/// Milliseconds encoded in a UUIDv7 id's leading 48 bits; `None` for other
/// versions — pre-v7 tokens carry no timestamp (REPORT §1.2).
pub fn uuid_v7_ms(id: &str) -> Option<i64> {
    let parsed = Uuid::parse_str(id).ok()?;
    if parsed.get_version_num() != 7 {
        return None;
    }
    Some((parsed.as_u128() >> 80) as i64)
}

/// Format decoded v7 milliseconds like the row timestamps
/// (`2026-11-02T10:15:00.999999+00:00`).
fn format_v7_ms(ms: i64) -> String {
    use jiff::tz::TimeZone;
    jiff::Timestamp::from_millisecond(ms)
        .map(|ts| ts.to_zoned(TimeZone::UTC).strftime("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string())
        .unwrap_or_else(|_| ms.to_string())
}

/// The classified publication outcome of one leaflet (REPORT §2.2).
#[derive(Debug, Clone)]
pub struct Classified {
    pub state: LeafletState,
    pub publication: Option<PublicationRef>,
    pub published_doc_uri: Option<String>,
    pub published_at: Option<String>,
    pub published_url: Option<String>,
}

/// `publishedAt` of a linkage's document record, `""` when absent — the
/// ordering key for picking the latest published linkage (REPORT §5).
fn linkage_published_at(l: &Linkage) -> &str {
    l.documents
        .as_ref()
        .and_then(|d| d.data.as_ref())
        .and_then(|d| d.published_at.as_deref())
        .unwrap_or("")
}

/// State derivation rule, verified against 30 real leaflets (REPORT §2.2):
/// any linkage item with `doc != null` ⇒ `published` (pick the latest by
/// `publishedAt`, first wins on ties); linkage items exist but all
/// `doc == null` ⇒ `in_publication`; no linkage items ⇒ `draft`.
pub fn classify(links: &[Linkage]) -> Classified {
    let published: Vec<&Linkage> = links.iter().filter(|l| l.doc.is_some()).collect();
    if !published.is_empty() {
        let published_at = linkage_published_at;
        let mut best = published[0];
        for l in published.iter().skip(1) {
            if published_at(l) > published_at(best) {
                best = l;
            }
        }
        let doc = best.documents.as_ref().and_then(|d| d.data.as_ref());
        let record = best.publications.as_ref().and_then(|p| p.record.as_ref());
        return Classified {
            state: LeafletState::Published,
            publication: Some(PublicationRef {
                uri: best.publication.clone(),
                name: record.and_then(|r| r.name.clone()),
                url: record.and_then(|r| r.url.clone()),
            }),
            published_doc_uri: best.doc.clone(),
            published_at: doc.and_then(|d| d.published_at.clone()),
            published_url: Some(derive_url(best)),
        };
    }
    if let Some(first) = links.first() {
        return Classified {
            state: LeafletState::InPublication,
            publication: Some(PublicationRef {
                uri: first.publication.clone(),
                name: None,
                url: None,
            }),
            published_doc_uri: None,
            published_at: None,
            published_url: None,
        };
    }
    Classified {
        state: LeafletState::Draft,
        publication: None,
        published_doc_uri: None,
        published_at: None,
        published_url: None,
    }
}

/// Published-URL derivation (REPORT §2.2, design slug rule): publication
/// record `url` (custom domain, trailing `/` trimmed) + doc record `path`;
/// else the `lish` form from the publication/doc at-uris; else empty string.
fn derive_url(link: &Linkage) -> String {
    let path = link
        .documents
        .as_ref()
        .and_then(|d| d.data.as_ref())
        .and_then(|d| d.path.as_deref())
        .unwrap_or("");
    let base = link
        .publications
        .as_ref()
        .and_then(|p| p.record.as_ref())
        .and_then(|r| r.url.as_deref())
        .unwrap_or("")
        .trim_end_matches('/');
    if !base.is_empty() && !path.is_empty() {
        return format!("{base}{path}");
    }
    let pub_caps = link.publication.as_deref().and_then(|uri| PUB_ATURI_RE.captures(uri));
    let doc_caps = link.doc.as_deref().and_then(|uri| DOC_ATURI_RE.captures(uri));
    if let (Some(p), Some(d)) = (pub_caps, doc_caps) {
        return format!("{LISH_HOST}/lish/{}/{}/{}", &p[1], &p[2], &d[2]);
    }
    String::new()
}

fn build_leaflet(row: &HomeRow, pt: &PermissionToken, contributor: bool) -> Leaflet {
    let ms = uuid_v7_ms(&pt.id);
    let rights = pt
        .permission_token_rights
        .iter()
        .find(|r| r.token.as_deref() == Some(pt.id.as_str()))
        .map(|r| Rights { read: r.read, write: r.write });
    let uuid_version = Uuid::parse_str(&pt.id).map(|u| u.get_version_num() as u8).unwrap_or(0);
    let classified = classify(&pt.leaflets_in_publications);
    Leaflet {
        token_id: pt.id.clone(),
        created_at: row.created_at.clone().or_else(|| ms.map(format_v7_ms)),
        created_at_source: if row.created_at.is_some() {
            Some(CreatedAtSource::Row)
        } else if ms.is_some() {
            Some(CreatedAtSource::Uuidv7)
        } else {
            None
        },
        // v7 ids are 7; legacy ids keep their version nibble (v4 observed).
        uuid_version: if ms.is_some() { 7 } else { uuid_version },
        title: pt.title.clone(),
        description: pt.description.clone(),
        root_entity: pt.root_entity.clone(),
        archived: row.archived.is_some(),
        on_homepage: !contributor,
        contributor,
        rights,
        state: classified.state,
        publication: classified.publication,
        published_doc_uri: classified.published_doc_uri,
        published_at: classified.published_at,
        published_url: classified.published_url,
        blocked_by_admin: None,
        reply_drafts: None,
    }
}

/// Full `drafts list` extraction from a `/home` response body (REPORT §4:
/// single-request RSC parse, chosen algorithm). `source` names the mode in
/// the output schema (`"home-rsc"`, or `"home-rsc+rpc"` after enrichment).
pub fn parse_home_leaflets(html: &str, source: &str) -> Result<LeafletList, Error> {
    let stream = flight_stream(html)?;
    let ident = identity_data(&stream)?.ok_or(Error::NoHomePayload)?;

    let mut leaflets: Vec<Leaflet> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for (rows, contributor) in [(&ident.permission_token_on_homepage, false), (&ident.contributor_leaflets, true)] {
        for row in rows {
            let leaflet = build_leaflet(row, &row.permission_tokens, contributor);
            match index.get(&leaflet.token_id) {
                // Owned + contributor rows can overlap by token id — merge
                // flags rather than emitting duplicates (REPORT §2.3).
                Some(&i) => leaflets[i].contributor = true,
                None => {
                    index.insert(leaflet.token_id.clone(), leaflets.len());
                    leaflets.push(leaflet);
                }
            }
        }
    }

    // UI order: created_at desc (row value; v7 lexical == chronological only
    // among v7 tokens — legacy v4 ids sort correctly only by row timestamp),
    // nulls last, token_id desc tiebreak.
    leaflets.sort_by(|a, b| {
        let key_a = (a.created_at.as_deref().unwrap_or("0000"), a.token_id.as_str());
        let key_b = (b.created_at.as_deref().unwrap_or("0000"), b.token_id.as_str());
        key_b.cmp(&key_a)
    });

    let home_ids: HashSet<&str> = leaflets.iter().filter(|l| l.on_homepage).map(|l| l.token_id.as_str()).collect();
    let hrefs = href_token_ids(html);
    let href_ids: HashSet<&str> = hrefs.iter().map(|s| s.as_str()).collect();
    let mut href_payload_mismatch: Vec<String> = href_ids.symmetric_difference(&home_ids).map(|s| s.to_string()).collect();
    href_payload_mismatch.sort();

    let profile = ident.bsky_profiles.as_ref();
    let identity = Identity {
        did: profile.and_then(|p| p.did.clone()).or(ident.atp_did.clone()),
        handle: profile.and_then(|p| p.handle.clone()),
    };
    let count = leaflets.len();
    Ok(LeafletList {
        version: 1,
        source: source.to_string(),
        fetched_at: ident.fetched_at,
        identity,
        count,
        href_payload_mismatch,
        leaflets,
    })
}

// ---------------------------------------------------------------------------
// HTTP client
// ---------------------------------------------------------------------------

/// Cookie-authenticated client for the leaflet.pub web surface.
///
/// The auth token becomes the `auth_token` request cookie; it is never
/// logged, traced, or included in error text. `Debug` deliberately prints
/// only the base URL.
#[derive(Clone)]
pub struct LeafletClient {
    http: reqwest::Client,
    base_url: String,
}

impl std::fmt::Debug for LeafletClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never derive: the underlying client config would surface header
        // values, including the auth cookie.
        f.debug_struct("LeafletClient").field("base_url", &self.base_url).finish_non_exhaustive()
    }
}

impl LeafletClient {
    /// Build a client for `base_url` (default
    /// [`DEFAULT_BASE_URL`]). `auth_token` is the `auth_token` cookie value.
    pub fn new(base_url: &str, auth_token: Option<String>) -> Result<Self, Error> {
        let mut base = base_url.trim().trim_end_matches('/').to_string();
        if base.is_empty() {
            base = DEFAULT_BASE_URL.to_string();
        }
        // Reject urls reqwest would later reject in a less legible way.
        reqwest::Url::parse(&base).map_err(|_| Error::BadBaseUrl(base.clone()))?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static(concat!("springleaf/", env!("CARGO_PKG_VERSION"))),
        );
        if let Some(token) = auth_token.filter(|t| !t.is_empty()) {
            let cookie = format!("auth_token={token}");
            let value = reqwest::header::HeaderValue::from_str(&cookie).map_err(|_| Error::BadToken)?;
            headers.insert(reqwest::header::COOKIE, value);
        }
        let http = reqwest::Client::builder().default_headers(headers).build()?;
        Ok(Self { http, base_url: base })
    }

    fn url_for(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path)
    }

    /// Raw `GET /home`: the HTML document including the RSC flight payload.
    pub async fn get_home(&self) -> Result<String, Error> {
        let url = self.url_for("home");
        // Only the URL is logged; never headers (the cookie lives there).
        tracing::debug!(url = %url, "GET /home");
        let response = self.http.get(&url).send().await?;
        let status = response.status();
        tracing::debug!(status = %status, "GET /home done");
        if !status.is_success() {
            return Err(Error::HttpStatus { url, status: status.as_u16() });
        }
        Ok(response.text().await?)
    }

    /// `drafts list` in one request: fetch `/home`, parse the flight payload.
    pub async fn list_leaflets(&self) -> Result<LeafletList, Error> {
        let html = self.get_home().await?;
        parse_home_leaflets(&html, "home-rsc")
    }

    /// `--fetch` enrichment: per-id `get_leaflet_data` (N+1 requests),
    /// merging `blocked_by_admin`/`reply_drafts` and marking the source
    /// `home-rsc+rpc` (REPORT §4).
    pub async fn enrich_leaflets(&self, list: &mut LeafletList) -> Result<(), Error> {
        for leaflet in &mut list.leaflets {
            let data = self.get_leaflet_data(&leaflet.token_id).await?;
            leaflet.blocked_by_admin = data.blocked_by_admin;
            leaflet.reply_drafts = data.reply_drafts;
        }
        list.source = "home-rsc+rpc".to_string();
        Ok(())
    }

    /// `get_leaflet_data` as raw JSON (verbatim `result.data`).
    pub async fn get_leaflet_data_raw(&self, token_id: &str) -> Result<serde_json::Value, Error> {
        let url = self.url_for("api/rpc/get_leaflet_data");
        tracing::debug!(url = %url, token_id = %token_id, "POST get_leaflet_data");
        let response = self.http.post(&url).json(&serde_json::json!({ "token_id": token_id })).send().await?;
        let status = response.status();
        let body = response.text().await?;
        parse_rpc_data("get_leaflet_data", status.as_u16(), &body).map_err(|err| match err {
            // The envelope's error body need not echo the id; we know it.
            Error::UnknownDraft(_) => Error::UnknownDraft(token_id.to_string()),
            other => other,
        })
    }

    /// `get_leaflet_data` typed for rendering.
    pub async fn get_leaflet_data(&self, token_id: &str) -> Result<LeafletData, Error> {
        let raw = self.get_leaflet_data_raw(token_id).await?;
        serde_json::from_value(raw).map_err(Error::Parse)
    }
}

/// Envelope handling shared by RPC callers: errors ride HTTP 200 as
/// `result.error` with a PostgREST code — `PGRST116` / `result.status` 406
/// is the "unknown token id" signal (REPORT §3). Returns `result.data`.
pub fn parse_rpc_data(rpc: &str, http_status: u16, body: &str) -> Result<serde_json::Value, Error> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|e| Error::BadResponse {
        rpc: rpc.into(),
        detail: format!("invalid JSON (HTTP {http_status}): {e}"),
    })?;
    let result = value.get("result").ok_or_else(|| Error::BadResponse {
        rpc: rpc.into(),
        detail: "no `result` envelope".into(),
    })?;
    if let Some(error) = result.get("error").filter(|e| !e.is_null()) {
        let code = error.get("code").and_then(|c| c.as_str()).unwrap_or("UNKNOWN");
        let message = error.get("message").and_then(|m| m.as_str()).unwrap_or_default();
        let result_status = result.get("status").and_then(|s| s.as_i64()).unwrap_or(0);
        if code == "PGRST116" || result_status == 406 {
            let token_id = extract_token_id_from_body(body);
            return Err(Error::UnknownDraft(token_id));
        }
        return Err(Error::Rpc {
            rpc: rpc.into(),
            code: code.into(),
            message: message.into(),
        });
    }
    match result.get("data") {
        Some(data) if !data.is_null() => Ok(data.clone()),
        _ => Err(Error::BadResponse {
            rpc: rpc.into(),
            detail: "`result.data` missing or null".into(),
        }),
    }
}

/// Best-effort recovery of the requested token id from a request-error body,
/// for a better UnknownDraft message; never secrets.
fn extract_token_id_from_body(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            v.get("hint")
                .or_else(|| v.get("details"))
                .and_then(|h| h.as_str())
                .and_then(|h| h.rsplit(' ').find(|w| Uuid::parse_str(w).is_ok()))
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Tests (synthetic fixtures only; no network — experiments/README.md rules)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const FIXTURE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../experiments/01-draft-list/fixture-home.html"));

    #[test]
    fn golden_fixture_list() {
        let list = parse_home_leaflets(FIXTURE, "home-rsc").expect("fixture parses");

        assert_eq!(list.version, 1);
        assert_eq!(list.source, "home-rsc");
        assert_eq!(list.fetched_at, Some(1_794_009_762_780));
        assert_eq!(list.identity.did.as_deref(), Some("did:plc:aaaaaaaaaaaaaaaaaaaaaaaa"));
        assert_eq!(list.identity.handle.as_deref(), Some("example.bsky.social"));
        assert_eq!(list.count, 5);
        assert_eq!(list.count, list.leaflets.len());
        assert!(list.href_payload_mismatch.is_empty(), "no href/payload drift");

        // created_at desc, legacy v4 (oldest row) last — REPORT §1.2.
        let ids: Vec<&str> = list.leaflets.iter().map(|l| l.token_id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "01a19bc1-50a0-7bbb-8b00-02bbbb000000", // 2026-11-02 draft, long title
                "01a16224-5e28-7ddd-8d00-04dddd000000", // 2026-10-22 contributor-only
                "01a0d0b8-89f0-7aaa-8a00-01aaaa000000", // 2026-09-24 published
                "019efcd9-d5b8-7ccc-8c00-03cccc000000", // 2026-06-25 draft
                "12345678-1234-4abc-9def-123456789abc", // 2025-11-24 legacy v4
            ]
        );

        let newest = &list.leaflets[0];
        assert_eq!(newest.state, LeafletState::Draft);
        assert_eq!(
            newest.title.as_deref(),
            Some("A synthetic draft title long enough to demonstrate human-output truncation in the table view")
        );
        assert_eq!(newest.description.as_deref(), Some("description field demo"));
        assert_eq!(newest.created_at_source, Some(CreatedAtSource::Row));
        assert_eq!(newest.uuid_version, 7);
        assert!(!newest.contributor);
        assert!(newest.on_homepage);
        assert_eq!(
            newest.rights,
            Some(Rights {
                read: Some(true),
                write: Some(true)
            })
        );

        // Contributor merge: separate array, no href card, flagged.
        let contributor = &list.leaflets[1];
        assert!(contributor.contributor);
        assert!(!contributor.on_homepage);
        assert_eq!(contributor.state, LeafletState::Published);
        assert_eq!(
            contributor.publication.as_ref().and_then(|p| p.name.as_deref()),
            Some("Example Synthetic Publication")
        );
        assert_eq!(contributor.published_url.as_deref(), Some("https://example-pub.leaflet.pub/3syndoctid002"));
        assert_eq!(contributor.published_at.as_deref(), Some("2026-09-24T00:02:30.040Z"));

        let published = &list.leaflets[2];
        assert_eq!(published.state, LeafletState::Published);
        assert_eq!(
            published.published_doc_uri.as_deref(),
            Some("at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3syndoctid002")
        );
        assert_eq!(
            published.publication.as_ref().and_then(|p| p.url.as_deref()),
            Some("https://example-pub.leaflet.pub")
        );

        let legacy = &list.leaflets[4];
        assert_eq!(legacy.uuid_version, 4);
        assert_eq!(legacy.created_at_source, Some(CreatedAtSource::Row));
        assert_eq!(legacy.created_at.as_deref(), Some("2025-11-24T23:46:56.126718+00:00"));
        assert_eq!(legacy.title.as_deref(), Some("Untitled"));
    }

    #[test]
    fn golden_fixture_json_shape() {
        let list = parse_home_leaflets(FIXTURE, "home-rsc").unwrap();
        let value = serde_json::to_value(&list).unwrap();
        let draft = &value["leaflets"][0];
        // Schema v1 field set, exactly once, per REPORT §5.
        let keys: Vec<&str> = draft.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(
            keys,
            vec![
                "token_id",
                "created_at",
                "created_at_source",
                "uuid_version",
                "title",
                "description",
                "root_entity",
                "archived",
                "on_homepage",
                "contributor",
                "rights",
                "state",
                "publication",
                "published_doc_uri",
                "published_at",
                "published_url",
            ]
        );
        assert_eq!(value["version"], json!(1));
        assert_eq!(value["identity"]["handle"], json!("example.bsky.social"));
        assert_eq!(value["leaflets"][3]["publication"], json!(null));
        // Enrichment fields absent until --fetch.
        assert!(value["leaflets"][0].get("blocked_by_admin").is_none());
    }

    /// Synthetic mini-home: a payload id with no href card and an href id
    /// with no payload row both land in href_payload_mismatch.
    #[test]
    fn mismatch_alarm_path() {
        let payload_id = "01a19bc1-50a0-7bbb-8b00-02bbbb000000"; // in payload, no href
        let href_only = "019efcd9-d5b8-7ccc-8c00-03cccc000000"; // in hrefs, no payload row
        let html = format!(
            r#"<a href="/{href_only}"></a><a href="/reader"></a>
            <script>self.__next_f.push([1, "{}"])</script>"#,
            serde_json::to_string(&json!({
                "atp_did": "did:plc:aaaaaaaaaaaaaaaaaaaaaaaa",
                "fetched_at": 1794009762780i64,
                "permission_token_on_homepage": [
                    {"archived": null, "created_at": "2026-11-02T10:15:00.999999+00:00",
                     "permission_tokens": {"id": payload_id, "title": "x",
                         "permission_token_rights": [], "leaflets_in_publications": []}}
                ],
                "contributor_leaflets": []
            }))
            .unwrap()
            .replace('"', "\\\"")
        );
        let list = parse_home_leaflets(&html, "home-rsc").unwrap();
        assert_eq!(list.href_payload_mismatch, vec![href_only.to_string(), payload_id.to_string()]);
    }

    #[test]
    fn no_payload_key_is_anonymous() {
        let html = r#"<script>self.__next_f.push([1, "0:{\"landing\":true}"])</script>"#;
        assert!(matches!(parse_home_leaflets(html, "home-rsc"), Err(Error::NoHomePayload)));
        assert!(identity_from_home(html).unwrap().is_none());
    }

    #[test]
    fn flight_stream_concatenates_chunks_in_order() {
        let html = r#"
        <script>self.__next_f.push([1, "0:{\"a\":"])</script>
        <script>self.__next_f.push([1, "1}"])</script>
        <script>self.__next_f=[]</script>
        <script>self.__next_f.push([1, "2:{\"b\":true}"])</script>
        "#;
        assert_eq!(flight_stream(html).unwrap(), "0:{\"a\":1}2:{\"b\":true}");
    }

    #[test]
    fn enclosing_object_walks_past_braces_inside_strings() {
        // Braces/brackets and escaped quotes inside string values between the
        // key and the true enclosing object must not affect the backward
        // match. The enclosing object of the key is the inner wrapper.
        let stream = r#"0:{"noise":"}{[}","x":{"k":"v\""}},"wrapper":{"permission_token_on_homepage":[{"a":1}]}}"#;
        let slice = enclosing_object(stream, "permission_token_on_homepage").expect("object found");
        let parsed: serde_json::Value = serde_json::from_str(slice).unwrap();
        assert_eq!(parsed["permission_token_on_homepage"][0]["a"], json!(1));
        // The slice is exactly the wrapper object, not a prefix of it.
        assert_eq!(slice, r#"{"permission_token_on_homepage":[{"a":1}]}"#);
    }

    #[test]
    fn enclosing_object_escapes_and_nesting() {
        let inner = r#"{"s":"quote\" and brace }{ [ ] and backslash \\","n":{"deep":[1,2,{"x":"}"}]}}"#;
        let stream = format!(r#"0:{{"pad":[1,2],"permission_token_on_homepage":{inner}}}"#);
        let slice = enclosing_object(&stream, "permission_token_on_homepage").unwrap();
        // The enclosing object carries the sibling members too.
        assert_eq!(slice, format!(r#"{{"pad":[1,2],"permission_token_on_homepage":{inner}}}"#));
        let parsed: serde_json::Value = serde_json::from_str(slice).unwrap();
        assert_eq!(parsed["permission_token_on_homepage"]["n"]["deep"][2]["x"], json!("}"));
    }

    #[test]
    fn enclosing_object_missing_key_and_key_inside_array() {
        assert!(enclosing_object("{}", "permission_token_on_homepage").is_none());
        // Key nested inside an array element: enclosing object is the element.
        let stream = r#"{"rows":[{"k":0,"t":{"permission_token_on_homepage":[]}}]}"#;
        let slice = enclosing_object(stream, "permission_token_on_homepage").unwrap();
        assert_eq!(slice, r#"{"permission_token_on_homepage":[]}"#);
    }

    #[test]
    fn uuid_v7_decode_matches_independent_hex_read() {
        for (id, expect_v7) in [("01a19bc1-50a0-7bbb-8b00-02bbbb000000", true), ("12345678-1234-4abc-9def-123456789abc", false)] {
            let uuid = Uuid::parse_str(id).unwrap();
            assert_eq!(uuid.get_version_num(), if expect_v7 { 7 } else { 4 });
            let hex_ms = u128::from_str_radix(&id.replace('-', "")[..12], 16).unwrap() as i64;
            match (uuid_v7_ms(id), expect_v7) {
                (Some(ms), true) => assert_eq!(ms, hex_ms),
                (None, false) => {}
                other => panic!("unexpected v7 decode for {id}: {other:?}"),
            }
        }
        // Fixture ids are minted from their displayed dates (REPORT §7):
        // the newest draft decodes to 2026-11-02T10:15:00.xxx.
        let formatted = format_v7_ms(uuid_v7_ms("01a19bc1-50a0-7bbb-8b00-02bbbb000000").unwrap());
        assert!(formatted.starts_with("2026-11-02T10:15:00."), "{formatted}");
        assert!(formatted.ends_with("+00:00"), "{formatted}");
    }

    #[test]
    fn row_without_created_at_falls_back_to_uuidv7() {
        let html = format!(
            r#"<script>self.__next_f.push([1, "{}"])</script>"#,
            serde_json::to_string(&json!({
                "permission_token_on_homepage": [
                    {"archived": null, "created_at": null,
                     "permission_tokens": {"id": "01a19bc1-50a0-7bbb-8b00-02bbbb000000",
                         "permission_token_rights": [], "leaflets_in_publications": []}}
                ],
                "contributor_leaflets": []
            }))
            .unwrap()
            .replace('"', "\\\"")
        );
        let list = parse_home_leaflets(&html, "home-rsc").unwrap();
        let leaflet = &list.leaflets[0];
        assert_eq!(leaflet.created_at_source, Some(CreatedAtSource::Uuidv7));
        let created = leaflet.created_at.as_deref().unwrap();
        assert!(created.starts_with("2026-11-02T10:15:00."), "{created}");
    }

    #[test]
    fn in_publication_rule_and_lish_url_fallback() {
        // Linkage exists, doc null ⇒ in_publication. And a published linkage
        // without a publication-record url derives the lish URL from at-uris.
        let links = vec![
            serde_json::from_value(json!({
                "doc": null,
                "publication": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/3synpubtid001"
            }))
            .unwrap(),
            serde_json::from_value(json!({
                "doc": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3syndoctid002",
                "publication": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/3synpubtid001",
                "documents": {"data": {"path": "/3syndoctid002",
                    "publishedAt": "2026-09-24T00:02:30.040Z"}},
                "publications": {"record": {"name": null, "url": null}}
            }))
            .unwrap(),
        ];
        let classified = classify(&links);
        assert_eq!(classified.state, LeafletState::Published);
        assert_eq!(
            classified.published_url.as_deref(),
            Some("https://leaflet.pub/lish/did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/3synpubtid001/3syndoctid002")
        );
        // Multiple published linkages: latest publishedAt wins.
        let mut older = links.clone();
        older.push(
            serde_json::from_value(json!({
                "doc": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/olderdoc1",
                "publication": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/3synpubtid001",
                "documents": {"data": {"path": "/olderdoc1", "publishedAt": "2025-01-01T00:00:00.000Z"}}
            }))
            .unwrap(),
        );
        assert_eq!(
            classify(&older).published_doc_uri.as_deref(),
            Some("at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3syndoctid002")
        );
        // Only null-doc linkage ⇒ in_publication with the publication uri kept.
        let null_only: Vec<Linkage> = vec![
            serde_json::from_value(json!({
                "doc": null,
                "publication": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/3synpubtid001"
            }))
            .unwrap(),
        ];
        let classified = classify(&null_only);
        assert_eq!(classified.state, LeafletState::InPublication);
        assert_eq!(
            classified.publication.as_ref().and_then(|p| p.uri.as_deref()),
            Some("at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/3synpubtid001")
        );
        assert!(classified.published_at.is_none());
    }

    #[test]
    fn href_extraction_ignores_routes_and_assets() {
        let html = r#"
        <a href="/01a19bc1-50a0-7bbb-8b00-02bbbb000000"></a>
        <a href="/reader"></a><a href="/home"></a>
        <a href="/lish/createPub"></a>
        <a href="/lish/did:plc:x/site.standard.publication/3p/dashboard"></a>
        <a href="/_next/static/chunks/x.js"></a>
        <link rel="icon" href="/favicon.ico?x"/>
        <a href="/12345678-1234-4abc-9def-123456789abc"></a>
        <a href="/019efcd9-d5b8-7ccc-8c00-03cccc000000/nested/route"></a>
        "#;
        assert_eq!(
            href_token_ids(html),
            vec![
                "01a19bc1-50a0-7bbb-8b00-02bbbb000000".to_string(),
                "12345678-1234-4abc-9def-123456789abc".to_string(),
            ]
        );
    }

    #[test]
    fn rpc_envelope_error_paths() {
        // Errors ride HTTP 200 (REPORT §3): PGRST116 / status 406 ⇒ unknown.
        let unknown = json!({
            "result": {"error": {"code": "PGRST116", "message": "no rows"},
                       "data": null, "status": 406, "statusText": ""}
        })
        .to_string();
        assert!(matches!(parse_rpc_data("get_leaflet_data", 200, &unknown), Err(Error::UnknownDraft(_))));
        // hint text carries the id in real bodies
        let unknown_hinted = json!({
            "result": {"error": {"code": "PGRST116",
                       "message": "no rows", "hint": "results contain 0 rows"},
                       "data": null, "status": 406},
            "hint": "token 01a19bc1-50a0-7bbb-8b00-02bbbb000000"
        })
        .to_string();
        assert_eq!(
            parse_rpc_data("get_leaflet_data", 200, &unknown_hinted).unwrap_err().to_string(),
            "unknown leaflet token id 01a19bc1-50a0-7bbb-8b00-02bbbb000000"
        );
        let other = json!({
            "result": {"error": {"code": "PGRST301", "message": "boom"},
                       "data": null, "status": 500, "statusText": ""}
        })
        .to_string();
        assert!(matches!(
            parse_rpc_data("get_leaflet_data", 200, &other),
            Err(Error::Rpc { code, .. }) if code == "PGRST301"
        ));
        let ok = json!({"result": {"error": null, "data": {"id": "x"}, "status": 200}}).to_string();
        assert_eq!(parse_rpc_data("get_leaflet_data", 200, &ok).unwrap(), json!({"id": "x"}));
        let missing = json!({"result": {"error": null, "data": null}}).to_string();
        assert!(matches!(parse_rpc_data("get_leaflet_data", 200, &missing), Err(Error::BadResponse { .. })));
    }

    macro_rules! exp02_fixture {
        ($file:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../experiments/02-draft-internals/fixtures/",
                $file
            ))
        };
    }
    #[test]
    fn leaflet_data_parses_experiment02_fixture_shapes() {
        // Read-only cross-check against experiment 02's committed synthetic
        // get_leaflet_data fixtures (pub-linked + pure draft).
        for raw in [
            exp02_fixture!("get_leaflet_data-pub-linked.json"),
            exp02_fixture!("get_leaflet_data-pure-draft.json"),
        ] {
            let envelope: serde_json::Value = serde_json::from_str(raw).unwrap();
            let data = envelope["result"]["data"].clone();
            let typed: LeafletData = serde_json::from_value(data).unwrap();
            assert!(typed.id.is_some());
            assert!(!typed.permission_token_rights.is_empty());
        }
        // classify over the pub-linked shape: doc present ⇒ published.
        let envelope: serde_json::Value = serde_json::from_str(exp02_fixture!("get_leaflet_data-pub-linked.json")).unwrap();
        let typed: LeafletData = serde_json::from_value(envelope["result"]["data"].clone()).unwrap();
        let classified = classify(&typed.leaflets_in_publications);
        assert_eq!(classified.state, LeafletState::Published);
        assert_eq!(
            classified.published_doc_uri.as_deref(),
            Some("at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3kz7exampledoc")
        );
        // Legacy pub.leaflet.publication at-uri ⇒ empty derived url (v1 rule
        // only knows site.standard.*; noted as a stage-3 question).
        assert_eq!(classified.published_url.as_deref(), Some(""));
    }

    #[test]
    fn client_debug_never_leaks_headers() {
        let client = LeafletClient::new("https://example.invalid", Some("secret-token-value".into())).unwrap();
        let dbg = format!("{client:?}");
        assert!(!dbg.contains("secret-token-value"), "{dbg}");
        assert!(dbg.contains("example.invalid"));
    }

    #[test]
    fn client_rejects_bad_inputs() {
        assert!(matches!(LeafletClient::new("not a url", None).unwrap_err(), Error::BadBaseUrl(_)));
        assert!(matches!(
            LeafletClient::new("https://x.invalid", Some("bad\ncookie".into())).unwrap_err(),
            Error::BadToken
        ));
        // Empty base falls back to production default.
        let client = LeafletClient::new("", None).unwrap();
        assert_eq!(client.url_for("home"), "https://leaflet.pub/home");
    }
}
