//! `springleaf drafts export` — pull a leaflet editor draft (read-only) and
//! write its canonical record to `drafts/<slug>.json`
//! (`design/draft-extraction.glm53m.md`, extraction half).
//!
//! Pull → convert → validate through the generated lexicon type → write the
//! PURE record (nothing added). The file is byte-compatible with
//! `publish build` output, so an exported draft can flow straight into
//! `publish run`. Network reads only: `get_leaflet_data` (root entity +
//! publication linkage), one `pull` (the fact graph), and — only for
//! unlinked drafts — `/home` for the DID the standalone `site` fallback
//! needs. Nothing here writes to leaflet.pub.
//!
//! Exit posture: 0 ok, 1 operational error, 2 refusal (existing slug
//! without `--overwrite`).

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use springleaf_drafts::LeafletData;
use springleaf_drafts::extract::{ExtractContext, ExtractOptions, Warn};
use springleaf_drafts::pull::PullSnapshot;
use springleaf_drafts::{Error as DraftsError, classify};

use crate::workspace::Workspace;

/// `drafts export` output (human summary + the `--json` schema v1).
#[derive(Debug, Serialize)]
pub(crate) struct ExportOutcome {
    pub slug: String,
    pub path: String,
    pub title: String,
    pub blocks: BlocksSummary,
    pub warns: Vec<Warn>,
}

#[derive(Debug, Serialize)]
pub(crate) struct BlocksSummary {
    pub converted: usize,
    pub skipped: usize,
    pub pages: usize,
}

/// Pull → convert → validate → write `drafts/<slug>.json` (pure record).
/// Refuses an existing slug without `overwrite` (exit-2 posture is the
/// caller's; here it is an error).
pub(crate) fn export_and_write(
    ws: &Workspace,
    snapshot: &PullSnapshot,
    data: &LeafletData,
    did: Option<&str>,
    options: &ExtractOptions,
    overwrite: bool,
) -> Result<ExportOutcome> {
    let context = ExtractContext {
        data,
        did,
        publication_tags: snapshot.publication_tags(),
    };
    let extracted = springleaf_drafts::extract::extract(snapshot.facts.clone(), &context, options)
        .map_err(|err| match err {
            DraftsError::Structure(detail) => anyhow::anyhow!("draft structure error: {detail}"),
            other => anyhow::Error::new(other),
        })
        .context("converting the draft fact graph")?;
    springleaf_leaflet::validate_document_record(&extracted.record)
        .with_context(|| format!("extracted record for slug {:?} fails lexicon validation", extracted.slug))?;
    springleaf_atproto::validate_rkey(&extracted.slug).with_context(|| format!("slug {:?} is not a valid rkey", extracted.slug))?;

    let draft_path = ws.drafts_dir().join(format!("{}.json", extracted.slug));
    if draft_path.exists() && !overwrite {
        bail!("{} already exists; pass --overwrite to replace it", draft_path.display());
    }
    std::fs::create_dir_all(ws.drafts_dir()).with_context(|| format!("creating {}", ws.drafts_dir().display()))?;
    let record_text = format!("{}\n", serde_json::to_string_pretty(&extracted.record)?);
    std::fs::write(&draft_path, &record_text).with_context(|| format!("writing {}", draft_path.display()))?;
    if ws.posted_dir().join(format!("{}.json", extracted.slug)).exists() {
        eprintln!(
            "warning: posted/{}.json already exists; sync will classify this export as local-newer",
            extracted.slug
        );
    }

    Ok(ExportOutcome {
        slug: extracted.slug,
        path: draft_path.display().to_string(),
        title: extracted.title,
        blocks: BlocksSummary {
            converted: extracted.blocks_converted,
            skipped: extracted.blocks_skipped,
            pages: extracted.record["content"]["pages"].as_array().map(Vec::len).unwrap_or(0),
        },
        warns: extracted.warns,
    })
}

