//! Deliberate deletion: guards, preview/execute discipline, observation-based
//! verification and the local unbind conversion — all through loopback HTTP.
use super::*;
use crate::pull::tests as old;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

const DID: &str = "did:plc:synthetic";
const DOCUMENT: &str = "at://did:plc:synthetic/site.standard.document/post";
const JS: &str = include_str!("../../../leaflet/web/tests/fixtures/delete-reference.js");
const VOID: &str = include_str!("../../../leaflet/web/tests/fixtures/delete-void.flight");
const TOKEN_MISSING: &str = r#"{"result":{"error":{"code":"PGRST116","message":"withheld"},"status":406,"data":null}}"#;

#[derive(Clone, Copy, PartialEq)]
enum PostMode {
    Success,
    Refuse,
    Lost,
}

/// One loopback Leaflet origin that imports a bound file and then serves the
/// delete flow. Flags flip server state as effects arrive; tests may also set
/// them directly to model out-of-band or already-deleted states.
struct DeleteServer {
    server: crate::import::tests::testserver::TestServer,
    fixture: Arc<std::sync::RwLock<Value>>,
    token_deleted: Arc<AtomicBool>,
    record_gone: Arc<AtomicBool>,
    posts: Arc<AtomicUsize>,
    post_mode: Arc<std::sync::RwLock<PostMode>>,
    lose_editor_response: Arc<AtomicBool>,
    editor_effect_applies: Arc<AtomicBool>,
}

impl DeleteServer {
    fn posts(&self) -> usize {
        self.posts.load(Ordering::SeqCst)
    }
}

