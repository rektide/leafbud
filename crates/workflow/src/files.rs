//! Atomic local writes. New imports never replace an existing destination.
use anyhow::{Context, Result, bail};
use std::{io::Write, path::Path};

#[cfg(unix)]
pub(crate) fn identity(path: &Path) -> Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(metadata.file_type().is_file(), "source is not a regular file");
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
pub(crate) fn identity(path: &Path) -> Result<std::time::SystemTime> {
    Ok(std::fs::symlink_metadata(path)?.created()?)
}

pub(crate) fn rename_noclobber(source: &Path, destination: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        rustix::fs::renameat_with(rustix::fs::CWD, source, rustix::fs::CWD, destination, rustix::fs::RenameFlags::NOREPLACE)
            .context("no-clobber rename failed; target/cross-device moves are never overwritten or copied")?;
        // The caller reports the new path if witness settlement fails. Do not
        // hide a successful rename behind a subsequent directory sync error.
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (source, destination);
        anyhow::bail!("naming moves require Linux no-replace rename; no overwrite fallback")
    }
}

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

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[test]
    fn no_replace_rename_primitive_never_overwrites_even_identical_bytes_or_links() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        let target = root.path().join("target");
        std::fs::write(&source, "same bytes").unwrap();
        std::fs::write(&target, "same bytes").unwrap();
        let id = super::identity(&target).unwrap();
        assert!(super::rename_noclobber(&source, &target).is_err());
        assert_eq!(super::identity(&target).unwrap(), id);
        assert!(source.exists());
        std::fs::remove_file(&target).unwrap();
        std::os::unix::fs::symlink(root.path().join("absent"), &target).unwrap();
        assert!(super::rename_noclobber(&source, &target).is_err());
        assert!(std::fs::symlink_metadata(&target).unwrap().is_symlink());
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "same bytes");
    }
}
