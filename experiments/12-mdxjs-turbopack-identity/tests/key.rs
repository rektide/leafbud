//! User-steered native Key candidate. This is not a claim of syntax adoption.
use markdown::mdast::Node;
use mdxjs_turbopack_identity_experiment::{Inspection, inspect, stock};

fn good(source: &str) -> Inspection<'_> {
    let result = inspect(source).unwrap();
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    result
}

fn codes(source: &str) -> Vec<&'static str> {
    inspect(source).unwrap().diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn native_self_closing_marker_and_markdown_are_separate_flow_siblings() {
    for separator in ["\n", "\n\n", "\n\n\n"] {
        let source = format!("<Key id=\"intro\" />{separator}## Introduction\n");
        let tree = stock(&source).unwrap();
        let children = tree.children().unwrap();
        assert_eq!(children.len(), 2);
        assert!(matches!(&children[0], Node::MdxJsxFlowElement(e) if e.name.as_deref() == Some("Key") && e.children.is_empty()));
        assert!(matches!(children[1], Node::Heading(_)));
        let result = good(&source);
        assert_eq!(result.bindings[0].key, "intro");
        assert_eq!(result.bindings[0].path, [1]);
        assert_eq!(result.consumed_annotations, [vec![0]]);
        assert!(result.components.is_empty(), "reserved marker is not a runtime component");
    }
}

#[test]
fn candidate_fixture_keeps_prose_unwrapped_and_targets_local_siblings() {
    let result = good(include_str!("../fixtures/key-candidate.mdx"));
    assert_eq!(
        result.bindings.iter().map(|b| b.key.as_str()).collect::<Vec<_>>(),
        ["introduction", "central-argument", "poll", "quoted", "item"]
    );
    assert_eq!(result.consumed_annotations.len(), 5);
    assert_eq!(result.components.len(), 1);
    assert_eq!(result.components[0].name.as_deref(), Some("Poll"));
    assert!(matches!(result.tree.children().unwrap()[3], Node::Paragraph(_)));
    assert!(matches!(result.tree.children().unwrap()[3].children().unwrap()[1], Node::Strong(_)));
}

#[test]
fn key_to_prose_needs_no_blank_line_but_preceding_content_has_boundary_guard() {
    for source in ["<Key id=\"body\" />\nOrdinary prose.\n", "Before.\n\n<Key id=\"body\" />\nOrdinary prose.\n"] {
        assert_eq!(good(source).bindings[0].key, "body");
    }
    assert_eq!(codes("Before.\n<Key id=\"body\" />\nOrdinary prose.\n"), ["misplaced-key"]);
}

#[test]
fn multiline_key_marker_is_native_jsx_not_a_line_regex() {
    let result = good("<Key\n  id=\"body\"\n/>\nOrdinary prose.\n");
    assert_eq!(result.bindings[0].key, "body");
    assert_eq!(result.consumed_annotations, [vec![0]]);
}

#[test]
fn unclosed_jsx_is_not_a_standalone_key_marker() {
    for source in ["<Key id=\"intro\">\n## Introduction\n", "<Key id=\"intro\">\n"] {
        assert!(stock(source).is_err(), "{source}");
        assert!(inspect(source).is_err());
    }
}

#[test]
fn paired_key_has_native_markdown_children_but_group_semantics_are_not_invented() {
    let source = include_str!("../fixtures/key-paired.mdx");
    let tree = stock(source).unwrap();
    let Node::MdxJsxFlowElement(key) = &tree.children().unwrap()[0] else {
        panic!("{tree:?}")
    };
    assert_eq!(key.children.len(), 2);
    assert!(matches!(key.children[0], Node::Heading(_)));
    assert!(matches!(key.children[1], Node::Paragraph(_)));
    assert_eq!(codes(source), ["paired-key-unsupported"]);
    assert!(inspect(source).unwrap().bindings.is_empty());
    assert_eq!(codes("<Key id=\"empty\"></Key>\n\nNext.\n"), ["paired-key-unsupported"]);
}

#[test]
fn malformed_id_requires_exactly_one_quoted_literal_property() {
    for marker in [
        "<Key />",
        "<Key id />",
        "<Key name=\"intro\" />",
        "<Key id=\"two words\" />",
        "<Key id=\"café\" />",
        "<Key id={\"intro\"} />",
        "<Key id={load()} />",
        "<Key {...props} />",
        "<Key id=\"a\" id=\"b\" />",
        "<Key id=\"a\" extra />",
    ] {
        assert_eq!(codes(&format!("{marker}\nText.\n")), ["malformed-key-component"], "{marker}");
    }
    assert_eq!(good("<Key id='trailing.' />\nText.\n").bindings[0].key, "trailing.");
}

#[test]
fn orphan_and_container_closure_never_key_outside_content() {
    for source in [
        "<Key id=\"lost\" />\n",
        "> <Key id=\"lost\" />\n\nOutside.\n",
        "- <Key id=\"lost\" />\n- Other.\n",
        "<Canvas>\n<Key id=\"lost\" />\n</Canvas>\n\nOutside.\n",
    ] {
        let result = inspect(source).unwrap();
        assert_eq!(result.diagnostics[0].code, "unbound-key", "{source}");
        assert!(result.bindings.is_empty());
    }
}

