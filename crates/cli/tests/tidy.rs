//! Full `tidy` command execution against a local workspace; no network and
//! no user document is touched.
use tokio::process::Command as TokioCommand;

/// A complete managed file: author-styled frontmatter (unquoted title with
/// comment, local-only customField), a literal-wrapped link block that
/// normalizes to Markdown, and a second already-clean block.
const MANAGED: &str = r#"---
"format": "leafbud-mdx/2"
title: T # author comment
customField: {"meaning": "retained"}
"@leafbud": {"id": "00000000-0000-0000-0000-000000000000", "state": "draft", "mainPage": "p-main", "binding": {"baseUrl": "https://leaflet.pub", "token": "00000000-0000-0000-0000-000000000000", "root": "e-root", "entitySet": "es", "entryFact": "f-entry", "publication": null}, "nodes": {"p-main": {"entity": "e-main", "kind": "page", "references": ["f-entry"]}, "b-1": {"entity": "e-b1", "kind": "block", "placement": "f-b1"}}, "sync": {"profile": "leaflet-editor/3", "localHash": "sha256:old-local", "remoteHash": "sha256:old-remote"}}
---

<Key id="b-1" />
<Paragraph>
<Run text={"guide"} marks={[{"kind":"link","href":"https://e.org/docs"}]} />

</Paragraph>
"#;

fn workspace() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("drafts")).unwrap();
    std::fs::write(root.path().join("drafts").join("doc.mdx"), MANAGED).unwrap();
    root
}

