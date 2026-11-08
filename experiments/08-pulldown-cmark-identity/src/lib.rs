//! An offline, event-layer author-identity experiment, not an MDX parser.

use std::{collections::BTreeMap, ops::Range};

use pulldown_cmark::{Event, Options, Parser, Tag, html};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Plain,
    Native,
    Attributes,
    Strict,
}

#[derive(Clone, Debug, Serialize)]
pub struct Record {
    pub event: Event<'static>,
    pub span: Range<usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Node {
    pub event: Event<'static>,
    /// Original upstream range, or explicitly labelled derived inline hull.
    pub span: Range<usize>,
    pub span_origin: &'static str,
    pub key: Option<String>,
    pub key_span: Option<Range<usize>>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn kind(&self) -> &'static str {
        match self.event {
            Event::Start(Tag::Paragraph) => "Paragraph",
            Event::Start(Tag::Heading { .. }) => "Heading",
            Event::Start(Tag::BlockQuote(_)) => "Quote",
            Event::Start(Tag::List(_)) => "List",
            Event::Start(Tag::Item) => "Item",
            Event::Start(Tag::CodeBlock(_)) => "CodeBlock",
            Event::Start(Tag::HtmlBlock) => "HtmlBlock",
            Event::Start(_) => "InlineContainer",
            _ => "Leaf",
        }
    }

