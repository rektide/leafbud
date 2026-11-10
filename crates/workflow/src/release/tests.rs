use super::*;
use crate::{import::tests::testserver::TestServer, pull::tests as support};
use leafbud_document::{RELEASE_PROFILE, Released};
use leafbud_web::parse_pull;
use serde_json::{Value, json};
use std::sync::{Arc, RwLock};

// Independent CID syntax vectors (empty bytes / hello world), used solely as
// PDS-reported opaque revisions. These fixtures intentionally do not claim a
// record rehash; Evidence names that confidence boundary explicitly.
const CID: &str = "bafyreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku";
const OTHER_CID: &str = "bafyreifzjut3te2nhyekklss27nh3k72ysco7y32koao5eei66wof36n5e";

fn data() -> (Document, Value) {
    let fixture = crate::published::tests::fixture(false);
    let original = crate::import::build(
        "https://leaflet.pub",
        &support::data(&fixture),
        &parse_pull(fixture["response"].clone()).unwrap(),
    )
    .unwrap();
    let record = fixture["leaflet_data"]["leaflets_to_documents"][0]["documents"]["data"].clone();
    let document = projection::from_record(&record, &original).unwrap().document;
    (document, record)
}
fn confirmed(document: &mut Document) {
    document.state.released = Some(Released {
        profile: RELEASE_PROFILE.into(),
        input_hash: import::hash(&projection::working(document).unwrap().value),
        cid: CID.into(),
        path: "/post".into(),
        published_at: "2026-10-01T10:00:00Z".into(),
        confirmed_at: "2026-10-01T11:00:00Z".into(),
    });
}
async fn serving(record: Value) -> (TestServer, Arc<RwLock<(u16, Value)>>, PublicReader) {
    let current = Arc::new(RwLock::new((
        200,
        json!({"uri":"at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/post","cid":CID,"value":record}),
    )));
    let handle = current.clone();
    let server = TestServer::spawn(Arc::new(move |request| {
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/xrpc/com.atproto.repo.getRecord");
        assert!(request.query.contains("repo=did%3Aplc%3Aabcdefghijklmnopqrstuvwx"));
        assert!(request.header("cookie").is_none() && request.header("authorization").is_none());
        let current = handle.read().unwrap();
        (current.0, current.1.to_string())
    }))
    .await;
    let reader = PublicReader::with_endpoint(XrpcClient::new(&server.base_url).unwrap());
    (server, current, reader)
}

#[tokio::test]
async fn independent_release_matrix_unknown_baseline_input_output_both_convergence_and_offline() {
    let (mut document, record) = data();
    let (_server, current, reader) = serving(record.clone()).await;
    let evidence = observe(&document, Some(&reader), false).await;
    assert_eq!(
        (evidence.state, evidence.baseline, evidence.endpoints_equal),
        ("equal-untracked", "unknown-imported", Some(true))
    );
    assert!(document.state.released.is_none(), "public CID alone must not create a release receipt");
    confirmed(&mut document);
    let original = document.state.clone();
    let evidence = observe(&document, Some(&reader), false).await;
    assert_eq!(evidence.state, "clean");
    document.title = "Local next".into();
    let evidence = observe(&document, Some(&reader), false).await;
    assert_eq!(
        (evidence.state, evidence.input, evidence.output),
        ("input-changed", Change::Changed, Change::Unchanged)
    );
    current.write().unwrap().1["value"]["title"] = json!("External next");
    current.write().unwrap().1["cid"] = json!(OTHER_CID);
    assert_eq!(observe(&document, Some(&reader), false).await.state, "both-changed");
    document.title = "Public title".into();
    assert_eq!(observe(&document, Some(&reader), false).await.state, "output-changed");
    document.title = "External next".into();
    assert_eq!(observe(&document, Some(&reader), false).await.state, "converged-unaccepted");
    let offline = observe(&document, None, true).await;
    assert_eq!(
        (offline.state, offline.freshness, offline.output),
        ("input-changed-output-unknown", "offline", Change::Unknown)
    );
    assert_eq!(document.state, original, "status never acknowledges either endpoint");
    let source = leafbud_mdx::render(&document).unwrap();
    let mut changed = document.state.clone();
    changed.released.as_mut().unwrap().cid = OTHER_CID.into();
    assert!(leafbud_mdx::patch_tracking(&source, &changed).is_err());
    let mut changed = document.clone();
    changed.state.released.as_mut().unwrap().profile = "future/2".into();
    assert!(leafbud_mdx::render(&changed).is_err());
}

