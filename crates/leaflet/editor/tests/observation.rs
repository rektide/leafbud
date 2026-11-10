//! Typed occurrence locators and flat-list observation over synthetic
//! native fact graphs (real Yrs text payloads; no network).
use leafbud_document::{BlockKind, ListStyle, Node, Occurrence};
use leafbud_editor::observe;
use serde_json::{Value, json};
use uuid::Uuid;

fn yrs_payload(text: &str) -> String {
    use base64::Engine;
    use yrs::{Doc, Transact, XmlFragment as _};
    let doc = Doc::new();
    let fragment = doc.get_or_insert_xml_fragment("prosemirror");
    let mut txn = doc.transact_mut();
    let paragraph = fragment.insert(&mut txn, 0, yrs::XmlElementPrelim::empty("paragraph"));
    paragraph.insert(&mut txn, 0, yrs::XmlTextPrelim::new(text));
    let update = txn.encode_update_v1();
    drop(txn);
    base64::engine::general_purpose::STANDARD.encode(update)
}

fn fact(id: &str, entity: &str, attribute: &str, data: Value) -> Value {
    json!({"id": id, "entity": entity, "attribute": attribute, "data": data})
}

/// One linear page whose blocks are: bullet item, ordered item, paragraph.
fn graph() -> Vec<Value> {
    vec![
        fact(
            "f-entry",
            "e-root",
            "root/page",
            json!({"type": "ordered-reference", "value": "e-page", "position": "a"}),
        ),
        fact("f-type", "e-page", "page/type", json!({"type": "page-type-union", "value": "doc"})),
        fact(
            "f-m1",
            "e-page",
            "card/block",
            json!({"type": "ordered-reference", "value": "e-b1", "position": "a"}),
        ),
        fact(
            "f-m2",
            "e-page",
            "card/block",
            json!({"type": "ordered-reference", "value": "e-b2", "position": "b"}),
        ),
        fact(
            "f-m3",
            "e-page",
            "card/block",
            json!({"type": "ordered-reference", "value": "e-b3", "position": "c"}),
        ),
        fact("f-t1", "e-b1", "block/type", json!({"type": "block-type-union", "value": "text"})),
        fact("f-t2", "e-b2", "block/type", json!({"type": "block-type-union", "value": "text"})),
        fact("f-t3", "e-b3", "block/type", json!({"type": "block-type-union", "value": "text"})),
        fact("f-l1", "e-b1", "block/is-list", json!({"type": "boolean", "value": true})),
        fact("f-l2", "e-b2", "block/is-list", json!({"type": "boolean", "value": true})),
        fact("f-l2s", "e-b2", "block/list-style", json!({"type": "list-style-union", "value": "ordered"})),
        fact("f-x1", "e-b1", "block/text", json!({"type": "text", "value": yrs_payload("bullet one")})),
        fact("f-x2", "e-b2", "block/text", json!({"type": "text", "value": yrs_payload("numbered two")})),
        fact("f-x3", "e-b3", "block/text", json!({"type": "text", "value": yrs_payload("plain three")})),
    ]
}

#[test]
fn flat_lists_and_typed_locators_observe() {
    let content = observe(&graph(), "e-root", Uuid::nil()).unwrap();
    assert_eq!(content.pages.len(), 1);
    let blocks = &content.pages[0].blocks;
    assert_eq!(
        blocks[0].kind,
        BlockKind::ListItem {
            style: ListStyle::Bullet,
            content: vec![leafbud_document::Inline::Run {
                text: "bullet one".into(),
                marks: vec![]
            }]
        }
    );
    assert_eq!(
        blocks[1].kind,
        BlockKind::ListItem {
            style: ListStyle::Ordered,
            content: vec![leafbud_document::Inline::Run {
                text: "numbered two".into(),
                marks: vec![]
            }]
        }
    );
    assert!(matches!(blocks[2].kind, BlockKind::Paragraph { .. }));
    let page_key = &content.pages[0].key;
    let Node { entity, occurrence } = &content.nodes[page_key];
    assert_eq!(entity, "e-page");
    assert_eq!(
        occurrence,
        &Occurrence::Page {
            references: vec!["f-entry".into()]
        }
    );
    for (key, placement, text) in [(0usize, "f-m1", "f-x1"), (1, "f-m2", "f-x2"), (2, "f-m3", "f-x3")] {
        let Node { entity, occurrence } = &content.nodes[&blocks[key].key];
        assert_eq!(entity, &format!("e-b{}", key + 1));
        assert_eq!(
            occurrence,
            &Occurrence::Block {
                placement: placement.into(),
                text_fact: Some(text.into()),
                inactive_text_fact: None
            },
            "each occurrence carries its own placement and active text locator"
        );
    }
}

#[test]
fn task_nested_and_heading_lists_are_diagnosed() {
    let mut task = graph();
    task.push(fact("f-cl", "e-b1", "block/check-list", json!({"type": "boolean", "value": true})));
    let error = observe(&task, "e-root", Uuid::nil()).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("task lists"), "{error}");

    let mut nested = graph();
    nested.push(fact(
        "f-n",
        "e-b1",
        "card/block",
        json!({"type": "ordered-reference", "value": "e-inner", "position": "a"}),
    ));
    let error = observe(&nested, "e-root", Uuid::nil()).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("nested"), "{error}");

    let mut heading = graph();
    heading.retain(|f| !(f["entity"] == "e-b1" && f["attribute"] == "block/text"));
    for fact in heading.iter_mut() {
        if fact["entity"] == "e-b1" && fact["attribute"] == "block/type" {
            fact["data"]["value"] = json!("heading");
        }
    }
    heading.push(fact("f-hl", "e-b1", "block/heading-level", json!({"type": "number", "value": 2})));
    let error = observe(&heading, "e-root", Uuid::nil()).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("heading list items"), "{error}");
}

