//! Offline interop bridge to the production writer, not an alternative codec.
use anyhow::{Context, Result};
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    let input: serde_json::Value = serde_json::from_str(&source)?;
    let desired: Vec<leafbud_document::Inline> = serde_json::from_value(input.get("desired").context("desired inline content required")?.clone())?;
    let update = leafbud_editor::write_text(input["current"].as_str(), &desired)?;
    println!("{}", serde_json::json!({"update":update}));
    Ok(())
}
