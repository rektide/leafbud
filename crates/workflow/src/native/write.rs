//! Compiler receiving seam. Kept crate-private until the COMPLETE keyed
//! compiler is commissioned/joined; there is deliberately no narrow public push.
use super::*;
use leafbud_document::{EDITOR_PROFILE, Node, Protection, PushStage, Sync, WriterState};
use leafbud_protocol::{BatchLimits, NativeMutation, Target, WriteScope, Writer};
use std::collections::{BTreeMap, BTreeSet};

/// Only actual native arguments and planned IDs, never an alternative document
/// tree. The future compiler must validate ALL desired source before returning.
#[allow(dead_code)] // Receiving type; only the controlled synthetic compiler exists in this batch.
pub(crate) struct Compiled {
    pub mutations: Vec<NativeMutation>,
    pub targets: BTreeMap<String, Node>,
    pub created_entities: Vec<Uuid>,
    pub allocated_facts: Vec<Uuid>,
}

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
}
fn report(outcome: RecoveryOutcome) -> RecoveryReport {
    RecoveryReport {
        outcome,
        settled: false,
        acknowledged: None,
        detached_facts_observed: 0,
        message: None,
    }
}

struct Observation {
    data: LeafletData,
    snapshot: PullSnapshot,
    detached_facts: usize,
}

impl RemoteReader {
    async fn native_observation(&self, binding: &Binding, writer: Option<&WriterState>, entities: &[Uuid]) -> Result<Observation> {
        ensure!(binding.base_url == self.client.base_url(), "native binding belongs to another origin");
        let data = self.client.get(binding.token).await?;
        let snapshot = match writer {
            Some(w) => self.client.pull_writer(binding.token, wire_writer(w)).await?,
            None => self.client.pull(binding.token).await?,
        };
        let fresh = import::build(self.client.base_url(), &data, &snapshot)?;
        crate::editor_state::same_identity(binding, &fresh)?;
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
        // Explicit-ID RPC is bounded to 128 roots per request. It does not
        // inherit the active graph's rooted visibility limit for detached IDs.
        for roots in entities.chunks(128) {
            let view = self.client.facts_for_entities(roots).await?;
            detached_facts += view.roots.values().map(Vec::len).sum::<usize>();
        }
        let confirmed = self.client.get(binding.token).await?;
        ensure!(
            import::remote_hash(&confirmed, &snapshot) == import::remote_hash(&data, &snapshot),
            "metadata changed during native observation"
        );
        Ok(Observation {
            data,
            snapshot,
            detached_facts,
        })
    }

    /// Finite observation/settlement only. Never remints, recompiles or replays.
    /// `settle` permits tracking-only writes with finality AND complete desired
    /// correspondence. Otherwise the report is read-only and retains evidence.
    pub async fn recover_native(&self, context: &config::Context, path: &Path, selection: Selection, settle: bool) -> Result<RecoveryReport> {
        let mut source = Source::open(context, path, selection)?;
        self.recover_locked(&mut source, settle).await
    }

