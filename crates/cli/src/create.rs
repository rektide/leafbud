//! `springleaf drafts create` — mint a fresh editor draft via `GET /new`
//! (design/draft-extraction.glm53m.md §"The new-draft gap", option d).
//!
//! Closes the "push cannot mint drafts" gap: no Replicache mutation
//! creates `entity_sets`/`permission_tokens`, but leaflet's `/new` route
//! is a plain GET that runs the server-side creation and answers 307
//! with the fresh token id in the Location header (source evidence in
//! `springleaf_drafts::create`; experiments/02 §3 is the fact-model
//! background).
//!
//! Default posture is dry-run, exactly like `drafts author` and
//! `publish run`: without `--yes` the command prints exactly what will
//! happen — the URL, the redirect harvest, that the server mints a real
//! draft — and exits 2 without touching the transport. `--yes` performs
//! the one gated write probe, prints the new token id, and suggests the
//! follow-on commands (`drafts author` into it, `drafts list` to
//! verify). The auth cookie rides the client; it is never printed or
//! logged.

use std::future::Future;
use std::pin::Pin;

use anyhow::{Context, Result};
use serde::Serialize;
use springleaf_drafts::create::NewDraft;

/// Injected creation transport (`FnOnce` — one probe per command). Tests
/// supply a canned future; production wraps `LeafletClient::create_draft`.
pub(crate) type CreateFuture = Pin<Box<dyn Future<Output = Result<NewDraft>> + Send>>;
pub(crate) type CreateFn = Box<dyn FnOnce() -> CreateFuture + Send>;

/// The dry-run plan (human summary + the `--json` schema before `--yes`):
/// exactly what the probe will do.
#[derive(Debug, Serialize)]
pub(crate) struct CreatePlan {
    pub version: u8,
    pub created: bool,
    pub request_url: String,
    pub follow_redirects: bool,
    pub steps: Vec<String>,
}

impl CreatePlan {
    pub(crate) fn for_base(base_url: &str) -> Self {
        CreatePlan {
            version: 1,
            created: false,
            request_url: format!("{base_url}/new"),
            follow_redirects: false,
            steps: vec![
                "the server mints a real doc draft (h1 seed block, watermark fact) and links it to the token owner's homepage".into(),
                "harvest the fresh token id from the 307 Location header's first path segment (/<token-id>?focusFirstBlock&addToHome)".into(),
            ],
        }
    }
}

/// `drafts create --yes` output (human summary + the `--json` schema v1).
#[derive(Debug, Serialize)]
pub(crate) struct CreateOutcome {
    pub version: u8,
    pub created: bool,
    pub token_id: String,
    pub location: String,
    pub editor_url: String,
}

/// The refusal line printed without `--yes` (both output modes).
const REFUSAL: &str = "\nrefusing to create without --yes: re-run with --yes to mint the draft on the server (operator-gated)";

/// The command body: refuse with the plan (exit 2), or mint through the
/// injected transport (production wraps `LeafletClient::create_draft`;
/// tests a canned future). The token is resolved by the caller's client,
/// never here.
pub(crate) async fn drafts_create_cmd(client: &springleaf_drafts::LeafletClient, yes: bool, json: bool, create: Option<CreateFn>) -> Result<i32> {
    let base_url = client.base_url().to_string();
    if !yes {
        let plan = CreatePlan::for_base(&base_url);
        if json {
            println!("{}", serde_json::to_string_pretty(&plan)?);
        } else {
            print!("{}", human_plan(&plan));
        }
        println!("{REFUSAL}");
        return Ok(2);
    }

    let create = create.unwrap_or_else(|| live_create(client));
    let new_draft = create().await.context("minting a draft via GET /new")?;
    let outcome = CreateOutcome {
        version: 1,
        created: true,
        editor_url: format!("{base_url}/{}", new_draft.token_id),
        token_id: new_draft.token_id.clone(),
        location: new_draft.location.clone(),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        print!("{}", human_created(&outcome));
    }
    Ok(0)
}

/// The live transport (the only write path, always behind `--yes`).
fn live_create(client: &springleaf_drafts::LeafletClient) -> CreateFn {
    let client = client.clone();
    Box::new(move || Box::pin(async move { client.create_draft().await.map_err(|err| anyhow::anyhow!("draft creation probe failed: {err}")) }))
}

/// Human dry-run rendering: the numbered plan — the URL, the harvest,
/// the server-side effect.
pub(crate) fn human_plan(plan: &CreatePlan) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "drafts create — mints a fresh editor draft on the server (a real write, operator-gated)");
    let _ = writeln!(out);
    let _ = writeln!(out, "  1. GET {} with the auth cookie — redirects NOT followed", plan.request_url);
    for (i, step) in plan.steps.iter().enumerate() {
        let _ = writeln!(out, "  {}. {step}", i + 2);
    }
    let _ = writeln!(
        out,
        "\n  note: the mint happens even with a missing/invalid token (an orphan draft nobody lists)\n  — check `springleaf auth status` first."
    );
    out
}

