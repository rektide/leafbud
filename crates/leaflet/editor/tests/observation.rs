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
