//! Deliberate local normalization passes. Only Markdown body normalization
//! is implemented today: parser-derived span edits proven by the production
//! reader/writer, applied atomically under the original-byte guard. Key
//! assignment and naming reconciliation are separately scoped future passes
//! the CLI rejects honestly instead of half-running.
use crate::config;
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Request {
    pub all: bool,
    pub files: Vec<PathBuf>,
    pub dry_run: bool,
}

#[derive(Serialize)]
pub struct Report {
    pub version: u8,
    pub workspace: PathBuf,
    pub pass: &'static str,
    pub dry_run: bool,
    pub selected: usize,
    pub normalized: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub items: Vec<Item>,
}

#[derive(Serialize)]
pub struct Item {
    pub path: PathBuf,
    pub outcome: &'static str,
    pub blocks_changed: usize,
    pub blocks_skipped: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Managed-looking .mdx files under the workspace. Files that fail later
/// parsing stay loud as failed items; unmanaged files are ignored.
fn managed_files(root: &Path) -> Result<Vec<PathBuf>> {
    fn visit(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_dir() {
                visit(&path, out)?;
            } else if kind.is_file() && path.extension().is_some_and(|e| e == "mdx") && std::fs::read_to_string(&path)?.contains("@leafspring") {
                out.push(path);
            }
        }
        Ok(())
    }
    let mut out = vec![];
    visit(root, &mut out)?;
    out.sort();
    Ok(out)
}

/// Run the Markdown tidy pass over the selected local files. No network,
/// no remote writes, no hash or receipt changes: formatting never accepts
/// local edits, so witnesses are only ever preserved byte-for-byte.
pub fn run(context: &config::Context, request: Request) -> Result<Report> {
    ensure!(request.all ^ !request.files.is_empty(), "select file operands or --all");
    let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
    let candidates: Vec<PathBuf> = if request.all {
        managed_files(&root)?
    } else {
        let mut resolved = Vec::new();
        for file in &request.files {
            let joined = if file.is_absolute() { file.clone() } else { root.join(file) };
            let path = std::fs::canonicalize(&joined).with_context(|| format!("tidy operand {} not found", file.display()))?;
            if !resolved.contains(&path) {
                resolved.push(path);
            }
        }
        resolved
    };
    let mut report = Report {
        version: 1,
        workspace: root,
        pass: "markdown",
        dry_run: request.dry_run,
        selected: candidates.len(),
        normalized: 0,
        unchanged: 0,
        failed: 0,
        items: vec![],
    };
    for path in candidates {
        let item = tidy_file(&path, request.dry_run);
        match item.outcome {
            "normalized" => report.normalized += 1,
            "unchanged" => report.unchanged += 1,
            _ => report.failed += 1,
        }
        report.items.push(item);
    }
    Ok(report)
}

fn tidy_file(path: &Path, dry_run: bool) -> Item {
    match try_tidy(path, dry_run) {
        Ok(item) => item,
        Err(error) => Item {
            path: path.to_path_buf(),
            outcome: "failed",
            blocks_changed: 0,
            blocks_skipped: 0,
            skip_reasons: vec![],
            error: Some(format!("{error:#}")),
        },
    }
}

fn try_tidy(path: &Path, dry_run: bool) -> Result<Item> {
    let original = std::fs::read(path).context("reading managed file")?;
    let source = String::from_utf8(original.clone()).context("managed file is not UTF-8")?;
    let plan = leafspring_mdx::plan_markdown_tidy(&source).context("unsupported managed source; nothing was written")?;
    let skip_reasons = plan.skip_reasons.iter().map(|(key, reason)| format!("{key}: {reason}")).collect::<Vec<_>>();
    let item = Item {
        path: path.to_path_buf(),
        outcome: "unchanged",
        blocks_changed: 0,
        blocks_skipped: plan.blocks_skipped,
        skip_reasons,
        error: None,
    };
    if plan.edits.is_empty() {
        return Ok(item);
    }
    // Compose and validate even in dry run, so previews report real outcomes.
    let next = leafspring_mdx::apply_markdown_tidy(&source, &plan).context("composing Markdown tidy")?;
    if !dry_run && let Err(error) = crate::files::write(path, Some(&original), next.as_bytes()).context("local file changed during tidy; not overwritten") {
        return Err(error);
    }
    Ok(Item {
        outcome: "normalized",
        blocks_changed: plan.blocks_changed,
        ..item
    })
}

#[cfg(test)]
mod tests;
