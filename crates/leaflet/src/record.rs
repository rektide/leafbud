//! Frontmatter → `site.standard.document` record builder.
//!
//! Produces the canonical on-disk/posted form (`design/records-canonical.
//! glm53m.md`): a pure record JSON exactly as `com.atproto.repo.putRecord`
//! would write it, proven end-to-end offline by
//! `experiments/03-post-builder/` (78/78 lexicon + offset checks).
//!
//! - **Frontmatter** (social-cli `extractFrontmatter` port): `---` fences,
//!   flat `key: value` lines; `title`, `description`, `tags`
//!   (comma-separated), `slug`. Values may carry matching quotes.
//! - **rkey/slug**: frontmatter `slug`, else `slugify(title)` (lowercase,
//!   non-`[a-z0-9]` runs → `-`, trimmed, ≤60 chars — social-cli's rule).
//!   The invariant that matters downstream is `path == "/" + rkey`
//!   (experiment 03 §6): Leaflet's middleware rewrites
//!   `{domain}/{path}` → `/lish/{did}/{pub-rkey}/{rkey}` and resolves by
//!   rkey.
//! - **URL derivation** ([`derive_document_url`]): the publication record's
//!   `url` field (custom domain, trailing `/` trimmed) + `/{rkey}`, else
//!   the deep permalink `https://leaflet.pub/lish/{did}/{pubRkey}/{rkey}`.
//! - **`$type` re-injection**: jacquard's codegen collapses single-variant
//!   unions (`Document.content`, the `blocks[]` ref) to bare structs,
//!   dropping their `$type` discriminators. Real records carry them
//!   (experiment 03's inventory; the committed fixtures), so the builder
//!   re-adds `pub.leaflet.content` and
//!   `pub.leaflet.pages.linearDocument#block` after serialization. On
//!   read, no help is needed: the discriminators land in the generated
//!   `extra_data` forward-compat maps.

use std::str::FromStr;

use jacquard_common::DefaultStr;
use jacquard_common::types::string::{Datetime, UriValue};
use springleaf_atproto::AtUri;

use crate::markdown::{blocks_to_text_content, convert_markdown};
use crate::pub_leaflet::content::{Content, ContentPagesItem};
use crate::pub_leaflet::pages::linear_document::LinearDocument;
use crate::site_standard::document::Document;

/// Record-building errors.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("markdown conversion failed: {0}")]
    Markdown(#[from] crate::markdown::MarkdownError),
    #[error("frontmatter: {0}")]
    Frontmatter(String),
    #[error("title is required (frontmatter `title:`) when no slug supplies one")]
    MissingTitle,
    #[error("title {title:?} produces an empty slug; set an explicit `slug:` in the frontmatter")]
    EmptySlug { title: String },
    #[error("invalid publication at-uri {0:?}: {1}")]
    BadPublication(String, String),
}

/// Parsed flat frontmatter (social-cli's YAML-ish subset).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub slug: Option<String>,
}

/// Split a `---`-fenced frontmatter block from the markdown body.
/// Content without a leading fence is all body (social-cli semantics:
/// the closing fence is the next `---` anywhere after the first).
pub fn parse_frontmatter(markdown: &str) -> Result<(Frontmatter, &str), BuildError> {
    if !markdown.starts_with("---") {
        return Ok((Frontmatter::default(), markdown));
    }
    let rest = &markdown[3..];
    let end = match rest.find("---") {
        Some(i) => i,
        None => return Ok((Frontmatter::default(), markdown)),
    };
    let block = &rest[..end];
    let body = &rest[end + 3..];
    let mut fm = Frontmatter::default();
    for line in block.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = strip_matching_quotes(value.trim());
        match key {
            "title" => fm.title = Some(value),
            "description" => fm.description = Some(value),
            "slug" => fm.slug = Some(value),
            "tags" => fm.tags = value.split(',').map(str::trim).filter(|t| !t.is_empty()).map(str::to_string).collect(),
            _ => {}
        }
    }
    Ok((fm, body))
}

/// Strip one layer of matching surrounding quotes, social-cli's
/// `/^["']|["']$/g` trim (which strips a lone quote at either end too).
fn strip_matching_quotes(value: &str) -> String {
    let mut out = value;
    if out.starts_with('"') || out.starts_with('\'') {
        out = &out[1..];
    }
    if out.ends_with('"') || out.ends_with('\'') {
        out = &out[..out.len() - 1];
    }
    out.to_string()
}

