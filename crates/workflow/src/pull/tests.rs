use super::*;
use crate::import;
use leafspring_document::BlockKind;
use leafspring_web::{LeafletClient, LeafletData, parse_pull, parse_rpc};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

// The loopback responder is already a module of this crate's test build
// (import tests); reusing it avoids a duplicate module load.
use crate::import::tests::testserver;

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const TOKEN2: &str = "01999000-0000-7aaa-8000-000000000002";
const TOKEN3: &str = "01999000-0000-7aaa-8000-000000000003";

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../../experiments/02-draft-internals/fixtures/export-pull-doc.json")).unwrap()
}

/// A fixture variant bound to a distinct permission token, so several
/// independent documents can coexist in one test workspace.
fn fixture_with_token(token: &str) -> Value {
    let mut fixture = fixture();
    fixture["leaflet_data"]["id"] = json!(token);
    fixture["leaflet_data"]["permission_token_rights"][0]["token"] = json!(token);
    fixture
}
fn data(fixture: &Value) -> LeafletData {
    parse_rpc(&json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()).unwrap()
}

/// A fresh native text payload with real Yrs state (optionally a strong mark).
fn yrs_payload(text: &str, strong_from: Option<(usize, usize)>) -> String {
    yrs_link_payload(text, strong_from, None)
}

/// Like [`yrs_payload`], optionally adding one href-only link mark.
fn yrs_link_payload(text: &str, strong_from: Option<(usize, usize)>, link: Option<(usize, usize, &str)>) -> String {
    use base64::Engine;
    use yrs::types::Attrs;
    use yrs::{Doc, Text as _, Transact, XmlFragment as _};
    let doc = Doc::new();
    let fragment = doc.get_or_insert_xml_fragment("prosemirror");
    let mut txn = doc.transact_mut();
    let paragraph = fragment.insert(&mut txn, 0, yrs::XmlElementPrelim::empty("paragraph"));
    let node = paragraph.insert(&mut txn, 0, yrs::XmlTextPrelim::new(text));
    if let Some((start, len)) = strong_from {
        let mut attrs = Attrs::new();
        attrs.insert("strong".into(), yrs::Any::Bool(true));
        node.format(&mut txn, start as u32, len as u32, attrs);
    }
    if let Some((start, len, href)) = link {
        let mut attrs = Attrs::new();
        attrs.insert("link".into(), yrs::Any::from_json(&json!({ "href": href }).to_string()).unwrap());
        node.format(&mut txn, start as u32, len as u32, attrs);
    }
    let update = txn.encode_update_v1();
    drop(txn);
    base64::engine::general_purpose::STANDARD.encode(update)
}

fn set_paragraph_text(fixture: &mut Value, payload: &str) {
    let response = fixture
        .get_mut("response")
        .and_then(|r| r.get_mut("patch"))
        .and_then(Value::as_array_mut)
        .expect("fixture response");
    let entity = response
        .iter()
        .find(|op| op["value"]["attribute"] == "block/type" && op["value"]["data"]["value"] == "text")
        .map(|op| op["value"]["entity"].clone())
        .expect("text block");
    for op in response {
        let Some(value) = op.get_mut("value") else { continue };
        if value["attribute"] == "block/text" && value["entity"] == entity {
            value["data"]["value"] = json!(payload);
        }
    }
}

fn swap_block_order(fixture: &mut Value) {
    let response = fixture
        .get_mut("response")
        .and_then(|r| r.get_mut("patch"))
        .and_then(Value::as_array_mut)
        .expect("fixture response");
    let mut placements: Vec<Value> = response
        .iter()
        .filter(|op| op["value"]["attribute"] == "card/block")
        .map(|op| op["value"]["data"]["position"].clone())
        .collect();
    assert_eq!(placements.len(), 2, "fixture must have two placements");
    placements.reverse();
    for (op, position) in response.iter_mut().filter(|op| op["value"]["attribute"] == "card/block").zip(placements) {
        op["value"]["data"]["position"] = position;
    }
}

type Fixtures = Arc<RwLock<BTreeMap<String, Value>>>;