/// Serve one origin and seed a bound local file through the real import path.
async fn origin(token: &str, published: bool) -> (DeleteServer, config::Context, tempfile::TempDir, PathBuf) {
    let mut fixture = old::fixture_with_token(token);
    if published {
        // The published-import profile reads its working metadata from the
        // linkage row and the pull response's extras, never the public body.
        fixture["leaflet_data"]["leaflets_to_documents"] = json!([{
            "document": DOCUMENT,
            "title": "Imported working title",
            "description": null,
            "tags": [],
            "documents": {"uri": DOCUMENT, "data": {
                "$type": "site.standard.document",
                "site": format!("https://leaflet.pub/p/{DID}"),
                "publishedAt": "2026-11-01T10:00:00Z"
            }}
        }]);
        fixture["response"]["extras"] = json!({
            "publication_title": "Imported working title",
            "publication_description": "",
            "publication_tags": []
        });
        for (key, value) in [
            ("publication_title", json!("Imported working title")),
            ("publication_description", json!("")),
            ("publication_tags", json!([])),
        ] {
            fixture["response"]["patch"]
                .as_array_mut()
                .unwrap()
                .push(json!({"op":"put","key":key,"value":value}));
        }
    }
    let fixture = Arc::new(std::sync::RwLock::new(fixture));
    let token_deleted: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let record_gone: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let posts: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let post_mode: Arc<std::sync::RwLock<PostMode>> = Arc::new(std::sync::RwLock::new(PostMode::Success));
    let lose_editor_response: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let editor_effect_applies: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let handler = (
        fixture.clone(),
        token_deleted.clone(),
        record_gone.clone(),
        posts.clone(),
        post_mode.clone(),
        lose_editor_response.clone(),
        editor_effect_applies.clone(),
    );
    let server = crate::import::tests::testserver::TestServer::spawn_headers(Arc::new(move |r| {
        let (fixture, token_deleted, record_gone, posts, post_mode, lose, applies) = &handler;
        match (r.method.as_str(), r.path.as_str()) {
            ("GET", "/home") => {
                let body = format!(
                    "<script src=\"/_next/static/delete.js\"></script><script>self.__next_f.push({})</script>",
                    json!([1, format!("0:{}", json!({"atp_did":DID,"bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]}))])
                );
                (200, vec![("Content-Type".into(), "text/html".into())], body)
            }
            ("GET", "/_next/static/delete.js") => (200, vec![("Content-Type".into(), "text/javascript".into())], JS.into()),
            ("POST", "/home") => {
                posts.fetch_add(1, Ordering::SeqCst);
                let flight = "text/x-component".to_owned();
                match r.header("next-action") {
                    Some("700000000000000000000000000000000000000004") => match *post_mode.read().unwrap() {
                        PostMode::Refuse => (
                            200,
                            vec![("Content-Type".into(), flight)],
                            "0:{\"a\":{\"success\":false,\"error\":{\"type\":\"not_authorized\",\"message\":\"withheld\",\"did\":\"\"}}}\n".into(),
                        ),
                        PostMode::Lost => {
                            // The record effect still applies; the response is lost.
                            record_gone.store(true, Ordering::SeqCst);
                            (200, vec![("Content-Type".into(), flight)], "0:{\"a\":\"$@1\"}\n".into())
                        }
                        PostMode::Success => {
                            record_gone.store(true, Ordering::SeqCst);
                            (200, vec![("Content-Type".into(), flight)], "0:{\"a\":{\"success\":true}}\n".into())
                        }
                    },
                    _ => {
                        if applies.load(Ordering::SeqCst) {
                            token_deleted.store(true, Ordering::SeqCst);
                        }
                        if lose.load(Ordering::SeqCst) {
                            (200, vec![("Content-Type".into(), flight)], "0:{\"a\":\"$@1\"}\n".into())
                        } else {
                            (200, vec![("Content-Type".into(), flight)], VOID.into())
                        }
                    }
                }
            }
            ("POST", "/api/rpc/get_leaflet_data") => {
                if token_deleted.load(Ordering::SeqCst) {
                    (200, vec![("Content-Type".into(), "application/json".into())], TOKEN_MISSING.into())
                } else {
                    let mut data = fixture.read().unwrap()["leaflet_data"].clone();
                    // After the record dies, the editor's linkage rows are gone.
                    if record_gone.load(Ordering::SeqCst) {
                        data["leaflets_to_documents"] = json!([]);
                        data["leaflets_in_publications"] = json!([]);
                    }
                    (
                        200,
                        vec![("Content-Type".into(), "application/json".into())],
                        json!({"result":{"error":null,"data":data}}).to_string(),
                    )
                }
            }
            ("POST", "/api/rpc/pull") => (
                200,
                vec![("Content-Type".into(), "application/json".into())],
                fixture.read().unwrap()["response"].to_string(),
            ),
            ("GET", "/xrpc/com.atproto.repo.getRecord") => {
                if record_gone.load(Ordering::SeqCst) {
                    // XRPC errors ride a non-2xx status with a JSON error body.
                    (400, vec![("Content-Type".into(), "application/json".into())], r#"{"error":"RecordNotFound","message":"withheld"}"#.into())
                } else {
                    let body = json!({"uri":DOCUMENT,"cid":null,"value":{"$type":"site.standard.document","site":format!("https://leaflet.pub/p/{DID}"),"publishedAt":"2026-11-01T10:00:00Z"}}).to_string();
                    (200, vec![("Content-Type".into(), "application/json".into())], body)
                }
            }
            _ => panic!("unexpected {} {}", r.method, r.path),
        }
    }))
    .await;
    let temp = tempfile::tempdir().unwrap();
    let mut context = old::context(temp.path());
    context.effective.pds = Some(server.base_url.clone());
    let path = old::seed(&server, temp.path(), token, "doc.mdx").await;
    if published {
        let mut document = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        document.state.state = "published".into();
        document.state.binding.as_mut().unwrap().uri = Some(DOCUMENT.into());
        std::fs::write(&path, leafbud_mdx::render(&document).unwrap()).unwrap();
    }
    (
        DeleteServer {
            server,
            fixture,
            token_deleted,
            record_gone,
            posts,
            post_mode,
            lose_editor_response,
            editor_effect_applies,
        },
        context,
        temp,
        path,
    )
}

fn reader(server: &crate::import::tests::testserver::TestServer) -> RemoteReader {
    RemoteReader::new(leafbud_web::LeafletClient::new(&server.base_url, Some("synthetic")).unwrap())
}

async fn delete(server: &DeleteServer, context: &config::Context, path: &Path, yes: bool, purge: bool) -> DeleteReport {
    reader(&server.server)
        .delete_editor(
            context,
            path,
            Selection {
                operands: vec![path.to_owned()],
            },
            DeleteOptions { yes, purge },
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn draft_preview_lists_what_dies_and_touches_nothing() {
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    let before = std::fs::read_to_string(&path).unwrap();
    let report = delete(&server, &context, &path, false, false).await;
    assert_eq!(report.outcome, "would-delete");
    assert_eq!(report.lifecycle, "draft");
    assert_eq!(report.token.to_string(), TOKEN);
    assert!(report.record.is_none() && report.observed.is_none());
    assert_eq!(report.local, Some("would-unbind"));
    assert!(report.message.as_deref().unwrap().contains("permanently deletes, for everyone"));
    assert!(!report.message.as_deref().unwrap().contains("PUBLISHED deletion additionally"));
    assert_eq!(server.posts(), 0, "preview dispatches nothing");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before, "preview writes nothing");
}

#[tokio::test]
async fn draft_execution_observes_gone_then_converts_local_source() {
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    let before = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "deleted");
    assert!(report.observed.as_ref().unwrap().token_gone);
    assert!(report.observed.as_ref().unwrap().record_gone.is_none());
    assert_eq!(server.posts(), 1, "exactly one deleteLeaflet dispatch");
    let after = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(after.state.id, before.state.id);
    assert!(after.state.binding.is_none() && after.state.nodes.is_empty() && after.state.sync.is_none());
    assert_eq!(after.authored_value(), before.authored_value());
    assert_eq!(report.local, Some("unbound"));
}

#[tokio::test]
async fn purge_removes_the_local_file_after_verified_deletion() {
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    let report = delete(&server, &context, &path, true, true).await;
    assert_eq!(report.outcome, "deleted");
    assert_eq!(report.local, Some("purged"));
    assert!(!path.exists());
}

#[tokio::test]
async fn pending_attempts_block_with_a_recover_hint() {
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    use leafbud_document::{Attempt, Effect, Protection, PushStage, WriterState};
    let mut state = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state;
    let mut writer = WriterState::fresh();
    let range = writer.reserve(2).unwrap();
    state.writer = Some(writer);
    let hash = format!("sha256:{}", "a".repeat(64));
    state.pending = Some(Attempt {
        id: Uuid::new_v4(),
        input_hash: hash.clone(),
        desired_hash: hash.clone(),
        effect: Effect::Push {
            range,
            stage: PushStage::Prepared,
            protection: Protection {
                hash,
                facts: vec![],
                entities: vec![],
                metadata_fields: vec![],
                verification_roots: vec![],
            },
            targets: Default::default(),
            created_entities: vec![],
        },
    });
    std::fs::write(&path, leafbud_mdx::patch_tracking(&std::fs::read_to_string(&path).unwrap(), &state).unwrap()).unwrap();
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "pending");
    assert!(report.message.as_deref().unwrap().contains("recover"));
    assert_eq!(server.posts(), 0);
}

#[tokio::test]
async fn wrong_origin_refuses_without_dispatch() {
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    let foreign = RemoteReader::new(leafbud_web::LeafletClient::new("https://leaflet.pub", Some("synthetic")).unwrap());
    let report = foreign
        .delete_editor(&context, &path, Selection { operands: vec![path.clone()] }, DeleteOptions::default())
        .await
        .unwrap();
    assert_eq!(report.outcome, "blocked");
    assert!(report.message.as_deref().unwrap().contains("another origin"));
    assert_eq!(server.posts(), 0);
    assert!(leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.binding.is_some());
}

#[tokio::test]
async fn server_side_drift_blocks_deletion_without_effects() {
    // Live root differs from the local binding.
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    server.fixture.write().unwrap()["leaflet_data"]["root_entity"] = json!("01999000-0000-7aaa-8000-000000000099");
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "blocked");
    assert!(report.message.as_deref().unwrap().contains("token/root differs"));
    assert_eq!(server.posts(), 0);

    // Lost write right: the native action would silently do nothing.
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    server.fixture.write().unwrap()["leaflet_data"]["permission_token_rights"][0]["write"] = json!(false);
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "blocked");
    assert!(report.message.as_deref().unwrap().contains("silently do nothing"));
    assert_eq!(server.posts(), 0);
    assert!(
        leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.binding.is_some(),
        "source preserved"
    );
}

#[tokio::test]
async fn uncertain_editor_response_resolves_only_by_observation() {
    // Lost response, no effect: uncertain, source preserved, no retry.
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    server.lose_editor_response.store(true, Ordering::SeqCst);
    server.editor_effect_applies.store(false, Ordering::SeqCst);
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "uncertain");
    assert!(!report.observed.as_ref().unwrap().token_gone);
    assert_eq!(report.local, Some("preserved"));
    assert_eq!(server.posts(), 1, "exactly one dispatch, never a retry");
    assert!(leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.binding.is_some());

    // Lost response whose effect happened: observation alone verifies it.
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    server.lose_editor_response.store(true, Ordering::SeqCst);
    server.editor_effect_applies.store(true, Ordering::SeqCst);
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "deleted");
    assert!(report.observed.as_ref().unwrap().token_gone);
    assert_eq!(report.local, Some("unbound"));
    assert_eq!(server.posts(), 1);
}

#[tokio::test]
async fn already_deleted_editor_unbinds_locally_without_any_dispatch() {
    // Draft preview: report the divergence, offer the local conversion.
    let (server, context, _temp, path) = origin(TOKEN, false).await;
    server.token_deleted.store(true, Ordering::SeqCst);
    let report = delete(&server, &context, &path, false, false).await;
    assert_eq!(report.outcome, "remote-deleted");
    assert_eq!(report.local, Some("would-unbind"));
    assert_eq!(server.posts(), 0);

    // Executed: only the local conversion runs.
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "unbound-local");
    assert_eq!(report.local, Some("unbound"));
    assert!(report.observed.as_ref().unwrap().token_gone);
    assert_eq!(server.posts(), 0);
    let after = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(after.state.binding.is_none() && after.state.state == "draft");

    // Published divergence: editor gone but record still resolving blocks.
    let (server, context, _temp, path) = origin(TOKEN, true).await;
    server.token_deleted.store(true, Ordering::SeqCst);
    server.record_gone.store(false, Ordering::SeqCst);
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "blocked");
    assert!(report.message.as_deref().unwrap().contains("public record still resolves"));
    assert!(leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.binding.is_some());
}

