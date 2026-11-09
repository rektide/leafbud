//! Executed CLI creation/preview/full first/revised/repeat saves via real HTTP.
#[path = "../../leaflet/protocol/tests/support/store.rs"]
mod store;
#[path = "../../atproto/src/testserver.rs"]
#[allow(dead_code)]
mod testserver;
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::process::Command;

fn cli(base: &str, root: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_leafbud"));
    c.env_remove("HOME")
        .env_remove("LEAFLET_AUTH_TOKEN")
        .env_remove("LEAFLET_BASE_URL")
        .env("XDG_CONFIG_HOME", root.join(".config"))
        .args(["--base-url", base, "--root", root.to_str().unwrap(), "--token", "synthetic"])
        .args(args);
    c
}
async fn invoke(base: &str, root: &Path, args: &[&str], code: i32) -> Value {
    let out = cli(base, root, args).output().await.unwrap();
    assert_eq!(
        out.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
async fn server(publication: bool) -> (testserver::TestServer, Arc<Mutex<store::Store>>) {
    let state = Arc::new(Mutex::new(store::Store::new(publication)));
    let remote = state.clone();
    let http = testserver::TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
        "/home" => (
            200,
            vec![],
            format!(
                "<script>self.__next_f.push({})</script>",
                json!([
                    1,
                    format!(
                        "0:{}",
                        json!({"atp_did":"did:plc:synthetic","bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]})
                    )
                ])
            ),
        ),
        "/lish/did:plc:synthetic/garden/dashboard" if r.method == "GET" => (200, vec![], "<script src=\"/_next/static/mint.js\"></script>".into()),
        "/_next/static/mint.js" => (200, vec![], include_str!("../../leaflet/web/tests/fixtures/mint-reference.js").into()),
        "/new" => (307, vec![("Location".into(), format!("/{}", store::TOKEN))], String::new()),
        "/lish/did:plc:synthetic/garden/dashboard" => (
            200,
            vec![("Content-Type".into(), "text/x-component".into())],
            include_str!("../../leaflet/web/tests/fixtures/mint-result.flight").into(),
        ),
        _ => (200, vec![], remote.lock().unwrap().rpc(&r.path, &r.body_json()).unwrap().to_string()),
    }))
    .await;
    (http, state)
}

#[tokio::test]
async fn complete_cli_both_contexts_preview_first_combined_revision_pull_and_unchanged_all() {
    for publication in [false, true] {
        let (server, store) = server(publication).await;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("doc.mdx");
        let args = if publication {
            vec!["--publication", store::URI, "create", "doc.mdx", "--title", "Independent title"]
        } else {
            vec!["create", "doc.mdx", "--title", "Independent title", "--standalone"]
        };
        let out = cli(&server.base_url, root.path(), &args).output().await.unwrap();
        assert!(out.status.success());
        let text = format!("{}\nAlpha 😀.\n\nBravo and delta.\n\nCharlie.\n", std::fs::read_to_string(&path).unwrap());
        std::fs::write(&path, &text).unwrap();
        let id = leafbud_mdx::parse(&text).unwrap().state.id;
        let report = invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--json"], 0).await;
        assert_eq!(report["items"][0]["outcome"], "would-bind-and-save");
        assert!(server.requests().is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        let report = invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await;
        assert_eq!(report["items"][0]["outcome"], "saved");
        let first_text = std::fs::read_to_string(&path).unwrap();
        let first = leafbud_mdx::parse(&first_text).unwrap();
        assert_eq!(first.state.id, id);
        assert_eq!(first.pages[0].blocks.len(), 3);
        let a = &first.pages[0].blocks[0].key;
        let b = &first.pages[0].blocks[1].key;
        let c = &first.pages[0].blocks[2].key;
        let header = first_text.split_once("\n---\n").unwrap().0;
        let revision = format!(
            "{header}\n---\n\n# Inserted heading, not a title\n\n<Key id=\"{b}\" />\n\n**Bravo**  \nretained 😀\n\nDelta split.\n\n<Key id=\"{a}\" />\n\nAlpha 😀.\n"
        );
        std::fs::write(&path, &revision).unwrap();
        let pushes = store.lock().unwrap().pushes;
        assert_eq!(
            invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--json"], 0).await["items"][0]["outcome"],
            "would-save"
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), revision);
        assert_eq!(store.lock().unwrap().pushes, pushes);
        assert_eq!(
            invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await["items"][0]["outcome"],
            "saved"
        );
        let second = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(second.title, "Independent title");
        assert_eq!(second.state.id, id);
        assert_eq!(second.state.binding, first.state.binding);
        for key in [a, b] {
            assert_eq!(second.state.nodes[key], first.state.nodes[key]);
        }
        assert!(!second.state.nodes.contains_key(c));
        let bytes = std::fs::read(&path).unwrap();
        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        let pushes = store.lock().unwrap().pushes;
        assert_eq!(
            invoke(&server.base_url, root.path(), &["push", "--all", "--state", "draft", "--yes", "--json"], 0).await["items"][0]["outcome"],
            "unchanged"
        );
        assert_eq!(invoke(&server.base_url, root.path(), &["pull", "doc.mdx", "--json"], 0).await["unchanged"], 1);
        assert_eq!(store.lock().unwrap().pushes, pushes);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
        assert_eq!(server.request_count("/new"), usize::from(!publication));
        assert_eq!(
            server.requests().iter().filter(|r| r.method == "POST" && r.path.ends_with("dashboard")).count(),
            usize::from(publication)
        );
    }
}

#[tokio::test]
async fn cli_decision_partial_recreation_and_usage_contracts() {
    let (server, store) = server(false).await;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("doc.mdx");
    assert!(
        cli(&server.base_url, root.path(), &["create", "doc.mdx", "--title", "Title", "--standalone"])
            .output()
            .await
            .unwrap()
            .status
            .success()
    );
    let text = format!("{}\nOriginal.\n", std::fs::read_to_string(&path).unwrap());
    std::fs::write(&path, text).unwrap();
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 0).await;
    store.lock().unwrap().data["title"] = "Remote title".into();
    let pushes = store.lock().unwrap().pushes;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 3).await["items"][0]["outcome"],
        "needs-decision"
    );
    assert_eq!(store.lock().unwrap().pushes, pushes);
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--take", "local", "--yes", "--json"], 0).await;
    let first = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let key = first.pages[0].blocks[0].key.clone();
    invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--recreate", &key, "--yes", "--json"], 0).await;
    let second = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_ne!(second.state.nodes[&key].entity, first.state.nodes[&key].entity);
    let text = std::fs::read_to_string(&path).unwrap().replace("Original.", "Changed.");
    std::fs::write(&path, &text).unwrap();
    store.lock().unwrap().skip_all = true;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 4).await["items"][0]["outcome"],
        "partial"
    );
    let pushes = store.lock().unwrap().pushes;
    assert_eq!(
        invoke(&server.base_url, root.path(), &["push", "doc.mdx", "--yes", "--json"], 4).await["items"][0]["outcome"],
        "pending"
    );
    assert_eq!(store.lock().unwrap().pushes, pushes);
    for args in [vec!["push"], vec!["push", "doc.mdx", "--all"], vec!["push", "doc.mdx", "--take", "remote"]] {
        assert_eq!(cli(&server.base_url, root.path(), &args).output().await.unwrap().status.code(), Some(2));
    }
    assert_eq!(
        cli(&server.base_url, root.path(), &["push", "doc.mdx", "--recreate-body"])
            .output()
            .await
            .unwrap()
            .status
            .code(),
        Some(1)
    );
}

