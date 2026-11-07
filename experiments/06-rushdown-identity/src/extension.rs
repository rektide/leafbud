use crate::{Assignment, Diagnostic, Mode, Span, State};
use rushdown::{as_extension_data, ast::*, parser::*, text, text::Reader as _};
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    HeadingSuffix,
    ParagraphTail,
    OrphanTail,
}

/// Temporary real AST nodes, not a preprocessing substitution.
#[derive(Debug, Clone)]
struct Marker {
    placement: Placement,
    key: Result<String, String>,
    span: Span,
}

impl NodeKind for Marker {
    fn typ(&self) -> NodeType {
        if self.placement != Placement::OrphanTail {
            NodeType::Inline
        } else {
            NodeType::LeafBlock
        }
    }

    fn kind_name(&self) -> &'static str {
        "AuthorKeyMarker"
    }
}

impl PrettyPrint for Marker {
    fn pretty_print(&self, w: &mut dyn fmt::Write, _: &str, _: usize) -> fmt::Result {
        write!(w, "{self:?}")
    }
}

impl From<Marker> for KindData {
    fn from(marker: Marker) -> Self {
        Self::Extension(Box::new(marker))
    }
}

fn valid_key(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes.next().is_some_and(|c| c.is_ascii_alphabetic()) && bytes.all(|c| c.is_ascii_alphanumeric() || b"_-.".contains(&c))
}

fn attribute(text: &str) -> Result<String, String> {
    let key = text.strip_prefix("{#").and_then(|s| s.strip_suffix('}')).filter(|s| valid_key(s));
    key.map(str::to_owned)
        .ok_or_else(|| "expected {#NAME}, NAME matching [A-Za-z][A-Za-z0-9_.-]*".into())
}

#[derive(Debug)]
struct TailParser;

impl BlockParser for TailParser {
    fn trigger(&self) -> &[u8] {
        b"{"
    }

    fn open(&self, arena: &mut Arena, _: NodeRef, reader: &mut text::BasicReader, ctx: &mut Context) -> Option<(NodeRef, StateFlags)> {
        let (line, segment) = reader.peek_line_bytes()?;
        let offset = ctx.block_offset()?;
        let text = std::str::from_utf8(&line[offset..]).ok()?.trim_end();
        if !text.starts_with("{#") {
            return None;
        }
        let start = segment.start() + offset - segment.padding();
        let marker = Marker {
            placement: Placement::OrphanTail,
            key: attribute(text),
            span: Span {
                start,
                end: start + text.len(),
            },
        };
        let node = arena.new_node(marker);
        reader.advance_to_eol();
        Some((node, StateFlags::NO_CHILDREN))
    }

    fn cont(&self, _: &mut Arena, _: NodeRef, _: &mut text::BasicReader, _: &mut Context) -> Option<StateFlags> {
        None
    }

    fn can_interrupt_paragraph(&self) -> bool {
        // Interruption here would steal lines inside multiline inline code.
        false
    }
}

// Avoid confusion between our diagnostic state and Rushdown's parser flags.
use rushdown::parser::State as StateFlags;

impl From<TailParser> for AnyBlockParser {
    fn from(parser: TailParser) -> Self {
        Self::Extension(Box::new(parser))
    }
}

#[derive(Debug)]
struct SuffixParser;

impl InlineParser for SuffixParser {
    fn trigger(&self) -> &[u8] {
        b"{"
    }

    fn parse(&self, arena: &mut Arena, parent: NodeRef, reader: &mut text::BlockReader, _: &mut Context) -> Option<NodeRef> {
        let placement = match arena[parent].kind_data() {
            KindData::Heading(_) => Placement::HeadingSuffix,
            KindData::Paragraph(_) if reader.line_offset() == 0 => Placement::ParagraphTail,
            _ => return None,
        };
        let (line, segment) = reader.peek_line_bytes()?;
        let text = std::str::from_utf8(&line).ok()?.trim_end();
        if !text.starts_with("{#") {
            return None;
        }
        // Suffix requires separating whitespace, not `word{#key}`.
        let start = segment.start();
        if placement == Placement::HeadingSuffix && (start == 0 || !reader.source().as_bytes()[start - 1].is_ascii_whitespace()) {
            return None;
        }
        // Only the final logical heading line may carry the suffix (setext too).
        let (saved_line, saved_position) = reader.position();
        reader.advance_line();
        let last_line = reader.peek_line_segment().is_none();
        reader.set_position(saved_line, saved_position);
        let key = if !last_line {
            Err("attribute must be on the final logical line of its block".into())
        } else if !explicit_container(arena, parent, start, reader.source()) {
            Err("annotation needs explicit quote markers and list indentation; lazy continuation is not allowed".into())
        } else {
            attribute(text)
        };
        let node = arena.new_node(Marker {
            placement,
            key,
            span: Span {
                start,
                end: start + text.len(),
            },
        });
        reader.advance(text.len());
        Some(node)
    }
}

impl From<SuffixParser> for AnyInlineParser {
    fn from(parser: SuffixParser) -> Self {
        Self::Extension(Box::new(parser))
    }
}

fn physical_prefix(source: &str, pos: usize) -> &str {
    let line = source[..pos].rfind('\n').map_or(0, |n| n + 1);
    &source[line..pos]
}

fn columns(prefix: &str) -> usize {
    prefix
        .bytes()
        .fold(0, |column, byte| if byte == b'\t' { column + 4 - column % 4 } else { column + 1 })
}

