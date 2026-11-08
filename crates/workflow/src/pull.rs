//! Guarded refresh (pull) of bound local files from their current editors.
//!
//! Pull observes the same bound native editor as import, validates identity,
//! rights, lifecycle and association, and compares current endpoints using the
//! stored receipts only — no base snapshots, CRDT history or server CAS.
//! Unchanged files are exact no-ops; remote-only changes are adopted with
//! stable document/source/node identities (manual key renames survive);
//! diverged files are preserved for an explicit decision unless `--take
//! remote` deliberately replaces local editor-owned content. Receipts settle
//! without content rewrites only after verified semantic correspondence.
use crate::{RemoteReader, config};
use anyhow::{Context, Result, ensure};
use leafspring_document::{Block, BlockKind, Document, Node, PROFILE, Page, State, Sync};
use leafspring_web::LeafletState;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use uuid::Uuid;

/// The only `extra` key the import profile currently treats as remote-owned.
const REMOTE_OWNED_EXTRA: &[&str] = &["contributors"];

#[derive(Default)]
pub struct Request {
    pub all: bool,
    pub files: Vec<PathBuf>,
    pub state: Option<LeafletState>,
    pub take_remote: bool,
    pub dry_run: bool,
}

#[derive(Serialize)]
pub struct Report {
    pub version: u8,
    pub workspace: PathBuf,
    pub dry_run: bool,
    pub take_remote: bool,
    pub selected: usize,
    pub skipped: usize,
    pub unchanged: usize,
    pub updated: usize,
    pub receipts_only: usize,
    pub converged: usize,
    pub needs_decision: usize,
    pub failed: usize,
    pub items: Vec<Item>,
}

#[derive(Serialize)]
pub struct Item {
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<Uuid>,
    pub outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Local workspace scan: bound files by token, plus managed-looking files
/// whose header cannot be inspected safely (kept loud, never pruned).
struct Bound {
    index: BTreeMap<Uuid, Vec<PathBuf>>,
    states: BTreeMap<PathBuf, String>,
    broken: BTreeMap<PathBuf, String>,
    unmanaged: usize,
}

fn scan(root: &Path, base: &str) -> Result<Bound> {
    fn visit(path: &Path, base: &str, bound: &mut Bound) -> Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_dir() {
                visit(&path, base, bound)?;
            } else if kind.is_file() && path.extension().is_some_and(|e| e == "mdx") {
                let source = std::fs::read_to_string(&path)?;
                let header = match leafspring_mdx::header(&source) {
                    Ok(header) => header,
                    Err(_) if !source.contains("@leafspring") => {
                        bound.unmanaged += 1;
                        continue;
                    }
                    Err(error) => {
                        bound.broken.insert(path, format!("{error:#}"));
                        continue;
                    }
                };
                if header["@leafspring"]["binding"]["baseUrl"] != base {
                    continue;
                }
                let token = header["@leafspring"]["binding"]["token"].as_str().context("managed file token missing")?;
                match Uuid::parse_str(token).context("managed file token invalid") {
                    Ok(token) => {
                        if let Some(state) = header["@leafspring"]["state"].as_str() {
                            bound.states.insert(path.clone(), state.to_owned());
                        }
                        bound.index.entry(token).or_default().push(path);
                    }
                    Err(error) => {
                        bound.broken.insert(path, format!("{error:#}"));
                    }
                }
            }
        }
        Ok(())
    }
    let mut bound = Bound {
        index: BTreeMap::new(),
        states: BTreeMap::new(),
        broken: BTreeMap::new(),
        unmanaged: 0,
    };
    visit(root, base, &mut bound)?;
    Ok(bound)
}

/// Source-key reconciliation: keep the existing local key whenever the fresh
/// binding identifies the same remote object (same entity, and matching
/// placements — exact membership set, or a unique entity when placements
/// grew). Imported deterministic IDs are first-association choices, never
/// permission to rekey an existing file on refresh.
fn reconcile_keys(existing: &Document, fresh: &Document) -> BTreeMap<String, String> {
    let mut rename = BTreeMap::new();
    let mut claimed = BTreeSet::new();
    for key in fresh.state.nodes.keys().filter(|k| existing.state.nodes.contains_key(*k)) {
        claimed.insert(key.clone());
        rename.insert(key.clone(), key.clone());
    }
    for (key, node) in &fresh.state.nodes {
        if rename.contains_key(key) {
            continue;
        }
        let same_entity: Vec<(&String, &Node)> = existing.state.nodes.iter().filter(|(_, n)| n.entity == node.entity).collect();
        let memberships: BTreeSet<_> = node.memberships.keys().collect();
        let exact: Vec<&String> = same_entity
            .iter()
            .filter(|(_, n)| n.memberships.keys().collect::<BTreeSet<_>>() == memberships)
            .map(|(k, _)| *k)
            .collect();
        let chosen = if exact.len() == 1 {
            exact[0].clone()
        } else if same_entity.len() == 1 {
            same_entity[0].0.clone()
        } else {
            key.clone()
        };
        if claimed.insert(chosen.clone()) {
            rename.insert(key.clone(), chosen);
        } else {
            claimed.insert(key.clone());
            rename.insert(key.clone(), key.clone());
        }
    }
    rename
}

