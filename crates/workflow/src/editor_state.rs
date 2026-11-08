//! Current endpoint observation and scoped agreement, shared by status and pull.
//! No change tree, historical base, mutation plan or automatic receipt writes.
use crate::{RemoteReader, import};
use anyhow::{Result, ensure};
use leafspring_document::{Block, BlockKind, Document, EDITOR_OWNED_EXTRA, EDITOR_PROFILE, Node, Page, State, Sync};
use leafspring_web::{LeafletData, LeafletState};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Change {
    Unchanged,
    Changed,
    Unknown,
}

impl Change {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::Changed => "changed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Agreement {
    pub state: &'static str,
    pub baseline: &'static str,
    pub local: Change,
    pub remote: Change,
    pub endpoints_equal: bool,
}

pub(crate) fn local_change(document: &Document) -> Change {
    if !valid_hash(&document.state.sync.local_hash) {
        return Change::Unknown;
    }
    if document.state.sync.profile == EDITOR_PROFILE {
        if import::hash(&document.editor_value()) == document.state.sync.local_hash {
            Change::Unchanged
        } else {
            Change::Changed
        }
    } else {
        // A matching legacy whole-document hash proves no local editor change.
        // A mismatch cannot distinguish local-only intent from an editor edit.
        if import::hash(&document.semantic_value()) == document.state.sync.local_hash {
            Change::Unchanged
        } else {
            Change::Unknown
        }
    }
}

fn valid_hash(hash: &str) -> bool {
    hash.strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

pub(crate) fn baseline(document: &Document) -> &'static str {
    if !valid_hash(&document.state.sync.local_hash) || !valid_hash(&document.state.sync.remote_hash) {
        "invalid-receipts"
    } else if document.state.sync.profile == EDITOR_PROFILE {
        "scoped"
    } else {
        "legacy-unscoped"
    }
}

pub(crate) fn classify(existing: &Document, merged: &Document, observed_remote_hash: &str) -> Agreement {
    let endpoints_equal = existing.editor_value() == merged.editor_value();
    let local = local_change(existing);
    let remote = remote_change(existing, observed_remote_hash);
    let scoped = existing.state.sync.profile == EDITOR_PROFILE;
    let state = if endpoints_equal {
        if scoped && local == Change::Unchanged && remote != Change::Unknown {
            if remote == Change::Unchanged { "clean" } else { "remote-changed" }
        } else {
            "converged"
        }
    } else if !scoped {
        "unverified"
    } else {
        match (local, remote) {
            (Change::Unchanged, Change::Changed) => "remote-changed",
            (Change::Changed, Change::Unchanged) => "local-changed",
            (Change::Changed, Change::Changed) => "both-changed",
            // Claimed receipts disagree with current correspondence; never
            // turn that inconsistency into permission to replace content.
            _ => "unverified",
        }
    };
    Agreement {
        state,
        baseline: baseline(existing),
        local,
        remote,
        endpoints_equal,
    }
}

pub(crate) fn remote_change(existing: &Document, observed_hash: &str) -> Change {
    if !valid_hash(&existing.state.sync.remote_hash) {
        Change::Unknown
    } else if existing.state.sync.remote_hash == observed_hash {
        Change::Unchanged
    } else {
        Change::Changed
    }
}

pub(crate) struct Observed {
    pub data: LeafletData,
    pub merged: Document,
    pub agreement: Agreement,
}

pub(crate) struct ObservationError {
    pub kind: &'static str,
    pub message: String,
    pub remote_hash: Option<String>,
}

impl RemoteReader {
    pub(crate) async fn observe_editor(&self, existing: &Document) -> Result<Observed, ObservationError> {
        let error = |kind, message: String| ObservationError {
            kind,
            message,
            remote_hash: None,
        };
        let token = existing.state.binding.token;
        let data = self.client.get(token).await.map_err(|e| {
            error(
                if matches!(e, leafspring_web::Error::NotFound) { "missing" } else { "failed" },
                if matches!(e, leafspring_web::Error::NotFound) {
                    "remote object missing or inaccessible (token not found); local file preserved".into()
                } else {
                    format!("fetching remote metadata: {e}")
                },
            )
        })?;
        if data.observed_state() != LeafletState::Draft {
            return Err(error(
                "unsupported",
                "remote lifecycle changed (published); this profile only observes draft editors".into(),
            ));
        }
        let snapshot = self
            .client
            .pull(token)
            .await
            .map_err(|e| error("failed", format!("fetching remote editor content: {e}")))?;
        // Acquire the conservative witness before mapping. Unsupported content
        // must not disappear simply because it cannot become a Document.
        let hash = import::remote_hash(&data, &snapshot, &existing.state.sync.profile);
        let conservative_hash = import::remote_hash(&data, &snapshot, EDITOR_PROFILE);
        let with_hash = |kind, message| ObservationError {
            kind,
            message,
            remote_hash: Some(conservative_hash.clone()),
        };
        let fresh = import::build(self.client.base_url(), &data, &snapshot).map_err(|e| {
            with_hash(
                "unsupported",
                format!("remote content is unsupported for the refresh profile; local file preserved: {e:#}"),
            )
        })?;
        same_identity(existing, &fresh).map_err(|e| with_hash("identity-changed", format!("{e:#}")))?;
        let mut merged = adopt(existing, &fresh);
        let agreement = classify(existing, &merged, &hash);
        if agreement.endpoints_equal {
            // Non-entry page definition ordering is source layout, not editor
            // content. Settling witnesses must keep its original body bytes.
            merged.pages = existing.pages.clone();
            if merged.editor_value() != existing.editor_value() {
                return Err(with_hash(
                    "unsupported",
                    "source association correspondence could not be retained safely".into(),
                ));
            }
        }
        Ok(Observed { data, merged, agreement })
    }
}

