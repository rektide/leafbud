//! Small source-derived controls: distinguish Rushdown behavior from our dialect.
use rushdown::{ast::*, parser::*, renderer::html, text, text::Reader as _};
use rushdown_identity_experiment::{Block, Mode, parse};

fn upstream_html(source: &str, options: Options, extension: impl ParserExtension) -> String {
    let (arena, root) = Parser::with_extensions(options, extension).parse(&mut text::BasicReader::new(source));
    let mut output = String::new();
    html::Renderer::with_options(html::Options::default())
        .render(&mut output, source, &arena, root)
        .unwrap();
    output
}

#[test]
fn builtin_heading_attributes_are_html_attributes_not_author_keys() {
    let source = "## Introduction {#introduction}\n\nParagraph.\n{#body}\n";
    let html = upstream_html(
        source,
        Options {
            attributes: true,
            ..Default::default()
        },
        NO_EXTENSIONS,
    );
    assert_eq!(html, "<h2 id=\"introduction\">Introduction</h2>\n<p>Paragraph.\n{#body}</p>\n");
    let custom = parse(source, Mode::Attributes);
    assert_eq!(custom.blocks[0].key.as_deref(), Some("introduction"));
    assert_eq!(custom.blocks[1].key.as_deref(), Some("body"));
    assert!(!custom.html.contains("id="));
}

/// Intentionally WRONG architecture retained only as a negative control.
/// A block parser interrupting prose has no inline-code delimiter context.
#[derive(Debug)]
struct EagerInterrupt;

impl BlockParser for EagerInterrupt {
    fn trigger(&self) -> &[u8] {
        b"{"
    }
    fn open(&self, arena: &mut Arena, _: NodeRef, reader: &mut text::BasicReader, _: &mut Context) -> Option<(NodeRef, State)> {
        let (line, _) = reader.peek_line()?;
        if !line.trim().starts_with("{#") {
            return None;
        }
        reader.advance_to_eol();
        Some((arena.new_node(ThematicBreak::new()), State::NO_CHILDREN))
    }
    fn cont(&self, _: &mut Arena, _: NodeRef, _: &mut text::BasicReader, _: &mut Context) -> Option<State> {
        None
    }
    fn can_interrupt_paragraph(&self) -> bool {
        true
    }
}

impl From<EagerInterrupt> for AnyBlockParser {
    fn from(parser: EagerInterrupt) -> Self {
        Self::Extension(Box::new(parser))
    }
}

#[test]
fn eager_block_interruption_really_destroys_multiline_inline_code() {
    let source = "Code `first\n{#literal}\nlast`.\n";
    let normal = upstream_html(source, Options::default(), NO_EXTENSIONS);
    let wrong = upstream_html(
        source,
        Options::default(),
        parser_extension(|parser| {
            parser.add_block_parser(|| EagerInterrupt, NoParserOptions, PRIORITY_PARAGRAPH - 10);
        }),
    );
    assert!(normal.contains("<code>first {#literal} last</code>"));
    assert!(wrong.contains("<hr>"));
    assert!(!wrong.contains("<code>"));
    assert_eq!(parse(source, Mode::Both).html, normal);
}

#[test]
fn component_html_and_expressions_are_not_an_mdx_implementation() {
    let source = include_str!("../fixtures/components.mdx");
    let parsed = parse(source, Mode::Both);
    assert!(parsed.diagnostics.is_empty());
    // JSON-quoted JSX properties fail CommonMark's HTML attribute grammar.
    assert_eq!(parsed.blocks[0].kind, "Paragraph");
    assert_eq!(parsed.blocks[0].content, "<Poll key=\"topic\" options={[\"A\", \"B\"]} />");
    assert_eq!(parsed.blocks[1].kind, "HtmlBlock");
    assert!(parsed.blocks[2].inlines.iter().any(|inline| inline.kind == "RawHtml"));
    assert!(parsed.blocks[2].content.contains("{expression}"));
    // The default HTML renderer suppresses the raw component; it does not run it.
    assert!(!parsed.html.contains("<Poll"));
    assert!(parsed.html.contains("&lt;Poll"));
}

#[test]
fn json_component_can_look_like_prose_and_is_not_classified_by_identity_extension() {
    let parsed = parse("{/* leafbud:key component-looking */}\n<Poll options={[\"A\"]} />\n", Mode::Comments);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.blocks[0].kind, "Paragraph");
    assert_eq!(parsed.blocks[0].key.as_deref(), Some("component-looking"));
    // A future literal-component compiler MUST classify/reject this before lowering prose.
}

#[test]
fn payload_segments_are_not_full_syntax_ranges() {
    let source = "## Head {#head}\n\n- Item.\n  {#item}\n\n```md\n{#literal}\n```\n";
    let parsed = parse(source, Mode::Attributes);
    assert!(parsed.diagnostics.is_empty());
    let heading = &parsed.blocks[0];
    let heading_segment = &heading.content_segments[0];
    assert_eq!(&source[heading_segment.start..heading_segment.end], "Head {#head}");
    assert_eq!(heading.start, Some(0));
    let list = &parsed.blocks[1];
    assert!(list.content_segments.is_empty());
    assert!(list.children[0].content_segments.is_empty());
    assert_eq!(parsed.blocks[2].content_segments.len(), 1);
    let code = &parsed.blocks[2].content_segments[0];
    assert_eq!(&source[code.start..code.end], "{#literal}\n");
    // Inline text has a source index even though delimiter containers may not.
    let text = heading.inlines.iter().find(|i| i.kind == "Text" && i.text.as_deref() == Some("Head")).unwrap();
    let span = text.text_span.unwrap();
    assert_eq!(&source[span.start..span.end], "Head");
}

#[test]
fn projection_intentionally_retains_raw_markdown_text_escapes_and_entities() {
    let parsed = parse("Escaped \\{brace} &amp; entity.\n{#body}\n", Mode::Attributes);
    assert_eq!(parsed.blocks[0].content, "Escaped \\{brace} &amp; entity.");
    assert!(parsed.html.contains("Escaped {brace} &amp; entity."));
    // This is an AST/identity proof, not yet a full semantic-text/facet compiler.
}

#[test]
fn annotation_shapes_inside_raw_html_remain_html_payload() {
    let source = "<div>\n{/* leafbud:key literal */}\n{#literal}\n</div>\n";
    let parsed = parse(source, Mode::Both);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.blocks.len(), 1);
    assert_eq!(parsed.blocks[0].kind, "HtmlBlock");
    assert_eq!(parsed.blocks[0].content, source);
}

#[test]
fn position_inventory_has_exact_marker_spans_but_no_invented_block_ends() {
    fn inventory(blocks: &[Block], counts: &mut [usize; 4]) {
        for block in blocks {
            counts[0] += 1;
            counts[1] += usize::from(block.start.is_some());
            counts[2] += usize::from(!block.content_segments.is_empty());
            counts[3] += usize::from(block.key_span.is_some());
            inventory(&block.children, counts);
        }
    }
    let source = include_str!("../fixtures/attributes.md");
    let parsed = parse(source, Mode::Attributes);
    let mut counts = [0; 4];
    inventory(&parsed.blocks, &mut counts);
    // All blocks have starts; only payload-bearing blocks have content segments.
    assert_eq!(counts, [13, 13, 9, 5]);
}
