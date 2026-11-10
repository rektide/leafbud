//! User-facing complete keyed save. The same stable-ID lock covers source
//! preparation, mint, native effects and guarded settlement.
use super::*;
use leafbud_protocol::NativeMutation;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct SaveOptions {
    pub yes: bool,
    pub take_local: bool,
    pub recreate: BTreeSet<String>,
}

#[derive(Debug, Serialize)]
pub struct SaveReport {
    pub outcome: &'static str,
    pub effects: BTreeMap<String, usize>,
    pub message: Option<String>,
}

pub(crate) fn prepared(text: &str) -> Result<Document> {
    let plan = leafbud_mdx::prepare_keys(
        text,
        leafbud_mdx::KeySource {
            block: &mut || format!("b-{}", Uuid::new_v4()),
            page: &mut || format!("p-{}", Uuid::new_v4()),
        },
    )?;
    let document = leafbud_mdx::parse(&leafbud_mdx::apply_key_preparation(text, &plan)?)?;
    leafbud_editor::write::preflight(&document)?;
    Ok(document)
}

pub(crate) fn effects(mutations: &[NativeMutation]) -> BTreeMap<String, usize> {
    let mut result = BTreeMap::new();
    for mutation in mutations {
        let name = match mutation {
            NativeMutation::CreateEntity(_) => "create-entity",
            NativeMutation::AssertFact(_) => "set-fact",
            NativeMutation::RetractFact { .. } => "detach-or-clear-fact",
            NativeMutation::AddLastBlock { .. } => "attach-block",
            NativeMutation::MoveBlock { .. } => "place-block",
            NativeMutation::UpdateLeafletMetadata(_) | NativeMutation::UpdatePublicationDraft(_) => "save-metadata",
        };
        *result.entry(name.into()).or_default() += 1;
    }
    result
}

fn compile(desired: &Document, data: &LeafletData, snapshot: &PullSnapshot, recreate: &BTreeSet<String>) -> Result<leafbud_editor::write::Compiled> {
    compile_reusing(desired, data, snapshot, recreate, &[])
}

pub(super) fn compile_reusing(
    desired: &Document,
    data: &LeafletData,
    snapshot: &PullSnapshot,
    recreate: &BTreeSet<String>,
    detached: &[serde_json::Value],
) -> Result<leafbud_editor::write::Compiled> {
    let current = import::build(&desired.state.binding.as_ref().context("binding")?.base_url, data, snapshot)?;
    let binding = desired.state.binding.as_ref().expect("above");
    let persisted_title = if binding.publication.is_some() || binding.uri.is_some() {
        snapshot.extras.get("publication_title").and_then(serde_json::Value::as_str)
    } else {
        data.metadata.title.as_deref()
    };
    leafbud_editor::write::compile_reusing(desired, &current, &snapshot.facts, persisted_title, recreate, detached)
}

