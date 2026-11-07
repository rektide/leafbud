//! `springleaf drafts author <token-id> --from <slug-or-file>` — the
//! authoring half of `design/draft-extraction.glm53m.md`.
//!
//! Converts a canonical record file into the push mutation batch that
//! writes it into an **existing** editor draft, and — only with `--yes` —
//! executes the push. Default posture is dry-run: the batch is printed
//! (mutation names, entity ids, positions, per-block summary — content,
//! never secrets) and the command exits 2 without touching the
//! transport. The live push is operator-gated exactly like `publish
//! run`; tests exercise it only through the injected transport.
//!
//! Network reads before any write decision: `get_leaflet_data` (root
//! entity, write right, entity set) and one read-only `pull` (start
//! page, existing block count, append seeds). Authoring into a non-empty
//! draft appends after its existing blocks — loudly warned, never
//! destructive.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use springleaf_drafts::author::{self, AuthorMutation, AuthorOptions, Authored};
use springleaf_drafts::pull::PullSnapshot;
use springleaf_drafts::{Error as DraftsError, LeafletData};

use crate::workspace::Workspace;

/// Injected push transport (`FnOnce` — one push per command). Tests
/// supply a canned future; production wraps `LeafletClient::push`.
pub(crate) type PushFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;
pub(crate) type PushFn = Box<dyn FnOnce(&[AuthorMutation]) -> PushFuture + Send>;

/// `drafts author` output (human summary + the `--json` schema v1).
#[derive(Debug, Serialize)]
pub(crate) struct AuthorOutcome {
    pub version: u8,
    pub token: String,
    pub root_entity: String,
    pub page_entity: String,
    pub record: String,
    pub slug: String,
    pub client_id: String,
    pub client_group_id: String,
    pub mutation_ids: (u64, u64),
    pub existing_blocks: usize,
    pub append_after: Option<String>,
    pub blocks: BlocksOut,
    pub warns: Vec<springleaf_drafts::extract::Warn>,
    pub mutations: Vec<AuthorMutation>,
    pub pushed: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct BlocksOut {
    pub authored: usize,
    pub skipped: usize,
}

/// The flow after the live reads: validate the record, resolve the
/// target, build the batch, print it — and push only through the
/// injected transport with `--yes` (production wraps `LeafletClient::
/// push`; tests a canned future).
#[allow(clippy::too_many_arguments)]
pub(crate) async fn author_run(
    token_id: &str,
    data: &LeafletData,
    snapshot: &PullSnapshot,
    record_path: &Path,
    record: &Value,
    yes: bool,
    json: bool,
    push: PushFn,
) -> Result<AuthorOutcome> {
    springleaf_leaflet::validate_document_record(record).with_context(|| format!("{} is not a valid site.standard.document", record_path.display()))?;
    let slug = record_path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("record filename has no slug stem")?
        .to_string();

    let root_entity = data.root_entity.clone().context("get_leaflet_data carries no root_entity")?;
    // Write right + entity set for THIS token (the row whose token
    // matches; a single-row fallback covers fixtures that omit it).
    let right = data
        .permission_token_rights
        .iter()
        .find(|r| r.token.as_deref() == Some(token_id))
        .or_else(|| (data.permission_token_rights.len() == 1).then(|| &data.permission_token_rights[0]));
    let Some(right) = right else {
        bail!("get_leaflet_data carries no permission rights for token {token_id}");
    };
    if right.write != Some(true) {
        bail!("token {token_id} holds no write right on this draft — authoring refused");
    }
    let permission_set = right.entity_set.clone().context("the token's right row carries no entity_set")?;

    let target = author::resolve_target(&root_entity, &snapshot.facts)
        .map_err(|err| match err {
            DraftsError::Structure(detail) => anyhow::anyhow!("draft structure error: {detail}"),
            other => anyhow::Error::new(other),
        })
        .context("resolving the authoring target from the pull")?;
    if target.root_page_count > 1 {
        eprintln!(
            "warning: {} root/page facts (publication nav machinery?) — the first by position is the authoring target",
            target.root_page_count
        );
    }
    if target.block_count > 0 {
        eprintln!(
            "warning: the target draft is not empty ({} existing blocks): authoring appends after them — nothing is replaced or removed",
            target.block_count
        );
    }

    // Session parameters: fresh ids per run (replay-safe from mutation id
    // 1 under a fresh clientID), uuid v7 minting for facts/entities.
    let options = AuthorOptions {
        root_entity: root_entity.clone(),
        page_entity: target.page_entity.clone(),
        permission_set,
        last_page_position: target.last_page_position.clone(),
        last_block_position: target.last_block_position.clone(),
        yjs_client_id: yjs_client_id(),
        client_id: format!("springleaf-author-{}", uuid::Uuid::now_v7().simple()),
        mutation_id_base: 1,
        timestamp_base: now_ms(),
        poll_definitions: poll_definitions(record).await,
    };
    let mut ids = std::iter::repeat_with(|| uuid::Uuid::now_v7().to_string());
    let authored: Authored = author::author(record, &options, &mut ids)
        .map_err(|err| match err {
            DraftsError::Structure(detail) => anyhow::anyhow!("record structure error: {detail}"),
            other => anyhow::Error::new(other),
        })
        .context("building the mutation batch")?;
    if authored.mutations.is_empty() {
        bail!("the record authors nothing (no pages and no metadata)");
    }

    let client_group_id = "springleaf-author".to_string();
    let mut outcome = AuthorOutcome {
        version: 1,
        token: token_id.to_string(),
        root_entity,
        page_entity: target.page_entity.clone(),
        record: record_path.display().to_string(),
        slug,
        client_id: options.client_id.clone(),
        client_group_id: client_group_id.clone(),
        mutation_ids: (
            authored.mutations.first().map(|m| m.id).unwrap_or(0),
            authored.mutations.last().map(|m| m.id).unwrap_or(0),
        ),
        existing_blocks: target.block_count,
        append_after: target.last_block_position.clone(),
        blocks: BlocksOut {
            authored: authored.blocks_authored,
            skipped: authored.blocks_skipped,
        },
        warns: authored.warns.clone(),
        mutations: authored.mutations.clone(),
        pushed: false,
    };

    if !yes {
        if json {
            println!("{}", serde_json::to_string_pretty(&outcome)?);
        } else {
            print!("{}", human_summary(&outcome, &authored));
        }
        println!("\nrefusing to push without --yes: re-run with --yes to execute the push against draft {token_id} (operator-gated)");
        return Ok(outcome); // the caller exits 2
    }

    push(&authored.mutations).await?;
    outcome.pushed = true;
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        print!("{}", human_summary(&outcome, &authored));
        println!("pushed: {} mutations to draft {}", outcome.mutation_ids.1, outcome.token);
    }
    Ok(outcome)
}

