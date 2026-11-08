use satteri_authoring_attributes_experiment::{
    compile,
    facts::{self, BuildContext},
    model::*,
    prepare, semantic_document, semantic_plan,
    simulator::Store,
    text,
};

const BRACES: &str = include_str!("../fixtures/garden-attributes.mdx");
const KEYS: &str = include_str!("../../13-satteri-authoring/fixtures/garden.mdx");
fn source(body: &str) -> String {
    format!("---\nformat: springleaf-mdx/1\ntitle: Test\n---\n\n{body}")
}
fn code(body: &str) -> &'static str {
    compile(&source(body)).unwrap_err()[0].code
}

#[test]
fn real_garden_fixture_matches_unchanged_native_key_compiler_ir_and_fact_plan() {
    let adapted = compile(BRACES).unwrap();
    let native = satteri_authoring_experiment::author::compile(KEYS).unwrap();
    assert_eq!(adapted.annotations.len(), 5);
    assert!(
        adapted.discovery_error_count > 0,
        "original expressions really errored before strict amended reparse"
    );
    assert_eq!(adapted.document.consumed_keys, 1, "code block keeps native Key");
    assert_eq!(semantic_document(&adapted.document), semantic_document(&native));
    let a = facts::build(&adapted.document, BuildContext::fixture(100)).unwrap();
    let b = facts::build(&native, BuildContext::fixture(100)).unwrap();
    assert_eq!(a.author_keys.keys().collect::<Vec<_>>(), b.author_keys.keys().collect::<Vec<_>>());
    assert_eq!(semantic_plan(&a).unwrap(), semantic_plan(&b).unwrap());
    assert_eq!((a.entities.len(), a.facts.len(), a.expected_text.len()), (17, 56, 9));
    for (entity, wanted) in &a.expected_text {
        let facts::Data::Text { value } = &a.facts.iter().find(|f| &f.entity == entity && f.attribute == "block/text").unwrap().data else {
            panic!()
        };
        assert_eq!(&text::read(value).unwrap(), wanted);
    }
    let mut store = Store::fixture();
    store.apply(&a).unwrap();
    store.verify(&a).unwrap();
}

#[test]
fn ordinary_heading_and_paragraph_forms_bind_actual_blocks_and_remove_no_prose() {
    let input = source("## Introduction {#introduction}\n\nAn ordinary paragraph with **formatting**.\n{#central-argument}\n");
    let result = compile(&input).unwrap();
    assert_eq!(result.annotations.len(), 2);
    assert_eq!(result.document.main.len(), 2);
    assert_eq!(result.document.main[0].key.as_ref().unwrap().name, "introduction");
    assert_eq!(result.document.main[1].key.as_ref().unwrap().name, "central-argument");
    let BlockKind::Paragraph { content } = &result.document.main[1].kind else {
        panic!()
    };
    assert_eq!(plaintext(content), "An ordinary paragraph with formatting.");
    assert!(content.contains(&Inline::Run {
        text: "formatting".into(),
        marks: vec![Mark::Strong]
    }));
    assert!(!semantic_document(&result.document).to_string().contains("hProperties"), "not HTML ids");
}

#[test]
fn formatted_heading_link_and_setext_control_keep_semantics() {
    for (brace, key) in [
        (
            "## **Café** [maps](https://example.test) {#heading}\n",
            "<Key id=\"heading\" />\n## **Café** [maps](https://example.test)\n",
        ),
        (
            "Café **maps** {#heading}\n----------------------\n",
            "<Key id=\"heading\" />\nCafé **maps**\n----------------------\n",
        ),
    ] {
        let a = source(brace);
        let b = source(key);
        let a = compile(&a).unwrap();
        let b = compile(&b).unwrap();
        assert_eq!(semantic_document(&a.document), semantic_document(&b.document));
    }
}

