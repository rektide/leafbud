//! Workspace: root resolution and sidecar state.
//!
//! The workspace is the canonical on-disk layout from
//! `design/records-canonical.glm53m.md`:
//!
//! ```text
//! <root>/
//!   drafts/               unposted canonical records (<slug>.json, pure)
//!   posted/               records as put (<slug>.json + <slug>.meta.json)
//!   springleaf.toml       identity defaults: handle, pds, publication
//! ```
//!
//! Root resolution: `--root` flag > the `workspace` key in the global
//! (XDG) config toml — read and layered by [`crate::config`] — > cwd.
//! (The workspace key must live there: a toml inside the workspace
//! cannot be found before the workspace is known.)
//!
//! Record files stay **pure** ATProto record JSON (no frontmatter, no
//! local fields); all local state lives in the sidecar
//! `<slug>.meta.json` written beside posted records.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::config::{self, Config};

/// A resolved workspace root.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: std::path::PathBuf,
}

impl Workspace {
    /// `explicit` (the `--root` flag) > the global layer's `workspace`
    /// key (relative values resolved against the config file's dir) >
    /// cwd. `global` is the already-read global config layer (see
    /// [`config::global_layer`]); tests pass an empty one so they never
    /// touch the machine's real config dir.
    pub fn resolve(explicit: Option<&std::path::Path>, global: &Config) -> Result<Self> {
        let root = match explicit {
            Some(path) => path.to_path_buf(),
            None => match config::workspace_key(global) {
                Some(path) => path,
                None => std::env::current_dir().context("resolving workspace root (cwd)")?,
            },
        };
        if !root.is_dir() {
            bail!("workspace root {} is not a directory", root.display());
        }
        Ok(Self { root })
    }

    pub fn drafts_dir(&self) -> std::path::PathBuf {
        self.root.join("drafts")
    }

    pub fn posted_dir(&self) -> std::path::PathBuf {
        self.root.join("posted")
    }

    /// Ensure drafts/ and posted/ exist (created on demand).
    pub fn ensure_layout(&self) -> Result<()> {
        std::fs::create_dir_all(self.drafts_dir()).with_context(|| format!("creating {}", self.drafts_dir().display()))?;
        std::fs::create_dir_all(self.posted_dir()).with_context(|| format!("creating {}", self.posted_dir().display()))?;
        Ok(())
    }
}

/// Sidecar state for a posted record (records-canonical §Layout): the
/// record file itself stays pure; everything local lives here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Sidecar {
    pub rkey: String,
    /// The record's at-uri (as returned by the last put / the remote).
    pub uri: String,
    /// The record cid as of the last put (`remoteCid` snapshot).
    pub cid: String,
    /// The publication at-uri the document belongs to (record `site`).
    pub publication: String,
    /// The record's own `publishedAt` field, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// When springleaf last put this record (null when adopted via
    /// `sync --pull`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_put_at: Option<String>,
    /// sha256 (hex) of the record file's bytes as of the last put.
    pub content_hash: String,
    /// The remote cid as last seen (listRecords or putRecord response).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_cid: Option<String>,
}

impl Sidecar {
    /// Path of the sidecar for `slug` inside `dir`.
    pub fn path_for(dir: &std::path::Path, slug: &str) -> std::path::PathBuf {
        dir.join(format!("{slug}.meta.json"))
    }
}

/// sha256 hex digest of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Best-effort unique temp dir with drop-cleanup for workspace tests.
#[cfg(test)]
pub(crate) struct TempDir(pub std::path::PathBuf);

#[cfg(test)]
impl TempDir {
    pub(crate) fn new(label: &str) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("springleaf-cli-{label}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }
}

#[cfg(test)]
impl Drop for TempDir {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).ok();
    }
}

#[cfg(test)]
impl std::ops::Deref for TempDir {
    type Target = std::path::Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// RFC3339 timestamp for "now" (sidecar fields).
pub fn now_rfc3339() -> String {
    jiff::Timestamp::now().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    /// The workspace-local toml is layered by [`crate::config`] (its
    /// parse coverage lives there); here: the layout + workspace-key
    /// root resolution through the global layer.
    #[test]
    fn workspace_layout_and_key_resolution() {
        let tmp = TempDir::new("ws-config");
        let ws = Workspace::resolve(Some(&tmp), &Config::default()).unwrap();
        ws.ensure_layout().unwrap();
        assert!(ws.drafts_dir().is_dir());
        assert!(ws.posted_dir().is_dir());

        // The global layer's workspace key resolves the root when no
        // --root flag is passed (relative resolution is covered in
        // config's tests).
        let root = tmp.join("keyed-ws");
        std::fs::create_dir_all(&root).unwrap();
        let global = Config {
            workspace: Some(root.clone()),
            ..Config::default()
        };
        let ws = Workspace::resolve(None, &global).unwrap();
        assert_eq!(ws.root, root);
        // unset key → cwd (resolves without error)
        assert!(Workspace::resolve(None, &Config::default()).is_ok());
    }

    #[test]
    fn workspace_root_must_be_a_directory() {
        let tmp = TempDir::new("ws-notdir");
        let missing = tmp.join("nope");
        assert!(Workspace::resolve(Some(&missing), &Config::default()).is_err());
    }

    #[test]
    fn sidecar_serializes_the_canonical_field_names() {
        let sidecar = Sidecar {
            rkey: "my-post".into(),
            uri: "at://did:plc:x/site.standard.document/my-post".into(),
            cid: "bafyreiput".into(),
            publication: "at://did:plc:x/site.standard.publication/3molsbwkh522o".into(),
            published_at: Some("2026-11-07T00:05:30.799Z".into()),
            last_put_at: None,
            content_hash: "cafe01".into(),
            remote_cid: Some("bafyreiput".into()),
        };
        let json = serde_json::to_value(&sidecar).unwrap();
        // Exactly the eight sidecar keys from records-canonical §Layout
        // (lastPutAt omitted when null).
        let keys: Vec<&str> = json.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["rkey", "uri", "cid", "publication", "publishedAt", "contentHash", "remoteCid"]);
        // Round-trips.
        let back: Sidecar = serde_json::from_value(json).unwrap();
        assert_eq!(back, sidecar);
    }

    #[test]
    fn sha256_known_vector() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