/// The live transport (the only write path, always behind `--yes`).
fn live_push(client: &springleaf_drafts::LeafletClient, token_id: &str, root_entity: String, client_group_id: String) -> PushFn {
    let client = client.clone();
    let token_id = token_id.to_string();
    Box::new(move |mutations: &'_ [AuthorMutation]| {
        let mutations = mutations.to_vec();
        Box::pin(async move {
            client
                .push(&token_id, &root_entity, &mutations, &client_group_id)
                .await
                .map_err(|err| anyhow::anyhow!("push failed: {err}"))
        })
    })
}

/// Human dry-run rendering: one line per mutation, then the summary.
pub(crate) fn human_summary(outcome: &AuthorOutcome, authored: &Authored) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "target: draft {} (root {}, page {})",
        outcome.token, outcome.root_entity, outcome.page_entity
    );
    let _ = writeln!(out, "record: {} (slug {})", outcome.record, outcome.slug);
    let existing = match outcome.existing_blocks {
        0 => "empty draft".to_string(),
        n => format!("{n} existing blocks, appending after {:?}", outcome.append_after.as_deref().unwrap_or("(none)")),
    };
    let _ = writeln!(out, "state:  {existing}");
    let _ = writeln!(
        out,
        "client: {} in group {}, mutation ids {}..={}",
        outcome.client_id, outcome.client_group_id, outcome.mutation_ids.0, outcome.mutation_ids.1
    );
    let _ = writeln!(out);
    for mutation in &outcome.mutations {
        let _ = writeln!(out, "  {}", mutation_line(mutation));
    }
    let _ = writeln!(out, "\nblocks: {} authored, {} skipped", authored.blocks_authored, authored.blocks_skipped);
    if outcome.warns.is_empty() {
        let _ = writeln!(out, "warns:  none");
    } else {
        let _ = writeln!(out, "warns:  {}", outcome.warns.len());
        for warn in &outcome.warns {
            let _ = writeln!(out, "  [{}] {}: {}", warn.block_type, warn.entity, warn.reason);
        }
    }
    out
}

