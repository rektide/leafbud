use anyhow::{Context, Result, ensure};
use satteri_authoring_attributes_experiment::{
    compile,
    facts::{self, BuildContext},
    semantic_document, semantic_plan,
    simulator::Store,
    stock,
};
use serde_json::json;

fn checked(source: &str) -> satteri_authoring_attributes_experiment::CheckedDocument<'_> {
    match compile(source) {
        Ok(doc) => doc,
        Err(diagnostics) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&json!({"diagnostics":diagnostics})).expect("diagnostics serialize")
            );
            std::process::exit(2);
        }
    }
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        (args.len() == 2 && ["check", "stock"].contains(&args[0].as_str())) || (args.len() == 3 && args[0] == "compare"),
        "usage: check|stock FILE.mdx OR compare ATTRIBUTES.mdx KEY.mdx"
    );
    let source = std::fs::read_to_string(&args[1]).context("read source")?;
    if args[0] == "stock" {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"mdx":stock(&source,false),"mdxWithAttributes":stock(&source,true)}))?
        );
        return Ok(());
    }
    let checked = checked(&source);
    let plan = facts::build(&checked.document, BuildContext::fixture(100))?;
    if args[0] == "check" {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"checked":checked,"plan":plan,"stagedMutations":plan.staged_mutations()}))?
        );
        return Ok(());
    }
    let native = std::fs::read_to_string(&args[2]).context("read native Key source")?;
    let native = satteri_authoring_experiment::author::compile(&native).map_err(|d| anyhow::anyhow!("native compiler diagnostics: {d:?}"))?;
    let native_plan = facts::build(&native, BuildContext::fixture(100))?;
    ensure!(semantic_document(&checked.document) == semantic_document(&native), "typed semantics differ");
    ensure!(semantic_plan(&plan)? == semantic_plan(&native_plan)?, "Leaflet plan semantics differ");
    let mut store = Store::fixture();
    store.apply(&plan)?;
    store.verify(&plan)?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"equivalentTypedDocument":true,"equivalentLeafletPlan":true,"stagingAndFreshTextVerified":true,
        "annotations":checked.annotations,"nativeKeyMarkersRetained":checked.document.consumed_keys,
        "entities":plan.entities.len(),"facts":plan.facts.len(),"freshTextPayloads":plan.expected_text.len(),"sourceUnchanged":checked.document.original_source.as_bytes()==source.as_bytes()})
        )?
    );
    Ok(())
}
