use super::*;
use crate::native::{RecoverOptions, ResolutionDirection, ResolveReport};

async fn recover(server: &Server, root: &Path, path: &Path, yes: bool, take: Option<ResolutionDirection>) -> Result<ResolveReport> {
    server
        .reader()
        .recover_editor(&old::context(root), path, Selection::default(), RecoverOptions { yes, take })
        .await
}

async fn partial(server: &Server, root: &Path, path: &Path) -> Attempt {
    assert_eq!(save(server, root, path, true).await.outcome, "saved");
    let mut desired = read(path);
    desired.pages[0].blocks[1] = paragraph("B", "Old intended revision");
    write(path, &desired);
    server.store.lock().unwrap().skip_all = true;
    assert_eq!(save(server, root, path, true).await.outcome, "partial");
    server.store.lock().unwrap().skip_all = false;
    read(path).state.pending.unwrap()
}

#[tokio::test]
async fn changed_intent_terminal_inspection_then_remote_adoption_preserves_old_provenance() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    let original = partial(&server, temp.path(), &path).await;
    let mut newer = read(&path);
    let nodes = newer.state.nodes.clone();
    let writer = newer.state.writer.clone();
    let binding = newer.state.binding.clone();
    newer.pages[0].blocks[1] = paragraph("B", "New local intent, never old success");
    write(&path, &newer);
    let text = std::fs::read_to_string(&path).unwrap().replacen("---\n", "---\n# source-only comment\n", 1);
    std::fs::write(&path, text).unwrap();
    let before = std::fs::read(&path).unwrap();
    let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    let pushes = server.store.lock().unwrap().pushes;
    let report = recover(&server, temp.path(), &path, false, None).await.unwrap();
    assert!(report.observation.terminal);
    assert_eq!(report.observation.source_matches_original, Some(false));
    assert_eq!(report.observation.outcome, RecoveryOutcome::ResolvedPartial);
    assert_eq!(
        recover(&server, temp.path(), &path, false, Some(ResolutionDirection::Remote))
            .await
            .unwrap()
            .outcome,
        "would-accept-remote"
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    assert_eq!(read(&path).state.pending, Some(original.clone()));
    assert_eq!(server.store.lock().unwrap().pushes, pushes);
    let result = recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Remote)).await.unwrap();
    assert_eq!(result.outcome, "accepted-remote", "{result:?}");
    let accepted = read(&path);
    assert_eq!(accepted.state.id, newer.state.id);
    assert_eq!(accepted.state.nodes, nodes);
    assert_eq!(accepted.state.binding, binding);
    assert_eq!(accepted.state.writer, writer);
    assert_eq!(accepted.extra["localOnly"], newer.extra["localOnly"]);
    assert!(std::fs::read_to_string(&path).unwrap().contains("# source-only comment"));
    let resolution = accepted.state.last_resolution.as_ref().unwrap();
    assert_eq!(resolution.attempt_id, original.id);
    assert_eq!(resolution.input_hash, original.input_hash);
    assert_eq!(resolution.desired_hash, original.desired_hash);
    assert!(resolution.acknowledged >= resolution.through);
    assert!(resolution.next_attempt.is_none());
    assert!(accepted.state.pending.is_none());
    assert!(server.reader().observe_editor(&accepted).await.unwrap().agreement.endpoints_equal);
    let bytes = std::fs::read(&path).unwrap();
    let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(
        recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Local))
            .await
            .unwrap()
            .outcome,
        "no-attempt"
    );
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    assert_eq!(server.store.lock().unwrap().pushes, pushes);
}

