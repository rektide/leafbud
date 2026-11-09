use super::*;

fn fixture(id: &str, state: &str, body: &str) -> String {
    format!("---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {{\"id\": \"{id}\", \"state\": \"{state}\", \"mainPage\": \"main\"}}\n---\n\n{body}\n")
}

#[test]
fn inventory_header_survives_bad_body_unknown_profiles_duplicates_and_filters() {
    let root = tempfile::tempdir().unwrap();
    let context = config::load(Some(root.path()), &config::Config::default(), None).unwrap();
    let id1 = "00000000-0000-0000-0000-000000000000";
    let id2 = "11111111-1111-4111-8111-111111111111";
    let id3 = "22222222-2222-4222-8222-222222222222";
    let files = [
        ("draft.mdx", fixture(id1, "draft", "text")),
        ("published.mdx", fixture(id1, "published", "text")),
        ("broken.mdx", fixture(id2, "draft", "<Future />")),
        (
            "legacy.mdx",
            fixture(id3, "draft", "text")
                .replace("leafbud-mdx/2", "springleaf-mdx/1")
                .replace("@leafbud", "@springleaf"),
        ),
        ("malformed.mdx", "---\n\"@leafbud\": [invalid\n---\n".into()),
        ("unbound.mdx", fixture(id3, "draft", "text")),
        ("unmanaged.mdx", "# Not managed".into()),
    ];
    for (path, bytes) in &files {
        std::fs::write(root.path().join(path), bytes).unwrap();
    }
    let report = inspect(
        &context,
        Request {
            state: Some(LeafletState::Published),
        },
    )
    .unwrap();
    assert_eq!(report.examined, 6);
    assert_eq!(report.ignored_unmanaged, 1);
    assert_eq!(report.filtered, 1);
    assert_eq!(report.failed, 5);
    let draft = report.items.iter().find(|i| i.path.ends_with("draft.mdx")).unwrap();
    assert_eq!(draft.outcome, "ambiguous");
    assert!(draft.duplicate_id);
    let bad = report.items.iter().find(|i| i.path.ends_with("broken.mdx")).unwrap();
    assert_eq!(bad.title.as_deref(), Some("T"));
    assert_eq!(bad.document_id, Some(id2.parse().unwrap()));
    assert!(!bad.valid);
    assert!(!root.path().join(".config").exists());
    for (path, bytes) in files {
        assert_eq!(std::fs::read_to_string(root.path().join(path)).unwrap(), bytes);
    }
}

#[test]
fn bound_listing_is_local_deterministic_follows_file_moves_and_skips_symlinks() {
    let root = tempfile::tempdir().unwrap();
    let context = config::load(Some(root.path()), &config::Config::default(), None).unwrap();
    let source = fixture("11111111-1111-4111-8111-111111111111", "draft", "text").replace("\"mainPage\": \"main\"", "\"mainPage\": \"main\", \"binding\": {\"baseUrl\": \"https://leaflet.pub\", \"token\": \"00000000-0000-0000-0000-000000000000\", \"root\": \"r\", \"entitySet\": \"es\", \"entryFact\": \"entry\", \"publication\": null}");
    let path = root.path().join("b.mdx");
    std::fs::write(&path, &source).unwrap();
    let copy = source.replace("11111111-1111-4111-8111-111111111111", "22222222-2222-4222-8222-222222222222");
    std::fs::write(root.path().join("a.mdx"), &copy).unwrap();
    let report = inspect(&context, Request::default()).unwrap();
    assert_eq!(report.items[0].path.file_name().unwrap(), "a.mdx");
    assert!(report.items.iter().all(|i| i.duplicate_binding && i.valid));
    std::fs::remove_file(root.path().join("a.mdx")).unwrap();
    std::fs::rename(&path, root.path().join("renamed.mdx")).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.path().join("renamed.mdx"), root.path().join("alias.mdx")).unwrap();
        std::os::unix::fs::symlink(root.path(), root.path().join("loop")).unwrap();
    }
    let report = inspect(&context, Request::default()).unwrap();
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].outcome, "bound");
    assert_eq!(report.items[0].binding.as_ref().unwrap().root.as_deref(), Some("r"));
    assert!(report.items[0].path.ends_with("renamed.mdx"));
}
