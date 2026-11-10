use super::*;
use leafbud_document::{Binding, Block, BlockKind, Document, EDITOR_PROFILE, FORMAT, Inline, Mark, Node, Occurrence, Page, Publication, State, Sync, Uuid};
use std::collections::BTreeMap;

fn document() -> Document {
    let state = State {
        names: None,
        last_resolution: None,
        writer: None,
        pending: None,
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
    std::fs::write(root.path().join("unreadable.mdx"), [0xff, 0xfe]).unwrap();
    let report = run(
        &context(root.path()),
        Request {
            all: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (report.selected, report.unchanged, report.failed),
        (2, 1, 1),
        "invalid UTF-8 is a per-item failure, not a batch-wide discovery error"
    );
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

fn named(root: &Path, file: &str, path: Option<&str>) -> PathBuf {
    let mut doc = document();
    doc.state.id = Uuid::new_v4();
    doc.state.binding = None;
    doc.state.nodes.clear();
    doc.state.sync = None;
    if let Some(path) = path {
        doc.extra.insert("path".into(), serde_json::json!(path));
    }
    let file = root.join(file);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, leafbud_mdx::render(&doc).unwrap()).unwrap();
    file
}

fn naming(context: &config::Context, path: &Path, from: Option<names::Direction>, accept: bool) -> Report {
    run(
        context,
        Request {
            files: vec![path.into()],
            from,
            accept,
            ..Default::default()
        },
    )
    .unwrap()
}

fn disposition(root: &Path, path: &Path) -> &'static str {
    names::inspect(root, path, &leafbud_mdx::parse(&read(path)).unwrap()).disposition
}

#[test]
fn names_pair_classification_direction_acceptance_and_repeat_noop() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path().canonicalize().unwrap();
    let ctx = context(&root);
    let file = named(&root, "draft.mdx", Some("/intent"));
    assert_eq!(disposition(&root, &file), "untracked");
    let before = read(&file);
    let r = naming(&ctx, &file, None, false);
    assert_eq!(r.needs_choice, 1);
    assert_eq!(read(&file), before);
    assert_eq!(naming(&ctx, &file, None, true).normalized, 1);
    assert_eq!(disposition(&root, &file), "accepted-mismatch");
    let bytes = std::fs::read(&file).unwrap();
    let mtime = std::fs::metadata(&file).unwrap().modified().unwrap();
    assert_eq!(naming(&ctx, &file, None, true).unchanged, 1);
    assert_eq!(naming(&ctx, &file, None, false).unchanged, 1);
    assert_eq!(std::fs::read(&file).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&file).unwrap().modified().unwrap(), mtime);
    let moved = root.join("manual.mdx");
    std::fs::rename(&file, &moved).unwrap();
    assert_eq!(disposition(&root, &moved), "file-changed");
    assert_eq!(naming(&ctx, &moved, None, false).normalized, 1);
    let doc = leafbud_mdx::parse(&read(&moved)).unwrap();
    assert_eq!(doc.extra["path"], "/manual");
    assert_eq!(disposition(&root, &moved), "equal");
    let witness = doc.state.names.unwrap();
    // Current author entry only; historical witness has a tagged value.
    let authored = read(&moved).replacen("\"path\": \"/manual\"", "\"path\": \"/later\"", 1);
    std::fs::write(&moved, authored).unwrap();
    assert_eq!(leafbud_mdx::parse(&read(&moved)).unwrap().state.names, Some(witness));
    assert_eq!(disposition(&root, &moved), "path-changed");
    assert_eq!(naming(&ctx, &moved, None, false).normalized, 1);
    let later = root.join("later.mdx");
    assert!(!moved.exists());
    assert_eq!(disposition(&root, &later), "equal");
    // Both sides changed differently: no default choice.
    let both = root.join("both.mdx");
    std::fs::rename(&later, &both).unwrap();
    std::fs::write(&both, read(&both).replacen("\"path\": \"/later\"", "\"path\": \"/another\"", 1)).unwrap();
    assert_eq!(disposition(&root, &both), "both-changed");
    assert_eq!(naming(&ctx, &both, None, false).needs_choice, 1);
    // Convergence is observed read-only, then deliberately settled.
    std::fs::write(&both, read(&both).replacen("\"path\": \"/another\"", "\"path\": \"/both\"", 1)).unwrap();
    assert_eq!(disposition(&root, &both), "converged");
    assert_eq!(naming(&ctx, &both, None, false).normalized, 1);
}

