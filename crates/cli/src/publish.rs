//! `publish build` and `publish run`.
//!
//! **build** is always safe: frontmatter+markdown → canonical record
//! (the existing builder) → pure record JSON at `drafts/<slug>.json`.
//! Network is touched only when `--publication` is passed explicitly
//! (one public `getRecord` to fetch the publication's `url` for URL
//! derivation); the fallback URL derives offline.
//!
//! **run** is the gated write: load the record file (drafts or posted),
//! validate it through the generated lexicon type, and — only with
//! `--yes` plus app-password env — `putRecord` (upsert), **move** the
//! file to `posted/<slug>.json` (a slug lives in exactly one place),
//! and write the sidecar. Without `--yes` the exact record, target uri,
//! and derived URL are printed and the command exits 2. Re-running over
//! an existing posted record re-puts with a hash check against the
//! sidecar.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use springleaf_leaflet::{BuildOptions, PublicationInfo, build_record, derive_document_url};

use crate::workspace::{Sidecar, Workspace, now_rfc3339, sha256_hex};

/// What a successful put returned.
#[derive(Debug, Clone)]
pub(crate) struct PutOutcome {
    pub uri: String,
    pub cid: String,
}

/// Injected transport: `put(rkey, record) -> (uri, cid)`. `Fn` so one
/// connection can serve multiple re-puts (`sync --force`); tests supply
/// a canned future, production wraps `XrpcClient::put_record` with a
/// resolved session.
pub(crate) type PutFuture = Pin<Box<dyn Future<Output = Result<PutOutcome>> + Send>>;
pub(crate) type PutFn = Box<dyn Fn(String, Value) -> PutFuture + Send>;

/// Everything the run flow needs from a live connection. `did` is the
/// repo records go into (the record `site` authority); the put closure
/// carries the session internally.
pub(crate) struct Connected {
    pub did: String,
    pub put: PutFn,
}
pub(crate) type ConnectFuture = Pin<Box<dyn Future<Output = Result<Connected>> + Send>>;
/// `connect(expected_repo_did)` — production resolves the identity and
/// opens (or resumes) a session; tests return a canned put.
pub(crate) type ConnectFn = Box<dyn FnOnce(String) -> ConnectFuture + Send>;

// ── build ──────────────────────────────────────────────────────────────────

/// `publish build` output (human text + `--json` schema v1).
#[derive(Serialize, Debug)]
pub(crate) struct BuildOutcome {
    pub rkey: String,
    pub uri: String,
    pub url: String,
    pub draft: String,
    pub record: Value,
}

/// Build the canonical record and write it — pure JSON, nothing added —
/// to `drafts/<slug>.json`. Errors when the slug already exists unless
/// `overwrite`. Network-free core: `publication_url` is passed in.
pub(crate) fn build_and_write(
    ws: &Workspace,
    markdown: &str,
    publication: &str,
    publication_url: Option<&str>,
    overwrite: bool,
    options: &BuildOptions,
) -> Result<BuildOutcome> {
    let built = build_record(markdown, publication, options)?;
    let draft_path = ws.drafts_dir().join(format!("{}.json", built.rkey));
    if draft_path.exists() && !overwrite {
        bail!("{} already exists; pass --overwrite to replace it", draft_path.display());
    }
    std::fs::create_dir_all(ws.drafts_dir()).with_context(|| format!("creating {}", ws.drafts_dir().display()))?;
    let record_text = format!("{}\n", serde_json::to_string_pretty(&built.record)?);
    std::fs::write(&draft_path, &record_text).with_context(|| format!("writing {}", draft_path.display()))?;
    let url = derive_document_url(publication, publication_url, &built.rkey)?;
    Ok(BuildOutcome {
        rkey: built.rkey,
        uri: built.uri,
        url,
        draft: draft_path.display().to_string(),
        record: built.record,
    })
}