#[tokio::test]
async fn missing_unavailable_mismatched_unsupported_and_not_requested_evidence_stay_distinct() {
    let (mut document, record) = data();
    confirmed(&mut document);
    let (_server, current, reader) = serving(record).await;
    assert_eq!(observe(&document, None, false).await.freshness, "not-requested");
    current.write().unwrap().0 = 404;
    current.write().unwrap().1 = json!({"error":"RecordNotFound","message":"fixture body withheld"});
    let missing = observe(&document, Some(&reader), false).await;
    assert_eq!(
        (missing.state, missing.freshness, missing.output),
        ("record-missing", "checked-missing", Change::Changed)
    );
    current.write().unwrap().1 = json!({"error":"AuthRequired","message":"not deletion"});
    assert_eq!(observe(&document, Some(&reader), false).await.state, "unavailable");
    let (_, record) = data();
    *current.write().unwrap() = (
        200,
        json!({"uri":"at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/wrong","cid":CID,"value":record}),
    );
    assert_eq!(observe(&document, Some(&reader), false).await.state, "identity-or-CID-mismatch");
    current.write().unwrap().1["uri"] = json!("at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/post");
    current.write().unwrap().1["value"]["site"] = json!("https://wrong.example");
    assert_eq!(observe(&document, Some(&reader), false).await.state, "linkage-mismatch");
    current.write().unwrap().1["value"]["site"] = json!("https://leaflet.pub/p/did:plc:abcdefghijklmnopqrstuvwx");
    current.write().unwrap().1["value"]["futurePublicField"] = json!("must not disappear");
    current.write().unwrap().1["cid"] = json!(OTHER_CID);
    let unsupported = observe(&document, Some(&reader), false).await;
    assert_eq!(
        (unsupported.state, unsupported.output_projection, unsupported.output),
        ("unsupported-projection", "unsupported", Change::Changed)
    );
    assert_eq!(
        unsupported.observed_cid.as_deref(),
        Some(OTHER_CID),
        "even unsupported public meaning retains independently observed output evidence"
    );
}

#[tokio::test]
async fn public_only_status_and_diff_use_fetched_current_markdown_without_native_authority_or_receipt_writes() {
    let (mut document, record) = data();
    confirmed(&mut document);
    let (server, current, reader) = serving(record).await;
    let temp = tempfile::tempdir().unwrap();
    let context = support::context(temp.path());
    let path = temp.path().join("published.mdx");
    std::fs::write(&path, leafbud_mdx::render(&document).unwrap()).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    let status = crate::status::inspect_with_public(&context, crate::status::Request { files: vec![path.clone()] }, None, Some(&reader), false)
        .await
        .unwrap();
    assert_eq!(status.items[0].release.state, "clean");
    assert_eq!(status.items[0].remote.freshness, "not-requested");
    assert!(!status.offline);
    current.write().unwrap().1["value"]["title"] = json!("Fresh external public title");
    current.write().unwrap().1["cid"] = json!(OTHER_CID);
    let tool = crate::diff::Tool {
        program: "diff".into(),
        args: vec!["-u".into()],
        exit: crate::diff::ExitMode::Diff,
    };
    let report = reader.diff_release(&context, &path, &tool).await.unwrap();
    assert!(!report.views_equal);
    assert_eq!(report.tool_exit, 1);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    assert!(server.requests().iter().all(|request| request.path == "/xrpc/com.atproto.repo.getRecord"));
}
