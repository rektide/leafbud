//! Offline MDX parse-only experiment; no source rewriting or JavaScript execution.
mod literal;

use markdown::mdast::{AttributeContent, AttributeValue, Node};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

fn span(node: &Node) -> Option<Span> {
    node.position().map(|p| Span {
        start: p.start.offset,
        end: p.end.offset,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Kind {
    Root,
    Paragraph,
    Heading,
    Blockquote,
    List,
    ListItem,
    Text,
    Strong,
    Emphasis,
    Link,
    LinkReference,
    Definition,
    Image,
    ImageReference,
    InlineCode,
    Code,
    Break,
    ThematicBreak,
    Component,
    Comment,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Details {
    None,
    Heading {
        depth: u8,
    },
    List {
        ordered: bool,
        start: Option<u32>,
        spread: bool,
    },
    ListItem {
        spread: bool,
        checked: Option<bool>,
    },
    Resource {
        url: String,
        title: Option<String>,
        alt: Option<String>,
    },
    Reference {
        identifier: String,
        label: Option<String>,
        reference_kind: String,
        alt: Option<String>,
    },
    Definition {
        identifier: String,
        label: Option<String>,
        url: String,
        title: Option<String>,
    },
    Code {
        lang: Option<String>,
        meta: Option<String>,
    },
    Component {
        name: Option<String>,
        flow: bool,
        attributes: Vec<Property>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Property {
    pub name: String,
    /// Only valid literals receive data. Invalid attributes still diagnose.
    pub value: Option<Value>,
    pub raw_expression: Option<String>,
    /// Upstream relative-value-byte -> absolute-source-byte mapping, not a full attribute range.
    pub expression_stops: Vec<markdown::mdast::Stop>,
    /// mdast has no complete JSX property span; never fabricate one.
    pub span: Option<Span>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum KeyOrigin {
    Comment,
    ComponentAttribute,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Projected {
    pub kind: Kind,
    pub span: Option<Span>,
    pub key: Option<String>,
    pub key_origin: Option<KeyOrigin>,
    pub key_span: Option<Span>,
    pub text: Option<String>,
    pub details: Details,
    /// Ordered blocks, inlines, or component children, never HTML re-parsing.
    pub children: Vec<Projected>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Serialize)]
pub struct Parsed<'a> {
    #[serde(skip)]
    pub original_source: &'a str,
    pub tree: Projected,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn stock(source: &str) -> Result<Node, markdown::message::Message> {
    mdxjs::mdast_util_from_mdx(source, &mdxjs::Options::default())
}

/// Full MDX syntax first; identity binding and a deliberately smaller data dialect second.
pub fn parse(source: &str) -> Result<Parsed<'_>, markdown::message::Message> {
    let mdast = stock(source)?;
    let mut compiler = Compiler { diagnostics: Vec::new() };
    let mut tree = compiler.project(&mdast);
    let mut counts = BTreeMap::new();
    count_keys(&tree, &mut counts);
    compiler.reject_duplicate_keys(&mut tree, &counts);
    Ok(Parsed {
        original_source: source,
        tree,
        diagnostics: compiler.diagnostics,
    })
}

fn valid_key(key: &str) -> bool {
    key.as_bytes().first().is_some_and(u8::is_ascii_alphabetic) && key.bytes().all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}

fn reserved(value: &str) -> bool {
    value.trim().starts_with("/* springleaf:")
}

fn directive(value: &str) -> Option<&str> {
    // This namespace deliberately has one spelling, not a JS comment interpreter.
    let key = value.strip_prefix("/* springleaf:key ")?.strip_suffix(" */")?;
    valid_key(key).then_some(key)
}

fn comment_only(value: &str) -> bool {
    let value = value.trim();
    value
        .strip_prefix("/*")
        .and_then(|v| v.strip_suffix("*/"))
        .is_some_and(|inside| !inside.contains("*/"))
}

fn count_keys(node: &Projected, counts: &mut BTreeMap<String, usize>) {
    if let Some(key) = &node.key {
        *counts.entry(key.clone()).or_default() += 1;
    }
    for child in &node.children {
        count_keys(child, counts);
    }
}

struct Compiler {
    diagnostics: Vec<Diagnostic>,
}

impl Compiler {
    fn error(&mut self, code: &'static str, message: impl Into<String>, span: Option<Span>) {
        self.diagnostics.push(Diagnostic {
            code,
            message: message.into(),
            span,
        });
    }

    fn reject_duplicate_keys(&mut self, node: &mut Projected, counts: &BTreeMap<String, usize>) {
        if let Some(key) = &node.key
            && counts[key] > 1
        {
            self.error("duplicate-key", format!("duplicate document key: {key}"), node.key_span.or(node.span));
            node.key = None;
            node.key_span = None;
            node.key_origin = None;
        }
        for child in &mut node.children {
            self.reject_duplicate_keys(child, counts);
        }
    }

    fn children(&mut self, nodes: &[Node], block_container: bool) -> Vec<Projected> {
        let mut output = Vec::new();
        let mut pending: Option<(String, Option<Span>)> = None;
        for (index, node) in nodes.iter().enumerate() {
            if let Node::MdxFlowExpression(expression) = node
                && reserved(&expression.value)
            {
                if let Some((key, location)) = pending.take() {
                    self.error("unbound-key", format!("{key}: next sibling is another directive"), location);
                }
                if let Some(key) = directive(&expression.value) {
                    // MDX flow expressions can interrupt an unmatched multiline code
                    // paragraph. Require an explicit blank boundary after any sibling;
                    // do not accidentally consume its apparent "code" as metadata.
                    let separated = index == 0
                        || nodes[index - 1]
                            .position()
                            .zip(node.position())
                            .is_some_and(|(previous, current)| current.start.line > previous.end.line + 1);
                    if !separated {
                        self.error(
                            "misplaced-key",
                            "insert a blank line before an identity comment after another block",
                            span(node),
                        );
                    } else if block_container {
                        pending = Some((key.into(), span(node)));
                    } else {
                        self.error("misplaced-key", "annotation requires a block container", span(node));
                    }
                } else {
                    self.error(
                        "malformed-key",
                        "expected {/* springleaf:key NAME */}; NAME=[A-Za-z][A-Za-z0-9_.-]*",
                        span(node),
                    );
                }
                continue;
            }
            let mut projected = self.project(node);
            if let Some((key, location)) = pending.take() {
                if matches!(
                    projected.kind,
                    Kind::Paragraph | Kind::Heading | Kind::Code | Kind::List | Kind::Blockquote | Kind::ThematicBreak
                ) || matches!(projected.details, Details::Component { flow: true, .. })
                {
                    if projected.key.is_some() {
                        self.error("key-collision", "comment and component attribute both key the target", location);
                        self.error("key-collision", "component key collides with preceding comment", projected.span);
                        projected.key = None;
                        projected.key_origin = None;
                        projected.key_span = None;
                    } else {
                        projected.key = Some(key);
                        projected.key_span = location;
                        projected.key_origin = Some(KeyOrigin::Comment);
                    }
                } else {
                    self.error("unbound-key", format!("{key}: immediate next sibling is not a supported block"), location);
                }
            }
            output.push(projected);
        }
        if let Some((key, location)) = pending {
            self.error("unbound-key", format!("{key}: end of this container"), location);
        }
        output
    }

    fn project(&mut self, node: &Node) -> Projected {
        let mut result = Projected {
            kind: Kind::Rejected,
            span: span(node),
            key: None,
            key_origin: None,
            key_span: None,
            text: None,
            details: Details::None,
            children: Vec::new(),
        };
        result.kind = match node {
            Node::Root(_) => Kind::Root,
            Node::Paragraph(_) => Kind::Paragraph,
            Node::Heading(h) => {
                result.details = Details::Heading { depth: h.depth };
                Kind::Heading
            }
            Node::Blockquote(_) => Kind::Blockquote,
            Node::List(l) => {
                result.details = Details::List {
                    ordered: l.ordered,
                    start: l.start,
                    spread: l.spread,
                };
                Kind::List
            }
            Node::ListItem(l) => {
                result.details = Details::ListItem {
                    spread: l.spread,
                    checked: l.checked,
                };
                Kind::ListItem
            }
            Node::Text(t) => {
                result.text = Some(t.value.clone());
                Kind::Text
            }
            Node::Strong(_) => Kind::Strong,
            Node::Emphasis(_) => Kind::Emphasis,
            Node::Link(l) => {
                result.details = Details::Resource {
                    url: l.url.clone(),
                    title: l.title.clone(),
                    alt: None,
                };
                Kind::Link
            }
            Node::Image(i) => {
                result.details = Details::Resource {
                    url: i.url.clone(),
                    title: i.title.clone(),
                    alt: Some(i.alt.clone()),
                };
                Kind::Image
            }
            Node::LinkReference(r) => {
                result.details = Details::Reference {
                    identifier: r.identifier.clone(),
                    label: r.label.clone(),
                    reference_kind: format!("{:?}", r.reference_kind),
                    alt: None,
                };
                Kind::LinkReference
            }
            Node::ImageReference(r) => {
                result.details = Details::Reference {
                    identifier: r.identifier.clone(),
                    label: r.label.clone(),
                    reference_kind: format!("{:?}", r.reference_kind),
                    alt: Some(r.alt.clone()),
                };
                Kind::ImageReference
            }
            Node::Definition(d) => {
                result.details = Details::Definition {
                    identifier: d.identifier.clone(),
                    label: d.label.clone(),
                    url: d.url.clone(),
                    title: d.title.clone(),
                };
                Kind::Definition
            }
            Node::InlineCode(c) => {
                result.text = Some(c.value.clone());
                Kind::InlineCode
            }
            Node::Code(c) => {
                result.text = Some(c.value.clone());
                result.details = Details::Code {
                    lang: c.lang.clone(),
                    meta: c.meta.clone(),
                };
                Kind::Code
            }
            Node::Break(_) => Kind::Break,
            Node::ThematicBreak(_) => Kind::ThematicBreak,
            Node::MdxJsxFlowElement(e) => {
                self.component(&mut result, e.name.as_deref(), &e.attributes, true);
                Kind::Component
            }
            Node::MdxJsxTextElement(e) => {
                self.component(&mut result, e.name.as_deref(), &e.attributes, false);
                Kind::Component
            }
            Node::MdxFlowExpression(e) => self.expression(&e.value, &mut result),
            Node::MdxTextExpression(e) => self.expression(&e.value, &mut result),
            Node::MdxjsEsm(e) => {
                result.text = Some(e.value.clone());
                self.error("executable-esm", "imports and exports are outside the storage dialect", result.span);
                Kind::Rejected
            }
            _ => {
                result.text = Some(node.to_string());
                self.error("unsupported-node", "node is outside this probe's Markdown subset", result.span);
                Kind::Rejected
            }
        };
        if let Some(children) = node.children() {
            // In JSX-text mode, children are phrasing; flow components contain blocks.
            let block_container = matches!(node, Node::Root(_) | Node::Blockquote(_) | Node::ListItem(_) | Node::MdxJsxFlowElement(_));
            result.children = self.children(children, block_container);
        }
        result
    }

    fn expression(&mut self, value: &str, result: &mut Projected) -> Kind {
        result.text = Some(value.into());
        if reserved(value) {
            self.error("misplaced-key", "identity comment must be a standalone flow expression", result.span);
            Kind::Rejected
        } else if comment_only(value) {
            Kind::Comment
        } else {
            self.error(
                "executable-expression",
                "body expressions (including {#id}) are not storage data; use a preceding identity comment",
                result.span,
            );
            Kind::Rejected
        }
    }

    fn component(&mut self, result: &mut Projected, name: Option<&str>, attributes: &[AttributeContent], flow: bool) {
        // Recognize, preserve, and diagnose unknown names rather than silently dropping them.
        if !matches!(name, Some("Poll" | "Canvas" | "Place" | "Image" | "Run" | "HardBreak")) {
            self.error("unknown-component", format!("unregistered component: {name:?}"), result.span);
        }
        let mut properties = Vec::new();
        let mut names = BTreeSet::new();
        for attribute in attributes {
            let AttributeContent::Property(property) = attribute else {
                self.error("spread-attribute", "spread attributes are not storage data", result.span);
                continue;
            };
            if !names.insert(property.name.clone()) {
                self.error("duplicate-attribute", format!("duplicate property: {}", property.name), result.span);
            }
            let (value, raw_expression) = match &property.value {
                None => (Some(Value::Bool(true)), None),
                Some(AttributeValue::Literal(value)) => (Some(Value::String(value.clone())), None),
                Some(AttributeValue::Expression(expression)) => {
                    let value = match literal::parse(&expression.value) {
                        Ok(value) => Some(value),
                        Err(error) => {
                            self.error("nonliteral-attribute", format!("{} requires strict JSON: {error}", property.name), result.span);
                            None
                        }
                    };
                    (value, Some(expression.value.clone()))
                }
            };
            if property.name == "key" {
                if let Some(Value::String(key)) = &value
                    && valid_key(key)
                {
                    result.key = Some(key.clone());
                    result.key_origin = Some(KeyOrigin::ComponentAttribute);
                } else {
                    self.error("malformed-key", "component key must be a string matching [A-Za-z][A-Za-z0-9_.-]*", result.span);
                }
            }
            let expression_stops = match &property.value {
                Some(AttributeValue::Expression(e)) => e.stops.clone(),
                _ => Vec::new(),
            };
            properties.push(Property {
                name: property.name.clone(),
                value,
                raw_expression,
                expression_stops,
                span: None,
            });
        }
        result.details = Details::Component {
            name: name.map(Into::into),
            flow,
            attributes: properties,
        };
    }
}
