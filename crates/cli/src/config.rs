//! Layered config — figment2 providers stacked by precedence.
//!
//! A springleaf.toml is a flat key set: `workspace`, `handle`, `pds`,
//! `publication` (unknown keys ignored). Two files can supply them:
//! the global toml at `$XDG_CONFIG_HOME/springleaf/springleaf.toml`
//! (else `~/.config/springleaf/springleaf.toml`) and the workspace
//! toml at `<root>/springleaf.toml` (see
//! `design/records-canonical2.glm53m.md`). Resolution is stacked
//! figment layers — later merges win — not hand-chained fallbacks:
//!
//! ```text
//! publication   --publication flag > workspace toml > global toml
//! handle        --handle flag      > workspace toml > global toml
//!                                    ($ATPROTO_HANDLE stays sync's
//!                                    final fallback, read at the edge)
//! workspace     --root resolves the root directly; else the global
//!                toml key; else cwd (a workspace toml cannot be found
//!                before the workspace is known)
//! ```
//!
//! Error posture — a deliberate change from the hand parser, which
//! never failed: an absent file or key is simply unset, but a present
//! file that fails to parse (or types a known key wrongly) is a loud
//! error naming the file, never a silent fallback. A typo'd config
//! silently shadowing defaults is a footgun in a publishing tool.
//!
//! Relative `workspace` values resolve against the config file's
//! directory. No secrets flow through here: the `auth-token` and
//! `session.json` files live beside the global toml but are read by
//! their own paths (see main's token resolution, atproto's store).
//!
//! Figment is read-only by design; this module only reads and layers.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use figment2::Figment;
use figment2::providers::{Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

/// The config file name in both locations.
pub const CONFIG_FILE_NAME: &str = "springleaf.toml";

/// Flat config values: one springleaf.toml layer, or the merged view.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Workspace root (global layer only — it is how the workspace is
    /// found). Relative values resolve against the config file's dir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pds: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication: Option<String>,
}

impl Config {
    /// The flag layer, fed in as the highest layer programmatically
    /// (flags exist for `handle` and `publication`; `--root` resolves
    /// the workspace directly and `pds` has no flag).
    pub fn from_flags(handle: Option<String>, publication: Option<String>) -> Self {
        Self {
            handle,
            publication,
            ..Self::default()
        }
    }
}

/// The springleaf XDG config dir: `$XDG_CONFIG_HOME/springleaf`, else
/// `~/.config/springleaf` — home of the optional `springleaf.toml`
/// (identity defaults + the `workspace` key) and the `auth-token` file.
pub fn xdg_config_dir() -> Result<PathBuf> {
    let dir = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(xdg) if !xdg.is_empty() => PathBuf::from(xdg),
        _ => {
            let home = std::env::var_os("HOME").context("config lookup: $HOME unset and no $XDG_CONFIG_HOME")?;
            PathBuf::from(home).join(".config")
        }
    };
    Ok(dir.join("springleaf"))
}

/// Path of the global config toml (the workspace toml lives at
/// `<root>/springleaf.toml`).
pub fn xdg_config_toml_path() -> Result<PathBuf> {
    Ok(xdg_config_dir()?.join(CONFIG_FILE_NAME))
}

/// The global (XDG) config layer: an absent file is an empty layer; a
/// malformed one is a loud error (see the module docs' posture).
pub fn global_layer() -> Result<Config> {
    Ok(layer_from_path(&xdg_config_toml_path()?)?.unwrap_or_default())
}

/// Parse one toml file as a [`Config`] layer: `None` when the file is
/// absent, loud — file path plus the parse/type error — when present
/// but malformed.
pub fn layer_from_path(path: &Path) -> Result<Option<Config>> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(anyhow::Error::new(err).context(format!("reading config {}", path.display())));
        }
    };
    let layer: Config = Figment::new()
        .merge(Toml::string(&text))
        .extract()
        .with_context(|| format!("parsing config {} (flat keys: workspace, handle, pds, publication)", path.display()))?;
    Ok(Some(unset_empty_values(layer)))
}

