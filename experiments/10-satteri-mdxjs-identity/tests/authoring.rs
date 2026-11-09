use satteri_mdxjs_identity_experiment::{Checked, Node, check, lower, metadata, reachable};

fn nodes(node: &Node) -> Vec<&Node> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}
fn keyed<'a>(p: &'a Checked<'_>) -> Vec<&'a Node> {
    nodes(&p.tree).into_iter().filter(|n| n.key.is_some()).collect()
}
fn good(source: &str) -> Checked<'_> {
    let p = check(source);
    assert!(p.diagnostics.is_empty(), "{source:?}: {:?}", p.diagnostics);
    p
}
fn has(source: &str, code: &str) {
    let p = check(source);
    assert!(p.diagnostics.iter().any(|e| e.code == code), "{source:?}: {:?}", p.diagnostics);
}

#[test]
fn real_block_metadata_and_inline_structure() {
    let source = include_str!("../fixtures/comments.mdx");
    let p = good(source);
    assert_eq!(
        keyed(&p).iter().map(|n| (n.key.as_deref().unwrap(), n.kind.as_str())).collect::<Vec<_>>(),
        [
            ("introduction", "heading"),
            ("central-argument", "paragraph"),
            ("list", "list"),
            ("first", "paragraph"),
            ("quote", "blockquote"),
            ("quoted", "paragraph")
        ]
    );
    for k in ["strong", "emphasis", "link", "break"] {
        assert!(nodes(&p.tree).iter().any(|n| n.kind == k), "{k}");
    }
    let (arena, errors) = lower(source);
    assert!(errors.is_empty());
    assert_eq!(
        reachable(&arena, 0)
            .iter()
            .filter(|&&id| metadata(&arena, id)["leafbudKey"].is_string())
            .count(),
        6
    );
    assert!(!nodes(&p.tree).iter().any(|n| n.kind == "mdxFlowExpression"));
    assert!(nodes(&p.tree).iter().filter(|n| n.kind == "listItem").all(|n| n.key.is_none()));
}

#[test]
fn nested_native_container_attachment_and_blank_after() {
    let p = good("> > {/* leafbud:key nested */}\n> >\n> > Deep prose.\n\n- Parent.\n  - {/* leafbud:key child */}\n    Child.\n");
    assert_eq!(keyed(&p).iter().map(|n| n.key.as_deref().unwrap()).collect::<Vec<_>>(), ["nested", "child"]);
}

#[test]
fn no_escape_from_quote_item_or_component() {
    for source in [
        "> {/* leafbud:key lost */}\n\nRoot.\n",
        "- {/* leafbud:key lost */}\n- Other.\n",
        "<Page>\n{/* leafbud:key lost */}\n</Page>\n\nRoot.\n",
    ] {
        has(source, "unbound-key");
        assert!(keyed(&check(source)).is_empty());
    }
}

#[test]
fn barriers_malformed_misplaced_and_duplicate_keys() {
    has("{/* leafbud:key a */}\n{/* leafbud:key b */}\nText.\n", "unsupported-target");
    has("{/* leafbud:key a */}\n[ref]: https://example.com\n\nText.\n", "unsupported-target");
    for marker in [
        "{/* leafbud:key 123 */}",
        "{/* leafbud:key é */}",
        "{/* leafbud:other a */}",
        "{/* leafbud:key two words */}",
    ] {
        has(&format!("{marker}\nText.\n"), "malformed-key");
    }
    has("Text {/* leafbud:key a */}.\n", "misplaced-key");
    has("{/* leafbud:key unfinished", "parse");
    let p = check("{/* leafbud:key same */}\n## One\n\n<Poll key=\"same\" />\n");
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "duplicate-key").count(), 2);
    assert!(keyed(&p).is_empty());
    let p = check("{/* leafbud:key one */}\n<Poll key=\"two\" />\n");
    assert!(p.diagnostics.iter().any(|d| d.code == "key-collision"));
    assert!(keyed(&p).is_empty());
}

#[test]
fn code_escape_and_fences_are_not_scanned() {
    let p = good("Inline `{/* leafbud:key inline */}`.\n\n```mdx\n{/* leafbud:key fence */}\n{#literal}\n```\n\n\\{/* leafbud:key escaped */}\n");
    assert!(keyed(&p).is_empty());
    assert_eq!(nodes(&p.tree).iter().filter(|n| n.kind == "inlineCode").count(), 1);
    assert_eq!(nodes(&p.tree).iter().filter(|n| n.kind == "code").count(), 1);
}

