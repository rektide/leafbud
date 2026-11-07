//! `sync` — reconcile `posted/` against `listRecords` on the resolved
//! PDS (public read, no auth), by rkey.
//!
//! Semantics from `design/records-canonical.glm53m.md`'s table, with the
//! rows the mission names: **clean** (local hash unchanged, remote cid
//! equals the sidecar's `remoteCid`), **local-newer** (hash changed, cid
//! unchanged → re-put), **remote-drift** (hash unchanged, cid changed →
//! `--pull` adopts remote bytes or `--force` re-puts), **true-conflict**
//! (both changed → block, explicit resolution), **remote-orphan**
//! (remote only → `--pull` adopts into posted/). One row the design
//! table does not spell out — **local-missing** (posted locally, absent
//! remotely) — is classified here and resolves by re-put, since posted/
//! is the source of truth for what should exist remotely. A posted
//! record **without a sidecar** (or sidecar without `remoteCid`) cannot
//! be compared and reports as **no-sidecar** rather than guessing.
//!
//! Default is report-only. `--pull` writes locally (never remotely);
//! `--force` re-put paths refuse without `--yes`. Nothing is ever
//! deleted.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

use crate::publish::ConnectFn;
use crate::workspace::{Sidecar, Workspace, now_rfc3339, sha256_hex};

/// The collection springleaf publishes and syncs.
pub(crate) const COLLECTION: &str = "site.standard.document";

/// One remote record from `listRecords`.
#[derive(Debug, Clone)]
pub(crate) struct RemoteRec {
    pub rkey: String,
    pub uri: String,
    pub cid: String,
    pub value: Value,
}

impl RemoteRec {
    pub(crate) fn from_raw(raw: &springleaf_atproto::RawRecord) -> Self {
        let rkey = raw.uri.rsplit('/').next().unwrap_or_default().to_string();
        Self {
            rkey,
            uri: raw.uri.clone(),
            cid: raw.cid.clone(),
            value: raw.value.clone(),
        }
    }
}

/// One local posted record (pure file + optional sidecar).
#[derive(Debug, Clone)]
pub(crate) struct LocalPosted {
    pub rkey: String,
    pub path: PathBuf,
    /// sha256 of the file's current bytes.
    pub hash: String,
    pub sidecar: Option<Sidecar>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SyncStatus {
    Clean,
    LocalNewer,
    RemoteDrift,
    TrueConflict,
    RemoteOrphan,
    LocalMissing,
    /// Posted file with no (or partial) sidecar — comparison impossible.
    NoSidecar,
}

impl SyncStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SyncStatus::Clean => "clean",
            SyncStatus::LocalNewer => "local-newer",
            SyncStatus::RemoteDrift => "remote-drift",
            SyncStatus::TrueConflict => "true-conflict",
            SyncStatus::RemoteOrphan => "remote-orphan",
            SyncStatus::LocalMissing => "local-missing",
            SyncStatus::NoSidecar => "no-sidecar",
        }
    }

    fn note(self) -> &'static str {
        match self {
            SyncStatus::Clean => "in sync",
            SyncStatus::LocalNewer => "local is newer: `sync --force --yes` re-puts",
            SyncStatus::RemoteDrift => "remote changed: `--pull` adopts remote, `--force --yes` re-puts local",
            SyncStatus::TrueConflict => "both sides changed: resolve explicitly (edit the record, or adopt the remote bytes by hand)",
            SyncStatus::RemoteOrphan => "remote only: `--pull` adopts into posted/",
            SyncStatus::LocalMissing => "posted locally but absent remotely: `--force --yes` re-puts",
            SyncStatus::NoSidecar => "no sidecar to compare: `--force --yes` re-put writes one",
        }
    }
}

#[derive(Debug)]
pub(crate) struct SyncRow {
    pub status: SyncStatus,
    pub rkey: String,
    pub local: Option<LocalPosted>,
    pub remote: Option<RemoteRec>,
}

#[derive(Debug)]
pub(crate) struct SyncReport {
    pub handle: Option<String>,
    pub did: String,
    pub pds: String,
    pub workspace: String,
    pub rows: Vec<SyncRow>,
}

impl SyncReport {
    pub fn count(&self, status: SyncStatus) -> usize {
        self.rows.iter().filter(|row| row.status == status).count()
    }

    /// Rows `--force` may re-put (everything except clean, true-conflict,
    /// and remote-only rows).
    pub fn force_eligible(&self) -> Vec<&SyncRow> {
        self.rows
            .iter()
            .filter(|row| {
                matches!(
                    row.status,
                    SyncStatus::LocalNewer | SyncStatus::LocalMissing | SyncStatus::RemoteDrift | SyncStatus::NoSidecar
                )
            })
            .collect()
    }
}

