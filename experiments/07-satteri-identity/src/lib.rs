//! Native Sätteri parsing + a deliberately small, non-executing authoring lowering.
//! No source preprocessing, HTML round-trip, or source writer.
use std::collections::{BTreeMap, HashMap};

use satteri_arena::{Arena, Mdast, StringRef, decode_string_ref_data};
use satteri_ast::mdast::*;
use satteri_pulldown_cmark::{MDX_OPTIONS, Options};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Plain,
    Attributes,
    Mdx,
    Author,
}

impl Mode {
    pub fn options(self) -> Options {
        match self {
            Self::Plain => Options::empty(),
            Self::Attributes => Options::ENABLE_ATTRIBUTES,
            Self::Mdx => MDX_OPTIONS,
            Self::Author => MDX_OPTIONS,
        }
    }

    fn lowers_keys(self) -> bool {
        matches!(self, Self::Attributes)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// Small typed inspection tree, not a complete Leaflet IR.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum Kind {
    Root,
    Heading,
    Paragraph,
    List,
    ListItem,
    Blockquote,
    Code,
    ThematicBreak,
    Text,
    Strong,
    Emphasis,
    InlineCode,
    Break,
    Link,
    Image,
    Component,
    Comment,
    Expression,
    Esm,
    Other(String),
}

#[derive(Debug, Serialize)]
pub struct Node {
    pub id: u32,
    pub kind: Kind,
    pub span: Span,
    pub key: Option<String>,
    /// Only comment keys expose an exact annotation range. Native attribute
    /// metadata and JSX property codecs do not expose individual token ranges.
    pub key_span: Option<Span>,
    pub value: Option<String>,
    pub details: Value,
    pub data: Value,
    pub children: Vec<Node>,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    /// A native node range, NOT a fabricated annotation range.
    pub span: Span,
}

#[derive(Debug, Serialize)]
pub struct Parsed {
    #[serde(skip)]
    original: String,
    /// Native arena offsets exclude an initial UTF-8 BOM. Add this base to
    /// slice original_source() rather than the parser's BOM-stripped source.
    pub position_source_start: usize,
    pub tree: Node,
    pub diagnostics: Vec<Diagnostic>,
}

impl Parsed {
    pub fn original_source(&self) -> &str {
        &self.original
    }
}

pub fn native(source: &str, mode: Mode) -> (Arena<Mdast>, Vec<(usize, String)>) {
    satteri_pulldown_cmark::parse(source, mode.options())
}

fn span(arena: &Arena<Mdast>, id: u32) -> Span {
    let n = arena.get_node(id);
    Span {
        start: n.start_offset as usize,
        end: n.end_offset as usize,
        start_line: n.start_line,
        start_column: n.start_column,
        end_line: n.end_line,
        end_column: n.end_column,
    }
}

fn data(arena: &Arena<Mdast>, id: u32) -> Value {
    arena
        .get_node_data(id)
        .map_or(json!({}), |bytes| serde_json::from_slice(bytes).expect("Sätteri node data must be JSON"))
}

fn diagnose(out: &mut Vec<Diagnostic>, arena: &Arena<Mdast>, id: u32, code: &str, message: impl Into<String>) {
    out.push(Diagnostic {
        code: code.into(),
        message: message.into(),
        span: span(arena, id),
    });
}

pub fn valid_key(key: &str) -> bool {
    let mut chars = key.bytes();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic()) && chars.all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}

fn block_kind(kind: MdastNodeType) -> bool {
    matches!(
        kind,
        MdastNodeType::Heading
            | MdastNodeType::Paragraph
            | MdastNodeType::List
            | MdastNodeType::ListItem
            | MdastNodeType::Blockquote
            | MdastNodeType::Code
            | MdastNodeType::ThematicBreak
            | MdastNodeType::MdxJsxFlowElement
    )
}

