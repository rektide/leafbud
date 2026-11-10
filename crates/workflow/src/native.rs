//! Guarded native saves and finite recovery. `stage_binding` alone leaves a
//! pending materialization; `save_native` joins the complete pure editor compiler
//! to preparation, native effects and verified tracking settlement.
use crate::{RemoteReader, config, files, import, inventory, lock};
use anyhow::{Context, Result, ensure};
use leafbud_document::{Attempt, Binding, Document, Effect, MintContext, MintStage, Publication, State};
use leafbud_web::{LeafletData, LeafletState, MintTarget, PullSnapshot};
use serde::Serialize;
use std::path::{Path, PathBuf};
use uuid::Uuid;

mod resolve;
mod save;
mod write;
pub use leafbud_document::ResolutionDirection;
pub use resolve::{RecoverOptions, ResolveReport};
pub use save::{SaveOptions, SaveReport};
pub(crate) use save::{effects as preview_effects, prepared as prepared_preview};
pub use write::{RecoveryOutcome, RecoveryReport};

pub(crate) fn compile_preview(desired: &Document, observed: &crate::editor_state::Observed) -> Result<leafbud_editor::write::Compiled> {
    save::compile_reusing(desired, &observed.data, &observed.snapshot, &Default::default(), &[])
}

#[derive(Default)]
pub struct Selection {
    /// ALL explicit operands, including external files; rechecked after awaits.
    pub operands: Vec<PathBuf>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BindingOutcome {
    PendingMaterialization,
    MintUncertain,
    LocatedUnverified,
    SourceChanged,
}

#[derive(Debug, Serialize)]
pub struct BindingReport {
    pub outcome: BindingOutcome,
    /// Capability locator for explicit recovery when a source race forbids its
    /// persistence. Never put it in incidental diagnostics/logs.
    pub token: Option<Uuid>,
    pub locator_persisted: bool,
    pub binding: Option<Binding>,
    pub message: Option<String>,
    /// Sole undecoded response, retained only in process for explicit private
    /// investigation. Never serialized in reports or saved into source state.
    #[serde(skip)]
    pub undecoded_mint: Option<leafbud_web::ActionResponse>,
}

pub(crate) struct Source<'a> {
    context: &'a config::Context,
    path: PathBuf,
    operands: Vec<PathBuf>,
    _guard: lock::Guard,
    bytes: Vec<u8>,
    text: String,
    document: Document,
}

impl<'a> Source<'a> {
    fn open(context: &'a config::Context, path: &Path, selection: Selection) -> Result<Self> {
        let path = std::fs::canonicalize(if path.is_absolute() {
            path.to_path_buf()
        } else {
            context.workspace_root.join(path)
        })?;
        let store = lock::Store::new(context)?;
        let (guard, bytes, text) = lock::read(&path, &store)?;
        let document = leafbud_mdx::parse(&text)?;
        let operands = selection
            .operands
            .iter()
            .map(|p| {
                let p = if p.is_absolute() { p.clone() } else { context.workspace_root.join(p) };
                std::fs::canonicalize(&p).unwrap_or(p)
            })
            .collect();
        let source = Self {
            context,
            path,
            operands,
            _guard: guard,
            bytes,
            text,
            document,
        };
        source.check()?;
        Ok(source)
    }

    fn check(&self) -> Result<()> {
        ensure!(
            std::fs::read(&self.path).ok().as_deref() == Some(self.bytes.as_slice()),
            "source changed/lost; preserved"
        );
        let mut inventory = inventory::scan(&self.context.workspace_root, None)?;
        let mut operands = self.operands.clone();
        operands.push(self.path.clone());
        inventory.include_operands(&operands);
        ensure!(!inventory.duplicates.contains(&self.path), "duplicate local identity/native locator; preserved");
        Ok(())
    }

    fn checkpoint(&mut self, state: State) -> Result<()> {
        self.check()?;
        // Check a newly introduced locator against the complete selection too;
        // indexing only the old bytes cannot catch a prospective duplicate.
        let locator = state
            .binding
            .as_ref()
            .map(|b| (b.base_url.as_str(), b.token))
            .or_else(|| match &state.pending.as_ref()?.effect {
                Effect::Mint {
                    base_url,
                    stage: MintStage::Located { token },
                    ..
                } => Some((base_url.as_str(), *token)),
                _ => None,
            });
        if let Some((base, token)) = locator {
            let mut inventory = inventory::scan(&self.context.workspace_root, None)?;
            inventory.include_operands(&self.operands);
            ensure!(
                inventory
                    .index
                    .get(&(base.to_owned(), token))
                    .is_none_or(|paths| paths.iter().all(|p| p == &self.path)),
                "selected native locator already belongs to another local source; preserved"
            );
        }
        let text = leafbud_mdx::patch_tracking(&self.text, &state)?;
        if text != self.text {
            files::write(&self.path, Some(&self.bytes), text.as_bytes())?;
        }
        self.bytes = text.as_bytes().to_vec();
        self.text = text;
        self.document = leafbud_mdx::parse(&self.text)?;
        Ok(())
    }