#[tokio::test]
async fn terminal_replan_saves_current_intent_with_new_attempt_and_never_reuses_reserved_suffix() {
    let server = Server::new(false).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    partial(&server, temp.path(), &path).await;
    let old_request = server.http.requests().into_iter().rfind(|r| r.path == "/api/rpc/push").unwrap().body_json();
    let mut desired = read(&path);
    // Emulate a terminal dispatched prefix plus a never-sent reservation suffix.
    if let Effect::Push { range, .. } = &mut desired.state.pending.as_mut().unwrap().effect {
        range.last += 300;
    }
    desired.state.writer.as_mut().unwrap().next_mutation_id += 300;
    desired.pages[0].blocks[1] = paragraph("B", "CURRENT local replan 😀");
    desired.pages[0].blocks.push(paragraph("D", "Actually new content"));
    write(&path, &desired);
    let original = desired.state.pending.clone().unwrap();
    let next_id = desired.state.writer.as_ref().unwrap().next_mutation_id;
    let original_bytes = std::fs::read(&path).unwrap();
    let preview = recover(&server, temp.path(), &path, false, Some(ResolutionDirection::Local)).await.unwrap();
    assert_eq!(preview.outcome, "would-replan-local");
    assert_eq!(std::fs::read(&path).unwrap(), original_bytes);
    // The pre-effect source contains BOTH old provenance and new pending identity.
    let capture = path.clone();
    let old_id = original.id;
    *server.hook.lock().unwrap() = Some(Box::new(move |r, _| {
        if r.path == "/api/rpc/push" {
            let doc = read(&capture);
            let summary = doc.state.last_resolution.unwrap();
            assert_eq!(summary.attempt_id, old_id);
            let pending = doc.state.pending.unwrap();
            assert_ne!(pending.id, old_id);
            assert_eq!(summary.next_attempt, Some(pending.id));
            let first = r.body_json()["pushRequest"]["mutations"][0]["id"].as_u64().unwrap();
            assert_eq!(first, next_id);
        }
    }));
    let result = recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Local)).await.unwrap();
    assert_eq!(result.outcome, "replanned-local", "{result:?}");
    let after = read(&path);
    assert_eq!(after.pages, desired.pages);
    assert_eq!(after.state.nodes["B"], desired.state.nodes["B"]);
    assert_eq!(after.state.last_resolution.as_ref().unwrap().input_hash, original.input_hash);
    assert_eq!(after.state.last_resolution.as_ref().unwrap().desired_hash, original.desired_hash);
    assert!(after.state.pending.is_none());
    {
        let mut store = server.store.lock().unwrap();
        let facts = store.facts.clone();
        store.rpc("/api/rpc/push", &old_request).unwrap();
        assert_eq!(store.facts, facts, "old delivered duplicate is ignored by the retained writer sequence");
    }
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
}

#[tokio::test]
async fn detached_partial_creations_are_reused_from_actual_rows_not_initialized_again() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    server.store.lock().unwrap().skip_attachments = true;
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "partial");
    let original = read(&path).state.pending.unwrap();
    let Effect::Push { targets, .. } = &original.effect else { panic!("push") };
    let entities = server.store.lock().unwrap().entities.clone();
    let old_payloads: Vec<_> = server
        .store
        .lock()
        .unwrap()
        .facts
        .values()
        .filter(|r| r["attribute"] == "block/text")
        .cloned()
        .collect();
    server.store.lock().unwrap().skip_attachments = false;
    let result = recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Local)).await.unwrap();
    assert_eq!(result.outcome, "replanned-local", "{result:?}");
    assert!(!result.effects.contains_key("create-entity"));
    let after = read(&path);
    for key in ["A", "B", "C"] {
        assert_eq!(after.state.nodes[key], targets[key]);
    }
    {
        let state = server.store.lock().unwrap();
        assert_eq!(state.entities, entities);
        for row in old_payloads {
            assert_eq!(state.facts[row["id"].as_str().unwrap()], row);
        }
    }
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
}

#[tokio::test]
async fn uncertain_or_lower_ack_cannot_be_overridden_even_after_local_intent_changes() {
    for take in [ResolutionDirection::Local, ResolutionDirection::Remote] {
        let server = Server::new(true).await;
        let temp = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
        let mut desired = read(&path);
        desired.pages[0].blocks[1] = paragraph("B", "Visible without writer finality");
        write(&path, &desired);
        server.store.lock().unwrap().no_ack = true;
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "pending");
        let mut changed = read(&path);
        changed.title = "Newer intent".into();
        write(&path, &changed);
        let bytes = std::fs::read(&path).unwrap();
        let pushes = server.store.lock().unwrap().pushes;
        let report = recover(&server, temp.path(), &path, true, Some(take)).await.unwrap();
        assert_eq!(report.outcome, "pending");
        assert!(!report.observation.terminal);
        assert_eq!(report.observation.source_matches_original, Some(false));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(server.store.lock().unwrap().pushes, pushes);
        assert!(read(&path).state.last_resolution.is_none());
    }
}

