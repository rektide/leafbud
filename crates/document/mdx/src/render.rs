//! Checked Markdown-first rendering for format 2: implicit main-page blocks,
//! explicit named page definitions, one Key marker per block. Every emitted
//! body is proven through the production parser before it is returned.
use super::keys::TidyEdit;
use super::{json, parse, yaml};
use anyhow::{Context, Result, ensure};
use leafbud_document::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;

// ============ Markdown-first inline rendering ============
//
// Ordinary links/emphasis/code are emitted as Markdown and proven through
// the production parser per block; ineligible runs stay exact `<Run>` atoms,
// and a block whose candidate does not validate keeps its literal typed
// wrapper. See design/markdown-roundtrip0.gpt6a.md.

/// Why one run stayed an exact typed atom (tidy diagnostics).
pub(crate) const ATOM_UNSUPPORTED_MARK: &str = "unsupported-mark";
const ATOM_EDGE_OR_CONTROL_TEXT: &str = "whitespace-or-control-text";
const ATOM_AUTOLINK_TEXT: &str = "autolink-text";

/// True when every mark can be spelled as Markdown wrappers: no
/// Underline/Highlight, and at most one Link per run.
fn wrapper_eligible(marks: &[Mark]) -> bool {
    let mut links = 0;
    marks.iter().all(|mark| match mark {
        Mark::Strong | Mark::Em | Mark::Strikethrough | Mark::Code => true,
        Mark::Link { .. } => {
            links += 1;
            links <= 1
        }
        Mark::Underline | Mark::Highlight { .. } => false,
    }) && links <= 1
}

/// Wrapper order for one run: marks shared with the currently open stack
/// keep their stable positions (no delimiter churn at crossing runs); new
/// marks open inside, links outermost among them. Nesting order is not
/// semantic, but Link-outermost matches the design's preferred spelling.
fn wrapper_order(open: &[Mark], marks: &[Mark]) -> Vec<Mark> {
    let mut wrappers: Vec<Mark> = open.iter().filter(|m| marks.contains(*m)).cloned().collect();
    if let Some(link) = marks.iter().find(|m| matches!(m, Mark::Link { .. }))
        && !wrappers.contains(link)
    {
        wrappers.push(link.clone());
    }
    for mark in [Mark::Strong, Mark::Em, Mark::Strikethrough] {
        if marks.contains(&mark) && !wrappers.contains(&mark) {
            wrappers.push(mark.clone());
        }
    }
    wrappers
}

/// Run text is Markdown-eligible when its bytes are representable; edge
/// whitespace is handled positionally (only line-leading and block-final
/// whitespace is stripped by the parser).
fn text_clean(text: &str) -> bool {
    !text.is_empty() && !text.contains(['\n', '\r', '\t']) && !text.chars().any(char::is_control)
}

/// True when no substring could be turned into an autolink-literal by the
/// GFM post-pass (which resolves escapes before scanning, so bare URL text
/// cannot be protected by escaping — only by staying inside a component).
fn autolink_free(text: &str) -> bool {
    let bytes = text.as_bytes();
    let lower = text.to_ascii_lowercase();
    if lower.contains("http://") || lower.contains("https://") || lower.contains("www.") {
        return false;
    }
    (0..bytes.len())
        .all(|i| !(bytes[i] == b'@' && i > 0 && i + 1 < bytes.len() && bytes[i - 1].is_ascii_alphanumeric() && bytes[i + 1].is_ascii_alphanumeric()))
}