/// Scan `posted/` for `<slug>.json` + sidecars (meta files skipped).
pub(crate) fn scan_posted(ws: &Workspace) -> Result<Vec<LocalPosted>> {
    let posted = ws.posted_dir();
    if !posted.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&posted).with_context(|| format!("reading {}", posted.display()))? {
        let entry = entry.with_context(|| format!("reading {} entries", posted.display()))?;
        let path = entry.path();
        let is_meta = path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".meta.json"));
        let is_json = path.extension().is_some_and(|e| e == "json");
        if !path.is_file() || is_meta || !is_json {
            continue;
        }
        let Some(rkey) = path.file_stem().and_then(|s| s.to_str()).map(str::to_string) else {
            continue;
        };
        let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let sidecar_path = Sidecar::path_for(&posted, &rkey);
        let sidecar = match std::fs::read(&sidecar_path) {
            Ok(raw) => Some(serde_json::from_slice(&raw).with_context(|| format!("parsing {}", sidecar_path.display()))?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => return Err(anyhow::Error::new(err).context(format!("reading {}", sidecar_path.display()))),
        };
        out.push(LocalPosted {
            rkey,
            path,
            hash: sha256_hex(&bytes),
            sidecar,
        });
    }
    out.sort_by(|a, b| a.rkey.cmp(&b.rkey));
    Ok(out)
}

/// Classify the union of local and remote rkeys per the semantics table.
pub(crate) fn classify(mut locals: Vec<LocalPosted>, mut remotes: Vec<RemoteRec>) -> Vec<SyncRow> {
    locals.sort_by(|a, b| a.rkey.cmp(&b.rkey));
    remotes.sort_by(|a, b| a.rkey.cmp(&b.rkey));
    let mut rows = Vec::new();
    let mut local_iter = locals.into_iter().peekable();
    let mut remote_iter = remotes.into_iter().peekable();
    loop {
        let next_local = local_iter.peek().map(|l| l.rkey.as_str());
        let next_remote = remote_iter.peek().map(|r| r.rkey.as_str());
        let rkey = match (next_local, next_remote) {
            (None, None) => break,
            (Some(l), Some(r)) => l.min(r).to_string(),
            (Some(l), None) => l.to_string(),
            (None, Some(r)) => r.to_string(),
        };
        let local = if next_local == Some(rkey.as_str()) { local_iter.next() } else { None };
        let remote = if next_remote == Some(rkey.as_str()) { remote_iter.next() } else { None };
        let status = match (&local, &remote) {
            (Some(l), Some(r)) => match &l.sidecar {
                None => SyncStatus::NoSidecar,
                Some(sidecar) => {
                    let local_changed = l.hash != sidecar.content_hash;
                    let remote_changed = match &sidecar.remote_cid {
                        Some(cid) => r.cid != *cid,
                        None => true, // indeterminate without a baseline → drift path
                    };
                    match (local_changed, remote_changed) {
                        (false, false) => SyncStatus::Clean,
                        (true, false) => SyncStatus::LocalNewer,
                        (false, true) => SyncStatus::RemoteDrift,
                        (true, true) => SyncStatus::TrueConflict,
                    }
                }
            },
            (Some(_), None) => match local.as_ref().and_then(|l| l.sidecar.as_ref()) {
                // A sidecar means springleaf put this once; absence
                // remotely needs a re-put. Without one it may never have
                // been put at all.
                Some(_) => SyncStatus::LocalMissing,
                None => SyncStatus::NoSidecar,
            },
            (None, Some(_)) => SyncStatus::RemoteOrphan,
            (None, None) => unreachable!("rkey came from one of the two sides"),
        };
        rows.push(SyncRow { status, rkey, local, remote });
    }
    rows
}

/// What `--pull` did to one rkey.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PullAction {
    /// Remote orphan adopted into posted/ (file + sidecar).
    Adopted,
    /// Remote drift overwritten onto the local file (bytes + sidecar).
    Updated,
}

/// Adopt remote records locally (never a remote write): remote orphans
/// land in posted/ with a fresh sidecar; drift rows get the remote bytes
/// (safe because a drift row's local bytes equal its last put).
/// `lastPutAt` stays null — springleaf never put these bytes.
pub(crate) fn apply_pull(ws: &Workspace, report: &SyncReport) -> Result<Vec<(String, PullAction)>> {
    std::fs::create_dir_all(ws.posted_dir()).with_context(|| format!("creating {}", ws.posted_dir().display()))?;
    let mut done = Vec::new();
    for row in &report.rows {
        let Some(remote) = &row.remote else { continue };
        match row.status {
            SyncStatus::RemoteOrphan => {
                write_remote_as_posted(ws, remote)?;
                done.push((row.rkey.clone(), PullAction::Adopted));
            }
            SyncStatus::RemoteDrift => {
                write_remote_as_posted(ws, remote)?;
                done.push((row.rkey.clone(), PullAction::Updated));
            }
            _ => {}
        }
    }
    Ok(done)
}