#[test]
fn byte_identity_source_map_and_bom_crlf_unicode_ranges_are_honest() {
    let input = format!(
        "\u{feff}{}",
        source("## Café 🦀 {#intro}\n\nNaïve **prose**.\n{#body}\n\n<Run text={\"é😀 {#data}\"} />\n").replace('\n', "\r\n")
    );
    let before = input.clone();
    let prepared = prepare(&input).unwrap();
    assert_eq!(prepared.parser_source.len(), input.len());
    assert_eq!(
        prepared
            .parser_source
            .as_bytes()
            .iter()
            .enumerate()
            .filter(|(_, b)| **b == b'\n')
            .map(|(i, _)| i)
            .collect::<Vec<_>>(),
        input
            .as_bytes()
            .iter()
            .enumerate()
            .filter(|(_, b)| **b == b'\n')
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    );
    for (i, (&original, &amended)) in input.as_bytes().iter().zip(prepared.parser_source.as_bytes()).enumerate() {
        if prepared.annotations.iter().any(|a| a.span.start <= i && i < a.span.end) {
            assert_eq!(amended, b' ');
        } else {
            assert_eq!(amended, original, "offset {i}");
        }
    }
    let checked = compile(&input).unwrap();
    assert_eq!(checked.document.original_source.as_bytes(), before.as_bytes());
    for annotation in &checked.annotations {
        assert_eq!(&input[annotation.span.start..annotation.span.end], format!("{{#{}}}", annotation.key));
        assert!(input.is_char_boundary(annotation.target_span.start));
    }
    assert_eq!(
        &input[checked.document.main[1].span.start..checked.document.main[1].span.end],
        "Naïve **prose**."
    );
    assert!(checked.document.main[2].key.is_none(), "JSON string text is not a key");
}

#[test]
fn recovered_discovery_errors_also_use_absolute_original_bom_offsets() {
    let input = format!("\u{feff}{}", source("## Café 🦀 {#heading}\n"));
    let (_, native) = satteri_pulldown_cmark::parse(&input, satteri_pulldown_cmark::MDX_OPTIONS);
    assert!(!native.is_empty());
    let prepared = prepare(&input).unwrap();
    assert_eq!(
        prepared.discovery_errors,
        native.into_iter().map(|(offset, message)| (offset + 3, message)).collect::<Vec<_>>()
    );
}

#[test]
fn nested_quote_list_and_page_tails_name_paragraphs_not_nearby_containers() {
    let input =
        source("> Quoted **prose**.\n> {#quote}\n\n- List paragraph.\n  {#item}\n- Other item.\n\n<Page name=\"p\">\n\nPage prose.\n{#page-body}\n\n</Page>\n");
    let result = compile(&input).unwrap();
    let BlockKind::Quote { paragraph } = &result.document.main[0].kind else {
        panic!()
    };
    assert_eq!(paragraph.key.as_ref().unwrap().name, "quote");
    assert!(result.document.main[0].key.is_none());
    let BlockKind::List { items, .. } = &result.document.main[1].kind else {
        panic!()
    };
    assert_eq!(items[0][0].key.as_ref().unwrap().name, "item");
    assert!(result.document.main[1].key.is_none());
    assert!(items[1][0].key.is_none());
    assert_eq!(result.document.pages[0].blocks[0].key.as_ref().unwrap().name, "page-body");
}

#[test]
fn scope_closure_or_blank_before_suffix_never_searches_backwards() {
    for body in [
        "{#orphan}\n",
        "Prose.\n\n{#orphan}\n",
        "> Prose.\n{#outside}\n",
        "- Item.\n{#outside}\n",
        "<Page name=\"p\">\n\nBody.\n\n</Page>\n{#outside}\n",
        "## Heading\n{#not-a-paragraph}\n",
    ] {
        assert_eq!(code(body), "orphan-attribute", "{body}");
    }
}

#[test]
fn paragraph_tail_ends_at_blank_line_container_closure_or_eof() {
    for body in [
        "Prose.\n{#body}",
        "Prose.\n{#body}\n\nNext.\n",
        "> Prose.\n> {#body}\n\nOutside.\n",
        "<Page name=\"p\">\n\nProse.\n{#body}\n</Page>\n",
    ] {
        assert!(compile(&source(body)).is_ok(), "{body}");
    }
    assert_eq!(code("Prose.\n{#body}\nContinued prose.\n"), "attribute-not-tail");
    assert_eq!(code("Prose.\n{#body}\n<Poll key=\"p\" options={[\"a\",\"b\"]} />\n"), "attribute-not-tail");
}

#[test]
fn fenced_inline_escaped_and_literal_component_strings_are_not_stolen() {
    let input = source(
        "## `{#heading-code}`\n\nInline `{#inline}` and \\{#escaped}.\n\n```mdx\n{#fenced}\n## Literal {#heading}\n```\n\n<Run text={\"{#string}\\n{#second}\"} />\n\n<Poll key=\"p\" options={[\"{#option}\",\"other\"]} />\n",
    );
    let prepared = prepare(&input).unwrap();
    assert!(prepared.annotations.is_empty());
    assert_eq!(prepared.parser_source, input);
    let a = compile(&input).unwrap();
    let b = satteri_authoring_experiment::author::compile(&input).unwrap();
    assert_eq!(semantic_document(&a.document), semantic_document(&b));
    assert_eq!(a.document.main.iter().filter(|b| b.key.is_some()).count(), 1, "only explicit Poll key");
}

