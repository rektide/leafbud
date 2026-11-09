use super::*;

#[test]
fn description_clear_equivalence_is_semantic_not_authored_or_raw_witness_equality() {
    let fixture = old::fixture();
    let data = old::data(&fixture);
    let view = leafbud_web::parse_pull(fixture["response"].clone()).unwrap();
    let mut absent = import::build("https://leaflet.pub", &data, &view).unwrap();
    absent.description = None;
    let mut empty = absent.clone();
    empty.description = Some(String::new());
    assert_eq!(absent.editor_value(), empty.editor_value());
    assert_eq!(absent.semantic_value(), empty.semantic_value());
    assert_ne!(absent.authored_value(), empty.authored_value());
    assert_eq!(
        leafbud_mdx::render_comparison(&absent).unwrap(),
        leafbud_mdx::render_comparison(&empty).unwrap()
    );
    for content in [" ", "\n", "\t", "description", " description "] {
        let mut nonempty = absent.clone();
        nonempty.description = Some(content.into());
        assert_eq!(nonempty.editor_value()["description"], content);
        assert_ne!(nonempty.editor_value(), absent.editor_value());
        assert_ne!(
            leafbud_mdx::render_comparison(&nonempty).unwrap(),
            leafbud_mdx::render_comparison(&absent).unwrap()
        );
    }
    for raw in [Value::Null, json!(""), json!(" "), json!("description")] {
        let mut raw_data = fixture["leaflet_data"].clone();
        raw_data["description"] = raw.clone();
        let raw_data = leafbud_web::parse_rpc(&json!({"result":{"error":null,"data":raw_data}}).to_string()).unwrap();
        let fresh = import::build("https://leaflet.pub", &raw_data, &view).unwrap();
        assert_eq!(fresh.description.as_deref(), raw.as_str().filter(|s| !s.is_empty()));
        if raw == "" {
            assert_ne!(
                import::remote_hash(&raw_data, &view),
                import::remote_hash(&data, &view),
                "empty native metadata stays in conservative evidence"
            );
        }
    }
}

#[test]
fn explicit_native_publication_clear_does_not_fall_back_to_stale_standalone_description() {
    let mut fixture = old::fixture();
    fixture["leaflet_data"]["leaflets_in_publications"] = json!([{"publication":URI,"doc":null}]);
    let mut data = old::data(&fixture);
    data.metadata.description = Some("stale standalone value".into());
    let view = leafbud_web::parse_pull(fixture["response"].clone()).unwrap();
    for raw in [Value::Null, json!(""), json!(" "), json!("exact publication value")] {
        let mut view = view.clone();
        view.extras.insert("publication_description".into(), raw.clone());
        let fresh = import::build("https://leaflet.pub", &data, &view).unwrap();
        assert_eq!(fresh.description.as_deref(), raw.as_str().filter(|s| !s.is_empty()));
    }
}

#[tokio::test]
async fn clear_representations_preserve_author_bytes_through_status_witness_refresh_and_noop() {
    for author in ["", "description: null # explicit null\n", "description: '' # explicit empty\n"] {
        for native in [None, Some(Value::Null), Some(json!(""))] {
            let server = EffectServer::new(0, None).await;
            server.fixture.lock().unwrap()["leaflet_data"]["title"] = "Wanted title".into();
            server.fixture.lock().unwrap()["leaflet_data"]["description"] = Value::Null;
            let temp = tempfile::tempdir().unwrap();
            let path = server.seed(temp.path());
            let source = std::fs::read_to_string(&path).unwrap().replacen("---\n", &format!("---\n{author}"), 1);
            std::fs::write(&path, &source).unwrap();
            let before = leafbud_mdx::parse(&source).unwrap();
            // Deliberately target the pure adoption adapter with an equivalent
            // canonical None as well, not only the workflow's preserved clone.
            let mut canonical = before.clone();
            canonical.description = None;
            assert_eq!(leafbud_mdx::adopt_remote(&source, &canonical).unwrap(), source);
            {
                let mut fixture = server.fixture.lock().unwrap();
                match native {
                    Some(ref value) => fixture["leaflet_data"]["description"] = value.clone(),
                    None => {
                        fixture["leaflet_data"].as_object_mut().unwrap().remove("description");
                    }
                }
            }
            let reader = client(&server.server);
            let context = old::context(temp.path());
            let observed = reader.observe_editor(&before).await.unwrap();
            assert!(observed.agreement.endpoints_equal);
            assert_eq!(observed.merged.description, before.description);
            assert_eq!(
                leafbud_mdx::render_comparison(&before).unwrap(),
                leafbud_mdx::render_comparison(&observed.merged).unwrap()
            );
            let report = reader.refresh_document(&context, &path, Default::default()).await;
            assert!(
                matches!(report.outcome, crate::refresh::Outcome::Unchanged | crate::refresh::Outcome::ReceiptsOnly),
                "{report:?}"
            );
            if native == Some(json!("")) {
                assert_eq!(report.outcome, crate::refresh::Outcome::ReceiptsOnly);
            }
            let after = std::fs::read_to_string(&path).unwrap();
            if !author.is_empty() {
                assert!(after.contains(author));
            }
            let parsed = leafbud_mdx::parse(&after).unwrap();
            assert_eq!(parsed.authored_value(), before.authored_value());
            assert_eq!(parsed.description, before.description);
            assert_eq!(after.split("---\n").nth(2), source.split("---\n").nth(2));
            let status = crate::status::inspect(&context, crate::status::Request { files: vec![path.clone()] }, Some(&reader))
                .await
                .unwrap();
            assert_eq!(status.items[0].outcome, "clean");
            let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
            assert_eq!(
                reader.refresh_document(&context, &path, Default::default()).await.outcome,
                crate::refresh::Outcome::Unchanged
            );
            let result = reader
                .execute_compiled(&context, &path, Selection::default(), None, |_, _, _| {
                    Ok(write::Compiled {
                        mutations: vec![],
                        targets: Default::default(),
                        created_entities: vec![],
                        allocated_facts: vec![],
                    })
                })
                .await
                .unwrap();
            assert_eq!(result.outcome, write::RecoveryOutcome::DesiredVerified);
            assert_eq!(server.pushes.load(Ordering::SeqCst), 0);
            assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
            assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
        }
    }
}

