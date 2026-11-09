//! Format-2 corpus: rendering round-trips, Markdown-first emission with
//! proven fallbacks, source-preserving adoption, tracking-only settlement,
//! one-time key preparation and Markdown tidy planning.
use super::*;
use leafbud_document::{Occurrence, Publication};
use serde_json::json;
use std::collections::BTreeMap;

fn node(entity: &str, occurrence: Occurrence) -> Node {
    Node {
        entity: entity.into(),
        occurrence,
    }
}

fn document() -> Document {
    let state = State {
        id: Uuid::nil(),
        state: "draft".into(),
        main_page: Some("p-main".into()),
        binding: Some(Binding {
            base_url: "https://leaflet.pub".into(),
            token: Uuid::nil(),
            root: "e-root".into(),
            entity_set: "es".into(),
            entry_fact: "f-entry".into(),
            publication: None,
        }),
        nodes: BTreeMap::from([
            (
                "p-main".into(),
                node(
                    "e-main",
                    Occurrence::Page {
                        references: vec!["f-entry".into()],
                    },
                ),
            ),
            (
                "b-1".into(),
                node(
                    "e-b1",
                    Occurrence::Block {
                        placement: "f-b1".into(),
                        text_fact: Some("f-t1".into()),
                        inactive_text_fact: None,
                    },
                ),
            ),
        ]),
        sync: Some(Sync {
            profile: EDITOR_PROFILE.into(),
            local_hash: "sha256:old-local".into(),
            remote_hash: "sha256:old-remote".into(),
        }),
    };
    Document {
        format: FORMAT.into(),
        title: "T".into(),
        description: None,
        tags: vec![],
        publication: Publication::Standalone,
        theme: None,
        extra: BTreeMap::new(),
        pages: vec![Page {
            key: "p-main".into(),
            blocks: vec![Block {
                key: "b-1".into(),
                kind: BlockKind::Paragraph {
                    content: vec![Inline::Run {
                        text: "hello".into(),
                        marks: vec![],
                    }],
                },
            }],
        }],
        state,
    }
}

/// An unbound, receipt-free draft with the same body.
fn unbound() -> Document {
    let mut document = document();
    document.publication = Publication::Inherit;
    document.state.binding = None;
    document.state.sync = None;
    document
}

#[test]
fn human_comparison_is_readable_exact_and_free_of_bookkeeping() {
    let mut doc = document();
    doc.description = Some("A description".into());
    doc.tags = vec!["one".into(), "two".into()];
    doc.theme = Some(Theme { leaflet_watermark: false });
    doc.extra.insert("contributors".into(), json!(["did:plc:author"]));
    doc.extra.insert("privateLocalOnly".into(), json!("not editor content"));
    doc.pages[0].blocks[0].kind = BlockKind::Paragraph {
        content: vec![
            Inline::Run {
                text: "Read".into(),
                marks: vec![Mark::Strong],
            },
            Inline::Run {
                text: " the ".into(),
                marks: vec![],
            },
            Inline::Run {
                text: "docs".into(),
                marks: vec![Mark::Link {
                    href: "https://example.org/".into(),
                }],
            },
        ],
    };
    let before = doc.clone();
    let view = render_comparison(&doc).unwrap();
    assert!(view.contains("**Read** the [docs](https://example.org/)"), "{view}");
    assert!(view.contains("\"description\": \"A description\""));
    assert!(view.contains("\"tags\": [\"one\",\"two\"]"));
    assert!(view.contains("\"contributors\": [\"did:plc:author\"]"));
    assert!(view.contains("\"leafletWatermark\": false"));
    assert!(view.contains("<Page name=\"page-1\" kind=\"linear\">"));
    for absent in ["@leafbud", "sha256:", "p-main", "b-1", "e-root", "<Key", "privateLocalOnly", "00000000-"] {
        assert!(!view.contains(absent), "unexpected {absent}: {view}");
    }
    assert_eq!(doc, before);
    doc.pages[0].blocks[0].kind = BlockKind::Quote {
        content: vec![
            Inline::Run {
                text: "underlined".into(),
                marks: vec![Mark::Underline],
            },
            Inline::HardBreak,
        ],
    };
    let view = render_comparison(&doc).unwrap();
    assert!(view.contains("<Run text={\"underlined\"} marks={[{\"kind\":\"underline\"}]} />"));
    assert!(view.contains("<HardBreak />"));
}

#[test]
fn human_views_normalize_serialization_keys_and_page_definition_order() {
    let mut doc = document();
    for (key, occurrence) in [
        ("z-page", Occurrence::Page { references: vec![] }),
        ("a-page", Occurrence::Page { references: vec![] }),
    ] {
        doc.state.nodes.insert(key.into(), node(&format!("entity-{key}"), occurrence));
    }
    doc.state.nodes.insert(
        "link".into(),
        node(
            "e-link",
            Occurrence::Block {
                placement: "f-link".into(),
                text_fact: None,
                inactive_text_fact: None,
            },
        ),
    );
    doc.pages[0].blocks.push(Block {
        key: "link".into(),
        kind: BlockKind::PageLink { page: "z-page".into() },
    });
    doc.pages.push(Page {
        key: "a-page".into(),
        blocks: vec![],
    });
    doc.pages.push(Page {
        key: "z-page".into(),
        blocks: vec![],
    });
    let expected = render_comparison(&doc).unwrap();
    assert!(expected.contains("<PageLink page=\"page-3\" />"), "{expected}");
    doc.pages.swap(1, 2);
    let source = render(&doc).unwrap().replace("b-1", "renamed-block");
    let source = source.replace(
        "<Key id=\"renamed-block\" />\nhello",
        "<Key id=\"renamed-block\" />\n<Paragraph>\n<Run text={\"hello\"} marks={[]} />\n</Paragraph>",
    );
    let parsed = parse(&source).unwrap();
    assert_eq!(render_comparison(&parsed).unwrap(), expected);
    let mut changed = parsed;
    changed.pages[0].blocks[0].kind = BlockKind::Heading {
        level: 2,
        content: vec![Inline::Run {
            text: "hello".into(),
            marks: vec![],
        }],
    };
    assert!(render_comparison(&changed).unwrap().contains("## hello"));
    assert_ne!(render_comparison(&changed).unwrap(), expected);
}

