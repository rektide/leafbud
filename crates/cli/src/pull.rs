use leafbud_workflow::{
    RemoteOptions,
    config::{self, Config},
    pull::Request,
};
use std::{path::Path, process::ExitCode};

/// Exit contract (accepted interface §1): 0 success/no-op, 1 operational
/// failure, 3 the operation needs a decision, 4 partial/diverged execution.
pub async fn run(options: RemoteOptions, root: Option<&Path>, flags: &Config, request: Request, json: bool) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = options.connect().await?.pull(&context, request).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for item in &report.items {
            if let Some(error) = &item.error {
                eprintln!("{}: {} ({})", item.path.display(), item.outcome, error);
            }
            println!("{} {}", item.outcome, item.path.display());
        }
        println!(
            "{} selected; {} skipped; {} unchanged; {} updated; {} receipts-only; {} converged; {} needs-decision; {} failed",
            report.selected, report.skipped, report.unchanged, report.updated, report.receipts_only, report.converged, report.needs_decision, report.failed
        );
        if report.dry_run {
            println!("dry run: no files were written");
        }
    }
    let successes = report.unchanged + report.updated + report.receipts_only + report.converged;
    Ok(if report.failed == 0 && report.needs_decision == 0 {
        ExitCode::SUCCESS
    } else if report.failed > 0 && successes == 0 && report.needs_decision == 0 {
        ExitCode::FAILURE
    } else if report.needs_decision > 0 && successes == 0 && report.failed == 0 {
        ExitCode::from(3)
    } else {
        ExitCode::from(4)
    })
}
