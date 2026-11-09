//! First-binding defaults are source preparation, never verifier wildcards.
use super::*;

fn located(path: &Path, base: &str, publication: bool) -> String {
    let source = std::fs::read_to_string(path).unwrap();
    let keys = leafbud_mdx::prepare_keys(
        &source,
        leafbud_mdx::KeySource {
            block: &mut || "b-authored".into(),
            page: &mut || "p-authored".into(),
        },
    )
    .unwrap();
    let source = leafbud_mdx::apply_key_preparation(&source, &keys).unwrap();
    let mut doc = leafbud_mdx::parse(&source).unwrap();
    doc.state.pending = Some(Attempt {
        id: Uuid::new_v4(),
        input_hash: import::hash(&doc.authored_value()),
        desired_hash: import::hash(&doc.editor_value()),
        effect: Effect::Mint {
            base_url: base.into(),
            publication: if publication {
                MintContext::Publication(URI.into())
            } else {
                MintContext::Standalone
            },
            stage: MintStage::Located { token: TOKEN.parse().unwrap() },
        },
    });
    leafbud_mdx::patch_tracking(&source, &doc.state).unwrap()
}

fn verified(base: &str, publication: bool) -> Document {
    let (data, view) = seed(publication);
    let data = leafbud_web::parse_rpc(&json!({"result":{"error":null,"data":data}}).to_string()).unwrap();
    import::build(base, &data, &leafbud_web::parse_pull(view).unwrap()).unwrap()
}

#[test]
fn narrow_defaults_patch_preserves_explicit_values_trivia_body_and_tracking_guards() {
    for explicit in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("draft.mdx");
        draft(
            &path,
            if explicit {
                "publication: null\ntheme: {leafletWatermark: false} # explicit false\ncontributors: [] # explicit empty"
            } else {
                ""
            },
        );
        let text = std::fs::read_to_string(&path).unwrap().replace(
            "title: Desired title",
            "# author comment\ntitle: 'Desired title' # spelling\ncustom: {chosen: true}\ndate: 2026-11-09",
        );
        std::fs::write(&path, text).unwrap();
        let text = located(&path, "https://leaflet.pub", false).replace('\n', "\r\n");
        let text = format!("\u{feff}{text}");
        let original = leafbud_mdx::parse(&text).unwrap();
        let mut remote = verified("https://leaflet.pub", false);
        remote.extra.insert("contributors".into(), json!(["did:plc:actual-observed"]));
        let prepared = leafbud_mdx::prepare_binding_defaults(&text, &remote).unwrap();
        let after = leafbud_mdx::parse(&prepared).unwrap();
        assert!(prepared.starts_with("\u{feff}---\r\n"));
        assert!(prepared.contains("# author comment\r\ntitle: 'Desired title' # spelling\r\ncustom: {chosen: true}\r\ndate: 2026-11-09\r\n"));
        assert_eq!(prepared.split("---\r\n").nth(2), text.split("---\r\n").nth(2));
        assert_eq!(after.pages, original.pages);
        assert_eq!(after.state, original.state, "pure preparation does not settle tracking");
        assert_eq!(after.publication, Publication::Standalone);
        assert_eq!(after.theme.unwrap().leaflet_watermark, !explicit);
        assert_eq!(
            after.extra["contributors"],
            if explicit { json!([]) } else { json!(["did:plc:actual-observed"]) }
        );
        if explicit {
            assert!(prepared.contains("theme: {leafletWatermark: false} # explicit false\r\ncontributors: [] # explicit empty\r\n"));
        }
        assert_eq!(leafbud_mdx::prepare_binding_defaults(&prepared, &remote).unwrap(), prepared);
        assert!(
            leafbud_mdx::patch_tracking(&text, &remote.state).is_err(),
            "ordinary tracking cannot adopt fields or change UUID/main page"
        );
        let mut retarget = remote.clone();
        retarget.state.binding.as_mut().unwrap().publication = Some(URI.into());
        assert!(leafbud_mdx::prepare_binding_defaults(&text, &retarget).is_err());
    }
}

