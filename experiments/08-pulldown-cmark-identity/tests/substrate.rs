use pulldown_cmark::{Event, Options, Parser, Tag, html};
use pulldown_identity_experiment::{Mode, parse};

#[test]
fn native_heading_attrs_render_anchors_but_leave_paragraphs_literal() {
    let result = parse("## H {#heading}\n\nProse.\n{#paragraph}\n", Mode::Native);
    assert_eq!(result.html, "<h2 id=\"heading\">H</h2>\n<p>Prose.\n{#paragraph}</p>\n");
    assert!(result.blocks.iter().all(|n| n.key.is_none()));
}

#[test]
fn native_last_id_wins_and_invalid_author_grammar_is_not_diagnosed() {
    let result = parse("## H {#first #123 .class custom=x}\n", Mode::Native);
    let Event::Start(Tag::Heading { id, classes, attrs, .. }) = &result.raw_events[0].event else {
        panic!()
    };
    assert_eq!(id.as_deref(), Some("123"));
    assert_eq!(classes[0].as_ref(), "class");
    assert_eq!(attrs[0].0.as_ref(), "custom");
    assert!(result.diagnostics.is_empty());
    assert!(parse("## H {#first #123}\n", Mode::Attributes).diagnostics.len() == 1);
}

#[test]
fn native_heading_escape_policy_differs_from_event_owned_dialect() {
    let source = "## H \\{#literal}\n";
    let native = parse(source, Mode::Native);
    assert!(native.html.contains("id=\"literal\""));
    let dialect = parse(source, Mode::Attributes);
    assert_eq!(dialect.html, parse(source, Mode::Plain).html);
    assert!(dialect.html.contains("{#literal}"));
    assert!(dialect.blocks[0].key.is_none());
}

#[test]
fn native_closing_hash_placement_is_not_the_same_dialect() {
    let before = parse("## H {#key} ##\n", Mode::Native);
    assert!(before.html.contains("{#key}"));
    let after = parse("## H ## {#key}\n", Mode::Native);
    assert_eq!(after.html, "<h2 id=\"key\">H</h2>\n");
    let dialect = parse("## H {#key} ##\n", Mode::Attributes);
    assert_eq!(dialect.html, "<h2>H</h2>\n");
}

#[test]
fn upstream_suppresses_tight_paragraph_tags_and_reuses_whole_tag_ranges() {
    let source = "## Heading\n\n- Tight\n- Items\n";
    let events: Vec<_> = Parser::new(source).into_offset_iter().collect();
    assert!(!events.iter().any(|(e, _)| matches!(e, Event::Start(Tag::Paragraph))));
    let start = &events[0].1;
    let end = events
        .iter()
        .find(|(e, _)| matches!(e, Event::End(pulldown_cmark::TagEnd::Heading(_))))
        .unwrap();
    assert_eq!(*start, end.1);
    assert_eq!(&source[start.clone()], "## Heading\n");
}

#[test]
fn projection_noop_html_is_upstream_html_not_a_source_roundtrip_claim() {
    for source in [
        "- *A*\n- B\n",
        "> Quote\n\n1. One\n\n2. Two\n",
        "Text &amp; \\*escaped\\*\n",
        "```rs\nlet x = 1;\n```\n",
    ] {
        let mut expected = String::new();
        html::push_html(&mut expected, Parser::new_ext(source, Options::empty()));
        assert_eq!(parse(source, Mode::Plain).html, expected);
        assert_eq!(parse(source, Mode::Plain).original_source, source);
    }
}

#[test]
fn jsx_json_props_are_prose_not_structured_components() {
    let source = "<Poll key=\"topic\" options={[\"Gardens\", \"Maps\"]} />\n";
    let result = parse(source, Mode::Plain);
    assert_eq!(result.blocks[0].kind(), "Paragraph");
    assert_eq!(result.blocks[0].text(), source.trim_end());
    assert!(result.html.contains("&lt;Poll"));
    assert!(result.raw_events.iter().any(|r| matches!(r.event, Event::Text(_)) && r.span.start == 0));
    assert!(!parse(source, Mode::Strict).diagnostics.is_empty());
}