#[test]
fn list_numbering_override_is_diagnosed() {
    let mut graph = graph();
    graph.push(fact("f-ln", "e-b2", "block/list-number", json!({"type": "number", "value": 5})));
    let error = observe(&graph, "e-root", Uuid::nil()).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("numbering overrides"), "{error}");
}

/// Inventory from pinned upstream attributes.ts 600c7b5f, not a claim that
/// recognizing a block/type string makes its semantics writable.
#[test]
fn upstream_richer_kind_inventory_refuses_instead_of_truncating() {
    for kind in [
        "datetime",
        "rsvp",
        "link",
        "mailbox",
        "embed",
        "html",
        "button",
        "poll",
        "bluesky-post",
        "standard-site-post",
        "standard-site-publication",
        "math",
        "members-only-delimiter",
        "posts-list",
        "recommended-pubs",
        "reply",
        "questions",
        "signup",
        "image-gallery",
        "post-header",
        "embedded-canvas",
        "group",
        "drawing",
    ] {
        let mut facts = graph();
        facts.iter_mut().find(|r| r["id"] == "f-t3").unwrap()["data"]["value"] = kind.into();
        let error = observe(&facts, "e-root", Uuid::nil()).err().unwrap().to_string();
        assert!(error.contains(&format!("unsupported block type: {kind}")), "{kind}: {error}");
    }
    for (entity, attribute, data) in [
        ("e-b3", "block/text-alignment", json!({"type":"text-alignment-type-union","value":"center"})),
        ("e-b3", "block/is-locked", json!({"type":"boolean","value":true})),
        ("e-b3", "block/footnote", json!({"type":"ordered-reference","value":"note","position":"a"})),
        ("e-b3", "block/comment", json!({"type":"ordered-reference","value":"comment","position":"a"})),
        ("e-page", "page/title", json!({"type":"string","value":"navigation"})),
        ("e-root", "theme/body-font", json!({"type":"string","value":"serif"})),
        ("e-root", "root/cover-image", json!({"type":"reference","value":"cover"})),
    ] {
        let mut facts = graph();
        facts.push(fact("unsupported", entity, attribute, data));
        let error = observe(&facts, "e-root", Uuid::nil()).err().unwrap().to_string();
        assert!(error.contains("unrepresented editor attributes") && error.contains(attribute), "{error}");
    }
    let mut canvas = graph();
    canvas.iter_mut().find(|r| r["id"] == "f-type").unwrap()["data"]["value"] = "canvas".into();
    assert!(
        observe(&canvas, "e-root", Uuid::nil())
            .err()
            .unwrap()
            .to_string()
            .contains("unsupported page type: canvas")
    );
}

#[test]
fn native_scalar_and_image_meaning_is_explicit_and_local_resources_refuse() {
    let mut facts = graph();
    facts.retain(|r| !["e-b1", "e-b2"].contains(&r["entity"].as_str().unwrap()) && !["f-m1", "f-m2"].contains(&r["id"].as_str().unwrap()));
    facts.iter_mut().find(|r| r["id"] == "f-t3").unwrap()["data"]["value"] = "code".into();
    facts.push(fact("code", "e-b3", "block/code", json!({"type":"string","value":"exact\n\n"})));
    facts.push(fact("lang", "e-b3", "block/code-language", json!({"type":"string","value":"rust"})));
    let code = observe(&facts, "e-root", Uuid::nil()).unwrap();
    assert_eq!(
        code.pages[0].blocks[0].kind,
        BlockKind::Code {
            language: "rust".into(),
            text: "exact\n\n".into()
        }
    );
    assert!(matches!(&code.nodes[&code.pages[0].blocks[0].key].occurrence, Occurrence::Block { inactive_text_fact: Some(id), .. } if id == "f-x3"));
    facts.retain(|r| !["code", "lang"].contains(&r["id"].as_str().unwrap()));
    facts.iter_mut().find(|r| r["id"] == "f-t3").unwrap()["data"]["value"] = "image".into();
    facts.push(fact(
        "image",
        "e-b3",
        "block/image",
        json!({"type":"image","src":"https://example.test/img","fallback":"data:image/synthetic","width":12,"height":34}),
    ));
    facts.push(fact("alt", "e-b3", "image/alt", json!({"type":"string","value":"Alt 😀"})));
    assert_eq!(
        observe(&facts, "e-root", Uuid::nil()).unwrap().pages[0].blocks[0].kind,
        BlockKind::Image {
            src: "https://example.test/img".into(),
            fallback: Some("data:image/synthetic".into()),
            width: Some(12),
            height: Some(34),
            alt: Some("Alt 😀".into())
        }
    );
    facts.iter_mut().find(|r| r["id"] == "image").unwrap()["data"]["local"] = "local-upload".into();
    assert!(
        observe(&facts, "e-root", Uuid::nil())
            .err()
            .unwrap()
            .to_string()
            .contains("local image resource")
    );
}
