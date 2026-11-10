//! Current endpoint observation and scoped agreement, shared by status and pull.
//! No change tree, historical base, mutation plan or automatic receipt writes.
use crate::{RemoteReader, import};
use anyhow::{Result, ensure};
use leafbud_document::{Block, BlockKind, Document, EDITOR_OWNED_EXTRA, EDITOR_PROFILE, Occurrence, Page, State, Sync};
use leafbud_web::LeafletData;
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
    let Some(sync) = &document.state.sync else {
        return Change::Unknown;
    };
    if !valid_hash(&sync.local_hash) {
        return Change::Unknown;
    }
    if import::hash(&document.editor_value()) == sync.local_hash {
        Change::Unchanged
    } else {
        Change::Changed
    }
}

fn valid_hash(hash: &str) -> bool {
    hash.strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

pub(crate) fn baseline(document: &Document) -> &'static str {
    match &document.state.sync {
        Some(sync) if sync.profile == EDITOR_PROFILE && valid_hash(&sync.local_hash) && valid_hash(&sync.remote_hash) => "scoped",
        Some(_) => "invalid-receipts",
        None => "none",
    }
}

pub(crate) fn classify(existing: &Document, merged: &Document, observed_remote_hash: &str) -> Agreement {
    let endpoints_equal = existing.editor_value() == merged.editor_value();
    let local = local_change(existing);
    let remote = remote_change(existing, observed_remote_hash);
    let state = if endpoints_equal {
        if local == Change::Unchanged && remote != Change::Unknown {
            if remote == Change::Unchanged { "clean" } else { "remote-changed" }
        } else {
            "converged"
        }
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
    let Some(sync) = &existing.state.sync else {
        return Change::Unknown;
    };
    if !valid_hash(&sync.remote_hash) {
        Change::Unknown
    } else if sync.remote_hash == observed_hash {
        Change::Unchanged
    } else {
        Change::Changed
    }
}

pub struct Observed {
    pub data: LeafletData,
    pub merged: Document,
    pub agreement: Agreement,
    /// Keep actual current fact/CRDT bytes for the keyed compiler; the typed
    /// document is not sufficient to reconstruct an existing Yjs document.
    pub snapshot: leafbud_web::PullSnapshot,
}

#[derive(Debug)]
pub struct ObservationError {
    pub kind: &'static str,
    pub message: String,
    pub remote_hash: Option<String>,
}

impl RemoteReader {
    pub async fn observe_editor(&self, existing: &Document) -> Result<Observed, ObservationError> {
        self.observe_editor_with_adoption(existing, false).await
    }

    pub(crate) async fn observe_editor_with_adoption(&self, existing: &Document, adopt_published: bool) -> Result<Observed, ObservationError> {
        let error = |kind, message: String| ObservationError {
            kind,
            message,
            remote_hash: None,
        };
        let Some(binding) = existing.state.binding.clone() else {
            return Err(error("unbound", "document has no native binding; push establishes one".into()));
        };
        let data = self.client.get(binding.token).await.map_err(|e| {
            error(
                if matches!(e, leafbud_web::Error::NotFound) { "missing" } else { "failed" },
                if matches!(e, leafbud_web::Error::NotFound) {
                    "remote object missing or inaccessible (token not found); local file preserved".into()
                } else {
                    format!("fetching remote metadata: {e}")
                },
            )
        })?;
        let transition = existing.state.state == "draft" && data.observed_state() == leafbud_web::LeafletState::Published && adopt_published;
        if data.observed_state().as_str() != existing.state.state && !transition {
            return Err(error(
                "unsupported",
                "remote lifecycle changed; explicit lifecycle adoption required, current source/receipts preserved".into(),
            ));
        }
        let snapshot = self
            .client
            .pull(binding.token)
            .await
            .map_err(|e| error("failed", format!("fetching remote editor content: {e}")))?;
        // Acquire the conservative witness before mapping. Unsupported content
        // must not disappear simply because it cannot become a Document.
        let hash = import::remote_hash(&data, &snapshot);
        let fresh = import::build(self.client.base_url(), &data, &snapshot).map_err(|e| ObservationError {
            kind: "unsupported",
            message: format!("remote content is unsupported for the refresh profile; local file preserved: {e:#}"),
            remote_hash: Some(hash.clone()),
        })?;
        let mut compared = binding.clone();
        if transition {
            compared.uri = fresh.state.binding.as_ref().and_then(|binding| binding.uri.clone());
        }
        same_identity(&compared, &fresh).map_err(|e| error("identity-changed", format!("{e:#}")))?;
        let mut merged = adopt(existing, &fresh);
        if transition {
            merged.state.state = "published".into();
            merged.state.binding = Some(compared);
            for field in ["path", "publishedAt"] {
                if existing.extra.get(field).is_none_or(serde_json::Value::is_null)
                    && let Some(value) = fresh.extra.get(field)
                {
                    merged.extra.insert(field.into(), value.clone());
                }
            }
        }
        let agreement = classify(existing, &merged, &hash);
        if agreement.endpoints_equal {
            // Non-entry page definition ordering is source layout, not editor
            // content. Settling witnesses must keep its original body bytes.
            merged.pages = existing.pages.clone();
            if merged.editor_value() != existing.editor_value() {
                return Err(ObservationError {
                    kind: "unsupported",
                    message: "source association correspondence could not be retained safely".into(),
                    remote_hash: Some(hash),
                });
            }
        }
        Ok(Observed {
            data,
            merged,
            agreement,
            snapshot,
        })
    }
}

pub(crate) fn same_identity(binding: &leafbud_document::Binding, fresh: &Document) -> Result<()> {
    let b = fresh.state.binding.as_ref().expect("built documents are bound");
    for (equal, name) in [
        (binding.base_url == b.base_url && binding.token == b.token, "origin/token"),
        (binding.root == b.root, "root entity"),
        (binding.entity_set == b.entity_set, "entity set (rights)"),
        (binding.entry_fact == b.entry_fact, "entry placement"),
        (binding.publication == b.publication, "publication association"),
        (binding.uri == b.uri, "public document URI"),
    ] {
        ensure!(equal, "remote {name} changed; identity/binding decision required (file preserved)");
    }
    Ok(())
}

/// Match each freshly observed occurrence to an existing source key: same
/// entity and same occurrence locator (block placement / page reference set).
fn reconcile_keys(existing: &Document, fresh: &Document) -> BTreeMap<String, String> {
    let occurrence_of = |node: &leafbud_document::Node| match &node.occurrence {
        Occurrence::Block { placement, .. } => node.entity.clone() + ":" + placement,
        Occurrence::Page { references } => {
            let mut refs = references.clone();
            refs.sort();
            format!("{}:page:{}", node.entity, refs.join(","))
        }
    };
    let mut rename = BTreeMap::new();
    let mut claimed = BTreeSet::new();
    // Reserve ALL exact occurrences before any entity-only fallback. Otherwise
    // a newly added shared occurrence sorting first can steal an existing key.
    for exact_pass in [true, false] {
        for (key, node) in &fresh.state.nodes {
            if rename.contains_key(key) {
                continue;
            }
            let locator = occurrence_of(node);
            let same_entity: Vec<_> = existing.state.nodes.iter().filter(|(_, n)| n.entity == node.entity).collect();
            let candidates: Vec<_> = same_entity
                .iter()
                .filter(|(_, n)| !exact_pass || occurrence_of(n) == locator)
                .map(|(k, _)| *k)
                .collect();
            let chosen = if candidates.len() == 1 && (exact_pass || same_entity.len() == 1) {
                Some(candidates[0].clone())
            } else {
                None
            };
            if let Some(chosen) = chosen
                && claimed.insert(chosen.clone())
            {
                rename.insert(key.clone(), chosen);
            }
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
///
/// Explicit-null standalone versus inherited intent must survive adoption:
/// a bound document always carries the confirmed association explicitly.
pub(crate) fn adopt(existing: &Document, fresh: &Document) -> Document {
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
    let mut extra: BTreeMap<_, _> = fresh
        .extra
        .iter()
        .filter(|(name, _)| EDITOR_OWNED_EXTRA.contains(&name.as_str()))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    for (name, value) in &existing.extra {
        if !EDITOR_OWNED_EXTRA.contains(&name.as_str()) {
            extra.insert(name.clone(), value.clone());
        }
    }
    let mut document = Document {
        format: fresh.format.clone(),
        title: fresh.title.clone(),
        description: if leafbud_document::canonical_description(existing.description.as_deref())
            == leafbud_document::canonical_description(fresh.description.as_deref())
        {
            existing.description.clone()
        } else {
            leafbud_document::canonical_description(fresh.description.as_deref()).map(str::to_owned)
        },
        tags: fresh.tags.clone(),
        publication: fresh.publication.clone(),
        theme: fresh.theme.clone(),
        extra,
        pages,
        state: State {
            publication_attempt: existing.state.publication_attempt.clone(),
            released: existing.state.released.clone(),
            names: existing.state.names.clone(),
            last_resolution: existing.state.last_resolution.clone(),
            writer: existing.state.writer.clone(),
            pending: existing.state.pending.clone(),
            id: existing.state.id,
            state: existing.state.state.clone(),
            main_page: fresh.state.main_page.as_ref().map(|main| key(main)),
            binding: existing.state.binding.clone(),
            nodes,
            sync: Some(Sync {
                profile: EDITOR_PROFILE.into(),
                local_hash: String::new(),
                remote_hash: fresh.state.sync.as_ref().expect("built receipts").remote_hash.clone(),
            }),
        },
    };
    document.state.sync.as_mut().expect("just set").local_hash = import::hash(&document.editor_value());
    document
}

#[cfg(test)]
mod tests;
