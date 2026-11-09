use super::*;
use serde_json::json;

fn input(body: &str) -> String {
    format!(
        "---\nformat: leafbud-mdx/2\ntitle: T # title comment\n# separator\ndescription: |-\n  old description\ntags: [a, b] # tag comment\npublishedAt: '2026-11-09T00:00:00Z'\ncustom: {{nested: [yes, 42]}} # unknown\n\"@leafbud\":\n  id: 00000000-0000-0000-0000-000000000000\n  state: draft\n  mainPage: main\n---\n{body}"
    )
}

#[test]
fn each_metadata_field_changes_only_its_header_value() {
    let source = input("\n<Key id='b' />\nbody\n");
    for (field, edit) in [
        (
            "title",
            MetadataEdit {
                title: Some("Changed".into()),
                ..Default::default()
            },
        ),
        (
            "description",
            MetadataEdit {
                description: Some("".into()),
                ..Default::default()
            },
        ),
        (
            "tags",
            MetadataEdit {
                tags: Some(vec![]),
                ..Default::default()
            },
        ),
        (
            "publishedAt",
            MetadataEdit {
                published_at: Some("2026-11-10T00:00:00Z".into()),
                ..Default::default()
            },
        ),
    ] {
        let out = edit_metadata(&source, &edit).unwrap();
        let mut before = header(&source).unwrap();
        let mut after = header(&out).unwrap();
        assert_ne!(before[field], after[field]);
        before.as_object_mut().unwrap().remove(field);
        after.as_object_mut().unwrap().remove(field);
        assert_eq!(before, after);
        assert_eq!(patch::split_managed(&source).unwrap().1, patch::split_managed(&out).unwrap().1);
    }
}

#[test]
fn metadata_selected_values_only_with_comments_body_and_unknowns() {
    let source = input("\n<Key id='a' />\n# Not title\n\nbody\n");
    let edit = MetadataEdit {
        title: Some("New: \"title\"".into()),
        description: Some(" ".into()),
        tags: Some(vec!["z".into(), "a".into(), "z".into()]),
        ..Default::default()
    };
    let out = edit_metadata(&source, &edit).unwrap();
    let before = header(&source).unwrap();
    let after = header(&out).unwrap();
    for key in ["format", "publishedAt", "custom", "@leafbud"] {
        assert_eq!(before[key], after[key]);
    }
    for comment in ["# title comment", "# separator", "# tag comment", "custom: {nested: [yes, 42]} # unknown"] {
        assert!(out.contains(comment));
    }
    assert_eq!(after["tags"], json!(["z", "a", "z"]));
    assert_eq!(parse(&out).unwrap().description.as_deref(), Some(" "));
    assert_eq!(patch::split_managed(&source).unwrap().1, patch::split_managed(&out).unwrap().1);
    assert_eq!(edit_metadata(&out, &edit).unwrap(), out);
}

