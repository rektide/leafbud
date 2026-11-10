//! Create one local unbound MDX draft. Offline: no remote object, token,
//! public path or publication date is invented; publication intent is
//! resolved from explicit input only (omitted/inherit, explicit standalone,
//! or a target URI) and stays available for first binding later.
use crate::{config, files};
use anyhow::{Context, Result, ensure};
use leafbud_document::{Document, FORMAT, Page, Publication, State};
use serde::Serialize;
use std::path::{Component, PathBuf};
use uuid::Uuid;

pub struct Request {
    pub file: PathBuf,
    pub title: String,
    /// Explicit first-binding intent; `None` inherits configuration later.
    pub publication: Publication,
}

#[derive(Serialize)]
pub struct Report {
    pub version: u8,
    pub path: PathBuf,
    pub document_id: Uuid,
    pub publication: String,
    pub outcome: &'static str,
}

pub fn run(context: &config::Context, request: Request) -> Result<Report> {
    ensure!(!request.title.trim().is_empty(), "document title must not be empty");
    let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
    let path = if request.file.is_absolute() {
        request.file.clone()
    } else {
        root.join(&request.file)
    };
    ensure!(path.extension().is_some_and(|e| e == "mdx"), "create destination must end in .mdx");
    let relative = path.strip_prefix(&root).context("create destination must be inside the document workspace")?;
    ensure!(
        relative.components().all(|c| matches!(c, Component::Normal(_))),
        "unsafe create destination path"
    );
    let mut cursor = root.clone();
    for part in relative.components() {
        cursor.push(part);
        if let Ok(metadata) = std::fs::symlink_metadata(&cursor) {
            ensure!(!metadata.file_type().is_symlink(), "create destination contains a symlink");
        }
    }
    let document = Document {
        format: FORMAT.into(),
        title: request.title.trim().to_owned(),
        description: None,
        tags: vec![],
        publication: request.publication.clone(),
        theme: None,
        extra: Default::default(),
        pages: vec![Page {
            key: leafbud_mdx::PROVISIONAL_MAIN_PAGE.into(),
            blocks: vec![],
        }],
        // A fresh draft has a stable random local UUID, no binding, no
        // receipts and no persisted main-page key: preparation assigns
        // main-page identity once, when the document first has content.
        state: State {
            names: None,
            last_resolution: None,
            writer: None,
            pending: None,
            id: Uuid::new_v4(),
            state: "draft".into(),
            main_page: None,
            binding: None,
            nodes: Default::default(),
            sync: None,
        },
    };
    let source = leafbud_mdx::render(&document).context("rendering the created draft")?;
    // No-clobber creation: an existing file is never replaced.
    files::write(&path, None, source.as_bytes()).with_context(|| format!("creating {}", path.display()))?;
    Ok(Report {
        version: 1,
        path,
        document_id: document.state.id,
        publication: match &request.publication {
            Publication::Inherit => "inherit".into(),
            Publication::Standalone => "standalone".into(),
            Publication::At(uri) => format!("uri:{uri}"),
        },
        outcome: "created",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn context(root: &Path) -> crate::config::Context {
        crate::config::load_at(&root.join("absent.toml"), root, Some(root), &crate::config::Config::default(), None).unwrap()
    }

    fn request(file: &str, title: &str, publication: Publication) -> Request {
        Request {
            file: file.into(),
            title: title.into(),
            publication,
        }
    }

    #[test]
    fn created_drafts_are_stable_unbound_and_checkable() {
        let root = tempfile::tempdir().unwrap();
        let report = run(&context(root.path()), request("drafts/essay.mdx", "An essay", Publication::Inherit)).unwrap();
        assert_eq!(report.outcome, "created");
        let source = std::fs::read_to_string(&report.path).unwrap();
        let parsed = leafbud_mdx::parse(&source).unwrap();
        assert_eq!(parsed.state.id, report.document_id, "the reported UUID is the persisted identity");
        assert!(parsed.state.binding.is_none());
        assert!(parsed.state.sync.is_none());
        assert!(parsed.state.main_page.is_none(), "main-page identity waits for preparation");
        assert_eq!(parsed.publication, Publication::Inherit);
        assert!(parsed.pages[0].blocks.is_empty(), "an empty implicit main page is valid");
        // Explicit standalone intent survives the round-trip.
        run(&context(root.path()), request("drafts/standalone.mdx", "Solo", Publication::Standalone)).unwrap();
        let parsed = leafbud_mdx::parse(&std::fs::read_to_string(root.path().join("drafts/standalone.mdx")).unwrap()).unwrap();
        assert_eq!(parsed.publication, Publication::Standalone);
        // Repeated creation never clobbers.
        let before = std::fs::read(&report.path).unwrap();
        assert!(run(&context(root.path()), request("drafts/essay.mdx", "Other", Publication::Inherit)).is_err());
        assert_eq!(std::fs::read(&report.path).unwrap(), before);
        // Refuse odd destinations.
        assert!(run(&context(root.path()), request("escape.mdx", "X", Publication::Inherit)).is_ok());
        assert!(run(&context(root.path()), request("../outside.mdx", "X", Publication::Inherit)).is_err());
        assert!(run(&context(root.path()), request("notes.md", "X", Publication::Inherit)).is_err());
        assert!(run(&context(root.path()), request("x.mdx", "  ", Publication::Inherit)).is_err());
    }
}