#[tokio::test]
async fn explicit_nulls_are_not_omission_and_fail_before_authentication_or_mint() {
    for fields in ["theme: null", "contributors: null", "contributors: not-a-list"] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("draft.mdx");
        draft(&path, fields);
        let before = std::fs::read(&path).unwrap();
        let server = TestServer::spawn(Arc::new(|_| panic!("invalid explicit field must fail before network"))).await;
        let error = client(&server)
            .stage_binding(&old::context(temp.path()), &path, Selection::default())
            .await
            .unwrap_err();
        assert!(format!("{error:#}").contains(if fields.starts_with("theme") { "theme null" } else { "contributors" }));
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(server.requests().is_empty());
    }
}

#[tokio::test]
async fn located_mint_recovery_atomically_prepares_defaults_once_and_keeps_previous_intent_on_race() {
    for publication in [false, true] {
        for race in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("draft.mdx");
            draft(&path, "");
            let (data, view) = seed(publication);
            let changed = path.clone();
            let server = TestServer::spawn(Arc::new(move |r| match r.path.as_str() {
                "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":data}}).to_string()),
                "/api/rpc/pull" => {
                    if race {
                        let text = std::fs::read_to_string(&changed).unwrap();
                        std::fs::write(&changed, text.replace("Desired prose", "New author prose")).unwrap();
                    }
                    (200, view.to_string())
                }
                _ => panic!("recovery never remints"),
            }))
            .await;
            let before = located(&path, &server.base_url, publication);
            std::fs::write(&path, &before).unwrap();
            let old_attempt = leafbud_mdx::parse(&before).unwrap().state.pending.unwrap();
            let reader = client(&server);
            let context = old::context(temp.path());
            // Read-only inspection is not permission to normalize source.
            if !race {
                reader.recover_native(&context, &path, Selection::default(), false).await.unwrap();
                assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
            }
            let result = reader.recover_native(&context, &path, Selection::default(), true).await;
            let after = std::fs::read_to_string(&path).unwrap();
            let doc = leafbud_mdx::parse(&after).unwrap();
            if race {
                assert!(result.is_err());
                assert!(after.contains("New author prose"));
                assert!(doc.theme.is_none() && doc.state.binding.is_none());
                assert_eq!(doc.state.pending.as_ref().unwrap(), &old_attempt);
                assert_eq!(
                    reader.recover_native(&context, &path, Selection::default(), true).await.unwrap().outcome,
                    write::RecoveryOutcome::SourceChanged
                );
            } else {
                assert_eq!(result.unwrap().outcome, write::RecoveryOutcome::PendingMaterialization);
                assert!(doc.theme.unwrap().leaflet_watermark);
                assert!(doc.state.binding.is_some());
                let attempt = doc.state.pending.as_ref().unwrap();
                assert_eq!(attempt.id, old_attempt.id);
                assert_eq!(attempt.effect, old_attempt.effect);
                assert_ne!(attempt.input_hash, old_attempt.input_hash);
                assert_eq!(attempt.input_hash, import::hash(&leafbud_mdx::parse(&after).unwrap().authored_value()));
                assert_eq!(attempt.desired_hash, import::hash(&leafbud_mdx::parse(&after).unwrap().editor_value()));
                let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
                reader.recover_native(&context, &path, Selection::default(), true).await.unwrap();
                assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
                assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
            }
        }
    }
}

