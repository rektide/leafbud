//! Explicit opt-in deployed verification, never part of normal tests. Budget
//! markers are created before mint and never removed/reused automatically.
use super::*;
use serde_json::Value;
use std::io::Write as _;

#[tokio::test]
#[ignore = "requires operator permission for at most two new unpublished drafts"]
async fn authorized_two_disposable_drafts() -> Result<()> {
    ensure!(
        std::env::var("LEAFBUD_NATIVE_PROBE").as_deref() == Ok("authorized-two-disposable-drafts"),
        "explicit probe authorization flag required"
    );
    let root = std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-effects"))?;
    let dir = root.join("deployed");
    std::fs::create_dir_all(&dir)?;
    let mut context = crate::config::load(None, &Default::default(), None)?;
    let publication = context.effective.publication.clone();
    context.workspace_root = std::fs::canonicalize(&dir)?;
    let remote = crate::RemoteOptions::default().connect().await?;
    let selected = std::env::var("LEAFBUD_NATIVE_PROBE_CONTEXT").ok();
    ensure!(
        selected.as_deref().is_none_or(|v| matches!(v, "standalone" | "publication")),
        "invalid probe context selector"
    );
    for (label, publication) in [("standalone", None), ("publication", publication)] {
        if selected.as_deref().is_some_and(|v| v != label) {
            continue;
        }
        if label == "publication" && publication.is_none() {
            eprintln!("configured publication absent; publication probe not attempted");
            continue;
        }
        let marker = dir.join(format!("{label}.started"));
        // No attempt, including a lost response, can allocate a second draft.
        let mut budget = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&marker)
            .context("draft budget already consumed; inspect private probe state, never remint")?;
        writeln!(
            budget,
            "Operator authorized one {label} draft in session ses_e4217c526ffeXeHByo5ZswnB3F; consumed before mint. No automatic cleanup/retry."
        )?;
        budget.sync_all()?;
        let path = dir.join(format!("{label}.mdx"));
        let title = format!("Leafbud verification — {label} native-effect foundation");
        let body = if label == "standalone" {
            "# Leafbud native-effect verification."
        } else {
            "Leafbud native-effect verification."
        };
        let source = format!(
            "---\nformat: leafbud-mdx/2\ntitle: {}\npublication: {}\ntheme: {{leafletWatermark: true}}\n\"@leafbud\": {{\"id\":\"{}\",\"state\":\"draft\"}}\n---\n\n{body}\n",
            serde_json::to_string(&title)?,
            serde_json::to_string(&publication)?,
            Uuid::new_v4()
        );
        files::write(&path, None, source.as_bytes())?;
        let result = remote.stage_binding(&context, &path, Selection::default()).await?;
        if let Some(response) = &result.undecoded_mint {
            std::fs::write(dir.join(format!("{label}.undecoded.flight")), response.body())?;
        }
        // Persist a returned locator even if the guarded source could not store
        // it. This private receipt is probe bookkeeping, not a tool journal.
        std::fs::write(dir.join(format!("{label}.locator.json")), serde_json::to_vec_pretty(&result)?)?;
        ensure!(
            matches!(result.outcome, BindingOutcome::PendingMaterialization),
            "mint did not verify; inspect private state; NO REMINT"
        );
        let original_id = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?.state.id;
        let report = remote
            .execute_compiled(&context, &path, Selection::default(), None, |desired, data, view| {
                let fresh = import::build(remote.client.base_url(), data, view)?;
                let main = desired.pages.first().context("prepared main")?;
                let authored = main.blocks.first().context("controlled source block")?;
                ensure!(
                    main.blocks.len() == 1 && desired.pages.len() == 1,
                    "probe only understands one controlled block"
                );
                let native_main = fresh.state.nodes.get(&fresh.pages[0].key).context("native main")?.clone();
                let native_block = fresh.state.nodes.get(&fresh.pages[0].blocks[0].key).context("native seed")?.clone();
                let leafbud_document::Occurrence::Block {
                    placement,
                    text_fact: None,
                    inactive_text_fact: None,
                } = &native_block.occurrence
                else {
                    anyhow::bail!("probe requires an empty seed with no existing CRDT");
                };
                let text_id = Uuid::new_v4();
                let mut node = native_block.clone();
                node.occurrence = leafbud_document::Occurrence::Block {
                    placement: placement.clone(),
                    text_fact: Some(text_id.to_string()),
                    inactive_text_fact: None,
                };
                let payload = crate::pull::tests::yrs_payload("Leafbud native-effect verification.", None);
                let metadata = match publication {
                    Some(_) => NativeMutation::UpdatePublicationDraft(leafbud_protocol::PublicationMetadata {
                        title: Some(desired.title.clone()),
                        ..Default::default()
                    }),
                    None => NativeMutation::UpdateLeafletMetadata(leafbud_protocol::LeafletMetadata {
                        title: Some(desired.title.clone()),
                        ..Default::default()
                    }),
                };
                Ok(Compiled {
                    mutations: vec![
                        metadata,
                        NativeMutation::AssertFact(leafbud_protocol::AssertFact {
                            id: Some(text_id),
                            entity: native_block.entity.parse()?,
                            attribute: "block/text".into(),
                            data: serde_json::json!({"type":"text","value":payload}),
                        }),
                    ],
                    targets: [(main.key.clone(), native_main), (authored.key.clone(), node)].into(),
                    created_entities: vec![],
                    allocated_facts: vec![text_id],
                })
            })
            .await?;
        std::fs::write(dir.join(format!("{label}.first-effect-report.json")), serde_json::to_vec_pretty(&report)?)?;
        ensure!(
            report.outcome == RecoveryOutcome::DesiredVerified && report.settled,
            "controlled native effect not verified; inspect pending, do not remint"
        );
        let doc = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
        ensure!(doc.state.id == original_id, "local UUID changed");
        let first_nodes = doc.state.nodes.clone();
        let block_entity = doc
            .state
            .nodes
            .values()
            .find(|n| matches!(n.occurrence, leafbud_document::Occurrence::Block { .. }))
            .context("verified block")?
            .entity
            .parse()?;
        let explicit = remote.client.facts_for_entities(&[block_entity]).await?;
        ensure!(
            explicit
                .roots
                .get(&block_entity)
                .is_some_and(|rows| rows.iter().any(|r| r["attribute"] == "block/text")),
            "deployed explicit-ID transport did not return the known text fact"
        );
        let mut source = Source::open(&context, &path, Selection::default())?;
        let binding = source.document.state.binding.clone().context("binding")?;
        let mut writer = source.document.state.writer.clone().context("writer")?;
        let before = remote.native_observation(&binding, Some(&writer), &[]).await?;
        source.check()?;
        ensure!(
            import::remote_hash(&before.data, &before.snapshot) == source.document.state.sync.as_ref().context("receipt")?.remote_hash,
            "probe drift before recovery proof"
        );
        // Same-value metadata effect deliberately tests identity/dedup/recovery;
        // it is NOT the complete keyed authoring compiler's no-op policy.
        let metadata = match publication {
            Some(_) => NativeMutation::UpdatePublicationDraft(leafbud_protocol::PublicationMetadata {
                title: Some(source.document.title.clone()),
                ..Default::default()
            }),
            None => NativeMutation::UpdateLeafletMetadata(leafbud_protocol::LeafletMetadata {
                title: Some(source.document.title.clone()),
                ..Default::default()
            }),
        };
        let compiled = Compiled {
            mutations: vec![metadata],
            targets: BTreeMap::new(),
            created_entities: vec![],
            allocated_facts: vec![],
        };
        let protection = protection(&before, &compiled)?;
        let encoded = leafbud_protocol::encode_pushes(
            wire_target(&binding)?,
            wire_writer(&writer),
            writer.next_mutation_id,
            0,
            &compiled.mutations,
            &scope(&before, &compiled, &binding)?,
            BatchLimits::default(),
        )?;
        let range = writer.reserve(1).map_err(anyhow::Error::msg)?;
        let mut state = source.document.state.clone();
        state.writer = Some(writer);
        state.pending = Some(Attempt {
            id: Uuid::new_v4(),
            input_hash: import::hash(&source.document.authored_value()),
            desired_hash: import::hash(&source.document.editor_value()),
            effect: Effect::Push {
                range,
                stage: PushStage::Dispatched { through: range.last },
                protection,
                targets: BTreeMap::new(),
                created_entities: vec![],
            },
        });
        source.checkpoint(state)?;
        remote.client.send_push(&encoded[0]).await?;
        remote.client.send_push(&encoded[0]).await?; // identical client/range/bytes
        drop(source); // emulate restart: request bytes no longer drive recovery
        let recovered = RemoteReader::new(remote.client.clone())
            .recover_native(&context, &path, Selection::default(), true)
            .await?;
        ensure!(
            recovered.outcome == RecoveryOutcome::DesiredVerified && recovered.settled,
            "restart observation did not settle known terminal desired state"
        );
        let after = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
        ensure!(after.state.nodes == first_nodes, "native IDs churned");
        let current = remote.client.get(binding.token).await?;
        ensure!(current.observed_state() == LeafletState::Draft, "verification object unexpectedly published");
        let observed = remote
            .observe_editor(&after)
            .await
            .map_err(|_| anyhow::anyhow!("final bound observation failed"))?;
        ensure!(observed.agreement.endpoints_equal, "final read-back disagrees");
        let original = std::fs::read(&path)?;
        let mtime = std::fs::metadata(&path)?.modified()?;
        let no_op = remote
            .execute_compiled(&context, &path, Selection::default(), None, |_, _, _| {
                Ok(Compiled {
                    mutations: vec![],
                    targets: BTreeMap::new(),
                    created_entities: vec![],
                    allocated_facts: vec![],
                })
            })
            .await?;
        ensure!(
            no_op.outcome == RecoveryOutcome::DesiredVerified && std::fs::read(&path)? == original && std::fs::metadata(&path)?.modified()? == mtime,
            "controlled unchanged read was not exact no-op"
        );
        std::fs::write(
            dir.join(format!("{label}.evidence.json")),
            serde_json::to_vec_pretty(
                &serde_json::json!({"mint":"verified","controlledTextAndMetadata":"verified","exactRequestReplay":"verified","restartSettlement":"verified","nativeIdentity":"retained","unchangedRead":"byte-and-mtime-noop","lifecycle":"unpublished-draft","completeKeyedCompiler":false}),
            )?,
        )?;
        eprintln!("{label}: mint, controlled native text/metadata, exact replay, restart settlement and unchanged read verified; left unpublished");
    }
    Ok(())
}

