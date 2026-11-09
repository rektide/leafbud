//! CLI lifetime/selection/JSONL proof; remote adoption is covered by workflow's
//! actual production HTTP+Phoenix controller tests. No deployed service used.
#![cfg(unix)]
#[path = "../../atproto/src/testserver.rs"]
#[allow(dead_code)]
mod testserver;
use serde_json::Value;
use std::{path::Path, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

fn cli(root: &Path, base: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_leafbud"));
    command
        .env_remove("LEAFLET_AUTH_TOKEN")
        .env_remove("LEAFLET_BASE_URL")
        .env_remove("LEAFBUD_REALTIME_PROJECT_URL")
        .env_remove("LEAFBUD_REALTIME_ANON_KEY")
        .env("XDG_CONFIG_HOME", root.join(".config"))
        .args(["--base-url", base, "--root"])
        .arg(root)
        .kill_on_drop(true);
    command
}

#[tokio::test]
async fn selection_and_persistent_take_are_usage_errors() {
    let root = tempfile::tempdir().unwrap();
    for args in [
        vec!["watch"],
        vec!["watch", "--all", "x.mdx"],
        vec!["watch", "--all", "--take", "remote"],
        vec!["watch", "--all", "--backstop", "0"],
    ] {
        let result = cli(root.path(), "http://127.0.0.1:1").args(args).output().await.unwrap();
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
    let output = cli(root.path(), "http://127.0.0.1:1")
        .args(["watch", "--all", "--project-url", "http://127.0.0.1:1"])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires both"));
}

#[tokio::test]
async fn jsonl_flushes_document_transitions_and_sigint_emits_final_summary() {
    let root = tempfile::tempdir().unwrap();
    let server = testserver::TestServer::spawn(Arc::new(|_| (404, String::new()))).await;
    let create = cli(root.path(), &server.base_url)
        .args(["create", "draft.mdx", "--title", "Local"])
        .output()
        .await
        .unwrap();
    assert!(create.status.success());
    let mut child = cli(root.path(), &server.base_url)
        .args(["watch", "draft.mdx", "--json", "--rescan", "1"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            let line = lines.next_line().await.unwrap().unwrap();
            let event: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(event["version"], 1);
            assert!(event["observed_at"].is_object());
            if event["type"] == "document" {
                assert_eq!(event["outcome"], "unbound");
                break;
            }
        }
    })
    .await
    .unwrap();
    let signal = Command::new("kill").args(["-INT", &child.id().unwrap().to_string()]).status().await.unwrap();
    assert!(signal.success());
    let mut summary = None;
    tokio::time::timeout(Duration::from_secs(4), async {
        while let Some(line) = lines.next_line().await.unwrap() {
            let event: Value = serde_json::from_str(&line).unwrap();
            if event["type"] == "summary" {
                summary = Some(event);
            }
        }
    })
    .await
    .unwrap();
    assert!(child.wait().await.unwrap().success());
    let summary = summary.unwrap();
    assert_eq!(summary["selected"], 1);
    assert_eq!(summary["writes"], 0);
    assert_eq!(summary["outcomes"]["unbound"], 1);
    assert_eq!(summary["stopped"], true);
}

#[tokio::test]
async fn empty_all_and_missing_explicit_operand_are_honest_and_cancellable() {
    let root = tempfile::tempdir().unwrap();
    let server = testserver::TestServer::spawn(Arc::new(|_| (404, String::new()))).await;
    for args in [vec!["watch", "--all", "--json"], vec!["watch", "missing.mdx", "--json"]] {
        let explicit = args[1] != "--all";
        let mut child = cli(root.path(), &server.base_url)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let first: Value = serde_json::from_str(&tokio::time::timeout(Duration::from_secs(3), lines.next_line()).await.unwrap().unwrap().unwrap()).unwrap();
        assert_eq!(first["type"], "inventory");
        assert_eq!(first["selected"], if explicit { 1 } else { 0 });
        if explicit {
            tokio::time::timeout(Duration::from_secs(3), async {
                loop {
                    let value: Value = serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
                    if value["type"] == "document" {
                        assert_eq!(value["outcome"], "source-lost");
                        break;
                    }
                }
            })
            .await
            .unwrap();
        }
        assert!(
            Command::new("kill")
                .args(["-TERM", &child.id().unwrap().to_string()])
                .status()
                .await
                .unwrap()
                .success()
        );
        assert!(tokio::time::timeout(Duration::from_secs(3), child.wait()).await.unwrap().unwrap().success());
        assert!(!root.path().join("missing.mdx").exists());
    }
}

#[tokio::test]
async fn another_process_obeys_the_same_stable_id_lock_and_release() {
    let root = tempfile::tempdir().unwrap();
    let id = "11111111-1111-4111-8111-111111111111";
    let path = root.path().join("doc.mdx");
    let source = format!("---\nformat: leafbud-mdx/2\ntitle: Locked\n\"@leafbud\": {{\"id\":\"{id}\",\"state\":\"draft\"}}\n---\n\nA new paragraph\n");
    std::fs::write(&path, &source).unwrap();
    let dir = root.path().join(".config/leafbud/locks");
    std::fs::create_dir_all(&dir).unwrap();
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(format!("document-{id}.lock")))
        .unwrap();
    lock.try_lock().unwrap();
    let report = cli(root.path(), "http://127.0.0.1:1")
        .args(["tidy", "doc.mdx", "--keys", "--json"])
        .output()
        .await
        .unwrap();
    assert_eq!(report.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&report.stdout).unwrap();
    assert!(report["items"][0]["error"].as_str().unwrap().contains("busy"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    drop(lock); // OS releases; the remaining .lock file is deliberately retained.
    let report = cli(root.path(), "http://127.0.0.1:1")
        .args(["tidy", "doc.mdx", "--keys", "--json"])
        .output()
        .await
        .unwrap();
    assert!(report.status.success());
    let report: Value = serde_json::from_slice(&report.stdout).unwrap();
    assert_eq!(report["normalized"], 1);
}
