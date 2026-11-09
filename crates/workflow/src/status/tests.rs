use super::*;
use crate::import::tests::testserver;
use crate::{import, pull::tests::*};
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const TOKEN2: &str = "01999000-0000-7aaa-8000-000000000002";

fn snapshot(path: &Path) -> (Vec<u8>, std::time::SystemTime) {
    (std::fs::read(path).unwrap(), std::fs::metadata(path).unwrap().modified().unwrap())
}

async fn one(root: &Path, path: &Path, remote: Option<&RemoteReader>) -> Item {
    let report = inspect(&context(root), Request { files: vec![path.to_owned()] }, remote).await.unwrap();
    assert_eq!(report.failed, 0, "{}", serde_json::to_string(&report).unwrap());
    report.items.into_iter().next().unwrap()
}

#[tokio::test]
async fn bound_without_receipts_is_reachable_and_never_claimed_clean() {
    let (server, _current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let source = std::fs::read_to_string(&path).unwrap();
    // One span covering the whole sync block, from its key line through the
    // remoteHash line, so both variants share exact boundaries.
    let start = source.find("  \"sync\":").expect("sync block");
    let hash_at = source[start..].find("\"remoteHash\"").expect("remote hash") + start;
    let end = source[hash_at..].find('\n').expect("line end") + hash_at + 1;
    let variants = [
        ("absent", format!("{}{}", &source[..start], &source[end..])),
        ("explicit-null", format!("{}  \"sync\": null\n{}", &source[..start], &source[end..])),
    ];
    for (name, source) in variants {
        assert!(leafbud_mdx::parse(&source).is_ok(), "{name}: the state is valid source");
        std::fs::write(&path, &source).unwrap();
        let reader = reader(&server);
        let calls = server.request_count("/api/rpc/get_leaflet_data");
        let item = one(temp.path(), &path, Some(&reader)).await;
        assert_eq!(item.outcome, "bound-no-receipt", "{name}");
        assert_eq!(item.baseline, "none");
        assert_eq!(item.receipt_profile, None);
        assert_eq!(server.request_count("/api/rpc/get_leaflet_data"), calls, "{name}: no fetch without receipts");
        // Offline status never turns the absence of receipts into clean.
        let item = one(temp.path(), &path, None).await;
        assert_eq!(item.outcome, "bound-no-receipt");
        assert_eq!(item.local, Change::Unknown);
    }
}

#[tokio::test]
async fn duplicate_unbound_identities_stay_explicit() {
    let (server, _current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    // Two unbound drafts sharing one document UUID: binding and receipts
    // disappear together (receipts without a binding are incoherent).
    let source = std::fs::read_to_string(&path).unwrap();
    let start = source.find("  \"binding\":").expect("binding");
    let hash_at = source.find("\"remoteHash\"").expect("remote hash");
    let end = source[hash_at..].find('\n').unwrap() + hash_at + 1;
    let unbound = format!("{}{}", &source[..start], &source[end..]);
    assert!(leafbud_mdx::parse(&unbound).is_ok());
    std::fs::write(&path, &unbound).unwrap();
    let copy = temp.path().join("copy.mdx");
    std::fs::write(&copy, &unbound).unwrap();
    let report = inspect(&context(temp.path()), Request::default(), None).await.unwrap();
    let items: Vec<_> = report.items.iter().map(|i| (i.path.clone(), i.outcome)).collect();
    assert!(
        items.iter().all(|(_, outcome)| *outcome == "ambiguous"),
        "duplicate unbound UUIDs must not silently report unbound: {items:?}"
    );
    assert_eq!(report.failed, 2);
}

#[tokio::test]
async fn status_state_matrix_is_read_only_including_convergence_and_offline() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let reader = reader(&server);
    let baseline = snapshot(&path);
    let item = one(temp.path(), &path, Some(&reader)).await;
    assert_eq!(item.outcome, "clean");
    assert_eq!(item.remote.freshness, "current");
    assert_eq!(item.endpoints_equal, Some(true));
    assert_eq!(snapshot(&path), baseline);
    let calls = server.request_count("/api/rpc/pull");
    let item = one(temp.path(), &path, None).await;
    assert_eq!(
        (item.outcome, item.remote.change, item.remote.freshness),
        ("unknown", Change::Unknown, "offline")
    );
    assert!(item.endpoints_equal.is_none());
    assert!(item.remote.observed_hash.is_none());
    assert_eq!(server.request_count("/api/rpc/pull"), calls);
    let local = String::from_utf8(baseline.0.clone()).unwrap().replace("Synthetic body copy", "Matching change");
    std::fs::write(&path, &local).unwrap();
    let item = one(temp.path(), &path, None).await;
    assert_eq!(
        (item.outcome, item.local, item.remote.change),
        ("local-changed", Change::Changed, Change::Unknown)
    );
    let item = one(temp.path(), &path, Some(&reader)).await;
    assert_eq!((item.outcome, item.remote.change), ("local-changed", Change::Unchanged));
    set_paragraph_text(current.write().unwrap().get_mut(TOKEN).unwrap(), &yrs_payload("Other remote edit", None));
    let item = one(temp.path(), &path, Some(&reader)).await;
    assert_eq!(item.outcome, "both-changed");
    set_paragraph_text(current.write().unwrap().get_mut(TOKEN).unwrap(), &yrs_payload("Matching change", None));
    let before_converged = snapshot(&path);
    let item = one(temp.path(), &path, Some(&reader)).await;
    assert_eq!((item.outcome, item.endpoints_equal), ("converged", Some(true)));
    assert_eq!(snapshot(&path), before_converged, "status never settles convergence");
    assert_eq!(one(temp.path(), &path, Some(&reader)).await.outcome, "converged", "receipts remain stale");
    std::fs::write(&path, &baseline.0).unwrap();
    assert_eq!(one(temp.path(), &path, Some(&reader)).await.outcome, "remote-changed");
}

#[tokio::test]
async fn isolated_remote_body_mark_and_move_changes_and_local_only_intent_have_correct_scopes() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replacen("\n\"@leafbud\":", "\ncustom: {keep: true}\npublishedAt: '2030-01-01'\n\"@leafbud\":", 1);
    std::fs::write(&path, &source).unwrap();
    let baseline = snapshot(&path);
    let reader = reader(&server);
    assert_eq!(one(temp.path(), &path, Some(&reader)).await.outcome, "clean");
    for variant in 0..3 {
        let mut changed = fixture();
        if variant == 0 {
            set_paragraph_text(&mut changed, &yrs_payload("Only body", None));
        }
        if variant == 1 {
            set_paragraph_text(&mut changed, &yrs_payload("Synthetic body copy", Some((0, 9))));
        }
        if variant == 2 {
            swap_block_order(&mut changed);
        }
        *current.write().unwrap().get_mut(TOKEN).unwrap() = changed;
        let item = one(temp.path(), &path, Some(&reader)).await;
        assert_eq!(
            (item.outcome, item.local, item.endpoints_equal),
            ("remote-changed", Change::Unchanged, Some(false))
        );
        assert_eq!(snapshot(&path), baseline);
    }
}

