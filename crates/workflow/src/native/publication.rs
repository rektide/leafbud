//! Save-first native publication. The stable source lock spans editor save,
//! dispatch checkpoint, observation and settlement. No uncertain action retries.
use super::*;
use crate::{publication::Preview, release::PublicReader};
use leafbud_atproto::AtUri;
use leafbud_document::{PublicationDate, PublicationStage};
use leafbud_records::projection;
use leafbud_web::publication::{Arguments, Outcome, Preparation};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct PublishOptions {
    pub yes: bool,
    pub take_local: bool,
}

impl RemoteReader {
    /// Finite observation/settlement only. An undispatched continuation can be
    /// cleared deliberately; an uncertain dispatch cannot be cleared or replayed.
    pub async fn recover_publication(&self, context: &config::Context, path: &Path, selection: Selection, settle: bool) -> Result<PublishReport> {
        let mut source = Source::open(context, path, selection)?;
        match self.recover_publication_locked(&mut source, settle).await {
            Ok(report) => Ok(report),
            Err(error) => {
                let mut report = PublishReport::new(if source.check().is_err() { "source-changed" } else { "uncertain" });
                report.message = Some(format!("publication evidence retained; observation/settlement incomplete: {error:#}"));
                Ok(report)
            }
        }
    }

    pub(super) async fn recover_publication_locked(&self, source: &mut Source<'_>, settle: bool) -> Result<PublishReport> {
        let Some(attempt) = source.document.state.publication_attempt.clone() else {
            return Ok(PublishReport::new("no-attempt"));
        };
        let mut report = PublishReport::new("uncertain");
        let binding = source.document.state.binding.as_ref().context("binding")?;
        let store = lock::Store::new(source.context)?;
        let _binding_guard = store
            .binding(&binding.base_url, binding.token)?
            .context("native binding busy during recovery")?;
        if matches!(attempt.stage, PublicationStage::EditorAchieved) {
            if !source_matches(source, &attempt) {
                report.outcome = "source-changed";
                report.message = Some("source intent changed after editor achievement; release undispatched, fingerprints retained".into());
                return Ok(report);
            }
            let binding = source.document.state.binding.as_ref().context("binding")?;
            let observed = self.native_observation(binding, source.document.state.writer.as_ref(), &[]).await?;
            source.check()?;
            let compiled = save::compile_reusing(&source.document, &observed.data, &observed.snapshot, &Default::default(), &[])?;
            ensure!(compiled.mutations.is_empty(), "saved editor changed; release undispatched, no effects replayed");
            report.outcome = "editor-achieved-release-undispatched";
            if settle {
                let mut state = source.document.state.clone();
                state.publication_attempt = None;
                source.checkpoint(state)?;
            }
            report.message = Some(
                "editor achieved; no release was dispatched. This call never publishes; repeat publish as a separate explicit action after recovery".into(),
            );
            return Ok(report);
        }
        if let Err(error) = self.verify_publication_locked(source, settle, None, &mut report).await {
            if source.check().is_err() {
                report.outcome = "source-changed";
            }
            report.message = Some(format!("publication evidence retained; no retry or new intent accepted: {error:#}"));
        }
        Ok(report)
    }

