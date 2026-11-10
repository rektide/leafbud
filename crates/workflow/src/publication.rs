//! Read-only native publication foundation. No workflow execution, key/default
//! persistence, attempt or receipt creation, mint, save, delivery or public PUT.
use crate::{RemoteReader, config, inventory, native, published};
use anyhow::{Context, Result, ensure};
use leafbud_atproto::AtUri;
use leafbud_document::{Document, Publication};
use leafbud_records::projection;
use serde::Serialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
pub mod size;

/// A summary of actual native behavior, not an executable plan or authorization.
#[derive(Debug, Serialize)]
pub struct Preview {
    pub standing: &'static str,
    pub mode: &'static str,
    pub keys_to_prepare: usize,
    pub native_pages_after_save: usize,
    pub editor: &'static str,
    pub editor_effects: BTreeMap<String, usize>,
    pub inline_size: Option<size::Witness>,
    pub actor: Option<String>,
    pub owner: Option<String>,
    pub publication: Option<String>,
    pub uri: Option<String>,
    pub path: Value,
    pub date: Value,
    pub send_email: bool,
    pub bluesky_share: bool,
    pub native_effects: Vec<&'static str>,
    pub blockers: Vec<String>,
}

/// Complete local validation, including unsupported release body, happens before
/// any remote await. A preview can report unsaved/unbound source without inventing
/// native IDs. It neither opens the stable-ID change lock nor returns authority.
pub async fn preview(
    context: &config::Context,
    path: &Path,
    selection: native::Selection,
    remote: Option<&RemoteReader>,
    now: jiff::Timestamp,
) -> Result<Preview> {
    let path = std::fs::canonicalize(if path.is_absolute() {
        path.to_owned()
    } else {
        context.workspace_root.join(path)
    })?;
    let original = std::fs::read(&path)?;
    let text = std::str::from_utf8(&original)?;
    let parsed = leafbud_mdx::parse_with_spans(text)?;
    let desired = native::prepared_preview(text)?;
    desired.state.validate_lifecycle().map_err(anyhow::Error::msg)?;
    ensure!(
        desired.state.pending.is_none(),
        "prior native attempt retained; resolve it before a new publication preview"
    );
    let missing_pages = projection::preflight(&desired)?;
    let check = || -> Result<()> {
        ensure!(
            std::fs::read(&path).ok().as_deref() == Some(original.as_slice()),
            "source changed during publication preview; all evidence stale, source preserved"
        );
        let mut inventory = inventory::scan(&context.workspace_root, None)?;
        let mut operands: Vec<_> = selection
            .operands
            .iter()
            .map(|operand| {
                let operand = if operand.is_absolute() {
                    operand.clone()
                } else {
                    context.workspace_root.join(operand)
                };
                std::fs::canonicalize(&operand).unwrap_or(operand)
            })
            .collect();
        operands.push(path.clone());
        inventory.include_operands(&operands);
        ensure!(
            !inventory.duplicates.contains(&path),
            "duplicate document/native/public identity; preview refused"
        );
        Ok(())
    };
    check()?;
    let publication = match &desired.state.binding {
        Some(binding) => {
            ensure!(
                desired.publication == binding.publication.clone().map(Publication::At).unwrap_or(Publication::Standalone),
                "source publication differs from verified binding; no default retargeting"
            );
            binding.publication.clone()
        }
        None => match &desired.publication {
            Publication::Inherit => context.effective.publication.clone(),
            Publication::Standalone => None,
            Publication::At(uri) => Some(uri.clone()),
        },
    };
    let uri = desired.state.binding.as_ref().and_then(|binding| binding.uri.clone());
    let mut result = Preview {
        standing: "internal-preflight-only; execution-not-wired",
        mode: if uri.is_some() { "update-same-URI" } else { "first-release" },
        keys_to_prepare: parsed.unkeyed().len() + usize::from(parsed.main_page_provisional()),
        native_pages_after_save: missing_pages,
        editor: if desired.state.binding.is_none() {
            "bind-and-save-required"
        } else {
            "not-observed"
        },
        editor_effects: BTreeMap::new(),
        inline_size: None,
        actor: None,
        owner: None,
        publication,
        uri,
        path: Value::Null,
        date: Value::Null,
        send_email: false,
        bluesky_share: false,
        native_effects: vec![
            "PDS putRecord",
            "appview document/linkage upserts",
            "conditional Pro publish-version snapshot",
            "path revalidation",
            "publish analytics",
        ],
        blockers: vec!["execution/recovery and verified release patch authority are not implemented".into()],
    };
    if result.publication.is_some() {
        // This runs for EVERY publication, even an image-free update, and can
        // delete old files not mentioned by today's editor. No suppression flag.
        result.native_effects.extend([
            "gated-image stale-copy cleanup",
            "schedule deletion",
            "actor contributor upsert",
            "conditional first-release newsletter skipped-slot claim",
        ]);
        result
            .blockers
            .push("native publication always prunes gated-image storage; no no-cleanup action flag or verified empty-history witness".into());
    }
    if result.uri.is_none() {
        result.native_effects.push("conditional first-release mention notifications");
    }
    normalize(&desired, None, now, &mut result)?;
    let Some(binding) = &desired.state.binding else {
        result
            .blockers
            .push("first binding/defaults/owner and actual saved native page identities require future guarded mint/save; none performed".into());
        result
            .blockers
            .push("native inline size not witnessed before first binding; needs actual seed/compiled cumulative state, not semantic estimate".into());
        return Ok(result);
    };
    let Some(remote) = remote else {
        result
            .blockers
            .push("native identity/rights/owner, effect fields and action lookup not observed".into());
        result.blockers.push("native inline size not witnessed offline".into());
        return Ok(result);
    };
    ensure!(
        binding.base_url == remote.client.base_url(),
        "native binding origin differs; preview never follows ambient defaults"
    );
    let observed = remote.observe_editor(&desired).await;
    check()?;
    let observed = observed.map_err(|error| anyhow::anyhow!("native publication preview {}: {}", error.kind, error.message))?;
    ensure!(
        observed
            .data
            .metadata
            .permission_token_rights
            .iter()
            .any(|right| right.token.as_deref() == Some(binding.token.to_string().as_str())
                && right.entity_set.as_deref() == Some(&binding.entity_set)
                && right.read == Some(true)
                && right.write == Some(true)),
        "native editor write rights not verified"
    );
    let compiled = native::compile_preview(&desired, &observed)?;
    match size::compiled(&desired, &observed.snapshot.facts, &compiled) {
        Ok(size) => {
            if !size.no_offload_at_witness {
                result.blockers.push(format!(
                    "native pages JSON upper bound {} exceeds {} bytes; no inline/no-offload proof",
                    size.pages_json_utf8_upper_bound, size.inline_threshold
                ));
            }
            result.inline_size = Some(size);
        }
        Err(error) => result.blockers.push(format!("native inline-size profile unproven: {error:#}")),
    }
    result.editor = if compiled.mutations.is_empty() {
        "saved-current-input"
    } else {
        "save-required"
    };
    result.editor_effects = native::preview_effects(&compiled.mutations);
    if !compiled.mutations.is_empty() && !observed.agreement.endpoints_equal && observed.agreement.remote != crate::editor_state::Change::Unchanged {
        result
            .blockers
            .push("editor drift/no current baseline requires a separate --take local decision before saving".into());
    }
    let associated = published::association(&observed.data)?;
    if let Some(association) = &associated {
        // Cover/share references and record preferences can survive an update
        // even when draft preferences are empty. Validate the existing output,
        // not only today's editor/body, before claiming a covered release.
        projection::from_record(&association.record, &desired)?;
    }
    normalize(&desired, associated.as_ref().map(|association| &association.record), now, &mut result)?;
    inspect_effect_fields(&observed.data, &mut result)?;
    let preparation = remote.client.prepare_publication(binding.token).await;
    check()?;
    let preparation = preparation?;
    result.actor = Some(preparation.actor().into());
    verify_owner(&desired, &observed.data, preparation.actor(), &mut result)?;
    // Discovery/identity requests can race native linkage or effect preferences.
    // Keep the entire raw metadata witness, not only published editor hashes
    // (which deliberately exclude reply/schedule/current public metadata).
    let confirmed = remote.client.get(binding.token).await;
    check()?;
    ensure!(
        confirmed?.raw == observed.data.raw,
        "native metadata/effect context changed during preview; rerun, no effect performed"
    );
    let confirmed = remote.client.pull(binding.token).await;
    check()?;
    ensure!(
        crate::import::remote_hash(&observed.data, &confirmed?) == crate::import::remote_hash(&observed.data, &observed.snapshot),
        "native facts changed during preview; size/input witness invalidated, no effect performed"
    );
    Ok(result)
}

