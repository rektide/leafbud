use super::*;
use leafbud_document::{Block, BlockKind, Inline, ListStyle, Mark, Occurrence, Page};
#[path = "../../../leaflet/protocol/tests/support/store.rs"]
mod store;

struct Server {
    http: TestServer,
    store: Arc<Mutex<store::Store>>,
    hook: Arc<Mutex<Option<Hook>>>,
}
type Hook = Box<dyn FnMut(&Recorded, &mut store::Store) + Send>;
impl Server {
    async fn new(publication: bool) -> Self {
        let store = Arc::new(Mutex::new(store::Store::new(publication)));
        let state = store.clone();
        let hook: Arc<Mutex<Option<Hook>>> = Arc::new(Mutex::new(None));
        let action = hook.clone();
        let http = TestServer::spawn_headers(Arc::new(move |r| match r.path.as_str() {
            "/home" => (200, vec![], home()),
            "/lish/did:plc:synthetic/garden/dashboard" if r.method == "GET" => (200, vec![], "<script src=\"/_next/static/mint.js\"></script>".into()),
            "/_next/static/mint.js" => (200, vec![], include_str!("../../../leaflet/web/tests/fixtures/mint-reference.js").into()),
            "/new" => (307, vec![("Location".into(), format!("/{}", store::TOKEN))], String::new()),
            "/lish/did:plc:synthetic/garden/dashboard" => (
                200,
                vec![("Content-Type".into(), "text/x-component".into())],
                include_str!("../../../leaflet/web/tests/fixtures/mint-result.flight").into(),
            ),
            _ => {
                let mut state = state.lock().unwrap();
                let response = state.rpc(&r.path, &r.body_json()).unwrap_or_else(|| panic!("unexpected {}", r.path));
                if let Some(action) = &mut *action.lock().unwrap() {
                    action(r, &mut state);
                }
                (200, vec![], response.to_string())
            }
        }))
        .await;
        Self { http, store, hook }
    }
    fn reader(&self) -> RemoteReader {
        client(&self.http)
    }
}
fn paragraph(key: &str, text: &str) -> Block {
    Block {
        key: key.into(),
        kind: BlockKind::Paragraph {
            content: vec![Inline::Run {
                text: text.into(),
                marks: vec![],
            }],
        },
    }
}
fn read(path: &Path) -> Document {
    leafbud_mdx::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn write(path: &Path, doc: &Document) {
    std::fs::write(path, leafbud_mdx::render(doc).unwrap()).unwrap();
}
async fn save(server: &Server, root: &Path, path: &Path, yes: bool) -> SaveReport {
    let mut context = old::context(root);
    context.effective.publication = server.store.lock().unwrap().data["leaflets_in_publications"]
        .as_array()
        .filter(|v| !v.is_empty())
        .map(|_| store::URI.into());
    server
        .reader()
        .save_native(&context, path, Selection::default(), SaveOptions { yes, ..Default::default() })
        .await
        .unwrap()
}
fn source(root: &Path) -> PathBuf {
    let path = root.join("draft.mdx");
    draft(&path, "# source-only comment\nlocalOnly: {keep: true}");
    let text = std::fs::read_to_string(&path).unwrap().replace(
        "Desired prose, untouched.",
        "<Key id=\"A\" />\n\nAlpha 😀.\n\n<Key id=\"B\" />\n\nBravo and delta.\n\n<Key id=\"C\" />\n\nCharlie.",
    );
    std::fs::write(&path, text).unwrap();
    path
}

#[tokio::test]
async fn both_first_mints_combined_revision_join_pull_and_zero_effect_repeat() {
    for publication in [false, true] {
        let server = Server::new(publication).await;
        let temp = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        let original = std::fs::read(&path).unwrap();
        let id = read(&path).state.id;
        assert_eq!(save(&server, temp.path(), &path, false).await.outcome, "would-bind-and-save");
        assert!(server.http.requests().is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        let result = save(&server, temp.path(), &path, true).await;
        assert_eq!(result.outcome, "saved", "{result:?}");
        let first = read(&path);
        let nodes = first.state.nodes.clone();
        let binding = first.state.binding.clone();
        assert_eq!(first.state.id, id);
        assert_eq!(nodes[&first.pages[0].key].entity, store::PAGE);
        assert!(first.state.pending.is_none());
        assert_eq!(server.http.request_count("/new"), usize::from(!publication));
        assert_eq!(
            server
                .http
                .requests()
                .iter()
                .filter(|r| r.method == "POST" && r.path.ends_with("dashboard"))
                .count(),
            usize::from(publication)
        );
        assert!(!server.store.lock().unwrap().facts.contains_key(store::MEMBER));
        assert!(server.store.lock().unwrap().entities.contains(store::SEED));
        let mut revision = first.clone();
        let heading = Block {
            key: "H".into(),
            kind: BlockKind::Heading {
                level: 2,
                content: vec![Inline::Run {
                    text: "Not the document title".into(),
                    marks: vec![Mark::Em],
                }],
            },
        };
        let mut b = paragraph("B", "Bravo");
        if let BlockKind::Paragraph { content } = &mut b.kind {
            content.push(Inline::HardBreak);
            content.push(Inline::Run {
                text: "é😀e\u{301}".into(),
                marks: vec![
                    Mark::Strong,
                    Mark::Link {
                        href: "https://example.test/new".into(),
                    },
                ],
            });
        }
        revision.pages[0].blocks = vec![heading, b, paragraph("D", "and delta."), first.pages[0].blocks[0].clone()];
        write(&path, &revision);
        let before = std::fs::read(&path).unwrap();
        let pushes = server.store.lock().unwrap().pushes;
        assert_eq!(save(&server, temp.path(), &path, false).await.outcome, "would-save");
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(server.store.lock().unwrap().pushes, pushes);
        let result = save(&server, temp.path(), &path, true).await;
        assert_eq!(result.outcome, "saved", "{result:?}");
        let second = read(&path);
        assert_eq!(second.state.binding, binding);
        assert_eq!(second.title, first.title);
        for key in ["A", "B"] {
            assert_eq!(second.state.nodes[key], nodes[key]);
        }
        assert!(!second.state.nodes.contains_key("C"));
        assert_ne!(second.state.nodes["D"].entity, nodes["C"].entity);
        assert!(
            server.store.lock().unwrap().one(&nodes["C"].entity, "block/text").is_some(),
            "detached C text retained and verified explicitly"
        );
        let before = std::fs::read(&path).unwrap();
        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        let pushes = server.store.lock().unwrap().pushes;
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
        assert_eq!(server.store.lock().unwrap().pushes, pushes);
        let context = old::context(temp.path());
        let observed = server.reader().observe_editor(&second).await.unwrap();
        assert!(observed.agreement.endpoints_equal);
        let pulled = server
            .reader()
            .pull(
                &context,
                crate::pull::Request {
                    files: vec![path.clone()],
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(pulled.unchanged, 1);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        // Join under B: D's placement retires, B's identity/text fact survives.
        let mut joined = second.clone();
        joined.pages[0].blocks = vec![paragraph("B", "Bravo and delta."), second.pages[0].blocks[3].clone()];
        write(&path, &joined);
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
        assert_eq!(read(&path).state.nodes["B"], nodes["B"]);
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
    }
}

#[tokio::test]
async fn complete_supported_profile_pages_scalar_transitions_and_selected_recreation() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    let mut doc = read(&path);
    doc.pages[0].blocks.extend([
        Block {
            key: "quote".into(),
            kind: BlockKind::Quote {
                content: vec![Inline::Run {
                    text: "all marks".into(),
                    marks: vec![
                        Mark::Strong,
                        Mark::Em,
                        Mark::Code,
                        Mark::Strikethrough,
                        Mark::Underline,
                        Mark::Highlight { color: "1".into() },
                        Mark::Link {
                            href: "https://example.test".into(),
                        },
                    ],
                }],
            },
        },
        Block {
            key: "bullet".into(),
            kind: BlockKind::ListItem {
                style: ListStyle::Bullet,
                content: vec![],
            },
        },
        Block {
            key: "ordered".into(),
            kind: BlockKind::ListItem {
                style: ListStyle::Ordered,
                content: vec![Inline::Run {
                    text: "item".into(),
                    marks: vec![],
                }],
            },
        },
        Block {
            key: "code".into(),
            kind: BlockKind::Code {
                language: "rs".into(),
                text: "let x = 1;\n".into(),
            },
        },
        Block {
            key: "rule".into(),
            kind: BlockKind::Rule,
        },
        Block {
            key: "image".into(),
            kind: BlockKind::Image {
                src: "https://example.test/a.png".into(),
                fallback: None,
                width: Some(20),
                height: Some(30),
                alt: Some("image".into()),
            },
        },
        Block {
            key: "link".into(),
            kind: BlockKind::PageLink { page: "Notes".into() },
        },
        Block {
            key: "link2".into(),
            kind: BlockKind::PageLink { page: "Notes".into() },
        },
    ]);
    doc.pages.push(Page {
        key: "Notes".into(),
        blocks: vec![paragraph("note", "Named page paragraph")],
    });
    write(&path, &doc);
    let result = save(&server, temp.path(), &path, true).await;
    assert_eq!(result.outcome, "saved", "{result:?}");
    let first = read(&path);
    let mut changed = first.clone();
    changed.pages[0].blocks.iter_mut().find(|b| b.key == "A").unwrap().kind = BlockKind::Code {
        language: "text".into(),
        text: "now code".into(),
    };
    changed.pages[0].blocks.iter_mut().find(|b| b.key == "code").unwrap().kind = BlockKind::Paragraph {
        content: vec![Inline::Run {
            text: "now text".into(),
            marks: vec![],
        }],
    };
    changed.pages[0].blocks.iter_mut().find(|b| b.key == "image").unwrap().kind = BlockKind::Image {
        src: "https://example.test/b.png".into(),
        fallback: Some("https://example.test/fallback.png".into()),
        width: None,
        height: None,
        alt: None,
    };
    changed.pages[0].blocks.swap(9, 10); // incoming page refs are a set, not traversal order
    let moved = changed.pages[0].blocks.remove(1); // B across pages, actual old parent
    changed.pages[1].blocks.push(moved);
    write(&path, &changed);
    let result = save(&server, temp.path(), &path, true).await;
    assert_eq!(result.outcome, "saved", "{result:?}");
    let second = read(&path);
    let old_text = match &first.state.nodes["A"].occurrence {
        Occurrence::Block { text_fact, .. } => text_fact.clone(),
        _ => unreachable!(),
    };
    assert!(matches!(&second.state.nodes["A"].occurrence,Occurrence::Block{inactive_text_fact,..} if inactive_text_fact==&old_text));
    assert_eq!(second.state.nodes["B"], first.state.nodes["B"]);
    assert_eq!(second.state.nodes["Notes"], first.state.nodes["Notes"]);
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
    let result = server
        .reader()
        .save_native(
            &old::context(temp.path()),
            &path,
            Selection::default(),
            SaveOptions {
                yes: true,
                recreate: ["A".into()].into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(result.outcome, "saved", "{result:?}");
    let recreated = read(&path);
    assert_ne!(recreated.state.nodes["A"].entity, second.state.nodes["A"].entity);
    assert!(
        matches!((&recreated.state.nodes["A"].occurrence,&second.state.nodes["A"].occurrence),(Occurrence::Block{placement:a,..},Occurrence::Block{placement:b,..}) if a==b)
    );
    assert!(server.store.lock().unwrap().entities.contains(&second.state.nodes["A"].entity));
    // Retire the named page and both incoming links, keeping its detached facts.
    let mut retired = recreated.clone();
    retired.pages[0].blocks.retain(|b| !matches!(b.kind, BlockKind::PageLink { .. }));
    retired.pages.pop();
    write(&path, &retired);
    let result = save(&server, temp.path(), &path, true).await;
    assert_eq!(result.outcome, "saved", "{result:?}");
    assert!(server.store.lock().unwrap().one(&second.state.nodes["note"].entity, "block/text").is_some());
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
}

#[tokio::test]
async fn unsupported_remainder_refuses_before_mint_and_preview_never_persists_keys() {
    let server = Server::new(false).await;
    for tail in [
        "\n<Page name=\"orphan\" kind=\"linear\">\n\nUnreferenced.\n\n</Page>\n",
        "\n![upload](file:///tmp/image.png)\n",
        "\n- [x] task\n",
        "\n- outer\n  - nested\n",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        let source = format!("{}{tail}", std::fs::read_to_string(&path).unwrap());
        std::fs::write(&path, &source).unwrap();
        let before = server.http.requests().len();
        let result = server
            .reader()
            .save_native(
                &old::context(temp.path()),
                &path,
                Selection::default(),
                SaveOptions {
                    yes: true,
                    ..Default::default()
                },
            )
            .await;
        assert!(result.is_err(), "{tail}");
        assert_eq!(server.http.requests().len(), before);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    }
}

#[tokio::test]
async fn skipped_creation_and_restart_pending_never_reencode_or_remint() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    server.store.lock().unwrap().skip_attachments = true;
    let result = save(&server, temp.path(), &path, true).await;
    assert_eq!(result.outcome, "partial", "{result:?}");
    let pending = read(&path).state.pending.unwrap();
    let Effect::Push { created_entities, .. } = &pending.effect else {
        panic!("push")
    };
    assert!(!created_entities.is_empty());
    let pushes = server.store.lock().unwrap().pushes;
    let before = std::fs::read(&path).unwrap();
    let result = save(&server, temp.path(), &path, true).await;
    assert_eq!(result.outcome, "pending");
    assert_eq!(server.store.lock().unwrap().pushes, pushes);
    assert_eq!(read(&path).state.pending, Some(pending));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(
        server
            .http
            .requests()
            .iter()
            .filter(|r| r.method == "POST" && r.path.ends_with("dashboard"))
            .count(),
        1
    );
}

#[tokio::test]
async fn shared_occurrence_divergence_requires_recreation_and_drop_preserves_remaining_entity() {
    for recreate in [false, true] {
        let server = Server::new(true).await;
        let temp = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
        let first = read(&path);
        let a = first.state.nodes["A"].clone();
        let page = Uuid::new_v4().to_string();
        let link = Uuid::new_v4().to_string();
        let placement = Uuid::new_v4().to_string();
        {
            let mut remote = server.store.lock().unwrap();
            remote.entities.extend([page.clone(), link.clone()]);
            for row in [
                json!({"id":Uuid::new_v4(),"entity":page,"attribute":"page/type","data":{"type":"page-type-union","value":"doc"}}),
                json!({"id":Uuid::new_v4(),"entity":link,"attribute":"block/type","data":{"type":"block-type-union","value":"card"}}),
                json!({"id":Uuid::new_v4(),"entity":link,"attribute":"block/card","data":{"type":"reference","value":page}}),
                json!({"id":Uuid::new_v4(),"entity":store::PAGE,"attribute":"card/block","data":{"type":"ordered-reference","value":link,"position":"z99999998"}}),
                json!({"id":placement,"entity":page,"attribute":"card/block","data":{"type":"ordered-reference","value":a.entity,"position":"a0"}}),
            ] {
                remote.assert(row);
            }
        }
        let observed = server.reader().observe_editor(&first).await.unwrap();
        write(&path, &observed.merged); // deliberate adoption establishes current baseline
        let mut desired = read(&path);
        let other = desired
            .state
            .nodes
            .iter()
            .find(|(key, n)| key.as_str() != "A" && n.entity == a.entity)
            .unwrap()
            .0
            .clone();
        assert_eq!(desired.pages.len(), 2);
        let pushes = server.store.lock().unwrap().pushes;
        if recreate {
            desired.pages[0].blocks[0] = paragraph("A", "Occurrence-specific fork");
            write(&path, &desired);
            let error = server
                .reader()
                .save_native(
                    &old::context(temp.path()),
                    &path,
                    Selection::default(),
                    SaveOptions {
                        yes: true,
                        ..Default::default()
                    },
                )
                .await
                .unwrap_err();
            assert!(format!("{error:#}").contains("shared entity"));
            assert_eq!(server.store.lock().unwrap().pushes, pushes);
            let result = server
                .reader()
                .save_native(
                    &old::context(temp.path()),
                    &path,
                    Selection::default(),
                    SaveOptions {
                        yes: true,
                        recreate: ["A".into()].into(),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            assert_eq!(result.outcome, "saved", "{result:?}");
            let after = read(&path);
            assert_ne!(after.state.nodes["A"].entity, a.entity);
            assert_eq!(after.state.nodes[&other].entity, a.entity);
            assert_eq!(after.pages[1].blocks[0].kind, first.pages[0].blocks[0].kind);
        } else {
            desired.pages[0].blocks.remove(0);
            write(&path, &desired);
            assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
            assert_eq!(read(&path).state.nodes[&other].entity, a.entity);
            assert!(server.store.lock().unwrap().one(&a.entity, "block/text").is_some());
        }
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
    }
}

#[tokio::test]
async fn yes_does_not_resolve_remote_drift_and_take_local_uses_current_complete_graph() {
    let server = Server::new(false).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
    {
        let mut remote = server.store.lock().unwrap();
        remote.data["description"] = "remote description".into();
        let entity = Uuid::new_v4().to_string();
        remote.entities.insert(entity.clone());
        remote.assert(json!({"id":Uuid::new_v4(),"entity":entity,"attribute":"block/type","data":{"type":"block-type-union","value":"horizontal-rule"}}));
        remote.assert(json!({"id":Uuid::new_v4(),"entity":store::PAGE,"attribute":"card/block","data":{"type":"ordered-reference","value":entity,"position":"z99999999"}}));
    }
    let source = std::fs::read(&path).unwrap();
    let pushes = server.store.lock().unwrap().pushes;
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "needs-decision");
    assert_eq!(std::fs::read(&path).unwrap(), source);
    assert_eq!(server.store.lock().unwrap().pushes, pushes);
    let result = server
        .reader()
        .save_native(
            &old::context(temp.path()),
            &path,
            Selection::default(),
            SaveOptions {
                yes: true,
                take_local: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(result.outcome, "saved", "{result:?}");
    assert!(server.store.lock().unwrap().data["description"].is_null());
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
}

#[tokio::test]
async fn detached_retained_fact_tampering_is_not_hidden_by_the_active_projection() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
    let mut desired = read(&path);
    let c = desired.state.nodes["C"].entity.clone();
    let old_receipt = desired.state.sync.clone();
    desired.pages[0].blocks.pop();
    write(&path, &desired);
    *server.hook.lock().unwrap() = Some(Box::new(move |request, store| {
        if request.path == "/api/rpc/push" {
            let row = store.one(&c, "block/text").unwrap().clone();
            let mut changed = row.clone();
            changed["data"]["value"] = old::yrs_payload("unrequested detached corruption", None).into();
            // Deliberate hostile state change, not a Yjs author's merge.
            store.facts.insert(row["id"].as_str().unwrap().into(), changed);
        }
    }));
    let result = save(&server, temp.path(), &path, true).await;
    assert_eq!(result.outcome, "partial", "{result:?}");
    assert!(result.message.unwrap().contains("protected"));
    assert_eq!(read(&path).state.sync, old_receipt);
    assert!(read(&path).state.pending.is_some());
}

#[tokio::test]
async fn actual_save_source_races_preserve_new_bytes_and_only_unchanged_intent_can_settle() {
    for comment_only in [false, true] {
        let server = Server::new(true).await;
        let temp = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
        let mut desired = read(&path);
        let old_receipt = desired.state.sync.clone();
        desired.pages[0].blocks[1] = paragraph("B", "Saved desired revision");
        write(&path, &desired);
        let race = path.clone();
        *server.hook.lock().unwrap() = Some(Box::new(move |request, _| {
            if request.path == "/api/rpc/push" {
                let text = std::fs::read_to_string(&race).unwrap();
                std::fs::write(
                    &race,
                    if comment_only {
                        text.replacen("---\n", "---\n# author saved during request\n", 1)
                    } else {
                        text.replace("Saved desired revision", "Newer local revision")
                    },
                )
                .unwrap();
            }
        }));
        let result = save(&server, temp.path(), &path, true).await;
        assert_eq!(result.outcome, "source-changed", "{result:?}");
        let text = std::fs::read_to_string(&path).unwrap();
        let after = read(&path);
        assert_eq!(after.state.sync, old_receipt);
        assert!(after.state.pending.is_some());
        let pushes = server.store.lock().unwrap().pushes;
        let result = save(&server, temp.path(), &path, true).await;
        assert_eq!(result.outcome, if comment_only { "saved" } else { "pending" }, "{result:?}");
        assert_eq!(server.store.lock().unwrap().pushes, pushes);
        if comment_only {
            assert!(std::fs::read_to_string(&path).unwrap().contains("author saved during request"));
        } else {
            assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        }
    }
}

#[tokio::test]
async fn desired_visible_without_writer_finality_stays_pending_on_restart() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
    let mut desired = read(&path);
    desired.pages[0].blocks[1] = paragraph("B", "Changed but unacknowledged");
    write(&path, &desired);
    server.store.lock().unwrap().no_ack = true;
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "pending");
    let text = std::fs::read(&path).unwrap();
    let pushes = server.store.lock().unwrap().pushes;
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "pending");
    assert_eq!(std::fs::read(&path).unwrap(), text);
    assert_eq!(server.store.lock().unwrap().pushes, pushes);
}