    fn prepare_keys(&mut self) -> Result<()> {
        let plan = leafbud_mdx::prepare_keys(
            &self.text,
            leafbud_mdx::KeySource {
                block: &mut || format!("b-{}", Uuid::new_v4()),
                page: &mut || format!("p-{}", Uuid::new_v4()),
            },
        )?;
        let text = leafbud_mdx::apply_key_preparation(&self.text, &plan)?;
        self.check()?;
        if text != self.text {
            files::write(&self.path, Some(&self.bytes), text.as_bytes())?;
        }
        self.bytes = text.as_bytes().to_vec();
        self.text = text;
        self.document = leafbud_mdx::parse(&self.text)?;
        Ok(())
    }

    fn adopt(&mut self, document: &Document) -> Result<()> {
        ensure!(
            document.state.id == self.document.state.id
                && document.state.binding == self.document.state.binding
                && document.state.writer == self.document.state.writer,
            "recovery adoption cannot change source/native/writer identity"
        );
        let text = leafbud_mdx::adopt_remote(&self.text, document)?;
        self.check()?;
        if text != self.text {
            files::write(&self.path, Some(&self.bytes), text.as_bytes())?;
        }
        self.bytes = text.as_bytes().to_vec();
        self.text = text;
        self.document = leafbud_mdx::parse(&self.text)?;
        Ok(())
    }

    /// Explicit no-content-write preparation transition, not receipt settlement.
    /// No persisted intermediate mixes adopted fields and the previous hashes.
    fn prepare_binding(&mut self, verified: &Document) -> Result<()> {
        let attempt = self.document.state.pending.as_ref().context("located mint required")?;
        ensure!(
            import::hash(&self.document.authored_value()) == attempt.input_hash && import::hash(&self.document.editor_value()) == attempt.desired_hash,
            "source differs from prior mint intent; defaults not adopted"
        );
        self.check()?;
        let text = leafbud_mdx::prepare_binding_defaults(&self.text, verified)?;
        let mut prepared = leafbud_mdx::parse(&text)?;
        prepared.state.binding = verified.state.binding.clone();
        let input_hash = import::hash(&prepared.authored_value());
        let desired_hash = import::hash(&prepared.editor_value());
        let attempt = prepared.state.pending.as_mut().expect("preparation preserves mint");
        attempt.input_hash = input_hash;
        attempt.desired_hash = desired_hash;
        let text = leafbud_mdx::patch_tracking(&text, &prepared.state)?;
        self.check()?;
        if text != self.text {
            files::write(&self.path, Some(&self.bytes), text.as_bytes())?;
        }
        self.bytes = text.as_bytes().to_vec();
        self.text = text;
        self.document = leafbud_mdx::parse(&self.text)?;
        Ok(())
    }
}

fn validate_binding_intent(source: &str) -> Result<()> {
    let header = leafbud_mdx::header(source)?;
    ensure!(
        !header.get("theme").is_some_and(serde_json::Value::is_null),
        "explicit theme null has no supported native meaning; omit the field to adopt the verified default"
    );
    if let Some(value) = header.get("contributors") {
        let _: Vec<String> = serde_json::from_value(value.clone()).context("explicit contributors must be a list; null has no supported native meaning")?;
    }
    Ok(())
}

fn resolved_context(document: &Document, context: &config::Context) -> MintContext {
    match &document.publication {
        Publication::Standalone => MintContext::Standalone,
        Publication::At(uri) => MintContext::Publication(uri.clone()),
        Publication::Inherit => context
            .effective
            .publication
            .clone()
            .map(MintContext::Publication)
            .unwrap_or(MintContext::Standalone),
    }
}

