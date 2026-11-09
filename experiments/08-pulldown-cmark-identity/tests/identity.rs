use pulldown_cmark::{Event, Tag};
use pulldown_identity_experiment::{Mode, Node, Parsed, parse};

fn keys(nodes: &[Node]) -> Vec<(&str, &str, String)> {
    let mut result = Vec::new();
    for node in nodes {
        if let Some(key) = &node.key {
            result.push((node.kind(), key.as_str(), node.text()));
        }
        result.extend(keys(&node.children));
    }
    result
}

fn good(source: &str) -> Parsed {
    let result = parse(source, Mode::Attributes);
    assert!(result.diagnostics.is_empty(), "{source:?}: {:#?}", result.diagnostics);
    result
}

#[test]
fn preferred_examples_are_internal_not_html_anchor_ids() {
    let result = good("## Introduction {#introduction}\n\nOrdinary **formatting**.\n{#central-argument}\n");
    assert_eq!(
        keys(&result.blocks),
        [
            ("Heading", "introduction", "Introduction".into()),
            ("Paragraph", "central-argument", "Ordinary formatting.".into())
        ]
    );
    assert_eq!(result.html, "<h2>Introduction</h2>\n<p>Ordinary <strong>formatting</strong>.</p>\n");
    assert!(!result.html.contains("id="));
}

#[test]
fn inline_structure_destinations_entities_and_breaks_survive() {
    let result = good("A **bold** *emphasis* [link](https://example.com \"Title\") &amp; \\*literal\\*\nwrap  \nbreak\n{#body}\n");
    assert!(result.html.contains("<strong>bold</strong> <em>emphasis</em>"));
    assert!(result.html.contains("<a href=\"https://example.com\" title=\"Title\">link</a>"));
    assert!(result.html.contains("&amp; *literal*\nwrap<br />\nbreak"));
    assert!(
        result.blocks[0]
            .children
            .iter()
            .any(|n| matches!(&n.event, Event::Start(Tag::Link { dest_url, .. }) if dest_url.as_ref() == "https://example.com"))
    );
    assert!(result.blocks[0].text().contains("& *literal* wrap\nbreak"));
}

#[test]
fn setext_and_closing_hashes_follow_plain_parser_content() {
    let result = good("Setext *heading* {#setext}\n===\n\n## Closed {#closed} ##\n");
    assert_eq!(
        keys(&result.blocks),
        [("Heading", "setext", "Setext heading".into()), ("Heading", "closed", "Closed".into())]
    );
}

#[test]
fn tight_items_get_derived_paragraphs_not_item_keys() {
    let result = good("- First.\n  {#first}\n- Second.\n  - Child.\n    {#child}\n");
    assert_eq!(
        keys(&result.blocks),
        [("Paragraph", "first", "First.".into()), ("Paragraph", "child", "Child.".into())]
    );
    assert_eq!(result.blocks[0].kind(), "List");
    let item = &result.blocks[0].children[0];
    assert_eq!(item.kind(), "Item");
    assert!(item.key.is_none());
    assert_eq!(item.children[0].span_origin, "derived-tight-inline-hull");
    assert!(!result.html.contains("<li><p>"));
}

#[test]
fn loose_items_retain_upstream_paragraph_tags() {
    let result = good("- First.\n  {#first}\n\n- Second.\n  {#second}\n");
    assert_eq!(keys(&result.blocks).len(), 2);
    assert_eq!(result.blocks[0].children[0].children[0].span_origin, "upstream-event");
    assert!(result.html.contains("<li>\n<p>First.</p>"));
}

#[test]
fn nested_quotes_and_quoted_lists_are_explicit() {
    let result = good("> > ## Head {#head}\n> >\n> > Paragraph.\n> > {#para}\n\n> - List prose.\n>   {#listed}\n");
    assert_eq!(
        keys(&result.blocks).iter().map(|(_, key, _)| *key).collect::<Vec<_>>(),
        ["head", "para", "listed"]
    );
}

#[test]
fn lazy_marker_continuations_are_rejected_not_keyed_to_container() {
    for source in ["- Item.\n{#outside}\n", "> Quote.\n{#outside}\n", "> > Quote.\n> {#outside}\n"] {
        let result = parse(source, Mode::Attributes);
        assert_eq!(result.diagnostics.len(), 1, "{source:?}");
        assert_eq!(result.diagnostics[0].code, "unbound-key");
        assert!(keys(&result.blocks).is_empty());
    }
}

#[test]
fn blanks_or_other_siblings_do_not_get_skipped() {
    for source in [
        "{#orphan}\n",
        "Paragraph.\n\n{#blank}\n",
        "## Heading\n{#wrong}\n",
        "Prose.\n\n---\n\n{#after-rule}\n",
        "[ref]: https://example.com\n{#ref-only}\n",
    ] {
        let result = parse(source, Mode::Attributes);
        assert_eq!(result.diagnostics.len(), 1, "{source:?}");
        assert_eq!(result.diagnostics[0].code, "unbound-key");
        assert!(keys(&result.blocks).is_empty());
    }
}

#[test]
fn marker_in_middle_is_not_silent_prose() {
    let result = parse("Before.\n{#middle}\nAfter.\n", Mode::Attributes);
    assert_eq!(result.diagnostics[0].code, "misplaced-key");
    assert!(keys(&result.blocks).is_empty());
}