/// A loopback origin serving the given token → fixture map. Unknown tokens
/// read as missing objects. RPC bodies carry token_id for both endpoints.
async fn serving(fixtures: BTreeMap<String, Value>) -> (testserver::TestServer, Fixtures) {
    let current: Fixtures = Arc::new(RwLock::new(fixtures));
    let handle = current.clone();
    let server = testserver::TestServer::spawn(Arc::new(move |r| {
        let fixtures = handle.read().expect("fixtures").clone();
        let token = r.body_json()["token_id"].as_str().unwrap_or_default().to_owned();
        let Some(fixture) = fixtures.get(&token) else {
            return (
                200,
                r#"{"result":{"error":{"code":"PGRST116","message":"withheld"},"status":406,"data":null}}"#.into(),
            );
        };
        match r.path.as_str() {
            "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()),
            "/api/rpc/pull" => (200, fixture["response"].to_string()),
            _ => (404, String::new()),
        }
    }))
    .await;
    (server, current)
}

async fn server_current() -> (testserver::TestServer, Fixtures) {
    serving(BTreeMap::from([(TOKEN.to_owned(), fixture())])).await
}

fn context(root: &Path) -> config::Context {
    config::load_at(&root.join("absent.toml"), root, Some(root), &config::Config::default(), None).unwrap()
}

async fn seed(server: &testserver::TestServer, root: &Path, token: &str, to: &str) -> PathBuf {
    let reader = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
    let report = reader
        .import(
            &context(root),
            import::Request {
                locator: Some(token.to_owned()),
                to: Some(to.into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!((report.imported, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    report.items[0].path.clone().unwrap()
}

fn reader(server: &testserver::TestServer) -> RemoteReader {
    RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap())
}

fn request(files: Vec<PathBuf>) -> Request {
    Request { files, ..Default::default() }
}

fn keys_of(source: &str) -> BTreeSet<String> {
    leafspring_mdx::parse(source).unwrap().state.nodes.keys().cloned().collect()
}

#[tokio::test]
async fn local_only_intent_and_key_spelling_never_conflict_with_remote_editor_edits() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let source = std::fs::read_to_string(&path).unwrap();
    let old_key = leafspring_mdx::parse(&source).unwrap().pages[0].blocks[0].key.clone();
    let source = source.replace(&format!("\"{old_key}\""), "\"friendly-key\"").replacen(
        "\n\"@leafspring\":",
        "\n# preserve intent\npublishedAt: '2030-01-01'\ncustomField: {nested: [1, 2]}\n\"@leafspring\":",
        1,
    );
    std::fs::write(&path, &source).unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.unchanged, 1);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    set_paragraph_text(current.write().unwrap().get_mut(TOKEN).unwrap(), &yrs_payload("Remote update", None));
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.updated, report.needs_decision, report.failed), (1, 0, 0));
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("Remote update"));
    assert!(after.contains("friendly-key"));
    assert!(after.contains("# preserve intent\npublishedAt: '2030-01-01'\ncustomField: {nested: [1, 2]}"));
}

#[tokio::test]
async fn legacy_receipts_require_current_agreement_to_settle_and_unknown_profiles_never_mutate() {
    use leafspring_document::{EDITOR_PROFILE, PROFILE};
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let source = std::fs::read_to_string(&path).unwrap();
    let mut legacy = leafspring_mdx::parse(&source).unwrap();
    legacy.state.sync.profile = PROFILE.into();
    legacy.state.sync.local_hash = import::hash(&legacy.semantic_value());
    let fixture = fixture();
    legacy.state.sync.remote_hash = import::remote_hash(&data(&fixture), &parse_pull(fixture["response"].clone()).unwrap(), PROFILE);
    let source = leafspring_mdx::update(&source, &legacy).unwrap();
    std::fs::write(&path, &source).unwrap();
    set_paragraph_text(
        current.write().unwrap().get_mut(TOKEN).unwrap(),
        &yrs_payload("Different current endpoint", None),
    );
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.needs_decision, 1);
    assert!(report.items[0].error.as_ref().unwrap().contains("baseline unverified"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    *current.write().unwrap().get_mut(TOKEN).unwrap() = fixture.clone();
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.converged, 1);
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(after.split_once("---\n\n").unwrap().1, source.split_once("---\n\n").unwrap().1);
    assert_eq!(leafspring_mdx::parse(&after).unwrap().state.sync.profile, EDITOR_PROFILE);
    assert_eq!(
        reader(&server)
            .pull(&context(temp.path()), request(vec![path.clone()]))
            .await
            .unwrap()
            .unchanged,
        1
    );
    let unknown = after.replace(EDITOR_PROFILE, "future-editor/99");
    std::fs::write(&path, &unknown).unwrap();
    let report = reader(&server)
        .pull(
            &context(temp.path()),
            Request {
                files: vec![path.clone()],
                take_remote: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.failed, 1);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), unknown);
}

