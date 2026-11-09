use super::*;
use crate::import;
use leafbud_document::{Inline, Mark, Node, Occurrence};
use leafbud_web::{parse_pull, parse_rpc};
use serde_json::{Value, json};

fn document() -> Document {
    let fixture: Value = serde_json::from_str(include_str!("../../../../experiments/02-draft-internals/fixtures/export-pull-doc.json")).unwrap();
    let data = parse_rpc(&json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()).unwrap();
    import::build("https://leaflet.pub", &data, &parse_pull(fixture["response"].clone()).unwrap()).unwrap()
}

#[test]
fn scoped_inputs_ignore_local_only_intent_and_spelling_not_editor_meaning() {
    let base = document();
    let mut local = base.clone();
    local.extra.insert("publishedAt".into(), json!("2030-01-01"));
    local.extra.insert("custom".into(), json!({"nested":[1,2]}));
    let key = local.pages[0].blocks[0].key.clone();
    let node = local.state.nodes.remove(&key).unwrap();
    local.state.nodes.insert("author-friendly".into(), node);
    local.pages[0].blocks[0].key = "author-friendly".into();
    assert_ne!(local.semantic_value(), base.semantic_value(), "full fidelity still includes author fields/keys");
    assert_eq!(local.editor_value(), base.editor_value());
    assert_eq!(local_change(&local), Change::Unchanged);
    for mutate in [
        |d: &mut Document| d.title.push('!'),
        |d: &mut Document| d.extra.insert("contributors".into(), json!(["did:plc:other"])).map(|_| ()).unwrap_or(()),
        |d: &mut Document| d.pages[0].blocks.reverse(),
        |d: &mut Document| {
            let key = d.pages[0].blocks[0].key.clone();
            match &mut d.state.nodes.get_mut(&key).unwrap().occurrence {
                Occurrence::Block { placement, .. } => placement.push('!'),
                Occurrence::Page { .. } => panic!("block occurrence"),
            }
        },
        |d: &mut Document| {
            let key = d.pages[0].blocks[0].key.clone();
            match &mut d.state.nodes.get_mut(&key).unwrap().occurrence {
                Occurrence::Block { text_fact, .. } => *text_fact = Some("different-fact".into()),
                Occurrence::Page { .. } => panic!("block occurrence"),
            }
        },
    ] {
        let mut changed = base.clone();
        mutate(&mut changed);
        assert_ne!(changed.editor_value(), base.editor_value());
        assert_eq!(local_change(&changed), Change::Changed);
    }
    for mark in [false, true] {
        let mut changed = base.clone();
        let BlockKind::Heading { content, .. } = &mut changed.pages[0].blocks[0].kind else {
            panic!("fixture heading")
        };
        let Inline::Run { text, marks } = &mut content[0] else {
            panic!("fixture text")
        };
        if mark {
            marks.push(Mark::Underline);
        } else {
            text.push('!');
        }
        assert_eq!(local_change(&changed), Change::Changed);
    }
}

#[test]
fn classification_keeps_true_both_sided_changes_and_receipt_uncertainty() {
    let base = document();
    let mut local = base.clone();
    local.extra.insert("custom".into(), json!(true));
    let mut remote = base.clone();
    remote.title.push('!');
    let a = classify(&local, &remote, "remote-changed");
    assert_eq!((a.state, a.local, a.remote), ("remote-changed", Change::Unchanged, Change::Changed));
    local.title.push('?');
    assert_eq!(classify(&local, &remote, "remote-changed").state, "both-changed");
    local.title = remote.title.clone();
    assert_eq!(classify(&local, &remote, "remote-changed").state, "converged");
    assert_eq!(classify(&local, &remote, "remote-changed").baseline, "scoped");
    // A locally retired association (row without content) is a local change.
    let mut retired = base.clone();
    retired.pages[0].blocks.clear();
    assert_eq!(local_change(&retired), Change::Changed);
    let value = retired.editor_value();
    assert_eq!(value["retired"].as_array().map(Vec::len), Some(2), "both block rows retire");
    assert!(
        value["retired"][0]["entity"].is_string(),
        "retirements carry their locator, not a null identity"
    );
}

#[test]
fn opaque_remote_evidence_survives_mapping_refusal_and_ignores_transport_only_fields() {
    let facts = vec![json!({"id":"a","entity":"b","attribute":"future/opaque","data":{"unknown":"a"},"version":1})];
    let extras = BTreeMap::new();
    let base = leafbud_editor::witness(&facts, &extras, json!({}));
    let mut changed = facts.clone();
    changed[0]["version"] = json!(2);
    changed[0]["created_at"] = json!("ignored");
    assert_eq!(base, leafbud_editor::witness(&changed, &extras, json!({})));
    changed[0]["data"]["unknown"] = json!("b");
    assert_ne!(base, leafbud_editor::witness(&changed, &extras, json!({})));
    changed[0]["future-field"] = json!(true);
    assert_ne!(base, leafbud_editor::witness(&changed, &extras, json!({})));
    assert!(leafbud_editor::observe(&changed, "b", uuid::Uuid::nil()).is_err());
}