/// social-cli's `slugify`: lowercase, non-`[a-z0-9]` runs → `-`, trim the
/// ends, cap at 60 characters (ASCII output, so chars == bytes).
pub fn slugify(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut in_run = false;
    for ch in title.to_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
            in_run = false;
        } else if !in_run {
            out.push('-');
            in_run = true;
        }
    }
    let trimmed = out.trim_matches('-');
    let mut slug = String::from(trimmed);
    if slug.chars().count() > 60 {
        slug = slug.chars().take(60).collect();
        // keep the cap boundary clean of a dangling '-'
        while slug.ends_with('-') {
            slug.pop();
        }
    }
    slug
}

/// Options for [`build_record`]. Everything derivable is derived.
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    /// Fixed `publishedAt` (offline determinism); default: now.
    pub published_at: Option<String>,
    /// Fixed page `id` (offline determinism); default: a fresh UUIDv7,
    /// social-cli's page-id convention.
    pub page_id: Option<String>,
}

/// A built document record: the rkey it would be put under, its at-uri,
/// and the pure record JSON (the canonical artifact, puttable as-is).
#[derive(Debug, Clone)]
pub struct BuiltRecord {
    pub rkey: String,
    pub uri: String,
    pub record: serde_json::Value,
}

/// Build a `site.standard.document` record (wrapping `pub.leaflet.content`)
/// from frontmatter + markdown.
///
/// `publication` is the target publication's at-uri
/// (`at://<did>/site.standard.publication/<rkey>`); the document at-uri is
/// derived from its authority (documents live in the publication owner's
/// repo).
pub fn build_record(markdown: &str, publication: &str, options: &BuildOptions) -> Result<BuiltRecord, BuildError> {
    let (fm, body) = parse_frontmatter(markdown)?;
    let pub_uri = AtUri::parse(publication).map_err(|e| BuildError::BadPublication(publication.to_string(), e.to_string()))?;
    if pub_uri.collection != "site.standard.publication" {
        return Err(BuildError::BadPublication(
            publication.to_string(),
            format!("collection must be site.standard.publication, got {:?}", pub_uri.collection),
        ));
    }
    let title = fm.title.clone().ok_or(BuildError::MissingTitle)?;
    let rkey = match fm.slug.as_deref().filter(|s| !s.is_empty()) {
        Some(slug) => slug.to_string(),
        None => {
            let slug = slugify(&title);
            if slug.is_empty() {
                return Err(BuildError::EmptySlug { title });
            }
            slug
        }
    };

    let blocks = convert_markdown(body)?;
    let text_content = blocks_to_text_content(&blocks);
    let page_id = options.page_id.clone().unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    let published_at = match &options.published_at {
        Some(fixed) => Datetime::from_str(fixed.as_str()).map_err(|e| BuildError::Frontmatter(format!("publishedAt override {fixed:?}: {e}")))?,
        None => Datetime::now(),
    };

    let content = Content {
        blob_pages: None,
        blobs: None,
        pages: vec![ContentPagesItem::LinearDocument(Box::new(LinearDocument {
            blocks,
            id: Some(page_id.into()),
            theme: None,
            extra_data: None,
        }))],
        extra_data: None,
    };
    let document = Document::<DefaultStr> {
        bsky_post_ref: None,
        content: Some(content),
        contributors: None,
        cover_image: None,
        description: fm.description.filter(|d| !d.is_empty()).map(Into::into),
        path: Some(format!("/{rkey}").into()),
        preferences: None,
        published_at,
        site: UriValue::new_owned(publication).map_err(|e| BuildError::BadPublication(publication.to_string(), format!("site field: {e}")))?,
        tags: (!fm.tags.is_empty()).then(|| fm.tags.clone()).map(|t| t.into_iter().map(Into::into).collect()),
        text_content: Some(text_content.into()),
        theme: None,
        title: title.into(),
        updated_at: None,
        extra_data: None,
    };

    let mut record = serde_json::to_value(&document).map_err(|e| BuildError::Frontmatter(format!("serialization failed: {e}")))?;
    reinject_type_discriminators(&mut record);
    let uri = format!("at://{}/site.standard.document/{rkey}", pub_uri.authority);
    Ok(BuiltRecord { rkey, uri, record })
}

