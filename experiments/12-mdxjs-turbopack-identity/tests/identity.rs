use markdown::mdast::Node;
use mdxjs_turbopack_identity_experiment::{Inspection, inspect};

fn good(source: &str) -> Inspection<'_> {
    let result = inspect(source).unwrap();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    result
}

fn codes(source: &str) -> Vec<&'static str> {
    inspect(source).unwrap().diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn comments_bind_normal_heading_and_paragraph_across_blank_lines() {
    let result = good("{/* leafspring:key introduction */}\n\n## Introduction\n\n{/* leafspring:key central-argument */}\nThis is **formatted** prose.\n");
    assert_eq!(
        result.bindings.iter().map(|b| b.key.as_str()).collect::<Vec<_>>(),
        ["introduction", "central-argument"]
    );
    let children = result.tree.children().unwrap();
    assert!(matches!(children[1], Node::Heading(_)));
    assert!(matches!(children[3].children().unwrap()[1], Node::Strong(_)));
    assert_eq!(result.bindings[0].path, [1]);
    assert_eq!(result.bindings[1].path, [3]);
}

#[test]
fn guard_rejects_root_quote_and_list_code_traps_without_assigning_keys() {
    for source in [
        include_str!("../fixtures/multiline-comment.mdx"),
        "> Code `first\n> {/* leafspring:key accidental */}\n> last`.\n",
        "- Code `first\n  {/* leafspring:key accidental */}\n  last`.\n",
    ] {
        let result = inspect(source).unwrap();
        assert_eq!(result.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(), ["misplaced-key"], "{source}");
        assert!(result.bindings.is_empty());
    }
}

#[test]
fn real_inline_multiline_fenced_and_escaped_markers_remain_literal() {
    let source = "Inline `{/* leafspring:key inline */}` and `{#hash}`.\n\nCode `first\ncontinued {/* leafspring:key multiline */}\nlast`.\n\n```mdx\n{/* leafspring:key fenced */}\n<Poll key=\"literal\" />\n```\n\n\\{/* leafspring:key escaped */}\n\nEscaped \\{#also-literal}.\n";
    let result = good(source);
    assert!(result.bindings.is_empty());
    assert!(result.components.is_empty());
    assert_eq!(result.original_source, source);
}

#[test]
fn annotations_cannot_escape_closed_containers() {
    for source in [
        "> {/* leafspring:key lost */}\n\nOutside.\n",
        "- {/* leafspring:key lost */}\n- Other item.\n",
        "<Canvas>\n{/* leafspring:key lost */}\n</Canvas>\n\nOutside.\n",
    ] {
        let result = inspect(source).unwrap();
        assert_eq!(result.diagnostics[0].code, "unbound-key", "{source}");
        assert!(result.bindings.is_empty());
    }
}

#[test]
fn quote_item_and_component_children_are_valid_local_targets() {
    let result = good(
        "> {/* leafspring:key quote-child */}\n> ## Heading\n\n- {/* leafspring:key item-child */}\n  Item.\n\n<Canvas>\n{/* leafspring:key canvas-child */}\nInside.\n</Canvas>\n",
    );
    assert_eq!(
        result.bindings.iter().map(|b| b.key.as_str()).collect::<Vec<_>>(),
        ["quote-child", "item-child", "canvas-child"]
    );
    assert!(result.bindings.iter().all(|b| b.path.len() > 1));
}

#[test]
fn malformed_unbound_and_inline_annotations_diagnose() {
    for source in [
        "{/* leafspring:key two words */}\nText.\n",
        "{/* leafspring:other id */}\nText.\n",
        "{/* leafspring:key café */}\nText.\n",
        "{/* leafspring:key name\n*/}\nText.\n",
    ] {
        assert_eq!(codes(source), ["malformed-key"], "{source}");
    }
    assert_eq!(codes("{/* leafspring:key orphan */}\n"), ["unbound-key"]);
    assert_eq!(codes("Text {/* leafspring:key inline */} more.\n"), ["misplaced-key"]);
    assert!(inspect("{/* leafspring:key unterminated\nText.\n").is_err());
}

