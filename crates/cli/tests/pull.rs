//! Full pull/status command execution against synthetic loopback responses; no
//! real account or user document is touched.
// Each integration test is its own binary; this one uses a subset of the
// shared responder's recorded surface.
#[path = "../../atproto/src/testserver.rs"]
#[allow(dead_code)]
mod testserver;

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use tokio::process::Command as TokioCommand;

const FIXTURE: &str = include_str!("../../../experiments/02-draft-internals/fixtures/export-pull-doc.json");
const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).unwrap()
}

/// Loopback origin serving a mutable token → fixture map.
async fn server() -> (testserver::TestServer, Arc<RwLock<BTreeMap<String, Value>>>) {
    let current: Arc<RwLock<BTreeMap<String, Value>>> = Arc::new(RwLock::new(BTreeMap::from([(TOKEN.to_owned(), fixture())])));
    let handle = current.clone();
    let server = testserver::TestServer::spawn(Arc::new(move |r| {
        let fixtures = handle.read().expect("fixtures").clone();
        let token = r.body_json()["token_id"].as_str().unwrap_or_default().to_owned();
        let Some(fixture) = fixtures.get(&token) else {
            return (200, r#"{"result":{"error":{"code":"PGRST116"},"status":406,"data":null}}"#.into());
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

fn cli(base: &str, root: &str, args: &[&str]) -> TokioCommand {
    let mut command = TokioCommand::new(env!("CARGO_BIN_EXE_leafspring"));
    command
        .env_remove("LEAFLET_AUTH_TOKEN")
        .env_remove("LEAFLET_BASE_URL")
        .env_remove("HOME")
        .env("XDG_CONFIG_HOME", concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/absent"))
        .args(["--base-url", base, "--root", root])
        .args(args);
    command
}

async fn import(server: &testserver::TestServer, root: &str) {
    let output = cli(&server.base_url, root, &["import", TOKEN, "--to", "doc.mdx", "--json"])
        .output()
        .await
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Rewrite the fixture's paragraph text payload (plain ASCII, no marks).
fn set_paragraph_text(fixture: &mut Value, text: &str) {
    use base64::Engine;
    let doc = yrs::Doc::new();
    let fragment = doc.get_or_insert_xml_fragment("prosemirror");
    let mut txn = doc.transact_mut();
    use yrs::{Transact as _, XmlFragment as _};
    let paragraph = fragment.insert(&mut txn, 0, yrs::XmlElementPrelim::empty("paragraph"));
    paragraph.insert(&mut txn, 0, yrs::XmlTextPrelim::new(text));
    let update = txn.encode_update_v1();
    drop(txn);
    let payload = base64::engine::general_purpose::STANDARD.encode(update);
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

#[tokio::test]
async fn pull_all_reports_noop_with_json_and_human_output() {
    let (server, _current) = server().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    import(&server, root).await;
    let doc = temp.path().join("doc.mdx");
    let bytes = std::fs::read(&doc).unwrap();
    let output = cli(&server.base_url, root, &["pull", "--all", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["version"], 1);
    assert_eq!(report["unchanged"], 1);
    assert_eq!(report["selected"], 1);
    assert_eq!(std::fs::read(&doc).unwrap(), bytes);
    let output = cli(&server.base_url, root, &["pull", "doc.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("unchanged"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 selected"));
}

#[tokio::test]
async fn pull_local_edits_exit_three_and_take_remote_adopts() {
    let (server, current) = server().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    import(&server, root).await;
    let doc = temp.path().join("doc.mdx");
    let source = std::fs::read_to_string(&doc).unwrap();
    let edited = source.replace("Synthetic body copy", "Edited locally");
    std::fs::write(&doc, &edited).unwrap();
    set_paragraph_text(current.write().unwrap().get_mut(TOKEN).unwrap(), "Remote rewrote the paragraph");
    let output = cli(&server.base_url, root, &["pull", "doc.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("needs-decision"));
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), edited);
    let output = cli(&server.base_url, root, &["pull", "doc.mdx", "--dry-run", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), edited);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["dry_run"], true);
    assert_eq!(report["needs_decision"], 1);
    let output = cli(&server.base_url, root, &["pull", "doc.mdx", "--take", "remote"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let after = std::fs::read_to_string(&doc).unwrap();
    assert!(!after.contains("Edited locally"));
}

#[tokio::test]
async fn pull_usage_and_operand_errors() {
    let (server, _current) = server().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let output = cli(&server.base_url, root, &["pull"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(2));
    let output = cli(&server.base_url, root, &["pull", "--all", "doc.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(2));
    let output = cli(&server.base_url, root, &["pull", "absent.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("not found"));
    // A lone operational failure is exit 1; mixed batches diverge with 4.
    std::fs::write(temp.path().join("plain.mdx"), "not managed\n").unwrap();
    import(&server, root).await;
    let output = cli(&server.base_url, root, &["pull", "doc.mdx", "plain.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["unchanged"], 1);
    assert_eq!(report["failed"], 1);
    let output = cli(&server.base_url, root, &["pull", "plain.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a managed document"));
}

#[tokio::test]
async fn status_json_conflict_and_convergence_are_successful_read_only_reports() {
    let (server, current) = server().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    import(&server, root).await;
    let doc = temp.path().join("doc.mdx");
    let local = std::fs::read_to_string(&doc).unwrap().replace("Synthetic body copy", "My current edit");
    std::fs::write(&doc, &local).unwrap();
    let modified = std::fs::metadata(&doc).unwrap().modified().unwrap();
    set_paragraph_text(current.write().unwrap().get_mut(TOKEN).unwrap(), "Another edit");
    let output = cli(&server.base_url, root, &["status", "doc.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!((report["version"].as_u64(), report["selected"].as_u64()), (Some(1), Some(1)));
    assert_eq!(report["items"][0]["outcome"], "both-changed");
    assert_eq!(report["items"][0]["remote"]["freshness"], "current");
    assert_eq!(report["names"], "not-tracked");
    assert_eq!(report["release"], "unknown-not-implemented");
    set_paragraph_text(current.write().unwrap().get_mut(TOKEN).unwrap(), "My current edit");
    let output = cli(&server.base_url, root, &["status", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["items"][0]["outcome"], "converged");
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), local);
    assert_eq!(std::fs::metadata(&doc).unwrap().modified().unwrap(), modified);
    let output = cli(&server.base_url, root, &["status", "doc.mdx"]).output().await.unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("converged") && text.contains("read-only") && text.contains("baseline: scoped"));
    assert!(!text.contains("My current edit"));
}

#[tokio::test]
async fn offline_status_uses_configured_workspace_and_does_not_even_load_credentials() {
    let (server, _) = server().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    std::fs::create_dir(&root).unwrap();
    import(&server, root.to_str().unwrap()).await;
    let config = temp.path().join("config/leafspring");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(
        config.join("leafspring.toml"),
        format!("workspace = {:?}\npublication = 'do-not-retarget'\n", root),
    )
    .unwrap();
    // An unusable credential file and invalid environment would fail connect;
    // offline status is independent of that path and uses no remote requests.
    std::fs::create_dir(config.join("auth-token")).unwrap();
    let calls = server.request_count("/api/rpc/get_leaflet_data");
    let output = TokioCommand::new(env!("CARGO_BIN_EXE_leafspring"))
        .env("XDG_CONFIG_HOME", config.parent().unwrap())
        .env("LEAFLET_AUTH_TOKEN", "")
        .env("LEAFLET_BASE_URL", "not a URL")
        .args(["status", "--offline", "--json"])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["workspace"], root.to_str().unwrap());
    assert_eq!(report["offline"], true);
    assert_eq!(report["items"][0]["outcome"], "unknown");
    assert_eq!(report["items"][0]["remote"]["freshness"], "offline");
    assert_eq!(report["items"][0]["remote"]["change"], "unknown");
    assert_eq!(server.request_count("/api/rpc/get_leaflet_data"), calls);
}

#[tokio::test]
async fn status_invalid_operands_are_per_file_and_missing_remote_is_a_report_not_failure() {
    let (server, current) = server().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    import(&server, root).await;
    std::fs::write(temp.path().join("broken.mdx"), "---\n\"@leafspring\": [unclosed\n---\nprivate-source-prose\n").unwrap();
    let output = cli(&server.base_url, root, &["status", "broken.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(serde_json::from_slice::<Value>(&output.stdout).unwrap()["items"][0]["outcome"], "invalid");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("private-source-prose"));
    let output = cli(&server.base_url, root, &["status", "doc.mdx", "absent.mdx", "--json"])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["items"][1]["outcome"], "local-missing");
    current.write().unwrap().remove(TOKEN);
    let output = cli(&server.base_url, root, &["status", "doc.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(serde_json::from_slice::<Value>(&output.stdout).unwrap()["items"][0]["outcome"], "missing");
    let mut malformed = fixture();
    malformed["response"] = json!({"private":"reflected-remote-secret"});
    current.write().unwrap().insert(TOKEN.into(), malformed);
    let output = cli(&server.base_url, root, &["status", "doc.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["items"][0]["outcome"], "failed");
    assert_eq!(report["items"][0]["remote"]["freshness"], "unavailable");
    assert_eq!(report["items"][0]["remote"]["change"], "unknown");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("reflected-remote-secret"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("reflected-remote-secret"));
    let output = cli(&server.base_url, root, &["status", "--take", "remote"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(2), "status has no mutation/settlement selector");
    let output = cli(&server.base_url, root, &["diff", "doc.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("not implemented"));
}