#[test]
fn checkpoint_boundary_never_persists_new_fields_with_old_hashes_or_accepts_push_as_mint() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("draft.mdx");
    draft(&path, "");
    let before = located(&path, "https://leaflet.pub", true);
    std::fs::write(&path, &before).unwrap();
    let context = old::context(temp.path());
    let verified = verified("https://leaflet.pub", true);
    {
        let source = Source::open(&context, &path, Selection::default()).unwrap();
        // A crash while computing the proposed source leaves the recoverable
        // located mint untouched, including its original input hashes.
        let _uncommitted = leafbud_mdx::prepare_binding_defaults(&source.text, &verified).unwrap();
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    let mut source = Source::open(&context, &path, Selection::default()).unwrap();
    source.prepare_binding(&verified).unwrap();
    let after = source.text.clone();
    let attempt = source.document.state.pending.as_ref().unwrap();
    assert_eq!(attempt.input_hash, import::hash(&source.document.authored_value()));
    assert_eq!(attempt.desired_hash, import::hash(&source.document.editor_value()));
    // Simulate the subsequent persisted content-write phase. The defaults API
    // cannot reclassify an already-dispatched push as pre-write preparation.
    let mut state = source.document.state.clone();
    let mut writer = leafbud_document::WriterState::fresh();
    let range = writer.reserve(1).unwrap();
    state.writer = Some(writer);
    state.pending.as_mut().unwrap().effect = Effect::Push {
        range,
        stage: PushStage::Dispatched { through: range.last },
        targets: Default::default(),
        created_entities: vec![],
        protection: leafbud_document::Protection {
            hash: import::hash(&json!(null)),
            facts: vec![],
            entities: vec![],
            metadata_fields: vec![],
        },
    };
    let pushed = leafbud_mdx::patch_tracking(&after, &state).unwrap();
    assert!(leafbud_mdx::prepare_binding_defaults(&pushed, &verified).is_err());
    // Adjacent source race aborts the full checkpoint, not just its hash patch.
    std::fs::write(&path, after.replace("Desired prose", "New author prose")).unwrap();
    assert!(source.prepare_binding(&verified).is_err());
    assert!(std::fs::read_to_string(&path).unwrap().contains("New author prose"));
}