/// Merge the fresh remote observation into the local document's identity:
/// remote-owned fields and content from `fresh`, existing document id and
/// binding, preserved manual key renames, and local-only `extra` metadata.
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
    let nodes = fresh.state.nodes.iter().map(|(k, node)| (key(k), node.clone())).collect::<BTreeMap<_, _>>();
    let pages = fresh
        .pages
        .iter()
        .map(|page| Page {
            key: key(&page.key),
            blocks: page.blocks.iter().map(block).collect(),
        })
        .collect::<Vec<_>>();
    let mut extra = fresh.extra.clone();
    for (name, value) in &existing.extra {
        if !REMOTE_OWNED_EXTRA.contains(&name.as_str()) && !extra.contains_key(name) {
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
                profile: PROFILE.into(),
                local_hash: String::new(),
                remote_hash: fresh.state.sync.remote_hash.clone(),
            },
        },
    };
    document.state.sync.local_hash = crate::import::hash(&document.semantic_value());
    document
}

/// What one guarded refresh decided for a file.
enum Outcome {
    Success(&'static str),
    /// Preserved; a human choice is required (reported, never a silent skip).
    Decision(String),
    Failure(String),
}

impl RemoteReader {
    pub async fn pull(&self, context: &config::Context, request: Request) -> Result<Report> {
        ensure!(
            request.all ^ !request.files.is_empty(),
            "select file operands or --all (pull refreshes existing bound files; import discovers new ones)"
        );
        let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
        let base = self.client.base_url();
        let bound = scan(&root, base)?;
        let mut report = Report {
            version: 1,
            workspace: root.clone(),
            dry_run: request.dry_run,
            take_remote: request.take_remote,
            selected: 0,
            skipped: 0,
            unchanged: 0,
            updated: 0,
            receipts_only: 0,
            converged: 0,
            needs_decision: 0,
            failed: 0,
            items: vec![],
        };
        // `--all` selects existing bound local files (plus loud broken managed
        // files), never remote discovery; unmanaged files are ignored.
        let candidates: Vec<PathBuf> = if request.all {
            let mut paths: Vec<PathBuf> = bound.index.values().flatten().cloned().chain(bound.broken.keys().cloned()).collect();
            paths.sort();
            paths
        } else {
            let mut resolved = Vec::new();
            for file in &request.files {
                let joined = if file.is_absolute() { file.clone() } else { root.join(file) };
                let path = std::fs::canonicalize(&joined).with_context(|| format!("pull operand {} not found", file.display()))?;
                if !resolved.contains(&path) {
                    resolved.push(path);
                }
            }
            resolved
        };
        report.selected = candidates
            .iter()
            .filter(|path| match request.state {
                None => true,
                // A broken managed file cannot prove its state; keep it loud.
                Some(state) => bound.states.get(*path).is_none_or(|local| local == state.as_str()),
            })
            .count();
        for path in candidates {
            if let Some(state) = request.state
                && bound.states.get(&path).is_some_and(|local| local != state.as_str())
            {
                report.skipped += 1;
                continue;
            }
            let item = self.refresh(&bound, &path, request.take_remote, request.dry_run).await;
            match item.outcome {
                "unchanged" => report.unchanged += 1,
                "updated" | "take-remote" => report.updated += 1,
                "receipts-only" => report.receipts_only += 1,
                "converged" => report.converged += 1,
                "needs-decision" => report.needs_decision += 1,
                _ => report.failed += 1,
            }
            report.items.push(item);
        }
        Ok(report)
    }

    async fn refresh(&self, bound: &Bound, path: &Path, take_remote: bool, dry_run: bool) -> Item {
        match self.try_refresh(bound, path, take_remote, dry_run).await {
            Outcome::Success(outcome) => Item {
                path: path.to_path_buf(),
                token: bound_token(bound, path),
                outcome,
                error: None,
            },
            Outcome::Decision(error) => Item {
                path: path.to_path_buf(),
                token: bound_token(bound, path),
                outcome: "needs-decision",
                error: Some(error),
            },
            Outcome::Failure(error) => Item {
                path: path.to_path_buf(),
                token: bound_token(bound, path),
                outcome: "failed",
                error: Some(error),
            },
        }
    }

