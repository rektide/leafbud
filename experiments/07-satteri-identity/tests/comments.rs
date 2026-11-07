use satteri_identity_experiment::{Kind, Mode, Node, Parsed, parse};

fn nodes(node: &Node) -> Vec<&Node> {
    let mut result = vec![node];
    for child in &node.children {
        result.extend(nodes(child));
    }
    result
}

fn keyed(parsed: &Parsed) -> Vec<&Node> {
    nodes(&parsed.tree).into_iter().filter(|n| n.key.is_some()).collect()
}

fn good(source: &str) -> Parsed {
    let parsed = parse(source, Mode::Author);
    assert!(parsed.diagnostics.is_empty(), "{source:?}: {:#?}", parsed.diagnostics);
    parsed
}

fn text(node: &Node) -> String {
    node.value.clone().unwrap_or_default() + &node.children.iter().map(text).collect::<String>()
}

#[test]
fn requested_comment_examples_attach_to_real_heading_and_paragraph() {
    let p = good("{/* springleaf:key intro */}\n## Introduction\n\n{/* springleaf:key body */}\nOrdinary **bold** prose.\n");
    let keys = keyed(&p);
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].kind, Kind::Heading);
    assert_eq!(keys[0].key.as_deref(), Some("intro"));
    assert_eq!(keys[0].details["depth"], 2);
    assert_eq!(keys[1].kind, Kind::Paragraph);
    assert_eq!(keys[1].key.as_deref(), Some("body"));
    assert_eq!(keys[1].data["springleafKey"], "body");
    assert!(nodes(keys[1]).iter().any(|n| n.kind == Kind::Strong));
    assert!(!nodes(&p.tree).iter().any(|n| n.kind == Kind::Expression));
}

#[test]
fn multiline_paragraph_keeps_links_marks_and_hard_break() {
    let p = good("{/* springleaf:key body */}\nFirst **bold** [link](https://example.com)\nsecond  \nthird.\n");
    let body = keyed(&p)[0];
    assert_eq!(body.kind, Kind::Paragraph);
    let descendants = nodes(body);
    assert!(descendants.iter().any(|n| n.kind == Kind::Strong));
    assert!(descendants.iter().any(|n| n.kind == Kind::Link && n.details["url"] == "https://example.com"));
    assert!(descendants.iter().any(|n| n.kind == Kind::Break));
    assert!(text(body).contains("second"));
}

#[test]
fn setext_and_closed_atx_do_not_need_special_identity_hooks() {
    let p = good("{/* springleaf:key setext */}\nA *heading*\n===\n\n{/* springleaf:key atx */}\n## Closed ##\n");
    let keys = keyed(&p);
    assert_eq!(keys.iter().map(|n| n.details["depth"].as_u64().unwrap()).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(text(keys[0]), "A heading");
    assert_eq!(text(keys[1]), "Closed");
}

#[test]
fn blank_lines_between_marker_and_target_are_allowed() {
    let p = good("{/* springleaf:key body */}\n\n\nParagraph.\n");
    assert_eq!(keyed(&p)[0].key.as_deref(), Some("body"));
}

#[test]
fn native_flow_comment_can_interrupt_preceding_prose_without_blank_line() {
    let p = good("First paragraph.\n{/* springleaf:key second */}\nSecond paragraph.\n");
    assert_eq!(p.tree.children.len(), 2);
    assert_eq!(p.tree.children[0].key, None);
    assert_eq!(keyed(&p)[0].key.as_deref(), Some("second"));
    assert_eq!(text(keyed(&p)[0]), "Second paragraph.");
}

#[test]
fn list_quote_and_inner_paragraph_have_distinct_keys() {
    let p = good(
        "{/* springleaf:key list */}\n- {/* springleaf:key first */}\n  First item.\n- Second item.\n\n{/* springleaf:key quote */}\n> {/* springleaf:key inside */}\n> Quoted prose.\n",
    );
    let keys = keyed(&p);
    assert_eq!(
        keys.iter().map(|n| (n.key.as_deref().unwrap(), &n.kind)).collect::<Vec<_>>(),
        [
            ("list", &Kind::List),
            ("first", &Kind::Paragraph),
            ("quote", &Kind::Blockquote),
            ("inside", &Kind::Paragraph)
        ]
    );
    assert_eq!(p.tree.children[0].children[0].kind, Kind::ListItem);
    assert_eq!(p.tree.children[0].children[0].key, None);
}

#[test]
fn nested_scope_stays_with_native_siblings() {
    let p = good("> > {/* springleaf:key nested */}\n> > Deep prose.\n\n- Parent.\n  - {/* springleaf:key child */}\n    Child prose.\n");
    let keys = keyed(&p);
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].kind, Kind::Paragraph);
    assert_eq!(keys[1].kind, Kind::Paragraph);
    assert_eq!(text(keys[0]), "Deep prose.");
    assert_eq!(text(keys[1]), "Child prose.");
}

