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

fn good(source: &str, mode: Mode) -> Parsed {
    let result = parse(source, mode);
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    result
}

#[test]
fn baseline_does_not_treat_mdx_comment_as_metadata() {
    let source = "{/* leafspring:key intro */}\n## Introduction\n";
    let plain = good(source, Mode::Plain);
    assert_eq!(plain.blocks[0].kind, "Paragraph");
    assert_eq!(plain.blocks[0].content, "{/* leafspring:key intro */}");
    assert!(plain.html.contains("{/* leafspring:key intro */}"));
    let custom = good(source, Mode::Comments);
    assert_eq!(keyed(&custom.blocks), [("Heading", "intro", "Introduction")]);
    assert_eq!(custom.html, "<h2>Introduction</h2>\n");
}

#[test]
fn paragraph_and_setext_keep_inlines() {
    let result = good(
        "{/* leafspring:key body */}\nA **bold** [link](https://example.com).\nMore prose.\n\n{/* leafspring:key setext */}\nA *heading*\n===\n",
        Mode::Comments,
    );
    assert_eq!(
        keyed(&result.blocks),
        [("Paragraph", "body", "A bold link. More prose."), ("Heading", "setext", "A heading")]
    );
    assert!(result.html.contains("<strong>bold</strong>"));
    assert!(result.html.contains("<a href=\"https://example.com\">link</a>"));
}

#[test]
fn blank_lines_are_allowed_only_between_directive_and_target() {
    let result = good("Before.\n\n{/* leafspring:key next */}\n\n\nAfter.\n", Mode::Comments);
    assert_eq!(keyed(&result.blocks), [("Paragraph", "next", "After.")]);
    let invalid = parse("Before.\n{/* leafspring:key next */}\nAfter.\n", Mode::Comments);
    assert_eq!(invalid.diagnostics[0].code, "malformed-key");
    assert!(invalid.diagnostics[0].message.contains("insert a blank line"));
}

#[test]
fn list_and_item_prose_have_distinct_explicit_targets() {
    let result = good(
        "{/* leafspring:key list */}\n- {/* leafspring:key first */}\n  First.\n- Second.\n",
        Mode::Comments,
    );
    assert_eq!(keyed(&result.blocks), [("List", "list", ""), ("Paragraph", "first", "First.")]);
    assert!(result.blocks[0].children.iter().all(|item| item.key.is_none()));
}

#[test]
fn nested_quotes_and_lists_bind_in_own_container() {
    let result = good(
        "> > {/* leafspring:key nested */}\n> > Deep.\n\n- Outer.\n\n  > {/* leafspring:key inside */}\n  > Quoted.\n",
        Mode::Comments,
    );
    assert_eq!(keyed(&result.blocks), [("Paragraph", "nested", "Deep."), ("Paragraph", "inside", "Quoted.")]);
}

#[test]
fn markers_cannot_leak_out_of_empty_quote_or_item() {
    for source in [
        "> {/* leafspring:key lost */}\n\nOutside.\n",
        "> {/* leafspring:key lost */}\nOutside.\n",
        "- {/* leafspring:key lost */}\n- Other item.\n",
        "- {/* leafspring:key lost */}\n\nOutside.\n",
    ] {
        let result = parse(source, Mode::Comments);
        assert_eq!(result.diagnostics.len(), 1, "{source:?}");
        assert_eq!(result.diagnostics[0].code, "unbound-key");
        assert!(keyed(&result.blocks).is_empty());
    }
}

#[test]
fn immediate_target_not_arbitrary_later_prose() {
    for source in [
        "{/* leafspring:key before-html */}\n<div>HTML</div>\n\nProse.\n",
        "{/* leafspring:key before-component */}\n<Poll options={[]} />\n\nProse.\n",
        "{/* leafspring:key before-ref */}\n[x]: https://example.com\n\nProse.\n",
        "{/* leafspring:key first */}\n{/* leafspring:key second */}\nProse.\n",
    ] {
        let result = parse(source, Mode::Comments);
        assert_eq!(result.diagnostics[0].code, "unbound-key", "{source:?}");
        assert!(!keyed(&result.blocks).iter().any(|(_, key, _)| *key == "first" || key.starts_with("before-")));
    }
}

#[test]
fn missing_duplicate_malformed_and_multiline_directives() {
    for (source, codes) in [
        ("{/* leafspring:key lost */}\n", vec!["unbound-key"]),
        (
            "{/* leafspring:key same */}\nOne.\n\n{/* leafspring:key same */}\nTwo.\n",
            vec!["duplicate-key", "duplicate-key"],
        ),
        ("{/* leafspring:key has spaces */}\nText.\n", vec!["malformed-key"]),
        ("{/* leafspring:unknown name */}\nText.\n", vec!["malformed-key"]),
        ("{/* leafspring:key */}\nText.\n", vec!["malformed-key"]),
        ("{/* leafspring:key name\n*/}\nText.\n", vec!["malformed-key"]),
    ] {
        let result = parse(source, Mode::Comments);
        assert_eq!(result.diagnostics.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(), codes, "{source:?}");
    }
}

