use leafspring_workflow::{
    RemoteOptions,
    config::{self, Config},
    status::{self, Request},
};
use std::{path::Path, process::ExitCode};

pub async fn run(
    options: RemoteOptions,
    root: Option<&Path>,
    flags: &Config,
    request: Request,
    offline: bool,
    json: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let remote = if offline { None } else { Some(options.connect().await?) };
    let report = status::inspect(&context, request, remote.as_ref()).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for item in &report.items {
            let path: String = item
                .path
                .to_string_lossy()
                .chars()
                .flat_map(|c| if c.is_control() { c.escape_default().collect::<Vec<_>>() } else { vec![c] })
                .collect();
            println!(
                "{} {} (local: {}; remote: {}, {}; baseline: {})\n  {}",
                item.outcome,
                path,
                item.local.as_str(),
                item.remote.change.as_str(),
                item.remote.freshness,
                item.baseline,
                item.detail
            );
        }
        println!(
            "{} selected; {} failed; read-only (no files, receipts or remotes written)",
            report.selected, report.failed
        );
        println!("Names: not tracked; release: unknown (not implemented); local-only metadata: not compared to editor");
    }
    Ok(if report.failed == 0 {
        ExitCode::SUCCESS
    } else if report.failed == report.selected {
        ExitCode::FAILURE
    } else {
        ExitCode::from(4)
    })
}