#[tokio::test]
async fn pull_noop_is_an_exact_byte_and_mtime_noop() {
    let (server, _current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let bytes = std::fs::read(&path).unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.unchanged, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    // Import used two metadata reads and one content read; the no-op refresh
    // adds exactly one of each and takes no write path.
    assert_eq!(server.request_count("/api/rpc/get_leaflet_data"), 3);
    assert_eq!(server.request_count("/api/rpc/pull"), 2);
}

/// A genuinely changed link-bearing remote body adopts through the
/// Markdown-first writer; the following no-op pull keeps those bytes exact.
#[tokio::test]
async fn pull_adopts_link_bodies_as_markdown_and_repeat_is_exact_noop() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    {
        let mut fixtures = current.write().unwrap();
        let fixture = fixtures.get_mut(TOKEN).unwrap();
        set_paragraph_text(fixture, &yrs_link_payload("see the docs now", None, Some((8, 4, "https://e.org/docs"))));
    }
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.updated, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("see the [docs](https://e.org/docs) now"), "ordinary link spelling");
    assert_eq!(leafspring_mdx::parse(&after).unwrap(), leafspring_mdx::parse(&after).unwrap());
    // No-op repeat: bytes and mtimes identical.
    let metadata = std::fs::symlink_metadata(&path).unwrap().modified().unwrap();
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.unchanged, report.updated), (1, 0));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
    assert_eq!(std::fs::symlink_metadata(&path).unwrap().modified().unwrap(), metadata);
}

#[tokio::test]
async fn pull_adopts_remote_text_mark_move_and_metadata_preserving_identities() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let before = std::fs::read_to_string(&path).unwrap();
    {
        let mut fixtures = current.write().unwrap();
        let fixture = fixtures.get_mut(TOKEN).unwrap();
        set_paragraph_text(fixture, &yrs_payload("Edited remote body", Some((7, 6))));
        swap_block_order(fixture);
        fixture["leaflet_data"]["title"] = json!("New remote title");
    }
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.updated, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    let after = std::fs::read_to_string(&path).unwrap();
    // Same source-key set: refresh never rekeys existing files.
    assert_eq!(keys_of(&before), keys_of(&after));
    let document = leafspring_mdx::parse(&after).unwrap();
    assert_eq!(document.title, "New remote title");
    let runs: Vec<_> = document.pages[0]
        .blocks
        .iter()
        .flat_map(|b| match &b.kind {
            BlockKind::Paragraph { content } | BlockKind::Heading { content, .. } => content.iter(),
            _ => [].iter(),
        })
        .collect();
    assert!(
        runs.contains(&&leafspring_document::Inline::Run {
            text: "remote".into(),
            marks: vec![leafspring_document::Mark::Strong],
        }),
        "{runs:?}"
    );
    // The moved paragraph now precedes the heading (order adopted).
    assert!(after.find("Edited ").unwrap() < after.find("Synthetic title").unwrap());
    assert!(after.find(" body").unwrap() > after.find("Edited ").unwrap());
    // Receipts correspond to the current remote observation.
    let now = current.read().unwrap().get(TOKEN).unwrap().clone();
    let fresh = import::build(&server.base_url, &data(&now), &parse_pull(now["response"].clone()).unwrap()).unwrap();
    assert_eq!(document.state.sync.remote_hash, fresh.state.sync.remote_hash);
    assert_eq!(document.state.sync.local_hash, import::hash(&document.editor_value()));
}

