use std::process::{Command, Output};

fn run(mode: &str, fixture: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdxjs-turbopack-identity-experiment"))
        .arg(mode)
        .arg(format!("{}/fixtures/{fixture}", env!("CARGO_MANIFEST_DIR")))
        .output()
        .unwrap()
}

#[test]
fn clean_document_emits_ephemeral_structured_data_and_exit_zero() {
    let output = run("check", "document.mdx");
    assert!(output.status.success(), "{:?}", output);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["bindings"].as_array().unwrap().len(), 3);
    assert_eq!(value["diagnostics"], serde_json::json!([]));
}

#[test]
fn stock_vs_adapter_attribute_and_code_trap_exits_are_distinct() {
    assert!(run("stock", "attributes.mdx").status.success());
    for fixture in ["attributes.mdx", "multiline-comment.mdx", "rejected.mdx"] {
        assert_eq!(run("check", fixture).status.code(), Some(2), "{fixture}");
    }
}

#[test]
fn negative_observation_component_code_trap_remains_false_clean() {
    let output = run("check", "multiline-component.mdx");
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["bindings"][0]["key"], "accidental");
}

#[test]
fn key_candidate_is_consumed_but_paired_group_is_not_silently_defined() {
    let output = run("check", "key-candidate.mdx");
    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["bindings"].as_array().unwrap().len(), 5);
    assert_eq!(value["consumed_annotations"].as_array().unwrap().len(), 5);
    assert_eq!(run("check", "key-paired.mdx").status.code(), Some(2));
}