    async fn verify_publication_locked(
        &self,
        source: &mut Source<'_>,
        settle: bool,
        action: Option<&leafbud_web::publication::Published>,
        report: &mut PublishReport,
    ) -> Result<()> {
        let attempt = source.document.state.publication_attempt.clone().context("publication attempt missing")?;
        let PublicationStage::Dispatched {
            owner,
            core_hash: expected_hash,
            old_uri,
            date,
            contributor,
            ..
        } = &attempt.stage
        else {
            anyhow::bail!("publication not dispatched")
        };
        let binding = source.document.state.binding.as_ref().context("binding")?.clone();
        let data = self.client.get(binding.token).await?;
        let associated = crate::published::association(&data)?;
        let Some(associated) = associated else {
            report.outcome = if source_matches(source, &attempt) { "uncertain" } else { "source-changed" };
            report.message = Some(
                "no associated public record observed; delayed request may still arrive, including a pre-send crash. No retry or negative-finality inference"
                    .into(),
            );
            return Ok(());
        };
        report.uri = Some(associated.uri.clone()); // actual locator even on source race
        ensure!(
            old_uri.as_ref().is_none_or(|old| old == &associated.uri),
            "native association changed established public URI"
        );
        let parsed = AtUri::parse(&associated.uri)?;
        ensure!(
            parsed.authority == *owner && parsed.collection == "site.standard.document",
            "association owner/collection mismatch"
        );
        if let Some(action) = action {
            ensure!(action.uri == associated.uri, "action/native association URI mismatch");
        }
        let public = PublicReader::new(source.context.effective.pds.as_deref())?;
        let fetched = public.fetch_content_verified(&associated.uri).await?;
        if !source_matches(source, &attempt) || source.check().is_err() {
            report.outcome = "source-changed";
            report.message = Some("actual association retained in report; source intent changed, old dispatch stays uncertain, no new intent accepted".into());
            return Ok(());
        }
        let record = fetched.record();
        report.outcome = "partial-unsupported";
        let actual_date = record.value["publishedAt"].as_str().context("actual publication date missing")?;
        ensure!(record.value["path"] == format!("/{}", parsed.rkey), "native path differs from fixed /rkey");
        ensure!(
            actual_date.parse::<jiff::Timestamp>()? <= jiff::Timestamp::now() || matches!(date, PublicationDate::Exact(_)),
            "normalized native now is in the future"
        );
        let mut desired = candidate(&source.document, owner, contributor.as_deref(), date, Some((&associated.uri, actual_date)))?;
        ensure!(core_hash(&desired)? == *expected_hash, "pending complete public core differs from source");
        let expected = projection::working(&desired)?.value;
        ensure!(
            projection::from_record(&record.value, &desired)?.value == expected,
            "content-verified public meaning differs from full dispatched intent"
        );
        ensure!(
            projection::from_record(&associated.record, &desired)?.value == expected,
            "native association record differs from verified PDS meaning"
        );
        if let Some(action) = action {
            ensure!(
                projection::from_record(&action.record, &desired)?.value == expected,
                "action record differs from verified PDS meaning"
            );
        }
        let snapshot = self.client.pull(binding.token).await?;
        let fresh = import::build(&binding.base_url, &data, &snapshot)?;
        crate::editor_state::same_identity(desired.state.binding.as_ref().expect("candidate bound"), &fresh)?;
        let merged = crate::editor_state::adopt(&desired, &fresh);
        ensure!(
            merged.editor_value() == desired.editor_value(),
            "current native editor differs from release intent; no receipt settlement"
        );
        let confirmed = self.client.get(binding.token).await?;
        source.check()?;
        ensure!(confirmed.raw == data.raw, "native association/editor changed during verification");
        desired.state.publication_attempt = None;
        desired.state.sync = Some(leafbud_document::Sync {
            profile: leafbud_document::EDITOR_PROFILE.into(),
            local_hash: import::hash(&desired.editor_value()),
            remote_hash: import::remote_hash(&data, &snapshot),
        });
        desired.state.released = Some(leafbud_document::Released {
            profile: leafbud_document::RELEASE_PROFILE.into(),
            input_hash: import::hash(&expected),
            cid: fetched.content().cid().to_owned(),
            path: format!("/{}", parsed.rkey),
            published_at: actual_date.parse::<jiff::Timestamp>()?.to_string(),
            confirmed_at: jiff::Timestamp::now().to_string(),
        });
        let text = leafbud_mdx::settle_release(&source.text, &desired)?;
        let store = lock::Store::new(source.context)?;
        let _public_guard = store.public(&associated.uri)?.context("public identity busy during release settlement")?;
        let mut inventory = inventory::scan(&source.context.workspace_root, None)?;
        inventory.include_operands(&source.operands);
        ensure!(
            inventory
                .public
                .get(&associated.uri)
                .is_none_or(|paths| paths.iter().all(|p| p == &source.path)),
            "verified public URI already belongs to another local source; no settlement"
        );
        source.check()?;
        if settle {
            files::write(&source.path, Some(&source.bytes), text.as_bytes())?;
            source.bytes = text.as_bytes().to_vec();
            source.text = text;
            source.document = desired;
        }
        report.outcome = if settle { "published" } else { "verified-published" };
        report.message = Some("full current public correspondence verified, not invocation causation or notification/delivery proof".into());
        Ok(())
    }
}