/// No creation route is called: the source MUST already be the settled,
/// human-selected P1. Both old creation slots stay consumed and standalone stays
/// byte/mtime untouched. All locators/content evidence is private ignored data.
#[tokio::test]
#[ignore = "requires existing selected P1 edit authorization; never creates a draft"]
async fn authorized_selected_p1_complete_bound_cli_save() {
    use leafbud_document::{Block, BlockKind, Inline, Mark};
    assert_eq!(std::env::var("LEAFBUD_KEYED_PROBE").as_deref(), Ok("authorized-selected-p1"));
    let root = std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-effects")).unwrap();
    let dir = root.join("deployed");
    let path = dir.join("publication.mdx");
    let budgets = [dir.join("standalone.started"), dir.join("publication.started")];
    let budget_bytes = [std::fs::read(&budgets[0]).unwrap(), std::fs::read(&budgets[1]).unwrap()];
    let standalone = dir.join("standalone.mdx");
    let standalone_bytes = std::fs::read(&standalone).unwrap();
    let standalone_mtime = std::fs::metadata(&standalone).unwrap().modified().unwrap();
    let catalogue: Value = serde_json::from_slice(&std::fs::read(root.join("recovery-candidates/catalogue.private.json")).unwrap()).unwrap();
    let candidate = catalogue["candidates"].as_array().unwrap().iter().find(|c| c["label"] == "P1").unwrap();
    let original_source = std::fs::read_to_string(&path).unwrap();
    let original = leafbud_mdx::parse(&original_source).unwrap();
    assert!(original.state.pending.is_none() && original.state.sync.is_some());
    let binding = original.state.binding.clone().expect("P1 must be already bound: NO MINT");
    assert_eq!(binding.token.to_string(), candidate["token"].as_str().unwrap());
    assert_eq!(original.pages.len(), 1);
    assert_eq!(original.pages[0].blocks.len(), 1);
    let baseline_evidence = dir.join("publication.keyed-before.private.json");
    assert!(
        !baseline_evidence.exists(),
        "proof already started; inspect actual source/pending rather than rerun or recreate old intent"
    );
    std::fs::write(&baseline_evidence,serde_json::to_vec_pretty(&json!({"session":"opencode://ses_e3d96851dffeS2cPD2vPHjGklF","source":original_source,"binding":binding,"originalMintAttribution":"not-established; human-selected-P1"})).unwrap()).unwrap();
    async fn live(dir: &Path, args: &[&str]) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_leafbud"))
            .arg("--root")
            .arg(dir)
            .args(args)
            .output()
            .await
            .unwrap();
        std::fs::write(dir.join(format!("publication.keyed-{}.stdout.private.json", args[0])), &output.stdout).unwrap();
        assert!(output.status.success(), "CLI failed; inspect private stdout/source; no replan/remint");
        serde_json::from_slice(&output.stdout).unwrap()
    }
    let a = original.pages[0].blocks[0].key.clone();
    let b = format!("b-keyed-{}", leafbud_document::Uuid::new_v4());
    let c = format!("b-keyed-{}", leafbud_document::Uuid::new_v4());
    let paragraph = |key: &str, text: &str| Block {
        key: key.into(),
        kind: BlockKind::Paragraph {
            content: vec![Inline::Run {
                text: text.into(),
                marks: vec![],
            }],
        },
    };
    let mut baseline = original.clone();
    baseline.pages[0].blocks = vec![
        original.pages[0].blocks[0].clone(),
        paragraph(&b, "Bravo and delta."),
        paragraph(&c, "Charlie retained when detached."),
    ];
    let baseline_source = leafbud_mdx::render(&baseline).unwrap();
    std::fs::write(&path, &baseline_source).unwrap();
    let preview = live(&dir, &["push", "publication.mdx", "--json"]).await;
    assert_eq!(preview["items"][0]["outcome"], "would-save");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), baseline_source);
    let first_report = live(&dir, &["push", "publication.mdx", "--yes", "--json"]).await;
    assert_eq!(first_report["items"][0]["outcome"], "saved");
    let first = leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(first.state.nodes[&a], original.state.nodes[&a]);
    assert_eq!(first.state.binding, Some(binding.clone()));
    assert_eq!(first.state.id, original.state.id);
    let h = format!("b-keyed-{}", leafbud_document::Uuid::new_v4());
    let d = format!("b-keyed-{}", leafbud_document::Uuid::new_v4());
    let mut revised = first.clone();
    revised.pages[0].blocks = vec![
        Block {
            key: h.clone(),
            kind: BlockKind::Heading {
                level: 2,
                content: vec![Inline::Run {
                    text: "Keyed save verification, not the title".into(),
                    marks: vec![],
                }],
            },
        },
        Block {
            key: b.clone(),
            kind: BlockKind::Paragraph {
                content: vec![
                    Inline::Run {
                        text: "Bravo é😀e\u{301}".into(),
                        marks: vec![
                            Mark::Strong,
                            Mark::Link {
                                href: "https://example.test/keyed".into(),
                            },
                        ],
                    },
                    Inline::HardBreak,
                    Inline::Run {
                        text: "retained paragraph and text fact".into(),
                        marks: vec![Mark::Em],
                    },
                ],
            },
        },
        paragraph(&d, "Delta split into a new keyed block."),
        first.pages[0].blocks[0].clone(),
    ];
    let revised_source = leafbud_mdx::render(&revised).unwrap();
    std::fs::write(&path, &revised_source).unwrap();
    let preview2 = live(&dir, &["push", "publication.mdx", "--json"]).await;
    assert_eq!(preview2["items"][0]["outcome"], "would-save");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), revised_source);
    let revised_report = live(&dir, &["push", "publication.mdx", "--yes", "--json"]).await;
    assert_eq!(revised_report["items"][0]["outcome"], "saved");
    let final_source = std::fs::read_to_string(&path).unwrap();
    let final_doc = leafbud_mdx::parse(&final_source).unwrap();
    assert_eq!(final_doc.state.binding, Some(binding));
    assert_eq!(final_doc.state.id, original.state.id);
    assert_eq!(final_doc.title, original.title);
    for key in [&a, &b] {
        assert_eq!(final_doc.state.nodes[key], first.state.nodes[key]);
    }
    assert!(!final_doc.state.nodes.contains_key(&c));
    assert_eq!(final_doc.pages[0].blocks.iter().map(|b| &b.key).collect::<Vec<_>>(), vec![&h, &b, &d, &a]);
    let before = std::fs::read(&path).unwrap();
    let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    let pull = live(&dir, &["pull", "publication.mdx", "--json"]).await;
    assert_eq!(pull["unchanged"], 1);
    let repeat = live(&dir, &["push", "publication.mdx", "--yes", "--json"]).await;
    assert_eq!(repeat["items"][0]["outcome"], "unchanged");
    assert_eq!(repeat["items"][0]["effects"], json!({}));
    let status = live(&dir, &["status", "publication.mdx", "--json"]).await;
    assert_eq!(status["items"][0]["outcome"], "clean");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    assert_eq!(std::fs::read(&standalone).unwrap(), standalone_bytes);
    assert_eq!(std::fs::metadata(&standalone).unwrap().modified().unwrap(), standalone_mtime);
    for i in 0..2 {
        assert_eq!(std::fs::read(&budgets[i]).unwrap(), budget_bytes[i]);
    }
    std::fs::write(dir.join("publication.keyed-proof.private.json"),serde_json::to_vec_pretty(&json!({
        "session":"opencode://ses_e3d96851dffeS2cPD2vPHjGklF","baseline":first_report,"revision":revised_report,"previewBaseline":preview,"previewRevision":preview2,
        "retainedNodes":[&a,&b],"droppedNode":c,"detachedEntity":first.state.nodes[&c].entity,"finalDocument":final_doc,"pull":pull,"repeat":repeat,"status":status,
        "draftCreationCalls":0,"standaloneUntouched":true,"budgetsUnchanged":true,"lifecycle":"unpublished-draft","originalMintAttribution":"not-established; human-selected-P1"
    })).unwrap()).unwrap();
}
