//! Native-effect foundation. `stage_binding` mints/verifies a seed and leaves a
//! pending materialization, NOT a saved desired document. No CLI push is exposed.
//! The complete compiler joins the internal request runner in `native::write`.
use crate::{RemoteReader, config, files, import, inventory, lock};
use anyhow::{Context, Result, ensure};
use leafbud_document::{Attempt, Binding, Document, Effect, MintContext, MintStage, Publication, State};
use leafbud_web::{LeafletData, LeafletState, MintTarget, PullSnapshot};
use serde::Serialize;
use std::path::{Path, PathBuf};
use uuid::Uuid;

mod write;
pub use write::{RecoveryOutcome, RecoveryReport};

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
    /// Internal drafting foundation: resolve/authenticate, prepare keys once,
    /// persist dispatch BEFORE mint, record returned locator BEFORE verification.
    /// Leaves pending evidence until full compiler materialization/read-back.
    pub async fn stage_binding(&self, context: &config::Context, path: &Path, selection: Selection) -> Result<BindingReport> {
        let mut source = Source::open(context, path, selection)?;
        ensure!(
            source.document.state.binding.is_none() && source.document.state.pending.is_none(),
            "already bound/pending; observe recovery instead of minting again"
        );
        let publication = resolved_context(&source.document, context);
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
                return Ok(BindingReport {
                    outcome: BindingOutcome::MintUncertain,
                    token: None,
                    locator_persisted: false,
                    binding: None,
                    message: Some(match other {
                        Err(e) => format!("mint may have created a draft; no automatic retry: {e}"),
                        _ => "mint returned no locator; no automatic retry".into(),
                    }),
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
            });
        }
        let verification: Result<Binding> = async {
            let store = lock::Store::new(context)?;
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
                import::remote_hash(&confirmed, &snapshot) == import::remote_hash(&data, &snapshot),
                "mint metadata changed during verification"
            );
            source.check()?;
            Ok(binding)
        }
        .await;
        match verification {
            Ok(binding) => {
                let mut state = source.document.state.clone();
                state.binding = Some(binding.clone());
                if let Err(error) = source.checkpoint(state) {
                    return Ok(BindingReport {
                        outcome: BindingOutcome::SourceChanged,
                        token: Some(token),
                        locator_persisted: true,
                        binding: Some(binding),
                        message: Some(format!("native binding verified but local source changed: {error:#}")),
                    });
                }
                Ok(BindingReport {
                    outcome: BindingOutcome::PendingMaterialization,
                    token: Some(token),
                    locator_persisted: true,
                    binding: Some(binding),
                    message: None,
                })
            }
            Err(error) => Ok(BindingReport {
                outcome: BindingOutcome::LocatedUnverified,
                token: Some(token),
                locator_persisted: true,
                binding: None,
                message: Some(format!("locator retained; verification/materialization still pending: {error:#}")),
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