/// One compact mutation line (content only — never the token).
fn mutation_line(mutation: &AuthorMutation) -> String {
    let args = &mutation.args;
    match mutation.name.as_str() {
        "addBlock" => {
            let entity = args["newEntityID"].as_str().unwrap_or("?");
            let position = args["position"].as_str().unwrap_or("?");
            let block_type = args["type"].as_str().unwrap_or("?");
            let parent = args["parent"].as_str().unwrap_or("?");
            let list = args.get("list").map(|l| format!(" list={l}")).unwrap_or_default();
            format!("{}  addBlock     {block_type} → {entity} @ {position} (parent {parent}){list}", mutation.id)
        }
        "assertFact" => {
            let entity = args["entity"].as_str().unwrap_or("?");
            let attribute = args["attribute"].as_str().unwrap_or("?");
            let detail = match args["data"]["type"].as_str() {
                Some("text") => format!(" ({} chars yjs)", args["data"]["value"].as_str().map(str::len).unwrap_or(0)),
                Some(kind) => format!(" {kind}"),
                None => String::new(),
            };
            let fact = args.get("id").map(|_| " fact-id").unwrap_or("");
            format!("{}  assertFact   {entity} {attribute}{detail}{fact}", mutation.id)
        }
        "createEntity" => format!("{}  createEntity {}", mutation.id, args["entityID"].as_str().unwrap_or("?")),
        "updateLeafletMetadata" => format!("{}  metadata     title {:?}", mutation.id, args["title"].as_str().unwrap_or("")),
        other => format!("{}  {other}       {args}", mutation.id),
    }
}

/// Random yjs client id (31 bits, like the python reference's default).
fn yjs_client_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    (nanos >> 1) & 0x7fff_ffff
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Resolve poll option texts for any poll blocks in the record (public
/// PDS reads: getRecord on each pollRef). Unresolvable refs warn + skip
/// inside the converter.
async fn poll_definitions(record: &Value) -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();
    let Some(pages) = record.pointer("/content/pages").and_then(Value::as_array) else {
        return map;
    };
    let uris: Vec<String> = pages
        .iter()
        .flat_map(|p| p.get("blocks").and_then(Value::as_array).into_iter().flatten())
        .filter_map(|b| {
            let uri = b.pointer("/block/pollRef/uri")?.as_str()?.to_string();
            Some(uri)
        })
        .collect();
    for uri in uris {
        match resolve_poll_definition(&uri).await {
            Ok(options) => {
                map.insert(uri, options);
            }
            Err(err) => eprintln!("warning: poll definition {uri} unresolvable ({err:#}) — its block will skip"),
        }
    }
    map
}

async fn resolve_poll_definition(uri: &str) -> Result<Vec<String>> {
    let parsed = springleaf_atproto::AtUri::parse(uri).with_context(|| format!("parsing pollRef {uri}"))?;
    let identity = springleaf_atproto::Resolver::new()
        .resolve(&parsed.authority)
        .await
        .with_context(|| format!("resolving {}", parsed.authority))?;
    let client = springleaf_atproto::XrpcClient::new(&identity.pds_endpoint)?;
    let record = client
        .get_record(&parsed.authority, &parsed.collection, &parsed.rkey)
        .await
        .with_context(|| format!("getRecord {uri}"))?
        .context("poll definition record not found")?;
    let options: Vec<String> = record
        .value
        .get("options")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(|o| o.get("text").and_then(Value::as_str).map(str::to_string)).collect())
        .unwrap_or_default();
    if options.is_empty() {
        bail!("poll definition record carries no options");
    }
    Ok(options)
}

