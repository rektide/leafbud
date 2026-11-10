use super::*;
use crate::import::tests::testserver::{Recorded, TestServer};
use crate::pull::tests as old;
use leafbud_document::PushStage;
use leafbud_protocol::{LeafletMetadata, NativeMutation};
use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[path = "defaults_tests.rs"]
mod defaults;

#[path = "description_tests.rs"]
mod descriptions;

#[path = "save_tests.rs"]
mod saves;

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const ROOT: &str = "11111111-1111-4111-8111-111111111111";
const PAGE: &str = "22222222-2222-4222-8222-222222222222";
const BLOCK: &str = "33333333-3333-4333-8333-333333333333";
const SET: &str = "44444444-4444-4444-8444-444444444444";
const ENTRY: &str = "55555555-5555-4555-8555-555555555555";
const MEMBER: &str = "66666666-6666-4666-8666-666666666666";
const KIND: &str = "77777777-7777-4777-8777-777777777777";
const LEVEL: &str = "88888888-8888-4888-8888-888888888888";
const URI: &str = "at://did:plc:synthetic/site.standard.publication/garden";

fn home() -> String {
    format!(
        "<script>self.__next_f.push({})</script>",
        json!([
            1,
            format!(
                "0:{}",
                json!({"atp_did":"did:plc:synthetic","bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]})
            )
        ])
    )
}
fn seed(publication: bool) -> (Value, Value) {
    let data = json!({"id":TOKEN,"root_entity":ROOT,"title":null,"description":null,
        "permission_token_rights":[{"token":TOKEN,"entity_set":SET,"read":true,"write":true}],
        "leaflets_in_publications":if publication{json!([{"publication":URI,"doc":null,"title":"Untitled"}])}else{json!([])},"leaflets_to_documents":[]});
    let mut facts = vec![
        json!({"id":ENTRY,"entity":ROOT,"attribute":"root/page","data":{"type":"ordered-reference","value":PAGE,"position":"a0"}}),
        json!({"id":MEMBER,"entity":PAGE,"attribute":"card/block","data":{"type":"ordered-reference","value":BLOCK,"position":"a0"}}),
        json!({"id":KIND,"entity":BLOCK,"attribute":"block/type","data":{"type":"block-type-union","value":if publication{"text"}else{"heading"}}}),
    ];
    if !publication {
        facts.push(json!({"id":LEVEL,"entity":BLOCK,"attribute":"block/heading-level","data":{"type":"number","value":1}}));
    }
    facts.push(
        json!({"id":"99999999-9999-4999-8999-999999999999","entity":ROOT,"attribute":"theme/page-leaflet-watermark","data":{"type":"boolean","value":true}}),
    );
    let mut patch = vec![json!({"op":"clear"}), json!({"op":"put","key":"initialized","value":true})];
    patch.extend(facts.into_iter().map(|f| json!({"op":"put","key":f["id"],"value":f})));
    if publication {
        patch.push(json!({"op":"put","key":"draft_contributors","value":["did:plc:contributor-default"]}));
    }
    (data, json!({"patch":patch,"lastMutationIDChanges":{}}))
}
fn draft(path: &Path, publication: &str) -> Uuid {
    let id = Uuid::new_v4();
    std::fs::write(path,format!("---\nformat: leafbud-mdx/2\ntitle: Desired title\n{publication}\n\"@leafbud\": {{\"id\":\"{id}\",\"state\":\"draft\"}}\n---\n\nDesired prose, untouched.\n")).unwrap();
    id
}
fn client(server: &TestServer) -> RemoteReader {
    RemoteReader::new(leafbud_web::LeafletClient::new(&server.base_url, Some("synthetic")).unwrap())
}

#[tokio::test]
async fn both_mint_contexts_checkpoint_before_effect_keep_uuid_keys_and_seed_pending() {
    for publication in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("draft.mdx");
        let id = draft(&path, if publication { "" } else { "publication: null" });
        let captured = path.clone();
        let (data, view) = seed(publication);
        let server = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
            "/home" => (200, vec![], home()),
            "/lish/did:plc:synthetic/garden/dashboard" if r.method == "GET" => (200, vec![], "<script src=\"/_next/static/mint.js\"></script>".into()),
            "/_next/static/mint.js" => (200, vec![], include_str!("../../../leaflet/web/tests/fixtures/mint-reference.js").into()),
            "/new" | "/lish/did:plc:synthetic/garden/dashboard" => {
                let source = std::fs::read_to_string(&captured).unwrap();
                let doc = leafbud_mdx::parse(&source).unwrap();
                assert_eq!(doc.state.id, id);
                assert!(doc.state.main_page.is_some());
                assert!(doc.pages[0].blocks[0].key.starts_with("b-"));
                assert!(matches!(
                    doc.state.pending.unwrap().effect,
                    Effect::Mint {
                        stage: MintStage::Dispatched,
                        ..
                    }
                ));
                if publication {
                    assert_eq!(r.body_json(), json!([URI, "doc", {}]));
                    (
                        200,
                        vec![("Content-Type".into(), "text/x-component".into())],
                        include_str!("../../../leaflet/web/tests/fixtures/mint-result.flight").into(),
                    )
                } else {
                    (307, vec![("Location".into(), format!("/{TOKEN}?focusFirstBlock"))], String::new())
                }
            }
            "/api/rpc/get_leaflet_data" => (200, vec![], json!({"result":{"error":null,"data":data}}).to_string()),
            "/api/rpc/pull" => (200, vec![], view.to_string()),
            _ => panic!("unexpected request"),
        }))
        .await;
        let mut context = old::context(temp.path());
        context.effective.publication = Some(URI.into());
        let reader = client(&server);
        let result = reader.stage_binding(&context, &path, Selection::default()).await.unwrap();
        assert!(matches!(result.outcome, BindingOutcome::PendingMaterialization), "{result:?}");
        let after = std::fs::read_to_string(&path).unwrap();
        let doc = leafbud_mdx::parse(&after).unwrap();
        assert_eq!(doc.state.id, id);
        assert_eq!(
            doc.state.binding.as_ref().unwrap().publication.as_deref(),
            if publication { Some(URI) } else { None }
        );
        assert!(doc.state.sync.is_none() && doc.state.nodes.is_empty());
        assert_eq!(doc.publication, if publication { Publication::At(URI.into()) } else { Publication::Standalone });
        assert!(doc.theme.as_ref().is_some_and(|theme| theme.leaflet_watermark));
        assert_eq!(
            doc.extra.get("contributors").cloned(),
            if publication { Some(json!(["did:plc:contributor-default"])) } else { None }
        );
        let attempt = doc.state.pending.as_ref().unwrap();
        assert_eq!(attempt.input_hash, import::hash(&doc.authored_value()));
        assert_eq!(attempt.desired_hash, import::hash(&doc.editor_value()));
        assert!(after.ends_with("Desired prose, untouched.\n"));
        assert!(reader.stage_binding(&context, &path, Selection::default()).await.is_err());
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|r| r.path == "/new" || r.method == "POST" && r.path.ends_with("dashboard"))
                .count(),
            1
        );
        let recovered = reader.recover_native(&context, &path, Selection::default(), true).await.unwrap();
        assert_eq!(recovered.outcome, write::RecoveryOutcome::PendingMaterialization);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            after,
            "recovery does not materialize/erase seed pending or contributors"
        );
    }
}

