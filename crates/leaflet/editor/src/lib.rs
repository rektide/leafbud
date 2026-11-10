//! Pure native fact observation → bounded, loss-checked current content.
//! Raw observations stay with the caller; no transport or local file effects.
use anyhow::{Context, Result, bail, ensure};
use leafbud_document::{Block, BlockKind, Inline, ListStyle, Node, Occurrence, Page, Theme};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

mod text;
pub use text::read as read_text;
pub use text::read_segments as read_text_segments;
pub use text::write::normalize as normalize_text;
pub use text::write::write as write_text;

pub mod write;

/// The shared wire view retains raw CRDT/fact evidence alongside projection.
pub fn observe_view(view: &leafbud_protocol::FullView, root: &str, namespace: Uuid) -> Result<Content> {
    observe(&view.facts, root, namespace)
}

/// Conservative current editor evidence, not a representable-content digest.
/// Unknown fact fields/payloads and extras remain witnessed even when observe
/// refuses them. Transport timestamps, row versions and indexes are excluded.
pub fn witness(facts: &[Value], extras: &BTreeMap<String, Value>, metadata: Value) -> Value {
    let mut facts: Vec<_> = facts
        .iter()
        .map(|row| {
            let mut row = row.clone();
            if let Some(row) = row.as_object_mut() {
                for key in ["created_at", "updated_at", "indexes", "version"] {
                    row.remove(key);
                }
            }
            row
        })
        .collect();
    facts.sort_by_key(Value::to_string);
    serde_json::json!({"profile":leafbud_document::EDITOR_PROFILE,"facts":facts,"extras":extras,"metadata":metadata})
}

pub struct Content {
    pub pages: Vec<Page>,
    pub nodes: BTreeMap<String, Node>,
    pub entry_fact: String,
    pub theme: Option<Theme>,
    pub derived_title: Option<String>,
}

/// Decode exactly one explicitly read, known allocated detached block. This
/// does not infer entity existence from absence or repair incomplete graphs.
/// Detached page graphs/card links intentionally require a separate decision.
pub fn observe_detached_block(facts: &[Value], entity: &str, placement: &str, namespace: Uuid) -> Result<Node> {
    validate_rows(facts)?;
    ensure!(facts.iter().all(|r| r["entity"] == entity), "detached block view contains unrelated entities");
    ensure!(
        !facts.iter().any(|r| r["attribute"] == "block/card"),
        "detached page-link recovery is unsupported"
    );
    let mut scan = Scan {
        rows: facts,
        used: BTreeSet::new(),
        namespace,
        nodes: BTreeMap::new(),
        pages: BTreeMap::new(),
        visiting: BTreeSet::new(),
    };
    let block = scan.block(entity, placement)?;
    ensure!(
        facts.iter().all(|r| r["id"].as_str().is_some_and(|id| scan.used.contains(id))),
        "unsupported detached block facts"
    );
    Ok(scan.nodes.remove(&block.key).expect("observed block"))
}

