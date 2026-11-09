use super::*;
use leafbud_document::{Binding, Block, BlockKind, Document, EDITOR_PROFILE, FORMAT, Inline, Mark, Node, Occurrence, Page, Publication, State, Sync, Uuid};
use std::collections::BTreeMap;

fn document() -> Document {
    let state = State {
        id: Uuid::nil(),
        state: "draft".into(),
        main_page: Some("p-main".into()),
        binding: Some(Binding {
            base_url: "https://leaflet.pub".into(),
            token: Uuid::nil(),
            root: "e-root".into(),
            entity_set: "es".into(),
            entry_fact: "f-entry".into(),
            publication: None,
        }),
        nodes: BTreeMap::from([
            (
                "p-main".into(),
                Node {
                    entity: "e-main".into(),
                    occurrence: Occurrence::Page {
                        references: vec!["f-entry".into()],
                    },
                },
            ),
            (
                "b-1".into(),
                Node {
                    entity: "e-b1".into(),
                    occurrence: Occurrence::Block {
                        placement: "f-b1".into(),
                        text_fact: None,
                        inactive_text_fact: None,
                    },
                },
            ),
        ]),
        sync: Some(Sync {
            profile: EDITOR_PROFILE.into(),
            local_hash: "sha256:old-local".into(),
            remote_hash: "sha256:old-remote".into(),
        }),
    };
    Document {
        format: FORMAT.into(),
        title: "T".into(),
        description: None,
        tags: vec![],
        publication: Publication::Standalone,
        theme: None,
        extra: BTreeMap::new(),
        pages: vec![Page {
            key: "p-main".into(),
            blocks: vec![Block {
                key: "b-1".into(),
                kind: BlockKind::Paragraph {
                    content: vec![Inline::Run {
                        text: "plain".into(),
                        marks: vec![],
                    }],
                },
            }],
        }],
        state,
    }
}

/// A managed file whose only block is hand-shaped as a literal wrapper
/// around a link run, with author comments and a local-only field.
fn authored_wrapped(root: &Path) -> PathBuf {
    let path = root.join("drafts").join("doc.mdx");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut source = document();
    source.pages[0].blocks[0].kind = BlockKind::Paragraph {
        content: vec![Inline::Run {
            text: "guide".into(),
            marks: vec![Mark::Link {
                href: "https://e.org/docs".into(),
            }],
        }],
    };
    let markdown_body = "[guide](https://e.org/docs)";
    let rendered = leafbud_mdx::render(&source).unwrap();
    let source = rendered
        .replacen("\"title\": \"T\"\n", "title: T # author comment\n", 1)
        .replacen("\n\"@leafbud\":", "\ncustomField: {\"meaning\": \"retained\"}\n\n\"@leafbud\":", 1)
        .replacen(
            &format!("<Key id=\"b-1\" />\n{markdown_body}\n"),
            "<Key id=\"b-1\" />\n<Paragraph>\n<Run text={\"guide\"} marks={[{\"kind\":\"link\",\"href\":\"https://e.org/docs\"}]} />\n\n</Paragraph>\n",
            1,
        );
    std::fs::write(&path, source).unwrap();
    path
}

fn context(root: &Path) -> config::Context {
    config::load_at(&root.join("absent.toml"), root, Some(root), &config::Config::default(), None).unwrap()
}

