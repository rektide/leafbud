use super::*;
use springleaf_web::{LeafletClient, parse_pull, parse_rpc};
use std::sync::Arc;

// Shared with the pull tests: one module load of the loopback responder.
#[path = "../../../atproto/src/testserver.rs"]
pub(crate) mod testserver;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../../experiments/02-draft-internals/fixtures/export-pull-doc.json")).unwrap()
}
fn data(fixture: &Value) -> LeafletData {
    parse_rpc(&json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()).unwrap()
}

#[test]
fn fixture_content_to_real_mdx_and_back_without_loss() {
    let fixture = fixture();
    let data = data(&fixture);
    let pull = parse_pull(fixture["response"].clone()).unwrap();
    let document = build("https://leaflet.pub", &data, &pull).unwrap();
    assert_eq!(document.title, "Synthetic title");
    assert_eq!(document.publication, None);
    assert!(document.extra.is_empty());
    let source = springleaf_mdx::render(&document).unwrap();
    assert!(source.contains("Synthetic body copy"));
    assert!(!source.contains("AQPY"));
    let parsed = springleaf_mdx::parse(&source).unwrap();
    assert_eq!(parsed.semantic_value(), document.semantic_value());
    assert_eq!(parsed.state.sync.local_hash, hash(&parsed.semantic_value()));
    assert_eq!(parsed.state.binding.token, data.metadata.id);
    let mut altered = parsed.clone();
    altered
        .state
        .nodes
        .values_mut()
        .next()
        .unwrap()
        .memberships
        .insert("changed".into(), "changed".into());
    assert_ne!(
        hash(&altered.semantic_value()),
        parsed.state.sync.local_hash,
        "source associations are fingerprinted, not only prose"
    );
}

#[test]
fn native_unsupported_and_malformed_content_never_degrades_to_empty() {
    let fixture = fixture();
    let data = data(&fixture);
    let mut pull = parse_pull(fixture["response"].clone()).unwrap();
    let text = pull.facts.iter_mut().find(|f| f["attribute"] == "block/text").unwrap();
    text["data"]["value"] = json!("broken");
    assert!(build("https://leaflet.pub", &data, &pull).is_err());
    let mut pull = parse_pull(fixture["response"].clone()).unwrap();
    pull.facts[0]["attribute"] = json!("future/opaque");
    assert!(build("https://leaflet.pub", &data, &pull).is_err());
    let mut pull = parse_pull(fixture["response"].clone()).unwrap();
    pull.extras.insert("unknown-extra".into(), json!({"state":"opaque"}));
    assert!(build("https://leaflet.pub", &data, &pull).is_err());
}

#[test]
fn restricted_yaml_and_unknown_author_metadata() {
    let fixture = fixture();
    let mut document = build("https://leaflet.pub", &data(&fixture), &parse_pull(fixture["response"].clone()).unwrap()).unwrap();
    document.extra.insert("customAuthorField".into(), json!({"meaning":"retained"}));
    let source = springleaf_mdx::render(&document).unwrap();
    assert_eq!(springleaf_mdx::parse(&source).unwrap().extra, document.extra);
    for invalid in ["title: 'duplicate'\n", "extra: &anchor 'value'\n", "extra: !custom 'value'\n", "<<: {}\n"] {
        let changed = source.replacen("---\n", &format!("---\n{invalid}"), 1);
        assert!(springleaf_mdx::parse(&changed).is_err(), "{invalid}");
    }
}

#[test]
fn visible_code_ignores_but_witnesses_native_inactive_richtext() {
    let fixture = fixture();
    let data = data(&fixture);
    let mut pull = parse_pull(fixture["response"].clone()).unwrap();
    let block = pull
        .facts
        .iter()
        .find(|f| f["attribute"] == "block/type" && f["data"]["value"] == "text")
        .unwrap()["entity"]
        .clone();
    let row = pull.facts.iter_mut().find(|f| f["entity"] == block && f["attribute"] == "block/type").unwrap();
    row["data"]["value"] = json!("code");
    pull.facts.push(
        json!({"id":"11111111-1111-4111-8111-111111111111","entity":block,"attribute":"block/code","data":{"type":"string","value":"visible current code"}}),
    );
    let document = build("https://leaflet.pub", &data, &pull).unwrap();
    let source = springleaf_mdx::render(&document).unwrap();
    assert!(source.contains("visible current code"));
    assert!(!source.contains("Synthetic body copy"));
    assert_eq!(document.state.nodes.values().filter(|n| n.inactive_text_fact.is_some()).count(), 1);
    let previous = document.state.sync.remote_hash;
    pull.facts.iter_mut().find(|f| f["entity"] == block && f["attribute"] == "block/text").unwrap()["data"]["value"] = json!("opaque-inactive-change");
    assert_ne!(build("https://leaflet.pub", &data, &pull).unwrap().state.sync.remote_hash, previous);
}

