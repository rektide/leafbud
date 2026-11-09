use super::*;
use crate::pull::tests::*;
use serde_json::json;

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";

fn unchanged(path: &Path) -> (Vec<u8>, std::time::SystemTime) {
    (std::fs::read(path).unwrap(), std::fs::metadata(path).unwrap().modified().unwrap())
}

#[tokio::test]
async fn current_views_are_symmetric_and_never_settle_receipts_even_when_converged() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let reader = reader(&server);
    let metadata_calls = server.request_count("/api/rpc/get_leaflet_data");
    let content_calls = server.request_count("/api/rpc/pull");
    let source = std::fs::read_to_string(&path).unwrap().replacen(
        "\n\"@leafbud\":",
        "\ncustomSecret: 'local-only, never show'\npublishedAt: '2030-01-01'\n\"@leafbud\":",
        1,
    );
    std::fs::write(&path, &source).unwrap();
    let before = unchanged(&path);
    let (local, editor) = reader.comparison_views(&context(temp.path()), &path).await.unwrap();
    assert_eq!(local, editor);
    assert!(local.contains("Synthetic body copy"));
    assert!(local.contains("<Page name=\"page-1\""));
    for absent in [TOKEN, "sha256:", "@leafbud", "customSecret", "local-only, never show", "publishedAt", "<Key"] {
        assert!(!local.contains(absent), "{absent}");
    }
    assert_eq!(unchanged(&path), before);

    let source = source.replace("Synthetic body copy", "Current local paragraph");
    std::fs::write(&path, &source).unwrap();
    let before = unchanged(&path);
    set_paragraph_text(
        current.write().unwrap().get_mut(TOKEN).unwrap(),
        &yrs_payload("Current remote paragraph", Some((0, 7))),
    );
    let (local, editor) = reader.comparison_views(&context(temp.path()), &path).await.unwrap();
    assert!(local.contains("Current local paragraph"));
    assert!(!local.contains("Current remote paragraph"));
    assert!(editor.contains("**Current** remote paragraph"), "{editor}");
    assert_ne!(local, editor);
    set_paragraph_text(current.write().unwrap().get_mut(TOKEN).unwrap(), &yrs_payload("Current local paragraph", None));
    let (local, editor) = reader.comparison_views(&context(temp.path()), &path).await.unwrap();
    assert_eq!(local, editor);
    assert_eq!(unchanged(&path), before);
    assert_eq!(server.request_count("/api/rpc/get_leaflet_data"), metadata_calls + 3);
    assert_eq!(server.request_count("/api/rpc/pull"), content_calls + 3);
}

#[tokio::test]
async fn unsupported_missing_and_ambiguous_inputs_do_not_fabricate_views_or_leak_content() {
    let (server, current) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let original = unchanged(&path);
    let reader = reader(&server);
    let ctx = context(temp.path());
    let duplicate = temp.path().join("copy.mdx");
    std::fs::copy(&path, &duplicate).unwrap();
    let calls = server.request_count("/api/rpc/get_leaflet_data");
    assert!(reader.comparison_views(&ctx, &path).await.unwrap_err().to_string().contains("duplicate"));
    std::fs::remove_file(duplicate).unwrap();
    for (from, to, expected) in [
        ("@leafbud", "@springleaf", "conversion"),
        ("leaflet-editor/2", "future/99", "unsupported local"),
        ("leafbud-mdx/1", "unknown-mdx/1", "unsupported local"),
    ] {
        let changed = String::from_utf8(original.0.clone()).unwrap().replace(from, to);
        std::fs::write(&path, &changed).unwrap();
        assert!(reader.comparison_views(&ctx, &path).await.unwrap_err().to_string().contains(expected));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), changed);
    }
    assert_eq!(server.request_count("/api/rpc/get_leaflet_data"), calls);
    std::fs::write(&path, &original.0).unwrap();
    current.write().unwrap().remove(TOKEN);
    assert!(
        reader
            .comparison_views(&ctx, &path)
            .await
            .unwrap_err()
            .to_string()
            .contains("missing or inaccessible")
    );
    let mut changed = fixture();
    changed["response"]["patch"].as_array_mut().unwrap().push(json!({"op":"put","key":"11111111-1111-4111-8111-111111111111","value":{
        "id":"11111111-1111-4111-8111-111111111111","entity":"root","attribute":"future/opaque","data":{"type":"private","value":"never print remote content"}
    }}));
    current.write().unwrap().insert(TOKEN.into(), changed);
    let error = reader.comparison_views(&ctx, &path).await.unwrap_err().to_string();
    assert!(error.contains("unsupported"));
    assert!(!error.contains("never print"));
    let mut changed = fixture();
    changed["leaflet_data"]["permission_token_rights"][0]["entity_set"] = json!("other");
    current.write().unwrap().insert(TOKEN.into(), changed);
    assert!(reader.comparison_views(&ctx, &path).await.unwrap_err().to_string().contains("binding"));
    let mut changed = fixture();
    changed["leaflet_data"]["leaflets_to_documents"] = json!([{"document":"at://did:plc:test/site.standard.document/test"}]);
    current.write().unwrap().insert(TOKEN.into(), changed);
    assert!(reader.comparison_views(&ctx, &path).await.unwrap_err().to_string().contains("unsupported"));
    assert_eq!(std::fs::read(&path).unwrap(), original.0);
}

