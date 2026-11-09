//! Native controls: nothing in this file relies on our author-key binder.
use satteri_arena::{
    Arena, Hast,
    mdx_types::{Location, Place},
};
use satteri_ast::mdast::MdastNodeType as M;
use satteri_mdxjs_identity_experiment::{kind, lower, metadata, reachable, staged, stock};
use satteri_pulldown_cmark::{MDX_OPTIONS, Options};

fn convert(arena: &Arena<satteri_arena::Mdast>) -> Arena<Hast> {
    let mut hast = satteri_ast::hast::mdast_arena_to_hast_arena_with_options(arena, &Default::default());
    hast.mdx = true;
    hast
}
fn compile(hast: &Arena<Hast>) -> String {
    satteri_mdxjs::compile_hast_arena(
        hast,
        &satteri_mdxjs::Options {
            jsx: true,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn actual_compiler_and_public_arena_route_agree_on_shared_corpus() {
    for source in [
        include_str!("../fixtures/comments.mdx"),
        include_str!("../fixtures/components.mdx"),
        "## Heading\n\nA **bold** paragraph.\n\n- First\n- Second\n\n> Quote\n\n```rs\nlet x = 1;\n```\n",
        include_str!("../fixtures/multiline-code.mdx"),
    ] {
        let (arena, errors) = satteri_pulldown_cmark::parse(source, MDX_OPTIONS);
        assert!(errors.is_empty());
        assert_eq!(stock(source).unwrap(), compile(&convert(&arena)), "{source:?}");
    }
}

#[test]
fn selected_authoring_gate_can_feed_actual_compiler_without_evaluation() {
    let js = staged(include_str!("../fixtures/components.mdx")).unwrap();
    for name in ["Poll", "Page", "Canvas", "Place", "Image", "Run"] {
        assert!(js.contains(name), "{name}");
    }
    let js = staged(include_str!("../fixtures/comments.mdx")).unwrap();
    assert!(!js.contains("leafbud"));
    assert!(!js.contains("central-argument"));
    assert!(js.contains("Introduction"));
    assert!(js.contains("strong"));
}

#[test]
fn plain_comment_keys_are_not_compiler_metadata_unless_we_lower_them() {
    let source = "{/* leafbud:key intro */}\n## Introduction\n";
    let stock_js = stock(source).unwrap();
    let staged_js = staged(source).unwrap();
    // Initial assumption that compiler output preserves the comment FAILED.
    // Stock codegen discards the text and leaves an empty JSX expression.
    assert!(!stock_js.contains("leafbud:key intro"));
    assert!(stock_js.contains("<>{}"));
    assert!(!staged_js.contains("<>{}"));
    assert!(!staged_js.contains("leafbud:key intro"));
}

#[test]
fn compiler_is_not_a_literal_gate() {
    let source = "import {danger} from 'not-installed'\n\n<Poll options={danger()} {...props} />\n";
    let js = stock(source).unwrap();
    assert!(js.contains("not-installed"));
    assert!(js.contains("danger()"));
    assert!(js.contains("...props"));
    assert!(staged(source).is_err());
    // Import resolution/execution never occurs: the nonexistent module is just emitted.
}

#[test]
fn hash_syntax_is_not_the_direct_mdxjs_private_name_accident() {
    let source = include_str!("../fixtures/attributes.mdx");
    let (arena, errors) = satteri_pulldown_cmark::parse(source, MDX_OPTIONS | Options::ENABLE_ATTRIBUTES);
    assert_eq!(metadata(&arena, arena.get_children(0)[0])["hProperties"]["id"], "introduction");
    assert!(!errors.is_empty());
    assert!(reachable(&arena, 0).iter().any(|&id| kind(&arena, id) == M::MdxFlowExpression));
    assert!(satteri_mdxjs::compile(source, &Default::default(), MDX_OPTIONS | Options::ENABLE_ATTRIBUTES).is_err());
}

#[test]
fn multiline_code_loss_is_identical_to_07_and_not_fixed_by_compiler() {
    let source = include_str!("../fixtures/multiline-code.mdx");
    let (plain, _) = satteri_pulldown_cmark::parse(source, Options::empty());
    let (mdx, errors) = satteri_pulldown_cmark::parse(source, MDX_OPTIONS);
    assert!(errors.is_empty());
    assert_eq!(reachable(&plain, 0).iter().filter(|&&id| kind(&plain, id) == M::InlineCode).count(), 2);
    assert!(!reachable(&mdx, 0).iter().any(|&id| kind(&mdx, id) == M::InlineCode));
    let js = stock(source).unwrap();
    assert!(!js.contains("leafbud:key accidental"));
    assert!(js.contains("Code `first"));
    assert!(js.contains("last`."));
    assert!(!js.contains("code: \"code\""));
    assert!(staged(source).is_err());
}

#[test]
fn separate_parse_options_argument_controls_grammar_not_options_parse() {
    let source = "## Heading\n\n**Bold**\n";
    let options = satteri_mdxjs::Options {
        jsx: true,
        ..Default::default()
    };
    let mut changed = options.clone();
    changed.parse.constructs.heading_atx = false;
    changed.parse.constructs.attention = false;
    assert_eq!(
        satteri_mdxjs::compile(source, &options, MDX_OPTIONS).unwrap(),
        satteri_mdxjs::compile(source, &changed, MDX_OPTIONS).unwrap()
    );
    // Explicit flags DO change grammar, unlike those inherited configuration fields.
    let source = "<Poll options={[]} />\n";
    assert_ne!(
        satteri_mdxjs::compile(source, &options, MDX_OPTIONS).unwrap(),
        satteri_mdxjs::compile(source, &options, Options::empty()).unwrap()
    );
}

#[test]
fn compiler_error_coordinate_helper_returns_utf16_not_arena_byte_offsets() {
    let source = "é🦀\r\n{#bad}\n";
    let byte = source.find('{').unwrap();
    let message = satteri_mdxjs::parse_error_to_message(source, byte, "probe");
    let Some(place) = message.place else { panic!("position missing") };
    let Place::Point(point) = *place else { panic!("not a point") };
    assert_eq!((point.line, point.column, point.offset), (2, 1, 5));
    assert_eq!(byte, 8);
    assert_ne!(point.offset, byte);
    assert_eq!(Location::new(source).to_point(byte).unwrap().offset, 5);
}

#[test]
fn compiler_tab_normalization_moves_error_offsets_relative_to_original() {
    let source = "\t{#bad}\n";
    let (_, errors) = satteri_pulldown_cmark::parse(source, MDX_OPTIONS);
    let (offset, reason) = &errors[0];
    let direct = satteri_mdxjs::parse_error_to_message(source, *offset, reason);
    let compiled = stock(source).unwrap_err();
    let get = |message: satteri_arena::mdx_types::Message| match *message.place.unwrap() {
        Place::Point(p) => p,
        _ => panic!(),
    };
    let a = get(direct);
    let b = get(compiled);
    assert_eq!(b.offset, a.offset + 3);
}

#[test]
fn compiler_tab_normalization_changes_literal_fenced_code_payload() {
    let source = "```text\n\tliteral\n```\n";
    let (arena, errors) = satteri_pulldown_cmark::parse(source, MDX_OPTIONS);
    assert!(errors.is_empty());
    let via_arena = compile(&convert(&arena));
    let via_source = stock(source).unwrap();
    assert!(via_arena.contains("\\tliteral") || via_arena.contains("\tliteral"), "{via_arena:?}");
    assert!(via_source.contains("    literal"));
    assert_ne!(via_arena, via_source);
    // expand_tabs() runs before Markdown parsing, even on fenced payload lines.
}

#[test]
fn default_codegen_compiles_away_jsx_but_still_does_not_execute_it() {
    let js = satteri_mdxjs::compile("<Poll options={[\"Gardens\", \"Maps\"]} />\n", &Default::default(), MDX_OPTIONS).unwrap();
    assert!(js.contains("react/jsx-runtime"));
    assert!(!js.contains("<Poll"));
    assert!(js.contains("Gardens"));
}

#[test]
fn metadata_crossing_conversion_is_not_a_general_identity_transport() {
    let (arena, errors) = lower("{/* leafbud:key heading */}\n## Heading\n\n<Poll key=\"poll\" options={[]} />\n");
    assert!(errors.is_empty());
    let hast = convert(&arena);
    let data: Vec<serde_json::Value> = (0..hast.len() as u32)
        .filter_map(|id| hast.get_node_data(id).map(|b| serde_json::from_slice(b).unwrap()))
        .collect();
    assert!(data.iter().any(|d| d["leafbudKey"] == "poll"));
    assert!(!data.iter().any(|d| d["leafbudKey"] == "heading"));
    let js = compile(&hast);
    assert!(!js.contains("leafbudKey"));
    // Keep MDAST for Leaflet lowering: converting to render HTML/HAST loses prose identity.
}

#[test]
fn expression_and_esm_estree_helpers_are_public_but_not_a_storage_schema() {
    let parsed: serde_json::Value = serde_json::from_str(&satteri_mdxjs::parse_expression_to_estree_json("danger()").unwrap()).unwrap();
    assert_eq!(parsed["body"][0]["expression"]["type"], "CallExpression");
    assert!(satteri_mdxjs::parse_esm_to_estree_json("import X from 'not-installed'").is_some());
    assert!(satteri_mdxjs::parse_expression_to_estree_json("a +").is_none());
}