#[tokio::test]
async fn pull_witness_only_change_settles_receipts_without_content_rewrite() {
    let mut converted = fixture();
    let entity = converted["response"]["patch"]
        .as_array()
        .unwrap()
        .iter()
        .find(|op| op["value"]["attribute"] == "block/type" && op["value"]["data"]["value"] == "text")
        .map(|op| op["value"]["entity"].clone())
        .unwrap();
    for op in converted["response"]["patch"].as_array_mut().unwrap() {
        if op["value"]["attribute"] == "block/type" && op["value"]["entity"] == entity {
            op["value"]["data"]["value"] = json!("code");
        }
    }
    converted["response"]["patch"]
        .as_array_mut()
        .unwrap()
        .push(json!({"op":"put","key":"11111111-1111-4111-8111-111111111111","value":{"id":"11111111-1111-4111-8111-111111111111","entity":entity,"attribute":"block/code","data":{"type":"string","value":"visible current code"}}}));
    let (server, current) = serving(BTreeMap::from([(TOKEN.to_owned(), converted)])).await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let before = std::fs::read_to_string(&path).unwrap();
    // Latent (inactive) text changes are witnessed but not projected content.
    {
        let mut fixtures = current.write().unwrap();
        for op in fixtures.get_mut(TOKEN).unwrap()["response"]["patch"].as_array_mut().unwrap() {
            if op["value"]["attribute"] == "block/text" && op["value"]["entity"] == entity {
                op["value"]["data"]["value"] = json!("opaque-inactive-change");
            }
        }
    }
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.receipts_only, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    let after = std::fs::read_to_string(&path).unwrap();
    let changed = before.lines().zip(after.lines()).filter(|(a, b)| a != b).collect::<Vec<_>>();
    assert_eq!(changed.len(), 1, "only the remote receipt line changes: {changed:?}");
    assert!(changed[0].1.contains("remoteHash"));
    assert!(after.contains("visible current code"));
    let document = leafspring_mdx::parse(&after).unwrap();
    assert_eq!(document.state.sync.local_hash, import::hash(&document.editor_value()));
}

#[tokio::test]
async fn pull_preserves_local_edits_and_reports_the_decision() {
    let (server, _current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let edited = std::fs::read_to_string(&path).unwrap().replace("Synthetic body copy", "Edited locally");
    std::fs::write(&path, &edited).unwrap();
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.needs_decision, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    assert!(report.items[0].error.as_deref().unwrap().contains("local edits preserved"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), edited);
    // Dry-run classifies the same way and still writes nothing.
    let report = reader(&server)
        .pull(
            &context(temp.path()),
            Request {
                files: vec![path.clone()],
                dry_run: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.needs_decision, 1);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), edited);
}

#[tokio::test]
async fn pull_conflict_needs_take_remote_which_preserves_local_metadata_and_renames() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let imported = std::fs::read_to_string(&path).unwrap();
    let old_key = leafspring_mdx::parse(&imported)
        .unwrap()
        .state
        .nodes
        .keys()
        .find(|k| k.starts_with('b'))
        .unwrap()
        .clone();
    let mut renamed = imported.replace(&format!("\"{old_key}\""), "\"my-renamed-block\"");
    assert_ne!(renamed, imported, "key appears in body and envelope");
    renamed = renamed.replacen(
        "\n\"@leafspring\":",
        "\n# author note\ncustomField: {\"meaning\": \"retained\"}\n\n\"@leafspring\":",
        1,
    );
    renamed = renamed.replace("Synthetic body copy", "Genuine local editor edit");
    std::fs::write(&path, renamed).unwrap();
    {
        let mut fixtures = current.write().unwrap();
        set_paragraph_text(fixtures.get_mut(TOKEN).unwrap(), &yrs_payload("Remote rewrote this paragraph", None));
    }
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.needs_decision, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    assert!(report.items[0].error.as_deref().unwrap().contains("diverge"));
    assert!(std::fs::read_to_string(&path).unwrap().contains("my-renamed-block"));
    let report = reader(&server)
        .pull(
            &context(temp.path()),
            Request {
                files: vec![path.clone()],
                take_remote: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.updated, 1, "{}", serde_json::to_string(&report).unwrap());
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("Remote rewrote this paragraph"));
    assert!(after.contains("\"my-renamed-block\""));
    assert!(!after.contains(&format!("\"{old_key}\"")), "manual rename survives, no rekeying");
    assert!(after.contains("# author note"));
    assert!(after.contains("customField: {\"meaning\": \"retained\"}"));
    let document = leafspring_mdx::parse(&after).unwrap();
    assert_eq!(document.extra["customField"], json!({"meaning": "retained"}));
    assert!(document.state.nodes.contains_key("my-renamed-block"));
}

