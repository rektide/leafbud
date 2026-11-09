//! Parse-only authoring gate plus actual Sätteri compiler integration controls.
//! The parser is NOT mdxjs/markdown-rs: satteri-mdxjs uses this exact arena seam.
mod literal;

use std::collections::{BTreeMap, HashMap};

use satteri_arena::{Arena, Mdast, decode_string_ref_data};
use satteri_ast::mdast::*;
use satteri_pulldown_cmark::MDX_OPTIONS;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    /// Arena ranges are BOM-stripped UTF-8 bytes; native errors are zero-width.
    pub span: Span,
}

#[derive(Debug, Serialize)]
pub struct Node {
    pub id: u32,
    pub kind: String,
    pub span: Span,
    pub key: Option<String>,
    pub data: Value,
    pub details: Value,
    pub value: Option<String>,
    pub children: Vec<Node>,
}

#[derive(Debug, Serialize)]
pub struct Checked<'a> {
    #[serde(skip)]
    pub original_source: &'a str,
    pub position_source_start: usize,
    pub tree: Node,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn kind(arena: &Arena<Mdast>, id: u32) -> MdastNodeType {
    MdastNodeType::from_u8(arena.get_node(id).node_type).unwrap()
}

pub fn span(arena: &Arena<Mdast>, id: u32) -> Span {
    let n = arena.get_node(id);
    Span {
        start: n.start_offset,
        end: n.end_offset,
        start_line: n.start_line,
        end_line: n.end_line,
        start_column: n.start_column,
        end_column: n.end_column,
    }
}

pub fn metadata(arena: &Arena<Mdast>, id: u32) -> Value {
    arena.get_node_data(id).map_or(json!({}), |bytes| serde_json::from_slice(bytes).unwrap())
}

fn write_data(arena: &mut Arena<Mdast>, id: u32, data: Value) {
    arena.set_node_data(id, serde_json::to_vec(&data).unwrap());
}

fn fail(errors: &mut Vec<Diagnostic>, arena: &Arena<Mdast>, id: u32, code: &str, message: impl Into<String>) {
    errors.push(Diagnostic {
        code: code.into(),
        message: message.into(),
        span: span(arena, id),
    });
}

fn valid_key(key: &str) -> bool {
    let mut chars = key.bytes();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic()) && chars.all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}

fn target(kind: MdastNodeType) -> bool {
    use MdastNodeType as M;
    matches!(
        kind,
        M::Heading | M::Paragraph | M::Code | M::List | M::Blockquote | M::ThematicBreak | M::MdxJsxFlowElement
    )
}

fn expression(arena: &Arena<Mdast>, id: u32) -> &str {
    arena.get_str(decode_expression_data(arena.get_type_data(id)).value)
}

fn comment(value: &str) -> bool {
    value.strip_prefix("/*").and_then(|v| v.strip_suffix("*/")).is_some_and(|v| !v.contains("*/"))
}

/// Same immediate-sibling policy as 07, with 09's explicit blank-before guard.
/// This mitigates accidental key capture; it does not repair the native parser.
fn bind(arena: &mut Arena<Mdast>, parent: u32, errors: &mut Vec<Diagnostic>) {
    let children = arena.get_children(parent).to_vec();
    for &id in &children {
        bind(arena, id, errors);
    }
    let mut keep = Vec::new();
    for (i, &id) in children.iter().enumerate() {
        let k = kind(arena, id);
        if !matches!(k, MdastNodeType::MdxFlowExpression | MdastNodeType::MdxTextExpression)
            || !expression(arena, id).trim_start().starts_with("/* leafbud:")
        {
            keep.push(id);
            continue;
        }
        let value = expression(arena, id).to_owned();
        let Some(key) = value
            .strip_prefix("/* leafbud:key ")
            .and_then(|v| v.strip_suffix(" */"))
            .filter(|k| valid_key(k))
        else {
            fail(errors, arena, id, "malformed-key", "expected {/* leafbud:key NAME */}");
            continue;
        };
        if k != MdastNodeType::MdxFlowExpression {
            fail(errors, arena, id, "misplaced-key", "key must be a standalone flow comment");
            continue;
        }
        if i > 0 && span(arena, children[i - 1]).end_line.saturating_add(1) >= span(arena, id).start_line {
            fail(errors, arena, id, "key-boundary", "require a blank line before a key after an earlier sibling");
            continue;
        }
        let Some(&next) = children.get(i + 1) else {
            fail(errors, arena, id, "unbound-key", "no next sibling in this container");
            continue;
        };
        if !target(kind(arena, next)) {
            fail(errors, arena, id, "unsupported-target", "immediate next sibling is not a supported block");
            continue;
        }
        let mut data = metadata(arena, next);
        data["leafbudKey"] = key.into();
        data["leafbudKeySpan"] = serde_json::to_value(span(arena, id)).unwrap();
        write_data(arena, next, data);
    }
    arena.set_children(parent, &keep);
}

