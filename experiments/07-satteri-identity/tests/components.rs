use satteri_identity_experiment::{Kind, Mode, Node, parse};
use serde_json::json;

fn nodes(node: &Node) -> Vec<&Node> {
    let mut result = vec![node];
    for child in &node.children {
        result.extend(nodes(child));
    }
    result
}

#[test]
fn json_array_object_number_string_and_boolean_props_are_semantic_data() {
    let p = parse(include_str!("../fixtures/components.mdx"), Mode::Author);
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let poll = &p.tree.children[0];
    assert_eq!(poll.kind, Kind::Component);
    assert_eq!(poll.key.as_deref(), Some("next-topic"));
    assert_eq!(poll.details["literal_props"]["options"], json!(["Gardens", "Maps"]));
    let image = &p.tree.children[1];
    assert_eq!(image.details["literal_props"]["width"], 640);
    assert_eq!(image.details["literal_props"]["metadata"], json!({"alt":"Diagram", "decorative":false}));
    assert!(nodes(&p.tree).iter().any(|n| n.kind == Kind::Component && n.details["flow"] == false));
}

#[test]
fn unknown_components_are_rejectable_by_ast_kind_not_prose_heuristics() {
    for source in [
        "<Mystery options={[\"A\"]} />\n",
        "Text <Mystery /> inline.\n",
        "<div>\nProse.\n</div>\n",
        "<>\nProse.\n</>\n",
    ] {
        let p = parse(source, Mode::Author);
        assert!(p.diagnostics.iter().any(|d| d.code == "unknown-component"), "{source:?}: {:?}", p.diagnostics);
        assert!(nodes(&p.tree).iter().any(|n| n.kind == Kind::Component));
    }
}

#[test]
fn arbitrary_js_and_js_but_not_json_literals_are_rejected_without_execution() {
    for expression in [
        "doSomething()",
        "() => 42",
        "{options}",
        "['A', 'B']",
        "[1, 2,]",
        "/* note */ 42",
        "undefined",
        "NaN",
        "1 + 2",
    ] {
        let source = format!("<Poll options={{{expression}}} />\n");
        let p = parse(&source, Mode::Author);
        assert!(p.diagnostics.iter().any(|d| d.code == "nonliteral-prop"), "{source:?}: {:?}", p.diagnostics);
    }
}

#[test]
fn spreads_duplicate_properties_standalone_expressions_and_esm_are_rejected() {
    for (source, code) in [
        ("<Poll {...props} />\n", "spread-prop"),
        ("<Poll key=\"first\" key=\"last\" />\n", "duplicate-prop"),
        ("{doSomething()}\n", "unsupported-expression"),
        ("export const value = 1\n", "unsupported-expression"),
        ("Text {1 + 2}.\n", "unsupported-expression"),
    ] {
        let p = parse(source, Mode::Author);
        assert!(p.diagnostics.iter().any(|d| d.code == code), "{source:?}: {:?}", p.diagnostics);
    }
}

#[test]
fn annotation_can_key_component_but_collision_is_not_silently_overwritten() {
    let p = parse("{/* leafspring:key poll */}\n<Poll options={[\"A\", \"B\"]} />\n", Mode::Author);
    assert!(p.diagnostics.is_empty());
    assert_eq!(p.tree.children[0].kind, Kind::Component);
    assert_eq!(p.tree.children[0].key.as_deref(), Some("poll"));
    let p = parse("{/* leafspring:key first */}\n<Poll key=\"second\" />\n", Mode::Author);
    assert!(p.diagnostics.iter().any(|d| d.code == "key-collision"));
    assert_eq!(p.tree.children[0].key, None);
}

#[test]
fn prose_component_keys_share_namespace() {
    let p = parse("{/* leafspring:key same */}\n## Head\n\n<Poll key=\"same\" />\n", Mode::Author);
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "duplicate-key").count(), 2);
    assert!(nodes(&p.tree).iter().all(|n| n.key.is_none()));
}

#[test]
fn multiline_props_and_markdown_children_survive_native_mdx_structure() {
    let p = parse(
        "<Poll\n key=\"topic\"\n options={[\"A\",\n \"B\"]}\n>\n\n## Markdown inside\n\nChild **bold** prose.\n\n</Poll>\n",
        Mode::Author,
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(p.tree.children[0].kind, Kind::Component);
    assert!(nodes(&p.tree.children[0]).iter().any(|n| n.kind == Kind::Heading));
    assert!(nodes(&p.tree.children[0]).iter().any(|n| n.kind == Kind::Strong));
}

#[test]
fn malformed_jsx_is_upstream_error_not_silently_accepted_prose() {
    let p = parse("<Poll options={[1,} />\n", Mode::Author);
    assert!(p.diagnostics.iter().any(|d| d.code == "mdx-parse"));
}
