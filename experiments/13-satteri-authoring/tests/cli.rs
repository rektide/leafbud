use std::process::Command;

#[test]
fn real_check_executable_emits_facts_decoded_text_and_no_shadow_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_satteri-authoring-experiment"))
        .args(["check", "fixtures/garden.mdx"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["document"]["metadata"]["title"], "A garden notebook");
    assert!(!value["plan"]["facts"].as_array().unwrap().is_empty());
    assert!(!value["decoded"].as_array().unwrap().is_empty());
    assert!(!std::path::Path::new("fixtures/garden.json").exists());
}

#[test]
fn rejected_component_produces_exit_two_no_plan_or_partial_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_satteri-authoring-experiment"))
        .args(["check", "fixtures/rejected.mdx"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let diagnostics: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(diagnostics["diagnostics"][0]["code"], "unimplemented-component");
}

#[test]
fn actual_offline_demo_checks_recreate_repeat_and_stable_binding() {
    let output = Command::new(env!("CARGO_BIN_EXE_satteri-authoring-experiment"))
        .args(["demo", "fixtures/garden.mdx"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    for name in [
        "offlineSimulation",
        "interruptedStageKeptOldEntry",
        "repeatDidNotAppend",
        "editedSourceProducedNewBody",
        "stablePollReused",
        "sourceUnchanged",
    ] {
        assert_eq!(value[name], true, "{name}");
    }
}