#[test]
fn human_views_keep_exact_code_image_and_empty_block_meaning() {
    let mut doc = document();
    doc.pages[0].blocks[0].kind = BlockKind::Code {
        language: "rust".into(),
        text: "let x = 1;\n\n".into(),
    };
    assert!(render_comparison(&doc).unwrap().contains("```rust\nlet x = 1;\n\n\n```"));
    doc.pages[0].blocks[0].kind = BlockKind::Code {
        language: "text".into(),
        text: "```\nembedded fence".into(),
    };
    assert!(render_comparison(&doc).unwrap().contains("````text\n```\nembedded fence\n````"));
    doc.pages[0].blocks[0].kind = BlockKind::Code {
        language: "rust meta".into(),
        text: "a\r\nb".into(),
    };
    assert!(
        render_comparison(&doc)
            .unwrap()
            .contains("<Code language={\"rust meta\"} text={\"a\\r\\nb\"} />")
    );
    doc.pages[0].blocks[0].kind = BlockKind::Image {
        src: "https://example.org/image".into(),
        fallback: Some("fallback".into()),
        width: Some(42),
        height: None,
        alt: Some("meaning".into()),
    };
    assert!(
        render_comparison(&doc)
            .unwrap()
            .contains("<Image src={\"https://example.org/image\"} fallback={\"fallback\"} width={42} height={null} alt={\"meaning\"} />")
    );
    doc.pages[0].blocks[0].kind = BlockKind::Paragraph { content: vec![] };
    assert!(render_comparison(&doc).unwrap().contains("<Run text={\"\"} />"));
    // New-keyed documents (no confirmed rows yet) are valid comparison input;
    // their associations are simply unconfirmed, not errors.
    doc.state.nodes.clear();
    assert!(render_comparison(&doc).is_ok());
}

/// Hand-shaped variant of a rendered file: author comments, an unquoted
/// key, blank separators and an unknown local-only field.
fn authored(source: &str) -> String {
    source
        .replacen("\"title\": \"T\"", "title: T # chosen by the author", 1)
        .replacen("\"format\": \"leafbud-mdx/2\"\n", "\"format\": \"leafbud-mdx/2\"\n# author note\n", 1)
        .replacen("\n\"@leafbud\":", "\ncustomField: {\"meaning\": \"retained\"}\n\n\"@leafbud\":", 1)
}

#[test]
fn adopt_remote_settles_receipts_preserving_author_text_and_body() {
    let original = authored(&render(&document()).unwrap());
    let mut next = parse(&original).unwrap();
    next.state.sync.as_mut().unwrap().remote_hash = "sha256:new-remote".into();
    let updated = adopt_remote(&original, &next).unwrap();
    assert!(updated.contains("title: T # chosen by the author"));
    assert!(updated.contains("# author note"));
    assert!(updated.contains("customField: {\"meaning\": \"retained\"}"));
    assert!(updated.contains("sha256:new-remote"));
    assert!(!updated.contains("sha256:old-remote"));
    assert_eq!(body_of(&updated), body_of(&original));
    assert_eq!(parse(&updated).unwrap(), next);
}

#[test]
fn adopt_remote_patches_changed_fields_and_rewrites_only_changed_bodies() {
    let original = authored(&render(&document()).unwrap());
    let mut next = parse(&original).unwrap();
    next.title = "Remote title".into();
    next.description = Some("From the editor".into());
    next.tags = vec!["one".into()];
    let updated = adopt_remote(&original, &next).unwrap();
    assert!(updated.contains("\"title\": \"Remote title\""));
    assert!(!updated.contains("chosen by the author"));
    assert!(updated.contains("\"description\": \"From the editor\""));
    assert!(updated.contains("customField: {\"meaning\": \"retained\"}"));
    assert_eq!(body_of(&updated), body_of(&original));
    // A truly changed body is re-serialized; unchanged frontmatter lines stay.
    next.pages[0].blocks[0].kind = BlockKind::Paragraph {
        content: vec![Inline::Run {
            text: "edited remotely".into(),
            marks: vec![],
        }],
    };
    let updated = adopt_remote(&updated, &next).unwrap();
    assert!(updated.contains("edited remotely"));
    assert_ne!(body_of(&updated), body_of(&original));
    assert!(updated.contains("customField: {\"meaning\": \"retained\"}"));
    let mut removed = next.clone();
    removed.description = None;
    removed.tags = vec![];
    let updated = adopt_remote(&updated, &removed).unwrap();
    assert!(!updated.contains("\"description\""));
    assert!(updated.contains("\"tags\": []"));
}

#[test]
fn publication_tri_state_survives_roundtrips_and_adoption() {
    // Inherit is omitted; standalone is explicit null; a URI is a string.
    let inherit = render(&unbound()).unwrap();
    assert!(!inherit.contains("publication"));
    assert_eq!(parse(&inherit).unwrap().publication, Publication::Inherit);
    let standalone = render(&document()).unwrap();
    assert!(standalone.contains("\"publication\": null"));
    assert_eq!(parse(&standalone).unwrap().publication, Publication::Standalone);
    let mut uri = document();
    uri.publication = Publication::At("at://did:plc:abc/site.standard.publication/p".into());
    let source = render(&uri).unwrap();
    assert!(source.contains("\"publication\": \"at://did:plc:abc/site.standard.publication/p\""));
    assert_eq!(
        parse(&source).unwrap().publication,
        Publication::At("at://did:plc:abc/site.standard.publication/p".into())
    );
    // Adoption rewrites the author field only when intent actually changed.
    let mut next = parse(&source).unwrap();
    next.publication = Publication::Inherit;
    let adopted = adopt_remote(&source, &next).unwrap();
    assert!(
        !adopted
            .lines()
            .any(|l| l == "\"publication\": \"at://did:plc:abc/site.standard.publication/p\""),
        "the top-level intent line is removed; the confirmed binding association stays"
    );
    assert_eq!(parse(&adopted).unwrap().publication, Publication::Inherit);
    next.publication = Publication::Standalone;
    let adopted = adopt_remote(&adopted, &next).unwrap();
    assert!(adopted.contains("\"publication\": null"));
    // Bound documents always carry the confirmed association explicitly.
    assert!(source.contains("\"publication\""));
}

