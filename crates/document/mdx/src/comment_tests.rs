use super::*;
use leafbud_document::{BlockKind, Inline};

fn source() -> String {
    "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {id: 00000000-0000-0000-0000-000000000000, state: draft}\n---\n\n{/* leading 😀 */}\n\n<Paragraph>\n\n<Run text={\"First\"} marks={[]} />\n\n</Paragraph>\n\n<Page name='details' kind='linear'>\n\n<Key id='inside' />\n{/* inside\npage */}\n\nInside.\n\n</Page>\n\n<Key id='link' />\n{/* link */}\n<PageLink page='details' />\n\n{/* last */}\n\nLast.\n".into()
}

#[test]
fn anchored_comments_prepare_tidy_rename_and_fingerprints() {
    let source = format!("\u{feff}{}", source().replace('\n', "\r\n"));
    let parsed = parse_with_spans(&source).unwrap();
    assert_eq!(parsed.comments.len(), 4);
    assert_eq!(parsed.comments[0].block, "b-auto-1");
    assert_eq!(parsed.comments[1].page, "details");
    let without = crate::source::splice(
        &source,
        parsed
            .comments
            .iter()
            .map(|c| TidyEdit {
                start: c.span.start,
                end: c.span.end,
                text: String::new(),
            })
            .collect(),
    )
    .unwrap();
    assert_eq!(parse(&without).unwrap().editor_value(), parsed.document.editor_value());
    assert_eq!(parse(&without).unwrap().semantic_value(), parsed.document.semantic_value());
    let mut n = 0;
    let mut block = || {
        n += 1;
        format!("assigned-{n}")
    };
    let mut page = || "main".into();
    let plan = prepare_keys(
        &source,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    let prepared = apply_key_preparation(&source, &plan).unwrap();
    let tidy = plan_markdown_tidy(&prepared).unwrap();
    assert_eq!(tidy.blocks_changed, 1);
    assert!(
        apply_markdown_tidy(&prepared.replace("leading", "changed"), &tidy)
            .unwrap_err()
            .to_string()
            .contains("changed since planning")
    );
    let mut damaged = tidy.clone();
    let comment = &parse_with_spans(&prepared).unwrap().comments[0];
    damaged.edits.push(TidyEdit {
        start: comment.span.start,
        end: comment.span.end,
        text: String::new(),
    });
    assert!(apply_markdown_tidy(&prepared, &damaged).unwrap_err().to_string().contains("source comments"));
    let tidy = apply_markdown_tidy(&prepared, &tidy).unwrap();
    let renamed = rename_key(&tidy, "inside", "appendix-text").unwrap();
    for raw in ["{/* leading 😀 */}", "{/* inside\r\npage */}", "{/* link */}", "{/* last */}"] {
        assert!(renamed.contains(raw));
    }
    assert!(plan_markdown_tidy(&renamed).unwrap().edits.is_empty());
    assert_eq!(parse_with_spans(&renamed).unwrap().comments[1].block, "appendix-text");
}

#[test]
fn comments_follow_same_page_blocks_but_deleted_cross_page_and_orphans_refuse() {
    let source = source().replace("state: draft", "state: draft, mainPage: main");
    let parsed = parse(&source).unwrap();
    let mut doc = parsed.clone();
    // Use a synthetic complete binding, not an actual remote effect.
    doc.state.main_page = Some(doc.pages[0].key.clone());
    doc.state.binding = Some(leafbud_document::Binding {
        base_url: "https://leaflet.pub".into(),
        token: uuid::Uuid::nil(),
        root: "root".into(),
        entity_set: "set".into(),
        entry_fact: "entry".into(),
        publication: None,
    });
    let base = patch_tracking(&source, &doc.state).unwrap();
    assert_eq!(adopt_remote(&base, &doc).unwrap(), base, "equal state/body keeps bytes");
    doc.pages[0].blocks.swap(0, 2);
    doc.pages[1].blocks[0].kind = BlockKind::Paragraph {
        content: vec![Inline::Run {
            text: "Remote inside".into(),
            marks: vec![],
        }],
    };
    let out = adopt_remote(&base, &doc).unwrap();
    assert!(out.contains("{/* inside\npage */}\n\nRemote inside"));
    assert!(out.find("{/* last */}").unwrap() < out.find("{/* leading 😀 */}").unwrap());
    assert_eq!(parse_with_spans(&out).unwrap().comments.len(), 4);
    let mut dropped = doc.clone();
    dropped.pages[0].blocks.remove(0);
    assert!(adopt_remote(&base, &dropped).unwrap_err().to_string().contains("comment anchor"));
    let mut moved = doc;
    let block = moved.pages[1].blocks.remove(0);
    moved.pages[0].blocks.push(block);
    assert!(adopt_remote(&base, &moved).unwrap_err().to_string().contains("comment anchor"));
    for body in [
        "text {/* inline */}",
        "{/* trailing */}",
        "{/* one */ 1 /* two */}\n\ntext",
        "{/* before page */}\n\n<Page name='details' kind='linear'>\n\ntext\n\n</Page>",
        "<Page name='details' kind='linear'>\n\ntext\n\n{/* page tail */}\n\n</Page>",
    ] {
        let head = source.split_once("\n---\n").unwrap().0.to_owned();
        assert!(parse(&format!("{head}\n---\n\n{body}\n")).is_err(), "unsupported {body}");
    }
}
