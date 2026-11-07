//! `config` — inspect and edit the layered springleaf.toml files.
//!
//! `config show` renders the effective layered values and which file
//! each came from (the layering itself is `config.rs`'s; show adds no
//! flag layer — flags are per-invocation and not persisted config).
//! The workspace is resolved the usual way (global `workspace` key,
//! else cwd), so the shown workspace toml is the one other commands
//! would read.
//!
//! `config set <key> <value>` writes one key to the file that key
//! belongs to: the global toml by default, or the workspace toml with
//! `--workspace-file` (valid for `handle`/`pds`/`publication` only —
//! the `workspace` key necessarily lives in the global file). Figment
//! is read-only, so the write path is deliberately separate:
//! toml_edit, comment- and formatting-preserving. A malformed file is
//! never silently rewritten — `config set` refuses it loudly, matching
//! the read posture in `config.rs`.
//!
//! Exit codes follow the CLI convention: 0 ok, 1 operational error,
//! 2 usage error (empty value, `workspace` + `--workspace-file`).

use std::path::Path;

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::Serialize;

use crate::config::{self, Config};
use crate::workspace::Workspace;

/// The settable keys (a clap value enum so unknown keys get clap's
/// listing error for free).
#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ConfigKey {
    Workspace,
    Handle,
    Pds,
    Publication,
}

impl ConfigKey {
    fn as_str(self) -> &'static str {
        match self {
            ConfigKey::Workspace => "workspace",
            ConfigKey::Handle => "handle",
            ConfigKey::Pds => "pds",
            ConfigKey::Publication => "publication",
        }
    }
}

/// One `config show` row: the effective value and the layer that
/// supplied it (`None` = unset).
#[derive(Serialize)]
struct ShownValue {
    key: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    /// "workspace" | "global" | absent when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<&'static str>,
    /// The file the value came from (absent when unset).
    #[serde(skip_serializing_if = "Option::is_none")]
    source_path: Option<String>,
}

/// The `config show --json` schema (version 1).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigShow {
    version: u8,
    global_path: String,
    workspace_root: String,
    workspace_toml_path: String,
    values: Vec<ShownValue>,
}

/// The layered config snapshot `config show` renders.
struct Snapshot {
    global_path: std::path::PathBuf,
    workspace_root: std::path::PathBuf,
    workspace_toml: std::path::PathBuf,
    workspace_via_key: bool,
    values: Vec<ShownValue>,
}

/// Read the global layer, resolve the workspace, and stack the layers
/// with per-key origins. Loud on malformed files (the read posture).
fn snapshot() -> Result<Snapshot> {
    let global_path = config::xdg_config_toml_path()?;
    let global = config::global_layer()?;
    let workspace_via_key = global.workspace.is_some();
    let ws = Workspace::resolve(None, &global)?;
    let workspace_toml = ws.root.join(config::CONFIG_FILE_NAME);
    let workspace_layer = config::layer_from_path(&workspace_toml)?;
    let effective = config::stack(&Config::default(), workspace_layer.as_ref(), &global)?;

    // Per-key origin: the highest layer that supplied it.
    let origin = |ws_value: Option<&str>, global_value: Option<&str>| match (ws_value, global_value) {
        (Some(_), _) => Some(("workspace", workspace_toml.display().to_string())),
        (None, Some(_)) => Some(("global", global_path.display().to_string())),
        (None, None) => None,
    };
    let row = |key: &'static str, effective: Option<String>, origin: Option<(&'static str, String)>| ShownValue {
        key,
        value: effective,
        source: origin.as_ref().map(|(source, _)| *source),
        source_path: origin.map(|(_, path)| path),
    };

    let workspace_key_display = global.workspace.as_ref().map(|p| p.display().to_string());
    let values = vec![
        row("workspace", workspace_key_display.clone(), origin(None, workspace_key_display.as_deref())),
        row(
            "handle",
            effective.handle.clone(),
            origin(workspace_layer.as_ref().and_then(|l| l.handle.as_deref()), global.handle.as_deref()),
        ),
        row(
            "pds",
            effective.pds.clone(),
            origin(workspace_layer.as_ref().and_then(|l| l.pds.as_deref()), global.pds.as_deref()),
        ),
        row(
            "publication",
            effective.publication.clone(),
            origin(workspace_layer.as_ref().and_then(|l| l.publication.as_deref()), global.publication.as_deref()),
        ),
    ];
    Ok(Snapshot {
        global_path,
        workspace_root: ws.root,
        workspace_toml,
        workspace_via_key,
        values,
    })
}

