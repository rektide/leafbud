use super::*;
use crate::import;
use crate::{import::tests::testserver::TestServer, published::tests as published_support, pull::tests as support};
use leafbud_web::{LeafletClient, parse_pull};
use std::sync::Arc;

const OWNER: &str = "did:plc:abcdefghijklmnopqrstuvwx";
const PUBLICATION: &str = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.publication/garden";
fn now() -> jiff::Timestamp {
    "2026-11-10T12:00:00Z".parse().unwrap()
}
fn context(root: &Path) -> config::Context {
    config::load_at(&root.join("absent.toml"), root, Some(root), &config::Config::default(), None).unwrap()
}
fn fixture(publication: bool, published: bool) -> Value {
    let mut fixture = published_support::fixture(publication);
    fixture["leaflet_data"]["reply_drafts"] = json!([]);
    fixture["leaflet_data"]["publication_scheduled_posts"] = json!([]);
    let link = &mut fixture["leaflet_data"][if publication { "leaflets_in_publications" } else { "leaflets_to_documents" }][0];
    link["preferences"] = Value::Null;
    if publication {
        link["publications"]["publication_contributors"] = json!([]);
    }
    if !published {
        link[if publication { "doc" } else { "document" }] = Value::Null;
        link["documents"] = Value::Null;
    }
    if !publication && !published {
        for op in fixture["response"]["patch"].as_array_mut().unwrap() {
            if op["key"] == "publication_tags" {
                op["value"] = json!([]);
            }
        }
    }
    fixture
}
fn home(actor: &str) -> String {
    let identity = json!({"atp_did":actor,"bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]});
    format!("<script>self.__next_f.push({})</script>", json!([1, format!("0:{identity}")]))
}
async fn serving(fixture: Value, actor: &'static str, on_read: impl Fn() + Send + Sync + 'static) -> TestServer {
    TestServer::spawn(Arc::new(move |r| {
        (
            200,
            match r.path.as_str() {
                "/api/rpc/get_leaflet_data" => json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string(),
                "/api/rpc/pull" => {
                    on_read();
                    fixture["response"].to_string()
                }
                "/home" => home(actor),
                "/_next/static/publish.js" => include_str!("../../../leaflet/web/tests/fixtures/publish-reference.js").into(),
                path if r.method == "GET" && path.ends_with("/publish") => "<script src=\"/_next/static/publish.js\"></script>".into(),
                _ => panic!("preview called an effect endpoint: {} {}", r.method, r.path),
            },
        )
    }))
    .await
}
fn document(fixture: &Value, base: &str) -> Document {
    import::build(base, &support::data(fixture), &parse_pull(fixture["response"].clone()).unwrap()).unwrap()
}

#[tokio::test]
async fn first_and_update_preview_normalization_no_file_or_effect_changes() {
    for publication in [false, true] {
        for published in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let context = context(root.path());
            let fixture = fixture(publication, published);
            let server = serving(fixture.clone(), OWNER, || {}).await;
            let doc = document(&fixture, &server.base_url);
            let path = root.path().join("post.mdx");
            let source = leafbud_mdx::render(&doc).unwrap();
            std::fs::write(&path, &source).unwrap();
            let before = std::fs::metadata(&path).unwrap().modified().unwrap();
            let remote = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
            let mut context = context;
            context.effective.publication = Some("at://did:plc:wrong/site.standard.publication/default".into());
            let report = preview(&context, &path, Default::default(), Some(&remote), now()).await.unwrap();
            assert_eq!(report.mode, if published { "update-same-URI" } else { "first-release" });
            assert_eq!(report.owner.as_deref(), Some(OWNER));
            assert_eq!(report.publication.as_deref(), publication.then_some(PUBLICATION));
            assert!(!report.send_email && !report.bluesky_share);
            assert_eq!(report.path["kind"], if published { "fixed-by-existing-rkey" } else { "server-chosen-rkey" });
            assert_eq!(report.date["kind"], if published { "explicit" } else { "server-now-on-first-release" });
            assert!(!report.blockers.iter().any(|s| s.contains("execution/recovery")));
            let size = report.inline_size.as_ref().expect("actual native facts witnessed");
            assert!(size.no_offload_at_witness);
            assert_eq!(size.scope.contains("conditional-on-save"), report.editor == "save-required");
            assert!(!report.blockers.iter().any(|s| s.contains("inline size") || s.contains("size profile")));
            assert!(!report.blockers.iter().any(|s| s.contains("prunes")));
            assert_eq!(report.native_effects.contains(&"old image copies may be deleted; count unknown"), publication);
            assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
            assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), before);
            assert!(
                server
                    .requests()
                    .iter()
                    .all(|r| r.method == "GET" || matches!(r.path.as_str(), "/api/rpc/get_leaflet_data" | "/api/rpc/pull"))
            );
            assert_eq!(server.request_count("/new"), 0);
        }
    }
}