/// A slice of one run: Markdown text or an exact atom.
enum Piece<'a> {
    Text(&'a str),
    Atom(&'a str),
}

/// Split an unmarked-text segment into autolink-free text pieces and
/// conservative URL/email atoms. Over-atomizing is safe; under-atomizing
/// would change semantics (and then fail validation).
fn split_autolink_free(text: &str) -> Vec<Piece<'_>> {
    let bytes = text.as_bytes();
    let mut pieces = vec![];
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        let trigger = {
            let rest = &text[i..];
            let lower = rest.to_ascii_lowercase();
            lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www.")
        } || (bytes[i] == b'@' && i > 0 && bytes[i - 1].is_ascii_alphanumeric() && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphanumeric());
        if !trigger {
            i += 1;
            continue;
        }
        let mut atom_start = i;
        if bytes[i] == b'@' {
            // Include the local part before the email marker.
            while atom_start > start && (bytes[atom_start - 1].is_ascii_alphanumeric() || matches!(bytes[atom_start - 1], b'.' | b'+' | b'-' | b'_')) {
                atom_start -= 1;
            }
        }
        let mut end = i;
        while end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b'<' {
            end += 1;
        }
        if atom_start > start {
            pieces.push(Piece::Text(&text[start..atom_start]));
        }
        pieces.push(Piece::Atom(&text[atom_start..end]));
        start = end;
        i = end;
    }
    if start < text.len() {
        pieces.push(Piece::Text(&text[start..]));
    }
    pieces
}

/// Escape one character of Markdown text in context; returns the escaped
/// spelling when the raw character would not stay literal.
fn escape_char(c: char, next: Option<char>, heading: bool, line_start: bool) -> String {
    match c {
        '\\' | '`' | '*' | '_' | '[' | ']' | '~' | '<' | '$' | '{' | '}' => format!("\\{c}"),
        '&' if next.is_some_and(|n| n.is_ascii_alphanumeric() || n == '#') => "\\&".into(),
        '!' if next == Some('[') => "\\!".into(),
        '#' if heading || line_start => "\\#".into(),
        '-' | '+' | '=' | '>' if line_start => format!("\\{c}"),
        _ => c.to_string(),
    }
}

/// Append escaped Markdown text. `line_start` tracks whether the next
/// character starts an emitted line (paragraph continuation, quote line);
/// digits that would form an ordered-list marker get their punctuation
/// escaped instead (backslash-digit is a literal backslash).
fn push_escaped_text(out: &mut String, text: &str, heading: bool, line_start: &mut bool) {
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let c = chars[index];
        let next = chars.get(index + 1).copied();
        if *line_start && c.is_ascii_digit() {
            // "12." / "1)" at a line start: escape the delimiter, not the
            // digit, and consume the whole digit run plus delimiter.
            let digits_end = index + chars[index..].iter().take_while(|c| c.is_ascii_digit()).count();
            if matches!(chars.get(digits_end), Some('.' | ')')) {
                for digit in &chars[index..digits_end] {
                    out.push(*digit);
                }
                out.push('\\');
                out.push(chars[digits_end]);
                index = digits_end + 1;
                *line_start = false;
                continue;
            }
        }
        let escape = escape_char(c, next, heading, *line_start);
        out.push_str(&escape);
        *line_start = false;
        index += 1;
    }
}

/// A bare or angle-bracket link destination; None keeps the run exact.
fn link_destination(href: &str) -> Option<String> {
    if href.chars().any(char::is_control) {
        return None;
    }
    let bare_safe = href
        .chars()
        .all(|c| !c.is_whitespace() && !matches!(c, '(' | ')' | '<' | '>' | '\\') && !c.is_control());
    if bare_safe {
        return Some(format!("({href})"));
    }
    if !href.contains(['<', '>']) {
        return Some(format!("(<{}>)", href.replace('\\', "\\\\")));
    }
    None
}

/// A backtick code span for clean-edged eligible text; the fence is longer
/// than any internal backtick run, so no boundary padding is ever needed.
fn code_span(text: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in text.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat(longest + 1);
    format!("{fence}{text}{fence}")
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RichContext {
    Paragraph,
    Heading,
    Quote,
    /// One list item line: `indent` spaces continue the item after a break.
    Item {
        indent: usize,
    },
}

/// One block's Markdown-first inline candidate.
struct Candidate<'a> {
    content: &'a [Inline],
    context: RichContext,
    newline: &'a str,
    out: String,
    open: Vec<Mark>,
    line_start: bool,
    reasons: Vec<&'static str>,
}

impl<'a> Candidate<'a> {
    fn new(content: &'a [Inline], context: RichContext, newline: &'a str) -> Self {
        Candidate {
            content,
            context,
            newline,
            out: String::new(),
            open: vec![],
            line_start: true,
            reasons: vec![],
        }
    }

