//! Deliberate, preview-first deletion of a bound native editor through
//! Leaflet's own `deleteLeaflet` action (and `deletePost` first for a
//! published document). Verified by observation, never by retry; local
//! sources convert to unbound drafts explicitly afterward.
use super::*;
use crate::release::PublicReader;
use leafbud_document::BlockKind;
use leafbud_web::Error as WebError;
use leafbud_web::delete::PostOutcome;

#[derive(Debug, Default)]
pub struct DeleteOptions {
    pub yes: bool,
    pub purge: bool,
}

/// Post-execution observation: what independent reads now report missing.
/// `None` until an effect was dispatched or discovered already deleted.
#[derive(Debug, Serialize)]
pub struct ObservedGone {
    pub token_gone: bool,
    /// Public record observation for a published document (`None` = not applicable).
    pub record_gone: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DeleteReport {
    pub version: u8,
    pub path: PathBuf,
    pub document_id: Uuid,
    /// Local lifecycle before the operation, from the parsed source.
    pub lifecycle: &'static str,
    pub outcome: &'static str,
    pub token: Uuid,
    pub entity_set: String,
    /// The exact public record that dies, for a published document.
    pub record: Option<String>,
    /// Advisory: locally authored image blocks whose native facts the server
    /// queues for image-storage cleanup. The server's own entity scan decides.
    pub image_blocks: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed: Option<ObservedGone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<&'static str>,
    pub message: Option<String>,
}

impl RemoteReader {
    pub async fn delete_editor(&self, context: &config::Context, path: &Path, selection: Selection, options: DeleteOptions) -> Result<DeleteReport> {
        let mut source = Source::open(context, path, selection)?;
        let state = source.document.state.clone();
        match self.delete_locked(&mut source, state, &options).await {
            Ok(report) => Ok(report),
            Err(error) => {
                let outcome = if source.check().is_err() { "source-changed" } else { "blocked" };
                Ok(DeleteReport {
                    version: 1,
                    path: source.path.clone(),
                    document_id: source.document.state.id,
                    lifecycle: lifecycle(&source.document),
                    outcome,
                    token: source.document.state.binding.as_ref().map(|b| b.token).unwrap_or_default(),
                    entity_set: source.document.state.binding.as_ref().map(|b| b.entity_set.clone()).unwrap_or_default(),
                    record: source.document.state.binding.as_ref().and_then(|b| b.uri.clone()),
                    image_blocks: 0,
                    observed: None,
                    local: Some("preserved"),
                    message: Some(format!("delete refused before any effect; source preserved: {error:#}")),
                })
            }
        }
    }

