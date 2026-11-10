//! Pure no-offload prerequisite, pinned to Leaflet 600c7b5f's image-free profile.
//! No second semantic model, fact mutation simulator, persistence or native RPC.
use anyhow::{Context, Result, ensure};
use leafbud_document::{BlockKind, Document};
use leafbud_editor::write::Compiled;
use leafbud_protocol::NativeMutation;
use serde::Serialize;
use serde_json::Value;

pub const INLINE_THRESHOLD: usize = 100 * 1024;

#[derive(Debug, Serialize)]
pub struct Witness {
    pub profile: &'static str,
    pub scope: &'static str,
    pub pages_json_utf8_upper_bound: usize,
    pub inline_threshold: usize,
    pub no_offload_at_witness: bool,
    pub provisional_page_ids: usize,
    pub active_text_segments: usize,
    pub requires_fresh_saved_native_witness_before_dispatch: bool,
}

/// Evaluate the existing compiler's cumulative text updates and targets without
/// running them. The desired graph supplies post-save ordering/grouping; every
/// active rich block is replaced with strictly decoded *actual* segmentation.
/// Call only with the compile result for these exact desired/current facts.
pub(crate) fn compiled(desired: &Document, facts: &[Value], compiled: &Compiled) -> Result<Witness> {
    let mut native = desired.clone();
    native.state.nodes = compiled.targets.clone();
    let mut segments = 0;
    for page in &mut native.pages {
        for block in &mut page.blocks {
            let node = native.state.nodes.get(&block.key).context("compiled block target missing")?;
            let value = |attribute: &str| -> Result<Option<Value>> {
                // Native compiler emits full cumulative block/text, not a delta
                // against a made-up fresh document. Other scalars use set/clear.
                if let Some(row) = compiled.mutations.iter().rev().find_map(|mutation| match mutation {
                    NativeMutation::AssertFact(row) if row.entity.to_string() == node.entity && row.attribute == attribute => Some(row),
                    _ => None,
                }) {
                    return Ok(Some(row.data["value"].clone()));
                }
                let mut retained = facts.iter().filter(|row| row["entity"] == node.entity && row["attribute"] == attribute);
                let row = retained.next();
                ensure!(retained.next().is_none(), "ambiguous active size fact");
                Ok(row
                    .filter(|row| {
                        !compiled
                            .mutations
                            .iter()
                            .any(|mutation| matches!(mutation, NativeMutation::RetractFact { fact_id } if row["id"] == fact_id.to_string()))
                    })
                    .map(|row| row["data"]["value"].clone()))
            };
            if let Some(list) = value("block/is-list")? {
                ensure!(list.is_boolean(), "size profile requires boolean native list flag");
                ensure!(
                    list == matches!(block.kind, BlockKind::ListItem { .. }),
                    "native list flag differs from supported post-save grouping"
                );
            } else {
                ensure!(!matches!(block.kind, BlockKind::ListItem { .. }), "post-save list flag absent");
            }
            let content = match &mut block.kind {
                BlockKind::Paragraph { content } | BlockKind::Heading { content, .. } | BlockKind::Quote { content } | BlockKind::ListItem { content, .. } => {
                    content
                }
                _ => continue, // Native ignores latent block/text on code/rule/card.
            };
            let actual = value("block/text")?
                .map(|value| leafbud_editor::read_text_segments(value.as_str().context("invalid active native text")?))
                .transpose()?
                .unwrap_or_default();
            ensure!(
                leafbud_editor::normalize_text(&actual)? == leafbud_editor::normalize_text(content)?,
                "compiled native text does not match desired size input"
            );
            segments += actual.len();
            *content = actual;
        }
    }
    let bytes = leafbud_records::projection::inline_pages_json_upper_bound(&native)?;
    let provisional_page_ids = desired.pages.iter().filter(|page| !desired.state.nodes.contains_key(&page.key)).count();
    Ok(Witness {
        profile: "leaflet-native-inline-size/1@600c7b5f",
        scope: if compiled.mutations.is_empty() {
            "loaded-native-facts"
        } else {
            "compiled-post-save-cumulative-state; conditional-on-save"
        },
        pages_json_utf8_upper_bound: bytes,
        inline_threshold: INLINE_THRESHOLD,
        no_offload_at_witness: bytes <= INLINE_THRESHOLD,
        provisional_page_ids,
        active_text_segments: segments,
        requires_fresh_saved_native_witness_before_dispatch: true,
    })
}

#[cfg(test)]
mod tests;