#[tokio::test]
async fn inherited_publication_and_defaults_survive_first_materialization_restart_status_pull_and_noop() {
    for (publication, explicit_theme, selected_locator) in [
        (false, None, false),
        (true, None, false),
        (false, Some(false), false),
        (true, Some(false), false),
        (false, Some(true), false),
        (true, Some(true), false),
        (false, None, true),
        (true, None, true),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("draft.mdx");
        let id = draft(&path, "# omitted fields adopt verified native defaults\nlocalOnly: {keep: true}");
        if let Some(value) = explicit_theme {
            let text = std::fs::read_to_string(&path)
                .unwrap()
                .replace("localOnly:", &format!("theme: {{leafletWatermark: {value}}} # explicit theme\nlocalOnly:"));
            std::fs::write(&path, text).unwrap();
        }
        if !publication {
            let text = std::fs::read_to_string(&path)
                .unwrap()
                .replace("Desired prose, untouched.", "# Desired prose, untouched.");
            std::fs::write(&path, text).unwrap();
        }
        let source_before = std::fs::read_to_string(&path).unwrap();
        let (data, view) = seed(publication);
        let fixture = Arc::new(Mutex::new((data, view)));
        let remote = fixture.clone();
        let ack = Arc::new(Mutex::new(None::<(String, u64)>));
        let acks = ack.clone();
        let pushes = Arc::new(AtomicUsize::new(0));
        let push_count = pushes.clone();
        let comment_race = Arc::new(AtomicUsize::new(0));
        let race = comment_race.clone();
        let race_path = path.clone();
        let server = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
            "/home" => (200, vec![], home()),
            "/lish/did:plc:synthetic/garden/dashboard" if r.method == "GET" => (200, vec![], "<script src=\"/_next/static/mint.js\"></script>".into()),
            "/_next/static/mint.js" => (200, vec![], include_str!("../../../leaflet/web/tests/fixtures/mint-reference.js").into()),
            "/new" => (307, vec![("Location".into(), format!("/{TOKEN}"))], String::new()),
            "/lish/did:plc:synthetic/garden/dashboard" => (
                200,
                vec![("Content-Type".into(), "text/x-component".into())],
                include_str!("../../../leaflet/web/tests/fixtures/mint-result.flight").into(),
            ),
            "/api/rpc/get_leaflet_data" => {
                // Lose only the local read-back settlement after effects, using
                // a real across-await comment edit, not fabricated hash updates.
                if race.compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    let text = std::fs::read_to_string(&race_path).unwrap();
                    std::fs::write(&race_path, text.replace("localOnly:", "# author saved during read-back\nlocalOnly:")).unwrap();
                }
                (200, vec![], json!({"result":{"error":null,"data":remote.lock().unwrap().0}}).to_string())
            }
            "/api/rpc/pull" => {
                let mut view = remote.lock().unwrap().1.clone();
                if let Some((client, id)) = &*acks.lock().unwrap() {
                    view["lastMutationIDChanges"] = json!({client:*id});
                }
                (200, vec![], view.to_string())
            }
            "/api/rpc/push" => {
                push_count.fetch_add(1, Ordering::SeqCst);
                let body = r.body_json();
                let mutations = body["pushRequest"]["mutations"].as_array().unwrap();
                let mut remote = remote.lock().unwrap();
                // Controlled synthetic native store, intentionally limited to
                // the known single seed fixture. Not the complete compiler.
                for mutation in mutations {
                    match mutation["name"].as_str().unwrap() {
                        "updateLeafletMetadata" => remote.0["title"] = mutation["args"]["title"].clone(),
                        "updatePublicationDraft" => remote.1["patch"]
                            .as_array_mut()
                            .unwrap()
                            .push(json!({"op":"put","key":"publication_title","value":mutation["args"]["title"]})),
                        "assertFact" => {
                            let row = &mutation["args"];
                            assert!(row["attribute"] == "block/text" || row["attribute"] == "theme/page-leaflet-watermark");
                            let patch = remote.1["patch"].as_array_mut().unwrap();
                            let update = json!({"op":"put","key":row["id"],"value":row});
                            if let Some(existing) = patch.iter_mut().find(|p| p["key"] == row["id"]) {
                                *existing = update;
                            } else {
                                patch.push(update);
                            }
                        }
                        _ => panic!("unexpected controlled write"),
                    }
                }
                let last = mutations.last().unwrap();
                *acks.lock().unwrap() = Some((last["clientID"].as_str().unwrap().into(), last["id"].as_u64().unwrap()));
                race.store(1, Ordering::SeqCst);
                (200, vec![], "{}".into())
            }
            _ => panic!("unexpected lifecycle request {}", r.path),
        }))
        .await;
        let reader = client(&server);
        let mut context = old::context(temp.path());
        context.effective.publication = if publication { Some(URI.into()) } else { None };
        let report = if selected_locator {
            let text = located(&path, &server.base_url, publication);
            let mut doc = leafbud_mdx::parse(&text).unwrap();
            if let Effect::Mint { stage, .. } = &mut doc.state.pending.as_mut().unwrap().effect {
                *stage = MintStage::Dispatched;
            }
            std::fs::write(&path, leafbud_mdx::patch_tracking(&text, &doc.state).unwrap()).unwrap();
            reader
                .select_mint_locator(&context, &path, Selection::default(), doc.state.pending.unwrap().id, TOKEN.parse().unwrap())
                .await
                .unwrap()
        } else {
            reader.stage_binding(&context, &path, Selection::default()).await.unwrap()
        };
        assert!(matches!(report.outcome, BindingOutcome::PendingMaterialization), "{report:?}");
        if selected_locator {
            assert!(report.message.as_ref().unwrap().contains("attribution is not established"));
            assert_eq!(server.request_count("/new"), 0);
            assert!(!server.requests().iter().any(|r| r.method == "POST" && r.path.ends_with("dashboard")));
        }
        let normalized = std::fs::read_to_string(&path).unwrap();
        let doc = leafbud_mdx::parse(&normalized).unwrap();
        let prepared_main = doc.state.main_page.clone();
        let attempt_id = doc.state.pending.as_ref().unwrap().id;
        assert_eq!(doc.publication, if publication { Publication::At(URI.into()) } else { Publication::Standalone });
        assert_eq!(doc.state.id, id);
        assert_eq!(doc.theme.as_ref().unwrap().leaflet_watermark, explicit_theme.unwrap_or(true));
        assert_eq!(
            doc.extra.get("contributors").cloned(),
            if publication { Some(json!(["did:plc:contributor-default"])) } else { None }
        );
        assert!(doc.description.is_none() && doc.tags.is_empty());
        assert!(normalized.contains("# omitted fields adopt verified native defaults\n"));
        assert!(normalized.contains("localOnly: {keep: true}"));
        assert!(normalized.ends_with(source_before.split("---\n").nth(2).unwrap().trim_start()));
        // Ambient configuration changes cannot retarget an already located mint.
        context.effective.publication = Some("at://did:plc:other/site.standard.publication/other".into());
        let report = reader
            .execute_compiled(&context, &path, Selection::default(), None, |desired, data, view| {
                let fresh = import::build(reader.client.base_url(), data, view)?;
                assert_eq!(desired.state.pending.as_ref().unwrap().id, attempt_id);
                assert_eq!(desired.state.pending.as_ref().unwrap().input_hash, import::hash(&desired.authored_value()));
                let text_id = Uuid::new_v4();
                let main = fresh.state.nodes[&fresh.pages[0].key].clone();
                let mut block = fresh.state.nodes[&fresh.pages[0].blocks[0].key].clone();
                block.occurrence = leafbud_document::Occurrence::Block {
                    placement: MEMBER.into(),
                    text_fact: Some(text_id.to_string()),
                    inactive_text_fact: None,
                };
                let metadata = if publication {
                    NativeMutation::UpdatePublicationDraft(leafbud_protocol::PublicationMetadata {
                        title: Some(desired.title.clone()),
                        ..Default::default()
                    })
                } else {
                    NativeMutation::UpdateLeafletMetadata(LeafletMetadata {
                        title: Some(desired.title.clone()),
                        ..Default::default()
                    })
                };
                let mut mutations = vec![
                    metadata,
                    NativeMutation::AssertFact(leafbud_protocol::AssertFact {
                        id: Some(text_id),
                        entity: BLOCK.parse()?,
                        attribute: "block/text".into(),
                        data: json!({"type":"text","value":old::yrs_payload("Desired prose, untouched.", None)}),
                    }),
                ];
                if explicit_theme == Some(false) {
                    mutations.push(NativeMutation::AssertFact(leafbud_protocol::AssertFact {
                        id: Some("99999999-9999-4999-8999-999999999999".parse()?),
                        entity: ROOT.parse()?,
                        attribute: "theme/page-leaflet-watermark".into(),
                        data: json!({"type":"boolean","value":false}),
                    }));
                }
                Ok(write::Compiled {
                    mutations,
                    targets: [(desired.pages[0].key.clone(), main), (desired.pages[0].blocks[0].key.clone(), block)].into(),
                    created_entities: vec![],
                    allocated_facts: vec![text_id],
                })
            })
            .await
            .unwrap();
        assert_eq!(report.outcome, write::RecoveryOutcome::SourceChanged);
        let after_effect = std::fs::read_to_string(&path).unwrap();
        assert!(after_effect.contains("# author saved during read-back"));
        let pending = leafbud_mdx::parse(&after_effect).unwrap();
        assert!(pending.state.sync.is_none());
        assert_eq!(pending.state.pending.as_ref().unwrap().id, attempt_id);
        assert_eq!(pending.state.pending.as_ref().unwrap().input_hash, import::hash(&pending.authored_value()));
        assert_eq!(pending.state.pending.as_ref().unwrap().desired_hash, import::hash(&pending.editor_value()));
        // Fresh reader has no retained request bytes and settles only actual
        // terminal acknowledgments + full desired/protected-scope read-back.
        let restarted = client(&server);
        let result = restarted.recover_native(&context, &path, Selection::default(), true).await.unwrap();
        assert_eq!(result.outcome, write::RecoveryOutcome::DesiredVerified, "{result:?}");
        assert!(result.settled);
        let settled = std::fs::read_to_string(&path).unwrap();
        let doc = leafbud_mdx::parse(&settled).unwrap();
        assert_eq!(doc.state.id, id);
        assert_eq!(doc.state.main_page, prepared_main);
        assert_eq!(doc.state.sync.as_ref().unwrap().local_hash, import::hash(&doc.editor_value()));
        assert!(doc.state.pending.is_none() && doc.state.nodes.len() == 2);
        let status = crate::status::inspect(&context, crate::status::Request { files: vec![path.clone()] }, Some(&restarted))
            .await
            .unwrap();
        assert_eq!(status.items[0].outcome, "clean");
        let observed = restarted.observe_editor(&doc).await.unwrap();
        assert!(observed.agreement.endpoints_equal);
        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        let pull = restarted.refresh_document(&context, &path, Default::default()).await;
        assert_eq!(pull.outcome, crate::refresh::Outcome::Unchanged, "{pull:?}");
        let no_op = restarted
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
        assert_eq!(no_op.outcome, write::RecoveryOutcome::DesiredVerified);
        assert!(no_op.settled);
        assert_eq!(pushes.load(Ordering::SeqCst), 1);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), settled);
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    }
}

