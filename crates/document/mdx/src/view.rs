//! Transient human presentation, not an import/export format or editor witness.
use super::*;
use leafbud_document::BlockKind;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Comparable current editor-authored meaning. Both endpoints use this same
/// writer. Omits tool state, source keys and local-only metadata. Pages receive
/// presentation-only ordinals, with the entry first and other pages sorted by
/// native association (never printed); links use those same ordinals. Exact MDX
/// components retain supported meaning not expressible as ordinary Markdown.
///
/// Input must be a validated supported document. Equality of these views says
/// nothing about receipt validity, hidden native state or object associations.
pub fn render_comparison(document: &Document) -> Result<String> {
    // Validate the complete typed input through the existing proven renderer;
    // never emit a plausible partial view of an invalid document.
    render(document)?;
    let mut out = "---\n".to_owned();
    yaml(&render::metadata_value(document), 0, &mut out);
    out.push_str("---\n\n");

    let mut pages: Vec<_> = document.pages.iter().skip(1).collect();
    pages.sort_by_key(|page| document.state.nodes.get(&page.key).map(|n| n.entity.clone()));
    if let Some(main) = document.pages.first() {
        pages.insert(0, main);
    }
    let labels: BTreeMap<_, _> = pages
        .iter()
        .enumerate()
        .map(|(i, page)| (page.key.as_str(), format!("page-{}", i + 1)))
        .collect();
    for page in pages {
        let _ = writeln!(out, "<Page name={} kind=\"linear\">\n", json(&labels[page.key.as_str()]));
        for block in &page.blocks {
            let line = match &block.kind {
                BlockKind::PageLink { page } => format!("<PageLink page={} />", json(&labels[page.as_str()])),
                BlockKind::Code { language, text } => {
                    // Read ordinary code as code, but prove exact language,
                    // whitespace and trailing-newline values with our parser.
                    let width = text.split(|c| c != '`').map(str::len).max().unwrap_or(0).saturating_add(1).max(3);
                    let fence = "`".repeat(width);
                    let candidate = format!("{fence}{language}\n{text}\n{fence}");
                    if render::block_candidate_valid(document, &page.key, block, &candidate) {
                        candidate
                    } else {
                        render::block_line(document, page, block, false)
                    }
                }
                BlockKind::ListItem { .. } => render::block_line(document, page, block, false),
                _ => render::block_line(document, page, block, false),
            };
            let _ = writeln!(out, "{line}\n");
        }
        out.push_str("</Page>\n\n");
    }
    Ok(out)
}