/// Human success rendering: the new token id plus the follow-on commands.
pub(crate) fn human_created(outcome: &CreateOutcome) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "created draft {}", outcome.token_id);
    let _ = writeln!(out, "location:  {}", outcome.location);
    let _ = writeln!(out, "editor:    {}", outcome.editor_url);
    let _ = writeln!(out);
    let _ = writeln!(out, "verify:    springleaf drafts list (the new draft appears at the top)");
    let _ = writeln!(out, "author:    springleaf drafts author {} --from <slug-or-file>", outcome.token_id);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Synthetic ids in the 01999000-… space the other crates' tests use.
    const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
    const LOCATION: &str = "/01999000-0000-7aaa-8000-000000000001?focusFirstBlock&addToHome";

    /// A client is constructible offline (no request happens here); the
    /// transport below is the only thing that could ever touch the
    /// network, and only `--yes` enters it.
    fn offline_client() -> springleaf_drafts::LeafletClient {
        springleaf_drafts::LeafletClient::new("https://leaflet.pub", None).unwrap()
    }

    /// A canned creation transport: records whether it ran.
    fn fake_create() -> (Arc<AtomicBool>, CreateFn) {
        let called = Arc::new(AtomicBool::new(false));
        let flag = called.clone();
        let create: CreateFn = Box::new(move || {
            let flag = flag.clone();
            Box::pin(async move {
                flag.store(true, Ordering::SeqCst);
                Ok(NewDraft {
                    token_id: TOKEN.into(),
                    location: LOCATION.into(),
                })
            })
        });
        (called, create)
    }

    #[tokio::test]
    async fn dry_run_refuses_before_the_transport_is_even_entered() {
        let (called, create) = fake_create();
        let code = drafts_create_cmd(&offline_client(), false, false, Some(create)).await.unwrap();
        assert_eq!(code, 2, "the refusal exits 2");
        assert!(!called.load(Ordering::SeqCst), "the transport must not even be entered without --yes");
    }

    #[tokio::test]
    async fn with_yes_mints_through_the_injected_transport() {
        let (called, create) = fake_create();
        let code = drafts_create_cmd(&offline_client(), true, false, Some(create)).await.unwrap();
        assert_eq!(code, 0);
        assert!(called.load(Ordering::SeqCst), "--yes executes the probe");
    }

    #[tokio::test]
    async fn transport_failure_is_an_operational_error() {
        let create: CreateFn = Box::new(|| Box::pin(async { anyhow::bail!("probe refused") }));
        let err = drafts_create_cmd(&offline_client(), true, false, Some(create)).await.unwrap_err();
        let chain = format!("{err:#}");
        assert!(chain.contains("minting a draft via GET /new"), "{chain}");
        assert!(chain.contains("probe refused"), "{chain}");
    }

    /// Schema v1 field set, exactly once (the author/export outcome
    /// tests' key-order style).
    #[test]
    fn outcome_and_plan_json_shapes() {
        let outcome = CreateOutcome {
            version: 1,
            created: true,
            token_id: TOKEN.into(),
            location: LOCATION.into(),
            editor_url: format!("https://leaflet.pub/{TOKEN}"),
        };
        let value = serde_json::to_value(&outcome).unwrap();
        let keys: Vec<&str> = value.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["version", "created", "token_id", "location", "editor_url"]);
        assert_eq!(value["token_id"], json!(TOKEN));
        assert_eq!(value["editor_url"], json!(format!("https://leaflet.pub/{TOKEN}")));

        let plan = CreatePlan::for_base("https://leaflet.pub");
        let value = serde_json::to_value(&plan).unwrap();
        let keys: Vec<&str> = value.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["version", "created", "request_url", "follow_redirects", "steps"]);
        assert_eq!(value["request_url"], json!("https://leaflet.pub/new"));
        assert_eq!(value["follow_redirects"], json!(false));
        assert_eq!(plan.steps.len(), 2);
    }

    /// The renderings carry the decision-relevant content: the URL, the
    /// harvest, the write semantics; and the follow-on commands.
    #[test]
    fn human_renderings_shape_the_decision() {
        let plan = CreatePlan::for_base("https://leaflet.pub");
        let text = human_plan(&plan);
        assert!(text.contains("GET https://leaflet.pub/new"), "{text}");
        assert!(text.contains("redirects NOT followed"), "{text}");
        assert!(text.contains("mints a real doc draft"), "{text}");
        assert!(text.contains("307 Location"), "{text}");
        assert!(REFUSAL.contains("--yes"), "the refusal names the gate");

        let outcome = CreateOutcome {
            version: 1,
            created: true,
            token_id: TOKEN.into(),
            location: LOCATION.into(),
            editor_url: format!("https://leaflet.pub/{TOKEN}"),
        };
        let text = human_created(&outcome);
        assert!(text.contains(&format!("created draft {TOKEN}")), "{text}");
        assert!(text.contains(&format!("https://leaflet.pub/{TOKEN}")), "{text}");
        assert!(text.contains(&format!("springleaf drafts author {TOKEN} --from <slug-or-file>")), "{text}");
        assert!(text.contains("springleaf drafts list"), "{text}");
    }
}