fn source_matches(source: &Source<'_>, attempt: &leafbud_document::PublicationAttempt) -> bool {
    import::hash(&source.document.authored_value()) == attempt.input_hash && import::hash(&source.document.editor_value()) == attempt.editor_hash
}

#[derive(Debug, Serialize)]
pub struct PublishReport {
    pub outcome: &'static str,
    pub effects: BTreeMap<String, usize>,
    pub uri: Option<String>,
    pub preview: Option<Preview>,
    pub message: Option<String>,
}
impl PublishReport {
    fn new(outcome: &'static str) -> Self {
        Self {
            outcome,
            effects: BTreeMap::new(),
            uri: None,
            preview: None,
            message: None,
        }
    }
}

impl RemoteReader {
    /// Reusable post-seed and pre-save guard. Default adoption is already explicit
    /// in source; no compiler/publication effect can precede this check.
    pub(super) async fn publication_guard(&self, source: &Source<'_>, data: &LeafletData, snapshot: &PullSnapshot) -> Result<(Preparation, Option<String>)> {
        let document = &source.document;
        projection::preflight(document)?;
        let binding = document.state.binding.as_ref().context("publication needs saved binding")?;
        let mut summary = Preview {
            publication: binding.publication.clone(),
            uri: binding.uri.clone(),
            ..Default::default()
        };
        crate::publication::inspect_effect_fields(data, &mut summary)?;
        let preparation = self.client.prepare_publication(binding.token).await?;
        source.check()?;
        crate::publication::verify_owner(document, data, preparation.actor(), &mut summary)?;
        ensure!(summary.blockers.is_empty(), "publication blocked: {}", summary.blockers.join("; "));
        if let Some(associated) = crate::published::association(data)? {
            projection::from_record(&associated.record, document)?;
        }
        let actual = import::build(&binding.base_url, data, snapshot)?;
        ensure!(
            document.extra.get("contributors") == actual.extra.get("contributors"),
            "authored contributors differ from actual saved native byline"
        );
        let mut contributors: Vec<String> = actual
            .extra
            .get("contributors")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_default();
        let contributor = if binding.publication.is_some() {
            let actor = preparation.actor().to_owned();
            if !contributors.contains(&actor) {
                contributors.push(actor.clone());
            }
            ensure!(
                contributors.len() <= 1,
                "native actor upsert would produce multi-contributor ordering; complete projection unavailable, no publication sent"
            );
            Some(actor)
        } else {
            ensure!(contributors.is_empty(), "standalone publisher does not project native contributors");
            None
        };
        let confirmed = self.client.get(binding.token).await?;
        source.check()?;
        ensure!(confirmed.raw == data.raw, "native publication context changed during action discovery");
        Ok((preparation, contributor))
    }

