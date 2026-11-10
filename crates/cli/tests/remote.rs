//! Full command execution against synthetic loopback responses; no real account.
#[path = "../../atproto/src/testserver.rs"]
mod testserver;

use serde_json::{Value, json};
use std::sync::Arc;
use testserver::TestServer;
use tokio::process::Command;

const HOME: &str = include_str!("../../../experiments/01-draft-list/fixture-home.html");
const DRAFT: &str = include_str!("../../../experiments/02-draft-internals/fixtures/get_leaflet_data-pure-draft.json");
const ID: &str = "01999000-0000-7aaa-8000-000000000001";

fn cli(base: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_leafbud"));
    command
        .env_remove("LEAFLET_AUTH_TOKEN")
        .env_remove("LEAFLET_BASE_URL")
        .env_remove("HOME")
        .env("XDG_CONFIG_HOME", concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/absent"))
        .arg("--base-url")
        .arg(base);
    command
}

async fn fixture_server() -> TestServer {
    TestServer::spawn(Arc::new(|request| match request.path.as_str() {
        "/home" => (200, HOME.into()),
        "/api/rpc/get_leaflet_data" => (200, DRAFT.into()),
        _ => (404, "unexpected request".into()),
    }))
    .await
}

#[tokio::test]
async fn list_filter_json_and_human_output() {
    let server = fixture_server().await;
    let output = cli(&server.base_url)
        .args(["remote", "list", "--state", "draft", "--json"])
        .output()
        .await
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let list: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(list["count"], 3);
    assert_eq!(list["discovered_count"], 5);
    assert_eq!(list["coverage"]["completeness"], "unknown");
    assert!(list["leaflets"].as_array().unwrap().iter().all(|l| l["state"] == "draft"));
    assert!(output.stderr.is_empty());
    let output = cli(&server.base_url).args(["remote", "list"]).output().await.unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
    assert!(text.contains("5 shown / 5 discovered"));
    assert!(text.contains("pagination are unverified"));
    assert!(text.contains("A synthetic draft title long enough…"));
    assert!(text.contains("* contributor access"));
    assert_eq!(server.request_count("/home"), 2);
}

#[tokio::test]
async fn show_native_url_json_and_human_output() {
    let server = fixture_server().await;
    let locator = format!("{}/{ID}", server.base_url);
    let output = cli(&server.base_url).args(["remote", "show", &locator, "--json"]).output().await.unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let show: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(show["version"], 1);
    assert_eq!(show["editor_url"], locator);
    assert_eq!(show["state"], "draft");
    assert_eq!(
        show["data"]["raw"]["permission_token_rights"][0]["entity_sets"]["permission_tokens"][0]["id"],
        ID
    );
    let output = cli(&server.base_url).args(["remote", "show", ID]).output().await.unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("read=true write=true (reported; not a write probe)"));
    assert!(text.contains("publication linkage: none"));
    assert!(text.contains("no editor content fetched"));
    assert_eq!(server.requests()[0].body_json(), json!({"token_id": ID}));
    assert_eq!(server.requests()[0].method, "POST");
    assert!(server.requests()[0].query.is_empty());
}

#[tokio::test]
async fn flag_env_and_xdg_credentials_have_proven_precedence() {
    let server = fixture_server().await;
    let file_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/config");
    for (flag, env, expected) in [
        (Some("synthetic-flag"), Some("synthetic-env"), "synthetic-flag"),
        (None, Some("synthetic-env"), "synthetic-env"),
        (None, None, "synthetic-file-auth"),
    ] {
        let mut command = cli(&server.base_url);
        command.env("XDG_CONFIG_HOME", file_dir);
        if let Some(env) = env {
            command.env("LEAFLET_AUTH_TOKEN", env);
        }
        if let Some(flag) = flag {
            command.args(["--token", flag]);
        }
        // An invalid environment URL must not override the explicit URL.
        command.env("LEAFLET_BASE_URL", "invalid environment URL");
        let output = command.args(["remote", "list", "--json"]).output().await.unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(!String::from_utf8_lossy(&output.stdout).contains(expected));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(expected));
        assert_eq!(
            server.requests().last().unwrap().header("cookie"),
            Some(format!("auth_token={expected}").as_str())
        );
    }
    // URL fallback really works when the flag is absent; no user env is used.
    let output = Command::new(env!("CARGO_BIN_EXE_leafbud"))
        .env_remove("LEAFLET_AUTH_TOKEN")
        .env("LEAFLET_BASE_URL", &server.base_url)
        .env("XDG_CONFIG_HOME", file_dir)
        .args(["remote", "list", "--json"])
        .output()
        .await
        .unwrap();
    assert!(output.status.success());
    let output = cli(&server.base_url)
        .env("LEAFLET_AUTH_TOKEN", " ")
        .env("XDG_CONFIG_HOME", file_dir)
        .args(["remote", "list"])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("empty"));
}