#[tokio::test]
async fn published_preview_and_execution_cover_record_then_editor() {
    let (server, context, _temp, path) = origin(TOKEN, true).await;
    let report = delete(&server, &context, &path, false, false).await;
    assert_eq!(report.outcome, "would-delete");
    assert_eq!(report.lifecycle, "published");
    assert_eq!(report.record.as_deref(), Some(DOCUMENT));
    assert!(report.message.as_deref().unwrap().contains("PUBLISHED deletion additionally"));
    assert_eq!(server.posts(), 0);

    let before = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "deleted", "{}", report.message.unwrap_or_default());
    let observed = report.observed.as_ref().unwrap();
    assert!(observed.token_gone && observed.record_gone == Some(true));
    assert_eq!(server.posts(), 2, "deletePost then deleteLeaflet, native order");
    assert!(report.message.as_deref().unwrap().contains("propagation may lag"));
    let after = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(after.state.id, before.state.id);
    assert!(after.state.binding.is_none() && after.state.released.is_none() && after.state.state == "draft");
    assert_eq!(after.authored_value(), before.authored_value());
}

#[tokio::test]
async fn published_refusal_and_uncertainty_are_observed_not_retried() {
    // Typed refusal: nothing is concluded, the editor is not dispatched.
    let (server, context, _temp, path) = origin(TOKEN, true).await;
    *server.post_mode.write().unwrap() = PostMode::Refuse;
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "refused");
    assert!(report.message.as_deref().unwrap().contains("deletePost refused"));
    assert_eq!(server.posts(), 1, "no deleteLeaflet after a typed refusal");
    assert!(leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.binding.is_some());

    // Lost deletePost response whose effect happened: continue through
    // observation, never a second deletePost dispatch.
    let (server, context, _temp, path) = origin(TOKEN, true).await;
    *server.post_mode.write().unwrap() = PostMode::Lost;
    let report = delete(&server, &context, &path, true, false).await;
    assert_eq!(report.outcome, "deleted");
    assert_eq!(server.posts(), 2, "one deletePost attempt total");
    let observed = report.observed.as_ref().unwrap();
    assert!(observed.token_gone && observed.record_gone == Some(true));

    // A record that vanished while the editor still resolves is divergence.
    let (server, context, _temp, path) = origin(TOKEN, true).await;
    server.record_gone.store(true, Ordering::SeqCst);
    let report = delete(&server, &context, &path, false, false).await;
    assert_eq!(report.outcome, "blocked");
    assert!(report.message.as_deref().unwrap().contains("already missing"));
    assert_eq!(server.posts(), 0);
}
