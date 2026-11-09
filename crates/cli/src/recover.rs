use leafbud_workflow::{
    RemoteOptions,
    config::{self, Config},
    native::{RecoverOptions, Selection},
};
use std::{path::Path, process::ExitCode};

pub async fn run(
    options: RemoteOptions,
    root: Option<&Path>,
    flags: &Config,
    file: &Path,
    recovery: RecoverOptions,
    json: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = options
        .connect()
        .await?
        .recover_editor(&context, file, Selection { operands: vec![file.into()] }, recovery)
        .await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "{} {} (document {}, attempt {:?}, binding known: {}, terminal: {})",
            report.outcome,
            report.path.display(),
            report.document_id,
            report.attempt_id,
            report.binding_known,
            report.observation.terminal
        );
        if let Some(message) = &report.message {
            eprintln!("{message}");
        }
    }
    Ok(match report.outcome {
        "no-attempt" | "settled-prior" | "would-settle-prior" | "accepted-remote" | "would-accept-remote" | "replanned-local" | "would-replan-local" => {
            ExitCode::SUCCESS
        }
        "pending" | "partial" | "source-changed" | "blocked" => ExitCode::from(4),
        _ => ExitCode::FAILURE,
    })
}
