//! Native request execution and terminal achievement verification. The public
//! save workflow supplies the complete pure editor compiler, not arbitrary bytes.
use super::*;
use leafbud_document::{EDITOR_PROFILE, Node, Protection, PushStage, Sync, WriterState};
use leafbud_protocol::{BatchLimits, NativeMutation, Target, WriteScope, Writer};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) use leafbud_editor::write::Compiled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecoveryOutcome {
    NoAttempt,
    PreparedNotSent,
    MintUncertain,
    PendingMaterialization,
    Uncertain,
    ResolvedPartial,
    DesiredVerified,
    SourceChanged,
}

#[derive(Debug, Serialize)]
pub struct RecoveryReport {
    pub outcome: RecoveryOutcome,
    pub settled: bool,
    pub acknowledged: Option<u64>,
    /// Positive facts found under explicitly selected IDs; zero is NOT proof
    /// of noncreation. No raw bodies or tokens in the incidental report.
    pub detached_facts_observed: usize,
    pub message: Option<String>,
    pub terminal: bool,
    pub source_matches_original: Option<bool>,
}
pub(super) fn report(outcome: RecoveryOutcome) -> RecoveryReport {
    RecoveryReport {
        outcome,
        settled: false,
        acknowledged: None,
        detached_facts_observed: 0,
        message: None,
        terminal: false,
        source_matches_original: None,
    }
}

pub(super) struct Observation {
    pub(super) data: LeafletData,
    pub(super) snapshot: PullSnapshot,
    detached_facts: usize,
    pub(super) verification: BTreeMap<String, serde_json::Value>,
}

impl RemoteReader {
    pub(super) async fn native_observation(&self, binding: &Binding, writer: Option<&WriterState>, entities: &[Uuid]) -> Result<Observation> {
        ensure!(binding.base_url == self.client.base_url(), "native binding belongs to another origin");
        let data = self.client.get(binding.token).await?;
        let snapshot = match writer {
            Some(w) => self.client.pull_writer(binding.token, wire_writer(w)).await?,
            None => self.client.pull(binding.token).await?,
        };
        if let Some(writer) = writer {
            ensure!(
                snapshot.acknowledged(writer.client_id).is_none_or(|id| id < writer.next_mutation_id),
                "native writer acknowledgment exceeds persisted reservation; explicit identity/sequence recovery required"
            );
        }
        observed_binding(binding, &data, &snapshot)?;
        ensure!(
            data.metadata
                .permission_token_rights
                .iter()
                .any(|r| r.entity_set.as_deref() == Some(&binding.entity_set)
                    && r.read == Some(true)
                    && r.write == Some(true)
                    && r.token.as_deref() == Some(binding.token.to_string().as_str())),
            "native write rights no longer verified"
        );
        let mut detached_facts = 0;
        let mut verification: BTreeMap<String, _> = snapshot
            .facts
            .iter()
            .map(|r| (r["id"].as_str().expect("validated fact ID").to_owned(), r.clone()))
            .collect();
        // Explicit-ID RPC is bounded to 128 roots per request. It does not
        // inherit the active graph's rooted visibility limit for detached IDs.
        for roots in entities.chunks(128) {
            let view = self.client.facts_for_entities(roots).await?;
            detached_facts += view.roots.values().map(Vec::len).sum::<usize>();
            for row in view.roots.values().flatten() {
                // SQL traversal annotation, not a stored fact field. Keep the
                // explicit view separate from the strict active observer.
                let mut row = row.clone();
                row.as_object_mut().context("explicit fact object")?.remove("root_id");
                let id = row["id"].as_str().context("explicit fact ID")?;
                if let Some(old) = verification.insert(id.to_owned(), row.clone()) {
                    ensure!(
                        leafbud_editor::witness(&[old], &BTreeMap::new(), serde_json::Value::Null)
                            == leafbud_editor::witness(std::slice::from_ref(&row), &BTreeMap::new(), serde_json::Value::Null),
                        "native fact changed across active/explicit verification reads"
                    );
                }
            }
        }
        let confirmed = self.client.get(binding.token).await?;
        observed_binding(binding, &confirmed, &snapshot)?;
        ensure!(
            confirmed
                .metadata
                .permission_token_rights
                .iter()
                .any(|r| r.token.as_deref() == Some(binding.token.to_string().as_str())
                    && r.entity_set.as_deref() == Some(&binding.entity_set)
                    && r.read == Some(true)
                    && r.write == Some(true)),
            "write rights changed during native observation"
        );
        ensure!(
            import::remote_hash(&confirmed, &snapshot) == import::remote_hash(&data, &snapshot),
            "metadata changed during native observation"
        );
        Ok(Observation {
            data,
            snapshot,
            detached_facts,
            verification,
        })
    }