#[tokio::test]
#[ignore = "read/settle only the privately recorded authorized verification drafts; never creates"]
async fn inspect_existing_authorized_probe() -> Result<()> {
    ensure!(
        std::env::var("LEAFBUD_NATIVE_PROBE").as_deref() == Ok("authorized-two-disposable-drafts"),
        "explicit probe flag required"
    );
    let dir = std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-effects/deployed"))?;
    let mut context = crate::config::load(None, &Default::default(), None)?;
    context.workspace_root = dir.clone();
    let remote = crate::RemoteOptions::default().connect().await?;
    for label in ["standalone", "publication"] {
        let path = dir.join(format!("{label}.mdx"));
        if !path.exists() {
            continue;
        }
        let recorded: Value = serde_json::from_slice(&std::fs::read(dir.join(format!("{label}.locator.json")))?)?;
        let source = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
        let selected = dir.join(format!("{label}.selected-p1.private.json"));
        let selected_token = if recorded["token"].is_null() && selected.exists() {
            // Original response evidence remains unchanged. A later explicit
            // human selection establishes a recovery object, not attribution.
            let selected: Value = serde_json::from_slice(&std::fs::read(selected)?)?;
            ensure!(selected["selection"] == "human-selected-P1", "unknown recovery selection record");
            selected["selectionReport"]["token"].as_str().map(str::parse::<Uuid>).transpose()?
        } else {
            None
        };
        let token = recorded["token"].as_str().map(str::parse::<Uuid>).transpose()?.or(selected_token);
        if token.is_none() {
            ensure!(
                source.state.binding.is_none()
                    && matches!(
                        &source.state.pending,
                        Some(Attempt {
                            effect: Effect::Mint {
                                stage: MintStage::Dispatched,
                                ..
                            },
                            ..
                        })
                    ),
                "unlocated probe state differs from budget record"
            );
        } else {
            let known = source
                .state
                .binding
                .as_ref()
                .map(|b| b.token)
                .or_else(|| match &source.state.pending.as_ref()?.effect {
                    Effect::Mint {
                        stage: MintStage::Located { token },
                        ..
                    } => Some(*token),
                    _ => None,
                });
            ensure!(known == token, "probe locator differs from authorization record");
        }
        let report = remote.recover_native(&context, &path, Selection::default(), true).await?;
        if let Some(Attempt {
            effect: Effect::Push { targets, .. },
            ..
        }) = &source.state.pending
        {
            for node in targets.values().filter(|n| matches!(n.occurrence, leafbud_document::Occurrence::Block { .. })) {
                let entity = node.entity.parse()?;
                let explicit = remote.client.facts_for_entities(&[entity]).await?;
                let found = match &node.occurrence {
                    leafbud_document::Occurrence::Block { text_fact: Some(id), .. } => explicit.fact(entity, id.parse()?).is_some(),
                    _ => false,
                };
                std::fs::write(
                    dir.join(format!("{label}.explicit.json")),
                    serde_json::to_vec_pretty(
                        &serde_json::json!({"knownEntityRead":true,"knownTextFactFound":found,"returnedRows":explicit.roots[&entity].len()}),
                    )?,
                )?;
                ensure!(found, "known deployed text fact not returned by explicit entity read");
                eprintln!("{label}: deployed known-entity/known-text-fact explicit read verified");
            }
        }
        if report.outcome == RecoveryOutcome::ResolvedPartial {
            let binding = source.state.binding.as_ref().context("binding")?;
            let observed = remote.native_observation(binding, source.state.writer.as_ref(), &[]).await?;
            let fresh = import::build(remote.client.base_url(), &observed.data, &observed.snapshot)?;
            let mut expected = source.clone();
            if let Some(Attempt {
                effect: Effect::Push { targets, .. },
                ..
            }) = &source.state.pending
            {
                expected.state.nodes.extend(targets.clone());
            }
            let actual = crate::editor_state::adopt(&expected, &fresh);
            expected.state.nodes = actual.state.nodes.clone();
            expected.publication = binding.publication.clone().map(Publication::At).unwrap_or(Publication::Standalone);
            let a = expected.editor_value();
            let b = actual.editor_value();
            let keys: Vec<_> = a.as_object().context("projection")?.keys().filter(|k| a[*k] != b[*k]).collect();
            std::fs::write(
                dir.join(format!("{label}.comparison.json")),
                serde_json::to_vec_pretty(&serde_json::json!({"expected":a,"actual":b}))?,
            )?;
            eprintln!("{label}: differing projection fields: {keys:?}");
        }
        std::fs::write(dir.join(format!("{label}.inspection.json")), serde_json::to_vec_pretty(&report)?)?;
        eprintln!("{label}: {}", serde_json::to_string(&report)?);
    }
    Ok(())
}

