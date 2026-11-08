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
use leafspring_web::LeafletState;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

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
        let observed = match self.observe_editor(&existing).await {
            Ok(observed) => observed,
            Err(error) if error.kind == "identity-changed" || (error.kind == "unsupported" && error.remote_hash.is_none()) => {
                return Outcome::Decision(error.message);
            }
            Err(error) => return Outcome::Failure(error.message),
        };
        let binding = &existing.state.binding;
        let data = observed.data;
        let merged = observed.merged;
        let agreement = observed.agreement;
        let outcome = if agreement.state == "clean" {
            "unchanged"
        } else if agreement.endpoints_equal {
            if agreement.baseline == "scoped" && agreement.local == crate::editor_state::Change::Unchanged {
                "receipts-only"
            } else {
                "converged"
            }
        } else if agreement.state == "remote-changed" {
            "updated"
        } else if take_remote {
            "take-remote"
        } else if agreement.state == "local-changed" {
            return Outcome::Decision(
                "local edits preserved (remote unchanged); nothing to adopt — keep editing locally or rerun with --take remote to replace local editor-owned content"
                    .into(),
            );
        } else if agreement.state == "both-changed" {
            return Outcome::Decision(
                "local and remote edits diverge; file preserved — rerun with --take remote to adopt current remote content (local-only metadata fields survive)"
                    .into(),
            );
        } else {
            return Outcome::Decision("editor baseline unverified for this receipt profile; differing current endpoints preserved. Equal current endpoints can be settled by an explicit pull; otherwise make an explicit content decision".into());
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
