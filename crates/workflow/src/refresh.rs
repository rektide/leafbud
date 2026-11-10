//! One guarded document transaction shared by pull and connected watch.
//! Advisory locks coordinate Leafbud writers; the adjacent original-byte check
//! protects edits across awaits, not a filesystem CAS against arbitrary editors.
use crate::{RemoteReader, config, inventory, lock};
use anyhow::{Context, Result, ensure};
use leafbud_document::Binding;
use leafbud_web::LeafletState;
use serde::Serialize;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub id: Uuid,
    pub binding: Binding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Unchanged,
    Updated,
    TakeRemote,
    ReceiptsOnly,
    Converged,
    LocalDirty,
    NeedsDecision,
    Pending,
    Unsupported,
    Invalid,
    Unbound,
    Ambiguous,
    SourceChanged,
    SourceLost,
    Busy,
    Unavailable,
    Failed,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::Updated => "updated",
            Self::TakeRemote => "take-remote",
            Self::ReceiptsOnly => "receipts-only",
            Self::Converged => "converged",
            Self::LocalDirty => "local-dirty",
            Self::NeedsDecision => "needs-decision",
            Self::Pending => "pending",
            Self::Unsupported => "unsupported",
            Self::Invalid => "invalid",
            Self::Unbound => "unbound",
            Self::Ambiguous => "ambiguous",
            Self::SourceChanged => "source-changed",
            Self::SourceLost => "source-lost",
            Self::Busy => "busy",
            Self::Unavailable => "unavailable",
            Self::Failed => "failed",
        }
    }
    pub fn retry(self) -> bool {
        matches!(self, Self::Busy | Self::Unavailable | Self::Failed | Self::SourceChanged)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub path: PathBuf,
    pub document_id: Option<Uuid>,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Default)]
pub struct Policy {
    pub adopt_published: bool,
    pub take_remote: bool,
    pub dry_run: bool,
    pub follow: bool,
    /// Selection captured by the controller. Never refresh a replacement file
    /// merely because it now occupies the same pathname.
    pub expected: Option<Identity>,
    pub state: Option<LeafletState>,
    pub operands: Vec<PathBuf>,
}

impl RemoteReader {
    pub async fn refresh_document(&self, context: &config::Context, path: &Path, policy: Policy) -> Report {
        let joined = if path.is_absolute() {
            path.to_path_buf()
        } else {
            context.workspace_root.join(path)
        };
        let path = std::fs::canonicalize(&joined).unwrap_or(joined);
        let mut report = Report {
            path: path.clone(),
            document_id: None,
            outcome: Outcome::Failed,
            message: None,
        };
        match self.refresh_transaction(context, &path, &policy, &mut report.document_id).await {
            Ok((outcome, message)) => {
                report.outcome = outcome;
                report.message = message;
            }
            Err(error) => report.message = Some(format!("{error:#}")),
        }
        report
    }

