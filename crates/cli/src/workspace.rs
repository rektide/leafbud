//! Workspace: root resolution, config parsing, sidecar state.
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
//! Root resolution: `--root` flag > the `workspace` key in the XDG config
//! toml (`$XDG_CONFIG_HOME/springleaf/springleaf.toml`, else
//! `~/.config/springleaf/`) > cwd. (The workspace key must live there —
//! a toml inside the workspace cannot be found before the workspace is
//! known.)
//!
//! Record files stay **pure** ATProto record JSON (no frontmatter, no
//! local fields); all local state lives in the sidecar
//! `<slug>.meta.json` written beside posted records.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// A resolved workspace root.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: std::path::PathBuf,
}

impl Workspace {
    pub fn resolve(explicit: Option<&std::path::Path>) -> Result<Self> {
        let root = match explicit {
            Some(path) => path.to_path_buf(),
            None => match workspace_key_from_xdg_config() {
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

    /// The workspace-local `springleaf.toml`, if present.
    pub fn config(&self) -> WorkspaceConfig {
        config_from_path(&self.root.join("springleaf.toml"))
    }
}

/// Identity defaults from `<root>/springleaf.toml` (hand-parsed flat
/// `key = "value"` lines — no toml crate, per the lean-deps posture).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceConfig {
    pub handle: Option<String>,
    pub pds: Option<String>,
    pub publication: Option<String>,
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

/// The springleaf XDG config dir: `$XDG_CONFIG_HOME/springleaf`, else
/// `~/.config/springleaf` — home of the optional `springleaf.toml`
/// (identity defaults + the `workspace` key) and the `auth-token` file.
pub fn xdg_config_dir() -> Result<std::path::PathBuf> {
    let dir = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(xdg) if !xdg.is_empty() => std::path::PathBuf::from(xdg),
        _ => {
            let home = std::env::var_os("HOME").context("config lookup: $HOME unset and no $XDG_CONFIG_HOME")?;
            std::path::PathBuf::from(home).join(".config")
        }
    };
    Ok(dir.join("springleaf"))
}

/// Path of the optional XDG config toml:
/// `$XDG_CONFIG_HOME/springleaf/springleaf.toml`, else
/// `~/.config/springleaf/springleaf.toml` — the same file `pubs list`
/// reads for its handle default and the only place a `workspace` key can
/// be found before the workspace itself is known.
pub fn xdg_config_toml_path() -> Result<std::path::PathBuf> {
    Ok(xdg_config_dir()?.join("springleaf.toml"))
}

/// The `workspace` key from the XDG config toml, if any. Relative paths
/// resolve against the config file's directory.
pub fn workspace_key_from_xdg_config() -> Option<std::path::PathBuf> {
    let path = xdg_config_toml_path().ok()?;
    let value = toml_value_at(&path, "workspace")?;
    let as_path = std::path::PathBuf::from(&value);
    Some(if as_path.is_absolute() {
        as_path
    } else {
        path.parent().unwrap_or(std::path::Path::new(".")).join(as_path)
    })
}

/// Parse `<root>/springleaf.toml` into a [`WorkspaceConfig`].
pub fn config_from_path(path: &std::path::Path) -> WorkspaceConfig {
    WorkspaceConfig {
        handle: toml_value_at(path, "handle"),
        pds: toml_value_at(path, "pds"),
        publication: toml_value_at(path, "publication"),
    }
}

/// Manual flat-toml value extraction for `key = "value"` (or unquoted
/// values): the same two-line-per-key parse `pubs list` uses for
/// `handle`, generalized. Comments (`#`) and unknown keys are ignored;
/// `None` when the file is absent or the key is missing/empty.
pub fn toml_value_at(path: &std::path::Path, key: &str) -> Option<String> {
    let contents = std::fs::read_to_string(path).ok()?;
    toml_value_in(&contents, key)
}

pub fn toml_value_in(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some(rest) = line.strip_prefix(key) else { continue };
        let rest = rest.trim_start();
        let Some(value) = rest.strip_prefix('=') else { continue };
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
            .unwrap_or(value);
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_key_parse_shapes() {
        let body = "# comment\nhandle = \"jauntywunderkind.bsky.social\"\npds = 'https://puffball.us-east.host.bsky.network'\npublication = at://did:plc:x/site.standard.publication/3molsbwkh522o\nworkspace = /tmp/ws\n";
        assert_eq!(toml_value_in(body, "handle").as_deref(), Some("jauntywunderkind.bsky.social"));
        assert_eq!(toml_value_in(body, "pds").as_deref(), Some("https://puffball.us-east.host.bsky.network"));
        assert_eq!(
            toml_value_in(body, "publication").as_deref(),
            Some("at://did:plc:x/site.standard.publication/3molsbwkh522o")
        );
        assert_eq!(toml_value_in(body, "workspace").as_deref(), Some("/tmp/ws"));
        // Prefix collisions must not match (`handleway` is not `handle`).
        assert_eq!(toml_value_in("handleway = x\n", "handle"), None);
        assert_eq!(toml_value_in("handle = \"\"\n", "handle"), None);
        assert_eq!(toml_value_in("", "handle"), None);
    }

    #[test]
    fn workspace_config_and_layout() {
        let tmp = TempDir::new("ws-config");
        let ws = Workspace::resolve(Some(&tmp)).unwrap();
        std::fs::write(
            ws.root.join("springleaf.toml"),
            "handle = h.example\npublication = at://did:plc:x/site.standard.publication/3molsbwkh522o\n",
        )
        .unwrap();
        let config = ws.config();
        assert_eq!(config.handle.as_deref(), Some("h.example"));
        assert_eq!(config.publication.as_deref(), Some("at://did:plc:x/site.standard.publication/3molsbwkh522o"));
        assert_eq!(config.pds, None);
        ws.ensure_layout().unwrap();
        assert!(ws.drafts_dir().is_dir());
        assert!(ws.posted_dir().is_dir());
    }

    #[test]
    fn workspace_root_must_be_a_directory() {
        let tmp = TempDir::new("ws-notdir");
        let missing = tmp.join("nope");
        assert!(Workspace::resolve(Some(&missing)).is_err());
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