    /// Finite observation/preparation/settlement. Never remints or replays.
    /// For a located mint, `settle` permits guarded first-binding/default source
    /// preparation, still pending materialization (not `settled`). For a push it
    /// permits tracking-only settlement with terminal complete correspondence.
    /// With `settle == false`, observation never changes the source.
    pub async fn recover_native(&self, context: &config::Context, path: &Path, selection: Selection, settle: bool) -> Result<RecoveryReport> {
        let mut source = Source::open(context, path, selection)?;
        self.recover_locked(&mut source, settle).await
    }

    pub(super) async fn recover_locked(&self, source: &mut Source<'_>, settle: bool) -> Result<RecoveryReport> {
        let Some(attempt) = source.document.state.pending.clone() else {
            return Ok(report(RecoveryOutcome::NoAttempt));
        };
        let source_matches =
            import::hash(&source.document.authored_value()) == attempt.input_hash && import::hash(&source.document.editor_value()) == attempt.desired_hash;
        if !source_matches
            && matches!(
                attempt.effect,
                Effect::Mint {
                    stage: MintStage::Dispatched | MintStage::Located { .. },
                    ..
                }
            )
        {
            return Ok(report(RecoveryOutcome::SourceChanged));
        }
        match &attempt.effect {
            Effect::Mint {
                stage: MintStage::Prepared, ..
            }
            | Effect::Push {
                stage: PushStage::Prepared, ..
            } => {
                let mut result = report(RecoveryOutcome::PreparedNotSent);
                if settle {
                    let mut state = source.document.state.clone();
                    state.pending = None;
                    source.checkpoint(state)?;
                    result.settled = true; // no new receipt; reservations survive
                }
                Ok(result)
            }
            Effect::Mint {
                stage: MintStage::Dispatched, ..
            } => Ok(report(RecoveryOutcome::MintUncertain)),
            Effect::Mint {
                base_url,
                publication,
                stage: MintStage::Located { token },
            } => {
                ensure!(base_url == self.client.base_url(), "pending mint belongs to another origin");
                let store = lock::Store::new(source.context)?;
                let _binding_guard = store.binding(base_url, *token)?.context("native locator busy")?;
                let data = self.client.get(*token).await?;
                let snapshot = self.client.pull(*token).await?;
                let binding = verify_seed(base_url, &data, &snapshot, publication)?;
                let confirmed = self.client.get(*token).await?;
                ensure!(
                    verify_seed(base_url, &confirmed, &snapshot, publication)? == binding
                        && import::remote_hash(&confirmed, &snapshot) == import::remote_hash(&data, &snapshot),
                    "mint identity/rights/metadata changed during recovery observation"
                );
                source.check()?;
                if let Some(existing) = &source.document.state.binding {
                    ensure!(existing == &binding, "pending mint binding changed");
                }
                if settle {
                    let verified = import::build(base_url, &confirmed, &snapshot)?;
                    source.prepare_binding(&verified)?;
                }
                // Seed/binding achievement is not authored materialization.
                Ok(report(RecoveryOutcome::PendingMaterialization))
            }
            Effect::Push {
                range,
                stage,
                created_entities,
                protection,
                targets,
            } => {
                let through = match stage {
                    PushStage::Dispatched { through } | PushStage::Partial { through } => *through,
                    PushStage::Prepared => unreachable!(),
                };
                let binding = source.document.state.binding.as_ref().context("pending push lacks binding")?;
                let writer = source.document.state.writer.as_ref().context("pending push lacks writer")?;
                let roots: Vec<_> = created_entities
                    .iter()
                    .chain(&protection.verification_roots)
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                let observed = self.native_observation(binding, Some(writer), &roots).await?;
                let ack = observed.snapshot.acknowledged(writer.client_id);
                let mut result = report(RecoveryOutcome::Uncertain);
                result.source_matches_original = Some(source_matches);
                result.acknowledged = ack;
                result.detached_facts_observed = observed.detached_facts;
                if source.check().is_err() {
                    result.outcome = RecoveryOutcome::SourceChanged;
                    return Ok(result);
                }
                // A lower/no acknowledgment after restart is not negative
                // finality: a delayed earlier request can still arrive.
                if ack.is_none_or(|id| id < through) {
                    result.message = Some("writer finality unavailable; old request may still arrive; no direction can override uncertainty".into());
                    return Ok(result);
                }
                result.terminal = true;
                if !source_matches {
                    result.outcome = RecoveryOutcome::ResolvedPartial;
                    result.message = Some("sent request is terminal, but source differs from original attempt; original achievement unverified; use recover --take remote|local --yes for a new decision".into());
                    return Ok(result);
                }
                if through != range.last {
                    result.outcome = RecoveryOutcome::ResolvedPartial;
                    if let Some(message) = &mut result.message {
                        message.push_str("; recover --take remote|local --yes makes a deliberate new decision");
                    }
                    result.message = Some("sent prefix is terminal; later reserved requests were never sent; retained for deliberate replan".into());
                } else {
                    let protected = protection_hash(&observed, protection);
                    let verified = verify_desired(&source.document, targets, &observed);
                    match verified {
                        Ok(state) if protected == protection.hash => {
                            result.outcome = RecoveryOutcome::DesiredVerified;
                            if settle {
                                source.checkpoint(state)?;
                                result.settled = true;
                            }
                            return Ok(result);
                        }
                        Ok(_) => result.message = Some("acknowledged but protected native scope changed; receipt not advanced".into()),
                        Err(error) => {
                            result.message = Some(format!(
                                "acknowledged does not mean applied; current desired/associations not verified: {error:#}"
                            ))
                        }
                    }
                    result.outcome = RecoveryOutcome::ResolvedPartial;
                }
                if settle {
                    let mut state = source.document.state.clone();
                    if let Effect::Push { stage, .. } = &mut state.pending.as_mut().expect("push").effect {
                        *stage = PushStage::Partial { through };
                    }
                    source.checkpoint(state)?;
                }
                Ok(result)
            }
        }
    }

