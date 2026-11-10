//! Read-only local inventory. Header evidence survives invalid bodies and
//! unknown profiles; partial header evidence is never claimed as validity.
use crate::{LeafletState, config, inventory};
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Request {
    pub state: Option<LeafletState>,
}

#[derive(Serialize)]
pub struct Report {
    pub version: u8,
    pub workspace: PathBuf,
    pub examined: usize,
    pub ignored_unmanaged: usize,
    pub filtered: usize,
    pub failed: usize,
    pub items: Vec<Item>,
}

#[derive(Serialize)]
pub struct Item {
    pub path: PathBuf,
    pub names: Option<crate::names::Evidence>,
    pub outcome: &'static str,
    /// Parsed current supported document, not merely a readable header.
    pub valid: bool,
    pub title: Option<String>,
    pub lifecycle: Option<String>,
    pub format: Option<String>,
    pub receipt_profile: Option<String>,
    pub document_id: Option<uuid::Uuid>,
    /// Header-declared binding evidence; may be incomplete on invalid files.
    pub binding: Option<Binding>,
    pub editing_locator: Option<EditingLocator>,
    pub pending: bool,
    pub pending_kind: Option<String>,
    pub pending_phase: Option<String>,
    pub last_resolution: bool,
    pub resolution_direction: Option<String>,
    pub duplicate_id: bool,
    pub duplicate_binding: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Serialize)]
pub struct Binding {
    pub base_url: Option<String>,
    pub token: Option<String>,
    pub root: Option<String>,
    pub entity_set: Option<String>,
    pub entry_fact: Option<String>,
    pub uri: Option<String>,
}

#[derive(Serialize)]
pub struct EditingLocator {
    pub base_url: String,
    pub token: uuid::Uuid,
    /// A located mint is evidence, not a confirmed binding.
    pub evidence: &'static str,
}

fn text(value: &Value, field: &str) -> Option<String> {
    value[field].as_str().map(str::to_owned)
}

pub fn inspect(context: &config::Context, request: Request) -> Result<Report> {
    let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
    // All identities, origins and lifecycles participate BEFORE filtering.
    let inventory = inventory::scan(&root, None)?;
    let mut report = Report {
        version: 2,
        workspace: root,
        examined: inventory.paths.len(),
        ignored_unmanaged: inventory.unmanaged,
        filtered: 0,
        failed: 0,
        items: vec![],
    };
    for path in &inventory.paths {
        let item = inspect_file(&report.workspace, path, &inventory);
        let state = request.state.map(|s| match s {
            LeafletState::Draft => "draft",
            LeafletState::Published => "published",
        });
        // Broken/unsupported/ambiguous files remain visible even when their
        // readable state differs, so a filter cannot hide an incomplete scan.
        if item.outcome == "unbound" || item.outcome == "bound" {
            if state.is_some_and(|s| item.lifecycle.as_deref() != Some(s)) {
                report.filtered += 1;
                continue;
            }
        } else {
            report.failed += 1;
        }
        report.items.push(item);
    }
    Ok(report)
}

fn inspect_file(root: &Path, path: &Path, inventory: &inventory::Inventory) -> Item {
    let mut item = Item {
        path: path.into(),
        names: None,
        outcome: "invalid",
        valid: false,
        title: None,
        lifecycle: None,
        format: None,
        receipt_profile: None,
        document_id: None,
        binding: None,
        editing_locator: None,
        pending: false,
        pending_kind: None,
        pending_phase: None,
        last_resolution: false,
        resolution_direction: None,
        duplicate_id: false,
        duplicate_binding: false,
        diagnostics: vec![],
    };
    let original = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            item.outcome = "failed";
            item.diagnostics.push(format!("cannot read local file: {e}"));
            return item;
        }
    };
    let source = match std::str::from_utf8(&original) {
        Ok(s) => s,
        Err(_) => {
            item.diagnostics.push("managed candidate is not UTF-8".into());
            return item;
        }
    };
    let header = match leafbud_mdx::header(source) {
        Ok(h) => h,
        Err(e) => {
            item.diagnostics.push(format!("unreadable managed header: {e:#}"));
            return item;
        }
    };
    item.title = text(&header, "title");
    item.format = text(&header, "format");
    let state = &header["@leafbud"];
    item.lifecycle = text(state, "state");
    item.document_id = text(state, "id").and_then(|s| s.parse().ok());
    item.receipt_profile = text(&state["sync"], "profile");
    let b = &state["binding"];
    item.editing_locator = inventory::native_locator(state).map(|(base, token)| EditingLocator {
        base_url: base.into(),
        token,
        evidence: if b.is_null() { "pending-located" } else { "header-binding" },
    });
    if !b.is_null() {
        item.binding = Some(Binding {
            base_url: text(b, "baseUrl"),
            token: text(b, "token"),
            root: text(b, "root"),
            entity_set: text(b, "entitySet"),
            entry_fact: text(b, "entryFact"),
            uri: text(b, "uri"),
        });
    }
    item.pending = !state["pending"].is_null();
    item.pending_kind = text(&state["pending"]["effect"], "kind");
    item.pending_phase = text(&state["pending"]["effect"]["stage"], "phase");
    item.last_resolution = !state["lastResolution"].is_null();
    item.resolution_direction = text(&state["lastResolution"], "direction");
    item.duplicate_id = inventory.ids.values().any(|paths| paths.len() > 1 && paths.iter().any(|p| p == path));
    item.duplicate_binding = inventory
        .index
        .values()
        .chain(inventory.public.values())
        .any(|paths| paths.len() > 1 && paths.iter().any(|p| p == path));
    let unsupported = header.get("@springleaf").is_some()
        || item.format.as_deref() != Some(leafbud_document::FORMAT)
        || item.lifecycle.as_deref().is_some_and(|s| !matches!(s, "draft" | "published"))
        || item.receipt_profile.as_deref().is_some_and(|p| p != leafbud_document::EDITOR_PROFILE);
    if unsupported {
        item.outcome = "unsupported";
        item.diagnostics
            .push("unsupported format, lifecycle, namespace or receipt profile; header evidence only, no migration or release adoption".into());
    } else {
        match leafbud_mdx::parse(source) {
            Ok(doc) => {
                item.names = Some(crate::names::inspect(root, path, &doc));
                item.valid = true;
                item.outcome = if doc.state.binding.is_some() { "bound" } else { "unbound" };
            }
            Err(e) => item.diagnostics.push(format!("invalid/unsupported body or owned state: {e:#}")),
        }
    }
    if item.duplicate_id || item.duplicate_binding {
        item.outcome = "ambiguous";
        item.diagnostics
            .push("duplicate local UUID, native binding or public URI in workspace (including files outside state filter)".into());
    }
    if std::fs::read(path).ok().as_deref() != Some(original.as_slice()) {
        item.outcome = "raced";
        item.valid = false;
        item.names = None;
        item.diagnostics.push("source changed while listing; rerun for current evidence".into());
    }
    item
}

#[cfg(test)]
mod tests;
