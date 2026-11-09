use leafbud_workflow::{
    author,
    config::{self, Config},
};
use std::{path::Path, process::ExitCode};

pub enum Operation<'a> {
    Edit(author::MetadataEdit),
    Rename(&'a str, &'a str),
}

pub fn run(root: Option<&Path>, flags: &Config, file: &Path, operation: Operation<'_>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let report = match operation {
        Operation::Edit(edit) => author::edit(&context, file, &edit)?,
        Operation::Rename(old, new) => author::rename(&context, file, old, new)?,
    };
    println!(
        "{} {} (document {}; local only)",
        if report.changed { "updated" } else { "unchanged" },
        report.path.display(),
        report.document_id
    );
    Ok(ExitCode::SUCCESS)
}
