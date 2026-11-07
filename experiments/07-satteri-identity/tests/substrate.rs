//! These controls inspect upstream nodes directly, before our lowering.
use satteri_arena::{Arena, Mdast};
use satteri_ast::mdast::*;
use satteri_identity_experiment::{Mode, native};
use serde_json::{Value, json};

fn metadata(arena: &Arena<Mdast>, id: u32) -> Value {
    arena.get_node_data(id).map_or(json!({}), |b| serde_json::from_slice(b).unwrap())
}

#[test]
fn native_attributes_work_for_headings_and_multiline_paragraphs_without_mdx() {
    let (arena, errors) = native("## Head {#intro}\n\nFirst **bold** line.\nSecond line.\n{#body}\n", Mode::Attributes);
    assert!(errors.is_empty());
    let blocks = arena.get_children(0);
    assert_eq!(blocks.len(), 2);
    assert_eq!(metadata(&arena, blocks[0])["hProperties"]["id"], "intro");
    assert_eq!(metadata(&arena, blocks[1])["hProperties"]["id"], "body");
    assert_eq!(text_content(&arena, blocks[1]), "First bold line.\nSecond line.");
}

#[test]
fn native_mdx_heading_attrs_work_but_requested_paragraph_tail_is_expression() {
    let source = "## Head {#intro}\n\nParagraph.\n{#central-argument}\n";
    let (arena, errors) = satteri_pulldown_cmark::parse(source, satteri_pulldown_cmark::MDX_OPTIONS | satteri_pulldown_cmark::Options::ENABLE_ATTRIBUTES);
    let blocks = arena.get_children(0);
    assert_eq!(blocks.len(), 3);
    assert_eq!(metadata(&arena, blocks[0])["hProperties"]["id"], "intro");
    assert!(metadata(&arena, blocks[1]).get("hProperties").is_none());
    assert_eq!(arena.get_node(blocks[2]).node_type, MdastNodeType::MdxFlowExpression as u8);
    assert!(!errors.is_empty(), "not a lowering error: upstream rejects the private-name expression");
}

#[test]
fn native_attributes_choose_list_item_not_inner_paragraph() {
    let (arena, errors) = native("- First.\n  {#first}\n- Second.\n", Mode::Attributes);
    assert!(errors.is_empty());
    let list = arena.get_children(0)[0];
    let item = arena.get_children(list)[0];
    let paragraph = arena.get_children(item)[0];
    assert_eq!(metadata(&arena, item)["hProperties"]["id"], "first");
    assert!(metadata(&arena, paragraph).get("hProperties").is_none());
}

#[test]
fn native_orphan_is_empty_paragraph_and_duplicate_ids_last_wins() {
    let (arena, errors) = native("{#orphan}\n\n## Head {#first #last}\n", Mode::Attributes);
    assert!(errors.is_empty());
    let blocks = arena.get_children(0);
    assert!(arena.get_children(blocks[0]).is_empty());
    assert_eq!(metadata(&arena, blocks[0])["hProperties"]["id"], "orphan");
    assert_eq!(metadata(&arena, blocks[1])["hProperties"]["id"], "last");
    // Source survives but semantic metadata no longer exposes both ID tokens.
    assert_eq!(arena.source(), "{#orphan}\n\n## Head {#first #last}\n");
}

#[test]
fn native_mdx_comment_is_typed_expression_not_prose() {
    let (arena, errors) = native("{/* springleaf:key intro */}\n## Heading\n", Mode::Mdx);
    assert!(errors.is_empty());
    let blocks = arena.get_children(0);
    assert_eq!(arena.get_node(blocks[0]).node_type, MdastNodeType::MdxFlowExpression as u8);
    let value = decode_expression_data(arena.get_type_data(blocks[0])).value;
    assert_eq!(arena.get_str(value), "/* springleaf:key intro */");
    assert_eq!(arena.get_node(blocks[1]).node_type, MdastNodeType::Heading as u8);
}

#[test]
fn native_mdx_component_has_name_typed_attrs_and_json_expression_source() {
    let source = "<Poll key=\"topic\" options={[\"Gardens\", \"Maps\"]} />\n";
    let (arena, errors) = native(source, Mode::Mdx);
    assert!(errors.is_empty());
    let id = arena.get_children(0)[0];
    assert_eq!(arena.get_node(id).node_type, MdastNodeType::MdxJsxFlowElement as u8);
    let bytes = arena.get_type_data(id);
    assert_eq!(arena.get_str(decode_mdx_jsx_element_name(bytes)), "Poll");
    assert_eq!(decode_mdx_jsx_attr_count(bytes), 2);
    let (kind, name, value) = decode_mdx_jsx_attr(bytes, 1);
    assert_eq!(kind, MDX_ATTR_EXPRESSION_PROP);
    assert_eq!(arena.get_str(name), "options");
    assert_eq!(arena.get_str(value), "[\"Gardens\", \"Maps\"]");
}

#[test]
fn native_mdx_is_not_literal_only_and_carries_arbitrary_javascript() {
    let (arena, errors) = native("<Poll options={danger()} {...props} />\n", Mode::Mdx);
    assert!(errors.is_empty());
    let bytes = arena.get_type_data(arena.get_children(0)[0]);
    let (_, _, value) = decode_mdx_jsx_attr(bytes, 0);
    assert_eq!(arena.get_str(value), "danger()");
    assert_eq!(decode_mdx_jsx_attr(bytes, 1).0, MDX_ATTR_SPREAD);
}

#[test]
fn native_mdx_intentionally_disables_indented_code() {
    let source = "    {/* springleaf:key looks-like-code */}\n\nFollowing prose.\n";
    let (plain, _) = native(source, Mode::Plain);
    let (mdx, errors) = native(source, Mode::Mdx);
    assert!(errors.is_empty());
    assert_eq!(plain.get_node(plain.get_children(0)[0]).node_type, MdastNodeType::Code as u8);
    assert_eq!(mdx.get_node(mdx.get_children(0)[0]).node_type, MdastNodeType::MdxFlowExpression as u8);
}