    pub fn text(&self) -> String {
        match &self.event {
            Event::Text(s) | Event::Code(s) | Event::Html(s) | Event::InlineHtml(s) => s.to_string(),
            Event::SoftBreak => " ".into(),
            Event::HardBreak => "\n".into(),
            _ => self.children.iter().map(Node::text).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub span: Range<usize>,
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Parsed {
    pub original_source: String,
    /// Untouched parser evidence, including the actual native Start/End ranges.
    pub raw_events: Vec<Record>,
    pub blocks: Vec<Node>,
    pub diagnostics: Vec<Diagnostic>,
    pub html: String,
}

pub fn parse(source: &str, mode: Mode) -> Parsed {
    let options = if mode == Mode::Native {
        Options::ENABLE_HEADING_ATTRIBUTES
    } else {
        Options::empty()
    };
    let raw_events: Vec<_> = Parser::new_ext(source, options)
        .into_offset_iter()
        .map(|(event, span)| Record {
            event: event.into_static(),
            span,
        })
        .collect();
    let mut at = 0;
    let mut blocks = tree(&raw_events, &mut at);
    normalize_tight_paragraphs(&mut blocks);
    let mut diagnostics = Vec::new();
    if matches!(mode, Mode::Attributes | Mode::Strict) {
        transform(&mut blocks, source, 0, &mut diagnostics);
        reject_duplicates(&mut blocks, &mut diagnostics);
    }
    if mode == Mode::Strict {
        reject_html_like(&blocks, source, &mut diagnostics);
    }
    let mut events = Vec::new();
    emit(&blocks, &mut events);
    let mut rendered = String::new();
    html::push_html(&mut rendered, events.into_iter());
    Parsed {
        original_source: source.into(),
        raw_events,
        blocks,
        diagnostics,
        html: rendered,
    }
}

fn tree(records: &[Record], at: &mut usize) -> Vec<Node> {
    let mut nodes = Vec::new();
    while let Some(record) = records.get(*at) {
        *at += 1;
        if matches!(record.event, Event::End(_)) {
            break;
        }
        let children = if matches!(record.event, Event::Start(_)) {
            tree(records, at)
        } else {
            Vec::new()
        };
        nodes.push(Node {
            event: record.event.clone(),
            span: record.span.clone(),
            span_origin: "upstream-event",
            key: None,
            key_span: None,
            children,
        });
    }
    nodes
}

fn is_inline(node: &Node) -> bool {
    matches!(
        node.event,
        Event::Text(_)
            | Event::Code(_)
            | Event::InlineHtml(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Start(Tag::Emphasis | Tag::Strong | Tag::Link { .. } | Tag::Image { .. })
    )
}

/// The public iterator omits Paragraph tags in tight lists. Do NOT attach their
/// keys to Item just because Item is the closest emitted tag.
fn normalize_tight_paragraphs(nodes: &mut [Node]) {
    for node in nodes {
        normalize_tight_paragraphs(&mut node.children);
        if !matches!(node.event, Event::Start(Tag::Item)) {
            continue;
        }
        let mut output = Vec::new();
        let mut input = std::mem::take(&mut node.children).into_iter().peekable();
        while let Some(first) = input.next() {
            if !is_inline(&first) {
                output.push(first);
                continue;
            }
            let start = first.span.start;
            let mut children = vec![first];
            while input.peek().is_some_and(is_inline) {
                children.push(input.next().unwrap());
            }
            let end = children.last().unwrap().span.end;
            output.push(Node {
                event: Event::Start(Tag::Paragraph),
                span: start..end,
                span_origin: "derived-tight-inline-hull",
                key: None,
                key_span: None,
                children,
            });
        }
        node.children = output;
    }
}

fn emit(nodes: &[Node], out: &mut Vec<Event<'static>>) {
    for node in nodes {
        let synthetic = node.span_origin == "derived-tight-inline-hull";
        if !synthetic {
            out.push(node.event.clone());
        }
        emit(&node.children, out);
        if let Event::Start(tag) = &node.event
            && !synthetic
        {
            out.push(Event::End(tag.to_end()));
        }
    }
}

fn diagnostic(out: &mut Vec<Diagnostic>, code: &'static str, span: Range<usize>, message: &'static str) {
    out.push(Diagnostic { code, span, message });
}

fn escaped(source: &str, at: usize) -> bool {
    source.as_bytes()[..at].iter().rev().take_while(|&&b| b == b'\\').count() % 2 == 1
}

fn valid_key(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_alphabetic()) && bytes.all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}

fn key_value(raw: &str) -> Option<&str> {
    let value = raw.strip_prefix("{#")?.strip_suffix('}')?;
    valid_key(value).then_some(value)
}

fn attach(node: &mut Node, source: &str, span: Range<usize>, out: &mut Vec<Diagnostic>) -> bool {
    if let Some(key) = key_value(&source[span.clone()]) {
        node.key = Some(key.into());
        node.key_span = Some(span);
        true
    } else {
        diagnostic(
            out,
            "malformed-key",
            span,
            "Expected exactly {#ASCII-key}; key begins with a letter and contains letters, digits, _, . or -",
        );
        false
    }
}

fn transform(nodes: &mut [Node], source: &str, quote_depth: usize, out: &mut Vec<Diagnostic>) {
    for node in nodes {
        let depth = quote_depth + usize::from(matches!(node.event, Event::Start(Tag::BlockQuote(_))));
        match node.event {
            Event::Start(Tag::Paragraph) => paragraph_key(node, source, depth, out),
            Event::Start(Tag::Heading { .. }) => heading_key(node, source, out),
            // Raw HTML and code own their complete payload. Never scan within them.
            Event::Start(Tag::CodeBlock(_) | Tag::HtmlBlock) => continue,
            _ => {}
        }
        for child in &node.children {
            if let Event::Text(_) = child.event {
                let raw = &source[child.span.clone()];
                // Consecutive Text events are not guaranteed merged: the '*' in
                // this spelling is emitted separately by upstream. Only the
                // opening must be claimed by a direct text leaf, never code.
                for (offset, _) in raw.match_indices("{/") {
                    let start = child.span.start + offset;
                    if !escaped(source, start) && source[start..].starts_with("{/* leafspring:") {
                        diagnostic(
                            out,
                            "unsupported-comment",
                            start..start + "{/* leafspring:".len(),
                            "MDX comment identities are not implemented in this experiment; use an attribute or plain mode",
                        );
                    }
                }
            }
        }
        transform(&mut node.children, source, depth, out);
    }
}

fn heading_key(node: &mut Node, source: &str, out: &mut Vec<Diagnostic>) {
    let mut candidates = Vec::new();
    for (index, child) in node.children.iter().enumerate() {
        let Event::Text(text) = &child.event else { continue };
        let raw = &source[child.span.clone()];
        // Normalized entity/escape leaves are not source-equivalent plain text.
        if raw != text.as_ref() {
            continue;
        }
        for (offset, _) in raw.match_indices("{#") {
            if !escaped(source, child.span.start + offset) && (offset == 0 || raw.as_bytes()[offset - 1].is_ascii_whitespace()) {
                candidates.push((index, offset, child.span.start + offset));
            }
        }
    }
    let Some(&(index, offset, start)) = candidates.last() else { return };
    let end = node.children.last().unwrap().span.end;
    let span = start..start + source[start..end].trim_end().len();
    if candidates.len() > 1 {
        diagnostic(out, "key-collision", candidates[0].2..span.end, "Multiple heading suffix keys; none wins");
        return;
    }
    if attach(node, source, span, out) {
        node.children.truncate(index + 1);
        let last = &mut node.children[index];
        let raw = &source[last.span.clone()];
        let content = raw[..offset].trim_end();
        last.event = Event::Text(content.to_string().into());
        // Preserve upstream span: modified semantic text is NOT a new exact source range.
    }
}

fn column(text: &str) -> usize {
    text.chars().fold(0, |col, c| if c == '\t' { (col / 4 + 1) * 4 } else { col + 1 })
}

fn prefix_at(source: &str, at: usize) -> &str {
    let start = source[..at].rfind('\n').map_or(0, |i| i + 1);
    &source[start..at]
}

fn paragraph_key(node: &mut Node, source: &str, quote_depth: usize, out: &mut Vec<Diagnostic>) {
    let mut groups: Vec<Range<usize>> = std::iter::once(0..0).collect();
    for (i, child) in node.children.iter().enumerate() {
        if matches!(child.event, Event::SoftBreak | Event::HardBreak) {
            groups.last_mut().unwrap().end = i;
            groups.push(i + 1..i + 1);
        } else {
            groups.last_mut().unwrap().end = i + 1;
        }
    }
    let mut candidates = Vec::new();
    for (line, group) in groups.iter().enumerate() {
        let Some(first) = node.children.get(group.start).filter(|_| !group.is_empty()) else {
            continue;
        };
        let Event::Text(text) = &first.event else { continue };
        let raw = &source[first.span.clone()];
        if !raw.starts_with("{#") || raw != text.as_ref() || escaped(source, first.span.start) {
            continue;
        }
        let last = &node.children[group.end - 1];
        let end = first.span.start + source[first.span.start..last.span.end].trim_end().len();
        candidates.push((line, group.clone(), first.span.start..end));
    }
    if candidates.len() > 1 {
        for (_, _, span) in candidates {
            diagnostic(out, "key-collision", span, "Multiple paragraph marker lines; none wins");
        }
        return;
    }
    let Some((line, group, span)) = candidates.pop() else { return };
    let prefix = prefix_at(source, span.start);
    let explicit = prefix.chars().all(|c| matches!(c, ' ' | '\t' | '>'))
        && prefix.bytes().filter(|&b| b == b'>').count() == quote_depth
        && column(prefix) >= column(prefix_at(source, node.span.start));
    if line == 0 || !explicit {
        diagnostic(
            out,
            "unbound-key",
            span,
            "A paragraph marker needs preceding prose in the same paragraph and explicit container indentation",
        );
    } else if line != groups.len() - 1 {
        diagnostic(out, "misplaced-key", span, "A paragraph marker must be the final logical line");
    } else if attach(node, source, span, out) {
        node.children.truncate(group.start - 1); // remove the marker's preceding break too
    }
}

fn key_inventory(nodes: &[Node], out: &mut BTreeMap<String, Vec<Range<usize>>>) {
    for node in nodes {
        if let (Some(key), Some(span)) = (&node.key, &node.key_span) {
            out.entry(key.clone()).or_default().push(span.clone());
        }
        key_inventory(&node.children, out);
    }
}

fn reject_duplicates(nodes: &mut [Node], diagnostics: &mut Vec<Diagnostic>) {
    let mut inventory = BTreeMap::new();
    key_inventory(nodes, &mut inventory);
    fn clear(nodes: &mut [Node], duplicate: &str) {
        for node in nodes {
            if node.key.as_deref() == Some(duplicate) {
                node.key = None;
                node.key_span = None;
            }
            clear(&mut node.children, duplicate);
        }
    }
    for (key, spans) in inventory {
        if spans.len() > 1 {
            for span in spans {
                diagnostic(
                    diagnostics,
                    "duplicate-key",
                    span,
                    "Author keys are document-wide unique; all duplicates are invalidated",
                );
            }
            clear(nodes, &key);
        }
    }
}

/// A deliberately fail-closed control, NOT a component grammar. Reject raw HTML
/// and unescaped '<' in prose even if CommonMark falls back to ordinary text.
fn reject_html_like(nodes: &[Node], source: &str, out: &mut Vec<Diagnostic>) {
    for node in nodes {
        match node.event {
            Event::Start(Tag::CodeBlock(_)) | Event::Code(_) => continue,
            Event::Html(_) | Event::InlineHtml(_) => diagnostic(
                out,
                "unsupported-html",
                node.span.clone(),
                "Raw HTML/components require a separate literal-component grammar",
            ),
            Event::Text(_) => {
                let raw = &source[node.span.clone()];
                for (offset, _) in raw.match_indices('<') {
                    let start = node.span.start + offset;
                    if !escaped(source, start) {
                        diagnostic(
                            out,
                            "unsupported-angle",
                            start..start + 1,
                            "Strict control rejects every unescaped prose '<', including comparisons; it does not parse JSX or JSON props",
                        );
                    }
                }
            }
            _ => {}
        }
        reject_html_like(&node.children, source, out);
    }
}