#[test]
fn unbound_markers_cannot_escape_quote_or_item() {
    for source in [
        "{/* springleaf:key orphan */}\n",
        "> {/* springleaf:key orphan */}\n\nRoot prose.\n",
        "- {/* springleaf:key orphan */}\n- Other item.\n",
    ] {
        let p = parse(source, Mode::Author);
        assert!(p.diagnostics.iter().any(|d| d.code == "unbound-key"), "{source:?}: {:?}", p.diagnostics);
        assert!(keyed(&p).is_empty());
    }
}

#[test]
fn consecutive_annotations_and_definition_siblings_are_not_skipped() {
    for source in [
        "{/* springleaf:key first */}\n{/* springleaf:key second */}\nParagraph.\n",
        "{/* springleaf:key first */}\n[ref]: https://example.com\n\nParagraph.\n",
    ] {
        let p = parse(source, Mode::Author);
        assert!(p.diagnostics.iter().any(|d| d.code == "unsupported-target"), "{source:?}: {:?}", p.diagnostics);
    }
}

#[test]
fn malformed_reserved_namespace_and_key_grammar_are_rejected() {
    for annotation in [
        "{/* springleaf:key */}",
        "{/* springleaf:key two words */}",
        "{/* springleaf:key 123 */}",
        "{/* springleaf:key é */}",
        "{/* springleaf:unknown body */}",
        "{/* springleaf:key body\n */}",
    ] {
        let p = parse(&format!("{annotation}\nParagraph.\n"), Mode::Author);
        assert!(p.diagnostics.iter().any(|d| d.code == "malformed-key"), "{annotation:?}: {:?}", p.diagnostics);
        assert!(keyed(&p).is_empty());
    }
}

#[test]
fn unterminated_comment_is_an_upstream_parse_error() {
    let p = parse("{/* springleaf:key unfinished\nParagraph.\n", Mode::Author);
    assert!(p.diagnostics.iter().any(|d| d.code == "mdx-parse"));
}

#[test]
fn inline_annotation_is_not_a_preceding_flow_directive() {
    for source in [
        "Text {/* springleaf:key body */} more.\n",
        "{/* springleaf:key body */} Text.\n",
        "## Heading {/* springleaf:key body */}\n",
    ] {
        let p = parse(source, Mode::Author);
        assert!(p.diagnostics.iter().any(|d| d.code == "misplaced-key"), "{source:?}: {:?}", p.diagnostics);
        assert!(keyed(&p).is_empty());
    }
}

#[test]
fn same_line_inline_and_fenced_code_keep_key_shaped_text() {
    let source = "Inline `{/* springleaf:key inline */}` and `{#attr}`.\n\n```mdx\n{/* springleaf:key fence */}\n{#literal}\n```\n";
    let p = good(source);
    assert!(keyed(&p).is_empty());
    let all = nodes(&p.tree);
    assert!(
        all.iter()
            .any(|n| n.kind == Kind::InlineCode && n.value.as_deref() == Some("{/* springleaf:key inline */}"))
    );
    assert_eq!(all.iter().filter(|n| n.kind == Kind::Code).count(), 1);
}

#[test]
fn known_failure_native_flow_comments_interrupt_multiline_inline_code() {
    // Failed desired invariant retained as an executable counterexample, NOT
    // papered over with a second lexer. The baseline controls isolate the cause.
    let source = "Code `first\n{/* springleaf:key literal */}\nlast`.\n\n> `quote\n> {/* springleaf:key quoted */}\n> end`\n";
    let plain = parse(source, Mode::Plain);
    assert_eq!(nodes(&plain.tree).iter().filter(|n| n.kind == Kind::InlineCode).count(), 2);
    let mdx = parse(source, Mode::Mdx);
    assert!(mdx.diagnostics.is_empty());
    assert_eq!(nodes(&mdx.tree).iter().filter(|n| n.kind == Kind::InlineCode).count(), 0);
    assert_eq!(nodes(&mdx.tree).iter().filter(|n| n.kind == Kind::Comment).count(), 2);
    let p = good(source);
    assert_eq!(keyed(&p).len(), 2, "undesired key capture follows the native MDX AST");
    assert_eq!(text(keyed(&p)[0]), "last`.");
}

