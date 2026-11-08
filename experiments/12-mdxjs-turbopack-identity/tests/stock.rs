use markdown::mdast::Node;
use mdxjs_turbopack_identity_experiment::stock;

#[test]
fn exact_hash_examples_are_expressions_not_native_identities() {
    let tree = stock(include_str!("../fixtures/attributes.mdx")).unwrap();
    let children = tree.children().unwrap();
    assert!(matches!(children[0].children().unwrap()[1], Node::MdxTextExpression(_)));
    assert!(matches!(children[2], Node::MdxFlowExpression(_)));
}

#[test]
fn punctuation_still_depends_on_javascript_grammar() {
    for source in ["## Heading {#trailing.}\n", "{#two..dots}\n", "{a + }\n"] {
        let error = stock(source).unwrap_err();
        assert!(error.reason.contains("swc"), "{error:?}");
    }
}

#[test]
fn comments_are_positioned_native_flow_expressions() {
    let tree = stock("{/* leafspring:key intro */}\n## Heading\n").unwrap();
    let children = tree.children().unwrap();
    let Node::MdxFlowExpression(comment) = &children[0] else { panic!("{tree:?}") };
    assert_eq!(comment.value, "/* leafspring:key intro */");
    assert_eq!(comment.position.as_ref().unwrap().start.offset, 0);
    assert!(matches!(children[1], Node::Heading(_)));
}

#[test]
fn negative_observation_flow_comment_still_breaks_multiline_inline_code() {
    let tree = stock(include_str!("../fixtures/multiline-comment.mdx")).unwrap();
    let children = tree.children().unwrap();
    assert_eq!(children.len(), 3);
    assert!(matches!(children[0], Node::Paragraph(_)));
    assert!(matches!(children[1], Node::MdxFlowExpression(_)));
    assert!(matches!(children[2], Node::Paragraph(_)));
    assert!(!children[0].children().unwrap().iter().any(|n| matches!(n, Node::InlineCode(_))));
    // CommonMark is a control, not substituted for the fork.
    let commonmark = markdown::to_mdast(include_str!("../fixtures/multiline-comment.mdx"), &Default::default()).unwrap();
    assert_eq!(commonmark.children().unwrap().len(), 1);
    assert!(
        commonmark.children().unwrap()[0]
            .children()
            .unwrap()
            .iter()
            .any(|n| matches!(n, Node::InlineCode(_)))
    );
}

#[test]
fn quoted_flow_interruption_is_not_fixed_either() {
    let tree = stock("> Code `first\n> {/* leafspring:key accidental */}\n> last`.\n").unwrap();
    let quoted = tree.children().unwrap()[0].children().unwrap();
    assert_eq!(quoted.len(), 3);
    assert!(matches!(quoted[1], Node::MdxFlowExpression(_)));
}

#[test]
fn negative_observation_indentation_does_not_protect_code() {
    let tree = stock("    {/* leafspring:key accidental */}\n\nAfter.\n").unwrap();
    assert!(matches!(tree.children().unwrap()[0], Node::MdxFlowExpression(_)));
    let tree = stock("    <Poll key=\"accidental\" />\n").unwrap();
    assert!(matches!(tree.children().unwrap()[0], Node::MdxJsxFlowElement(_)));
}

#[test]
fn javascript_syntax_validation_still_accepts_executable_input() {
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
fn parse_only_fork_still_installs_swc_callbacks() {
    assert!(stock("{a + }\n").is_err());
    assert!(markdown::to_mdast("{a + }\n", &markdown::ParseOptions::mdx()).is_ok());
}
