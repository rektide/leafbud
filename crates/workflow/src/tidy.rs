//! Deliberate local normalization passes: one-time source-key assignment and
//! Markdown normalization and independent names, under original-byte guards. By
//! default every applicable pass runs; selectors narrow to their union.
//! Formatting/preparation/naming never advance editor agreement or rewrite
//! historical attempt provenance. Only naming can settle its independent pair.
use crate::{config, files, names};
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
    pub from: Option<names::Direction>,
    pub accept: bool,
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
    pub needs_choice: usize,
    pub items: Vec<Item>,
}

#[derive(Serialize)]
pub struct Item {
    pub path: PathBuf,
    pub destination: Option<PathBuf>,
    pub names: Option<names::Evidence>,
    pub proposed_names: Option<leafbud_document::Names>,
    pub naming_action: &'static str,
    pub outcome: &'static str,
    pub keys_assigned: usize,
    pub blocks_changed: usize,
    pub blocks_skipped: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Reuse inventory selection; broken/unreadable managed candidates remain
/// per-item failures instead of aborting discovery of their neighbors.
fn managed_files(root: &Path) -> Result<Vec<PathBuf>> {
    Ok(crate::inventory::scan(root, None)?.paths)
}

/// One durable proposed key: `b-<random>` for blocks, `p-<random>` for the
/// main page. Assignment is one-time; proposals are not durable until applied.
fn fresh_key(prefix: &'static str) -> impl FnMut() -> String {
    move || format!("{prefix}-{}", Uuid::new_v4())
}

/// Run the selected local passes over the selected files. No network, no
/// remote writes, no receipt changes.
pub fn run(context: &config::Context, request: Request) -> Result<Report> {
    run_with(context, request, &mut |_, _| Ok(()))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Prepared,
    Renamed,
}

fn run_with(context: &config::Context, request: Request, hook: &mut impl FnMut(Stage, &Path) -> Result<()>) -> Result<Report> {
    ensure!(request.all ^ !request.files.is_empty(), "select file operands or --all");
    ensure!(!(request.from.is_some() && request.accept), "--from and --accept are exclusive");
    let default = !request.keys && !request.markdown && request.from.is_none() && !request.accept;
    let run_keys = request.keys || default;
    let run_markdown = request.markdown || default;
    let run_names = request.from.is_some() || request.accept || default;
    let pass = match (run_keys, run_markdown, run_names) {
        (true, true, true) => "all",
        (true, false, false) => "keys",
        (false, true, false) => "markdown",
        (false, false, true) => "names",
        _ => "selected",
    };
    let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
    let mut routes = Vec::new();
    let candidates: Vec<PathBuf> = if request.all {
        managed_files(&root)?
    } else {
        let mut resolved = Vec::new();
        for file in &request.files {
            let joined = if file.is_absolute() { file.clone() } else { root.join(file) };
            let path = std::fs::canonicalize(&joined).unwrap_or_else(|_| joined.clone());
            routes.push((joined, path.clone()));
            if !resolved.contains(&path) {
                resolved.push(path);
            }
        }
        resolved
    };
    let mut report = Report {
        version: 2,
        workspace: root,
        pass,
        dry_run: request.dry_run,
        selected: candidates.len(),
        normalized: 0,
        unchanged: 0,
        failed: 0,
        needs_choice: 0,
        items: vec![],
    };
    let selection = crate::native::Selection { operands: candidates.clone() };
    let passes = Passes {
        keys: run_keys,
        markdown: run_markdown,
        names: run_names,
        direction: if request.accept { Some(names::Direction::Accept) } else { request.from },
        dry_run: request.dry_run,
    };
    let mut planned_destinations = std::collections::BTreeSet::new();
    for path in &candidates {
        let mut item = tidy_file(context, path, &report.workspace, &selection, &routes, &passes, hook);
        // Preview has no filesystem effects to make an earlier target visible.
        // Account for that selected destination explicitly, without inventing
        // reorder/swap or overwrite behavior for the executing batch.
        if request.dry_run
            && item.outcome == "normalized"
            && let Some(target) = &item.destination
            && !planned_destinations.insert(target.clone())
        {
            item.outcome = "needs-choice";
            item.keys_assigned = 0;
            item.blocks_changed = 0;
            item.error = Some(format!(
                "destination {} is already selected by an earlier batch plan; choose distinct names",
                target.display()
            ));
        }
        match item.outcome {
            "normalized" => report.normalized += 1,
            "unchanged" => report.unchanged += 1,
            "needs-choice" => report.needs_choice += 1,
            _ => report.failed += 1,
        }
        report.items.push(item);
    }
    Ok(report)
}

struct Passes {
    keys: bool,
    markdown: bool,
    names: bool,
    direction: Option<names::Direction>,
    dry_run: bool,
}

#[allow(clippy::too_many_arguments)]
fn tidy_file(
    context: &config::Context,
    path: &Path,
    root: &Path,
    selection: &crate::native::Selection,
    routes: &[(PathBuf, PathBuf)],
    passes: &Passes,
    hook: &mut impl FnMut(Stage, &Path) -> Result<()>,
) -> Item {
    match try_tidy(context, path, root, selection, routes, passes, hook) {
        Ok(item) => item,
        Err(error) => Item {
            path: path.to_path_buf(),
            destination: None,
            names: None,
            proposed_names: None,
            naming_action: "none",
            outcome: "failed",
            keys_assigned: 0,
            blocks_changed: 0,
            blocks_skipped: 0,
            skip_reasons: vec![],
            error: Some(format!("{error:#}")),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn try_tidy(
    context: &config::Context,
    path: &Path,
    root: &Path,
    selection: &crate::native::Selection,
    routes: &[(PathBuf, PathBuf)],
    passes: &Passes,
    hook: &mut impl FnMut(Stage, &Path) -> Result<()>,
) -> Result<Item> {
    let store = crate::lock::Store::new(context)?;
    let (_guard, original, source) = crate::lock::read(path, &store)?;
    let identity = files::identity(path)?;
    ensure!(
        leafbud_mdx::header(&source)?["@leafbud"].get("pending").is_none_or(serde_json::Value::is_null),
        "pending native attempt; tidy preserves source for explicit recovery"
    );
    let check_duplicates = |current: &Path| -> Result<()> {
        let mut inventory = crate::inventory::scan(root, None)?;
        let mut operands = selection.operands.clone();
        operands.extend(routes.iter().filter_map(|(route, _)| std::fs::canonicalize(route).ok()));
        operands.push(current.into());
        operands.sort();
        operands.dedup();
        inventory.include_operands(&operands);
        ensure!(!inventory.duplicates.contains(current), "duplicate local identity/binding; preserved");
        Ok(())
    };
    check_duplicates(path)?;
    let naming = if passes.names {
        match names::plan(root, path, &source, passes.direction) {
            Ok(plan) => Some(plan),
            Err(error) => {
                return Ok(Item {
                    path: path.into(),
                    destination: None,
                    proposed_names: None,
                    names: Some(names::inspect(root, path, &leafbud_mdx::parse(&source)?)),
                    naming_action: "needs-choice",
                    outcome: "needs-choice",
                    keys_assigned: 0,
                    blocks_changed: 0,
                    blocks_skipped: 0,
                    skip_reasons: vec![],
                    error: Some(format!("{error:#}")),
                });
            }
        }
    } else {
        None
    };
    let mut keys_assigned = 0;
    let mut working = naming.as_ref().map_or_else(|| source.clone(), |plan| plan.source.clone());
    if passes.keys {
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
    if passes.markdown {
        let plan = leafbud_mdx::plan_markdown_tidy(&working).context("unsupported managed source; nothing was written")?;
        skip_reasons = plan.skip_reasons.iter().map(|(key, reason)| format!("{key}: {reason}")).collect();
        blocks_skipped = plan.blocks_skipped;
        if !plan.edits.is_empty() {
            blocks_changed = plan.blocks_changed;
            // Compose and validate even in dry run, so previews report real outcomes.
            working = leafbud_mdx::apply_markdown_tidy(&working, &plan).context("composing Markdown tidy")?;
        }
    }
    let destination = naming.as_ref().map_or_else(|| path.to_path_buf(), |plan| plan.destination.clone());
    let moved = destination != path;
    let changed = working != source || moved;
    let mut item = Item {
        path: path.to_path_buf(),
        destination: moved.then(|| destination.clone()),
        naming_action: naming.as_ref().map_or("none", |p| p.action),
        proposed_names: naming.as_ref().and_then(|p| p.proposed.clone()),
        names: naming.map(|p| p.evidence),
        outcome: if changed { "normalized" } else { "unchanged" },
        keys_assigned,
        blocks_changed,
        blocks_skipped,
        skip_reasons,
        error: None,
    };
    if moved && let Err(error) = names::destination(root, &destination) {
        item.outcome = "needs-choice";
        item.error = Some(format!(
            "destination {}: {error:#}; choose another path or explicitly resolve the collision",
            destination.display()
        ));
        return Ok(item);
    }
    let mut renamed = false;
    let apply = (|| -> Result<()> {
        hook(Stage::Prepared, path)?;
        if !changed {
            return Ok(());
        }
        check_duplicates(path)?;
        ensure!(std::fs::canonicalize(path)? == path, "source route changed during tidy; retry");
        for (route, expected) in routes.iter().filter(|(_, expected)| expected == path) {
            ensure!(std::fs::canonicalize(route)? == *expected, "operand route changed during tidy; retry");
        }
        ensure!(
            files::identity(path)? == identity && std::fs::read(path)? == original,
            "source bytes/file identity changed during tidy; preserved"
        );
        if moved {
            names::destination(root, &destination)?;
        }
        if passes.dry_run {
            return Ok(());
        }
        if moved {
            files::rename_noclobber(path, &destination)?;
            renamed = true;
            hook(Stage::Renamed, &destination)?;
            std::fs::File::open(path.parent().context("source parent missing")?)?.sync_all()?;
            std::fs::File::open(destination.parent().context("destination parent missing")?)?.sync_all()?;
            check_duplicates(&destination)?;
            ensure!(files::identity(&destination)? == identity, "destination identity changed after rename");
            files::write(&destination, Some(&original), working.as_bytes())?;
        } else {
            files::write(path, Some(&original), working.as_bytes()).context("local file changed during tidy; not overwritten")?;
        }
        Ok(())
    })();
    if let Err(error) = apply {
        item.outcome = if renamed { "partial" } else { "failed" };
        item.keys_assigned = 0;
        item.blocks_changed = 0;
        if renamed {
            item.naming_action = "renamed-settlement-unconfirmed";
            item.error = Some(format!(
                "renamed to {}; source/witness settlement not confirmed: {error:#}; inspect and rerun tidy at the new path; it may contain original, completely patched or externally edited source; no rollback was attempted",
                destination.display()
            ));
        } else {
            item.error = Some(format!("{error:#}"));
        }
    }
    Ok(item)
}

#[cfg(test)]
mod tests;
