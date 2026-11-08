use anyhow::{Context, Result, ensure};
use satteri_authoring_experiment::{
    author,
    facts::{self, BuildContext},
    simulator::Store,
    text,
};
use serde_json::{Value, json};

fn checked(source: &str) -> satteri_authoring_experiment::model::Document<'_> {
    match author::compile(source) {
        Ok(doc) => doc,
        Err(errors) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&json!({"diagnostics":errors})).expect("serializable diagnostics")
            );
            std::process::exit(2);
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 2 && ["check", "demo"].contains(&args[0].as_str()),
        "usage: satteri-authoring-experiment check|demo FILE.mdx"
    );
    let source = std::fs::read_to_string(&args[1]).context("read source")?;
    let doc = checked(&source);
    let plan = facts::build(&doc, BuildContext::fixture(100))?;
    let decoded: Result<Vec<Value>> = plan
        .facts
        .iter()
        .filter_map(|f| match &f.data {
            facts::Data::Text { value } => Some(text::read(value).map(|content| json!({"entity":f.entity,"prosemirror":text::prosemirror(&content)}))),
            _ => None,
        })
        .collect();
    if args[0] == "check" {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"document":doc,"plan":plan,"stagedMutations":plan.staged_mutations(),"decoded":decoded?}))?
        );
        return Ok(());
    }
    let mut store = Store::fixture();
    let binding = store.binding.clone();
    let old = store.active_page().to_owned();
    ensure!(store.stage(&plan, Some(5)).is_err(), "expected injected interruption");
    ensure!(store.active_page() == old, "failed staging changed active body");
    store.stage(&plan, None)?;
    store.verify(&plan)?;
    let first_switch = store.switch(&plan)?;
    let count = store.facts.len();
    store.apply(&plan)?;
    ensure!(store.facts.len() == count, "repeat appended facts");
    let edited = format!("{source}\nA new locally authored paragraph.\n");
    let next_doc = checked(&edited);
    let next = facts::build(&next_doc, BuildContext::fixture(101))?;
    store.apply(&next)?;
    ensure!(store.binding == binding, "binding changed");
    ensure!(!store.reachable(store.active_page()).contains(&plan.main_page), "old main still in active body");
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "offlineSimulation":true, "binding":store.binding, "firstSwitch":first_switch,
            "interruptedStageKeptOldEntry":true, "repeatDidNotAppend":true,
            "editedSourceProducedNewBody":plan.main_page != next.main_page,
            "stablePollReused":plan.polls.keys().eq(next.polls.keys()),
            "sourceUnchanged":doc.original_source.as_bytes() == source.as_bytes(),
            "activePage":store.active_page(), "authorKeys":next.author_keys,
            "decodedFirstGeneration":decoded?, "requires":next.requires,
        }))?
    );
    Ok(())
}