/// `config show`: the effective layered config and each value's origin.
pub(crate) fn config_show_cmd(json: bool) -> Result<i32> {
    let snap = snapshot()?;
    if json {
        let show = ConfigShow {
            version: 1,
            global_path: snap.global_path.display().to_string(),
            workspace_root: snap.workspace_root.display().to_string(),
            workspace_toml_path: snap.workspace_toml.display().to_string(),
            values: snap.values,
        };
        println!("{}", serde_json::to_string_pretty(&show)?);
    } else {
        print!("{}", show_text(&snap));
    }
    Ok(0)
}

/// Human rendering: header (both files + the workspace root), then a
/// KEY | VALUE | SOURCE table.
fn show_text(snap: &Snapshot) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "global:     {}", snap.global_path.display());
    let _ = writeln!(
        out,
        "workspace:  {} ({})",
        snap.workspace_root.display(),
        if snap.workspace_via_key {
            "global `workspace` key"
        } else {
            "cwd fallback — no `workspace` key set"
        }
    );
    let key_width = snap.values.iter().map(|v| v.key.len()).max().unwrap_or(3).max(3);
    let value_width = snap.values.iter().map(|v| v.value.as_deref().map_or(1, str::len)).max().unwrap_or(1).max(5);
    let _ = writeln!(out, "\n{:<key_width$} {:<value_width$} SOURCE", "KEY", "VALUE");
    for value in &snap.values {
        let _ = writeln!(
            out,
            "{:<key_width$} {:<value_width$} {}",
            value.key,
            value.value.as_deref().unwrap_or("-"),
            match (&value.source, &value.source_path) {
                (Some(source), Some(path)) => format!("{source} ({path})"),
                _ => "-".to_string(),
            }
        );
    }
    out
}

/// `config set`: write one key to the file it belongs to.
pub(crate) fn config_set_cmd(key: ConfigKey, value: String, workspace_file: bool) -> Result<i32> {
    if value.trim().is_empty() {
        eprintln!("error: value must not be empty — empty counts as unset; edit the toml to remove a key");
        return Ok(2);
    }
    let target = if workspace_file {
        if key == ConfigKey::Workspace {
            eprintln!(
                "error: the `workspace` key cannot live in a workspace toml — \
                 the workspace must be found before its toml can be read"
            );
            return Ok(2);
        }
        let global = config::global_layer()?;
        let ws = Workspace::resolve(None, &global)?;
        ws.root.join(config::CONFIG_FILE_NAME)
    } else {
        config::xdg_config_toml_path()?
    };
    set_key(&target, key.as_str(), &value)?;
    println!("set {} = {value} in {}", key.as_str(), target.display());
    Ok(0)
}

