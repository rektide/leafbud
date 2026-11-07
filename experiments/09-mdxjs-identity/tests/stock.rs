use markdown::mdast::Node;

fn stock(source: &str) -> Result<Node, markdown::message::Message> {
    mdxjs::mdast_util_from_mdx(source, &mdxjs::Options::default())
}

#[test]
fn pinned_mdx_surprisingly_accepts_requested_attribute_spellings_as_expressions() {
    for source in ["## Introduction {#introduction}\n", "Ordinary prose.\n{#body}\n"] {
        assert!(stock(source).is_ok(), "{source}");
    }
    let tree = stock(include_str!("../fixtures/attribute-conflict.mdx")).unwrap();
    let children = tree.children().unwrap();
    assert!(matches!(children[0].children().unwrap()[1], Node::MdxTextExpression(_)));
    assert!(matches!(children[2], Node::MdxFlowExpression(_)));
    // Compiling is a control only: the returned JavaScript is never evaluated.
    let javascript = mdxjs::compile("## Heading {#id}\n", &mdxjs::Options::default()).unwrap();
    assert!(javascript.contains("#id"));
}

#[test]
fn expression_grammar_is_not_a_general_attribute_key_grammar() {
    for source in ["## Heading {#trailing.}\n", "{#two..dots}\n", "{a + }\n"] {
        let error = stock(source).unwrap_err();
        assert!(error.reason.contains("swc"), "{error:?}");
    }
}

#[test]
fn native_comment_is_a_positioned_flow_expression_not_identity() {
    let tree = stock("{/* springleaf:key intro */}\n## Heading\n").unwrap();
    let children = tree.children().unwrap();
    let Node::MdxFlowExpression(comment) = &children[0] else { panic!("{tree:?}") };
    assert_eq!(comment.value, "/* springleaf:key intro */");
    assert_eq!(comment.position.as_ref().unwrap().start.offset, 0);
    assert!(matches!(&children[1], Node::Heading(_)));
}

#[test]
fn javascript_syntax_validation_is_not_literal_validation() {
    for source in [
        "{run()}\n",
        "<Poll options={load()} />\n",
        "<Poll {...props} />\n",
        "import X from 'x'\n\n<X />\n",
    ] {
        assert!(stock(source).is_ok(), "{source}");
    }
}

#[test]
fn code_and_escaped_attribute_shapes_remain_literal() {
    for source in ["`{#id}`\n", "```mdx\n{#id}\n```\n", "## Heading \\{#id}\n"] {
        assert!(stock(source).is_ok(), "{source}");
    }
}

#[test]
fn underlying_markdown_without_swc_callback_is_not_equivalent() {
    let options = markdown::ParseOptions::mdx();
    assert!(markdown::to_mdast("{a + }\n", &options).is_ok());
    assert!(stock("{a + }\n").is_err());
}

#[test]
fn components_are_real_jsx_nodes_with_ordered_attributes_and_children() {
    let tree = stock("<Canvas width={800}>\n  <Place x={12.25}>\n    <Image src=\"map.png\" />\n  </Place>\n</Canvas>\n").unwrap();
    let Node::MdxJsxFlowElement(canvas) = &tree.children().unwrap()[0] else {
        panic!("{tree:?}")
    };
    assert_eq!(canvas.name.as_deref(), Some("Canvas"));
    assert_eq!(canvas.attributes.len(), 1);
    assert!(matches!(canvas.children[0], Node::MdxJsxFlowElement(_)));
}

#[test]
fn native_mdx_flow_comments_interrupt_apparent_multiline_code() {
    let tree = stock("Code `first\n{/* springleaf:key accidental */}\nlast`.\n").unwrap();
    let children = tree.children().unwrap();
    assert_eq!(children.len(), 3);
    assert!(matches!(children[0], Node::Paragraph(_)));
    assert!(matches!(children[1], Node::MdxFlowExpression(_)));
    assert!(matches!(children[2], Node::Paragraph(_)));
    assert!(!children[0].children().unwrap().iter().any(|n| matches!(n, Node::InlineCode(_))));
}