    async fn refresh_transaction(&self, context: &config::Context, path: &Path, policy: &Policy, id: &mut Option<Uuid>) -> Result<(Outcome, Option<String>)> {
        let decision = |outcome, message: &str| Ok((outcome, Some(message.to_owned())));
        let before = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return decision(Outcome::SourceLost, "local source removed; not recreated"),
            Err(error) => return Err(error).context("reading managed file"),
        };
        if !before.contains("@leafbud") && !before.contains("@springleaf") {
            return decision(Outcome::Unbound, "not a managed document; import creates bindings");
        }
        let header = match leafbud_mdx::header(&before) {
            Ok(header) => header,
            Err(_) => return decision(Outcome::Invalid, "invalid managed frontmatter; preserved"),
        };
        if header.get("@leafbud").is_none() {
            return decision(Outcome::Unbound, "not a current managed document; import creates bindings");
        }
        *id = Some(lock::source_id(&before)?);
        let store = lock::Store::new(context)?;
        let Some(_guard) = store.document(id.expect("set above"))? else {
            return decision(Outcome::Busy, "document busy in another Leafbud operation; retry later");
        };
        let original = std::fs::read(path).context("rereading locked source")?;
        let source = String::from_utf8(original.clone()).context("managed file is not UTF-8")?;
        if lock::source_id(&source)? != id.expect("set above") || source != before {
            return decision(Outcome::SourceChanged, "source changed during lock acquisition; retry");
        }
        // Pending belongs to future write recovery, not this follower. Inspect
        // without deserializing/dropping any unknown attempt evidence.
        if header["@leafbud"].get("pending").is_some_and(|v| !v.is_null()) {
            return decision(Outcome::Pending, "unresolved pending state; preserved for explicit recovery");
        }
        let parsed = match leafbud_mdx::parse_with_spans(&source) {
            Ok(parsed) => parsed,
            Err(_) => return decision(Outcome::Invalid, "unsupported/invalid local source or owned state; preserved"),
        };
        let existing = &parsed.document;
        let Some(binding) = existing.state.binding.as_ref() else {
            return decision(Outcome::Unbound, "unbound local draft; watch does not establish bindings");
        };
        let expected = Identity {
            id: existing.state.id,
            binding: binding.clone(),
        };
        if policy.expected.as_ref().is_some_and(|wanted| wanted != &expected) {
            return decision(Outcome::SourceChanged, "source binding/identity no longer selected; preserved");
        }
        if binding.base_url != self.client.base_url() {
            return decision(Outcome::Unbound, "file is not bound to this origin; preserved");
        }
        if let Err(error) = validate_selection(context, path, &expected, policy) {
            return decision(Outcome::Ambiguous, &format!("{error:#}"));
        }
        if policy.follow
            && (parsed.main_page_provisional()
                || !parsed.unkeyed().is_empty()
                || existing.state.nodes.keys().collect::<std::collections::BTreeSet<_>>()
                    != existing
                        .pages
                        .iter()
                        .flat_map(|p| std::iter::once(&p.key).chain(p.blocks.iter().map(|b| &b.key)))
                        .collect())
        {
            return decision(
                Outcome::LocalDirty,
                "new or retired authored objects remain local; watch never prepares keys or pushes",
            );
        }
        ensure!(!policy.adopt_published || !policy.follow, "watch cannot adopt a lifecycle transition");
        let observed = match self.observe_editor_with_adoption(existing, policy.adopt_published).await {
            Ok(observed) => observed,
            Err(error) => {
                return decision(
                    match error.kind {
                        "identity-changed" => Outcome::NeedsDecision,
                        "unsupported" if error.remote_hash.is_none() => Outcome::NeedsDecision,
                        "unsupported" => Outcome::Unsupported,
                        "missing" => Outcome::Unavailable,
                        _ => Outcome::Unavailable,
                    },
                    &error.message,
                );
            }
        };
        let transitioning = existing.state.state != observed.merged.state.state;
        let _public_guard = if transitioning {
            Some(
                store
                    .public(
                        observed
                            .merged
                            .state
                            .binding
                            .as_ref()
                            .and_then(|binding| binding.uri.as_deref())
                            .context("verified public URI missing")?,
                    )?
                    .context("public identity busy in another Leafbud adoption; retry later")?,
            )
        } else {
            None
        };
        if let Err(error) = validate_public_selection(context, path, &observed.merged, policy) {
            return decision(Outcome::Ambiguous, &format!("{error:#}"));
        }
        let agreement = observed.agreement;
        let outcome = if agreement.state == "clean" {
            Outcome::Unchanged
        } else if agreement.endpoints_equal {
            if agreement.baseline == "scoped" && agreement.local == crate::editor_state::Change::Unchanged {
                Outcome::ReceiptsOnly
            } else {
                Outcome::Converged
            }
        } else if agreement.state == "remote-changed" {
            Outcome::Updated
        } else if policy.take_remote {
            Outcome::TakeRemote
        } else if agreement.state == "local-changed" {
            return decision(
                Outcome::LocalDirty,
                "local edits preserved (remote unchanged); nothing to adopt — explicit pull --take remote makes a one-shot decision",
            );
        } else if agreement.state == "both-changed" {
            return decision(
                Outcome::NeedsDecision,
                "local and remote edits diverge; file preserved — explicit pull --take remote can adopt the current remote",
            );
        } else {
            return decision(
                Outcome::NeedsDecision,
                "no verified directional baseline; differing current endpoints preserved; explicit pull can settle equal endpoints or decide --take remote",
            );
        };
        // Also validate no-op observations: a source save during a read is not
        // a clean observation of the newer bytes.
        if std::fs::read(path).ok().as_deref() != Some(original.as_slice()) {
            return decision(Outcome::SourceChanged, "local file changed during pull/refresh; not overwritten");
        }
        if outcome == Outcome::Unchanged || policy.dry_run {
            return Ok((outcome, None));
        }
        let verified = self.client.get(binding.token).await.context("re-verifying remote identity")?;
        ensure!(
            verified.metadata.root_entity == observed.data.metadata.root_entity
                && verified.observed_state().as_str() == observed.merged.state.state
                && crate::import::association(&verified)? == binding.publication
                && crate::published::association(&verified)?.as_ref().map(|association| &association.uri)
                    == observed.merged.state.binding.as_ref().and_then(|binding| binding.uri.as_ref()),
            "remote identity/lifecycle/association changed during refresh; preserved"
        );
        // Rescan after all network awaits. A rename, duplicate or changed
        // selection must not recreate an old path or advance its receipt.
        if let Err(error) = validate_selection(context, path, &expected, policy) {
            return decision(Outcome::Ambiguous, &format!("{error:#}"));
        }
        validate_public_selection(context, path, &observed.merged, policy)?;
        let next = if existing.state.state != observed.merged.state.state {
            leafbud_mdx::adopt_published(&source, &observed.merged)
        } else {
            leafbud_mdx::adopt_remote(&source, &observed.merged)
        }
        .context("composing guarded refresh")?;
        if next.as_bytes() != original.as_slice() {
            crate::files::write(path, Some(&original), next.as_bytes()).context("local file changed during pull; not overwritten")?;
        }
        Ok((outcome, None))
    }
}

