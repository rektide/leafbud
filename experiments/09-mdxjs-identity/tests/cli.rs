use serde_json::Value;
use std::process::{Command, Output};

fn run(mode: &str, fixture: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdxjs-identity-experiment"))
        .args([mode, &format!("{}/fixtures/{fixture}", env!("CARGO_MANIFEST_DIR"))])
        .output()
        .unwrap()
}

#[test]
fn clean_comment_projection_is_transient_stdout() {
    let output = run("check", "comments.mdx");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["tree"]["kind"], "Root");
    assert_eq!(json["tree"]["children"][0]["key"], "introduction");
    assert_eq!(json["diagnostics"], serde_json::json!([]));
    assert!(json.get("original_source").is_none());
}

#[test]
fn stock_accepts_attribute_example_but_selected_dialect_exits_two() {
    let stock = run("stock", "attribute-conflict.mdx");
    assert!(stock.status.success());
    let output = run("check", "attribute-conflict.mdx");
    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["diagnostics"].as_array().unwrap().len(), 2);
}

#[test]
fn rich_components_pass_and_invalid_data_exits_two() {
    assert!(run("check", "components.mdx").status.success());
    let output = run("check", "rejected.mdx");
    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["diagnostics"].as_array().unwrap().len(), 5);
}