#[test]
fn metadata_clear_noops_dates_validation_and_bom_crlf() {
    for spelling in ["", "description: null\n", "description: ''\n"] {
        let source = input("\nbody\n").replace("description: |-\n  old description\n", spelling);
        assert_eq!(
            edit_metadata(
                &source,
                &MetadataEdit {
                    description: Some(String::new()),
                    ..Default::default()
                }
            )
            .unwrap(),
            source
        );
    }
    let source = format!("\u{feff}{}", input("\nbody\n").replace('\n', "\r\n"));
    let out = edit_metadata(
        &source,
        &MetadataEdit {
            published_at: Some("2026-11-09T01:00:00+01:00".into()),
            tags: Some(vec![]),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.starts_with("\u{feff}---\r\n"));
    assert_eq!(header(&out).unwrap()["tags"], json!([]));
    assert!(!out.replace("\r\n", "").contains('\n'));
    for date in ["yesterday", "2026-11-09", "2026-11-09T00:00:00"] {
        assert!(
            edit_metadata(
                &source,
                &MetadataEdit {
                    published_at: Some(date.into()),
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
    assert!(
        edit_metadata(
            &source,
            &MetadataEdit {
                title: Some("  ".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
    for source in [
        source.replace("tags: [a, b]", "tags: null"),
        source.replace("state: draft", "state: draft\r\n  future: true"),
        source.replace("publishedAt: '2026-11-09T00:00:00Z'", "publishedAt: 42"),
    ] {
        assert!(edit_metadata(&source, &MetadataEdit::default()).is_err());
    }
}

#[test]
fn metadata_block_tags_comments_and_unsupported_flow_root() {
    let source = input("\nbody\n").replace("tags: [a, b] # tag comment", "tags:\n  - a # item a\n  # between\n  - b");
    let out = edit_metadata(
        &source,
        &MetadataEdit {
            tags: Some(vec![]),
            ..Default::default()
        },
    )
    .unwrap();
    for comment in ["# item a", "  # between"] {
        assert!(out.contains(comment));
    }
    let source = format!("---\n{}\n---\n\nbody\n", serde_json::to_string(&header(&source).unwrap()).unwrap());
    assert!(parse(&source).is_ok());
    assert!(
        edit_metadata(
            &source,
            &MetadataEdit {
                title: Some("New".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn rename_block_page_main_properties_not_prose_or_code() {
    let source = input(
        "\n<Key id = { \"old\" } />\nold and `old`\n\n<Page kind = 'linear' name = {\"other\"}>\n\n<Key id='inner' />\n<PageLink page = { \"main\" } />\n\n</Page>\n\n<Key id='link' />\n<PageLink page = 'other' />\n\n<Key id='link2' />\n<PageLink page=\"other\" />\n\n<Key id='code' />\n```mdx\n<Key id='old' />\n<Page name='other' kind='linear'>\n```\n",
    );
    let renamed = rename_key(&source, "old", "new").unwrap();
    assert!(renamed.contains("<Key id = \"new\" />\nold and `old`"));
    assert!(renamed.contains("```mdx\n<Key id='old' />"));
    assert_eq!(rename_key(&renamed, "new", "new").unwrap(), renamed);
    let renamed = rename_key(&renamed, "other", "renamed-page").unwrap();
    assert!(renamed.contains("<Page kind = 'linear' name = \"renamed-page\">"));
    assert_eq!(renamed.matches("<PageLink page = \"renamed-page\" />").count(), 1);
    assert!(renamed.contains("<PageLink page=\"renamed-page\" />"));
    let renamed = rename_key(&renamed, "main", "start").unwrap();
    assert!(renamed.contains("mainPage: \"start\""));
    assert!(renamed.contains("<PageLink page = \"start\" />"));
    for (old, new) in [("missing", "ok"), ("new", "link"), ("new", "?"), ("b-auto-1", "ok")] {
        assert!(rename_key(&renamed, old, new).is_err());
    }
}

#[test]
fn rename_confirmed_identity_receipts_inactive_text_and_retired_collision() {
    let mut source = input("\n<Key id='old' />\ntext\n");
    source = source.replace("  mainPage: main\n", "  mainPage: main\n  nodes:\n    old: {entity: e-block, kind: block, placement: f-place, inactiveTextFact: f-inactive}\n    retired: {entity: e-old, kind: block, placement: f-old}\n");
    let before = parse(&source).unwrap();
    let renamed = rename_key(&source, "old", "fresh").unwrap();
    let after = parse(&renamed).unwrap();
    assert_eq!(after.state.nodes["fresh"], before.state.nodes["old"]);
    assert_eq!(after.editor_value(), before.editor_value());
    assert_eq!(after.state.sync, before.state.sync);
    assert!(rename_key(&source, "old", "retired").is_err());
    assert!(rename_key(&source, "retired", "unused").is_err());
    let crlf = format!("\u{feff}{}", source.replace('\n', "\r\n"));
    let renamed = rename_key(&crlf, "old", "fresh").unwrap();
    assert_eq!(
        renamed,
        format!("\u{feff}{}", rename_key(&source, "old", "fresh").unwrap().replace('\n', "\r\n"))
    );
}