    /// Controlled compiler seam for protocol/failure fixtures and opt-in probes.
    /// Production callers use save_native, which supplies the full compiler.
    #[cfg(test)]
    pub(crate) async fn execute_compiled(
        &self,
        context: &config::Context,
        path: &Path,
        selection: Selection,
        decision_witness: Option<&str>,
        compile: impl FnOnce(&Document, &LeafletData, &PullSnapshot) -> Result<Compiled>,
    ) -> Result<RecoveryReport> {
        let mut source = Source::open(context, path, selection)?;
        self.execute_locked(&mut source, decision_witness, compile).await
    }

    pub(super) async fn execute_locked(
        &self,
        source: &mut Source<'_>,
        decision_witness: Option<&str>,
        compile: impl FnOnce(&Document, &LeafletData, &PullSnapshot) -> Result<Compiled>,
    ) -> Result<RecoveryReport> {
        self.execute_resolving_locked(source, decision_witness, None, compile).await
    }

    pub(super) async fn execute_resolving_locked(
        &self,
        source: &mut Source<'_>,
        decision_witness: Option<&str>,
        replan: Option<super::resolve::Replan>,
        compile: impl FnOnce(&Document, &LeafletData, &PullSnapshot) -> Result<Compiled>,
    ) -> Result<RecoveryReport> {
        validate_binding_intent(&source.text)?;
        let mint = match source.document.state.pending.clone() {
            None => None,
            Some(attempt) if replan.as_ref().is_some_and(|r| r.original == attempt) => None,
            Some(attempt)
                if matches!(
                    &attempt.effect,
                    Effect::Mint {
                        stage: MintStage::Located { .. },
                        ..
                    }
                ) && source.document.state.binding.is_some()
                    && import::hash(&source.document.authored_value()) == attempt.input_hash
                    && import::hash(&source.document.editor_value()) == attempt.desired_hash =>
            {
                Some(attempt)
            }
            _ => anyhow::bail!("unresolved attempt; recover before compiling another effect"),
        };
        source.prepare_keys()?;
        let binding = source
            .document
            .state
            .binding
            .clone()
            .context("binding/materialization compiler join required")?;
        ensure!(
            source.document.publication == binding.publication.clone().map(Publication::At).unwrap_or(Publication::Standalone)
                || mint.is_some() && source.document.publication == Publication::Inherit,
            "source publication differs from confirmed binding; retargeting is unsupported, no push sent"
        );
        let mut writer = source.document.state.writer.clone().unwrap_or_else(WriterState::fresh);
        let roots = replan.as_ref().map(|r| r.roots.as_slice()).unwrap_or_default();
        let mut before = self.native_observation(&binding, Some(&writer), roots).await?;
        source.check()?;
        if let Some(replan) = &replan {
            ensure!(
                super::resolve::observation_hash(&before) == replan.resolution.observed_hash,
                "native drift across recovery decision; old attempt retained"
            );
        }
        let hash = import::remote_hash(&before.data, &before.snapshot);
        if let Some(Attempt {
            effect: Effect::Mint { publication, .. },
            ..
        }) = &mint
        {
            ensure!(
                verify_seed(self.client.base_url(), &before.data, &before.snapshot, publication)? == binding,
                "mint seed changed before first materialization"
            );
            let verified = import::build(self.client.base_url(), &before.data, &before.snapshot)?;
            source.prepare_binding(&verified)?;
        }
        let observed = import::build(self.client.base_url(), &before.data, &before.snapshot)?;
        ensure!(
            source.document.extra.get("contributors") == observed.extra.get("contributors"),
            "explicit contributor changes are unsupported by the native save vocabulary; authored values preserved, no push sent"
        );
        ensure!(
            mint.is_some()
                || decision_witness == Some(hash.as_str())
                || source
                    .document
                    .state
                    .sync
                    .as_ref()
                    .is_some_and(|sync| sync.profile == EDITOR_PROFILE && sync.remote_hash == hash),
            "remote drift/no verified baseline; deliberate current-witness decision required before effects"
        );
        let mut effective = source.document.clone();
        if let Some(replan) = &replan {
            effective.state.pending = None;
            effective.state.nodes.extend(replan.nodes.clone());
        }
        let compiled = compile(&effective, &before.data, &before.snapshot)?;
        let mut protection = protection(&before, &compiled)?;
        protection.verification_roots.extend(roots);
        protection.verification_roots.sort();
        protection.verification_roots.dedup();
        if !protection.verification_roots.is_empty() {
            before = self.native_observation(&binding, Some(&writer), &protection.verification_roots).await?;
            ensure!(
                import::remote_hash(&before.data, &before.snapshot) == hash,
                "native drift while acquiring detached-scope baseline"
            );
            if let Some(replan) = &replan {
                ensure!(
                    super::resolve::observation_hash(&before) == replan.resolution.observed_hash,
                    "native detached drift while acquiring recovery baseline; old attempt retained"
                );
            }
            protection.hash = protection_hash(&before, &protection);
        }
        let scope = scope(&before, &compiled, &binding)?;
        let target = wire_target(&binding)?;
        let requests = leafbud_protocol::encode_pushes(
            target,
            wire_writer(&writer),
            writer.next_mutation_id,
            0,
            &compiled.mutations,
            &scope,
            BatchLimits::default(),
        )?;
        if requests.is_empty() {
            let mut state = verify_desired(&effective, &compiled.targets, &before)?;
            if let Some(replan) = replan {
                state.last_resolution = Some(replan.resolution);
            }
            source.checkpoint(state)?;
            let mut result = report(RecoveryOutcome::DesiredVerified);
            result.settled = true;
            return Ok(result);
        }
        let range = writer.reserve(compiled.mutations.len()).map_err(anyhow::Error::msg)?;
        // Whole compiler validation and encoding finished before first effect.
        let current = self.native_observation(&binding, Some(&writer), roots).await?;
        ensure!(
            import::remote_hash(&current.data, &current.snapshot) == hash,
            "native drift after compilation; no effects sent"
        );
        if let Some(replan) = &replan {
            ensure!(
                super::resolve::observation_hash(&current) == replan.resolution.observed_hash,
                "native drift before recovery effects; old attempt retained"
            );
        }
        source.check()?;
        let mut state = source.document.state.clone();
        state.writer = Some(writer);
        let next_id = mint.map(|a| a.id).unwrap_or_else(Uuid::new_v4);
        if let Some(mut replan) = replan {
            replan.resolution.next_attempt = Some(next_id);
            state.last_resolution = Some(replan.resolution);
        }
        state.pending = Some(Attempt {
            id: next_id,
            input_hash: import::hash(&source.document.authored_value()),
            desired_hash: import::hash(&source.document.editor_value()),
            effect: Effect::Push {
                range,
                stage: PushStage::Prepared,
                protection,
                targets: compiled.targets,
                created_entities: compiled.created_entities,
            },
        });
        source.checkpoint(state)?;
        for request in &requests {
            let mut state = source.document.state.clone();
            if let Effect::Push { stage, .. } = &mut state.pending.as_mut().expect("prepared").effect {
                *stage = PushStage::Dispatched { through: request.last() };
            }
            source.checkpoint(state)?;
            // Only identical bytes are retried, and only while held in memory.
            // A single retry bounds network work; after restart recovery NEVER
            // pretends that recompiling would recreate these Yrs/request bytes.
            if self.client.send_push(request).await.is_err() && self.client.send_push(request).await.is_err() {
                break;
            }
            if source.check().is_err() {
                return Ok(report(RecoveryOutcome::SourceChanged));
            }
            // Ack this ordered slice before allowing dependent later requests.
            let check = self.client.pull_writer(binding.token, request.writer()).await?;
            if check.acknowledged(request.writer().client_id).is_none_or(|id| id < request.last()) {
                break;
            }
        }
        match self.recover_locked(source, true).await {
            Ok(result) => Ok(result),
            Err(error) => {
                let mut result = report(if source.check().is_err() {
                    RecoveryOutcome::SourceChanged
                } else {
                    RecoveryOutcome::Uncertain
                });
                result.message = Some(format!(
                    "request may have applied; pending evidence retained after read-back failure: {error:#}"
                ));
                Ok(result)
            }
        }
    }
}

