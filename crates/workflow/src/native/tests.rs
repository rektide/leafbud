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

struct EffectServer {
    server: TestServer,
    fixture: Arc<Mutex<Value>>,
    ack: Arc<Mutex<Option<(String, u64)>>>,
    pushes: Arc<AtomicUsize>,
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
                if mode!=1 {remote.lock().unwrap()["leaflet_data"]["title"]=last["args"]["title"].clone();}
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
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    *server.ack.lock().unwrap() = Some((client_id.to_string(), 0));
    assert_eq!(
        reader.recover_native(&context, &path, Selection::default(), true).await.unwrap().outcome,
        write::RecoveryOutcome::Uncertain
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
}
