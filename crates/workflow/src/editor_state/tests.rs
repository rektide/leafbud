use super::*;
use crate::import;
use leafspring_document::{Inline, Mark, PROFILE};
use leafspring_web::{parse_pull, parse_rpc};
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
        |d: &mut Document| d.state.nodes.values_mut().next().unwrap().entity.push('!'),
        |d: &mut Document| {
            let key = d.pages[0].blocks[0].key.clone();
            d.state.nodes.get_mut(&key).unwrap().text_fact = Some("different-fact".into());
        },
        |d: &mut Document| {
            d.state
                .nodes
                .values_mut()
                .next()
                .unwrap()
                .memberships
                .insert("new-occurrence".into(), "new-occurrence".into())
                .map(|_| ())
                .unwrap_or(())
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
fn classification_keeps_legacy_uncertainty_and_true_both_sided_changes() {
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
    local.state.sync.profile = PROFILE.into();
    local.state.sync.local_hash = import::hash(&local.semantic_value());
    assert_eq!(classify(&local, &remote, "remote-changed").baseline, "legacy-unscoped");
    remote.title.push('!');
    assert_eq!(classify(&local, &remote, "remote-changed").state, "unverified");
    local.extra.insert("localOnly".into(), json!(true));
    assert_eq!(local_change(&local), Change::Unknown);
}

#[test]
fn opaque_remote_evidence_survives_mapping_refusal_and_ignores_transport_only_fields() {
    let facts = vec![json!({"id":"a","entity":"b","attribute":"future/opaque","data":{"unknown":"a"},"version":1})];
    let extras = BTreeMap::new();
    let base = leafspring_editor::witness(&facts, &extras, json!({}));
    let mut changed = facts.clone();
    changed[0]["version"] = json!(2);
    changed[0]["created_at"] = json!("ignored");
    assert_eq!(base, leafspring_editor::witness(&changed, &extras, json!({})));
    changed[0]["data"]["unknown"] = json!("b");
    assert_ne!(base, leafspring_editor::witness(&changed, &extras, json!({})));
    changed[0]["future-field"] = json!(true);
    assert_ne!(base, leafspring_editor::witness(&changed, &extras, json!({})));
    assert!(leafspring_editor::observe(&changed, "b", uuid::Uuid::nil()).is_err());
}