    fn raw(&mut self, delimiter: &str) {
        self.out.push_str(delimiter);
        self.line_start = false;
    }

    fn open_mark(&mut self, mark: &Mark) {
        match mark {
            Mark::Link { .. } => self.raw("["),
            Mark::Strong => self.raw("**"),
            Mark::Em => self.raw("*"),
            Mark::Strikethrough => self.raw("~~"),
            _ => unreachable!("wrapper order excludes leaf marks"),
        }
        self.open.push(mark.clone());
    }

    fn close_mark(&mut self) {
        let Some(mark) = self.open.pop() else { return };
        match mark {
            Mark::Link { href } => {
                let destination = link_destination(&href).expect("validated before opening");
                self.raw(&format!("]{destination}"));
            }
            Mark::Strong => self.raw("**"),
            Mark::Em => self.raw("*"),
            Mark::Strikethrough => self.raw("~~"),
            _ => unreachable!("wrapper order excludes leaf marks"),
        }
    }

    fn close_to(&mut self, depth: usize) {
        while self.open.len() > depth {
            self.close_mark();
        }
    }

    /// Emit an exact typed component atom at the current wrapper depth.
    /// The atom carries the run's own marks, so wrapper coverage around it
    /// is preserved by the surrounding delimiters.
    fn component(&mut self, text: &str, marks: &[Mark]) {
        let _ = write!(self.out, "<Run text={{{}}} marks={{{}}} />", json(text), json(marks));
        self.line_start = false;
    }

    fn text(&mut self, text: &str) {
        let mut line_start = self.line_start;
        push_escaped_text(&mut self.out, text, self.context == RichContext::Heading, &mut line_start);
        self.line_start = line_start;
    }

    fn code(&mut self, text: &str) {
        self.out.push_str(&code_span(text));
        self.line_start = false;
    }

    fn hard_break(&mut self, trailing: bool) {
        self.close_to(0);
        if self.context == RichContext::Heading || trailing {
            // ATX headings cannot wrap, and a trailing backslash-newline at
            // the end of a paragraph stays literal text.
            self.out.push_str("<HardBreak />");
            self.line_start = false;
        } else {
            self.out.push('\\');
            self.out.push_str(self.newline);
            match self.context {
                RichContext::Quote => self.out.push_str("> "),
                RichContext::Item { indent } => {
                    self.out.push_str(&" ".repeat(indent));
                }
                _ => {}
            }
            self.line_start = true;
        }
    }

    /// One text piece: emit as Markdown when its whitespace positions are
    /// parser-stable, else as an exact component atom.
    fn text_piece(&mut self, text: &str, marks: &[Mark], ends_block: bool) {
        if !text_clean(text) || (self.line_start && text.starts_with(char::is_whitespace)) || (ends_block && text.ends_with(char::is_whitespace)) {
            self.reasons.push(ATOM_EDGE_OR_CONTROL_TEXT);
            self.close_to(0);
            self.component(text, marks);
            return;
        }
        self.text(text);
    }

    fn run(&mut self, text: &str, marks: &[Mark], remaining: &[Inline]) {
        let ends_block = remaining.is_empty();
        if !wrapper_eligible(marks) {
            self.reasons.push(ATOM_UNSUPPORTED_MARK);
            self.close_to(0);
            self.component(text, marks);
            return;
        }
        if marks.iter().any(|m| matches!(m, Mark::Link { href } if link_destination(href).is_none())) {
            self.reasons.push(ATOM_UNSUPPORTED_MARK);
            self.close_to(0);
            self.component(text, marks);
            return;
        }
        if !text_clean(text) {
            self.reasons.push(ATOM_EDGE_OR_CONTROL_TEXT);
            self.close_to(0);
            self.component(text, marks);
            return;
        }
        // Shared wrappers stay open in their current order; close the rest,
        // open the new suffix.
        let wrappers = wrapper_order(&self.open, marks);
        let common = wrappers.iter().zip(&self.open).take_while(|(a, b)| a == b).count();
        self.close_to(common);
        for mark in &wrappers[common..] {
            self.open_mark(mark);
        }
        if marks.contains(&Mark::Code) {
            self.code(text);
            return;
        }
        if wrappers.iter().any(|m| matches!(m, Mark::Link { .. })) {
            // Inside a link label autolinks cannot form, so label text is safe.
            self.text_piece(text, marks, ends_block);
            return;
        }
        if autolink_free(text) {
            self.text_piece(text, marks, ends_block);
            return;
        }
        // Plain text would autolink: keep only the URL-ish pieces exact,
        // inside the currently open delimiters so mark coverage is intact.
        self.reasons.push(ATOM_AUTOLINK_TEXT);
        let mut pieces = split_autolink_free(text).into_iter().peekable();
        while let Some(piece) = pieces.next() {
            let piece_ends_block = ends_block && pieces.peek().is_none();
            match piece {
                Piece::Text(slice) => self.text_piece(slice, marks, piece_ends_block),
                Piece::Atom(slice) => self.component(slice, marks),
            }
        }
    }

