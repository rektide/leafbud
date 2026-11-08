use satteri_arena::{Arena, Mdast};
use satteri_ast::mdast::*;
use satteri_pulldown_cmark::{MDX_OPTIONS, Options};
use serde_json::Value;

fn ids(arena: &Arena<Mdast>, id: u32) -> Vec<u32> {
    let mut all = vec![id];
    for &child in arena.get_children(id) {
        all.extend(ids(arena, child));
    }
    all
}
fn kind(arena: &Arena<Mdast>, id: u32) -> MdastNodeType {
    MdastNodeType::from_u8(arena.get_node(id).node_type).unwrap()
}

#[test]
fn stock_default_mdx_reports_braces_as_expressions_not_keys_but_retains_their_nodes() {
    let (arena, errors) = satteri_pulldown_cmark::parse(
        "## Introduction {#introduction}\n\nOrdinary **formatted** prose.\n{#central-argument}\n",
        MDX_OPTIONS,
    );
    assert!(!errors.is_empty());
    let expressions: Vec<_> = ids(&arena, 0)
        .into_iter()
        .filter(|&id| matches!(kind(&arena, id), MdastNodeType::MdxTextExpression | MdastNodeType::MdxFlowExpression))
        .collect();
    assert_eq!(expressions.len(), 2);
    assert_eq!(
        arena.get_str(decode_expression_data(arena.get_type_data(expressions[0])).value),
        "#introduction"
    );
    assert_eq!(
        arena.get_str(decode_expression_data(arena.get_type_data(expressions[1])).value),
        "#central-argument"
    );
}

#[test]
fn stock_heading_flags_work_with_mdx_but_general_attributes_do_not_solve_paragraph_tails() {
    for flag in [Options::ENABLE_HEADING_ATTRIBUTES, Options::ENABLE_ATTRIBUTES] {
        let (arena, errors) = satteri_pulldown_cmark::parse("## **Introduction** [link](https://example.test) {#introduction}\n", MDX_OPTIONS | flag);
        assert!(errors.is_empty(), "{errors:?}");
        let heading = arena.get_children(0)[0];
        let data: Value = serde_json::from_slice(arena.get_node_data(heading).unwrap()).unwrap();
        assert_eq!(data["hProperties"]["id"], "introduction");
        assert!(!ids(&arena, 0).iter().any(|&id| kind(&arena, id) == MdastNodeType::MdxTextExpression));
        let (arena, errors) = satteri_pulldown_cmark::parse("Ordinary **formatted** prose.\n{#central-argument}\n", MDX_OPTIONS | flag);
        assert!(!errors.is_empty());
        assert!(ids(&arena, 0).iter().any(|&id| kind(&arena, id) == MdastNodeType::MdxFlowExpression));
    }
}

#[test]
fn markdown_only_attributes_have_different_list_identity_semantics() {
    let (arena, errors) = satteri_pulldown_cmark::parse("- Item.\n  {#item}\n", Options::ENABLE_ATTRIBUTES);
    assert!(errors.is_empty());
    let keyed: Vec<_> = ids(&arena, 0)
        .into_iter()
        .filter(|&id| {
            arena
                .get_node_data(id)
                .is_some_and(|data| serde_json::from_slice::<Value>(data).unwrap()["hProperties"]["id"] == "item")
        })
        .collect();
    assert_eq!(keyed.len(), 1);
    assert_eq!(
        kind(&arena, keyed[0]),
        MdastNodeType::ListItem,
        "native attributes name the item, not the paragraph required by our Key contract"
    );
}

#[test]
fn routine_code_and_component_prop_literals_are_not_native_key_expressions() {
    let source = "Inline `{#inline}`.\n\n```mdx\n{#fenced}\n```\n\n<Run text={\"{#string}\"} />\n\n<Poll key=\"p\" options={[\"{#option}\",\"other\"]} />\n";
    let (arena, errors) = satteri_pulldown_cmark::parse(source, MDX_OPTIONS);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        !ids(&arena, 0)
            .iter()
            .any(|&id| matches!(kind(&arena, id), MdastNodeType::MdxTextExpression | MdastNodeType::MdxFlowExpression))
    );
}
