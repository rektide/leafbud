use super::*;
use leafbud_document::canonical_description;
use leafbud_protocol::{LeafletMetadata, PublicationMetadata};

pub(super) fn prepare(builder: &mut Builder<'_>, current: &Document, persisted_title: Option<&str>) -> Result<()> {
    let desired = builder.desired;
    ensure!(
        desired.extra.get("contributors") == current.extra.get("contributors"),
        "contributor permission edits are unsupported"
    );
    // The reader's display fallback (e.g. Untitled) is not persisted metadata.
    let title = (Some(desired.title.as_str()) != persisted_title).then(|| desired.title.clone());
    let description = canonical_description(desired.description.as_deref());
    let description_changed = description != canonical_description(current.description.as_deref());
    let binding = desired.state.binding.as_ref().expect("bound");
    if binding.publication.is_some() || binding.uri.is_some() {
        let tags = (desired.tags != current.tags).then(|| desired.tags.clone());
        if title.is_some() || description_changed || tags.is_some() {
            builder.result.mutations.push(NativeMutation::UpdatePublicationDraft(PublicationMetadata {
                title,
                description: description_changed.then(|| description.unwrap_or_default().to_owned()),
                tags,
            }));
        }
    } else {
        ensure!(desired.tags.is_empty(), "standalone tags have no established native storage");
        if title.is_some() || description_changed {
            builder.result.mutations.push(NativeMutation::UpdateLeafletMetadata(LeafletMetadata {
                title,
                description: description_changed.then(|| description.map(str::to_owned)),
            }));
        }
    }
    if desired.theme != current.theme {
        let theme = desired
            .theme
            .as_ref()
            .context("theme clearing is unsupported; preserve the explicit native theme")?;
        builder.set(
            desired.state.binding.as_ref().expect("bound").root.parse()?,
            "theme/page-leaflet-watermark",
            json!({"type":"boolean","value":theme.leaflet_watermark}),
        )?;
    }
    Ok(())
}