#[tokio::test]
async fn anonymous_rpc_and_locator_errors_never_emit_success_json() {
    for (body, args, expected) in [
        ("<html>anonymous</html>", vec!["remote", "list", "--json"], "anonymous"),
        (
            r#"{"result":{"error":{"code":"PGRST116","message":"reflected-secret"},"status":406,"data":null}}"#,
            vec!["remote", "show", ID, "--json"],
            "not found",
        ),
        ("not JSON", vec!["remote", "show", ID, "--json"], "invalid JSON"),
    ] {
        let server = TestServer::spawn(Arc::new(move |_| (200, body.into()))).await;
        let output = cli(&server.base_url).args(args).output().await.unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(expected), "{error}");
        assert!(!error.contains("reflected-secret"));
    }
    let server = fixture_server().await;
    let output = cli(&server.base_url)
        .args(["remote", "show", "at://did:plc:x/site.standard.document/one", "--json"])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("AT-URI resolution is not supported"));
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn uncommissioned_commands_remain_unimplemented_and_usage_is_two() {
    let server = fixture_server().await;
    for command in ["publish", "export"] {
        let output = cli(&server.base_url).args([command, "doc.mdx"]).output().await.unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("'{command}' is not implemented")));
    }
    let output = cli(&server.base_url)
        .args(["remote", "list", "--state", "in_publication"])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn restored_config_cli_preserves_settings_and_routes_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let xdg = temp.path().join("config");
    let global = xdg.join("leafbud/leafbud.toml");
    std::fs::create_dir_all(global.parent().unwrap()).unwrap();
    let ws = temp.path().join("workspace");
    std::fs::create_dir(&ws).unwrap();
    std::fs::write(&global, "# retained\npublication = 'global-pub'\nunknown-secret = 'synthetic-do-not-show'\n").unwrap();
    let command = || {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_leafbud"));
        cmd.env("XDG_CONFIG_HOME", &xdg).env_remove("LEAFLET_AUTH_TOKEN");
        cmd
    };
    let output = command().args(["config", "set", "workspace", ws.to_str().unwrap()]).output().await.unwrap();
    assert!(output.status.success());
    std::fs::write(ws.join("leafbud.toml"), "publication = 'workspace-pub' # publication\n").unwrap();
    let output = command().args(["config", "show", "--json"]).output().await.unwrap();
    assert!(output.status.success());
    let shown: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(shown["workspaceRoot"], ws.to_str().unwrap());
    let publication = shown["values"].as_array().unwrap().iter().find(|v| v["key"] == "publication").unwrap();
    assert_eq!(publication["value"], "workspace-pub");
    assert_eq!(publication["source"], "workspace");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("synthetic-do-not-show"));
    let output = command()
        .args(["--publication", "flag-pub", "config", "show", "--json"])
        .output()
        .await
        .unwrap();
    let shown: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(shown["values"][3]["value"], "flag-pub");
    let output = command()
        .args(["config", "set", "publication", "new-workspace-pub", "--workspace-file"])
        .output()
        .await
        .unwrap();
    assert!(output.status.success());
    assert!(std::fs::read_to_string(ws.join("leafbud.toml")).unwrap().contains("# publication"));
    assert!(std::fs::read_to_string(global).unwrap().contains("publication = 'global-pub'"));
}