/// Re-add the `$type` discriminators jacquard's single-variant-union
/// codegen drops (see module docs): `content.$type =
/// "pub.leaflet.content"` and `pages[].blocks[].$type =
/// "pub.leaflet.pages.linearDocument#block"`. Shared by the record builder
/// and the draft-extraction converter so both emit the same wire shape.
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

/// Validate record JSON through the generated lexicon type: decode +
/// constraint checks (the pre-put gate for `publish run`).
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("record does not decode as a site.standard.document: {0}")]
    Decode(String),
    #[error("record fails site.standard.document lexicon validation: {0}")]
    Constraints(String),
}

pub fn validate_document_record(record: &serde_json::Value) -> Result<(), ValidationError> {
    let document: Document<DefaultStr> = serde_json::from_value(record.clone()).map_err(|e| ValidationError::Decode(e.to_string()))?;
    use jacquard_lexicon::schema::LexiconSchema;
    document.validate().map_err(|e| ValidationError::Constraints(e.to_string()))
}

/// A standalone `site` value: the HTTPS form Leaflet writes for drafts
/// with no publication linkage, `https://leaflet.pub/p/{did}`
/// (`publishLeaflet.ts:418-419`). Returns the embedded DID.
pub fn standalone_site_did(site: &str) -> Option<String> {
    let rest = site.strip_prefix("https://leaflet.pub/p/")?;
    let did = rest.trim_end_matches('/');
    (did.starts_with("did:") && !did.is_empty() && !did.contains('/')).then(|| did.to_string())
}