/// The command body: the read-only network reads, then [`author_run`]
/// with the production push transport.
pub(crate) async fn drafts_author_cmd(
    client: &springleaf_drafts::LeafletClient,
    token_id: &str,
    from: &str,
    root: Option<PathBuf>,
    yes: bool,
    json: bool,
    push: Option<PushFn>,
) -> Result<i32> {
    let global = crate::config::global_layer()?;
    let ws = Workspace::resolve(root.as_deref(), &global)?;
    let (source, _slug) = crate::publish::resolve_source(&ws, from)?;
    let record_path = crate::publish::source_path(&source).to_path_buf();
    let record_text = std::fs::read_to_string(&record_path).with_context(|| format!("reading {}", record_path.display()))?;
    let record: Value = serde_json::from_str(&record_text).context("record file is not valid JSON")?;

    let data = client
        .get_leaflet_data(token_id)
        .await
        .with_context(|| format!("fetching leaflet data for {token_id}"))?;
    let snapshot = client
        .pull(token_id)
        .await
        .with_context(|| format!("pulling the draft fact graph for {token_id}"))?;

    let push = push.unwrap_or_else(|| live_push(client, token_id, outcome_root(&data), "springleaf-author".to_string()));
    let outcome = author_run(token_id, &data, &snapshot, &record_path, &record, yes, json, push).await?;
    Ok(if outcome.pushed { 0 } else { 2 })
}

