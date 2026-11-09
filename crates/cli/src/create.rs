//! Thin rendering for offline `create` (no network).
use leafbud_workflow::{
    config::{self, Config},
    create::{self, Request},
};
use std::{path::Path, process::ExitCode};

pub fn run(root: Option<&Path>, flags: &Config, request: Request) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = create::run(&context, request)?;
    println!(
        "{} {} (document {}; publication intent: {})",
        report.outcome,
        report.path.display(),
        report.document_id,
        report.publication
    );
    Ok(ExitCode::SUCCESS)
}
