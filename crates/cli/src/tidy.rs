//! Thin rendering for the local tidy passes (no network).
use leafbud_workflow::{
    config::{self, Config},
    tidy::Request,
};
use std::{path::Path, process::ExitCode};

/// Exit contract (accepted interface §1): 0 success/no-op, 1 operational
/// failure, 4 partial batch execution.
pub fn run(root: Option<&Path>, flags: &Config, request: Request, json: bool) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = leafbud_workflow::tidy::run(&context, request)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for item in &report.items {
            if let Some(error) = &item.error {
                eprintln!("{}: {} ({})", item.path.display(), item.outcome, error);
            }
            let keys = if item.keys_assigned > 0 {
                format!("{} keys assigned; ", item.keys_assigned)
            } else {
                String::new()
            };
            let skipped = if item.blocks_skipped > 0 {
                format!("; {} blocks kept exact ({})", item.blocks_skipped, item.skip_reasons.join(", "))
            } else {
                String::new()
            };
            println!(
                "{} {} ({}{} blocks normalized{skipped})",
                item.outcome,
                item.path.display(),
                keys,
                item.blocks_changed
            );
        }
        println!(
            "{} selected; {} normalized; {} unchanged; {} failed (pass: {})",
            report.selected, report.normalized, report.unchanged, report.failed, report.pass
        );
        if report.dry_run {
            println!("dry run: no files were written");
        }
    }
    let successes = report.normalized + report.unchanged;
    Ok(if report.failed == 0 {
        ExitCode::SUCCESS
    } else if successes == 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::from(4)
    })
}