fn read(path: &Path) -> String {
    String::from_utf8(std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn tidy_normalizes_preserving_comments_fields_and_witnesses() {
    let root = tempfile::tempdir().unwrap();
    let path = authored_wrapped(root.path());
    let before = read(&path);
    assert!(before.contains("<Paragraph>"));
    let report = run(
        &context(root.path()),
        Request {
            files: vec![path.clone()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.selected, 1);
    assert_eq!(report.normalized, 1);
    let after = read(&path);
    assert!(after.contains("[guide](https://e.org/docs)"));
    assert!(!after.contains("<Paragraph>"));
    assert!(after.contains("title: T # author comment"), "author comments survive");
    assert!(after.contains("customField"), "local-only fields survive");
    assert!(after.contains("sha256:old-local"), "witness bytes survive");
    assert!(after.contains("<Key id=\"b-1\" />"));
    // Repeat: already tidy.
    let repeat = run(
        &context(root.path()),
        Request {
            files: vec![path.clone()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(repeat.unchanged, 1);
    assert_eq!(read(&path), after, "idempotent second run writes nothing");
}

#[test]
fn tidy_dry_run_previews_without_writing() {
    let root = tempfile::tempdir().unwrap();
    let path = authored_wrapped(root.path());
    let before = read(&path);
    let report = run(
        &context(root.path()),
        Request {
            files: vec![path.clone()],
            dry_run: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.normalized, 1);
    assert!(report.dry_run);
    assert_eq!(read(&path), before, "dry run never writes");
}

#[test]
fn tidy_all_selects_managed_files_only() {
    let root = tempfile::tempdir().unwrap();
    let managed = authored_wrapped(root.path());
    std::fs::write(root.path().join("notes.md"), "plain markdown, not managed").unwrap();
    std::fs::write(root.path().join("other.txt"), "not an mdx file").unwrap();
    let report = run(
        &context(root.path()),
        Request {
            all: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.selected, 1);
    assert_eq!(report.items[0].path, managed.canonicalize().unwrap());
    assert_eq!(report.normalized, 1);
}

#[test]
fn tidy_fails_loud_on_unsupported_bodies_without_writing() {
    let root = tempfile::tempdir().unwrap();
    let path = authored_wrapped(root.path());
    let mut broken = read(&path).replacen("customField", "{/* body comment */}\ncustomField", 1);
    broken = broken.replace("<Paragraph>\n", "<Unknown />\n<Paragraph>\n");
    let path2 = root.path().join("drafts").join("broken.mdx");
    std::fs::write(&path2, broken).unwrap();
    let report = run(
        &context(root.path()),
        Request {
            files: vec![path2.clone()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.failed, 1);
    assert!(report.items[0].error.as_deref().is_some_and(|e| !e.is_empty()));
    assert!(read(&path2).contains("{/* body comment */}"), "failed items keep their bytes");
}

#[test]
fn tidy_requires_operands_or_all() {
    let root = tempfile::tempdir().unwrap();
    assert!(run(&context(root.path()), Request::default()).is_err());
    assert!(
        run(
            &context(root.path()),
            Request {
                all: true,
                files: vec![root.path().join("x.mdx")],
                ..Default::default()
            }
        )
        .is_err()
    );
}

/// A local semantic edit (body differs from stored receipts) still tidies:
/// formatting never touches witnesses, so the stale localHash survives
/// byte-for-byte alongside the normalized body span.
#[test]
fn tidy_preserves_local_semantic_edits() {
    let root = tempfile::tempdir().unwrap();
    let path = authored_wrapped(root.path());
    let edited = read(&path).replace("\"guide\"", "\"my guide\"");
    std::fs::write(&path, edited).unwrap();
    let report = run(
        &context(root.path()),
        Request {
            files: vec![path.clone()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.normalized, 1);
    let after = read(&path);
    assert!(after.contains("[my guide](https://e.org/docs)"));
    assert!(after.contains("sha256:old-local"), "stale receipt untouched");
}

/// The atomic writer refuses to replace a file whose bytes changed after
/// the tidy read — the concurrent edit wins.
#[test]
fn tidy_write_guard_refuses_changed_original() {
    let root = tempfile::tempdir().unwrap();
    let path = authored_wrapped(root.path());
    let stale = b"stale original".to_vec();
    let next = b"replacement".to_vec();
    assert!(crate::files::write(&path, Some(&stale), &next).is_err());
    let current = read(&path);
    assert!(current.contains("<Paragraph>"), "the raced file is intact");
}