#[test]
fn code_escapes_inline_literals_and_other_namespaces_survive() {
    let source = "Inline `{/* leafspring:key inline */}`.\n\nCode `first\n{/* leafspring:key multiline */}\nlast`.\n\n```md\n{/* leafspring:key fenced */}\n```\n\n    {/* leafspring:key indented */}\n\n\\{/* leafspring:key escaped */}\n\n{/* other:key ordinary */}\n\nLiteral {braces} and {expression}.\n";
    let result = good(source, Mode::Both);
    assert!(keyed(&result.blocks).is_empty());
    for literal in ["inline", "multiline", "fenced", "indented", "escaped", "other:key ordinary", "{expression}"] {
        assert!(result.html.contains(literal), "{literal}");
    }
}

#[test]
fn comments_can_key_fence_or_quote_but_do_not_parse_inside_fence() {
    let result = good(
        "{/* leafspring:key code */}\n```md\n{/* leafspring:key literal */}\n```\n\n{/* leafspring:key quotation */}\n> Quote.\n",
        Mode::Comments,
    );
    assert_eq!(
        keyed(&result.blocks),
        [("CodeBlock", "code", "{/* leafspring:key literal */}\n"), ("Blockquote", "quotation", "")]
    );
}

#[test]
fn modes_are_independent_and_combination_has_no_hidden_precedence() {
    let source = include_str!("../fixtures/collision.md");
    let comments = good(source, Mode::Comments);
    assert_eq!(keyed(&comments.blocks), [("Heading", "from-comment", "Heading {#from-attribute}")]);
    let attributes = good(source, Mode::Attributes);
    assert_eq!(keyed(&attributes.blocks), [("Heading", "from-attribute", "Heading")]);
    assert!(attributes.html.contains("leafspring:key from-comment"));
    let both = parse(source, Mode::Both);
    assert!(keyed(&both.blocks).is_empty());
    assert_eq!(
        both.diagnostics.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(),
        ["key-collision", "key-collision"]
    );
}

#[test]
fn paragraph_collision_same_key_and_cross_syntax_duplicates_are_errors() {
    for (source, code) in [
        ("{/* leafspring:key same */}\nProse.\n{#same}\n", "key-collision"),
        ("{/* leafspring:key first */}\nProse.\n{#second}\n", "key-collision"),
        ("## First {#same}\n\n{/* leafspring:key same */}\nSecond.\n", "duplicate-key"),
    ] {
        let result = parse(source, Mode::Both);
        assert!(keyed(&result.blocks).is_empty());
        assert_eq!(result.diagnostics.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(), [code, code]);
    }
}

#[test]
fn reorder_preserves_comment_keys() {
    let first = good("{/* leafspring:key one */}\nOne.\n\n{/* leafspring:key two */}\n## Two\n", Mode::Comments);
    let second = good("{/* leafspring:key two */}\n## Two\n\n{/* leafspring:key one */}\nOne.\n", Mode::Comments);
    assert_eq!(keyed(&first.blocks)[0], keyed(&second.blocks)[1]);
    assert_eq!(keyed(&first.blocks)[1], keyed(&second.blocks)[0]);
}

#[test]
fn crlf_unicode_spans_and_no_op_bytes_are_exact() {
    let source = "{/* leafspring:key intro */}\r\n## Café 🦀\r\n\r\n> {/* leafspring:key quote */}\r\n> naïve.\r\n";
    let result = good(source, Mode::Comments);
    assert_eq!(result.original_source().as_bytes(), source.as_bytes());
    fn check(blocks: &[Block], source: &str) {
        for block in blocks {
            if let Some(span) = block.key_span {
                assert_eq!(
                    &source[span.start..span.end],
                    format!("{{/* leafspring:key {} */}}", block.key.as_ref().unwrap())
                );
            }
            check(&block.children, source);
        }
    }
    check(&result.blocks, source);
}

#[test]
fn full_fixtures_are_deterministic_and_composable() {
    let comments = good(include_str!("../fixtures/comments.md"), Mode::Comments);
    assert_eq!(keyed(&comments.blocks).len(), 7);
    let both = good(include_str!("../fixtures/combined.md"), Mode::Both);
    assert_eq!(keyed(&both.blocks).len(), 4);
    let comments_in_both = good(include_str!("../fixtures/comments.md"), Mode::Both);
    assert_eq!(comments.blocks, comments_in_both.blocks);
    let attributes = good(include_str!("../fixtures/attributes.md"), Mode::Both);
    assert_eq!(attributes.blocks, parse(include_str!("../fixtures/attributes.md"), Mode::Attributes).blocks);
}