    async fn try_refresh(&self, bound: &Bound, path: &Path, take_remote: bool, dry_run: bool) -> Outcome {
        let fail = |error: anyhow::Error| Outcome::Failure(format!("{error:#}"));
        // Original bytes are retained across every await; the atomic writer
        // rechecks them before replacement.
        let original = match std::fs::read(path).context("reading managed file") {
            Ok(original) => original,
            Err(error) => return fail(error),
        };
        let source = match String::from_utf8(original.clone()).context("managed file is not UTF-8") {
            Ok(source) => source,
            Err(error) => return fail(error),
        };
        if !source.contains("@leafspring") {
            return Outcome::Failure("not a managed document; import creates bindings".into());
        }
        let existing = match leafspring_mdx::parse(&source).context("existing document has unsupported/invalid local edits; preserved") {
            Ok(existing) => existing,
            Err(error) => return fail(error),
        };
        let token = existing.state.binding.token;
        let Some(bound_here) = bound.index.get(&token) else {
            return fail(anyhow::anyhow!("file is not bound to this origin; import creates bindings"));
        };
        if bound_here.len() != 1 {
            return fail(anyhow::anyhow!("duplicate local bindings for this remote require resolution; file preserved"));
        }
        let data = match self.client.get(token).await {
            Ok(data) => data,
            Err(leafspring_web::Error::NotFound) => {
                return Outcome::Failure("remote object missing or inaccessible (token not found); local file preserved".into());
            }
            Err(other) => return Outcome::Failure(format!("fetching remote metadata: {other}")),
        };
        if data.observed_state() != LeafletState::Draft {
            return Outcome::Decision("remote lifecycle changed (published); this refresh profile only adopts draft editors".into());
        }
        let pull_snapshot = match self.client.pull(token).await.context("fetching remote editor content") {
            Ok(pull) => pull,
            Err(error) => return fail(error),
        };
        let fresh = match crate::import::build(self.client.base_url(), &data, &pull_snapshot)
            .context("remote content is unsupported for the refresh profile; local file preserved")
        {
            Ok(fresh) => fresh,
            Err(error) => return fail(error),
        };
        // Identity, rights, lifecycle and association guards: pull never mints
        // another object, retargets a binding or adopts a different native
        // structure silently.
        let binding = &existing.state.binding;
        let changed = |what: &str| Outcome::Decision(format!("remote {what} changed; identity/binding decision required (file preserved)"));
        if fresh.state.id != existing.state.id {
            return changed("document identity");
        }
        if fresh.state.binding.root != binding.root {
            return changed("root entity");
        }
        if fresh.state.binding.entity_set != binding.entity_set {
            return changed("entity set (rights)");
        }
        if fresh.state.binding.entry_fact != binding.entry_fact {
            return changed("entry placement");
        }
        if fresh.state.binding.publication != binding.publication {
            return changed("publication association");
        }
        let merged = adopt(&existing, &fresh);
        // Classify against current endpoints using receipts only.
        let local_unchanged = crate::import::hash(&existing.semantic_value()) == existing.state.sync.local_hash;
        let remote_unchanged = fresh.state.sync.remote_hash == existing.state.sync.remote_hash;
        let endpoints_equal = existing.semantic_value() == merged.semantic_value();
        let outcome = if endpoints_equal {
            if local_unchanged && remote_unchanged {
                "unchanged"
            } else if local_unchanged {
                "receipts-only"
            } else {
                "converged"
            }
        } else if local_unchanged {
            "updated"
        } else if take_remote {
            "take-remote"
        } else if remote_unchanged {
            return Outcome::Decision(
                "local edits preserved (remote unchanged); nothing to adopt — keep editing locally or rerun with --take remote to replace local editor-owned content"
                    .into(),
            );
        } else {
            return Outcome::Decision(
                "local and remote edits diverge; file preserved — rerun with --take remote to adopt current remote content (local-only metadata fields survive)"
                    .into(),
            );
        };
        if outcome == "unchanged" || dry_run {
            return Outcome::Success(outcome);
        }
        // Recheck remote identity/lifecycle/association after content
        // acquisition; this is not server CAS, and the local original-byte
        // guard still applies through the atomic replacement.
        let verified = match self.client.get(token).await.context("re-verifying remote identity") {
            Ok(verified) => verified,
            Err(error) => return fail(error),
        };
        let association_ok = match crate::import::association(&verified) {
            Ok(association) => association == binding.publication,
            Err(_) => false,
        };
        if verified.metadata.root_entity != data.metadata.root_entity || verified.observed_state() != LeafletState::Draft || !association_ok {
            return fail(anyhow::anyhow!("remote identity/lifecycle/association changed during pull; file preserved"));
        }
        let next = match leafspring_mdx::update(&source, &merged).context("composing guarded refresh") {
            Ok(next) => next,
            Err(error) => return fail(error),
        };
        if next.as_bytes() != original.as_slice()
            && let Err(error) = crate::files::write(path, Some(&original), next.as_bytes()).context("local file changed during pull; not overwritten")
        {
            return fail(error);
        }
        Outcome::Success(outcome)
    }
}

fn bound_token(bound: &Bound, path: &Path) -> Option<Uuid> {
    bound.index.iter().find(|(_, paths)| paths.iter().any(|p| p == path)).map(|(token, _)| *token)
}

#[cfg(test)]
mod tests;
