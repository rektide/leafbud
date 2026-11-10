//! One shared read-only naming policy and pure local reconciliation plan.
use anyhow::{Context, Result, bail, ensure};
use leafbud_document::{Document, NamePath, Names};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Filename,
    Path,
    Accept,
}

#[derive(Clone, Debug, Serialize)]
pub struct Evidence {
    pub disposition: &'static str,
    pub file: Option<String>,
    /// None means unsupported author-field type, not a missing author field.
    pub path: Option<NamePath>,
    pub accepted: Option<Names>,
    pub file_changed: Option<bool>,
    pub path_changed: Option<bool>,
    pub mapped_file: Option<String>,
    pub limitation: Option<String>,
    pub native: &'static str,
}

const NATIVE: &str = "local intent only; native public path/identity is unchanged and custom native path mutation is not implemented";

pub(crate) fn mapped_file(path: &str) -> Result<String> {
    Names::file_for_path(path).map_err(anyhow::Error::msg)
}

fn file_intent(file: &str) -> Result<String> {
    Names::path_for_file(file).map_err(anyhow::Error::msg)
}

pub(crate) fn coordinate(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .context("naming requires a file inside the workspace; external key/Markdown passes remain available")?;
    let file = relative.to_str().context("naming requires a UTF-8 workspace coordinate")?;
    file_intent(file)?;
    Ok(file.into())
}

pub fn inspect(root: &Path, file: &Path, document: &Document) -> Evidence {
    let path = match NamePath::from_author(&document.extra) {
        Ok(path) => path,
        Err(error) => {
            return Evidence {
                disposition: "unsupported",
                file: None,
                path: None,
                accepted: document.state.names.clone(),
                file_changed: None,
                path_changed: None,
                mapped_file: None,
                limitation: Some(error.into()),
                native: NATIVE,
            };
        }
    };
    let coordinate = coordinate(root, file);
    let mapped = match &path {
        NamePath::Value(value) => mapped_file(value).map(Some),
        _ => Ok(None),
    };
    let limitation = coordinate.as_ref().err().or_else(|| mapped.as_ref().err()).map(|e| format!("{e:#}"));
    let file = coordinate.ok();
    let mapped_file = mapped.ok().flatten();
    let accepted = document.state.names.clone();
    let disposition = if file.is_none() {
        "unsupported"
    } else if !matches!(path, NamePath::Value(_)) {
        "unassigned"
    } else {
        let equal = file == mapped_file;
        match &accepted {
            None => {
                if equal {
                    "equal"
                } else {
                    "untracked"
                }
            }
            Some(before) => {
                let file_changed = Some(&before.file) != file.as_ref();
                let path_changed = before.path != path;
                match (file_changed, path_changed, equal) {
                    (false, false, true) => "equal",
                    (false, false, false) => "accepted-mismatch",
                    (_, _, true) => "converged",
                    (true, false, false) => "file-changed",
                    (false, true, false) => "path-changed",
                    (true, true, false) => "both-changed",
                }
            }
        }
    };
    let file_changed = accepted.as_ref().and_then(|before| file.as_ref().map(|file| before.file != *file));
    let path_changed = accepted.as_ref().map(|before| before.path != path);
    Evidence {
        disposition,
        file,
        path: Some(path),
        accepted,
        file_changed,
        path_changed,
        mapped_file,
        limitation,
        native: NATIVE,
    }
}

pub(crate) struct Plan {
    pub evidence: Evidence,
    pub proposed: Option<Names>,
    pub destination: PathBuf,
    pub source: String,
    pub action: &'static str,
}

pub(crate) fn plan(root: &Path, file: &Path, source: &str, direction: Option<Direction>) -> Result<Plan> {
    let document = leafbud_mdx::parse(source)?;
    ensure!(
        document.state.pending.is_none() && document.state.publication_attempt.is_none(),
        "pending native attempt; recover before naming"
    );
    let current_path = NamePath::from_author(&document.extra).map_err(anyhow::Error::msg)?;
    let evidence = inspect(root, file, &document);
    let selected = match direction {
        Some(direction) => Some(direction),
        None => match evidence.disposition {
            "unassigned" => None,
            "equal" | "converged" => Some(Direction::Accept),
            "accepted-mismatch" => None,
            "file-changed" => Some(Direction::Filename),
            "path-changed" => Some(Direction::Path),
            "unsupported" if current_path == NamePath::Missing && evidence.accepted.is_none() => None,
            _ => bail!(
                "names need choice ({}); use --from filename|path or --accept; {}",
                evidence.disposition,
                evidence.limitation.as_deref().unwrap_or(NATIVE)
            ),
        },
    };
    let Some(selected) = selected else {
        return Ok(Plan {
            proposed: document.state.names,
            evidence,
            destination: file.into(),
            source: source.into(),
            action: "unchanged",
        });
    };
    let current_file = coordinate(root, file)?;
    let (target_file, path_edit, path) = match selected {
        Direction::Filename => {
            let path = file_intent(&current_file)?;
            (current_file, Some(path.clone()), NamePath::Value(path))
        }
        Direction::Path => {
            let NamePath::Value(path) = &current_path else {
                bail!("author path is unassigned; --from path cannot invent one");
            };
            (mapped_file(path)?, None, current_path.clone())
        }
        Direction::Accept => (current_file, None, current_path),
    };
    let names = Names {
        file: target_file.clone(),
        path,
    };
    let output = leafbud_mdx::patch_names(source, path_edit.as_deref(), &names)?;
    Ok(Plan {
        proposed: Some(names),
        evidence,
        destination: root.join(target_file),
        source: output,
        action: match selected {
            Direction::Filename => "from-filename",
            Direction::Path => "from-path",
            Direction::Accept => "accept",
        },
    })
}

/// Refuse symlink directory aliases even when they point back inside root.
pub(crate) fn destination(root: &Path, target: &Path) -> Result<()> {
    let relative = target.strip_prefix(root).context("destination is outside workspace")?;
    let mut parent = root.to_path_buf();
    let parts: Vec<_> = relative.components().collect();
    for part in &parts[..parts.len().saturating_sub(1)] {
        ensure!(matches!(part, std::path::Component::Normal(_)), "unsafe destination component");
        parent.push(part);
        let metadata = std::fs::symlink_metadata(&parent).context("destination directory must already exist (tidy never creates it)")?;
        ensure!(
            metadata.file_type().is_dir(),
            "destination parent is not a real directory (symlinks are unsupported)"
        );
    }
    ensure!(std::fs::canonicalize(&parent)? == parent, "destination directory route changed");
    ensure!(
        std::fs::symlink_metadata(target).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
        "destination already exists (including dangling symlinks); no overwrite"
    );
    Ok(())
}