/// Set `key = value` in the toml at `path` with toml_edit — comments
/// and sibling keys preserved. Creates the file (and its parent dirs)
/// when absent; refuses a malformed file loudly rather than rewriting
/// it under the operator.
fn set_key(path: &Path, key: &str, value: &str) -> Result<()> {
    let mut doc = match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse::<toml_edit::DocumentMut>()
            .with_context(|| format!("parsing config {} — fix it before setting keys", path.display()))?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => toml_edit::DocumentMut::new(),
        Err(err) => return Err(anyhow::Error::new(err).context(format!("reading config {}", path.display()))),
    };
    // A same-line trailing comment belongs to the value's decor; carry
    // it across the replacement instead of dropping it with the old
    // value (comments above the line already survive on their own).
    let suffix = doc
        .get_mut(key)
        .and_then(|item| item.as_value_mut())
        .and_then(|value| value.decor().suffix().and_then(|suffix| suffix.as_str().map(str::to_string)));
    doc[key] = toml_edit::value(value);
    if let Some(suffix) = suffix
        && let Some(new_value) = doc[key].as_value_mut()
    {
        new_value.decor_mut().set_suffix(suffix);
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(path, doc.to_string()).with_context(|| format!("writing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::with_env_vars;
    use crate::workspace::TempDir;
    use serde_json::json;

    /// A machine-independent world: XDG_CONFIG_HOME points at a temp
    /// dir holding the global toml (with a `workspace` key), and the
    /// workspace toml lives in the pointed-at root.
    struct World {
        config_home: TempDir,
        ws_root: std::path::PathBuf,
    }

    impl World {
        fn new(global_toml: &str, workspace_toml: Option<&str>) -> Self {
            let config_home = TempDir::new("cfg-cmd");
            let springleaf_dir = config_home.join("springleaf");
            std::fs::create_dir_all(&springleaf_dir).unwrap();
            std::fs::write(springleaf_dir.join(config::CONFIG_FILE_NAME), global_toml).unwrap();
            // `workspace = "ws"` (the usual fixture) points at <home>/ws
            let ws_root = springleaf_dir.join("ws");
            if let Some(body) = workspace_toml {
                std::fs::create_dir_all(&ws_root).unwrap();
                std::fs::write(ws_root.join(config::CONFIG_FILE_NAME), body).unwrap();
            }
            Self { config_home, ws_root }
        }

        fn with<R>(&self, f: impl FnOnce() -> R) -> R {
            with_env_vars(&[("XDG_CONFIG_HOME", Some(self.config_home.to_str().unwrap()))], f)
        }
    }

    #[test]
    fn show_reports_origins_per_key() {
        let world = World::new(
            "workspace = \"ws\"\nhandle = \"global.example\"\npds = \"https://pds.example\"\n",
            Some("handle = \"ws.example\"\n"),
        );
        world.with(|| {
            let snap = snapshot().unwrap();
            assert_eq!(snap.workspace_root, world.ws_root);
            let by_key = |key: &str| snap.values.iter().find(|v| v.key == key).unwrap_or_else(|| panic!("no {key} row"));
            // workspace toml wins for handle
            let handle = by_key("handle");
            assert_eq!(handle.value.as_deref(), Some("ws.example"));
            assert_eq!(handle.source, Some("workspace"));
            assert_eq!(
                handle.source_path.as_deref(),
                Some(world.ws_root.join(config::CONFIG_FILE_NAME).display().to_string().as_str())
            );
            // global supplies pds and the workspace key itself
            assert_eq!(
                (by_key("pds").value.clone(), by_key("pds").source),
                (Some("https://pds.example".into()), Some("global"))
            );
            assert_eq!(by_key("workspace").source, Some("global"));
            // unset stays unset
            assert_eq!((by_key("publication").value.clone(), by_key("publication").source), (None, None));
            // human rendering names files and sources
            let text = show_text(&snap);
            assert!(text.contains("global `workspace` key"), "{text}");
            assert!(text.contains("ws.example"), "{text}");
            assert!(text.contains("workspace ("), "{text}");
            assert!(text.contains("global ("), "{text}");
        });
    }

    #[test]
    fn show_cwd_fallback_without_workspace_key() {
        let world = World::new("handle = \"global.example\"\n", None);
        world.with(|| {
            let snap = snapshot().unwrap();
            assert!(!snap.workspace_via_key);
            let text = show_text(&snap);
            assert!(text.contains("cwd fallback"), "{text}");
            // the workspace toml under cwd is simply absent → no layer
            assert_eq!(snap.values.iter().find(|v| v.key == "handle").unwrap().source, Some("global"));
        });
    }

    #[test]
    fn set_creates_the_global_file_and_reads_back() {
        let world = World::new("", None);
        world.with(|| {
            let code = config_set_cmd(ConfigKey::Handle, "h.example".into(), false).unwrap();
            assert_eq!(code, 0);
            let global = config::global_layer().unwrap();
            assert_eq!(global.handle.as_deref(), Some("h.example"));
            // created under the overridden config home, not the machine's
            assert!(world.config_home.join("springleaf").join(config::CONFIG_FILE_NAME).is_file());
        });
    }

    #[test]
    fn set_preserves_comments_and_sibling_keys() {
        let world = World::new("# identity defaults\nhandle = \"old.example\" # the handle\n", None);
        world.with(|| {
            let code = config_set_cmd(ConfigKey::Handle, "new.example".into(), false).unwrap();
            assert_eq!(code, 0);
            let text = std::fs::read_to_string(world.config_home.join("springleaf").join(config::CONFIG_FILE_NAME)).unwrap();
            assert!(text.contains("# identity defaults"), "leading comment survives: {text}");
            assert!(text.contains("# the handle"), "inline comment survives: {text}");
            assert!(!text.contains("old.example"), "{text}");
            assert!(text.contains("new.example"), "{text}");
            // and parses back with the new value
            assert_eq!(config::global_layer().unwrap().handle.as_deref(), Some("new.example"));
        });
    }

    #[test]
    fn set_refuses_malformed_files() {
        let world = World::new("handle = \"unterminated\n", None);
        world.with(|| {
            let err = config_set_cmd(ConfigKey::Handle, "h.example".into(), false).unwrap_err();
            let text = format!("{err:#}");
            assert!(text.contains("fix it before setting keys"), "{text}");
            // the malformed bytes are untouched
            let raw = std::fs::read_to_string(world.config_home.join("springleaf").join(config::CONFIG_FILE_NAME)).unwrap();
            assert_eq!(raw, "handle = \"unterminated\n");
        });
    }

    #[test]
    fn set_routing_rules() {
        let world = World::new("workspace = \"ws\"\n", Some(""));
        // empty value → exit 2, nothing written
        assert_eq!(config_set_cmd(ConfigKey::Handle, " ".into(), false).unwrap(), 2);
        // workspace key + --workspace-file → exit 2 (cannot bootstrap)
        assert_eq!(config_set_cmd(ConfigKey::Workspace, "/tmp/ws".into(), true).unwrap(), 2);
        // --workspace-file writes the resolved workspace's toml
        world.with(|| {
            assert_eq!(
                config_set_cmd(ConfigKey::Publication, "at://did:plc:x/site.standard.publication/p".into(), true).unwrap(),
                0
            );
            let ws_toml = config::layer_from_path(&world.ws_root.join(config::CONFIG_FILE_NAME)).unwrap().unwrap();
            assert_eq!(ws_toml.publication.as_deref(), Some("at://did:plc:x/site.standard.publication/p"));
        });
    }

    #[test]
    fn show_json_schema_shape() {
        let world = World::new(
            "workspace = \"ws\"\nhandle = \"global.example\"\n",
            Some("publication = \"at://did:plc:x/site.standard.publication/p\"\n"),
        );
        world.with(|| {
            let snap = snapshot().unwrap();
            let show = ConfigShow {
                version: 1,
                global_path: snap.global_path.display().to_string(),
                workspace_root: snap.workspace_root.display().to_string(),
                workspace_toml_path: snap.workspace_toml.display().to_string(),
                values: snap.values,
            };
            let value = serde_json::to_value(&show).unwrap();
            assert_eq!(value["version"], json!(1));
            let handle = &value["values"].as_array().unwrap().iter().find(|v| v["key"] == json!("handle")).unwrap();
            assert_eq!(handle["value"], json!("global.example"));
            assert_eq!(handle["source"], json!("global"));
            let publication = &value["values"].as_array().unwrap().iter().find(|v| v["key"] == json!("publication")).unwrap();
            assert_eq!(publication["source"], json!("workspace"));
        });
    }
}
