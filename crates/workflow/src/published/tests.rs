use super::*;
use crate::{RemoteReader, import, pull::tests as support};
use leafbud_web::{LeafletClient, parse_pull};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const OWNER: &str = "did:plc:abcdefghijklmnopqrstuvwx";
const URI: &str = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/post";
const PUBLICATION: &str = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.publication/garden";

pub(crate) fn fixture(publication: bool) -> Value {
    let mut fixture = support::fixture();
    let record = json!({"$type":"site.standard.document","title":"Public title","description":"Public description","tags":["released"],
        "site":if publication {PUBLICATION.to_owned()} else {format!("https://leaflet.pub/p/{OWNER}")},
        "path":"/post","publishedAt":"2026-10-01T10:00:00.000Z",
        "content":{"$type":"pub.leaflet.content","pages":[{"$type":"pub.leaflet.pages.linearDocument","id":"public-page",
            "blocks":[{"$type":"pub.leaflet.pages.linearDocument#block","block":{"$type":"pub.leaflet.blocks.text","plaintext":"Already released prose"}}]}]}});
    let mut link =
        json!({"leaflet":TOKEN,"title":"Working title","description":"Working description","tags":["unreleased"],"documents":{"uri":URI,"data":record}});
    if publication {
        link["doc"] = json!(URI);
        link["publication"] = json!(PUBLICATION);
        link["publications"] = json!({"uri":PUBLICATION,"identity_did":OWNER});
        fixture["leaflet_data"]["leaflets_in_publications"] = json!([link]);
    } else {
        link["document"] = json!(URI);
        fixture["leaflet_data"]["leaflets_to_documents"] = json!([link]);
    }
    // The SQL pull_data_cvr/buildExtras projection reads the linkage rows above,
    // not documents.data or loose permission-token metadata.
    fixture["response"]["patch"].as_array_mut().unwrap().extend([
        json!({"op":"put","key":"publication_title","value":"Working title"}),
        json!({"op":"put","key":"publication_description","value":"Working description"}),
        json!({"op":"put","key":"publication_tags","value":["unreleased"]}),
    ]);
    support::set_paragraph_text(&mut fixture, &support::yrs_payload("Current unreleased editor prose", None));
    fixture
}

fn link(fixture: &mut Value, publication: bool) -> &mut Value {
    &mut fixture["leaflet_data"][if publication { "leaflets_in_publications" } else { "leaflets_to_documents" }][0]
}

