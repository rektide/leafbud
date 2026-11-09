//! Satteri-based format-2 reader: implicit main page, named page definitions,
//! flat lists, Key markers, and new blocks without keys. Source validity is
//! distinct from preparation: missing keys and lagging node rows are ordinary
//! authorship, not parse errors.
use super::frontmatter;
use anyhow::{Context, Result, bail, ensure};
use leafbud_document::*;
use satteri_arena::{Arena, Mdast, decode_string_ref_data};
use satteri_ast::mdast::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Provisional main-page key while `@leafbud.mainPage` is not yet persisted.
/// Deterministic (never random) so previews cannot present churn as identity.
pub const PROVISIONAL_MAIN_PAGE: &str = "p-auto";
pub(crate) use PROVISIONAL_MAIN_PAGE as PROVISIONAL_PAGE;

struct Reader {
    arena: Arena<Mdast>,
    keys: BTreeSet<String>,
    /// Byte spans of each block in document order (parser-derived; relative
    /// to the BOM-stripped source the arena was built from).
    spans: Vec<(u32, u32)>,
    /// Whether each block carried an explicit Key marker.
    keyed: Vec<bool>,
    /// Next provisional suffix (`b-auto-N`), skipping supplied spellings.
    auto: usize,
}

impl Reader {
    fn kind(&self, id: u32) -> MdastNodeType {
        MdastNodeType::from_u8(self.arena.get_node(id).node_type).expect("native node kind")
    }
    fn name(&self, id: u32) -> &str {
        self.arena.get_str(decode_mdx_jsx_element_name(self.arena.get_type_data(id)))
    }
    fn value(&self, id: u32) -> &str {
        self.arena.get_str(decode_string_ref_data(self.arena.get_type_data(id)))
    }
    fn props(&self, id: u32) -> Result<BTreeMap<String, Value>> {
        let data = self.arena.get_type_data(id);
        let mut props = BTreeMap::new();
        for i in 0..decode_mdx_jsx_attr_count(data) {
            let (kind, name, value) = decode_mdx_jsx_attr(data, i);
            let name = self.arena.get_str(name).to_owned();
            let value = match kind {
                MDX_ATTR_LITERAL_PROP => Value::String(self.arena.get_str(value).into()),
                MDX_ATTR_EXPRESSION_PROP => {
                    serde_json::from_str(self.arena.get_str(value)).map_err(|_| anyhow::anyhow!("component expression is not literal JSON"))?
                }
                _ => bail!("spread/bare JSX attribute is unsupported"),
            };
            ensure!(props.insert(name, value).is_none(), "duplicate JSX property");
        }
        Ok(props)
    }
    fn prop<T: serde::de::DeserializeOwned>(&self, props: &mut BTreeMap<String, Value>, name: &str) -> Result<T> {
        serde_json::from_value(props.remove(name).with_context(|| format!("missing {name} property"))?).map_err(|_| anyhow::anyhow!("invalid {name} property"))
    }
    fn valid_key(key: &str) -> bool {
        let mut bytes = key.bytes();
        bytes.next().is_some_and(|b| b.is_ascii_alphabetic()) && bytes.all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    }
    fn register(&mut self, key: String) -> Result<String> {
        ensure!(Self::valid_key(&key), "invalid source key spelling");
        ensure!(self.keys.insert(key.clone()), "duplicate source key");
        Ok(key)
    }
    /// One deterministic provisional key for a new block. Never persisted:
    /// preparation replaces it with a one-time assigned key.
    fn provisional(&mut self) -> String {
        loop {
            self.auto += 1;
            let key = format!("b-auto-{}", self.auto);
            if !self.keys.contains(&key) {
                self.keys.insert(key.clone());
                return key;
            }
        }
    }
    fn inline(&self, parent: u32, marks: &[Mark], out: &mut Vec<Inline>) -> Result<()> {
        use MdastNodeType as M;
        for &id in self.arena.get_children(parent) {
            match self.kind(id) {
                M::Text => push_run(out, self.value(id), marks),
                M::InlineCode => {
                    let mut next = marks.to_vec();
                    next.push(Mark::Code);
                    push_run(out, self.value(id), &next);
                }
                M::Strong | M::Emphasis | M::Delete | M::Link => {
                    let mut next = marks.to_vec();
                    next.push(match self.kind(id) {
                        M::Strong => Mark::Strong,
                        M::Emphasis => Mark::Em,
                        M::Delete => Mark::Strikethrough,
                        _ => {
                            let data = LinkData::from_bytes(self.arena.get_type_data(id));
                            ensure!(self.arena.get_str(data.title).is_empty(), "link title is not supported by import profile");
                            Mark::Link {
                                href: self.arena.get_str(data.url).into(),
                            }
                        }
                    });
                    self.inline(id, &next, out)?;
                }
                M::Break => out.push(Inline::HardBreak),
                M::MdxJsxTextElement | M::MdxJsxFlowElement => {
                    let mut props = self.props(id)?;
                    ensure!(self.arena.get_children(id).is_empty(), "inline component children unsupported");
                    match self.name(id) {
                        "Run" => {
                            let text: String = self.prop(&mut props, "text")?;
                            let mut nested: Vec<Mark> = if props.contains_key("marks") {
                                self.prop(&mut props, "marks")?
                            } else {
                                vec![]
                            };
                            nested.extend_from_slice(marks);
                            push_run(out, &text, &nested);
                        }
                        "HardBreak" => out.push(Inline::HardBreak),
                        _ => bail!("unsupported inline component"),
                    }
                    ensure!(props.is_empty(), "unknown inline properties");
                }
                _ => bail!("unsupported inline syntax: {}", self.kind(id).name()),
            }
        }
        Ok(())
    }
    fn rich(&self, id: u32) -> Result<Vec<Inline>> {
        let mut out = vec![];
        self.inline(id, &[], &mut out)?;
        Ok(out)
    }
    fn literal_rich(&self, parent: u32, out: &mut Vec<Inline>) -> Result<()> {
        for &id in self.arena.get_children(parent) {
            if self.kind(id) == MdastNodeType::Paragraph {
                self.literal_rich(id, out)?;
            } else {
                ensure!(
                    matches!(self.kind(id), MdastNodeType::MdxJsxFlowElement | MdastNodeType::MdxJsxTextElement),
                    "literal block requires Run/HardBreak children"
                );
                let mut props = self.props(id)?;
                ensure!(self.arena.get_children(id).is_empty(), "unexpected inline children");
                match self.name(id) {
                    "Run" => {
                        let text: String = self.prop(&mut props, "text")?;
                        let marks: Vec<Mark> = self.prop(&mut props, "marks")?;
                        push_run(out, &text, &marks);
                    }
                    "HardBreak" => out.push(Inline::HardBreak),
                    _ => bail!("unknown literal inline component"),
                }
                ensure!(props.is_empty(), "unknown literal inline properties");
            }
        }
        Ok(())
    }
    fn block(&mut self, id: u32, key: String) -> Result<Block> {
        use MdastNodeType as M;
        let kind = match self.kind(id) {
            M::Paragraph => BlockKind::Paragraph { content: self.rich(id)? },
            M::Heading => BlockKind::Heading {
                level: self.arena.get_type_data(id)[0],
                content: self.rich(id)?,
            },
            M::Blockquote => {
                let children = self.arena.get_children(id);
                ensure!(children.len() == 1, "only single-paragraph blockquotes supported");
                let content = if self.kind(children[0]) == M::Paragraph {
                    self.rich(children[0])?
                } else {
                    let mut out = vec![];
                    self.inline(id, &[], &mut out)?;
                    out
                };
                BlockKind::Quote { content }
            }
            M::ThematicBreak => BlockKind::Rule,
            M::Code => {
                let data = CodeData::from_bytes(self.arena.get_type_data(id));
                ensure!(self.arena.get_str(data.meta).is_empty(), "code fence metadata unsupported");
                BlockKind::Code {
                    language: self.arena.get_str(data.lang).into(),
                    text: self.arena.get_str(data.value).into(),
                }
            }
            M::MdxJsxFlowElement => {
                let mut props = self.props(id)?;
                if matches!(self.name(id), "Paragraph" | "Heading" | "Quote" | "ListItem") {
                    let mut content = vec![];
                    self.literal_rich(id, &mut content)?;
                    let kind = match self.name(id) {
                        "Paragraph" => BlockKind::Paragraph { content },
                        "Quote" => BlockKind::Quote { content },
                        "ListItem" => {
                            let style: ListStyle = self.prop(&mut props, "style")?;
                            BlockKind::ListItem { style, content }
                        }
                        _ => {
                            let level: u8 = self.prop(&mut props, "level")?;
                            ensure!((1..=6).contains(&level), "invalid heading level");
                            BlockKind::Heading { level, content }
                        }
                    };
                    ensure!(props.is_empty(), "unknown literal block properties");
                    return Ok(Block { key, kind });
                }
                ensure!(self.arena.get_children(id).is_empty(), "unexpected leaf component children");
                let kind = match self.name(id) {
                    "Run" => {
                        let text: String = self.prop(&mut props, "text")?;
                        let marks: Vec<Mark> = if props.contains_key("marks") {
                            self.prop(&mut props, "marks")?
                        } else {
                            vec![]
                        };
                        let mut content = vec![];
                        push_run(&mut content, &text, &marks);
                        BlockKind::Paragraph { content }
                    }
                    "Code" => BlockKind::Code {
                        language: self.prop(&mut props, "language")?,
                        text: self.prop(&mut props, "text")?,
                    },
                    "PageLink" => BlockKind::PageLink {
                        page: self.prop(&mut props, "page")?,
                    },
                    "Image" => BlockKind::Image {
                        src: self.prop(&mut props, "src")?,
                        fallback: self.prop(&mut props, "fallback")?,
                        width: self.prop(&mut props, "width")?,
                        height: self.prop(&mut props, "height")?,
                        alt: self.prop(&mut props, "alt")?,
                    },
                    _ => bail!("unsupported block component"),
                };
                ensure!(props.is_empty(), "unknown block component properties");
                kind
            }
            _ => bail!("unsupported block syntax: {}", self.kind(id).name()),
        };
        Ok(Block { key, kind })
    }
    /// One flat list item: a tight single-paragraph, non-task item. Task and
    /// nested lists are unsupported input, diagnosed loudly.
    fn list_item(&mut self, id: u32, style: ListStyle) -> Result<BlockKind> {
        ensure!(self.kind(id) == MdastNodeType::ListItem, "malformed list");
        let item_data = ListItemData::from_bytes(self.arena.get_type_data(id));
        ensure!(item_data.checked == 2, "task lists are not supported by this profile");
        ensure!(!item_data.spread, "loose (multi-paragraph) list items are not supported");
        let item_children = self.arena.get_children(id).to_vec();
        if item_children.iter().any(|&child| self.kind(child) == MdastNodeType::List) {
            bail!("nested lists are not supported by this profile");
        }
        let content = match item_children.as_slice() {
            [] => vec![],
            [only] if self.kind(*only) == MdastNodeType::Paragraph => self.rich(*only)?,
            // A lone inline component as the item's whole content parses as a
            // flow element (no paragraph wrapper); accept Run/HardBreak atoms.
            [only] if self.kind(*only) == MdastNodeType::MdxJsxFlowElement && matches!(self.name(*only), "Run" | "HardBreak") => {
                let mut props = self.props(*only)?;
                match self.name(*only) {
                    "Run" => {
                        let text: String = self.prop(&mut props, "text")?;
                        let marks: Vec<Mark> = if props.contains_key("marks") {
                            self.prop(&mut props, "marks")?
                        } else {
                            vec![]
                        };
                        ensure!(props.is_empty(), "unknown Run properties");
                        let mut content = vec![];
                        push_run(&mut content, &text, &marks);
                        content
                    }
                    _ => vec![Inline::HardBreak],
                }
            }
            _ => bail!("list items must be single tight paragraphs"),
        };
        Ok(BlockKind::ListItem { style, content })
    }
    fn key_marker(&mut self, id: u32) -> Result<String> {
        let mut props = self.props(id)?;
        let key: String = self.prop(&mut props, "id")?;
        ensure!(props.is_empty() && self.arena.get_children(id).is_empty(), "invalid Key marker");
        Ok(key)
    }
    fn push_block(&mut self, id: u32, key: Option<String>) -> Result<Vec<Block>> {
        // A multi-item authored list expands to one keyed block per item: the
        // first item may carry an explicit Key marker, the rest are new.
        if self.kind(id) == MdastNodeType::List {
            let data = ListData::from_bytes(self.arena.get_type_data(id));
            let style = if data.ordered { ListStyle::Ordered } else { ListStyle::Bullet };
            let mut blocks = vec![];
            for item in self.arena.get_children(id).to_vec() {
                let was_keyed = blocks.is_empty() && key.is_some();
                let key = match if blocks.is_empty() { key.clone() } else { None } {
                    Some(key) => self.register(key)?,
                    None => self.provisional(),
                };
                let node = self.arena.get_node(item);
                self.spans.push((node.start_offset, node.end_offset));
                self.keyed.push(was_keyed);
                let kind = self.list_item(item, style)?;
                blocks.push(Block { key, kind });
            }
            return Ok(blocks);
        }
        let was_keyed = key.is_some();
        let key = match key {
            Some(key) => self.register(key)?,
            None => self.provisional(),
        };
        let node = self.arena.get_node(id);
        self.spans.push((node.start_offset, node.end_offset));
        self.keyed.push(was_keyed);
        Ok(vec![self.block(id, key)?])
    }
    /// Blocks of one container: Key markers bind to the next block; blocks
    /// without a marker are new (provisional identity, preparation assigns).
    fn blocks(&mut self, children: &[u32]) -> Result<Vec<Block>> {
        let mut blocks = vec![];
        let mut pending = None;
        for &id in children {
            if self.kind(id) == MdastNodeType::MdxJsxFlowElement && self.name(id) == "Key" {
                ensure!(pending.is_none(), "duplicate Key marker");
                pending = Some(self.key_marker(id)?);
            } else {
                blocks.extend(self.push_block(id, pending.take())?);
            }
        }
        ensure!(pending.is_none(), "orphan Key marker");
        Ok(blocks)
    }
}