#[tokio::test]
async fn unbound_new_structure_and_unsupported_body_preflight_before_remote_reads() {
    let root = tempfile::tempdir().unwrap();
    let context = context(root.path());
    let path = root.path().join("post.mdx");
    let header = "---\nformat: leafbud-mdx/2\ntitle: T\npublication: null\n\"@leafbud\":\n  id: 11111111-1111-4111-8111-111111111111\n  state: draft\n---\n\n";
    let source = format!("{header}## New heading\n\nNew paragraph.\n");
    std::fs::write(&path, &source).unwrap();
    let report = preview(&context, &path, Default::default(), None, now()).await.unwrap();
    assert!(report.keys_to_prepare >= 2);
    assert_eq!(report.native_pages_after_save, 1);
    assert_eq!(report.editor, "bind-and-save-required");
    assert!(report.inline_size.is_none());
    assert!(report.blockers.iter().any(|s| s.contains("inline size not witnessed")));
    assert_eq!(report.uri, None);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    let server = serving(fixture(false, false), OWNER, || {}).await;
    let remote = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
    std::fs::write(
        &path,
        format!("{header}Paragraph before unsupported release resource.\n\n<Image src={{\"https://example.test/a.png\"}} fallback={{null}} width={{42}} height={{42}} alt={{null}} />\n"),
    )
    .unwrap();
    let error = preview(&context, &path, Default::default(), Some(&remote), now()).await.unwrap_err();
    assert!(format!("{error:#}").contains("images"));
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn unsaved_content_reported_without_key_or_receipt_preparation() {
    let root = tempfile::tempdir().unwrap();
    let context = context(root.path());
    let fixture = fixture(false, true);
    let server = serving(fixture.clone(), OWNER, || {}).await;
    let path = root.path().join("post.mdx");
    let source = format!(
        "{}\nA new paragraph before publishing.\n",
        leafbud_mdx::render(&document(&fixture, &server.base_url)).unwrap()
    );
    std::fs::write(&path, &source).unwrap();
    let remote = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
    let report = preview(&context, &path, Default::default(), Some(&remote), now()).await.unwrap();
    assert_eq!(report.editor, "save-required");
    assert_eq!(report.keys_to_prepare, 1);
    assert!(!report.editor_effects.is_empty());
    let size = report.inline_size.as_ref().unwrap();
    assert!(size.no_offload_at_witness);
    assert!(size.scope.contains("conditional-on-save"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
}

#[tokio::test]
async fn native_fact_race_after_size_proof_invalidates_preview_without_effects() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let root = tempfile::tempdir().unwrap();
    let context = context(root.path());
    let fixture = fixture(false, true);
    let source_fixture = fixture.clone();
    let pulls = Arc::new(AtomicUsize::new(0));
    let count = pulls.clone();
    let server = TestServer::spawn(Arc::new(move |r| {
        (
            200,
            match r.path.as_str() {
                "/api/rpc/get_leaflet_data" => json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string(),
                "/api/rpc/pull" => {
                    let mut response = fixture.clone();
                    if count.fetch_add(1, Ordering::SeqCst) > 0 {
                        support::set_paragraph_text(&mut response, &support::yrs_payload("Changed during action discovery", None));
                    }
                    response["response"].to_string()
                }
                "/home" => home(OWNER),
                "/_next/static/publish.js" => include_str!("../../../leaflet/web/tests/fixtures/publish-reference.js").into(),
                path if r.method == "GET" && path.ends_with("/publish") => "<script src=\"/_next/static/publish.js\"></script>".into(),
                _ => panic!("publication preview attempted an effect"),
            },
        )
    }))
    .await;
    let path = root.path().join("post.mdx");
    let source = leafbud_mdx::render(&document(&source_fixture, &server.base_url)).unwrap();
    std::fs::write(&path, &source).unwrap();
    let before = std::fs::metadata(&path).unwrap().modified().unwrap();
    let remote = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
    let error = preview(&context, &path, Default::default(), Some(&remote), now()).await.unwrap_err();
    assert!(format!("{error:#}").contains("size/input witness invalidated"));
    assert_eq!(pulls.load(Ordering::SeqCst), 2);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), before);
}

