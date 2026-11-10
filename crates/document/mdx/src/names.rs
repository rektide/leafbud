//! Explicit naming authority, never editor receipt settlement or body rendering.
use anyhow::{Context, Result, ensure};
use leafbud_document::{NamePath, Names};
use serde_json::json;

/// Set the author path only when explicitly supplied; settle the selected pair.
/// The workflow establishes filesystem evidence. This pure API proves that no
/// other author field, body byte, association or receipt changes.
pub fn patch_names(input: &str, path: Option<&str>, names: &Names) -> Result<String> {
    let existing = super::parse(input).context("unsupported managed source; naming preserved")?;
    ensure!(
        existing.state.pending.is_none() && existing.state.publication_attempt.is_none(),
        "pending native attempt; recover before naming"
    );
    let mut expected = super::header(input)?;
    let mut working = input.to_owned();
    if let Some(path) = path {
        expected["path"] = json!(path);
        if existing.extra.get("path") != Some(&json!(path)) {
            working = super::author::patch_fields(&working, &[("path", json!(path))])?;
        }
    }
    let mut state = existing.state.clone();
    state.names = Some(names.clone());
    if state != existing.state {
        working = super::patch::replace_envelope(&working, &state)?;
    }
    expected["@leafbud"] = json!(state);
    // Missing optional fields are semantically absent, not a mandate to
    // normalize the existing envelope on a genuine no-op.
    if working == input {
        ensure!(
            NamePath::from_author(&existing.extra).map_err(anyhow::Error::msg)? == names.path,
            "witness does not match author path"
        );
        return Ok(working);
    }
    ensure!(super::header(&working)? == expected, "naming patch changed unrelated YAML");
    let parsed = super::parse(&working)?;
    let mut wanted = existing;
    wanted.state = state;
    if let Some(path) = path {
        wanted.extra.insert("path".into(), json!(path));
    }
    ensure!(parsed == wanted, "naming patch changed body, metadata or tracking beyond its authority");
    ensure!(
        NamePath::from_author(&parsed.extra).map_err(anyhow::Error::msg)? == names.path,
        "witness does not match author path"
    );
    ensure!(
        super::patch::split_managed(input)?.1 == super::patch::split_managed(&working)?.1,
        "naming changed body bytes"
    );
    Ok(working)
}
