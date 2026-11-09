use leafbud_workflow::{
    config::{self, Config},
    list,
};
use std::{path::Path, process::ExitCode};

pub fn run(root: Option<&Path>, flags: &Config, request: list::Request, json: bool) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = list::inspect(&context, request)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for item in &report.items {
            let native = item
                .binding
                .as_ref()
                .map(|b| {
                    format!(
                        "{} / {}",
                        b.base_url.as_deref().unwrap_or("unknown origin"),
                        b.token.as_deref().or(b.uri.as_deref()).unwrap_or("editing unavailable")
                    )
                })
                .unwrap_or_else(|| "unbound".into());
            // Debug quoting escapes authored controls/newlines in a one-line inventory.
            println!(
                "{} {} — {:?} [{}; {}; {}] (document {}; {}){}{}",
                item.outcome,
                item.path.strip_prefix(&report.workspace).unwrap_or(&item.path).display(),
                item.title.as_deref().unwrap_or("unknown title"),
                item.lifecycle.as_deref().unwrap_or("unknown state"),
                item.format.as_deref().unwrap_or("unknown format"),
                item.receipt_profile.as_deref().unwrap_or("no receipt"),
                item.document_id.map(|id| id.to_string()).unwrap_or_else(|| "unknown UUID".into()),
                native,
                if item.pending { " [pending]" } else { "" },
                if item.last_resolution { " [last resolution retained]" } else { "" }
            );
            for diagnostic in &item.diagnostics {
                eprintln!("{}: {diagnostic}", item.path.display());
            }
        }
        println!(
            "{} managed candidates; {} listed; {} filtered; {} failed/unsupported/ambiguous; {} unmanaged ignored (local only)",
            report.examined,
            report.items.len(),
            report.filtered,
            report.failed,
            report.ignored_unmanaged
        );
    }
    Ok(if report.failed == 0 {
        ExitCode::SUCCESS
    } else if report.failed == report.items.len() {
        ExitCode::FAILURE
    } else {
        ExitCode::from(4)
    })
}