fn normalize(document: &Document, existing: Option<&Value>, now: jiff::Timestamp, result: &mut Preview) -> Result<()> {
    let authored_path = document.extra.get("path").filter(|value| !value.is_null());
    if let Some(path) = authored_path {
        ensure!(path.is_string(), "path intent must be text or null");
    }
    if let Some(uri) = &result.uri {
        let uri = AtUri::parse(uri)?;
        ensure!(
            uri.collection == "site.standard.document",
            "native release profile does not support legacy collection"
        );
        let path = format!("/{}", uri.rkey);
        result.path = json!({"kind":"fixed-by-existing-rkey","actual":path,"authored":authored_path,"normalizes":authored_path.is_some_and(|value| value != &json!(path))});
    } else {
        result.path = json!({"kind":"server-chosen-rkey","actual":null,"authored":authored_path,"normalizes":authored_path.is_some()});
    }
    let authored = document.extra.get("publishedAt").filter(|value| !value.is_null());
    let date = authored.or_else(|| existing.and_then(|record| record.get("publishedAt")));
    result.date = if let Some(date) = date {
        let date = date.as_str().context("publishedAt must be a timestamp or null")?;
        let timestamp: jiff::Timestamp = date.parse()?;
        if authored.is_some() && timestamp > now {
            json!({"kind":"native-future-clamped-to-server-now","authored":authored,"actual":null,"scheduling":false})
        } else {
            json!({"kind":if authored.is_some(){"explicit"}else{"preserve-observed"},"actual":timestamp.to_string(),"wire":authored.map(|_| date)})
        }
    } else if result.uri.is_some() {
        json!({"kind":"preserve-existing-date-needs-observation","actual":null})
    } else {
        json!({"kind":"server-now-on-first-release","actual":null})
    };
    Ok(())
}