fn validate_public_selection(context: &config::Context, path: &Path, document: &leafbud_document::Document, policy: &Policy) -> Result<()> {
    if let Some(uri) = document.state.binding.as_ref().and_then(|binding| binding.uri.as_ref()) {
        let mut inventory = inventory::scan(&context.workspace_root, None)?;
        inventory.include_operands(&policy.operands);
        ensure!(
            inventory.public.get(uri).is_none_or(|paths| paths.iter().all(|candidate| candidate == path)),
            "public identity already belongs to another local source; adoption refused"
        );
    }
    Ok(())
}

fn validate_selection(context: &config::Context, path: &Path, expected: &Identity, policy: &Policy) -> Result<()> {
    let mut inventory = inventory::scan(&context.workspace_root, Some(&expected.binding.base_url))?;
    let mut operands = policy.operands.clone();
    if !operands.contains(&path.to_path_buf()) {
        operands.push(path.into());
    }
    inventory.include_operands(&operands);
    ensure!(
        !inventory.duplicates.contains(path),
        "duplicate local identities/bindings require resolution; file preserved"
    );
    let source = std::fs::read_to_string(path).context("selected source lost; not recreated")?;
    let header = leafbud_mdx::header(&source)?;
    ensure!(
        lock::source_id(&source)? == expected.id && serde_json::from_value::<Binding>(header["@leafbud"]["binding"].clone())? == expected.binding,
        "selected source identity/binding changed; preserved"
    );
    ensure!(
        policy.state.is_none_or(|s| header["@leafbud"]["state"] == s.as_str()),
        "source no longer matches selected lifecycle; preserved"
    );
    Ok(())
}
