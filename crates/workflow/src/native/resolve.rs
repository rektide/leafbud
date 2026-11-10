//! Explicit terminal accept-current/replan. Unknown requests never enter this
//! boundary. The old pending attempt survives until one atomic guarded decision.
use super::*;
use leafbud_document::{Node, Occurrence, PushStage, Resolution, ResolutionDisposition};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct RecoverOptions {
    pub yes: bool,
    pub take: Option<ResolutionDirection>,
}

#[derive(Debug, Serialize)]
pub struct ResolveReport {
    pub version: u8,
    pub path: PathBuf,
    pub document_id: Uuid,
    pub attempt_id: Option<Uuid>,
    pub current_attempt_id: Option<Uuid>,
    pub binding_known: bool,
    pub outcome: &'static str,
    pub observation: RecoveryReport,
    pub direction: Option<ResolutionDirection>,
    pub effects: BTreeMap<String, usize>,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication: Option<super::PublishReport>,
}

pub(super) struct Replan {
    pub original: Attempt,
    pub resolution: Resolution,
    pub nodes: BTreeMap<String, Node>,
    pub roots: Vec<Uuid>,
}

pub(super) fn observation_hash(observed: &write::Observation) -> String {
    import::hash(&serde_json::json!({"active": import::remote_hash(&observed.data, &observed.snapshot),
        "explicit": leafbud_editor::witness(&observed.verification.values().cloned().collect::<Vec<_>>(), &BTreeMap::new(), serde_json::Value::Null)}))
}

fn roots(attempt: &Attempt) -> Vec<Uuid> {
    match &attempt.effect {
        Effect::Push {
            created_entities, protection, ..
        } => created_entities
            .iter()
            .chain(&protection.verification_roots)
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        _ => vec![],
    }
}

// Only actually observed associations can be reused. Missing old allocations
// are not evidence of noncreation and must never fall back to a new entity.
fn correspondence(
    document: &Document,
    attempt: &Attempt,
    fresh: &Document,
    observed: &write::Observation,
) -> Result<(BTreeMap<String, Node>, Vec<serde_json::Value>)> {
    let Effect::Push { targets, created_entities, .. } = &attempt.effect else {
        anyhow::bail!("push required")
    };
    let mut known = document.state.nodes.clone();
    known.extend(targets.clone());
    let mut nodes = BTreeMap::new();
    let mut detached = BTreeMap::new();
    for key in document.content_keys() {
        let Some(prior) = known.get(&key) else { continue };
        let matches: Vec<_> = fresh
            .state
            .nodes
            .values()
            .filter(|n| {
                n.entity == prior.entity
                    && match (&n.occurrence, &prior.occurrence) {
                        (Occurrence::Block { placement: a, .. }, Occurrence::Block { placement: b, .. }) => a == b,
                        (Occurrence::Page { .. }, Occurrence::Page { .. }) => true,
                        _ => false,
                    }
            })
            .collect();
        if matches.len() == 1 {
            nodes.insert(key, matches[0].clone());
            continue;
        }
        ensure!(matches.is_empty(), "ambiguous planned association: {key}");
        let Occurrence::Block { placement, .. } = &prior.occurrence else {
            anyhow::bail!("planned page absent from complete current graph: {key}; no automatic repair");
        };
        ensure!(
            created_entities.iter().any(|id| id.to_string() == prior.entity),
            "confirmed/planned occurrence no longer present: {key}; no automatic resurrection"
        );
        ensure!(
            observed.verification.values().all(|r| r["id"] != *placement),
            "planned placement occupied/changed: {key}"
        );
        let rows: Vec<_> = observed.verification.values().filter(|r| r["entity"] == prior.entity).cloned().collect();
        let actual = leafbud_editor::observe_detached_block(&rows, &prior.entity, placement, document.state.id)
            .with_context(|| format!("known detached allocation incomplete/unsupported: {key}; old attempt retained, no duplicate creation"))?;
        for row in rows {
            detached.insert(row["id"].as_str().context("fact ID")?.to_owned(), row);
        }
        nodes.insert(key, actual);
    }
    Ok((nodes, detached.into_values().collect()))
}

