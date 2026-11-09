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
use leafbud_web::LeafletState;
use serde::Serialize;
use std::path::PathBuf;
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

impl RemoteReader {
    pub async fn pull(&self, context: &config::Context, request: Request) -> Result<Report> {
        ensure!(
            request.all ^ !request.files.is_empty(),
            "select file operands or --all (pull refreshes existing bound files; import discovers new ones)"
        );
        let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
        let base = self.client.base_url();
        let mut bound = crate::inventory::scan(&root, Some(base))?;
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
            bound.paths.clone()
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
        bound.include_operands(&candidates);
        report.selected = candidates
            .iter()
            .filter(|path| match request.state {
                None => true,
                // A broken managed file cannot prove its state; keep it loud.
                Some(state) => bound.states.get(*path).is_none_or(|local| local == state.as_str()),
            })
            .count();
        for path in &candidates {
            if let Some(state) = request.state
                && bound.states.get(path).is_some_and(|local| local != state.as_str())
            {
                report.skipped += 1;
                continue;
            }
            let result = self
                .refresh_document(
                    context,
                    path,
                    crate::refresh::Policy {
                        take_remote: request.take_remote,
                        dry_run: request.dry_run,
                        state: request.state,
                        operands: candidates.clone(),
                        ..Default::default()
                    },
                )
                .await;
            use crate::refresh::Outcome;
            let outcome = match result.outcome {
                Outcome::LocalDirty | Outcome::NeedsDecision => "needs-decision",
                Outcome::Unchanged | Outcome::Updated | Outcome::ReceiptsOnly | Outcome::Converged | Outcome::TakeRemote => result.outcome.as_str(),
                _ => "failed",
            };
            let item = Item {
                path: path.clone(),
                token: bound.token(path),
                outcome,
                error: result.message,
            };
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
}

#[cfg(test)]
pub(crate) mod tests;