#[tokio::test]
async fn explicit_fields_remain_authoritative_and_unsupported_contributor_changes_stop_before_push() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("draft.mdx");
    draft(
        &path,
        &format!("publication: {URI}\ntheme: {{leafletWatermark: false}} # authored\ncontributors: [] # intentional"),
    );
    let (data, view) = seed(true);
    let server = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
        "/home" => (200, vec![], home()),
        "/lish/did:plc:synthetic/garden/dashboard" if r.method == "GET" => (200, vec![], "<script src=\"/_next/static/mint.js\"></script>".into()),
        "/_next/static/mint.js" => (200, vec![], include_str!("../../../leaflet/web/tests/fixtures/mint-reference.js").into()),
        "/lish/did:plc:synthetic/garden/dashboard" => (
            200,
            vec![("Content-Type".into(), "text/x-component".into())],
            include_str!("../../../leaflet/web/tests/fixtures/mint-result.flight").into(),
        ),
        "/api/rpc/get_leaflet_data" => (200, vec![], json!({"result":{"error":null,"data":data}}).to_string()),
        "/api/rpc/pull" => (200, vec![], view.to_string()),
        _ => panic!("no push may run for unsupported contributor intent"),
    }))
    .await;
    let reader = client(&server);
    let context = old::context(temp.path());
    let result = reader.stage_binding(&context, &path, Selection::default()).await.unwrap();
    assert!(
        matches!(result.outcome, BindingOutcome::PendingMaterialization),
        "false is desired content, not a reason to reject a native true seed: {result:?}"
    );
    let before = std::fs::read_to_string(&path).unwrap();
    let doc = leafbud_mdx::parse(&before).unwrap();
    assert!(!doc.theme.as_ref().unwrap().leaflet_watermark);
    assert_eq!(doc.extra["contributors"], json!([]));
    assert!(before.contains("theme: {leafletWatermark: false} # authored\ncontributors: [] # intentional"));
    let error = reader
        .execute_compiled(&context, &path, Selection::default(), None, |_, _, _| {
            panic!("unsupported contributor intent precedes compiler/effects")
        })
        .await
        .unwrap_err();
    assert!(error.to_string().contains("explicit contributor changes are unsupported"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    assert!(matches!(
        doc.state.pending.unwrap().effect,
        Effect::Mint {
            stage: MintStage::Located { .. },
            ..
        }
    ));
}

#[tokio::test]
async fn observed_empty_contributors_are_recorded_without_inventing_an_identity() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("draft.mdx");
    draft(&path, "");
    let (data, mut view) = seed(true);
    view["patch"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|p| p["key"] == "draft_contributors")
        .unwrap()["value"] = json!([]);
    let server = TestServer::spawn(Arc::new(move |r| match r.path.as_str() {
        "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":data}}).to_string()),
        "/api/rpc/pull" => (200, view.to_string()),
        _ => panic!("located recovery never mints"),
    }))
    .await;
    std::fs::write(&path, located(&path, &server.base_url, true)).unwrap();
    let reader = client(&server);
    let context = old::context(temp.path());
    reader.recover_native(&context, &path, Selection::default(), true).await.unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let doc = leafbud_mdx::parse(&text).unwrap();
    assert_eq!(doc.extra["contributors"], json!([]));
    assert!(text.contains("\"contributors\": []"));
    reader.recover_native(&context, &path, Selection::default(), true).await.unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
}