#[test]
fn nested_quotes_lists_and_component_children_are_valid_local_scopes() {
    let result = good(
        "> > <Key id=\"deep\" />\n> > Nested quote.\n\n- Outer.\n\n  - <Key id=\"item\" />\n    Nested item.\n\n<Canvas>\n<Key id=\"canvas-body\" />\nText.\n</Canvas>\n",
    );
    assert_eq!(
        result.bindings.iter().map(|b| b.key.as_str()).collect::<Vec<_>>(),
        ["deep", "item", "canvas-body"]
    );
    assert!(result.bindings.iter().all(|b| b.path.len() >= 2));
}

#[test]
fn duplicate_ids_and_cross_syntax_collisions_are_invalid() {
    for source in [
        "<Key id=\"same\" />\nOne.\n\n<Key id=\"same\" />\n## Two\n",
        "<Key id=\"same\" />\nOne.\n\n<Poll key=\"same\" />\n",
        "<Key id=\"same\" />\nOne.\n\n{/* leafspring:key same */}\nTwo.\n",
    ] {
        let result = inspect(source).unwrap();
        assert_eq!(
            result.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
            ["duplicate-key", "duplicate-key"]
        );
        assert!(result.bindings.is_empty());
    }
    assert_eq!(codes("<Key id=\"first\" />\n<Poll key=\"second\" />\n"), ["key-collision", "key-collision"]);
}

#[test]
fn markers_never_skip_each_other_comments_or_definitions() {
    assert_eq!(codes("<Key id=\"a\" />\n\n<Key id=\"b\" />\nText.\n"), ["unbound-key"]);
    assert_eq!(codes("<Key id=\"a\" />\n<Key id=\"b\" />\nText.\n"), ["unbound-key", "misplaced-key"]);
    assert_eq!(codes("<Key id=\"a\" />\n{/* comment */}\nText.\n"), ["unbound-key"]);
    assert_eq!(codes("<Key id=\"a\" />\n[x]: https://example.com\n\nText.\n"), ["unbound-key"]);
}

#[test]
fn adjacent_other_jsx_is_the_target_not_something_to_skip() {
    let result = good("<Key id=\"poll\" />\n<Poll options={[]} />\n\nAfter.\n");
    assert_eq!(result.bindings[0].path, [1]);
    assert_eq!(result.components[0].name.as_deref(), Some("Poll"));
    let result = inspect("<Key id=\"unknown\" />\n<Mystery />\n\nAfter.\n").unwrap();
    assert_eq!(result.diagnostics[0].code, "unknown-component");
    assert_eq!(result.bindings[0].path, [1]);
}

#[test]
fn inline_marker_is_not_zero_width_metadata() {
    assert_eq!(codes("Text <Key id=\"inline\" /> more.\n"), ["misplaced-key"]);
    assert_eq!(codes("<Key id=\"inline\" /> Ordinary text.\n"), ["misplaced-key"]);
}

#[test]
fn inline_fenced_escaped_and_literal_prop_key_text_remain_literal() {
    let source =
        "Inline `<Key id=\"inline\" />`.\n\n```mdx\n<Key id=\"fenced\" />\n```\n\n\\<Key id=\"escaped\" />\n\n<Run text={\"<Key id=\\\"literal\\\" />\"} />\n";
    let result = good(source);
    assert!(result.bindings.is_empty());
    assert!(result.consumed_annotations.is_empty());
    assert_eq!(result.components.len(), 1);
    assert_eq!(result.original_source, source);
}

#[test]
fn multiline_code_key_traps_are_guard_rejections_not_native_code_preservation() {
    for source in [
        "Code `first\n<Key id=\"accidental\" />\nlast`.\n",
        "> Code `first\n> <Key id=\"accidental\" />\n> last`.\n",
        "- Code `first\n  <Key id=\"accidental\" />\n  last`.\n",
    ] {
        let tree = stock(source).unwrap();
        assert!(format!("{tree:?}").contains("name: Some(\"Key\")"));
        let result = inspect(source).unwrap();
        assert_eq!(result.diagnostics[0].code, "misplaced-key");
        assert!(result.bindings.is_empty());
    }
}

#[test]
fn negative_observation_indented_key_is_active_mdx_not_commonmark_code() {
    let result = good("    <Key id=\"accidental\" />\n\nAfter.\n");
    assert_eq!(result.bindings[0].key, "accidental");
    assert_eq!(result.bindings[0].path, [1]);
}

#[test]
fn crlf_unicode_noop_and_complete_marker_span_are_preserved() {
    let source = "Café 🦀.\r\n\r\n> <Key id=\"intro\" />\r\n> ## Naïve 🦀\r\n";
    let result = good(source);
    assert_eq!(result.original_source.as_bytes(), source.as_bytes());
    let span = result.bindings[0].annotation_span.unwrap();
    assert_eq!(&source[span.start..span.end], "<Key id=\"intro\" />");
    assert_eq!(result.bindings[0].path, [1, 1]);
}