/// Derive the human-facing URL for a document rkey (experiment 03 §6):
/// the publication's own `url` field (custom domain) + `/{rkey}`, else the
/// `https://leaflet.pub/lish/{did}/{pubRkey}/{rkey}` deep permalink.
/// A standalone site value derives `{site}/{rkey}` — the
/// `/p/{did}/{rkey}` published-post route (Leaflet `getDocumentURL`: no
/// publication ⇒ base = doc.site + path).
///
/// (social-cli builds the domain form with the slug and the lish form
/// with the rkey; here `rkey` serves both — it equals the slug under the
/// default rule and is what the middleware resolves.)
pub fn derive_document_url(publication_at_uri: &str, publication_url: Option<&str>, rkey: &str) -> Result<String, BuildError> {
    if let Some(url) = publication_url.filter(|u| !u.is_empty()) {
        return Ok(format!("{}/{}", url.trim_end_matches('/'), rkey));
    }
    if standalone_site_did(publication_at_uri).is_some() {
        return Ok(format!("{}/{}", publication_at_uri.trim_end_matches('/'), rkey));
    }
    let uri = AtUri::parse(publication_at_uri).map_err(|e| BuildError::BadPublication(publication_at_uri.to_string(), e.to_string()))?;
    Ok(format!("https://leaflet.pub/lish/{}/{}/{}", uri.authority, uri.rkey, rkey))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SAMPLE_MD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../experiments/03-post-builder/sample.md"));
    const GOLDEN_RECORD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../experiments/03-post-builder/built-record.json"));
    const PUBLICATION: &str = "at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.publication/3molsbwkh522o";

    /// Golden test: building experiment 03's sample with the experiment's
    /// frozen timestamp + page id reproduces built-record.json exactly
    /// (structural JSON equality — every field, every facet byte offset).
    #[test]
    fn golden_sample_matches_experiment03() {
        let built = build_record(
            SAMPLE_MD,
            PUBLICATION,
            &BuildOptions {
                published_at: Some("2026-11-07T00:05:30.799Z".into()),
                page_id: Some("01a1b353-1c2f-7d4d-9284-3fa5d8782a4a".into()),
            },
        )
        .expect("sample builds");
        let expect: serde_json::Value = serde_json::from_str(GOLDEN_RECORD).unwrap();
        assert_eq!(built.record, expect, "record must equal experiments/03's built-record.json");
        assert_eq!(built.rkey, "springleaf-protocol-probe");
        assert_eq!(
            built.uri,
            "at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.document/springleaf-protocol-probe"
        );
        // path == "/" + rkey invariant
        assert_eq!(built.record["path"], json!("/springleaf-protocol-probe"));
    }

    #[test]
    fn generated_document_validates_against_lexicon_constraints() {
        let built = build_record(SAMPLE_MD, PUBLICATION, &BuildOptions::default()).unwrap();
        let document: Document = serde_json::from_value(built.record).expect("round-trips through the generated type");
        use jacquard_lexicon::schema::LexiconSchema;
        document.validate().expect("grapheme/length constraints hold");
    }

    #[test]
    fn url_derivation_both_forms() {
        // Publication-domain form: url field wins, trailing slash trimmed.
        assert_eq!(
            derive_document_url(PUBLICATION, Some("https://technautical.leaflet.pub/"), "springleaf-protocol-probe").unwrap(),
            "https://technautical.leaflet.pub/springleaf-protocol-probe"
        );
        // Fallback: /lish/{did}/{pubRkey}/{rkey}.
        assert_eq!(
            derive_document_url(PUBLICATION, None, "springleaf-protocol-probe").unwrap(),
            "https://leaflet.pub/lish/did:plc:zjbq26wybii5ojoypkso2mso/3molsbwkh522o/springleaf-protocol-probe"
        );
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

    #[test]
    fn frontmatter_parsing_and_tag_splitting() {
        let (fm, body) =
            parse_frontmatter("---\ntitle: T\ndescription: \"quoted desc\"\ntags: a, b ,c\nslug: my-slug\nother: ignored\n---\n\nbody text").unwrap();
        assert_eq!(fm.title.as_deref(), Some("T"));
        assert_eq!(fm.description.as_deref(), Some("quoted desc"));
        assert_eq!(fm.tags, vec!["a", "b", "c"]);
        assert_eq!(fm.slug.as_deref(), Some("my-slug"));
        assert_eq!(body.trim(), "body text");
        // No fence: all body.
        let (fm, body) = parse_frontmatter("just text").unwrap();
        assert_eq!(fm, Frontmatter::default());
        assert_eq!(body, "just text");
        // Unterminated fence: all body (social-cli falls back too).
        let (_, body) = parse_frontmatter("---\ntitle: T\n").unwrap();
        assert_eq!(body, "---\ntitle: T\n");
    }

    #[test]
    fn slugify_rules() {
        assert_eq!(slugify("Springleaf protocol probe"), "springleaf-protocol-probe");
        assert_eq!(slugify("  Émigré — cafés!  "), "migr-caf-s"); // non-ASCII collapses
        assert_eq!(slugify("a   b"), "a-b");
        assert_eq!(slugify("--x--"), "x");
        let long = "w".repeat(80);
        assert_eq!(slugify(&long).len(), 60);
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn rkey_falls_back_to_slugify_and_errors_cleanly() {
        // No slug: derived from title.
        let md = "---\ntitle: Hello World\n---\n\npara";
        let built = build_record(md, PUBLICATION, &BuildOptions::default()).unwrap();
        assert_eq!(built.rkey, "hello-world");
        assert_eq!(built.record["path"], json!("/hello-world"));
        // Title that cannot form a slug: explicit error.
        let md = "---\ntitle: !!!\n---\n\npara";
        assert!(matches!(
            build_record(md, PUBLICATION, &BuildOptions::default()),
            Err(BuildError::EmptySlug { .. })
        ));
        // Missing title entirely.
        assert!(matches!(
            build_record("no frontmatter", PUBLICATION, &BuildOptions::default()),
            Err(BuildError::MissingTitle)
        ));
        // Wrong publication collection.
        let bad = "at://did:plc:x/site.standard.document/other";
        assert!(matches!(
            build_record("---\ntitle: T\n---\n\np", bad, &BuildOptions::default()),
            Err(BuildError::BadPublication(..))
        ));
    }

    #[test]
    fn descriminator_reinjection_is_idempotent() {
        let built = build_record(
            SAMPLE_MD,
            PUBLICATION,
            &BuildOptions {
                published_at: Some("2026-11-07T00:05:30.799Z".into()),
                page_id: Some("01a1b353-1c2f-7d4d-9284-3fa5d8782a4a".into()),
            },
        )
        .unwrap();
        // Re-serializing through the generated type (which now sees the
        // discriminators as extra_data) and re-injecting changes nothing.
        let document: Document = serde_json::from_value(built.record.clone()).unwrap();
        let mut again = serde_json::to_value(&document).unwrap();
        reinject_type_discriminators(&mut again);
        assert_eq!(again, built.record);
    }
}
