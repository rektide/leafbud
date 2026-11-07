use std::process::Command;

use serde_json::Value;

fn run(mode: &str, fixture: &str) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_satteri-identity-experiment"))
        .args([mode, &format!("{}/fixtures/{fixture}", env!("CARGO_MANIFEST_DIR"))])
        .output()
        .unwrap();
    (output.status.code().unwrap(), serde_json::from_slice(&output.stdout).unwrap())
}

#[test]
fn author_cli_outputs_internal_key_and_semantic_component_data() {
    let (exit, output) = run("author", "comments.mdx");
    assert_eq!(exit, 0);
    assert_eq!(output["diagnostics"].as_array().unwrap().len(), 0);
    assert_eq!(output["tree"]["children"][0]["key"], "introduction");
    assert_eq!(output["tree"]["children"][4]["details"]["literal_props"]["options"][0], "Gardens");
}

#[test]
fn invalid_author_cli_exits_two_with_inspectable_diagnostics() {
    let (exit, output) = run("author", "negative.mdx");
    assert_eq!(exit, 2);
    let codes: Vec<&str> = output["diagnostics"].as_array().unwrap().iter().map(|d| d["code"].as_str().unwrap()).collect();
    assert_eq!(codes.iter().filter(|&&c| c == "duplicate-key").count(), 2);
    for expected in ["unknown-component", "nonliteral-prop", "spread-prop"] {
        assert!(codes.contains(&expected));
    }
}

#[test]
fn cli_retains_counterexample_instead_of_claiming_code_protection() {
    let (exit, plain) = run("plain", "multiline-code.mdx");
    assert_eq!(exit, 0);
    assert_eq!(plain["tree"]["children"][0]["children"][1]["kind"], "InlineCode");
    let (exit, author) = run("author", "multiline-code.mdx");
    assert_eq!(exit, 0, "known failure: no diagnostic for misinterpreted multiline code");
    assert_eq!(author["tree"]["children"][1]["key"], "literal");
}