#[tokio::test]
async fn source_omission_requires_actual_context_specific_clear_of_nonempty_native_description() {
    for publication in [false, true] {
        let server = EffectServer::new(0, None).await;
        {
            let mut fixture = server.fixture.lock().unwrap();
            fixture["leaflet_data"]["title"] = "Wanted title".into();
            fixture["leaflet_data"]["description"] = "old description".into();
            if publication {
                fixture["leaflet_data"]["leaflets_in_publications"] = json!([{"publication":URI,"doc":null}]);
                fixture["response"]["patch"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"op":"put","key":"publication_description","value":"old description"}));
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let path = server.seed(temp.path());
        let mut doc = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        doc.description = None;
        std::fs::write(&path, leafbud_mdx::render(&doc).unwrap()).unwrap();
        let reader = client(&server.server);
        let context = old::context(temp.path());
        // An empty compiler cannot claim omission preserves the old string.
        assert!(
            reader
                .execute_compiled(&context, &path, Selection::default(), None, |_, _, _| Ok(write::Compiled {
                    mutations: vec![],
                    targets: Default::default(),
                    created_entities: vec![],
                    allocated_facts: vec![]
                }))
                .await
                .is_err()
        );
        assert_eq!(server.pushes.load(Ordering::SeqCst), 0);
        let result = reader
            .execute_compiled(&context, &path, Selection::default(), None, |_, _, _| {
                Ok(write::Compiled {
                    mutations: vec![if publication {
                        NativeMutation::UpdatePublicationDraft(leafbud_protocol::PublicationMetadata {
                            description: Some(String::new()),
                            ..Default::default()
                        })
                    } else {
                        NativeMutation::UpdateLeafletMetadata(LeafletMetadata {
                            description: Some(None),
                            ..Default::default()
                        })
                    }],
                    targets: Default::default(),
                    created_entities: vec![],
                    allocated_facts: vec![],
                })
            })
            .await
            .unwrap();
        assert_eq!(result.outcome, write::RecoveryOutcome::DesiredVerified, "{result:?}");
        let request = server.server.requests().into_iter().find(|r| r.path == "/api/rpc/push").unwrap().body_json();
        assert!(request["pushRequest"]["mutations"][0]["args"].get("description").is_some());
        assert_eq!(
            request["pushRequest"]["mutations"][0]["args"]["description"],
            if publication { json!("") } else { Value::Null }
        );
        assert_eq!(server.pushes.load(Ordering::SeqCst), 1);
        let source = std::fs::read_to_string(&path).unwrap();
        let after = leafbud_mdx::parse(&source).unwrap();
        assert!(after.description.is_none() && after.state.pending.is_none());
        assert_eq!(after.authored_value(), doc.authored_value());
        assert!(reader.observe_editor(&after).await.unwrap().agreement.endpoints_equal);
    }
}

#[tokio::test]
async fn legacy_explicit_empty_pending_hash_is_preserved_not_automatically_reinterpreted() {
    let server = EffectServer::new(0, None).await;
    let temp = tempfile::tempdir().unwrap();
    let path = server.seed(temp.path());
    let source = std::fs::read_to_string(&path).unwrap();
    let mut doc = leafbud_mdx::parse(&source).unwrap();
    doc.description = Some(String::new());
    let source = leafbud_mdx::render(&doc).unwrap();
    let mut legacy = doc.editor_value();
    legacy["description"] = json!("");
    let mut writer = leafbud_document::WriterState::fresh();
    let range = writer.reserve(1).unwrap();
    doc.state.writer = Some(writer);
    doc.state.pending = Some(Attempt {
        id: Uuid::new_v4(),
        input_hash: import::hash(&doc.authored_value()),
        desired_hash: import::hash(&legacy),
        effect: Effect::Push {
            range,
            stage: PushStage::Partial { through: range.last },
            protection: leafbud_document::Protection {
                hash: import::hash(&json!(null)),
                facts: vec![],
                entities: vec![],
                metadata_fields: vec![],
            },
            targets: Default::default(),
            created_entities: vec![],
        },
    });
    let source = leafbud_mdx::patch_tracking(&source, &doc.state).unwrap();
    std::fs::write(&path, &source).unwrap();
    let result = client(&server.server)
        .recover_native(&old::context(temp.path()), &path, Selection::default(), true)
        .await
        .unwrap();
    assert_eq!(result.outcome, write::RecoveryOutcome::SourceChanged);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    assert!(server.server.requests().is_empty());
}
