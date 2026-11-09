//! One-time source-key preparation: propose durable keys for new blocks,
//! insert `<Key id="…" />` markers while preserving surrounding bytes, and
//! persist main-page identity in the reserved envelope. Plans are inert;
//! only [`apply_key_preparation`] produces bytes, and it proves the result
//! through the production parser before returning it.
use super::{json, parse, parse_with_spans, patch};
use anyhow::{Result, ensure};
use leafbud_document::*;

/// One nonoverlapping byte replacement in the original source (shared with
/// Markdown tidy planning).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TidyEdit {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// A proposed preparation: insertion edits for missing block keys, the
/// proposed main-page key when `@leafbud.mainPage` is not persisted yet, and
/// the fully prepared document those edits produce. Proposed keys are
/// provisional until applied and persisted by the caller.
#[derive(Clone, Debug)]
pub struct KeyPreparation {
    /// Ascending insertion edits (start == end) of Key marker lines.
    pub edits: Vec<TidyEdit>,
    /// Proposed main-page key; None when identity is already persisted.
    pub main_page: Option<String>,
    /// The prepared document: every block keyed, main-page identity set.
    pub document: Document,
    /// How many block keys this preparation would assign.
    pub assigned: usize,
}

impl KeyPreparation {
    pub fn is_noop(&self) -> bool {
        self.edits.is_empty() && self.main_page.is_none()
    }
}

/// Plan key preparation. `fresh` supplies candidate key spellings (callers
/// own randomness/uniqueness policy; typical spelling is `b-<uuid>`).
/// Preview/check never apply this plan; executed tidy/push do.
pub fn prepare_keys(source: &str, fresh: &mut dyn FnMut() -> String) -> Result<KeyPreparation> {
    let parsed = parse_with_spans(source)?;
    let unkeyed = parsed.unkeyed();
    // Propose keys in document order; insertions stay in ascending span order.
    let mut proposed: Vec<(usize, String)> = vec![];
    let mut claimed: Vec<String> = parsed
        .document
        .pages
        .iter()
        .flat_map(|page| page.blocks.iter().map(|block| block.key.clone()))
        .filter(|key| !unkeyed_contains(&unkeyed, &parsed, key))
        .collect();
    for &index in &unkeyed {
        let key = fresh();
        ensure!(
            key.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && key.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c)),
            "proposed key {key:?} is not a valid source key spelling"
        );
        ensure!(!claimed.contains(&key), "proposed key {key:?} collides with an existing key");
        claimed.push(key.clone());
        proposed.push((index, key));
    }
    // Build the prepared document: substitute proposed keys for provisional
    // ones (and any PageLink targets referencing the provisional main page).
    let mut document = parsed.document.clone();
    let substitution: std::collections::BTreeMap<usize, String> = unkeyed
        .iter()
        .zip(proposed.iter().map(|(_, key)| key.clone()))
        .map(|(&index, key)| (index, key))
        .collect();
    let main_page = document.state.main_page.is_none().then(fresh);
    if let Some(main) = &main_page {
        for page in &mut document.pages {
            if page.key == super::parse::PROVISIONAL_PAGE {
                page.key = main.clone();
            }
            for block in &mut page.blocks {
                if let BlockKind::PageLink { page: target } = &mut block.kind
                    && target == super::parse::PROVISIONAL_PAGE
                {
                    *target = main.clone();
                }
            }
        }
        document.state.main_page = Some(main.clone());
    }
    let mut index = 0;
    for page in &mut document.pages {
        for block in &mut page.blocks {
            if let Some(key) = substitution.get(&index) {
                block.key = key.clone();
            }
            index += 1;
        }
    }
    // Insertion edits: one Key marker line before each new block's span,
    // keeping the file's line-ending convention.
    let newline = if source.contains("\r\n") { "\r\n" } else { "\n" };
    let mut edits = vec![];
    for (index, key) in &proposed {
        let span = &parsed.spans[*index];
        edits.push(TidyEdit {
            start: span.start,
            end: span.start,
            text: format!("<Key id={} />{newline}", json(key)),
        });
    }
    edits.sort_by_key(|edit| edit.start);
    ensure!(
        edits.windows(2).all(|pair| pair[0].start <= pair[1].start),
        "internal error: unsorted preparation edits"
    );
    Ok(KeyPreparation {
        edits,
        main_page,
        document,
        assigned: proposed.len(),
    })
}

/// True when `key` is the provisional spelling of an unkeyed block.
fn unkeyed_contains(unkeyed: &[usize], parsed: &super::parse::Parsed, key: &str) -> bool {
    unkeyed.iter().any(|&index| {
        parsed
            .document
            .pages
            .iter()
            .flat_map(|page| page.blocks.iter())
            .nth(index)
            .is_some_and(|block| block.key == key)
    })
}

/// Apply a preparation and prove the result: same proposed identities,
/// equivalent semantic content, and a source-valid document whose blocks all
/// carry explicit keys. Applying a no-op preparation returns the input.
pub fn apply_key_preparation(source: &str, preparation: &KeyPreparation) -> Result<String> {
    if preparation.is_noop() {
        return Ok(source.to_owned());
    }
    let mut result = source.to_owned();
    for TidyEdit { start, end, text } in preparation.edits.iter().rev() {
        result.replace_range(start..end, text);
    }
    if let Some(main_page) = &preparation.main_page {
        result = patch::insert_state_line(&result, &format!("\"mainPage\": {}", json(main_page)))?;
    }
    let parsed = parse(&result).expect("prepared source must parse");
    ensure!(
        parsed.semantic_value() == preparation.document.semantic_value() && parsed.state == preparation.document.state,
        "key preparation would change content/associations"
    );
    ensure!(parse_with_spans(&result)?.unkeyed().is_empty(), "prepared source still has unkeyed blocks");
    Ok(result)
}
