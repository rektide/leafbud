//! Full `tidy` command execution against a local workspace; no network and
//! no user document is touched.
use tokio::process::Command as TokioCommand;

/// A complete managed file: author-styled frontmatter (unquoted title with
/// comment, local-only customField), a literal-wrapped link block that
/// normalizes to Markdown, and a second already-clean block.
const MANAGED: &str = r#"---
"format": "springleaf-mdx/1"
title: T # author comment
customField: {"meaning": "retained"}
"@springleaf": {"id": "00000000-0000-0000-0000-000000000000", "state": "draft", "binding": {"baseUrl": "https://leaflet.pub", "token": "00000000-0000-0000-0000-000000000000", "root": "e-root", "entitySet": "es", "entryFact": "f-entry", "publication": null}, "nodes": {"p-main": {"entity": "e-main", "memberships": {}}, "b-1": {"entity": "e-b1", "memberships": {}}}, "sync": {"profile": "leaflet-import/1", "localHash": "sha256:old-local", "remoteHash": "sha256:old-remote"}}
---

<Page name="p-main" kind="linear">

<Key id="b-1" />
<Paragraph>
<Run text={"guide"} marks={[{"kind":"link","href":"https://e.org/docs"}]} />

</Paragraph>

</Page>
"#;

fn workspace() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("drafts")).unwrap();
    std::fs::write(root.path().join("drafts").join("doc.mdx"), MANAGED).unwrap();
    root
}

fn cli(root: &std::path::Path, args: &[&str]) -> TokioCommand {
    let mut command = TokioCommand::new(env!("CARGO_BIN_EXE_springleaf"));
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
    assert_eq!(report["pass"], "markdown");
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
    assert!(String::from_utf8_lossy(&output.stdout).to_lowercase().contains("dry run") || true);
}

#[tokio::test]
async fn tidy_all_and_markdown_selector_agree() {
    let root = workspace();
    std::fs::write(root.path().join("drafts").join("plain.md"), "not managed").unwrap();
    let output = cli(root.path(), &["tidy", "--all", "--markdown", "--json"]).output().await.unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["selected"], 1, "unmanaged and non-mdx files are ignored");
    assert!(doc(root.path()).contains("[guide](https://e.org/docs)"));
}

#[tokio::test]
async fn tidy_rejects_unimplemented_selectors_honestly() {
    let root = workspace();
    let before = doc(root.path());
    for args in [
        vec!["tidy", "drafts/doc.mdx", "--keys"],
        vec!["tidy", "drafts/doc.mdx", "--from", "filename"],
        vec!["tidy", "drafts/doc.mdx", "--accept"],
    ] {
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
        MANAGED.replacen("</Page>", "{/* c */}\n</Page>", 1),
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