/// Human selection relayed by parent ses_e4217c526ffeXeHByo5ZswnB3F. This test
/// cannot create a draft or discover/select a different candidate. It is also
/// restart-safe: a push is only observed, never independently re-encoded.
#[tokio::test]
#[ignore = "requires explicit P1 recovery selection; only existing authorized verification objects"]
async fn authorized_selected_p1_create_free_materialization() -> Result<()> {
    ensure!(
        std::env::var("LEAFBUD_NATIVE_PROBE").as_deref() == Ok("authorized-selected-p1"),
        "explicit selected-P1 authorization flag required"
    );
    let root = std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-effects"))?;
    let dir = root.join("deployed");
    ensure!(
        dir.join("standalone.started").exists() && dir.join("publication.started").exists(),
        "existing consumed creation budgets required; no mint in this harness"
    );
    let catalogue: Value = serde_json::from_slice(&std::fs::read(root.join("recovery-candidates/catalogue.private.json"))?)?;
    let candidate = catalogue["candidates"]
        .as_array()
        .context("private candidate catalogue")?
        .iter()
        .find(|c| c["label"] == "P1")
        .context("human-selected P1 missing")?;
    let token: Uuid = candidate["token"].as_str().context("P1 token missing")?.parse()?;
    let mut context = crate::config::load(None, &Default::default(), None)?;
    context.workspace_root = dir.clone();
    let remote = crate::RemoteOptions::default().connect().await?;
    let path = dir.join("publication.mdx");
    // Read-only check of the original standalone attempt; no policy rewrite,
    // receipt settlement or independent re-encoding of its dispatched intent.
    let standalone = dir.join("standalone.mdx");
    let old = std::fs::read(&standalone)?;
    let standalone_report = remote.recover_native(&context, &standalone, Selection::default(), false).await?;
    ensure!(std::fs::read(&standalone)? == old, "standalone inspection changed original evidence");
    std::fs::write(
        dir.join("standalone.default-policy-inspection.json"),
        serde_json::to_vec_pretty(&standalone_report)?,
    )?;
    let original_bytes = std::fs::read(&path)?;
    let source = leafbud_mdx::parse(std::str::from_utf8(&original_bytes)?)?;
    let local_id = source.state.id;
    if let Some(Attempt {
        id,
        effect: Effect::Mint { publication, stage, .. },
        ..
    }) = &source.state.pending
    {
        ensure!(
            publication.publication() == candidate["publication"].as_str(),
            "selected candidate differs from pinned pending publication"
        );
        match stage {
            MintStage::Dispatched | MintStage::Located { .. } => {
                let selected = remote.select_mint_locator(&context, &path, Selection::default(), *id, token).await?;
                std::fs::write(
                    dir.join("publication.selected-p1.private.json"),
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "parentSelectionSession":"opencode://ses_e4217c526ffeXeHByo5ZswnB3F",
                        "implementationSession":"opencode://ses_e3dcb1f8effebiT0KtCRZB8dWf",
                        "selection":"human-selected-P1", "originalAttribution":"not-established",
                        "previousAttempt":source.state.pending, "selectionReport":selected,
                    }))?,
                )?;
                ensure!(
                    matches!(selected.outcome, BindingOutcome::PendingMaterialization),
                    "selected P1 did not verify/prepare; retained, no remint/retry"
                );
            }
            MintStage::Prepared => anyhow::bail!("probe never dispatched; selected-locator handoff is not applicable"),
        }
    }
    let source = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    ensure!(
        source.state.binding.as_ref().is_some_and(|b| b.token == token),
        "binding differs from human-selected recovery object"
    );
    if matches!(
        source.state.pending.as_ref().map(|a| &a.effect),
        Some(Effect::Mint {
            stage: MintStage::Located { .. },
            ..
        })
    ) {
        let report = remote
            .execute_compiled(&context, &path, Selection::default(), None, |desired, data, view| {
                let fresh = import::build(remote.client.base_url(), data, view)?;
                ensure!(
                    desired.pages.len() == 1 && desired.pages[0].blocks.len() == 1 && desired.description.is_none() && desired.tags.is_empty(),
                    "controlled probe requires its original single-block source without extra metadata"
                );
                let authored = &desired.pages[0].blocks[0];
                let leafbud_document::BlockKind::Paragraph { content } = &authored.kind else {
                    anyhow::bail!("controlled publication paragraph required")
                };
                ensure!(
                    content
                        == &vec![leafbud_document::Inline::Run {
                            text: "Leafbud native-effect verification.".into(),
                            marks: vec![]
                        }],
                    "controlled source text differs; no write"
                );
                let native_main = fresh.state.nodes[&fresh.pages[0].key].clone();
                let mut native_block = fresh.state.nodes[&fresh.pages[0].blocks[0].key].clone();
                let leafbud_document::Occurrence::Block {
                    placement,
                    text_fact: None,
                    inactive_text_fact: None,
                } = &native_block.occurrence
                else {
                    anyhow::bail!("controlled publication seed is no longer empty; no independent CRDT initialization")
                };
                let text_id = Uuid::new_v4();
                native_block.occurrence = leafbud_document::Occurrence::Block {
                    placement: placement.clone(),
                    text_fact: Some(text_id.to_string()),
                    inactive_text_fact: None,
                };
                Ok(Compiled {
                    mutations: vec![
                        NativeMutation::UpdatePublicationDraft(leafbud_protocol::PublicationMetadata {
                            title: Some(desired.title.clone()),
                            ..Default::default()
                        }),
                        NativeMutation::AssertFact(leafbud_protocol::AssertFact {
                            id: Some(text_id),
                            entity: native_block.entity.parse()?,
                            attribute: "block/text".into(),
                            data: serde_json::json!({"type":"text","value":crate::pull::tests::yrs_payload("Leafbud native-effect verification.", None)}),
                        }),
                    ],
                    targets: [(desired.pages[0].key.clone(), native_main), (authored.key.clone(), native_block)].into(),
                    created_entities: vec![],
                    allocated_facts: vec![text_id],
                })
            })
            .await?;
        std::fs::write(dir.join("publication.selected-p1.effect.json"), serde_json::to_vec_pretty(&report)?)?;
        if report.outcome == RecoveryOutcome::ResolvedPartial {
            inspect_selected_partial(&remote, &path, &dir).await?;
        }
        ensure!(
            report.outcome == RecoveryOutcome::DesiredVerified && report.settled,
            "controlled P1 effect not verified; retain pending and inspect, no re-encoding"
        );
    } else if source.state.pending.is_some() {
        let report = remote.recover_native(&context, &path, Selection::default(), true).await?;
        if report.outcome == RecoveryOutcome::ResolvedPartial {
            inspect_selected_partial(&remote, &path, &dir).await?;
        }
        ensure!(
            report.outcome == RecoveryOutcome::DesiredVerified && report.settled,
            "existing P1 push is not fully resolved; no retry/re-encoding"
        );
    }
    let doc = leafbud_mdx::parse(&std::fs::read_to_string(&path)?)?;
    ensure!(
        doc.state.id == local_id && doc.state.pending.is_none() && doc.state.sync.is_some(),
        "P1 source identity/settlement mismatch"
    );
    let observed = remote.observe_editor(&doc).await.map_err(|_| anyhow::anyhow!("P1 final observation failed"))?;
    ensure!(
        observed.agreement.endpoints_equal && observed.agreement.state == "clean" && observed.data.observed_state() == LeafletState::Draft,
        "P1 final read-back is not clean unpublished desired state"
    );
    for node in doc
        .state
        .nodes
        .values()
        .filter(|n| matches!(n.occurrence, leafbud_document::Occurrence::Block { .. }))
    {
        let entity = node.entity.parse()?;
        let explicit = remote.client.facts_for_entities(&[entity]).await?;
        let leafbud_document::Occurrence::Block { text_fact: Some(id), .. } = &node.occurrence else {
            anyhow::bail!("P1 verified text fact missing")
        };
        ensure!(explicit.fact(entity, id.parse()?).is_some(), "P1 explicit known text fact read failed");
    }
    let bytes = std::fs::read(&path)?;
    let mtime = std::fs::metadata(&path)?.modified()?;
    let pull = remote.refresh_document(&context, &path, Default::default()).await;
    ensure!(pull.outcome == crate::refresh::Outcome::Unchanged, "P1 pull was not unchanged");
    let no_op = remote
        .execute_compiled(&context, &path, Selection::default(), None, |_, _, _| {
            Ok(Compiled {
                mutations: vec![],
                targets: BTreeMap::new(),
                created_entities: vec![],
                allocated_facts: vec![],
            })
        })
        .await?;
    ensure!(
        no_op.outcome == RecoveryOutcome::DesiredVerified && std::fs::read(&path)? == bytes && std::fs::metadata(&path)?.modified()? == mtime,
        "P1 repeat was not exact no-op"
    );
    std::fs::write(
        dir.join("publication.selected-p1.evidence.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "selection":"human-selected-P1", "originalMintAttribution":"not-established", "creationCalls":0,
            "verifiedSeedAndRights":true, "defaultsRecordedInSource":true, "controlledTitleAndText":"verified",
            "knownTextFactRead":"verified", "wholeDesiredAndProtectedScope":"verified", "pull":"unchanged", "repeat":"byte-and-mtime-noop",
            "lifecycle":"unpublished-draft", "completeKeyedCompiler":false, "standalone":standalone_report,
        }))?,
    )?;
    eprintln!(
        "P1: human-selected existing recovery target; verified defaults, controlled text/title, full read-back, known text fact, unchanged pull/repeat; no creation/publication/mail. Standalone remains {:?}, original evidence preserved.",
        standalone_report.outcome
    );
    Ok(())
}