/// The effective config for a resolved workspace root: **flags >
/// workspace toml > global toml** (later figment merges win). The
/// workspace toml is read here — loud if malformed.
pub fn layered(flags: &Config, workspace_root: Option<&Path>, global: &Config) -> Result<Config> {
    let workspace_layer = match workspace_root {
        Some(root) => layer_from_path(&root.join(CONFIG_FILE_NAME))?,
        None => None,
    };
    stack(flags, workspace_layer.as_ref(), global)
}

/// The pure figment stack over already-parsed layers (the layering
/// matrix, testable without touching disk). `config show` also uses it
/// to attribute each effective value to the layer that supplied it.
pub(crate) fn stack(flags: &Config, workspace_layer: Option<&Config>, global: &Config) -> Result<Config> {
    let mut figment = Figment::new().merge(Serialized::defaults(global.clone()));
    if let Some(workspace) = workspace_layer {
        figment = figment.merge(Serialized::defaults(workspace.clone()));
    }
    figment
        .merge(Serialized::defaults(flags.clone()))
        .extract()
        .context("merging springleaf config layers")
}

/// The resolved `workspace` key from the global layer: relative values
/// resolve against the config file's directory (the hand parser's
/// rule, preserved). `None` when unset or the config dir is
/// unknowable — the caller falls back to cwd.
pub fn workspace_key(global: &Config) -> Option<PathBuf> {
    let raw = global.workspace.as_ref()?;
    let config_dir = xdg_config_toml_path().ok()?.parent()?.to_path_buf();
    Some(resolve_workspace_key(raw, &config_dir))
}

/// Pure resolution rule: absolute wins, relative joins onto the config
/// file's directory.
fn resolve_workspace_key(raw: &Path, config_dir: &Path) -> PathBuf {
    if raw.is_absolute() { raw.to_path_buf() } else { config_dir.join(raw) }
}

/// The hand parser treated an explicitly empty value as absent; keep
/// that rule so `handle = ""` cannot shadow a deeper layer.
fn unset_empty_values(mut config: Config) -> Config {
    if config.workspace.as_deref() == Some(Path::new("")) {
        config.workspace = None;
    }
    if config.handle.as_deref() == Some("") {
        config.handle = None;
    }
    if config.pds.as_deref() == Some("") {
        config.pds = None;
    }
    if config.publication.as_deref() == Some("") {
        config.publication = None;
    }
    config
}

