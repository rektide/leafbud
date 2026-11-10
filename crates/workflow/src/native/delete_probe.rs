//! One-time, opt-in disposable live smokes: mint a fresh draft via the normal
//! push first-binding path, verify it, then delete it through the delete
//! capability and verify the token is gone. Marker-gated; never reruns.
//! Two variants: standalone (`publication: null`) and a draft inside the
//! actually configured publication (publication omitted → inherited), which
//! exercises the publication mint action transport. Published deletion is
//! deliberately NOT exercised live: test-publish authorization is deferred
//! until the operator confirms.
use super::*;

#[tokio::test]
#[ignore = "requires operator authorization for one disposable standalone draft (mint, verify, delete)"]
async fn authorized_disposable_draft_delete_smoke() -> Result<()> {
    ensure!(
        std::env::var("LEAFBUD_NATIVE_DELETE_PROBE").as_deref() == Ok("authorized-disposable-draft"),
        "explicit delete-probe authorization flag required"
    );
    let root = std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-delete"))?;
    std::fs::create_dir_all(&root)?;
    let remote = match crate::RemoteOptions::default().connect().await {
        Ok(remote) => remote,
        Err(error) => {
            eprintln!("SKIPPED: live credentials/environment unavailable: {error}");
            return Ok(());
        }
    };
    // No attempt, including a lost response, can mint a second draft.
    let marker = root.join("draft-smoke.started");
    let mut budget = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
        .context("delete-probe budget already consumed; inspect private probe state; never remint")?;
    use std::io::Write as _;
    writeln!(
        budget,
        "Operator authorized one disposable standalone draft for the delete smoke on 2026-11-10 (session ses_e39c64844ffeh98er3US2A0gQC); consumed before mint. No automatic cleanup/retry."
    )?;
    budget.sync_all()?;
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace)?;
    let path = workspace.join("disposable-delete-smoke.mdx");
    ensure!(!path.exists(), "prior disposable source still present; inspect before rerunning");
    let id = Uuid::new_v4();
    files::write(
        &path,
        None,
        format!(
            "---\nformat: leafbud-mdx/2\ntitle: \"Leafbud delete smoke\"\npublication: null\n\"@leafbud\": {{\"id\":\"{id}\",\"state\":\"draft\"}}\n---\n\n# Leafbud delete smoke.\n\nDisposable paragraph for the mint-verify-delete loop.\n"
        )
        .as_bytes(),
    )?;
    let mut context = crate::config::load(None, &Default::default(), None)?;
    context.workspace_root = workspace.clone();
    // Ordinary push first-binding path: mint, seed default, save content.
    let saved = remote
        .save_native(
            &context,
            &path,
            Selection { operands: vec![path.clone()] },
            crate::native::SaveOptions {
                yes: true,
                ..Default::default()
            },
        )
        .await?;
    ensure!(
        matches!(saved.outcome, "saved" | "unchanged"),
        "disposable draft did not save: {}",
        saved.message.unwrap_or_default()
    );
    let bound = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    let binding = bound.state.binding.clone().context("smoke source not bound after push")?;
    ensure!(binding.base_url == remote.client.base_url());
    // Delete through the new capability, converting the local source.
    let report = remote
        .delete_editor(
            &context,
            &path,
            Selection { operands: vec![path.clone()] },
            DeleteOptions { yes: true, purge: false },
        )
        .await?;
    ensure!(
        report.outcome == "deleted",
        "delete smoke did not verify: {}",
        serde_json::to_string(&report).unwrap_or_default()
    );
    ensure!(report.observed.as_ref().is_some_and(|observed| observed.token_gone), "token not observed gone");
    // Independent re-observation: the token must not resolve.
    ensure!(
        matches!(remote.client.get(binding.token).await, Err(leafbud_web::Error::NotFound)),
        "token still resolves after deletion"
    );
    let after = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    ensure!(
        after.state.binding.is_none() && after.state.state == "draft" && after.state.id == id,
        "local source not cleanly unbound"
    );
    ensure!(std::fs::read_to_string(&path)?.contains("Disposable paragraph"), "authored content lost");
    eprintln!(
        "delete smoke complete: minted {}, verified, deleted, token 404, local unbound; private receipt written",
        binding.token
    );
    std::fs::write(
        root.join("draft-smoke.result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "token": binding.token,
            "document_id": id,
            "deleted_at": jiff::Timestamp::now().to_string(),
            "outcome": report.outcome,
            "token_gone": true,
            "local": "unbound",
            "published_path_exercised": false
        }))?,
    )?;
    Ok(())
}

