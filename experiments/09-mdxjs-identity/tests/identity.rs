use mdxjs_identity_experiment::{Kind, Parsed, Projected, parse};

fn good(source: &str) -> Parsed<'_> {
    let parsed = parse(source).unwrap();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
    parsed
}

fn keyed(node: &Projected) -> Vec<(&str, Kind)> {
    let mut keys = Vec::new();
    if let Some(key) = &node.key {
        keys.push((key.as_str(), node.kind));
    }
    for child in &node.children {
        keys.extend(keyed(child));
    }
    keys
}

fn text(node: &Projected) -> String {
    node.text.clone().unwrap_or_else(|| node.children.iter().map(text).collect())
}

fn codes(source: &str) -> Vec<&'static str> {
    parse(source).unwrap().diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn preceding_comments_attach_heading_and_formatted_prose_without_wrappers() {
    let parsed =
        good("{/* leafbud:key intro */}\n## A *heading*\n\n{/* leafbud:key body */}\nOrdinary **bold** and [linked](https://example.com) prose.\n");
    assert_eq!(keyed(&parsed.tree), [("intro", Kind::Heading), ("body", Kind::Paragraph)]);
    assert_eq!(text(&parsed.tree.children[0]), "A heading");
    assert_eq!(parsed.tree.children[0].children[1].kind, Kind::Emphasis);
    assert_eq!(parsed.tree.children[1].children[1].kind, Kind::Strong);
    assert_eq!(parsed.tree.children[1].children[3].kind, Kind::Link);
}

#[test]
fn blank_lines_before_target_are_allowed_and_setext_is_supported() {
    let parsed = good("Before.\n\n{/* leafbud:key title */}\n\n\nSetext\n===\n");
    assert_eq!(keyed(&parsed.tree), [("title", Kind::Heading)]);
}

#[test]
fn lists_and_nested_item_paragraphs_are_distinct_targets() {
    let parsed = good("{/* leafbud:key list */}\n- {/* leafbud:key first */}\n  First.\n\n  - {/* leafbud:key nested */}\n    Nested.\n- Second.\n");
    assert_eq!(
        keyed(&parsed.tree),
        [("list", Kind::List), ("first", Kind::Paragraph), ("nested", Kind::Paragraph)]
    );
    let item = &parsed.tree.children[0].children[0];
    assert_eq!(item.kind, Kind::ListItem);
    assert!(item.key.is_none());
}

#[test]
fn quote_and_list_attachments_are_container_local() {
    let parsed = good("> > {/* leafbud:key deep */}\n> > Deep.\n\n- Outer.\n\n  > {/* leafbud:key inner */}\n  > Quoted.\n");
    assert_eq!(keyed(&parsed.tree), [("deep", Kind::Paragraph), ("inner", Kind::Paragraph)]);
    for source in [
        "> {/* leafbud:key lost */}\n\nOutside.\n",
        "- {/* leafbud:key lost */}\n- Other item.\n",
        "- {/* leafbud:key lost */}\n\nOutside.\n",
    ] {
        let parsed = parse(source).unwrap();
        assert_eq!(parsed.diagnostics[0].code, "unbound-key", "{source:?}");
        assert!(keyed(&parsed.tree).is_empty());
    }
}

#[test]
fn quote_code_list_and_rule_can_be_keyed() {
    let parsed = good(
        "{/* leafbud:key quote */}\n> Text.\n\n{/* leafbud:key fence */}\n```mdx\n{/* leafbud:key literal */}\n```\n\n{/* leafbud:key rule */}\n***\n",
    );
    assert_eq!(
        keyed(&parsed.tree),
        [("quote", Kind::Blockquote), ("fence", Kind::Code), ("rule", Kind::ThematicBreak)]
    );
    assert_eq!(parsed.tree.children[1].text.as_deref(), Some("{/* leafbud:key literal */}"));
}

#[test]
fn inline_and_fenced_code_escapes_and_other_comments_do_not_bind() {
    let source = "Inline `{#id}` and `{/* leafbud:key inline */}`.\n\nCode `first\ncontinued {/* leafbud:key multiline */}\nlast`.\n\n```mdx\n{/* leafbud:key fenced */}\n```\n\n\\{/* leafbud:key escaped */}\n\n{/* another:key normal */}\n";
    let parsed = good(source);
    assert!(keyed(&parsed.tree).is_empty());
    assert!(text(&parsed.tree).contains("{/* leafbud:key escaped */}"));
    assert_eq!(parsed.tree.children.last().unwrap().kind, Kind::Comment);
}

#[test]
fn inline_identity_comments_are_not_silently_metadata() {
    assert_eq!(codes("Text {/* leafbud:key body */} more.\n"), ["misplaced-key"]);
}

#[test]
fn malformed_reserved_namespace_and_orphans_diagnose() {
    for source in [
        "{/* leafbud:key two words */}\nText.\n",
        "{/* leafbud:key */}\nText.\n",
        "{/* leafbud:other id */}\nText.\n",
        "{/* leafbud:key café */}\nText.\n",
        "{/* leafbud:key name\n*/}\nText.\n",
    ] {
        assert_eq!(codes(source), ["malformed-key"], "{source:?}");
    }
    assert_eq!(codes("{/* leafbud:key orphan */}\n"), ["unbound-key"]);
    // Truly broken MDX is a baseline parse error, not an adapter diagnostic.
    assert!(parse("{/* leafbud:key unterminated\nText.\n").is_err());
}

