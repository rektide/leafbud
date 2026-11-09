//! Human external-tool comparison only. No merge, historical base or edit plan.
use crate::{RemoteReader, config, inventory};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;

/// Explicit external-tool exit convention. GUI/interactive viewers normally
/// use Success; POSIX-style diff uses Diff. Never infer this from the tool name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExitMode {
    #[default]
    Success,
    Diff,
}

impl std::str::FromStr for ExitMode {
    type Err = &'static str;
    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "success" => Ok(Self::Success),
            "diff" => Ok(Self::Diff),
            _ => Err("expected success (0 only) or diff (0 equal, 1 different)"),
        }
    }
}

/// Direct executable invocation: args followed by absolute local.md/editor.md
/// paths, in that order. No shell, substitution or detached-viewer lifecycle.
pub struct Tool {
    pub program: String,
    pub args: Vec<String>,
    pub exit: ExitMode,
}

impl Tool {
    pub fn from_config(config: &config::Config) -> Result<Self> {
        let tool = Self {
            program: config
                .diff_program
                .clone()
                .context("configure diff-program first (e.g. config set diff-program diff; config set diff-args '[\"-u\"]'; config set diff-exit diff)")?,
            args: config.diff_args.clone().unwrap_or_default(),
            exit: config.diff_exit.unwrap_or_default(),
        };
        ensure!(
            !tool.program.trim().is_empty() && !tool.program.contains('\0'),
            "diff-program must be a nonempty executable name/path, not a shell command"
        );
        ensure!(tool.args.iter().all(|arg| !arg.contains('\0')), "diff-args cannot contain NUL bytes");
        Ok(tool)
    }
}

#[derive(Debug)]
pub struct Report {
    /// Presentation equality only, not scoped editor/receipt equality.
    pub views_equal: bool,
    pub tool_exit: i32,
}

impl RemoteReader {
    /// Observe current local and bound native content, render the same human
    /// view on both sides, wait for the trusted foreground tool, then remove
    /// the private temporary directory. Source bytes/receipts are never written.
    pub async fn diff(&self, context: &config::Context, file: &Path, tool: &Tool) -> Result<Report> {
        let (local, editor) = self.comparison_views(context, file).await?;
        run_tool(&context.workspace_root, tool, &local, &editor).await
    }

    async fn comparison_views(&self, context: &config::Context, file: &Path) -> Result<(String, String)> {
        let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
        let path =
            std::fs::canonicalize(if file.is_absolute() { file.to_path_buf() } else { root.join(file) }).context("diff operand not found or inaccessible")?;
        let original = std::fs::read(&path).context("reading diff operand")?;
        let source = std::str::from_utf8(&original).context("diff operand is not UTF-8")?;
        let header = leafbud_mdx::header(source).map_err(|_| anyhow::anyhow!("invalid local frontmatter; use check to diagnose"))?;
        ensure!(
            header.get("@springleaf").is_none(),
            "legacy springleaf namespace requires deliberate conversion; diff never migrates files"
        );
        ensure!(header.get("@leafbud").is_some(), "not a managed document; import establishes a native binding");
        let existing = leafbud_mdx::parse(source)
            .map_err(|_| anyhow::anyhow!("invalid or unsupported local format/content/binding/lifecycle; use check; no partial comparison produced"))?;
        let mut inventory = inventory::scan(&root, None)?;
        inventory.include_operands(std::slice::from_ref(&path));
        ensure!(
            !inventory.duplicates.contains(&path),
            "duplicate local document ID or native binding; resolve the copies before comparison"
        );
        let binding = existing
            .state
            .binding
            .as_ref()
            .context("unbound local draft; push establishes the native binding diff compares against")?;
        ensure!(
            !binding.root.is_empty() && !binding.entity_set.is_empty() && !binding.entry_fact.is_empty(),
            "incoherent binding; resolve its identity explicitly"
        );
        ensure!(
            binding.base_url == self.client.base_url(),
            "bound to another origin; select its --base-url explicitly (defaults never retarget bindings)"
        );
        let observed = self.observe_editor(&existing).await.map_err(|error| {
            anyhow::anyhow!(match error.kind {
                "missing" => "native editor is missing or inaccessible; no comparison produced (this does not prove deletion)",
                "identity-changed" => "remote identity/rights/placement/association changed; resolve the binding before comparison",
                "unsupported" => "remote lifecycle/content is unsupported; no partial comparison produced",
                _ => "remote read failed; no comparison produced; credentials and response bodies withheld",
            })
        })?;
        ensure!(
            std::fs::read(&path).ok().as_deref() == Some(original.as_slice()),
            "local file changed during observation; rerun diff; nothing was written"
        );
        let local = leafbud_mdx::render_comparison(&existing).context("rendering local comparison view")?;
        let editor = leafbud_mdx::render_comparison(&observed.merged).context("rendering editor comparison view")?;
        Ok((local, editor))
    }
}

async fn run_tool(root: &Path, tool: &Tool, local: &str, editor: &str) -> Result<Report> {
    let dir = tempfile::Builder::new()
        .prefix("leafbud-diff-")
        .tempdir()
        .context("creating private comparison directory")?;
    let directory = std::fs::canonicalize(dir.path()).context("resolving private comparison directory")?;
    let local_path = directory.join("local.md");
    let editor_path = directory.join("editor.md");
    std::fs::write(&local_path, local).context("writing temporary local view")?;
    std::fs::write(&editor_path, editor).context("writing temporary editor view")?;
    let status = tokio::process::Command::new(&tool.program)
        .args(&tool.args)
        .arg(&local_path)
        .arg(&editor_path)
        .current_dir(root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .status()
        .await
        .context("starting/waiting for diff tool (check executable and arguments; no shell is used)")?;
    // Close explicitly on ordinary completion, including non-success exit, so
    // cleanup failure is observable. Drop also cleans up on spawn/wait errors.
    dir.close().context("removing temporary comparison views")?;
    match status.code() {
        Some(code) if code == 0 || (tool.exit == ExitMode::Diff && code == 1) => Ok(Report {
            views_equal: local == editor,
            tool_exit: code,
        }),
        Some(code) => bail!(
            "diff tool exited with {code}; expected {}",
            if tool.exit == ExitMode::Diff {
                "0 or 1 in diff mode"
            } else {
                "0 in success mode"
            }
        ),
        None => bail!("diff tool terminated without an exit code (signal); comparison did not complete"),
    }
}

#[cfg(test)]
mod tests;
