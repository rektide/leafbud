use std::{path::Path, process::Command};

fn run(mode: &str, file: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pulldown-identity-experiment"))
        .args([mode, file])
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
        .output()
        .unwrap()
}

#[test]
fn valid_cli_emits_transient_json_and_original_source() {
    let output = run("attributes", "fixtures/attributes.md");
    assert!(output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["original_source"], include_str!("../fixtures/attributes.md"));
    assert_eq!(result["blocks"][0]["key"], "introduction");
    assert!(result["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn invalid_cli_exits_two_and_does_not_hide_diagnostics() {
    let output = run("attributes", "fixtures/invalid.md");
    assert_eq!(output.status.code(), Some(2));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["diagnostics"].as_array().unwrap().len(), 4);
}

#[test]
fn strict_component_control_exits_two_plain_control_succeeds() {
    assert!(run("plain", "fixtures/components.mdx").status.success());
    assert_eq!(run("strict", "fixtures/components.mdx").status.code(), Some(2));
}