#[tokio::test]
async fn pull_converged_settles_receipts_without_body_rewrite() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let edited = std::fs::read_to_string(&path).unwrap().replace("Synthetic body copy", "Merged edit");
    std::fs::write(&path, &edited).unwrap();
    {
        let mut fixtures = current.write().unwrap();
        set_paragraph_text(fixtures.get_mut(TOKEN).unwrap(), &yrs_payload("Merged edit", None));
    }
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!((report.converged, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    let after = std::fs::read_to_string(&path).unwrap();
    let (before_fm, before_body) = edited.split_once("---\n\n").unwrap();
    let (after_fm, after_body) = after.split_once("---\n\n").unwrap();
    assert_eq!(before_body, after_body, "converged endpoints never rewrite the body");
    assert_ne!(before_fm, after_fm, "receipts settle");
    let document = leafspring_mdx::parse(&after).unwrap();
    assert_eq!(document.state.sync.local_hash, import::hash(&document.editor_value()));
    // A repeat is now an exact no-op.
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.unchanged, 1);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
}

#[tokio::test]
async fn pull_missing_published_and_unsupported_remotes_are_explicit() {
    let temp = tempfile::tempdir().unwrap();
    let (server, current) = serving(BTreeMap::from([
        (TOKEN.to_owned(), fixture()),
        (TOKEN2.to_owned(), fixture_with_token(TOKEN2)),
        (TOKEN3.to_owned(), fixture_with_token(TOKEN3)),
    ]))
    .await;

    // A remote object that disappeared is distinct from an unlinked input.
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let before = std::fs::read_to_string(&path).unwrap();
    current.write().unwrap().remove(TOKEN);
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.failed, 1);
    assert!(report.items[0].error.as_deref().unwrap().contains("missing or inaccessible"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);

    // Unsupported new remote content is an explicit per-file limitation.
    let path = seed(&server, temp.path(), TOKEN2, "doc2.mdx").await;
    current.write().unwrap().get_mut(TOKEN2).unwrap()["response"]["patch"]
        .as_array_mut()
        .unwrap()
        .push(json!({"op":"put","key":"future-extra","value":{"opaque":true}}));
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.failed, 1);
    assert!(report.items[0].error.as_deref().unwrap().contains("unsupported for the refresh profile"));

    // A published remote is a lifecycle decision, never a silent adoption.
    let path = seed(&server, temp.path(), TOKEN3, "doc3.mdx").await;
    current.write().unwrap().get_mut(TOKEN3).unwrap()["leaflet_data"]["leaflets_to_documents"] =
        json!([{"document": "at://did:plc:synthetic/site.standard.document/1"}]);
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.needs_decision, 1);
    assert!(report.items[0].error.as_deref().unwrap().contains("lifecycle changed"));
}

#[tokio::test]
async fn pull_refuses_to_overwrite_a_local_edit_made_during_the_network_reads() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let temp = tempfile::tempdir().unwrap();
    let fixtures: Fixtures = Arc::new(RwLock::new(BTreeMap::from([(TOKEN.to_owned(), fixture())])));
    let racing = Arc::new(AtomicBool::new(false));
    let seeded: Arc<RwLock<Option<PathBuf>>> = Arc::new(RwLock::new(None));
    let handle = (fixtures.clone(), racing.clone(), seeded.clone());
    let server = testserver::TestServer::spawn(Arc::new(move |r| {
        let (fixtures, racing, seeded) = &handle;
        let fixture = fixtures.read().expect("fixtures").get(TOKEN).cloned().expect("token fixture");
        match r.path.as_str() {
            "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()),
            "/api/rpc/pull" => {
                // The user saves a local edit while the refresh is in flight.
                if racing.load(Ordering::Relaxed)
                    && let Some(path) = seeded.read().expect("seeded").clone()
                {
                    let mut text = std::fs::read_to_string(&path).unwrap_or_default();
                    text.push_str("<!-- concurrent local edit -->\n");
                    std::fs::write(&path, text).unwrap();
                }
                (200, fixture["response"].to_string())
            }
            _ => (404, String::new()),
        }
    }))
    .await;
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    *seeded.write().expect("seeded") = Some(path.clone());
    set_paragraph_text(
        fixtures.write().expect("fixtures").get_mut(TOKEN).unwrap(),
        &yrs_payload("Remote edit racing the user", None),
    );
    racing.store(true, Ordering::Relaxed);
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.failed, 1, "{}", serde_json::to_string(&report).unwrap());
    assert!(
        report.items[0].error.as_deref().unwrap().contains("changed during pull"),
        "{}",
        serde_json::to_string(&report).unwrap()
    );
    let now = std::fs::read_to_string(&path).unwrap();
    assert!(now.contains("<!-- concurrent local edit -->"));
    assert!(!now.contains("Remote edit racing the user"));
}