/// The command body: the read-only network reads, then
/// [`export_and_write`].
pub(crate) async fn drafts_export_cmd(
    client: &springleaf_drafts::LeafletClient,
    token_id: &str,
    root: Option<PathBuf>,
    slug: Option<String>,
    overwrite: bool,
    json: bool,
) -> Result<i32> {
    let global = crate::config::global_layer()?;
    let ws = Workspace::resolve(root.as_deref(), &global)?;
    let data = client
        .get_leaflet_data(token_id)
        .await
        .with_context(|| format!("fetching leaflet data for {token_id}"))?;
    let snapshot = client
        .pull(token_id)
        .await
        .with_context(|| format!("pulling the draft fact graph for {token_id}"))?;

    // The standalone site fallback needs the operator's DID; the
    // publication-linked form does not (one extra GET only when needed).
    let linked = classify(&data.leaflets_in_publications);
    let did = match linked.publication.as_ref().and_then(|p| p.uri.as_deref()) {
        Some(_) => None,
        None => {
            let html = client.get_home().await.context("fetching /home for the identity (standalone site fallback)")?;
            match springleaf_drafts::identity_from_home(&html)? {
                Some(identity) => identity.did,
                None => {
                    eprintln!("error: no DID resolvable from /home and no publication linkage — cannot derive the record `site`");
                    return Ok(1);
                }
            }
        }
    };

    let outcome = export_and_write(&ws, &snapshot, &data, did.as_deref(), &ExtractOptions { published_at: None, slug }, overwrite)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        println!("title:  {}", outcome.title);
        println!("slug:   {}", outcome.slug);
        println!("draft:  {}", outcome.path);
        println!(
            "blocks: {} converted, {} skipped, {} page(s)",
            outcome.blocks.converted, outcome.blocks.skipped, outcome.blocks.pages
        );
        if outcome.warns.is_empty() {
            println!("warns:  none");
        } else {
            println!("warns:  {}", outcome.warns.len());
            for warn in &outcome.warns {
                println!("  [{}] {}: {}", warn.block_type, warn.entity, warn.reason);
            }
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::TempDir;
    use serde_json::Value;
    use serde_json::json;

    macro_rules! exp02_fixture {
        ($file:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../experiments/02-draft-internals/fixtures/",
                $file
            ))
        };
    }

    fn fixture_export() -> (PullSnapshot, LeafletData, Option<String>) {
        let fixture: Value = serde_json::from_str(exp02_fixture!("export-pull-doc.json")).unwrap();
        let snapshot = springleaf_drafts::pull::parse_pull_snapshot(&fixture["response"].to_string()).unwrap();
        let data: LeafletData = serde_json::from_value(fixture["leaflet_data"].clone()).unwrap();
        let did = fixture["did"].as_str().map(str::to_string);
        (snapshot, data, did)
    }

    fn options(slug: Option<String>) -> ExtractOptions {
        ExtractOptions {
            published_at: Some("2026-11-07T00:05:30.799Z".into()),
            slug,
        }
    }

    #[test]
    fn export_writes_the_pure_golden_record() {
        let tmp = TempDir::new("export");
        let ws = Workspace::resolve(Some(&tmp), &crate::config::Config::default()).unwrap();
        let (snapshot, data, did) = fixture_export();
        let outcome = export_and_write(&ws, &snapshot, &data, did.as_deref(), &options(None), false).unwrap();
        assert_eq!(outcome.slug, "synthetic-title");
        assert_eq!(outcome.blocks.converted, 2);
        assert_eq!(outcome.blocks.skipped, 0);
        assert!(outcome.warns.is_empty());

        // The file is exactly the committed golden record + newline.
        let text = std::fs::read_to_string(tmp.join("drafts/synthetic-title.json")).unwrap();
        assert!(text.ends_with('\n') && !text.ends_with("\n\n"));
        let expected: Value = serde_json::from_str(exp02_fixture!("export-record-doc.json")).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), expected);
        // Pure: no local fields.
        assert!(!text.contains("_local"));
        springleaf_leaflet::validate_document_record(&serde_json::from_str(&text).unwrap()).unwrap();
    }

    #[test]
    fn export_refuses_existing_and_overwrites() {
        let tmp = TempDir::new("export-overwrite");
        let ws = Workspace::resolve(Some(&tmp), &crate::config::Config::default()).unwrap();
        let (snapshot, data, did) = fixture_export();
        export_and_write(&ws, &snapshot, &data, did.as_deref(), &options(None), false).unwrap();
        let err = export_and_write(&ws, &snapshot, &data, did.as_deref(), &options(None), false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("already exists"), "{err}");
        export_and_write(&ws, &snapshot, &data, did.as_deref(), &options(None), true).unwrap_or_else(|err| panic!("overwrite replaces: {err}"));
    }

    #[test]
    fn slug_override_and_validation_gate() {
        let tmp = TempDir::new("export-slug");
        let ws = Workspace::resolve(Some(&tmp), &crate::config::Config::default()).unwrap();
        let (snapshot, data, did) = fixture_export();
        let outcome = export_and_write(&ws, &snapshot, &data, did.as_deref(), &options(Some("custom-slug".into())), false).unwrap();
        assert_eq!(outcome.slug, "custom-slug");
        let record: Value = serde_json::from_str(&std::fs::read_to_string(tmp.join("drafts/custom-slug.json")).unwrap()).unwrap();
        assert_eq!(record["path"], json!("/custom-slug"));
        // An invalid slug (rkey rules) is refused before any write.
        let err = export_and_write(&ws, &snapshot, &data, did.as_deref(), &options(Some("NOT VALID".into())), false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a valid rkey"), "{err}");
        assert!(!tmp.join("drafts").join("NOT VALID.json").exists());
    }
}