#[tokio::test]
async fn unknown_receipt_profiles_are_not_silently_accepted() {
    let (server, _current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let source = std::fs::read_to_string(&path).unwrap();
    let reader = reader(&server);
    let unknown = source.replace(leafbud_document::EDITOR_PROFILE, "future/99");
    std::fs::write(&path, &unknown).unwrap();
    let calls = server.request_count("/api/rpc/get_leaflet_data");
    let report = inspect(&context(temp.path()), Request { files: vec![path.clone()] }, Some(&reader))
        .await
        .unwrap();
    assert_eq!(report.failed, 1);
    assert_eq!(report.items[0].outcome, "invalid", "unknown profiles fail the typed parse before any fetch");
    assert_eq!(server.request_count("/api/rpc/get_leaflet_data"), calls);
    assert_eq!(snapshot(&path).0, unknown.into_bytes(), "status never rewrites files");
    // Restoring the current profile makes the ordinary outcomes available again.
    std::fs::write(&path, source).unwrap();
    assert_eq!(one(temp.path(), &path, Some(&reader)).await.outcome, "clean");
}

#[tokio::test]
async fn workspace_selection_keeps_malformed_unbound_and_duplicate_identities_actionable() {
    let (server, _current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let duplicate = temp.path().join("duplicate.mdx");
    std::fs::copy(&path, &duplicate).unwrap();
    std::fs::write(temp.path().join("broken.mdx"), "---\n\"@leafbud\": [unclosed\n---\nprivate prose\n").unwrap();
    std::fs::write(
        temp.path().join("bad-token.mdx"),
        "---\n\"@leafbud\": {binding: {baseUrl: 'https://leaflet.pub', token: invalid, publication: null}, sync: {profile: leaflet-editor/3}, state: draft}\n---\n",
    )
    .unwrap();
    std::fs::write(
        temp.path().join("unbound.mdx"),
        "---\nformat: leafbud-mdx/2\ntitle: U\n\"@leafbud\": {id: \"00000000-0000-0000-0000-000000000000\", state: draft}\n---\n\nlocal prose\n",
    )
    .unwrap();
    std::fs::write(temp.path().join("ignored.mdx"), "just prose").unwrap();
    let report = inspect(&context(temp.path()), Request::default(), Some(&reader(&server))).await.unwrap();
    assert_eq!((report.selected, report.failed, report.ignored_unmanaged), (5, 4, 1));
    assert_eq!(report.items.iter().filter(|i| i.outcome == "ambiguous").count(), 2);
    assert_eq!(report.items.iter().filter(|i| i.outcome == "invalid").count(), 2);
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("private prose"));
    assert_eq!(one(temp.path(), &temp.path().join("ignored.mdx"), None).await.outcome, "untracked");
    let report = inspect(
        &context(temp.path()),
        Request {
            files: vec![path, temp.path().join("absent.mdx")],
        },
        None,
    )
    .await
    .unwrap();
    assert_eq!(report.items[1].outcome, "local-missing");
}

#[tokio::test]
async fn duplicate_document_id_is_ambiguous_even_when_native_bindings_differ() {
    let (server, _) = serving(BTreeMap::from([(TOKEN.to_owned(), fixture()), (TOKEN2.to_owned(), fixture_with_token(TOKEN2))])).await;
    let temp = tempfile::tempdir().unwrap();
    let a = seed(&server, temp.path(), TOKEN, "a.mdx").await;
    let b = seed(&server, temp.path(), TOKEN2, "b.mdx").await;
    let source = std::fs::read_to_string(&b).unwrap();
    let a_id = leafbud_mdx::parse(&std::fs::read_to_string(&a).unwrap()).unwrap().state.id;
    let b_id = leafbud_mdx::parse(&source).unwrap().state.id;
    std::fs::write(&b, source.replace(&b_id.to_string(), &a_id.to_string())).unwrap();
    let report = inspect(&context(temp.path()), Request::default(), None).await.unwrap();
    assert_eq!(report.items.iter().filter(|i| i.outcome == "ambiguous").count(), 2);
}

#[tokio::test]
async fn missing_lifecycle_opaque_and_identity_changes_remain_distinct_without_payload_leaks() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let baseline = snapshot(&path);
    let reader = reader(&server);
    current.write().unwrap().remove(TOKEN);
    let item = one(temp.path(), &path, Some(&reader)).await;
    assert_eq!((item.outcome, item.remote.freshness), ("missing", "checked-missing"));
    let mut changed = fixture();
    changed["leaflet_data"]["leaflets_to_documents"] = json!([{"document":"at://did:plc:synthetic/site.standard.document/x"}]);
    current.write().unwrap().insert(TOKEN.into(), changed);
    let item = one(temp.path(), &path, Some(&reader)).await;
    assert_eq!((item.outcome, item.remote.freshness), ("unsupported", "metadata-only"));
    let mut changed = fixture();
    changed["response"]["patch"].as_array_mut().unwrap().push(json!({"op":"put","key":"11111111-1111-4111-8111-111111111111","value":{
        "id":"11111111-1111-4111-8111-111111111111","entity":"root","attribute":"future/opaque","data":{"type":"private","value":"do not print private opaque content"}
    }}));
    current.write().unwrap().insert(TOKEN.into(), changed.clone());
    let item = one(temp.path(), &path, Some(&reader)).await;
    assert_eq!(
        (item.outcome, item.remote.change, item.remote.freshness),
        ("unsupported", Change::Changed, "current")
    );
    let hash = item.remote.observed_hash.clone();
    assert!(!serde_json::to_string(&item).unwrap().contains("do not print"));
    changed["response"]["patch"].as_array_mut().unwrap().last_mut().unwrap()["value"]["data"]["value"] = json!("different opaque content");
    current.write().unwrap().insert(TOKEN.into(), changed);
    assert_ne!(one(temp.path(), &path, Some(&reader)).await.remote.observed_hash, hash);
    let mut changed = fixture();
    changed["leaflet_data"]["permission_token_rights"][0]["entity_set"] = json!("different-rights");
    current.write().unwrap().insert(TOKEN.into(), changed);
    assert_eq!(one(temp.path(), &path, Some(&reader)).await.outcome, "identity-changed");
    assert_eq!(snapshot(&path), baseline);
}

