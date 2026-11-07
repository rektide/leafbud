//! Native attribute lowering is a separate Markdown-only control, not author mode.
use satteri_identity_experiment::{Kind, Mode, parse};

#[test]
fn native_ids_become_internal_keys_not_html_anchors() {
    let p = parse("## Head {#intro}\n\nParagraph.\n{#body}\n", Mode::Attributes);
    assert!(p.diagnostics.is_empty());
    assert_eq!(p.tree.children[0].key.as_deref(), Some("intro"));
    assert_eq!(p.tree.children[1].key.as_deref(), Some("body"));
    assert!(p.tree.children[0].data["hProperties"].get("id").is_none());
}

#[test]
fn orphan_invalid_inline_and_duplicate_native_ids_diagnose() {
    for (source, code) in [
        ("{#orphan}\n", "unbound-key"),
        ("## Head {#123}\n", "invalid-key"),
        ("*em*{#inline}\n", "non-block-key"),
        ("## One {#same}\n\nTwo.\n{#same}\n", "duplicate-key"),
    ] {
        let p = parse(source, Mode::Attributes);
        assert!(p.diagnostics.iter().any(|d| d.code == code), "{source:?}: {:?}", p.diagnostics);
    }
}

#[test]
fn blank_before_attribute_creates_orphan_not_backwards_search() {
    let p = parse("Prose.\n\n{#body}\n", Mode::Attributes);
    assert_eq!(p.tree.children[0].key, None);
    assert!(p.diagnostics.iter().any(|d| d.code == "unbound-key"));
}

#[test]
fn native_same_line_paragraph_suffix_is_supported_unlike_rushdown_probe() {
    let p = parse("Paragraph {#body}\n", Mode::Attributes);
    assert!(p.diagnostics.is_empty());
    assert_eq!(p.tree.children[0].kind, Kind::Paragraph);
    assert_eq!(p.tree.children[0].key.as_deref(), Some("body"));
}

#[test]
fn known_gap_native_attribute_metadata_loses_same_node_id_collisions() {
    let p = parse("## Head {#first #last}\n", Mode::Attributes);
    assert!(p.diagnostics.is_empty());
    assert_eq!(p.tree.children[0].key.as_deref(), Some("last"));
    // This mode is a native-facility control, not a completed identity dialect.
}

#[test]
fn known_gap_malformed_attribute_is_plain_text_in_markdown_mode() {
    let p = parse("## Head {#unterminated\n", Mode::Attributes);
    assert!(p.diagnostics.is_empty());
    assert_eq!(p.tree.children[0].key, None);
    assert!(p.tree.children[0].children[0].value.as_deref().unwrap().contains("{#unterminated"));
}
