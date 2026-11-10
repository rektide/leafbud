//! One-time source-key preparation: propose durable keys for new blocks,
//! insert `<Key id="…" />` markers while preserving surrounding bytes, and
//! persist main-page identity in the reserved envelope. Plans are inert;
//! only [`apply_key_preparation`] produces bytes, and it proves the result
//! through the production parser before returning it.
use super::{json, parse_with_spans, patch};
use anyhow::{Context, Result, ensure};
use leafbud_document::*;

/// One nonoverlapping byte replacement in the original source (shared with
/// Markdown tidy planning).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TidyEdit {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// A proposed preparation: insertion edits for missing block keys, retarget
/// edits for page links addressing the provisional main page, the proposed
/// main-page key when `@leafbud.mainPage` is not persisted yet, and the
/// fully prepared document those edits produce. Proposed keys are
/// provisional until applied and persisted by the caller. The plan is bound
/// to the exact source it was planned from (in memory, never persisted), so
/// applying validates the operand instead of splicing stale coordinates.
#[derive(Clone, Debug)]
pub struct KeyPreparation {
    /// Ascending edits: Key marker insertion lines (start == end) and
    /// provisional-main-page link retargets (exact attribute replacements).
    pub edits: Vec<TidyEdit>,
    /// Proposed main-page key; None when identity is already persisted.
    pub main_page: Option<String>,
    /// The prepared document: every block keyed, main-page identity set.
    pub document: Document,
    /// How many block keys this preparation would assign.
    pub assigned: usize,
    /// The original input this plan was computed from.
    pub source: String,
}

impl KeyPreparation {
    pub fn is_noop(&self) -> bool {
        self.edits.is_empty() && self.main_page.is_none()
    }
}

/// Key proposals for preparation: one source per kind of identity. Callers
/// own randomness and spelling policy; typical spellings are `b-<uuid>` for
/// block/list-item keys and `p-<uuid>` for the implicit main page.
pub struct KeySource<'a> {
    pub block: &'a mut dyn FnMut() -> String,
    pub page: &'a mut dyn FnMut() -> String,
}

/// Plan key preparation. Preview/check never apply this plan; executed
/// tidy/push do.
pub fn prepare_keys(source: &str, keys: KeySource<'_>) -> Result<KeyPreparation> {
    let KeySource { block, page } = keys;
    let parsed = parse_with_spans(source)?;
    let unkeyed = parsed.unkeyed();
    let provisional_main = parsed.provisional_main_key().map(str::to_owned);
    // Propose keys in document order; insertions stay in ascending span order.
    let mut proposed: Vec<(usize, String)> = vec![];
    // Every existing identity is claimed: page names (including the main
    // page's persisted key), keyed blocks, and confirmed/retired node-table
    // rows. The provisional spellings being replaced are not durable and do
    // not reserve their namespace.
    let mut claimed: Vec<String> = parsed
        .document
        .pages
        .iter()
        .flat_map(|page| std::iter::once(page.key.clone()).chain(page.blocks.iter().map(|block| block.key.clone())))
        .filter(|key| Some(key.as_str()) != provisional_main.as_deref() && !unkeyed_contains(&unkeyed, &parsed, key))
        .collect();
    claimed.extend(parsed.document.state.nodes.keys().cloned());
    for &index in &unkeyed {
        let key = block();
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
    let main_page = document.state.main_page.is_none().then(page);
    if let Some(main) = &main_page {
        ensure!(
            main.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && main.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c)),
            "proposed main-page key {main:?} is not a valid source key spelling"
        );
        ensure!(!claimed.contains(main), "proposed main-page key {main:?} collides with an existing key");
        if let Some(provisional) = &provisional_main {
            for page in &mut document.pages {
                if page.key == *provisional {
                    page.key = main.clone();
                }
                for block in &mut page.blocks {
                    if let BlockKind::PageLink { page: target } = &mut block.kind
                        && target == provisional
                    {
                        *target = main.clone();
                    }
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
    // keeping the file's line-ending convention. Page links addressing the
    // provisional main page are retargeted to the persisted identity in the
    // same plan, replacing only the attribute spelling inside their block.
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
    if let (Some(provisional), Some(main)) = (&provisional_main, &main_page) {
        edits.extend(super::parse::retarget_page_links(source, provisional, main)?);
    }
    edits.sort_by_key(|edit| edit.start);
    ensure!(
        edits.windows(2).all(|pair| pair[0].end <= pair[1].start),
        "internal error: overlapping preparation edits"
    );
    Ok(KeyPreparation {
        edits,
        main_page,
        document,
        assigned: proposed.len(),
        source: source.to_owned(),
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
/// carry explicit keys. The plan is bound to its original input: applying it
/// to any other source (including same-length alterations) is an error, and
/// no-op semantics hold only for the matching source. Caller-visible edits
/// are validated (bounded, UTF-8 aligned, ascending, nonoverlapping) before
/// any splicing; failures are errors, never panics.
pub fn apply_key_preparation(source: &str, preparation: &KeyPreparation) -> Result<String> {
    ensure!(
        source == preparation.source,
        "preparation input changed since planning; re-plan before applying"
    );
    if preparation.is_noop() {
        return Ok(source.to_owned());
    }
    for TidyEdit { start, end, .. } in &preparation.edits {
        ensure!(start <= end, "preparation edit has reversed bounds");
        ensure!(*end <= source.len(), "preparation edit is out of bounds");
        ensure!(
            source.is_char_boundary(*start) && source.is_char_boundary(*end),
            "preparation edit is not UTF-8 aligned"
        );
    }
    ensure!(
        preparation
            .edits
            .windows(2)
            .all(|pair| pair[0].start <= pair[1].start && pair[0].end <= pair[1].start),
        "preparation edits are unordered or overlap"
    );
    let mut result = source.to_owned();
    for TidyEdit { start, end, text } in preparation.edits.iter().rev() {
        result.replace_range(start..end, text);
    }
    if let Some(main_page) = &preparation.main_page {
        result = patch::insert_state_line(&result, &format!("\"mainPage\": {}", json(main_page)))?;
    }
    let parsed = parse_with_spans(&result).context("prepared source must parse")?;
    ensure!(
        parsed.document.semantic_value() == preparation.document.semantic_value() && parsed.document.state == preparation.document.state,
        "key preparation would change content/associations"
    );
    ensure!(parsed.unkeyed().is_empty(), "prepared source still has unkeyed blocks");
    ensure!(
        parse_with_spans(source)?.comment_value(source) == parsed.comment_value(&result),
        "key preparation would change source comments"
    );
    Ok(result)
}
