//! Foreground viewer adapter. Deliberately no --json: the tool owns stdio/TTY.
use leafbud_workflow::{RemoteOptions, config, diff::Tool};
use std::path::Path;

pub async fn run(options: RemoteOptions, root: Option<&Path>, flags: &config::Config, file: &Path, release: bool) -> Result<(), Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let tool = Tool::from_config(&context.effective)?;
    let report = if release {
        let public = leafbud_workflow::release::PublicReader::new(flags.pds.as_deref())?;
        public.diff_release(&context, file, &tool).await?
    } else {
        options.connect().await?.diff(&context, file, &tool).await?
    };
    eprintln!(
        "read-only current local → {}: {} normalized views; tool exit {}; no source or receipts written",
        if release { "fetched public" } else { "editor" },
        if report.views_equal { "equal" } else { "different" },
        report.tool_exit
    );
    Ok(())
}