    fn render(mut self) -> (String, Vec<&'static str>) {
        if self.content.is_empty() {
            return ("<Run text={\"\"} />".into(), vec![]);
        }
        for (index, value) in self.content.iter().enumerate() {
            let remaining = &self.content[index + 1..];
            match value {
                Inline::Run { text, marks } => self.run(text, marks, remaining),
                Inline::HardBreak => self.hard_break(index + 1 == self.content.len()),
            }
        }
        self.close_to(0);
        (self.out, self.reasons)
    }
}

/// The Markdown-first candidate source for one rich block, or None when the
/// block kind keeps its exact component spelling.
pub(crate) fn markdown_block_candidate(block: &Block, newline: &str) -> Option<(String, Vec<&'static str>)> {
    match &block.kind {
        BlockKind::Paragraph { content } => Some(Candidate::new(content, RichContext::Paragraph, newline).render()),
        BlockKind::Heading { level, content } => {
            let (line, reasons) = Candidate::new(content, RichContext::Heading, newline).render();
            Some((format!("{} {}", "#".repeat((*level).into()), line), reasons))
        }
        BlockKind::Quote { content } => {
            let (line, reasons) = Candidate::new(content, RichContext::Quote, newline).render();
            Some((format!("> {line}"), reasons))
        }
        BlockKind::ListItem { style, content } => {
            let marker = match style {
                ListStyle::Bullet => "- ".to_owned(),
                // Normalized numbering: the native model sequences ordered
                // items itself and display numbering is not source identity.
                ListStyle::Ordered => "1. ".to_owned(),
            };
            let (line, reasons) = Candidate::new(content, RichContext::Item { indent: marker.len() }, newline).render();
            Some((format!("{marker}{line}"), reasons))
        }
        _ => None,
    }
}

/// Prove a block candidate through the production parser inside the real
/// container shape (implicit main page, Key markers) with a synthetic
/// single-block document. Node rows may be absent: new blocks are valid.
pub(crate) fn block_candidate_valid(document: &Document, page_key: &str, block: &Block, candidate: &str) -> bool {
    let mut nodes = BTreeMap::new();
    for key in [page_key, &block.key] {
        if let Some(node) = document.state.nodes.get(key) {
            nodes.insert(key.to_owned(), node.clone());
        }
    }
    let mut probe = document.clone();
    probe.pages = vec![Page {
        key: page_key.to_owned(),
        blocks: vec![block.clone()],
    }];
    probe.state.nodes = nodes;
    probe.state.main_page = Some(page_key.to_owned());
    let Ok(frontmatter) = render_frontmatter(&probe) else {
        return false;
    };
    let body = format!("\n<Key id={} />\n{candidate}\n", json(&block.key));
    parse(&format!("{frontmatter}{body}")).is_ok_and(|parsed| parsed.semantic_value() == probe.semantic_value() && parsed.state == probe.state)
}

