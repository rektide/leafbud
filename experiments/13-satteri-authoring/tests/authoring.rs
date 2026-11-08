use satteri_authoring_experiment::{author, model::*};

pub const GARDEN: &str = include_str!("../fixtures/garden.mdx");
fn source(body: &str) -> String {
    format!("---\nformat: leafspring-mdx/1\ntitle: Test\n---\n\n{body}")
}
fn good(body: &str) -> Document<'_> {
    author::compile(body).unwrap_or_else(|d| panic!("{d:#?}"))
}
fn bad(body: &str) -> Vec<Diagnostic> {
    author::compile(body).unwrap_err()
}

#[test]
fn representative_document_is_typed_without_prose_wrappers() {
    let doc = good(GARDEN);
    assert_eq!(doc.metadata.title, "A garden notebook");
    assert_eq!(doc.metadata.tags, ["gardens", "maps"]);
    assert_eq!(doc.metadata.published_at.as_deref(), Some("2026-11-08T12:00:00Z"));
    assert_eq!(doc.pages.len(), 1);
    assert_eq!(doc.pages[0].name, "details");
    assert_eq!(doc.consumed_keys, 6);
    assert!(matches!(doc.main[0].kind, BlockKind::Heading { level: 2, .. }));
    assert_eq!(doc.main[0].key.as_ref().unwrap().name, "introduction");
    assert!(matches!(doc.main[1].kind, BlockKind::Paragraph { .. }));
    assert_eq!(doc.main[1].key.as_ref().unwrap().name, "argument");
    assert!(matches!(doc.main[2].kind, BlockKind::List { ordered: false, .. }));
    assert!(matches!(doc.main[3].kind, BlockKind::List { ordered: true, start: 3, .. }));
    assert!(matches!(doc.main[4].kind, BlockKind::Quote { .. }));
    assert!(matches!(doc.main[5].kind, BlockKind::Code { .. }));
    assert!(matches!(doc.main[6].kind, BlockKind::Poll { .. }));
    assert!(matches!(doc.main[7].kind, BlockKind::PageLink { .. }));
    assert_eq!(doc.main.len(), 8, "Page definition and Key markers are not body blocks");
}

#[test]
fn prose_decodes_entities_escapes_unicode_marks_links_and_explicit_break() {
    let doc = good(GARDEN);
    let BlockKind::Paragraph { content } = &doc.main[1].kind else { panic!() };
    assert_eq!(
        plaintext(content),
        "The beds are ready, and Café maps help us plan. An escaped *star* & Unicode é😀 stay text.\nBring seeds, not a runtime."
    );
    assert!(content.contains(&Inline::Run {
        text: "beds".into(),
        marks: vec![Mark::Strong]
    }));
    assert!(content.contains(&Inline::Run {
        text: "ready".into(),
        marks: vec![Mark::Em]
    }));
    assert!(content.contains(&Inline::Run {
        text: "Café maps".into(),
        marks: vec![Mark::Link {
            href: "https://example.test/maps?q=a&b=2".into()
        }]
    }));
    assert!(content.contains(&Inline::Run {
        text: "seeds".into(),
        marks: vec![Mark::Code]
    }));
    assert_eq!(content.iter().filter(|n| **n == Inline::HardBreak).count(), 1);
}

#[test]
fn nested_native_components_preserve_exact_literal_text_and_break_topology() {
    let doc = good(GARDEN);
    let BlockKind::Paragraph { content } = &doc.pages[0].blocks[1].kind else {
        panic!()
    };
    assert_eq!(plaintext(content), "Exact data:   raw\ntext <>{} & 🦀\tend  then\na new line.");
    assert_eq!(
        content.iter().filter(|n| **n == Inline::HardBreak).count(),
        1,
        "Run's literal newline is not a hard_break"
    );
    let BlockKind::Poll { options } = &doc.main[6].kind else { panic!() };
    assert_eq!(options, &["Gardens & maps", "Café 🦀", "  exact spacing  "]);
}

#[test]
fn native_key_and_following_markdown_are_distinct_siblings_before_binding() {
    use satteri_ast::mdast::MdastNodeType as M;
    for separator in ["\n", "\n\n", "\n\n\n"] {
        let body = format!("<Key id=\"intro\" />{separator}## Introduction\n");
        let (arena, errors) = satteri_pulldown_cmark::parse(&body, satteri_pulldown_cmark::MDX_OPTIONS);
        assert!(errors.is_empty(), "{errors:?}");
        let children = arena.get_children(0);
        assert_eq!(children.len(), 2);
        assert_eq!(arena.get_node(children[0]).node_type, M::MdxJsxFlowElement as u8);
        assert_eq!(arena.get_node(children[1]).node_type, M::Heading as u8);
        let input = source(&body);
        let doc = good(&input);
        assert_eq!(doc.main.len(), 1);
        assert_eq!(doc.consumed_keys, 1);
        assert_eq!(doc.main[0].key.as_ref().unwrap().name, "intro");
    }
}