fn inspect_effect_fields(data: &leafbud_web::LeafletData, result: &mut Preview) -> Result<()> {
    // Only this token's action context is usable. Sibling-only adoption does
    // not prove the publish action's .eq(leaflet, leaflet_id) lookup will work.
    for field in ["reply_drafts", "publication_scheduled_posts"] {
        let values = data.raw[field]
            .as_array()
            .with_context(|| format!("native {field} evidence missing; cannot assume absent"))?;
        if !values.is_empty() {
            result
                .blockers
                .push(format!("native {field} is nonempty; publication would submit a reply or cancel a schedule"));
        }
    }
    let table = if result.publication.is_some() {
        "leaflets_in_publications"
    } else {
        "leaflets_to_documents"
    };
    let links = data.raw[table].as_array().context("native action linkage evidence missing")?;
    ensure!(
        links.len() <= 1 && (result.publication.is_none() || links.len() == 1),
        "native action needs a unique direct publication draft row; sibling association alone is insufficient"
    );
    for link in links {
        ensure!(link["leaflet"] == data.metadata.id.to_string(), "native action direct row/token mismatch");
        if link.get("email_only").is_some_and(|value| value != false && !value.is_null()) {
            result.blockers.push("native email-only draft cannot be published as a post".into());
        }
        // The action inherits these when omitted; do not smuggle them through
        // a model that currently cannot compare preferences/discovery output.
        ensure!(link.get("preferences").is_some(), "native action preferences witness missing");
        if !link["preferences"].is_null() && link["preferences"] != json!({}) {
            result
                .blockers
                .push("native nondefault preferences/discovery output has no supported release mapping".into());
        }
    }
    Ok(())
}

fn verify_owner(document: &Document, data: &leafbud_web::LeafletData, actor: &str, result: &mut Preview) -> Result<()> {
    let binding = document.state.binding.as_ref().context("binding")?;
    let owner = if let Some(publication) = &binding.publication {
        let links = data.raw["leaflets_in_publications"].as_array().context("direct publication linkage missing")?;
        ensure!(
            links.len() == 1 && links[0]["publication"] == publication.as_str(),
            "action context/publication mismatch"
        );
        let container = &links[0]["publications"];
        ensure!(
            container["uri"] == publication.as_str(),
            "publication owner must come from verified native join"
        );
        let owner = container["identity_did"].as_str().context("native publication owner missing")?;
        let contributors = container["publication_contributors"]
            .as_array()
            .context("publication membership witness missing")?;
        ensure!(
            owner == actor || contributors.iter().any(|row| row["contributor_did"] == actor && row["confirmed"] == true),
            "actor is neither native publication owner nor confirmed contributor"
        );
        result
            .blockers
            .push("publication actor is upserted into byline; verify actual post-save contributor set/order before dispatch".into());
        owner.to_owned()
    } else {
        actor.to_owned()
    };
    if let Some(uri) = &binding.uri {
        ensure!(
            AtUri::parse(uri)?.authority == owner,
            "native publication owner/actor differs from existing public URI owner"
        );
    }
    result.owner = Some(owner);
    Ok(())
}

#[cfg(test)]
mod tests;