#[tokio::test]
async fn mint_verification_source_edit_or_external_duplicate_preserves_located_attempt_without_defaults() {
    for duplicate in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let path = temp.path().join("draft.mdx");
        let other = external.path().join("other.mdx");
        let id = draft(&path, "");
        draft(&other, "");
        let (data, view) = seed(false);
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let changed = path.clone();
        let other_changed = other.clone();
        let server = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
            "/home" => (200, vec![], home()),
            "/new" => (307, vec![("Location".into(), format!("/{TOKEN}"))], String::new()),
            "/api/rpc/pull" => (200, vec![], view.to_string()),
            "/api/rpc/get_leaflet_data" => {
                if counter.fetch_add(1, Ordering::SeqCst) == 1 {
                    if duplicate {
                        let text = std::fs::read_to_string(&other_changed).unwrap();
                        let mut doc = leafbud_mdx::parse(&text).unwrap();
                        doc.state.id = id;
                        std::fs::write(&other_changed, leafbud_mdx::render(&doc).unwrap()).unwrap();
                    } else {
                        let text = std::fs::read_to_string(&changed).unwrap();
                        std::fs::write(&changed, text.replace("Desired prose", "New author prose")).unwrap();
                    }
                }
                (200, vec![], json!({"result":{"error":null,"data":data}}).to_string())
            }
            _ => panic!("unexpected request"),
        }))
        .await;
        let result = client(&server)
            .stage_binding(
                &old::context(temp.path()),
                &path,
                Selection {
                    operands: vec![path.clone(), other],
                },
            )
            .await
            .unwrap();
        assert!(matches!(result.outcome, BindingOutcome::SourceChanged));
        assert!(result.locator_persisted);
        let text = std::fs::read_to_string(&path).unwrap();
        let doc = leafbud_mdx::parse(&text).unwrap();
        assert!(doc.theme.is_none() && doc.state.binding.is_none());
        assert_eq!(doc.publication, Publication::Inherit);
        assert!(matches!(
            doc.state.pending.unwrap().effect,
            Effect::Mint {
                stage: MintStage::Located { .. },
                ..
            }
        ));
        if !duplicate {
            assert!(text.contains("New author prose"));
        }
        assert_eq!(server.request_count("/new"), 1);
    }
}

