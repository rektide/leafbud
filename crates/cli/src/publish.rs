use leafbud_workflow::{
    RemoteOptions,
    config::{self, Config},
    native::{PublishOptions, Selection},
};
use std::{path::Path, process::ExitCode};

pub async fn run(
    options: RemoteOptions,
    root: Option<&Path>,
    flags: &Config,
    file: &Path,
    publish: PublishOptions,
    json: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = options
        .connect()
        .await?
        .publish_native(&context, file, Selection { operands: vec![file.into()] }, publish)
        .await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{} {} {:?}", report.outcome, file.display(), report.effects);
        if let Some(preview) = &report.preview {
            println!("{}", serde_json::to_string_pretty(preview)?);
        }
        if let Some(message) = &report.message {
            eprintln!("{message}");
        }
    }
    Ok(match report.outcome {
        "published" | "verified-published" | "unchanged" | "would-publish" | "would-bind-and-publish" | "editor-achieved-release-undispatched" => {
            ExitCode::SUCCESS
        }
        "blocked" | "uncertain" | "pending" | "source-changed" | "partial-unsupported" => ExitCode::from(4),
        _ => ExitCode::FAILURE,
    })
}