// Identity/rights observation must still work for an acknowledged, partially
// applied graph that the complete typed content observer correctly rejects.
// Compilation and desired verification separately require supported full meaning.
fn observed_binding(binding: &Binding, data: &LeafletData, snapshot: &PullSnapshot) -> Result<()> {
    ensure!(
        data.metadata.id == binding.token && data.metadata.root_entity.as_deref() == Some(binding.root.as_str()),
        "native origin/token/root changed"
    );
    ensure!(
        data.observed_state() == LeafletState::Draft && import::association(data)? == binding.publication,
        "native lifecycle/publication changed"
    );
    let sets: BTreeSet<_> = data
        .metadata
        .permission_token_rights
        .iter()
        .filter(|r| r.read == Some(true))
        .filter_map(|r| r.entity_set.as_deref())
        .collect();
    ensure!(sets == BTreeSet::from([binding.entity_set.as_str()]), "native readable entity set changed");
    let entries: Vec<_> = snapshot
        .facts
        .iter()
        .filter(|r| r["entity"] == binding.root && r["attribute"] == "root/page")
        .collect();
    ensure!(
        entries.len() == 1 && entries[0]["id"] == binding.entry_fact && entries[0]["data"]["type"] == "ordered-reference",
        "native entry identity changed"
    );
    Ok(())
}