#[test]
fn components_with_brace_strings_can_share_a_document_with_real_keys() {
    let input = source(
        "## A heading {#real}\n\nLiteral <Run text={\"{#not-a-key}\"} /> data.\n{#paragraph}\n\n<Poll key=\"p\" options={[\"{#option}\",\"other\"]} />\n",
    );
    let a = compile(&input).unwrap();
    assert_eq!(a.annotations.len(), 2);
    let BlockKind::Paragraph { content } = &a.document.main[1].kind else {
        panic!()
    };
    assert_eq!(plaintext(content), "Literal {#not-a-key} data.");
    assert_eq!(a.document.main[2].key.as_ref().unwrap().name, "p");
}

#[test]
fn duplicate_namespace_and_both_spellings_on_one_block_diagnose() {
    assert_eq!(code("## One {#same}\n\nTwo.\n{#same}\n"), "duplicate-key");
    assert_eq!(code("## One {#same}\n\n<Key id=\"same\" />\nTwo.\n"), "duplicate-key");
    assert_eq!(code("## One {#same}\n\n<Poll key=\"same\" options={[\"a\",\"b\"]} />\n"), "duplicate-key");
    for key in ["same", "different"] {
        assert_eq!(code(&format!("<Key id=\"same\" />\n## Heading {{#{key}}}\n")), "key-collision");
        assert_eq!(code(&format!("<Key id=\"same\" />\nParagraph.\n{{#{key}}}\n")), "key-collision");
    }
}

#[test]
fn unsupported_malformed_and_unterminated_forms_fail_without_a_document() {
    for body in [
        "## H {#two words}\n",
        "## H {#123}\n",
        "## H {#café}\n",
        "## H {#a .class}\n",
        "## H { #spaced }\n",
    ] {
        assert_eq!(code(body), "malformed-attribute", "{body}");
    }
    for body in [
        "## H {#a} afterwards\n",
        "## H{#a}\n",
        "Paragraph {#same-line}\n",
        "Text {#inline} more.\n",
        "## H {#a} {#b}\n",
    ] {
        assert_eq!(code(body), "attribute-position", "{body}");
    }
    assert_eq!(code("## H {#unterminated\n"), "parse");
}

#[test]
fn strict_reparse_still_rejects_unrelated_syntax_and_executable_content() {
    assert_eq!(code("## H {#good}\n\n{danger()}\n"), "unsupported-block");
    assert_eq!(code("## H {#good}\n\n<Mystery />\n"), "unknown-component");
    assert_eq!(code("## H {#good}\n\n<Poll key=\"p\" options={load()} />\n"), "nonliteral-prop");
    assert_eq!(code("## H {#good}\n\n<Broken>\n"), "parse");
    let input = source("{\"{#not-annotation}\"}\n");
    assert!(
        prepare(&input).unwrap().annotations.is_empty(),
        "literal body string isn't a marker, but body expressions remain unsupported"
    );
    assert!(compile(&input).is_err());
}

#[test]
fn native_key_only_source_still_uses_unchanged_compiler_and_source_bytes() {
    let a = compile(KEYS).unwrap();
    let b = satteri_authoring_experiment::author::compile(KEYS).unwrap();
    assert!(a.annotations.is_empty());
    assert_eq!(a.discovery_error_count, 0);
    assert_eq!(semantic_document(&a.document), semantic_document(&b));
    assert_eq!(a.document.original_source.as_bytes(), KEYS.as_bytes());
}

#[test]
fn known_multiline_single_backtick_brace_control_is_rejected_not_repaired() {
    assert_eq!(code("Code `first\n{#accidental}\nlast`.\n"), "attribute-not-tail");
    let input = source("Code `first\n<Poll key=\"p\" options={[\"a\",\"b\"]} />\nlast`.\n");
    let result = compile(&input).unwrap();
    assert!(result.annotations.is_empty());
    assert!(
        result.document.main.iter().any(|b| matches!(b.kind, BlockKind::Poll { .. })),
        "negative observation: unchanged MDX flow-component interpretation remains"
    );
}