#[test]
fn adopt_remote_refuses_foreign_identity_and_verifies_output() {
    let original = render(&document()).unwrap();
    let mut foreign = document();
    foreign.state.id = Uuid::new_v5(&Uuid::NAMESPACE_URL, b"foreign");
    assert!(adopt_remote(&original, &foreign).is_err());
    // A body change that cannot round-trip (duplicate key) is caught.
    let mut broken = document();
    broken.pages[0].blocks.push(Block {
        key: "b-1".into(),
        kind: BlockKind::Rule,
    });
    assert!(adopt_remote(&original, &broken).is_err());
    // Unbound documents cannot be adoption targets.
    assert!(adopt_remote(&original, &unbound()).is_err());
}

/// A future/unknown field inside the reserved `@leafbud` envelope must
/// block managed mutation instead of silently disappearing through typed
/// State reserialization. `header` stays untyped and still reads it.
#[test]
fn unknown_state_fields_block_managed_mutation() {
    let original = render(&document()).unwrap();
    // Synthetic future field: a sibling of the owned State fields.
    let future = original.replacen("\"@leafbud\":\n", "\"@leafbud\":\n  \"futurePolicy\": {\"mode\": \"reserved\"}\n", 1);
    assert!(parse(&future).is_err(), "full parse must reject unknown owned-state semantics");
    let mut next = document();
    next.state.sync.as_mut().unwrap().remote_hash = "sha256:newer".into();
    let refused = adopt_remote(&future, &next);
    assert!(refused.is_err(), "adoption must refuse rather than drop the unknown field");
    assert!(parse(&original).is_ok());
    // Untyped header inspection still observes the raw state, future field included.
    let header = header(&future).unwrap();
    assert_eq!(header["@leafbud"]["futurePolicy"]["mode"], "reserved");
}

fn body_of(source: &str) -> &str {
    source.split_once("---\n\n").expect("fence").1
}

/// One rich block (paragraph by default) inside the standard document.
fn rich(kind: BlockKind) -> Document {
    let mut document = document();
    document.pages[0].blocks = vec![Block { key: "b-1".into(), kind }];
    document
}

fn run(text: &str, marks: &[Mark]) -> Inline {
    // Canonical typed shape: the model normalizes marks through push_run.
    let mut marks = marks.to_vec();
    marks.sort();
    marks.dedup();
    Inline::Run { text: text.into(), marks }
}

/// Render and independently expect the typed content (not only roundtrip
/// equality): the motivating href-only link must be ordinary Markdown.
#[test]
fn markdown_link_emission() {
    let url = "https://example.org/read?q=notes";
    let document = rich(BlockKind::Paragraph {
        content: vec![run(url, &[Mark::Link { href: url.into() }])],
    });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains(&format!("[{url}]({url})")));
    assert!(!body_of(&source).contains("<Run"), "no typed atoms for a plain link");
    let parsed = parse(&source).unwrap();
    assert_eq!(
        parsed.pages[0].blocks[0].kind,
        BlockKind::Paragraph {
            content: vec![run(url, &[Mark::Link { href: url.into() }])]
        }
    );

    // A link embedded between prose runs.
    let document = rich(BlockKind::Paragraph {
        content: vec![run("see ", &[]), run("guide", &[Mark::Link { href: url.into() }]), run(" now", &[])],
    });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains(&format!("see [guide]({url}) now")));
    assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, document.pages[0].blocks[0].kind);
}

#[test]
fn ordinary_marks_and_code() {
    let cases: Vec<(Vec<Inline>, &str)> = vec![
        (vec![run("bold", &[Mark::Strong])], "**bold**"),
        (vec![run("it", &[Mark::Em])], "*it*"),
        (vec![run("gone", &[Mark::Strikethrough])], "~~gone~~"),
        (vec![run("x*y", &[Mark::Code])], "`x*y`"),
        (vec![run("a`b", &[Mark::Code])], "``a`b``"),
        (vec![run("b", &[Mark::Strong, Mark::Em])], "***b***"),
        (vec![run("code", &[Mark::Strong, Mark::Code])], "**`code`**"),
        (
            vec![run("doc", &[Mark::Link { href: "https://e.org".into() }, Mark::Em])],
            "[*doc*](https://e.org)",
        ),
    ];
    for (content, expect) in cases {
        let document = rich(BlockKind::Paragraph { content: content.clone() });
        let source = match render(&document) {
            Ok(s) => s,
            Err(e) => {
                let body = render::render_body(&document, false);
                let fm = render::render_frontmatter(&document).unwrap();
                let parsed = parse(&format!("{fm}{body}")).unwrap();
                println!(
                    "CASE {expect:?}
BODY {body:?}
PARSED {:?}
EXPECTED {:?}",
                    parsed.pages[0].blocks[0].kind, document.pages[0].blocks[0].kind
                );
                panic!("{e:#}");
            }
        };
        let body = body_of(&source);
        assert!(body.contains(expect), "expected {expect:?} in {body:?}");
        assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, BlockKind::Paragraph { content });
    }
}