#[tokio::test]
async fn old_acknowledged_partial_push_does_not_acquire_new_defaults_or_rewrite_old_intent() {
    let server = EffectServer::new(1, None).await;
    let temp = tempfile::tempdir().unwrap();
    let path = server.seed(temp.path());
    let context = old::context(temp.path());
    // Recreate the older policy's omitted theme before its dispatched push.
    let text = std::fs::read_to_string(&path).unwrap();
    let text = text
        .lines()
        .filter(|line| !line.contains("theme") && !line.contains("leafletWatermark"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&path, &text).unwrap();
    let reader = client(&server.server);
    let report = reader
        .execute_compiled(&context, &path, Selection::default(), None, metadata_compiler(1))
        .await
        .unwrap();
    assert_eq!(report.outcome, write::RecoveryOutcome::ResolvedPartial);
    let before = std::fs::read_to_string(&path).unwrap();
    let doc = leafbud_mdx::parse(&before).unwrap();
    let attempt = doc.state.pending.as_ref().unwrap().clone();
    let mut native = verified(&server.server.base_url, false);
    native.state.binding = doc.state.binding.clone();
    assert!(leafbud_mdx::prepare_binding_defaults(&before, &native).is_err());
    assert_eq!(
        reader.recover_native(&context, &path, Selection::default(), true).await.unwrap().outcome,
        write::RecoveryOutcome::ResolvedPartial
    );
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(after, before);
    let after = leafbud_mdx::parse(&after).unwrap();
    assert_eq!(after.state.pending.as_ref().unwrap(), &attempt);
    assert!(after.theme.is_none());
    assert_eq!(after.state.sync, doc.state.sync);
    assert_eq!(server.pushes.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn explicit_locator_selection_checkpoints_before_observation_and_retains_failed_or_racing_candidate() {
    for mode in ["verified", "wrong-rights", "non-seed", "source-race"] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("draft.mdx");
        draft(&path, "");
        let (mut data, mut view) = seed(true);
        if mode == "wrong-rights" {
            data["permission_token_rights"][0]["write"] = false.into();
        }
        if mode == "non-seed" {
            view["patch"][4]["value"]["data"]["value"] = "heading".into();
        }
        let observed_path = path.clone();
        let before_observation = Arc::new(Mutex::new(None::<Attempt>));
        let captured = before_observation.clone();
        let server = TestServer::spawn(Arc::new(move |r| match r.path.as_str() {
            "/api/rpc/get_leaflet_data" => {
                let text = std::fs::read_to_string(&observed_path).unwrap();
                let doc = leafbud_mdx::parse(&text).unwrap();
                if captured.lock().unwrap().is_none() {
                    assert!(matches!(
                        doc.state.pending.as_ref().unwrap().effect,
                        Effect::Mint {
                            stage: MintStage::Located { .. },
                            ..
                        }
                    ));
                    assert!(doc.theme.is_none() && doc.state.binding.is_none());
                    *captured.lock().unwrap() = doc.state.pending;
                }
                if mode == "source-race" {
                    std::fs::write(&observed_path, text.replace("Desired prose", "New author prose")).unwrap();
                }
                (200, json!({"result":{"error":null,"data":data}}).to_string())
            }
            "/api/rpc/pull" => (200, view.to_string()),
            _ => panic!("selected locator recovery has no mint/push"),
        }))
        .await;
        let text = located(&path, &server.base_url, true);
        let mut doc = leafbud_mdx::parse(&text).unwrap();
        let attempt = doc.state.pending.as_mut().unwrap();
        if let Effect::Mint { stage, .. } = &mut attempt.effect {
            *stage = MintStage::Dispatched;
        }
        let original_attempt = attempt.clone();
        std::fs::write(&path, leafbud_mdx::patch_tracking(&text, &doc.state).unwrap()).unwrap();
        let result = client(&server)
            .select_mint_locator(
                &old::context(temp.path()),
                &path,
                Selection::default(),
                original_attempt.id,
                TOKEN.parse().unwrap(),
            )
            .await
            .unwrap();
        assert!(result.locator_persisted);
        let intermediate = before_observation.lock().unwrap().clone().unwrap();
        assert_eq!(intermediate.id, original_attempt.id);
        assert_eq!(intermediate.input_hash, original_attempt.input_hash);
        assert_eq!(intermediate.desired_hash, original_attempt.desired_hash);
        let after = std::fs::read_to_string(&path).unwrap();
        let doc = leafbud_mdx::parse(&after).unwrap();
        if mode == "verified" {
            assert!(matches!(result.outcome, BindingOutcome::PendingMaterialization));
            assert!(doc.theme.as_ref().unwrap().leaflet_watermark && doc.state.binding.is_some());
            assert_ne!(doc.state.pending.as_ref().unwrap().input_hash, original_attempt.input_hash);
        } else {
            assert!(matches!(result.outcome, BindingOutcome::LocatedUnverified | BindingOutcome::SourceChanged));
            assert_eq!(doc.state.pending.as_ref().unwrap(), &intermediate);
            assert!(doc.theme.is_none() && doc.state.binding.is_none());
            if mode == "source-race" {
                assert!(after.contains("New author prose"));
            }
        }
    }
}

#[tokio::test]
async fn explicit_locator_selection_rejects_stale_attempt_changed_source_replacement_and_prospective_duplicates() {
    for mode in ["stale-id", "edited-source", "prepared", "different-known-token", "external-duplicate"] {
        let temp = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let path = temp.path().join("draft.mdx");
        let other = external.path().join("other.mdx");
        draft(&path, "");
        draft(&other, "");
        let server = TestServer::spawn(Arc::new(|_| panic!("invalid explicit selection must fail before observation"))).await;
        let text = located(&path, &server.base_url, true);
        let mut doc = leafbud_mdx::parse(&text).unwrap();
        if let Effect::Mint { stage, .. } = &mut doc.state.pending.as_mut().unwrap().effect {
            *stage = match mode {
                "prepared" => MintStage::Prepared,
                "different-known-token" => MintStage::Located { token: Uuid::new_v4() },
                _ => MintStage::Dispatched,
            };
        }
        let mut attempt_id = doc.state.pending.as_ref().unwrap().id;
        if mode == "stale-id" {
            attempt_id = Uuid::new_v4();
        }
        let mut text = leafbud_mdx::patch_tracking(&text, &doc.state).unwrap();
        if mode == "edited-source" {
            text = text.replace("Desired prose", "New author prose");
        }
        if mode == "external-duplicate" {
            std::fs::write(&other, located(&other, &server.base_url, true)).unwrap();
        }
        std::fs::write(&path, &text).unwrap();
        assert!(
            client(&server)
                .select_mint_locator(
                    &old::context(temp.path()),
                    &path,
                    Selection {
                        operands: vec![path.clone(), other]
                    },
                    attempt_id,
                    TOKEN.parse().unwrap()
                )
                .await
                .is_err(),
            "{mode}"
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        assert!(server.requests().is_empty());
    }
}