fn write_remote_as_posted(ws: &Workspace, remote: &RemoteRec) -> Result<()> {
    let bytes = format!("{}\n", serde_json::to_string_pretty(&remote.value)?);
    let record_path = ws.posted_dir().join(format!("{}.json", remote.rkey));
    std::fs::write(&record_path, &bytes).with_context(|| format!("writing {}", record_path.display()))?;
    let sidecar = Sidecar {
        rkey: remote.rkey.clone(),
        uri: remote.uri.clone(),
        cid: remote.cid.clone(),
        publication: remote.value.get("site").and_then(Value::as_str).unwrap_or_default().to_string(),
        published_at: remote.value.get("publishedAt").and_then(Value::as_str).map(str::to_string),
        last_put_at: None,
        content_hash: sha256_hex(bytes.as_bytes()),
        remote_cid: Some(remote.cid.clone()),
    };
    let sidecar_path = Sidecar::path_for(&ws.posted_dir(), &remote.rkey);
    std::fs::write(&sidecar_path, serde_json::to_vec_pretty(&sidecar)?).with_context(|| format!("writing {}", sidecar_path.display()))?;
    Ok(())
}

/// `--force --yes` execution: re-put each eligible local record and
/// refresh its sidecar. Transport injected (offline tests use a canned
/// put; the live path is operator-gated).
pub(crate) async fn execute_force(ws: &Workspace, report: &SyncReport, connect: ConnectFn) -> Result<Vec<String>> {
    let rows = report.force_eligible();
    let Some(first) = rows.first() else { return Ok(Vec::new()) };
    let expected_did = first
        .local
        .as_ref()
        .and_then(|l| l.sidecar.as_ref())
        .and_then(|s| springleaf_atproto::AtUri::parse(&s.uri).ok())
        .map(|uri| uri.authority)
        .or_else(|| first.local.as_ref().and_then(|l| did_from_record_file(&l.path)));
    let Some(expected_did) = expected_did else {
        bail!("cannot determine the repo did for force re-puts (no sidecar uri, no record site field)");
    };
    let connected = connect(expected_did.clone()).await?;
    if connected.did != expected_did {
        bail!(
            "session did {} does not match the records' repo {expected_did} — wrong account or ATPROTO_HANDLE?",
            connected.did
        );
    }
    let mut done = Vec::new();
    for row in rows {
        let Some(local) = &row.local else { continue };
        let bytes = std::fs::read(&local.path).with_context(|| format!("reading {}", local.path.display()))?;
        let record: Value = serde_json::from_slice(&bytes).context(format!("parsing {}", local.path.display()))?;
        let outcome = (connected.put)(row.rkey.clone(), record.clone()).await?;
        let sidecar = Sidecar {
            rkey: row.rkey.clone(),
            uri: outcome.uri.clone(),
            cid: outcome.cid.clone(),
            publication: record.get("site").and_then(Value::as_str).unwrap_or_default().to_string(),
            published_at: record.get("publishedAt").and_then(Value::as_str).map(str::to_string),
            last_put_at: Some(now_rfc3339()),
            content_hash: sha256_hex(&bytes),
            remote_cid: Some(outcome.cid.clone()),
        };
        let sidecar_path = Sidecar::path_for(&ws.posted_dir(), &row.rkey);
        std::fs::write(&sidecar_path, serde_json::to_vec_pretty(&sidecar)?).with_context(|| format!("writing {}", sidecar_path.display()))?;
        done.push(row.rkey.clone());
    }
    Ok(done)
}

fn did_from_record_file(path: &std::path::Path) -> Option<String> {
    let record: Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    let site = record.get("site")?.as_str()?;
    springleaf_atproto::AtUri::parse(site).ok().map(|uri| uri.authority)
}

/// The remote side, resolved for real: handle → identity → listRecords.
pub(crate) async fn load_remote(handle: &str) -> Result<SyncTarget> {
    let resolver = springleaf_atproto::Resolver::new();
    let identity = resolver.resolve(handle).await.with_context(|| format!("resolving identity for {handle}"))?;
    let client = springleaf_atproto::XrpcClient::new(&identity.pds_endpoint)?;
    let did = identity.did.to_string();
    let raws = client
        .list_all_records(&did, COLLECTION)
        .await
        .with_context(|| format!("listRecords {COLLECTION} on {}", identity.pds_endpoint))?;
    Ok(SyncTarget {
        handle: springleaf_atproto::Handle::parse(handle).ok().map(|h| h.to_string()),
        did,
        pds: identity.pds_endpoint,
        records: raws.iter().map(RemoteRec::from_raw).collect(),
    })
}