fn wire_writer(writer: &WriterState) -> Writer {
    Writer {
        client_group_id: writer.client_group_id,
        client_id: writer.client_id,
    }
}
fn wire_target(binding: &Binding) -> Result<Target> {
    Ok(Target {
        token: binding.token,
        root: binding.root.parse()?,
        entity_set: binding.entity_set.parse()?,
    })
}

fn scope(before: &Observation, compiled: &Compiled, binding: &Binding) -> Result<WriteScope> {
    let mut scope = WriteScope {
        publication: binding.publication.is_some(),
        ..Default::default()
    };
    for row in before.verification.values() {
        scope.entities.insert(row["entity"].as_str().context("entity")?.parse()?);
        scope.facts.insert(row["id"].as_str().context("fact ID")?.parse()?);
        if matches!(row["data"]["type"].as_str(), Some("reference" | "ordered-reference" | "spatial-reference")) {
            scope.entities.insert(row["data"]["value"].as_str().context("reference")?.parse()?);
        }
    }
    scope.entities.extend(&compiled.created_entities);
    scope.facts.extend(&compiled.allocated_facts);
    Ok(scope)
}

pub(super) fn protection(before: &Observation, compiled: &Compiled) -> Result<Protection> {
    let mut facts = BTreeSet::new();
    let mut fields = BTreeSet::new();
    let mut placements: BTreeMap<Uuid, (Uuid, Uuid)> = before
        .snapshot
        .facts
        .iter()
        .filter(|r| r["attribute"] == "card/block")
        .map(|r| {
            Ok((
                r["id"].as_str().context("placement ID")?.parse()?,
                (
                    r["entity"].as_str().context("parent")?.parse()?,
                    r["data"]["value"].as_str().context("child")?.parse()?,
                ),
            ))
        })
        .collect::<Result<_>>()?;
    let mut detached_scope = false;
    for mutation in &compiled.mutations {
        match mutation {
            NativeMutation::AssertFact(row) => {
                if row.attribute == "block/card" {
                    detached_scope |= before
                        .snapshot
                        .facts
                        .iter()
                        .any(|r| r["entity"] == row.entity.to_string() && r["attribute"] == "block/card" && r["data"] != row.data);
                }
                if row.attribute == "card/block" {
                    let id = row.id.context("placement assertion requires ID")?;
                    let target = row.data["value"].as_str().context("placement target")?.parse()?;
                    detached_scope |= placements.get(&id).is_some_and(|old| old.1 != target);
                    placements.insert(id, (row.entity, target));
                }
                if row.attribute == "block/text" {
                    leafbud_editor::read_text(row.data["value"].as_str().context("text payload")?)
                        .context("generated native text fails production decoder before effects")?;
                }
                if let Some(id) = row.id {
                    facts.insert(id);
                }
                for old in &before.snapshot.facts {
                    if old["entity"] == row.entity.to_string()
                        && old["attribute"] == row.attribute
                        && !matches!(row.attribute.as_str(), "root/page" | "card/block")
                    {
                        facts.insert(old["id"].as_str().context("fact ID")?.parse()?);
                    }
                }
            }
            NativeMutation::RetractFact { fact_id } => {
                detached_scope |= placements.remove(fact_id).is_some();
                // A retired page can also become unreachable through a card ref.
                detached_scope |= before
                    .snapshot
                    .facts
                    .iter()
                    .any(|r| r["id"] == fact_id.to_string() && r["attribute"] == "block/card");
                facts.insert(*fact_id);
            }
            NativeMutation::AddLastBlock { fact_id, parent, entity } => {
                ensure!(
                    placements.insert(*fact_id, (*parent, *entity)).is_none(),
                    "addLastBlock reuses an existing placement"
                );
                facts.insert(*fact_id);
            }
            NativeMutation::MoveBlock {
                old_parent, block, new_parent, ..
            } => {
                let rows: Vec<_> = placements
                    .iter()
                    .filter(|(_, (parent, child))| parent == old_parent && child == block)
                    .map(|(id, _)| *id)
                    .collect();
                ensure!(rows.len() == 1, "ambiguous/missing moveBlock occurrence before effects");
                facts.insert(rows[0]);
                placements.insert(rows[0], (*new_parent, *block));
            }
            NativeMutation::UpdateLeafletMetadata(meta) => {
                if meta.title.is_some() {
                    fields.insert("title".to_owned());
                }
                if meta.description.is_some() {
                    fields.insert("description".to_owned());
                }
            }
            NativeMutation::UpdatePublicationDraft(meta) => {
                if meta.title.is_some() {
                    fields.insert("title".to_owned());
                }
                if meta.description.is_some() {
                    fields.insert("description".to_owned());
                }
                if meta.tags.is_some() {
                    fields.insert("tags".to_owned());
                }
            }
            NativeMutation::CreateEntity(_) => {}
        }
    }
    let mut p = Protection {
        hash: String::new(),
        facts: facts.into_iter().collect(),
        entities: compiled.created_entities.clone(),
        metadata_fields: fields.into_iter().collect(),
        verification_roots: if detached_scope {
            before
                .snapshot
                .facts
                .iter()
                .filter_map(|r| r["entity"].as_str().and_then(|s| s.parse().ok()))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        } else {
            vec![]
        },
    };
    p.hash = protection_hash(before, &p);
    Ok(p)
}

