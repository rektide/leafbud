use super::*;

pub(super) fn prepare(builder: &mut Builder<'_>, current: &Content, recreate: &BTreeSet<String>) -> Result<()> {
    let mut shared: BTreeMap<String, BlockKind> = BTreeMap::new();
    let mut written: BTreeMap<String, (Option<String>, bool)> = BTreeMap::new();
    for page in &builder.desired.pages {
        for block in &page.blocks {
            let prior = builder.desired.state.nodes.get(&block.key);
            if let Some(prior) = prior {
                ensure!(matches!(prior.occurrence, Occurrence::Block { .. }), "block key addresses a page");
                ensure!(
                    current.nodes.values().any(|n| n.entity == prior.entity && same_placement(n, prior)),
                    "confirmed block occurrence absent/changed"
                );
            }
            let replacement = recreate.contains(&block.key);
            let entity: Uuid = match prior {
                Some(prior) if !replacement => prior.entity.parse()?,
                _ => builder.create(),
            };
            let entity_key = entity.to_string();
            if let Some(previous) = shared.insert(entity_key.clone(), block.kind.clone()) {
                ensure!(
                    previous == block.kind,
                    "shared entity requests divergent content; use --recreate KEY to fork one occurrence"
                );
            }
            let (text_fact, active) = if let Some(done) = written.get(&entity_key) {
                done.clone()
            } else {
                // Retained supported meaning, not raw default spelling, decides
                // whether this block needs work. Do not initialize absent blank
                // text/default scalars or clear inactive fields on an import.
                let unchanged = prior.is_some()
                    && !replacement
                    && current.pages.iter().flat_map(|p| &p.blocks).any(|old| {
                        current.nodes[&old.key].entity == entity_key
                            && match (&old.kind, &block.kind) {
                                (BlockKind::PageLink { page: a }, BlockKind::PageLink { page: b }) => {
                                    current.nodes[a].entity == builder.result.targets[b].entity
                                }
                                (a, b) => a == b,
                            }
                    });
                let done = if unchanged {
                    let text = builder.one(entity, "block/text")?.and_then(|r| r["id"].as_str()).map(str::to_owned);
                    (
                        text,
                        matches!(
                            block.kind,
                            BlockKind::Paragraph { .. } | BlockKind::Heading { .. } | BlockKind::Quote { .. } | BlockKind::ListItem { .. }
                        ),
                    )
                } else {
                    fields(builder, entity, &block.kind)?
                };
                written.insert(entity_key.clone(), done.clone());
                done
            };
            let placement = match prior.map(|p| &p.occurrence) {
                Some(Occurrence::Block { placement, .. }) => placement.clone(),
                _ => builder.id().to_string(),
            };
            if replacement {
                let old = builder.facts.iter().find(|r| r["id"] == placement).context("recreate placement absent")?;
                let mut data = old["data"].clone();
                data["value"] = entity_key.clone().into();
                builder.result.mutations.push(NativeMutation::AssertFact(AssertFact {
                    id: Some(placement.parse()?),
                    entity: old["entity"].as_str().context("parent")?.parse()?,
                    attribute: "card/block".into(),
                    data,
                }));
            }
            builder.result.targets.insert(
                block.key.clone(),
                Node {
                    entity: entity_key,
                    occurrence: Occurrence::Block {
                        placement,
                        text_fact: if active { text_fact.clone() } else { None },
                        inactive_text_fact: if active { None } else { text_fact },
                    },
                },
            );
        }
    }
    Ok(())
}

fn same_placement(a: &Node, b: &Node) -> bool {
    matches!((&a.occurrence, &b.occurrence), (Occurrence::Block { placement: a, .. }, Occurrence::Block { placement: b, .. }) if a == b)
}

fn fields(builder: &mut Builder<'_>, entity: Uuid, kind: &BlockKind) -> Result<(Option<String>, bool)> {
    let (native, content) = match kind {
        BlockKind::Paragraph { content } | BlockKind::ListItem { content, .. } => ("text", Some(content)),
        BlockKind::Heading { content, .. } => ("heading", Some(content)),
        BlockKind::Quote { content } => ("blockquote", Some(content)),
        BlockKind::Code { .. } => ("code", None),
        BlockKind::Rule => ("horizontal-rule", None),
        BlockKind::Image { .. } => ("image", None),
        BlockKind::PageLink { .. } => ("card", None),
    };
    builder.set(entity, "block/type", json!({"type":"block-type-union","value":native}))?;
    let mut scalars: BTreeMap<&str, Value> = BTreeMap::new();
    match kind {
        BlockKind::Heading { level, .. } => {
            scalars.insert("block/heading-level", json!({"type":"number","value":level}));
        }
        BlockKind::ListItem { style, .. } => {
            scalars.insert("block/is-list", json!({"type":"boolean","value":true}));
            scalars.insert(
                "block/list-style",
                json!({"type":"list-style-union","value":match style { ListStyle::Bullet => "unordered", ListStyle::Ordered => "ordered" }}),
            );
        }
        BlockKind::Code { language, text } => {
            scalars.insert("block/code", json!({"type":"string","value":text}));
            scalars.insert("block/code-language", json!({"type":"string","value":language}));
        }
        BlockKind::Image {
            src,
            fallback,
            width,
            height,
            alt,
        } => {
            scalars.insert(
                "block/image",
                json!({"type":"image","src":src,"fallback":fallback,"width":width,"height":height}),
            );
            if let Some(alt) = alt {
                scalars.insert("image/alt", json!({"type":"string","value":alt}));
            }
        }
        BlockKind::PageLink { page } => {
            scalars.insert(
                "block/card",
                json!({"type":"reference","value":builder.result.targets.get(page).context("unresolved page")?.entity}),
            );
        }
        _ => {}
    }
    for attribute in [
        "block/heading-level",
        "block/is-list",
        "block/list-style",
        "block/code",
        "block/code-language",
        "block/image",
        "image/alt",
        "block/card",
    ] {
        if let Some(data) = scalars.remove(attribute) {
            builder.set(entity, attribute, data)?;
        } else {
            builder.clear(entity, attribute)?;
        }
    }
    let old = builder.one(entity, "block/text")?;
    let old_id = old.and_then(|r| r["id"].as_str()).map(str::to_owned);
    let text = if let Some(content) = content {
        let payload = crate::write_text(old.map(|r| r["data"]["value"].as_str().context("text payload")).transpose()?, content)?;
        if let Some(payload) = payload {
            Some(builder.set(entity, "block/text", json!({"type":"text","value":payload}))?.to_string())
        } else {
            old_id
        }
    } else {
        old_id
    };
    Ok((text, content.is_some()))
}