#[tokio::test]
async fn lost_mint_response_and_token_then_failed_verification_never_remint() {
    for lost in [true, false] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("draft.mdx");
        draft(&path, "publication: null");
        let server = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
            "/home" => (200, vec![], home()),
            "/new" if lost => (503, vec![], "{}".into()),
            "/new" => (307, vec![("Location".into(), format!("/{TOKEN}"))], String::new()),
            "/api/rpc/get_leaflet_data" => (503, vec![], "{}".into()),
            _ => panic!("unexpected request"),
        }))
        .await;
        let reader = client(&server);
        let context = old::context(temp.path());
        let result = reader.stage_binding(&context, &path, Selection::default()).await.unwrap();
        let doc = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(doc.state.binding.is_none() && doc.state.sync.is_none());
        if lost {
            assert!(matches!(result.outcome, BindingOutcome::MintUncertain));
            assert!(matches!(
                doc.state.pending.unwrap().effect,
                Effect::Mint {
                    stage: MintStage::Dispatched,
                    ..
                }
            ));
            assert_eq!(
                reader.recover_native(&context, &path, Selection::default(), true).await.unwrap().outcome,
                write::RecoveryOutcome::MintUncertain
            );
            let public = reader
                .recover_editor(&context, &path, Selection::default(), RecoverOptions::default())
                .await
                .unwrap();
            assert_eq!(public.outcome, "pending");
            assert!(public.message.unwrap().contains("select_mint_locator"));
        } else {
            assert!(matches!(result.outcome, BindingOutcome::LocatedUnverified));
            assert!(matches!(
                doc.state.pending.unwrap().effect,
                Effect::Mint {
                    stage: MintStage::Located { .. },
                    ..
                }
            ));
        }
        assert!(reader.stage_binding(&context, &path, Selection::default()).await.is_err());
        assert_eq!(server.request_count("/new"), 1);
    }
}

