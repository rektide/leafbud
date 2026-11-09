use super::*;
use leafbud_protocol::Position;

pub(super) fn prepare(builder: &mut Builder<'_>, current: &Content, _mint: bool) -> Result<()> {
    let desired_placements: BTreeSet<_> = builder
        .result
        .targets
        .values()
        .filter_map(|n| match &n.occurrence {
            Occurrence::Block { placement, .. } => Some(placement.clone()),
            _ => None,
        })
        .collect();
    // Omission detaches only the selected placements. Neither entities nor their
    // text/resources are deleted, including blocks inside a retired page.
    for (key, node) in &builder.desired.state.nodes {
        if let Occurrence::Block { placement, .. } = &node.occurrence
            && !desired_placements.contains(placement)
            && let Some(row) = builder.facts.iter().find(|r| r["id"] == *placement)
        {
            ensure!(
                row["attribute"] == "card/block" && row["data"]["value"] == node.entity,
                "retired placement changed: {key}"
            );
        }
    }
    // Workflow authorizes this complete-current comparison only under the
    // agreement witness or an explicit take-local decision. It includes a
    // verified first-bind seed and deliberately rejected remote-only additions.
    for node in current.nodes.values() {
        if let Occurrence::Block { placement, .. } = &node.occurrence
            && !desired_placements.contains(placement)
        {
            builder.result.mutations.push(NativeMutation::RetractFact { fact_id: placement.parse()? });
        }
    }
    for page in &builder.desired.pages {
        let parent: Uuid = builder.result.targets[&page.key].entity.parse()?;
        let mut wanted = Vec::new();
        let mut entities = BTreeSet::new();
        let mut changed = false;
        let mut moves = Vec::new();
        for block in &page.blocks {
            let node = &builder.result.targets[&block.key];
            let entity: Uuid = node.entity.parse()?;
            ensure!(
                entities.insert(entity),
                "same block entity repeated in one parent; exact occurrence ordering is not implemented"
            );
            let Occurrence::Block { placement, .. } = &node.occurrence else {
                unreachable!()
            };
            let placement: Uuid = placement.parse()?;
            wanted.push(placement.to_string());
            let old = builder.facts.iter().find(|r| r["id"] == placement.to_string());
            let old_parent = if let Some(old) = old {
                old["entity"].as_str().context("current parent")?.parse()?
            } else {
                builder.result.mutations.push(NativeMutation::AddLastBlock {
                    parent,
                    fact_id: placement,
                    entity,
                });
                changed = true;
                parent
            };
            changed |= old_parent != parent;
            moves.push(NativeMutation::MoveBlock {
                old_parent,
                block: entity,
                new_parent: parent,
                position: Position::End,
            });
        }
        let mut old: Vec<_> = builder
            .facts
            .iter()
            .filter(|r| r["entity"] == parent.to_string() && r["attribute"] == "card/block")
            .collect();
        old.sort_by(|a, b| (a["data"]["position"].as_str(), a["id"].as_str()).cmp(&(b["data"]["position"].as_str(), b["id"].as_str())));
        let old: Vec<_> = old.iter().filter_map(|r| r["id"].as_str().map(str::to_owned)).collect();
        if changed || old != wanted {
            builder.result.mutations.extend(moves);
        }
    }
    Ok(())
}
