use std::process::Command;

#[test]
fn compare_executable_proves_real_fixture_equivalence_and_preserves_file() {
    let path = "fixtures/garden-attributes.mdx";
    let before = std::fs::read(path).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_satteri-authoring-attributes-experiment"))
        .args(["compare", path, "../13-satteri-authoring/fixtures/garden.mdx"])
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    for flag in [
        "equivalentTypedDocument",
        "equivalentLeafletPlan",
        "stagingAndFreshTextVerified",
        "sourceUnchanged",
    ] {
        assert_eq!(value[flag], true, "{flag}");
    }
    assert_eq!(value["annotations"].as_array().unwrap().len(), 5);
    assert_eq!(value["facts"], 56);
    assert_eq!(value["freshTextPayloads"], 9);
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert!(!std::path::Path::new("fixtures/garden-attributes.json").exists());
}

#[test]
fn invalid_tail_exits_two_with_original_source_span_and_no_partial_plan() {
    let source = std::fs::read_to_string("fixtures/rejected.mdx").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_satteri-authoring-attributes-experiment"))
        .args(["check", "fixtures/rejected.mdx"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    let d = &value["diagnostics"][0];
    assert_eq!(d["code"], "orphan-attribute");
    assert_eq!(
        &source[d["span"]["start"].as_u64().unwrap() as usize..d["span"]["end"].as_u64().unwrap() as usize],
        "{#orphan}"
    );
}