/// Parser-derived byte span of one block in the original source (including
/// a leading BOM when present, so spans address the caller's string).
#[derive(Clone, Debug)]
pub struct BlockSpan {
    pub start: usize,
    pub end: usize,
}

/// Parse result: the document plus per-block spans and which blocks carried
/// explicit Key markers (document order: main page blocks, then each named
/// page's blocks).
#[derive(Clone, Debug)]
pub struct Parsed {
    pub document: Document,
    pub spans: Vec<BlockSpan>,
    pub keyed: Vec<bool>,
}

impl Parsed {
    pub fn unkeyed(&self) -> Vec<usize> {
        self.keyed.iter().enumerate().filter(|(_, keyed)| !**keyed).map(|(i, _)| i).collect()
    }
    /// True while main-page identity is not persisted yet.
    pub fn main_page_provisional(&self) -> bool {
        self.document.state.main_page.is_none()
    }
}

pub fn parse(source: &str) -> Result<Document> {
    parse_with_spans(source).map(|parsed| parsed.document)
}

pub fn parse_with_spans(source: &str) -> Result<Parsed> {
    let (arena, errors) = satteri_pulldown_cmark::parse(source, satteri_pulldown_cmark::MDX_OPTIONS);
    ensure!(errors.is_empty(), "invalid native MDX syntax ({} diagnostics)", errors.len());
    let mut reader = Reader {
        arena,
        keys: BTreeSet::new(),
        spans: vec![],
        keyed: vec![],
        auto: 0,
    };
    let children = reader.arena.get_children(0).to_vec();
    let first = *children.first().context("empty managed document")?;
    ensure!(reader.kind(first) == MdastNodeType::Yaml, "YAML frontmatter required");
    let raw: Value = frontmatter(reader.value(first))?;
    let mut document: Document = serde_json::from_value(raw.clone()).map_err(|_| anyhow::anyhow!("invalid/restricted managed frontmatter"))?;
    // A bound document must carry its publication association explicitly:
    // Option deserialization would silently treat a missing field as null
    // (standalone). Missing means unknown, never silently standalone.
    if document.state.binding.is_some() {
        ensure!(
            raw["@leafbud"]["binding"].get("publication").is_some(),
            "bound publication association is unknown (explicit null or URI required)"
        );
    }
    ensure!(
        document.format == FORMAT && document.state.state == "draft",
        "unsupported managed format/lifecycle"
    );
    if let Some(sync) = &document.state.sync {
        ensure!(sync.profile == EDITOR_PROFILE, "unsupported receipt profile");
        ensure!(document.state.binding.is_some(), "receipts without a binding are incoherent");
    }
    if let Some(binding) = &document.state.binding {
        ensure!(
            !binding.base_url.trim().is_empty() && !binding.root.is_empty() && !binding.entity_set.is_empty() && !binding.entry_fact.is_empty(),
            "binding must be a complete editor locator group"
        );
    }
    ensure!(!document.title.trim().is_empty(), "empty document title");

    // Implicit main page: top-level Key markers and ordinary blocks, in order.
    // Named Page definitions are lifted out wherever they appear; their
    // position among main blocks is not semantic.
    let main_key = match document.state.main_page.as_deref() {
        Some(key) => reader.register(key.to_owned())?,
        None => {
            reader.keys.insert(PROVISIONAL_PAGE.into());
            PROVISIONAL_PAGE.into()
        }
    };
    let mut main_blocks = Vec::new();
    let mut pending = None;
    let mut definitions = Vec::new();
    for id in children.iter().skip(1) {
        if reader.kind(*id) == MdastNodeType::MdxJsxFlowElement && reader.name(*id) == "Page" {
            ensure!(
                pending.is_none(),
                "orphan Key marker: top-level Keys bind main-page blocks, not Page definitions"
            );
            definitions.push(named_page(&mut reader, *id)?);
        } else if reader.kind(*id) == MdastNodeType::MdxJsxFlowElement && reader.name(*id) == "Key" {
            ensure!(pending.is_none(), "duplicate Key marker");
            pending = Some(reader.key_marker(*id)?);
        } else {
            let key = pending.take();
            main_blocks.extend(reader.push_block(*id, key)?);
        }
    }
    ensure!(pending.is_none(), "orphan Key marker");
    let mut pages = vec![Page {
        key: main_key,
        blocks: main_blocks,
    }];
    pages.extend(definitions);
    document.pages = pages;

    let page_names: BTreeSet<&str> = document.pages.iter().map(|p| p.key.as_str()).collect();
    ensure!(page_names.len() == document.pages.len(), "duplicate page key");
    for page in &document.pages {
        for block in &page.blocks {
            if let BlockKind::PageLink { page } = &block.kind {
                ensure!(page_names.contains(page.as_str()), "unresolved page link");
            }
        }
    }
    // Arena offsets are relative to the BOM-stripped source.
    let bom = if source.starts_with('\u{feff}') { '\u{feff}'.len_utf8() } else { 0 };
    let spans = reader
        .spans
        .iter()
        .map(|&(start, end)| BlockSpan {
            start: start as usize + bom,
            end: end as usize + bom,
        })
        .collect();
    Ok(Parsed {
        document,
        spans,
        keyed: reader.keyed,
    })
}