struct Scan<'a> {
    rows: &'a [Value],
    used: BTreeSet<String>,
    namespace: Uuid,
    nodes: BTreeMap<String, Node>,
    pages: BTreeMap<String, Page>,
    visiting: BTreeSet<String>,
}
impl Scan<'_> {
    fn key(&self, role: &str, id: &str) -> String {
        format!("{role}-{}", Uuid::new_v5(&self.namespace, id.as_bytes()))
    }
    fn take(&mut self, entity: &str, attribute: &str) -> Result<Vec<Value>> {
        let rows: Vec<_> = self
            .rows
            .iter()
            .filter(|f| f["entity"] == entity && f["attribute"] == attribute)
            .cloned()
            .collect();
        for row in &rows {
            self.used.insert(row["id"].as_str().context("fact id")?.into());
        }
        Ok(rows)
    }
    fn one(&mut self, entity: &str, attribute: &str, kind: &str) -> Result<Option<Value>> {
        let rows = self.take(entity, attribute)?;
        ensure!(rows.len() <= 1, "ambiguous cardinality-one attribute {attribute}");
        let Some(row) = rows.first() else {
            return Ok(None);
        };
        ensure!(row["data"]["type"] == kind, "wrong native data type for {attribute}");
        ensure!(
            row["data"]
                .as_object()
                .is_some_and(|v| v.keys().all(|k| matches!(k.as_str(), "type" | "value"))),
            "unknown native fields for {attribute}"
        );
        Ok(Some(row.clone()))
    }
    fn string(&mut self, entity: &str, attribute: &str, kind: &str) -> Result<Option<String>> {
        self.one(entity, attribute, kind)?
            .map(|r| {
                r["data"]["value"]
                    .as_str()
                    .map(str::to_owned)
                    .with_context(|| format!("non-string {attribute}"))
            })
            .transpose()
    }
    fn ordered(&mut self, entity: &str, attribute: &str) -> Result<Vec<Value>> {
        let mut rows = self.take(entity, attribute)?;
        for row in &rows {
            let data = &row["data"];
            ensure!(
                data["type"] == "ordered-reference" && data["value"].is_string() && data["position"].as_str().is_some_and(|s| !s.is_empty()),
                "malformed {attribute} placement"
            );
            ensure!(data.as_object().is_some_and(|v| v.len() == 3), "unknown placement fields");
        }
        rows.sort_by(|a, b| (a["data"]["position"].as_str(), a["id"].as_str()).cmp(&(b["data"]["position"].as_str(), b["id"].as_str())));
        Ok(rows)
    }
    fn page(&mut self, entity: &str, membership: &str) -> Result<String> {
        let key = self.key("p", entity);
        if self.pages.contains_key(entity) {
            match self.nodes.get_mut(&key).context("page binding")? {
                Node {
                    occurrence: Occurrence::Page { references },
                    ..
                } => references.push(membership.into()),
                Node { .. } => bail!("page entity carries a non-page occurrence"),
            }
            return Ok(key);
        }
        ensure!(
            self.visiting.insert(entity.into()),
            "cyclic page references are not supported by import profile 1"
        );
        ensure!(self.visiting.len() <= 64, "page nesting exceeds import limit");
        let page_type = self.string(entity, "page/type", "page-type-union")?.unwrap_or_else(|| "doc".into());
        ensure!(page_type == "doc", "unsupported page type: {page_type}");
        self.nodes.insert(
            key.clone(),
            Node {
                entity: entity.into(),
                occurrence: Occurrence::Page {
                    references: vec![membership.into()],
                },
            },
        );
        let mut blocks = vec![];
        for row in self.ordered(entity, "card/block")? {
            blocks.push(self.block(
                row["data"]["value"].as_str().expect("validated reference"),
                row["id"].as_str().expect("validated id"),
            )?);
        }
        self.pages.insert(entity.into(), Page { key: key.clone(), blocks });
        self.visiting.remove(entity);
        Ok(key)
    }
    fn block(&mut self, entity: &str, membership: &str) -> Result<Block> {
        let key = self.key("b", membership);
        let kind = self.string(entity, "block/type", "block-type-union")?.context("block has no block/type")?;
        // Blocks never own block children in this profile; a block entity with
        // card/block facts is nested list/container content.
        ensure!(
            self.rows.iter().all(|f| !(f["entity"] == entity && f["attribute"] == "card/block")),
            "nested list/container content is not supported by this profile"
        );
        let is_list = self.one(entity, "block/is-list", "boolean")?.map(|f| f["data"]["value"] == Value::Bool(true));
        if self.one(entity, "block/check-list", "boolean")?.is_some() {
            bail!("task lists are not supported by this profile");
        }
        if self.one(entity, "block/list-number", "number")?.is_some() {
            bail!("ordered-list numbering overrides are not supported by this profile");
        }
        let list_style = match self.string(entity, "block/list-style", "list-style-union")? {
            None => ListStyle::Bullet,
            Some(style) => match style.as_str() {
                "unordered" => ListStyle::Bullet,
                "ordered" => ListStyle::Ordered,
                other => bail!("unsupported list style: {other}"),
            },
        };
        let text_fact = self.one(entity, "block/text", "text")?;
        let owns_text = matches!(kind.as_str(), "text" | "heading" | "blockquote");
        let content = match text_fact.as_ref().filter(|_| owns_text) {
            Some(f) => read_text(f["data"]["value"].as_str().context("non-string block/text")?).context("block/text cannot be represented losslessly")?,
            None => vec![],
        };
        let kind = match kind.as_str() {
            "text" => match is_list {
                Some(true) => BlockKind::ListItem { style: list_style, content },
                _ => BlockKind::Paragraph { content },
            },
            "blockquote" => BlockKind::Quote { content },
            "heading" => {
                ensure!(is_list != Some(true), "heading list items are not supported by this profile");
                let level = self.one(entity, "block/heading-level", "number")?.context("heading level absent")?["data"]["value"]
                    .as_u64()
                    .context("invalid heading level")?;
                ensure!((1..=6).contains(&level), "heading level outside 1..6");
                BlockKind::Heading { level: level as u8, content }
            }
            "code" => BlockKind::Code {
                text: self.string(entity, "block/code", "string")?.unwrap_or_default(),
                language: self.string(entity, "block/code-language", "string")?.unwrap_or_default(),
            },
            "horizontal-rule" => BlockKind::Rule,
            "image" => {
                let rows = self.take(entity, "block/image")?;
                ensure!(rows.len() == 1, "image payload missing or ambiguous");
                let data = &rows[0]["data"];
                ensure!(
                    data["type"] == "image"
                        && data
                            .as_object()
                            .is_some_and(|o| o.keys().all(|k| matches!(k.as_str(), "type" | "src" | "fallback" | "width" | "height"))),
                    "unsupported image fields/local image resource"
                );
                let string = |key: &str| -> Result<Option<String>> {
                    match data.get(key) {
                        None | Some(Value::Null) => Ok(None),
                        Some(Value::String(v)) => Ok(Some(v.clone())),
                        _ => bail!("invalid image {key}"),
                    }
                };
                let number = |key: &str| -> Result<Option<u64>> {
                    match data.get(key) {
                        None | Some(Value::Null) => Ok(None),
                        Some(v) => Ok(Some(v.as_u64().with_context(|| format!("non-integer image {key}"))?)),
                    }
                };
                BlockKind::Image {
                    src: string("src")?.context("missing image src")?,
                    fallback: string("fallback")?,
                    width: number("width")?,
                    height: number("height")?,
                    alt: self.string(entity, "image/alt", "string")?,
                }
            }
            "card" => {
                let link = self.one(entity, "block/card", "reference")?.context("card target absent")?;
                let target = link["data"]["value"].as_str().context("invalid card target")?;
                BlockKind::PageLink {
                    page: self.page(target, link["id"].as_str().context("card reference id")?)?,
                }
            }
            other => bail!("unsupported block type: {other}"),
        };
        let text_id = text_fact.and_then(|f| f["id"].as_str().map(str::to_owned));
        // Native ImageBlock/CodeBlock render their selected payload, not latent
        // block/text left behind by type conversion. Keep its locator separately
        // and include its opaque payload in the caller's conservative witness.
        self.nodes.insert(
            key.clone(),
            Node {
                entity: entity.into(),
                occurrence: Occurrence::Block {
                    placement: membership.into(),
                    text_fact: if owns_text { text_id.clone() } else { None },
                    inactive_text_fact: if owns_text { None } else { text_id },
                },
            },
        );
        Ok(Block { key, kind })
    }
}