#[tokio::test]
async fn remote_acceptance_uses_pending_associations_and_original_success_remains_separate() {
    for complete in [false, true] {
        let server = Server::new(true).await;
        let temp = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        server.store.lock().unwrap().no_ack = true;
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "pending");
        let mut doc = read(&path);
        let old = doc.state.pending.clone().unwrap();
        let Effect::Push { targets, .. } = &old.effect else { panic!("push") };
        let writer = doc.state.writer.as_ref().unwrap();
        server
            .store
            .lock()
            .unwrap()
            .acks
            .entry(writer.client_group_id.to_string())
            .or_default()
            .insert(writer.client_id.to_string(), writer.next_mutation_id - 1);
        if !complete {
            doc.title = "Changed authored title".into();
            write(&path, &doc);
        }
        let pushes = server.store.lock().unwrap().pushes;
        let result = recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Remote)).await.unwrap();
        assert_eq!(result.outcome, if complete { "settled-prior" } else { "accepted-remote" });
        assert_eq!(read(&path).state.last_resolution.is_none(), complete);
        assert_eq!(read(&path).state.nodes, *targets);
        assert_eq!(server.store.lock().unwrap().pushes, pushes);
    }
}

#[tokio::test]
async fn unsupported_current_graph_or_unobserved_known_allocation_blocks_without_replacement() {
    for detached in [false, true] {
        let server = Server::new(true).await;
        let temp = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        server.store.lock().unwrap().skip_attachments = detached;
        server.store.lock().unwrap().skip_all = !detached;
        assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "partial");
        if detached {
            let pending = read(&path).state.pending.unwrap();
            let Effect::Push { targets, .. } = pending.effect else { panic!("push") };
            let entity = &targets["B"].entity;
            server.store.lock().unwrap().facts.retain(|_, r| r["entity"] != *entity);
        } else {
            server
                .store
                .lock()
                .unwrap()
                .facts
                .retain(|_, r| !(r["entity"] == store::SEED && r["attribute"] == "block/type"));
        }
        let before = std::fs::read(&path).unwrap();
        let pushes = server.store.lock().unwrap().pushes;
        for direction in [ResolutionDirection::Local, ResolutionDirection::Remote] {
            let result = recover(&server, temp.path(), &path, true, Some(direction)).await;
            if detached && direction == ResolutionDirection::Remote {
                // Accept-current can deliberately accept the complete empty page;
                // the missing detached allocation is not resurrected or deleted.
                assert_eq!(result.unwrap().outcome, "accepted-remote");
            } else {
                let blocked = result.unwrap();
                assert_eq!(blocked.outcome, "blocked");
                let error = blocked.message.unwrap();
                assert!(error.contains("incomplete/unsupported"), "{error}");
                assert_eq!(std::fs::read(&path).unwrap(), before);
            }
            assert_eq!(server.store.lock().unwrap().pushes, pushes);
        }
    }
}

