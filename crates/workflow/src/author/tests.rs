use super::*;

fn fixture() -> (tempfile::TempDir, config::Context, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let context = config::Context {
        workspace_root: root.path().into(),
        global_path: root.path().join("config/leafbud.toml"),
        ..config::load(Some(root.path()), &config::Config::default(), None).unwrap()
    };
    let path = root.path().join("one.mdx");
    std::fs::write(&path, "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"main\"}\n---\n\n<Key id='old' />\ntext\n").unwrap();
    (root, context, path)
}

#[test]
fn noops_do_not_replace_and_shared_lock_prevents_edit_and_rename() {
    let (_root, context, path) = fixture();
    let before = std::fs::read(&path).unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    assert!(
        !edit(
            &context,
            &path,
            &MetadataEdit {
                title: Some("T".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .changed
    );
    assert!(!rename(&context, &path, "old", "old").unwrap().changed);
    assert_eq!(modified, std::fs::metadata(&path).unwrap().modified().unwrap());
    let store = lock::Store::new(&context).unwrap();
    let held = store.document(uuid::Uuid::nil()).unwrap().unwrap();
    assert!(
        edit(
            &context,
            &path,
            &MetadataEdit {
                title: Some("new".into()),
                ..Default::default()
            }
        )
        .unwrap_err()
        .to_string()
        .contains("busy")
    );
    assert!(rename(&context, &path, "old", "new").is_err());
    assert_eq!(before, std::fs::read(&path).unwrap());
    drop(held);
    assert!(rename(&context, &path, "old", "new").unwrap().changed);
    assert!(
        edit(
            &context,
            &path,
            &MetadataEdit {
                title: Some("new".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .changed
    );
}

#[test]
fn source_changes_and_late_identity_duplicates_are_not_overwritten() {
    let (root, context, path) = fixture();
    let changed = std::fs::read_to_string(&path).unwrap().replace("text", "human changed");
    let result = apply(&context, &path, |source| {
        let proposed = leafbud_mdx::rename_key(source, "old", "new")?;
        std::fs::write(&path, &changed)?;
        Ok(proposed)
    });
    assert!(result.is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), changed);
    let result = apply(&context, &path, |source| {
        let proposed = leafbud_mdx::rename_key(source, "old", "new")?;
        std::fs::write(root.path().join("copy.mdx"), source)?;
        Ok(proposed)
    });
    assert!(result.unwrap_err().to_string().contains("duplicate"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), changed);
    assert!(edit(&context, &path, &MetadataEdit::default()).is_err());
}

#[test]
fn external_operands_and_symlink_aliases_obey_identity_guards() {
    let (_root, context, path) = fixture();
    let external = tempfile::tempdir().unwrap();
    let copy = external.path().join("copy.mdx");
    std::fs::copy(&path, &copy).unwrap();
    assert!(rename(&context, &copy, "old", "new").unwrap_err().to_string().contains("duplicate"));
    #[cfg(unix)]
    {
        let link = external.path().join("alias.mdx");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(rename(&context, &link, "old", "new").unwrap().changed);
        assert!(std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    }
}