    pub async fn publish_native(&self, context: &config::Context, path: &Path, selection: Selection, options: PublishOptions) -> Result<PublishReport> {
        if !options.yes {
            let absolute = if path.is_absolute() {
                path.to_owned()
            } else {
                context.workspace_root.join(path)
            };
            if leafbud_mdx::parse(&std::fs::read_to_string(&absolute)?)?.state.publication_attempt.is_some() {
                return self.recover_publication(context, path, selection, false).await;
            }
            let mut report = PublishReport::new("would-publish");
            let preview = crate::publication::preview(context, path, selection, Some(self), jiff::Timestamp::now()).await?;
            if preview.editor == "bind-and-save-required" {
                report.outcome = "would-bind-and-publish";
            } else if !preview.blockers.is_empty() {
                report.outcome = "blocked";
            }
            report.uri = preview.uri.clone();
            report.preview = Some(preview);
            return Ok(report);
        }
        let mut source = Source::open(context, path, selection)?;
        let mut report = PublishReport::new("blocked");
        let result = self.publish_locked(&mut source, options.take_local, &mut report).await;
        if let Err(error) = result {
            report.outcome = if source.check().is_err() {
                "source-changed"
            } else if report.outcome == "partial-unsupported" {
                "partial-unsupported"
            } else if source
                .document
                .state
                .publication_attempt
                .as_ref()
                .is_some_and(|a| matches!(a.stage, PublicationStage::Dispatched { .. }))
            {
                "uncertain"
            } else if source.document.state.pending.is_some() {
                "pending"
            } else {
                "blocked"
            };
            report.message = Some(format!("publication not settled; evidence retained, no automatic retry: {error:#}"));
        }
        Ok(report)
    }