#[test]
fn names_missing_null_equal_external_and_selectors_are_independent() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let ctx = context(root.path());
    let file = named(root.path(), "new.mdx", None);
    let before = read(&file);
    assert_eq!(naming(&ctx, &file, None, false).unchanged, 1);
    assert_eq!(read(&file), before);
    assert_eq!(naming(&ctx, &file, Some(names::Direction::Path), false).needs_choice, 1);
    assert_eq!(naming(&ctx, &file, None, true).normalized, 1);
    assert_eq!(
        leafbud_mdx::parse(&read(&file)).unwrap().state.names.unwrap().path,
        leafbud_document::NamePath::Missing
    );
    std::fs::write(&file, read(&file).replacen("\"@leafbud\":", "path: null\n\"@leafbud\":", 1)).unwrap();
    assert_eq!(disposition(root.path(), &file), "unassigned");
    assert_eq!(naming(&ctx, &file, None, true).normalized, 1);
    assert_eq!(
        leafbud_mdx::parse(&read(&file)).unwrap().state.names.unwrap().path,
        leafbud_document::NamePath::Null
    );
    assert_eq!(naming(&ctx, &file, Some(names::Direction::Filename), false).normalized, 1);
    assert_eq!(leafbud_mdx::header(&read(&file)).unwrap()["path"], "/new");
    let ext = named(external.path(), "outside.mdx", Some("/elsewhere"));
    assert_eq!(naming(&ctx, &ext, Some(names::Direction::Filename), false).needs_choice, 1);
    for (keys, markdown) in [(true, false), (false, true)] {
        assert_eq!(
            run(
                &ctx,
                Request {
                    files: vec![ext.clone()],
                    keys,
                    markdown,
                    ..Default::default()
                }
            )
            .unwrap()
            .unchanged,
            1
        );
    }
    let equal = named(root.path(), "equal.mdx", Some("/equal"));
    let before = read(&equal);
    let report = run(
        &ctx,
        Request {
            all: true,
            markdown: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.failed, 0);
    assert_eq!(read(&equal), before, "--all selects files, --markdown does not settle names");
    assert_eq!(naming(&ctx, &equal, None, false).normalized, 1);
    assert_eq!(disposition(root.path(), &equal), "equal");
    let invalid = named(root.path(), "invalid-type.mdx", None);
    let source = read(&invalid).replacen("\"@leafbud\":", "path: 42\n\"@leafbud\":", 1);
    std::fs::write(&invalid, &source).unwrap();
    for accept in [false, true] {
        let report = naming(&ctx, &invalid, None, accept);
        assert_eq!(report.needs_choice, 1);
        assert_eq!(report.items[0].names.as_ref().unwrap().disposition, "unsupported");
        assert!(
            report.items[0].names.as_ref().unwrap().path.is_none(),
            "an invalid type must not masquerade as missing path"
        );
        assert_eq!(read(&invalid), source);
    }
}

