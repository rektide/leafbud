use super::*;
use crate::{import::tests::testserver::TestServer, pull::tests as support};
use leafbud_document::{BlockKind, Inline};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use crate::native::tests::saves::store;

const OWNER: &str = "did:plc:abcdefghijklmnopqrstuvwx";
const URI: &str = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/post";
const PUBLICATION: &str = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.publication/garden";

fn home() -> String {
    format!(
        "<script>self.__next_f.push({})</script>",
        json!([
            1,
            format!(
                "0:{}",
                json!({"atp_did":OWNER,"bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]})
            )
        ])
    )
}
fn data(store: &store::Store) -> LeafletData {
    support::data(&json!({"leaflet_data":store.data}))
}
fn snapshot(store: &mut store::Store) -> PullSnapshot {
    leafbud_web::parse_pull(store.rpc("/api/rpc/pull", &json!({"pullRequest":{"clientGroupID":"synthetic"}})).unwrap()).unwrap()
}
fn record(store: &mut store::Store, publication: bool, args: &Value) -> Value {
    let data = data(store);
    let snapshot = snapshot(store);
    let doc = import::build("https://leaflet.pub", &data, &snapshot).unwrap();
    let pages: Vec<_> = doc
        .pages
        .iter()
        .map(|p| {
            let blocks: Vec<_> = p
                .blocks
                .iter()
                .map(|b| {
                    let plaintext = |content: &[Inline]| {
                        content
                            .iter()
                            .map(|i| match i {
                                Inline::Run { text, marks } => {
                                    assert!(marks.is_empty());
                                    text.as_str()
                                }
                                Inline::HardBreak => "\n",
                            })
                            .collect::<String>()
                    };
                    let block = match &b.kind {
                        BlockKind::Paragraph { content } => json!({"$type":"pub.leaflet.blocks.text","plaintext":plaintext(content)}),
                        BlockKind::Heading { level, content } => json!({"$type":"pub.leaflet.blocks.header","level":level,"plaintext":plaintext(content)}),
                        other => panic!("fixture encoding not supplied: {other:?}"),
                    };
                    json!({"$type":"pub.leaflet.pages.linearDocument#block","block":block})
                })
                .collect();
            json!({"$type":"pub.leaflet.pages.linearDocument","id":doc.state.nodes[&p.key].entity,"blocks":blocks})
        })
        .collect();
    let mut record = json!({"$type":"site.standard.document","title":args["title"],"description":args["description"],"tags":args["tags"],
        "site":if publication {PUBLICATION.to_owned()} else {format!("https://leaflet.pub/p/{OWNER}")},"path":"/post","publishedAt":args.get("publishedAt").cloned().unwrap_or_else(|| json!(jiff::Timestamp::now().to_string())),
        "content":{"$type":"pub.leaflet.content","pages":pages}});
    if publication {
        record["contributors"] = json!([{"did":OWNER}]);
    }
    if record["publishedAt"].as_str().unwrap().parse::<jiff::Timestamp>().unwrap() > jiff::Timestamp::now() {
        record["publishedAt"] = json!(jiff::Timestamp::now().to_string());
    }
    record
}
struct Server {
    http: TestServer,
    store: Arc<Mutex<store::Store>>,
    mode: Arc<AtomicUsize>,
    calls: Arc<AtomicUsize>,
}
impl Server {
    async fn new(publication: bool, path: PathBuf) -> Self {
        let mut store = store::Store::new(publication);
        store.data["reply_drafts"] = json!([]);
        store.data["publication_scheduled_posts"] = json!([]);
        if publication {
            store.data["leaflets_in_publications"][0]["leaflet"] = json!(store::TOKEN);
            store.data["leaflets_in_publications"][0]["publication"] = json!(PUBLICATION);
            store.data["leaflets_in_publications"][0]["preferences"] = Value::Null;
            store.data["leaflets_in_publications"][0]["publications"] = json!({"uri":PUBLICATION,"identity_did":OWNER,"publication_contributors":[]});
            store.extras.insert("draft_contributors".into(), json!([]));
        }
        let store = Arc::new(Mutex::new(store));
        let state = store.clone();
        let mode = Arc::new(AtomicUsize::new(0));
        let failure = mode.clone();
        let calls = Arc::new(AtomicUsize::new(0));
        let sent = calls.clone();
        let http = TestServer::spawn_headers(Arc::new(move |r| {
            let mut store = state.lock().unwrap();
            let mode = failure.load(Ordering::SeqCst);
            let response = match r.path.as_str() {
                "/home" => {
                    if mode == 6 && read(&path).state.publication_attempt.is_some() {
                        return (503, vec![], "post-editor discovery unavailable".into());
                    }
                    home()
                }
                "/_next/static/publish.js" => include_str!("../../../leaflet/web/tests/fixtures/publish-reference.js").into(),
                "/_next/static/mint.js" => include_str!("../../../leaflet/web/tests/fixtures/mint-reference.js").into(),
                "/lish/did:plc:abcdefghijklmnopqrstuvwx/garden/dashboard" if r.method == "GET" => {
                    return (200, vec![], "<script src=\"/_next/static/mint.js\"></script>".into());
                }
                "/lish/did:plc:abcdefghijklmnopqrstuvwx/garden/dashboard" => {
                    return (
                        200,
                        vec![("Content-Type".into(), "text/x-component".into())],
                        include_str!("../../../leaflet/web/tests/fixtures/mint-result.flight").into(),
                    );
                }
                "/new" => return (307, vec![("Location".into(), format!("/{}", store::TOKEN))], String::new()),
                p if p.ends_with("/publish") && r.method == "GET" => "<script src=\"/_next/static/publish.js\"></script>".into(),
                p if p.ends_with("/publish") => {
                    sent.fetch_add(1, Ordering::SeqCst);
                    let source = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
                    assert!(
                        matches!(source.state.publication_attempt.unwrap().stage, PublicationStage::Dispatched { .. }),
                        "checkpoint MUST precede send"
                    );
                    let args = r.body_json()[0].clone();
                    assert_eq!(args["sendEmail"], false);
                    assert!(args.get("rkey").is_none() && args.get("entitiesToDelete").is_none());
                    let first = crate::published::association(&data(&store)).unwrap().is_none();
                    if mode == 2 {
                        return (503, vec![], "lost before effect".into());
                    }
                    let mut record = record(&mut store, publication, &args);
                    if mode == 3 {
                        record["title"] = json!("wrong meaning");
                    }
                    if mode == 8 {
                        record["publishedAt"] = json!("2026-01-01T00:00:00Z");
                    }
                    if mode == 9 {
                        record["path"] = json!("/wrong");
                    }
                    let table = if publication { "leaflets_in_publications" } else { "leaflets_to_documents" };
                    if !publication {
                        store.data[table] = json!([{"leaflet":store::TOKEN,"preferences":null}]);
                    }
                    store.data[table][0][if publication { "doc" } else { "document" }] = json!(URI);
                    store.data[table][0]["documents"] = json!({"uri":URI,"data":record});
                    for key in ["title", "description", "tags"] {
                        store.extras.insert(format!("publication_{key}"), args[key].clone());
                    }
                    if publication {
                        store.extras.insert("draft_contributors".into(), json!([OWNER]));
                    }
                    if mode == 4 {
                        let source = std::fs::read_to_string(&path).unwrap();
                        std::fs::write(&path, format!("{source}\nChanged during publish.\n")).unwrap();
                    }
                    if mode == 1 {
                        return (503, vec![], "lost after effect".into());
                    }
                    let result = json!({"success":true,"uri":URI,"rkey":"post","record":record,"firstPublish":first,"blocks":1});
                    return (
                        200,
                        vec![("Content-Type".into(), "text/x-component".into())],
                        format!("0:{{\"a\":\"$@1\"}}\n1:{result}\n"),
                    );
                }
                p if p.starts_with("/xrpc/com.atproto.repo.getRecord") => {
                    let record = &store.data[if publication { "leaflets_in_publications" } else { "leaflets_to_documents" }][0]["documents"]["data"];
                    let cid = leafbud_atproto::content::record_content_cid(record).unwrap();
                    json!({"uri":URI,"cid":if mode == 5 {"bafyreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"}else{&cid},"value":record}).to_string()
                }
                "/api/rpc/get_leaflet_data" if mode == 7 && sent.load(Ordering::SeqCst) > 0 => {
                    return (503, vec![], "post-dispatch observation unavailable".into());
                }
                _ => {
                    let response = store
                        .rpc(&r.path, &r.body_json())
                        .unwrap_or_else(|| panic!("unexpected {} {}", r.method, r.path));
                    if mode == 12 && r.path == "/api/rpc/push" {
                        let fixtures: Value = serde_json::from_str(include_str!("../../tests/fixtures/native-publication-size.json")).unwrap();
                        let payload = fixtures["fixtures"][0]["facts"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|f| f["attribute"] == "block/text")
                            .unwrap()["data"]
                            .clone();
                        for row in store.facts.values_mut().filter(|r| r["attribute"] == "block/text") {
                            row["data"] = payload.clone();
                        }
                    }
                    response.to_string()
                }
            };
            (200, vec![], response)
        }))
        .await;
        Self { http, store, mode, calls }
    }
    fn reader(&self) -> RemoteReader {
        RemoteReader::new(leafbud_web::LeafletClient::new(&self.http.base_url, Some("synthetic")).unwrap())
    }
    fn context(&self, root: &Path) -> config::Context {
        let mut context = support::context(root);
        context.effective.pds = Some(self.http.base_url.clone());
        context
    }
}
fn draft(path: &Path, publication: bool) {
    std::fs::write(path,format!("---\nformat: leafbud-mdx/2\ntitle: Disposable fixture\npublication: {}\n# retain author comment\nlocalOnly: keep\n\"@leafbud\": {{id: \"{}\", state: draft}}\n---\n\n## Added heading\n\nAdded paragraph.\n",if publication{format!("\"{PUBLICATION}\"")}else{"null".into()},Uuid::new_v4())).unwrap();
}
fn read(path: &Path) -> Document {
    leafbud_mdx::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[tokio::test]
async fn malformed_release_metadata_refuses_before_auth_mint_or_save() {
    for field in ["publishedAt: not-a-date", "path: 17"] {
        let root=tempfile::tempdir().unwrap(); let path=root.path().join("post.mdx"); draft(&path,false);
        let source=std::fs::read_to_string(&path).unwrap().replace("# retain author comment",field);
        std::fs::write(&path,&source).unwrap();
        let server=Server::new(false,path.clone()).await;
        let report=server.reader().publish_native(&server.context(root.path()),&path,Selection::default(),PublishOptions{yes:true,..Default::default()}).await.unwrap();
        assert_eq!(report.outcome,"blocked","{report:?}"); assert!(server.http.requests().is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(),source);
    }
}

#[tokio::test]
async fn first_from_unbound_update_same_uri_and_clean_repeat_both_contexts() {
    for publication in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("post.mdx");
        draft(&path, publication);
        let server = Server::new(publication, path.clone()).await;
        let reader = server.reader();
        let context = server.context(root.path());
        let id = read(&path).state.id;
        let result = reader
            .publish_native(
                &context,
                &path,
                Selection::default(),
                PublishOptions {
                    yes: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(result.outcome, "published", "{result:?}");
        let doc = read(&path);
        assert_eq!(doc.state.id, id);
        assert_eq!(doc.state.binding.as_ref().unwrap().uri.as_deref(), Some(URI));
        assert!(doc.state.released.is_some() && doc.state.publication_attempt.is_none());
        let bytes = std::fs::read(&path).unwrap();
        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        let repeat = reader
            .publish_native(
                &context,
                &path,
                Selection::default(),
                PublishOptions {
                    yes: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(repeat.outcome, "unchanged", "{repeat:?}");
        assert!(repeat.effects.is_empty());
        assert_eq!(server.calls.load(Ordering::SeqCst), 1);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, text.replace("Added paragraph.", "Updated paragraph.\n\nNew paragraph.")).unwrap();
        let update = reader
            .publish_native(
                &context,
                &path,
                Selection::default(),
                PublishOptions {
                    yes: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(update.outcome, "published", "{update:?}");
        assert_eq!(server.calls.load(Ordering::SeqCst), 2);
        assert_eq!(read(&path).state.binding.as_ref().unwrap().uri, doc.state.binding.as_ref().unwrap().uri);
        assert!(std::fs::read_to_string(&path).unwrap().contains("# retain author comment"));
        assert_eq!(
            crate::release::observe(&read(&path), Some(&PublicReader::new(context.effective.pds.as_deref()).unwrap()), false)
                .await
                .state,
            "clean"
        );
    }
}

#[tokio::test]
async fn lost_response_observes_without_retry_and_never_blind_replays() {
    for mode in [1, 2, 3, 4, 5, 8, 9] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("post.mdx");
        draft(&path, false);
        let server = Server::new(false, path.clone()).await;
        server.mode.store(mode, Ordering::SeqCst);
        let reader = server.reader();
        let context = server.context(root.path());
        let result = reader
            .publish_native(
                &context,
                &path,
                Selection::default(),
                PublishOptions {
                    yes: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(
            result.outcome,
            if mode == 1 {
                "published"
            } else if mode == 4 {
                "source-changed"
            } else if matches!(mode, 3 | 8 | 9) {
                "partial-unsupported"
            } else {
                "uncertain"
            },
            "mode {mode}: {result:?}"
        );
        assert_eq!(server.calls.load(Ordering::SeqCst), 1);
        if mode != 1 {
            assert!(read(&path).state.publication_attempt.is_some() && read(&path).state.released.is_none());
            assert!(
                reader
                    .save_native(
                        &context,
                        &path,
                        Selection::default(),
                        SaveOptions {
                            yes: true,
                            take_local: true,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            let recovery = reader
                .recover_editor(
                    &context,
                    &path,
                    Selection::default(),
                    RecoverOptions {
                        yes: true,
                        take: Some(ResolutionDirection::Local),
                    },
                )
                .await
                .unwrap();
            assert!(
                matches!(recovery.outcome, "uncertain" | "source-changed" | "partial-unsupported"),
                "{recovery:?}"
            );
            assert_eq!(server.calls.load(Ordering::SeqCst), 1);
        }
    }
}

#[tokio::test]
async fn editor_join_and_post_dispatch_crash_windows_are_finite_observation_only() {
    for mode in [6, 7] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("post.mdx");
        draft(&path, false);
        let server = Server::new(false, path.clone()).await;
        server.mode.store(mode, Ordering::SeqCst);
        let reader = server.reader();
        let context = server.context(root.path());
        let result = reader
            .publish_native(
                &context,
                &path,
                Selection::default(),
                PublishOptions {
                    yes: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(result.outcome, if mode == 6 { "blocked" } else { "uncertain" }, "{result:?}");
        let doc = read(&path);
        assert!(doc.state.pending.is_none() && doc.state.sync.is_some());
        assert_eq!(
            matches!(doc.state.publication_attempt.unwrap().stage, PublicationStage::EditorAchieved),
            mode == 6
        );
        server.mode.store(0, Ordering::SeqCst);
        let result = reader.recover_publication(&context, &path, Selection::default(), true).await.unwrap();
        assert_eq!(
            result.outcome,
            if mode == 6 { "editor-achieved-release-undispatched" } else { "published" },
            "{result:?}"
        );
        assert_eq!(server.calls.load(Ordering::SeqCst), usize::from(mode == 7));
        assert!(read(&path).state.publication_attempt.is_none());
    }
}

#[tokio::test]
async fn byline_upsert_refuses_before_editor_effects_and_drift_requires_take_local() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("post.mdx");
    draft(&path, true);
    let server = Server::new(true, path.clone()).await;
    server
        .store
        .lock()
        .unwrap()
        .extras
        .insert("draft_contributors".into(), json!(["did:plc:other"]));
    let result = server
        .reader()
        .publish_native(
            &server.context(root.path()),
            &path,
            Selection::default(),
            PublishOptions {
                yes: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(result.outcome, "pending", "{result:?}"); // mint locator/defaults, no authored save
    assert_eq!(server.store.lock().unwrap().pushes, 0);
    assert_eq!(server.calls.load(Ordering::SeqCst), 0);
    assert!(result.message.unwrap().contains("multi-contributor"));

    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("post.mdx");
    draft(&path, false);
    let server = Server::new(false, path.clone()).await;
    let reader = server.reader();
    let context = server.context(root.path());
    assert_eq!(
        reader
            .save_native(
                &context,
                &path,
                Selection::default(),
                SaveOptions {
                    yes: true,
                    ..Default::default()
                }
            )
            .await
            .unwrap()
            .outcome,
        "saved"
    );
    server.store.lock().unwrap().data["description"] = json!("remote drift");
    let source = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, source.replace("Added paragraph.", "Local change.")).unwrap();
    let report = reader
        .publish_native(
            &context,
            &path,
            Selection::default(),
            PublishOptions {
                yes: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.outcome, "blocked", "{report:?}");
    assert_eq!(server.calls.load(Ordering::SeqCst), 0);
    let report = reader
        .publish_native(&context, &path, Selection::default(), PublishOptions { yes: true, take_local: true })
        .await
        .unwrap();
    assert_eq!(report.outcome, "published", "{report:?}");
}

#[tokio::test]
async fn fresh_post_save_fragmentation_refuses_dispatch_despite_small_compiled_estimate() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("post.mdx");
    draft(&path, false);
    let fixtures: Value = serde_json::from_str(include_str!("../../tests/fixtures/native-publication-size.json")).unwrap();
    let fixture = &fixtures["fixtures"][0];
    let observed = leafbud_editor::observe(fixture["facts"].as_array().unwrap(), fixture["root"].as_str().unwrap(), Uuid::new_v4()).unwrap();
    let mut doc = read(&path);
    doc.pages = observed.pages;
    doc.state.main_page = Some(doc.pages[0].key.clone());
    std::fs::write(&path, leafbud_mdx::render(&doc).unwrap()).unwrap();
    let server = Server::new(false, path.clone()).await;
    server.mode.store(12, Ordering::SeqCst);
    let report = server
        .reader()
        .publish_native(
            &server.context(root.path()),
            &path,
            Selection::default(),
            PublishOptions {
                yes: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.outcome, "blocked", "{report:?}");
    assert!(report.message.unwrap().contains("fresh saved native pages JSON"));
    assert_eq!(server.calls.load(Ordering::SeqCst), 0);
    assert!(server.store.lock().unwrap().pushes > 0);
    assert!(matches!(read(&path).state.publication_attempt.unwrap().stage, PublicationStage::EditorAchieved));
}