fn outcome_root(data: &LeafletData) -> String {
    data.root_entity.clone().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::TempDir;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    macro_rules! exp02_fixture {
        ($file:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../experiments/02-draft-internals/fixtures/",
                $file
            ))
        };
    }

    /// The export fixture pair: the draft (data + pulled facts) and the
    /// record `drafts export` produced from it — authoring that record
    /// back into its own draft is the round-trip shape (appending after
    /// the existing a0/a1 blocks).
    fn fixture_draft() -> (LeafletData, PullSnapshot, Value) {
        let fixture: Value = serde_json::from_str(exp02_fixture!("export-pull-doc.json")).unwrap();
        let snapshot = springleaf_drafts::pull::parse_pull_snapshot(&fixture["response"].to_string()).unwrap();
        let data: LeafletData = serde_json::from_value(fixture["leaflet_data"].clone()).unwrap();
        let record: Value = serde_json::from_str(exp02_fixture!("export-record-doc.json")).unwrap();
        (data, snapshot, record)
    }

    fn record_file(record: &Value) -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new("author");
        let path = tmp.join("synthetic-title.json");
        std::fs::write(&path, serde_json::to_string_pretty(record).unwrap()).unwrap();
        (tmp, path)
    }

    /// A canned push transport: records whether it ran and what it got.
    fn fake_push() -> (Arc<AtomicBool>, Arc<Mutex<Vec<AuthorMutation>>>, PushFn) {
        let called = Arc::new(AtomicBool::new(false));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let push: PushFn = {
            let called = called.clone();
            let seen = seen.clone();
            Box::new(move |mutations| {
                let called = called.clone();
                let seen = seen.clone();
                let mutations = mutations.to_vec();
                Box::pin(async move {
                    called.store(true, Ordering::SeqCst);
                    seen.lock().unwrap().extend(mutations);
                    Ok(())
                })
            })
        };
        (called, seen, push)
    }

    #[tokio::test]
    async fn dry_run_builds_prints_and_refuses_without_touching_the_transport() {
        let (data, snapshot, record) = fixture_draft();
        let (_tmp, path) = record_file(&record);
        let (called, _seen, push) = fake_push();
        let outcome = author_run("01999000-0000-7aaa-8000-000000000001", &data, &snapshot, &path, &record, false, false, push)
            .await
            .unwrap();
        assert!(!outcome.pushed, "dry-run posture");
        assert!(!called.load(Ordering::SeqCst), "the transport must not even be entered without --yes");
        // The batch targets the fixture's own draft: appends after its
        // existing a0/a1 blocks.
        assert_eq!(outcome.existing_blocks, 2);
        assert_eq!(outcome.append_after.as_deref(), Some("a1"));
        assert_eq!(outcome.blocks.authored, 2);
        assert!(outcome.warns.is_empty());
        let names: Vec<&str> = outcome.mutations.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["addBlock", "assertFact", "assertFact", "addBlock", "assertFact", "updateLeafletMetadata"]
        );
        let positions: Vec<&str> = outcome
            .mutations
            .iter()
            .filter(|m| m.name == "addBlock")
            .map(|m| m.args["position"].as_str().unwrap())
            .collect();
        assert_eq!(positions, vec!["a2", "a3"], "appended after the draft's a1");
        // Mutation ids monotonic from 1 (replay-safe under a fresh clientID).
        assert_eq!(outcome.mutation_ids, (1, 6));
    }

    #[tokio::test]
    async fn with_yes_pushes_the_exact_batch_through_the_injected_transport() {
        let (data, snapshot, record) = fixture_draft();
        let (_tmp, path) = record_file(&record);
        let (called, seen, push) = fake_push();
        let outcome = author_run("01999000-0000-7aaa-8000-000000000001", &data, &snapshot, &path, &record, true, false, push)
            .await
            .unwrap();
        assert!(outcome.pushed);
        assert!(called.load(Ordering::SeqCst));
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 6);
        assert_eq!(seen[0].name, "addBlock");
        assert_eq!(seen[0].args["position"], json!("a2"));
        assert_eq!(seen[5].name, "updateLeafletMetadata");
        assert_eq!(seen[5].args["title"], json!("Synthetic title"));
        // The batch facts reference the fixture draft's entities.
        assert_eq!(seen[0].args["parent"], json!("01999000-0000-7aaa-8000-000000000003"));
        assert_eq!(seen[0].args["permission_set"], json!("01999000-0000-7aaa-8000-000000000009"));
    }

    #[tokio::test]
    async fn write_right_and_record_gates_refuse_before_anything_moves() {
        let (mut data, snapshot, record) = fixture_draft();
        data.permission_token_rights[0].write = Some(false);
        let (_tmp, path) = record_file(&record);
        let (called, _seen, push) = fake_push();
        let err = author_run("01999000-0000-7aaa-8000-000000000001", &data, &snapshot, &path, &record, true, false, push)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("no write right"), "{err}");
        assert!(!called.load(Ordering::SeqCst), "the rights gate refuses before the transport is entered");

        // An invalid record is refused by the lexicon gate.
        let (data, snapshot, _record) = fixture_draft();
        let bad = json!({"$type": "site.standard.document"});
        let (_tmp, path) = record_file(&bad);
        let (called, _seen, push) = fake_push();
        let err = author_run("01999000-0000-7aaa-8000-000000000001", &data, &snapshot, &path, &bad, true, false, push)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a valid site.standard.document"), "{err}");
        assert!(!called.load(Ordering::SeqCst), "gates run before the transport is entered");
    }

    /// The human dry-run rendering carries the decision-relevant content:
    /// target ids, per-mutation lines, block summary.
    #[tokio::test]
    async fn human_rendering_shapes_the_batch() {
        let (data, snapshot, record) = fixture_draft();
        let (_tmp, path) = record_file(&record);
        let (_called, _seen, push) = fake_push();
        // (author_run prints; capture its summary by rebuilding it.)
        let outcome = author_run("01999000-0000-7aaa-8000-000000000001", &data, &snapshot, &path, &record, false, true, push)
            .await
            .unwrap();
        let authored = springleaf_drafts::author::author(
            &record,
            &AuthorOptions {
                root_entity: outcome.root_entity.clone(),
                page_entity: outcome.page_entity.clone(),
                permission_set: "ps".into(),
                last_page_position: outcome.append_after.clone(),
                last_block_position: outcome.append_after.clone(),
                yjs_client_id: 1,
                client_id: outcome.client_id.clone(),
                mutation_id_base: 1,
                timestamp_base: 0,
                poll_definitions: Default::default(),
            },
            &mut (0..64).map(|i| format!("id-{i}")),
        )
        .unwrap();
        let summary = human_summary(&outcome, &authored);
        assert!(summary.contains("target: draft 01999000-0000-7aaa-8000-000000000001"), "{summary}");
        assert!(summary.contains("2 existing blocks"), "{summary}");
        assert!(summary.contains("addBlock     heading"), "{summary}");
        assert!(summary.contains("assertFact   "), "{summary}");
        assert!(summary.contains("block/heading-level"), "{summary}");
        assert!(summary.contains("metadata     title \"Synthetic title\""), "{summary}");
        let line = mutation_line(&outcome.mutations[0]);
        assert!(line.contains("@ a2 (parent 01999000-0000-7aaa-8000-000000000003)"), "{line}");
    }
}