/// Remote data for a sync run (injected in offline tests).
pub(crate) struct SyncTarget {
    pub handle: Option<String>,
    pub did: String,
    pub pds: String,
    pub records: Vec<RemoteRec>,
}

/// Inputs to [`sync_cmd`] (flag-derived plus test injection points).
pub(crate) struct SyncOptions {
    pub handle: Option<String>,
    pub root: Option<PathBuf>,
    pub pull: bool,
    pub force: bool,
    pub yes: bool,
    pub json: bool,
    /// Remote side (injected in offline tests; `None` resolves and
    /// lists for real).
    pub target: Option<SyncTarget>,
    /// Put transport for `--force` (injected in offline tests).
    pub connect: Option<ConnectFn>,
}

/// `sync` command body.
pub(crate) async fn sync_cmd(opts: SyncOptions) -> Result<i32> {
    let SyncOptions {
        handle: handle_flag,
        root,
        pull,
        force,
        yes,
        json,
        target,
        connect,
    } = opts;
    let ws = Workspace::resolve(root.as_deref())?;

    let target = match target {
        Some(t) => t,
        None => {
            let input = match sync_handle(handle_flag, &ws) {
                Some(h) => h,
                None => {
                    eprintln!(
                        "error: no handle: pass --handle <handle> (or set `handle` in {} or {})",
                        ws.root.join("springleaf.toml").display(),
                        crate::workspace::xdg_config_toml_path()?.display()
                    );
                    return Ok(2);
                }
            };
            load_remote(&input).await?
        }
    };

    let locals = scan_posted(&ws)?;
    let rows = classify(locals, target.records);
    let report = SyncReport {
        handle: target.handle.clone(),
        did: target.did.clone(),
        pds: target.pds.clone(),
        workspace: ws.root.display().to_string(),
        rows,
    };

    // Report-only default; pull and force both act after it.
    if json {
        println!("{}", serde_json::to_string_pretty(&json_report(&report))?);
    } else {
        print!("{}", report_text(&report, pull));
    }

    if pull {
        let done = apply_pull(&ws, &report)?;
        if !done.is_empty() {
            let summary: Vec<String> = done
                .iter()
                .map(|(rkey, action)| {
                    format!(
                        "{rkey} ({})",
                        match action {
                            PullAction::Adopted => "adopted",
                            PullAction::Updated => "updated to remote",
                        }
                    )
                })
                .collect();
            eprintln!("pull: {} (never a remote write)", summary.join(", "));
        } else {
            eprintln!("pull: nothing to adopt");
        }
    }

    if force {
        let eligible = report.force_eligible();
        if eligible.is_empty() {
            eprintln!("force: nothing to re-put");
        } else if !yes {
            eprintln!(
                "refusing to re-put {} record(s) without --yes: re-run with `sync --force --yes` \
                 (requires ATPROTO_HANDLE and ATPROTO_APP_PASSWORD)",
                eligible.len()
            );
            return Ok(2);
        } else {
            let connect = connect.unwrap_or_else(|| Box::new(|expected| Box::pin(crate::publish::connect_from_env(expected))));
            let done = execute_force(&ws, &report, connect).await?;
            eprintln!(
                "force: re-put {} ({} unchanged, conflicts untouched)",
                done.len(),
                report.count(SyncStatus::Clean)
            );
        }
    }

    Ok(0)
}

/// Handle for sync: `--handle` > workspace toml > XDG config toml >
/// `$ATPROTO_HANDLE` (the env var because sync itself needs no session).
fn sync_handle(flag: Option<String>, ws: &Workspace) -> Option<String> {
    flag.or_else(|| ws.config().handle)
        .or_else(|| {
            crate::workspace::xdg_config_toml_path()
                .ok()
                .and_then(|p| crate::workspace::toml_value_at(&p, "handle"))
        })
        .or_else(|| std::env::var("ATPROTO_HANDLE").ok())
        .filter(|h| !h.is_empty())
}