    async fn publish_locked(&self, source: &mut Source<'_>, take_local: bool, report: &mut PublishReport) -> Result<()> {
        validate_binding_intent(&source.text)?;
        let desired = save::prepared(&source.text)?;
        projection::preflight(&desired)?;
        let mut normalization = Preview {
            uri: desired.state.binding.as_ref().and_then(|b| b.uri.clone()),
            ..Default::default()
        };
        crate::publication::normalize(&desired, None, jiff::Timestamp::now(), &mut normalization)?;
        if source.document.state.publication_attempt.is_some() {
            *report = self.recover_publication_locked(source, true).await?;
            return Ok(()); // recovery only; NEVER dispatch again in this call
        }
        ensure!(
            source.document.state.pending.is_none(),
            "prior editor attempt retained; recover first, no publication performed"
        );
        let public = PublicReader::new(source.context.effective.pds.as_deref())?;
        // A repeat clean publication is a genuine zero-effect no-op, including
        // source bytes/mtime. Rehash before trusting the previous receipt.
        if let (Some(receipt), Some(uri)) = (
            &source.document.state.released,
            source.document.state.binding.as_ref().and_then(|b| b.uri.as_deref()),
        ) {
            let current = public.fetch_content_verified(uri).await?;
            source.check()?;
            if projection::working(&source.document).is_ok_and(|p| {
                import::hash(&p.value) == receipt.input_hash
                    && projection::from_record(&current.record().value, &source.document).is_ok_and(|out| out.value == p.value)
                    && current.record().cid.as_deref() == Some(&receipt.cid)
            }) {
                let observed = self.observe_editor(&source.document).await.map_err(|e| anyhow::anyhow!("{}", e.message))?;
                source.check()?;
                ensure!(
                    observed.agreement.endpoints_equal,
                    "editor drift despite clean release; inspect before publishing"
                );
                report.outcome = "unchanged";
                report.uri = Some(uri.to_owned());
                return Ok(());
            }
        }
        source.publication_join = Some(Uuid::new_v4());
        let saved = self
            .save_locked(
                source,
                SaveOptions {
                    yes: true,
                    take_local,
                    ..Default::default()
                },
            )
            .await?;
        report.effects = saved.effects;
        ensure!(
            matches!(saved.outcome, "saved" | "unchanged"),
            "editor not achieved: {} ({:?})",
            saved.outcome,
            saved.message
        );
        source.publication_join = None;
        let attempt = source
            .document
            .state
            .publication_attempt
            .clone()
            .context("atomic editor→publication checkpoint missing")?;
        ensure!(matches!(attempt.stage, PublicationStage::EditorAchieved), "release dispatch already uncertain");
        let binding = source.document.state.binding.clone().context("saved binding")?;
        let store = lock::Store::new(source.context)?;
        let _binding = store.binding(&binding.base_url, binding.token)?.context("native binding busy")?;
        let observed = self.native_observation(&binding, source.document.state.writer.as_ref(), &[]).await?;
        source.check()?;
        ensure!(
            source
                .document
                .state
                .sync
                .as_ref()
                .is_some_and(|s| s.remote_hash == import::remote_hash(&observed.data, &observed.snapshot)),
            "saved editor changed before publication"
        );
        let (preparation, contributor) = self.publication_guard(source, &observed.data, &observed.snapshot).await?;
        let compiled = save::compile_reusing(&source.document, &observed.data, &observed.snapshot, &Default::default(), &[])?;
        ensure!(compiled.mutations.is_empty(), "editor needs saving again; release undispatched");
        let size = crate::publication::size::compiled(&source.document, &observed.snapshot.facts, &compiled)?;
        ensure!(
            size.no_offload_at_witness,
            "fresh saved native pages JSON upper bound exceeds inline limit; no no-offload proof, release undispatched"
        );
        let old = match &binding.uri {
            Some(uri) => Some(public.fetch_content_verified(uri).await?),
            None => None,
        };
        source.check()?;
        let now = jiff::Timestamp::now();
        let existing = crate::published::association(&observed.data)?;
        let date = date_policy(&source.document, existing.as_ref().map(|a| &a.record), now)?;
        let owner = owner(&source.document, &observed.data, preparation.actor())?;
        let resolved = candidate(&source.document, &owner, contributor.as_deref(), &date, None)?;
        let args = arguments(&source.document)?;
        let stage = PublicationStage::Dispatched {
            owner,
            arguments_hash: import::hash(&args.wire()),
            core_hash: core_hash(&resolved)?,
            contributor,
            old_uri: binding.uri.clone(),
            old_cid: old.as_ref().and_then(|r| r.record().cid.clone()),
            date,
        };
        // Fresh full facts+metadata, not the narrower editor hash alone. This is
        // still not server CAS; concurrent edits after this read remain possible.
        let data = self.client.get(binding.token).await?;
        let facts = self.client.pull(binding.token).await?;
        source.check()?;
        ensure!(
            data.raw == observed.data.raw && import::remote_hash(&data, &facts) == import::remote_hash(&observed.data, &observed.snapshot),
            "native input raced before dispatch; release undispatched"
        );
        let mut state = source.document.state.clone();
        state.publication_attempt.as_mut().expect("joined").stage = stage;
        source.checkpoint(state)?; // MAY HAVE BEEN SENT, including pre-send crash
        let result = self.client.invoke_publication(&preparation, &args).await;
        report.effects.insert("native-publication-dispatch".into(), 1);
        match result {
            Ok(Outcome::Published(result)) => {
                report.uri = Some(result.uri.clone());
                ensure!(result.reply.is_none(), "unexpected native reply effect; retain uncertain publication");
                ensure!(
                    result.first_publish == binding.uri.is_none() && AtUri::parse(&result.uri)?.rkey == result.rkey,
                    "publication result identity/lifecycle mismatch"
                );
                self.verify_publication_locked(source, true, Some(&result), report).await?;
            }
            other => {
                report.message = Some(match other {
                    Ok(Outcome::Failed(f)) => format!("native reported {:?}; effects not ruled out; observation only, no retry", f.kind),
                    _ => "publication response unavailable; observation only, no retry".into(),
                });
                self.verify_publication_locked(source, true, None, report).await?;
            }
        }
        Ok(())
    }
}

