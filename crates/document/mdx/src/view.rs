//! Transient human presentation, not an import/export format or editor witness.
use super::*;

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
    let mut metadata = json!({"title": document.title, "description": document.description,
        "tags": document.tags, "publication": document.publication, "theme": document.theme});
    for name in EDITOR_OWNED_EXTRA {
        if let Some(value) = document.extra.get(*name) {
            metadata[*name] = value.clone();
        }
    }
    let mut out = "---\n".to_owned();
    yaml(&metadata, 0, &mut out);
    out.push_str("---\n\n");

    let mut pages: Vec<_> = document.pages.iter().skip(1).collect();
    pages.sort_by_key(|page| {
        let node = &document.state.nodes[&page.key];
        (&node.entity, &node.memberships)
    });
    pages.insert(0, &document.pages[0]);
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
                _ => block_line(document, page, block, false),
            };
            let _ = writeln!(out, "{line}\n");
        }
        out.push_str("</Page>\n\n");
    }
    Ok(out)
}
