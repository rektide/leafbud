//! Executed CLI creation/preview/full first/revised/repeat saves via real HTTP.
#[path = "../../leaflet/protocol/tests/support/store.rs"]
mod store;
#[path = "../../atproto/src/testserver.rs"]
#[allow(dead_code)]
mod testserver;
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::process::Command;

fn cli(base: &str, root: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_leafbud"));
    c.env_remove("HOME")
        .env_remove("LEAFLET_AUTH_TOKEN")
        .env_remove("LEAFLET_BASE_URL")
        .env("XDG_CONFIG_HOME", root.join(".config"))
        .args(["--base-url", base, "--root", root.to_str().unwrap(), "--token", "synthetic"])
        .args(args);
    c
}
async fn invoke(base: &str, root: &Path, args: &[&str], code: i32) -> Value {
    let out = cli(base, root, args).output().await.unwrap();
    assert_eq!(
        out.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
async fn server(publication: bool) -> (testserver::TestServer, Arc<Mutex<store::Store>>) {
    let state = Arc::new(Mutex::new(store::Store::new(publication)));
    let remote = state.clone();
    let http = testserver::TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
        "/home" => (
            200,
            vec![],
            format!(
                "<script>self.__next_f.push({})</script>",
                json!([
                    1,
                    format!(
                        "0:{}",
                        json!({"atp_did":"did:plc:synthetic","bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]})
                    )
                ])
            ),
        ),
        "/lish/did:plc:synthetic/garden/dashboard" if r.method == "GET" => (200, vec![], "<script src=\"/_next/static/mint.js\"></script>".into()),
        "/_next/static/mint.js" => (200, vec![], include_str!("../../leaflet/web/tests/fixtures/mint-reference.js").into()),
        "/new" => (307, vec![("Location".into(), format!("/{}", store::TOKEN))], String::new()),
        "/lish/did:plc:synthetic/garden/dashboard" => (
            200,
            vec![("Content-Type".into(), "text/x-component".into())],
            include_str!("../../leaflet/web/tests/fixtures/mint-result.flight").into(),
        ),
        _ => (200, vec![], remote.lock().unwrap().rpc(&r.path, &r.body_json()).unwrap().to_string()),
    }))
    .await;
    (http, state)
}

#[tokio::test]
async fn complete_cli_both_contexts_preview_first_combined_revision_pull_and_unchanged_all() {
    for publication in [false, true] {
        let (server, store) = server(publication).await;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("doc.mdx");
        let args = if publication {
            vec!["--publication", store::URI, "create", "doc.mdx", "--title", "Independent title"]
        } else {
            vec!["create", "doc.mdx", "--title", "Independent title", "--standalone"]
        };
        let out = cli(&server.base_url, root.path(), &args).output().await.unwrap();
        assert!(out.status.success());
        let text = format!("{}\nAlpha 😀.\n\nBravo and delta.\n\nCharlie.\n", std::fs::read_to_string(&path).unwrap());
        std::fs::write(&path, &text).unwrap();
        let id = leafbud_mdx::parse(&text).unwrap().state.id;
        let report = invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--json"], 0).await;
        assert_eq!(report["items"][0]["outcome"], "would-bind-and-save");
        assert!(server.requests().is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        let report = invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await;
        assert_eq!(report["items"][0]["outcome"], "saved");
        let first_text = std::fs::read_to_string(&path).unwrap();
        let first = leafbud_mdx::parse(&first_text).unwrap();
        assert_eq!(first.state.id, id);
        assert_eq!(first.pages[0].blocks.len(), 3);
        let a = &first.pages[0].blocks[0].key;
        let b = &first.pages[0].blocks[1].key;
        let c = &first.pages[0].blocks[2].key;
        let header = first_text.split_once("\n---\n").unwrap().0;
        let revision = format!(
            "{header}\n---\n\n# Inserted heading, not a title\n\n<Key id=\"{b}\" />\n\n**Bravo**  \nretained 😀\n\nDelta split.\n\n<Key id=\"{a}\" />\n\nAlpha 😀.\n"
        );
        std::fs::write(&path, &revision).unwrap();
        let pushes = store.lock().unwrap().pushes;
        assert_eq!(
            invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--json"], 0).await["items"][0]["outcome"],
            "would-save"
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), revision);
        assert_eq!(store.lock().unwrap().pushes, pushes);
        assert_eq!(
            invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await["items"][0]["outcome"],
            "saved"
        );
        let second = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(second.title, "Independent title");
        assert_eq!(second.state.id, id);
        assert_eq!(second.state.binding, first.state.binding);
        for key in [a, b] {
            assert_eq!(second.state.nodes[key], first.state.nodes[key]);
        }
        assert!(!second.state.nodes.contains_key(c));
        let bytes = std::fs::read(&path).unwrap();
        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        let pushes = store.lock().unwrap().pushes;
        assert_eq!(
            invoke(&server.base_url, root.path(), &["push", "--all", "--state", "draft", "--yes", "--json"], 0).await["items"][0]["outcome"],
            "unchanged"
        );
        assert_eq!(invoke(&server.base_url, root.path(), &["pull", "doc.mdx", "--json"], 0).await["unchanged"], 1);
        assert_eq!(store.lock().unwrap().pushes, pushes);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
        assert_eq!(server.request_count("/new"), usize::from(!publication));
        assert_eq!(
            server.requests().iter().filter(|r| r.method == "POST" && r.path.ends_with("dashboard")).count(),
            usize::from(publication)
        );
    }
}

#[tokio::test]
async fn confirmed_naming_is_content_noop_push_and_never_settles_editor_drift() {
    let (server, store) = server(true).await;
    let root = tempfile::tempdir().unwrap();
    assert!(
        cli(
            &server.base_url,
            root.path(),
            &["--publication", store::URI, "create", "doc.mdx", "--title", "Names"]
        )
        .output()
        .await
        .unwrap()
        .status
        .success()
    );
    let file = root.path().join("doc.mdx");
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, format!("{source}\n{{/* name-independent comment */}}\nA saved paragraph.\n")).unwrap();
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await;
    let before_source = std::fs::read_to_string(&file).unwrap();
    let before = leafbud_mdx::parse(&before_source).unwrap();
    let pushes = store.lock().unwrap().pushes;
    // Only local author path and its witness change; no native route claim.
    invoke(&server.base_url, root.path(), &["tidy", "doc.mdx", "--from", "filename", "--json"], 0).await;
    let source = std::fs::read_to_string(&file).unwrap();
    let source = source.replacen("\"path\": \"/doc\"", "\"path\": \"/renamed\"", 1);
    std::fs::write(&file, &source).unwrap();
    let status = invoke(&server.base_url, root.path(), &["status", "doc.mdx", "--json"], 0).await;
    assert_eq!(status["items"][0]["outcome"], "clean");
    assert_eq!(status["items"][0]["names"]["disposition"], "path-changed");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source, "status does not accept names");
    invoke(&server.base_url, root.path(), &["tidy", "doc.mdx", "--from", "path", "--json"], 0).await;
    let moved = root.path().join("renamed.mdx");
    assert!(!file.exists());
    let bytes = std::fs::read(&moved).unwrap();
    let modified = std::fs::metadata(&moved).unwrap().modified().unwrap();
    let after = leafbud_mdx::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
    assert_eq!(after.state.id, before.state.id);
    assert_eq!(after.state.binding, before.state.binding);
    assert_eq!(after.state.nodes, before.state.nodes);
    assert_eq!(after.state.sync, before.state.sync);
    assert_eq!(after.state.writer, before.state.writer);
    for args in [vec!["push", "renamed.mdx", "--json"], vec!["push", "renamed.mdx", "--yes", "--json"]] {
        let report = invoke(&server.base_url, root.path(), &args, 0).await;
        assert_eq!(report["items"][0]["outcome"], "unchanged");
    }
    invoke(&server.base_url, root.path(), &["pull", "renamed.mdx", "--json"], 0).await;
    assert_eq!(store.lock().unwrap().pushes, pushes);
    assert_eq!(std::fs::read(&moved).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&moved).unwrap().modified().unwrap(), modified);
    // Naming acceptance cannot clear a real editor input change.
    let dirty = std::str::from_utf8(&bytes).unwrap().replace("A saved paragraph.", "A local unsaved paragraph.");
    std::fs::write(&moved, &dirty).unwrap();
    invoke(&server.base_url, root.path(), &["tidy", "renamed.mdx", "--accept", "--json"], 0).await;
    let status = invoke(&server.base_url, root.path(), &["status", "renamed.mdx", "--json"], 0).await;
    assert_eq!(status["items"][0]["outcome"], "local-changed");
    assert_eq!(status["items"][0]["names"]["disposition"], "equal");
    assert_eq!(
        leafbud_mdx::parse(&std::fs::read_to_string(&moved).unwrap()).unwrap().state.sync,
        before.state.sync
    );
    assert_eq!(store.lock().unwrap().pushes, pushes);
}