fn publication_receipt(root: &std::path::Path, value: serde_json::Value) -> Result<()> {
    std::fs::write(root.join("publication-draft-smoke.result.json"), serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires operator authorization for one disposable in-publication draft (mint, verify, delete)"]
async fn authorized_disposable_publication_draft_delete_smoke() -> Result<()> {
    ensure!(
        std::env::var("LEAFBUD_NATIVE_DELETE_PROBE").as_deref() == Ok("authorized-disposable-publication-draft"),
        "explicit delete-probe authorization flag required"
    );
    let root = std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-delete"))?;
    std::fs::create_dir_all(&root)?;
    let remote = match crate::RemoteOptions::default().connect().await {
        Ok(remote) => remote,
        Err(error) => {
            eprintln!("SKIPPED: live credentials/environment unavailable: {error}");
            return Ok(());
        }
    };
    // The probe target is the user's actually configured publication. Scratch
    // workspace isolation below must not change that resolution.
    let mut context = crate::config::load(None, &Default::default(), None)?;
    let publication_uri = context
        .effective
        .publication
        .clone()
        .context("no configured publication; the publication-draft delete smoke has no target")?;
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace)?;
    context.workspace_root = workspace.clone();
    // No attempt, including a lost response, can mint a second publication draft.
    let marker = root.join("publication-draft-smoke.started");
    let mut budget = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
        .context("publication-draft probe budget already consumed; inspect private probe state; never remint")?;
    use std::io::Write as _;
    writeln!(
        budget,
        "Operator authorized one disposable in-publication draft for the delete smoke on 2026-11-10 (session ses_e399f9ec9ffe4LZ1pGURvkeU6L); consumed before mint. No automatic cleanup/retry."
    )?;
    budget.sync_all()?;
    let path = workspace.join("disposable-publication-delete-smoke.mdx");
    ensure!(!path.exists(), "prior disposable source still present; inspect before rerunning");
    let id = Uuid::new_v4();
    // `publication` is deliberately omitted: Inherit resolves through config to
    // the configured publication, exercising the publication mint action (not
    // the standalone /new redirect transport).
    files::write(
        &path,
        None,
        format!(
            "---\nformat: leafbud-mdx/2\ntitle: \"Leafbud publication delete smoke\"\n\"@leafbud\": {{\"id\":\"{id}\",\"state\":\"draft\"}}\n---\n\n# Leafbud publication delete smoke.\n\nDisposable paragraph for the publication mint-verify-delete loop.\n"
        )
        .as_bytes(),
    )?;
    let unbound = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    ensure!(
        matches!(resolved_context(&unbound, &context), MintContext::Publication(ref uri) if uri == &publication_uri),
        "resolved mint context is not the configured publication; refusing to spend the one-shot budget on another transport"
    );
    // Ordinary push first-binding path: publication mint action, seed/default
    // verification and adoption, then the content save.
    let saved = match remote
        .save_native(
            &context,
            &path,
            Selection { operands: vec![path.clone()] },
            crate::native::SaveOptions {
                yes: true,
                ..Default::default()
            },
        )
        .await
    {
        Ok(saved) => saved,
        Err(error) => {
            publication_receipt(
                &root,
                serde_json::json!({
                    "document_id": id,
                    "publication": publication_uri,
                    "stage": "save-error",
                    "error": format!("{error:#}"),
                    "recorded_at": jiff::Timestamp::now().to_string(),
                }),
            )?;
            return Err(error.context("publication-draft save failed after budget consumption; attempt state retained; no retry"));
        }
    };
    if !matches!(saved.outcome, "saved" | "unchanged") {
        publication_receipt(
            &root,
            serde_json::json!({
                "document_id": id,
                "publication": publication_uri,
                "stage": "save-not-saved",
                "outcome": saved.outcome,
                "message": saved.message,
                "recorded_at": jiff::Timestamp::now().to_string(),
            }),
        )?;
        anyhow::bail!(
            "publication draft did not save (outcome {}, message {:?}); pending attempt retained, no retry: recover deliberately, then inspect candidates",
            saved.outcome,
            saved.message
        );
    }
    let bound = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    let binding = bound.state.binding.clone().context("publication smoke source not bound after push")?;
    ensure!(binding.base_url == remote.client.base_url());
    ensure!(
        binding.publication.as_deref() == Some(publication_uri.as_str()),
        "bound publication association differs from the configured target"
    );
    ensure!(binding.uri.is_none(), "fresh publication draft must not carry a published record");
    // Preview-first discipline: without --yes this observes and reports only.
    let preview = remote
        .delete_editor(
            &context,
            &path,
            Selection { operands: vec![path.clone()] },
            DeleteOptions { yes: false, purge: false },
        )
        .await?;
    ensure!(
        preview.outcome == "would-delete" && preview.record.is_none(),
        "publication draft delete preview did not verify: {}",
        serde_json::to_string(&preview).unwrap_or_default()
    );
    // Delete through the capability, converting the local source.
    let report = match remote
        .delete_editor(
            &context,
            &path,
            Selection { operands: vec![path.clone()] },
            DeleteOptions { yes: true, purge: false },
        )
        .await
    {
        Ok(report) => report,
        Err(error) => {
            publication_receipt(
                &root,
                serde_json::json!({
                    "token": binding.token,
                    "document_id": id,
                    "publication": publication_uri,
                    "stage": "delete-error",
                    "error": format!("{error:#}"),
                    "recorded_at": jiff::Timestamp::now().to_string(),
                }),
            )?;
            return Err(error.context("publication-draft delete failed after verified save; source preserved; deliberate rerun required"));
        }
    };
    if report.outcome != "deleted" {
        publication_receipt(
            &root,
            serde_json::json!({
                "token": binding.token,
                "document_id": id,
                "publication": publication_uri,
                "stage": "delete-not-deleted",
                "report": serde_json::to_value(&report)?,
                "recorded_at": jiff::Timestamp::now().to_string(),
            }),
        )?;
        anyhow::bail!(
            "publication draft delete did not verify (outcome {}): {}",
            report.outcome,
            serde_json::to_string(&report).unwrap_or_default()
        );
    }
    ensure!(report.observed.as_ref().is_some_and(|observed| observed.token_gone), "token not observed gone");
    ensure!(
        report.observed.as_ref().is_some_and(|observed| observed.record_gone.is_none()),
        "a fresh draft must not have a public record to observe"
    );
    // Independent re-observation: the token must not resolve.
    ensure!(
        matches!(remote.client.get(binding.token).await, Err(leafbud_web::Error::NotFound)),
        "token still resolves after deletion"
    );
    let after = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    ensure!(
        after.state.binding.is_none() && after.state.state == "draft" && after.state.id == id,
        "local source not cleanly unbound"
    );
    // First binding persists the publication association as an explicit
    // top-level key; unbinding removes the binding, not that authored context.
    ensure!(
        after.publication == leafbud_document::Publication::At(publication_uri.clone()),
        "unbound source did not keep its explicit publication context: {:?}",
        after.publication
    );
    ensure!(std::fs::read_to_string(&path)?.contains("Disposable paragraph"), "authored content lost");
    eprintln!(
        "publication delete smoke complete: minted {} in the configured publication, verified, previewed, deleted, token 404, local unbound; private receipt written",
        binding.token
    );
    publication_receipt(
        &root,
        serde_json::json!({
            "token": binding.token,
            "document_id": id,
            "publication": publication_uri,
            "deleted_at": jiff::Timestamp::now().to_string(),
            "outcome": report.outcome,
            "token_gone": true,
            "local": "unbound",
            "published_path_exercised": false,
            "publication_draft_path_exercised": true
        }),
    )?;
    Ok(())
}

/// Continuation after an uncertain publication mint: the dispatched action's
/// response was undecodable, so the authorized disposable draft's locator was
/// recovered by read-only inspection (inspect.py) and recorded privately. This
/// selects THAT locator — the probe's own in-window product, proven absent
/// from the prior publication inventory — and finishes the authorized
/// materialize → delete → verify loop. It creates nothing and never remints.
#[tokio::test]
#[ignore = "requires explicit selection of the delete-probe's own in-window publication draft"]
async fn authorized_selected_publication_draft_delete_smoke() -> Result<()> {
    ensure!(
        std::env::var("LEAFBUD_NATIVE_DELETE_PROBE").as_deref() == Ok("authorized-selected-publication-draft"),
        "explicit delete-probe selection flag required"
    );
    let root = std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-delete"))?;
    let candidate: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join("candidate.private.json"))?)?;
    ensure!(
        candidate["finding"].as_str().is_some_and(|text| text.contains("one new in-window draft")),
        "private candidate record does not evidence exactly one in-window draft"
    );
    let token: Uuid = candidate["candidate"]["token"].as_str().context("candidate token missing")?.parse()?;
    let prior: Vec<String> = candidate["prior_inventory_tokens"]
        .as_array()
        .context("prior inventory tokens missing")?
        .iter()
        .filter_map(|value| value.as_str().map(str::to_owned))
        .collect();
    ensure!(
        !prior.iter().any(|known| known == &token.to_string()),
        "selected token is pre-existing; refusing"
    );
    let publication_uri = candidate["publication"].as_str().context("candidate publication missing")?.to_owned();
    let mut context = crate::config::load(None, &Default::default(), None)?;
    ensure!(
        context.effective.publication.as_deref() == Some(publication_uri.as_str()),
        "configured publication differs from the recorded attempt target"
    );
    let workspace = root.join("workspace");
    context.workspace_root = workspace.clone();
    let path = workspace.join("disposable-publication-delete-smoke.mdx");
    let mut source_document = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    let attempt_id = match &source_document.state.pending {
        Some(attempt) => attempt.id,
        None => anyhow::bail!("no pending mint attempt; selection is not applicable"),
    };
    match &source_document.state.pending.as_ref().context("attempt")?.effect {
        Effect::Mint {
            publication,
            stage: MintStage::Dispatched,
            ..
        } if publication.publication() == Some(publication_uri.as_str()) => {}
        other => anyhow::bail!("pending attempt is not the dispatched publication mint: {other:?}"),
    }
    // One-shot selection marker; a rerun fails the Dispatched-stage check above.
    let selected_marker = root.join("publication-draft-smoke.selected");
    let mut selected_budget = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&selected_marker)
        .context("publication-draft selection already performed; never reselect")?;
    use std::io::Write as _;
    writeln!(
        selected_budget,
        "Selected the delete-probe's own in-window draft (session ses_e399f9ec9ffe4LZ1pGURvkeU6L) after an undecodable mint response; evidence in candidate.private.json. No new creation."
    )?;
    selected_budget.sync_all()?;
    let remote = match crate::RemoteOptions::default().connect().await {
        Ok(remote) => remote,
        Err(error) => {
            eprintln!("SKIPPED: live credentials/environment unavailable: {error}");
            return Ok(());
        }
    };
    let selected = remote
        .select_mint_locator(&context, &path, Selection { operands: vec![path.clone()] }, attempt_id, token)
        .await?;
    publication_receipt(
        &root,
        serde_json::json!({
            "stage": "selected",
            "token": token,
            "selection_report": serde_json::to_value(&selected)?,
            "recorded_at": jiff::Timestamp::now().to_string(),
        }),
    )?;
    ensure!(
        matches!(selected.outcome, crate::native::BindingOutcome::PendingMaterialization),
        "selected locator did not verify/prepare; retained, no remint/retry: {}",
        serde_json::to_string(&selected).unwrap_or_default()
    );
    // Normal product save: recovers the pending mint, materializes the keyed
    // authored content, settles the binding.
    let saved = remote
        .save_native(
            &context,
            &path,
            Selection { operands: vec![path.clone()] },
            crate::native::SaveOptions {
                yes: true,
                ..Default::default()
            },
        )
        .await
        .context("publication-draft materializing save failed; attempt retained; no retry")?;
    ensure!(
        matches!(saved.outcome, "saved" | "unchanged"),
        "publication draft did not materialize (outcome {}, message {:?})",
        saved.outcome,
        saved.message
    );
    source_document = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    let binding = source_document.state.binding.clone().context("not bound after selected save")?;
    ensure!(binding.token == token, "bound token differs from the selected locator");
    ensure!(
        binding.publication.as_deref() == Some(publication_uri.as_str()) && binding.uri.is_none(),
        "binding is not the fresh in-publication draft"
    );
    // Preview-first discipline, then the authorized delete.
    let preview = remote
        .delete_editor(
            &context,
            &path,
            Selection { operands: vec![path.clone()] },
            DeleteOptions { yes: false, purge: false },
        )
        .await?;
    ensure!(
        preview.outcome == "would-delete" && preview.record.is_none(),
        "publication draft delete preview did not verify: {}",
        serde_json::to_string(&preview).unwrap_or_default()
    );
    let report = remote
        .delete_editor(
            &context,
            &path,
            Selection { operands: vec![path.clone()] },
            DeleteOptions { yes: true, purge: false },
        )
        .await
        .context("publication-draft delete failed; source preserved; deliberate rerun required")?;
    ensure!(
        report.outcome == "deleted" && report.token == token,
        "publication draft delete did not verify: {}",
        serde_json::to_string(&report).unwrap_or_default()
    );
    ensure!(
        report
            .observed
            .as_ref()
            .is_some_and(|observed| observed.token_gone && observed.record_gone.is_none()),
        "token not observed gone (or a record was unexpectedly involved)"
    );
    ensure!(
        matches!(remote.client.get(token).await, Err(leafbud_web::Error::NotFound)),
        "token still resolves after deletion"
    );
    let after = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    ensure!(
        after.state.binding.is_none() && after.state.state == "draft" && after.state.id == source_document.state.id,
        "local source not cleanly unbound"
    );
    ensure!(
        after.publication == leafbud_document::Publication::At(publication_uri.clone()),
        "unbound source did not keep its explicit publication context: {:?}",
        after.publication
    );
    ensure!(std::fs::read_to_string(&path)?.contains("Disposable paragraph"), "authored content lost");
    eprintln!(
        "publication delete smoke complete via selected locator {}: materialized, previewed, deleted, token 404, local unbound",
        token
    );
    publication_receipt(
        &root,
        serde_json::json!({
            "token": token,
            "document_id": source_document.state.id,
            "publication": publication_uri,
            "deleted_at": jiff::Timestamp::now().to_string(),
            "outcome": report.outcome,
            "token_gone": true,
            "local": "unbound",
            "published_path_exercised": false,
            "publication_draft_path_exercised": true,
            "mint_recovery": "selected-locator after undecodable action response"
        }),
    )?;
    Ok(())
}