/// `publish build` command body. `publication_url` derivation goes to
/// the network only when the publication came from the `--publication`
/// flag (caller-supplied); the toml default derives the fallback URL
/// offline.
pub(crate) async fn publish_build_cmd(file: &Path, publication_flag: Option<String>, root: Option<PathBuf>, overwrite: bool, json: bool) -> Result<i32> {
    let ws = Workspace::resolve(root.as_deref())?;
    let markdown = std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    let config = ws.config();
    let publication = match publication_flag.as_deref().or(config.publication.as_deref()) {
        Some(p) => p.to_string(),
        None => bail!(
            "no publication target: pass --publication <at-uri> or set `publication` in {}",
            ws.root.join("springleaf.toml").display()
        ),
    };

    // URL derivation: the flag opts into one public getRecord; toml
    // defaults stay offline (fallback /lish URL).
    let mut publication_url = None;
    if publication_flag.is_some() {
        match fetch_publication_url(&publication).await {
            Ok(Some(url)) => publication_url = Some(url),
            Ok(None) => eprintln!("warning: publication record {publication} not found; deriving the fallback URL"),
            Err(err) => eprintln!("warning: could not fetch publication url ({err:#}); deriving the fallback URL"),
        }
    }

    let outcome = build_and_write(&ws, &markdown, &publication, publication_url.as_deref(), overwrite, &BuildOptions::default())?;
    if ws.posted_dir().join(format!("{}.json", outcome.rkey)).exists() {
        eprintln!(
            "warning: posted/{}.json already exists; sync will classify this build as local-newer",
            outcome.rkey
        );
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&outcome.record)?);
        println!("\nrkey:   {}", outcome.rkey);
        println!("uri:    {}", outcome.uri);
        println!("url:    {}", outcome.url);
        println!("draft:  {}", outcome.draft);
    }
    Ok(0)
}

/// Resolve a publication at-uri → its `url` field (public reads: DID
/// document for the repo, then getRecord on the publication).
pub(crate) async fn fetch_publication_url(publication_at_uri: &str) -> Result<Option<String>> {
    let uri = springleaf_atproto::AtUri::parse(publication_at_uri).with_context(|| format!("parsing publication {publication_at_uri}"))?;
    let identity = springleaf_atproto::Resolver::new()
        .resolve(&uri.authority)
        .await
        .with_context(|| format!("resolving {}", uri.authority))?;
    let client = springleaf_atproto::XrpcClient::new(&identity.pds_endpoint)?;
    let info: PublicationInfo = springleaf_leaflet::publication_for_record(&client, publication_at_uri)
        .await
        .with_context(|| format!("fetching publication record {publication_at_uri}"))?
        .context("publication record has no url to derive from")?;
    Ok(info.url)
}

// ── run ────────────────────────────────────────────────────────────────────

/// Where `publish run <arg>` found the record file.
#[derive(Debug, PartialEq)]
pub(crate) enum RecordSource {
    Drafts(PathBuf),
    Posted(PathBuf),
    ExplicitFile(PathBuf),
}

/// Resolve `slug-or-file` to a record file + slug (filename stem). An
/// existing file wins; else `drafts/<arg>.json`, then `posted/<arg>.json`.
pub(crate) fn resolve_source(ws: &Workspace, arg: &str) -> Result<(RecordSource, String)> {
    let as_path = Path::new(arg);
    if as_path.is_file() {
        let slug = slug_from_path(as_path)?;
        return Ok((RecordSource::ExplicitFile(as_path.to_path_buf()), slug));
    }
    let draft = ws.drafts_dir().join(format!("{arg}.json"));
    if draft.is_file() {
        return Ok((RecordSource::Drafts(draft), arg.to_string()));
    }
    let posted = ws.posted_dir().join(format!("{arg}.json"));
    if posted.is_file() {
        return Ok((RecordSource::Posted(posted), arg.to_string()));
    }
    bail!(
        "no record found for {arg:?}: looked for an existing file, {}, and {}",
        draft.display(),
        posted.display()
    );
}

fn slug_from_path(path: &Path) -> Result<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string)
        .context("record filename has no slug stem")
}

/// Result of the sidecar hash check on a re-put.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HashCheck {
    /// No prior sidecar — first put of this record.
    New,
    /// File bytes match the sidecar's contentHash.
    Unchanged,
    /// File was edited since the last put (local-newer re-put).
    Changed,
}

/// `publish run` output (also the `--json` schema v1).
#[derive(Serialize)]
pub(crate) struct RunOutcome {
    pub rkey: String,
    pub uri: String,
    pub cid: String,
    pub url: String,
    pub posted: String,
    pub sidecar: String,
    pub hash_check: &'static str,
}

/// Validate a record file's contents through the generated lexicon type
/// (decode + constraint validation).
pub(crate) fn validate_record_file(record: &Value, source: &Path) -> Result<()> {
    springleaf_leaflet::validate_document_record(record).with_context(|| format!("{} is not a valid site.standard.document", source.display()))
}

