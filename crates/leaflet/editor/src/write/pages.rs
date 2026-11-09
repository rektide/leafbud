use super::*;

pub(super) fn validate(desired: &Document) -> Result<()> {
    fn visit<'a>(key: &'a str, desired: &'a Document, seen: &mut BTreeSet<&'a str>, active: &mut BTreeSet<&'a str>) -> Result<()> {
        ensure!(!active.contains(key), "cyclic page references are unsupported");
        if !seen.insert(key) {
            return Ok(());
        }
        active.insert(key);
        let page = desired.pages.iter().find(|p| p.key == key).context("unresolved page link")?;
        ensure!(active.len() <= 64, "page nesting exceeds limit");
        for block in &page.blocks {
            if let BlockKind::PageLink { page } = &block.kind {
                visit(page, desired, seen, active)?;
            }
        }
        active.remove(key);
        Ok(())
    }
    let mut seen = BTreeSet::new();
    visit(&desired.pages[0].key, desired, &mut seen, &mut BTreeSet::new())?;
    ensure!(
        seen.len() == desired.pages.len(),
        "named page is unreferenced; add a PageLink or remove its definition"
    );
    Ok(())
}

pub(super) fn prepare(builder: &mut Builder<'_>, current: &Content, mint: bool) -> Result<()> {
    for (index, page) in builder.desired.pages.iter().enumerate() {
        let node = if mint && index == 0 {
            current.nodes[&current.pages[0].key].clone()
        } else if let Some(node) = builder.desired.state.nodes.get(&page.key) {
            ensure!(matches!(node.occurrence, Occurrence::Page { .. }), "page key addresses a block");
            ensure!(
                current
                    .nodes
                    .values()
                    .any(|n| n.entity == node.entity && matches!(n.occurrence, Occurrence::Page { .. })),
                "confirmed page no longer reachable"
            );
            node.clone()
        } else {
            let entity = builder.create();
            builder.set(entity, "page/type", json!({"type":"page-type-union","value":"doc"}))?;
            Node {
                entity: entity.to_string(),
                occurrence: Occurrence::Page { references: vec![] },
            }
        };
        if index == 0 {
            ensure!(
                node.entity == current.nodes[&current.pages[0].key].entity,
                "main page identity cannot be replaced by ordinary save"
            );
        }
        builder.result.targets.insert(page.key.clone(), node);
    }
    Ok(())
}

pub(super) fn references(builder: &mut Builder<'_>) -> Result<()> {
    let mut references: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let main = &builder.desired.pages[0].key;
    references
        .entry(main.clone())
        .or_default()
        .insert(builder.desired.state.binding.as_ref().expect("bound").entry_fact.clone());
    for page in &builder.desired.pages {
        for block in &page.blocks {
            if let BlockKind::PageLink { page } = &block.kind {
                let entity: Uuid = builder.result.targets[&block.key].entity.parse()?;
                let planned = builder.result.mutations.iter().find_map(|m| match m {
                    NativeMutation::AssertFact(row) if row.entity == entity && row.attribute == "block/card" => row.id,
                    _ => None,
                });
                let id = planned
                    .map(|id| id.to_string())
                    .or_else(|| {
                        builder
                            .one(entity, "block/card")
                            .ok()
                            .flatten()
                            .and_then(|r| r["id"].as_str().map(str::to_owned))
                    })
                    .context("missing planned card reference")?;
                references.entry(page.clone()).or_default().insert(id);
            }
        }
    }
    for (key, node) in &mut builder.result.targets {
        if let Occurrence::Page { references: refs } = &mut node.occurrence {
            *refs = references.remove(key).unwrap_or_default().into_iter().collect();
        }
    }
    Ok(())
}