/// Transfer already-parsed hProperties.id to internal metadata. We do NOT guess
/// attachment from lines or make native attribute parsing stricter than it is.
fn lower_keys(arena: &mut Arena<Mdast>, diagnostics: &mut Vec<Diagnostic>) {
    let mut keys: HashMap<String, Vec<u32>> = HashMap::new();
    for id in 0..arena.len() as u32 {
        let mut metadata = data(arena, id);
        let Some(properties) = metadata.get_mut("hProperties").and_then(Value::as_object_mut) else {
            continue;
        };
        let Some(key) = properties.remove("id") else { continue };
        let kind = MdastNodeType::from_u8(arena.get_node(id).node_type).unwrap();
        if let Some(key) = key.as_str() {
            if !valid_key(key) {
                diagnose(diagnostics, arena, id, "invalid-key", format!("invalid author key {key:?}"));
            } else if !block_kind(kind) {
                diagnose(
                    diagnostics,
                    arena,
                    id,
                    "non-block-key",
                    "native attributes attached this ID to an inline/unsupported node",
                );
            } else if kind == MdastNodeType::Paragraph && arena.get_children(id).is_empty() {
                diagnose(
                    diagnostics,
                    arena,
                    id,
                    "unbound-key",
                    "native attributes produced an empty paragraph; no authored block to key",
                );
            } else {
                keys.entry(key.into()).or_default().push(id);
            }
        } else {
            diagnose(diagnostics, arena, id, "invalid-key", "ID is not a string");
        }
        arena.set_node_data(id, serde_json::to_vec(&metadata).unwrap());
    }
    for (key, ids) in keys {
        if ids.len() > 1 {
            for id in ids {
                diagnose(diagnostics, arena, id, "duplicate-key", format!("document-wide duplicate author key {key:?}"));
            }
        } else {
            let id = ids[0];
            let mut metadata = data(arena, id);
            metadata["springleafKey"] = key.into();
            arena.set_node_data(id, serde_json::to_vec(&metadata).unwrap());
        }
    }
}

fn reserved_directive(value: &str) -> Option<Result<&str, &'static str>> {
    if !value.trim_start().starts_with("/* springleaf:") {
        return None;
    }
    Some(
        value
            .strip_prefix("/* springleaf:key ")
            .and_then(|v| v.strip_suffix(" */"))
            .filter(|key| valid_key(key))
            .ok_or("expected one comment: {/* springleaf:key NAME */}; NAME matches [A-Za-z][A-Za-z0-9_.-]*"),
    )
}

fn comment_only(value: &str) -> bool {
    value
        .strip_prefix("/*")
        .and_then(|v| v.strip_suffix("*/"))
        .is_some_and(|body| !body.contains("*/"))
}

/// Bind native MDX expression nodes to immediate siblings, with no scanning of
/// source lines and no injected lexer. Removal is through the public arena API.
fn bind_comments(arena: &mut Arena<Mdast>, parent: u32, diagnostics: &mut Vec<Diagnostic>) {
    let children = arena.get_children(parent).to_vec();
    for &child in &children {
        bind_comments(arena, child, diagnostics);
    }
    let mut keep = Vec::new();
    for (index, &id) in children.iter().enumerate() {
        let kind = MdastNodeType::from_u8(arena.get_node(id).node_type).unwrap();
        if !matches!(kind, MdastNodeType::MdxFlowExpression | MdastNodeType::MdxTextExpression) {
            keep.push(id);
            continue;
        }
        let value = arena.get_str(decode_expression_data(arena.get_type_data(id)).value).to_owned();
        let Some(directive) = reserved_directive(&value) else {
            keep.push(id);
            continue;
        };
        let key = match directive {
            Ok(key) => key,
            Err(message) => {
                diagnose(diagnostics, arena, id, "malformed-key", message);
                continue;
            }
        };
        if kind != MdastNodeType::MdxFlowExpression {
            diagnose(
                diagnostics,
                arena,
                id,
                "misplaced-key",
                "key comments must be native flow expressions on their own line",
            );
            continue;
        }
        let Some(&target) = children.get(index + 1) else {
            diagnose(diagnostics, arena, id, "unbound-key", "no following sibling in this container");
            continue;
        };
        let target_kind = MdastNodeType::from_u8(arena.get_node(target).node_type).unwrap();
        if !block_kind(target_kind) {
            diagnose(
                diagnostics,
                arena,
                id,
                "unsupported-target",
                format!("immediate sibling is {target_kind:?}; no skipping"),
            );
            continue;
        }
        let mut metadata = data(arena, target);
        metadata["springleafKey"] = key.into();
        metadata["springleafKeySpan"] = serde_json::to_value(span(arena, id)).unwrap();
        arena.set_node_data(target, serde_json::to_vec(&metadata).unwrap());
    }
    arena.set_children(parent, &keep);
}