#[test]
fn published_build_uses_current_working_metadata_and_preserves_exact_public_identity() {
    for publication in [false, true] {
        let fixture = fixture(publication);
        let doc = import::build(
            "https://leaflet.pub",
            &support::data(&fixture),
            &parse_pull(fixture["response"].clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(doc.state.state, "published");
        assert_eq!(doc.state.binding.as_ref().unwrap().uri.as_deref(), Some(URI));
        assert_eq!(doc.title, "Working title");
        assert_eq!(doc.tags, ["unreleased"]);
        assert_eq!(doc.description.as_deref(), Some("Working description"));
        assert_eq!(doc.extra["publishedAt"], "2026-10-01T10:00:00.000Z");
        let source = leafbud_mdx::render(&doc).unwrap();
        assert!(source.contains("Current unreleased editor prose"));
        assert!(!source.contains("Already released prose"));
        assert!(!source.contains("Public title"));
        assert_eq!(leafbud_mdx::parse(&source).unwrap(), doc);
        let mut wrong = doc.clone();
        wrong.state.state = "draft".into();
        assert!(leafbud_mdx::render(&wrong).is_err());
        wrong = doc.clone();
        wrong.state.binding.as_mut().unwrap().uri = None;
        assert!(leafbud_mdx::render(&wrong).is_err());
        for invalid in [
            "at://example.com/site.standard.document/post",
            "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document",
            "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/post#part",
            "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.publication/post",
        ] {
            wrong = doc.clone();
            wrong.state.binding.as_mut().unwrap().uri = Some(invalid.into());
            assert!(leafbud_mdx::render(&wrong).is_err(), "{invalid}");
        }
    }
}

#[test]
fn legacy_public_identity_still_adopts_current_native_content_without_claiming_release_projection() {
    for publication in [false, true] {
        let mut fixture = fixture(publication);
        let link = link(&mut fixture, publication);
        let uri = URI.replace("site.standard.document", "pub.leaflet.document");
        link[if publication { "doc" } else { "document" }] = json!(uri);
        link["documents"]["uri"] = json!(uri);
        let record = link["documents"]["data"].as_object_mut().unwrap();
        record.insert("$type".into(), json!("pub.leaflet.document"));
        record.insert("author".into(), json!(OWNER));
        record.remove("site");
        record.remove("path");
        let pages = record.remove("content").unwrap()["pages"].clone();
        record.insert("pages".into(), pages);
        if publication {
            record.insert("publication".into(), json!(PUBLICATION));
        }
        let document = import::build(
            "https://leaflet.pub",
            &support::data(&fixture),
            &parse_pull(fixture["response"].clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(document.state.binding.as_ref().unwrap().uri.as_deref(), Some(uri.as_str()));
        let source = leafbud_mdx::render(&document).unwrap();
        assert!(source.contains("Current unreleased editor prose"));
        assert!(leafbud_records::projection::working(&document).is_err());
    }
}

#[test]
fn published_linkage_missing_ambiguous_wrong_owner_and_working_metadata_refuse() {
    for publication in [false, true] {
        for case in 0..7 {
            let mut fixture = fixture(publication);
            match case {
                0 => link(&mut fixture, publication)["documents"]["uri"] = json!("at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/other"),
                1 => link(&mut fixture, publication)["documents"]["data"]["site"] = json!("https://wrong.example"),
                2 if publication => link(&mut fixture, true)["publications"]["identity_did"] = json!("did:plc:wrong"),
                2 => link(&mut fixture, false)["documents"] = Value::Null,
                3 => fixture["response"]["patch"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|op| op["key"] != "publication_title"),
                4 => fixture["leaflet_data"]["permission_token_rights"][0]["read"] = json!(false),
                5 => {
                    let mut other = link(&mut fixture, publication).clone();
                    other[if publication { "doc" } else { "document" }] = json!("at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/other");
                    other["documents"]["uri"] = other[if publication { "doc" } else { "document" }].clone();
                    fixture["leaflet_data"][if publication { "leaflets_in_publications" } else { "leaflets_to_documents" }]
                        .as_array_mut()
                        .unwrap()
                        .push(other);
                }
                _ => {
                    let mut sibling = fixture["leaflet_data"].clone();
                    sibling["root_entity"] = json!("different-root");
                    fixture["leaflet_data"]["leaflets_in_publications"] = json!([]);
                    fixture["leaflet_data"]["leaflets_to_documents"] = json!([]);
                    fixture["leaflet_data"]["permission_token_rights"][0]["entity_sets"] = json!({"permission_tokens":[sibling]});
                }
            }
            assert!(
                import::build(
                    "https://leaflet.pub",
                    &support::data(&fixture),
                    &parse_pull(fixture["response"].clone()).unwrap()
                )
                .is_err(),
                "publication={publication}, case={case}"
            );
        }
    }
}

#[tokio::test]
async fn published_import_refresh_retains_uuid_keys_names_comments_and_local_date_without_public_body() {
    for publication in [false, true] {
        let (server, current) = support::serving(BTreeMap::from([(TOKEN.into(), fixture(publication))])).await;
        let temp = tempfile::tempdir().unwrap();
        let path = support::seed(&server, temp.path(), TOKEN, "drafts/published.mdx").await;
        let context = support::context(temp.path());
        let reader = support::reader(&server);
        let source = std::fs::read_to_string(&path).unwrap();
        let doc = leafbud_mdx::parse(&source).unwrap();
        let source = source.replacen("<Key", "{/* retain this note */}\n\n<Key", 1);
        let source = leafbud_mdx::patch_names(
            &source,
            None,
            &leafbud_document::Names {
                file: "drafts/published.mdx".into(),
                path: leafbud_document::NamePath::Value("/post".into()),
            },
        )
        .unwrap();
        let source = source.replace("2026-10-01T10:00:00.000Z", "2026-09-01T10:00:00.000Z");
        std::fs::write(&path, &source).unwrap();
        let before = leafbud_mdx::parse(&source).unwrap();
        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(
            reader.refresh_document(&context, &path, Default::default()).await.outcome,
            crate::refresh::Outcome::Unchanged
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
        support::set_paragraph_text(
            current.write().unwrap().get_mut(TOKEN).unwrap(),
            &support::yrs_payload("New native unreleased text", None),
        );
        let result = reader.refresh_document(&context, &path, Default::default()).await;
        assert_eq!(result.outcome, crate::refresh::Outcome::Updated, "{:?}", result.message);
        let after_source = std::fs::read_to_string(&path).unwrap();
        let after = leafbud_mdx::parse(&after_source).unwrap();
        assert!(after_source.contains("retain this note") && after_source.contains("New native unreleased text"));
        assert!(!after_source.contains("Already released prose"));
        assert_eq!(after.state.binding, doc.state.binding);
        assert_eq!(after.state.id, doc.state.id);
        assert_eq!(after.state.nodes, doc.state.nodes);
        assert_eq!(after.state.names, before.state.names);
        assert_eq!(after.extra["publishedAt"], "2026-09-01T10:00:00.000Z");
        assert_eq!(after.state.state, "published");
        let status = crate::status::inspect(&context, crate::status::Request { files: vec![path.clone()] }, Some(&reader))
            .await
            .unwrap();
        assert_eq!(status.items[0].outcome, "clean");
        let settled = std::fs::read(&path).unwrap();
        {
            let mut fixtures = current.write().unwrap();
            let link = link(fixtures.get_mut(TOKEN).unwrap(), publication);
            link["documents"]["data"]["title"] = json!("Changed released title only");
            link["documents"]["data"]["publishedAt"] = json!("2026-10-02T10:00:00Z");
        }
        let status = crate::status::inspect(&context, crate::status::Request { files: vec![path.clone()] }, Some(&reader))
            .await
            .unwrap();
        assert_eq!(
            status.items[0].outcome, "clean",
            "public-only appview changes do not contaminate the editor witness"
        );
        assert_eq!(
            reader.refresh_document(&context, &path, Default::default()).await.outcome,
            crate::refresh::Outcome::Unchanged
        );
        assert_eq!(std::fs::read(&path).unwrap(), settled);
        // A public-URI retarget is never silently adopted, even under take-remote.
        let saved = std::fs::read(&path).unwrap();
        {
            let mut fixtures = current.write().unwrap();
            let fixture = fixtures.get_mut(TOKEN).unwrap();
            let link = link(fixture, publication);
            let changed = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/changed";
            link[if publication { "doc" } else { "document" }] = json!(changed);
            link["documents"]["uri"] = json!(changed);
        }
        let result = reader
            .refresh_document(
                &context,
                &path,
                crate::refresh::Policy {
                    take_remote: true,
                    ..Default::default()
                },
            )
            .await;
        assert_eq!(result.outcome, crate::refresh::Outcome::NeedsDecision);
        assert_eq!(std::fs::read(&path).unwrap(), saved);
    }
}

#[tokio::test]
async fn exact_uri_discovery_import_all_and_duplicate_public_binding_are_guarded() {
    let current = Arc::new(RwLock::new(fixture(true)));
    let handle = current.clone();
    let server = crate::import::tests::testserver::TestServer::spawn(Arc::new(move |request| {
        let fixture = handle.read().unwrap();
        match request.path.as_str() {
            "/home" => {
                let identity = json!({"permission_token_on_homepage":[{"permission_tokens":fixture["leaflet_data"]}],"contributor_leaflets":[]});
                (200, format!("<script>self.__next_f.push({})</script>", json!([1, format!("0:{identity}")])))
            }
            "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()),
            "/api/rpc/pull" => (200, fixture["response"].to_string()),
            _ => panic!("unexpected effect/read {}", request.path),
        }
    }))
    .await;
    let temp = tempfile::tempdir().unwrap();
    let context = support::context(temp.path());
    let reader = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
    let report = reader
        .import(
            &context,
            import::Request {
                all: true,
                state: Some(leafbud_web::LeafletState::Published),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!((report.imported, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    let path = report.items[0].path.as_ref().unwrap();
    let original = std::fs::read(path).unwrap();
    let report = reader
        .import(
            &context,
            import::Request {
                locator: Some(URI.into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!((report.unchanged, report.failed), (1, 0));
    assert_eq!(std::fs::read(path).unwrap(), original);
    assert!(
        reader
            .import(
                &context,
                import::Request {
                    locator: Some(URI.replace("/post", "/missing")),
                    ..Default::default()
                }
            )
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("editing-unavailable")
    );
    // Even a different origin/token cannot cause a second source for this public URI.
    let mut copy = leafbud_mdx::parse(std::str::from_utf8(&original).unwrap()).unwrap();
    copy.state.id = uuid::Uuid::new_v4();
    copy.state.binding.as_mut().unwrap().token = uuid::Uuid::new_v4();
    copy.state.binding.as_mut().unwrap().base_url = "https://another-origin.example".into();
    std::fs::write(temp.path().join("copy.mdx"), leafbud_mdx::render(&copy).unwrap()).unwrap();
    let report = reader
        .import(
            &context,
            import::Request {
                locator: Some(URI.into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.failed, 1);
    assert_eq!(std::fs::read(path).unwrap(), original);
    let listed = crate::list::inspect(
        &context,
        crate::list::Request {
            state: Some(leafbud_web::LeafletState::Published),
        },
    )
    .unwrap();
    assert_eq!(listed.items.len(), 2);
    assert!(
        listed
            .items
            .iter()
            .all(|item| item.valid && item.duplicate_binding && item.outcome == "ambiguous"),
        "list supports published sources and independently detects URI copies across origins"
    );
}

#[tokio::test]
async fn observed_native_publication_transition_requires_explicit_guarded_adoption() {
    let (server, current) = support::serving(BTreeMap::from([(TOKEN.into(), support::fixture())])).await;
    let temp = tempfile::tempdir().unwrap();
    let context = support::context(temp.path());
    let path = support::seed(&server, temp.path(), TOKEN, "draft.mdx").await;
    let original = std::fs::read_to_string(&path).unwrap();
    let old = leafbud_mdx::parse(&original).unwrap();
    current.write().unwrap().insert(TOKEN.into(), fixture(false));
    let reader = support::reader(&server);
    let result = reader.refresh_document(&context, &path, Default::default()).await;
    assert_eq!(result.outcome, crate::refresh::Outcome::NeedsDecision);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    let preview = reader
        .refresh_document(
            &context,
            &path,
            crate::refresh::Policy {
                adopt_published: true,
                dry_run: true,
                ..Default::default()
            },
        )
        .await;
    assert_eq!(preview.outcome, crate::refresh::Outcome::Updated, "{:?}", preview.message);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    let result = reader
        .refresh_document(
            &context,
            &path,
            crate::refresh::Policy {
                adopt_published: true,
                ..Default::default()
            },
        )
        .await;
    assert_eq!(result.outcome, crate::refresh::Outcome::Updated, "{:?}", result.message);
    let after = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(after.state.state, "published");
    assert_eq!(after.state.id, old.state.id);
    assert_eq!(after.state.nodes, old.state.nodes);
    assert_eq!(after.state.binding.as_ref().unwrap().uri.as_deref(), Some(URI));
    let mut expected = old.state.binding.clone().unwrap();
    expected.uri = Some(URI.into());
    assert_eq!(after.state.binding, Some(expected));
    assert!(
        after.state.released.is_none(),
        "explicit lifecycle adoption must not invent release-input correspondence"
    );
    assert_eq!(after.extra["publishedAt"], "2026-10-01T10:00:00.000Z");
    assert!(
        leafbud_mdx::adopt_remote(&original, &after).is_err(),
        "ordinary adoption has no lifecycle authority"
    );
    assert!(
        leafbud_mdx::patch_tracking(&original, &after.state).is_err(),
        "tracking has no lifecycle authority"
    );
    // The old draft has no URI to index yet; a different alias already owning
    // the prospective public identity must block even a dry-run adoption.
    let mut alias = after.clone();
    alias.state.id = uuid::Uuid::new_v4();
    alias.state.binding.as_mut().unwrap().token = uuid::Uuid::new_v4();
    std::fs::write(temp.path().join("alias.mdx"), leafbud_mdx::render(&alias).unwrap()).unwrap();
    std::fs::write(&path, &original).unwrap();
    let result = reader
        .refresh_document(
            &context,
            &path,
            crate::refresh::Policy {
                adopt_published: true,
                dry_run: true,
                ..Default::default()
            },
        )
        .await;
    assert_eq!(result.outcome, crate::refresh::Outcome::Ambiguous);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
}
