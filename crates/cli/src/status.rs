use leafbud_workflow::{
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
    public_only: bool,
    json: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let remote = if offline || public_only { None } else { Some(options.connect().await?) };
    let public = if offline {
        None
    } else {
        Some(leafbud_workflow::release::PublicReader::new(flags.pds.as_deref())?)
    };
    let report = status::inspect_with_public(&context, request, remote.as_ref(), public.as_ref(), offline).await?;
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
            let preparation = if item.needs_preparation > 0 {
                format!("; {} objects need keys (tidy --keys)", item.needs_preparation)
            } else {
                String::new()
            };
            println!(
                "{} {} (local: {}; remote: {}, {}; baseline: {}{preparation})\n  {}",
                item.outcome,
                path,
                item.local.as_str(),
                item.remote.change.as_str(),
                item.remote.freshness,
                item.baseline,
                item.detail
            );
            if let Some(names) = &item.names {
                println!("  names: {} (file: {:?}; path: {:?})", names.disposition, names.file, names.path);
                if let Some(limitation) = &names.limitation {
                    println!("    {limitation}");
                }
                println!("    {}", names.native);
            }
            println!(
                "  release: {} (input: {}; output: {}; {}; baseline: {})",
                item.release.state,
                item.release.input.as_str(),
                item.release.output.as_str(),
                item.release.freshness,
                item.release.baseline
            );
        }
        println!(
            "{} selected; {} failed; read-only (no files, receipts or remotes written)",
            report.selected, report.failed
        );
        println!("Editor, names and release evidence are independent; read-only observations never accept changes");
    }
    Ok(if report.failed == 0 {
        ExitCode::SUCCESS
    } else if report.failed == report.selected {
        ExitCode::FAILURE
    } else {
        ExitCode::from(4)
    })
}
