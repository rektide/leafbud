//! Publication-record lookup for URL derivation in publish flows.
//!
//! `publish build --publication <at-uri>` and `publish run` need the
//! publication's `url` field to derive the document's public URL
//! (`{pubDomain}/{rkey}`, experiment 03 §6). This module fetches the
//! publication record via a public `getRecord` and extracts name + url:
//! the jacquard-generated type first, raw field extraction as fallback —
//! third-party publications are not guaranteed lexicon-clean (blento's
//! open-object `preferences` breaks the generated type; experiment 03 §2).

use jacquard_common::DefaultStr;

use crate::site_standard::publication::Publication;
use springleaf_atproto::XrpcClient;

/// Errors from [`publication_for_record`].
#[derive(Debug, thiserror::Error)]
pub enum PublicationError {
    #[error("invalid publication at-uri {0:?} ({1}); expected at://<did>/site.standard.publication/<rkey>")]
    BadUri(String, String),
    #[error("fetching publication record: {0}")]
    Fetch(#[from] springleaf_atproto::Error),
}

/// The name/url pair a publish flow needs from a publication record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationInfo {
    pub name: Option<String>,
    /// The publication's `url` field (custom domain), when present.
    pub url: Option<String>,
}

/// Fetch the publication record at `publication_at_uri` and extract
/// name + url. `Ok(None)` when the record does not exist on the repo.
///
/// Typed decode first; a record that does not fit the generated type
/// still yields its raw `name`/`url` fields (which is all URL
/// derivation needs).
pub async fn publication_for_record(client: &XrpcClient, publication_at_uri: &str) -> Result<Option<PublicationInfo>, PublicationError> {
    let uri = springleaf_atproto::AtUri::parse(publication_at_uri).map_err(|e| PublicationError::BadUri(publication_at_uri.to_string(), e.to_string()))?;
    if uri.collection != "site.standard.publication" {
        return Err(PublicationError::BadUri(
            publication_at_uri.to_string(),
            format!("collection must be site.standard.publication, got {:?}", uri.collection),
        ));
    }
    let Some(record) = client.get_record(&uri.authority, &uri.collection, &uri.rkey).await? else {
        return Ok(None);
    };
    Ok(Some(info_from_value(&record.value)))
}

/// Typed-first, raw-fallback extraction (mirrors `pubs list`'s decoder
/// policy, same evidence: experiment 03 §2's blento/skyreader shapes).
fn info_from_value(value: &serde_json::Value) -> PublicationInfo {
    if let Ok(publication) = serde_json::from_value::<Publication<DefaultStr>>(value.clone()) {
        return PublicationInfo {
            name: Some(publication.name.as_str().to_string()),
            url: Some(publication.url.as_str().to_string()),
        };
    }
    PublicationInfo {
        name: value.get("name").and_then(|v| v.as_str()).map(str::to_string),
        url: value.get("url").and_then(|v| v.as_str()).map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn info_extraction_typed_and_fallback() {
        // Lexicon-clean publication (technautical, experiment 03 §2).
        let typed = info_from_value(&json!({
            "$type": "site.standard.publication",
            "name": "JauntyWunderKind's Technautical Expeditions",
            "url": "https://technautical.leaflet.pub"
        }));
        assert_eq!(typed.name.as_deref(), Some("JauntyWunderKind's Technautical Expeditions"));
        assert_eq!(typed.url.as_deref(), Some("https://technautical.leaflet.pub"));
        // blento shape: open-object preferences defeats the generated
        // type; the raw fallback still extracts name/url.
        let blento = info_from_value(&json!({
            "$type": "site.standard.publication",
            "name": "JauntyWunderKind",
            "url": "https://blento.app/jauntywk.bsky.social",
            "preferences": {"baseColor": "#ffffff", "accentColor": "#000000"}
        }));
        assert_eq!(blento.name.as_deref(), Some("JauntyWunderKind"));
        assert_eq!(blento.url.as_deref(), Some("https://blento.app/jauntywk.bsky.social"));
        // No url field at all (a hypothetical legacy-shaped record).
        let bare = info_from_value(&json!({"$type": "site.standard.publication", "name": "No domain", "url": null}));
        assert_eq!(bare.url, None);
    }

    #[tokio::test]
    async fn bad_uris_rejected_before_the_wire() {
        let client = XrpcClient::new("https://puffball.us-east.host.bsky.network").unwrap();
        for bad in [
            "https://example.com/not-an-at-uri",
            "at://did:plc:x/site.standard.document/wrong-collection",
            "at://did:plc:x/site.standard.publication",
        ] {
            let err = publication_for_record(&client, bad).await;
            assert!(matches!(&err, Err(PublicationError::BadUri(got, _)) if got == bad), "{bad}: {err:?}");
        }
    }
}