impl RemoteReader {
    /// Preview by default. --yes permits original settlement; --take chooses a
    /// NEW resolution only for terminal pushes. No locator discovery/remint.
    pub async fn recover_editor(&self, context: &config::Context, path: &Path, selection: Selection, options: RecoverOptions) -> Result<ResolveReport> {
        let mut source = Source::open(context, path, selection)?;
        let original = source.document.state.pending.clone();
        let mut report = ResolveReport {
            version: 1,
            path: source.path.clone(),
            document_id: source.document.state.id,
            attempt_id: original.as_ref().map(|a| a.id),
            current_attempt_id: original.as_ref().map(|a| a.id),
            binding_known: source.document.state.binding.is_some(),
            outcome: "pending",
            direction: options.take,
            effects: BTreeMap::new(),
            message: None,
            observation: write::report(RecoveryOutcome::Uncertain),
            publication: None,
        };
        if source.document.state.publication_attempt.is_some() {
            // --take local/remote is an editor decision, never publication retry
            // authority. All publication recovery remains observation-only.
            let result = self.recover_publication_locked(&mut source, options.yes).await;
            match result {
                Ok(publication) => {
                    report.outcome = publication.outcome;
                    report.message = publication.message.clone();
                    report.publication = Some(publication);
                }
                Err(error) => {
                    report.outcome = if source.check().is_err() { "source-changed" } else { "uncertain" };
                    report.message = Some(format!("publication observation incomplete; evidence retained, no retry: {error:#}"));
                }
            }
            return Ok(report);
        }
        if let Err(error) = self.resolve_locked(&mut source, original, options, &mut report).await {
            report.outcome = if source.check().is_err() {
                "source-changed"
            } else if source.document.state.pending.as_ref().map(|a| a.id) != report.attempt_id {
                "pending"
            } else {
                "blocked"
            };
            report.message = Some(format!(
                "resolution blocked; current pending evidence retained, inspect recover before any new action: {error:#}"
            ));
        }
        report.current_attempt_id = source.document.state.pending.as_ref().map(|a| a.id);
        Ok(report)
    }