fn arguments(document: &Document) -> Result<Arguments> {
    let binding = document.state.binding.as_ref().context("binding")?;
    Ok(Arguments {
        root_entity: binding.root.parse()?,
        leaflet_id: binding.token,
        publication_uri: binding.publication.clone(),
        title: document.title.clone(),
        description: document.description.clone().unwrap_or_default(),
        tags: document.tags.clone(),
        published_at: document
            .extra
            .get("publishedAt")
            .filter(|v| !v.is_null())
            .map(|v| v.as_str().context("invalid date").map(str::to_owned))
            .transpose()?,
    })
}
fn owner(document: &Document, data: &LeafletData, actor: &str) -> Result<String> {
    let mut preview = Preview {
        publication: document.state.binding.as_ref().context("binding")?.publication.clone(),
        ..Default::default()
    };
    crate::publication::verify_owner(document, data, actor, &mut preview)?;
    preview.owner.context("owner missing")
}
fn date_policy(document: &Document, existing: Option<&Value>, now: jiff::Timestamp) -> Result<PublicationDate> {
    let authored = document.extra.get("publishedAt").filter(|v| !v.is_null());
    let value = authored.or_else(|| existing.and_then(|v| v.get("publishedAt")));
    if let Some(value) = value {
        let date: jiff::Timestamp = value.as_str().context("publication date missing")?.parse()?;
        if authored.is_none() || date <= now {
            return Ok(PublicationDate::Exact(date.to_string()));
        }
    } else {
        ensure!(document.state.binding.as_ref().is_some_and(|b| b.uri.is_none()), "existing date unavailable");
    }
    Ok(PublicationDate::NativeNow {
        not_before: now.to_string(),
        not_after: None,
    })
}

/// Resolve ONLY the declared server-assigned fields. Sentinel values are used
/// transiently for the core digest; no fake URI/date is ever persisted as actual.
fn candidate(document: &Document, owner: &str, contributor: Option<&str>, date: &PublicationDate, actual: Option<(&str, &str)>) -> Result<Document> {
    let mut desired = document.clone();
    let binding = desired.state.binding.as_mut().context("binding")?;
    let uri = actual
        .map(|a| a.0.to_owned())
        .or_else(|| binding.uri.clone())
        .unwrap_or_else(|| format!("at://{owner}/site.standard.document/leafbud-unassigned"));
    let parsed = AtUri::parse(&uri)?;
    ensure!(
        parsed.authority == owner && parsed.collection == "site.standard.document",
        "public owner/collection mismatch"
    );
    ensure!(binding.uri.as_ref().is_none_or(|old| old == &uri), "publication changed established URI");
    binding.uri = Some(uri);
    desired.state.state = "published".into();
    desired.extra.insert("path".into(), json!(format!("/{}", parsed.rkey)));
    let actual_date = match date {
        PublicationDate::Exact(value) => {
            if let Some((_, actual)) = actual {
                ensure!(
                    actual.parse::<jiff::Timestamp>()? == value.parse::<jiff::Timestamp>()?,
                    "explicit/preserved date mismatch"
                );
            }
            document.extra.get("publishedAt").and_then(Value::as_str).unwrap_or(value).to_owned()
        }
        PublicationDate::NativeNow { not_before, not_after } => {
            if let Some((_, value)) = actual {
                let value: jiff::Timestamp = value.parse()?;
                ensure!(
                    value >= not_before.parse()?
                        && not_after
                            .as_ref()
                            .is_none_or(|end| end.parse::<jiff::Timestamp>().is_ok_and(|end| value <= end)),
                    "native normalized date outside dispatch interval"
                );
                value.to_string()
            } else {
                "2000-01-01T00:00:00Z".into()
            }
        }
    };
    desired.extra.insert("publishedAt".into(), json!(actual_date));
    if let Some(actor) = contributor {
        desired.extra.insert("contributors".into(), json!([actor]));
    }
    Ok(desired)
}

fn core_hash(document: &Document) -> Result<String> {
    let mut value = projection::working(document)?.value;
    // Date/path are separately checked by the exact normalization policy. URI
    // owner/site, page identities and EVERY other supported field stay hashed.
    value["metadata"].as_object_mut().expect("metadata").remove("path");
    value["metadata"].as_object_mut().expect("metadata").remove("publishedAt");
    Ok(import::hash(&value))
}

#[cfg(test)]
#[path = "publication_tests.rs"]
mod tests;
