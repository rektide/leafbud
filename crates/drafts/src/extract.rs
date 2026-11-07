//! Draft extraction — a leaflet editor draft's fact graph → the canonical
//! `site.standard.document` record (`design/draft-extraction.glm53m.md`,
//! the extraction half).
//!
//! Mirrors Leaflet's own `src/utils/factsToPagesRecord.ts` (@`600c7b5f`):
//! where Leaflet's publish drops a block, extraction drops it too — but
//! *warns* instead of staying silent. Unmappable blocks never fail the run;
//! only structural errors do (no root page, missing positions, no site
//! source). The output reuses the generated record types from
//! `springleaf-leaflet`, so an exported draft is byte-compatible with
//! `publish build` output and can flow straight into `publish run`.
//!
//! v0 read-only scope (per the design): `image`, `image-gallery`, and
//! `poll` blocks skip + warn (they need PDS/blob side effects a read-only
//! pull cannot perform); canvas pages and page/envelope themes warn and are
//! omitted; the members-only delimiter stays with the *full* content kept.

use std::collections::{HashMap, HashSet};

use std::str::FromStr as _;

use jacquard_common::DefaultStr;
use jacquard_common::types::cid::Cid;
use jacquard_common::types::string::{AtUri, Datetime, UriValue};
use springleaf_leaflet::pub_leaflet::blocks::blockquote::Blockquote;
use springleaf_leaflet::pub_leaflet::blocks::bsky_post::{BskyPost, BskyPostView};
use springleaf_leaflet::pub_leaflet::blocks::button::Button;
use springleaf_leaflet::pub_leaflet::blocks::code::Code;
use springleaf_leaflet::pub_leaflet::blocks::drawing::{Drawing as DrawingBlock, Fill as DrawingFill, Stroke as DrawingStroke, ViewBox as DrawingViewBox};
use springleaf_leaflet::pub_leaflet::blocks::embedded_canvas::EmbeddedCanvas;
use springleaf_leaflet::pub_leaflet::blocks::header::Header;
use springleaf_leaflet::pub_leaflet::blocks::horizontal_rule::HorizontalRule;
use springleaf_leaflet::pub_leaflet::blocks::html::Html as HtmlBlock;
use springleaf_leaflet::pub_leaflet::blocks::iframe::Iframe;
use springleaf_leaflet::pub_leaflet::blocks::math::Math;
use springleaf_leaflet::pub_leaflet::blocks::members_only_delimiter::{MembersOnlyDelimiter, MembersOnlyDelimiterAudience};
use springleaf_leaflet::pub_leaflet::blocks::ordered_list::{ListItemContent as OrderedListItemContent, OrderedList};
use springleaf_leaflet::pub_leaflet::blocks::page::{Page as PageBlock, PageDisplay};
use springleaf_leaflet::pub_leaflet::blocks::post_header::PostHeader;
use springleaf_leaflet::pub_leaflet::blocks::posts_list::{PostsList, PostsListView};
use springleaf_leaflet::pub_leaflet::blocks::questions::{Questions, QuestionsAudience};
use springleaf_leaflet::pub_leaflet::blocks::recommended_pubs::RecommendedPubs;
use springleaf_leaflet::pub_leaflet::blocks::reply::Reply;
use springleaf_leaflet::pub_leaflet::blocks::signup::Signup;
use springleaf_leaflet::pub_leaflet::blocks::standard_site_post::{StandardSitePost, StandardSitePostSize};
use springleaf_leaflet::pub_leaflet::blocks::standard_site_publication::StandardSitePublication;
use springleaf_leaflet::pub_leaflet::blocks::text::Text;
use springleaf_leaflet::pub_leaflet::blocks::unordered_list::UnorderedList;
use springleaf_leaflet::pub_leaflet::blocks::website::Website;
use springleaf_leaflet::pub_leaflet::content::{Content, ContentPagesItem};
use springleaf_leaflet::pub_leaflet::pages::linear_document::{Block, BlockAlignment, BlockBlock, LinearDocument};
use springleaf_leaflet::pub_leaflet::richtext::facet::{
    Bold, ByteSlice, Code as CodeFacet, Facet, FacetFeaturesItem, Highlight, Italic, Link, Strikethrough, Underline,
};
use springleaf_leaflet::site_standard::document::Document;
use springleaf_leaflet::slugify;

use crate::Error;
use crate::facts::{FactData, FactRow, InkStroke, ScanIndex};
use crate::fractional;
use crate::yjs::{self, DecodedText, Mark};

/// A structured warn: what was dropped/deferred, where, and why.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Warn {
    /// The `block/type` value involved (or `page` / `root` / `text` for
    /// non-block scopes).
    pub block_type: String,
    pub entity: String,
    pub reason: String,
}

/// Options for [`extract`].
#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    /// Fixed `publishedAt` (offline determinism); default: the linked
    /// published document's `publishedAt`, else now.
    pub published_at: Option<String>,
    /// Slug override (also drives the record `path`); default `slugify(title)`.
    pub slug: Option<String>,
}

/// Everything extraction needs beyond the fact graph.
#[derive(Debug, Clone)]
pub struct ExtractContext<'a> {
    /// `get_leaflet_data` result: `root_entity`, permission-token title /
    /// description, publication linkage.
    pub data: &'a crate::LeafletData,
    /// The operator's DID (the `/home` identity payload) — the
    /// `https://leaflet.pub/p/{did}` site fallback for unlinked drafts.
    pub did: Option<&'a str>,
    /// `publication_tags` from the pull extras, when the draft sits in a
    /// publication.
    pub publication_tags: Option<Vec<String>>,
}

/// A completed extraction.
#[derive(Debug, Clone)]
pub struct Extracted {
    /// The pure canonical record JSON (`$type` discriminators re-injected).
    pub record: serde_json::Value,
    pub slug: String,
    pub title: String,
    pub warns: Vec<Warn>,
    /// Block records emitted across all pages (list wrappers included).
    pub blocks_converted: usize,
    /// Block entities dropped (each with a warn).
    pub blocks_skipped: usize,
}

// ---------------------------------------------------------------------------
// The flat block model (getBlocks.ts assembleBlocks)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct ListData {
    depth: u32,
    #[allow(dead_code)]
    parent: String,
    checklist: bool,
    checked: Option<bool>,
    list_style: Option<String>,
    list_start: Option<f64>,
}

#[derive(Debug, Clone)]
struct FlatBlock {
    entity: String,
    block_type: String,
    list: Option<ListData>,
}

/// One parsed list node (parseBlocksToList's `List`).
#[derive(Debug, Clone)]
struct ListNode {
    block: FlatBlock,
    depth: u32,
    children: Vec<ListNode>,
}

#[derive(Debug, Clone)]
enum ParsedNode {
    Block(FlatBlock),
    List { children: Vec<ListNode> },
}

// ---------------------------------------------------------------------------
// Converter
// ---------------------------------------------------------------------------

pub(crate) struct Converter {
    scan: ScanIndex,
    root: String,
    warns: Vec<Warn>,
    pages: Vec<LinearDocument<DefaultStr>>,
    /// Page entities on the current recursion stack (cycle guard).
    visiting: HashSet<String>,
    visited_pages: HashSet<String>,
    text_cache: HashMap<String, DecodedText>,
    blocks_converted: usize,
    blocks_skipped: usize,
}

impl Converter {
    fn warn(&mut self, block_type: &str, entity: &str, reason: impl Into<String>) {
        self.warns.push(Warn {
            block_type: block_type.to_string(),
            entity: entity.to_string(),
            reason: reason.into(),
        });
    }

    fn skip(&mut self, block_type: &str, entity: &str, reason: impl Into<String>) {
        self.blocks_skipped += 1;
        self.warn(block_type, entity, reason);
    }

    /// The spine walk (`assembleBlocks`): a parent entity's `card/block`
    /// facts in `(position, fact id)` order; list heads expand recursively
    /// by depth, interleaving children in document order.
    fn flat_blocks(&mut self, parent: &str) -> Result<Vec<FlatBlock>, Error> {
        // Owned copies first: the warn path below borrows self mutably.
        /// (fact id, entity, (target, position) when reference-shaped)
        type CardFact = (String, String, Option<(String, String)>);
        let card_facts: Vec<CardFact> = self
            .scan
            .eav(parent, "card/block")
            .iter()
            .map(|fact| {
                let value_position = match &fact.data {
                    FactData::OrderedReference { value, position } => Some((value.clone(), position.clone())),
                    _ => None,
                };
                (fact.id.clone(), fact.entity.clone(), value_position)
            })
            .collect();
        let mut ordered: Vec<(String, String, String)> = Vec::new(); // (fact id, position, target)
        for (fact_id, entity, value_position) in card_facts {
            let Some((value, position)) = value_position else {
                self.warn("card/block", &entity, format!("fact {fact_id} is not an ordered reference"));
                continue;
            };
            if position.is_empty() {
                return Err(Error::Structure(format!("card/block fact {fact_id} on {parent} has no position")));
            }
            ordered.push((fact_id, position, value));
        }
        fractional::sort_by_position(&mut ordered);

        let mut out = Vec::new();
        for (_, _, target) in ordered {
            let Some(block_type) = self.scan.str(&target, "block/type").map(str::to_string) else {
                continue; // no block/type: invisible to publish (and to us)
            };
            let is_list = self.scan.boolean(&target, "block/is-list").unwrap_or(false);
            if is_list {
                self.list_children(&target, parent, 1, &mut out)?;
            } else {
                out.push(FlatBlock {
                    entity: target,
                    block_type,
                    list: None,
                });
            }
        }
        Ok(out)
    }

    /// `getChildren`: the list head itself (depth `depth`, parent
    /// `parent_holder`), then its children at depth + 1.
    fn list_children(&self, entity: &str, parent_holder: &str, depth: u32, out: &mut Vec<FlatBlock>) -> Result<(), Error> {
        let Some(block_type) = self.scan.str(entity, "block/type").map(str::to_string) else {
            return Ok(());
        };
        let checklist = self
            .scan
            .eav(entity, "block/check-list")
            .first()
            .is_some_and(|f| matches!(f.data, FactData::Boolean { value: _ }));
        let checked = match self.scan.one(entity, "block/check-list").map(|f| &f.data) {
            Some(FactData::Boolean { value }) => Some(*value),
            _ => None,
        };
        out.push(FlatBlock {
            entity: entity.to_string(),
            block_type,
            list: Some(ListData {
                depth,
                parent: parent_holder.to_string(),
                checklist,
                checked,
                list_style: self.scan.str(entity, "block/list-style").map(str::to_string),
                list_start: self.scan.number(entity, "block/list-number"),
            }),
        });
        let mut ordered: Vec<(String, String, String)> = Vec::new();
        for fact in self.scan.eav(entity, "card/block") {
            if let FactData::OrderedReference { value, position } = &fact.data {
                if position.is_empty() {
                    return Err(Error::Structure(format!("card/block fact {} on {entity} has no position", fact.id)));
                }
                ordered.push((fact.id.clone(), position.clone(), value.clone()));
            }
        }
        fractional::sort_by_position(&mut ordered);
        for (_, _, target) in ordered {
            self.list_children(&target, entity, depth + 1, out)?;
        }
        Ok(())
    }