fn same_identity(existing: &Document, fresh: &Document) -> Result<()> {
    let a = &existing.state.binding;
    let b = &fresh.state.binding;
    for (equal, name) in [
        (existing.state.id == fresh.state.id, "document identity"),
        (a.base_url == b.base_url && a.token == b.token, "origin/token"),
        (a.root == b.root, "root entity"),
        (a.entity_set == b.entity_set, "entity set (rights)"),
        (a.entry_fact == b.entry_fact, "entry placement"),
        (a.publication == b.publication, "publication association"),
    ] {
        ensure!(equal, "remote {name} changed; identity/binding decision required (file preserved)");
    }
    Ok(())
}

fn reconcile_keys(existing: &Document, fresh: &Document) -> BTreeMap<String, String> {
    let mut rename = BTreeMap::new();
    let mut claimed = BTreeSet::new();
    for (key, node) in &fresh.state.nodes {
        let same_entity: Vec<(&String, &Node)> = existing.state.nodes.iter().filter(|(_, n)| n.entity == node.entity).collect();
        let memberships: BTreeSet<_> = node.memberships.keys().collect();
        let exact: Vec<&String> = same_entity
            .iter()
            .filter(|(_, n)| n.memberships.keys().collect::<BTreeSet<_>>() == memberships)
            .map(|(k, _)| *k)
            .collect();
        let chosen = if exact.len() == 1 {
            Some(exact[0].clone())
        } else if same_entity.len() == 1 {
            Some(same_entity[0].0.clone())
        } else {
            None
        };
        if let Some(chosen) = chosen
            && claimed.insert(chosen.clone())
        {
            rename.insert(key.clone(), chosen);
        }
    }
    // Reserve every matched author label before naming unmatched fresh nodes.
    // A newly observed deterministic label must not steal a manually renamed
    // existing object's spelling or make collect() silently drop a node row.
    for key in fresh.state.nodes.keys().filter(|k| !rename.contains_key(*k)).cloned().collect::<Vec<_>>() {
        let mut chosen = key.clone();
        let mut suffix = 2;
        while !claimed.insert(chosen.clone()) {
            chosen = format!("{key}-imported-{suffix}");
            suffix += 1;
        }
        rename.insert(key, chosen);
    }
    rename
}

/// Remote-owned content in existing source spelling. Local-only intent is
/// retained, not merged into the editor fingerprint.
fn adopt(existing: &Document, fresh: &Document) -> Document {
    let rename = reconcile_keys(existing, fresh);
    let key = |k: &str| rename.get(k).cloned().unwrap_or_else(|| k.to_owned());
    let block = |block: &Block| Block {
        key: key(&block.key),
        kind: match &block.kind {
            BlockKind::PageLink { page } => BlockKind::PageLink { page: key(page) },
            other => other.clone(),
        },
    };
    let nodes = fresh.state.nodes.iter().map(|(k, node)| (key(k), node.clone())).collect();
    let pages = fresh
        .pages
        .iter()
        .map(|page| Page {
            key: key(&page.key),
            blocks: page.blocks.iter().map(block).collect(),
        })
        .collect();
    let mut extra = fresh.extra.clone();
    for (name, value) in &existing.extra {
        if !EDITOR_OWNED_EXTRA.contains(&name.as_str()) && !extra.contains_key(name) {
            extra.insert(name.clone(), value.clone());
        }
    }
    let mut document = Document {
        format: fresh.format.clone(),
        title: fresh.title.clone(),
        description: fresh.description.clone(),
        tags: fresh.tags.clone(),
        publication: fresh.publication.clone(),
        theme: fresh.theme.clone(),
        extra,
        pages,
        state: State {
            id: existing.state.id,
            state: existing.state.state.clone(),
            binding: existing.state.binding.clone(),
            nodes,
            sync: Sync {
                profile: EDITOR_PROFILE.into(),
                local_hash: String::new(),
                remote_hash: fresh.state.sync.remote_hash.clone(),
            },
        },
    };
    document.state.sync.local_hash = import::hash(&document.editor_value());
    document
}

#[cfg(test)]
mod tests;