/// Crossing mark sets: the wrapper prefix stays open across runs with a
/// shared mark, splitting only where coverage actually changes.
#[test]
fn crossing_marks() {
    let content = vec![
        run("a ", &[Mark::Strong]),
        run("b", &[Mark::Strong, Mark::Em]),
        run(" c", &[Mark::Strong]),
        run(" and ", &[]),
        run("d ", &[Mark::Em]),
        run("e", &[Mark::Em, Mark::Strong]),
        run(" f", &[Mark::Em]),
    ];
    let document = rich(BlockKind::Paragraph { content: content.clone() });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains("**a *b* c** and *d **e** f*"));
    assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, BlockKind::Paragraph { content });
}

#[test]
fn link_destination_matrix() {
    let destinations = [
        "https://e.org/x_(y)",
        "https://e.org/a(b)c",
        "https://e.org/x y",
        "https://e.org/brack[et]",
        "https://e.org/?a=1&b=2",
        "https://e.org/%41%2f",
        "https://e.org/ünicode",
        "https://e.org/#frag",
        "relative/path.md",
        "",
        "https://e.org/back\\slash",
    ];
    for href in destinations {
        let document = rich(BlockKind::Paragraph {
            content: vec![run("label", &[Mark::Link { href: href.into() }])],
        });
        let source = render(&document).unwrap();
        assert!(!body_of(&source).contains("<Run"), "link {href:?} must be Markdown");
        let parsed = parse(&source).unwrap();
        assert_eq!(
            parsed.pages[0].blocks[0].kind,
            BlockKind::Paragraph {
                content: vec![run("label", &[Mark::Link { href: href.into() }])]
            },
            "href {href:?} must round-trip exactly"
        );
    }
    // Control characters in a destination keep the exact run.
    let document = rich(BlockKind::Paragraph {
        content: vec![run(
            "label",
            &[Mark::Link {
                href: "https://e.org/a\u{07}b".into(),
            }],
        )],
    });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains("<Run"));
    assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, document.pages[0].blocks[0].kind);
}

/// Canonical multi-run content built through the model's normalizer.
fn runs(parts: &[(&str, &[Mark])]) -> Vec<Inline> {
    let mut out = vec![];
    for (text, marks) in parts {
        push_run(&mut out, text, marks);
    }
    out
}

/// Ineligible runs stay exact typed atoms inside otherwise-Markdown prose.
#[test]
fn atom_fallbacks() {
    let cases: Vec<Vec<Inline>> = vec![
        runs(&[("plain ", &[]), ("under", &[Mark::Underline]), (" tail", &[])]),
        runs(&[("plain ", &[]), ("lit", &[Mark::Highlight { color: "#ff0".into() }]), (" tail", &[])]),
        runs(&[("edge ", &[]), (" padded ", &[Mark::Strong]), (" tail", &[])]),
        runs(&[("ctrl ", &[]), ("a\u{07}b", &[Mark::Strong]), (" tail", &[])]),
        runs(&[
            ("two ", &[]),
            (
                "links",
                &[Mark::Link { href: "https://a.org".into() }, Mark::Link { href: "https://b.org".into() }],
            ),
            (" tail", &[]),
        ]),
        runs(&[("bare url ", &[Mark::Strong]), ("see https://e.org/x now", &[Mark::Strong])]),
        runs(&[("mail ", &[]), ("contact foo@e.org today", &[])]),
    ];
    for content in cases {
        let document = rich(BlockKind::Paragraph { content: content.clone() });
        let source = render(&document).unwrap();
        assert!(body_of(&source).contains("<Run"), "expected atoms for {content:?}");
        assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, BlockKind::Paragraph { content });
    }
    // A bare URL next to safe prose splits into text + atom + text, not
    // one monolithic atom.
    let document = rich(BlockKind::Paragraph {
        content: vec![run("see www.e.org/x for more", &[])],
    });
    let source = render(&document).unwrap();
    let body = body_of(&source);
    assert!(body.contains("see <Run"), "prefix stays prose");
    assert!(body.contains("for more"), "suffix stays prose");
}

/// Break spellings: backslash-newline where valid, typed atom where the
/// flow cannot preserve it; literal LF text and soft wraps stay atoms.
#[test]
fn break_matrix() {
    let cases: Vec<(Vec<Inline>, &str)> = vec![
        (vec![run("one", &[]), Inline::HardBreak, run("two", &[])], "one\\\ntwo"),
        (vec![Inline::HardBreak, run("two", &[])], "\\\ntwo"),
        (vec![run("a", &[]), Inline::HardBreak, Inline::HardBreak, run("b", &[])], "a\\\n\\\nb"),
        (vec![run("tail", &[]), Inline::HardBreak], "tail<HardBreak />"),
    ];
    for (content, expect) in cases {
        let document = rich(BlockKind::Paragraph { content: content.clone() });
        let source = match render(&document) {
            Ok(s) => s,
            Err(e) => {
                let body = render::render_body(&document, false);
                let fm = render::render_frontmatter(&document).unwrap();
                let parsed = parse(&format!("{fm}{body}")).unwrap();
                println!(
                    "CASE {expect:?}
BODY {body:?}
PARSED {:?}
EXPECTED {:?}",
                    parsed.pages[0].blocks[0].kind, document.pages[0].blocks[0].kind
                );
                panic!("{e:#}");
            }
        };
        assert!(body_of(&source).contains(expect), "expected {expect:?} in {}", body_of(&source));
        assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, BlockKind::Paragraph { content });
    }
    // Literal newline inside a run (distinct from HardBreak) stays exact.
    let document = rich(BlockKind::Paragraph {
        content: vec![run("soft\nwrapped", &[])],
    });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains("<Run"));
    assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, document.pages[0].blocks[0].kind);
    // Headings cannot wrap: break spelled as the typed atom.
    let document = rich(BlockKind::Heading {
        level: 2,
        content: vec![run("a", &[]), Inline::HardBreak, run("b", &[])],
    });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains("## a<HardBreak />b"));
    assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, document.pages[0].blocks[0].kind);
    // Quote breaks continue the `>` prefix on each emitted line.
    let document = rich(BlockKind::Quote {
        content: vec![run("a", &[]), Inline::HardBreak, run("b", &[Mark::Strong])],
    });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains("> a\\\n> **b**"));
    assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, document.pages[0].blocks[0].kind);
}