#[test]
fn key_binding_stays_in_root_quote_item_and_page_scopes() {
    let doc = good(GARDEN);
    let BlockKind::List { items, .. } = &doc.main[2].kind else { panic!() };
    assert_eq!(items[0][0].key.as_ref().unwrap().name, "first-item");
    assert!(items[1][0].key.is_none());
    let BlockKind::Quote { paragraph } = &doc.main[4].kind else { panic!() };
    assert_eq!(paragraph.key.as_ref().unwrap().name, "reminder");
    assert_eq!(doc.pages[0].blocks[0].key.as_ref().unwrap().name, "details-heading");
}

#[test]
fn orphan_keys_do_not_escape_closed_containers_or_skip_siblings() {
    for body in [
        "<Key id=\"lost\" />\n",
        "> <Key id=\"lost\" />\n\nOutside.\n",
        "- <Key id=\"lost\" />\n- Other.\n",
        "<Page name=\"p\">\n<Key id=\"lost\" />\n</Page>\n\nOutside.\n",
        "<Key id=\"a\" />\n\n<Key id=\"b\" />\nText.\n",
        "<Key id=\"a\" />\n{/* ordinary comment */}\nText.\n",
        "<Key id=\"a\" />\n[x]: https://example.test\n\nText.\n",
    ] {
        assert_eq!(bad(&source(body))[0].code, "orphan-key", "{body}");
    }
}

#[test]
fn malformed_inline_paired_and_duplicate_keys_are_diagnostics_not_partial_documents() {
    for marker in [
        "<Key />",
        "<Key id />",
        "<Key name=\"a\" />",
        "<Key id={\"a\"} />",
        "<Key id=\"a\" extra />",
        "<Key id=\"a\" id=\"b\" />",
        "<Key {...props} />",
    ] {
        assert_eq!(bad(&source(&format!("{marker}\nText.\n")))[0].code, "malformed-key", "{marker}");
    }
    assert_eq!(bad(&source("<Key id=\"two words\" />\nText.\n"))[0].code, "invalid-key");
    assert_eq!(bad(&source("Text <Key id=\"inline\" /> more.\n"))[0].code, "misplaced-key");
    assert_eq!(bad(&source("<Key id=\"paired\"></Key>\n"))[0].code, "paired-key-unsupported");
    assert_eq!(bad(&source("<Key id=\"a\" />\nOne.\n\n<Key id=\"a\" />\nTwo.\n"))[0].code, "duplicate-key");
    assert_eq!(
        bad(&source("<Key id=\"a\" />\nOne.\n\n<Poll key=\"a\" options={[\"X\",\"Y\"]} />\n"))[0].code,
        "duplicate-key"
    );
    assert_eq!(
        bad(&source("<Key id=\"a\" />\n<Poll key=\"b\" options={[\"X\",\"Y\"]} />\n"))[0].code,
        "key-collision"
    );
}

#[test]
fn key_can_name_a_component_without_a_runtime_key_prop() {
    let input = source("<Key id=\"poll\" />\n<Poll options={[\"X\",\"Y\"]} />\n");
    assert_eq!(good(&input).main[0].key.as_ref().unwrap().name, "poll");
}

#[test]
fn strict_component_schemas_reject_executable_nonliteral_unknown_and_bad_types() {
    for (body, code) in [
        ("<Poll key=\"p\" options={load()} />", "nonliteral-prop"),
        ("<Poll key=\"p\" options={[1,2]} />", "component-schema"),
        ("<Poll key=\"p\" options={[\"a\",\"b\"]} extra={true} />", "component-schema"),
        ("<Poll key=\"p\" options={[\"a\",\"b\"]} {...props} />", "spread-prop"),
        ("<Poll key=\"p\" options={[]} options={[]} />", "duplicate-prop"),
        ("<Poll options={[\"a\",\"b\"]} />", "poll-key-required"),
        ("<Poll key=\"p\" options={[\"a\",\"a\"]} />", "poll-options"),
        ("<Poll key=\"p\" options={[\"a\"]} />", "poll-options"),
        ("<Run text={{\"a\":{\"duplicate\":1,\"duplicate\":2}}} />", "nonliteral-prop"),
        ("<Canvas name=\"c\" />", "unimplemented-component"),
        ("<Mystery />", "unknown-component"),
        ("{danger()}", "unsupported-block"),
        ("import value from 'missing';\n", "unsupported-block"),
    ] {
        assert_eq!(bad(&source(body))[0].code, code, "{body}");
    }
}

