use leafbud_workflow::{
    RemoteOptions,
    config::{self, Config},
    native::{DeleteOptions, Selection},
};
use std::{path::Path, process::ExitCode};

pub async fn run(
    options: RemoteOptions,
    root: Option<&Path>,
    flags: &Config,
    file: &Path,
    delete: DeleteOptions,
    json: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = options
        .connect()
        .await?
        .delete_editor(&context, file, Selection { operands: vec![file.into()] }, delete)
        .await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "{} {} (document {}, lifecycle {}, token {}, record {})",
            report.outcome,
            report.path.display(),
            report.document_id,
            report.lifecycle,
            report.token,
            report.record.as_deref().unwrap_or("none"),
        );
        if let Some(message) = &report.message {
            eprintln!("{message}");
        }
    }
    Ok(match report.outcome {
        "would-delete" | "deleted" | "remote-deleted" | "unbound-local" => ExitCode::SUCCESS,
        "pending" | "blocked" | "uncertain" | "source-changed" => ExitCode::from(4),
        _ => ExitCode::FAILURE,
    })
}
