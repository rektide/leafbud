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
/// A document that explicitly owns this spelling elsewhere yields
/// `p-auto-2`, `p-auto-3`, … for the provisional main page.
pub const PROVISIONAL_MAIN_PAGE: &str = "p-auto";

struct Reader {
    arena: Arena<Mdast>,
    keys: BTreeSet<String>,
    /// Explicit identities known to exist but not yet traversed (Key marker
    /// ids, Page names, node-table rows). Provisional allocation skips them
    /// so a new object can never collide with a supplied later key or borrow
    /// a confirmed/retired association.
    reserved: BTreeSet<String>,
    /// Byte spans of each block in document order (parser-derived; relative
    /// to the BOM-stripped source the arena was built from).
    spans: Vec<(u32, u32)>,
    /// Whether each block carried an explicit Key marker.
    keyed: Vec<bool>,
    /// Source-only comments, attached to the following block in its container.
    comments: Vec<(String, u32, u32)>,
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
    pub(crate) fn valid_key(key: &str) -> bool {
        let mut bytes = key.bytes();
        bytes.next().is_some_and(|b| b.is_ascii_alphabetic()) && bytes.all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    }
    fn register(&mut self, key: String) -> Result<String> {
        ensure!(Self::valid_key(&key), "invalid source key spelling");
        // Consume this occurrence's pre-reserved spelling; a second explicit
        // use of the same spelling no longer finds it reserved and fails the
        // duplicate check below.
        self.reserved.remove(&key);
        ensure!(self.keys.insert(key.clone()), "duplicate source key");
        Ok(key)
    }
    /// One deterministic provisional key for a new block. Never persisted:
    /// preparation replaces it with a one-time assigned key.
    fn provisional(&mut self) -> String {
        loop {
            self.auto += 1;
            let key = format!("b-auto-{}", self.auto);
            if !self.keys.contains(&key) && !self.reserved.contains(&key) {
                self.keys.insert(key.clone());
                return key;
            }
        }
    }
    /// Deterministic provisional main-page key (`p-auto`, then `p-auto-2`,
    /// …), skipping every reserved or allocated spelling. Never persisted.
    fn provisional_page(&mut self) -> String {
        let mut n = 1;
        loop {
            let key = if n == 1 {
                PROVISIONAL_MAIN_PAGE.to_owned()
            } else {
                format!("{PROVISIONAL_MAIN_PAGE}-{n}")
            };
            if !self.keys.contains(&key) && !self.reserved.contains(&key) {
                self.keys.insert(key.clone());
                return key;
            }
            n += 1;
        }
    }
    /// Leniently read one string property, for reserving spellings ahead of
    /// allocation. Malformed markers still fail the real traversal.
    fn string_prop(&self, id: u32, name: &str) -> Option<String> {
        let data = self.arena.get_type_data(id);
        for i in 0..decode_mdx_jsx_attr_count(data) {
            let (kind, attr, value) = decode_mdx_jsx_attr(data, i);
            if self.arena.get_str(attr) != name {
                continue;
            }
            return match kind {
                MDX_ATTR_LITERAL_PROP => Some(self.arena.get_str(value).to_owned()),
                MDX_ATTR_EXPRESSION_PROP => serde_json::from_str::<Value>(self.arena.get_str(value)).ok()?.as_str().map(str::to_owned),
                _ => None,
            };
        }
        None
    }
    /// Reserve every explicit identity anywhere in the source tree before
    /// any provisional allocation: Key marker ids and Page names, whatever
    /// container they appear in and however their attributes are spelled.
    fn reserve_explicit(&mut self, id: u32) {
        if matches!(self.kind(id), MdastNodeType::MdxJsxFlowElement | MdastNodeType::MdxJsxTextElement) {
            match self.name(id) {
                "Key" => {
                    if let Some(key) = self.string_prop(id, "id") {
                        self.reserved.insert(key);
                    }
                }
                "Page" => {
                    if let Some(name) = self.string_prop(id, "name") {
                        self.reserved.insert(name);
                    }
                }
                _ => {}
            }
        }
        let children: Vec<u32> = self.arena.get_children(id).to_vec();
        for child in children {
            self.reserve_explicit(child);
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

    fn comment(&self, id: u32) -> Result<Option<(u32, u32)>> {
        if self.kind(id) != MdastNodeType::MdxFlowExpression {
            return Ok(None);
        }
        let value = self.value(id).trim();
        let comment = value.strip_prefix("/*").and_then(|v| v.strip_suffix("*/"));
        ensure!(
            comment.is_some_and(|v| !v.contains("*/")),
            "only standalone {{/* comment */}} expressions are supported; expressions are never evaluated"
        );
        let node = self.arena.get_node(id);
        Ok(Some((node.start_offset, node.end_offset)))
    }

    fn attach_comments(&mut self, pending: &mut Vec<(u32, u32)>, blocks: &[Block]) -> Result<()> {
        if pending.is_empty() {
            return Ok(());
        }
        let block = blocks.first().context("body comments require a following block in the same page")?;
        self.comments.extend(pending.drain(..).map(|(start, end)| (block.key.clone(), start, end)));
        Ok(())
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
        let mut comments = vec![];
        for &id in children {
            if let Some(span) = self.comment(id)? {
                comments.push(span);
            } else if self.kind(id) == MdastNodeType::MdxJsxFlowElement && self.name(id) == "Key" {
                ensure!(pending.is_none(), "duplicate Key marker");
                pending = Some(self.key_marker(id)?);
            } else {
                let next = self.push_block(id, pending.take())?;
                self.attach_comments(&mut comments, &next)?;
                blocks.extend(next);
            }
        }
        ensure!(pending.is_none(), "orphan Key marker");
        ensure!(
            comments.is_empty(),
            "body comments require a following block in the same page; trailing/orphan comments are unsupported"
        );
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

/// One source-only standalone MDX comment. It follows its block within this
/// page on remote adoption; deletion or cross-page movement requires a local
/// author decision. It never participates in native/desired fingerprints.
#[derive(Clone, Debug)]
pub struct BlockComment {
    pub page: String,
    pub block: String,
    pub span: BlockSpan,
}

/// Parse result: the document plus per-block spans and which blocks carried
/// explicit Key markers (document order: main page blocks, then each named
/// page's blocks).
#[derive(Clone, Debug)]
pub struct Parsed {
    pub document: Document,
    pub spans: Vec<BlockSpan>,
    pub keyed: Vec<bool>,
    pub comments: Vec<BlockComment>,
}

impl Parsed {
    /// Comment identity across key preparation: object order, not provisional
    /// spelling, so assigning a durable key does not detach its source trivia.
    pub(crate) fn comment_value(&self, source: &str) -> Vec<(usize, usize, String)> {
        self.comments
            .iter()
            .map(|comment| {
                let page = self.document.pages.iter().position(|p| p.key == comment.page).expect("comment page");
                let block = self.document.pages[page]
                    .blocks
                    .iter()
                    .position(|b| b.key == comment.block)
                    .expect("comment block");
                (page, block, source[comment.span.start..comment.span.end].to_owned())
            })
            .collect()
    }

    pub fn unkeyed(&self) -> Vec<usize> {
        self.keyed.iter().enumerate().filter(|(_, keyed)| !**keyed).map(|(i, _)| i).collect()
    }
    /// True while main-page identity is not persisted yet.
    pub fn main_page_provisional(&self) -> bool {
        self.document.state.main_page.is_none()
    }
    /// The main page's provisional key spelling while identity is not
    /// persisted (preparation substitutes the assigned key for it).
    pub fn provisional_main_key(&self) -> Option<&str> {
        self.main_page_provisional().then(|| self.document.pages[0].key.as_str())
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
        reserved: BTreeSet::new(),
        spans: vec![],
        keyed: vec![],
        comments: vec![],
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
    ensure!(document.format == FORMAT, "unsupported managed format");
    document.state.validate_lifecycle().map_err(anyhow::Error::msg)?;
    if let Some(released) = &document.state.released {
        ensure!(
            document.state.state == "published" && released.profile == RELEASE_PROFILE,
            "unsupported release evidence/lifecycle"
        );
        ensure!(
            released
                .input_hash
                .strip_prefix("sha256:")
                .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))),
            "invalid released input fingerprint"
        );
        validate_record_cid(&released.cid).map_err(anyhow::Error::msg)?;
        let _: jiff::Timestamp = released.published_at.parse().context("invalid released publication date")?;
        let _: jiff::Timestamp = released.confirmed_at.parse().context("invalid release confirmation date")?;
        ensure!(released.path.starts_with('/'), "released path must be absolute logical path");
    }
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
    document.state.validate_attempt().map_err(anyhow::Error::msg)?;
    if let Some(names) = &document.state.names {
        names.validate().map_err(anyhow::Error::msg).context("invalid naming witness coordinate")?;
    }

    // Implicit main page: top-level Key markers and ordinary blocks, in order.
    // Named Page definitions are lifted out wherever they appear; their
    // position among main blocks is not semantic.
    //
    // Before allocating any provisional identity, reserve every explicit
    // source identity (Key ids, Page names) plus every node-table row: a new
    // unkeyed object must not collide with a supplied later spelling or
    // silently adopt a confirmed/retired association.
    for &id in &children {
        reader.reserve_explicit(id);
    }
    for key in document.state.nodes.keys() {
        reader.reserved.insert(key.clone());
    }
    let main_key = match document.state.main_page.as_deref() {
        Some(key) => reader.register(key.to_owned())?,
        None => reader.provisional_page(),
    };
    let mut main_blocks = Vec::new();
    // Source-position bookkeeping for the final document-order grouping:
    // spans/keyed are recorded in traversal order, then regrouped below.
    let mut main_span_indices: Vec<usize> = Vec::new();
    let mut definition_span_ranges: Vec<(usize, usize)> = Vec::new();
    let mut pending = None;
    let mut comments = vec![];
    let mut definitions = Vec::new();
    for id in children.iter().skip(1) {
        if reader.kind(*id) == MdastNodeType::MdxJsxFlowElement && reader.name(*id) == "Page" {
            ensure!(
                comments.is_empty(),
                "comments before Page definitions are ambiguous; place them before a block inside the page"
            );
            ensure!(
                pending.is_none(),
                "orphan Key marker: top-level Keys bind main-page blocks, not Page definitions"
            );
            let spans_before = reader.spans.len();
            definitions.push(named_page(&mut reader, *id)?);
            definition_span_ranges.push((spans_before, reader.spans.len()));
        } else if let Some(span) = reader.comment(*id)? {
            comments.push(span);
        } else if reader.kind(*id) == MdastNodeType::MdxJsxFlowElement && reader.name(*id) == "Key" {
            ensure!(pending.is_none(), "duplicate Key marker");
            pending = Some(reader.key_marker(*id)?);
        } else {
            let key = pending.take();
            let spans_before = reader.spans.len();
            let next = reader.push_block(*id, key)?;
            reader.attach_comments(&mut comments, &next)?;
            main_blocks.extend(next);
            main_span_indices.extend(spans_before..reader.spans.len());
        }
    }
    ensure!(pending.is_none(), "orphan Key marker");
    ensure!(
        comments.is_empty(),
        "body comments require a following block in the same page; trailing/orphan comments are unsupported"
    );
    let mut pages = vec![Page {
        key: main_key,
        blocks: main_blocks,
    }];
    pages.extend(definitions);
    document.pages = pages;

    // Spans and keyed flags follow the returned document order (main page
    // blocks, then each named page's blocks), not the interleaved source
    // traversal order, so per-block addressing stays attached to its object.
    let mut order: Vec<usize> = main_span_indices;
    for &(start, end) in &definition_span_ranges {
        order.extend(start..end);
    }
    debug_assert_eq!(order.len(), reader.keyed.len());

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
    let spans = order
        .iter()
        .map(|&index| {
            let &(start, end) = &reader.spans[index];
            BlockSpan {
                start: start as usize + bom,
                end: end as usize + bom,
            }
        })
        .collect();
    let keyed = order.iter().map(|&index| reader.keyed[index]).collect();
    let comments = reader
        .comments
        .into_iter()
        .map(|(block, start, end)| {
            let page = document
                .pages
                .iter()
                .find(|p| p.blocks.iter().any(|b| b.key == block))
                .expect("attached block")
                .key
                .clone();
            BlockComment {
                page,
                block,
                span: BlockSpan {
                    start: start as usize + bom,
                    end: end as usize + bom,
                },
            }
        })
        .collect();
    Ok(Parsed {
        document,
        spans,
        keyed,
        comments,
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

/// Actual JSX identity/reference attributes, selected through the production
/// arena/decoded property meaning, not text searches over prose or code.
pub(crate) fn rename_attributes(source: &str, old: &str, new: &str) -> Result<Vec<super::TidyEdit>> {
    replace_attributes(source, old, new, true)
}

/// Preparation owns references only, never explicit Key/Page identities.
pub(crate) fn retarget_page_links(source: &str, old: &str, new: &str) -> Result<Vec<super::TidyEdit>> {
    replace_attributes(source, old, new, false)
}

fn replace_attributes(source: &str, old: &str, new: &str, identities: bool) -> Result<Vec<super::TidyEdit>> {
    let (arena, errors) = satteri_pulldown_cmark::parse(source, satteri_pulldown_cmark::MDX_OPTIONS);
    ensure!(errors.is_empty(), "invalid MDX syntax");
    let reader = Reader {
        arena,
        keys: BTreeSet::new(),
        reserved: BTreeSet::new(),
        spans: vec![],
        keyed: vec![],
        comments: vec![],
        auto: 0,
    };
    fn visit(reader: &Reader, id: u32, source: &str, bom: usize, replacement: (&str, &str), identities: bool, edits: &mut Vec<super::TidyEdit>) -> Result<()> {
        let (old, new) = replacement;
        if reader.kind(id) == MdastNodeType::MdxJsxFlowElement {
            let prop = match reader.name(id) {
                "Key" if identities => Some("id"),
                "Page" if identities => Some("name"),
                "PageLink" => Some("page"),
                _ => None,
            };
            if let Some(prop) = prop.filter(|prop| reader.string_prop(id, prop).as_deref() == Some(old)) {
                let node = reader.arena.get_node(id);
                let start = node.start_offset as usize + bom;
                let end = node.end_offset as usize + bom;
                let range = attribute_range(&source[start..end], prop)?;
                edits.push(super::TidyEdit {
                    start: start + range.start,
                    end: start + range.end,
                    text: super::json(new),
                });
            }
        }
        for &child in reader.arena.get_children(id) {
            visit(reader, child, source, bom, replacement, identities, edits)?;
        }
        Ok(())
    }
    let mut edits = vec![];
    visit(
        &reader,
        0,
        source,
        usize::from(source.starts_with('\u{feff}')) * 3,
        (old, new),
        identities,
        &mut edits,
    )?;
    Ok(edits)
}

pub(crate) fn valid_key(key: &str) -> bool {
    Reader::valid_key(key)
}

/// Token extent only, within a parser-validated opening tag. This does not
/// interpret MDX or choose components. Literal/expression properties already
/// have their meaning checked by Reader; we just preserve surrounding trivia.
fn attribute_range(tag: &str, selected: &str) -> Result<std::ops::Range<usize>> {
    let b = tag.as_bytes();
    let mut at = 1;
    while at < b.len() && !b[at].is_ascii_whitespace() && b[at] != b'>' {
        at += 1;
    }
    while at < b.len() {
        while at < b.len() && b[at].is_ascii_whitespace() {
            at += 1;
        }
        ensure!(at < b.len() && !b"/>".contains(&b[at]), "selected JSX property not found in opening tag");
        let name_start = at;
        while at < b.len() && !b[at].is_ascii_whitespace() && b[at] != b'=' {
            at += 1;
        }
        let name = &tag[name_start..at];
        while at < b.len() && b[at].is_ascii_whitespace() {
            at += 1;
        }
        ensure!(b.get(at) == Some(&b'='), "unsupported JSX property token");
        at += 1;
        while at < b.len() && b[at].is_ascii_whitespace() {
            at += 1;
        }
        let start = at;
        let first = *b.get(at).context("missing JSX value")?;
        if first == b'\'' || first == b'"' {
            at += 1;
            while at < b.len() && b[at] != first {
                at += 1;
            }
            ensure!(at < b.len(), "unterminated literal attribute");
            at += 1;
        } else {
            ensure!(first == b'{', "unsupported JSX property token");
            at += 1;
            let mut depth = 1;
            let mut quoted = false;
            while at < b.len() && depth > 0 {
                match b[at] {
                    b'\\' if quoted => {
                        at += 2;
                        continue;
                    }
                    b'"' => quoted = !quoted,
                    b'{' if !quoted => depth += 1,
                    b'}' if !quoted => depth -= 1,
                    _ => {}
                }
                at += 1;
            }
            ensure!(depth == 0, "unterminated expression attribute");
        }
        if name == selected {
            return Ok(start..at);
        }
    }
    bail!("selected JSX property absent")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bound_state() -> State {
        State {
            publication_attempt: None,
            released: None,
            names: None,
            last_resolution: None,
            writer: None,
            pending: None,
            id: Uuid::nil(),
            state: "draft".into(),
            main_page: Some("p-main".into()),
            binding: Some(Binding {
                uri: None,
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
    fn spans_follow_document_order_not_source_traversal_order() {
        // A named Page definition between two main blocks: the document
        // groups main blocks first, so spans/keyed must be regrouped to
        // follow the returned object order, not the interleaved source walk.
        let state = "\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\"}";
        let source = format!(
            "---\nformat: leafbud-mdx/2\ntitle: T\n{state}\n---\n\n<Key id=\"b-a\" />\nmain A\n\n<Page name=\"p-about\" kind=\"linear\">\n\n<Key id=\"b-about\" />\nabout body\n\nabout new\n\n</Page>\n\nmain B\n"
        );
        let parsed = parse_with_spans(&source).unwrap();
        let flattened: Vec<&str> = parsed
            .document
            .pages
            .iter()
            .flat_map(|page| page.blocks.iter().map(|block| block.key.as_str()))
            .collect();
        assert_eq!(
            flattened,
            ["b-a", "b-auto-2", "b-about", "b-auto-1"],
            "main blocks first; provisional numbering follows the source walk"
        );
        assert_eq!(parsed.keyed, [true, false, true, false]);
        // Each span must address its own block's source text.
        for (text, index) in [("main A", 0usize), ("main B", 1), ("about body", 2), ("about new", 3)] {
            let span = &parsed.spans[index];
            assert!(
                source[span.start..span.end].contains(text),
                "span {text:?} addresses {:?} instead",
                &source[span.start..span.end]
            );
        }
    }

    #[test]
    fn provisional_keys_never_borrow_supplied_or_confirmed_identities() {
        let base = "\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\"}";
        // A later explicit key is reserved before provisional allocation, so
        // an earlier unkeyed block does not false-collide with it.
        let source = format!("---\nformat: leafbud-mdx/2\ntitle: T\n{base}\n---\n\nnew first\n\n<Key id=\"b-auto-1\" />\nkept\n");
        let parsed = parse_with_spans(&source).unwrap();
        assert_eq!(parsed.document.pages[0].blocks[0].key, "b-auto-2", "the supplied later spelling is skipped");
        assert_eq!(parsed.document.pages[0].blocks[1].key, "b-auto-1");
        // A retired node row is reserved: a new block never borrows its
        // confirmed association.
        let retired = "\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"mainPage\": \"p-main\", \"nodes\": {\"b-auto-1\": {\"entity\": \"e-gone\", \"kind\": \"block\", \"placement\": \"f-gone\"}}}";
        let source = format!("---\nformat: leafbud-mdx/2\ntitle: T\n{retired}\n---\n\nfresh\n");
        let parsed = parse_with_spans(&source).unwrap();
        assert_eq!(parsed.document.pages[0].blocks[0].key, "b-auto-2");
        let value = parsed.document.editor_value();
        assert_eq!(
            value["main"]["blocks"][0]["association"],
            json!({"kind": "new", "key": "b-auto-2"}),
            "no hidden association adopted"
        );
        assert_eq!(value["retired"][0]["entity"], "e-gone", "the row stays retired");
    }

    #[test]
    fn provisional_main_page_yields_to_explicit_p_auto_identities() {
        let base = "\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\"}";
        // A named page owning the p-auto spelling forces the provisional
        // main-page key to skip it.
        let source = format!("---\nformat: leafbud-mdx/2\ntitle: T\n{base}\n---\n\nhello\n\n<Page name=\"p-auto\" kind=\"linear\">\n\nabout\n\n</Page>\n");
        let parsed = parse_with_spans(&source).unwrap();
        assert_eq!(parsed.document.pages[0].key, "p-auto-2");
        assert_eq!(parsed.document.pages[1].key, "p-auto");
        // A retired page row keyed p-auto is equally reserved.
        let retired = "\"@leafbud\": {\"id\": \"00000000-0000-0000-0000-000000000000\", \"state\": \"draft\", \"nodes\": {\"p-auto\": {\"entity\": \"e-old-main\", \"kind\": \"page\", \"references\": []}}}";
        let source = format!("---\nformat: leafbud-mdx/2\ntitle: T\n{retired}\n---\n\nhello\n");
        let parsed = parse_with_spans(&source).unwrap();
        assert_eq!(parsed.document.pages[0].key, "p-auto-2");
        let value = parsed.document.editor_value();
        assert_eq!(value["main"]["association"], json!({"kind": "new", "key": "p-auto-2"}));
        assert_eq!(value["retired"][0]["entity"], "e-old-main");
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
        // Provisional main-page identity is explicit new identity, never a
        // hidden alias, and carries its deterministic source spelling.
        let value = parsed.document.editor_value();
        assert_eq!(value["main"]["association"], json!({"kind": "new", "key": "p-auto"}));
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
        assert_eq!(value["main"]["blocks"][0]["association"], json!({"kind": "new", "key": "b-fresh"}));
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
