//! Comparison adapter, not a Leaflet compiler. All input passes through the actual fork.
//! Identity policy follows experiment 09; literal validation is ours, not a fork feature.
mod literal;

use markdown::mdast::{AttributeContent, AttributeValue, Node};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub fn span(node: &Node) -> Option<Span> {
    node.position().map(|p| Span {
        start: p.start.offset,
        end: p.end.offset,
    })
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub span: Option<Span>,
}

#[derive(Debug, Serialize)]
pub struct Binding {
    pub key: String,
    /// Index walk through the original ordered mdast, not a persistent identity.
    pub path: Vec<usize>,
    /// Complete comment span; None for component attributes, absent in upstream AST.
    pub annotation_span: Option<Span>,
}

#[derive(Debug, Serialize)]
pub struct Property {
    pub name: String,
    pub value: Option<Value>,
    pub raw_expression: Option<String>,
    pub expression_stops: Vec<markdown::mdast::Stop>,
    pub span: Option<Span>,
}

#[derive(Debug, Serialize)]
pub struct Component {
    pub path: Vec<usize>,
    pub name: Option<String>,
    pub flow: bool,
    pub span: Option<Span>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Serialize)]
pub struct Inspection<'a> {
    #[serde(skip)]
    pub original_source: &'a str,
    /// Unchanged typed AST: ordered block/inline/component children and semantic text.
    pub tree: Node,
    /// Ephemeral data views over tree, never a maintained sidecar.
    pub components: Vec<Component>,
    pub bindings: Vec<Binding>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn stock(source: &str) -> Result<Node, markdown::message::Message> {
    mdxjs::mdast_util_from_mdx(source, &mdxjs::Options::default())
}

pub fn inspect(source: &str) -> Result<Inspection<'_>, markdown::message::Message> {
    let tree = stock(source)?;
    let mut adapter = Adapter {
        components: vec![],
        bindings: vec![],
        diagnostics: vec![],
    };
    adapter.visit(&tree, &[]);
    let mut key_counts = BTreeMap::new();
    let mut target_counts = BTreeMap::new();
    for binding in &adapter.bindings {
        *key_counts.entry(binding.key.clone()).or_insert(0usize) += 1;
        *target_counts.entry(binding.path.clone()).or_insert(0usize) += 1;
    }
    adapter.bindings.retain(|b| {
        let code = if target_counts[&b.path] > 1 {
            Some("key-collision")
        } else if key_counts[&b.key] > 1 {
            Some("duplicate-key")
        } else {
            None
        };
        if let Some(code) = code {
            adapter.diagnostics.push(Diagnostic { code, span: b.annotation_span });
            false
        } else {
            true
        }
    });
    Ok(Inspection {
        original_source: source,
        tree,
        components: adapter.components,
        bindings: adapter.bindings,
        diagnostics: adapter.diagnostics,
    })
}

fn valid_key(key: &str) -> bool {
    key.as_bytes().first().is_some_and(u8::is_ascii_alphabetic) && key.bytes().all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}

fn reserved(value: &str) -> bool {
    value.trim().starts_with("/* springleaf:")
}

fn directive(value: &str) -> Option<&str> {
    let key = value.strip_prefix("/* springleaf:key ")?.strip_suffix(" */")?;
    valid_key(key).then_some(key)
}

fn comment_only(value: &str) -> bool {
    value
        .trim()
        .strip_prefix("/*")
        .and_then(|v| v.strip_suffix("*/"))
        .is_some_and(|v| !v.contains("*/"))
}

struct Adapter {
    components: Vec<Component>,
    bindings: Vec<Binding>,
    diagnostics: Vec<Diagnostic>,
}

impl Adapter {
    fn error(&mut self, code: &'static str, node: &Node) {
        self.diagnostics.push(Diagnostic { code, span: span(node) });
    }