#[test]
fn rich_runs_empty_blocks_images_and_pages_roundtrip() {
    use springleaf_document::*;
    let fixture = fixture();
    let mut document = build("https://leaflet.pub", &data(&fixture), &parse_pull(fixture["response"].clone()).unwrap()).unwrap();
    let inline = vec![
        Inline::Run {
            text: " literal <jsx> {expression} \\ spaces\nline ".into(),
            marks: vec![
                Mark::Strong,
                Mark::Link {
                    href: "https://example.invalid/a?x=1&y=2".into(),
                },
            ],
        },
        Inline::HardBreak,
        Inline::Run {
            text: "unicode 🦀 & quotes\"".into(),
            marks: vec![],
        },
    ];
    let kinds = vec![
        BlockKind::Paragraph { content: inline.clone() },
        BlockKind::Heading {
            level: 2,
            content: inline.clone(),
        },
        BlockKind::Quote { content: inline },
        BlockKind::Paragraph { content: vec![] },
        BlockKind::Heading { level: 1, content: vec![] },
        BlockKind::Quote { content: vec![] },
        BlockKind::Code {
            language: "rust".into(),
            text: "fn main() {\n}\n\n".into(),
        },
        BlockKind::Image {
            src: "https://example.invalid/i.png".into(),
            fallback: Some("data:image/png;base64,synthetic".into()),
            width: Some(200),
            height: Some(100),
            alt: Some("a \"quote\"".into()),
        },
        BlockKind::PageLink { page: "p-other".into() },
        BlockKind::Rule,
    ];
    document.pages[0].blocks.clear();
    // Rebuild exact source-node identities for this synthetic writer control.
    document.state.nodes.clear();
    let page_key = document.pages[0].key.clone();
    document.state.nodes.insert(
        page_key,
        Node {
            entity: "main".into(),
            memberships: BTreeMap::new(),
            text_fact: None,
            inactive_text_fact: None,
        },
    );
    for (i, kind) in kinds.into_iter().enumerate() {
        let key = format!("b-{i}");
        document.state.nodes.insert(
            key.clone(),
            Node {
                entity: format!("e-{i}"),
                memberships: BTreeMap::new(),
                text_fact: None,
                inactive_text_fact: None,
            },
        );
        document.pages[0].blocks.push(Block { key, kind });
    }
    document.state.nodes.insert(
        "p-other".into(),
        Node {
            entity: "other".into(),
            memberships: BTreeMap::new(),
            text_fact: None,
            inactive_text_fact: None,
        },
    );
    document.pages.push(Page {
        key: "p-other".into(),
        blocks: vec![],
    });
    let source = springleaf_mdx::render(&document).unwrap();
    assert_eq!(springleaf_mdx::parse(&source).unwrap().semantic_value(), document.semantic_value());
}

async fn server() -> testserver::TestServer {
    let fixture = fixture();
    testserver::TestServer::spawn(Arc::new(move |r| match r.path.as_str() {
        "/home" => {
            let identity = json!({"permission_token_on_homepage":[{"permission_tokens":fixture["leaflet_data"]}],"contributor_leaflets":[]});
            (200, format!("<script>self.__next_f.push({})</script>", json!([1, format!("0:{identity}")])))
        }
        "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()),
        "/api/rpc/pull" => {
            assert_eq!(r.body_json()["pullRequest"]["cookie"], Value::Null);
            assert_eq!(r.method, "POST");
            assert!(r.query.is_empty());
            assert_eq!(r.header("cookie"), Some("auth_token=synthetic"));
            (200, fixture["response"].to_string())
        }
        _ => (404, "unexpected".into()),
    }))
    .await
}