#[tokio::test]
async fn local_race_is_reported_without_a_status_write() {
    let temp = tempfile::tempdir().unwrap();
    let path: Arc<RwLock<Option<PathBuf>>> = Arc::new(RwLock::new(None));
    let target = path.clone();
    let fixture = fixture();
    let server = testserver::TestServer::spawn(Arc::new(move |r| match r.path.as_str() {
        "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()),
        "/api/rpc/pull" => {
            if let Some(path) = target.read().unwrap().as_ref() {
                std::fs::write(path, "user saved during read").unwrap();
            }
            (200, fixture["response"].to_string())
        }
        _ => (404, String::new()),
    }))
    .await;
    let document = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    *path.write().unwrap() = Some(document.clone());
    let report = inspect(&context(temp.path()), Request::default(), Some(&reader(&server))).await.unwrap();
    assert_eq!((report.failed, report.items[0].outcome), (1, "raced"));
    assert_eq!(std::fs::read_to_string(document).unwrap(), "user saved during read");
}

#[tokio::test]
async fn foreign_origins_and_unknown_owned_state_do_not_fetch_or_retarget() {
    let (server, _) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let source = std::fs::read_to_string(&path).unwrap();
    let (other, _) = server_current().await;
    assert_eq!(one(temp.path(), &path, Some(&reader(&other))).await.outcome, "unsupported");
    assert_eq!(other.request_count("/api/rpc/get_leaflet_data"), 0);
    let source = source.replace("\"@leafbud\":\n", "\"@leafbud\":\n  futurePolicy: {secret: 'never print'}\n");
    std::fs::write(&path, &source).unwrap();
    let calls = server.request_count("/api/rpc/get_leaflet_data");
    let report = inspect(&context(temp.path()), Request::default(), Some(&reader(&server))).await.unwrap();
    assert_eq!(report.items[0].outcome, "invalid");
    assert!(!serde_json::to_string(&report).unwrap().contains("never print"));
    assert_eq!(server.request_count("/api/rpc/get_leaflet_data"), calls);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    let original = snapshot(&path);
    let mut header = leafbud_mdx::header(&source).unwrap();
    header["@leafbud"].as_object_mut().unwrap().remove("futurePolicy");
    header["@leafbud"]["binding"].as_object_mut().unwrap().remove("publication");
    let body = source.split_once("---\n\n").unwrap().1;
    let missing_association = format!("---\n{}\n---\n\n{body}", serde_json::to_string(&header).unwrap());
    assert!(
        leafbud_mdx::parse(&missing_association)
            .unwrap_err()
            .to_string()
            .contains("association is unknown")
    );
    assert_eq!(snapshot(&path), original, "validation itself never writes");
}