pub fn observe(facts: &[Value], root: &str, namespace: Uuid) -> Result<Content> {
    let mut scan = Scan {
        rows: facts,
        used: BTreeSet::new(),
        namespace,
        nodes: BTreeMap::new(),
        pages: BTreeMap::new(),
        visiting: BTreeSet::new(),
    };
    validate_rows(facts)?;
    let entries = scan.ordered(root, "root/page")?;
    ensure!(entries.len() == 1, "import requires exactly one root entry page (no truncation)");
    let entry = &entries[0];
    let entity = entry["data"]["value"].as_str().expect("validated reference");
    let entry_fact = entry["id"].as_str().expect("validated fact").to_owned();
    scan.page(entity, &entry_fact)?;
    let theme = scan
        .one(root, "theme/page-leaflet-watermark", "boolean")?
        .map(|f| {
            f["data"]["value"]
                .as_bool()
                .map(|leaflet_watermark| Theme { leaflet_watermark })
                .context("invalid watermark flag")
        })
        .transpose()?;
    let unrepresented: BTreeSet<_> = facts
        .iter()
        .filter(|f| !scan.used.contains(f["id"].as_str().expect("validated fact")))
        .filter_map(|f| f["attribute"].as_str())
        .collect();
    ensure!(
        unrepresented.is_empty(),
        "unrepresented editor attributes (including detached content): {}",
        unrepresented.into_iter().collect::<Vec<_>>().join(", ")
    );
    let main = scan.pages.remove(entity).context("entry page missing")?;
    let derived_title = main
        .blocks
        .iter()
        .find_map(|b| match &b.kind {
            BlockKind::Heading { content, .. } | BlockKind::Paragraph { content } => Some(
                content
                    .iter()
                    .map(|i| match i {
                        Inline::Run { text, .. } => text.as_str(),
                        Inline::HardBreak => "\n",
                    })
                    .collect::<String>(),
            ),
            _ => None,
        })
        .filter(|s| !s.trim().is_empty());
    let mut pages = vec![main];
    pages.extend(scan.pages.into_values());
    for node in scan.nodes.values_mut() {
        if let Occurrence::Page { references } = &mut node.occurrence {
            references.sort();
            references.dedup();
        }
    }
    Ok(Content {
        pages,
        nodes: scan.nodes,
        entry_fact,
        theme,
        derived_title,
    })
}

fn validate_rows(facts: &[Value]) -> Result<()> {
    for row in facts {
        ensure!(
            row.as_object().is_some_and(|row| row.keys().all(|k| matches!(
                k.as_str(),
                "id" | "entity" | "attribute" | "data" | "author_did" | "created_at" | "updated_at" | "indexes" | "version"
            ))),
            "unknown native fact fields"
        );
        ensure!(
            row["id"].is_string() && row["entity"].is_string() && row["attribute"].is_string(),
            "invalid fact shape"
        );
    }
    Ok(())
}
