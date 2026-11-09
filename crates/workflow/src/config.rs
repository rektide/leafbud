//! Restored Figment2 defaults and comment-preserving config edits, without
//! reviving the old sidecar lifecycle. Credentials are not TOML settings.
use anyhow::{Context as _, Result, bail};
use figment2::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = "leafbud.toml";

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pds: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication: Option<String>,
    #[serde(default, rename = "diff-program", skip_serializing_if = "Option::is_none")]
    pub diff_program: Option<String>,
    #[serde(default, rename = "diff-args", skip_serializing_if = "Option::is_none")]
    pub diff_args: Option<Vec<String>>,
    #[serde(default, rename = "diff-exit", skip_serializing_if = "Option::is_none")]
    pub diff_exit: Option<crate::diff::ExitMode>,
}

impl Config {
    fn normalized(mut self) -> Self {
        if self.workspace.as_deref() == Some(Path::new("")) {
            self.workspace = None;
        }
        for value in [&mut self.handle, &mut self.pds, &mut self.publication] {
            if value.as_deref() == Some("") {
                *value = None;
            }
        }
        self
    }
    fn value(&self, key: Key) -> Option<String> {
        match key {
            Key::Workspace => self.workspace.as_ref().map(|v| v.display().to_string()),
            Key::Handle => self.handle.clone(),
            Key::Pds => self.pds.clone(),
            Key::Publication => self.publication.clone(),
            Key::DiffProgram => self.diff_program.clone(),
            Key::DiffArgs => self.diff_args.as_ref().map(|v| serde_json::to_string(v).expect("argument JSON")),
            Key::DiffExit => self.diff_exit.map(|v| {
                match v {
                    crate::diff::ExitMode::Success => "success",
                    crate::diff::ExitMode::Diff => "diff",
                }
                .into()
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Workspace,
    Handle,
    Pds,
    Publication,
    DiffProgram,
    DiffArgs,
    DiffExit,
}
impl Key {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Handle => "handle",
            Self::Pds => "pds",
            Self::Publication => "publication",
            Self::DiffProgram => "diff-program",
            Self::DiffArgs => "diff-args",
            Self::DiffExit => "diff-exit",
        }
    }
}
impl std::str::FromStr for Key {
    type Err = &'static str;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "workspace" => Ok(Self::Workspace),
            "handle" => Ok(Self::Handle),
            "pds" => Ok(Self::Pds),
            "publication" => Ok(Self::Publication),
            "diff-program" => Ok(Self::DiffProgram),
            "diff-args" => Ok(Self::DiffArgs),
            "diff-exit" => Ok(Self::DiffExit),
            _ => Err("expected workspace, handle, pds, publication, diff-program, diff-args or diff-exit"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShownValue {
    pub key: &'static str,
    pub value: Option<String>,
    pub source: Option<&'static str>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub version: u8,
    pub global_path: PathBuf,
    pub workspace_root: PathBuf,
    pub workspace_source: &'static str,
    pub workspace_toml_path: PathBuf,
    pub values: Vec<ShownValue>,
    #[serde(skip)]
    pub effective: Config,
}

pub fn global_path() -> Result<PathBuf> {
    let home = crate::config_home(std::env::var_os("XDG_CONFIG_HOME"), std::env::var_os("HOME"))?;
    Ok(home.join("leafbud").join(FILE_NAME))
}

fn read(path: &Path) -> Result<Option<(Config, Vec<u8>)>> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("reading config {}", path.display())),
    };
    let text = std::str::from_utf8(&bytes).context("config is not UTF-8")?;
    // Do not print Figment's raw parse error: unknown settings may be secrets.
    let config: Config = Figment::new().merge(Toml::string(text)).extract().map_err(|_| {
        anyhow::anyhow!(
            "invalid config {} (expected string defaults, diff-program string, diff-args string array, diff-exit success|diff)",
            path.display()
        )
    })?;
    Ok(Some((config.normalized(), bytes)))
}

/// Flags > unbound document intent > workspace > global. This only resolves
/// defaults: a bound document's actual identity/association must never use them.
pub fn load(root: Option<&Path>, flags: &Config, document: Option<&Config>) -> Result<Context> {
    load_at(&global_path()?, &std::env::current_dir()?, root, flags, document)
}

pub fn load_at(global_path: &Path, cwd: &Path, root: Option<&Path>, flags: &Config, document: Option<&Config>) -> Result<Context> {
    let global = read(global_path)?.map(|v| v.0).unwrap_or_default();
    let (root, workspace_source) = if let Some(root) = root {
        (root.to_path_buf(), "flag")
    } else if let Some(root) = &global.workspace {
        (
            if root.is_absolute() {
                root.clone()
            } else {
                global_path.parent().context("global config parent")?.join(root)
            },
            "global",
        )
    } else {
        (cwd.to_path_buf(), "cwd")
    };
    let root = if root.is_absolute() { root } else { cwd.join(root) };
    if !root.is_dir() {
        bail!("workspace root {} is not a directory", root.display());
    }
    let workspace_toml_path = root.join(FILE_NAME);
    let mut workspace = read(&workspace_toml_path)?.map(|v| v.0).unwrap_or_default();
    workspace.workspace = None;
    let mut document = document.cloned().unwrap_or_default().normalized();
    document.workspace = None;
    let mut flags = flags.clone().normalized();
    flags.workspace = None;
    let mut effective: Config = Figment::new()
        .merge(Serialized::defaults(&global))
        .merge(Serialized::defaults(&workspace))
        .merge(Serialized::defaults(&document))
        .merge(Serialized::defaults(&flags))
        .extract()
        .context("merging config layers")?;
    effective.workspace = Some(root.clone());
    let values = [
        Key::Workspace,
        Key::Handle,
        Key::Pds,
        Key::Publication,
        Key::DiffProgram,
        Key::DiffArgs,
        Key::DiffExit,
    ]
    .into_iter()
    .map(|key| {
        let (source, source_path) = if key == Key::Workspace {
            (Some(workspace_source), (workspace_source == "global").then(|| global_path.to_path_buf()))
        } else if flags.value(key).is_some() {
            (Some("flag"), None)
        } else if document.value(key).is_some() {
            (Some("document"), None)
        } else if workspace.value(key).is_some() {
            (Some("workspace"), Some(workspace_toml_path.clone()))
        } else if global.value(key).is_some() {
            (Some("global"), Some(global_path.to_path_buf()))
        } else {
            (None, None)
        };
        ShownValue {
            key: key.as_str(),
            value: effective.value(key),
            source,
            source_path,
        }
    })
    .collect();
    Ok(Context {
        version: 1,
        global_path: global_path.into(),
        workspace_root: root,
        workspace_source,
        workspace_toml_path,
        effective,
        values,
    })
}

pub fn set(key: Key, value: &str, workspace_file: bool, root: Option<&Path>) -> Result<PathBuf> {
    if value.trim().is_empty() {
        bail!("config value must not be empty");
    }
    if workspace_file && key == Key::Workspace {
        bail!("workspace belongs in global config, not a workspace file");
    }
    let target = if workspace_file {
        load(root, &Config::default(), None)?.workspace_toml_path
    } else {
        global_path()?
    };
    set_at(&target, key, value)?;
    Ok(target)
}

pub fn set_at(path: &Path, key: Key, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("config value must not be empty");
    }
    let old = read(path)?;
    let mut doc = match &old {
        Some((_, bytes)) => std::str::from_utf8(bytes)?
            .parse::<toml_edit::DocumentMut>()
            .map_err(|_| anyhow::anyhow!("invalid config; fix before setting keys"))?,
        None => toml_edit::DocumentMut::new(),
    };
    let decor = doc.get(key.as_str()).and_then(|v| v.as_value()).map(|v| v.decor().clone());
    doc[key.as_str()] = match key {
        Key::DiffArgs => {
            let args: Vec<String> = serde_json::from_str(value).map_err(|_| anyhow::anyhow!("diff-args requires a JSON array of strings, e.g. '[\"-u\"]'"))?;
            let mut array = toml_edit::Array::new();
            for arg in args {
                array.push(arg);
            }
            toml_edit::value(array)
        }
        Key::DiffExit => {
            value.parse::<crate::diff::ExitMode>().map_err(anyhow::Error::msg)?;
            toml_edit::value(value)
        }
        _ => toml_edit::value(value),
    };
    if let Some(decor) = decor {
        *doc[key.as_str()].as_value_mut().expect("new value").decor_mut() = decor;
    }
    let bytes = doc.to_string();
    if old.as_ref().is_some_and(|(_, old)| old == bytes.as_bytes()) {
        return Ok(());
    }
    crate::files::write(path, old.as_ref().map(|(_, v)| v.as_slice()), bytes.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diff_config_layers_and_preserving_typed_sets() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        std::fs::create_dir(&root).unwrap();
        let global = temp.path().join("global.toml");
        std::fs::write(
            &global,
            "diff-program = 'global viewer'\ndiff-args = ['--wait', 'two words']\ndiff-exit = 'diff'\n",
        )
        .unwrap();
        let local = root.join(FILE_NAME);
        std::fs::write(
            &local,
            "# preserve\nhandle = 'kept'\ndiff-args = ['old'] # arguments\nsecret = 'not emitted on errors'\n",
        )
        .unwrap();
        set_at(&local, Key::DiffArgs, r#"["-u", "two words", "$(literal)"]"#).unwrap();
        assert!(std::fs::read_to_string(&local).unwrap().contains("# arguments"));
        let context = load_at(&global, temp.path(), Some(&root), &Config::default(), None).unwrap();
        assert_eq!(context.effective.diff_program.as_deref(), Some("global viewer"));
        assert_eq!(context.effective.diff_args.unwrap(), vec!["-u", "two words", "$(literal)"]);
        assert_eq!(context.values.iter().find(|v| v.key == "diff-args").unwrap().source, Some("workspace"));
        let before = std::fs::read(&local).unwrap();
        for (key, bad) in [(Key::DiffArgs, "--not-json"), (Key::DiffArgs, "[1]"), (Key::DiffExit, "guess")] {
            assert!(set_at(&local, key, bad).is_err());
            assert_eq!(std::fs::read(&local).unwrap(), before);
        }
        let flags = Config {
            diff_program: Some("flag viewer".into()),
            diff_args: Some(vec![]),
            diff_exit: Some(crate::diff::ExitMode::Success),
            ..Default::default()
        };
        let loaded = load_at(&global, temp.path(), Some(&root), &flags, None).unwrap();
        assert_eq!(loaded.effective.diff_args, Some(vec![]));
        assert_eq!(loaded.effective.diff_program.as_deref(), Some("flag viewer"));
        assert_eq!(loaded.effective.diff_exit, Some(crate::diff::ExitMode::Success));
        assert!(crate::diff::Tool::from_config(&Config::default()).is_err());
        assert!(
            crate::diff::Tool::from_config(&Config {
                diff_program: Some(" ".into()),
                ..Default::default()
            })
            .is_err()
        );
        std::fs::write(&local, "diff-args = 'secret value not printed'\n").unwrap();
        let error = load_at(&global, temp.path(), Some(&root), &Config::default(), None).unwrap_err().to_string();
        assert!(!error.contains("secret value"));
    }
    #[test]
    fn layering_roots_and_provenance() {
        let temp = tempfile::tempdir().unwrap();
        let ws = temp.path().join("ws");
        std::fs::create_dir(&ws).unwrap();
        let global = temp.path().join(FILE_NAME);
        std::fs::write(
            &global,
            "workspace = 'ws'\nhandle = 'global.example'\npds = 'https://pds.example'\npublication = 'global-pub'\n",
        )
        .unwrap();
        std::fs::write(ws.join(FILE_NAME), "handle = 'workspace.example'\npublication = 'workspace-pub'\n").unwrap();
        let intent = Config {
            publication: Some("document-pub".into()),
            ..Config::default()
        };
        let flags = Config {
            handle: Some("flag.example".into()),
            ..Config::default()
        };
        let loaded = load_at(&global, temp.path(), None, &flags, Some(&intent)).unwrap();
        assert_eq!(loaded.workspace_root, ws);
        assert_eq!(loaded.effective.handle.as_deref(), Some("flag.example"));
        assert_eq!(loaded.effective.publication.as_deref(), Some("document-pub"));
        assert_eq!(loaded.effective.pds.as_deref(), Some("https://pds.example"));
        assert_eq!(loaded.values[3].source, Some("document"));
        assert_eq!(
            load_at(&global, temp.path(), Some(temp.path()), &Config::default(), None)
                .unwrap()
                .workspace_source,
            "flag"
        );
        std::fs::remove_file(&global).unwrap();
        assert_eq!(load_at(&global, &ws, None, &Config::default(), None).unwrap().workspace_source, "cwd");
    }
    #[test]
    fn preserving_writes_malformed_noop_and_race_safety() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join(FILE_NAME);
        let original = "# defaults\npublication = 'kept-pub'\nworkspace = 'old' # root\nsecret = 'never shown'\n[unknown]\nflag = true\n";
        std::fs::write(&file, original).unwrap();
        set_at(&file, Key::Workspace, "/new").unwrap();
        let changed = std::fs::read_to_string(&file).unwrap();
        assert!(changed.contains("# root"));
        assert!(changed.contains("publication = 'kept-pub'"));
        assert!(changed.contains("secret = 'never shown'"));
        assert!(changed.contains("[unknown]"));
        let modified = std::fs::metadata(&file).unwrap().modified().unwrap();
        set_at(&file, Key::Workspace, "/new").unwrap();
        assert_eq!(std::fs::metadata(&file).unwrap().modified().unwrap(), modified);
        assert!(crate::files::write(&file, Some(original.as_bytes()), b"bad overwrite").is_err());
        std::fs::write(&file, "handle = 42\n").unwrap();
        assert!(set_at(&file, Key::Handle, "new").is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "handle = 42\n");
    }
}
