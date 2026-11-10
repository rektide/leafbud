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
            if let Some(destination) = &item.destination {
                println!("{} -> {}", item.path.display(), destination.display());
            }
            if let Some(names) = &item.names {
                println!("names: {} -> {} ({})", names.disposition, item.naming_action, names.native);
            }
            if let Some(pair) = &item.proposed_names {
                println!("  selected pair: {:?} / {:?}", pair.file, pair.path);
            }
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
            "{} selected; {} normalized; {} unchanged; {} failed; {} need choice (pass: {})",
            report.selected, report.normalized, report.unchanged, report.failed, report.needs_choice, report.pass
        );
        if report.dry_run {
            println!("dry run: no files were written");
        }
    }
    let successes = report.normalized + report.unchanged;
    Ok(if report.items.iter().any(|item| item.outcome == "partial") {
        ExitCode::from(4)
    } else if report.failed == 0 && report.needs_choice == 0 {
        ExitCode::SUCCESS
    } else if report.failed == 0 && successes == 0 {
        ExitCode::from(3)
    } else if successes == 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::from(4)
    })
}