/// The exact literal typed wrapper for one rich block (proven fallback).
fn literal_block(block: &Block) -> String {
    let (tag, content, head) = match &block.kind {
        BlockKind::Paragraph { content } => ("Paragraph", content, String::new()),
        BlockKind::Heading { content, level } => ("Heading", content, format!(" level={{{level}}}")),
        BlockKind::Quote { content } => ("Quote", content, String::new()),
        BlockKind::ListItem { content, style } => ("ListItem", content, format!(" style={{{}}}", json(style))),
        _ => unreachable!("literal wrappers exist for rich blocks only"),
    };
    let mut out = format!("<{tag}{head}>\n\n");
    for run in content {
        match run {
            Inline::Run { text, marks } => {
                let _ = writeln!(out, "<Run text={{{}}} marks={{{}}} />\n", json(text), json(marks));
            }
            Inline::HardBreak => out.push_str("<HardBreak />\n\n"),
        }
    }
    let _ = write!(out, "</{tag}>");
    out
}

/// One block's checked body line: the validated Markdown candidate, or the
/// exact literal wrapper when the candidate does not round-trip.
fn verified_block_line(document: &Document, page_key: &str, block: &Block) -> String {
    if let Some((candidate, _)) = markdown_block_candidate(block, "\n")
        && block_candidate_valid(document, page_key, block, &candidate)
    {
        return candidate;
    }
    match &block.kind {
        BlockKind::Paragraph { .. } | BlockKind::Heading { .. } | BlockKind::Quote { .. } | BlockKind::ListItem { .. } => literal_block(block),
        _ => unreachable!("non-rich blocks have no candidate"),
    }
}

pub(crate) fn render_frontmatter(document: &Document) -> Result<String> {
    let mut out = "---\n".to_owned();
    yaml(&serde_json::to_value(document)?, 0, &mut out);
    out.push_str("---\n");
    Ok(out)
}

/// Compose the body: the implicit main page's blocks first (each with its
/// Key marker), then the named page definitions. Each rich block is the
/// checked Markdown candidate (or its literal wrapper); structural blocks
/// keep exact component spelling.
pub(crate) fn render_body(document: &Document, literal_blocks: bool) -> String {
    // The blank line after the frontmatter fence is part of the body part,
    // so `frontmatter + body` reproduces the original single-writer format.
    let mut out = "\n".to_owned();
    if let Some(main) = document.pages.first() {
        for block in &main.blocks {
            let _ = writeln!(out, "<Key id={} />", json(&block.key));
            let line = block_line(document, main, block, literal_blocks);
            let _ = writeln!(out, "{line}");
            out.push('\n');
        }
    }
    for page in document.pages.iter().skip(1) {
        let _ = writeln!(out, "<Page name={} kind=\"linear\">\n", json(&page.key));
        for block in &page.blocks {
            let _ = writeln!(out, "<Key id={} />", json(&block.key));
            let line = block_line(document, page, block, literal_blocks);
            let _ = writeln!(out, "{line}");
            out.push('\n');
        }
        out.push_str("</Page>\n\n");
    }
    out
}

pub(crate) fn block_line(document: &Document, page: &Page, block: &Block, literal_blocks: bool) -> String {
    match &block.kind {
        BlockKind::Paragraph { .. } | BlockKind::Heading { .. } | BlockKind::Quote { .. } | BlockKind::ListItem { .. } => {
            if literal_blocks {
                literal_block(block)
            } else {
                verified_block_line(document, &page.key, block)
            }
        }
        // Exact trailing whitespace/newline semantics; no fence normalization.
        BlockKind::Code { language, text } => {
            format!("<Code language={{{}}} text={{{}}} />", json(language), json(text))
        }
        BlockKind::Rule => "***".into(),
        BlockKind::PageLink { page } => format!("<PageLink page={} />", json(page)),
        BlockKind::Image {
            src,
            fallback,
            width,
            height,
            alt,
        } => {
            format!(
                "<Image src={{{}}} fallback={{{}}} width={{{}}} height={{{}}} alt={{{}}} />",
                json(src),
                json(fallback),
                json(width),
                json(height),
                json(alt)
            )
        }
    }
}

/// Prefer Markdown-first blocks, prove the composed source through Satteri,
/// and fall back to the proven all-literal body when a full-file boundary
/// interaction defeats the individually validated candidates. If even the
/// exact spelling differs, fail: never write unverified output.
pub(crate) fn verified_body(document: &Document, frontmatter: &str) -> Result<String> {
    let body = render_body(document, false);
    let verified =
        |body: &str| parse(&format!("{frontmatter}{body}")).is_ok_and(|p| p.semantic_value() == document.semantic_value() && p.state == document.state);
    if verified(&body) {
        return Ok(body);
    }
    let literal = render_body(document, true);
    ensure!(verified(&literal), "MDX semantic round-trip would lose content/associations");
    Ok(literal)
}