impl RemoteReader {
    /// Explicit caller-selected existing recovery object for a dispatched mint
    /// whose response was lost. No creation, discovery or attribution inference.
    /// `attempt_id` guards stale selections; a known locator is never replaced.
    /// The supplied token is checkpointed BEFORE native verification so failure
    /// remains recoverable, then normal rights/seed/default preparation applies.
    pub async fn select_mint_locator(
        &self,
        context: &config::Context,
        path: &Path,
        selection: Selection,
        attempt_id: Uuid,
        token: Uuid,
    ) -> Result<BindingReport> {
        let mut source = Source::open(context, path, selection)?;
        validate_binding_intent(&source.text)?;
        let attempt = source
            .document
            .state
            .pending
            .as_ref()
            .context("no pending mint for explicit locator selection")?;
        ensure!(attempt.id == attempt_id, "pending attempt differs from explicit recovery selection");
        ensure!(
            import::hash(&source.document.authored_value()) == attempt.input_hash && import::hash(&source.document.editor_value()) == attempt.desired_hash,
            "source differs from pending mint intent; explicit locator not adopted"
        );
        let Effect::Mint { base_url, stage, .. } = &attempt.effect else {
            anyhow::bail!("locator selection requires a pending mint, not a push");
        };
        ensure!(base_url == self.client.base_url(), "pending mint belongs to another origin");
        ensure!(
            matches!(stage, MintStage::Dispatched) || matches!(stage, MintStage::Located { token: existing } if *existing == token),
            "only a dispatched mint or the same known locator can receive explicit selection"
        );
        let mut state = source.document.state.clone();
        if let Effect::Mint { stage, .. } = &mut state.pending.as_mut().expect("mint").effect {
            *stage = MintStage::Located { token };
        }
        source.checkpoint(state)?;
        let verification = self.recover_locked(&mut source, true).await;
        match verification {
            Ok(result) => Ok(BindingReport {
                outcome: if result.outcome == RecoveryOutcome::SourceChanged {
                    BindingOutcome::SourceChanged
                } else {
                    BindingOutcome::PendingMaterialization
                },
                token: Some(token),
                locator_persisted: true,
                binding: source.document.state.binding.clone(),
                message: Some(
                    "caller-selected recovery object; original mint-result attribution is not established; authored materialization remains pending".into(),
                ),
                undecoded_mint: None,
            }),
            Err(error) => Ok(BindingReport {
                outcome: if source.check().is_err() {
                    BindingOutcome::SourceChanged
                } else {
                    BindingOutcome::LocatedUnverified
                },
                token: Some(token),
                locator_persisted: true,
                binding: source.document.state.binding.clone(),
                message: Some(format!(
                    "caller-selected locator retained, not proof of original mint attribution; verification/preparation pending: {error:#}"
                )),
                undecoded_mint: None,
            }),
        }
    }

    /// Internal drafting foundation: resolve/authenticate, prepare keys once,
    /// persist dispatch BEFORE mint, record returned locator BEFORE verification.
    /// Leaves pending evidence until full compiler materialization/read-back.
    pub async fn stage_binding(&self, context: &config::Context, path: &Path, selection: Selection) -> Result<BindingReport> {
        let mut source = Source::open(context, path, selection)?;
        self.stage_binding_locked(&mut source).await
    }

