//! Intentional alias rename, never inferred from text similarity or omission.
use super::{TidyEdit, source};
use anyhow::{Context, Result, ensure};
use leafbud_document::BlockKind;

/// Rename one explicit current source identity and references, preserving
/// native associations and all receipts. Pending/future owned state blocks.
pub fn rename_key(input: &str, old: &str, new: &str) -> Result<String> {
    ensure!(
        super::parse::valid_key(new),
        "invalid new source key; use an ASCII letter followed by letters, digits, _, . or -"
    );
    let parsed = super::parse_with_spans(input).context("unsupported managed source; nothing was changed")?;
    let mut expected = parsed.document.clone();
    ensure!(
        expected.state.pending.is_none(),
        "pending native attempt; run recover before renaming source keys"
    );
    let explicit_block = expected
        .pages
        .iter()
        .flat_map(|p| &p.blocks)
        .zip(&parsed.keyed)
        .any(|(b, keyed)| b.key == old && *keyed);
    let explicit_page = expected.pages.iter().skip(1).any(|p| p.key == old) || expected.state.main_page.as_deref() == Some(old);
    ensure!(
        explicit_block || explicit_page,
        "source key {old:?} is not a current authored identity (retired/provisional rows cannot be renamed)"
    );
    if old == new {
        return Ok(input.to_owned());
    }
    ensure!(
        !expected.state.nodes.contains_key(new) && !expected.pages.iter().any(|p| p.key == new || p.blocks.iter().any(|b| b.key == new)),
        "new source key {new:?} collides with a current or confirmed/retired key"
    );
    for page in &mut expected.pages {
        if page.key == old {
            page.key = new.into();
        }
        for block in &mut page.blocks {
            if block.key == old {
                block.key = new.into();
            }
            if let BlockKind::PageLink { page } = &mut block.kind
                && page == old
            {
                *page = new.into();
            }
        }
    }
    let mut edits = super::parse::rename_attributes(input, old, new)?;
    let header = source::header(input)?;
    let (_, state) = header.node.entry("@leafbud").context("reserved envelope absent")?;
    if expected.state.main_page.as_deref() == Some(old) {
        expected.state.main_page = Some(new.into());
        let (_, value) = state.value.entry("mainPage").context("mainPage value absent")?;
        let range = source::range(value)?;
        ensure!(
            super::frontmatter::<String>(&input[header.offset + range.start..header.offset + range.end])
                .ok()
                .as_deref()
                == Some(old),
            "unsupported mainPage scalar patch shape; rewrite it as a single-line string first"
        );
        edits.push(TidyEdit {
            start: header.offset + range.start,
            end: header.offset + range.end,
            text: super::json(new),
        });
    }
    if let Some(row) = expected.state.nodes.remove(old) {
        expected.state.nodes.insert(new.into(), row);
        let (_, nodes) = state.value.entry("nodes").context("nodes mapping absent")?;
        let (key, _) = nodes.value.entry(old).context("node key absent")?;
        let range = source::range(key)?;
        edits.push(TidyEdit {
            start: header.offset + range.start,
            end: header.offset + range.end,
            text: super::json(new),
        });
    }
    let out = source::splice(input, edits)?;
    let actual = super::parse(&out).context("renamed source failed validation; nothing was changed")?;
    ensure!(actual == expected, "rename would change unrelated content or associations; nothing was changed");
    Ok(out)
}
