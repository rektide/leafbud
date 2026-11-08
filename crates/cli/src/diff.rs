//! Foreground viewer adapter. Deliberately no --json: the tool owns stdio/TTY.
use leafspring_workflow::{RemoteOptions, config, diff::Tool};
use std::path::Path;

pub async fn run(options: RemoteOptions, root: Option<&Path>, flags: &config::Config, file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let tool = Tool::from_config(&context.effective)?;
    let reader = options.connect().await?;
    let report = reader.diff(&context, file, &tool).await?;
    eprintln!(
        "read-only local.md → editor.md: {} normalized views; tool exit {}; no source or receipts written",
        if report.views_equal { "equal" } else { "different" },
        report.tool_exit
    );
    Ok(())
}