#[tokio::test]
async fn foreign_origin_and_external_duplicate_are_refused_before_fetch() {
    let (server, _) = server_current().await;
    let temp = tempfile::tempdir().unwrap();
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    let (other, _) = server_current().await;
    let error = reader(&other).comparison_views(&context(temp.path()), &path).await.unwrap_err();
    assert!(error.to_string().contains("another origin"));
    assert_eq!(other.request_count("/api/rpc/get_leaflet_data"), 0);
    let external = tempfile::tempdir().unwrap();
    let copy = external.path().join("copy.mdx");
    std::fs::copy(path, &copy).unwrap();
    let error = reader(&server).comparison_views(&context(temp.path()), &copy).await.unwrap_err();
    assert!(error.to_string().contains("duplicate"));
}

#[tokio::test]
async fn racing_local_save_is_left_alone_and_not_presented_as_current() {
    use crate::import::tests::testserver;
    use std::sync::{Arc, RwLock};
    let temp = tempfile::tempdir().unwrap();
    let target = Arc::new(RwLock::new(None::<std::path::PathBuf>));
    let handler_target = target.clone();
    let fixture = fixture();
    let server = testserver::TestServer::spawn(Arc::new(move |request| match request.path.as_str() {
        "/api/rpc/get_leaflet_data" => (200, json!({"result":{"error":null,"data":fixture["leaflet_data"]}}).to_string()),
        "/api/rpc/pull" => {
            if let Some(path) = handler_target.read().unwrap().as_ref() {
                std::fs::write(path, "user save during comparison").unwrap();
            }
            (200, fixture["response"].to_string())
        }
        _ => (404, String::new()),
    }))
    .await;
    let path = seed(&server, temp.path(), TOKEN, "doc.mdx").await;
    *target.write().unwrap() = Some(path.clone());
    let error = reader(&server).comparison_views(&context(temp.path()), &path).await.unwrap_err();
    assert!(error.to_string().contains("changed during observation"));
    assert_eq!(std::fs::read_to_string(path).unwrap(), "user save during comparison");
}

#[cfg(unix)]
#[tokio::test]
async fn direct_argv_paths_labels_exit_modes_stdio_and_cleanup() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::Builder::new().prefix("viewer root with spaces ").tempdir().unwrap();
    let program = temp.path().join("viewer with spaces");
    let record = temp.path().join("argv record");
    std::fs::write(
        &program,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$1\"\ncat \"$4\" > \"$1.local\"\ncat \"$5\" > \"$1.editor\"\npwd > \"$1.cwd\"\nexit \"$3\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut tool = Tool {
        program: program.to_str().unwrap().into(),
        args: vec![record.to_str().unwrap().into(), "literal spaces; $(no shell) * --flag".into(), "0".into()],
        exit: ExitMode::Success,
    };
    for (exit, mode, ok) in [
        (0, ExitMode::Success, true),
        (1, ExitMode::Success, false),
        (1, ExitMode::Diff, true),
        (2, ExitMode::Diff, false),
    ] {
        tool.args[2] = exit.to_string();
        tool.exit = mode;
        let result = run_tool(temp.path(), &tool, "LOCAL writing\n", "EDITOR writing\n").await;
        assert_eq!(result.is_ok(), ok);
        if let Ok(report) = result {
            assert!(!report.views_equal);
            assert_eq!(report.tool_exit, exit);
        }
        let recorded = std::fs::read_to_string(&record).unwrap();
        let args: Vec<_> = recorded.lines().collect();
        assert_eq!(args[1], "literal spaces; $(no shell) * --flag");
        assert_eq!(Path::new(args[3]).file_name().unwrap(), "local.md");
        assert_eq!(Path::new(args[4]).file_name().unwrap(), "editor.md");
        assert!(
            !Path::new(args[3]).parent().unwrap().exists(),
            "views removed after every normal/tool-error exit"
        );
        assert_eq!(std::fs::read_to_string(record.with_file_name("argv record.local")).unwrap(), "LOCAL writing\n");
        assert_eq!(
            std::fs::read_to_string(record.with_file_name("argv record.editor")).unwrap(),
            "EDITOR writing\n"
        );
        assert_eq!(
            std::fs::read_to_string(record.with_file_name("argv record.cwd")).unwrap().trim(),
            temp.path().to_str().unwrap()
        );
    }
    tool.program = temp.path().join("nonexistent program").to_str().unwrap().into();
    assert!(
        run_tool(temp.path(), &tool, "same", "same")
            .await
            .unwrap_err()
            .to_string()
            .contains("starting/waiting")
    );
    tool.program = program.to_str().unwrap().into();
    tool.args[2] = "0".into();
    assert!(run_tool(temp.path(), &tool, "same", "same").await.unwrap().views_equal);
    std::fs::write(&program, "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$1\"\nkill -TERM $$\n").unwrap();
    assert!(run_tool(temp.path(), &tool, "same", "same").await.unwrap_err().to_string().contains("signal"));
    let recorded = std::fs::read_to_string(&record).unwrap();
    assert!(!Path::new(recorded.lines().nth(3).unwrap()).parent().unwrap().exists());
}
