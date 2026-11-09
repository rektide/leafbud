//! Wire-shape helpers for Leaflet public records.
//!
//! What is retained after the scaffold reset, and why:
//!
//! - [`validate_document_record`] — decode + lexicon-constraint check of a
//!   `site.standard.document` JSON value through the generated type. This
//!   is the crate's named "validation" ownership and the pre-write gate
//!   any future publication projection will reuse.
//! - [`reinject_type_discriminators`] — the wire-shape adaptation for
//!   jacquard's single-variant-union codegen (see crate docs); anything
//!   that serializes a document record needs it.
//! - [`standalone_site_did`] / [`derive_document_url`] — the public-URL
//!   derivation rules proven by `experiments/03-post-builder/` §6
//!   (publication custom domain, `/lish/{did}/{pubRkey}/{rkey}` deep
//!   permalink, and the standalone `/p/{did}/{rkey}` route).
//!
//! The old frontmatter → record builder (`build_record`), its social-cli
//! frontmatter parser, the slug→rkey policy, and the markdown→blocks
//! converter reset with the sidecar/JSON lifecycle: the accepted design
//! replaces local record assembly with an MDX document model and a
//! document-to-public projection that does not exist yet. Pre-reset code
//! is recoverable at revision `80436b61`.

use jacquard_common::DefaultStr;
use leafbud_atproto::AtUri;

use crate::site_standard::document::Document;

/// Errors from [`validate_document_record`].
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("record does not decode as a site.standard.document: {0}")]
    Decode(String),
    #[error("record fails site.standard.document lexicon validation: {0}")]
    Constraints(String),
}

/// Validate record JSON through the generated lexicon type: decode +
/// constraint checks (the pre-put gate for any future publication write).
pub fn validate_document_record(record: &serde_json::Value) -> Result<(), ValidationError> {
    let document: Document<DefaultStr> = serde_json::from_value(record.clone()).map_err(|e| ValidationError::Decode(e.to_string()))?;
    use jacquard_lexicon::schema::LexiconSchema;
    document.validate().map_err(|e| ValidationError::Constraints(e.to_string()))
}

/// Re-add the `$type` discriminators jacquard's single-variant-union
/// codegen drops (see crate docs): `content.$type =
/// "pub.leaflet.content"` and `pages[].blocks[].$type =
/// "pub.leaflet.pages.linearDocument#block"`. Every serializer that emits
/// a document record through the generated types shares this helper so
/// all emit the same wire shape.
pub fn reinject_type_discriminators(record: &mut serde_json::Value) {
    let Some(content) = record.get_mut("content").and_then(|c| c.as_object_mut()) else {
        return;
    };
    content.entry("$type").or_insert_with(|| "pub.leaflet.content".into());
    let Some(pages) = record.get_mut("content").and_then(|c| c.get_mut("pages")).and_then(|p| p.as_array_mut()) else {
        return;
    };
    for page in pages {
        let Some(blocks) = page.get_mut("blocks").and_then(|b| b.as_array_mut()) else {
            continue;
        };
        for block in blocks {
            if let Some(obj) = block.as_object_mut() {
                obj.entry("$type").or_insert_with(|| "pub.leaflet.pages.linearDocument#block".into());
            }
        }
    }
}

/// A standalone `site` value: the HTTPS form Leaflet writes for drafts
/// with no publication linkage, `https://leaflet.pub/p/{did}`
/// (`publishLeaflet.ts:418-419`). Returns the embedded DID.
pub fn standalone_site_did(site: &str) -> Option<String> {
    let rest = site.strip_prefix("https://leaflet.pub/p/")?;
    let did = rest.trim_end_matches('/');
    (did.starts_with("did:") && !did.is_empty() && !did.contains('/')).then(|| did.to_string())
}