async fn inspect_selected_partial(remote: &RemoteReader, path: &Path, dir: &Path) -> Result<()> {
    let source = leafbud_mdx::parse(&std::fs::read_to_string(path)?)?;
    let binding = source.state.binding.as_ref().context("selected binding")?;
    let observed = remote.native_observation(binding, source.state.writer.as_ref(), &[]).await?;
    let fresh = import::build(remote.client.base_url(), &observed.data, &observed.snapshot)?;
    let mut expected = source.clone();
    if let Some(Attempt {
        effect: Effect::Push { targets, .. },
        ..
    }) = &source.state.pending
    {
        expected.state.nodes.extend(targets.clone());
    }
    let actual = crate::editor_state::adopt(&expected, &fresh);
    expected.state.nodes = actual.state.nodes.clone();
    let a = expected.editor_value();
    let b = actual.editor_value();
    let differences: Vec<_> = a.as_object().context("projection")?.keys().filter(|k| a[*k] != b[*k]).collect();
    let protection_matches = match source.state.pending.as_ref().map(|a| &a.effect) {
        Some(Effect::Push { protection, .. }) => protection_hash(&observed, protection) == protection.hash,
        _ => false,
    };
    let mut known_text_facts = 0;
    for node in actual.state.nodes.values() {
        if let leafbud_document::Occurrence::Block { text_fact: Some(id), .. } = &node.occurrence {
            let entity = node.entity.parse()?;
            let explicit = remote.client.facts_for_entities(&[entity]).await?;
            ensure!(
                explicit.fact(entity, id.parse()?).is_some(),
                "selected partial's known text fact not found through explicit entity read"
            );
            known_text_facts += 1;
        }
    }
    std::fs::write(
        dir.join("publication.selected-p1.comparison.private.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"expected":a,"actual":b,"differences":differences,"protectedScopeMatches":protection_matches,"explicitKnownTextFacts":known_text_facts}),
        )?,
    )?;
    eprintln!("P1 terminal partial; differing projection fields: {differences:?}; old intent retained, no retry/re-encoding");
    Ok(())
}