#[tokio::test]
async fn mint_source_race_preserves_new_bytes_and_returns_unpersisted_real_locator() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("draft.mdx");
    draft(&path, "publication: null");
    let changed = path.clone();
    let server = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
        "/home" => (200, vec![], home()),
        "/new" => {
            let source = std::fs::read_to_string(&changed).unwrap();
            std::fs::write(&changed, source.replace("Desired prose", "New author prose")).unwrap();
            (307, vec![("Location".into(), format!("/{TOKEN}"))], String::new())
        }
        _ => panic!("verification must not run after token persistence source race"),
    }))
    .await;
    let result = client(&server)
        .stage_binding(&old::context(temp.path()), &path, Selection::default())
        .await
        .unwrap();
    assert!(matches!(result.outcome, BindingOutcome::SourceChanged));
    assert_eq!(result.token, Some(TOKEN.parse().unwrap()));
    assert!(!result.locator_persisted);
    let source = std::fs::read_to_string(path).unwrap();
    assert!(source.contains("New author prose"));
    assert!(!source.contains(TOKEN));
}

#[test]
fn wrong_rights_seed_and_publication_context_are_not_confirmed_bindings() {
    for case in 0..4 {
        let (mut raw, mut view) = seed(false);
        if case == 0 {
            raw["permission_token_rights"][0]["write"] = false.into();
        }
        if case == 1 {
            view["patch"][4]["value"]["data"]["value"] = "text".into();
        }
        if case == 2 {
            raw["leaflets_in_publications"] = json!([{"publication":URI,"doc":null}]);
        }
        if case == 3 {
            raw["permission_token_rights"][0]["token"] = Uuid::new_v4().to_string().into();
        }
        let data = leafbud_web::parse_rpc(&json!({"result":{"error":null,"data":raw}}).to_string()).unwrap();
        assert!(verify_seed("https://leaflet.pub", &data, &leafbud_web::parse_pull(view).unwrap(), &MintContext::Standalone).is_err());
    }
}