/// Run `f` with env vars set, restoring prior values afterwards.
/// Guarded by a process-wide lock so parallel `cargo test` runs are
/// safe (nextest runs each test per-process; the lock covers the plain
/// runner).
#[cfg(test)]
pub(crate) fn with_env_vars<R>(sets: &[(&str, Option<&str>)], f: impl FnOnce() -> R) -> R {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let saved: Vec<(&str, Option<String>)> = sets.iter().map(|(key, _)| (*key, std::env::var(key).ok())).collect();
    for (key, value) in sets {
        match value {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }
    }
    let out = f();
    for (key, value) in saved {
        match value {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::TempDir;

    fn write(path: &std::path::Path, body: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn absent_file_is_not_a_layer() {
        let tmp = TempDir::new("cfg-absent");
        assert_eq!(layer_from_path(&tmp.join(CONFIG_FILE_NAME)).unwrap(), None);
    }

    #[test]
    fn layer_reads_flat_keys_and_ignores_unknown() {
        let tmp = TempDir::new("cfg-flat");
        let path = tmp.join(CONFIG_FILE_NAME);
        write(
            &path,
            "# leading comment\n\
             handle = \"jauntywunderkind.bsky.social\" # trailing comment\n\
             pds = \"https://puffball.us-east.host.bsky.network\"\n\
             publication = \"at://did:plc:x/site.standard.publication/3molsbwkh522o\"\n\
             workspace = \"/tmp/ws\"\n\
             unknown-key = \"ignored\"\n\
             [unknown-table]\n\
             nested = true\n",
        );
        let layer = layer_from_path(&path).unwrap().unwrap();
        // The trailing comment is stripped by real toml parsing — the
        // hand parser kept the quotes and the comment in the value.
        assert_eq!(layer.handle.as_deref(), Some("jauntywunderkind.bsky.social"));
        assert_eq!(layer.pds.as_deref(), Some("https://puffball.us-east.host.bsky.network"));
        assert_eq!(layer.publication.as_deref(), Some("at://did:plc:x/site.standard.publication/3molsbwkh522o"));
        assert_eq!(layer.workspace, Some(PathBuf::from("/tmp/ws")));
    }

    #[test]
    fn layer_is_loud_on_malformed_toml() {
        let tmp = TempDir::new("cfg-malformed");
        let path = tmp.join(CONFIG_FILE_NAME);
        write(&path, "handle = \"unterminated\n");
        let err = layer_from_path(&path).unwrap_err();
        let text = format!("{err:#}");
        assert!(text.contains(&path.display().to_string()), "names the file: {text}");
        assert!(text.contains("parsing config"), "{text}");
        assert!(text.contains("line"), "carries the parser's position: {text}");
    }

    #[test]
    fn layer_is_loud_on_wrongly_typed_keys() {
        let tmp = TempDir::new("cfg-typed");
        let path = tmp.join(CONFIG_FILE_NAME);
        write(&path, "handle = 42\n");
        let err = layer_from_path(&path).unwrap_err();
        let text = format!("{err:#}");
        assert!(text.contains(&path.display().to_string()), "names the file: {text}");
    }

    #[test]
    fn layer_treats_explicitly_empty_values_as_unset() {
        let tmp = TempDir::new("cfg-empty");
        let path = tmp.join(CONFIG_FILE_NAME);
        write(&path, "handle = \"\"\nworkspace = \"\"\npds = \"\"\npublication = \"\"\n");
        let layer = layer_from_path(&path).unwrap().unwrap();
        assert_eq!(layer, Config::default());
    }

    /// The layering matrix: flag > workspace toml > global toml, per
    /// key, with fall-through for keys a layer leaves unset.
    #[test]
    fn stack_flag_over_workspace_over_global() {
        let global = Config {
            workspace: Some(PathBuf::from("/tmp/g-ws")),
            handle: Some("global.example".into()),
            pds: Some("https://pds.global.example".into()),
            publication: Some("at://did:plc:g/site.standard.publication/global".into()),
        };
        let workspace = Config {
            handle: Some("workspace.example".into()),
            publication: Some("at://did:plc:w/site.standard.publication/workspace".into()),
            ..Config::default()
        };

        // Every key set everywhere: flags win.
        let flags = Config::from_flags(Some("flag.example".into()), Some("at://did:plc:f/site.standard.publication/flag".into()));
        let layered = stack(&flags, Some(&workspace), &global).unwrap();
        assert_eq!(layered.handle.as_deref(), Some("flag.example"));
        assert_eq!(layered.publication.as_deref(), Some("at://did:plc:f/site.standard.publication/flag"));
        // untouched-by-higher-layers keys fall through
        assert_eq!(layered.pds.as_deref(), Some("https://pds.global.example"));
        assert_eq!(layered.workspace, Some(PathBuf::from("/tmp/g-ws")));

        // No flags: the workspace layer overrides the global one —
        // the operator's explicit requirement.
        let layered = stack(&Config::default(), Some(&workspace), &global).unwrap();
        assert_eq!(layered.handle.as_deref(), Some("workspace.example"));
        assert_eq!(layered.publication.as_deref(), Some("at://did:plc:w/site.standard.publication/workspace"));
        assert_eq!(layered.pds.as_deref(), Some("https://pds.global.example"));

        // No workspace layer: global values pass through.
        let layered = stack(&Config::default(), None, &global).unwrap();
        assert_eq!(layered, global);

        // A layer that leaves a key unset does not clobber the deeper
        // value (None never serializes into the stack).
        let sparse = Config {
            publication: Some("at://did:plc:s/site.standard.publication/sparse".into()),
            ..Config::default()
        };
        let layered = stack(&Config::default(), Some(&sparse), &global).unwrap();
        assert_eq!(layered.handle.as_deref(), Some("global.example"));
        assert_eq!(layered.publication.as_deref(), Some("at://did:plc:s/site.standard.publication/sparse"));
    }

    #[test]
    fn layered_reads_the_workspace_toml_from_the_root() {
        let tmp = TempDir::new("cfg-ws-root");
        let root = tmp.join("ws");
        let toml = root.join(CONFIG_FILE_NAME);
        write(&toml, "publication = \"at://did:plc:w/site.standard.publication/w\"\n");

        let effective = layered(&Config::default(), Some(&root), &Config::default()).unwrap();
        assert_eq!(effective.publication.as_deref(), Some("at://did:plc:w/site.standard.publication/w"));

        // A malformed workspace toml is loud and names its file.
        write(&toml, "publication = \"oops\n");
        let err = layered(&Config::default(), Some(&root), &Config::default()).unwrap_err();
        let text = format!("{err:#}");
        assert!(text.contains(&toml.display().to_string()), "{text}");
        assert!(text.contains("parsing config"), "{text}");

        // No root (no workspace context): global only.
        let global = Config {
            handle: Some("global.example".into()),
            ..Config::default()
        };
        let merged = layered(&Config::default(), None, &global).unwrap();
        assert_eq!(merged.handle.as_deref(), Some("global.example"));
    }

    #[test]
    fn workspace_key_resolution_rules() {
        // absolute wins
        assert_eq!(resolve_workspace_key(Path::new("/abs/ws"), Path::new("/cfg/dir")), PathBuf::from("/abs/ws"));
        // relative joins onto the config file's directory
        assert_eq!(
            resolve_workspace_key(Path::new("rel/ws"), Path::new("/cfg/dir")),
            PathBuf::from("/cfg/dir/rel/ws")
        );
    }

    /// The XDG override end to end: `global_layer()` reads from
    /// `$XDG_CONFIG_HOME` (never the machine's real config dir), and
    /// the `workspace` key resolves against that same file's dir.
    #[test]
    fn global_layer_and_workspace_key_honor_xdg_config_home() {
        let config_home = TempDir::new("cfg-xdg-home");
        let other_home = TempDir::new("cfg-xdg-alt-home");
        let toml = config_home.join("springleaf").join(CONFIG_FILE_NAME);
        write(&toml, "workspace = \"rel-ws\"\nhandle = \"h.example\"\n");

        with_env_vars(
            &[
                ("XDG_CONFIG_HOME", Some(config_home.to_str().unwrap())),
                ("HOME", Some(other_home.to_str().unwrap())),
            ],
            || {
                let global = global_layer().unwrap();
                assert_eq!(global.handle.as_deref(), Some("h.example"));
                assert_eq!(workspace_key(&global).as_deref(), Some(config_home.join("springleaf").join("rel-ws").as_path()));
                // no workspace key → None (caller falls back to cwd)
                write(&toml, "handle = \"h.example\"\n");
                let global = global_layer().unwrap();
                assert_eq!(workspace_key(&global), None);
            },
        );

        // No $XDG_CONFIG_HOME: ~/.config/springleaf/springleaf.toml.
        with_env_vars(&[("XDG_CONFIG_HOME", None), ("HOME", Some(other_home.to_str().unwrap()))], || {
            let path = xdg_config_toml_path().unwrap();
            assert_eq!(path, other_home.join(".config").join("springleaf").join(CONFIG_FILE_NAME));
        });
    }
}
