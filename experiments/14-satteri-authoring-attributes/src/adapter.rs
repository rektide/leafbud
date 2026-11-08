//! Adapter, NOT stock attribute support: discover narrowly reserved brace nodes
//! in Satteri's recovered native arena, blank their exact bytes, strictly compile
//! again through unchanged experiment 13, then attach by actual node start/kind.
//! No custom Markdown lexer, HTML round trip, generated Key tags or source writes.
use std::collections::BTreeMap;

use satteri_arena::{Arena, Mdast};
use satteri_ast::mdast::{MdastNodeType as M, decode_expression_data};
use satteri_authoring_experiment::{author, facts::Plan, model::*, text};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Heading,
    Paragraph,
}

#[derive(Clone, Debug, Serialize)]
pub struct Annotation {
    pub key: String,
    pub span: Span,
    pub target: Target,
    /// Native target range from the original discovery parse. Final binding
    /// requires this exact start + kind, not a nearest-text search.
    pub target_span: Span,
}

#[derive(Debug, Serialize)]
pub struct Prepared {
    pub parser_source: String,
    pub annotations: Vec<Annotation>,
    /// Discovery-only errors, rebased to absolute original UTF-8 bytes.
    pub discovery_errors: Vec<(usize, String)>,
}

#[derive(Debug, Serialize)]
pub struct CheckedDocument<'a> {
    pub document: Document<'a>,
    pub annotations: Vec<Annotation>,
    pub discovery_error_count: usize,
}

fn valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|c| c.is_ascii_alphabetic()) && bytes.all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
fn kind(arena: &Arena<Mdast>, id: u32) -> M {
    M::from_u8(arena.get_node(id).node_type).expect("native node kind")
}
fn span(arena: &Arena<Mdast>, id: u32, base: usize) -> Span {
    let n = arena.get_node(id);
    Span {
        start: base + n.start_offset as usize,
        end: base + n.end_offset as usize,
    }
}
fn error(span: Span, code: &'static str, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        message: message.into(),
        span,
    }
}
fn line(source: &str, offset: usize) -> usize {
    source.as_bytes()[..offset].iter().filter(|&&c| c == b'\n').count()
}

struct Discovery<'a> {
    source: &'a str,
    arena: Arena<Mdast>,
    base: usize,
    annotations: Vec<Annotation>,
}
impl Discovery<'_> {
    fn walk(&mut self, parent: u32) -> Result<(), Diagnostic> {
        let children = self.arena.get_children(parent).to_vec();
        for (index, &id) in children.iter().enumerate() {
            let k = kind(&self.arena, id);
            if !matches!(k, M::MdxFlowExpression | M::MdxTextExpression) {
                self.walk(id)?;
                continue;
            }
            let value = self.arena.get_str(decode_expression_data(self.arena.get_type_data(id)).value);
            if !value.trim_start().starts_with('#') {
                continue;
            }
            let location = span(&self.arena, id, self.base);
            let Some(key) = value.strip_prefix('#').filter(|key| valid_name(key)) else {
                return Err(error(
                    location,
                    "malformed-attribute",
                    "expected exactly {#name}, using [A-Za-z][A-Za-z0-9_.-]*; no classes or arbitrary attributes",
                ));
            };
            if self.source.get(location.start..location.end) != Some(format!("{{#{key}}}").as_str()) {
                return Err(error(location, "malformed-attribute", "expected exact single-line {#name} source"));
            }
            let (target, target_id) = match (k, kind(&self.arena, parent)) {
                (M::MdxTextExpression, M::Heading) => {
                    if index + 1 != children.len() || !self.source[..location.start].ends_with([' ', '\t']) {
                        return Err(error(
                            location,
                            "attribute-position",
                            "heading key must be the final inline, separated from heading content by whitespace",
                        ));
                    }
                    (Target::Heading, parent)
                }
                (M::MdxFlowExpression, M::Root | M::ListItem | M::Blockquote | M::MdxJsxFlowElement) => {
                    let Some(&previous) = index.checked_sub(1).and_then(|i| children.get(i)) else {
                        return Err(error(location, "orphan-attribute", "no immediately preceding paragraph in this container"));
                    };
                    let previous_span = span(&self.arena, previous, self.base);
                    if kind(&self.arena, previous) != M::Paragraph
                        || line(self.source, previous_span.end.saturating_sub(1)) + 1 != line(self.source, location.start)
                    {
                        return Err(error(
                            location,
                            "orphan-attribute",
                            "paragraph key must be on its immediately following line, without a blank or escaped container boundary",
                        ));
                    }
                    if let Some(&next) = children.get(index + 1) {
                        let next_span = span(&self.arena, next, self.base);
                        if line(self.source, next_span.start) <= line(self.source, location.end) + 1 {
                            return Err(error(
                                location,
                                "attribute-not-tail",
                                "end the paragraph key with a blank line or container closure before following content",
                            ));
                        }
                    }
                    (Target::Paragraph, previous)
                }
                _ => {
                    return Err(error(
                        location,
                        "attribute-position",
                        "use a heading suffix or a separate final line after an ordinary paragraph; inline paragraph suffixes are outside this experiment",
                    ));
                }
            };
            self.annotations.push(Annotation {
                key: key.into(),
                span: location,
                target,
                target_span: span(&self.arena, target_id, self.base),
            });
        }
        Ok(())
    }
}

