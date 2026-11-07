use mdxjs_identity_experiment::{Details, KeyOrigin, Kind, Parsed, Projected, parse};
use serde_json::{Value, json};

fn good(source: &str) -> Parsed<'_> {
    let parsed = parse(source).unwrap();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
    parsed
}

fn property<'a>(node: &'a Projected, name: &str) -> &'a Value {
    let Details::Component { attributes, .. } = &node.details else {
        panic!("not component")
    };
    attributes.iter().find(|a| a.name == name).unwrap().value.as_ref().unwrap()
}

fn diagnostics(source: &str) -> Vec<&'static str> {
    parse(source).unwrap().diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn poll_retains_key_ordered_properties_arrays_and_objects() {
    let parsed = good("<Poll key=\"topic\" options={[\"Gardens\", \"Maps\"]} config={{\"anonymous\": true, \"limit\": 2}} />\n");
    let poll = &parsed.tree.children[0];
    assert_eq!(poll.kind, Kind::Component);
    assert_eq!(poll.key.as_deref(), Some("topic"));
    assert_eq!(poll.key_origin, Some(KeyOrigin::ComponentAttribute));
    assert!(poll.key_span.is_none());
    assert_eq!(property(poll, "options"), &json!(["Gardens", "Maps"]));
    assert_eq!(property(poll, "config"), &json!({"anonymous": true, "limit": 2}));
    let Details::Component { attributes, .. } = &poll.details else {
        unreachable!()
    };
    assert_eq!(attributes.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["key", "options", "config"]);
    assert!(attributes.iter().all(|p| p.span.is_none()));
}

#[test]
fn nested_canvas_and_markdown_keep_order_and_geometry() {
    let parsed = good(include_str!("../fixtures/components.mdx"));
    let canvas = &parsed.tree.children[1];
    assert_eq!(property(canvas, "width"), &json!(800));
    assert_eq!(canvas.children.len(), 2);
    let place = &canvas.children[0];
    assert_eq!(property(place, "x"), &json!(12.25));
    assert_eq!(property(place, "rotation"), &json!(-2.75));
    let image = &place.children[0];
    assert_eq!(property(image, "alt"), &json!("A & B"));
    assert_eq!(canvas.children[1].kind, Kind::Paragraph);
    assert_eq!(canvas.children[1].children[1].kind, Kind::Strong);
}

#[test]
fn exact_text_is_literal_data_not_jsx_whitespace_guessing() {
    let source = "Exact <Run text={\"  literal\\ntext <>{}\\t🦀\"} /> then <HardBreak />end.\n";
    let parsed = good(source);
    let children = &parsed.tree.children[0].children;
    assert_eq!(
        children.iter().map(|n| n.kind).collect::<Vec<_>>(),
        [Kind::Text, Kind::Component, Kind::Text, Kind::Component, Kind::Text]
    );
    assert_eq!(property(&children[1], "text"), &json!("  literal\ntext <>{}\t🦀"));
    assert_eq!(children[2].text.as_deref(), Some(" then "));
    assert_eq!(children[4].text.as_deref(), Some("end."));
    assert_eq!(parsed.original_source, source);
}

#[test]
fn key_shaped_text_inside_expression_strings_is_not_rewritten() {
    let parsed = good("<Run text={\"{#literal} {/* springleaf:key string */}\"} />\n\n<Poll options={[\"{#id}\", \"{/* springleaf:key id */}\"]} />\n");
    assert_eq!(property(&parsed.tree.children[0], "text"), &json!("{#literal} {/* springleaf:key string */}"));
    assert!(parsed.tree.children.iter().all(|n| n.key.is_none()));
}

#[test]
fn boolean_null_numbers_and_nested_literals_are_data() {
    let parsed = good("<Poll disabled config={{\"a\": [null, true, false, -2.75, 1e3, {\"b\": \"x\"}]}} />\n");
    assert_eq!(property(&parsed.tree.children[0], "disabled"), &json!(true));
    assert_eq!(
        property(&parsed.tree.children[0], "config"),
        &json!({"a": [null, true, false, -2.75, 1e3, {"b": "x"}]})
    );
}

#[test]
fn valid_javascript_is_rejected_unless_it_is_strict_json() {
    for expression in [
        "load()",
        "a.b",
        "1 + 2",
        "() => 1",
        "`text`",
        "undefined",
        "NaN",
        "Infinity",
        "({})",
        "{x: 1}",
        "{'x': 1}",
        "[1,]",
        "[...x]",
        "{\"x\": 1, ...y}",
        "/* comment */ 1",
    ] {
        let source = format!("<Poll config={{{expression}}} />\n");
        assert_eq!(diagnostics(&source), ["nonliteral-attribute"], "{source}");
    }
}

