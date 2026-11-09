//! Deliberate local normalization passes: one-time source-key assignment and
//! Markdown body normalization, applied under the original-byte guard. By
//! default every applicable pass runs; selectors narrow to their union.
//! Formatting and preparation never settle agreement: receipts are preserved
//! byte-for-byte and never advanced here.
use crate::{config, files};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Default)]
pub struct Request {
    pub all: bool,
    pub files: Vec<PathBuf>,
    pub dry_run: bool,
    /// `--keys`: assign missing durable source identities only.
    pub keys: bool,
    /// `--markdown`: Markdown body normalization only.
    pub markdown: bool,
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
    pub keys_assigned: usize,
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
            } else if kind.is_file() && path.extension().is_some_and(|e| e == "mdx") && std::fs::read_to_string(&path)?.contains("@leafbud") {
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

/// One durable proposed key: `b-<random>` for blocks, `p-<random>` for the
/// main page. Assignment is one-time; proposals are not durable until applied.
fn fresh_key(prefix: &'static str) -> impl FnMut() -> String {
    move || format!("{prefix}-{}", Uuid::new_v4())
}

/// Run the selected local passes over the selected files. No network, no
/// remote writes, no receipt changes.
pub fn run(context: &config::Context, request: Request) -> Result<Report> {
    ensure!(request.all ^ !request.files.is_empty(), "select file operands or --all");
    let run_keys = request.keys || !request.markdown;
    let run_markdown = request.markdown || !request.keys;
    if !run_keys && !run_markdown {
        ensure!(false, "select at least one pass");
    }
    let pass = match (run_keys, run_markdown) {
        (true, true) => "all",
        (true, false) => "keys",
        _ => "markdown",
    };
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
        pass,
        dry_run: request.dry_run,
        selected: candidates.len(),
        normalized: 0,
        unchanged: 0,
        failed: 0,
        items: vec![],
    };
    for path in candidates {
        let item = tidy_file(context, &path, request.dry_run, run_keys, run_markdown);
        match item.outcome {
            "normalized" => report.normalized += 1,
            "unchanged" => report.unchanged += 1,
            _ => report.failed += 1,
        }
        report.items.push(item);
    }
    Ok(report)
}

fn tidy_file(context: &config::Context, path: &Path, dry_run: bool, run_keys: bool, run_markdown: bool) -> Item {
    match try_tidy(context, path, dry_run, run_keys, run_markdown) {
        Ok(item) => item,
        Err(error) => Item {
            path: path.to_path_buf(),
            outcome: "failed",
            keys_assigned: 0,
            blocks_changed: 0,
            blocks_skipped: 0,
            skip_reasons: vec![],
            error: Some(format!("{error:#}")),
        },
    }
}

fn try_tidy(context: &config::Context, path: &Path, dry_run: bool, run_keys: bool, run_markdown: bool) -> Result<Item> {
    let store = crate::lock::Store::new(context)?;
    let (_guard, original, source) = crate::lock::read(path, &store)?;
    let mut inventory = crate::inventory::scan(&context.workspace_root, None)?;
    inventory.include_operands(&[path.into()]);
    ensure!(!inventory.duplicates.contains(path), "duplicate local identity/binding; preserved");
    let mut keys_assigned = 0;
    let mut working = source.clone();
    if run_keys {
        let (mut block_source, mut page_source) = (fresh_key("b"), fresh_key("p"));
        let plan = leafbud_mdx::prepare_keys(
            &working,
            leafbud_mdx::KeySource {
                block: &mut block_source,
                page: &mut page_source,
            },
        )
        .context("unsupported managed source; nothing was written")?;
        if !plan.is_noop() {
            keys_assigned = plan.assigned + usize::from(plan.main_page.is_some());
            working = leafbud_mdx::apply_key_preparation(&working, &plan).context("composing key preparation")?;
        }
    }
    let mut blocks_changed = 0;
    let mut blocks_skipped = 0;
    let mut skip_reasons = Vec::new();
    if run_markdown {
        let plan = leafbud_mdx::plan_markdown_tidy(&working).context("unsupported managed source; nothing was written")?;
        skip_reasons = plan.skip_reasons.iter().map(|(key, reason)| format!("{key}: {reason}")).collect();
        blocks_skipped = plan.blocks_skipped;
        if !plan.edits.is_empty() {
            blocks_changed = plan.blocks_changed;
            // Compose and validate even in dry run, so previews report real outcomes.
            working = leafbud_mdx::apply_markdown_tidy(&working, &plan).context("composing Markdown tidy")?;
        }
    }
    let changed = working != source;
    let outcome = if changed { "normalized" } else { "unchanged" };
    if changed && !dry_run {
        files::write(path, Some(&original), working.as_bytes()).context("local file changed during tidy; not overwritten")?;
    }
    Ok(Item {
        path: path.to_path_buf(),
        outcome,
        keys_assigned,
        blocks_changed,
        blocks_skipped,
        skip_reasons,
        error: None,
    })
}

#[cfg(test)]
mod tests;