#[test]
fn multiline_code_counterexample_rejected_not_repaired() {
    let source = include_str!("../fixtures/multiline-code.mdx");
    let p = check(source);
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "key-boundary").count(), 2);
    assert!(keyed(&p).is_empty());
    assert!(!nodes(&p.tree).iter().any(|n| n.kind == "inlineCode"), "native code semantics are STILL lost");
    assert_eq!(p.original_source, source);
}

#[test]
fn indented_comment_still_interpreted_as_key_not_code_limitation() {
    let p = good("    {/* leafbud:key indented */}\n\nFollowing.\n");
    assert_eq!(keyed(&p)[0].key.as_deref(), Some("indented"));
    assert!(!nodes(&p.tree).iter().any(|n| n.kind == "code"));
}

#[test]
fn prior_sibling_requires_blank_but_target_does_not() {
    has("Before.\n{/* leafbud:key after */}\nAfter.\n", "key-boundary");
    assert_eq!(
        keyed(&good("Before.\n\n{/* leafbud:key after */}\nAfter.\n"))[0].key.as_deref(),
        Some("after")
    );
}

#[test]
fn unicode_crlf_bom_byte_ranges_and_noop_source() {
    let source = "\u{feff}{/* leafbud:key intro */}\r\n## Café 🦀\r\n\r\n> {/* leafbud:key quote */}\r\n> naïve.\r\n";
    let p = good(source);
    assert_eq!(p.original_source.as_bytes(), source.as_bytes());
    assert_eq!(p.position_source_start, 3);
    for n in nodes(&p.tree) {
        assert!(source.get(3 + n.span.start as usize..3 + n.span.end as usize).is_some());
    }
    let heading = keyed(&p)[0];
    assert_eq!(&source[3 + heading.span.start as usize..3 + heading.span.end as usize], "## Café 🦀");
    assert_eq!(heading.span.end_column, 11);
    let s = &heading.data["leafbudKeySpan"];
    assert_eq!(
        &source[3 + s["start"].as_u64().unwrap() as usize..3 + s["end"].as_u64().unwrap() as usize],
        "{/* leafbud:key intro */}"
    );
}

#[test]
fn reorder_changes_spans_not_keys() {
    let a = good("{/* leafbud:key one */}\n## One\n\n{/* leafbud:key two */}\nTwo.\n");
    let b = good("{/* leafbud:key two */}\nTwo.\n\n{/* leafbud:key one */}\n## One\n");
    assert_eq!(keyed(&a)[0].key, keyed(&b)[1].key);
    assert_ne!(keyed(&a)[0].span, keyed(&b)[1].span);
}

#[test]
fn literal_component_tree_and_exact_data() {
    let p = good(include_str!("../fixtures/components.mdx"));
    let all = nodes(&p.tree);
    let named = |name| *all.iter().find(|n| n.details["name"] == name).unwrap();
    assert_eq!(named("Poll").key.as_deref(), Some("topic"));
    assert_eq!(named("Poll").data["literalProps"]["options"], serde_json::json!(["Gardens", "Maps"]));
    assert_eq!(named("Image").data["literalProps"]["alt"], "A & B");
    assert_eq!(named("Run").data["literalProps"]["text"], "  literal\ntext <>{} 🦀");
    assert!(nodes(named("Page")).iter().any(|n| n.details["name"] == "Canvas"));
    assert!(nodes(named("Canvas")).iter().any(|n| n.kind == "strong"));
    assert_eq!(named("Place").data["literalProps"]["rotation"], -2.75);
    assert_eq!(named("Run").kind, "mdxJsxTextElement");
}

#[test]
fn literal_gate_rejects_javascript_and_recursive_duplicate_json_keys() {
    for raw in [
        "load()",
        "a.b",
        "1 + 2",
        "() => 1",
        "`template`",
        "undefined",
        "NaN",
        "Infinity",
        "({\"a\":1})",
        "{a:1}",
        "{'a':1}",
        "[1,]",
        "[/*comment*/1]",
        "{\"a\":{\"b\":1,\"b\":2}}",
    ] {
        has(&format!("<Poll options={{{raw}}} />\n"), "nonliteral-prop");
    }
    has("<Poll {...props} />", "spread");
    has("<Poll options={[]} options={[]} />", "duplicate-prop");
    has("<Mystery />", "unknown-component");
    has("<div />", "unknown-component");
    has("<></>", "unknown-component");
    has("import X from 'x'\n\nText.\n", "esm");
    has("{1 + 2}\n", "body-expression");
}

#[test]
fn expression_strings_are_not_global_directive_replacements() {
    let p = good("<Run text={\"{/* leafbud:key fake */} {#fake}\"} />\n");
    assert!(keyed(&p).is_empty());
    assert_eq!(p.tree.children[0].data["literalProps"]["text"], "{/* leafbud:key fake */} {#fake}");
}