impl RemoteReader {
    pub async fn save_native(&self, context: &config::Context, path: &Path, selection: Selection, options: SaveOptions) -> Result<SaveReport> {
        let mut source = Source::open(context, path, selection)?;
        validate_binding_intent(&source.text)?;
        let desired = prepared(&source.text)?;
        desired.state.validate_lifecycle().map_err(anyhow::Error::msg)?;
        if desired.state.state == "draft" && resolved_context(&desired, context) == MintContext::Standalone {
            ensure!(desired.tags.is_empty(), "standalone tags have no established native storage");
        }
        let mut summary = BTreeMap::new();
        if source.document.state.pending.is_some() {
            let recovered = match self.recover_locked(&mut source, options.yes).await {
                Ok(result) => result,
                Err(error) => {
                    return Ok(SaveReport {
                        outcome: "pending",
                        effects: summary,
                        message: Some(format!(
                            "prior attempt retained; observation/settlement unavailable, no new request compiled: {error:#}"
                        )),
                    });
                }
            };
            match recovered.outcome {
                RecoveryOutcome::DesiredVerified => {
                    return Ok(SaveReport {
                        outcome: if options.yes { "settled-prior" } else { "would-settle" },
                        effects: summary,
                        message: Some("prior attempt verified; only its settlement was requested/executed, no new save or --recreate action ran; repeat push for a separate action".into()),
                    });
                }
                RecoveryOutcome::PreparedNotSent if options.yes => {}
                RecoveryOutcome::PendingMaterialization if options.yes => {}
                _ => {
                    return Ok(SaveReport {
                        outcome: "pending",
                        effects: summary,
                        message: Some(
                            recovered
                                .message
                                .unwrap_or_else(|| format!("prior attempt: {:?}; retained, no new request compiled", recovered.outcome)),
                        ),
                    });
                }
            }
        }
        if source.document.state.binding.is_none() {
            ensure!(options.recreate.is_empty(), "--recreate requires confirmed supported block occurrences");
            if !options.yes {
                // A native seed does not yet exist to compile against. Do not
                // fabricate mutation counts or persist provisional identities.
                return Ok(SaveReport {
                    outcome: "would-bind-and-save",
                    effects: summary,
                    message: Some(format!(
                        "complete source preflight passed: {} pages, {} blocks; no creation, source changes or publication",
                        desired.pages.len(),
                        desired.pages.iter().map(|p| p.blocks.len()).sum::<usize>()
                    )),
                });
            }
            let staged = self.stage_binding_locked(&mut source).await?;
            if !matches!(staged.outcome, BindingOutcome::PendingMaterialization) {
                return Ok(SaveReport {
                    outcome: "pending",
                    effects: summary,
                    message: staged.message,
                });
            }
            summary.insert("mint-draft".into(), 1);
        }
        let binding = source.document.state.binding.clone().expect("bound above");
        ensure!(
            source.document.publication == binding.publication.clone().map(Publication::At).unwrap_or(Publication::Standalone),
            "source publication differs from confirmed binding; retargeting is unsupported"
        );
        let before = self.native_observation(&binding, source.document.state.writer.as_ref(), &[]).await?;
        source.check()?;
        let witness = import::remote_hash(&before.data, &before.snapshot);
        let mint = source.document.state.pending.as_ref().is_some_and(|a| matches!(a.effect, Effect::Mint { .. }));
        let baseline = source
            .document
            .state
            .sync
            .as_ref()
            .is_some_and(|s| s.profile == leafbud_document::EDITOR_PROFILE && s.remote_hash == witness);
        // Converged current endpoints may settle without choosing a replacement
        // direction, but only the real compiler/verifier can prove an empty save
        // (display-fallback metadata equality alone is insufficient).
        let converged = !mint
            && !baseline
            && options.recreate.is_empty()
            && compile(&desired, &before.data, &before.snapshot, &options.recreate).is_ok_and(|c| c.mutations.is_empty());
        if !mint && !baseline && !options.take_local && !converged {
            return Ok(SaveReport { outcome: "needs-decision", effects: summary, message: Some("remote drift/no verified baseline; inspect diff, then use --take local against the current observation (--yes alone does not resolve drift)".into()) });
        }
        if !options.yes {
            let desired = prepared(&source.text)?;
            let compiled = compile(&desired, &before.data, &before.snapshot, &options.recreate)?;
            write::protection(&before, &compiled)?;
            return Ok(SaveReport {
                outcome: if compiled.mutations.is_empty() { "unchanged" } else { "would-save" },
                effects: effects(&compiled.mutations),
                message: None,
            });
        }
        let result = self
            .execute_locked(
                &mut source,
                (options.take_local || converged).then_some(witness.as_str()),
                |desired, data, snapshot| {
                    let compiled = compile(desired, data, snapshot, &options.recreate)?;
                    summary.extend(effects(&compiled.mutations));
                    Ok(compiled)
                },
            )
            .await;
        let result = match result {
            Ok(result) => result,
            Err(error) if source.document.state.pending.is_some() => {
                return Ok(SaveReport {
                    outcome: "pending",
                    effects: summary,
                    message: Some(format!("known native attempt retained; no independent replan: {error:#}")),
                });
            }
            Err(error) => return Err(error),
        };
        Ok(SaveReport {
            outcome: match result.outcome {
                RecoveryOutcome::DesiredVerified => {
                    if summary.is_empty() {
                        "unchanged"
                    } else {
                        "saved"
                    }
                }
                RecoveryOutcome::SourceChanged => "source-changed",
                RecoveryOutcome::ResolvedPartial => "partial",
                _ => "pending",
            },
            effects: summary,
            message: result.message,
        })
    }
}