#[tokio::test]
async fn actual_import_noop_rename_and_local_changes_preserved() {
    let server = server().await;
    let temp = tempfile::tempdir().unwrap();
    let global = temp.path().join("global.toml");
    std::fs::write(&global, "publication = 'never-inherit-for-bound-import'\n").unwrap();
    let context = config::load_at(&global, temp.path(), Some(temp.path()), &config::Config::default(), None).unwrap();
    let reader = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
    let request = || Request {
        all: true,
        state: Some(LeafletState::Draft),
        ..Default::default()
    };
    let report = reader.import(&context, request()).await.unwrap();
    assert_eq!((report.imported, report.failed), (1, 0), "{}", serde_json::to_string(&report).unwrap());
    let path = report.items[0].path.as_ref().unwrap();
    let bytes = std::fs::read(path).unwrap();
    let modified = std::fs::metadata(path).unwrap().modified().unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("never-inherit-for-bound-import"));
    let report = reader.import(&context, request()).await.unwrap();
    assert_eq!((report.imported, report.unchanged, report.failed), (0, 1, 0));
    assert_eq!(std::fs::metadata(path).unwrap().modified().unwrap(), modified);
    let renamed = temp.path().join("renamed.mdx");
    std::fs::rename(path, &renamed).unwrap();
    let edited = String::from_utf8(bytes).unwrap().replace("Synthetic body copy", "Edited body");
    std::fs::write(&renamed, &edited).unwrap();
    let report = reader.import(&context, request()).await.unwrap();
    assert_eq!(report.items[0].outcome, "kept-local-changes");
    assert_eq!(report.items[0].path.as_ref(), Some(&renamed));
    assert_eq!(std::fs::read_to_string(&renamed).unwrap(), edited);
    assert_eq!(server.request_count("/api/rpc/pull"), 3);
}

#[tokio::test]
async fn single_destination_collision_and_unsafe_paths_are_refused() {
    let server = server().await;
    let temp = tempfile::tempdir().unwrap();
    let context = config::load_at(
        &temp.path().join("absent.toml"),
        temp.path(),
        Some(temp.path()),
        &config::Config::default(),
        None,
    )
    .unwrap();
    let reader = RemoteReader::new(LeafletClient::new(&server.base_url, Some("synthetic")).unwrap());
    let id = data(&fixture()).metadata.id;
    let request = |to: &str| Request {
        locator: Some(id.to_string()),
        to: Some(to.into()),
        ..Default::default()
    };
    std::fs::write(temp.path().join("existing.mdx"), "unmanaged local text").unwrap();
    let report = reader.import(&context, request("existing.mdx")).await.unwrap();
    assert_eq!(report.failed, 1);
    assert_eq!(std::fs::read_to_string(temp.path().join("existing.mdx")).unwrap(), "unmanaged local text");
    let report = reader.import(&context, request("../escape.mdx")).await.unwrap();
    assert_eq!(report.failed, 1);
    let report = reader.import(&context, request("chosen.mdx")).await.unwrap();
    assert_eq!(report.imported, 1);
    assert!(temp.path().join("chosen.mdx").exists());
}

#[tokio::test]
async fn remote_drift_does_not_replace_existing_local_source() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let changed = Arc::new(AtomicBool::new(false));
    let flag = changed.clone();
    let fixture = fixture();
    let id = data(&fixture).metadata.id;
    let server = testserver::TestServer::spawn(Arc::new(move |r| {
        let mut fixture = fixture.clone();
        if flag.load(Ordering::Relaxed) {
            fixture["leaflet_data"]["title"] = json!("Changed remote title");
        }
        match r.path.as_str() {
            "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()),
            "/api/rpc/pull" => (200, fixture["response"].to_string()),
            _ => (404, String::new()),
        }
    }))
    .await;
    let temp = tempfile::tempdir().unwrap();
    let context = config::load_at(
        &temp.path().join("absent.toml"),
        temp.path(),
        Some(temp.path()),
        &config::Config::default(),
        None,
    )
    .unwrap();
    let reader = RemoteReader::new(LeafletClient::new(&server.base_url, None).unwrap());
    let request = || Request {
        locator: Some(id.to_string()),
        ..Default::default()
    };
    let first = reader.import(&context, request()).await.unwrap();
    assert_eq!(first.imported, 1);
    let path = first.items[0].path.as_ref().unwrap();
    let bytes = std::fs::read(path).unwrap();
    changed.store(true, Ordering::Relaxed);
    let second = reader.import(&context, request()).await.unwrap();
    assert_eq!((second.failed, second.imported), (1, 0));
    assert!(second.items[0].error.as_ref().unwrap().contains("remote changed"));
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