fn string(arena: &Arena<Mdast>, reference: StringRef) -> String {
    arena.get_str(reference).into()
}

fn project(arena: &Arena<Mdast>, id: u32) -> Node {
    use MdastNodeType as M;
    let raw_kind = M::from_u8(arena.get_node(id).node_type).unwrap();
    let bytes = arena.get_type_data(id);
    let mut value = None;
    let mut details = json!({});
    let kind = match raw_kind {
        M::Root => Kind::Root,
        M::Paragraph => Kind::Paragraph,
        M::Heading => {
            details = json!({"depth": bytes[0]});
            Kind::Heading
        }
        M::Blockquote => Kind::Blockquote,
        M::List => {
            let list = ListData::from_bytes(bytes);
            details = json!({"ordered": list.ordered, "start": list.start, "spread": list.spread});
            Kind::List
        }
        M::ListItem => {
            let item = ListItemData::from_bytes(bytes);
            details = json!({"checked": item.checked, "spread": item.spread});
            Kind::ListItem
        }
        M::Code => {
            let code = CodeData::from_bytes(bytes);
            value = Some(string(arena, code.value));
            details = json!({"lang": string(arena, code.lang), "meta": string(arena, code.meta)});
            Kind::Code
        }
        M::ThematicBreak => Kind::ThematicBreak,
        M::Text | M::InlineCode => {
            value = Some(string(arena, decode_string_ref_data(bytes)));
            if raw_kind == M::Text { Kind::Text } else { Kind::InlineCode }
        }
        M::Strong => Kind::Strong,
        M::Emphasis => Kind::Emphasis,
        M::Break => Kind::Break,
        M::Link => {
            let link = LinkData::from_bytes(bytes);
            details = json!({"url": string(arena, link.url), "title": string(arena, link.title)});
            Kind::Link
        }
        M::Image => {
            let image = ImageData::from_bytes(bytes);
            details = json!({"url": string(arena, image.url), "alt": string(arena, image.alt), "title": string(arena, image.title)});
            Kind::Image
        }
        M::MdxJsxFlowElement | M::MdxJsxTextElement => {
            let attrs: Vec<Value> = (0..decode_mdx_jsx_attr_count(bytes))
                .map(|i| {
                    let (kind, name, value) = decode_mdx_jsx_attr(bytes, i);
                    json!({"kind": kind, "name": string(arena, name), "value": string(arena, value)})
                })
                .collect();
            details = json!({"name": string(arena, decode_mdx_jsx_element_name(bytes)),
                "flow": raw_kind == M::MdxJsxFlowElement, "attributes": attrs});
            Kind::Component
        }
        M::MdxFlowExpression | M::MdxTextExpression | M::MdxjsEsm => {
            value = Some(string(arena, decode_expression_data(bytes).value));
            if raw_kind == M::MdxjsEsm {
                Kind::Esm
            } else if comment_only(value.as_deref().unwrap()) {
                Kind::Comment
            } else {
                Kind::Expression
            }
        }
        other => Kind::Other(other.name().into()),
    };
    let metadata = data(arena, id);
    Node {
        id,
        kind,
        span: span(arena, id),
        key: metadata.get("springleafKey").and_then(Value::as_str).map(str::to_owned),
        key_span: metadata.get("springleafKeySpan").map(|value| serde_json::from_value(value.clone()).unwrap()),
        value,
        details,
        data: metadata,
        children: arena.get_children(id).iter().map(|&id| project(arena, id)).collect(),
    }
}

