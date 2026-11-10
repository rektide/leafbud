use super::*;
use serde_json::json;

fn setup() -> (Target, Writer, WriteScope) {
    let target = Target {
        token: Uuid::new_v4(),
        root: Uuid::new_v4(),
        entity_set: Uuid::new_v4(),
    };
    let writer = Writer {
        client_group_id: Uuid::new_v4(),
        client_id: Uuid::new_v4(),
    };
    let scope = WriteScope {
        entities: [target.root].into(),
        ..Default::default()
    };
    (target, writer, scope)
}

#[test]
fn native_envelope_order_count_bytes_and_all_request_validation() {
    let (target, writer, scope) = setup();
    let mutation = NativeMutation::UpdateLeafletMetadata(LeafletMetadata {
        title: Some("Author title".into()),
        description: Some(None),
    });
    let rows = vec![mutation.clone(); 501];
    let batches = encode_pushes(target, writer, 19, 42, &rows, &scope, BatchLimits::default()).unwrap();
    assert_eq!(
        batches.iter().map(|r| (r.first(), r.last())).collect::<Vec<_>>(),
        [(19, 268), (269, 518), (519, 519)]
    );
    let body: Value = serde_json::from_slice(batches[0].bytes()).unwrap();
    assert_eq!(body["token"]["id"], target.token.to_string());
    assert_eq!(body["pushRequest"]["clientGroupID"], writer.client_group_id.to_string());
    assert_eq!(
        body["pushRequest"]["mutations"][0],
        json!({"id":19,"name":"updateLeafletMetadata","args":{"title":"Author title","description":null},"timestamp":42,"clientID":writer.client_id})
    );
    assert_eq!(batches[0].clone().bytes(), batches[0].bytes());
    let small = BatchLimits {
        bytes: 600,
        ..Default::default()
    };
    assert!(encode_pushes(target, writer, 1, 0, &rows, &scope, small).is_err()); // >128 requests, not a sent prefix
    let limited = BatchLimits {
        bytes: 600,
        requests: 1000,
        ..Default::default()
    };
    assert!(
        encode_pushes(target, writer, 1, 0, &rows, &scope, limited)
            .unwrap()
            .iter()
            .all(|b| b.bytes().len() <= 600)
    );
    let mut invalid = rows;
    invalid.push(NativeMutation::CreateEntity(vec![CreateEntity {
        entity_id: target.root,
        permission_set: Uuid::new_v4(),
    }]));
    assert!(encode_pushes(target, writer, 1, 0, &invalid, &scope, BatchLimits::default()).is_err());
    assert!(
        encode_pushes(
            target,
            writer,
            MAX_SAFE_INTEGER,
            0,
            &[mutation.clone(), mutation],
            &scope,
            BatchLimits::default()
        )
        .is_err()
    );
}

#[test]
fn target_reference_fact_and_metadata_context_cannot_be_injected() {
    let (target, writer, scope) = setup();
    let invalid = NativeMutation::AssertFact(AssertFact {
        id: Some(Uuid::new_v4()),
        entity: target.root,
        attribute: "root/page".into(),
        data: json!({"type":"ordered-reference","value":Uuid::new_v4(),"position":"a0"}),
    });
    assert!(encode_pushes(target, writer, 1, 0, &[invalid], &scope, BatchLimits::default()).is_err());
    let metadata = NativeMutation::UpdatePublicationDraft(PublicationMetadata {
        title: Some("Wrong storage".into()),
        ..Default::default()
    });
    assert!(metadata.validate(target, &scope).is_err());
    let mut scope = scope;
    scope.publication = true;
    assert!(metadata.validate(target, &scope).is_ok());
    assert!(serde_json::from_value::<NativeMutation>(json!({"name":"deleteEntity","args":target.root})).is_err());
    assert!(serde_json::from_value::<NativeMutation>(json!({"name":"updatePublicationDraft","args":{"contributors":[]}})).is_err());
}

#[test]
fn publication_preferences_wire_full_replacement_clear_and_unknown_refusal() {
    let (target, writer, mut scope) = setup();
    scope.publication = true;
    for args in [
        json!({"preferences":{}}),
        json!({"preferences":{"showComments":false,"showMentions":true,"showRecommends":false,"showInDiscover":true}}),
    ] {
        let expected = json!({"name":"updatePublicationDraft","args":args});
        let mutation: NativeMutation = serde_json::from_value(expected.clone()).unwrap();
        assert_eq!(serde_json::to_value(&mutation).unwrap(), expected);
        let batches = encode_pushes(target, writer, 1, 0, &[mutation], &scope, BatchLimits::default()).unwrap();
        let body: Value = serde_json::from_slice(batches[0].bytes()).unwrap();
        assert_eq!(body["pushRequest"]["mutations"][0]["args"], args);
    }
    for preferences in [json!({"showComments":"false"}), json!({"showComments":null}), json!({"future":true})] {
        let mutation = serde_json::from_value::<NativeMutation>(json!({"name":"updatePublicationDraft","args":{"preferences":preferences}}));
        assert!(mutation.is_err() || mutation.unwrap().validate(target, &scope).is_err());
    }
}

#[test]
fn full_view_keeps_unknown_witness_fields_and_validates_processing_evidence() {
    let (target, writer, _) = setup();
    let id = Uuid::new_v4();
    let fact = json!({"id":id,"entity":target.root,"attribute":"future/field","data":{"type":"new","opaque":true},"future_row":42});
    let raw = json!({"patch":[{"op":"clear"},{"op":"put","key":"initialized","value":true},{"op":"put","key":id,"value":fact}],"lastMutationIDChanges":{writer.client_id.to_string():42},"cookie":{"order":1}});
    let view = FullView::parse(raw.clone()).unwrap();
    assert_eq!(view.facts, [fact]);
    assert_eq!(view.raw, raw);
    assert_eq!(view.acknowledged(writer.client_id), Some(42));
    let mut invalid = raw;
    invalid["lastMutationIDChanges"][writer.client_id.to_string()] = json!(-1);
    assert!(FullView::parse(invalid).is_err());
    assert!(FullView::parse(json!({"patch":[]})).is_err());
}

#[test]
fn explicit_detached_fact_read_is_not_an_existence_proof() {
    let entity = Uuid::new_v4();
    let id = Uuid::new_v4();
    let row = json!({"root_id":entity,"id":id,"entity":entity,"attribute":"block/type","data":{"type":"block-type-union","value":"text"}});
    let absent = Uuid::new_v4();
    let explicit = ExplicitFacts::parse(json!({"result":{"facts":{entity.to_string():[row.clone(),row.clone()]}}}), &[entity, absent]).unwrap();
    assert_eq!(explicit.fact(entity, id), Some(&row));
    assert!(explicit.roots[&absent].is_empty());
    assert_eq!(explicit.roots[&entity].len(), 1);
    assert!(ExplicitFacts::parse(explicit.raw, &[absent]).is_err());
}