#[test]
fn page_definition_layout_and_page_key_spelling_are_not_remote_content_edits() {
    let mut base = document();
    for suffix in ["one", "two"] {
        let key = format!("page-{suffix}");
        base.state.nodes.insert(
            key.clone(),
            Node {
                entity: suffix.into(),
                occurrence: Occurrence::Page {
                    references: vec![format!("ref-{suffix}")],
                },
            },
        );
        base.pages.push(Page { key, blocks: vec![] });
    }
    let key = "link-one".to_owned();
    base.state.nodes.insert(
        key.clone(),
        Node {
            entity: "link".into(),
            occurrence: Occurrence::Block {
                placement: "placement".into(),
                text_fact: None,
                inactive_text_fact: None,
            },
        },
    );
    base.pages[0].blocks.push(Block {
        key,
        kind: BlockKind::PageLink { page: "page-one".into() },
    });
    let mut changed = base.clone();
    let node = changed.state.nodes.remove("page-one").unwrap();
    changed.state.nodes.insert("renamed-page".into(), node);
    changed.pages[1].key = "renamed-page".into();
    changed.pages[0].blocks.last_mut().unwrap().kind = BlockKind::PageLink { page: "renamed-page".into() };
    changed.pages.swap(1, 2);
    assert_eq!(base.editor_value(), changed.editor_value());
    changed.pages[0].blocks.last_mut().unwrap().kind = BlockKind::PageLink { page: "page-two".into() };
    assert_ne!(
        base.editor_value(),
        changed.editor_value(),
        "target association, not its label, is editor meaning"
    );
}

#[test]
fn malformed_receipts_never_claim_change_direction() {
    let mut local = document();
    local.state.sync.as_mut().unwrap().local_hash = "not-a-receipt".into();
    local.state.sync.as_mut().unwrap().remote_hash.clear();
    let mut remote = local.clone();
    remote.title.push('!');
    let agreement = classify(&local, &remote, "new-current-hash");
    assert_eq!(
        (agreement.state, agreement.local, agreement.remote, agreement.baseline),
        ("unverified", Change::Unknown, Change::Unknown, "invalid-receipts")
    );
    assert_eq!(
        classify(&local, &local, "new-current-hash").state,
        "converged",
        "current equality is still observable, not accepted"
    );
}

#[test]
fn unbound_and_receipt_free_documents_have_no_change_direction() {
    let mut unbound = document();
    unbound.state.binding = None;
    unbound.state.sync = None;
    assert_eq!(local_change(&unbound), Change::Unknown);
    assert_eq!(baseline(&unbound), "none");
    // Bound without receipts: coherent, still no direction from nothing.
    let mut document = document();
    document.state.sync = None;
    assert_eq!(local_change(&document), Change::Unknown);
    assert_eq!(baseline(&document), "none");
}

#[test]
fn new_native_nodes_cannot_steal_matched_author_labels_or_drop_association_rows() {
    let base = document();
    let mut local = base.clone();
    let key = local.pages[0].blocks[0].key.clone();
    let node = local.state.nodes.remove(&key).unwrap();
    local.state.nodes.insert("new-native-node".into(), node);
    local.pages[0].blocks[0].key = "new-native-node".into();
    let mut fresh = base;
    fresh.state.nodes.insert(
        "new-native-node".into(),
        Node {
            entity: "new-entity".into(),
            occurrence: Occurrence::Block {
                placement: "new-occurrence".into(),
                text_fact: None,
                inactive_text_fact: None,
            },
        },
    );
    fresh.pages[0].blocks.push(Block {
        key: "new-native-node".into(),
        kind: BlockKind::Rule,
    });
    let merged = adopt(&local, &fresh);
    assert_eq!(merged.state.nodes.len(), fresh.state.nodes.len());
    assert_eq!(merged.state.nodes["new-native-node"], local.state.nodes["new-native-node"]);
    assert_eq!(merged.pages[0].blocks.last().unwrap().key, "new-native-node-imported-2");
    assert_eq!(merged.editor_value(), fresh.editor_value());
}

#[test]
fn locally_created_uuids_survive_observation_without_import_derivation() {
    let base = document();
    let mut local = base.clone();
    // A locally created document keeps its random UUID after binding.
    local.state.id = uuid::Uuid::new_v4();
    let merged = adopt(&local, &base);
    assert_eq!(merged.state.id, local.state.id, "adoption preserves the local document UUID");
    // Identity checks compare the native locator group, not state.id.
    let binding = base.state.binding.clone().expect("bound");
    assert!(super::same_identity(&binding, &base).is_ok());
    let mut moved = base.clone();
    moved.state.binding.as_mut().unwrap().root = "other-root".into();
    let moved_binding = moved.state.binding.clone().unwrap();
    assert!(super::same_identity(&moved_binding, &base).is_err());
}