/// Errors from [`derive_document_url`].
#[derive(Debug, thiserror::Error)]
pub enum UrlError {
    #[error("invalid publication at-uri {0:?} ({1}); expected at://<did>/site.standard.publication/<rkey>")]
    BadPublication(String, #[source] leafbud_atproto::Error),
}

/// Derive the human-facing URL for a document rkey (experiment 03 §6):
/// the publication's own `url` field (custom domain) + `/{rkey}`, else the
/// `https://leaflet.pub/lish/{did}/{pubRkey}/{rkey}` deep permalink.
/// A standalone site value derives `{site}/{rkey}` — the
/// `/p/{did}/{rkey}` published-post route (Leaflet `getDocumentURL`: no
/// publication ⇒ base = doc.site + path).
pub fn derive_document_url(publication_at_uri: &str, publication_url: Option<&str>, rkey: &str) -> Result<String, UrlError> {
    if let Some(url) = publication_url.filter(|u| !u.is_empty()) {
        return Ok(format!("{}/{}", url.trim_end_matches('/'), rkey));
    }
    if standalone_site_did(publication_at_uri).is_some() {
        return Ok(format!("{}/{}", publication_at_uri.trim_end_matches('/'), rkey));
    }
    let uri = AtUri::parse(publication_at_uri).map_err(|e| UrlError::BadPublication(publication_at_uri.to_string(), e))?;
    Ok(format!("https://leaflet.pub/lish/{}/{}/{}", uri.authority, uri.rkey, rkey))
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOLDEN_RECORD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../experiments/03-post-builder/built-record.json"));
    const PUBLICATION: &str = "at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.publication/3molsbwkh522o";

    /// Pin-drift alarm (LEAFLET-PIN.md regeneration checklist step 3):
    /// the committed experiment-03 record — a real wire-shape document
    /// with its `$type` discriminators — must decode through the
    /// generated type, pass lexicon validation, and round-trip
    /// byte-identically after discriminator re-injection. A regenerated
    /// tree that changes decode or serialization behavior fails here.
    #[test]
    fn built_record_fixture_round_trips_through_generated_type() {
        let mut record: serde_json::Value = serde_json::from_str(GOLDEN_RECORD).unwrap();
        validate_document_record(&record).expect("fixture passes lexicon validation");
        // The fixture already carries the discriminators; re-injection is
        // a no-op on it.
        reinject_type_discriminators(&mut record);
        let document: Document = serde_json::from_value(record.clone()).expect("decodes through the generated type");
        let mut again = serde_json::to_value(&document).unwrap();
        reinject_type_discriminators(&mut again);
        assert_eq!(again, record, "round-trip + reinject must reproduce the fixture exactly");
        assert_eq!(record["content"]["$type"], "pub.leaflet.content");
        assert_eq!(record["content"]["pages"][0]["blocks"][0]["$type"], "pub.leaflet.pages.linearDocument#block");
    }

    #[test]
    fn url_derivation_both_forms() {
        // Publication-domain form: url field wins, trailing slash trimmed.
        assert_eq!(
            derive_document_url(PUBLICATION, Some("https://technautical.leaflet.pub/"), "leafbud-protocol-probe").unwrap(),
            "https://technautical.leaflet.pub/leafbud-protocol-probe"
        );
        // Fallback: /lish/{did}/{pubRkey}/{rkey}.
        assert_eq!(
            derive_document_url(PUBLICATION, None, "leafbud-protocol-probe").unwrap(),
            "https://leaflet.pub/lish/did:plc:zjbq26wybii5ojoypkso2mso/3molsbwkh522o/leafbud-protocol-probe"
        );
        // A malformed publication at-uri is a typed error, not a panic.
        assert!(matches!(derive_document_url("not-an-at-uri", None, "x"), Err(UrlError::BadPublication(..))));
    }

    /// Standalone `site` values (drafts with no publication linkage):
    /// `https://leaflet.pub/p/{did}` — the URL is `{site}/{rkey}` (the
    /// `/p/{did}/{rkey}` published-post route; `getDocumentURL` uses the
    /// site as the base when no publication exists).
    #[test]
    fn url_derivation_standalone_site_form() {
        let site = "https://leaflet.pub/p/did:plc:zjbq26wybii5ojoypkso2mso";
        assert_eq!(standalone_site_did(site), Some("did:plc:zjbq26wybii5ojoypkso2mso".to_string()));
        assert_eq!(
            derive_document_url(site, None, "my-post").unwrap(),
            "https://leaflet.pub/p/did:plc:zjbq26wybii5ojoypkso2mso/my-post"
        );
        // Trailing slash on the site is trimmed, did recovered either way.
        assert_eq!(standalone_site_did(&format!("{site}/")).as_deref(), Some("did:plc:zjbq26wybii5ojoypkso2mso"));
        assert_eq!(
            derive_document_url(&format!("{site}/"), None, "x").unwrap(),
            "https://leaflet.pub/p/did:plc:zjbq26wybii5ojoypkso2mso/x"
        );
        // Near-misses stay non-standalone: wrong host, extra path
        // segments, non-DID payload.
        assert_eq!(standalone_site_did("https://example.com/p/did:plc:x"), None);
        assert_eq!(standalone_site_did("https://leaflet.pub/p/did:plc:x/sub"), None);
        assert_eq!(standalone_site_did("https://leaflet.pub/p/handle.example"), None);
        assert_eq!(standalone_site_did("http://leaflet.pub/p/did:plc:x"), None);
    }
}