#[test]
fn identity_only_mode_cannot_disambiguate_json_component_prose_but_strict_refuses_it() {
    let source = "<Poll options={[\"A\"]} />\n{#looks-like-prose}\n";
    let attributes = parse(source, Mode::Attributes);
    assert!(attributes.diagnostics.is_empty());
    assert_eq!(attributes.blocks[0].key.as_deref(), Some("looks-like-prose"));
    assert_eq!(attributes.blocks[0].kind(), "Paragraph");
    // Diagnostic-bearing output is inspectable partial data, NOT safe compiled output.
    let strict = parse(source, Mode::Strict);
    assert!(!strict.diagnostics.is_empty());
    assert_eq!(strict.blocks[0].key.as_deref(), Some("looks-like-prose"));
}

#[test]
fn html_shaped_component_and_inline_component_are_raw_html() {
    let source = "<Poll key=\"topic\" />\n\nText <Image src=\"x\" />.\n";
    let result = parse(source, Mode::Plain);
    assert_eq!(result.blocks[0].kind(), "HtmlBlock");
    assert!(result.raw_events.iter().any(|r| matches!(r.event, Event::InlineHtml(_))));
    // Unlike Rushdown's default renderer, pulldown's HTML renderer passes this through.
    assert!(result.html.contains("<Poll key=\"topic\" />"));
    assert!(parse(source, Mode::Strict).diagnostics.iter().all(|d| d.code == "unsupported-html"));
}

#[test]
fn strict_control_rejects_json_props_js_spreads_and_multiline_without_execution() {
    for source in [
        "<Poll options={[\"A\"]} />\n",
        "<Poll options={danger()} />\n",
        "<Poll {...props} />\n",
        "<Poll\n options={[\"A\"]}\n/>\n",
        "Text <Poll options={[\"A\"]} /> tail.\n",
    ] {
        let result = parse(source, Mode::Strict);
        assert!(!result.diagnostics.is_empty(), "{source:?}");
        assert_eq!(result.original_source, source);
    }
}

#[test]
fn strict_control_is_explicitly_overrestrictive_not_a_jsx_validator() {
    assert!(!parse("Compare a < b.\n", Mode::Strict).diagnostics.is_empty());
    assert!(
        parse("Escaped \\<Poll /> and `<Poll />`.\n\n```mdx\n<Poll options={run()} />\n```\n", Mode::Strict)
            .diagnostics
            .is_empty()
    );
    // This is CommonMark content. No JavaScript AST or expression validation exists.
    assert!(parse("{danger()}\n", Mode::Strict).diagnostics.is_empty());
}

#[test]
fn requested_mdx_comment_is_visible_baseline_and_explicitly_unsupported() {
    let source = "{/* springleaf:key introduction */}\n## Introduction\n";
    let plain = parse(source, Mode::Plain);
    assert!(plain.html.contains("{/* springleaf:key introduction */}"));
    assert_eq!(plain.blocks[0].kind(), "Paragraph");
    let attrs = parse(source, Mode::Attributes);
    assert_eq!(attrs.diagnostics.len(), 1);
    assert_eq!(attrs.diagnostics[0].code, "unsupported-comment");
    assert!(attrs.blocks.iter().all(|n| n.key.is_none()));
}

#[test]
fn ordinary_html_comment_is_raw_html_not_an_identity_directive() {
    let source = "<!-- springleaf:key introduction -->\n## Introduction\n";
    let result = parse(source, Mode::Attributes);
    assert_eq!(result.blocks[0].kind(), "HtmlBlock");
    assert!(result.blocks.iter().all(|n| n.key.is_none()));
    assert!(result.diagnostics.is_empty());
}

#[test]
fn native_text_event_ranges_do_not_always_equal_semantic_text() {
    let source = "é &amp; `a\n b` \\*literal\\*\r\n";
    let result = parse(source, Mode::Plain);
    assert!(
        result
            .raw_events
            .iter()
            .any(|r| matches!(&r.event, Event::Text(t) if t.as_ref() == "&" && &source[r.span.clone()] == "&amp;"))
    );
    assert!(
        result
            .raw_events
            .iter()
            .any(|r| matches!(&r.event, Event::Code(t) if t.as_ref() == "a  b" && source[r.span.clone()].contains('`')))
    );
    for record in result.raw_events {
        assert!(source.get(record.span).is_some());
    }
}