    /// Serialize one page (`pageToRecord`). Canvas pages warn and are
    /// omitted (v0); doc pages become `pub.leaflet.pages.linearDocument`.
    fn serialize_page(&mut self, page: &str) -> Result<(), Error> {
        if !self.visiting.insert(page.to_string()) {
            self.warn("page", page, "page recursion cycle — subpage skipped");
            return Ok(());
        }
        let result = self.serialize_page_inner(page);
        self.visiting.remove(page);
        result
    }

    fn serialize_page_inner(&mut self, page: &str) -> Result<(), Error> {
        let page_type = self.scan.str(page, "page/type").unwrap_or("doc");
        if page_type == "canvas" {
            self.warn("page", page, "canvas page serialization deferred (v0) — page omitted");
            return Ok(());
        }
        // Page-level theme (factsToPagesRecord pageThemeToRecord): warn +
        // skip — colors need hsba parsing (v1).
        for attribute in ["theme/card-background", "theme/primary", "theme/accent-background", "theme/accent-text"] {
            if self.scan.one(page, attribute).is_some() {
                self.warn("page", page, "page theme extraction deferred (hsba color parsing is v1) — theme omitted");
                break;
            }
        }
        let flat = self.flat_blocks(page)?;
        let parsed = parse_blocks_to_list(flat);
        // Members-only: the delimiter stays; everything after it is
        // members-only. We keep the full content (the fully-inflated record
        // Leaflet persists locally); the publish-time PDS truncation is a
        // separate, later step.
        let delimiter_entity = parsed.iter().find_map(|node| match node {
            ParsedNode::Block(block) if block.block_type == "members-only-delimiter" => Some(block.entity.clone()),
            _ => None,
        });
        if let Some(entity) = delimiter_entity {
            self.warn(
                "members-only-delimiter",
                &entity,
                "members-only delimiter present: the record keeps full content; a future publish step must truncate the PDS copy",
            );
        }
        let blocks = self.blocks_to_record(parsed);
        if self.visited_pages.insert(page.to_string()) {
            self.pages.push(LinearDocument {
                blocks,
                id: Some(page.into()),
                theme: None,
                extra_data: None,
            });
        } else {
            self.warn("page", page, "page serialized twice (two cards reference it) — second copy dropped");
        }
        Ok(())
    }

    /// `blocksToRecord`: fold parsed blocks, splitting top-level list runs
    /// by style into orderedList/unorderedList records.
    fn blocks_to_record(&mut self, parsed: Vec<ParsedNode>) -> Vec<Block<DefaultStr>> {
        let mut out = Vec::new();
        for node in parsed {
            match node {
                ParsedNode::Block(block) => {
                    let alignment = self.alignment(&block.entity);
                    if let Some(inner) = self.block_to_record(&block) {
                        out.push(Block {
                            alignment,
                            block: inner,
                            extra_data: None,
                        });
                    }
                }
                ParsedNode::List { children, .. } => {
                    for run in split_list_by_style(children) {
                        let inner = if run.style == "ordered" {
                            // `startIndex: listStart || 1` — Leaflet always
                            // writes it, defaulting 1.
                            let start = run
                                .children
                                .first()
                                .and_then(|child| child.block.list.as_ref())
                                .and_then(|list| list.list_start)
                                .filter(|start| *start > 0.0)
                                .unwrap_or(1.0);
                            BlockBlock::OrderedList(Box::new(OrderedList {
                                children: self.ordered_children(&run.children),
                                start_index: Some(js_round(start)),
                                extra_data: None,
                            }))
                        } else {
                            BlockBlock::UnorderedList(Box::new(UnorderedList {
                                children: self.unordered_children(&run.children),
                                extra_data: None,
                            }))
                        };
                        self.blocks_converted += 1;
                        out.push(Block {
                            alignment: None,
                            block: inner,
                            extra_data: None,
                        });
                    }
                }
            }
        }
        out
    }

    fn ordered_children(&mut self, children: &[ListNode]) -> Vec<springleaf_leaflet::pub_leaflet::blocks::ordered_list::ListItem<DefaultStr>> {
        use springleaf_leaflet::pub_leaflet::blocks::ordered_list::ListItem;
        children
            .iter()
            .filter_map(|child| {
                let content = self.list_item_content(&child.block)?;
                let mut item = ListItem {
                    checked: child.block.list.as_ref().filter(|l| l.checklist).and_then(|l| l.checked),
                    children: None,
                    content,
                    unordered_list_children: None,
                    extra_data: None,
                };
                let (same, diff): (Vec<ListNode>, Vec<ListNode>) = child
                    .children
                    .iter()
                    .cloned()
                    // Ordered context: ordered children nest under
                    // `children`; unordered ones under `unorderedListChildren`.
                    .partition(|c| c.block.list.as_ref().and_then(|l| l.list_style.as_deref()) == Some("ordered"));
                if !same.is_empty() {
                    item.children = Some(self.ordered_children(&same));
                }
                if !diff.is_empty() {
                    item.unordered_list_children = Some(UnorderedList {
                        children: self.unordered_children(&diff),
                        extra_data: None,
                    });
                }
                Some(item)
            })
            .collect()
    }

    fn unordered_children(&mut self, children: &[ListNode]) -> Vec<springleaf_leaflet::pub_leaflet::blocks::unordered_list::ListItem<DefaultStr>> {
        use springleaf_leaflet::pub_leaflet::blocks::unordered_list::ListItem;
        use springleaf_leaflet::pub_leaflet::blocks::unordered_list::ListItemContent;
        children
            .iter()
            .filter_map(|child| {
                let content = match self.list_item_content(&child.block)? {
                    OrderedListItemContent::Text(text) => ListItemContent::Text(text),
                    OrderedListItemContent::Header(header) => ListItemContent::Header(header),
                    _ => return None,
                };
                let mut item = ListItem {
                    checked: child.block.list.as_ref().filter(|l| l.checklist).and_then(|l| l.checked),
                    children: None,
                    content,
                    ordered_list_children: None,
                    extra_data: None,
                };
                let (same, diff): (Vec<ListNode>, Vec<ListNode>) = child
                    .children
                    .iter()
                    .cloned()
                    // Unordered context: unordered children nest under
                    // `children`; ordered ones under `orderedListChildren`.
                    .partition(|c| c.block.list.as_ref().and_then(|l| l.list_style.as_deref()) != Some("ordered"));
                if !same.is_empty() {
                    item.children = Some(self.unordered_children(&same));
                }
                if !diff.is_empty() {
                    item.ordered_list_children = Some(OrderedList {
                        children: self.ordered_children(&diff),
                        start_index: None,
                        extra_data: None,
                    });
                }
                Some(item)
            })
            .collect()
    }

    /// A list item's content: only text/header blocks are lexicon-valid
    /// item contents; anything else warns and drops the item.
    fn list_item_content(&mut self, block: &FlatBlock) -> Option<OrderedListItemContent<DefaultStr>> {
        match self.block_to_record(block)? {
            BlockBlock::Text(text) => Some(OrderedListItemContent::Text(text)),
            BlockBlock::Header(header) => Some(OrderedListItemContent::Header(header)),
            other => {
                self.skip(
                    &block.block_type,
                    &block.entity,
                    format!("{} block inside a list is not a valid list item content — item dropped", block.block_type),
                );
                let _ = other;
                None
            }
        }
    }

    /// `blockAlignment`: the lex:-prefixed knownValue strings Leaflet
    /// writes (`factsToPagesRecord.ts:743-756`).
    fn alignment(&self, entity: &str) -> Option<BlockAlignment<DefaultStr>> {
        let value = self.scan.str(entity, "block/text-alignment")?;
        let alignment = match value {
            "left" => "lex:pub.leaflet.pages.linearDocument#textAlignLeft",
            "center" => "lex:pub.leaflet.pages.linearDocument#textAlignCenter",
            "right" => "lex:pub.leaflet.pages.linearDocument#textAlignRight",
            "justify" => "lex:pub.leaflet.pages.linearDocument#textAlignJustify",
            _ => return None,
        };
        Some(BlockAlignment::Other(alignment.into()))
    }

    /// A block's decoded text (tolerant: undecodable values — including the
    /// 3-byte `b5ec6d` non-update — warn once and yield empty, matching
    /// `factsToPagesRecord.ts:163-164`'s missing-fact behavior).
    fn block_text(&mut self, entity: &str) -> DecodedText {
        if let Some(cached) = self.text_cache.get(entity) {
            return cached.clone();
        }
        let decoded = match self.scan.str(entity, "block/text") {
            None => DecodedText::default(),
            Some(value) => match yjs::decode_block_text(value) {
                Ok(decoded) => decoded,
                Err(err) => {
                    self.warn("text", entity, format!("block/text value undecodable ({err}) — emitted as empty text"));
                    DecodedText::default()
                }
            },
        };
        for note in &decoded.notes {
            self.warn("text", entity, format!("richtext: {note}"));
        }
        if decoded.paragraph_count > 1 {
            self.warn(
                "text",
                entity,
                format!(
                    "{} paragraphs in one block: only the first is emitted (Leaflet publish parity)",
                    decoded.paragraph_count
                ),
            );
        }
        self.text_cache.insert(entity.to_string(), decoded.clone());
        decoded
    }

    fn text_facets(&mut self, decoded: &DecodedText) -> Option<Vec<Facet<DefaultStr>>> {
        if decoded.facets.is_empty() {
            return None;
        }
        Some(
            decoded
                .facets
                .iter()
                .map(|run| Facet {
                    features: run
                        .marks
                        .iter()
                        .map(|mark| match mark {
                            Mark::Strikethrough => FacetFeaturesItem::Strikethrough(Box::new(Strikethrough::default())),
                            Mark::Code => FacetFeaturesItem::Code(Box::new(CodeFacet::default())),
                            Mark::Highlight => FacetFeaturesItem::Highlight(Box::new(Highlight::default())),
                            Mark::Underline => FacetFeaturesItem::Underline(Box::new(Underline::default())),
                            Mark::Bold => FacetFeaturesItem::Bold(Box::new(Bold::default())),
                            Mark::Italic => FacetFeaturesItem::Italic(Box::new(Italic::default())),
                            Mark::Link(uri) => FacetFeaturesItem::Link(Box::new(Link {
                                uri: uri.clone().into(),
                                extra_data: None,
                            })),
                        })
                        .collect(),
                    index: ByteSlice {
                        byte_start: run.byte_start as i64,
                        byte_end: run.byte_end as i64,
                        extra_data: None,
                    },
                    extra_data: None,
                })
                .collect(),
        )
    }

