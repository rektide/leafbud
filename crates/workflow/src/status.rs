//! Read-only status of selected current files and their bound draft editors.
//! Observation never accepts correspondence or writes receipts, even on convergence.
use crate::{
    RemoteReader, config,
    editor_state::{self, Change},
    inventory,
};
use anyhow::{Context, Result};
use leafbud_document::EDITOR_PROFILE;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Request {
    /// Empty means the managed workspace inventory, not remote discovery.
    pub files: Vec<PathBuf>,
}

#[derive(Serialize)]
pub struct Report {
    pub version: u8,
    pub workspace: PathBuf,
    pub offline: bool,
    pub selected: usize,
    pub failed: usize,
    pub ignored_unmanaged: usize,
    pub names: &'static str,
    pub release: &'static str,
    pub items: Vec<Item>,
}

#[derive(Serialize)]
pub struct Item {
    pub path: PathBuf,
    pub outcome: &'static str,
    pub detail: &'static str,
    pub local: Change,
    pub remote: RemoteEvidence,
    pub receipt_profile: Option<String>,
    pub baseline: &'static str,
    pub endpoints_equal: Option<bool>,
    pub document_id: Option<uuid::Uuid>,
    /// Blocks/main page still needing durable source keys (never assigned by
    /// status; tidy --keys or executed push assign them).
    pub needs_preparation: usize,
    /// Local publication/delivery/custom intent is not this editor edge.
    pub local_only_metadata: &'static str,
}

#[derive(Serialize)]
pub struct RemoteEvidence {
    pub observed_profile: Option<&'static str>,
    /// `current` means fetched in this invocation, not a transaction or CAS.
    pub freshness: &'static str,
    pub change: Change,
    pub observed_hash: Option<String>,
}