/// Text escapes: literal punctuation stays literal, entities do not
/// decode, block-leading constructs stay in the paragraph, MDX braces
/// and dollars stay text.
#[test]
fn text_escaping() {
    let texts = [
        "a *b* c",
        "a _b_ c",
        "snake_case and a_b_c",
        "[bracket] and (paren)",
        "back`tick",
        "tilde~single~",
        "a<b and a<b c",
        "AT&T and R&D",
        "&#65; and &amp;",
        "{braces} and $math$",
        "a\\b",
        "#tag and mid #5",
        "- dash and + plus",
        "1. numbered",
        "2024. year",
        "= equals",
        "> quote",
        "!important and ![img]",
        "héllo 中文 🌍 — …",
        "***",
        "---",
        "___",
    ];
    for text in texts {
        let document = rich(BlockKind::Paragraph { content: vec![run(text, &[])] });
        let source = render(&document).unwrap();
        assert!(!body_of(&source).contains("<Run"), "plain text {text:?} needs no atom");
        let parsed = parse(&source).unwrap();
        assert_eq!(
            parsed.pages[0].blocks[0].kind,
            BlockKind::Paragraph { content: vec![run(text, &[])] },
            "text {text:?} must round-trip exactly"
        );
    }
    // Headings escape every hash so no closing sequence can form.
    let document = rich(BlockKind::Heading {
        level: 2,
        content: vec![run("C# #5", &[])],
    });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains("## C\\# \\#5"));
    assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, document.pages[0].blocks[0].kind);
}

/// One unsupported paragraph must not wrap its unrelated neighbors.
#[test]
fn block_isolation() {
    let mut document = document();
    document.state.nodes.insert(
        "b-2".into(),
        node(
            "e-b2",
            Occurrence::Block {
                placement: "f-b2".into(),
                text_fact: None,
                inactive_text_fact: None,
            },
        ),
    );
    document.state.nodes.insert(
        "b-3".into(),
        node(
            "e-b3",
            Occurrence::Block {
                placement: "f-b3".into(),
                text_fact: None,
                inactive_text_fact: None,
            },
        ),
    );
    document.pages[0].blocks = vec![
        Block {
            key: "b-2".into(),
            kind: BlockKind::Paragraph {
                content: runs(&[("clean", &[Mark::Strong])]),
            },
        },
        Block {
            key: "b-1".into(),
            kind: BlockKind::Paragraph {
                content: runs(&[("under", &[Mark::Underline])]),
            },
        },
        Block {
            key: "b-3".into(),
            kind: BlockKind::Heading {
                level: 1,
                content: runs(&[("head", &[Mark::Strikethrough])]),
            },
        },
    ];
    let source = render(&document).unwrap();
    let body = body_of(&source);
    assert!(body.contains("**clean**"));
    assert!(body.contains("# ~~head~~"));
    // The underline block is a bare flow atom, not a wrapper: isolation
    // means its neighbors stayed ordinary Markdown.
    assert_eq!(body.matches("<Paragraph>").count(), 0, "no literal wrappers: {body}");
    assert_eq!(parse(&source).unwrap().pages[0].blocks, document.pages[0].blocks);
}

/// An empty rich block keeps its Key and exact empty representation.
#[test]
fn empty_paragraph_stays_present() {
    let document = rich(BlockKind::Paragraph { content: vec![] });
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains("<Run text={\"\"} />"));
    assert_eq!(parse(&source).unwrap().pages[0].blocks[0].kind, BlockKind::Paragraph { content: vec![] });
}

/// Flat lists render as ordinary Markdown items and round-trip exactly.
#[test]
fn list_rendering_and_breaks() {
    let mut document = document();
    document.state.nodes.insert(
        "b-2".into(),
        node(
            "e-b2",
            Occurrence::Block {
                placement: "f-b2".into(),
                text_fact: None,
                inactive_text_fact: None,
            },
        ),
    );
    document.state.nodes.insert(
        "b-3".into(),
        node(
            "e-b3",
            Occurrence::Block {
                placement: "f-b3".into(),
                text_fact: None,
                inactive_text_fact: None,
            },
        ),
    );
    document.pages[0].blocks = vec![
        Block {
            key: "b-1".into(),
            kind: BlockKind::ListItem {
                style: ListStyle::Bullet,
                content: runs(&[("first", &[Mark::Strong])]),
            },
        },
        Block {
            key: "b-2".into(),
            kind: BlockKind::ListItem {
                style: ListStyle::Ordered,
                content: runs(&[("second", &[])]),
            },
        },
        Block {
            key: "b-3".into(),
            kind: BlockKind::ListItem {
                style: ListStyle::Bullet,
                content: vec![run("one", &[]), Inline::HardBreak, run("two", &[])],
            },
        },
    ];
    let source = render(&document).unwrap();
    let body = body_of(&source);
    assert!(body.contains("- **first**"), "{body}");
    assert!(body.contains("1. second"));
    assert!(body.contains("- one\\\n  two"), "item break continuation indents to the content column");
    assert_eq!(parse(&source).unwrap().pages[0].blocks, document.pages[0].blocks);
    // An empty item keeps an exact empty representation.
    document.pages[0].blocks = vec![Block {
        key: "b-1".into(),
        kind: BlockKind::ListItem {
            style: ListStyle::Bullet,
            content: vec![],
        },
    }];
    let source = render(&document).unwrap();
    assert!(body_of(&source).contains("- <Run text={\"\"} />"));
    assert_eq!(
        parse(&source).unwrap().pages[0].blocks[0].kind,
        BlockKind::ListItem {
            style: ListStyle::Bullet,
            content: vec![]
        }
    );
}

