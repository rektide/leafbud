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
        let preferences = desired
            .preferences
            .as_ref()
            .filter(|p| Some(*p) != current.preferences.as_ref())
            .map(|p| serde_json::from_value(serde_json::to_value(p).expect("preferences JSON")))
            .transpose()?;
        if title.is_some() || description_changed || tags.is_some() || preferences.is_some() {
            builder.result.mutations.push(NativeMutation::UpdatePublicationDraft(PublicationMetadata {
                title,
                description: description_changed.then(|| description.unwrap_or_default().to_owned()),
                tags,
                preferences,
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