/// No RemoteReader/credentials are needed for offline status.
pub async fn inspect(context: &config::Context, request: Request, remote: Option<&RemoteReader>) -> Result<Report> {
    let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
    let mut inventory = inventory::scan(&root, None)?;
    let mut paths = Vec::new();
    if request.files.is_empty() {
        paths = inventory.paths.clone();
    } else {
        for file in request.files {
            let joined = if file.is_absolute() { file } else { root.join(file) };
            // A missing operand is a per-file outcome, not a batch-wide abort.
            let path = std::fs::canonicalize(&joined).unwrap_or(joined);
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    inventory.include_operands(&paths);
    let mut report = Report {
        version: 1,
        workspace: root,
        offline: remote.is_none(),
        selected: paths.len(),
        failed: 0,
        ignored_unmanaged: inventory.unmanaged,
        names: "not-tracked",
        release: "unknown-not-implemented",
        items: vec![],
    };
    for path in paths {
        let item = inspect_file(&path, &inventory, remote).await;
        if matches!(item.outcome, "invalid" | "ambiguous" | "failed" | "local-missing" | "raced") {
            report.failed += 1;
        }
        report.items.push(item);
    }
    Ok(report)
}

async fn inspect_file(path: &Path, inventory: &inventory::Inventory, remote: Option<&RemoteReader>) -> Item {
    let mut item = Item {
        path: path.to_owned(),
        outcome: "unknown",
        detail: "remote state is unknown; offline status never implies clean agreement",
        local: Change::Unknown,
        remote: RemoteEvidence {
            observed_profile: None,
            freshness: if remote.is_none() { "offline" } else { "not-requested" },
            change: Change::Unknown,
            observed_hash: None,
        },
        receipt_profile: None,
        baseline: "unknown",
        endpoints_equal: None,
        document_id: None,
        needs_preparation: 0,
        local_only_metadata: "not-compared-to-editor",
    };
    let original = match std::fs::read(path) {
        Ok(original) => original,
        Err(error) => {
            item.outcome = if error.kind() == std::io::ErrorKind::NotFound {
                "local-missing"
            } else {
                "failed"
            };
            item.detail = "cannot read local operand; check its path and permissions";
            return item;
        }
    };
    let Ok(source) = std::str::from_utf8(&original) else {
        item.outcome = "invalid";
        item.detail = "local source is not UTF-8";
        return item;
    };
    let header = match leafbud_mdx::header(source) {
        Ok(header) => header,
        Err(_) if !source.contains("@leafbud") && !source.contains("@springleaf") => {
            item.outcome = "untracked";
            item.detail = "not a managed document; import establishes a native binding";
            return item;
        }
        Err(_) => {
            item.outcome = "invalid";
            item.detail = "invalid managed frontmatter; use check to diagnose the local file";
            return item;
        }
    };
    if header.get("@springleaf").is_some() {
        item.outcome = "unsupported";
        item.detail =
            "legacy springleaf format/namespace; explicit namespace conversion required before interpreting editor evidence; status never migrates files";
        return item;
    }
    if header.get("@leafbud").is_none() {
        item.outcome = "untracked";
        item.detail = "not a managed document; import establishes a native binding";
        return item;
    }
    let parsed = match leafbud_mdx::parse_with_spans(source) {
        Ok(parsed) => parsed,
        Err(_) => {
            item.outcome = "invalid";
            item.detail = "invalid or unsupported local content/owned state; use check; nothing was omitted or rewritten";
            return item;
        }
    };
    let existing = &parsed.document;
    item.document_id = Some(existing.state.id);
    item.needs_preparation = parsed.unkeyed().len() + usize::from(parsed.main_page_provisional());
    item.local = editor_state::local_change(existing);
    if item.needs_preparation > 0 && item.local == Change::Unchanged {
        // New content is a real local edit even while its keys are not
        // persisted yet; report it without claiming durable identity.
        item.local = Change::Changed;
    }
    // Duplicate identities stay explicit in every reachable state, unbound
    // and newly bound included: resolving copies precedes any other verdict.
    if inventory.duplicates.contains(path) {
        item.outcome = "ambiguous";
        item.detail = "duplicate local document ID or native binding; resolve the copies before managed operations";
        return item;
    }
    // Typed optional state: an explicit `sync: null` deserializes to the
    // same absent receipts as a missing field.
    if existing.state.binding.is_none() {
        item.outcome = "unbound";
        item.baseline = "none";
        item.detail = if item.needs_preparation > 0 {
            "local draft; no native binding yet, and some content still needs source keys (tidy --keys)"
        } else {
            "local draft; no native binding yet (push establishes one), so remote editor state does not exist"
        };
        return item;
    }
    match &existing.state.sync {
        None => {
            item.outcome = "bound-no-receipt";
            item.baseline = "none";
            item.detail = "bound but no verified agreement yet; receipts settle only after a verified save/pull";
            return item;
        }
        // Defensive: the typed parse already rejects unknown profiles and
        // lifecycles, so this branch keeps the diagnostic explicit if the
        // two validations ever diverge.
        Some(sync) if sync.profile != EDITOR_PROFILE || existing.state.state != "draft" => {
            item.outcome = "unsupported";
            item.detail = "unsupported receipt profile or lifecycle; no historical editor agreement inferred";
            return item;
        }
        Some(_) => {
            item.receipt_profile = existing.state.sync.as_ref().map(|sync| sync.profile.clone());
        }
    }
    item.baseline = editor_state::baseline(existing);
    let binding = existing.state.binding.as_ref().expect("binding present");
    if binding.root.is_empty() || binding.entity_set.is_empty() || binding.entry_fact.is_empty() {
        item.outcome = "unsupported";
        item.detail = "incoherent binding; identity must be resolved explicitly";
        return item;
    }
    let Some(remote) = remote else {
        if item.local == Change::Changed {
            item.outcome = "local-changed";
        }
        return item;
    };
    if binding.base_url != remote.client.base_url() {
        item.outcome = "unsupported";
        item.detail = "bound to another origin; select its --base-url explicitly (defaults never retarget a binding)";
        return item;
    }
    match remote.observe_editor(existing).await {
        Ok(observed) => {
            item.outcome = observed.agreement.state;
            item.detail = match item.outcome {
                "clean" => "scoped receipts and current endpoints agree",
                "local-changed" => "local editor input changed; remote witness matches its receipt",
                "remote-changed" => "remote owned state changed; local editor input matches its receipt (including changes outside the visible projection)",
                "both-changed" => "both editor inputs changed and current endpoints differ; status chooses neither side",
                "converged" => "current supported endpoints agree; receipts remain untouched (legacy agreement can be settled by explicit pull)",
                _ => "differing endpoints lack a verified scoped baseline; no direction or historical body inferred",
            };
            item.local = observed.agreement.local;
            item.endpoints_equal = Some(observed.agreement.endpoints_equal);
            item.remote.freshness = "current";
            item.remote.change = observed.agreement.remote;
            item.remote.observed_hash = observed.merged.state.sync.as_ref().map(|sync| sync.remote_hash.clone());
            item.remote.observed_profile = Some(EDITOR_PROFILE);
        }
        Err(error) => {
            item.outcome = error.kind;
            item.remote.freshness = if error.remote_hash.is_some() {
                "current"
            } else if error.kind == "missing" {
                "checked-missing"
            } else if error.kind == "unsupported" {
                "metadata-only"
            } else {
                "unavailable"
            };
            item.detail = match error.kind {
                "missing" => "native object is missing or inaccessible; this does not prove deletion; local file remains",
                "identity-changed" => "remote identity, rights, placement or publication association differs; no rebinding inferred",
                "unsupported" => "remote lifecycle/content cannot be represented by this draft profile; conservative evidence retained when available",
                _ => "remote read failed; freshness and change direction are unknown; no credentials or remote bodies included",
            };
            if let Some(hash) = &error.remote_hash {
                item.remote.change = editor_state::remote_change(existing, hash);
            }
            item.remote.observed_hash = error.remote_hash;
            if item.remote.observed_hash.is_some() {
                item.remote.observed_profile = Some(EDITOR_PROFILE);
            }
        }
    }
    // Read-only does not mean race-free: never label an old local read as the
    // current file after a user saved during remote acquisition.
    if std::fs::read(path).ok().as_deref() != Some(original.as_slice()) {
        item.outcome = "raced";
        item.detail = "local file changed during observation; rerun status; no files were written";
        item.local = Change::Unknown;
        item.endpoints_equal = None;
    }
    item
}

#[cfg(test)]
mod tests;
