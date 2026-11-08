//! Atomic local writes. New imports never replace an existing destination.
use anyhow::{Context, Result, bail};
use std::{io::Write, path::Path};

pub(crate) fn write(path: &Path, expected: Option<&[u8]>, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("destination has no parent")?;
    std::fs::create_dir_all(parent).context("creating destination directory")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).context("creating atomic write")?;
    temp.write_all(bytes).context("writing temporary file")?;
    if let Some(expected) = expected {
        let metadata = std::fs::symlink_metadata(path).context("checking original file")?;
        if !metadata.file_type().is_file() {
            bail!("refusing to replace a non-regular file");
        }
        temp.as_file().set_permissions(metadata.permissions())?;
        if std::fs::read(path).context("rechecking original file")? != expected {
            bail!("file changed during operation; not overwritten");
        }
        temp.as_file().sync_all()?;
        temp.persist(path).map_err(|e| e.error).context("atomically replacing unchanged file")?;
    } else {
        temp.as_file().sync_all()?;
        temp.persist_noclobber(path)
            .map_err(|e| e.error)
            .context("destination already exists or cannot be created")?;
    }
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}