#[test]
fn names_mapping_nested_unicode_extension_and_unsafe_intent() {
    let root = tempfile::tempdir().unwrap();
    let ctx = context(root.path());
    std::fs::create_dir(root.path().join("notes")).unwrap();
    let file = named(root.path(), "start.mdx", Some("/notes/Éssay.mdx"));
    let before = leafbud_mdx::parse(&read(&file)).unwrap();
    let r = naming(&ctx, &file, Some(names::Direction::Path), false);
    assert_eq!(r.normalized, 1, "{:?}", r.items[0].error);
    let target = root.path().join("notes/Éssay.mdx.mdx");
    assert_eq!(r.items[0].destination, Some(target.clone()));
    assert!(!file.exists());
    let after = leafbud_mdx::parse(&read(&target)).unwrap();
    assert_eq!(before.state.id, after.state.id);
    assert_eq!(before.pages, after.pages);
    assert_eq!(after.state.names.unwrap().file, "notes/Éssay.mdx.mdx");
    for value in [
        "/../escape",
        "/a/../b",
        "/a//b",
        "//absolute",
        "/a/",
        "relative",
        "C:\\drive",
        "/%2e%2e/x",
        "/x?y",
        "/x#y",
        "/x\ny",
    ] {
        let file = named(root.path(), "unsafe.mdx", Some(value));
        let before = read(&file);
        let r = naming(&ctx, &file, Some(names::Direction::Path), false);
        assert_eq!(r.needs_choice, 1, "{value}: {:?}", r.items[0].error);
        assert_eq!(read(&file), before);
        assert_eq!(
            naming(&ctx, &file, None, true).normalized,
            1,
            "explicit mismatch acceptance does not enact unsafe intent"
        );
    }
    let file = named(root.path(), "missing-parent.mdx", Some("/not-created/file"));
    let before = read(&file);
    let r = run(
        &ctx,
        Request {
            files: vec![file.clone()],
            from: Some(names::Direction::Path),
            dry_run: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r.needs_choice, 1);
    assert_eq!(read(&file), before);
    assert!(!root.path().join("not-created").exists());
}

#[test]
fn names_dry_run_composes_keys_reports_path_and_preserves_body_state() {
    let root = tempfile::tempdir().unwrap();
    let ctx = context(root.path());
    let file = authored_wrapped(root.path());
    let source = format!(
        "\u{feff}{}",
        read(&file)
            .replace("sha256:old-local", &crate::import::hash(&document().editor_value()))
            .replacen("title: T", "path: /destination\ntitle: T", 1)
            .replace("<Key id=\"b-1\" />", "{/* note 😀 */}\n<Key id=\"b-1\" />")
            .replace('\n', "\r\n")
    );
    std::fs::write(&file, &source).unwrap();
    let before = leafbud_mdx::parse(&source).unwrap();
    let mtime = std::fs::metadata(&file).unwrap().modified().unwrap();
    let report = run(
        &ctx,
        Request {
            files: vec![file.clone()],
            from: Some(names::Direction::Path),
            keys: true,
            dry_run: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.normalized, 1, "{:?}", report.items[0].error);
    let target = root.path().join("destination.mdx");
    assert_eq!(report.items[0].destination, Some(target.clone()));
    assert_eq!(read(&file), source);
    assert_eq!(std::fs::metadata(&file).unwrap().modified().unwrap(), mtime);
    assert!(!target.exists());
    assert_eq!(
        run(
            &ctx,
            Request {
                files: vec![file.clone()],
                from: Some(names::Direction::Path),
                keys: true,
                ..Default::default()
            }
        )
        .unwrap()
        .normalized,
        1
    );
    let after_source = read(&target);
    assert!(after_source.starts_with("\u{feff}---\r\n"));
    assert_eq!(source.split("---\r\n").nth(2), after_source.split("---\r\n").nth(2));
    assert!(after_source.contains("customField: {\"meaning\": \"retained\"}"));
    let mut after = leafbud_mdx::parse(&after_source).unwrap();
    after.state.names = None;
    assert_eq!(after, before, "native identity, stale receipts and dirty editor input preserved");
    assert_eq!(crate::editor_state::local_change(&after), crate::editor_state::Change::Changed);
}

#[test]
fn names_no_clobber_symlinks_aliases_and_batch_destination_conflicts() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let ctx = context(root.path());
    let a = named(root.path(), "a.mdx", Some("/target"));
    let b = named(root.path(), "b.mdx", Some("/target"));
    let preview = run(
        &ctx,
        Request {
            files: vec![a.clone(), b.clone()],
            from: Some(names::Direction::Path),
            dry_run: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (preview.normalized, preview.needs_choice),
        (1, 1),
        "preview accounts for common absent destinations too"
    );
    assert!(a.exists() && b.exists() && !root.path().join("target.mdx").exists());
    assert_eq!(preview.items[0].destination, preview.items[1].destination);
    let report = run(
        &ctx,
        Request {
            files: vec![a.clone(), b.clone()],
            from: Some(names::Direction::Path),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((report.normalized, report.needs_choice), (1, 1));
    assert!(!a.exists());
    let target = root.path().join("target.mdx");
    let bytes = std::fs::read(&target).unwrap();
    assert!(b.exists());
    assert_eq!(naming(&ctx, &b, Some(names::Direction::Path), false).needs_choice, 1);
    assert_eq!(std::fs::read(&target).unwrap(), bytes);
    std::fs::remove_file(&target).unwrap();
    symlink(root.path().join("absent"), &target).unwrap();
    assert_eq!(naming(&ctx, &b, Some(names::Direction::Path), false).needs_choice, 1);
    assert!(std::fs::symlink_metadata(&target).unwrap().file_type().is_symlink());
    std::fs::remove_file(&target).unwrap();
    let alias = root.path().join("alias.mdx");
    symlink(&b, &alias).unwrap();
    let report = run(
        &ctx,
        Request {
            files: vec![b.clone(), alias],
            from: Some(names::Direction::Path),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((report.selected, report.normalized), (1, 1));
    std::fs::create_dir(root.path().join("real")).unwrap();
    symlink(root.path().join("real"), root.path().join("dir-alias")).unwrap();
    let file = named(root.path(), "link-parent.mdx", Some("/dir-alias/file"));
    assert_eq!(naming(&ctx, &file, Some(names::Direction::Path), false).needs_choice, 1);
    assert!(file.exists());
}

#[test]
fn names_late_source_identity_route_target_and_duplicate_guards() {
    use std::os::unix::fs::symlink;
    for kind in ["bytes", "inode", "route", "target", "duplicate"] {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let ctx = context(root.path());
        let a = named(root.path(), "a.mdx", Some("/target"));
        let b = named(external.path(), "external.mdx", None);
        let alias = root.path().join("alias.mdx");
        symlink(&a, &alias).unwrap();
        let before = read(&a);
        let report = run_with(
            &ctx,
            Request {
                files: vec![alias.clone(), b.clone()],
                from: Some(names::Direction::Path),
                ..Default::default()
            },
            &mut |stage, path| {
                if stage == Stage::Prepared && path == a {
                    match kind {
                        "bytes" => std::fs::write(&a, before.replace("plain", "Author save"))?,
                        "inode" => {
                            let replacement = root.path().join("replacement");
                            std::fs::write(&replacement, &before)?;
                            std::fs::rename(replacement, &a)?;
                        }
                        "route" => {
                            std::fs::remove_file(&alias)?;
                            symlink(&b, &alias)?;
                        }
                        "target" => std::fs::write(root.path().join("target.mdx"), "other writer")?,
                        "duplicate" => std::fs::write(&b, &before)?,
                        _ => unreachable!(),
                    }
                }
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(report.normalized, 0, "{kind}");
        assert!(report.items[0].error.is_some(), "{kind}");
        assert!(a.exists());
        if kind != "bytes" {
            assert_eq!(read(&a), before);
        }
        if kind == "target" {
            assert_eq!(read(&root.path().join("target.mdx")), "other writer");
        }
    }
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let a = named(external.path(), "a.mdx", None);
    let b = external.path().join("b.mdx");
    std::fs::copy(&a, &b).unwrap();
    let report = run(
        &context(root.path()),
        Request {
            files: vec![a, b],
            keys: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.failed, 2, "all explicit external copies participate even without naming");
}

#[test]
fn names_interruption_after_move_retains_good_source_and_retry_settles_truth() {
    let root = tempfile::tempdir().unwrap();
    let ctx = context(root.path());
    let file = named(root.path(), "a.mdx", Some("/target"));
    naming(&ctx, &file, None, true);
    let before = read(&file);
    let target = root.path().join("target.mdx");
    let report = run_with(
        &ctx,
        Request {
            files: vec![file.clone()],
            from: Some(names::Direction::Path),
            ..Default::default()
        },
        &mut |stage, _| {
            ensure!(stage != Stage::Renamed, "injected interruption before witness");
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(report.items[0].outcome, "partial");
    assert_eq!(report.items[0].destination, Some(target.clone()));
    assert!(!file.exists());
    assert_eq!(read(&target), before);
    assert_eq!(disposition(root.path(), &target), "converged");
    assert_eq!(naming(&ctx, &target, None, false).normalized, 1);
    assert_eq!(disposition(root.path(), &target), "equal");
}

#[tokio::test]
async fn names_readonly_status_and_list_do_not_accept_or_clear_dirty_editor() {
    let root = tempfile::tempdir().unwrap();
    let ctx = context(root.path());
    let file = authored_wrapped(root.path());
    let source = read(&file)
        .replace("sha256:old-local", &crate::import::hash(&document().editor_value()))
        .replacen("title: T", "path: /other\ntitle: T", 1);
    std::fs::write(&file, &source).unwrap();
    let before = leafbud_mdx::parse(&source).unwrap();
    let report = crate::status::inspect(&ctx, crate::status::Request { files: vec![file.clone()] }, None)
        .await
        .unwrap();
    assert_eq!(report.version, 2);
    assert_eq!(report.items[0].names.as_ref().unwrap().disposition, "untracked");
    assert_eq!(report.items[0].local, crate::editor_state::Change::Changed);
    assert_eq!(read(&file), source);
    naming(&ctx, &file, None, true);
    let after = leafbud_mdx::parse(&read(&file)).unwrap();
    assert_eq!(before.state.sync, after.state.sync);
    assert_eq!(before.state.binding, after.state.binding);
    let listed = crate::list::inspect(&ctx, crate::list::Request::default()).unwrap();
    assert_eq!(listed.version, 2);
    assert_eq!(listed.items[0].names.as_ref().unwrap().disposition, "accepted-mismatch");
    let report = crate::status::inspect(&ctx, crate::status::Request::default(), None).await.unwrap();
    assert_eq!(report.items[0].local, crate::editor_state::Change::Changed);
}

#[test]
fn names_after_move_external_source_save_wins_over_witness_patch() {
    let root = tempfile::tempdir().unwrap();
    let ctx = context(root.path());
    let file = named(root.path(), "a.mdx", Some("/target"));
    let target = root.path().join("target.mdx");
    let before = read(&file);
    let edited = before.replace("plain", "New author save after rename");
    let report = run_with(
        &ctx,
        Request {
            files: vec![file.clone()],
            from: Some(names::Direction::Path),
            ..Default::default()
        },
        &mut |stage, path| {
            if stage == Stage::Renamed {
                std::fs::write(path, &edited)?;
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(report.items[0].outcome, "partial");
    assert_eq!(report.items[0].naming_action, "renamed-settlement-unconfirmed");
    assert_eq!(report.items[0].destination, Some(target.clone()));
    assert_eq!(read(&target), edited);
    assert!(!file.exists());
    assert!(leafbud_mdx::parse(&edited).unwrap().state.names.is_none());
}