#[test]
fn invalid_key_spellings_and_reserved_trailing_junk_diagnose() {
    for source in [
        "## Heading {#123}\n",
        "## Heading {#é}\n",
        "## Heading {#has *spaces*}\n",
        "## Heading {#unfinished\n",
        "Prose.\n{#}\n",
        "Prose.\n{#has spaces}\n",
        "Prose.\n{#body} junk\n",
    ] {
        let result = parse(source, Mode::Attributes);
        assert_eq!(result.diagnostics.len(), 1, "{source:?}");
        assert_eq!(result.diagnostics[0].code, "malformed-key", "{source:?}");
        assert!(keys(&result.blocks).is_empty());
    }
}

#[test]
fn duplicate_keys_invalidate_all_occurrences() {
    let result = parse("## One {#same}\n\nTwo.\n{#same}\n", Mode::Attributes);
    assert_eq!(
        result.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
        ["duplicate-key", "duplicate-key"]
    );
    assert!(keys(&result.blocks).is_empty());
}

#[test]
fn multiple_keys_on_same_block_do_not_silently_win() {
    for (source, count) in [("## H {#one} {#two}\n", 1), ("Prose.\n{#one}\n{#two}\n", 2)] {
        let result = parse(source, Mode::Attributes);
        assert_eq!(result.diagnostics.len(), count);
        assert!(result.diagnostics.iter().all(|d| d.code == "key-collision"));
        assert!(keys(&result.blocks).is_empty());
    }
}

#[test]
fn code_literal_and_escaped_shapes_match_baseline() {
    let source = "## Code `{#inline}`\n\n## Escaped \\{#escaped}\n\nEscaped \\{#line}. Ordinary {braces}.\n\n\\{#standalone}\n\n```md\n{#fenced}\n{/* leafbud:key literal */}\n```\n\n    {#indented}\n\nParagraph {#same-line-literal}\n";
    let result = good(source);
    assert!(keys(&result.blocks).is_empty());
    assert_eq!(result.html, parse(source, Mode::Plain).html);
}

#[test]
fn multiline_inline_code_is_claimed_before_marker_recognition() {
    let source = "Code `first\n{#literal}\nlast`.\n\n> `quote\n> {#literal}\n> end`\n";
    let result = good(source);
    assert!(keys(&result.blocks).is_empty());
    assert_eq!(result.html, parse(source, Mode::Plain).html);
    assert!(result.html.contains("<code>first {#literal} last</code>"));
}

#[test]
fn heading_link_and_emphasis_payloads_are_not_keys() {
    let source = "## [Label {#literal}](https://example.com)\n\n## **{#literal}**\n";
    let result = good(source);
    assert!(keys(&result.blocks).is_empty());
    assert_eq!(result.html, parse(source, Mode::Plain).html);
}

#[test]
fn raw_html_payload_is_not_scanned_for_keys() {
    let source = "<div>\n{#literal}\n{/* leafbud:key literal */}\n</div>\n";
    let result = good(source);
    assert!(keys(&result.blocks).is_empty());
    assert_eq!(result.html, source);
}

#[test]
fn eof_and_key_alphabet_are_supported() {
    for source in ["## H {#a_B.c-2}", "Prose.\n{#a_B.c-2}", "## H {#a_b_c}"] {
        assert_eq!(keys(&good(source).blocks)[0].1, source.split("{#").nth(1).unwrap().trim_end_matches('}'));
    }
}

#[test]
fn moving_blocks_retains_keys_not_offsets() {
    let a = good("## One {#one}\n\nTwo.\n{#two}\n");
    let b = good("Two.\n{#two}\n\n## One {#one}\n");
    assert_eq!(keys(&a.blocks)[0], keys(&b.blocks)[1]);
    assert_eq!(keys(&a.blocks)[1], keys(&b.blocks)[0]);
    assert_ne!(a.blocks[0].key_span, b.blocks[1].key_span);
}

#[test]
fn crlf_unicode_original_bytes_and_exact_key_ranges() {
    let source = "## Café 🦀 {#intro}\r\n\r\nnaïve prose.\r\n{#body}\r\n\r\n- élève\r\n  {#item}\r\n";
    let result = good(source);
    assert_eq!(result.original_source.as_bytes(), source.as_bytes());
    fn check(nodes: &[Node], source: &str) {
        for node in nodes {
            assert!(source.get(node.span.clone()).is_some());
            if let Some(key) = &node.key {
                assert_eq!(&source[node.key_span.clone().unwrap()], format!("{{#{key}}}"));
            }
            check(&node.children, source);
        }
    }
    check(&result.blocks, source);
    assert_eq!(keys(&result.blocks).len(), 3);
    for record in &result.raw_events {
        assert!(source.get(record.span.clone()).is_some());
    }
}

#[test]
fn fixture_and_plain_control_are_distinct() {
    let source = include_str!("../fixtures/attributes.md");
    let result = good(source);
    assert_eq!(keys(&result.blocks).len(), 6);
    let plain = parse(source, Mode::Plain);
    assert!(keys(&plain.blocks).is_empty());
    assert!(plain.html.contains("{#introduction}"));
    assert_eq!(
        serde_json::to_value(result.raw_events).unwrap(),
        serde_json::to_value(plain.raw_events).unwrap()
    );
}
