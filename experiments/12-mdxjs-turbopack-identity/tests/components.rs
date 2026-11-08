use markdown::mdast::Node;
use mdxjs_turbopack_identity_experiment::{Component, inspect};
use serde_json::{Value, json};

fn property<'a>(component: &'a Component, name: &str) -> &'a Value {
    component.properties.iter().find(|p| p.name == name).unwrap().value.as_ref().unwrap()
}

fn codes(source: &str) -> Vec<&'static str> {
    inspect(source).unwrap().diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn document_has_real_nested_page_canvas_place_nodes_and_ordered_children() {
    let result = inspect(include_str!("../fixtures/document.mdx")).unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.bindings.len(), 3);
    assert_eq!(
        result.components.iter().map(|c| c.name.as_deref().unwrap()).collect::<Vec<_>>(),
        ["Poll", "Page", "Canvas", "Place", "Image", "Run", "HardBreak"]
    );
    let page = &result.tree.children().unwrap()[5];
    let Node::MdxJsxFlowElement(page) = page else { panic!("{page:?}") };
    assert!(matches!(page.children[0], Node::Paragraph(_)));
    let Node::MdxJsxFlowElement(canvas) = &page.children[1] else {
        panic!("{page:?}")
    };
    assert!(matches!(canvas.children[0], Node::MdxJsxFlowElement(_)));
    assert!(matches!(canvas.children[1], Node::Paragraph(_)));
    assert!(matches!(canvas.children[1].children().unwrap()[1], Node::Strong(_)));
    assert_eq!(property(&result.components[0], "options"), &json!(["Gardens", "Maps"]));
    assert_eq!(property(&result.components[2], "width"), &json!(800));
    assert_eq!(property(&result.components[3], "rotation"), &json!(-2.75));
    assert_eq!(property(&result.components[4], "alt"), &json!("A & B"));
    assert_eq!(property(&result.components[5], "text"), &json!("  literal\ntext <>{}\t🦀"));
}

#[test]
fn literal_types_attributes_order_and_exact_inline_text_survive() {
    let source = "Before <Run text={\"  exact\\ntext <>{}\\t🦀\"} /> then <HardBreak />end.\n\n<Poll disabled config={{\"a\": [null, true, false, -2.75, 1e3, {\"b\": \"x\"}]}} />\n";
    let result = inspect(source).unwrap();
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.original_source.as_bytes(), source.as_bytes());
    assert_eq!(property(&result.components[0], "text"), &json!("  exact\ntext <>{}\t🦀"));
    assert_eq!(property(&result.components[2], "disabled"), &json!(true));
    assert_eq!(
        property(&result.components[2], "config"),
        &json!({"a": [null, true, false, -2.75, 1e3, {"b": "x"}]})
    );
    assert_eq!(
        result.components[2].properties.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["disabled", "config"]
    );
    let inlines = result.tree.children().unwrap()[0].children().unwrap();
    assert_eq!(inlines.len(), 5);
    assert!(matches!(&inlines[2], Node::Text(t) if t.value == " then "));
    assert!(matches!(&inlines[4], Node::Text(t) if t.value == "end."));
}

#[test]
fn key_shaped_literal_strings_are_not_rewritten_or_bound() {
    let result =
        inspect("<Run text={\"{#literal} {/* leafspring:key string */}\"} />\n\n<Poll options={[\"{#id}\", \"{/* leafspring:key id */}\"]} />\n").unwrap();
    assert!(result.diagnostics.is_empty());
    assert!(result.bindings.is_empty());
    assert_eq!(property(&result.components[0], "text"), &json!("{#literal} {/* leafspring:key string */}"));
}

#[test]
fn strict_json_policy_rejects_executable_and_non_json_properties() {
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
        assert_eq!(codes(&source), ["nonliteral-attribute"], "{source}");
    }
}

#[test]
fn recursive_duplicate_json_keys_and_duplicate_properties_diagnose() {
    assert_eq!(codes("<Poll config={{\"outer\": {\"a\": 1, \"a\": 2}}} />\n"), ["nonliteral-attribute"]);
    assert_eq!(codes("<Poll options={[]} options={[]} />\n"), ["duplicate-attribute"]);
    for key in ["123", "true", "\"two words\""] {
        assert_eq!(codes(&format!("<Poll key={{{key}}} />\n")), ["malformed-key"]);
    }
}

#[test]
fn imports_exports_spreads_and_body_expressions_diagnose_without_execution() {
    assert_eq!(codes("import X from 'x'\n\n<Poll />\n"), ["executable-esm"]);
    assert_eq!(codes("export const value = 42\n\n<Poll />\n"), ["executable-esm"]);
    assert_eq!(codes("<Poll {...props} />\n"), ["spread-attribute"]);
    assert_eq!(codes("{run()}\n"), ["executable-expression"]);
    assert_eq!(codes("Text {run()}.\n"), ["executable-expression"]);
}

#[test]
fn unknown_components_preserve_structured_data_but_diagnose() {
    let result = inspect("<Mystery data={{\"a\": 1}}>\n  Inner **prose**.\n</Mystery>\n").unwrap();
    assert_eq!(result.diagnostics[0].code, "unknown-component");
    assert_eq!(result.components[0].name.as_deref(), Some("Mystery"));
    assert_eq!(property(&result.components[0], "data"), &json!({"a": 1}));
    assert!(matches!(
        result.tree.children().unwrap()[0].children().unwrap()[0].children().unwrap()[1],
        Node::Strong(_)
    ));
}

#[test]
fn unicode_crlf_element_spans_and_expression_stops_exist_but_attribute_spans_do_not() {
    let source = "Café 🦀.\r\n\r\n<Poll key=\"topic\" options={[\"é\", \"🦀\"]} />\r\n";
    let result = inspect(source).unwrap();
    assert!(result.diagnostics.is_empty());
    let poll = &result.components[0];
    let span = poll.span.unwrap();
    assert_eq!(&source[span.start..span.end], "<Poll key=\"topic\" options={[\"é\", \"🦀\"]} />");
    assert!(result.bindings[0].annotation_span.is_none());
    assert!(poll.properties.iter().all(|p| p.span.is_none()));
    assert_eq!(poll.properties[1].expression_stops[0], (0, source.find("[\"é\"").unwrap()));
    assert_eq!(result.original_source.as_bytes(), source.as_bytes());
}

#[test]
fn malformed_jsx_is_upstream_failure_not_adapter_acceptance() {
    for source in ["<Poll options={[1, } />\n", "<Canvas><Place /></Poll>\n"] {
        assert!(inspect(source).is_err(), "{source}");
    }
}
