//! Actual offline commands, isolated config/files; deliberately invalid native
//! credentials/origin prove these commands do not even construct a client.
use std::path::Path;
use tokio::process::Command;

fn cli(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_leafbud"));
    c.env_remove("HOME")
        .env("XDG_CONFIG_HOME", root.join(".config"))
        .env("LEAFLET_AUTH_TOKEN", "invalid\nheader")
        .env("LEAFLET_BASE_URL", "not a URL")
        .arg("--root")
        .arg(root)
        .args(args);
    c
}

async fn run(root: &Path, args: &[&str], code: i32) -> String {
    let out = cli(root, args).output().await.unwrap();
    assert_eq!(
        out.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[tokio::test]
async fn edit_flags_dates_tags_clear_noops_and_keys_are_offline() {
    let root = tempfile::tempdir().unwrap();
    run(root.path(), &["create", "essay.mdx", "--title", "Garden"], 0).await;
    let path = root.path().join("essay.mdx");
    let source = format!("{}\n<Key id='old' />\nold prose\n", std::fs::read_to_string(&path).unwrap());
    std::fs::write(&path, &source).unwrap();
    run(
        root.path(),
        &[
            "edit",
            "essay.mdx",
            "--description",
            "",
            "--title",
            "Garden",
            "--published-at",
            "2026-11-09T10:00:00Z",
            "--tag",
            "b",
            "--tag",
            "a",
        ],
        0,
    )
    .await;
    let after = std::fs::read_to_string(&path).unwrap();
    let header = leafbud_mdx::header(&after).unwrap();
    assert_eq!(header["tags"], serde_json::json!(["b", "a"]));
    assert!(header.get("description").is_none());
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    run(root.path(), &["edit", "essay.mdx", "--title", "Garden", "--tag", "b", "--tag", "a"], 0).await;
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    for args in [
        vec!["edit", "essay.mdx"],
        vec!["edit", "essay.mdx", "--now", "--published-at", "2026-11-09T00:00:00Z"],
        vec!["edit", "essay.mdx", "--tag", "a", "--clear-tags"],
    ] {
        run(root.path(), &args, 2).await;
    }
    for args in [
        vec!["edit", "essay.mdx", "--published-at", "2026-11-09T00:00:00"],
        vec!["keys", "rename", "essay.mdx", "missing", "new"],
    ] {
        run(root.path(), &args, 1).await;
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
    run(root.path(), &["edit", "essay.mdx", "--now", "--clear-tags", "--description", " "], 0).await;
    let now = leafbud_mdx::header(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(now["description"], " ");
    assert_eq!(now["tags"], serde_json::json!([]));
    assert_ne!(now["publishedAt"], header["publishedAt"]);
    run(root.path(), &["keys", "rename", "essay.mdx", "old", "introduction"], 0).await;
    let source = std::fs::read_to_string(&path).unwrap();
    assert!(source.contains("<Key id=\"introduction\" />\nold prose"));
}

#[tokio::test]
async fn list_reads_config_root_without_locks_or_auth_and_reports_partial_failures() {
    let root = tempfile::tempdir().unwrap();
    run(root.path(), &["create", "published-dir/draft.mdx", "--title", "One"], 0).await;
    std::fs::remove_dir_all(root.path().join(".config")).ok();
    let out = run(root.path(), &["list", "--json"], 0).await;
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(report["version"], 1);
    assert_eq!(report["items"][0]["lifecycle"], "draft");
    assert_eq!(report["items"][0]["outcome"], "unbound");
    assert!(!root.path().join(".config").exists());
    let config = root.path().join(".config/leafbud");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("leafbud.toml"), format!("workspace = {:?}\n", root.path().to_str().unwrap())).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_leafbud"))
        .env_remove("HOME")
        .env("XDG_CONFIG_HOME", root.path().join(".config"))
        .args(["list", "--json"])
        .output()
        .await
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["items"][0]["title"], "One");
    assert!(!config.join("locks").exists());
    std::fs::write(root.path().join("broken.mdx"), "---\n\"@leafbud\": [broken\n---\n").unwrap();
    let report: serde_json::Value = serde_json::from_str(&run(root.path(), &["list", "--state", "draft", "--json"], 4).await).unwrap();
    assert_eq!(report["failed"], 1);
    assert_eq!(report["items"].as_array().unwrap().len(), 2);
    let human = run(root.path(), &["list"], 4).await;
    assert!(human.contains("One") && human.contains("broken.mdx"));
}