fn protection_hash(observed: &Observation, protection: &Protection) -> String {
    let facts: Vec<_> = observed
        .verification
        .values()
        .filter(|row| {
            let excluded_fact = row["id"].as_str().and_then(|v| v.parse().ok()).is_some_and(|id| protection.facts.contains(&id));
            let excluded_entity = row["entity"]
                .as_str()
                .and_then(|v| v.parse().ok())
                .is_some_and(|id| protection.entities.contains(&id));
            !excluded_fact && !excluded_entity
        })
        .cloned()
        .collect();
    let fields = &protection.metadata_fields;
    let mut extras = observed.snapshot.extras.clone();
    for field in fields {
        extras.remove(&format!("publication_{field}"));
    }
    import::hash(&leafbud_editor::witness(
        &facts,
        &extras,
        serde_json::json!({"root":observed.data.metadata.root_entity,
        "publication":import::association(&observed.data).ok(),
        "title":if fields.iter().any(|v|v=="title"){None}else{observed.data.metadata.title.as_ref()},
        "description":if fields.iter().any(|v|v=="description"){None}else{observed.data.metadata.description.as_ref()}}),
    ))
}

fn verify_desired(existing: &Document, targets: &BTreeMap<String, Node>, observed: &Observation) -> Result<State> {
    let fresh = import::build(
        existing.state.binding.as_ref().context("binding")?.base_url.as_str(),
        &observed.data,
        &observed.snapshot,
    )?;
    crate::editor_state::same_identity(existing.state.binding.as_ref().expect("above"), &fresh)?;
    let mut expected = existing.clone();
    expected.state.nodes.extend(targets.clone());
    let merged = crate::editor_state::adopt(&expected, &fresh);
    ensure!(
        merged.content_keys() == expected.content_keys(),
        "current native object set differs from complete desired source"
    );
    for key in expected.content_keys() {
        let planned = expected.state.nodes.get(&key).context("desired object lacks a planned/confirmed association")?;
        let actual = merged.state.nodes.get(&key).context("desired association absent from native read-back")?;
        ensure!(planned == actual, "native association differs from planned/retained source identity");
    }
    // Retired rows are removed only after the whole current graph proves their
    // placements absent. The local UUID/mainPage/authored fields stay unchanged.
    expected.state.nodes = merged.state.nodes.clone();
    ensure!(
        expected.editor_value() == merged.editor_value(),
        "current native meaning/order/metadata does not equal complete desired source"
    );
    let mut state = existing.state.clone();
    state.nodes = merged.state.nodes;
    state.pending = None;
    let mut confirmed = existing.clone();
    confirmed.state = state.clone();
    state.sync = Some(Sync {
        profile: EDITOR_PROFILE.into(),
        local_hash: import::hash(&confirmed.editor_value()),
        remote_hash: import::remote_hash(&observed.data, &observed.snapshot),
    });
    Ok(state)
}

#[cfg(test)]
#[path = "probe.rs"]
mod probe;
