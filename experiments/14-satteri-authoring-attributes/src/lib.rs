//! Optional brace-key experiment over the unchanged experiment 13 compiler.
mod adapter;
pub use adapter::{Annotation, CheckedDocument, Prepared, compile, prepare, semantic_document, semantic_plan};
pub use satteri_authoring_experiment::{facts, model, simulator, text};

use satteri_arena::{Arena, Mdast, decode_string_ref_data};
use satteri_ast::mdast::*;
use satteri_pulldown_cmark::{MDX_OPTIONS, Options};
use serde_json::{Value, json};

/// Inspection only, including the recovered tree when native MDX reports errors.
pub fn stock(source: &str, attributes: bool) -> Value {
    fn node(arena: &Arena<Mdast>, id: u32) -> Value {
        let native = arena.get_node(id);
        let kind = MdastNodeType::from_u8(native.node_type).expect("native node kind");
        let bytes = arena.get_type_data(id);
        let value = match kind {
            MdastNodeType::Text | MdastNodeType::InlineCode => Some(arena.get_str(decode_string_ref_data(bytes))),
            MdastNodeType::MdxFlowExpression | MdastNodeType::MdxTextExpression => Some(arena.get_str(decode_expression_data(bytes).value)),
            _ => None,
        };
        let metadata: Option<Value> = arena.get_node_data(id).map(|data| serde_json::from_slice(data).expect("native metadata"));
        json!({"kind":kind.name(),"start":native.start_offset,"end":native.end_offset,"value":value,"metadata":metadata,
            "children":arena.get_children(id).iter().map(|&id| node(arena,id)).collect::<Vec<_>>()})
    }
    let options = if attributes { MDX_OPTIONS | Options::ENABLE_ATTRIBUTES } else { MDX_OPTIONS };
    let (arena, errors) = satteri_pulldown_cmark::parse(source, options);
    json!({"attributes":attributes,"errors":errors,"tree":node(&arena,0)})
}