#[tokio::test]
async fn resolution_source_duplicate_rights_and_native_drift_guards_preserve_pending() {
    for case in 0..4 {
        let server = Server::new(true).await;
        let temp = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        let original = partial(&server, temp.path(), &path).await;
        let outside = external.path().join("other.mdx");
        draft(&outside, "publication: null");
        let bytes = std::fs::read(&path).unwrap();
        let pushes = server.store.lock().unwrap().pushes;
        let count = Arc::new(AtomicUsize::new(0));
        let selected = path.clone();
        let duplicate = outside.clone();
        *server.hook.lock().unwrap() = Some(Box::new(move |request, store| {
            // First pull is inspect; second is the new decision's observation.
            if request.path == "/api/rpc/pull" && count.fetch_add(1, Ordering::SeqCst) == 1 {
                match case {
                    0 => {
                        let s = std::fs::read_to_string(&selected).unwrap();
                        std::fs::write(&selected, s.replacen("---\n", "---\n# concurrent author save\n", 1)).unwrap();
                    }
                    1 => {
                        std::fs::copy(&selected, &duplicate).unwrap();
                    }
                    2 => store.data["permission_token_rights"][0]["write"] = false.into(),
                    _ => store.data["description"] = "drift across await".into(),
                }
            }
        }));
        let result = server
            .reader()
            .recover_editor(
                &old::context(temp.path()),
                &path,
                Selection {
                    operands: vec![path.clone(), outside],
                },
                RecoverOptions {
                    yes: true,
                    take: Some(ResolutionDirection::Remote),
                },
            )
            .await;
        let report = result.unwrap();
        assert!(matches!(report.outcome, "blocked" | "source-changed"), "case {case}: {report:?}");
        assert_eq!(read(&path).state.pending, Some(original));
        assert!(read(&path).state.last_resolution.is_none());
        if case != 0 {
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
        assert_eq!(server.store.lock().unwrap().pushes, pushes);
    }
}

#[tokio::test]
async fn failed_new_replan_retains_previous_resolution_and_new_pending_without_false_receipt() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    let original = partial(&server, temp.path(), &path).await;
    let receipt = read(&path).state.sync;
    server.store.lock().unwrap().skip_all = true;
    let report = recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Local)).await.unwrap();
    assert_eq!(report.outcome, "partial");
    let after = read(&path);
    let new = after.state.pending.as_ref().unwrap();
    let summary = after.state.last_resolution.as_ref().unwrap();
    assert_ne!(new.id, original.id);
    assert_eq!(summary.attempt_id, original.id);
    assert_eq!(summary.next_attempt, Some(new.id));
    assert_eq!(after.state.sync, receipt);
    server.store.lock().unwrap().skip_all = false;
    assert_eq!(
        recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Local))
            .await
            .unwrap()
            .outcome,
        "replanned-local"
    );
    let final_doc = read(&path);
    assert_eq!(
        final_doc.state.last_resolution.as_ref().unwrap().attempt_id,
        new.id,
        "one bounded last decision, not an unbounded history"
    );
    assert!(final_doc.state.pending.is_none());
}

#[tokio::test]
async fn converged_current_metadata_settles_without_effects_but_missing_persisted_title_does_not() {
    let server = Server::new(false).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "saved");
    let mut doc = read(&path);
    doc.title = "Converged explicit title".into();
    write(&path, &doc);
    server.store.lock().unwrap().data["title"] = doc.title.clone().into();
    let pushes = server.store.lock().unwrap().pushes;
    let writer = doc.state.writer;
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
    assert_eq!(server.store.lock().unwrap().pushes, pushes);
    assert_eq!(read(&path).state.writer, writer);
    let text = std::fs::read(&path).unwrap();
    let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "unchanged");
    assert_eq!(std::fs::read(&path).unwrap(), text);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
    let mut doc = read(&path);
    doc.title = "Untitled".into();
    write(&path, &doc);
    server.store.lock().unwrap().data["title"] = Value::Null;
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "needs-decision");
}

#[tokio::test]
async fn current_intent_already_remote_replan_records_new_correspondence_without_an_attempt() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    let original = partial(&server, temp.path(), &path).await;
    let mut doc = read(&path);
    doc.pages[0].blocks[1] = paragraph("B", "Bravo and delta."); // current remote, not the old intended revision
    write(&path, &doc);
    let pushes = server.store.lock().unwrap().pushes;
    let writer = doc.state.writer.clone();
    let result = recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Local)).await.unwrap();
    assert_eq!(result.outcome, "replanned-local");
    assert!(result.effects.is_empty());
    let after = read(&path);
    assert!(after.state.pending.is_none());
    let resolution = after.state.last_resolution.as_ref().unwrap();
    assert_eq!(resolution.attempt_id, original.id);
    assert_eq!(resolution.desired_hash, original.desired_hash);
    assert!(resolution.next_attempt.is_none());
    assert_eq!(after.state.writer, writer);
    assert_eq!(server.store.lock().unwrap().pushes, pushes);
}