#[tokio::test]
async fn complete_request_validation_and_final_drift_gate_precede_any_effect() {
    let server = EffectServer::new(0, None).await;
    let temp = tempfile::tempdir().unwrap();
    let path = server.seed(temp.path());
    let reader = client(&server.server);
    let context = old::context(temp.path());
    let original = std::fs::read(&path).unwrap();
    let result = reader
        .execute_compiled(&context, &path, Selection::default(), None, |doc, data, view| {
            let mut compiled = metadata_compiler(251)(doc, data, view)?;
            compiled.mutations.push(NativeMutation::RetractFact { fact_id: Uuid::new_v4() });
            Ok(compiled)
        })
        .await;
    assert!(result.is_err());
    assert_eq!(server.pushes.load(Ordering::SeqCst), 0);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let result = reader
        .execute_compiled(&context, &path, Selection::default(), None, |doc, data, view| {
            let compiled = metadata_compiler(1)(doc, data, view)?;
            server.fixture.lock().unwrap()["leaflet_data"]["description"] = "remote drift during compile".into();
            Ok(compiled)
        })
        .await;
    assert!(result.unwrap_err().to_string().contains("drift"));
    assert_eq!(server.pushes.load(Ordering::SeqCst), 0);
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[tokio::test]
async fn equal_visible_meaning_cannot_hide_a_protected_native_scope_change() {
    let server = EffectServer::new(4, None).await;
    let temp = tempfile::tempdir().unwrap();
    let path = server.seed(temp.path());
    let reader = client(&server.server);
    let context = old::context(temp.path());
    let before = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.sync;
    let result = reader
        .execute_compiled(&context, &path, Selection::default(), None, metadata_compiler(1))
        .await
        .unwrap();
    assert_eq!(result.outcome, write::RecoveryOutcome::ResolvedPartial);
    assert!(result.message.unwrap().contains("protected"));
    assert_eq!(leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.sync, before);
}

#[tokio::test]
async fn prepared_unsent_recovery_clears_only_attempt_and_keeps_reserved_writer_sequence() {
    let server = EffectServer::new(0, None).await;
    let temp = tempfile::tempdir().unwrap();
    let path = server.seed(temp.path());
    let reader = client(&server.server);
    let context = old::context(temp.path());
    let source = std::fs::read_to_string(&path).unwrap();
    let mut doc = leafbud_mdx::parse(&source).unwrap();
    let sync = doc.state.sync.clone();
    let mut writer = leafbud_document::WriterState::fresh();
    let range = writer.reserve(3).unwrap();
    doc.state.writer = Some(writer);
    doc.state.pending = Some(Attempt {
        id: Uuid::new_v4(),
        input_hash: import::hash(&doc.authored_value()),
        desired_hash: import::hash(&doc.editor_value()),
        effect: Effect::Push {
            range,
            stage: PushStage::Prepared,
            protection: leafbud_document::Protection {
                hash: format!("sha256:{}", "0".repeat(64)),
                facts: vec![],
                entities: vec![],
                metadata_fields: vec![],
                verification_roots: vec![],
            },
            targets: Default::default(),
            created_entities: vec![],
        },
    });
    let changed_source = leafbud_mdx::patch_tracking(&source, &doc.state)
        .unwrap()
        .replace("Wanted title", "Changed while definitely unsent");
    std::fs::write(&path, changed_source).unwrap();
    let result = reader.recover_native(&context, &path, Selection::default(), true).await.unwrap();
    assert_eq!(result.outcome, write::RecoveryOutcome::PreparedNotSent);
    assert!(result.settled);
    let after = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(after.state.pending.is_none());
    assert_eq!(after.state.sync, sync);
    assert_eq!(after.state.writer.unwrap().next_mutation_id, 4);
    assert!(server.server.requests().is_empty());
}

#[tokio::test]
async fn in_flight_external_operand_duplicate_blocks_mint_locator_patch() {
    let temp = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let path = temp.path().join("draft.mdx");
    let other = external.path().join("other.mdx");
    let id = draft(&path, "publication: null");
    draft(&other, "publication: null");
    let duplicate = other.clone();
    let server = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
        "/home" => (200, vec![], home()),
        "/new" => {
            let mut doc = leafbud_mdx::parse(&std::fs::read_to_string(&duplicate).unwrap()).unwrap();
            doc.state.id = id;
            std::fs::write(&duplicate, leafbud_mdx::render(&doc).unwrap()).unwrap();
            (307, vec![("Location".into(), format!("/{TOKEN}"))], String::new())
        }
        _ => panic!("duplicate must prevent verification"),
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
    assert!(result.token.is_some() && !result.locator_persisted);
    let doc = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(doc.state.binding.is_none());
    assert!(matches!(
        doc.state.pending.unwrap().effect,
        Effect::Mint {
            stage: MintStage::Dispatched,
            ..
        }
    ));
}

struct EffectServer {
    server: TestServer,
    fixture: Arc<Mutex<Value>>,
    ack: Arc<Mutex<Option<(String, u64)>>>,
    pushes: Arc<AtomicUsize>,
}

#[tokio::test]
async fn late_mint_rights_change_preserves_locator_without_confirming_binding() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("draft.mdx");
    draft(&path, "publication: null");
    let (data, view) = seed(false);
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let server = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
        "/home" => (200, vec![], home()),
        "/new" => (307, vec![("Location".into(), format!("/{TOKEN}"))], String::new()),
        "/api/rpc/pull" => (200, vec![], view.to_string()),
        "/api/rpc/get_leaflet_data" => {
            let mut data = data.clone();
            if counter.fetch_add(1, Ordering::SeqCst) > 0 {
                data["permission_token_rights"][0]["write"] = false.into();
            }
            (200, vec![], json!({"result":{"error":null,"data":data}}).to_string())
        }
        _ => panic!("unexpected request"),
    }))
    .await;
    let result = client(&server)
        .stage_binding(&old::context(temp.path()), &path, Selection::default())
        .await
        .unwrap();
    assert!(matches!(result.outcome, BindingOutcome::LocatedUnverified));
    let doc = leafbud_mdx::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert!(doc.state.binding.is_none() && doc.state.sync.is_none());
    assert!(matches!(
        doc.state.pending.unwrap().effect,
        Effect::Mint {
            stage: MintStage::Located { .. },
            ..
        }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
impl EffectServer {
    async fn new(mode: usize, race: Option<PathBuf>) -> Self {
        let fixture = Arc::new(Mutex::new(old::fixture()));
        let remote = fixture.clone();
        let ack = Arc::new(Mutex::new(None::<(String, u64)>));
        let acks = ack.clone();
        let pushes = Arc::new(AtomicUsize::new(0));
        let count = pushes.clone();
        let server=TestServer::spawn(Arc::new(move |r:&Recorded|match r.path.as_str(){
            "/api/rpc/get_leaflet_data"=>(200,json!({"result":{"error":null,"data":remote.lock().unwrap()["leaflet_data"]}}).to_string()),
            "/api/rpc/pull"=>{let mut view=remote.lock().unwrap()["response"].clone();if let Some((client,id))=&*acks.lock().unwrap(){view["lastMutationIDChanges"]=json!({client:*id});}(200,view.to_string())},
            "/api/rpc/getFactsForRoots"=>{
                let roots=r.body_json()["roots"].as_array().unwrap().clone();
                let root=roots[0].as_str().unwrap();
                (200,json!({"result":{"facts":{root:[{"id":KIND,"entity":root,"root_id":root,"attribute":"block/type","data":{"type":"block-type-union","value":"text"}}]}}}).to_string())
            }
            "/api/rpc/push"=>{
                let call=count.fetch_add(1,Ordering::SeqCst);
                if mode==2 && call>0 {return (503,"{}".into());}
                let body=r.body_json();let mutations=body["pushRequest"]["mutations"].as_array().unwrap();let last=mutations.last().unwrap();
                *acks.lock().unwrap()=Some((last["clientID"].as_str().unwrap().into(),last["id"].as_u64().unwrap()));
                // A controlled synthetic native effect, not an implementation
                // mirror/compiler simulator: only persisted metadata changes.
                 if mode!=1 {
                     let mut fixture=remote.lock().unwrap();
                     for mutation in mutations {
                         let args=&mutation["args"];
                         if mutation["name"]=="updateLeafletMetadata" {
                             for field in ["title","description"] {
                                 if let Some(value)=args.get(field) { fixture["leaflet_data"][field]=value.clone(); }
                             }
                         } else if mutation["name"]=="updatePublicationDraft" {
                             for field in ["title","description","tags"] {
                                 if let Some(value)=args.get(field) {
                                     let key=format!("publication_{field}");
                                     let patch=fixture["response"]["patch"].as_array_mut().unwrap();
                                     let update=json!({"op":"put","key":key,"value":value});
                                     if let Some(old)=patch.iter_mut().find(|p|p["key"]==key) {*old=update;}else{patch.push(update);}
                                 }
                             }
                         }
                     }
                 }
                if mode==4 {
                    let mut fixture=remote.lock().unwrap();
                    let entity=fixture["response"]["patch"].as_array().unwrap().iter().find(|p|p["value"]["attribute"]=="block/type" && p["value"]["data"]["value"]=="text").unwrap()["value"]["entity"].clone();
                    fixture["response"]["patch"].as_array_mut().unwrap().push(json!({"op":"put","key":KIND,"value":{"id":KIND,"entity":entity,"attribute":"block/is-list","data":{"type":"boolean","value":false}}}));
                }
                if let Some(path)=&race {let source=std::fs::read_to_string(path).unwrap();std::fs::write(path,source.replace("Wanted title","New source title")).unwrap();}
                if mode==3 && call==0 {(503,"{}".into())}else{(200,"{}".into())}
            }
            _=>panic!("unexpected endpoint"),
        })).await;
        Self { server, fixture, ack, pushes }
    }
    fn seed(&self, root: &Path) -> PathBuf {
        let f = self.fixture.lock().unwrap();
        let mut doc = import::build(&self.server.base_url, &old::data(&f), &leafbud_web::parse_pull(f["response"].clone()).unwrap()).unwrap();
        doc.title = "Wanted title".into();
        let path = root.join("bound.mdx");
        std::fs::write(&path, leafbud_mdx::render(&doc).unwrap()).unwrap();
        path
    }
}
fn metadata_compiler(count: usize) -> impl FnOnce(&Document, &LeafletData, &PullSnapshot) -> Result<write::Compiled> {
    move |doc, _, _| {
        Ok(write::Compiled {
            mutations: vec![
                NativeMutation::UpdateLeafletMetadata(LeafletMetadata {
                    title: Some(doc.title.clone()),
                    ..Default::default()
                });
                count
            ],
            targets: Default::default(),
            created_entities: vec![],
            allocated_facts: vec![],
        })
    }
}

#[tokio::test]
async fn actual_execution_distinguishes_acknowledged_skips_achievement_and_exact_replay() {
    for mode in [0, 1, 3] {
        let server = EffectServer::new(mode, None).await;
        let temp = tempfile::tempdir().unwrap();
        let path = server.seed(temp.path());
        let reader = client(&server.server);
        let result = reader
            .execute_compiled(&old::context(temp.path()), &path, Selection::default(), None, metadata_compiler(1))
            .await
            .unwrap();
        let doc = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        if mode == 1 {
            assert_eq!(result.outcome, write::RecoveryOutcome::ResolvedPartial);
            assert!(!result.settled);
            assert!(matches!(
                doc.state.pending.unwrap().effect,
                Effect::Push {
                    stage: PushStage::Partial { .. },
                    ..
                }
            ));
        } else {
            assert_eq!(result.outcome, write::RecoveryOutcome::DesiredVerified, "{result:?}");
            assert!(result.settled && doc.state.pending.is_none());
        }
        if mode == 3 {
            let requests: Vec<_> = server.server.requests().into_iter().filter(|r| r.path == "/api/rpc/push").collect();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].body, requests[1].body);
        }
        assert_eq!(doc.state.writer.unwrap().next_mutation_id, 2);
    }
}

