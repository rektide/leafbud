//! Cooperative operation locks, not content storage or stale-owner files.
//! Files deliberately remain: unlinking a locked inode would split the lock.
//! All callers use the same user config directory, independent of workspace/path.
use anyhow::{Context, Result};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub(crate) struct Store(PathBuf);
pub(crate) struct Guard(File);

impl Store {
    pub fn new(context: &crate::config::Context) -> Result<Self> {
        Ok(Self(context.global_path.parent().context("config has no parent")?.join("locks")))
    }

    pub fn document(&self, id: Uuid) -> Result<Option<Guard>> {
        self.acquire(&format!("document-{id}"))
    }

    /// Serializes discovery/creation for one remote binding, including imports
    /// before they know an existing locally assigned document UUID.
    pub fn binding(&self, base: &str, token: Uuid) -> Result<Option<Guard>> {
        use sha2::{Digest, Sha256};
        self.acquire(&format!("binding-{:x}", Sha256::digest(format!("{base}/{token}"))))
    }

    fn acquire(&self, name: &str) -> Result<Option<Guard>> {
        std::fs::create_dir_all(&self.0).context("creating cooperative lock directory")?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.0.join(format!("{name}.lock")))
            .context("opening cooperative lock")?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Guard(file))),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(std::fs::TryLockError::Error(error)) => Err(error).context("acquiring cooperative lock"),
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

pub(crate) fn source_id(source: &str) -> Result<Uuid> {
    let header = leafbud_mdx::header(source).context("reading managed identity")?;
    header["@leafbud"]["id"]
        .as_str()
        .context("managed document UUID missing")?
        .parse()
        .context("invalid managed document UUID")
}

/// Reread after acquisition; path/identity changes never borrow the old lock.
pub(crate) fn read(path: &Path, store: &Store) -> Result<(Guard, Vec<u8>, String)> {
    let before = std::fs::read_to_string(path).context("reading managed file identity")?;
    let id = source_id(&before)?;
    let guard = store.document(id)?.context("document busy in another Leafbud operation; retry later")?;
    let bytes = std::fs::read(path).context("reading locked managed file")?;
    let source = String::from_utf8(bytes.clone()).context("managed file is not UTF-8")?;
    anyhow::ensure!(source_id(&source)? == id, "document identity changed while acquiring lock; retry");
    Ok((guard, bytes, source))
}