#[test]
fn children_and_unimplemented_markdown_fail_instead_of_disappearing() {
    for (body, code) in [
        ("<Poll key=\"p\" options={[\"a\",\"b\"]}>\n\nText\n\n</Poll>", "component-children"),
        ("- first\n\n  second paragraph\n", "list-shape-unsupported"),
        ("- [x] task\n", "task-list-unsupported"),
        ("| a | b |\n| - | - |\n| c | d |\n", "unsupported-block"),
        ("```rust title=example\nlet x = 1;\n```\n", "code-meta-unsupported"),
        ("[link](javascript:alert)", "link-scheme"),
        ("[link](https://example.test \"a title\")", "link-title-unsupported"),
    ] {
        assert_eq!(bad(&source(body))[0].code, code, "{body}");
    }
}

#[test]
fn named_pages_allow_forward_references_and_reject_missing_duplicate_cycles() {
    let doc = good(GARDEN);
    assert_eq!(doc.pages[0].name, "details");
    for (body, code) in [
        ("<PageLink page=\"missing\" />", "missing-page"),
        ("<Page name=\"p\" />\n\n<Page name=\"p\" />", "page-name"),
        ("<Page name=\"main\" />", "page-name"),
        ("<Page name=\"p\" kind=\"canvas\" />", "page-kind"),
        ("<Page name=\"p\">\n<PageLink page=\"p\" />\n</Page>", "page-cycle"),
        (
            "<Page name=\"a\">\n<PageLink page=\"b\" />\n</Page>\n\n<Page name=\"b\">\n<PageLink page=\"a\" />\n</Page>",
            "page-cycle",
        ),
    ] {
        assert_eq!(bad(&source(body))[0].code, code, "{body}");
    }
}

#[test]
fn metadata_is_author_owned_heading_is_not_title_and_bad_metadata_fails() {
    assert_eq!(good(GARDEN).metadata.title, "A garden notebook");
    assert_eq!(bad("## No frontmatter\n")[0].code, "frontmatter-required");
    assert_eq!(bad("---\nformat: wrong\ntitle: T\n---\n\nBody.\n")[0].code, "frontmatter");
    assert_eq!(
        bad("---\nformat: leafspring-mdx/1\ntitle: T\nunknown: true\n---\n\nBody.\n")[0].code,
        "frontmatter"
    );
}

#[test]
fn exact_original_bytes_and_marker_ranges_include_bom_unicode_crlf() {
    let input = format!(
        "\u{feff}{}",
        source("Café 🦀.\n\n> <Key id=\"quoted\" />\n> ## Not supported quote shape\n").replace('\n', "\r\n")
    );
    // The marker's actual native span remains inspectable even when the semantic
    // quote shape is outside the compiler subset.
    assert_eq!(bad(&input)[0].code, "quote-shape-unsupported");
    let input = format!("\u{feff}{}", source("Café 🦀.\n\n<Key id=\"body\" />\nOrdinary prose.\n").replace('\n', "\r\n"));
    let doc = good(&input);
    assert_eq!(doc.original_source.as_bytes(), input.as_bytes());
    let key = doc.main[1].key.as_ref().unwrap();
    assert_eq!(&input[key.span.start..key.span.end], "<Key id=\"body\" />");
    let span = doc.main[1].span;
    assert!(input[span.start..span.end].starts_with("Ordinary prose."));
}

#[test]
fn ordinary_inline_fenced_code_and_escaped_component_text_are_literal() {
    let input = source("Inline `<Key id=\"inline\" />`.\n\n```mdx\n\t<Key id=\"fenced\" />\n```\n\n\\<Key id=\"escaped\" />\n");
    let doc = good(&input);
    assert_eq!(doc.consumed_keys, 0);
    let BlockKind::Code { text, .. } = &doc.main[1].kind else { panic!() };
    assert_eq!(text, "\t<Key id=\"fenced\" />", "direct parser keeps fenced tabs; no compiler tab expansion");
}

#[test]
fn known_mdx_multiline_single_backtick_control_remains_a_dialect_limit() {
    let input = source("Code `first\n<Key id=\"accidental\" />\nlast`.\n");
    assert_eq!(bad(&input)[0].code, "key-boundary");
    let input = source("Code `first\n<Poll key=\"accidental\" options={[\"a\",\"b\"]} />\nlast`.\n");
    let doc = good(&input);
    assert!(
        doc.main.iter().any(|b| matches!(b.kind, BlockKind::Poll { .. })),
        "negative observation: native MDX activates flow JSX here; use fences"
    );
}
