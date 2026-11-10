//! One-time, opt-in disposable live smoke: mint a fresh standalone draft via
//! the normal push first-binding path, verify it, then delete it through the
//! new capability and verify the token is gone. Marker-gated; never reruns.
//! Published deletion is deliberately NOT exercised live: test-publish
//! authorization is deferred until the operator confirms.
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