#[tokio::test]
async fn ordered_multi_batch_failure_preserves_reserved_range_and_restart_uncertainty() {
    let server = EffectServer::new(2, None).await;
    let temp = tempfile::tempdir().unwrap();
    let path = server.seed(temp.path());
    let reader = client(&server.server);
    let context = old::context(temp.path());
    let result = reader
        .execute_compiled(&context, &path, Selection::default(), None, metadata_compiler(501))
        .await
        .unwrap();
    assert_eq!(result.outcome, write::RecoveryOutcome::Uncertain);
    assert_eq!(result.acknowledged, Some(250));
    assert_eq!(server.pushes.load(Ordering::SeqCst), 3);
    let original = std::fs::read(&path).unwrap();
    let doc = leafbud_mdx::parse(std::str::from_utf8(&original).unwrap()).unwrap();
    assert_eq!(doc.state.writer.as_ref().unwrap().next_mutation_id, 502);
    assert!(matches!(
        doc.state.pending.unwrap().effect,
        Effect::Push {
            stage: PushStage::Dispatched { through: 500 },
            ..
        }
    ));
    // New process/session has no exact bytes. Current desired metadata is
    // already present but a delayed request is not proven terminal by that.
    let result = client(&server.server)
        .recover_native(&context, &path, Selection::default(), true)
        .await
        .unwrap();
    assert_eq!(result.outcome, write::RecoveryOutcome::Uncertain);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(server.pushes.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn source_change_and_complete_preflight_failure_do_not_advance_receipts() {
    let server = EffectServer::new(0, None).await;
    let temp = tempfile::tempdir().unwrap();
    let path = server.seed(temp.path());
    let reader = client(&server.server);
    let context = old::context(temp.path());
    assert!(
        reader
            .execute_compiled(&context, &path, Selection::default(), None, |_, _, _| anyhow::bail!(
                "unsupported final desired remainder"
            ))
            .await
            .is_err()
    );
    assert_eq!(server.pushes.load(Ordering::SeqCst), 0);
    let before = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.sync;
    let racing = EffectServer::new(0, Some(path.clone())).await;
    let mut doc = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    doc.state.binding.as_mut().unwrap().base_url = racing.server.base_url.clone();
    std::fs::write(&path, leafbud_mdx::render(&doc).unwrap()).unwrap();
    let result = client(&racing.server)
        .execute_compiled(&context, &path, Selection::default(), None, metadata_compiler(1))
        .await
        .unwrap();
    assert_eq!(result.outcome, write::RecoveryOutcome::SourceChanged);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("New source title"));
    let doc = leafbud_mdx::parse(&text).unwrap();
    assert_eq!(doc.state.sync, before);
    assert!(doc.state.pending.is_some());
}

#[tokio::test]
async fn typed_pending_survives_status_pull_tidy_and_inspects_known_detached_entities() {
    let server = EffectServer::new(1, None).await;
    let temp = tempfile::tempdir().unwrap();
    let path = server.seed(temp.path());
    let reader = client(&server.server);
    let context = old::context(temp.path());
    reader
        .execute_compiled(&context, &path, Selection::default(), None, metadata_compiler(1))
        .await
        .unwrap();
    let mut doc = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let client_id = doc.state.writer.as_ref().unwrap().client_id;
    if let Effect::Push { created_entities, .. } = &mut doc.state.pending.as_mut().unwrap().effect {
        created_entities.push(BLOCK.parse().unwrap());
    }
    let text = leafbud_mdx::patch_tracking(&std::fs::read_to_string(&path).unwrap(), &doc.state).unwrap();
    std::fs::write(&path, &text).unwrap();
    let report = reader.recover_native(&context, &path, Selection::default(), false).await.unwrap();
    assert_eq!(report.detached_facts_observed, 1);
    assert_eq!(report.outcome, write::RecoveryOutcome::ResolvedPartial);
    let status = crate::status::inspect(&context, crate::status::Request { files: vec![path.clone()] }, Some(&reader))
        .await
        .unwrap();
    assert_eq!(status.items[0].outcome, "resolved-partial");
    assert_eq!(
        reader
            .refresh_document(
                &context,
                &path,
                crate::refresh::Policy {
                    take_remote: true,
                    ..Default::default()
                }
            )
            .await
            .outcome,
        crate::refresh::Outcome::Pending
    );
    assert_eq!(
        crate::tidy::run(
            &context,
            crate::tidy::Request {
                files: vec![path.clone()],
                ..Default::default()
            }
        )
        .unwrap()
        .failed,
        1
    );
    for (from, accept) in [
        (None, true),
        (Some(crate::names::Direction::Filename), false),
        (Some(crate::names::Direction::Path), false),
    ] {
        let report = crate::tidy::run(
            &context,
            crate::tidy::Request {
                files: vec![path.clone()],
                from,
                accept,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(report.failed, 1);
        assert!(report.items[0].error.as_ref().unwrap().contains("pending"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    }
    *server.ack.lock().unwrap() = Some((client_id.to_string(), 0));
    assert_eq!(
        reader.recover_native(&context, &path, Selection::default(), true).await.unwrap().outcome,
        write::RecoveryOutcome::Uncertain
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
}