pub fn reachable(arena: &Arena<Mdast>, id: u32) -> Vec<u32> {
    let mut ids = vec![id];
    for &child in arena.get_children(id) {
        ids.extend(reachable(arena, child));
    }
    ids
}

fn gate(arena: &mut Arena<Mdast>, errors: &mut Vec<Diagnostic>) {
    use MdastNodeType as M;
    for id in reachable(arena, 0) {
        match kind(arena, id) {
            M::MdxFlowExpression | M::MdxTextExpression if !comment(expression(arena, id)) => {
                fail(errors, arena, id, "body-expression", "body expressions are not literal document data");
            }
            M::MdxjsEsm => fail(errors, arena, id, "esm", "imports/exports are not document data"),
            M::MdxJsxFlowElement | M::MdxJsxTextElement => {
                let bytes = arena.get_type_data(id);
                let name = arena.get_str(decode_mdx_jsx_element_name(bytes)).to_owned();
                if !["Poll", "Canvas", "Page", "Place", "Image", "Run", "HardBreak"].contains(&name.as_str()) {
                    fail(errors, arena, id, "unknown-component", format!("unknown component {name:?}"));
                }
                let mut props = BTreeMap::new();
                for i in 0..decode_mdx_jsx_attr_count(bytes) {
                    let (k, name, raw) = decode_mdx_jsx_attr(bytes, i);
                    let name = arena.get_str(name).to_owned();
                    let raw = arena.get_str(raw);
                    let value = match k {
                        MDX_ATTR_LITERAL_PROP => Value::String(raw.into()),
                        MDX_ATTR_BOOLEAN_PROP => Value::Bool(true),
                        MDX_ATTR_EXPRESSION_PROP => match literal::parse(raw) {
                            Ok(value) => value,
                            Err(error) => {
                                fail(errors, arena, id, "nonliteral-prop", format!("{name}: {error}"));
                                continue;
                            }
                        },
                        MDX_ATTR_SPREAD => {
                            fail(errors, arena, id, "spread", "spread attributes are not data");
                            continue;
                        }
                        _ => {
                            fail(errors, arena, id, "attribute-kind", "unknown native attribute kind");
                            continue;
                        }
                    };
                    if props.insert(name, value).is_some() {
                        fail(errors, arena, id, "duplicate-prop", "duplicate JSX attribute");
                    }
                }
                let mut data = metadata(arena, id);
                if let Some(key) = props.get("key") {
                    if let Some(key) = key.as_str().filter(|k| valid_key(k)) {
                        if data.get("leafbudKey").is_some() {
                            fail(errors, arena, id, "key-collision", "comment and component key both supplied");
                            data.as_object_mut().unwrap().remove("leafbudKey");
                            data.as_object_mut().unwrap().remove("leafbudKeySpan");
                        } else {
                            data["leafbudKey"] = key.into();
                        }
                    } else {
                        fail(errors, arena, id, "invalid-key", "component key must be an author-key string");
                    }
                }
                data["literalProps"] = serde_json::to_value(props).unwrap();
                write_data(arena, id, data);
            }
            _ => {}
        }
    }
    let mut keys: HashMap<String, Vec<u32>> = HashMap::new();
    for id in reachable(arena, 0) {
        if let Some(key) = metadata(arena, id)["leafbudKey"].as_str() {
            keys.entry(key.into()).or_default().push(id);
        }
    }
    for (key, ids) in keys.into_iter().filter(|(_, ids)| ids.len() > 1) {
        for id in ids {
            fail(errors, arena, id, "duplicate-key", format!("duplicate author key {key:?}"));
            let mut data = metadata(arena, id);
            data.as_object_mut().unwrap().remove("leafbudKey");
            data.as_object_mut().unwrap().remove("leafbudKeySpan");
            write_data(arena, id, data);
        }
    }
}