/// One explicit named page definition.
fn named_page(reader: &mut Reader, id: u32) -> Result<Page> {
    let mut props = reader.props(id)?;
    let name: String = reader.prop(&mut props, "name")?;
    let kind: String = reader.prop(&mut props, "kind")?;
    ensure!(props.is_empty() && kind == "linear", "unsupported page properties");
    let key = reader.register(name)?;
    let children = reader.arena.get_children(id).to_vec();
    let blocks = reader.blocks(&children)?;
    Ok(Page { key, blocks })
}

/// Identity indexing is allowed even when a user is mid-edit in the body.
/// Nothing is rewritten from this partial inspection.
pub fn header(source: &str) -> Result<Value> {
    let (arena, _) = satteri_pulldown_cmark::parse(source, satteri_pulldown_cmark::MDX_OPTIONS);
    let first = *arena.get_children(0).first().context("managed frontmatter absent")?;
    ensure!(
        MdastNodeType::from_u8(arena.get_node(first).node_type) == Some(MdastNodeType::Yaml),
        "managed frontmatter absent"
    );
    let value = arena.get_str(decode_string_ref_data(arena.get_type_data(first)));
    frontmatter(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bound_state() -> State {
        State {
            id: Uuid::nil(),
            state: "draft".into(),
            main_page: Some("p-main".into()),
            binding: Some(Binding {
                base_url: "https://leaflet.pub".into(),
                token: Uuid::nil(),
                root: "e-root".into(),
                entity_set: "es".into(),
                entry_fact: "f-entry".into(),
                publication: None,
            }),
            nodes: BTreeMap::new(),
            sync: None,
        }
    }

    #[test]
    fn implicit_main_page_and_named_definitions_interleave() {
        let source = "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\"}\n---\n\n# Head\n\npara one\n\n<Page name=\"p-about\" kind=\"linear\">\n\nabout\n\n</Page>\n\npara two\n";
        let parsed = parse_with_spans(source).unwrap();
        let doc = &parsed.document;
        assert_eq!(doc.pages.len(), 2);
        assert_eq!(doc.pages[0].key, "p-main");
        assert_eq!(doc.pages[0].blocks.len(), 3);
        assert_eq!(
            doc.pages[0].blocks[0].kind,
            BlockKind::Heading {
                level: 1,
                content: vec![Inline::Run {
                    text: "Head".into(),
                    marks: vec![]
                }]
            }
        );
        assert_eq!(
            doc.pages[0].blocks[2].kind,
            BlockKind::Paragraph {
                content: vec![Inline::Run {
                    text: "para two".into(),
                    marks: vec![]
                }]
            }
        );
        assert_eq!(doc.pages[1].key, "p-about");
        assert_eq!(doc.pages[1].blocks.len(), 1);
    }

    #[test]
    fn unkeyed_blocks_are_new_not_errors() {
        let source = "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\"}\n---\n\nfirst\n\nsecond\n";
        let parsed = parse_with_spans(source).unwrap();
        assert_eq!(parsed.unkeyed().len(), 2);
        assert_eq!(parsed.document.pages[0].blocks[0].key, "b-auto-1");
        assert_eq!(parsed.document.pages[0].blocks[1].key, "b-auto-2");
        // Deterministic across runs; supplied spellings are skipped.
        let again = parse_with_spans(source).unwrap();
        assert_eq!(again.document.pages[0].blocks[0].key, "b-auto-1");
        let colliding = "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\"}\n---\n\n<Key id=\"b-auto-1\" />\nfirst\n\nsecond\n";
        let parsed = parse_with_spans(colliding).unwrap();
        assert_eq!(parsed.document.pages[0].blocks[1].key, "b-auto-2");
        assert_eq!(parsed.document.pages[0].blocks[0].key, "b-auto-1");
    }

    #[test]
    fn main_page_identity_defaults_provisional_until_preparation() {
        let source =
            "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\"}\n---\n\nhello\n";
        let parsed = parse_with_spans(source).unwrap();
        assert!(parsed.main_page_provisional());
        assert_eq!(parsed.document.pages[0].key, "p-auto");
        // Provisional main-page identity does not leak into editor value.
        let value = parsed.document.editor_value();
        assert_eq!(value["main"]["association"], json!({"kind": "new"}));
    }

    #[test]
    fn flat_lists_parse_one_keyed_item_per_block() {
        let state =
            "format: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\"}";
        let source = format!("---\n{state}\n---\n\n<Key id=\"b-1\" />\n- one\n\n<Key id=\"b-2\" />\n1. first\n\n* two\n");
        let parsed = parse_with_spans(&source).unwrap();
        let blocks = &parsed.document.pages[0].blocks;
        assert_eq!(blocks.len(), 3);
        assert_eq!(
            blocks[0].kind,
            BlockKind::ListItem {
                style: ListStyle::Bullet,
                content: vec![Inline::Run {
                    text: "one".into(),
                    marks: vec![]
                }]
            }
        );
        assert_eq!(
            blocks[1].kind,
            BlockKind::ListItem {
                style: ListStyle::Ordered,
                content: vec![Inline::Run {
                    text: "first".into(),
                    marks: vec![]
                }]
            }
        );
        assert_eq!(parsed.unkeyed(), vec![2]);
        // An authored multi-item list without key markers parses as new items
        // needing preparation, each its own block.
        let multi = format!("---\n{state}\n---\n\n- one\n- two\n");
        let parsed = parse_with_spans(&multi).unwrap();
        assert_eq!(parsed.document.pages[0].blocks.len(), 2);
        assert_eq!(parsed.unkeyed().len(), 2);
        assert!(
            parsed.document.pages[0]
                .blocks
                .iter()
                .all(|b| matches!(b.kind, BlockKind::ListItem { style: ListStyle::Bullet, .. }))
        );
        // Task and nested lists are diagnosed, never approximated.
        let task = format!("---\n{state}\n---\n\n- [ ] todo\n");
        assert!(parse(&task).unwrap_err().to_string().contains("task lists"));
        let nested = format!("---\n{state}\n---\n\n- one\n  - nested\n");
        let message = parse(&nested).unwrap_err().to_string();
        assert!(message.contains("nested lists"), "{message}");
    }

    #[test]
    fn retired_rows_and_new_keys_are_valid_source() {
        let mut state = bound_state();
        state.sync = None;
        let mut document = Document {
            format: FORMAT.into(),
            title: "T".into(),
            description: None,
            tags: vec![],
            publication: Publication::Inherit,
            theme: None,
            extra: BTreeMap::new(),
            pages: vec![],
            state,
        };
        document.state.nodes.insert(
            "b-gone".into(),
            Node {
                entity: "e-gone".into(),
                occurrence: Occurrence::Block {
                    placement: "f-gone".into(),
                    text_fact: None,
                    inactive_text_fact: None,
                },
            },
        );
        document.pages = vec![Page {
            key: "p-main".into(),
            blocks: vec![Block {
                key: "b-fresh".into(),
                kind: BlockKind::Rule,
            }],
        }];
        let source = crate::render(&document).unwrap();
        let parsed = parse_with_spans(&source).unwrap();
        let value = parsed.document.editor_value();
        assert_eq!(value["main"]["blocks"][0]["association"], json!({"kind": "new"}));
        assert_eq!(value["retired"][0]["entity"], "e-gone");
        assert_eq!(value["retired"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn binding_and_receipt_coherence() {
        let bound = "\"binding\": {\"baseUrl\": \"https://l\", \"token\": \"00000000-0000-0000-0000-000000000000\", \"root\": \"r\", \"entitySet\": \"e\", \"entryFact\": \"f\", \"publication\": null}";
        let sync = ", \"sync\": {\"profile\": \"leaflet-editor/3\", \"localHash\": \"sha256:x\", \"remoteHash\": \"sha256:y\"}";
        let source = |state: String| {
            format!(
                "---\nformat: leafbud-mdx/2\ntitle: T\n\"@leafbud\": {{\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\", {state}, \"nodes\": {{}}}}\n---\n\n<Key id=\"b-1\" />\nhello\n"
            )
        };
        // Sync without binding is incoherent.
        assert!(parse(&source(format!("\"binding\": null{sync}"))).is_err());
        // Bound without receipts is a valid bound-without-receipt state.
        assert!(parse(&source(bound.to_owned())).is_ok());
        // Bound with matching-profile receipts is valid.
        assert!(parse(&source(format!("{bound}{sync}"))).is_ok());
        // Unknown future profiles are rejected.
        let future = format!("{bound}, \"sync\": {{\"profile\": \"leaflet-editor/99\", \"localHash\": \"sha256:x\", \"remoteHash\": \"sha256:y\"}}");
        assert!(parse(&source(future)).is_err());
    }

    #[test]
    fn format_one_is_gone() {
        let old = "---\nformat: leafbud-mdx/1\n\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"binding\": {\"baseUrl\": \"https://l\", \"token\": \"00000000-0000-0000-0000-000000000000\", \"root\": \"r\", \"entitySet\": \"e\", \"entryFact\": \"f\", \"publication\": null}, \"nodes\": {}, \"sync\": {\"profile\": \"leaflet-editor/2\", \"localHash\": \"h\", \"remoteHash\": \"h\"}}\n---\n\nhello\n";
        assert!(parse(old).is_err());
    }
}