#[tokio::test]
async fn source_spellings_prepare_save_and_repeat_preserve_page_associations() {
    let (server, store) = server(false).await;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("doc.mdx");
    assert!(
        cli(&server.base_url, root.path(), &["create", "doc.mdx", "--title", "Héllo", "--standalone"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    let source = format!(
        "{}\n<PageLink page = {{ \"detail\\u0073\" }} />\n\n<Page name='details' kind='linear'>\n\n<PageLink page = {{ \"p-au\\u0074o\" }} />\n\n</Page>\n\nAfter the page.\n",
        std::fs::read_to_string(&path).unwrap()
    );
    std::fs::write(&path, &source).unwrap();
    let refused = invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 1).await;
    assert!(refused["items"][0]["message"].as_str().unwrap().contains("cyclic page"));
    assert!(server.requests().is_empty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    // Cycles are source-valid but outside the native writer's current profile.
    // An acyclic source carries these same property spellings through save.
    let source = source.replace("<PageLink page = { \"p-au\\u0074o\" } />", "Inside the page.");
    std::fs::write(&path, &source).unwrap();
    let preview = invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--json"], 0).await;
    assert_eq!(preview["items"][0]["outcome"], "would-bind-and-save");
    assert!(server.requests().is_empty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await["items"][0]["outcome"],
        "saved"
    );
    let bytes = std::fs::read_to_string(&path).unwrap();
    let doc = leafbud_mdx::parse(&bytes).unwrap();
    assert_eq!(doc.pages.len(), 2);
    assert_eq!(doc.pages[0].blocks.len(), 2);
    assert_eq!(doc.pages[0].blocks[0].kind, leafbud_document::BlockKind::PageLink { page: "details".into() });
    assert_eq!(
        doc.pages[1].blocks[0].kind,
        leafbud_document::BlockKind::Paragraph {
            content: vec![leafbud_document::Inline::Run {
                text: "Inside the page.".into(),
                marks: vec![]
            }]
        }
    );
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let pushes = store.lock().unwrap().pushes;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await["items"][0]["outcome"],
        "unchanged"
    );
    assert_eq!(invoke(&server.base_url, root.path(), &["pull", "doc.mdx", "--json"], 0).await["unchanged"], 1);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert_eq!(store.lock().unwrap().pushes, pushes);
    assert_eq!(leafbud_mdx::parse(&bytes).unwrap().state.nodes, doc.state.nodes);
}

#[tokio::test]
async fn whole_flow_header_edit_defaults_save_refresh_and_noop_keep_unknown_author_source() {
    let (server, store) = server(true).await;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("doc.mdx");
    assert!(
        cli(
            &server.base_url,
            root.path(),
            &["--publication", store::URI, "create", "doc.mdx", "--title", "Title"]
        )
        .output()
        .await
        .unwrap()
        .status
        .success()
    );
    let header = leafbud_mdx::header(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut flow = serde_json::to_string(&header).unwrap();
    flow.insert_str(1, "custom: {nested: [1, 'yes']}, # custom stays\r\n");
    let source = format!("\u{feff}---\r\n{flow}\r\n---\r\n\r\n<Key id='b' />\r\nbody 😀\r\n");
    std::fs::write(&path, &source).unwrap();
    let edit = cli(
        &server.base_url,
        root.path(),
        &["edit", "doc.mdx", "--title", "Edited", "--tag", "one", "--description", "Description"],
    )
    .output()
    .await
    .unwrap();
    assert!(edit.status.success(), "{}", String::from_utf8_lossy(&edit.stderr));
    assert!(server.requests().is_empty());
    let edited = std::fs::read_to_string(&path).unwrap();
    assert!(edited.contains("custom: {nested: [1, 'yes']}, # custom stays\r\n"));
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--json"], 0).await["items"][0]["outcome"],
        "would-bind-and-save"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), edited);
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await["items"][0]["outcome"],
        "saved"
    );
    let saved = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(saved.title, "Edited");
    assert_eq!(saved.tags, ["one"]);
    assert_eq!(saved.extra["contributors"], json!(["did:plc:contributor-default"]));
    assert!(saved.theme.as_ref().unwrap().leaflet_watermark);
    store.lock().unwrap().data["title"] = json!("Remote title");
    store.lock().unwrap().data["leaflets_in_publications"][0]["title"] = json!("Remote title");
    store.lock().unwrap().extras.insert("publication_title".into(), json!("Remote title"));
    let pull = invoke(&server.base_url, root.path(), &["pull", "doc.mdx", "--json"], 0).await;
    assert_eq!(pull["updated"], 1, "{pull}");
    let bytes = std::fs::read_to_string(&path).unwrap();
    let adopted = leafbud_mdx::parse(&bytes).unwrap();
    assert_eq!(adopted.title, "Remote title");
    assert_eq!(adopted.state.nodes, saved.state.nodes);
    assert!(bytes.contains("custom: {nested: [1, 'yes']}, # custom stays\r\n"));
    assert!(bytes.starts_with("\u{feff}---\r\n{"));
    assert!(bytes.ends_with("\r\n<Key id='b' />\r\nbody 😀\r\n"));
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let pushes = store.lock().unwrap().pushes;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await["items"][0]["outcome"],
        "unchanged"
    );
    assert_eq!(invoke(&server.base_url, root.path(), &["pull", "doc.mdx", "--json"], 0).await["unchanged"], 1);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert_eq!(store.lock().unwrap().pushes, pushes);
}