#[tokio::test]
async fn pull_all_selects_bound_files_only_with_loud_broken_and_state_filter() {
    let (server, _current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    std::fs::write(temp.path().join("unmanaged.mdx"), "just local prose\n").unwrap();
    std::fs::write(temp.path().join("notes.txt"), "not a document\n").unwrap();
    std::fs::write(temp.path().join("broken.mdx"), "---\n@leafspring: [unclosed\n---\nbody\n").unwrap();
    let report = reader(&server)
        .pull(
            &context(temp.path()),
            Request {
                all: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(
        (report.selected, report.unchanged, report.failed),
        (2, 1, 1),
        "{}",
        serde_json::to_string(&report).unwrap()
    );
    assert_eq!(report.items.iter().filter(|i| i.path == path).count(), 1, "no duplicate processing");
    assert_eq!(std::fs::read_to_string(temp.path().join("unmanaged.mdx")).unwrap(), "just local prose\n");
    assert_eq!(
        std::fs::read_to_string(temp.path().join("broken.mdx")).unwrap(),
        "---\n@leafspring: [unclosed\n---\nbody\n"
    );
    let report = reader(&server)
        .pull(
            &context(temp.path()),
            Request {
                all: true,
                state: Some(LeafletState::Published),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // The bound draft file is filtered out; the broken managed file cannot
    // prove its state and stays loud rather than being silently hidden.
    assert_eq!((report.selected, report.skipped), (1, 1));
    assert_eq!(report.failed, 1);
    let report = reader(&server)
        .pull(
            &context(temp.path()),
            Request {
                all: true,
                state: Some(LeafletState::Draft),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!((report.unchanged, report.failed), (1, 1));
}

#[tokio::test]
async fn pull_duplicate_bindings_fail_and_foreign_origins_are_refused() {
    let (server, _current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let copy = temp.path().join("copy.mdx");
    std::fs::copy(&path, &copy).unwrap();
    let report = reader(&server)
        .pull(&context(temp.path()), request(vec![path.clone(), copy.clone()]))
        .await
        .unwrap();
    assert_eq!(report.failed, 2);
    assert!(report.items.iter().all(|i| i.error.as_deref().unwrap().contains("duplicate")));

    let (other, _other_current) = server_current().await;
    let report = reader(&other)
        .pull(&context(temp.path()), request(vec![temp.path().join("doc.mdx")]))
        .await
        .unwrap();
    assert_eq!(report.failed, 1);
    assert!(report.items[0].error.as_deref().unwrap().contains("not bound to this origin"));

    std::fs::write(temp.path().join("plain.mdx"), "ordinary file\n").unwrap();
    let report = reader(&server)
        .pull(&context(temp.path()), request(vec![temp.path().join("plain.mdx")]))
        .await
        .unwrap();
    assert_eq!(report.failed, 1);
    assert!(report.items[0].error.as_deref().unwrap().contains("not a managed document"));

    assert!(
        reader(&server)
            .pull(&context(temp.path()), request(vec![temp.path().join("absent.mdx")]))
            .await
            .is_err()
    );
    assert!(reader(&server).pull(&context(temp.path()), Request { ..Default::default() },).await.is_err());
}

#[tokio::test]
async fn pull_dry_run_previews_adoption_without_writes() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    {
        let mut fixtures = current.write().unwrap();
        set_paragraph_text(fixtures.get_mut(TOKEN).unwrap(), &yrs_payload("Dry run remote edit", None));
    }
    let bytes = std::fs::read(&path).unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let report = reader(&server)
        .pull(
            &context(temp.path()),
            Request {
                files: vec![path.clone()],
                dry_run: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(
        (report.updated, report.needs_decision, report.failed),
        (1, 0, 0),
        "{}",
        serde_json::to_string(&report).unwrap()
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert!(!std::fs::read_to_string(&path).unwrap().contains("Dry run remote edit"));
    // The same refresh without --dry-run performs the adoption.
    let report = reader(&server).pull(&context(temp.path()), request(vec![path.clone()])).await.unwrap();
    assert_eq!(report.updated, 1);
    assert!(std::fs::read_to_string(&path).unwrap().contains("Dry run remote edit"));
}
