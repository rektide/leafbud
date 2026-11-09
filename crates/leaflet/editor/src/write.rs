//! Complete desired-document compilation into the native mutation vocabulary.
//! Pure: no HTTP, source writes, saved bodies or semantic operation AST.
use crate::*;
use leafbud_document::{Document, Effect};
use leafbud_protocol::{AssertFact, CreateEntity, NativeMutation};
use serde_json::json;

mod blocks;
mod metadata;
mod order;
mod pages;

pub struct Compiled {
    pub mutations: Vec<NativeMutation>,
    pub targets: BTreeMap<String, Node>,
    pub created_entities: Vec<Uuid>,
    pub allocated_facts: Vec<Uuid>,
}

/// Source-only preflight, also used BEFORE first mint. Native observations add
/// identity/causal/permission checks, never narrow this supported source profile.
pub fn preflight(desired: &Document) -> Result<()> {
    ensure!(!desired.pages.is_empty(), "document requires a main page");
    let mut keys = BTreeSet::new();
    for page in &desired.pages {
        ensure!(!page.key.is_empty() && keys.insert(&page.key), "missing/duplicate page key");
        for block in &page.blocks {
            ensure!(!block.key.is_empty() && keys.insert(&block.key), "missing/duplicate block key");
            match &block.kind {
                BlockKind::Paragraph { content } | BlockKind::Quote { content } | BlockKind::ListItem { content, .. } => {
                    text::write::normalize(content)?;
                }
                BlockKind::Heading { level, content } => {
                    ensure!((1..=6).contains(level), "heading level outside 1..6");
                    text::write::normalize(content)?;
                }
                BlockKind::Image { src, fallback, .. } => {
                    ensure!(
                        url(src) && fallback.as_deref().is_none_or(url),
                        "images require remote HTTP(S) URLs; uploads are unsupported"
                    );
                }
                _ => {}
            }
        }
    }
    pages::validate(desired)
}

fn url(value: &str) -> bool {
    (value.starts_with("https://") || value.starts_with("http://")) && !value.split("://").nth(1).unwrap_or_default().is_empty()
}

struct Builder<'a> {
    desired: &'a Document,
    facts: &'a [Value],
    set: Uuid,
    result: Compiled,
}
impl Builder<'_> {
    fn create(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.result.created_entities.push(id);
        self.result.mutations.push(NativeMutation::CreateEntity(vec![CreateEntity {
            entity_id: id,
            permission_set: self.set,
        }]));
        id
    }
    fn id(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.result.allocated_facts.push(id);
        id
    }
    fn one(&self, entity: Uuid, attribute: &str) -> Result<Option<&Value>> {
        let rows: Vec<_> = self
            .facts
            .iter()
            .filter(|r| r["entity"] == entity.to_string() && r["attribute"] == attribute)
            .collect();
        ensure!(rows.len() <= 1, "ambiguous {attribute}");
        Ok(rows.first().copied())
    }
    fn set(&mut self, entity: Uuid, attribute: &str, data: Value) -> Result<Uuid> {
        let old = self.one(entity, attribute)?;
        if let Some(old) = old
            && old["data"] == data
        {
            return Ok(old["id"].as_str().context("fact ID")?.parse()?);
        }
        let id = if let Some(old) = old {
            old["id"].as_str().context("fact ID")?.parse()?
        } else {
            self.id()
        };
        self.result.mutations.push(NativeMutation::AssertFact(AssertFact {
            id: Some(id),
            entity,
            attribute: attribute.into(),
            data,
        }));
        Ok(id)
    }
    fn clear(&mut self, entity: Uuid, attribute: &str) -> Result<()> {
        if let Some(row) = self.one(entity, attribute)? {
            self.result.mutations.push(NativeMutation::RetractFact {
                fact_id: row["id"].as_str().context("fact ID")?.parse()?,
            });
        }
        Ok(())
    }
}

/// Current raw facts remain beside typed meaning; they are not persisted as a
/// second body. Recreation forks only the selected occurrence's entity.
pub fn compile(desired: &Document, observed: &Document, facts: &[Value], recreate: &BTreeSet<String>) -> Result<Compiled> {
    preflight(desired)?;
    let binding = desired.state.binding.as_ref().context("compiler requires a verified binding")?;
    let current = observe(facts, &binding.root, desired.state.id)?;
    let mint = desired.state.pending.as_ref().is_some_and(|a| matches!(a.effect, Effect::Mint { .. }));
    for key in recreate {
        ensure!(
            desired.pages.iter().flat_map(|p| &p.blocks).any(|b| &b.key == key) && desired.state.nodes.contains_key(key),
            "--recreate requires an existing supported block key: {key}"
        );
    }
    let mut builder = Builder {
        desired,
        facts,
        set: binding.entity_set.parse()?,
        result: Compiled {
            mutations: vec![],
            targets: BTreeMap::new(),
            created_entities: vec![],
            allocated_facts: vec![],
        },
    };
    pages::prepare(&mut builder, &current, mint)?;
    blocks::prepare(&mut builder, &current, recreate)?;
    order::prepare(&mut builder, &current, mint)?;
    pages::references(&mut builder)?;
    metadata::prepare(&mut builder, observed)?;
    Ok(builder.result)
}
