use crate::{Assignment, Span};
use rushdown::{ast::*, text};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSegment {
    pub start: usize,
    pub end: usize,
    pub padding: usize,
    pub force_newline: bool,
}

impl From<&text::Segment> for SourceSegment {
    fn from(segment: &text::Segment) -> Self {
        Self {
            start: segment.start(),
            end: segment.stop(),
            padding: segment.padding(),
            force_newline: segment.force_newline(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Inline {
    pub kind: String,
    /// Rushdown text leaves retain Markdown escapes/entities; not decoded prose.
    pub text: Option<String>,
    pub destination: Option<String>,
    pub soft_break: bool,
    pub hard_break: bool,
    pub start: Option<usize>,
    /// Only exposed where Rushdown retains a Text index, not fabricated for containers.
    pub text_span: Option<Span>,
    pub children: Vec<Inline>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Block {
    pub kind: String,
    pub key: Option<String>,
    pub key_span: Option<Span>,
    pub start: Option<usize>,
    /// Parser payload segments, NOT full concrete-syntax block extents.
    pub content_segments: Vec<SourceSegment>,
    /// Inspection flattening of the inline tree (soft wrap -> space).
    /// Escapes/entities in raw Text leaves still need semantic decoding downstream.
    pub content: String,
    pub inlines: Vec<Inline>,
    pub children: Vec<Block>,
}

fn inline(arena: &Arena, node: NodeRef, source: &str) -> Inline {
    let data = arena[node].kind_data();
    let text = match data {
        KindData::Text(text) => Some(text.str(source).into()),
        KindData::CodeSpan(code) => Some(code.str(source).into_owned()),
        KindData::RawHtml(html) => Some(html.str(source).into_owned()),
        _ => None,
    };
    let destination = match data {
        KindData::Link(link) => Some(link.destination_str(source).into()),
        KindData::Image(image) => Some(image.destination_str(source).into()),
        _ => None,
    };
    Inline {
        kind: data.kind_name().into(),
        text,
        destination,
        soft_break: matches!(data, KindData::Text(t) if t.has_qualifiers(TextQualifier::SOFT_LINE_BREAK)),
        hard_break: matches!(data, KindData::Text(t) if t.has_qualifiers(TextQualifier::HARD_LINE_BREAK)),
        start: arena[node].pos(),
        text_span: match data {
            KindData::Text(text) => text.index().map(|i| Span {
                start: i.start(),
                end: i.stop(),
            }),
            _ => None,
        },
        children: arena[node].children(arena).map(|child| inline(arena, child, source)).collect(),
    }
}

fn plain(inlines: &[Inline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        if let Some(value) = &inline.text {
            text.push_str(value);
        }
        text.push_str(&plain(&inline.children));
        if inline.soft_break {
            text.push(' ');
        }
        if inline.hard_break {
            text.push('\n');
        }
    }
    text
}

pub(crate) fn block(arena: &Arena, node: NodeRef, source: &str, assignments: &[Assignment]) -> Block {
    let data = arena[node].kind_data();
    let assignment = assignments.iter().find(|a| a.target == node);
    let lines = match data {
        KindData::CodeBlock(code) => Some(code.value()),
        KindData::HtmlBlock(html) => Some(html.value()),
        _ => None,
    };
    let segments = match (lines, arena[node].type_data()) {
        (Some(text::Lines::Segments(segments)), _) => segments.iter().map(SourceSegment::from).collect(),
        (None, TypeData::Block(block)) => block.source().iter().map(SourceSegment::from).collect(),
        _ => Vec::new(),
    };
    let mut children = Vec::new();
    let mut inlines = Vec::new();
    for child in arena[node].children(arena) {
        if arena[child].kind_data().typ() == NodeType::Inline {
            inlines.push(inline(arena, child, source));
        } else {
            children.push(block(arena, child, source, assignments));
        }
    }
    let content = match data {
        KindData::CodeBlock(code) => code.value().iter(source).collect::<String>(),
        KindData::HtmlBlock(html) => html.value().iter(source).collect::<String>(),
        _ => plain(&inlines),
    };
    Block {
        kind: data.kind_name().into(),
        key: assignment.map(|a| a.key.clone()),
        key_span: assignment.map(|a| a.span),
        start: arena[node].pos(),
        content_segments: segments,
        content,
        inlines,
        children,
    }
}