/// The put + move + sidecar flow, transport injected. Requires `--yes`
/// gating to have happened at the caller. `derived_url` is computed by
/// the caller (publication url when known, else the /lish fallback).
pub(crate) async fn execute_publish(ws: &Workspace, source: &Path, slug: &str, record: &Value, derived_url: String, put: PutFn) -> Result<RunOutcome> {
    let posted_path = ws.posted_dir().join(format!("{slug}.json"));
    let sidecar_path = Sidecar::path_for(&ws.posted_dir(), slug);

    // Hash check against the existing sidecar (idempotent re-put path).
    let prior: Option<Sidecar> = match std::fs::read(&sidecar_path) {
        Ok(bytes) => Some(serde_json::from_slice(&bytes).context("parsing existing sidecar")?),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => return Err(anyhow::Error::new(err).context(format!("reading {}", sidecar_path.display()))),
    };
    let source_bytes = std::fs::read(source).with_context(|| format!("reading {}", source.display()))?;
    let source_hash = sha256_hex(&source_bytes);
    let hash_check = match &prior {
        None => HashCheck::New,
        Some(sidecar) if sidecar.content_hash == source_hash => HashCheck::Unchanged,
        Some(_) => HashCheck::Changed,
    };
    if hash_check == HashCheck::Changed {
        eprintln!("warning: record file changed since the last put (re-putting the edited bytes)");
    }

    // Put.
    let outcome = put(slug.to_string(), record.clone()).await?;
    if !outcome.uri.ends_with(&format!("/site.standard.document/{slug}")) {
        bail!("PDS returned an unexpected uri {} (expected .../site.standard.document/{slug})", outcome.uri);
    }

    // Move, not copy: a slug lives in exactly one place. Cross-device
    // moves degrade to copy + remove (rename can't cross filesystems).
    ws.ensure_layout()?;
    if source != posted_path {
        match std::fs::rename(source, &posted_path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::CrossesDevices || err.raw_os_error() == Some(18) => {
                std::fs::copy(source, &posted_path).with_context(|| format!("copying {} (cross-device move)", source.display()))?;
                std::fs::remove_file(source).with_context(|| format!("removing moved-away {}", source.display()))?;
            }
            Err(err) => {
                return Err(anyhow::Error::new(err).context(format!("moving {} to {}", source.display(), posted_path.display())));
            }
        }
    }
    // The moved bytes are `source_bytes`; hash them for the sidecar.
    let sidecar = Sidecar {
        rkey: slug.to_string(),
        uri: outcome.uri.clone(),
        cid: outcome.cid.clone(),
        publication: record.get("site").and_then(Value::as_str).unwrap_or_default().to_string(),
        published_at: record.get("publishedAt").and_then(Value::as_str).map(str::to_string),
        last_put_at: Some(now_rfc3339()),
        content_hash: sha256_hex(&source_bytes),
        remote_cid: Some(outcome.cid.clone()),
    };
    std::fs::write(&sidecar_path, serde_json::to_vec_pretty(&sidecar)?).with_context(|| format!("writing {}", sidecar_path.display()))?;

    Ok(RunOutcome {
        rkey: slug.to_string(),
        uri: outcome.uri,
        cid: outcome.cid,
        url: derived_url,
        posted: posted_path.display().to_string(),
        sidecar: sidecar_path.display().to_string(),
        hash_check: match hash_check {
            HashCheck::New => "new",
            HashCheck::Unchanged => "unchanged",
            HashCheck::Changed => "changed",
        },
    })
}

/// The did a record is put under: the `site` field's at-uri authority —
/// or, for standalone records, the DID embedded in the
/// `https://leaflet.pub/p/{did}` site form (`drafts export` writes it
/// for drafts with no publication linkage).
fn record_site_did(record: &Value) -> Result<String> {
    let site = record.get("site").and_then(Value::as_str).context("record has no `site` field")?;
    if let Some(did) = springleaf_leaflet::standalone_site_did(site) {
        return Ok(did);
    }
    let uri = springleaf_atproto::AtUri::parse(site).with_context(|| format!("parsing site at-uri {site:?}"))?;
    Ok(uri.authority)
}