/// A diagnostic-bearing discovery tree is never treated as a validated document.
/// Only exact reserved nodes are masked; the amended source must pass the full
/// unchanged MDX/parser/schema compiler before any result can be returned.
pub fn prepare(source: &str) -> Result<Prepared, Vec<Diagnostic>> {
    let (arena, discovery_errors) = satteri_pulldown_cmark::parse(source, satteri_pulldown_cmark::MDX_OPTIONS);
    let base = if source.starts_with('\u{feff}') { 3 } else { 0 };
    let mut discovery = Discovery {
        source,
        arena,
        base,
        annotations: vec![],
    };
    discovery.walk(0).map_err(|d| vec![d])?;
    let mut bytes = source.as_bytes().to_vec();
    for annotation in &discovery.annotations {
        // IDs are ASCII; no line ending is removed and every byte offset stays
        // identical. There is no length-changing insertion/source-map heuristic.
        bytes[annotation.span.start..annotation.span.end].fill(b' ');
    }
    Ok(Prepared {
        parser_source: String::from_utf8(bytes).expect("only ASCII annotation bytes changed"),
        annotations: discovery.annotations,
        discovery_errors: discovery_errors.into_iter().map(|(offset, message)| (base + offset, message)).collect(),
    })
}

fn visit_blocks(blocks: &mut [Block], visit: &mut impl FnMut(&mut Block)) {
    for block in blocks {
        visit(block);
        match &mut block.kind {
            BlockKind::List { items, .. } => {
                for item in items {
                    visit_blocks(item, visit);
                }
            }
            BlockKind::Quote { paragraph } => visit_blocks(std::slice::from_mut(paragraph), visit),
            _ => {}
        }
    }
}
fn visit_document(doc: &mut Document<'_>, mut visit: impl FnMut(&mut Block)) {
    visit_blocks(&mut doc.main, &mut visit);
    for page in &mut doc.pages {
        visit_blocks(&mut page.blocks, &mut visit);
    }
}

pub fn compile(source: &str) -> Result<CheckedDocument<'_>, Vec<Diagnostic>> {
    let prepared = prepare(source)?;
    let mut doc = author::compile(&prepared.parser_source)?;
    let mut namespace = BTreeMap::new();
    visit_document(&mut doc, |block| {
        if let Some(key) = &block.key {
            namespace.insert(key.name.clone(), key.span);
        }
    });
    for annotation in &prepared.annotations {
        let mut matches = 0;
        let mut collision = false;
        visit_document(&mut doc, |block| {
            let target_kind = match block.kind {
                BlockKind::Heading { .. } => Some(Target::Heading),
                BlockKind::Paragraph { .. } => Some(Target::Paragraph),
                _ => None,
            };
            if target_kind == Some(annotation.target) && block.span.start == annotation.target_span.start {
                matches += 1;
                if block.key.is_some() {
                    collision = true;
                } else {
                    block.key = Some(AuthorKey {
                        name: annotation.key.clone(),
                        span: annotation.span,
                    });
                }
            }
        });
        if collision {
            return Err(vec![error(
                annotation.span,
                "key-collision",
                "both native Key and brace key name this block; neither takes precedence",
            )]);
        }
        if matches != 1 {
            return Err(vec![error(
                annotation.span,
                "attribute-target-changed",
                "strict reparse did not produce exactly the original target start/kind",
            )]);
        }
        if let Some(first) = namespace.get(&annotation.key) {
            return Err(vec![error(
                annotation.span,
                "duplicate-key",
                format!("duplicate author key {:?}; first key at bytes {}..{}", annotation.key, first.start, first.end),
            )]);
        }
        namespace.insert(annotation.key.clone(), annotation.span);
    }
    // Only the source field borrows parser_source. All IR content is owned; bind
    // the returned document to the retained ORIGINAL source, not the masked copy.
    let Document {
        metadata,
        main,
        pages,
        consumed_keys,
        ..
    } = doc;
    Ok(CheckedDocument {
        document: Document {
            original_source: source,
            metadata,
            main,
            pages,
            consumed_keys,
        },
        discovery_error_count: prepared.discovery_errors.len(),
        annotations: prepared.annotations,
    })
}

fn remove_source_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for key in ["span", "source", "consumed_keys"] {
                map.remove(key);
            }
            for value in map.values_mut() {
                remove_source_fields(value);
            }
        }
        Value::Array(items) => {
            for value in items {
                remove_source_fields(value);
            }
        }
        _ => {}
    }
}
/// Compare semantics, not different annotation spellings' byte positions/counts.
pub fn semantic_document(doc: &Document<'_>) -> Value {
    let mut value = serde_json::to_value(doc).expect("serializable authoring document");
    remove_source_fields(&mut value);
    value
}
/// Use the same injected IDs/generation. Only Yrs representation bytes and source
/// attribution are normalized; text, marks, geometry/ordering and identities are not.
pub fn semantic_plan(plan: &Plan) -> anyhow::Result<Value> {
    let mut value = serde_json::to_value(plan)?;
    for fact in value["facts"].as_array_mut().expect("facts array") {
        if fact["data"]["type"] == "text" {
            let content = text::read(fact["data"]["value"].as_str().expect("text value"))?;
            fact["data"]["value"] = serde_json::to_value(content)?;
        }
    }
    remove_source_fields(&mut value);
    Ok(value)
}