    /// The coverage matrix (`blockTypeToRecord`). Returns the inner block
    /// variant; `None` = dropped (always with a warn here, where Leaflet's
    /// publish drops silently).
    fn block_to_record(&mut self, block: &FlatBlock) -> Option<BlockBlock<DefaultStr>> {
        let entity = block.entity.clone();
        let block_type = block.block_type.clone();
        let converted: Option<BlockBlock<DefaultStr>> = match block_type.as_str() {
            // ── direct ──────────────────────────────────────────────────
            "text" => {
                let decoded = self.block_text(&entity);
                let facets = self.text_facets(&decoded);
                Some(BlockBlock::Text(Box::new(Text {
                    facets,
                    plaintext: decoded.plaintext.into(),
                    text_size: known_union(self, &entity, "block/text-size", &["default", "small", "large"]),
                    extra_data: None,
                })))
            }
            "heading" => {
                let decoded = self.block_text(&entity);
                let facets = self.text_facets(&decoded);
                let raw = self.scan.number(&entity, "block/heading-level").unwrap_or(1.0);
                let level = js_round(raw.floor()).clamp(1, 6);
                if !(1.0..=6.0).contains(&raw.floor()) {
                    self.warn(
                        &block_type,
                        &entity,
                        format!("heading level {} out of the 1..=6 lexicon range — clamped to {level}", raw.floor()),
                    );
                }
                Some(BlockBlock::Header(Box::new(Header {
                    facets,
                    level: Some(level),
                    plaintext: decoded.plaintext.into(),
                    extra_data: None,
                })))
            }
            "blockquote" => {
                let decoded = self.block_text(&entity);
                let facets = self.text_facets(&decoded);
                Some(BlockBlock::Blockquote(Box::new(Blockquote {
                    facets,
                    plaintext: decoded.plaintext.into(),
                    extra_data: None,
                })))
            }
            "math" => Some(BlockBlock::Math(Box::new(Math {
                tex: self.scan.str(&entity, "block/math").unwrap_or_default().into(),
                extra_data: None,
            }))),
            "code" => {
                let theme = self.scan.str(&self.root, "theme/code-theme").map(str::to_string);
                Some(BlockBlock::Code(Box::new(Code {
                    language: self.scan.str(&entity, "block/code-language").map(Into::into),
                    plaintext: self.scan.str(&entity, "block/code").unwrap_or_default().into(),
                    syntax_highlighting_theme: theme.map(Into::into),
                    extra_data: None,
                })))
            }
            "horizontal-rule" => Some(BlockBlock::HorizontalRule(Box::new(HorizontalRule::default()))),
            "signup" => Some(BlockBlock::Signup(Box::new(Signup::default()))),
            "post-header" => Some(BlockBlock::PostHeader(Box::new(PostHeader {
                compact: (self.scan.boolean(&entity, "post-header/compact") == Some(true)).then_some(true),
                extra_data: None,
            }))),
            "button" => match (self.scan.str(&entity, "button/text"), self.scan.str(&entity, "button/url")) {
                (Some(text), Some(url)) => match UriValue::new_owned(url) {
                    Ok(url) => Some(BlockBlock::Button(Box::new(Button {
                        text: text.into(),
                        url,
                        extra_data: None,
                    }))),
                    Err(err) => {
                        self.skip(&block_type, &entity, format!("button/url {url:?} is not a valid URI ({err})"));
                        None
                    }
                },
                _ => {
                    self.skip(&block_type, &entity, "button needs both button/text and button/url");
                    None
                }
            },
            "bluesky-post" => {
                let post = self.scan.one(&entity, "block/bluesky-post").and_then(|fact| match &fact.data {
                    FactData::BlueskyPost { value } => Some(value.clone()),
                    _ => None,
                });
                let uri = post.as_ref().and_then(|p| p.get("post").and_then(|p| p.get("uri")).and_then(|v| v.as_str()));
                let cid = post.as_ref().and_then(|p| p.get("post").and_then(|p| p.get("cid")).and_then(|v| v.as_str()));
                match (uri, cid) {
                    (Some(uri), Some(cid)) => match (AtUri::new_owned(uri), Cid::<DefaultStr>::new_owned(cid.as_bytes())) {
                        (Ok(uri), Ok(cid)) => Some(BlockBlock::BskyPost(Box::new(BskyPost {
                            client_host: self.scan.str(&entity, "bluesky-post/host").map(Into::into),
                            post_ref: springleaf_leaflet::com_atproto::repo::strong_ref::StrongRef { cid, uri, extra_data: None },
                            view: (self.scan.str(&entity, "bluesky-post/view") == Some("media")).then(|| BskyPostView::Other("media".into())),
                            extra_data: None,
                        }))),
                        _ => {
                            self.skip(&block_type, &entity, "bluesky post uri/cid does not parse as an at-uri/cid");
                            None
                        }
                    },
                    _ => {
                        self.skip(&block_type, &entity, "bluesky-post fact carries no embedded post");
                        None
                    }
                }
            }
            "standard-site-post" => {
                let uri = self.scan.str(&entity, "block/standard-site-post");
                match uri.map(AtUri::new_owned) {
                    Some(Ok(uri)) => Some(BlockBlock::StandardSitePost(Box::new(StandardSitePost {
                        cid: None,
                        show_publication_theme: (self.scan.boolean(&entity, "standard-site-post/show-publication-theme") == Some(false)).then_some(false),
                        size: known_union(self, &entity, "standard-site-post/size", &["large", "medium", "small"]).map(StandardSitePostSize::from_value),
                        uri,
                        extra_data: None,
                    }))),
                    Some(Err(err)) => {
                        self.skip(&block_type, &entity, format!("standard-site-post uri invalid ({err})"));
                        None
                    }
                    None => {
                        self.skip(&block_type, &entity, "no block/standard-site-post uri");
                        None
                    }
                }
            }
            "standard-site-publication" => {
                let uri = self.scan.str(&entity, "block/standard-site-publication");
                match uri.map(AtUri::new_owned) {
                    Some(Ok(uri)) => Some(BlockBlock::StandardSitePublication(Box::new(StandardSitePublication {
                        cid: None,
                        show_publication_theme: (self.scan.boolean(&entity, "standard-site-publication/show-publication-theme") == Some(false))
                            .then_some(false),
                        uri,
                        extra_data: None,
                    }))),
                    Some(Err(err)) => {
                        self.skip(&block_type, &entity, format!("standard-site-publication uri invalid ({err})"));
                        None
                    }
                    None => {
                        self.skip(&block_type, &entity, "no block/standard-site-publication uri");
                        None
                    }
                }
            }
            "recommended-pubs" => Some(BlockBlock::RecommendedPubs(Box::new(RecommendedPubs {
                compact: self.scan.boolean(&entity, "recommended-pubs/compact"),
                extra_data: None,
            }))),
            "reply" => Some(BlockBlock::Reply(Box::new(Reply {
                button_text: self.scan.str(&entity, "reply/button-text").filter(|s| !s.is_empty()).map(Into::into),
                key: Some(entity.clone().into()),
                prompt_text: self.scan.str(&entity, "reply/prompt-text").filter(|s| !s.is_empty()).map(Into::into),
                show_publication_theme: (self.scan.boolean(&entity, "reply/show-publication-theme") == Some(false)).then_some(false),
                extra_data: None,
            }))),
            "questions" => {
                let audience = match self.scan.str(&entity, "questions/audience") {
                    Some(value @ ("follows" | "followers")) => Some(QuestionsAudience::Other(value.into())),
                    _ => None,
                };
                Some(BlockBlock::Questions(Box::new(Questions {
                    audience,
                    button_text: self.scan.str(&entity, "questions/button-text").filter(|s| !s.is_empty()).map(Into::into),
                    key: Some(entity.clone().into()),
                    show_pending: (self.scan.boolean(&entity, "questions/show-pending") == Some(true)).then_some(true),
                    extra_data: None,
                })))
            }
            // ── convert ─────────────────────────────────────────────────
            "members-only-delimiter" => {
                let mut tier_ids: Vec<String> = self
                    .scan
                    .eav(&entity, "block/members-only-tier")
                    .iter()
                    .filter_map(|fact| fact.data.str_value().map(str::to_string))
                    .collect();
                tier_ids.sort();
                let explicit = self.scan.str(&entity, "block/members-only-audience");
                let audience = match explicit {
                    Some(value @ ("subscribers" | "paid" | "tiers")) => value,
                    _ if !tier_ids.is_empty() => "tiers",
                    _ => "paid",
                };
                Some(BlockBlock::MembersOnlyDelimiter(Box::new(MembersOnlyDelimiter {
                    audience: MembersOnlyDelimiterAudience::Other(audience.into()),
                    tier_ids: (audience == "tiers" && !tier_ids.is_empty()).then_some(tier_ids.iter().map(|t| t.clone().into()).collect()),
                    extra_data: None,
                })))
            }
            "posts-list" => {
                let mut posts_list = PostsList {
                    filter_by_tags: (!self
                        .scan
                        .eav(&entity, "posts-list/filter-tag")
                        .iter()
                        .filter_map(|fact| fact.data.str_value())
                        .collect::<Vec<_>>()
                        .is_empty())
                    .then(|| {
                        self.scan
                            .eav(&entity, "posts-list/filter-tag")
                            .iter()
                            .filter_map(|fact| fact.data.str_value().map(Into::into))
                            .collect()
                    }),
                    highlight_first_post: self.scan.boolean(&entity, "posts-list/highlight-first-post"),
                    limit: self.scan.number(&entity, "posts-list/limit").filter(|limit| *limit > 0.0).map(js_round),
                    reader_controls: None,
                    reader_search: None,
                    reader_sort: None,
                    reader_tag_filter: None,
                    show_page_count: self.scan.boolean(&entity, "posts-list/show-page-count"),
                    view: known_union(self, &entity, "posts-list/view", &["small", "medium", "chapter"]).map(PostsListView::from_value),
                    extra_data: None,
                };
                if self.scan.boolean(&entity, "posts-list/reader-controls") == Some(true) {
                    posts_list.reader_controls = Some(true);
                    posts_list.reader_search = Some(self.scan.boolean(&entity, "posts-list/reader-search").unwrap_or(true));
                    posts_list.reader_tag_filter = Some(self.scan.boolean(&entity, "posts-list/reader-tag-filter").unwrap_or(true));
                    posts_list.reader_sort = Some(self.scan.boolean(&entity, "posts-list/reader-sort").unwrap_or(true));
                }
                Some(BlockBlock::PostsList(Box::new(posts_list)))
            }
            "link" => {
                let src = self.scan.str(&entity, "link/url");
                match src.map(UriValue::new_owned) {
                    Some(Ok(src)) => {
                        if self.scan.one(&entity, "link/preview").is_some() {
                            self.warn(&block_type, &entity, "link/preview image needs a PDS upload (read-only) — previewImage omitted");
                        }
                        Some(BlockBlock::Website(Box::new(Website {
                            description: self.scan.str(&entity, "link/description").map(Into::into),
                            preview_image: None,
                            src,
                            title: self.scan.str(&entity, "link/title").map(Into::into),
                            extra_data: None,
                        })))
                    }
                    Some(Err(err)) => {
                        self.skip(&block_type, &entity, format!("link/url invalid ({err})"));
                        None
                    }
                    None => {
                        self.skip(&block_type, &entity, "no link/url");
                        None
                    }
                }
            }
            "embed" | "html" => {
                let url = self.scan.str(&entity, "embed/url");
                let html = self.scan.str(&entity, "embed/html");
                if url.is_none() && html.is_none() {
                    self.skip(&block_type, &entity, "embed needs embed/url or (legacy) embed/html");
                    return None;
                }
                let height = self
                    .scan
                    .number(&entity, "embed/height")
                    .filter(|height| !height.is_nan() && *height != 0.0)
                    .unwrap_or(600.0)
                    .floor() as i64; // Math.floor(height || 600)
                let aspect_ratio = self.aspect_ratio(&entity);
                if html.is_some() {
                    Some(BlockBlock::Html(Box::new(HtmlBlock {
                        aspect_ratio: aspect_ratio.map(|(width, height)| springleaf_leaflet::pub_leaflet::blocks::html::AspectRatio {
                            width,
                            height,
                            extra_data: None,
                        }),
                        height: Some(height),
                        html: html.unwrap_or_default().into(),
                        extra_data: None,
                    })))
                } else {
                    match UriValue::new_owned(url.unwrap_or_default()) {
                        Ok(url) => Some(BlockBlock::Iframe(Box::new(Iframe {
                            aspect_ratio: aspect_ratio.map(|(width, height)| springleaf_leaflet::pub_leaflet::blocks::iframe::AspectRatio {
                                width,
                                height,
                                extra_data: None,
                            }),
                            height: Some(height),
                            html: None,
                            url: Some(url),
                            extra_data: None,
                        }))),
                        Err(err) => {
                            self.skip(&block_type, &entity, format!("embed/url invalid ({err})"));
                            None
                        }
                    }
                }
            }
            "card" | "embedded-canvas" => self.subpage_block(block),
            "drawing" => self.drawing_block(block),
            // ── draft-only / canvas-only / upload-dependent: skip + warn ─
            "datetime" => None.drop_with(self, &block_type, &entity, "draft-only block type: Leaflet's publish drops it"),
            "rsvp" => None.drop_with(self, &block_type, &entity, "draft-only block type: Leaflet's publish drops it"),
            "mailbox" => None.drop_with(self, &block_type, &entity, "draft-only block type: Leaflet's publish drops it"),
            "group" => None.drop_with(self, &block_type, &entity, "canvas-only block type: invisible in linear flow"),
            "image" => None.drop_with(self, &block_type, &entity, "image blocks need a PDS blob upload — skipped in read-only v0"),
            "image-gallery" => None.drop_with(
                self,
                &block_type,
                &entity,
                "image galleries need per-image PDS uploads — skipped in read-only v0",
            ),
            "poll" => None.drop_with(
                self,
                &block_type,
                &entity,
                "poll blocks need a pub.leaflet.poll.definition record write — skipped in read-only v0",
            ),
            other => None.drop_with(
                self,
                &block_type,
                &entity,
                format!("unknown block type {other:?} — dropped like Leaflet's own-property check"),
            ),
        };
        if converted.is_some() {
            self.blocks_converted += 1;
        }
        converted
    }