fn cli(root: &std::path::Path, args: &[&str]) -> TokioCommand {
    let mut command = TokioCommand::new(env!("CARGO_BIN_EXE_leafbud"));
    command
        .env_remove("LEAFLET_AUTH_TOKEN")
        .env_remove("LEAFLET_BASE_URL")
        .env_remove("HOME")
        .env("XDG_CONFIG_HOME", concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/absent"))
        .arg("--root")
        .arg(root)
        .args(args);
    command
}

fn doc(root: &std::path::Path) -> String {
    String::from_utf8(std::fs::read(root.join("drafts/doc.mdx")).unwrap()).unwrap()
}

#[tokio::test]
async fn tidy_normalizes_and_reports_json() {
    let root = workspace();
    let output = cli(root.path(), &["tidy", "drafts/doc.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["pass"], "all");
    assert_eq!(report["normalized"], 1);
    assert_eq!(report["items"][0]["blocks_changed"], 1);
    let after = doc(root.path());
    assert!(after.contains("[guide](https://e.org/docs)"));
    assert!(after.contains("title: T # author comment"));
    assert!(after.contains("customField"));
    assert!(after.contains("sha256:old-local"));
    // Second run is a no-op.
    let output = cli(root.path(), &["tidy", "drafts/doc.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["unchanged"], 1);
    assert_eq!(report["normalized"], 0);
}

#[tokio::test]
async fn tidy_dry_run_writes_nothing() {
    let root = workspace();
    let before = doc(root.path());
    let output = cli(root.path(), &["tidy", "drafts/doc.mdx", "--dry-run", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["dry_run"], true);
    assert_eq!(report["normalized"], 1);
    assert_eq!(doc(root.path()), before, "dry run writes nothing");
}

#[tokio::test]
async fn tidy_all_and_markdown_selector_agree() {
    let root = workspace();
    std::fs::write(root.path().join("drafts").join("plain.md"), "not managed").unwrap();
    let output = cli(root.path(), &["tidy", "--all", "--markdown", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["pass"], "markdown");
    assert_eq!(report["selected"], 1, "unmanaged and non-mdx files are ignored");
    assert!(doc(root.path()).contains("[guide](https://e.org/docs)"));
}

#[tokio::test]
async fn tidy_keys_assigns_and_persists_main_page_and_idempotence() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("drafts")).unwrap();
    std::fs::write(
        root.path().join("drafts").join("fresh.mdx"),
        "---\nformat: leafbud-mdx/2\ntitle: Fresh\n\"@leafbud\": {\"id\": \"11111111-1111-4111-8111-111111111111\", \"state\": \"draft\"}\n---\n\n# Intro\n\nfirst paragraph\n\n- item one\n- item two\n",
    )
    .unwrap();
    let output = cli(root.path(), &["tidy", "drafts/fresh.mdx", "--keys", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["pass"], "keys");
    assert_eq!(report["items"][0]["keys_assigned"], 5, "four blocks plus the main page");
    let after = String::from_utf8(std::fs::read(root.path().join("drafts/fresh.mdx")).unwrap()).unwrap();
    assert!(after.contains("<Key id=\"b-"), "block keys are durable markers");
    assert!(after.contains("\"mainPage\": \"p-"), "main-page identity persisted");
    assert!(after.contains("- item one\n<Key id=\"b-"), "multi-item lists split with per-item markers");
    assert!(!after.contains("<Paragraph>"), "keys pass never normalizes Markdown");
    // Repeat is an exact no-op.
    let output = cli(root.path(), &["tidy", "drafts/fresh.mdx", "--keys", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["unchanged"], 1);
    assert_eq!(report["items"][0]["keys_assigned"], 0);
    // The default pass would also normalize; --keys alone must not have.
    assert!(after.contains("# Intro"));
}

#[tokio::test]
async fn create_builds_checkable_drafts_offline() {
    let root = tempfile::tempdir().unwrap();
    let output = cli(root.path(), &["create", "drafts/essay.mdx", "--title", "Garden notes", "--standalone"])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("created"), "{stdout}");
    assert!(stdout.contains("standalone"));
    let path = root.path().join("drafts/essay.mdx");
    let source = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert!(source.contains("\"format\": \"leafbud-mdx/2\""));
    assert!(source.contains("\"title\": \"Garden notes\""));
    assert!(source.contains("\"publication\": null"));
    assert!(source.contains("\"state\": \"draft\""));
    assert!(!source.contains("\"binding\""));
    // No-clobber: a second create fails without touching the file.
    let before = std::fs::read(&path).unwrap();
    let output = cli(root.path(), &["create", "drafts/essay.mdx", "--title", "Other"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    // The fresh draft checks offline and survives authored edits + tidy.
    std::fs::write(&path, format!("{source}# First observations\n\nThe seedlings are growing.\n")).unwrap();
    let output = cli(root.path(), &["check", "drafts/essay.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3 objects need key preparation"), "{stdout}");
    let output = cli(root.path(), &["status", "drafts/essay.mdx", "--offline"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unbound"), "{stdout}");
    assert!(stdout.contains("need keys"), "{stdout}");
    let output = cli(root.path(), &["tidy", "drafts/essay.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let output = cli(root.path(), &["check", "drafts/essay.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("need key preparation"),
        "all identities assigned"
    );
}

#[tokio::test]
async fn tidy_rejects_unimplemented_selectors_honestly() {
    let root = workspace();
    let before = doc(root.path());
    for args in [vec!["tidy", "drafts/doc.mdx", "--from", "filename"], vec!["tidy", "drafts/doc.mdx", "--accept"]] {
        let output = cli(root.path(), &args).output().await.unwrap();
        assert_eq!(output.status.code(), Some(1), "{args:?} must fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("not implemented"), "{args:?}");
    }
    assert_eq!(doc(root.path()), before, "rejected selectors never touch the file");
}

#[tokio::test]
async fn tidy_usage_and_failure_exit_codes() {
    let root = workspace();
    let output = cli(root.path(), &["tidy"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(2), "no operands is a usage error");
    let output = cli(root.path(), &["tidy", "drafts/missing.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(1), "a lone missing operand is an operational failure");
    // A broken managed file fails loudly; mixed batches exit 4.
    std::fs::write(
        root.path().join("drafts").join("broken.mdx"),
        MANAGED.replace("customField", "{/* c */}\ncustomField"),
    )
    .unwrap();
    let output = cli(root.path(), &["tidy", "drafts/broken.mdx", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(1));
    let output = cli(root.path(), &["tidy", "--all", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(4), "mixed batch exits 4");
}

#[tokio::test]
async fn tidy_human_output_is_line_per_file() {
    let root = workspace();
    let output = cli(root.path(), &["tidy", "drafts/doc.mdx"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("normalized "));
    assert!(stdout.contains("drafts/doc.mdx"));
    assert!(stdout.contains("1 selected; 1 normalized; 0 unchanged; 0 failed"));
}