pub fn lower(source: &str) -> (Arena<Mdast>, Vec<Diagnostic>) {
    let (mut arena, native_errors) = satteri_pulldown_cmark::parse(source, MDX_OPTIONS);
    let mut errors: Vec<_> = native_errors
        .into_iter()
        .map(|(offset, message)| Diagnostic {
            code: "parse".into(),
            message,
            span: Span {
                start: offset as u32,
                end: offset as u32,
                start_line: 0,
                end_line: 0,
                start_column: 0,
                end_column: 0,
            },
        })
        .collect();
    bind(&mut arena, 0, &mut errors);
    gate(&mut arena, &mut errors);
    errors.sort_by(|a, b| (a.span.start, &a.code).cmp(&(b.span.start, &b.code)));
    (arena, errors)
}

pub fn project(arena: &Arena<Mdast>, id: u32) -> Node {
    use MdastNodeType as M;
    let k = kind(arena, id);
    let bytes = arena.get_type_data(id);
    let mut details = json!({});
    let value = match k {
        M::Text | M::InlineCode => Some(arena.get_str(decode_string_ref_data(bytes)).into()),
        M::MdxFlowExpression | M::MdxTextExpression | M::MdxjsEsm => Some(expression(arena, id).into()),
        M::Heading => {
            details["depth"] = bytes[0].into();
            None
        }
        M::Code => {
            let code = CodeData::from_bytes(bytes);
            details = json!({"lang": arena.get_str(code.lang), "meta": arena.get_str(code.meta)});
            Some(arena.get_str(code.value).into())
        }
        M::List => {
            let list = ListData::from_bytes(bytes);
            details = json!({"ordered": list.ordered, "start": list.start, "spread": list.spread});
            None
        }
        M::ListItem => {
            let item = ListItemData::from_bytes(bytes);
            details = json!({"checked": item.checked, "spread": item.spread});
            None
        }
        M::Link => {
            let link = LinkData::from_bytes(bytes);
            details = json!({"url": arena.get_str(link.url), "title": arena.get_str(link.title)});
            None
        }
        M::MdxJsxFlowElement | M::MdxJsxTextElement => {
            let attrs: Vec<_> = (0..decode_mdx_jsx_attr_count(bytes))
                .map(|i| {
                    let (kind, name, raw) = decode_mdx_jsx_attr(bytes, i);
                    json!({"kind": kind, "name": arena.get_str(name), "raw": arena.get_str(raw)})
                })
                .collect();
            details = json!({"name": arena.get_str(decode_mdx_jsx_element_name(bytes)), "attributes": attrs});
            None
        }
        _ => None,
    };
    let data = metadata(arena, id);
    Node {
        id,
        kind: k.name().into(),
        span: span(arena, id),
        key: data["leafbudKey"].as_str().map(str::to_owned),
        data,
        details,
        value,
        children: arena.get_children(id).iter().map(|&child| project(arena, child)).collect(),
    }
}

pub fn check(source: &str) -> Checked<'_> {
    let (arena, diagnostics) = lower(source);
    Checked {
        original_source: source,
        position_source_start: if source.starts_with('\u{feff}') { 3 } else { 0 },
        tree: project(&arena, 0),
        diagnostics,
    }
}

/// Compiler control only: emitted JavaScript is never executed or published.
pub fn stock(source: &str) -> Result<String, satteri_arena::mdx_types::Message> {
    satteri_mdxjs::compile(
        source,
        &satteri_mdxjs::Options {
            jsx: true,
            ..Default::default()
        },
        MDX_OPTIONS,
    )
}

/// Public intended integration: mutate MDAST, convert, set MDX flag, compile HAST.
/// The original source is not fed back through compile(), which would discard edits.
pub fn staged(source: &str) -> Result<String, Vec<Diagnostic>> {
    let (arena, diagnostics) = lower(source);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut hast = satteri_ast::hast::mdast_arena_to_hast_arena_with_options(&arena, &Default::default());
    hast.mdx = true;
    satteri_mdxjs::compile_hast_arena(
        &hast,
        &satteri_mdxjs::Options {
            jsx: true,
            ..Default::default()
        },
    )
    .map_err(|error| {
        vec![Diagnostic {
            code: "compile".into(),
            message: error.to_string(),
            span: span(&arena, 0),
        }]
    })
}