/// Bounded component control: two names, JSON-only expression values, no runtime.
/// It is a gate/projection, not a renderer or complete component schema compiler.
fn literal_gate(node: &mut Node, diagnostics: &mut Vec<Diagnostic>) {
    let mut fail = |code: &str, message: String| {
        diagnostics.push(Diagnostic {
            code: code.into(),
            message,
            span: node.span,
        })
    };
    if matches!(node.kind, Kind::Expression | Kind::Esm) {
        fail("unsupported-expression", "standalone expressions/ESM are not literal document data".into());
    }
    if node.kind == Kind::Component {
        let name = node.details["name"].as_str().unwrap();
        if !["Poll", "Image"].contains(&name) {
            fail("unknown-component", format!("unknown component {name:?}; not prose"));
        }
        let mut props = BTreeMap::new();
        for attr in node.details["attributes"].as_array().unwrap() {
            let name = attr["name"].as_str().unwrap();
            let raw = attr["value"].as_str().unwrap();
            let value = match attr["kind"].as_u64().unwrap() as u8 {
                MDX_ATTR_LITERAL_PROP => Value::String(raw.into()),
                MDX_ATTR_BOOLEAN_PROP => Value::Bool(true),
                MDX_ATTR_EXPRESSION_PROP => match serde_json::from_str::<Value>(raw) {
                    Ok(value) => value,
                    Err(_) => {
                        fail("nonliteral-prop", format!("property {name:?} must contain exactly one JSON value, not {raw:?}"));
                        continue;
                    }
                },
                MDX_ATTR_SPREAD => {
                    fail("spread-prop", "spread attributes are not document data".into());
                    continue;
                }
                other => {
                    fail("unknown-prop-kind", format!("unknown attribute kind {other}"));
                    continue;
                }
            };
            if props.insert(name.to_owned(), value).is_some() {
                fail("duplicate-prop", format!("duplicate property {name:?}"));
            }
        }
        // Component keys share the document's author-key namespace below.
        if let Some(key) = props.get("key") {
            if let Some(key) = key.as_str().filter(|key| valid_key(key)) {
                if node.key.is_some() {
                    fail("key-collision", "component has both comment annotation and key property".into());
                    node.key = None;
                    node.key_span = None;
                    node.data.as_object_mut().unwrap().remove("springleafKey");
                    node.data.as_object_mut().unwrap().remove("springleafKeySpan");
                } else {
                    node.key = Some(key.into());
                    node.data["springleafKey"] = key.into();
                }
            } else {
                fail("invalid-key", "component key must be an author-key string".into());
            }
        }
        node.details["literal_props"] = serde_json::to_value(props).unwrap();
    }
    for child in &mut node.children {
        literal_gate(child, diagnostics);
    }
}

fn collect_keys(node: &Node, keys: &mut HashMap<String, Vec<Span>>) {
    if let Some(key) = &node.key {
        keys.entry(key.clone()).or_default().push(node.span);
    }
    for child in &node.children {
        collect_keys(child, keys);
    }
}

fn reject_duplicates(node: &mut Node, duplicates: &HashMap<String, Vec<Span>>, diagnostics: &mut Vec<Diagnostic>) {
    if node.key.as_ref().is_some_and(|key| duplicates.contains_key(key)) {
        diagnostics.push(Diagnostic {
            code: "duplicate-key".into(),
            message: format!("duplicate prose/component key {:?}", node.key),
            span: node.key_span.unwrap_or(node.span),
        });
        node.key = None;
        node.key_span = None;
        node.data.as_object_mut().unwrap().remove("springleafKey");
        node.data.as_object_mut().unwrap().remove("springleafKeySpan");
    }
    for child in &mut node.children {
        reject_duplicates(child, duplicates, diagnostics);
    }
}

pub fn parse(source: &str, mode: Mode) -> Parsed {
    let (mut arena, errors) = native(source, mode);
    let mut diagnostics: Vec<Diagnostic> = errors
        .into_iter()
        .map(|(offset, message)| Diagnostic {
            code: "mdx-parse".into(),
            message,
            // Upstream error exposes one byte offset, not a delimiter range or lines.
            span: Span {
                start: offset,
                end: offset,
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
            },
        })
        .collect();
    if mode.lowers_keys() {
        lower_keys(&mut arena, &mut diagnostics);
    }
    if matches!(mode, Mode::Author) {
        bind_comments(&mut arena, 0, &mut diagnostics);
    }
    let mut tree = project(&arena, 0);
    if matches!(mode, Mode::Author) {
        literal_gate(&mut tree, &mut diagnostics);
        let mut keys = HashMap::new();
        collect_keys(&tree, &mut keys);
        keys.retain(|_, spans| spans.len() > 1);
        reject_duplicates(&mut tree, &keys, &mut diagnostics);
    }
    diagnostics.sort_by(|a, b| (a.span.start, &a.code).cmp(&(b.span.start, &b.code)));
    Parsed {
        original: source.into(),
        position_source_start: if source.starts_with('\u{feff}') { 3 } else { 0 },
        tree,
        diagnostics,
    }
}