#[test]
fn nested_duplicate_object_keys_are_not_silently_overwritten() {
    let source = "<Poll config={{\"outer\": {\"a\": 1, \"a\": 2}}} />\n";
    let parsed = parse(source).unwrap();
    assert_eq!(parsed.diagnostics[0].code, "nonliteral-attribute");
    assert!(parsed.diagnostics[0].message.contains("duplicate object key"));
}

#[test]
fn imports_exports_and_spreads_are_rejected_without_execution() {
    assert_eq!(diagnostics("import X from 'x'\n\n<Poll />\n"), ["executable-esm"]);
    assert_eq!(diagnostics("export const value = 42\n\n<Poll />\n"), ["executable-esm"]);
    assert_eq!(diagnostics("<Poll {...props} />\n"), ["spread-attribute"]);
    assert_eq!(diagnostics("{run()}\n"), ["executable-expression"]);
    assert_eq!(diagnostics("Text {run()}.\n"), ["executable-expression"]);
}

#[test]
fn duplicate_properties_and_bad_component_keys_diagnose() {
    assert_eq!(diagnostics("<Poll options={[]} options={[]} />\n"), ["duplicate-attribute"]);
    for key in ["123", "true", "\"two words\""] {
        let source = format!("<Poll key={{{key}}} />\n");
        assert_eq!(diagnostics(&source), ["malformed-key"]);
    }
}

#[test]
fn unknown_components_are_preserved_with_a_compilation_diagnostic() {
    let source = "<Mystery data={{\"a\": 1}}>\n  Inner **prose**.\n</Mystery>\n";
    let parsed = parse(source).unwrap();
    assert_eq!(parsed.diagnostics[0].code, "unknown-component");
    let unknown = &parsed.tree.children[0];
    assert_eq!(unknown.kind, Kind::Component);
    assert_eq!(property(unknown, "data"), &json!({"a": 1}));
    assert_eq!(unknown.children[0].children[1].kind, Kind::Strong);
    assert_eq!(parsed.original_source, source);
}

#[test]
fn component_comments_are_container_local_and_do_not_cross_closing_tags() {
    let parsed = good("<Canvas>\n{/* springleaf:key inner */}\nInner.\n</Canvas>\n");
    assert_eq!(parsed.tree.children[0].children[0].key.as_deref(), Some("inner"));
    let parsed = parse("<Canvas>\n{/* springleaf:key lost */}\n</Canvas>\n\nOutside.\n").unwrap();
    assert_eq!(parsed.diagnostics[0].code, "unbound-key");
    assert!(parsed.tree.children[1].key.is_none());
}

#[test]
fn comment_and_component_key_collide_and_document_duplicates_cross_kinds() {
    assert_eq!(
        diagnostics("{/* springleaf:key first */}\n<Poll key=\"second\" />\n"),
        ["key-collision", "key-collision"]
    );
    let parsed = parse("{/* springleaf:key same */}\nProse.\n\n<Poll key=\"same\" />\n").unwrap();
    assert_eq!(
        parsed.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
        ["duplicate-key", "duplicate-key"]
    );
    assert!(parsed.tree.children.iter().all(|n| n.key.is_none()));
}

#[test]
fn component_and_expression_spans_slice_original_crlf_unicode_bytes() {
    let source = "Café 🦀.\r\n\r\n<Poll key=\"topic\" options={[\"é\", \"🦀\"]} />\r\n";
    let parsed = good(source);
    let poll = &parsed.tree.children[1];
    let span = poll.span.unwrap();
    assert_eq!(&source[span.start..span.end], "<Poll key=\"topic\" options={[\"é\", \"🦀\"]} />");
    assert!(poll.key_span.is_none());
    let Details::Component { attributes, .. } = &poll.details else {
        unreachable!()
    };
    let options = attributes.iter().find(|a| a.name == "options").unwrap();
    assert_eq!(options.expression_stops[0], (0, source.find("[\"é\"").unwrap()));
    assert_eq!(parsed.original_source.as_bytes(), source.as_bytes());
}

#[test]
fn jsx_syntax_errors_remain_upstream_errors_not_validator_claims() {
    for source in ["<Poll options={[1, } />\n", "<Canvas><Place /></Poll>\n"] {
        assert!(parse(source).is_err(), "{source}");
    }
}

#[test]
fn ordinary_markdown_text_decodes_escapes_and_entities_but_keeps_source_span() {
    let source = "Escaped \\{#literal} &amp; entity.\n";
    let parsed = good(source);
    let text = &parsed.tree.children[0].children[0];
    assert_eq!(text.text.as_deref(), Some("Escaped {#literal} & entity."));
    let span = text.span.unwrap();
    assert_eq!(&source[span.start..span.end], source.trim_end());
}
