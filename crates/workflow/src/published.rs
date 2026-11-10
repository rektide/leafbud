//! Authoritative native/public association, not a public-body import fallback.
use anyhow::{Context, Result, ensure};
use leafbud_web::LeafletData;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Association {
    pub uri: String,
    pub publication: Option<String>,
    /// Appview's joined released metadata; never the working editor body.
    pub record: Value,
}

pub(crate) fn association(data: &LeafletData) -> Result<Option<Association>> {
    let mut found: Option<Association> = None;
    for row in data.association_rows() {
        for (table, field) in [("leaflets_in_publications", "doc"), ("leaflets_to_documents", "document")] {
            for link in row[table].as_array().into_iter().flatten() {
                if link[field].is_null() {
                    continue;
                }
                ensure!(
                    row["root_entity"] == data.raw["root_entity"],
                    "published sibling has a different native root; ambiguous association"
                );
                let uri = link[field].as_str().context("published linkage has no document URI")?;
                leafbud_document::validate_document_uri(uri).map_err(anyhow::Error::msg)?;
                let parsed = leafbud_atproto::AtUri::parse(uri)?;
                let joined = &link["documents"];
                ensure!(joined["uri"] == uri, "published joined document identity missing or mismatched");
                let record = &joined["data"];
                ensure!(record["$type"] == parsed.collection, "published document type differs from URI collection");
                let publication = if table == "leaflets_in_publications" {
                    let publication = link["publication"].as_str().context("published publication URI missing")?;
                    let container = &link["publications"];
                    ensure!(container["uri"] == publication, "joined publication identity missing or mismatched");
                    ensure!(
                        container["identity_did"] == parsed.authority,
                        "public owner differs from verified publication owner"
                    );
                    ensure!(
                        (parsed.collection == "site.standard.document" && record["site"] == publication)
                            || (parsed.collection == "pub.leaflet.document" && record["publication"] == publication && record["author"] == parsed.authority),
                        "public record publication/owner differs from native association"
                    );
                    Some(publication.to_owned())
                } else {
                    ensure!(
                        (parsed.collection == "site.standard.document" && record["site"] == format!("https://leaflet.pub/p/{}", parsed.authority))
                            || (parsed.collection == "pub.leaflet.document" && record["publication"].is_null() && record["author"] == parsed.authority),
                        "public record standalone owner/site differs from native association"
                    );
                    None
                };
                let current = Association {
                    uri: uri.to_owned(),
                    publication,
                    record: record.clone(),
                };
                ensure!(
                    found.as_ref().is_none_or(|previous| previous == &current),
                    "multiple or inconsistent published associations; explicit decision required"
                );
                found = Some(current);
            }
        }
    }
    if let Some(found) = &found {
        ensure!(
            crate::import::association(data)? == found.publication,
            "published and working publication associations differ"
        );
    }
    Ok(found)
}

impl crate::RemoteReader {
    /// The observed authenticated inventory selects candidates only. Every
    /// selected capability is verified again by get_leaflet_data during import.
    /// Omission is not proof of public deletion, nor permission to mint/copy.
    pub(crate) async fn published_locator(&self, uri: &str) -> Result<uuid::Uuid> {
        leafbud_document::validate_document_uri(uri).map_err(anyhow::Error::msg)?;
        let discovery = self
            .client
            .list()
            .await
            .context("editing-unavailable: native discovery unavailable; public identity alone is not an editor capability")?;
        let mut tokens = std::collections::BTreeSet::new();
        for leaflet in discovery.leaflets {
            if leaflet.metadata.leaflets_in_publications.iter().any(|link| link.doc.as_deref() == Some(uri))
                || leaflet.metadata.leaflets_to_documents.iter().any(|link| link.document.as_deref() == Some(uri))
            {
                tokens.insert(leaflet.metadata.id);
            }
        }
        ensure!(
            tokens.len() == 1,
            "editing-unavailable: native inventory has no unique legitimate editor for that exact public URI; no local copy or new editor created"
        );
        Ok(tokens.into_iter().next().expect("one token"))
    }
}

#[cfg(test)]
pub(crate) mod tests;