    async fn delete_locked(&self, source: &mut Source<'_>, state: State, options: &DeleteOptions) -> Result<DeleteReport> {
        let binding = state
            .binding
            .clone()
            .context("no native binding; an unbound local draft has no server objects to delete")?;
        let lifecycle = if binding.uri.is_some() { "published" } else { "draft" };
        let mut report = DeleteReport {
            version: 1,
            path: source.path.clone(),
            document_id: state.id,
            lifecycle,
            outcome: "blocked",
            token: binding.token,
            entity_set: binding.entity_set.clone(),
            record: binding.uri.clone(),
            image_blocks: source
                .document
                .pages
                .iter()
                .flat_map(|page| &page.blocks)
                .filter(|block| matches!(block.kind, BlockKind::Image { .. }))
                .count(),
            observed: None,
            local: Some("preserved"),
            message: None,
        };
        if state.pending.is_some() || state.publication_attempt.is_some() {
            report.outcome = "pending";
            report.message = Some("unresolved native attempt retained; run recover first; deletion waits".into());
            return Ok(report);
        }
        ensure!(
            binding.base_url == self.client.base_url(),
            "binding belongs to another origin; select it with --base-url explicitly"
        );
        let store = lock::Store::new(source.context)?;
        let _binding_guard = store
            .binding(&binding.base_url, binding.token)?
            .context("native locator busy in another operation")?;
        let observed = match self.client.get(binding.token).await {
            Ok(data) => Some(data),
            Err(WebError::NotFound) => None,
            Err(error) => return Err(error).context("observing the bound editor before deletion"),
        };
        let public = PublicReader::new(source.context.effective.pds.as_deref())?;
        let Some(data) = observed else {
            // The editor is already gone: a prior uncertain response, an
            // out-of-band UI deletion, or upstream maintenance. A published
            // document must ALSO have lost its record before local unbinding.
            let mut record_gone = None;
            if let Some(uri) = &binding.uri {
                let gone = matches!(public.fetch(uri).await, Err(crate::release::FetchError::Missing));
                record_gone = Some(gone);
                ensure!(
                    gone,
                    "native editor is gone but the public record still resolves; local source preserved until that divergence is understood"
                );
            }
            report.observed = Some(ObservedGone { token_gone: true, record_gone });
            report.outcome = if options.yes { "unbound-local" } else { "remote-deleted" };
            report.local = Some(if options.yes {
                local_convert(source, options)?
            } else if options.purge {
                "would-purge"
            } else {
                "would-unbind"
            });
            report.message = Some("editor already deleted remotely; only the local unbind conversion was requested/executed; no effect dispatched".into());
            return Ok(report);
        };
        if binding.uri.is_some() {
            match public.fetch(binding.uri.as_deref().expect("published uri")).await {
                Ok(_) => {}
                Err(crate::release::FetchError::Missing) => {
                    anyhow::bail!("public record already missing while the editor still resolves; resolve the divergence before deleting")
                }
                Err(error) => return Err(anyhow::anyhow!("public record preflight failed: {:?}", error)),
            }
        }
        verify_deletable(&binding, &data)?;
        if !options.yes {
            report.outcome = "would-delete";
            report.local = Some(if options.purge { "would-purge" } else { "would-unbind" });
            report.message = Some(preview_message(&report));
            return Ok(report);
        }
        source.check()?;
        let preparation = self
            .client
            .prepare_delete(binding.token)
            .await
            .context("discovering native delete actions (no effect yet)")?;
        source.check()?;
        // Native order for a published post: the record (and its rows/linkage)
        // die first, then the editor's entity set and permission token.
        if let Some(uri) = &binding.uri {
            match self.client.invoke_delete_post(&preparation, uri).await {
                Ok(PostOutcome::Deleted) => {}
                Ok(PostOutcome::Failed(failure)) => {
                    report.outcome = "refused";
                    report.message = Some(format!(
                        "deletePost refused server-side ({:?}); no retry, source preserved; the record and editor were NOT deleted",
                        failure.kind
                    ));
                    return Ok(report);
                }
                Err(error) => {
                    // The response is lost/ambiguous, not a typed refusal.
                    // Observe the record before concluding anything.
                    let gone = matches!(public.fetch(uri).await, Err(crate::release::FetchError::Missing));
                    report.observed = Some(ObservedGone {
                        token_gone: false,
                        record_gone: Some(gone),
                    });
                    if !gone {
                        report.outcome = "uncertain";
                        report.message = Some(format!(
                            "deletePost outcome uncertain ({error}); record still resolves; no retry, source preserved; reinspect and rerun deliberately"
                        ));
                        return Ok(report);
                    }
                    // The record is gone. Continue only through a coherent
                    // post-record editor observation, never a second dispatch.
                    let after = self.client.get(binding.token).await.context("reobserving editor after uncertain deletePost")?;
                    verify_record_deleted(&binding, &after)?;
                }
            }
        }
        let dispatched = self.client.invoke_delete_leaflet(&preparation, &data).await;
        let token_gone = matches!(self.client.get(binding.token).await, Err(WebError::NotFound));
        let record_gone = match &binding.uri {
            Some(uri) => Some(matches!(public.fetch(uri).await, Err(crate::release::FetchError::Missing))),
            None => None,
        };
        report.observed = Some(ObservedGone { token_gone, record_gone });
        if !token_gone {
            report.outcome = "uncertain";
            report.message = Some(match dispatched {
                // A void success with a surviving token is just as uncertain
                // as a lost response; only observation decides.
                Ok(()) => "delete reported success but the token still resolves; no retry, source preserved; reinspect and rerun deliberately".into(),
                Err(error) => format!("delete dispatched but the token still resolves ({error}); no retry, source preserved; reinspect and rerun deliberately"),
            });
            return Ok(report);
        }
        if binding.uri.is_some() && record_gone != Some(true) {
            report.outcome = "uncertain";
            report.message = Some("editor deleted but the public record still resolves or could not be observed; no retry, source preserved".into());
            return Ok(report);
        }
        report.outcome = "deleted";
        report.local = Some(local_convert(source, options)?);
        report.message = Some(if binding.uri.is_some() {
            "public record and native editor verified gone by observation; local source converted; appview/search propagation may lag the record's disappearance".into()
        } else {
            "native editor verified gone by observation; local source converted; no public record existed".into()
        });
        Ok(report)
    }
}

/// Verify the live editor is exactly the locally bound one and that the
/// native action would not silently no-op. Returns nothing; every mismatch
/// is a hard block.
fn verify_deletable(binding: &Binding, data: &LeafletData) -> Result<()> {
    ensure!(
        data.metadata.id == binding.token && data.metadata.root_entity.as_deref() == Some(binding.root.as_str()),
        "observed editor token/root differs from the local binding"
    );
    ensure!(
        data.metadata
            .permission_token_rights
            .iter()
            .any(|r| r.token.as_deref() == Some(binding.token.to_string().as_str())
                && r.entity_set.as_deref() == Some(binding.entity_set.as_str())
                && r.read == Some(true)
                && r.write == Some(true)),
        "observed token rights lack a writable right for the bound entity set; the native action would silently do nothing"
    );
    let association = import::association(data)?;
    ensure!(
        association == binding.publication,
        "observed publication association differs from the local binding"
    );
    let published = data.observed_state() == LeafletState::Published;
    ensure!(published == binding.uri.is_some(), "observed lifecycle differs from the local binding");
    if let Some(uri) = &binding.uri {
        let observed = crate::published::association(data)?.context("published editor has no document association")?;
        ensure!(&observed.uri == uri, "observed public record URI differs from the local binding");
    }
    Ok(())
}

/// After the public record is verified gone, the editor must still be exactly
/// the bound one — now observed as a draft with no document association.
fn verify_record_deleted(binding: &Binding, data: &LeafletData) -> Result<()> {
    ensure!(
        data.metadata.id == binding.token && data.metadata.root_entity.as_deref() == Some(binding.root.as_str()),
        "observed editor token/root differs from the local binding"
    );
    ensure!(
        data.metadata
            .permission_token_rights
            .iter()
            .any(|r| r.token.as_deref() == Some(binding.token.to_string().as_str())
                && r.entity_set.as_deref() == Some(binding.entity_set.as_str())
                && r.read == Some(true)
                && r.write == Some(true)),
        "observed token rights lack a writable right for the bound entity set"
    );
    ensure!(
        data.observed_state() == LeafletState::Draft,
        "record is gone but the editor still reports published linkage"
    );
    ensure!(
        crate::published::association(data)?.is_none(),
        "record is gone but a document association remains"
    );
    ensure!(
        import::association(data)? == binding.publication,
        "observed publication association differs from the local binding"
    );
    Ok(())
}

fn lifecycle(document: &Document) -> &'static str {
    if document.state.binding.as_ref().is_some_and(|binding| binding.uri.is_some()) {
        "published"
    } else {
        "draft"
    }
}