/// Human report.
fn report_text(report: &SyncReport, pull: bool) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let who = report
        .handle
        .as_deref()
        .map(|h| format!("{h} ({})", report.did))
        .unwrap_or_else(|| report.did.clone());
    let _ = writeln!(
        out,
        "sync report (report-only{})",
        if pull { "; --pull acted" } else { " — no writes without --pull/--force" }
    );
    let _ = writeln!(out, "identity:   {who} @ {}", report.pds);
    let _ = writeln!(out, "collection: {COLLECTION}");
    let _ = writeln!(out, "workspace:  {}", report.workspace);
    let _ = writeln!(out);
    let status_width = 13;
    let rkey_width = report.rows.iter().map(|r| r.rkey.len()).max().unwrap_or(4).max(4);
    let _ = writeln!(out, "{:<status_width$} {:<rkey_width$} LOCAL     REMOTE-CID         NOTE", "STATUS", "RKEY");
    for row in &report.rows {
        let local = match &row.local {
            None => "-".to_string(),
            Some(l) => match &l.sidecar {
                Some(sidecar) if l.hash == sidecar.content_hash => "unchanged".to_string(),
                Some(_) => "changed".to_string(),
                None => "no-sidecar".to_string(),
            },
        };
        let cid = row.remote.as_ref().map(|r| r.cid.as_str()).unwrap_or("-");
        let _ = writeln!(
            out,
            "{:<status_width$} {:<rkey_width$} {:<9} {:<18} {}",
            row.status.as_str(),
            row.rkey,
            local,
            cid,
            row.status.note()
        );
    }
    let total = report.rows.len();
    let _ = write!(
        out,
        "\n{total} row(s): {} clean, {} local-newer, {} remote-drift, {} true-conflict, {} remote-orphan, {} local-missing, {} no-sidecar",
        report.count(SyncStatus::Clean),
        report.count(SyncStatus::LocalNewer),
        report.count(SyncStatus::RemoteDrift),
        report.count(SyncStatus::TrueConflict),
        report.count(SyncStatus::RemoteOrphan),
        report.count(SyncStatus::LocalMissing),
        report.count(SyncStatus::NoSidecar),
    );
    let orphans = report.count(SyncStatus::RemoteOrphan) + report.count(SyncStatus::RemoteDrift);
    if orphans > 0 {
        let _ = write!(out, "\nhint: `--pull` adopts {orphans} remote record(s) into posted/ (a local write)");
    }
    let eligible = report.force_eligible().len();
    if eligible > 0 {
        let _ = write!(out, "\nhint: `--force --yes` re-puts {eligible} record(s)");
    }
    out.push('\n');
    out
}

/// The `--json` schema (version 1).
#[derive(Serialize)]
struct JsonReport<'a> {
    version: u8,
    identity: JsonIdentity<'a>,
    workspace: &'a str,
    collection: &'a str,
    counts: JsonCounts,
    rows: Vec<JsonRow<'a>>,
}

#[derive(Serialize)]
struct JsonIdentity<'a> {
    handle: Option<&'a str>,
    did: &'a str,
    pds: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonCounts {
    clean: usize,
    local_newer: usize,
    remote_drift: usize,
    true_conflict: usize,
    remote_orphan: usize,
    local_missing: usize,
    no_sidecar: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonRow<'a> {
    status: SyncStatus,
    rkey: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_hash_changed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sidecar_cid: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_cid: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uri: Option<&'a str>,
    note: &'static str,
}