    /// `embed/aspect-ratio` `"w/h"` string → (width, height).
    fn aspect_ratio(&self, entity: &str) -> Option<(i64, i64)> {
        let value = self.scan.str(entity, "embed/aspect-ratio")?;
        let (w, h) = value.split_once('/')?;
        let width: f64 = w.trim().parse().ok()?;
        let height: f64 = h.trim().parse().ok()?;
        (width > 0.0 && height > 0.0).then_some((js_round(width), js_round(height)))
    }

    /// `card` / `embedded-canvas`: serialize the referenced subpage and
    /// append it to `pages[]`, emitting the linking block. Canvas
    /// subpages (the normal `embedded-canvas` target) warn and skip in v0.
    fn subpage_block(&mut self, block: &FlatBlock) -> Option<BlockBlock<DefaultStr>> {
        let entity = &block.entity;
        let page = self.scan.reference(entity, "block/card").map(str::to_string);
        let Some(page) = page else {
            self.skip(&block.block_type, entity, "no block/card page reference");
            return None;
        };
        if self.scan.str(&page, "page/type").unwrap_or("doc") == "canvas" {
            self.skip(
                &block.block_type,
                entity,
                "referenced subpage is a canvas page (serialization deferred in v0) — block and subpage skipped",
            );
            return None;
        }
        if self.visiting.contains(&page) || self.visited_pages.contains(&page) {
            self.skip(
                &block.block_type,
                entity,
                "subpage recursion/two-card reference — skipped to keep the record acyclic",
            );
            return None;
        }
        self.serialize_page(&page).ok()?;
        if block.block_type == "card" {
            let mut page_block = PageBlock {
                display: None,
                id: page.clone().into(),
                extra_data: None,
            };
            if let Some(display @ "compact") = self.scan.str(entity, "page-link/display") {
                page_block.display = Some(PageDisplay::Other(display.into()));
            }
            Some(BlockBlock::Page(Box::new(page_block)))
        } else {
            let alt = self.scan.str(entity, "image/alt").filter(|alt| !alt.is_empty());
            Some(BlockBlock::EmbeddedCanvas(Box::new(EmbeddedCanvas {
                alt: alt.map(Into::into),
                id: page.clone().into(),
                extra_data: None,
            })))
        }
    }

    /// `drawing`: view-box + strokes sorted by fact id; fills derived by
    /// descending polygon area (`drawingFills`).
    fn drawing_block(&mut self, block: &FlatBlock) -> Option<BlockBlock<DefaultStr>> {
        let entity = &block.entity;
        let view_box = match self.scan.one(entity, "drawing/view-box").map(|fact| &fact.data) {
            Some(FactData::ViewBox { value }) => *value,
            _ => {
                self.skip(&block.block_type, entity, "drawing without drawing/view-box");
                return None;
            }
        };
        let mut strokes: Vec<(String, InkStroke)> = self
            .scan
            .eav(entity, "drawing/stroke")
            .iter()
            .filter_map(|fact| match &fact.data {
                FactData::InkStroke { value } => Some((fact.id.clone(), value.clone())),
                _ => None,
            })
            .collect();
        if strokes.is_empty() {
            self.skip(&block.block_type, entity, "drawing without strokes");
            return None;
        }
        strokes.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut fills: Vec<(f64, DrawingFill<DefaultStr>)> = Vec::new();
        let mut stroke_records = Vec::with_capacity(strokes.len());
        for (_, stroke) in &strokes {
            if let Some(fill) = &stroke.fill {
                fills.push((
                    polygon_area(&fill.points),
                    DrawingFill {
                        color: fill.color.clone().into(),
                        points: fill.points.iter().map(|p| js_round(*p)).collect(),
                        extra_data: None,
                    },
                ));
            }
            stroke_records.push(DrawingStroke {
                color: stroke.color.clone().into(),
                points: stroke.points.iter().map(|p| js_round(*p)).collect(),
                simulate_pressure: stroke.simulate_pressure.filter(|p| *p),
                size: js_round(stroke.size).max(1),
                extra_data: None,
            });
        }
        fills.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let record = DrawingBlock {
            fills: (!fills.is_empty()).then(|| fills.into_iter().map(|(_, fill)| fill).collect()),
            strokes: stroke_records,
            view_box: DrawingViewBox {
                height: js_round(view_box.height).max(1),
                width: js_round(view_box.width).max(1),
                x: js_round(view_box.x),
                y: js_round(view_box.y),
                extra_data: None,
            },
            extra_data: None,
        };
        Some(BlockBlock::Drawing(Box::new(record)))
    }
}

/// Helper for the skip-arm pattern above: `None.drop_with(converter, ..)`
/// records the warn and returns `None`.
trait DropWith {
    fn drop_with(self, converter: &mut Converter, block_type: &str, entity: &str, reason: impl Into<String>) -> Self;
}

impl<T> DropWith for Option<T> {
    fn drop_with(self, converter: &mut Converter, block_type: &str, entity: &str, reason: impl Into<String>) -> Self {
        converter.skip(block_type, entity, reason);
        None
    }
}

/// JS `Math.round`: half rounds toward +∞ (Rust's `f64::round` rounds half
/// away from zero — different on negatives).
fn js_round(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}

/// Shoelace area over flattened x/y pairs with wraparound
/// (`components/Blocks/DrawingBlock/ink.ts polygonArea`).
fn polygon_area(points: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut i = 0;
    while i + 1 < points.len() {
        let j = (i + 2) % points.len();
        sum += points[i] * points[j + 1] - points[j] * points[i + 1];
        i += 2;
    }
    sum.abs() / 2.0
}

/// A union-typed fact read that only emits known lexicon values (unknown
/// values warn and omit the field — a garbage knownValue would produce an
/// invalid record).
fn known_union(converter: &mut Converter, entity: &str, attribute: &str, known: &[&str]) -> Option<DefaultStr> {
    let value = converter.scan.str(entity, attribute)?;
    if known.contains(&value) {
        Some(value.into())
    } else {
        converter.warn(attribute, entity, format!("union value {value:?} not in {known:?} — field omitted"));
        None
    }
}

// ---------------------------------------------------------------------------
// parseBlocksToList + splitListByStyle ports
// ---------------------------------------------------------------------------

fn parse_blocks_to_list(blocks: Vec<FlatBlock>) -> Vec<ParsedNode> {
    let mut parsed: Vec<ParsedNode> = Vec::new();
    for block in blocks {
        let Some(list) = &block.list else {
            parsed.push(ParsedNode::Block(block));
            continue;
        };
        let depth = list.depth;
        // A new top-level list node starts when the previous node is not a
        // list or is deeper than us (`parseBlocksToList`'s guard).
        let joins_previous = match parsed.last() {
            Some(ParsedNode::List { children, .. }) => children.first().is_some_and(|first| first.depth <= depth),
            _ => false,
        };
        if !joins_previous {
            parsed.push(ParsedNode::List {
                children: vec![ListNode {
                    block,
                    depth,
                    children: Vec::new(),
                }],
            });
            continue;
        }
        let Some(ParsedNode::List { children, .. }) = parsed.last_mut() else {
            unreachable!("joins_previous covers the non-list case");
        };
        if children.first().is_some_and(|first| first.depth == depth) {
            // Sibling of the list's own children.
            children.push(ListNode {
                block,
                depth,
                children: Vec::new(),
            });
        } else {
            // Deeper: descend from the last child while its own last child
            // is still shallower than us (`depth - 1` steps down).
            let mut parent: &mut ListNode = children.last_mut().expect("list node has children");
            let mut remaining = depth;
            while remaining > 1 {
                let descend = parent.children.last().is_some_and(|last| last.depth < depth);
                if !descend {
                    break;
                }
                parent = parent.children.last_mut().expect("descend checked");
                remaining -= 1;
            }
            parent.children.push(ListNode {
                block,
                depth,
                children: Vec::new(),
            });
        }
    }
    parsed
}

fn split_list_by_style(children: Vec<ListNode>) -> Vec<ListRun> {
    let mut runs: Vec<ListRun> = Vec::new();
    for child in children {
        let style = child.block.list.as_ref().and_then(|list| list.list_style.as_deref()).unwrap_or("unordered");
        let style = if style == "ordered" { "ordered" } else { "unordered" };
        match runs.last_mut() {
            Some(run) if run.style == style => run.children.push(child),
            _ => runs.push(ListRun { style, children: vec![child] }),
        }
    }
    runs
}