/// Deliberately strict authoring rule on top of CommonMark's lazy continuations.
fn explicit_container(arena: &Arena, block: NodeRef, pos: usize, source: &str) -> bool {
    let prefix = physical_prefix(source, pos);
    let mut quotes = 0;
    let mut in_item = false;
    let mut parent = arena[block].parent();
    while let Some(node) = parent {
        match arena[node].kind_data() {
            KindData::Blockquote(_) => quotes += 1,
            KindData::ListItem(_) => in_item = true,
            _ => {}
        }
        parent = arena[node].parent();
    }
    prefix.bytes().filter(|&b| b == b'>').count() >= quotes
        && (!in_item
            || arena[block]
                .pos()
                .is_some_and(|start| columns(prefix) >= columns(physical_prefix(source, start))))
}

#[derive(Debug)]
struct BindKeys(Rc<RefCell<State>>);

fn collect(arena: &Arena, node: NodeRef, nodes: &mut Vec<NodeRef>) {
    nodes.push(node);
    for child in arena[node].children(arena) {
        collect(arena, child, nodes);
    }
}

fn marker(arena: &Arena, node: NodeRef) -> Option<Marker> {
    match arena[node].kind_data() {
        KindData::Extension(data) if (data.as_ref() as &dyn std::any::Any).is::<Marker>() => Some(as_extension_data!(arena, node, Marker).clone()),
        _ => None,
    }
}

fn diagnostic(state: &mut State, code: &str, message: impl Into<String>, span: Span) {
    state.diagnostics.push(Diagnostic {
        code: code.into(),
        message: message.into(),
        span,
    });
}

impl AstTransformer for BindKeys {
    fn transform(&self, arena: &mut Arena, root: NodeRef, reader: &mut text::BasicReader, _: &mut Context) {
        let mut nodes = Vec::new();
        collect(arena, root, &mut nodes);
        let mut state = self.0.borrow_mut();
        let mut candidates = Vec::new();
        // Resolve against the intact sibling graph; remove markers only afterwards.
        for &node in &nodes {
            let Some(mark) = marker(arena, node) else { continue };
            let key = match &mark.key {
                Ok(key) => key.clone(),
                Err(message) => {
                    diagnostic(&mut state, "malformed-key", message, mark.span);
                    continue;
                }
            };
            let target = match mark.placement {
                Placement::HeadingSuffix => arena[node].parent().filter(|&parent| matches!(arena[parent].kind_data(), KindData::Heading(_))),
                Placement::ParagraphTail => arena[node]
                    .parent()
                    .filter(|&parent| matches!(arena[parent].kind_data(), KindData::Paragraph(_))),
                Placement::OrphanTail => None,
            };
            if let Some(target) = target {
                candidates.push(Assignment { target, key, span: mark.span });
            } else {
                diagnostic(
                    &mut state,
                    "unbound-key",
                    "trailing attribute requires the immediately preceding paragraph in the same container, without a blank line",
                    mark.span,
                );
            }
        }
        // Source order makes diagnostic and collision behavior deterministic.
        candidates.sort_by_key(|a| a.span.start);
        let mut key_counts = HashMap::new();
        for assignment in &candidates {
            *key_counts.entry(assignment.key.clone()).or_insert(0) += 1;
        }
        for assignment in &candidates {
            if candidates.iter().filter(|a| a.target == assignment.target).count() > 1 {
                diagnostic(
                    &mut state,
                    "key-collision",
                    "more than one key on the same block; no precedence",
                    assignment.span,
                );
            } else if key_counts[&assignment.key] > 1 {
                diagnostic(
                    &mut state,
                    "duplicate-key",
                    format!("duplicate document-wide author key {:?}", assignment.key),
                    assignment.span,
                );
            } else {
                // Internal author metadata, deliberately NOT the HTML `id` attribute.
                arena[assignment.target]
                    .attributes_mut()
                    .insert("springleaf-key", assignment.key.clone().into());
                state.assignments.push(assignment.clone());
            }
        }
        for node in nodes {
            if let Some(mark) = marker(arena, node) {
                if mark.placement != Placement::OrphanTail
                    && let Some(previous) = arena[node].previous_sibling()
                    && let KindData::Text(text) = arena[previous].kind_data_mut()
                {
                    let value = text.str(reader.source()).trim_end().to_owned();
                    let qualifiers = if text.has_qualifiers(TextQualifier::HARD_LINE_BREAK) {
                        TextQualifier::HARD_LINE_BREAK
                    } else {
                        TextQualifier::empty()
                    };
                    *text = Text::with_qualifiers(value, qualifiers);
                }
                node.delete(arena);
            }
        }
        state.diagnostics.sort_by_key(|d| d.span.start);
    }
}

impl From<BindKeys> for AnyAstTransformer {
    fn from(transformer: BindKeys) -> Self {
        Self::Extension(Box::new(transformer))
    }
}

pub(crate) fn identity(mode: Mode, state: Rc<RefCell<State>>) -> impl ParserExtension {
    parser_extension(move |parser| {
        if mode == Mode::Attributes {
            parser.add_block_parser(|| TailParser, NoParserOptions, PRIORITY_PARAGRAPH - 10);
            parser.add_inline_parser(|| SuffixParser, NoParserOptions, PRIORITY_EMPHASIS + 10);
            parser.add_ast_transformer(move || BindKeys(state), NoParserOptions, 100);
        }
    })
}
