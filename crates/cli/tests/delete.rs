//! Deliberate deletion through the real CLI binary and loopback HTTP.
#![allow(clippy::disallowed_types)]
#[path = "../../atproto/src/testserver.rs"]
#[allow(dead_code)]
mod testserver;
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use tokio::process::Command;

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const ROOT: &str = "01999000-0000-7aaa-8000-000000000002";
const SET: &str = "01999000-0000-7aaa-8000-000000000009";
const DID: &str = "did:plc:synthetic";
const JS: &str = include_str!("../../leaflet/web/tests/fixtures/delete-reference.js");
const VOID: &str = include_str!("../../leaflet/web/tests/fixtures/delete-void.flight");
const TOKEN_MISSING: &str = r#"{"result":{"error":{"code":"PGRST116","message":"withheld"},"status":406,"data":null}}"#;

fn fixture() -> Value {
    let mut fixture: Value = serde_json::from_str(include_str!("../../../experiments/02-draft-internals/fixtures/export-pull-doc.json")).unwrap();
    fixture["leaflet_data"]["permission_token_rights"] = json!([{"token":TOKEN,"entity_set":SET,"read":true,"write":true}]);
    fixture
}

/// Same-origin delete origin: metadata/pull reads, /home discovery, action
/// POSTs and the post-effect disappearance, all flag-driven.
async fn origin(deleted: Arc<AtomicBool>) -> (testserver::TestServer, tempfile::TempDir) {
    let posts = Arc::new(AtomicUsize::new(0));
    let handler_posts = posts.clone();
    let fixture = fixture();
    let server = testserver::TestServer::spawn_headers(Arc::new(move |r| match (r.method.as_str(), r.path.as_str()) {
        ("GET", "/home") => (
            200,
            vec![("Content-Type".into(), "text/html".into())],
            format!(
                "<script src=\"/_next/static/delete.js\"></script><script>self.__next_f.push({})</script>",
                json!([1, format!("0:{}", json!({"atp_did":DID,"bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]}))])
            ),
        ),
        ("GET", "/_next/static/delete.js") => (200, vec![("Content-Type".into(), "text/javascript".into())], JS.into()),
        ("POST", "/home") => {
            handler_posts.fetch_add(1, Ordering::SeqCst);
            deleted.store(true, Ordering::SeqCst);
            (200, vec![("Content-Type".into(), "text/x-component".into())], VOID.into())
        }
        ("POST", "/api/rpc/get_leaflet_data") => {
            let body = if deleted.load(Ordering::SeqCst) {
                TOKEN_MISSING.into()
            } else {
                json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()
            };
            (200, vec![("Content-Type".into(), "application/json".into())], body)
        }
        ("POST", "/api/rpc/pull") => (200, vec![("Content-Type".into(), "application/json".into())], fixture["response"].to_string()),
        _ => panic!("unexpected {} {}", r.method, r.path),
    }))
    .await;
    let temp = tempfile::tempdir().unwrap();
    (server, temp)
}

fn source(server: &testserver::TestServer, root: &Path) -> std::path::PathBuf {
    let path = root.join("doc.mdx");
    let id = leafbud_document::Uuid::new_v4();
    std::fs::write(
        &path,
        format!(
            "---\nformat: leafbud-mdx/2\ntitle: T\n'@leafbud':\n  id: {id}\n  state: draft\n  binding: {{baseUrl: '{}', token: {}, root: {}, entitySet: {}, entryFact: f-entry, publication: null}}\n---\n\n<Key id=\"b-1\" />\nhello\n",
            server.base_url,
            TOKEN,
            ROOT,
            SET
        ),
    )
    .unwrap();
    path
}

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

#[tokio::test]
async fn delete_previews_executes_and_purges_through_the_binary() {
    let deleted = Arc::new(AtomicBool::new(false));
    let (server, temp) = origin(deleted.clone()).await;
    let path = source(&server, temp.path());
    let base = server.base_url.clone();
    let root = temp.path().to_path_buf();

    // Preview: honest report, exit 0, no effect, no write.
    let report = invoke(&base, &root, &["delete", "doc.mdx", "--json"], 0).await;
    assert_eq!(report["outcome"], "would-delete");
    assert_eq!(report["lifecycle"], "draft");
    assert!(!deleted.load(Ordering::SeqCst));
    assert!(std::fs::read_to_string(&path).unwrap().contains("binding:"));

    // Execute: verified deletion, exit 0, local file converted.
    let report = invoke(&base, &root, &["delete", "doc.mdx", "--yes", "--json"], 0).await;
    assert_eq!(report["outcome"], "deleted");
    assert_eq!(report["observed"]["token_gone"], true);
    assert_eq!(report["local"], "unbound");
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(!after.contains("binding:"), "binding removed");
    assert!(after.contains("hello"), "authored body preserved");
    assert!(after.contains("<Key id=\"b-1\" />"), "source keys preserved");

    // An unbound operand has nothing to delete: preserved, needs-decision.
    invoke(&base, &root, &["delete", "doc.mdx", "--yes", "--json"], 4).await;

    // Purge on a fresh binding removes the file entirely.
    let deleted = Arc::new(AtomicBool::new(false));
    let (server, temp) = origin(deleted).await;
    let path = source(&server, temp.path());
    let base = server.base_url.clone();
    let root = temp.path().to_path_buf();
    let report = invoke(&base, &root, &["delete", "doc.mdx", "--yes", "--purge", "--json"], 0).await;
    assert_eq!(report["outcome"], "deleted");
    assert_eq!(report["local"], "purged");
    assert!(!path.exists());
}

#[tokio::test]
async fn mismatched_binding_blocks_with_exit_four_and_no_dispatch() {
    let deleted = Arc::new(AtomicBool::new(false));
    let (server, temp) = origin(deleted.clone()).await;
    let path = source(&server, temp.path());
    // Corrupt the local root: the live editor differs from the binding.
    let corrupted = std::fs::read_to_string(&path).unwrap().replace(ROOT, "01999000-0000-7aaa-8000-000000000099");
    std::fs::write(&path, corrupted).unwrap();
    let report = invoke(&server.base_url, temp.path(), &["delete", "doc.mdx", "--yes", "--json"], 4).await;
    assert_eq!(report["outcome"], "blocked");
    assert!(!deleted.load(Ordering::SeqCst), "no effect dispatched");
    assert!(std::fs::read_to_string(&path).unwrap().contains("binding:"), "source preserved");
}