/// `publish run` command body. `pub_info`/`connect` are injection points
/// for offline tests (`None` means "use the network paths").
pub(crate) async fn publish_run_cmd(
    slug_or_file: &str,
    root: Option<PathBuf>,
    yes: bool,
    json: bool,
    pub_info: Option<PublicationInfo>,
    connect: Option<ConnectFn>,
) -> Result<i32> {
    let ws = Workspace::resolve(root.as_deref())?;
    let (source, slug) = resolve_source(&ws, slug_or_file)?;
    let record_text = std::fs::read_to_string(source_path(&source)).with_context(|| format!("reading {}", source_path(&source).display()))?;
    let record: Value = serde_json::from_str(&record_text).context("record file is not valid JSON")?;
    validate_record_file(&record, source_path(&source))?;
    springleaf_atproto::validate_rkey(&slug).with_context(|| format!("filename slug {slug:?}"))?;
    if record.get("path").and_then(Value::as_str) != Some(&format!("/{slug}")) {
        eprintln!(
            "warning: record path {:?} does not match its filename rkey /{slug} (Leaflet resolves by rkey; path drives the URL)",
            record.get("path").and_then(Value::as_str).unwrap_or("(absent)")
        );
    }

    // Derived URL: publication url when we know it, else the fallback.
    let site = record.get("site").and_then(Value::as_str).context("record has no `site` field")?.to_string();
    let pub_info = match pub_info {
        Some(info) => Some(info),
        None => fetch_pub_info(&site).await,
    };
    let url = derive_document_url(&site, pub_info.as_ref().and_then(|i| i.url.as_deref()), &slug)?;
    let expected_did = record_site_did(&record)?;
    let target_uri = format!("at://{expected_did}/site.standard.document/{slug}");

    if !yes {
        // The refusal path prints the exact record + destination.
        println!("{}", serde_json::to_string_pretty(&record)?);
        println!("\nrkey:   {slug}");
        println!("target: {target_uri}");
        println!("url:    {url}");
        println!("record: {}", source_path(&source).display());
        println!(
            "\nrefusing to publish without --yes: re-run with --yes (requires ATPROTO_HANDLE and \
             ATPROTO_APP_PASSWORD; ATPROTO_PDS optional, default https://bsky.social)"
        );
        return Ok(2);
    }

    let connect = connect.unwrap_or_else(|| Box::new(|expected_did| Box::pin(connect_from_env(expected_did))));
    let connected = connect(expected_did.clone()).await?;
    if connected.did != expected_did {
        bail!(
            "session did {} does not match the record's publication owner {expected_did} — wrong account?",
            connected.did
        );
    }
    let outcome = execute_publish(&ws, source_path(&source), &slug, &record, url, connected.put).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        println!("put:    {} (cid {})", outcome.uri, outcome.cid);
        println!("url:    {}", outcome.url);
        println!("posted: {}", outcome.posted);
        println!("meta:   {}", outcome.sidecar);
        println!("hash:   {}", outcome.hash_check);
    }
    Ok(0)
}

/// Fetch publication info for URL derivation, degrading to "no url"
/// (fallback derivation) when the public read fails. Standalone HTTPS
/// sites skip this entirely — there is no publication record to fetch.
async fn fetch_pub_info(site: &str) -> Option<PublicationInfo> {
    let uri = springleaf_atproto::AtUri::parse(site).ok()?;
    let identity = springleaf_atproto::Resolver::new().resolve(&uri.authority).await.ok()?;
    let client = springleaf_atproto::XrpcClient::new(&identity.pds_endpoint).ok()?;
    match springleaf_leaflet::publication_for_record(&client, site).await {
        Ok(info) => info,
        Err(err) => {
            eprintln!("warning: publication fetch failed ({err:#}); deriving the fallback URL");
            None
        }
    }
}

/// Real transport: app-password env → resume-or-create session → the
/// PDS resolved from the record's repo did. The session JWTs live inside
/// the returned closure; nothing is logged.
pub(crate) async fn connect_from_env(expected_did: String) -> Result<Connected> {
    let handle = std::env::var("ATPROTO_HANDLE").context("set ATPROTO_HANDLE (the session identifier)")?;
    let password = std::env::var("ATPROTO_APP_PASSWORD").context("set ATPROTO_APP_PASSWORD (an app password, not the account password)")?;
    let pds = std::env::var("ATPROTO_PDS").unwrap_or_else(|_| "https://bsky.social".to_string());
    let store = springleaf_atproto::SessionStore::xdg()?;
    let session = springleaf_atproto::resume_or_create_session(&store, &handle, &password, &pds).await?;
    if session.did != expected_did {
        bail!(
            "session did {} does not match the record's publication owner {expected_did} — wrong account \
             or ATPROTO_HANDLE?",
            session.did
        );
    }
    let identity = springleaf_atproto::Resolver::new()
        .resolve(&expected_did)
        .await
        .with_context(|| format!("resolving {expected_did}"))?;
    let client = springleaf_atproto::XrpcClient::new(&identity.pds_endpoint)?;
    let did = identity.did.to_string();
    Ok(Connected {
        did: did.clone(),
        put: Box::new(move |rkey, record| {
            let client = client.clone();
            let session = session.clone();
            let did = did.clone();
            Box::pin(async move {
                let out = client.put_record(&session, &did, "site.standard.document", &rkey, &record).await?;
                Ok(PutOutcome { uri: out.uri, cid: out.cid })
            })
        }),
    })
}