    async fn recover_locked(&self, source: &mut Source<'_>, settle: bool) -> Result<RecoveryReport> {
        let Some(attempt) = source.document.state.pending.clone() else {
            return Ok(report(RecoveryOutcome::NoAttempt));
        };
        if import::hash(&source.document.authored_value()) != attempt.input_hash || import::hash(&source.document.editor_value()) != attempt.desired_hash {
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
                source.check()?;
                if let Some(existing) = &source.document.state.binding {
                    ensure!(existing == &binding, "pending mint binding changed");
                }
                if settle && source.document.state.binding.is_none() {
                    let mut state = source.document.state.clone();
                    state.binding = Some(binding);
                    source.checkpoint(state)?;
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
                let observed = self.native_observation(binding, Some(writer), created_entities).await?;
                let ack = observed.snapshot.acknowledged(writer.client_id);
                let mut result = report(RecoveryOutcome::Uncertain);
                result.acknowledged = ack;
                result.detached_facts_observed = observed.detached_facts;
                if source.check().is_err() {
                    result.outcome = RecoveryOutcome::SourceChanged;
                    return Ok(result);
                }
                // A lower/no acknowledgment after restart is not negative
                // finality: a delayed earlier request can still arrive.
                if ack.is_none_or(|id| id < through) {
                    return Ok(result);
                }
                if through != range.last {
                    result.outcome = RecoveryOutcome::ResolvedPartial;
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

    /// Internal compiler join. No public CLI or caller-supplied encoded bytes.
    /// Essential drift/complete-request/read-back gates apply in this seam now.
    #[allow(dead_code)] // Joined to the complete public save workflow in the compiler batch.
    pub(crate) async fn execute_compiled(
        &self,
        context: &config::Context,
        path: &Path,
        selection: Selection,
        decision_witness: Option<&str>,
        compile: impl FnOnce(&Document, &LeafletData, &PullSnapshot) -> Result<Compiled>,
    ) -> Result<RecoveryReport> {
        let mut source = Source::open(context, path, selection)?;
        ensure!(
            source.document.state.pending.is_none(),
            "unresolved attempt; recover before compiling another effect"
        );
        source.prepare_keys()?;
        let binding = source
            .document
            .state
            .binding
            .clone()
            .context("binding/materialization compiler join required")?;
        let mut writer = source.document.state.writer.clone().unwrap_or_else(WriterState::fresh);
        let before = self.native_observation(&binding, Some(&writer), &[]).await?;
        source.check()?;
        let hash = import::remote_hash(&before.data, &before.snapshot);
        ensure!(
            decision_witness == Some(hash.as_str())
                || source
                    .document
                    .state
                    .sync
                    .as_ref()
                    .is_some_and(|sync| sync.profile == EDITOR_PROFILE && sync.remote_hash == hash),
            "remote drift/no verified baseline; deliberate current-witness decision required before effects"
        );
        let compiled = compile(&source.document, &before.data, &before.snapshot)?;
        let protection = protection(&before, &compiled)?;
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
            let state = verify_desired(&source.document, &compiled.targets, &before)?;
            source.checkpoint(state)?;
            let mut result = report(RecoveryOutcome::DesiredVerified);
            result.settled = true;
            return Ok(result);
        }
        let range = writer.reserve(compiled.mutations.len()).map_err(anyhow::Error::msg)?;
        // Whole compiler validation and encoding finished before first effect.
        let current = self.native_observation(&binding, Some(&writer), &[]).await?;
        ensure!(
            import::remote_hash(&current.data, &current.snapshot) == hash,
            "native drift after compilation; no effects sent"
        );
        source.check()?;
        let mut state = source.document.state.clone();
        state.writer = Some(writer);
        state.pending = Some(Attempt {
            id: Uuid::new_v4(),
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
        self.recover_locked(&mut source, true).await
    }
}

fn wire_writer(writer: &WriterState) -> Writer {
    Writer {
        client_group_id: writer.client_group_id,
        client_id: writer.client_id,
    }
}
#[allow(dead_code)]
fn wire_target(binding: &Binding) -> Result<Target> {
    Ok(Target {
        token: binding.token,
        root: binding.root.parse()?,
        entity_set: binding.entity_set.parse()?,
    })
}

#[allow(dead_code)]
fn scope(before: &Observation, compiled: &Compiled, binding: &Binding) -> Result<WriteScope> {
    let mut scope = WriteScope {
        publication: binding.publication.is_some(),
        ..Default::default()
    };
    for row in &before.snapshot.facts {
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

#[allow(dead_code)]
fn protection(before: &Observation, compiled: &Compiled) -> Result<Protection> {
    let mut facts = BTreeSet::new();
    let mut fields = BTreeSet::new();
    for mutation in &compiled.mutations {
        match mutation {
            NativeMutation::AssertFact(row) => {
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
            NativeMutation::RetractFact { fact_id } | NativeMutation::AddLastBlock { fact_id, .. } => {
                facts.insert(*fact_id);
            }
            NativeMutation::MoveBlock { old_parent, block, .. } => {
                let rows: Vec<_> = before
                    .snapshot
                    .facts
                    .iter()
                    .filter(|r| r["entity"] == old_parent.to_string() && r["attribute"] == "card/block" && r["data"]["value"] == block.to_string())
                    .collect();
                ensure!(rows.len() == 1, "ambiguous/missing moveBlock occurrence before effects");
                facts.insert(rows[0]["id"].as_str().context("placement")?.parse()?);
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
    };
    p.hash = protection_hash(before, &p);
    Ok(p)
}

fn protection_hash(observed: &Observation, protection: &Protection) -> String {
    let facts: Vec<_> = observed
        .snapshot
        .facts
        .iter()
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
    let binding = expected.state.binding.as_ref().expect("binding");
    expected.publication = binding.publication.clone().map(Publication::At).unwrap_or(Publication::Standalone);
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
