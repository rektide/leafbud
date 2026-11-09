//! Thin rendering for reusable workflow configuration.
use leafbud_workflow::config::{self, Config, Key};
use std::path::Path;

pub fn show(root: Option<&Path>, flags: &Config, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&context)?);
    } else {
        println!("global: {}", context.global_path.display());
        println!("workspace: {} ({})", context.workspace_root.display(), context.workspace_source);
        for value in context.values {
            println!(
                "{} = {} [{}]",
                value.key,
                value.value.as_deref().unwrap_or("(unset)"),
                value.source.unwrap_or("unset")
            );
        }
    }
    Ok(())
}

pub fn set(key: Key, value: &str, workspace_file: bool, root: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let path = config::set(key, value, workspace_file, root)?;
    println!("set {} in {}", key.as_str(), path.display());
    Ok(())
}