pub fn render(document: &Document) -> Result<String> {
    let frontmatter = render_frontmatter(document)?;
    let body = verified_body(document, &frontmatter)?;
    Ok(frontmatter + &body)
}

pub(crate) fn metadata_value(document: &Document) -> Value {
    let mut metadata = json!({"title": document.title, "description": canonical_description(document.description.as_deref()),
        "tags": document.tags, "publication": document.publication.value(), "theme": document.theme});
    if let Some(map) = metadata.as_object_mut() {
        for name in EDITOR_OWNED_EXTRA {
            if let Some(value) = document.extra.get(*name) {
                map.insert((*name).to_owned(), value.clone());
            }
        }
    }
    metadata
}

// ============ Deliberate Markdown normalization (tidy) ============

/// A Markdown tidy plan: block-body edits in ascending source order plus
/// diagnostics. Frontmatter, comments outside replaced spans, Key markers
/// and Page boundaries are never touched.
#[derive(Clone, Debug, Default)]
pub struct TidyPlan {
    pub edits: Vec<super::keys::TidyEdit>,
    pub blocks_examined: usize,
    pub blocks_changed: usize,
    pub blocks_skipped: usize,
    /// (block key, reason) pairs: why a block kept its exact spelling.
    pub skip_reasons: Vec<(String, &'static str)>,
}

/// Plan source-span Markdown normalization for a managed file. Only rich
/// blocks whose Markdown-first candidate validates through the production
/// parser are rewritten; everything else is reported, never approximated.
/// Unparsable files (including body MDX comments/unknown components) are
/// refused whole rather than partially rewritten.
pub fn plan_markdown_tidy(source: &str) -> Result<TidyPlan> {
    let parsed = super::parse::parse_with_spans(source)?;
    let document = &parsed.document;
    let mut plan = TidyPlan::default();
    let mut index = 0;
    for page in &document.pages {
        for block in &page.blocks {
            let span = &parsed.spans[index];
            index += 1;
            let Some((candidate, reasons)) = markdown_block_candidate(block, "\n") else {
                continue;
            };
            plan.blocks_examined += 1;
            // Preserve the block's own line-ending convention in inserted
            // syntax, then validate exactly what would be spliced.
            let newline = if source[span.start..span.end].contains("\r\n") { "\r\n" } else { "\n" };
            let candidate = if newline == "\n" { candidate } else { candidate.replace('\n', newline) };
            if !block_candidate_valid(document, &page.key, block, &candidate) {
                plan.blocks_skipped += 1;
                plan.skip_reasons.push((block.key.clone(), "candidate-mismatch"));
                for reason in reasons {
                    plan.skip_reasons.push((block.key.clone(), reason));
                }
                continue;
            }
            if candidate.as_bytes() != &source.as_bytes()[span.start..span.end] {
                plan.blocks_changed += 1;
                plan.edits.push(TidyEdit {
                    start: span.start,
                    end: span.end,
                    text: candidate,
                });
            }
        }
    }
    plan.edits.sort_by_key(|edit| edit.start);
    ensure!(
        plan.edits.windows(2).all(|pair| pair[0].end <= pair[1].start),
        "internal error: overlapping tidy edits"
    );
    Ok(plan)
}

/// Apply a tidy plan: splice descending, then require the complete result
/// to parse back to exactly the same semantics, bindings and state. On any
/// mismatch the original source is returned untouched by failing.
pub fn apply_markdown_tidy(source: &str, plan: &TidyPlan) -> Result<String> {
    let expected = parse(source)?;
    let mut result = source.to_owned();
    for TidyEdit { start, end, text } in plan.edits.iter().rev() {
        result.replace_range(start..end, text);
    }
    let parsed = parse(&result).context("tidy validation")?;
    ensure!(
        parsed.semantic_value() == expected.semantic_value() && parsed.state == expected.state,
        "Markdown tidy would change content/associations"
    );
    Ok(result)
}