#[tokio::test]
async fn source_and_late_duplicate_races_refuse_after_remote_await() {
    for duplicate in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let context = context(root.path());
        let fixture = fixture(false, true);
        let path = root.path().join("post.mdx");
        let target = if duplicate { root.path().join("copy.mdx") } else { path.clone() };
        let original_path = path.clone();
        let server = serving(fixture.clone(), OWNER, move || {
            let source = std::fs::read_to_string(&original_path).unwrap();
            std::fs::write(&target, if duplicate { source } else { format!("{source}\nChanged while waiting.\n") }).unwrap();
        })
        .await;
        let source = leafbud_mdx::render(&document(&fixture, &server.base_url)).unwrap();
        std::fs::write(&path, &source).unwrap();
        let remote = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
        let error = preview(&context, &path, Default::default(), Some(&remote), now()).await.unwrap_err();
        assert!(format!("{error:#}").contains(if duplicate { "duplicate" } else { "source changed" }));
        assert_eq!(server.request_count("/home"), 0);
    }
}

#[tokio::test]
async fn rights_owner_binding_and_inherited_public_fields_refuse_without_effects() {
    for case in 0..4 {
        let root = tempfile::tempdir().unwrap();
        let context = context(root.path());
        let baseline = fixture(false, true);
        let mut changed = baseline.clone();
        match case {
            0 => changed["leaflet_data"]["permission_token_rights"][0]["write"] = json!(false),
            1 => {} // Different authenticated actor below cannot update this owner.
            2 => {
                changed["leaflet_data"]["leaflets_to_documents"][0]["documents"]["data"]["preferences"] =
                    json!({"$type":"pub.leaflet.publication#preferences","showInDiscover":false})
            }
            _ => changed["leaflet_data"]["leaflets_to_documents"][0]["document"] = json!("at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/other"),
        }
        let server = serving(changed, if case == 1 { "did:plc:intruder" } else { OWNER }, || {}).await;
        let path = root.path().join("post.mdx");
        let source = leafbud_mdx::render(&document(&baseline, &server.base_url)).unwrap();
        std::fs::write(&path, &source).unwrap();
        let remote = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
        assert!(preview(&context, &path, Default::default(), Some(&remote), now()).await.is_err(), "case {case}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
        assert!(
            server
                .requests()
                .iter()
                .all(|r| r.method == "GET" || matches!(r.path.as_str(), "/api/rpc/get_leaflet_data" | "/api/rpc/pull"))
        );
    }
}

#[test]
fn explicit_future_dates_preservation_and_native_effect_blockers() {
    let fixture = fixture(true, true);
    let mut doc = document(&fixture, "https://leaflet.pub");
    let mut report = Preview {
        standing: "internal",
        mode: "update",
        keys_to_prepare: 0,
        native_pages_after_save: 0,
        editor: "unknown",
        editor_effects: Default::default(),
        inline_size: None,
        actor: None,
        owner: None,
        publication: Some(PUBLICATION.into()),
        uri: doc.state.binding.as_ref().unwrap().uri.clone(),
        path: Value::Null,
        date: Value::Null,
        send_email: false,
        bluesky_share: false,
        native_effects: vec![],
        blockers: vec![],
    };
    doc.extra.insert("path".into(), json!("/custom"));
    doc.extra.insert("publishedAt".into(), json!("2027-01-01T00:00:00Z"));
    normalize(&doc, None, now(), &mut report).unwrap();
    assert_eq!(report.path["actual"], "/post");
    assert_eq!(report.path["normalizes"], true);
    assert_eq!(report.date["actual"], Value::Null);
    assert_eq!(report.date["kind"], "native-future-clamped-to-server-now");
    doc.extra.remove("publishedAt");
    normalize(&doc, Some(&json!({"publishedAt":"2026-10-01T10:00:00Z"})), now(), &mut report).unwrap();
    assert_eq!(report.date["kind"], "preserve-observed");
    let mut raw = fixture.clone();
    raw["leaflet_data"]["reply_drafts"] = json!([{"subject":"at://did:plc:x/site.standard.document/reply"}]);
    inspect_effect_fields(&support::data(&raw), &mut report).unwrap();
    assert!(report.blockers.iter().any(|s| s.contains("reply_drafts")));
    assert!(verify_owner(&doc, &support::data(&raw), "did:plc:intruder", &mut report).is_err());
}