fn preview_message(report: &DeleteReport) -> String {
    let mut lines = vec![format!(
        "preview only; nothing runs without --yes. This permanently deletes, for everyone: the native editor's complete entity set {} and permission token {} (rights and linkage rows cascade),",
        report.entity_set, report.token
    )];
    lines.push("and queues cleanup of any image blobs referenced by its image facts.".into());
    if report.record.is_some() {
        lines.push("PUBLISHED deletion additionally removes the public PDS record (both document collections are attempted server-side), its documents rows and publication linkage — a bigger, publicly visible action. The editor dies after the record, mirroring the native UI's order. Appview/search propagation may lag.".into());
    } else {
        lines.push("No public record exists; nothing public is affected.".into());
    }
    match report.local {
        Some("would-purge") => lines.push("Afterward the local file is REMOVED (--purge).".into()),
        _ => lines.push("Afterward the local file converts to an unbound draft: binding, native associations and receipts are dropped; authored content, keys, names, comments and the local UUID are preserved.".into()),
    }
    lines.join(" ")
}

/// Deliberate local conversion after verified deletion, under the held
/// stable-ID lock and original-byte guard. `--purge` removes the file.
fn local_convert(source: &mut Source<'_>, options: &DeleteOptions) -> Result<&'static str> {
    if options.purge {
        std::fs::remove_file(&source.path).context("purging the local source after verified deletion")?;
        return Ok("purged");
    }
    let text = leafbud_mdx::unbind_deleted(&source.text)?;
    source.check()?;
    files::write(&source.path, Some(&source.bytes), text.as_bytes())?;
    source.bytes = text.clone().into_bytes();
    source.text = text;
    source.document = leafbud_mdx::parse(&source.text)?;
    Ok("unbound")
}