#[test]
fn escaped_literals_and_ordinary_comments_do_not_allocate_keys() {
    let p = good("\\{/* springleaf:key literal */}\n\nEscaped \\{#literal} &amp; entity.\n\n{/* ordinary comment */}\nParagraph.\n");
    assert!(keyed(&p).is_empty());
    assert!(text(&p.tree).contains("Escaped {#literal} & entity."));
    assert!(nodes(&p.tree).iter().any(|n| n.kind == Kind::Comment));
}

#[test]
fn duplicates_diagnose_both_without_selecting_a_winner() {
    let p = parse("{/* springleaf:key same */}\n## One\n\n{/* springleaf:key same */}\nTwo.\n", Mode::Author);
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "duplicate-key").count(), 2);
    assert!(keyed(&p).is_empty());
    for d in &p.diagnostics {
        assert_eq!(&p.original_source()[d.span.start..d.span.end], "{/* springleaf:key same */}");
    }
}

#[test]
fn move_reorder_preserves_key_not_index_or_arena_id() {
    let first = good("{/* springleaf:key one */}\n## One\n\n{/* springleaf:key two */}\nTwo.\n");
    let second = good("{/* springleaf:key two */}\nTwo.\n\n{/* springleaf:key one */}\n## One\n");
    let a = keyed(&first);
    let b = keyed(&second);
    assert_eq!(a[0].key, b[1].key);
    assert_eq!(text(a[0]), text(b[1]));
    assert_ne!(a[0].span.start, b[1].span.start);
    assert_ne!(a[0].id, b[1].id);
}

#[test]
fn unicode_crlf_exact_annotation_ranges_and_unchanged_source() {
    let source = "{/* springleaf:key intro */}\r\n## Café 🦀\r\n\r\n> {/* springleaf:key prose */}\r\n> naïve prose.\r\n";
    let p = good(source);
    assert_eq!(p.original_source().as_bytes(), source.as_bytes());
    for n in keyed(&p) {
        let s = n.key_span.unwrap();
        assert_eq!(&source[s.start..s.end], format!("{{/* springleaf:key {} */}}", n.key.as_ref().unwrap()));
    }
    let heading = keyed(&p)[0];
    assert_eq!(&source[heading.span.start..heading.span.end], "## Café 🦀");
    // Columns count UTF-16 code units, not UTF-8 bytes or graphemes.
    assert_eq!(heading.span.end_column, 11);
    for n in nodes(&p.tree) {
        assert!(source.get(n.span.start..n.span.end).is_some());
    }
}

#[test]
fn bom_has_explicit_coordinate_base_and_noop_retains_it() {
    let source = "\u{feff}{/* springleaf:key head */}\n## Head\n";
    let p = good(source);
    assert_eq!(p.original_source(), source);
    assert_eq!(p.position_source_start, 3);
    let span = keyed(&p)[0].key_span.unwrap();
    assert_eq!(
        &source[p.position_source_start + span.start..p.position_source_start + span.end],
        "{/* springleaf:key head */}"
    );
}

#[test]
fn marker_can_key_fence_and_rule_without_touching_payload() {
    let p = good("{/* springleaf:key snippet */}\n```md\n{#literal}\n```\n\n{/* springleaf:key rule */}\n---\n");
    assert_eq!(keyed(&p)[0].kind, Kind::Code);
    assert_eq!(keyed(&p)[0].value.as_deref(), Some("{#literal}"));
    let code = keyed(&p)[0];
    assert_eq!(&p.original_source()[code.span.start..code.span.end], "```md\n{#literal}\n```");
    assert_eq!(keyed(&p)[1].kind, Kind::ThematicBreak);
}

#[test]
fn eof_without_newline_and_extended_ascii_key_grammar() {
    let p = good("{/* springleaf:key a_B.c-2 */}\nParagraph without final newline.");
    assert_eq!(keyed(&p)[0].key.as_deref(), Some("a_B.c-2"));
}

#[test]
fn whole_fixture_and_disabled_modes_are_baseline_controls() {
    let source = include_str!("../fixtures/comments.mdx");
    let p = good(source);
    assert_eq!(keyed(&p).len(), 6);
    assert!(keyed(&parse(source, Mode::Mdx)).is_empty());
    let plain = parse(source, Mode::Plain);
    assert!(keyed(&plain).is_empty());
    assert!(text(&plain.tree).contains("springleaf:key"));
}
