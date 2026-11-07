use rushdown_identity_experiment::{Block, Mode, Parsed, parse};

fn keyed(blocks: &[Block]) -> Vec<(&str, &str, &str)> {
    let mut result = Vec::new();
    for block in blocks {
        if let Some(key) = &block.key {
            result.push((block.kind.as_str(), key.as_str(), block.content.as_str()));
        }
        result.extend(keyed(&block.children));
    }
    result
}

fn good(source: &str) -> Parsed {
    let result = parse(source, Mode::Attributes);
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    result
}

#[test]
fn preferred_examples() {
    let result = good("## Introduction {#introduction}\n\nThis is ordinary.\n{#central-argument}\n");
    assert_eq!(
        keyed(&result.blocks),
        [
            ("Heading", "introduction", "Introduction"),
            ("Paragraph", "central-argument", "This is ordinary.")
        ]
    );
    assert_eq!(result.html, "<h2>Introduction</h2>\n<p>This is ordinary.</p>\n");
    assert!(!result.html.contains("id="));
}

#[test]
fn multiline_formatting_links_and_hard_break_survive() {
    let result = good("A **bold** [link](https://example.com)\nsecond  \nthird\n{#body}\n");
    assert_eq!(keyed(&result.blocks), [("Paragraph", "body", "A bold link second\nthird")]);
    assert!(result.html.contains("<strong>bold</strong>"));
    assert!(result.html.contains("<a href=\"https://example.com\">link</a>"));
    assert!(
        result.blocks[0]
            .inlines
            .iter()
            .any(|i| i.kind == "Link" && i.destination.as_deref() == Some("https://example.com"))
    );
}

#[test]
fn setext_and_closing_hashes() {
    let result = good("A *heading* {#setext}\n===\n\n## Closed {#closed} ##\n");
    assert_eq!(keyed(&result.blocks), [("Heading", "setext", "A heading"), ("Heading", "closed", "Closed")]);
}

#[test]
fn list_tail_targets_paragraph_not_item_or_list() {
    let result = good("- First.\n  {#first}\n- Second.\n\n> Quote.\n> {#quote}\n");
    assert_eq!(keyed(&result.blocks), [("Paragraph", "first", "First."), ("Paragraph", "quote", "Quote.")]);
    assert_eq!(result.blocks[0].kind, "List");
    assert_eq!(result.blocks[0].children[0].kind, "ListItem");
    assert_eq!(result.blocks[0].key, None);
    assert_eq!(result.blocks[0].children[0].key, None);
}

#[test]
fn orphan_blank_and_container_exit_are_errors() {
    for source in [
        "{#orphan}\n",
        "Prose.\n\n{#blank}\n",
        "- Item.\n{#outside}\n",
        "> Quote.\n{#outside}\n",
        "## Heading\n{#wrong-form}\n",
    ] {
        let result = parse(source, Mode::Attributes);
        assert_eq!(result.diagnostics.len(), 1, "{source:?}");
        assert!(["unbound-key", "malformed-key"].contains(&result.diagnostics[0].code.as_str()));
        assert!(keyed(&result.blocks).is_empty());
    }
}

#[test]
fn code_and_literals_are_not_stolen() {
    let source = "## Code `{#inline}`\n\nEscaped \\{#escaped}. Literal {braces}. Inline `{#code}`.\n\n\\{#escaped-line}\n\n```md\n{#fenced}\n```\n\n    {#indented}\n\nProse {#not-a-paragraph-suffix}\n";
    let result = good(source);
    assert!(keyed(&result.blocks).is_empty());
    assert!(result.html.contains("{#inline}"));
    assert!(result.html.contains("{#escaped-line}"));
    assert!(result.html.contains("{#fenced}"));
    assert!(result.html.contains("{#indented}"));
    assert!(result.html.contains("Prose {#not-a-paragraph-suffix}"));
}

#[test]
fn multiline_inline_code_is_not_interrupted_by_attribute_lines() {
    let result = good("Code `first\n{#not-a-key}\nlast` here.\n\n> `quote\n> {#not-a-key}\n> end`\n");
    assert!(keyed(&result.blocks).is_empty());
    assert!(result.html.contains("<code>first {#not-a-key} last</code>"));
}

#[test]
fn attribute_in_middle_is_an_error_not_silent_prose() {
    let result = parse("Prose.\n{#not-final}\nMore prose.\n", Mode::Attributes);
    assert_eq!(result.diagnostics[0].code, "malformed-key");
    assert!(result.diagnostics[0].message.contains("final logical line"));
}

#[test]
fn duplicate_and_malformed_diagnose() {
    for (source, codes) in [
        ("## One {#same}\n\nTwo.\n{#same}\n", vec!["duplicate-key", "duplicate-key"]),
        ("Prose.\n{#has spaces}\n", vec!["malformed-key"]),
        ("## Heading {#123}\n", vec!["malformed-key"]),
        ("Prose.\n{#key} extra\n", vec!["malformed-key"]),
    ] {
        let result = parse(source, Mode::Attributes);
        assert_eq!(result.diagnostics.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(), codes);
        assert!(keyed(&result.blocks).is_empty());
    }
}

#[test]
fn reorder_keeps_author_identity_not_array_position() {
    let first = good("## One {#one}\n\nTwo.\n{#two}\n");
    let second = good("Two.\n{#two}\n\n## One {#one}\n");
    assert_eq!(keyed(&first.blocks)[0], keyed(&second.blocks)[1]);
    assert_eq!(keyed(&first.blocks)[1], keyed(&second.blocks)[0]);
}

#[test]
fn crlf_unicode_exact_annotation_spans_and_unchanged_source() {
    let source = "## Café 🦀 {#intro}\r\n\r\nnaïve prose.\r\n{#body}\r\n";
    let result = good(source);
    assert_eq!(result.original_source().as_bytes(), source.as_bytes());
    for block in &result.blocks {
        let span = block.key_span.unwrap();
        assert_eq!(&source[span.start..span.end], format!("{{#{}}}", block.key.as_ref().unwrap()));
        for segment in &block.content_segments {
            assert!(source.get(segment.start..segment.end).is_some());
        }
    }
    assert_eq!(keyed(&result.blocks), [("Heading", "intro", "Café 🦀"), ("Paragraph", "body", "naïve prose.")]);
}

#[test]
fn full_fixture_and_plain_mode() {
    let source = include_str!("../fixtures/attributes.md");
    let result = good(source);
    assert_eq!(keyed(&result.blocks).len(), 5);
    let plain = parse(source, Mode::Plain);
    assert!(keyed(&plain.blocks).is_empty());
    assert!(plain.html.contains("{#introduction}"));
    assert_eq!(result.original_source(), plain.original_source());
}