    fn visit(&mut self, node: &Node, path: &[usize]) {
        match node {
            Node::MdxJsxFlowElement(e) => self.component(node, path, e.name.as_deref(), &e.attributes, true),
            Node::MdxJsxTextElement(e) => self.component(node, path, e.name.as_deref(), &e.attributes, false),
            Node::MdxFlowExpression(e) => self.expression(node, &e.value),
            Node::MdxTextExpression(e) => self.expression(node, &e.value),
            Node::MdxjsEsm(_) => self.error("executable-esm", node),
            Node::Root(_)
            | Node::Paragraph(_)
            | Node::Heading(_)
            | Node::Blockquote(_)
            | Node::List(_)
            | Node::ListItem(_)
            | Node::Text(_)
            | Node::Strong(_)
            | Node::Emphasis(_)
            | Node::Link(_)
            | Node::LinkReference(_)
            | Node::Definition(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::InlineCode(_)
            | Node::Code(_)
            | Node::Break(_)
            | Node::ThematicBreak(_) => {}
            _ => self.error("unsupported-node", node),
        }
        let Some(children) = node.children() else { return };
        let container = matches!(node, Node::Root(_) | Node::Blockquote(_) | Node::ListItem(_) | Node::MdxJsxFlowElement(_));
        let mut pending: Option<(String, &Node)> = None;
        for (index, child) in children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(index);
            if let Node::MdxFlowExpression(expression) = child
                && reserved(&expression.value)
            {
                if let Some((_, annotation)) = pending.take() {
                    self.error("unbound-key", annotation);
                }
                if let Some(key) = directive(&expression.value) {
                    // Same blank-before guard as 09. This is rejection, not code repair.
                    let separated = index == 0
                        || children[index - 1]
                            .position()
                            .zip(child.position())
                            .is_some_and(|(previous, current)| current.start.line > previous.end.line + 1);
                    if !container || !separated {
                        self.error("misplaced-key", child);
                    } else {
                        pending = Some((key.into(), child));
                    }
                } else {
                    self.error("malformed-key", child);
                }
                continue;
            }
            self.visit(child, &child_path);
            if let Some((key, annotation)) = pending.take() {
                if matches!(
                    child,
                    Node::Paragraph(_)
                        | Node::Heading(_)
                        | Node::Code(_)
                        | Node::List(_)
                        | Node::Blockquote(_)
                        | Node::ThematicBreak(_)
                        | Node::MdxJsxFlowElement(_)
                ) {
                    self.bindings.push(Binding {
                        key,
                        path: child_path,
                        annotation_span: span(annotation),
                    });
                } else {
                    self.error("unbound-key", annotation);
                }
            }
        }
        if let Some((_, annotation)) = pending {
            self.error("unbound-key", annotation);
        }
    }

    fn expression(&mut self, node: &Node, value: &str) {
        if reserved(value) {
            self.error("misplaced-key", node);
        } else if !comment_only(value) {
            self.error("executable-expression", node);
        }
    }

    fn component(&mut self, node: &Node, path: &[usize], name: Option<&str>, attributes: &[AttributeContent], flow: bool) {
        if !matches!(name, Some("Poll" | "Canvas" | "Page" | "PageLink" | "Place" | "Image" | "Run" | "HardBreak")) {
            self.error("unknown-component", node);
        }
        let mut properties = vec![];
        let mut names = BTreeSet::new();
        for attribute in attributes {
            let AttributeContent::Property(property) = attribute else {
                self.error("spread-attribute", node);
                continue;
            };
            if !names.insert(&property.name) {
                self.error("duplicate-attribute", node);
            }
            let (value, raw_expression, expression_stops) = match &property.value {
                None => (Some(Value::Bool(true)), None, vec![]),
                Some(AttributeValue::Literal(value)) => (Some(Value::String(value.clone())), None, vec![]),
                Some(AttributeValue::Expression(e)) => {
                    let value = literal::parse(&e.value).map_err(|_| self.error("nonliteral-attribute", node)).ok();
                    (value, Some(e.value.clone()), e.stops.clone())
                }
            };
            if property.name == "key" {
                if let Some(Value::String(key)) = &value
                    && valid_key(key)
                {
                    self.bindings.push(Binding {
                        key: key.clone(),
                        path: path.to_vec(),
                        annotation_span: None,
                    });
                } else {
                    self.error("malformed-key", node);
                }
            }
            properties.push(Property {
                name: property.name.clone(),
                value,
                raw_expression,
                expression_stops,
                span: None,
            });
        }
        self.components.push(Component {
            path: path.to_vec(),
            name: name.map(Into::into),
            flow,
            span: span(node),
            properties,
        });
    }
}