pub(crate) fn source_path(source: &RecordSource) -> &Path {
    match source {
        RecordSource::Drafts(path) | RecordSource::Posted(path) | RecordSource::ExplicitFile(path) => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::TempDir;
    use serde_json::json;
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    const SAMPLE_MD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../experiments/03-post-builder/sample.md"));
    const GOLDEN_RECORD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../experiments/03-post-builder/built-record.json"));
    const PUBLICATION: &str = "at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.publication/3molsbwkh522o";
    const DID: &str = "did:plc:zjbq26wybii5ojoypkso2mso";

    fn frozen_options() -> BuildOptions {
        BuildOptions {
            published_at: Some("2026-11-07T00:05:30.799Z".into()),
            page_id: Some("01a1b353-1c2f-7d4d-9284-3fa5d8782a4a".into()),
        }
    }

    fn workspace() -> (TempDir, Workspace) {
        let tmp = TempDir::new("publish");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        (tmp, ws)
    }

    /// A canned put transport: records calls, returns sequential cids.
    struct FakePut {
        calls: Arc<Mutex<Vec<(String, Value)>>>,
        cids: Arc<Mutex<std::vec::IntoIter<String>>>,
    }

    impl FakePut {
        fn new(cids: &[&str]) -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
                cids: Arc::new(Mutex::new(cids.iter().map(|s| s.to_string()).collect::<Vec<_>>().into_iter())),
            }
        }

        fn into_put_fn(self) -> PutFn {
            let calls = self.calls;
            let cids = self.cids;
            Box::new(move |rkey, record| {
                let calls = calls.clone();
                let cids = cids.clone();
                Box::pin(async move {
                    calls.lock().unwrap().push((rkey.clone(), record));
                    let cid = cids.lock().unwrap().next().unwrap_or_else(|| "bafyreifresh".into());
                    Ok(PutOutcome {
                        uri: format!("at://{DID}/site.standard.document/{rkey}"),
                        cid,
                    })
                })
            })
        }
    }

    // ── build ─────────────────────────────────────────────────────────────

    /// Purity: the draft file is exactly the record — exact key set, no
    /// local fields — and building sample.md reproduces experiment 03's
    /// golden record modulo the frozen timestamp/page id (here frozen,
    /// so: exactly).
    #[test]
    fn build_writes_pure_record_matching_the_golden() {
        let (_tmp, ws) = workspace();
        let outcome = build_and_write(&ws, SAMPLE_MD, PUBLICATION, None, false, &frozen_options()).unwrap();
        assert_eq!(outcome.rkey, "springleaf-protocol-probe");
        let text = std::fs::read_to_string(&outcome.draft).unwrap();
        assert!(text.ends_with('\n'), "single trailing newline, nothing else added");
        let value: Value = serde_json::from_str(&text).unwrap();
        let keys: BTreeSet<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            BTreeSet::from(["$type", "content", "description", "path", "publishedAt", "site", "tags", "textContent", "title"]),
            "draft file must be the pure record — no extra keys"
        );
        assert!(!text.contains("_local") && !text.contains("frontmatter"));
        let golden: Value = serde_json::from_str(GOLDEN_RECORD).unwrap();
        assert_eq!(value, golden, "frozen options reproduce built-record.json byte-for-value");
        // Fallback URL (no publication url fetched).
        assert_eq!(
            outcome.url,
            "https://leaflet.pub/lish/did:plc:zjbq26wybii5ojoypkso2mso/3molsbwkh522o/springleaf-protocol-probe"
        );
        // Publication-domain URL when the url is known.
        let outcome = build_and_write(&ws, SAMPLE_MD, PUBLICATION, Some("https://technautical.leaflet.pub/"), true, &frozen_options()).unwrap();
        assert_eq!(outcome.url, "https://technautical.leaflet.pub/springleaf-protocol-probe");
    }

    /// Golden parity with *live* (non-frozen) inputs: everything equals
    /// the golden record except publishedAt and the page id.
    #[test]
    fn build_parity_modulo_slug_timestamps() {
        let (_tmp, ws) = workspace();
        let options = BuildOptions {
            published_at: None,
            page_id: Some("fresh-page-id".into()),
        };
        let outcome = build_and_write(&ws, SAMPLE_MD, PUBLICATION, None, false, &options).unwrap();
        let value: Value = serde_json::from_str(&std::fs::read_to_string(&outcome.draft).unwrap()).unwrap();
        let mut golden: Value = serde_json::from_str(GOLDEN_RECORD).unwrap();
        golden["publishedAt"] = value["publishedAt"].clone();
        golden["content"]["pages"][0]["id"] = json!("fresh-page-id");
        assert_eq!(value, golden);
        // The timestamp is a real now, not the frozen one.
        assert_ne!(value["publishedAt"], json!("2026-11-07T00:05:30.799Z"));
    }

    #[test]
    fn build_refuses_existing_slug_and_overwrites_on_demand() {
        let (_tmp, ws) = workspace();
        build_and_write(&ws, SAMPLE_MD, PUBLICATION, None, false, &frozen_options()).unwrap();
        let err = build_and_write(&ws, SAMPLE_MD, PUBLICATION, None, false, &frozen_options())
            .unwrap_err()
            .to_string();
        assert!(err.contains("already exists"), "{err}");
        build_and_write(&ws, SAMPLE_MD, PUBLICATION, None, true, &frozen_options()).unwrap_or_else(|e| panic!("overwrite replaces: {e}"));
    }

    #[test]
    fn source_resolution_slug_drafts_posted_and_files() {
        let (_tmp, ws) = workspace();
        ws.ensure_layout().unwrap();
        let err = resolve_source(&ws, "ghost").unwrap_err().to_string();
        assert!(err.contains("ghost"), "{err}");
        std::fs::write(ws.drafts_dir().join("draft-one.json"), "{}").unwrap();
        assert_eq!(
            resolve_source(&ws, "draft-one").unwrap(),
            (RecordSource::Drafts(ws.drafts_dir().join("draft-one.json")), "draft-one".into())
        );
        // Posted wins for its own slug only when drafts lacks it.
        std::fs::write(ws.posted_dir().join("posted-one.json"), "{}").unwrap();
        assert_eq!(
            resolve_source(&ws, "posted-one").unwrap(),
            (RecordSource::Posted(ws.posted_dir().join("posted-one.json")), "posted-one".into())
        );
        // An existing file path wins outright.
        let outside = ws.root.join("loose.json");
        std::fs::write(&outside, "{}").unwrap();
        assert_eq!(
            resolve_source(&ws, outside.to_str().unwrap()).unwrap(),
            (RecordSource::ExplicitFile(outside.clone()), "loose".into())
        );
    }

    // ── run ───────────────────────────────────────────────────────────────

    fn draft_record(ws: &Workspace) -> (std::path::PathBuf, Value) {
        let outcome = build_and_write(ws, SAMPLE_MD, PUBLICATION, None, false, &frozen_options()).unwrap();
        let path = std::path::PathBuf::from(&outcome.draft);
        let record: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        (path, record)
    }

    #[tokio::test]
    async fn run_moves_the_file_and_writes_the_sidecar() {
        let (_tmp, ws) = workspace();
        let (draft_path, record) = draft_record(&ws);
        let put = FakePut::new(&["bafyreifirst"]);
        let outcome = execute_publish(&ws, &draft_path, "springleaf-protocol-probe", &record, "https://x/y".into(), put.into_put_fn())
            .await
            .unwrap();
        // Move, not copy: drafts no longer holds it; posted does, byte-faithful.
        assert!(!draft_path.exists(), "drafts/<slug>.json must be gone (move, not copy)");
        let posted = ws.posted_dir().join("springleaf-protocol-probe.json");
        let posted_text = std::fs::read_to_string(&posted).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&posted_text).unwrap(), record);
        assert_eq!(outcome.hash_check, "new");
        // The put received the record verbatim.
        // (FakePut is moved into the put fn; sidecar assertions follow.)
        assert_eq!(outcome.uri, format!("at://{DID}/site.standard.document/springleaf-protocol-probe"));
        assert_eq!(outcome.cid, "bafyreifirst");
        // Sidecar: the eight canonical fields.
        let sidecar: crate::workspace::Sidecar =
            serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), "springleaf-protocol-probe")).unwrap()).unwrap();
        assert_eq!(sidecar.rkey, "springleaf-protocol-probe");
        assert_eq!(sidecar.uri, outcome.uri);
        assert_eq!(sidecar.cid, "bafyreifirst");
        assert_eq!(sidecar.remote_cid.as_deref(), Some("bafyreifirst"));
        assert_eq!(sidecar.publication, PUBLICATION);
        assert_eq!(sidecar.published_at.as_deref(), Some("2026-11-07T00:05:30.799Z"));
        assert!(sidecar.last_put_at.is_some(), "lastPutAt records our put");
        assert_eq!(sidecar.content_hash, sha256_hex(posted_text.as_bytes()));
    }

    #[tokio::test]
    async fn rerun_over_posted_is_an_idempotent_reput_with_hash_check() {
        let (_tmp, ws) = workspace();
        let (draft_path, record) = draft_record(&ws);
        let put = FakePut::new(&["bafyreifirst"]);
        let _ = execute_publish(&ws, &draft_path, "springleaf-protocol-probe", &record, String::new(), put.into_put_fn())
            .await
            .unwrap();
        // Re-run over the posted file: hash unchanged → re-put, sidecar's cid/remoteCid refresh.
        let posted = ws.posted_dir().join("springleaf-protocol-probe.json");
        let put = FakePut::new(&["bafyreisecond"]);
        let outcome = execute_publish(&ws, &posted, "springleaf-protocol-probe", &record, String::new(), put.into_put_fn())
            .await
            .unwrap();
        assert_eq!(outcome.hash_check, "unchanged");
        let sidecar: crate::workspace::Sidecar =
            serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), "springleaf-protocol-probe")).unwrap()).unwrap();
        assert_eq!(sidecar.cid, "bafyreisecond");
        assert_eq!(sidecar.remote_cid.as_deref(), Some("bafyreisecond"));
        // Edit the file → hash check reports changed and still re-puts the new bytes.
        let mut edited = record.clone();
        edited["title"] = json!("Edited title");
        std::fs::write(&posted, serde_json::to_string_pretty(&edited).unwrap()).unwrap();
        let put = FakePut::new(&["bafyreithird"]);
        let outcome = execute_publish(&ws, &posted, "springleaf-protocol-probe", &edited, String::new(), put.into_put_fn())
            .await
            .unwrap();
        assert_eq!(outcome.hash_check, "changed");
        let sidecar: crate::workspace::Sidecar =
            serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), "springleaf-protocol-probe")).unwrap()).unwrap();
        assert_eq!(sidecar.cid, "bafyreithird");
        assert_eq!(sidecar.content_hash, sha256_hex(std::fs::read_to_string(&posted).unwrap().as_bytes()));
    }

    #[tokio::test]
    async fn run_without_yes_exits_2_without_touching_the_transport() {
        let (_tmp, ws) = workspace();
        let (_path, _record) = draft_record(&ws);
        let called = Arc::new(AtomicBool::new(false));
        let called_for_connect = called.clone();
        let connect: ConnectFn = Box::new(move |expected| {
            let called = called_for_connect.clone();
            Box::pin(async move {
                called.store(true, Ordering::SeqCst);
                Ok(Connected {
                    did: expected,
                    put: Box::new(|_, _| Box::pin(async { bail!("refusal path must not put") })),
                })
            })
        });
        let code = publish_run_cmd(
            "springleaf-protocol-probe",
            Some(ws.root.clone()),
            false, // no --yes
            false,
            Some(PublicationInfo {
                name: Some("Technautical".into()),
                url: Some("https://technautical.leaflet.pub".into()),
            }),
            Some(connect),
        )
        .await
        .unwrap();
        assert_eq!(code, 2, "without --yes the command refuses with exit 2");
        assert!(!called.load(Ordering::SeqCst), "the transport must not even be opened");
    }

    #[tokio::test]
    async fn run_with_yes_puts_through_the_injected_transport() {
        let (_tmp, ws) = workspace();
        let (_path, _record) = draft_record(&ws);
        let connect: ConnectFn = Box::new(|expected| {
            Box::pin(async move {
                Ok(Connected {
                    did: expected,
                    put: FakePut::new(&["bafyreirun"]).into_put_fn(),
                })
            })
        });
        let code = publish_run_cmd(
            "springleaf-protocol-probe",
            Some(ws.root.clone()),
            true,
            true,
            Some(PublicationInfo {
                name: None,
                url: Some("https://technautical.leaflet.pub".into()),
            }),
            Some(connect),
        )
        .await
        .unwrap();
        assert_eq!(code, 0);
        assert!(ws.posted_dir().join("springleaf-protocol-probe.json").exists());
        assert!(Sidecar::path_for(&ws.posted_dir(), "springleaf-protocol-probe").exists());
        assert!(!ws.drafts_dir().join("springleaf-protocol-probe.json").exists());
    }

    #[test]
    fn invalid_records_are_rejected_by_validation() {
        let missing = json!({"$type": "site.standard.document"});
        assert!(crate::publish::validate_record_file(&missing, Path::new("x.json")).is_err());
        let bad_type = json!({"$type": "com.example.other", "title": "t"});
        assert!(crate::publish::validate_record_file(&bad_type, Path::new("x.json")).is_err());
    }

    // ── standalone site records (the 5a gap) ─────────────────────────────

    /// A record whose `site` is the standalone HTTPS form — what
    /// `drafts export` writes for drafts with no publication linkage.
    fn standalone_record(ws: &Workspace) -> std::path::PathBuf {
        let (_path, mut record) = draft_record(ws);
        record["site"] = json!("https://leaflet.pub/p/did:plc:zjbq26wybii5ojoypkso2mso");
        let path = ws.drafts_dir().join("standalone-probe.json");
        std::fs::write(&path, serde_json::to_string_pretty(&record).unwrap()).unwrap();
        path
    }

    /// Standalone records flow through `publish run` end to end: the put
    /// target repo is the DID embedded in the site URL, the derived URL
    /// is the `/p/{did}/{rkey}` form, and the execution path (with the
    /// injected transport) puts + moves + writes the sidecar like any
    /// publication-linked record.
    #[tokio::test]
    async fn run_accepts_standalone_site_records() {
        let (_tmp, ws) = workspace();
        let path = standalone_record(&ws);

        // Dry-run: exit 2, target + /p/ URL printed, no transport opened.
        let called = Arc::new(AtomicBool::new(false));
        let called_for_connect = called.clone();
        let connect: ConnectFn = Box::new(move |expected| {
            let called = called_for_connect.clone();
            Box::pin(async move {
                called.store(true, Ordering::SeqCst);
                Ok(Connected {
                    did: expected,
                    put: Box::new(|_, _| Box::pin(async { bail!("refusal path must not put") })),
                })
            })
        });
        let code = publish_run_cmd("standalone-probe", Some(ws.root.clone()), false, false, None, Some(connect))
            .await
            .unwrap();
        assert_eq!(code, 2, "standalone dry-run refuses with exit 2 like the linked form");
        assert!(!called.load(Ordering::SeqCst), "the transport must not even be opened");
        assert!(ws.drafts_dir().join("standalone-probe.json").exists(), "dry-run leaves the draft in place");

        // Execute through the injected transport: put under the site's did.
        let connect: ConnectFn = Box::new(|expected| {
            assert_eq!(expected, DID, "the target repo comes from the site URL's DID");
            Box::pin(async {
                Ok(Connected {
                    did: expected,
                    put: FakePut::new(&["bafyreistandalone"]).into_put_fn(),
                })
            })
        });
        let code = publish_run_cmd("standalone-probe", Some(ws.root.clone()), true, true, None, Some(connect))
            .await
            .unwrap();
        assert_eq!(code, 0);
        assert!(ws.posted_dir().join("standalone-probe.json").exists());
        let sidecar: crate::workspace::Sidecar =
            serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), "standalone-probe")).unwrap()).unwrap();
        assert_eq!(sidecar.publication, "https://leaflet.pub/p/did:plc:zjbq26wybii5ojoypkso2mso");
        assert_eq!(sidecar.uri, format!("at://{DID}/site.standard.document/standalone-probe"));
        // And the pre-existing at-uri form still resolves untouched.
        let code = publish_run_cmd(
            "springleaf-protocol-probe",
            Some(ws.root.clone()),
            false,
            false,
            Some(PublicationInfo {
                name: None,
                url: Some("https://technautical.leaflet.pub".into()),
            }),
            None::<ConnectFn>,
        )
        .await
        .unwrap();
        assert_eq!(code, 2, "the linked at-uri form keeps refusing in dry-run");
    }

    /// The standalone dry-run prints the /p/ URL derivation (checked via
    /// the shared helper rather than stdout capture).
    #[test]
    fn standalone_url_derives_the_p_form() {
        let site = "https://leaflet.pub/p/did:plc:zjbq26wybii5ojoypkso2mso";
        assert_eq!(
            derive_document_url(site, None, "standalone-probe").unwrap(),
            "https://leaflet.pub/p/did:plc:zjbq26wybii5ojoypkso2mso/standalone-probe"
        );
        assert_eq!(record_site_did(&json!({"site": site})).unwrap(), "did:plc:zjbq26wybii5ojoypkso2mso");
        assert_eq!(record_site_did(&json!({"site": PUBLICATION})).unwrap(), DID);
        assert!(record_site_did(&json!({"site": "https://example.com/p/did:plc:x"})).is_err());
    }
}