#[test]
fn directives_do_not_skip_barriers_or_each_other() {
    assert_eq!(codes("{/* leafspring:key first */}\n\n{/* leafspring:key second */}\nText.\n"), ["unbound-key"]);
    assert_eq!(
        codes("{/* leafspring:key first */}\n{/* leafspring:key second */}\nText.\n"),
        ["unbound-key", "misplaced-key"]
    );
    assert_eq!(codes("{/* leafspring:key blocked */}\n[x]: https://example.com\n\nText.\n"), ["unbound-key"]);
    assert_eq!(codes("{/* leafspring:key blocked */}\n{/* ordinary comment */}\nText.\n"), ["unbound-key"]);
}

#[test]
fn duplicates_and_comment_component_collisions_remove_bindings() {
    for source in [
        "{/* leafspring:key same */}\nOne.\n\n{/* leafspring:key same */}\n## Two\n",
        "{/* leafspring:key same */}\nProse.\n\n<Poll key=\"same\" />\n",
    ] {
        let result = inspect(source).unwrap();
        assert_eq!(
            result.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
            ["duplicate-key", "duplicate-key"]
        );
        assert!(result.bindings.is_empty());
    }
    let result = inspect("{/* leafspring:key first */}\n<Poll key=\"second\" />\n").unwrap();
    assert_eq!(
        result.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
        ["key-collision", "key-collision"]
    );
    assert!(result.bindings.is_empty());
}

#[test]
fn hash_examples_rejected_by_policy_while_comment_key_punctuation_is_independent() {
    let result = inspect(include_str!("../fixtures/attributes.mdx")).unwrap();
    assert_eq!(
        result.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
        ["executable-expression", "executable-expression"]
    );
    assert!(result.bindings.is_empty());
    let result = good("{/* leafspring:key trailing. */}\nText.\n\n{/* leafspring:key two..dots */}\n## Heading\n");
    assert_eq!(result.bindings.len(), 2);
}

#[test]
fn crlf_unicode_comment_and_node_offsets_slice_original_bytes() {
    let source = "Préface 🦀.\r\n\r\n> {/* leafspring:key intro */}\r\n> ## Naïve 🦀\r\n";
    let result = good(source);
    assert_eq!(result.original_source.as_bytes(), source.as_bytes());
    let annotation = result.bindings[0].annotation_span.unwrap();
    assert_eq!(&source[annotation.start..annotation.end], "{/* leafspring:key intro */}");
    let heading = &result.tree.children().unwrap()[1].children().unwrap()[1];
    let position = heading.position().unwrap();
    assert_eq!(&source[position.start.offset..position.end.offset], "## Naïve 🦀");
}

#[test]
fn reordered_blocks_keep_keys_not_source_offsets() {
    let a = good("{/* leafspring:key one */}\nOne.\n\n{/* leafspring:key two */}\n## Two\n");
    let b = good("{/* leafspring:key two */}\n## Two\n\n{/* leafspring:key one */}\nOne.\n");
    assert_eq!(a.bindings[0].key, b.bindings[1].key);
    assert_ne!(a.bindings[0].annotation_span, b.bindings[1].annotation_span);
}

#[test]
fn negative_observation_indented_comment_is_cleanly_misbound_under_commonmark_expectation() {
    let result = good("    {/* leafspring:key accidental */}\n\nAfter.\n");
    assert_eq!(result.bindings[0].key, "accidental");
    assert_eq!(result.bindings[0].path, [1]);
    // This green assertion records an unwanted interpretation, not an identity guarantee.
}

#[test]
fn negative_observation_component_key_bypasses_comment_guard_in_multiline_code() {
    let result = good(include_str!("../fixtures/multiline-component.mdx"));
    assert_eq!(result.bindings[0].key, "accidental");
    assert_eq!(result.components.len(), 1);
    assert_eq!(result.bindings[0].path, [1]);
    let commonmark = markdown::to_mdast(include_str!("../fixtures/multiline-component.mdx"), &Default::default()).unwrap();
    assert_eq!(commonmark.children().unwrap().len(), 1);
    assert!(
        commonmark.children().unwrap()[0]
            .children()
            .unwrap()
            .iter()
            .any(|n| matches!(n, Node::InlineCode(_)))
    );
}
