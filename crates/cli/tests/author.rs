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
async fn offline_naming_directions_preview_selectors_status_and_noop() {
    use serde_json::Value;
    let root = tempfile::tempdir().unwrap();
    run(root.path(), &["create", "draft.mdx", "--title", "Naming"], 0).await;
    let file = root.path().join("draft.mdx");
    let source = std::fs::read_to_string(&file)
        .unwrap()
        .replacen("\"@leafbud\":", "path: /notes/Éssay\nunknown: {keep: 'exact bytes'}\n\"@leafbud\":", 1);
    let source = format!(
        "\u{feff}{}",
        format!("{source}\n{{/* preserved comment */}}\nNew unkeyed prose.\n").replace('\n', "\r\n")
    );
    std::fs::write(&file, &source).unwrap();
    std::fs::create_dir(root.path().join("notes")).unwrap();
    let parsed = leafbud_mdx::parse(&source).unwrap();
    let r: Value = serde_json::from_str(&run(root.path(), &["status", "draft.mdx", "--offline", "--json"], 0).await).unwrap();
    assert_eq!(r["version"], 3);
    assert_eq!(r["items"][0]["names"]["disposition"], "untracked");
    run(root.path(), &["tidy", "draft.mdx", "--json"], 3).await;
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    let preview: Value = serde_json::from_str(&run(root.path(), &["tidy", "draft.mdx", "--keys", "--from", "path", "--dry-run", "--json"], 0).await).unwrap();
    assert_eq!(preview["version"], 2);
    assert_eq!(preview["items"][0]["keys_assigned"], 2);
    assert_eq!(preview["items"][0]["proposed_names"]["file"], "notes/Éssay.mdx");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    assert!(!root.path().join("notes/Éssay.mdx").exists());
    run(root.path(), &["tidy", "draft.mdx", "--keys", "--from", "path", "--json"], 0).await;
    let moved = root.path().join("notes/Éssay.mdx");
    assert!(!file.exists());
    let text = std::fs::read_to_string(&moved).unwrap();
    assert!(text.starts_with("\u{feff}---\r\n"));
    assert!(text.contains("{/* preserved comment */}"));
    assert!(text.contains("unknown: {keep: 'exact bytes'}"));
    let after = leafbud_mdx::parse(&text).unwrap();
    assert_eq!(after.state.id, parsed.state.id);
    assert!(after.state.binding.is_none());
    let modified = std::fs::metadata(&moved).unwrap().modified().unwrap();
    run(root.path(), &["tidy", "notes/Éssay.mdx", "--from", "path", "--json"], 0).await;
    assert_eq!(std::fs::read_to_string(&moved).unwrap(), text);
    assert_eq!(std::fs::metadata(&moved).unwrap().modified().unwrap(), modified);
    let human = run(root.path(), &["status", "notes/Éssay.mdx", "--offline"], 0).await;
    assert!(human.contains("names: equal"), "{human}");
    // Move outside the accepted pair. Read-only list reports, never settles.
    let current = root.path().join("notes/current.mdx");
    std::fs::rename(&moved, &current).unwrap();
    let listed: Value = serde_json::from_str(&run(root.path(), &["list", "--json"], 0).await).unwrap();
    assert_eq!(listed["items"][0]["names"]["disposition"], "file-changed");
    run(root.path(), &["tidy", "notes/current.mdx", "--from", "filename", "--json"], 0).await;
    let source = std::fs::read_to_string(&current).unwrap();
    assert_eq!(leafbud_mdx::header(&source).unwrap()["path"], "/notes/current");
    // --accept witnesses an explicit current mismatch without patching author path.
    std::fs::rename(&current, &file).unwrap();
    run(root.path(), &["tidy", "draft.mdx", "--accept", "--json"], 0).await;
    let accepted = std::fs::read_to_string(&file).unwrap();
    assert_eq!(leafbud_mdx::header(&accepted).unwrap()["path"], "/notes/current");
    let report: Value = serde_json::from_str(&run(root.path(), &["status", "--offline", "--json"], 0).await).unwrap();
    assert_eq!(report["items"][0]["names"]["disposition"], "accepted-mismatch");
    run(root.path(), &["tidy", "--all", "--markdown", "--json"], 0).await;
    assert_eq!(std::fs::read_to_string(&file).unwrap(), accepted);
}

#[tokio::test]
async fn list_reads_config_root_without_locks_or_auth_and_reports_partial_failures() {
    let root = tempfile::tempdir().unwrap();
    run(root.path(), &["create", "published-dir/draft.mdx", "--title", "One"], 0).await;
    std::fs::remove_dir_all(root.path().join(".config")).ok();
    let out = run(root.path(), &["list", "--json"], 0).await;
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(report["version"], 2);
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