    async fn stage_binding_locked(&self, source: &mut Source<'_>) -> Result<BindingReport> {
        ensure!(
            source.document.state.binding.is_none() && source.document.state.pending.is_none(),
            "already bound/pending; observe recovery instead of minting again"
        );
        validate_binding_intent(&source.text)?;
        // The complete desired source must be representable BEFORE mint, not
        // merely the first supported block. Proposed keys remain transient.
        save::prepared(&source.text)?;
        let publication = resolved_context(&source.document, source.context);
        if matches!(publication, MintContext::Standalone) {
            ensure!(
                source.document.tags.is_empty(),
                "standalone tags have no established native storage; no mint performed"
            );
        }
        let target = match &publication {
            MintContext::Standalone => MintTarget::Standalone,
            MintContext::Publication(uri) => MintTarget::Publication(uri.clone()),
        };
        let preparation = self
            .client
            .prepare_mint(target)
            .await
            .context("authenticating/discovering native mint (no effect yet)")?;
        source.check()?;
        source.prepare_keys()?;
        let attempt = Attempt {
            id: Uuid::new_v4(),
            input_hash: import::hash(&source.document.authored_value()),
            desired_hash: import::hash(&source.document.editor_value()),
            effect: Effect::Mint {
                base_url: self.client.base_url().into(),
                publication: publication.clone(),
                stage: MintStage::Prepared,
            },
        };
        let mut state = source.document.state.clone();
        state.pending = Some(attempt);
        source.checkpoint(state)?;
        let mut state = source.document.state.clone();
        if let Effect::Mint { stage, .. } = &mut state.pending.as_mut().expect("just prepared").effect {
            *stage = MintStage::Dispatched;
        }
        source.checkpoint(state)?;
        let result = self.client.mint(&preparation).await;
        let token = match result {
            Ok(Some(token)) => token,
            other => {
                let (message, undecoded_mint) = match other {
                    Err(error) => {
                        let message = format!("mint may have created a draft; no automatic retry: {error}");
                        (message, error.into_action_response())
                    }
                    _ => ("mint returned no locator; no automatic retry".into(), None),
                };
                return Ok(BindingReport {
                    outcome: BindingOutcome::MintUncertain,
                    token: None,
                    locator_persisted: false,
                    binding: None,
                    message: Some(message),
                    undecoded_mint,
                });
            }
        };
        // No network wait precedes this checkpoint. If a source race prohibits
        // it, return the real locator explicitly rather than overwrite new prose.
        let mut state = source.document.state.clone();
        if let Effect::Mint { stage, .. } = &mut state.pending.as_mut().expect("mint attempt").effect {
            *stage = MintStage::Located { token };
        }
        if let Err(error) = source.checkpoint(state) {
            return Ok(BindingReport {
                outcome: BindingOutcome::SourceChanged,
                token: Some(token),
                locator_persisted: false,
                binding: None,
                message: Some(format!("returned locator not persisted; preserve it for explicit recovery: {error:#}")),
                undecoded_mint: None,
            });
        }
        let verification: Result<Document> = async {
            let store = lock::Store::new(source.context)?;
            let _binding_guard = store
                .binding(self.client.base_url(), token)?
                .context("native locator busy in another operation")?;
            let data = self.client.get(token).await?;
            let snapshot = self.client.pull(token).await?;
            let binding = verify_seed(self.client.base_url(), &data, &snapshot, &publication)?;
            // Metadata is separately acquired; this is conservative rechecking,
            // not a remote CAS/atomic multi-request snapshot.
            let confirmed = self.client.get(token).await?;
            ensure!(
                verify_seed(self.client.base_url(), &confirmed, &snapshot, &publication)? == binding,
                "mint rights/binding changed during verification"
            );
            ensure!(
                import::remote_hash(&confirmed, &snapshot) == import::remote_hash(&data, &snapshot),
                "mint metadata changed during verification"
            );
            source.check()?;
            let verified = import::build(self.client.base_url(), &confirmed, &snapshot)?;
            // Keep the locator lock through the guarded first-binding checkpoint.
            source.prepare_binding(&verified)?;
            Ok(verified)
        }
        .await;
        match verification {
            Ok(verified) => Ok(BindingReport {
                outcome: BindingOutcome::PendingMaterialization,
                token: Some(token),
                locator_persisted: true,
                binding: verified.state.binding,
                message: None,
                undecoded_mint: None,
            }),
            Err(error) => Ok(BindingReport {
                outcome: if source.check().is_err() {
                    BindingOutcome::SourceChanged
                } else {
                    BindingOutcome::LocatedUnverified
                },
                token: Some(token),
                locator_persisted: true,
                binding: None,
                message: Some(format!("locator retained; verification/materialization still pending: {error:#}")),
                undecoded_mint: None,
            }),
        }
    }
}

fn verify_seed(base: &str, data: &LeafletData, snapshot: &PullSnapshot, publication: &MintContext) -> Result<Binding> {
    ensure!(
        data.observed_state() == LeafletState::Draft && import::association(data)?.as_deref() == publication.publication(),
        "mint lifecycle/publication association differs from resolved intent"
    );
    let built = import::build(base, data, snapshot)?;
    let binding = built.state.binding.context("verified seed binding missing")?;
    ensure!(
        built.theme.as_ref().is_some_and(|theme| theme.leaflet_watermark),
        "mint root lacks the pinned watermark default"
    );
    ensure!(
        data.metadata
            .permission_token_rights
            .iter()
            .any(|r| r.entity_set.as_deref() == Some(&binding.entity_set)
                && r.read == Some(true)
                && r.write == Some(true)
                && r.token.as_deref() == Some(binding.token.to_string().as_str())),
        "mint has no coherent readable/writable token rights"
    );
    ensure!(
        built.pages.len() == 1 && built.pages[0].blocks.len() == 1,
        "mint does not contain exactly one seeded block"
    );
    use leafbud_document::BlockKind;
    let expected = match (&publication, &built.pages[0].blocks[0].kind) {
        (MintContext::Standalone, BlockKind::Heading { level: 1, content }) | (MintContext::Publication(_), BlockKind::Paragraph { content }) => {
            content.is_empty()
        }
        _ => false,
    };
    ensure!(expected, "mint seed kind/content differs from pinned native creator");
    Ok(binding)
}

#[cfg(test)]
mod tests;