#[test]
fn duplicate_document_keys_remove_both_bindings() {
    let parsed = parse("{/* leafbud:key same */}\nOne.\n\n{/* leafbud:key same */}\n## Two\n").unwrap();
    assert_eq!(
        parsed.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
        ["duplicate-key", "duplicate-key"]
    );
    assert!(keyed(&parsed.tree).is_empty());
}

#[test]
fn consecutive_comments_do_not_skip_each_other_or_definitions() {
    let parsed = parse("{/* leafbud:key first */}\n\n{/* leafbud:key second */}\nText.\n").unwrap();
    assert_eq!(parsed.diagnostics[0].code, "unbound-key");
    assert_eq!(keyed(&parsed.tree), [("second", Kind::Paragraph)]);
    assert_eq!(codes("{/* leafbud:key blocked */}\n[x]: https://example.com\n\nText.\n"), ["unbound-key"]);
    assert_eq!(codes("{/* leafbud:key blocked */}\n{/* ordinary comment */}\nText.\n"), ["unbound-key"]);
}

#[test]
fn mdx_flow_interruption_cannot_turn_apparent_multiline_code_into_identity() {
    let source = "Code `first\n{/* leafbud:key accidental */}\nlast`.\n";
    let parsed = parse(source).unwrap();
    assert_eq!(parsed.diagnostics[0].code, "misplaced-key");
    assert!(keyed(&parsed.tree).is_empty());
    assert_eq!(codes("Before.\n{/* leafbud:key next */}\nNext.\n"), ["misplaced-key"]);
}

#[test]
fn moving_blocks_changes_positions_not_author_keys() {
    let a = good("{/* leafbud:key one */}\nOne.\n\n{/* leafbud:key two */}\n## Two\n");
    let b = good("{/* leafbud:key two */}\n## Two\n\n{/* leafbud:key one */}\nOne.\n");
    assert_eq!(keyed(&a.tree)[0], keyed(&b.tree)[1]);
    assert_eq!(keyed(&a.tree)[1], keyed(&b.tree)[0]);
    assert_ne!(a.tree.children[0].span, b.tree.children[1].span);
}

#[test]
fn utf8_byte_ranges_crlf_and_original_bytes_are_preserved() {
    let source = "Préface 🦀.\r\n\r\n> {/* leafbud:key cafe-shape */}\r\n> ## Naïve 🦀\r\n";
    let parsed = good(source);
    assert_eq!(parsed.original_source.as_bytes(), source.as_bytes());
    let heading = &parsed.tree.children[1].children[0];
    let span = heading.key_span.unwrap();
    assert_eq!(&source[span.start..span.end], "{/* leafbud:key cafe-shape */}");
}

#[test]
fn ascii_key_policy_is_independent_of_unicode_prose() {
    let source = "Préface 🦀.\r\n\r\n> {/* leafbud:key intro */}\r\n> ## Naïve 🦀\r\n";
    let parsed = good(source);
    let heading = &parsed.tree.children[1].children[0];
    let key_span = heading.key_span.unwrap();
    assert_eq!(&source[key_span.start..key_span.end], "{/* leafbud:key intro */}");
    let heading_span = heading.span.unwrap();
    assert_eq!(&source[heading_span.start..heading_span.end], "## Naïve 🦀");
    assert_eq!(parsed.original_source.as_bytes(), source.as_bytes());
}

#[test]
fn accepted_expression_shaped_ids_are_rejected_by_selected_comment_dialect() {
    let parsed = parse(include_str!("../fixtures/attribute-conflict.mdx")).unwrap();
    assert_eq!(
        parsed.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
        ["executable-expression", "executable-expression"]
    );
    assert!(keyed(&parsed.tree).is_empty());
}

#[test]
fn comment_key_grammar_does_not_depend_on_javascript_member_expression_grammar() {
    let parsed = good("{/* leafbud:key trailing. */}\nText.\n\n{/* leafbud:key two..dots */}\n## Heading\n");
    assert_eq!(keyed(&parsed.tree), [("trailing.", Kind::Paragraph), ("two..dots", Kind::Heading)]);
}

#[test]
fn normal_markdown_structure_and_hard_break_survive_projection() {
    let parsed = good("3. First **bold**.\n4. Second.\n\nText\\\nnext with ![image](./asset.png \"title\").\n");
    let mdxjs_identity_experiment::Details::List { ordered, start, .. } = parsed.tree.children[0].details else {
        panic!("expected list")
    };
    assert!(ordered);
    assert_eq!(start, Some(3));
    let paragraph = &parsed.tree.children[1];
    assert_eq!(paragraph.children[1].kind, Kind::Break);
    assert_eq!(paragraph.children[3].kind, Kind::Image);
    assert!(keyed(&parsed.tree).is_empty());
}

#[test]
fn fixture_is_a_clean_six_key_projection() {
    let parsed = good(include_str!("../fixtures/comments.mdx"));
    assert_eq!(keyed(&parsed.tree).len(), 6);
}
