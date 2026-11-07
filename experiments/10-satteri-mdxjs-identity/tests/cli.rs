use std::{path::Path, process::Command};
fn run(mode: &str, fixture: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_satteri-mdxjs-identity-experiment"))
        .arg(mode)
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(fixture))
        .output()
        .unwrap()
}
#[test]
fn check_and_actual_compiler_success() {
    for mode in ["check", "stock", "staged"] {
        for fixture in ["comments.mdx", "components.mdx"] {
            let output = run(mode, fixture);
            assert!(output.status.success(), "{mode} {fixture}: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
}
#[test]
fn gate_rejects_where_stock_compiler_emits_code() {
    assert!(run("stock", "rejected.mdx").status.success());
    for mode in ["check", "staged"] {
        let output = run(mode, "rejected.mdx");
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stdout).contains("duplicate-key"));
    }
}
#[test]
fn code_hazard_is_a_gate_failure_not_a_parser_repair() {
    assert!(run("stock", "multiline-code.mdx").status.success());
    assert_eq!(run("check", "multiline-code.mdx").status.code(), Some(2));
    assert_eq!(run("staged", "multiline-code.mdx").status.code(), Some(2));
    assert_eq!(run("stock", "attributes.mdx").status.code(), Some(2));
}