struct ListRun {
    style: &'static str,
    children: Vec<ListNode>,
}

// ---------------------------------------------------------------------------
// Envelope + entry point
// ---------------------------------------------------------------------------

/// Extract the canonical record from a pulled fact graph.
pub fn extract(facts: Vec<FactRow>, ctx: &ExtractContext, options: &ExtractOptions) -> Result<Extracted, Error> {
    let scan = ScanIndex::from_facts(facts);
    let root = ctx
        .data
        .root_entity
        .clone()
        .ok_or_else(|| Error::Structure("get_leaflet_data carries no root_entity".to_string()))?;

    let mut converter = Converter {
        scan,
        root: root.clone(),
        warns: Vec::new(),
        pages: Vec::new(),
        visiting: HashSet::new(),
        visited_pages: HashSet::new(),
        text_cache: HashMap::new(),
        blocks_converted: 0,
        blocks_skipped: 0,
    };

    // Upload-dependent envelope fields: warn + skip.
    if converter.scan.one(&root, "root/cover-image").is_some() {
        converter.warn("root", &root, "cover image needs a PDS upload — coverImage omitted");
    }
    for attribute in [
        "theme/page-background",
        "theme/card-background",
        "theme/primary",
        "theme/accent-background",
        "theme/accent-text",
        "theme/card-border-hidden",
        "theme/background-image",
        "theme/wordmark-image",
        "theme/page-width",
        "theme/heading-font",
        "theme/body-font",
    ] {
        if converter.scan.one(&root, attribute).is_some() {
            converter.warn("root", &root, "document theme extraction deferred — theme omitted");
            break;
        }
    }

    // Root pages in fractional order; >1 warns (publication nav machinery).
    let mut root_pages: Vec<(String, String, String)> = converter
        .scan
        .eav(&root, "root/page")
        .iter()
        .filter_map(|fact| match &fact.data {
            FactData::OrderedReference { value, position } => Some((fact.id.clone(), position.clone(), value.clone())),
            _ => None,
        })
        .collect();
    if root_pages.is_empty() {
        return Err(Error::Structure(format!("no root/page fact on root entity {root}")));
    }
    if root_pages.len() > 1 {
        converter.warn(
            "root",
            &root,
            format!(
                "{} root/page facts (publication nav machinery?) — the first by position is exported",
                root_pages.len()
            ),
        );
    }
    for (fact_id, position, _) in &root_pages {
        if position.is_empty() {
            return Err(Error::Structure(format!("root/page fact {fact_id} has no position")));
        }
    }
    fractional::sort_by_position(&mut root_pages);
    let start_page = root_pages[0].2.clone();
    converter.serialize_page(&start_page)?;
    // The start page serializes first but its card subpages append during
    // its block conversion — unshift it to the front (pages.unshift parity).
    if let Some(position) = converter.pages.iter().position(|page| page.id.as_deref() == Some(start_page.as_str())) {
        let start = converter.pages.remove(position);
        converter.pages.insert(0, start);
    }

    // Title: first text/heading block of the start page's spine
    // (extractTitleFromFacts rule), else the permission-token title, else
    // "Untitled".
    let flat = converter.flat_blocks(&start_page)?;
    let mut title = flat
        .iter()
        .find(|block| block.block_type == "text" || block.block_type == "heading")
        .map(|block| converter.block_text(&block.entity).plaintext)
        .unwrap_or_default();
    if title.is_empty() {
        title = ctx.data.title.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| "Untitled".to_string());
    }
    let slug = match &options.slug {
        Some(slug) => slug.clone(),
        None => slugify(&title),
    };

    // textContent: the builder's rule — text/header plaintexts joined with
    // newlines, empties dropped (top-level blocks of every page).
    let text_content: Vec<String> = converter
        .pages
        .iter()
        .flat_map(|page| page.blocks.iter())
        .filter_map(|block| match &block.block {
            BlockBlock::Text(text) => Some(text.plaintext.as_str().to_string()),
            BlockBlock::Header(header) => Some(header.plaintext.as_str().to_string()),
            _ => None,
        })
        .filter(|text| !text.is_empty())
        .collect();

    // Site: the publication at-uri when linked, else the standalone
    // https://leaflet.pub/p/{did} form (publishLeaflet.ts:418-419).
    let classified = crate::classify(&ctx.data.leaflets_in_publications);
    let site = match classified.publication.as_ref().and_then(|p| p.uri.clone()) {
        Some(uri) => uri,
        None => match ctx.did {
            Some(did) => format!("https://leaflet.pub/p/{did}"),
            None => {
                return Err(Error::Structure(
                    "no site source: the draft has no publication linkage and no DID was resolved for the standalone fallback".into(),
                ));
            }
        },
    };
    let site = UriValue::new_owned(&site).map_err(|err| Error::Structure(format!("site {site:?} is not a valid URI: {err}")))?;

    // publishedAt: linked published document's date, else now.
    let published_at = match &options.published_at {
        Some(fixed) => Some(fixed.clone()),
        None => classified.published_at.clone(),
    };
    let published_at = match published_at {
        Some(value) => match Datetime::from_str(value.as_str()) {
            Ok(datetime) => datetime,
            Err(err) => {
                converter.warn("root", &root, format!("publishedAt {value:?} does not parse ({err}) — using now"));
                Datetime::now()
            }
        },
        None => Datetime::now(),
    };

    let document = Document::<DefaultStr> {
        bsky_post_ref: None,
        content: Some(Content {
            blob_pages: None,
            blobs: None,
            pages: converter.pages.drain(..).map(|page| ContentPagesItem::LinearDocument(Box::new(page))).collect(),
            extra_data: None,
        }),
        contributors: None,
        cover_image: None,
        description: ctx.data.description.clone().filter(|d| !d.is_empty()).map(Into::into),
        path: Some(format!("/{slug}").into()),
        preferences: None,
        published_at,
        site,
        tags: ctx
            .publication_tags
            .clone()
            .filter(|tags| !tags.is_empty())
            .map(|tags| tags.into_iter().map(Into::into).collect()),
        text_content: Some(text_content.join("\n").into()),
        theme: None,
        title: title.clone().into(),
        updated_at: None,
        extra_data: None,
    };
    let mut record = serde_json::to_value(&document).map_err(|err| Error::Structure(format!("record serialization failed: {err}")))?;
    springleaf_leaflet::reinject_type_discriminators(&mut record);

    Ok(Extracted {
        record,
        slug,
        title,
        warns: converter.warns,
        blocks_converted: converter.blocks_converted,
        blocks_skipped: converter.blocks_skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pull::parse_pull_snapshot;
    use base64::Engine as _;
    use serde_json::{Value, json};

    const ROOT: &str = "01999000-0000-7aaa-8000-000000000002";
    const PAGE: &str = "01999000-0000-7aaa-8000-000000000003";

    fn leaflet_data(root: &str) -> crate::LeafletData {
        serde_json::from_value(json!({
            "id": "01999000-0000-7aaa-8000-000000000001",
            "root_entity": root,
            "title": null,
            "description": null,
            "permission_token_rights": [],
            "leaflets_in_publications": []
        }))
        .unwrap()
    }

    fn ctx<'a>(data: &'a crate::LeafletData) -> ExtractContext<'a> {
        ExtractContext {
            data,
            did: Some("did:plc:aaaaaaaaaaaaaaaaaaaaaaaa"),
            publication_tags: None,
        }
    }

    fn extract_with(facts: Vec<FactRow>, data: &crate::LeafletData, options: &ExtractOptions) -> Extracted {
        extract(facts, &ctx(data), options).expect("extraction succeeds")
    }

    fn fact(id: &str, entity: &str, attribute: &str, data: Value) -> FactRow {
        FactRow {
            id: id.into(),
            entity: entity.into(),
            attribute: attribute.into(),
            data: crate::facts::FactData::parse(&data),
        }
    }

    /// Test-local `createYjsText` encoder (port of
    /// experiments/02's yjs_encode_text.py, fixed client) — byte-parity
    /// with the fixture sample is asserted below.
    fn yjs_b64(text: &str) -> String {
        let client: u64 = 979_896_792;
        let mut bytes: Vec<u8> = Vec::new();
        let varuint = |bytes: &mut Vec<u8>, mut n: u64| {
            while n > 0x7f {
                bytes.push((n & 0x7f) as u8 | 0x80);
                n >>= 7;
            }
            bytes.push(n as u8);
        };
        let varstring = |bytes: &mut Vec<u8>, s: &str| {
            varuint(bytes, s.len() as u64);
            bytes.extend_from_slice(s.as_bytes());
        };
        bytes.push(1);
        bytes.push(if text.is_empty() { 2 } else { 3 });
        varuint(&mut bytes, client);
        varuint(&mut bytes, 0);
        bytes.push(0x07);
        bytes.push(1);
        varstring(&mut bytes, "prosemirror");
        varuint(&mut bytes, 3);
        varstring(&mut bytes, "paragraph");
        bytes.push(0x07);
        bytes.push(0);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 0);
        varuint(&mut bytes, 6);
        if !text.is_empty() {
            bytes.push(0x04);
            bytes.push(0);
            varuint(&mut bytes, client);
            varuint(&mut bytes, 1);
            varstring(&mut bytes, text);
        }
        bytes.push(0);
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    #[test]
    fn test_encoder_matches_the_committed_python_encoder() {
        // The hex fixtures/yjs-samples.md documents for "Hello world":
        // 0103d893a0d30300... (63 bytes).
        assert_eq!(
            yjs_b64("Hello world"),
            "AQPYk6DTAwAHAQtwcm9zZW1pcnJvcgMJcGFyYWdyYXBoBwDYk6DTAwAGBADYk6DTAwELSGVsbG8gd29ybGQA"
        );
    }

    fn minimal_doc_facts() -> Vec<FactRow> {
        vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a0"})),
            fact("f02", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact(
                "f03",
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": "e-heading", "position": "a0"}),
            ),
            fact("f04", "e-heading", "block/type", json!({"type": "block-type-union", "value": "heading"})),
            fact("f05", "e-heading", "block/heading-level", json!({"type": "number", "value": 1})),
            fact("f06", "e-heading", "block/text", json!({"type": "text", "value": yjs_b64("Synthetic title")})),
            fact(
                "f07",
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": "e-text", "position": "a1"}),
            ),
            fact("f08", "e-text", "block/type", json!({"type": "block-type-union", "value": "text"})),
            fact("f09", "e-text", "block/text", json!({"type": "text", "value": yjs_b64("Synthetic body copy")})),
        ]
    }

    /// The committed pull-snapshot fixture end-to-end: pull parse → extract
    /// → the golden pure record (exact key set, validated by the generated
    /// lexicon type).
    #[test]
    fn golden_doc_record_from_pull_snapshot_fixture() {
        let fixture: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../experiments/02-draft-internals/fixtures/export-pull-doc.json"
        )))
        .unwrap();
        let snapshot = parse_pull_snapshot(&fixture["response"].to_string()).unwrap();
        assert_eq!(snapshot.facts.len(), 9);
        let data: crate::LeafletData = serde_json::from_value(fixture["leaflet_data"].clone()).unwrap();
        let context = ExtractContext {
            data: &data,
            did: fixture["did"].as_str(),
            publication_tags: snapshot.publication_tags(),
        };
        let options = ExtractOptions {
            published_at: fixture["options"]["published_at"].as_str().map(str::to_string),
            slug: None,
        };
        let extracted = extract(snapshot.facts, &context, &options).unwrap();

        let expected: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../experiments/02-draft-internals/fixtures/export-record-doc.json"
        )))
        .unwrap();
        assert_eq!(extracted.record, expected, "the golden record is the pure canonical form");
        assert_eq!(extracted.slug, "synthetic-title");
        assert!(extracted.warns.is_empty());
        assert_eq!(extracted.blocks_converted, 2);
        assert_eq!(extracted.blocks_skipped, 0);

        // Purity: exactly the builder's key set (publish parity).
        let keys: std::collections::BTreeSet<&str> = extracted.record.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            std::collections::BTreeSet::from(["$type", "content", "description", "path", "publishedAt", "site", "textContent", "title"])
        );
        // And the generated lexicon validation passes (publish run's gate).
        springleaf_leaflet::validate_document_record(&extracted.record).expect("generated validate() passes");
        // Envelope values.
        assert_eq!(extracted.record["site"], json!("https://leaflet.pub/p/did:plc:aaaaaaaaaaaaaaaaaaaaaaaa"));
        assert_eq!(extracted.record["publishedAt"], json!("2026-11-07T00:05:30.799Z"));
        assert_eq!(extracted.record["textContent"], json!("Synthetic title\nSynthetic body copy"));
        assert_eq!(extracted.record["content"]["pages"][0]["id"], json!(PAGE));
    }

    #[test]
    fn block_ordering_by_position_then_fact_id() {
        // Same position: fact id decides; positions out of wire order.
        let mut facts = vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a0"})),
            fact("f02", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact("fb", PAGE, "card/block", json!({"type": "ordered-reference", "value": "e-b", "position": "a1"})),
            fact("fa", PAGE, "card/block", json!({"type": "ordered-reference", "value": "e-a", "position": "a1"})),
            fact("fc", PAGE, "card/block", json!({"type": "ordered-reference", "value": "e-c", "position": "a0"})),
            fact("f3", "e-a", "block/type", json!({"type": "block-type-union", "value": "text"})),
            fact("f4", "e-a", "block/text", json!({"type": "text", "value": yjs_b64("A")})),
            fact("f5", "e-b", "block/type", json!({"type": "block-type-union", "value": "text"})),
            fact("f6", "e-b", "block/text", json!({"type": "text", "value": yjs_b64("B")})),
            fact("f7", "e-c", "block/type", json!({"type": "block-type-union", "value": "text"})),
            fact("f8", "e-c", "block/text", json!({"type": "text", "value": yjs_b64("C")})),
        ];
        let data = leaflet_data(ROOT);
        let extracted = extract_with(std::mem::take(&mut facts), &data, &ExtractOptions::default());
        let blocks = &extracted.record["content"]["pages"][0]["blocks"];
        let texts: Vec<&str> = blocks.as_array().unwrap().iter().map(|b| b["block"]["plaintext"].as_str().unwrap()).collect();
        assert_eq!(texts, vec!["C", "A", "B"], "a0 first; a1 ties broken by fact id (fa < fb)");
    }

    #[test]
    fn list_folding_runs_nesting_and_checklist() {
        // Ordered list (start 3, first item checked) with a nested unordered
        // child; then a top-level unordered run after it.
        let facts = vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a0"})),
            fact("f02", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact(
                "f03",
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": "e-h", "position": "a0"}),
            ),
            fact("f04", "e-h", "block/type", json!({"type": "block-type-union", "value": "heading"})),
            fact("f05", "e-h", "block/heading-level", json!({"type": "number", "value": 1})),
            fact("f06", "e-h", "block/text", json!({"type": "text", "value": yjs_b64("Lists")})),
            // ordered list head
            fact(
                "f10",
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": "e-l1", "position": "a1"}),
            ),
            fact("f11", "e-l1", "block/type", json!({"type": "block-type-union", "value": "text"})),
            fact("f12", "e-l1", "block/is-list", json!({"type": "boolean", "value": true})),
            fact("f13", "e-l1", "block/list-style", json!({"type": "list-style-union", "value": "ordered"})),
            fact("f14", "e-l1", "block/list-number", json!({"type": "number", "value": 3})),
            fact("f15", "e-l1", "block/check-list", json!({"type": "boolean", "value": true})),
            fact("f16", "e-l1", "block/text", json!({"type": "text", "value": yjs_b64("first")})),
            // second ordered item
            fact(
                "f20",
                "e-l1",
                "card/block",
                json!({"type": "ordered-reference", "value": "e-l2", "position": "a0"}),
            ),
            fact("f21", "e-l2", "block/type", json!({"type": "block-type-union", "value": "text"})),
            fact("f22", "e-l2", "block/is-list", json!({"type": "boolean", "value": true})),
            fact("f23", "e-l2", "block/list-style", json!({"type": "list-style-union", "value": "ordered"})),
            fact("f24", "e-l2", "block/text", json!({"type": "text", "value": yjs_b64("second")})),
            // nested unordered child under e-l2
            fact(
                "f30",
                "e-l2",
                "card/block",
                json!({"type": "ordered-reference", "value": "e-l3", "position": "a0"}),
            ),
            fact("f31", "e-l3", "block/type", json!({"type": "block-type-union", "value": "text"})),
            fact("f32", "e-l3", "block/is-list", json!({"type": "boolean", "value": true})),
            fact("f33", "e-l3", "block/list-style", json!({"type": "list-style-union", "value": "unordered"})),
            fact("f34", "e-l3", "block/check-list", json!({"type": "boolean", "value": false})),
            fact("f35", "e-l3", "block/text", json!({"type": "text", "value": yjs_b64("nested")})),
            // top-level unordered item (new run)
            fact(
                "f40",
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": "e-u1", "position": "a2"}),
            ),
            fact("f41", "e-u1", "block/type", json!({"type": "block-type-union", "value": "text"})),
            fact("f42", "e-u1", "block/is-list", json!({"type": "boolean", "value": true})),
            fact("f43", "e-u1", "block/list-style", json!({"type": "list-style-union", "value": "unordered"})),
            fact("f44", "e-u1", "block/text", json!({"type": "text", "value": yjs_b64("bullet")})),
        ];
        let data = leaflet_data(ROOT);
        let extracted = extract_with(facts, &data, &ExtractOptions::default());
        let blocks = extracted.record["content"]["pages"][0]["blocks"].as_array().unwrap().clone();
        let types: Vec<&str> = blocks.iter().map(|b| b["block"]["$type"].as_str().unwrap()).collect();
        assert_eq!(
            types,
            vec![
                "pub.leaflet.blocks.header",
                "pub.leaflet.blocks.orderedList",
                "pub.leaflet.blocks.unorderedList",
            ]
        );
        let ordered = &blocks[1]["block"];
        assert_eq!(ordered["startIndex"], json!(3));
        let children = ordered["children"].as_array().unwrap();
        assert_eq!(children[0]["content"]["plaintext"], json!("first"));
        assert_eq!(children[0]["checked"], json!(true), "checklist item carries its own checked state");
        // "second" is a card/block child of e-l1 → a nested list item under
        // "first" (parseBlocksToList nesting), not a sibling.
        let second = &children[0]["children"][0];
        assert_eq!(second["content"]["plaintext"], json!("second"));
        assert!(second.get("checked").is_none(), "check-list is per-block; second has no fact");
        // The nested unordered child (card/block on e-l2, unordered style)
        // rides inside "second".
        let nested = &second["unorderedListChildren"];
        assert_eq!(nested["children"][0]["content"]["plaintext"], json!("nested"));
        assert_eq!(nested["children"][0]["checked"], json!(false), "explicitly unchecked nested item");
        assert_eq!(blocks[2]["block"]["children"][0]["content"]["plaintext"], json!("bullet"));
        springleaf_leaflet::validate_document_record(&extracted.record).unwrap();
    }

    /// One block of each of the 30 `block/type` values: the record carries
    /// exactly the mappable ones and the warn set is exact.
    #[test]
    fn coverage_matrix_all_thirty_types() {
        let mut facts = vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a0"})),
            fact("f02", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact("f99", ROOT, "theme/code-theme", json!({"type": "string", "value": "github-dark"})),
        ];
        let mut position = 0u32;
        let mut entity_no = 0u32;
        let new_entity = |entity_no: &mut u32| {
            let entity = format!("e-block-{entity_no:02}");
            *entity_no += 1;
            entity
        };
        let add = |facts: &mut Vec<FactRow>, entity: &str, attribute: &str, data: Value| {
            facts.push(fact(&format!("f-{entity}-{attribute}"), entity, attribute, data));
        };
        let mut block = |facts: &mut Vec<FactRow>, block_type: &str, ornament: Vec<(&str, Value)>| {
            let entity = new_entity(&mut entity_no);
            // Fractional keys are lexicographically ordered: a0..a9, b0..b9,
            // c0.. — NOT a10 after a9 (generation order == sort order).
            let key = format!("{}{}", (b'a' + (position / 10) as u8) as char, (b'0' + (position % 10) as u8) as char);
            position += 1;
            facts.push(fact(
                &format!("f-cb-{entity}"),
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": entity, "position": key}),
            ));
            add(facts, &entity, "block/type", json!({"type": "block-type-union", "value": block_type}));
            for (attribute, data) in ornament {
                add(facts, &entity, attribute, data);
            }
            entity
        };
        let text_ornament: Vec<(&str, Value)> = vec![("block/text", json!({"type": "text", "value": yjs_b64("x")}))];

        block(
            &mut facts,
            "heading",
            vec![
                ("block/heading-level", json!({"type": "number", "value": 2})),
                ("block/text", json!({"type": "text", "value": yjs_b64("Coverage")})),
                ("block/text-alignment", json!({"type": "text-alignment-type-union", "value": "center"})),
            ],
        );
        block(
            &mut facts,
            "text",
            vec![
                ("block/text", json!({"type": "text", "value": yjs_b64("body")})),
                ("block/text-size", json!({"type": "text-size-union", "value": "large"})),
            ],
        );
        block(&mut facts, "blockquote", text_ornament.clone());
        block(&mut facts, "math", vec![("block/math", json!({"type": "string", "value": "e^{i\\pi} = -1"}))]);
        block(
            &mut facts,
            "code",
            vec![
                ("block/code", json!({"type": "string", "value": "fn main() {}"})),
                ("block/code-language", json!({"type": "string", "value": "rust"})),
            ],
        );
        block(&mut facts, "horizontal-rule", vec![]);
        block(&mut facts, "signup", vec![]);
        block(
            &mut facts,
            "post-header",
            vec![("post-header/compact", json!({"type": "boolean", "value": true}))],
        );
        block(
            &mut facts,
            "button",
            vec![
                ("button/text", json!({"type": "string", "value": "Go"})),
                ("button/url", json!({"type": "string", "value": "https://example.com"})),
            ],
        );
        block(
            &mut facts,
            "bluesky-post",
            vec![
                (
                    "block/bluesky-post",
                    json!({"type": "bluesky-post", "value": {"post": {"uri": "at://did:plc:b/app.bsky.feed.post/3k", "cid": "bafyreicid"}}}),
                ),
                ("bluesky-post/host", json!({"type": "string", "value": "https://bsky.social"})),
                ("bluesky-post/view", json!({"type": "bluesky-post-view-union", "value": "media"})),
            ],
        );
        block(
            &mut facts,
            "standard-site-post",
            vec![
                (
                    "block/standard-site-post",
                    json!({"type": "string", "value": "at://did:plc:b/site.standard.document/3doc"}),
                ),
                ("standard-site-post/size", json!({"type": "standard-site-post-size-union", "value": "medium"})),
                ("standard-site-post/show-publication-theme", json!({"type": "boolean", "value": false})),
            ],
        );
        block(
            &mut facts,
            "standard-site-publication",
            vec![
                (
                    "block/standard-site-publication",
                    json!({"type": "string", "value": "at://did:plc:b/site.standard.publication/3pub"}),
                ),
                ("standard-site-publication/show-publication-theme", json!({"type": "boolean", "value": false})),
            ],
        );
        block(
            &mut facts,
            "recommended-pubs",
            vec![("recommended-pubs/compact", json!({"type": "boolean", "value": true}))],
        );
        let reply_entity = block(
            &mut facts,
            "reply",
            vec![
                ("reply/button-text", json!({"type": "string", "value": "Reply"})),
                ("reply/prompt-text", json!({"type": "string", "value": "Say something"})),
                ("reply/show-publication-theme", json!({"type": "boolean", "value": false})),
            ],
        );
        let questions_entity = block(
            &mut facts,
            "questions",
            vec![
                ("questions/button-text", json!({"type": "string", "value": "Ask"})),
                ("questions/audience", json!({"type": "string", "value": "follows"})),
                ("questions/show-pending", json!({"type": "boolean", "value": true})),
            ],
        );
        block(
            &mut facts,
            "members-only-delimiter",
            vec![
                ("block/members-only-audience", json!({"type": "string", "value": "tiers"})),
                ("block/members-only-tier", json!({"type": "string", "value": "tier-b"})),
                ("block/members-only-tier", json!({"type": "string", "value": "tier-a"})),
            ],
        );
        block(
            &mut facts,
            "posts-list",
            vec![
                ("posts-list/view", json!({"type": "posts-list-view-union", "value": "chapter"})),
                ("posts-list/highlight-first-post", json!({"type": "boolean", "value": true})),
                ("posts-list/show-page-count", json!({"type": "boolean", "value": false})),
                ("posts-list/filter-tag", json!({"type": "string", "value": "news"})),
                ("posts-list/filter-tag", json!({"type": "string", "value": "meta"})),
                ("posts-list/limit", json!({"type": "number", "value": 5})),
                ("posts-list/reader-controls", json!({"type": "boolean", "value": true})),
                ("posts-list/reader-sort", json!({"type": "boolean", "value": false})),
            ],
        );
        block(
            &mut facts,
            "link",
            vec![
                ("link/url", json!({"type": "string", "value": "https://leaflet.pub"})),
                ("link/title", json!({"type": "string", "value": "Leaflet"})),
                ("link/description", json!({"type": "string", "value": "Publishing"})),
                ("link/preview", json!({"type": "image", "src": "https://x/p.png", "height": 100, "width": 50})),
            ],
        );
        block(
            &mut facts,
            "embed",
            vec![
                ("embed/url", json!({"type": "string", "value": "https://example.com/embed"})),
                ("embed/height", json!({"type": "number", "value": 400.9})),
                ("embed/aspect-ratio", json!({"type": "string", "value": "16/9"})),
            ],
        );
        block(
            &mut facts,
            "html",
            vec![
                ("embed/html", json!({"type": "string", "value": "<p>hi</p>"})),
                ("embed/height", json!({"type": "number", "value": 0})),
            ],
        );
        block(
            &mut facts,
            "image",
            vec![("block/image", json!({"type": "image", "src": "https://x/i.png", "height": 10, "width": 10}))],
        );
        block(
            &mut facts,
            "image-gallery",
            vec![
                ("gallery/image", json!({"type": "ordered-reference", "value": "e-g1", "position": "a0"})),
                ("gallery/format", json!({"type": "gallery-format-union", "value": "grid"})),
                ("gallery/gap", json!({"type": "number", "value": 8})),
                ("gallery/max-width", json!({"type": "number", "value": 400})),
            ],
        );
        facts.push(fact("f-g1-type", "e-g1", "block/type", json!({"type": "block-type-union", "value": "image"})));
        facts.push(fact(
            "f-g1-img",
            "e-g1",
            "block/image",
            json!({"type": "image", "src": "https://x/g1.png", "height": 5, "width": 5}),
        ));
        // card → doc subpage
        let subpage = "e-subpage";
        facts.push(fact("f-sub-type", subpage, "page/type", json!({"type": "page-type-union", "value": "doc"})));
        facts.push(fact(
            "f-sub-cb",
            subpage,
            "card/block",
            json!({"type": "ordered-reference", "value": "e-sub-text", "position": "a0"}),
        ));
        facts.push(fact(
            "f-sub-t1",
            "e-sub-text",
            "block/type",
            json!({"type": "block-type-union", "value": "text"}),
        ));
        facts.push(fact(
            "f-sub-t2",
            "e-sub-text",
            "block/text",
            json!({"type": "text", "value": yjs_b64("subpage body")}),
        ));
        block(
            &mut facts,
            "card",
            vec![
                ("block/card", json!({"type": "reference", "value": subpage})),
                ("page-link/display", json!({"type": "page-link-display-union", "value": "compact"})),
            ],
        );
        // embedded-canvas → canvas page (deferred)
        let canvas_page = "e-canvas-page";
        facts.push(fact(
            "f-cp-type",
            canvas_page,
            "page/type",
            json!({"type": "page-type-union", "value": "canvas"}),
        ));
        block(
            &mut facts,
            "embedded-canvas",
            vec![
                ("block/card", json!({"type": "reference", "value": canvas_page})),
                ("image/alt", json!({"type": "string", "value": "alt text"})),
            ],
        );
        // drawing: two strokes, one with a fill
        block(
            &mut facts,
            "drawing",
            vec![
                (
                    "drawing/view-box",
                    json!({"type": "view-box", "value": {"x": 0.4, "y": -0.5, "width": 100.2, "height": 50.7}}),
                ),
                (
                    "drawing/stroke",
                    json!({"type": "ink-stroke", "value": {"points": [10.4, 20.5, 30, 40], "color": "#111111", "size": 3.2, "fill": {"points": [0, 0, 10, 0, 10, 10], "color": "#eeeeee"}}}),
                ),
                (
                    "drawing/stroke",
                    json!({"type": "ink-stroke", "value": {"points": [1, 2, 3, 4, 5, 6], "color": "#222222", "size": 2, "simulatePressure": true}}),
                ),
            ],
        );
        block(
            &mut facts,
            "poll",
            vec![("poll/options", json!({"type": "ordered-reference", "value": "e-poll-opt", "position": "a0"}))],
        );
        facts.push(fact(
            "f-poll-opt-name",
            "e-poll-opt",
            "poll-option/name",
            json!({"type": "string", "value": "Yes"}),
        ));
        block(&mut facts, "datetime", vec![]);
        block(&mut facts, "rsvp", vec![]);
        block(&mut facts, "mailbox", vec![]);
        block(&mut facts, "group", vec![]);
        // Not one of the 30: Leaflet's own-property check drops it, we warn.
        block(&mut facts, "unknown-future-type", vec![]);

        let data = leaflet_data(ROOT);
        let extracted = extract_with(facts, &data, &ExtractOptions::default());

        // Exact warn set (block-type scoped; the members-only notice rides
        // along as a non-block warn).
        let warn_types: std::collections::BTreeSet<&str> = extracted.warns.iter().map(|warn| warn.block_type.as_str()).collect();
        assert_eq!(
            warn_types,
            std::collections::BTreeSet::from([
                "datetime",
                "rsvp",
                "mailbox",
                "group",
                "image",
                "image-gallery",
                "poll",
                "embedded-canvas",
                "unknown-future-type",
                "members-only-delimiter",
                "link",
            ]),
            "warns: 3 draft-only + group + 3 read-only v0 skips + canvas subpage + unknown + members-only + link preview"
        );

        let pages = extracted.record["content"]["pages"].as_array().unwrap();
        assert_eq!(pages.len(), 2, "start page + appended card subpage");
        assert_eq!(pages[0]["id"], json!(PAGE), "start page first (unshift parity)");
        assert_eq!(pages[1]["id"], json!(subpage));
        let blocks = pages[0]["blocks"].as_array().unwrap();
        let types: Vec<&str> = blocks.iter().map(|b| b["block"]["$type"].as_str().unwrap()).collect();
        assert_eq!(
            types,
            vec![
                "pub.leaflet.blocks.header",
                "pub.leaflet.blocks.text",
                "pub.leaflet.blocks.blockquote",
                "pub.leaflet.blocks.math",
                "pub.leaflet.blocks.code",
                "pub.leaflet.blocks.horizontalRule",
                "pub.leaflet.blocks.signup",
                "pub.leaflet.blocks.postHeader",
                "pub.leaflet.blocks.button",
                "pub.leaflet.blocks.bskyPost",
                "pub.leaflet.blocks.standardSitePost",
                "pub.leaflet.blocks.standardSitePublication",
                "pub.leaflet.blocks.recommendedPubs",
                "pub.leaflet.blocks.reply",
                "pub.leaflet.blocks.questions",
                "pub.leaflet.blocks.membersOnlyDelimiter",
                "pub.leaflet.blocks.postsList",
                "pub.leaflet.blocks.website",
                "pub.leaflet.blocks.iframe",
                "pub.leaflet.blocks.html",
                "pub.leaflet.blocks.page",
                "pub.leaflet.blocks.drawing",
            ]
        );
        // Spot checks per class.
        assert_eq!(blocks[0]["block"]["level"], json!(2));
        assert_eq!(blocks[0]["alignment"], json!("lex:pub.leaflet.pages.linearDocument#textAlignCenter"));
        assert_eq!(blocks[1]["block"]["textSize"], json!("large"));
        assert_eq!(blocks[4]["block"]["syntaxHighlightingTheme"], json!("github-dark"), "root-scoped code theme");
        assert_eq!(blocks[4]["block"]["language"], json!("rust"));
        assert_eq!(blocks[7]["block"]["compact"], json!(true));
        assert_eq!(blocks[8]["block"]["url"], json!("https://example.com"));
        assert_eq!(blocks[9]["block"]["postRef"]["uri"], json!("at://did:plc:b/app.bsky.feed.post/3k"));
        assert_eq!(blocks[9]["block"]["view"], json!("media"));
        assert_eq!(blocks[9]["block"]["clientHost"], json!("https://bsky.social"));
        assert_eq!(blocks[10]["block"]["size"], json!("medium"));
        assert_eq!(blocks[10]["block"]["showPublicationTheme"], json!(false));
        assert_eq!(blocks[13]["block"]["key"], json!(reply_entity));
        assert_eq!(blocks[14]["block"]["key"], json!(questions_entity));
        assert_eq!(blocks[14]["block"]["audience"], json!("follows"));
        assert_eq!(blocks[14]["block"]["showPending"], json!(true));
        assert_eq!(blocks[15]["block"]["audience"], json!("tiers"));
        assert_eq!(blocks[15]["block"]["tierIds"], json!(["tier-a", "tier-b"]), "tiers sorted");
        assert_eq!(blocks[16]["block"]["filterByTags"], json!(["news", "meta"]));
        assert_eq!(blocks[16]["block"]["limit"], json!(5));
        assert_eq!(blocks[16]["block"]["readerControls"], json!(true));
        assert_eq!(blocks[16]["block"]["readerSearch"], json!(true), "sub-flags default true");
        assert_eq!(blocks[16]["block"]["readerSort"], json!(false), "explicit false kept");
        assert_eq!(blocks[17]["block"]["previewImage"], Value::Null, "preview omitted (warn)");
        assert_eq!(blocks[18]["block"]["height"], json!(400), "embed height floors 400.9");
        assert_eq!(blocks[18]["block"]["aspectRatio"], json!({"width": 16, "height": 9}));
        assert_eq!(blocks[19]["block"]["height"], json!(600), "height defaults 600 when 0");
        assert_eq!(blocks[20]["block"]["id"], json!(subpage));
        assert_eq!(blocks[20]["block"]["display"], json!("compact"));
        let drawing = &blocks[21]["block"];
        assert_eq!(
            drawing["viewBox"],
            json!({"x": 0, "y": -0, "width": 100, "height": 51}),
            "js rounding (0.5 up, -0.5 toward +inf)"
        );
        assert_eq!(drawing["strokes"].as_array().unwrap().len(), 2);
        assert_eq!(drawing["strokes"][1]["simulatePressure"], json!(true));
        assert_eq!(drawing["strokes"][1]["points"], json!([1, 2, 3, 4, 5, 6]));
        assert_eq!(drawing["fills"][0]["points"], json!([0, 0, 10, 0, 10, 10]));
        assert_eq!(drawing["fills"][0]["color"], json!("#eeeeee"));
        assert_eq!(drawing["strokes"][0]["size"], json!(3), "size 3.2 rounds to 3");
        // The whole record validates through the generated lexicon types.
        springleaf_leaflet::validate_document_record(&extracted.record).expect("coverage record validates");
    }

    #[test]
    fn envelope_fallbacks_title_site_publishedat_tags() {
        // No text blocks at all: title falls back to permission_tokens.title.
        let mut data = leaflet_data(ROOT);
        data.title = Some("Token title".into());
        data.description = Some("Token description".into());
        let facts = vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a0"})),
            fact("f02", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact(
                "f03",
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": "e-hr", "position": "a0"}),
            ),
            fact("f04", "e-hr", "block/type", json!({"type": "block-type-union", "value": "horizontal-rule"})),
        ];
        let extracted = extract_with(facts, &data, &ExtractOptions::default());
        assert_eq!(extracted.title, "Token title");
        assert_eq!(extracted.slug, "token-title");
        assert_eq!(extracted.record["description"], json!("Token description"));

        // Neither text nor token title: Untitled.
        let data = leaflet_data(ROOT);
        let facts = vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a0"})),
            fact("f02", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
        ];
        let extracted = extract_with(facts, &data, &ExtractOptions::default());
        assert_eq!(extracted.title, "Untitled");
        assert_eq!(extracted.slug, "untitled");

        // Publication-linked: site = publication at-uri, publishedAt from
        // the linked doc, tags from the pull extras.
        let mut data = leaflet_data(ROOT);
        data.leaflets_in_publications = vec![
            serde_json::from_value(json!({
                "doc": "at://did:plc:bbbb/site.standard.document/3doc",
                "publication": "at://did:plc:bbbb/site.standard.publication/3pub",
                "documents": {"data": {"path": "/3doc", "publishedAt": "2026-02-03T05:06:07.123456Z"}},
                "publications": {"record": {"name": "P", "url": null}}
            }))
            .unwrap(),
        ];
        let facts = minimal_doc_facts();
        let context = ExtractContext {
            data: &data,
            did: Some("did:plc:aaaaaaaaaaaaaaaaaaaaaaaa"),
            publication_tags: Some(vec!["news".into(), "meta".into()]),
        };
        let extracted = extract(facts, &context, &ExtractOptions::default()).unwrap();
        assert_eq!(extracted.record["site"], json!("at://did:plc:bbbb/site.standard.publication/3pub"));
        assert_eq!(extracted.record["publishedAt"], json!("2026-02-03T05:06:07.123456Z"));
        assert_eq!(extracted.record["tags"], json!(["news", "meta"]));
        springleaf_leaflet::validate_document_record(&extracted.record).unwrap();

        // No linkage and no did: structural error.
        let data = leaflet_data(ROOT);
        let err = extract(
            minimal_doc_facts(),
            &ExtractContext {
                data: &data,
                did: None,
                publication_tags: None,
            },
            &ExtractOptions::default(),
        );
        assert!(matches!(err, Err(Error::Structure(_))), "{err:?}");
    }

    #[test]
    fn undecodable_text_empties_the_block_with_a_warn() {
        let facts = vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a0"})),
            fact("f02", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact(
                "f03",
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": "e-bad", "position": "a0"}),
            ),
            fact("f04", "e-bad", "block/type", json!({"type": "block-type-union", "value": "text"})),
            // b5ec6d bytes, base64-encoded, as the fact value.
            fact(
                "f05",
                "e-bad",
                "block/text",
                json!({"type": "text", "value": base64::engine::general_purpose::STANDARD.encode([0xb5, 0xec, 0x6d])}),
            ),
        ];
        let data = leaflet_data(ROOT);
        let extracted = extract_with(facts, &data, &ExtractOptions::default());
        assert_eq!(extracted.record["content"]["pages"][0]["blocks"][0]["block"]["plaintext"], json!(""));
        assert!(extracted.warns.iter().any(|warn| warn.reason.contains("undecodable")), "{:?}", extracted.warns);
        // The block itself still exists (never dropped for bad text).
        assert_eq!(extracted.blocks_skipped, 0);
    }

    #[test]
    fn structural_errors_missing_root_page_or_entity() {
        let mut data = leaflet_data(ROOT);
        data.root_entity = None;
        let err = extract(vec![], &ctx(&data), &ExtractOptions::default());
        assert!(matches!(err, Err(Error::Structure(_))));

        let data = leaflet_data(ROOT);
        // root exists but no root/page fact.
        let err = extract(
            vec![fact("f0", ROOT, "page/type", json!({"type": "page-type-union", "value": "doc"}))],
            &ctx(&data),
            &ExtractOptions::default(),
        );
        assert!(matches!(err, Err(Error::Structure(_))));
    }

    #[test]
    fn multiple_root_pages_warn_but_first_wins() {
        let facts = vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a1"})),
            fact(
                "f02",
                ROOT,
                "root/page",
                json!({"type": "ordered-reference", "value": "e-other-page", "position": "a0"}),
            ),
            fact("f03", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact("f04", "e-other-page", "page/type", json!({"type": "page-type-union", "value": "doc"})),
        ];
        let data = leaflet_data(ROOT);
        let extracted = extract_with(facts, &data, &ExtractOptions::default());
        assert_eq!(extracted.record["content"]["pages"][0]["id"], json!("e-other-page"), "position a0 wins");
        assert!(extracted.warns.iter().any(|warn| warn.reason.contains("root/page facts")));
    }

    #[test]
    fn envelope_theme_and_cover_image_warn_and_skip() {
        let mut facts = minimal_doc_facts();
        facts.push(fact("f10", ROOT, "root/cover-image", json!({"type": "reference", "value": "e-cover"})));
        facts.push(fact(
            "f11",
            "e-cover",
            "block/image",
            json!({"type": "image", "src": "https://x/c.png", "height": 1, "width": 1}),
        ));
        facts.push(fact("f12", ROOT, "theme/page-width", json!({"type": "number", "value": 720})));
        facts.push(fact("f13", PAGE, "theme/card-background", json!({"type": "color", "value": "0.1,0.2,0.3,1"})));
        let data = leaflet_data(ROOT);
        let extracted = extract_with(facts, &data, &ExtractOptions::default());
        assert!(extracted.record.get("coverImage").is_none());
        assert!(extracted.record.get("theme").is_none());
        assert!(extracted.record["content"]["pages"][0].get("theme").is_none());
        let reasons: Vec<&str> = extracted.warns.iter().map(|warn| warn.reason.as_str()).collect();
        assert!(reasons.iter().any(|r| r.contains("cover image")), "{reasons:?}");
        assert!(reasons.iter().any(|r| r.contains("document theme")), "{reasons:?}");
        assert!(reasons.iter().any(|r| r.contains("page theme")), "{reasons:?}");
    }

    #[test]
    fn slug_override_drives_path() {
        let data = leaflet_data(ROOT);
        let extracted = extract_with(
            minimal_doc_facts(),
            &data,
            &ExtractOptions {
                slug: Some("custom-slug".into()),
                published_at: None,
            },
        );
        assert_eq!(extracted.slug, "custom-slug");
        assert_eq!(extracted.record["path"], json!("/custom-slug"));
    }

    #[test]
    fn cycle_guard_and_duplicate_card_references() {
        // A card whose subpage contains a card back at the start page.
        let facts = vec![
            fact("f01", ROOT, "root/page", json!({"type": "ordered-reference", "value": PAGE, "position": "a0"})),
            fact("f02", PAGE, "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact(
                "f03",
                PAGE,
                "card/block",
                json!({"type": "ordered-reference", "value": "e-card", "position": "a0"}),
            ),
            fact("f04", "e-card", "block/type", json!({"type": "block-type-union", "value": "card"})),
            fact("f05", "e-card", "block/card", json!({"type": "reference", "value": "e-sub"})),
            fact("f06", "e-sub", "page/type", json!({"type": "page-type-union", "value": "doc"})),
            fact(
                "f07",
                "e-sub",
                "card/block",
                json!({"type": "ordered-reference", "value": "e-back", "position": "a0"}),
            ),
            fact("f08", "e-back", "block/type", json!({"type": "block-type-union", "value": "card"})),
            fact("f09", "e-back", "block/card", json!({"type": "reference", "value": PAGE})),
        ];
        let data = leaflet_data(ROOT);
        let extracted = extract_with(facts, &data, &ExtractOptions::default());
        // Terminates, warns about the cycle, and produces start + subpage.
        assert_eq!(extracted.record["content"]["pages"].as_array().unwrap().len(), 2);
        assert!(
            extracted
                .warns
                .iter()
                .any(|warn| warn.reason.contains("recursion") || warn.reason.contains("acyclic")),
            "{:?}",
            extracted.warns
        );
    }
}