#[tokio::test]
async fn replan_rechecks_the_decision_before_checkpoint_or_new_native_effects() {
    for local in [false, true] {
        let server = Server::new(true).await;
        let temp = tempfile::tempdir().unwrap();
        let path = source(temp.path());
        let original = partial(&server, temp.path(), &path).await;
        let bytes = std::fs::read(&path).unwrap();
        let pushes = server.store.lock().unwrap().pushes;
        let selected = path.clone();
        let mut pulls = 0;
        *server.hook.lock().unwrap() = Some(Box::new(move |r, store| {
            if r.path == "/api/rpc/pull" {
                pulls += 1;
                if pulls == 4 {
                    // inspect, resolution, execution baseline, final pre-effect read
                    if local {
                        let text = std::fs::read_to_string(&selected)
                            .unwrap()
                            .replace("Old intended revision", "Concurrent newer intent");
                        std::fs::write(&selected, text).unwrap();
                    } else {
                        store.data["title"] = "Concurrent native title".into();
                    }
                }
            }
        }));
        let result = recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Local)).await.unwrap();
        assert!(matches!(result.outcome, "blocked" | "source-changed"), "{result:?}");
        assert_eq!(read(&path).state.pending, Some(original));
        assert!(read(&path).state.last_resolution.is_none());
        if !local {
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
        assert_eq!(server.store.lock().unwrap().pushes, pushes);
    }
}

#[tokio::test]
async fn connected_follower_stays_blocked_until_explicit_resolution_then_resumes_adoption() {
    use crate::{
        refresh::Outcome,
        watch::{
            self,
            tests::{Socket, copy_event, outcome},
        },
    };
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    let original = partial(&server, temp.path(), &path).await;
    let receipt = read(&path).state.sync;
    let socket = Socket::new().await;
    let context = old::context(temp.path());
    let reader = server.reader();
    let (cancel, receiver) = tokio::sync::watch::channel(false);
    let (tx, mut events) = tokio::sync::mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        watch::Request {
            files: vec![path.clone()],
            all: false,
        },
        socket.options(),
        receiver,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        outcome(&mut events, Outcome::Pending).await;
        assert_eq!(read(&path).state.pending, Some(original.clone()));
        assert_eq!(read(&path).state.sync, receipt);
        let accepted = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                match recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Remote)).await {
                    Ok(result) => break result,
                    Err(e) if e.to_string().contains("document busy") => tokio::time::sleep(std::time::Duration::from_millis(5)).await,
                    Err(e) => panic!("{e:#}"),
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(accepted.outcome, "accepted-remote");
        let summary = read(&path).state.last_resolution;
        socket.poke();
        outcome(&mut events, Outcome::Unchanged).await;
        server
            .store
            .lock()
            .unwrap()
            .extras
            .insert("publication_description".into(), json!("Remote edit after deliberate resolution"));
        socket.poke();
        outcome(&mut events, Outcome::Updated).await;
        assert_eq!(read(&path).description.as_deref(), Some("Remote edit after deliberate resolution"));
        assert_eq!(read(&path).state.last_resolution, summary);
        assert!(read(&path).state.pending.is_none());
        cancel.send_replace(true);
    };
    let (result, ()) = tokio::join!(run, actions);
    assert_eq!(result.unwrap().writes, 1);
}