/// A genuinely changed body is adopted through the Markdown-first writer,
/// while an unchanged body keeps its exact bytes.
#[test]
fn adopt_remote_adopts_markdown_for_changed_body() {
    let original = authored(&render(&document()).unwrap());
    let mut next = parse(&original).unwrap();
    next.pages[0].blocks[0].kind = BlockKind::Paragraph {
        content: vec![
            run("see ", &[]),
            run(
                "guide",
                &[Mark::Link {
                    href: "https://e.org/docs".into(),
                }],
            ),
        ],
    };
    let updated = adopt_remote(&original, &next).unwrap();
    assert!(updated.contains("see [guide](https://e.org/docs)"));
    assert_eq!(parse(&updated).unwrap(), next);
    // No-op adoption keeps the original body bytes, Runs included.
    let again = adopt_remote(&updated, &parse(&updated).unwrap()).unwrap();
    assert_eq!(body_of(&again), body_of(&updated));
}

/// Hand-shaped body: replace one rendered block line with a literal
/// wrapper carrying the given literal runs.
fn wrapped_body(source: &str, line: &str, wrapper_runs: &str) -> String {
    source.replacen(
        &format!("<Key id=\"b-1\" />\n{line}\n"),
        &format!("<Key id=\"b-1\" />\n<Paragraph>\n{wrapper_runs}\n</Paragraph>\n"),
        1,
    )
}

#[test]
fn tidy_normalizes_source_spans_only() {
    let original = wrapped_body(&authored(&render(&document()).unwrap()), "hello", "<Run text={\"hello world\"} marks={[]} />");
    assert!(original.contains("# author note"));
    assert!(original.contains("customField"));
    let plan = plan_markdown_tidy(&original).unwrap();
    assert_eq!(plan.blocks_changed, 1, "the wrapped block normalizes");
    assert!(plan.edits[0].text.contains("hello world"));
    let tidied = apply_markdown_tidy(&original, &plan).unwrap();
    fn front(source: &str) -> &str {
        source.split_once("---\n\n").expect("fence").0
    }
    assert_eq!(front(&tidied), front(&original), "frontmatter bytes untouched");
    assert!(tidied.contains("<Key id=\"b-1\" />\nhello world\n"), "Key marker and spacing preserved");
    assert!(!tidied.contains("<Paragraph>"));
    assert_eq!(parse(&tidied).unwrap(), parse(&original).unwrap());
    // Second tidy plans nothing.
    let repeat = plan_markdown_tidy(&tidied).unwrap();
    assert_eq!(repeat.blocks_changed, 0);
    assert!(repeat.edits.is_empty());
}

/// A local semantic edit (body text changed, receipts stale) survives
/// tidy byte-for-byte outside the normalized span.
#[test]
fn tidy_preserves_local_edits_and_witnesses() {
    let original = wrapped_body(&authored(&render(&document()).unwrap()), "hello", "<Run text={\"hello world\"} marks={[]} />");
    let edited = original.replacen("\"localHash\": \"sha256:old-local\"", "\"localHash\": \"sha256:stale\"", 1);
    assert!(edited.contains("sha256:stale"));
    let plan = plan_markdown_tidy(&edited).unwrap();
    let tidied = apply_markdown_tidy(&edited, &plan).unwrap();
    assert!(tidied.contains("sha256:stale"), "witness bytes untouched");
    assert!(tidied.contains("customField"));
    assert!(!tidied.contains("<Paragraph>"));
}

#[test]
fn tidy_refuses_unparsable_source() {
    let original = render(&document()).unwrap();
    let commented = original.replacen("\n---\n\n", "\n{/* author comment */}\n\n", 1);
    assert!(plan_markdown_tidy(&commented).is_err(), "body comments are unsupported input today");
}

/// A block whose candidate cannot validate is skipped with a reason,
/// never approximated; its neighbors can still normalize.
#[test]
fn tidy_skips_failed_candidates_with_reasons() {
    let mut source = document();
    source.state.nodes.insert(
        "b-2".into(),
        node(
            "e-b2",
            Occurrence::Block {
                placement: "f-b2".into(),
                text_fact: None,
                inactive_text_fact: None,
            },
        ),
    );
    source.pages[0].blocks = vec![
        Block {
            key: "b-1".into(),
            kind: BlockKind::Quote {
                content: vec![run("a", &[]), Inline::HardBreak, run(" x ", &[Mark::Underline])],
            },
        },
        Block {
            key: "b-2".into(),
            kind: BlockKind::Paragraph {
                content: runs(&[("clean", &[Mark::Strong])]),
            },
        },
    ];
    let original = render(&source).unwrap();
    let plan = plan_markdown_tidy(&original).unwrap();
    let tidied = apply_markdown_tidy(&original, &plan).unwrap();
    assert_eq!(parse(&tidied).unwrap(), parse(&original).unwrap());
}

#[test]
fn tidy_preserves_crlf_line_endings() {
    let document = rich(BlockKind::Paragraph {
        content: vec![run("one", &[]), Inline::HardBreak, run("two", &[])],
    });
    let lf = render(&document).unwrap();
    let crlf = wrapped_body(
        &lf,
        "one\\\ntwo",
        "<Run text={\"one\"} marks={[]} />\n\n<HardBreak />\n\n<Run text={\"two\"} marks={[]} />",
    )
    .replace('\n', "\r\n");
    let plan = plan_markdown_tidy(&crlf).unwrap();
    assert_eq!(plan.blocks_changed, 1);
    assert!(plan.edits[0].text.contains("\\\r\n"), "inserted break keeps the CRLF convention");
    let tidied = apply_markdown_tidy(&crlf, &plan).unwrap();
    assert_eq!(parse(&tidied).unwrap(), parse(&crlf).unwrap());
}