#[tokio::test]
async fn legacy_project_namespace_is_reported_not_ignored_migrated_or_duplicated() {
    let (server, _) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("@leafbud", "@springleaf")
        .replace("leafbud-mdx/2", "springleaf-mdx/1");
    std::fs::write(&path, &source).unwrap();
    let before = snapshot(&path);
    let calls = server.request_count("/api/rpc/get_leaflet_data");
    for remote in [None, Some(reader(&server))] {
        let report = inspect(&context(temp.path()), Request::default(), remote.as_ref()).await.unwrap();
        assert_eq!((report.selected, report.failed, report.ignored_unmanaged), (1, 0, 0));
        assert_eq!(report.items[0].outcome, "unsupported");
        assert!(report.items[0].detail.contains("explicit namespace conversion"));
    }
    assert_eq!(server.request_count("/api/rpc/get_leaflet_data"), calls);
    assert!(
        reader(&server)
            .import(
                &context(temp.path()),
                import::Request {
                    all: true,
                    ..Default::default()
                }
            )
            .await
            .is_err()
    );
    assert_eq!(
        std::fs::read_dir(temp.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|e| e == "mdx"))
            .count(),
        1
    );
    assert_eq!(snapshot(&path), before);
}

#[tokio::test]
async fn explicit_external_operands_join_identity_ambiguity_without_discovery_or_writes() {
    let (server, _) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let copy = external.path().join("copy.mdx");
    std::fs::copy(&path, &copy).unwrap();
    let report = inspect(&context(temp.path()), Request { files: vec![path, copy] }, None).await.unwrap();
    assert_eq!(report.items.iter().filter(|i| i.outcome == "ambiguous").count(), 2);
    assert_eq!(report.failed, 2);
}