#[tokio::test]
async fn body_comment_workflow_preserves_trivia_and_refuses_deleted_anchor_even_take_remote() {
    let (server, store) = server(false).await;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("doc.mdx");
    assert!(
        cli(&server.base_url, root.path(), &["create", "doc.mdx", "--title", "Title", "--standalone"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    let source = format!(
        "{}\n{{/* author note 😀 */}}\n\n<Paragraph>\n\n<Run text={{\"Body\"}} marks={{[]}} />\n\n</Paragraph>\n\n***\n",
        std::fs::read_to_string(&path).unwrap()
    );
    std::fs::write(&path, &source).unwrap();
    let preview = invoke(&server.base_url, root.path(), &["tidy", "doc.mdx", "--dry-run", "--json"], 0).await;
    assert_eq!(preview["normalized"], 1);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    invoke(&server.base_url, root.path(), &["tidy", "doc.mdx", "--json"], 0).await;
    assert!(std::fs::read_to_string(&path).unwrap().contains("{/* author note 😀 */}"));
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--json"], 0).await["items"][0]["outcome"],
        "would-bind-and-save"
    );
    assert!(server.requests().is_empty());
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await;
    let saved = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let key = &saved.pages[0].blocks[0].key;
    assert!(
        cli(&server.base_url, root.path(), &["keys", "rename", "doc.mdx", key, "intro"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    // A local-only comment edit is not editor drift and must be an exact no-op save.
    let source = std::fs::read_to_string(&path).unwrap().replace("author note", "revised note");
    std::fs::write(&path, &source).unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let pushes = store.lock().unwrap().pushes;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await["items"][0]["outcome"],
        "unchanged"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    let entity = saved.state.nodes[key].entity.clone();
    for row in store.lock().unwrap().facts.values_mut() {
        if row["entity"] == entity && row["attribute"] == "block/type" {
            row["data"]["value"] = "horizontal-rule".into();
        }
    }
    let pull = invoke(&server.base_url, root.path(), &["pull", "doc.mdx", "--json"], 0).await;
    assert_eq!(pull["updated"], 1, "{pull}");
    let source = std::fs::read_to_string(&path).unwrap();
    assert!(source.contains("{/* revised note 😀 */}\n\n***"));
    let adopted = leafbud_mdx::parse(&source).unwrap();
    assert_eq!(adopted.state.nodes["intro"].entity, entity);
    let placement = match &adopted.state.nodes["intro"].occurrence {
        leafbud_document::Occurrence::Block { placement, .. } => placement.clone(),
        _ => panic!("block"),
    };
    store.lock().unwrap().facts.remove(&placement);
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let refused = invoke(&server.base_url, root.path(), &["pull", "doc.mdx", "--take", "remote", "--json"], 1).await;
    assert_eq!(refused["failed"], 1, "{refused}");
    assert!(refused.to_string().contains("comment anchor"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert_eq!(store.lock().unwrap().pushes, pushes);
}

#[tokio::test]
async fn confirmed_key_rename_preserves_receipts_and_no_effect_push_and_metadata_classification() {
    let (server, store) = server(false).await;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("doc.mdx");
    assert!(
        cli(&server.base_url, root.path(), &["create", "doc.mdx", "--title", "Title", "--standalone"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    std::fs::write(&path, format!("{}\n<Key id='old' />\nText\n\n<Key id='link' />\n<PageLink page='details' />\n\n<Page name='details' kind='linear'>\n\n<Key id='inside' />\nInside\n\n</Page>\n", std::fs::read_to_string(&path).unwrap())).unwrap();
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await;
    let before = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let requests = server.requests().len();
    let pushes = store.lock().unwrap().pushes;
    for (old, new) in [("old", "intro"), ("details", "appendix"), (before.state.main_page.as_deref().unwrap(), "start")] {
        let out = cli(&server.base_url, root.path(), &["keys", "rename", "doc.mdx", old, new])
            .output()
            .await
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    }
    assert_eq!(server.requests().len(), requests, "rename cannot access the remote");
    let renamed = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(renamed.editor_value(), before.editor_value());
    assert_eq!(renamed.state.sync, before.state.sync);
    assert_eq!(renamed.state.writer, before.state.writer);
    let bytes = std::fs::read(&path).unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let preview = invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--json"], 0).await;
    assert_eq!(preview["items"][0]["outcome"], "unchanged");
    let repeat = invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await;
    assert_eq!(repeat["items"][0]["effects"], json!({}));
    assert_eq!(repeat["items"][0]["outcome"], "unchanged");
    assert_eq!(store.lock().unwrap().pushes, pushes);
    assert_eq!(
        invoke(&server.base_url, root.path(), &["status", "doc.mdx", "--json"], 0).await["items"][0]["outcome"],
        "clean"
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert!(
        cli(&server.base_url, root.path(), &["edit", "doc.mdx", "--published-at", "2026-11-09T00:00:00Z"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        invoke(&server.base_url, root.path(), &["status", "doc.mdx", "--json"], 0).await["items"][0]["outcome"],
        "clean"
    );
    for fields in [
        vec!["--title", "Changed title"],
        vec!["--description", "Changed description"],
        vec!["--tag", "one", "--tag", "two"],
    ] {
        let current = std::fs::read(&path).unwrap();
        let mut args = vec!["edit", "doc.mdx"];
        args.extend(fields);
        let requests = server.requests().len();
        let out = cli(&server.base_url, root.path(), &args).output().await.unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(server.requests().len(), requests);
        let edited = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(edited.state.sync, before.state.sync);
        assert_eq!(
            invoke(&server.base_url, root.path(), &["status", "doc.mdx", "--offline", "--json"], 0).await["items"][0]["outcome"],
            "local-changed"
        );
        std::fs::write(&path, current).unwrap();
    }
}

#[tokio::test]
async fn cli_decision_partial_recreation_and_usage_contracts() {
    let (server, store) = server(false).await;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("doc.mdx");
    assert!(
        cli(&server.base_url, root.path(), &["create", "doc.mdx", "--title", "Title", "--standalone"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    let text = format!("{}\nOriginal.\n", std::fs::read_to_string(&path).unwrap());
    std::fs::write(&path, text).unwrap();
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await;
    store.lock().unwrap().data["title"] = "Remote title".into();
    let pushes = store.lock().unwrap().pushes;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 3).await["items"][0]["outcome"],
        "needs-decision"
    );
    assert_eq!(store.lock().unwrap().pushes, pushes);
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--take", "local", "--yes", "--json"], 0).await;
    let first = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let key = first.pages[0].blocks[0].key.clone();
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--recreate", &key, "--yes", "--json"], 0).await;
    let second = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_ne!(second.state.nodes[&key].entity, first.state.nodes[&key].entity);
    let text = std::fs::read_to_string(&path).unwrap().replace("Original.", "Changed.");
    std::fs::write(&path, &text).unwrap();
    store.lock().unwrap().skip_all = true;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 4).await["items"][0]["outcome"],
        "partial"
    );
    let pushes = store.lock().unwrap().pushes;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 4).await["items"][0]["outcome"],
        "pending"
    );
    assert_eq!(store.lock().unwrap().pushes, pushes);
    for args in [vec!["push"], vec!["push", "doc.mdx", "--all"], vec!["push", "doc.mdx", "--take", "remote"]] {
        assert_eq!(cli(&server.base_url, root.path(), &args).output().await.unwrap().status.code(), Some(2));
    }
    assert_eq!(
        cli(&server.base_url, root.path(), &["push", "doc.mdx", "--recreate-body"])
            .output()
            .await
            .unwrap()
            .status
            .code(),
        Some(1)
    );
}

#[tokio::test]
async fn cli_terminal_recovery_previews_reuses_then_accepts_remote_without_retroactive_success() {
    let (server, store) = server(true).await;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("doc.mdx");
    assert!(
        cli(
            &server.base_url,
            root.path(),
            &["--publication", store::URI, "create", "doc.mdx", "--title", "Recovery"]
        )
        .output()
        .await
        .unwrap()
        .status
        .success()
    );
    std::fs::write(
        &path,
        format!("{}\nFirst paragraph.\n\nSecond paragraph.\n", std::fs::read_to_string(&path).unwrap()),
    )
    .unwrap();
    store.lock().unwrap().skip_attachments = true;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 4).await["items"][0]["outcome"],
        "partial"
    );
    let old = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.pending.unwrap();
    let original = std::fs::read(&path).unwrap();
    let requests = server.requests().len();
    let pending_doc = leafbud_mdx::parse(std::str::from_utf8(&original).unwrap()).unwrap();
    for args in [
        vec!["edit", "doc.mdx", "--title", "Not applied"],
        vec!["keys", "rename", "doc.mdx", &pending_doc.pages[0].blocks[0].key, "not-applied"],
    ] {
        let out = cli(&server.base_url, root.path(), &args).output().await.unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("recover"));
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
    let listing = invoke(&server.base_url, root.path(), &["list", "--json"], 0).await;
    assert_eq!(listing["items"][0]["pending"], true);
    assert_eq!(server.requests().len(), requests);
    let entities = store.lock().unwrap().entities.clone();
    let pushes = store.lock().unwrap().pushes;
    let preview = invoke(&server.base_url, root.path(), &["recover", "doc.mdx", "--take", "local", "--json"], 0).await;
    assert_eq!(preview["outcome"], "would-replan-local");
    assert_eq!(preview["observation"]["terminal"], true);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(store.lock().unwrap().pushes, pushes);
    store.lock().unwrap().skip_attachments = false;
    let result = invoke(&server.base_url, root.path(), &["recover", "doc.mdx", "--take", "local", "--yes", "--json"], 0).await;
    assert_eq!(result["outcome"], "replanned-local");
    assert_eq!(store.lock().unwrap().entities, entities);
    let settled = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(settled.state.pending.is_none());
    assert_eq!(settled.state.last_resolution.as_ref().unwrap().attempt_id, old.id);
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("Second paragraph.", "Unapplied original intent.");
    std::fs::write(&path, source).unwrap();
    store.lock().unwrap().skip_all = true;
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 4).await;
    store.lock().unwrap().skip_all = false;
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("Unapplied original intent.", "New local intent.");
    std::fs::write(&path, &source).unwrap();
    let inspected = invoke(&server.base_url, root.path(), &["recover", "doc.mdx", "--json"], 4).await;
    assert_eq!(inspected["observation"]["source_matches_original"], false);
    assert_eq!(inspected["observation"]["terminal"], true);
    let pushes = store.lock().unwrap().pushes;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["recover", "doc.mdx", "--take", "remote", "--json"], 0).await["outcome"],
        "would-accept-remote"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    assert_eq!(
        invoke(&server.base_url, root.path(), &["recover", "doc.mdx", "--take", "remote", "--yes", "--json"], 0).await["outcome"],
        "accepted-remote"
    );
    let accepted = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(accepted.pages, settled.pages);
    let bytes = std::fs::read(&path).unwrap();
    let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(
        invoke(&server.base_url, root.path(), &["recover", "doc.mdx", "--yes", "--json"], 0).await["outcome"],
        "no-attempt"
    );
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await["items"][0]["effects"],
        json!({})
    );
    assert_eq!(invoke(&server.base_url, root.path(), &["pull", "doc.mdx", "--json"], 0).await["unchanged"], 1);
    assert_eq!(store.lock().unwrap().pushes, pushes);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    let before_authoring = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let old_key = &before_authoring.pages[0].blocks[0].key;
    assert!(
        cli(&server.base_url, root.path(), &["keys", "rename", "doc.mdx", old_key, "recovered-intro"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    assert!(
        cli(&server.base_url, root.path(), &["edit", "doc.mdx", "--description", "Author description"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    let after_authoring = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(after_authoring.state.last_resolution, before_authoring.state.last_resolution);
    assert_eq!(after_authoring.state.sync, before_authoring.state.sync);
    assert_eq!(after_authoring.state.writer, before_authoring.state.writer);
    let listing = invoke(&server.base_url, root.path(), &["list", "--json"], 0).await;
    assert_eq!(listing["items"][0]["last_resolution"], true);
    let mut future = leafbud_mdx::header(&std::fs::read_to_string(&path).unwrap()).unwrap();
    future["@leafbud"]["lastResolution"]["futureDirection"] = true.into();
    let future_source = format!("---\n{}\n---\n\n<Key id='recovered-intro' />\nText\n", serde_json::to_string(&future).unwrap());
    std::fs::write(&path, &future_source).unwrap();
    for args in [
        vec!["edit", "doc.mdx", "--title", "Not applied"],
        vec!["keys", "rename", "doc.mdx", "recovered-intro", "nope"],
    ] {
        assert_eq!(cli(&server.base_url, root.path(), &args).output().await.unwrap().status.code(), Some(1));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), future_source);
    }
}

#[tokio::test]
async fn cli_uncertain_recovery_choice_never_sends_a_new_request_or_claims_success() {
    let (server, store) = server(false).await;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("doc.mdx");
    assert!(
        cli(&server.base_url, root.path(), &["create", "doc.mdx", "--title", "Uncertain", "--standalone"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    std::fs::write(&path, format!("{}\nUnknown finality.\n", std::fs::read_to_string(&path).unwrap())).unwrap();
    store.lock().unwrap().no_ack = true;
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 4).await;
    let bytes = std::fs::read(&path).unwrap();
    let pushes = store.lock().unwrap().pushes;
    for side in ["local", "remote"] {
        let report = invoke(&server.base_url, root.path(), &["recover", "doc.mdx", "--take", side, "--yes", "--json"], 4).await;
        assert_eq!(report["outcome"], "pending");
        assert_eq!(report["observation"]["terminal"], false);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(store.lock().unwrap().pushes, pushes);
    }
    assert_eq!(server.request_count("/new"), 1);
}

/// Existing standalone only: this cannot mint or replay the old request. The
/// operator commissioned deliberate accept-current in ses_e3d6d728effe6RFnyZcrUtifT9.
#[tokio::test]
#[ignore = "requires existing standalone terminal accept-current authorization; CREATE-FREE"]
async fn authorized_existing_standalone_terminal_accept_current_cli() {
    use leafbud_document::{Effect, PushStage, ResolutionDirection, ResolutionDisposition};
    assert_eq!(std::env::var("LEAFBUD_TERMINAL_PROBE").as_deref(), Ok("authorized-existing-standalone"));
    let dir = std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-effects/deployed")).unwrap();
    let path = dir.join("standalone.mdx");
    let source = std::fs::read_to_string(&path).unwrap();
    let original = leafbud_mdx::parse(&source).unwrap();
    let pending = original
        .state
        .pending
        .clone()
        .expect("existing old partial required; never reconstruct old intent");
    let binding = original.state.binding.clone().expect("bound existing draft required; NO MINT");
    let locator: Value = serde_json::from_slice(&std::fs::read(dir.join("standalone.locator.json")).unwrap()).unwrap();
    assert!(
        locator["token"].as_str() == Some(binding.token.to_string().as_str()),
        "selected target differs from private original locator"
    );
    assert!(
        matches!(
            pending.effect,
            Effect::Push {
                stage: PushStage::Partial { through: 2 },
                ..
            }
        ),
        "expected original terminal partial only"
    );
    assert!(original.state.sync.is_none() && original.state.last_resolution.is_none());
    let publication = dir.join("publication.mdx");
    let p1 = std::fs::read(&publication).unwrap();
    let p1_mtime = std::fs::metadata(&publication).unwrap().modified().unwrap();
    let budget_paths = [dir.join("standalone.started"), dir.join("publication.started")];
    let budgets = [std::fs::read(&budget_paths[0]).unwrap(), std::fs::read(&budget_paths[1]).unwrap()];
    let before_path = dir.join("standalone.terminal-before.private.json");
    assert!(
        !before_path.exists(),
        "proof already started: inspect private current state, do not reconstruct old intent"
    );
    std::fs::write(
        &before_path,
        serde_json::to_vec_pretty(&json!({"session":"opencode://ses_e3d6d728effe6RFnyZcrUtifT9","source":source,"attempt":pending,"binding":binding})).unwrap(),
    )
    .unwrap();
    async fn actual(dir: &Path, label: &str, args: &[&str], code: i32) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_leafbud"))
            .arg("--root")
            .arg(dir)
            .args(args)
            .output()
            .await
            .unwrap();
        std::fs::write(dir.join(format!("standalone.terminal-{label}.stdout.private.json")), &output.stdout).unwrap();
        std::fs::write(dir.join(format!("standalone.terminal-{label}.stderr.private.txt")), &output.stderr).unwrap();
        assert_eq!(
            output.status.code(),
            Some(code),
            "CLI result unexpected; inspect ignored private outputs, no remint/replay"
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    let inspect = actual(&dir, "inspect", &["recover", "standalone.mdx", "--json"], 4).await;
    assert_eq!(inspect["observation"]["terminal"], true);
    assert_eq!(inspect["observation"]["acknowledged"], 2);
    assert_eq!(inspect["observation"]["outcome"], "resolved-partial");
    let preview = actual(&dir, "preview", &["recover", "standalone.mdx", "--take", "remote", "--json"], 0).await;
    assert_eq!(preview["outcome"], "would-accept-remote");
    assert!(std::fs::read_to_string(&path).unwrap() == source, "preview changed source");
    let result = actual(&dir, "accept", &["recover", "standalone.mdx", "--take", "remote", "--yes", "--json"], 0).await;
    assert_eq!(result["outcome"], "accepted-remote");
    assert_eq!(result["effects"], json!({}));
    let accepted = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(
        accepted.state.id == original.state.id && accepted.state.binding == original.state.binding && accepted.state.writer == original.state.writer,
        "source/binding/writer changed"
    );
    assert!(accepted.state.pending.is_none() && accepted.state.sync.is_some());
    let resolution = accepted.state.last_resolution.as_ref().expect("old provenance preserved");
    assert!(
        resolution.attempt_id == pending.id && resolution.input_hash == pending.input_hash && resolution.desired_hash == pending.desired_hash,
        "old provenance changed"
    );
    assert_eq!(resolution.direction, ResolutionDirection::Remote);
    assert_eq!(resolution.disposition, ResolutionDisposition::OriginalNotVerified);
    assert_eq!(resolution.acknowledged, 2);
    assert!(resolution.next_attempt.is_none());
    assert!(accepted.theme.as_ref().is_some_and(|theme| theme.leaflet_watermark));
    let bytes = std::fs::read(&path).unwrap();
    let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    let status = actual(&dir, "status", &["status", "standalone.mdx", "--json"], 0).await;
    assert_eq!(status["items"][0]["outcome"], "clean");
    assert_eq!(actual(&dir, "pull", &["pull", "standalone.mdx", "--json"], 0).await["unchanged"], 1);
    let repeat = actual(&dir, "repeat", &["push", "standalone.mdx", "--yes", "--json"], 0).await;
    assert_eq!(repeat["items"][0]["outcome"], "unchanged");
    assert_eq!(repeat["items"][0]["effects"], json!({}));
    assert_eq!(
        actual(&dir, "again", &["recover", "standalone.mdx", "--yes", "--json"], 0).await["outcome"],
        "no-attempt"
    );
    assert!(
        std::fs::read(&path).unwrap() == bytes && std::fs::metadata(&path).unwrap().modified().unwrap() == mtime,
        "repeat changed accepted source"
    );
    assert!(
        std::fs::read(&publication).unwrap() == p1 && std::fs::metadata(&publication).unwrap().modified().unwrap() == p1_mtime,
        "P1 changed"
    );
    for (path, before) in budget_paths.iter().zip(budgets) {
        assert!(std::fs::read(path).unwrap() == before, "consumed creation budget changed");
    }
    std::fs::write(dir.join("standalone.terminal-proof.private.json"), serde_json::to_vec_pretty(&json!({"session":"opencode://ses_e3d6d728effe6RFnyZcrUtifT9","policy":"accept-current; original not verified","resolution":resolution,"newStatus":"clean","pull":"unchanged","repeat":"empty-effects byte/mtime exact","creationCalls":0,"nativeContentEffects":0,"p1":"untouched","budgets":"unchanged"})).unwrap()).unwrap();
}

/// No creation route is called: the source MUST already be the settled,
/// human-selected P1. Both old creation slots stay consumed and standalone stays
/// byte/mtime untouched. All locators/content evidence is private ignored data.
#[tokio::test]
#[ignore = "requires existing selected P1 edit authorization; never creates a draft"]
async fn authorized_selected_p1_complete_bound_cli_save() {
    use leafbud_document::{Block, BlockKind, Inline, Mark};
    assert_eq!(std::env::var("LEAFBUD_KEYED_PROBE").as_deref(), Ok("authorized-selected-p1"));
    let root = std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-effects")).unwrap();
    let dir = root.join("deployed");
    let path = dir.join("publication.mdx");
    let budgets = [dir.join("standalone.started"), dir.join("publication.started")];
    let budget_bytes = [std::fs::read(&budgets[0]).unwrap(), std::fs::read(&budgets[1]).unwrap()];
    let standalone = dir.join("standalone.mdx");
    let standalone_bytes = std::fs::read(&standalone).unwrap();
    let standalone_mtime = std::fs::metadata(&standalone).unwrap().modified().unwrap();
    let catalogue: Value = serde_json::from_slice(&std::fs::read(root.join("recovery-candidates/catalogue.private.json")).unwrap()).unwrap();
    let candidate = catalogue["candidates"].as_array().unwrap().iter().find(|c| c["label"] == "P1").unwrap();
    let original_source = std::fs::read_to_string(&path).unwrap();
    let original = leafbud_mdx::parse(&original_source).unwrap();
    assert!(original.state.pending.is_none() && original.state.sync.is_some());
    let binding = original.state.binding.clone().expect("P1 must be already bound: NO MINT");
    assert_eq!(binding.token.to_string(), candidate["token"].as_str().unwrap());
    assert_eq!(original.pages.len(), 1);
    assert_eq!(original.pages[0].blocks.len(), 1);
    let baseline_evidence = dir.join("publication.keyed-before.private.json");
    assert!(
        !baseline_evidence.exists(),
        "proof already started; inspect actual source/pending rather than rerun or recreate old intent"
    );
    std::fs::write(&baseline_evidence,serde_json::to_vec_pretty(&json!({"session":"opencode://ses_e3d96851dffeS2cPD2vPHjGklF","source":original_source,"binding":binding,"originalMintAttribution":"not-established; human-selected-P1"})).unwrap()).unwrap();
    async fn live(dir: &Path, args: &[&str]) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_leafbud"))
            .arg("--root")
            .arg(dir)
            .args(args)
            .output()
            .await
            .unwrap();
        std::fs::write(dir.join(format!("publication.keyed-{}.stdout.private.json", args[0])), &output.stdout).unwrap();
        assert!(output.status.success(), "CLI failed; inspect private stdout/source; no replan/remint");
        serde_json::from_slice(&output.stdout).unwrap()
    }
    let a = original.pages[0].blocks[0].key.clone();
    let b = format!("b-keyed-{}", leafbud_document::Uuid::new_v4());
    let c = format!("b-keyed-{}", leafbud_document::Uuid::new_v4());
    let paragraph = |key: &str, text: &str| Block {
        key: key.into(),
        kind: BlockKind::Paragraph {
            content: vec![Inline::Run {
                text: text.into(),
                marks: vec![],
            }],
        },
    };
    let mut baseline = original.clone();
    baseline.pages[0].blocks = vec![
        original.pages[0].blocks[0].clone(),
        paragraph(&b, "Bravo and delta."),
        paragraph(&c, "Charlie retained when detached."),
    ];
    let baseline_source = leafbud_mdx::render(&baseline).unwrap();
    std::fs::write(&path, &baseline_source).unwrap();
    let preview = live(&dir, &["push", "publication.mdx", "--json"]).await;
    assert_eq!(preview["items"][0]["outcome"], "would-save");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), baseline_source);
    let first_report = live(&dir, &["push", "publication.mdx", "--yes", "--json"]).await;
    assert_eq!(first_report["items"][0]["outcome"], "saved");
    let first = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(first.state.nodes[&a], original.state.nodes[&a]);
    assert_eq!(first.state.binding, Some(binding.clone()));
    assert_eq!(first.state.id, original.state.id);
    let h = format!("b-keyed-{}", leafbud_document::Uuid::new_v4());
    let d = format!("b-keyed-{}", leafbud_document::Uuid::new_v4());
    let mut revised = first.clone();
    revised.pages[0].blocks = vec![
        Block {
            key: h.clone(),
            kind: BlockKind::Heading {
                level: 2,
                content: vec![Inline::Run {
                    text: "Keyed save verification, not the title".into(),
                    marks: vec![],
                }],
            },
        },
        Block {
            key: b.clone(),
            kind: BlockKind::Paragraph {
                content: vec![
                    Inline::Run {
                        text: "Bravo é😀e\u{301}".into(),
                        marks: vec![
                            Mark::Strong,
                            Mark::Link {
                                href: "https://example.test/keyed".into(),
                            },
                        ],
                    },
                    Inline::HardBreak,
                    Inline::Run {
                        text: "retained paragraph and text fact".into(),
                        marks: vec![Mark::Em],
                    },
                ],
            },
        },
        paragraph(&d, "Delta split into a new keyed block."),
        first.pages[0].blocks[0].clone(),
    ];
    let revised_source = leafbud_mdx::render(&revised).unwrap();
    std::fs::write(&path, &revised_source).unwrap();
    let preview2 = live(&dir, &["push", "publication.mdx", "--json"]).await;
    assert_eq!(preview2["items"][0]["outcome"], "would-save");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), revised_source);
    let revised_report = live(&dir, &["push", "publication.mdx", "--yes", "--json"]).await;
    assert_eq!(revised_report["items"][0]["outcome"], "saved");
    let final_source = std::fs::read_to_string(&path).unwrap();
    let final_doc = leafbud_mdx::parse(&final_source).unwrap();
    assert_eq!(final_doc.state.binding, Some(binding));
    assert_eq!(final_doc.state.id, original.state.id);
    assert_eq!(final_doc.title, original.title);
    for key in [&a, &b] {
        assert_eq!(final_doc.state.nodes[key], first.state.nodes[key]);
    }
    assert!(!final_doc.state.nodes.contains_key(&c));
    assert_eq!(final_doc.pages[0].blocks.iter().map(|b| &b.key).collect::<Vec<_>>(), vec![&h, &b, &d, &a]);
    let before = std::fs::read(&path).unwrap();
    let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    let pull = live(&dir, &["pull", "publication.mdx", "--json"]).await;
    assert_eq!(pull["unchanged"], 1);
    let repeat = live(&dir, &["push", "publication.mdx", "--yes", "--json"]).await;
    assert_eq!(repeat["items"][0]["outcome"], "unchanged");
    assert_eq!(repeat["items"][0]["effects"], json!({}));
    let status = live(&dir, &["status", "publication.mdx", "--json"]).await;
    assert_eq!(status["items"][0]["outcome"], "clean");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    assert_eq!(std::fs::read(&standalone).unwrap(), standalone_bytes);
    assert_eq!(std::fs::metadata(&standalone).unwrap().modified().unwrap(), standalone_mtime);
    for i in 0..2 {
        assert_eq!(std::fs::read(&budgets[i]).unwrap(), budget_bytes[i]);
    }
    std::fs::write(dir.join("publication.keyed-proof.private.json"),serde_json::to_vec_pretty(&json!({
        "session":"opencode://ses_e3d96851dffeS2cPD2vPHjGklF","baseline":first_report,"revision":revised_report,"previewBaseline":preview,"previewRevision":preview2,
        "retainedNodes":[&a,&b],"droppedNode":c,"detachedEntity":first.state.nodes[&c].entity,"finalDocument":final_doc,"pull":pull,"repeat":repeat,"status":status,
        "draftCreationCalls":0,"standaloneUntouched":true,"budgetsUnchanged":true,"lifecycle":"unpublished-draft","originalMintAttribution":"not-established; human-selected-P1"
    })).unwrap()).unwrap();
}
