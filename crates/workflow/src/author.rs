//! Offline explicit authoring operations. One shared stable-ID lock, full
//! inventory ambiguity check and adjacent original-byte write guard.
use crate::{config, files, inventory, lock};
use anyhow::{Context, Result, ensure};
pub use leafbud_mdx::MetadataEdit;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Report {
    pub path: PathBuf,
    pub document_id: uuid::Uuid,
    pub changed: bool,
}

pub fn edit(context: &config::Context, file: &Path, edit: &MetadataEdit) -> Result<Report> {
    apply(context, file, |source| leafbud_mdx::edit_metadata(source, edit))
}

pub fn rename(context: &config::Context, file: &Path, old: &str, new: &str) -> Result<Report> {
    apply(context, file, |source| leafbud_mdx::rename_key(source, old, new))
}

fn apply(context: &config::Context, file: &Path, prepare: impl FnOnce(&str) -> Result<String>) -> Result<Report> {
    let joined = if file.is_absolute() {
        file.to_owned()
    } else {
        context.workspace_root.join(file)
    };
    let path = std::fs::canonicalize(&joined).context("resolving local authoring operand")?;
    let store = lock::Store::new(context)?;
    let (_guard, original, source) = lock::read(&path, &store)?;
    let check_duplicates = || -> Result<()> {
        let mut inventory = inventory::scan(&context.workspace_root, None)?;
        inventory.include_operands(std::slice::from_ref(&path));
        ensure!(
            !inventory.duplicates.contains(&path),
            "duplicate local UUID/native binding; resolve the copies before editing"
        );
        Ok(())
    };
    check_duplicates()?;
    let proposed = prepare(&source)?;
    let changed = proposed != source;
    if changed {
        check_duplicates()?;
        ensure!(std::fs::canonicalize(&joined)? == path, "operand path changed during preparation; retry");
        files::write(&path, Some(&original), proposed.as_bytes()).context("source changed during local authoring; not overwritten")?;
    }
    Ok(Report {
        path,
        document_id: lock::source_id(&source)?,
        changed,
    })
}

#[cfg(test)]
mod tests;
