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
        if recorded["token"].is_null() {
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
            ensure!(
                source.state.binding.as_ref().is_some_and(|b| recorded["token"] == b.token.to_string()),
                "probe locator differs from authorization record"
            );
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