#[tokio::test]
async fn prior_settlement_does_not_claim_a_separately_requested_recreation_ran() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    server.store.lock().unwrap().no_ack = true;
    assert_eq!(save(&server, temp.path(), &path, true).await.outcome, "pending");
    let doc = read(&path);
    let Effect::Push { targets, .. } = &doc.state.pending.as_ref().unwrap().effect else {
        panic!("push")
    };
    let original_a = targets["A"].clone();
    let writer = doc.state.writer.as_ref().unwrap();
    {
        let mut state = server.store.lock().unwrap();
        state.no_ack = false;
        state
            .acks
            .entry(writer.client_group_id.to_string())
            .or_default()
            .insert(writer.client_id.to_string(), writer.next_mutation_id - 1);
    }
    let pushes = server.store.lock().unwrap().pushes;
    let options = || SaveOptions {
        yes: true,
        recreate: ["A".into()].into(),
        ..Default::default()
    };
    let report = server
        .reader()
        .save_native(&old::context(temp.path()), &path, Selection::default(), options())
        .await
        .unwrap();
    assert_eq!(report.outcome, "settled-prior");
    assert!(report.message.unwrap().contains("no new save or --recreate action ran"));
    assert!(report.effects.is_empty());
    assert_eq!(read(&path).state.nodes["A"], original_a);
    assert_eq!(server.store.lock().unwrap().pushes, pushes);
    let report = server
        .reader()
        .save_native(&old::context(temp.path()), &path, Selection::default(), options())
        .await
        .unwrap();
    assert_eq!(report.outcome, "saved");
    assert_ne!(read(&path).state.nodes["A"].entity, original_a.entity);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancellation_during_resolution_observation_releases_lock_without_forgetting_pending() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    let original = partial(&server, temp.path(), &path).await;
    let bytes = std::fs::read(&path).unwrap();
    let entered = Arc::new(tokio::sync::Notify::new());
    let notice = entered.clone();
    let (release, wait) = std::sync::mpsc::channel();
    let mut held = false;
    *server.hook.lock().unwrap() = Some(Box::new(move |r, _| {
        if r.path == "/api/rpc/pull" && !held {
            held = true;
            notice.notify_one();
            wait.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        }
    }));
    let context = old::context(temp.path());
    let reader = server.reader();
    let mut operation = Box::pin(reader.recover_editor(
        &context,
        &path,
        Selection::default(),
        RecoverOptions {
            yes: true,
            take: Some(ResolutionDirection::Remote),
        },
    ));
    tokio::select! {
        result = &mut operation => panic!("unexpected completion: {result:?}"),
        entered = tokio::time::timeout(std::time::Duration::from_secs(3), entered.notified()) => { entered.expect("observation entered"); },
    }
    assert!(
        Source::open(&context, &path, Selection::default()).is_err(),
        "cooperating writer must not enter while observation holds UUID lock"
    );
    drop(operation);
    assert!(
        Source::open(&context, &path, Selection::default()).is_ok(),
        "cancellation releases UUID lock immediately"
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(read(&path).state.pending, Some(original));
    release.send(()).unwrap();
}

#[tokio::test]
async fn resolution_provenance_survives_tidy_and_future_unknown_resolution_is_never_dropped() {
    let server = Server::new(true).await;
    let temp = tempfile::tempdir().unwrap();
    let path = source(temp.path());
    partial(&server, temp.path(), &path).await;
    assert_eq!(
        recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Remote))
            .await
            .unwrap()
            .outcome,
        "accepted-remote"
    );
    let context = old::context(temp.path());
    let summary = read(&path).state.last_resolution;
    let tidy = crate::tidy::run(
        &context,
        crate::tidy::Request {
            files: vec![path.clone()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(tidy.failed, 0);
    assert_eq!(read(&path).state.last_resolution, summary);
    for (from, accept) in [(None, true), (Some(crate::names::Direction::Filename), false)] {
        let before = read(&path).state;
        let tidy = crate::tidy::run(
            &context,
            crate::tidy::Request {
                files: vec![path.clone()],
                from,
                accept,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(tidy.failed, 0);
        assert_eq!(tidy.needs_choice, 0);
        let mut after = read(&path).state;
        after.names = before.names.clone();
        assert_eq!(
            after, before,
            "names do not rewrite original attempt/resolution, writer, binding or receipt evidence"
        );
    }
    let text = std::fs::read_to_string(&path).unwrap();
    let mut header = leafbud_mdx::header(&text).unwrap();
    header["@leafbud"]["lastResolution"]["futureEvidence"] = json!({"keep":"unknown future state"});
    let source = format!(
        "---\n{}\n---\n{}",
        serde_json::to_string(&header).unwrap(),
        text.split_once("\n---\n").unwrap().1
    );
    std::fs::write(&path, &source).unwrap();
    assert!(recover(&server, temp.path(), &path, true, Some(ResolutionDirection::Remote)).await.is_err());
    assert_eq!(
        server.reader().refresh_document(&context, &path, Default::default()).await.outcome,
        crate::refresh::Outcome::Invalid
    );
    assert_eq!(
        crate::status::inspect(&context, crate::status::Request { files: vec![path.clone()] }, Some(&server.reader()))
            .await
            .unwrap()
            .items[0]
            .outcome,
        "invalid"
    );
    assert_eq!(
        crate::tidy::run(
            &context,
            crate::tidy::Request {
                files: vec![path.clone()],
                accept: true,
                ..Default::default()
            }
        )
        .unwrap()
        .failed,
        1
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
}