fn json_report(report: &SyncReport) -> JsonReport<'_> {
    JsonReport {
        version: 1,
        identity: JsonIdentity {
            handle: report.handle.as_deref(),
            did: &report.did,
            pds: &report.pds,
        },
        workspace: &report.workspace,
        collection: COLLECTION,
        counts: JsonCounts {
            clean: report.count(SyncStatus::Clean),
            local_newer: report.count(SyncStatus::LocalNewer),
            remote_drift: report.count(SyncStatus::RemoteDrift),
            true_conflict: report.count(SyncStatus::TrueConflict),
            remote_orphan: report.count(SyncStatus::RemoteOrphan),
            local_missing: report.count(SyncStatus::LocalMissing),
            no_sidecar: report.count(SyncStatus::NoSidecar),
        },
        rows: report
            .rows
            .iter()
            .map(|row| JsonRow {
                status: row.status,
                rkey: &row.rkey,
                local_hash_changed: row.local.as_ref().map(|l| match &l.sidecar {
                    Some(sidecar) => l.hash != sidecar.content_hash,
                    None => false,
                }),
                sidecar_cid: row.local.as_ref().and_then(|l| l.sidecar.as_ref()).map(|s| s.cid.as_str()),
                remote_cid: row.remote.as_ref().map(|r| r.cid.as_str()),
                uri: row
                    .remote
                    .as_ref()
                    .map(|r| r.uri.as_str())
                    .or_else(|| row.local.as_ref().and_then(|l| l.sidecar.as_ref()).map(|s| s.uri.as_str())),
                note: row.status.note(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::TempDir;
    use serde_json::json;

    const DID: &str = "did:plc:zjbq26wybii5ojoypkso2mso";
    const PUBLICATION: &str = "at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.publication/3molsbwkh522o";

    fn record_value(rkey: &str) -> Value {
        json!({
            "$type": "site.standard.document",
            "site": PUBLICATION,
            "title": format!("Record {rkey}"),
            "publishedAt": "2026-11-07T00:05:30.799Z",
            "path": format!("/{rkey}")
        })
    }

    /// How the sidecar's contentHash relates to the file just written.
    #[derive(Clone, Copy, PartialEq)]
    enum HashState {
        /// contentHash matches the file bytes (unchanged since put).
        Matching,
        /// contentHash is bogus — the file was edited since the put.
        EditedSincePut,
    }

    /// Write posted/<rkey>.json plus a sidecar in the given state
    /// (`remote_cid` None + Matching also covers the missing-remoteCid
    /// shape; `SidecarSpec::None` writes no sidecar at all).
    enum SidecarSpec {
        None,
        With { hash: HashState, remote_cid: Option<String> },
    }

    fn write_posted(ws: &Workspace, rkey: &str, spec: SidecarSpec) {
        std::fs::create_dir_all(ws.posted_dir()).unwrap();
        let path = ws.posted_dir().join(format!("{rkey}.json"));
        std::fs::write(&path, serde_json::to_string_pretty(&record_value(rkey)).unwrap()).unwrap();
        let SidecarSpec::With { hash, remote_cid } = spec else { return };
        let content_hash = match hash {
            HashState::Matching => sha256_hex(&std::fs::read(&path).unwrap()),
            HashState::EditedSincePut => "0".repeat(64),
        };
        std::fs::write(
            Sidecar::path_for(&ws.posted_dir(), rkey),
            serde_json::to_vec_pretty(&sidecar(rkey, content_hash, remote_cid)).unwrap(),
        )
        .unwrap();
    }

    fn matching(remote_cid: &str) -> SidecarSpec {
        SidecarSpec::With {
            hash: HashState::Matching,
            remote_cid: Some(remote_cid.into()),
        }
    }

    fn edited(remote_cid: &str) -> SidecarSpec {
        SidecarSpec::With {
            hash: HashState::EditedSincePut,
            remote_cid: Some(remote_cid.into()),
        }
    }

    fn sidecar(rkey: &str, content_hash: String, remote_cid: Option<String>) -> Sidecar {
        Sidecar {
            rkey: rkey.to_string(),
            uri: format!("at://{DID}/site.standard.document/{rkey}"),
            cid: "bafyreisidecar".into(),
            publication: PUBLICATION.into(),
            published_at: Some("2026-11-07T00:05:30.799Z".into()),
            last_put_at: Some("2026-11-07T01:00:00Z".into()),
            content_hash,
            remote_cid,
        }
    }

    fn remote(rkey: &str, cid: &str) -> RemoteRec {
        RemoteRec {
            rkey: rkey.to_string(),
            uri: format!("at://{DID}/site.standard.document/{rkey}"),
            cid: cid.to_string(),
            value: record_value(rkey),
        }
    }

    fn status_by_rkey(rows: &[SyncRow]) -> std::collections::HashMap<String, SyncStatus> {
        rows.iter().map(|r| (r.rkey.clone(), r.status)).collect()
    }

    fn report_over(ws: &Workspace, rows: Vec<SyncRow>) -> SyncReport {
        SyncReport {
            handle: Some("jauntywunderkind.bsky.social".into()),
            did: DID.into(),
            pds: "https://puffball.us-east.host.bsky.network".into(),
            workspace: ws.root.display().to_string(),
            rows,
        }
    }

    /// The full semantics table on a temp dir: all six rows plus the
    /// no-sidecar indeterminate case.
    #[test]
    fn semantics_table_all_rows() {
        let tmp = TempDir::new("sync-table");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        // clean: hash matches, remote cid matches sidecar's remoteCid
        write_posted(&ws, "clean-one", matching("bafyrei-clean"));
        // local-newer: file edited since the sidecar (hash differs), remote cid unchanged
        write_posted(&ws, "local-newer", edited("bafyrei-ln"));
        // remote-drift: hash unchanged, remote cid differs from sidecar's remoteCid
        write_posted(&ws, "remote-drift", matching("bafyrei-old"));
        // true conflict: both changed
        write_posted(&ws, "true-conflict", edited("bafyrei-old"));
        // local-missing: put once (sidecar), absent remotely
        write_posted(&ws, "local-missing", matching("bafyrei-was"));
        // no sidecar, remote present → indeterminate
        write_posted(&ws, "no-sidecar", SidecarSpec::None);
        let locals = scan_posted(&ws).unwrap();
        let remotes = vec![
            remote("clean-one", "bafyrei-clean"),
            remote("local-newer", "bafyrei-ln"),
            remote("remote-drift", "bafyrei-new"),
            remote("true-conflict", "bafyrei-new"),
            remote("no-sidecar", "bafyrei-whatever"),
            // local-missing absent remotely; remote-orphan has no local
            remote("remote-orphan", "bafyrei-orphan"),
        ];
        let statuses = status_by_rkey(&classify(locals, remotes));
        assert_eq!(statuses["clean-one"], SyncStatus::Clean);
        assert_eq!(statuses["local-newer"], SyncStatus::LocalNewer);
        assert_eq!(statuses["remote-drift"], SyncStatus::RemoteDrift);
        assert_eq!(statuses["true-conflict"], SyncStatus::TrueConflict);
        assert_eq!(statuses["local-missing"], SyncStatus::LocalMissing);
        assert_eq!(statuses["remote-orphan"], SyncStatus::RemoteOrphan);
        assert_eq!(statuses["no-sidecar"], SyncStatus::NoSidecar);
    }

    /// A sidecar without `remoteCid` cannot be compared against the
    /// remote → drift path (report, resolvable), never a silent clean.
    #[test]
    fn sidecar_without_remote_cid_is_not_clean() {
        let tmp = TempDir::new("sync-no-remote-cid");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        write_posted(
            &ws,
            "stale-sidecar",
            SidecarSpec::With {
                hash: HashState::Matching,
                remote_cid: None,
            },
        );
        let locals = scan_posted(&ws).unwrap();
        let statuses = status_by_rkey(&classify(locals, vec![remote("stale-sidecar", "bafyrei-x")]));
        assert_eq!(statuses["stale-sidecar"], SyncStatus::RemoteDrift);
    }

    #[test]
    fn empty_workspace_reports_everything_remote() {
        let tmp = TempDir::new("sync-empty");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        let locals = scan_posted(&ws).unwrap();
        assert!(locals.is_empty());
        let statuses = status_by_rkey(&classify(locals, vec![remote("r1", "c1"), remote("r2", "c2")]));
        assert_eq!(statuses.len(), 2);
        assert!(statuses.values().all(|s| *s == SyncStatus::RemoteOrphan));
    }

    /// `--pull`: remote orphans adopted into posted/ (+ sidecar, no
    /// lastPutAt); drift rows updated to the remote bytes; conflicts and
    /// clean rows untouched.
    #[test]
    fn pull_adopts_orphans_and_updates_drift_only() {
        let tmp = TempDir::new("sync-pull");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        write_posted(&ws, "clean-one", matching("bafyrei-clean"));
        write_posted(&ws, "remote-drift", matching("bafyrei-old"));
        write_posted(&ws, "true-conflict", edited("bafyrei-old"));
        let before_conflict = std::fs::read(ws.posted_dir().join("true-conflict.json")).unwrap();
        let before_drift = std::fs::read(ws.posted_dir().join("remote-drift.json")).unwrap();

        let locals = scan_posted(&ws).unwrap();
        let remotes = vec![
            remote("clean-one", "bafyrei-clean"),
            remote("remote-drift", "bafyrei-new-drift"),
            remote("true-conflict", "bafyrei-new-conflict"),
            remote("remote-orphan", "bafyrei-orphan"),
        ];
        let report = report_over(&ws, classify(locals, remotes));
        let done = apply_pull(&ws, &report).unwrap();
        assert_eq!(
            done,
            vec![
                ("remote-drift".to_string(), PullAction::Updated),
                ("remote-orphan".to_string(), PullAction::Adopted)
            ]
        );

        // Orphan adopted: pure record file + sidecar with null lastPutAt.
        let adopted: Value = serde_json::from_str(&std::fs::read_to_string(ws.posted_dir().join("remote-orphan.json")).unwrap()).unwrap();
        assert_eq!(adopted, record_value("remote-orphan"));
        let sidecar: Sidecar = serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), "remote-orphan")).unwrap()).unwrap();
        assert_eq!(sidecar.rkey, "remote-orphan");
        assert_eq!(sidecar.remote_cid.as_deref(), Some("bafyrei-orphan"));
        assert_eq!(sidecar.last_put_at, None, "adopted bytes were never put by us");
        assert_eq!(
            sidecar.content_hash,
            sha256_hex(std::fs::read(ws.posted_dir().join("remote-orphan.json")).unwrap().as_slice())
        );

        // Drift updated: local file now equals the remote record.
        let after_drift = std::fs::read(ws.posted_dir().join("remote-drift.json")).unwrap();
        assert_ne!(before_drift, after_drift, "pull adopts the remote bytes for drift rows");
        let drift_value: Value = serde_json::from_slice(&after_drift).unwrap();
        assert_eq!(drift_value, record_value("remote-drift"));
        let sidecar: Sidecar = serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), "remote-drift")).unwrap()).unwrap();
        assert_eq!(sidecar.remote_cid.as_deref(), Some("bafyrei-new-drift"));

        // Conflicts and clean rows untouched.
        assert_eq!(before_conflict, std::fs::read(ws.posted_dir().join("true-conflict.json")).unwrap());
        assert!(Sidecar::path_for(&ws.posted_dir(), "clean-one").is_file());
    }

    #[tokio::test]
    async fn force_refuses_without_yes() {
        let tmp = TempDir::new("sync-force-refuse");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        write_posted(&ws, "local-newer", edited("bafyrei-ln"));
        let target = SyncTarget {
            handle: None,
            did: DID.into(),
            pds: "https://pds.example".into(),
            records: vec![remote("local-newer", "bafyrei-ln")],
        };
        // No connect injection: the refusal must happen before any
        // transport would be opened.
        let code = sync_cmd(SyncOptions {
            handle: None,
            root: Some(ws.root.clone()),
            pull: false,
            force: true,
            yes: false,
            json: false,
            target: Some(target),
            connect: None,
        })
        .await
        .unwrap();
        assert_eq!(code, 2, "sync --force without --yes refuses with exit 2");
    }

    #[tokio::test]
    async fn force_with_yes_reputs_eligible_rows_via_mocked_transport() {
        let tmp = TempDir::new("sync-force-run");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        write_posted(&ws, "local-newer", edited("bafyrei-ln"));
        write_posted(&ws, "local-missing", matching("bafyrei-was"));
        write_posted(&ws, "true-conflict", edited("bafyrei-old"));
        write_posted(&ws, "clean-one", matching("bafyrei-clean"));
        let target = SyncTarget {
            handle: None,
            did: DID.into(),
            pds: "https://pds.example".into(),
            records: vec![
                remote("local-newer", "bafyrei-ln"),
                remote("local-missing", "bafyrei-unexpected"),
                remote("true-conflict", "bafyrei-new"),
                remote("clean-one", "bafyrei-clean"),
            ],
        };
        let connect: ConnectFn = Box::new(|expected| {
            Box::pin(async move {
                Ok(crate::publish::Connected {
                    did: expected,
                    put: Box::new(|rkey, _record| {
                        Box::pin(async move {
                            Ok(crate::publish::PutOutcome {
                                uri: format!("at://{DID}/site.standard.document/{rkey}"),
                                cid: format!("bafyreiforce-{rkey}"),
                            })
                        })
                    }),
                })
            })
        });
        let code = sync_cmd(SyncOptions {
            handle: None,
            root: Some(ws.root.clone()),
            pull: false,
            force: true,
            yes: true,
            json: false,
            target: Some(target),
            connect: Some(connect),
        })
        .await
        .unwrap();
        assert_eq!(code, 0);
        // local-newer and local-missing re-put; clean untouched;
        // true-conflict untouched (sidecar still has the old cid).
        for (rkey, want_cid) in [("local-newer", "bafyreiforce-local-newer"), ("local-missing", "bafyreiforce-local-missing")] {
            let sidecar: Sidecar = serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), rkey)).unwrap()).unwrap();
            assert_eq!(sidecar.cid, want_cid, "{rkey} re-put");
            assert_eq!(sidecar.remote_cid.as_deref(), Some(want_cid));
            assert!(sidecar.last_put_at.is_some());
        }
        let clean: Sidecar = serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), "clean-one")).unwrap()).unwrap();
        assert_eq!(clean.cid, "bafyreisidecar", "clean rows are not re-put");
        let conflict: Sidecar = serde_json::from_str(&std::fs::read_to_string(Sidecar::path_for(&ws.posted_dir(), "true-conflict")).unwrap()).unwrap();
        assert_eq!(conflict.cid, "bafyreisidecar", "true conflicts are never force-resolved");
    }

    #[test]
    fn report_text_and_json_shapes() {
        let tmp = TempDir::new("sync-report");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        write_posted(&ws, "clean-one", matching("bafyrei-clean"));
        let locals = scan_posted(&ws).unwrap();
        let report = report_over(&ws, classify(locals, vec![remote("clean-one", "bafyrei-clean"), remote("orphan", "bafyrei-o")]));
        let text = report_text(&report, false);
        assert!(text.contains("sync report (report-only"), "{text}");
        assert!(text.contains("jauntywunderkind.bsky.social"), "{text}");
        assert!(text.contains("site.standard.document"), "{text}");
        assert!(text.contains("clean"), "{text}");
        assert!(text.contains("remote-orphan orphan"), "{text}");
        assert!(
            text.contains("2 row(s): 1 clean, 0 local-newer, 0 remote-drift, 0 true-conflict, 1 remote-orphan"),
            "{text}"
        );
        assert!(text.contains("`--pull` adopts 1 remote record(s)"), "{text}");
        let json = serde_json::to_value(json_report(&report)).unwrap();
        assert_eq!(json["version"], json!(1));
        assert_eq!(json["counts"]["clean"], json!(1));
        assert_eq!(json["counts"]["remoteOrphan"], json!(1));
        assert_eq!(json["rows"][0]["status"], json!("clean"));
        assert_eq!(json["rows"][0]["localHashChanged"], json!(false));
        assert_eq!(json["rows"][1]["remoteCid"], json!("bafyrei-o"));
    }
}