    async fn resolve_locked(&self, source: &mut Source<'_>, original: Option<Attempt>, options: RecoverOptions, report: &mut ResolveReport) -> Result<()> {
        report.observation = self.recover_locked(source, false).await?;
        report.message = report.observation.message.clone();
        match report.observation.outcome {
            RecoveryOutcome::NoAttempt => {
                report.outcome = "no-attempt";
                return Ok(());
            }
            RecoveryOutcome::MintUncertain => {
                report.message = Some("mint response/locator is uncertain; retain this attempt. An explicitly selected existing locator can use select_mint_locator; never guess by title/recency or mint again".into());
                return Ok(());
            }
            RecoveryOutcome::SourceChanged => {
                report.outcome = "source-changed";
                report.message = Some("source intent or selected operands changed; evidence preserved. Reinspect the current source/attempt before any new action; no old intent was declared successful".into());
                return Ok(());
            }
            RecoveryOutcome::DesiredVerified | RecoveryOutcome::PreparedNotSent | RecoveryOutcome::PendingMaterialization => {
                report.outcome = "would-settle-prior";
                if options.yes {
                    report.observation = self.recover_locked(source, true).await?;
                    report.outcome = if report.observation.settled { "settled-prior" } else { "pending" };
                }
                report.message = Some("only prior intent settlement/preparation; no new save or recreation executed".into());
                return Ok(());
            }
            _ => {}
        }
        let Some(direction) = options.take else { return Ok(()) };
        if !report.observation.terminal {
            return Ok(());
        }
        let original = original.context("terminal push required")?;
        let Effect::Push { range, stage, targets, .. } = &original.effect else {
            anyhow::bail!("terminal push required")
        };
        let through = match stage {
            PushStage::Dispatched { through } | PushStage::Partial { through } => *through,
            _ => anyhow::bail!("unsent push is not terminal partial"),
        };
        let binding = source.document.state.binding.clone().context("binding")?;
        let writer = source.document.state.writer.clone().context("writer")?;
        let roots = roots(&original);
        let observed = self.native_observation(&binding, Some(&writer), &roots).await?;
        source.check()?;
        let ack = observed
            .snapshot
            .acknowledged(writer.client_id)
            .context("terminal acknowledgment unavailable")?;
        ensure!(ack >= through, "writer finality no longer available; old attempt retained");
        let fresh = import::build(&binding.base_url, &observed.data, &observed.snapshot)
            .context("terminal current graph incomplete/unsupported; cannot adopt or replan, old attempt retained")?;
        crate::editor_state::same_identity(&binding, &fresh)?;
        let resolution = Resolution {
            attempt_id: original.id,
            input_hash: original.input_hash.clone(),
            desired_hash: original.desired_hash.clone(),
            writer,
            range: *range,
            through,
            acknowledged: ack,
            direction,
            disposition: ResolutionDisposition::OriginalNotVerified,
            observed_hash: observation_hash(&observed),
            next_attempt: None,
        };
        match direction {
            ResolutionDirection::Remote => {
                let mut known = source.document.clone();
                known.state.nodes.extend(targets.clone());
                let mut adopted = crate::editor_state::adopt(&known, &fresh);
                adopted.state.pending = None;
                adopted.state.last_resolution = Some(resolution.clone());
                // Compose/validate even preview without changing source state.
                leafbud_mdx::adopt_remote(&source.text, &adopted)?;
                report.outcome = "would-accept-remote";
                if options.yes {
                    let current = self.native_observation(&binding, source.document.state.writer.as_ref(), &roots).await?;
                    ensure!(
                        observation_hash(&current) == resolution.observed_hash,
                        "native drift across accept-current; old attempt retained"
                    );
                    source.adopt(&adopted)?;
                    report.outcome = "accepted-remote";
                }
            }
            ResolutionDirection::Local => {
                validate_binding_intent(&source.text)?;
                let mut desired = save::prepared(&source.text)?;
                let (nodes, detached) = correspondence(&desired, &original, &fresh, &observed)?;
                desired.state.nodes.extend(nodes.clone());
                desired.state.pending = None;
                let compiled = save::compile_reusing(&desired, &observed.data, &observed.snapshot, &BTreeSet::new(), &detached)?;
                report.effects = save::effects(&compiled.mutations);
                write::protection(&observed, &compiled)?;
                report.outcome = "would-replan-local";
                if options.yes {
                    let result = self
                        .execute_resolving_locked(
                            source,
                            Some(&import::remote_hash(&observed.data, &observed.snapshot)),
                            Some(Replan {
                                original,
                                resolution,
                                nodes,
                                roots,
                            }),
                            |desired, data, snapshot| save::compile_reusing(desired, data, snapshot, &BTreeSet::new(), &detached),
                        )
                        .await?;
                    report.outcome = match result.outcome {
                        RecoveryOutcome::DesiredVerified => "replanned-local",
                        RecoveryOutcome::SourceChanged => "source-changed",
                        RecoveryOutcome::ResolvedPartial => "partial",
                        _ => "pending",
                    };
                    report.message = result.message;
                }
            }
        }
        if report.message.is_none()
            || matches!(
                report.outcome,
                "accepted-remote" | "replanned-local" | "would-accept-remote" | "would-replan-local"
            )
        {
            report.message =
                Some("new decision resolves old terminal attempt without claiming original success; no old request replay, remint or cleanup".into());
        }
        Ok(())
    }
}