#[test]
fn tidy_handles_bom_prefixed_source() {
    let plain = wrapped_body(&render(&document()).unwrap(), "hello", "<Run text={\"hello world\"} marks={[]} />");
    let bommed = format!("\u{feff}{plain}");
    let plan = plan_markdown_tidy(&bommed).unwrap();
    let tidied = apply_markdown_tidy(&bommed, &plan).unwrap();
    assert!(tidied.starts_with('\u{feff}'), "the BOM survives");
    assert!(tidied.contains("hello world\n"));
    assert!(!tidied.contains("<Paragraph>"));
}

// ============ Key preparation ============

/// A tiny deterministic key source for tests.
fn counter(prefix: &str) -> impl FnMut() -> String + use<> {
    let mut next = 0;
    let prefix = prefix.to_owned();
    move || {
        next += 1;
        format!("{prefix}-{next}")
    }
}

#[test]
fn preparation_assigns_stable_keys_and_persists_main_page() {
    let authored_source = "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\"}\n---\n\n# Intro\n\nfirst paragraph\n\n- item one\n- item two\n";
    let (mut block, mut page) = (counter("b"), counter("p"));
    let plan = prepare_keys(
        authored_source,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    assert_eq!(plan.assigned, 4, "heading, paragraph and two list items");
    assert_eq!(plan.main_page.as_deref(), Some("p-1"), "the page source supplies the main-page key");
    let prepared = apply_key_preparation(authored_source, &plan).unwrap();
    assert!(prepared.contains("<Key id=\"b-1\" />\n# Intro"));
    assert!(prepared.contains("<Key id=\"b-2\" />\nfirst paragraph"));
    // The multi-item list is split by per-item markers with bytes preserved.
    assert!(prepared.contains("<Key id=\"b-3\" />\n- item one\n<Key id=\"b-4\" />\n- item two"));
    assert!(prepared.contains("\"mainPage\": \"p-1\""), "main-page identity persisted in the envelope");
    let parsed = parse_with_spans(&prepared).unwrap();
    assert!(parsed.unkeyed().is_empty());
    assert_eq!(parsed.document.state.main_page.as_deref(), Some("p-1"));
    // Repeated preparation is an exact no-op.
    let (mut block, mut page) = (counter("z"), counter("q"));
    let again = prepare_keys(
        &prepared,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    assert!(again.is_noop());
    assert_eq!(apply_key_preparation(&prepared, &again).unwrap(), prepared);
}

#[test]
fn preparation_is_preview_only_until_applied() {
    let authored_source =
        "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\"}\n---\n\nfirst\n";
    let (mut block, mut page) = (counter("b"), counter("p"));
    let plan = prepare_keys(
        authored_source,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    assert_eq!(plan.assigned, 1);
    // Planning does not mutate the source; two plans propose different keys.
    let (mut block, mut page) = (counter("c"), counter("r"));
    let other = prepare_keys(
        authored_source,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    assert_ne!(plan.edits[0].text, other.edits[0].text);
    let applied = apply_key_preparation(authored_source, &plan).unwrap();
    assert_eq!(parse_with_spans(authored_source).unwrap().unkeyed().len(), 1, "original stays unprepared");
    assert_eq!(parse_with_spans(&applied).unwrap().unkeyed().len(), 0);
    // Proof uses the same proposed keys; a mismatched key fails the proof.
    let (mut block, mut page) = (counter("b"), counter("p"));
    let mismatched = prepare_keys(
        authored_source,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    let mut tampered = mismatched.clone();
    tampered.edits[0].text = tampered.edits[0].text.replace("b-1", "b-9");
    assert!(apply_key_preparation(authored_source, &tampered).is_err());
}

#[test]
fn preparation_preserves_crlf_and_bom() {
    let authored_source = "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\"}\n---\n\nfirst\r\n\r\nsecond\r\n".replace('\n', "\r\n");
    let (mut block, mut page) = (counter("b"), counter("p"));
    let plan = prepare_keys(
        &authored_source,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    let prepared = apply_key_preparation(&authored_source, &plan).unwrap();
    assert!(prepared.contains("<Key id=\"b-1\" />\r\nfirst"));
    assert!(prepared.contains("<Key id=\"b-2\" />\r\nsecond"));
    let bommed = format!("\u{feff}{}", authored_source);
    let (mut block, mut page) = (counter("b"), counter("p"));
    let plan = prepare_keys(
        &bommed,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    let prepared = apply_key_preparation(&bommed, &plan).unwrap();
    assert!(prepared.starts_with('\u{feff}'));
}

#[test]
fn preparation_keeps_retired_rows_and_existing_keys() {
    // A retired row in the envelope plus one keyed block and one new block.
    let source = "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\", \"nodes\": {\"b-gone\": {\"entity\": \"e-gone\", \"kind\": \"block\", \"placement\": \"f-gone\"}}}\n---\n\n<Key id=\"b-1\" />\nkept\n\nadded\n";
    let (mut block, mut page) = (counter("fresh"), counter("p"));
    let plan = prepare_keys(
        source,
        KeySource {
            block: &mut block,
            page: &mut page,
        },
    )
    .unwrap();
    assert_eq!(plan.assigned, 1);
    assert!(plan.main_page.is_none(), "persisted main-page identity is untouched");
    let prepared = apply_key_preparation(source, &plan).unwrap();
    assert!(prepared.contains("\"b-gone\""), "retired rows survive preparation");
    assert!(prepared.contains("<Key id=\"fresh-1\" />\nadded"));
    assert_eq!(parse_with_spans(&prepared).unwrap().unkeyed().len(), 0);
}

// ============ Tracking-only settlement ============

#[test]
fn patch_tracking_settles_envelope_without_touching_body_or_fields() {
    // An authored unbound draft (comments and custom fields included).
    let original = authored(&render(&unbound()).unwrap()).replace("<Key id=\"b-1\" />", "");
    let before = parse(&original).unwrap();
    // First binding transition: unbound → bound, receipts appear.
    let mut next = before.state.clone();
    next.binding = Some(Binding {
        base_url: "https://leaflet.pub".into(),
        token: Uuid::new_v5(&Uuid::NAMESPACE_URL, b"fresh"),
        root: "e-root2".into(),
        entity_set: "es".into(),
        entry_fact: "f-entry2".into(),
        publication: None,
    });
    next.sync = Some(Sync {
        profile: EDITOR_PROFILE.into(),
        local_hash: "sha256:settled-local".into(),
        remote_hash: "sha256:settled-remote".into(),
    });
    let settled = patch_tracking(&original, &next).unwrap();
    assert!(settled.contains("sha256:settled-remote"));
    assert!(settled.contains("title: T # chosen by the author"), "author fields untouched");
    assert!(settled.contains("customField"));
    assert_eq!(body_of(&settled), body_of(&original), "body bytes untouched");
    assert_eq!(parse(&settled).unwrap().state, next);
    // Rebinding an established binding is refused.
    let mut rebinding = next.clone();
    rebinding.binding = Some(Binding {
        base_url: "https://elsewhere".into(),
        token: Uuid::nil(),
        root: "r".into(),
        entity_set: "e".into(),
        entry_fact: "f".into(),
        publication: None,
    });
    assert!(patch_tracking(&settled, &rebinding).is_err());
    // Foreign identity is refused.
    let mut foreign = next.clone();
    foreign.id = Uuid::new_v4();
    assert!(patch_tracking(&settled, &foreign).is_err());
}

#[test]
fn editor_value_is_stable_under_key_rename_and_new_key_assignment() {
    let document = document();
    let base = document.editor_value();
    // Renaming a retained key does not alter editor identity.
    let mut renamed = document.clone();
    let block = renamed.pages[0].blocks[0].key.clone();
    let row = renamed.state.nodes.remove(&block).expect("row");
    renamed.state.nodes.insert("b-renamed".into(), row);
    renamed.pages[0].blocks[0].key = "b-renamed".into();
    assert_eq!(renamed.editor_value(), base);
    // Removing the block moves its locator into explicit retirements.
    let mut deleted = document.clone();
    deleted.pages[0].blocks.clear();
    let value = deleted.editor_value();
    assert_eq!(value["retired"].as_array().map(Vec::len), Some(1), "the removed block's row retires");
    assert_eq!(value["retired"][0]["entity"], "e-b1");
    assert_ne!(value, base);
}

/// Ordinary structural authoring: splitting keeps one retained key and one
/// new piece; joining retires one key; ordering never infers identity from
/// position or text similarity.
#[test]
fn split_join_order_and_the_last_block() {
    let state = "\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\", \"nodes\": {\"b-1\": {\"entity\": \"e-b1\", \"kind\": \"block\", \"placement\": \"f-b1\"}}}";
    // Split: the retained key stays on the first piece; the second is new.
    let split = format!("---\nformat: leafbud-mdx/2\ntitle: T\n{state}\n---\n\n<Key id=\"b-1\" />\nfirst half\n\nsecond half\n");
    let parsed = parse_with_spans(&split).unwrap();
    assert_eq!(parsed.document.pages[0].blocks.len(), 2);
    assert_eq!(parsed.document.pages[0].blocks[0].key, "b-1");
    assert_eq!(parsed.unkeyed().len(), 1, "the new piece needs preparation, not inference");
    let value = parsed.document.editor_value();
    assert_eq!(value["main"]["blocks"][0]["association"]["entity"], "e-b1");
    assert_eq!(value["main"]["blocks"][1]["association"], json!({"kind": "new"}));
    // Join: one keyed block disappears; its row retires without a successor.
    let joined = format!("---\nformat: leafbud-mdx/2\ntitle: T\n{state}\n---\n\n<Key id=\"b-1\" />\njoined text\n");
    let parsed = parse_with_spans(&joined).unwrap();
    assert_eq!(parsed.document.pages[0].blocks.len(), 1);
    // Deleting the last block leaves a valid empty main page.
    let emptied = format!("---\nformat: leafbud-mdx/2\ntitle: T\n{state}\n---\n\n");
    let parsed = parse_with_spans(&emptied).unwrap();
    assert!(parsed.document.pages[0].blocks.is_empty());
    assert_eq!(parsed.document.editor_value()["retired"].as_array().map(Vec::len), Some(1));
}

/// Page links may reference named pages defined after the link (forward
/// references are ordinary authorship order, not errors).
#[test]
fn page_links_may_reference_forward_definitions() {
    let state = "\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\", \"nodes\": {\"p-main\": {\"entity\": \"e-main\", \"kind\": \"page\", \"references\": []}, \"b-1\": {\"entity\": \"e-b1\", \"kind\": \"block\", \"placement\": \"f-b1\"}, \"p-later\": {\"entity\": \"e-later\", \"kind\": \"page\", \"references\": []}}}";
    let source = format!(
        "---\nformat: leafbud-mdx/2\ntitle: T\n{state}\n---\n\n<Key id=\"b-1\" />\n<PageLink page=\"p-later\" />\n\n<Page name=\"p-later\" kind=\"linear\">\n\nabout\n\n</Page>\n"
    );
    let parsed = parse_with_spans(&source).unwrap();
    assert_eq!(parsed.document.pages.len(), 2);
    // Unresolved targets stay errors.
    let dangling = source.replace("p-later\" />\n\n<Page name=\"p-later\"", "p-gone\" />\n\n<Page name=\"p-later\"");
    assert!(parse(&dangling).unwrap_err().to_string().contains("unresolved page link"));
}
