//! Leaflet fact shapes read at 600c7b5f, emitted without live transport.
use anyhow::{Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use crate::{model::*, text};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum Data {
    #[serde(rename = "string")]
    String { value: String },
    #[serde(rename = "number")]
    Number { value: u32 },
    #[serde(rename = "boolean")]
    Boolean { value: bool },
    #[serde(rename = "text")]
    Text { value: String },
    #[serde(rename = "reference")]
    Reference { value: String },
    #[serde(rename = "ordered-reference")]
    OrderedReference { value: String, position: String },
    #[serde(rename = "page-type-union")]
    PageType { value: String },
    #[serde(rename = "block-type-union")]
    BlockType { value: String },
    #[serde(rename = "list-style-union")]
    ListStyle { value: String },
}
impl Data {
    pub fn reference(&self) -> Option<&str> {
        match self {
            Self::Reference { value } | Self::OrderedReference { value, .. } => Some(value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fact {
    pub id: String,
    pub entity: String,
    pub attribute: String,
    pub data: Data,
}

#[derive(Clone, Debug, Serialize)]
pub struct Attribution {
    pub source: Span,
    /// A list is an IR collection, not a fabricated remote group entity.
    pub entity: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BuildContext {
    pub local_id: Uuid,
    pub generation: Uuid,
    pub permission_set: String,
    pub first_text_client: u64,
}
impl BuildContext {
    pub fn fixture(generation: u128) -> Self {
        Self {
            local_id: Uuid::from_u128(1),
            generation: Uuid::from_u128(generation),
            permission_set: Uuid::from_u128(2).to_string(),
            first_text_client: 1001,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Plan {
    pub generation: Uuid,
    pub permission_set: String,
    pub main_page: String,
    pub pages: BTreeMap<String, String>,
    pub entities: BTreeSet<String>,
    pub facts: Vec<Fact>,
    pub author_keys: BTreeMap<String, Attribution>,
    pub expected_text: BTreeMap<String, Vec<Inline>>,
    /// Inline, unpublished definitions; no made-up pollRef URI/CID.
    pub polls: BTreeMap<String, Vec<String>>,
    /// Preparation boundary for a future native publisher, not evidence of release.
    pub requires: Vec<&'static str>,
}

struct Builder {
    context: BuildContext,
    plan: Plan,
    client: u64,
}
fn position(index: usize) -> Result<String> {
    const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    ensure!(index < DIGITS.len(), "experiment supports at most 62 ordered siblings");
    Ok(format!("a{}", DIGITS[index] as char))
}
impl Builder {
    fn id(&self, path: &str) -> String {
        Uuid::new_v5(&self.context.generation, path.as_bytes()).to_string()
    }
    fn create(&mut self, id: &str) {
        self.plan.entities.insert(id.into());
    }
    fn fact(&mut self, entity: &str, attribute: &str, data: Data) {
        let slot = match &data {
            Data::OrderedReference { position, .. } => position.as_str(),
            _ => "",
        };
        let id = Uuid::new_v5(&Uuid::parse_str(entity).expect("generated UUID"), format!("{attribute}/{slot}").as_bytes()).to_string();
        self.plan.facts.push(Fact {
            id,
            entity: entity.into(),
            attribute: attribute.into(),
            data,
        });
    }
    fn attribute(&mut self, block: &Block, entity: Option<&str>) {
        if let Some(key) = &block.key {
            self.plan.author_keys.insert(
                key.name.clone(),
                Attribution {
                    source: key.span,
                    entity: entity.map(str::to_owned),
                },
            );
        }
    }
    fn blocks(&mut self, blocks: &[Block], parent: &str, path: &str) -> Result<()> {
        let mut index = 0;
        for (ordinal, block) in blocks.iter().enumerate() {
            let path = format!("{path}/{ordinal}");
            if let BlockKind::List { ordered, start, items } = &block.kind {
                self.attribute(block, None);
                for (item_index, item) in items.iter().enumerate() {
                    let paragraph = &item[0];
                    let entity = self.block(paragraph, &format!("{path}/item/{item_index}"))?;
                    self.fact(&entity, "block/is-list", Data::Boolean { value: true });
                    self.fact(
                        &entity,
                        "block/list-style",
                        Data::ListStyle {
                            value: if *ordered { "ordered" } else { "unordered" }.into(),
                        },
                    );
                    if *ordered {
                        let number = start.checked_add(item_index as u32).ok_or_else(|| anyhow::anyhow!("list number overflow"))?;
                        self.fact(&entity, "block/list-number", Data::Number { value: number });
                    }
                    self.fact(
                        parent,
                        "card/block",
                        Data::OrderedReference {
                            value: entity,
                            position: position(index)?,
                        },
                    );
                    index += 1;
                }
            } else {
                let entity = self.block(block, &path)?;
                self.fact(
                    parent,
                    "card/block",
                    Data::OrderedReference {
                        value: entity,
                        position: position(index)?,
                    },
                );
                index += 1;
            }
        }
        Ok(())
    }
    fn richtext(&mut self, entity: &str, content: &[Inline]) -> Result<()> {
        let update = text::fresh(content, self.client)?;
        self.client += 1;
        self.fact(entity, "block/text", Data::Text { value: update });
        self.plan.expected_text.insert(entity.into(), content.into());
        Ok(())
    }
    fn block(&mut self, block: &Block, path: &str) -> Result<String> {
        let entity = if matches!(block.kind, BlockKind::Poll { .. }) {
            let key = &block.key.as_ref().expect("validated Poll key").name;
            Uuid::new_v5(&self.context.local_id, format!("poll/{key}").as_bytes()).to_string()
        } else {
            self.id(path)
        };
        self.create(&entity);
        self.attribute(block, Some(&entity));
        let kind = match &block.kind {
            BlockKind::Paragraph { content } => {
                self.richtext(&entity, content)?;
                "text"
            }
            BlockKind::Heading { level, content } => {
                self.richtext(&entity, content)?;
                self.fact(&entity, "block/heading-level", Data::Number { value: (*level).into() });
                "heading"
            }
            BlockKind::Quote { paragraph } => {
                let BlockKind::Paragraph { content } = &paragraph.kind else {
                    unreachable!("validated quote")
                };
                self.attribute(paragraph, Some(&entity));
                self.richtext(&entity, content)?;
                "blockquote"
            }
            BlockKind::Code { language, text } => {
                self.fact(&entity, "block/code", Data::String { value: text.clone() });
                if !language.is_empty() {
                    self.fact(&entity, "block/code-language", Data::String { value: language.clone() });
                }
                "code"
            }
            BlockKind::Rule => "horizontal-rule",
            BlockKind::PageLink { page } => {
                self.fact(
                    &entity,
                    "block/card",
                    Data::Reference {
                        value: self.plan.pages[page].clone(),
                    },
                );
                "card"
            }
            BlockKind::Poll { options } => {
                self.plan.polls.insert(entity.clone(), options.clone());
                for (index, option) in options.iter().enumerate() {
                    let option_id = Uuid::new_v5(&Uuid::parse_str(&entity)?, format!("option/{index}").as_bytes()).to_string();
                    self.create(&option_id);
                    self.fact(&option_id, "poll-option/name", Data::String { value: option.clone() });
                    self.fact(
                        &entity,
                        "poll/options",
                        Data::OrderedReference {
                            value: option_id,
                            position: position(index)?,
                        },
                    );
                }
                "poll"
            }
            BlockKind::List { .. } => unreachable!("lists are linear collections, not remote group blocks"),
        };
        self.fact(&entity, "block/type", Data::BlockType { value: kind.into() });
        Ok(entity)
    }
}

/// Consume only a successfully typed document; all unsupported input fails earlier.
pub fn build(doc: &Document<'_>, context: BuildContext) -> Result<Plan> {
    let main_page = Uuid::new_v5(&context.generation, b"page/main").to_string();
    let mut pages = BTreeMap::from([("main".into(), main_page.clone())]);
    for page in &doc.pages {
        pages.insert(
            page.name.clone(),
            Uuid::new_v5(&context.generation, format!("page/{}", page.name).as_bytes()).to_string(),
        );
    }
    let mut b = Builder {
        client: context.first_text_client,
        plan: Plan {
            generation: context.generation,
            permission_set: context.permission_set.clone(),
            main_page: main_page.clone(),
            pages,
            entities: BTreeSet::new(),
            facts: vec![],
            author_keys: BTreeMap::new(),
            expected_text: BTreeMap::new(),
            polls: BTreeMap::new(),
            requires: vec![
                "authenticated entity-set staging/read-back/entry switch",
                "source metadata application",
                "native publication and poll upload/read-back",
            ],
        },
        context,
    };
    for id in b.plan.pages.values().cloned().collect::<Vec<_>>() {
        b.create(&id);
        b.fact(&id, "page/type", Data::PageType { value: "doc".into() });
    }
    b.blocks(&doc.main, &main_page, "main")?;
    for page in &doc.pages {
        let id = b.plan.pages[&page.name].clone();
        b.blocks(&page.blocks, &id, &format!("page/{}", page.name))?;
    }
    Ok(b.plan)
}

impl Plan {
    /// Low-level native mutation argument shapes, not a signed push envelope.
    pub fn staged_mutations(&self) -> Vec<Value> {
        self.entities
            .iter()
            .map(|id| json!({"name":"createEntity","args":{"entityID":id,"permission_set":self.permission_set}}))
            .chain(self.facts.iter().map(|fact| json!({"name":"assertFact","args":fact})))
            .collect()
    }
}
