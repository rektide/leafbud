//! Reverse converter — canonical record → Replicache push mutation batch
//! (`design/draft-extraction.glm53m.md` §Reverse).
//!
//! Turns the pure `site.standard.document` record (a `drafts export` or
//! `publish build` output) into the ordered mutation list that writes it
//! **into an existing leaflet editor draft**: per block an `addBlock`
//! (caller-minted uuid v7 fact/entity ids, fractional append position)
//! followed by its `assertFact`s, subpages via `createEntity` +
//! `root/page`, and the `updateLeafletMetadata` title tail.
//!
//! **Append-only.** Authoring never destroys existing facts: blocks land
//! after the draft's current last position (`generateKeyBetween(last,
//! null)` per block), and pages after its last `root/page` key. Authoring
//! into a non-empty draft therefore appends after the existing content —
//! replacing/overwriting draft content is a deliberate non-goal for v0
//! (that would take `removeBlock`/`retractFact` sequences). Ids are
//! caller-minted (uuid v7 matches Leaflet's own client convention) via
//! the `ids` iterator, so tests are fully deterministic offline.
//!
//! v0 warn + skip: image, imageGallery (need the storage upload surface),
//! drawing, embeddedCanvas (canvas pages deferred), theme/alignment facts.
//! Poll blocks author draft-side only (option entities + `poll/options`);
//! the poll definition record is publish-time, so option texts must be
//! resolved by the caller (from the record's `pollRef` PDS record) and
//! passed in [`AuthorOptions::poll_definitions`].

use std::collections::HashMap;

use serde::Serialize;
use serde_json::{Value, json};

use crate::Error;
use crate::extract::Warn;
use crate::fractional::generate_key_between;
use crate::yjs::{FacetRun, Mark, encode_block_text};

/// Everything the converter needs about the target draft and the push
/// session. The position seeds come from a read-only pull of the draft;
/// `client_id`/ids/timestamps from the caller.
#[derive(Debug, Clone)]
pub struct AuthorOptions {
    /// Target draft root entity (`get_leaflet_data.root_entity`).
    pub root_entity: String,
    /// The start page entity (first `root/page` by position) blocks
    /// append into.
    pub page_entity: String,
    /// The entity set write rights resolve against (token rights'
    /// `entity_set`) — `addBlock`'s `permission_set`.
    pub permission_set: String,
    /// The draft's current last `root/page` position, when any.
    pub last_page_position: Option<String>,
    /// The start page's current last `card/block` position, when any.
    pub last_block_position: Option<String>,
    /// Yjs clientID for encoded `block/text` values (random u32 per run).
    pub yjs_client_id: u32,
    /// Replicache clientID for the mutation rows (stable within one push).
    pub client_id: String,
    /// First mutation id (monotonic per clientID; replay protection skips
    /// `id <= lastMutationID`, experiments/02 §3). Default 1 with a fresh
    /// clientID per run.
    pub mutation_id_base: u64,
    /// Base mutation timestamp (ms epoch); each mutation takes base + n.
    pub timestamp_base: i64,
    /// Resolved poll option texts by pollRef at-uri (poll blocks whose
    /// ref is absent from this map warn + skip).
    pub poll_definitions: HashMap<String, Vec<String>>,
}

/// One push mutation row (`PushRequestV1.mutations[]` shape,
/// `fixtures/push-minimal-authoring.json`).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthorMutation {
    pub id: u64,
    pub name: String,
    /// The wire spells this `clientID` (capital D), not camelCase's
    /// `clientId`.
    #[serde(rename = "clientID")]
    pub client_id: String,
    pub timestamp: i64,
    pub args: Value,
}

/// The built batch plus its warnings and counts.
#[derive(Debug)]
pub struct Authored {
    pub mutations: Vec<AuthorMutation>,
    pub warns: Vec<Warn>,
    pub blocks_authored: usize,
    pub blocks_skipped: usize,
    /// The draft's new last positions (for a follow-up authoring pass).
    pub last_block_position: Option<String>,
    pub last_page_position: Option<String>,
}

/// A per-parent fractional position chain: each append draws
/// `generate_key_between(last, None)`.
struct PositionChain {
    last: Option<String>,
}

impl PositionChain {
    fn new(seed: Option<String>) -> Self {
        Self { last: seed }
    }

    fn next(&mut self) -> Result<String, Error> {
        let key = generate_key_between(self.last.as_deref(), None).map_err(|err| Error::Structure(format!("fractional key minting failed: {err}")))?;
        self.last = Some(key.clone());
        Ok(key)
    }
}

// ── data payload helpers (the FactData wire shapes) ─────────────────────────

fn string_data(value: &str) -> Value {
    json!({"type": "string", "value": value})
}

fn number_data(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() < 9.007_199_254_740_992e15 {
        json!({"type": "number", "value": value as i64})
    } else {
        json!({"type": "number", "value": value})
    }
}

fn boolean_data(value: bool) -> Value {
    json!({"type": "boolean", "value": value})
}

fn union_data(kind: &str, value: &str) -> Value {
    json!({"type": kind, "value": value})
}

fn text_data(base64_value: &str) -> Value {
    json!({"type": "text", "value": base64_value})
}

// ── the builder ─────────────────────────────────────────────────────────────

struct Builder<'a> {
    options: &'a AuthorOptions,
    ids: &'a mut dyn Iterator<Item = String>,
    mutations: Vec<AuthorMutation>,
    warns: Vec<Warn>,
    blocks_authored: usize,
    blocks_skipped: usize,
    /// The draft's root/page chain — subpages append here.
    page_chain: PositionChain,
}

impl<'a> Builder<'a> {
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

    fn push(&mut self, name: &str, args: Value) {
        let offset = self.mutations.len() as u64;
        self.mutations.push(AuthorMutation {
            id: self.options.mutation_id_base + offset,
            name: name.to_string(),
            client_id: self.options.client_id.clone(),
            timestamp: self.options.timestamp_base + offset as i64,
            args,
        });
    }

    /// `assertFact` on a cardinality-`one` attribute (server reuses the
    /// existing fact id — update-in-place).
    fn assert(&mut self, entity: &str, attribute: &str, data: Value) {
        self.push("assertFact", json!({"entity": entity, "attribute": attribute, "data": data}));
    }

    /// `assertFact` on a cardinality-`many` attribute: the fact id is
    /// caller-chosen (client/server independence, mutations.ts).
    fn assert_id(&mut self, entity: &str, attribute: &str, data: Value, id: String) {
        self.push("assertFact", json!({"entity": entity, "attribute": attribute, "data": data, "id": id}));
    }

    fn create_entity(&mut self, entity: &str) {
        self.push("createEntity", json!({"entityID": entity, "permission_set": self.options.permission_set}));
    }

    /// `addBlock` and its entity id. `list` rides the same mutation so a
    /// list head is never briefly a non-list block (mutations.ts).
    fn add_block(&mut self, parent: &str, block_type: &str, position: String, list: Option<Value>) -> String {
        let fact_id = self.ids.next().expect("id iterator exhausted");
        let entity = self.ids.next().expect("id iterator exhausted");
        let mut args = json!({
            "parent": parent,
            "permission_set": self.options.permission_set,
            "factID": fact_id,
            "type": block_type,
            "newEntityID": entity,
            "position": position,
        });
        if let Some(list) = list {
            args["list"] = list;
        }
        self.push("addBlock", args);
        entity
    }

    // ── text encoding (record plaintext + facets → block/text) ──────────

    /// Record facet JSON → converter-neutral runs. Unknown feature types
    /// warn and drop (the block still authors); spans ride through as
    /// byte offsets exactly as the record carries them.
    fn record_facets(&mut self, owner: &str, facets: Option<&Value>) -> Vec<FacetRun> {
        let Some(items) = facets.and_then(Value::as_array) else {
            return Vec::new();
        };
        let mut runs = Vec::new();
        for item in items {
            let Some(index) = item.get("index") else { continue };
            let (Some(start), Some(end)) = (index.get("byteStart").and_then(Value::as_i64), index.get("byteEnd").and_then(Value::as_i64)) else {
                self.warn("facets", owner, "facet without byteStart/byteEnd index — facet dropped");
                continue;
            };
            let mut marks = Vec::new();
            for feature in item.get("features").and_then(Value::as_array).into_iter().flatten() {
                let kind = feature.get("$type").and_then(Value::as_str).unwrap_or_default();
                let mark = match kind.trim_start_matches("pub.leaflet.richtext.facet#") {
                    "bold" => Some(Mark::Bold),
                    "italic" => Some(Mark::Italic),
                    "code" => Some(Mark::Code),
                    "strikethrough" => Some(Mark::Strikethrough),
                    "underline" => Some(Mark::Underline),
                    "highlight" => Some(Mark::Highlight),
                    "link" => feature.get("uri").and_then(Value::as_str).map(|uri| Mark::Link(uri.to_string())),
                    other => {
                        self.warn("facets", owner, format!("facet feature {other:?} not authorable in v0 — feature dropped"));
                        None
                    }
                };
                if let Some(mark) = mark {
                    marks.push(mark);
                }
            }
            if !marks.is_empty() && start >= 0 && end > start {
                runs.push(FacetRun {
                    byte_start: start as usize,
                    byte_end: end as usize,
                    marks,
                });
            }
        }
        runs
    }

    /// Encode + assert a block's text. Fails the whole authoring (never
    /// writes a half-text) if a facet span is invalid.
    fn assert_block_text(&mut self, entity: &str, owner: &str, plaintext: &str, facets: Option<&Value>) -> Result<(), Error> {
        let runs = self.record_facets(owner, facets);
        let encoded = encode_block_text(plaintext, &runs, self.options.yjs_client_id).map_err(|err| {
            Error::Structure(format!(
                "text block {owner:?}: facet span {}..{} invalid against a {}-byte text",
                err.byte_start, err.byte_end, err.len
            ))
        })?;
        self.assert(entity, "block/text", text_data(&encoded));
        Ok(())
    }

    // ── blocks ───────────────────────────────────────────────────────────

    /// Author one page's blocks into `page` (record order, append-only).
    fn author_blocks(&mut self, blocks: &[Value], page: &str, chain: &mut PositionChain, pages: &[Value], consumed: &mut Vec<String>) -> Result<(), Error> {
        for block in blocks {
            let inner = match block.get("block") {
                Some(inner) => inner,
                None => {
                    self.skip("block", "?", "record block without a `block` member — dropped");
                    continue;
                }
            };
            if block.get("alignment").is_some() {
                self.warn(
                    "alignment",
                    inner["$type"].as_str().unwrap_or("?"),
                    "text alignment authoring deferred (v0) — alignment fact not written",
                );
            }
            let kind = inner
                .get("$type")
                .and_then(Value::as_str)
                .and_then(|t| t.strip_prefix("pub.leaflet.blocks."))
                .unwrap_or_default()
                .to_string();
            self.author_block(&kind, inner, page, chain, pages, consumed)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn author_block(
        &mut self,
        kind: &str,
        inner: &Value,
        page: &str,
        chain: &mut PositionChain,
        pages: &[Value],
        consumed: &mut Vec<String>,
    ) -> Result<(), Error> {
        let str_at = |key: &str| inner.get(key).and_then(Value::as_str);
        let plaintext = str_at("plaintext").unwrap_or_default();
        match kind {
            "text" | "header" | "blockquote" => {
                let (block_type, needs_level) = match kind {
                    "text" => ("text", false),
                    "header" => ("heading", true),
                    _ => ("blockquote", false),
                };
                let entity = self.add_block(page, block_type, chain.next()?, None);
                if needs_level {
                    let level = inner.get("level").and_then(Value::as_i64).unwrap_or(1).clamp(1, 6);
                    self.assert(&entity, "block/heading-level", number_data(level as f64));
                }
                self.assert_block_text(&entity, &format!("{kind} {plaintext:?}"), plaintext, inner.get("facets"))?;
                if let Some(size) = str_at("textSize").filter(|s| ["default", "small", "large"].contains(s)) {
                    self.assert(&entity, "block/text-size", union_data("text-size-union", size));
                }
                self.blocks_authored += 1;
            }
            "math" => {
                let entity = self.add_block(page, "math", chain.next()?, None);
                self.assert(&entity, "block/math", string_data(str_at("tex").unwrap_or_default()));
                self.blocks_authored += 1;
            }
            "code" => {
                let entity = self.add_block(page, "code", chain.next()?, None);
                self.assert(&entity, "block/code", string_data(str_at("plaintext").unwrap_or_default()));
                if let Some(language) = str_at("language") {
                    self.assert(&entity, "block/code-language", string_data(language));
                }
                if let Some(theme) = str_at("syntaxHighlightingTheme") {
                    self.warn(
                        kind,
                        theme,
                        "syntaxHighlightingTheme is a root theme fact (theme/code-theme) — not authored per block",
                    );
                }
                self.blocks_authored += 1;
            }
            "horizontalRule" => {
                self.add_block(page, "horizontal-rule", chain.next()?, None);
                self.blocks_authored += 1;
            }
            "signup" => {
                self.add_block(page, "signup", chain.next()?, None);
                self.blocks_authored += 1;
            }
            "postHeader" => {
                let entity = self.add_block(page, "post-header", chain.next()?, None);
                if inner.get("compact") == Some(&Value::Bool(true)) {
                    self.assert(&entity, "post-header/compact", boolean_data(true));
                }
                self.blocks_authored += 1;
            }
            "button" => {
                let (Some(text), Some(url)) = (str_at("text"), str_at("url")) else {
                    self.skip(kind, page, "button needs both text and url — block dropped");
                    return Ok(());
                };
                let entity = self.add_block(page, "button", chain.next()?, None);
                self.assert(&entity, "button/text", string_data(text));
                self.assert(&entity, "button/url", string_data(url));
                self.blocks_authored += 1;
            }
            "bskyPost" => {
                let Some(post_ref) = inner.get("postRef") else {
                    self.skip(kind, page, "bskyPost without postRef — block dropped");
                    return Ok(());
                };
                let (Some(uri), Some(cid)) = (post_ref.get("uri").and_then(Value::as_str), post_ref.get("cid").and_then(Value::as_str)) else {
                    self.skip(kind, page, "bskyPost postRef without uri/cid — block dropped");
                    return Ok(());
                };
                let entity = self.add_block(page, "bluesky-post", chain.next()?, None);
                // Minimal thread blob: our extraction (and the fact graph)
                // reads post.uri/cid; Leaflet's own client stores the full
                // getPostThread response, so the editor's rich render of
                // this block may be degraded.
                self.assert(
                    &entity,
                    "block/bluesky-post",
                    json!({"type": "bluesky-post", "value": {"post": {"uri": uri, "cid": cid}}}),
                );
                if let Some(host) = str_at("clientHost") {
                    self.assert(&entity, "bluesky-post/host", string_data(host));
                }
                if str_at("view").is_some() {
                    self.assert(&entity, "bluesky-post/view", union_data("bluesky-post-view-union", "media"));
                }
                self.warn(
                    kind,
                    &entity,
                    "bluesky-post fact carries a minimal {post:{uri,cid}} thread blob — the editor's rich render may be degraded",
                );
                self.blocks_authored += 1;
            }
            "standardSitePost" | "standardSitePublication" => {
                let (block_type, attribute) = if kind == "standardSitePost" {
                    ("standard-site-post", "block/standard-site-post")
                } else {
                    ("standard-site-publication", "block/standard-site-publication")
                };
                let Some(uri) = str_at("uri") else {
                    self.skip(kind, page, "block without uri — dropped");
                    return Ok(());
                };
                let entity = self.add_block(page, block_type, chain.next()?, None);
                self.assert(&entity, attribute, string_data(uri));
                if let Some(size) = str_at("size").filter(|s| ["large", "medium", "small"].contains(s)) {
                    self.assert(&entity, "standard-site-post/size", union_data("standard-site-post-size-union", size));
                }
                if inner.get("showPublicationTheme") == Some(&Value::Bool(false)) {
                    let attribute = format!("{block_type}/show-publication-theme");
                    self.assert(&entity, &attribute, boolean_data(false));
                }
                if let Some(cid) = inner.get("cid").filter(|c| !c.is_null()) {
                    self.warn(
                        kind,
                        &entity,
                        format!("record carries a cid {cid} — draft-side blocks keep no cid fact; dropped"),
                    );
                }
                self.blocks_authored += 1;
            }
            "recommendedPubs" => {
                let entity = self.add_block(page, "recommended-pubs", chain.next()?, None);
                if let Some(compact) = inner.get("compact").and_then(Value::as_bool) {
                    self.assert(&entity, "recommended-pubs/compact", boolean_data(compact));
                }
                self.blocks_authored += 1;
            }
            "reply" | "questions" => {
                let entity = self.add_block(page, kind, chain.next()?, None);
                if kind == "reply" {
                    if let Some(text) = str_at("promptText") {
                        self.assert(&entity, "reply/prompt-text", string_data(text));
                    }
                    if let Some(text) = str_at("buttonText") {
                        self.assert(&entity, "reply/button-text", string_data(text));
                    }
                    if inner.get("showPublicationTheme") == Some(&Value::Bool(false)) {
                        self.assert(&entity, "reply/show-publication-theme", boolean_data(false));
                    }
                } else {
                    if let Some(audience @ ("follows" | "followers")) = str_at("audience") {
                        self.assert(&entity, "questions/audience", string_data(audience));
                    }
                    if let Some(text) = str_at("buttonText") {
                        self.assert(&entity, "questions/button-text", string_data(text));
                    }
                    if inner.get("showPending") == Some(&Value::Bool(true)) {
                        self.assert(&entity, "questions/show-pending", boolean_data(true));
                    }
                }
                if inner.get("key").is_some() {
                    self.warn(
                        kind,
                        &entity,
                        "record `key` is a stale source-draft entity reference — not authored (the new entity id is the live key)",
                    );
                }
                self.blocks_authored += 1;
            }
            "membersOnlyDelimiter" => {
                let entity = self.add_block(page, "members-only-delimiter", chain.next()?, None);
                let audience = str_at("audience").filter(|a| ["subscribers", "paid", "tiers"].contains(a)).unwrap_or("paid");
                self.assert(&entity, "block/members-only-audience", string_data(audience));
                if let Some(tiers) = inner.get("tierIds").and_then(Value::as_array) {
                    for tier in tiers.iter().filter_map(Value::as_str) {
                        let id = self.ids.next().expect("id iterator exhausted");
                        self.assert_id(&entity, "block/members-only-tier", string_data(tier), id);
                    }
                }
                self.blocks_authored += 1;
            }
            "postsList" => {
                let entity = self.add_block(page, "posts-list", chain.next()?, None);
                if let Some(tags) = inner.get("filterByTags").and_then(Value::as_array) {
                    for tag in tags.iter().filter_map(Value::as_str) {
                        let id = self.ids.next().expect("id iterator exhausted");
                        self.assert_id(&entity, "posts-list/filter-tag", string_data(tag), id);
                    }
                }
                if let Some(value) = inner.get("highlightFirstPost").and_then(Value::as_bool) {
                    self.assert(&entity, "posts-list/highlight-first-post", boolean_data(value));
                }
                if let Some(limit) = inner.get("limit").and_then(Value::as_i64).filter(|l| *l > 0) {
                    self.assert(&entity, "posts-list/limit", number_data(limit as f64));
                }
                if let Some(view) = str_at("view").filter(|v| ["small", "medium", "chapter"].contains(v)) {
                    self.assert(&entity, "posts-list/view", union_data("posts-list-view-union", view));
                }
                if inner.get("showPageCount").and_then(Value::as_bool).is_some() {
                    let value = inner.get("showPageCount").and_then(Value::as_bool).unwrap_or(false);
                    self.assert(&entity, "posts-list/show-page-count", boolean_data(value));
                }
                if inner.get("readerControls") == Some(&Value::Bool(true)) {
                    self.assert(&entity, "posts-list/reader-controls", boolean_data(true));
                    for (field, attribute) in [
                        ("readerSearch", "posts-list/reader-search"),
                        ("readerTagFilter", "posts-list/reader-tag-filter"),
                        ("readerSort", "posts-list/reader-sort"),
                    ] {
                        if let Some(value) = inner.get(field).and_then(Value::as_bool) {
                            self.assert(&entity, attribute, boolean_data(value));
                        }
                    }
                }
                self.blocks_authored += 1;
            }
            "website" => {
                let Some(src) = str_at("src") else {
                    self.skip(kind, page, "website without src — block dropped");
                    return Ok(());
                };
                let entity = self.add_block(page, "link", chain.next()?, None);
                self.assert(&entity, "link/url", string_data(src));
                if let Some(title) = str_at("title") {
                    self.assert(&entity, "link/title", string_data(title));
                }
                if let Some(description) = str_at("description") {
                    self.assert(&entity, "link/description", string_data(description));
                }
                if inner.get("previewImage").is_some() {
                    self.warn(kind, &entity, "previewImage needs a PDS upload — not authored");
                }
                self.blocks_authored += 1;
            }
            "iframe" | "html" => {
                let block_type = if kind == "html" { "html" } else { "embed" };
                let entity = self.add_block(page, block_type, chain.next()?, None);
                if let Some(url) = str_at("url") {
                    self.assert(&entity, "embed/url", string_data(url));
                }
                if let Some(html) = str_at("html") {
                    self.assert(&entity, "embed/html", string_data(html));
                }
                if let Some(height) = inner.get("height").and_then(Value::as_i64) {
                    self.assert(&entity, "embed/height", number_data(height as f64));
                }
                if let Some(ratio) = inner.get("aspectRatio") {
                    match (ratio.get("width").and_then(Value::as_i64), ratio.get("height").and_then(Value::as_i64)) {
                        (Some(w), Some(h)) => {
                            self.assert(&entity, "embed/aspect-ratio", string_data(&format!("{w}/{h}")));
                        }
                        _ => self.warn(kind, &entity, "aspectRatio without width/height — not authored"),
                    }
                }
                self.blocks_authored += 1;
            }
            "orderedList" | "unorderedList" => {
                let style = if kind == "orderedList" { "ordered" } else { "unordered" };
                let items = inner.get("children").and_then(Value::as_array).cloned().unwrap_or_default();
                let start_index = inner.get("startIndex").and_then(Value::as_i64);
                self.author_list_items(&items, style, page, chain, true, start_index)?;
                self.blocks_authored += 1;
            }
            "page" => {
                self.author_subpage(inner, page, chain, pages, consumed)?;
                self.blocks_authored += 1;
            }
            "poll" => {
                let Some(poll_ref) = inner.get("pollRef") else {
                    self.skip(kind, page, "poll without pollRef — block dropped");
                    return Ok(());
                };
                let Some(uri) = poll_ref.get("uri").and_then(Value::as_str) else {
                    self.skip(kind, page, "poll pollRef without uri — block dropped");
                    return Ok(());
                };
                let Some(options) = self.options.poll_definitions.get(uri).cloned() else {
                    self.skip(
                        kind,
                        page,
                        format!("poll definition {uri} not resolved (pass its option texts via the poll map) — block dropped"),
                    );
                    return Ok(());
                };
                let entity = self.add_block(page, "poll", chain.next()?, None);
                let mut option_chain = PositionChain::new(None);
                for text in options {
                    let option_entity = self.ids.next().expect("id iterator exhausted");
                    self.create_entity(&option_entity);
                    self.assert(&option_entity, "poll-option/name", string_data(&text));
                    let id = self.ids.next().expect("id iterator exhausted");
                    self.assert_id(
                        &entity,
                        "poll/options",
                        json!({"type": "ordered-reference", "value": option_entity, "position": option_chain.next()?}),
                        id,
                    );
                }
                self.blocks_authored += 1;
            }
            other => {
                let reason = match other {
                    "image" => "image blocks need the storage upload surface — not authorable in v0",
                    "imageGallery" => "image galleries need per-image uploads — not authorable in v0",
                    "drawing" => "drawing authoring deferred (v1; the record data is all present)",
                    "embeddedCanvas" => "embedded canvas targets a canvas page (deferred in v0)",
                    _ => "unknown block type — dropped",
                };
                self.skip(other, page, reason);
            }
        }
        Ok(())
    }

    /// List items: each is a card/block child of `parent` (the page for a
    /// top-level run, the enclosing item entity for nested ones), every
    /// addBlock carrying its list data so the block is never briefly a
    /// non-list block. Nested runs hang off the item entity too — same
    /// style under `children`, other style under `orderedListChildren`/
    /// `unorderedListChildren` (one shared position chain per item).
    fn author_list_items(
        &mut self,
        items: &[Value],
        style: &'static str,
        parent: &str,
        chain: &mut PositionChain,
        top_level: bool,
        start_index: Option<i64>,
    ) -> Result<(), Error> {
        for (i, item) in items.iter().enumerate() {
            let Some(content) = item.get("content") else {
                self.skip("listItem", parent, "list item without content — item dropped");
                continue;
            };
            let kind = content.get("$type").and_then(Value::as_str).unwrap_or_default();
            if !matches!(kind, "pub.leaflet.blocks.text" | "pub.leaflet.blocks.header") {
                self.skip(kind, parent, "list item content must be text or header — item dropped");
                continue;
            }
            let checked = item.get("checked").and_then(Value::as_bool);
            let mut list = json!({"listStyle": style});
            if let Some(checked) = checked {
                list["checklist"] = json!(checked);
            }
            let (block_type, plaintext, facets) = if kind == "pub.leaflet.blocks.header" {
                (
                    "heading",
                    content.get("plaintext").and_then(Value::as_str).unwrap_or_default(),
                    content.get("facets"),
                )
            } else {
                (
                    "text",
                    content.get("plaintext").and_then(Value::as_str).unwrap_or_default(),
                    content.get("facets"),
                )
            };
            let entity = self.add_block(parent, block_type, chain.next()?, Some(list));
            if block_type == "heading" {
                let level = content.get("level").and_then(Value::as_i64).unwrap_or(1).clamp(1, 6);
                self.assert(&entity, "block/heading-level", number_data(level as f64));
            }
            if top_level
                && i == 0
                && style == "ordered"
                && let Some(start) = start_index.filter(|s| *s != 1)
            {
                self.assert(&entity, "block/list-number", number_data(start as f64));
            }
            self.assert_block_text(&entity, &format!("list item {plaintext:?}"), plaintext, facets)?;
            self.blocks_authored += 1;

            // Nested runs (same style under `children`, other style under
            // the cross field — an orderedList/unorderedList *object*
            // wrapping its own children) — one chain per item.
            let mut nested = PositionChain::new(None);
            if let Some(children) = item.get("children").and_then(Value::as_array) {
                self.author_list_items(children, style, &entity, &mut nested, false, item.get("startIndex").and_then(Value::as_i64))?;
            }
            let cross_field = if style == "ordered" { "unorderedListChildren" } else { "orderedListChildren" };
            let cross_style: &'static str = if style == "ordered" { "unordered" } else { "ordered" };
            if let Some(wrapper) = item.get(cross_field) {
                let children = wrapper.get("children").and_then(Value::as_array).cloned().unwrap_or_default();
                let start = wrapper.get("startIndex").and_then(Value::as_i64);
                self.author_list_items(&children, cross_style, &entity, &mut nested, false, start)?;
            }
        }
        Ok(())
    }

    /// A `blocks.page` card: the card block in the parent flow, then the
    /// target page entity, its `root/page` link + `page/type`, the
    /// `block/card` reference, and the page's own blocks.
    fn author_subpage(&mut self, inner: &Value, page: &str, chain: &mut PositionChain, pages: &[Value], consumed: &mut Vec<String>) -> Result<(), Error> {
        let Some(id) = inner.get("id").and_then(Value::as_str) else {
            self.skip("page", page, "page block without id — block dropped");
            return Ok(());
        };
        let Some(entry) = pages.iter().find(|p| p.get("id").and_then(Value::as_str) == Some(id)) else {
            self.skip(
                "page",
                page,
                format!("page block references {id:?} but no pages[] entry carries that id — block dropped"),
            );
            return Ok(());
        };
        if consumed.contains(&id.to_string()) {
            self.skip(
                "page",
                page,
                format!("page {id:?} already authored once (records are acyclic) — duplicate card dropped"),
            );
            return Ok(());
        }
        consumed.push(id.to_string());

        let card_entity = self.add_block(page, "card", chain.next()?, None);
        let page_entity = self.ids.next().expect("id iterator exhausted");
        self.create_entity(&page_entity);
        let page_position = self.page_chain.next()?;
        let root_page_fact_id = self.ids.next().expect("id iterator exhausted");
        self.assert_id(
            &self.options.root_entity,
            "root/page",
            json!({"type": "ordered-reference", "value": page_entity, "position": page_position}),
            root_page_fact_id,
        );
        self.assert(&page_entity, "page/type", union_data("page-type-union", "doc"));
        self.assert(&card_entity, "block/card", json!({"type": "reference", "value": page_entity}));
        if inner.get("display").and_then(Value::as_str) == Some("compact") {
            self.assert(&card_entity, "page-link/display", union_data("page-link-display-union", "compact"));
        }
        let blocks = entry.get("blocks").and_then(Value::as_array).cloned().unwrap_or_default();
        let mut sub_chain = PositionChain::new(None);
        self.author_blocks(&blocks, &page_entity, &mut sub_chain, pages, consumed)?;
        Ok(())
    }
}

/// Build the mutation batch for `record` against the target draft
/// described by `options`. `ids` mints every caller-generated uuid v7
/// (fact + entity ids, in deterministic order). Structural record
/// problems (no pages, invalid facet spans) are [`Error::Structure`];
/// everything unmappable warns and skips.
pub fn author(record: &Value, options: &AuthorOptions, ids: &mut dyn Iterator<Item = String>) -> Result<Authored, Error> {
    let pages = record
        .pointer("/content/pages")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Structure("record has no content.pages array".to_string()))?;
    let start = pages
        .first()
        .ok_or_else(|| Error::Structure("record has an empty content.pages array".to_string()))?;

    let mut builder = Builder {
        options,
        ids,
        mutations: Vec::new(),
        warns: Vec::new(),
        blocks_authored: 0,
        blocks_skipped: 0,
        page_chain: PositionChain::new(options.last_page_position.clone()),
    };
    let mut chain = PositionChain::new(options.last_block_position.clone());
    let mut consumed: Vec<String> = Vec::new();
    let blocks = start.get("blocks").and_then(Value::as_array).cloned().unwrap_or_default();
    builder.author_blocks(&blocks, &options.page_entity, &mut chain, pages, &mut consumed)?;

    // Orphan pages: entries no card block referenced.
    for entry in pages.iter().skip(1) {
        let id = entry.get("id").and_then(Value::as_str).unwrap_or("?");
        if !consumed.contains(&id.to_string()) {
            builder.warn("page", id, "record page has no referencing card block — not authored");
        }
    }

    // The tail: updateLeafletMetadata {title, description?}.
    let title = record.get("title").and_then(Value::as_str).unwrap_or_default();
    let mut args = json!({"title": title});
    if let Some(description) = record.get("description").and_then(Value::as_str) {
        args["description"] = json!(description);
    }
    builder.push("updateLeafletMetadata", args);

    Ok(Authored {
        last_block_position: chain.last,
        last_page_position: builder.page_chain.last.clone(),
        mutations: builder.mutations,
        warns: builder.warns,
        blocks_authored: builder.blocks_authored,
        blocks_skipped: builder.blocks_skipped,
    })
}

impl AuthorOptions {
    /// The structural members (root entity, start page, permission set);
    /// callers fill the position seeds and session parameters.
    pub fn new(root_entity: String, page_entity: String, permission_set: String) -> Self {
        Self {
            root_entity,
            page_entity,
            permission_set,
            last_page_position: None,
            last_block_position: None,
            yjs_client_id: 0,
            client_id: String::new(),
            mutation_id_base: 1,
            timestamp_base: 0,
            poll_definitions: HashMap::new(),
        }
    }
}

// ── target resolution (from a read-only pull) ───────────────────────────────

use crate::facts::{FactData, FactRow, ScanIndex};

/// What a pull says about the draft being authored into.
#[derive(Debug, Clone, PartialEq)]
pub struct TargetDraft {
    /// The start page entity (first `root/page` by position).
    pub page_entity: String,
    /// Its current `card/block` count.
    pub block_count: usize,
    /// Its current last block position, when any.
    pub last_block_position: Option<String>,
    /// The draft's last `root/page` position, when any.
    pub last_page_position: Option<String>,
    /// How many `root/page` facts exist (>1 warns: publication nav).
    pub root_page_count: usize,
}

/// Resolve the authoring target from a pulled fact graph — the same
/// start-page rule extraction uses (first `root/page` by position, fact
/// id tiebreak).
pub fn resolve_target(root_entity: &str, facts: &[FactRow]) -> Result<TargetDraft, Error> {
    let scan = ScanIndex::from_facts(facts.to_vec());
    let mut root_pages: Vec<(String, String, String)> = scan
        .eav(root_entity, "root/page")
        .iter()
        .filter_map(|fact| match &fact.data {
            FactData::OrderedReference { value, position } => Some((fact.id.clone(), position.clone(), value.clone())),
            _ => None,
        })
        .collect();
    if root_pages.is_empty() {
        return Err(Error::Structure(format!(
            "draft root {root_entity} has no root/page fact — cannot resolve the start page"
        )));
    }
    crate::fractional::sort_by_position(&mut root_pages);
    let page_entity = root_pages[0].2.clone();
    let last_page_position = root_pages
        .iter()
        .map(|(_, position, _)| position.clone())
        .max_by(|a, b| crate::fractional::compare_keys(a, b));

    let mut blocks: Vec<(String, String, String)> = scan
        .eav(&page_entity, "card/block")
        .iter()
        .filter_map(|fact| match &fact.data {
            FactData::OrderedReference { value, position } => Some((fact.id.clone(), position.clone(), value.clone())),
            _ => None,
        })
        .collect();
    crate::fractional::sort_by_position(&mut blocks);
    let block_count = blocks.len();
    let last_block_position = blocks
        .iter()
        .map(|(_, position, _)| position.clone())
        .max_by(|a, b| crate::fractional::compare_keys(a, b));
    Ok(TargetDraft {
        page_entity,
        block_count,
        last_block_position,
        last_page_position,
        root_page_count: root_pages.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yjs::decode_block_text;
    use serde_json::json;

    macro_rules! exp02_fixture {
        ($file:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../experiments/02-draft-internals/fixtures/",
                $file
            ))
        };
    }

    /// The experiment-02 target shape: page entity …003, permission set
    /// …013, root …002 — the same ids the committed push fixture uses, so
    /// the minimal record authors into the exact fixture batch.
    fn fixture_options() -> AuthorOptions {
        AuthorOptions {
            root_entity: "01999000-0000-7aaa-8000-000000000002".into(),
            page_entity: "01999000-0000-7aaa-8000-000000000003".into(),
            permission_set: "01999000-0000-7aaa-8000-000000000013".into(),
            last_page_position: None,
            last_block_position: None,
            yjs_client_id: 979_896_792,
            client_id: "springleaf-client-1".into(),
            mutation_id_base: 1,
            timestamp_base: 1_794_000_000_000,
            poll_definitions: HashMap::new(),
        }
    }

    fn id_pool() -> Box<dyn Iterator<Item = String>> {
        Box::new(
            [
                "01999000-0000-7aaa-8000-000000000031",
                "01999000-0000-7aaa-8000-000000000032",
                "01999000-0000-7aaa-8000-000000000033",
                "01999000-0000-7aaa-8000-000000000034",
                // subpage entity + root/page fact id
                "01999000-0000-7aaa-8000-000000000041",
                "01999000-0000-7aaa-8000-000000000042",
                "01999000-0000-7aaa-8000-000000000043",
                // poll/bsky extras
                "01999000-0000-7aaa-8000-000000000051",
                "01999000-0000-7aaa-8000-000000000052",
                "01999000-0000-7aaa-8000-000000000053",
                "01999000-0000-7aaa-8000-000000000054",
                "01999000-0000-7aaa-8000-000000000055",
                "01999000-0000-7aaa-8000-000000000056",
                "01999000-0000-7aaa-8000-000000000057",
                "01999000-0000-7aaa-8000-000000000058",
            ]
            .iter()
            .map(|s| s.to_string()),
        )
    }

    fn block(kind: &str, mut inner: Value) -> Value {
        if let Some(map) = inner.as_object_mut() {
            map.insert("$type".into(), json!(format!("pub.leaflet.blocks.{kind}")));
        }
        json!({"$type": "pub.leaflet.pages.linearDocument#block", "block": inner})
    }

    fn record_with(blocks: Vec<Value>) -> Value {
        json!({
            "$type": "site.standard.document",
            "title": "Synthetic title",
            "description": "Synthetic description",
            "content": {"$type": "pub.leaflet.content", "pages": [
                {"$type": "pub.leaflet.pages.linearDocument", "id": "page-0", "blocks": blocks}
            ]}
        })
    }

    /// Golden parity with experiments/02's committed minimal-authoring
    /// fixture: an empty-target authoring of "h1 title + one text block"
    /// reproduces its six-mutation batch exactly (names, entities,
    /// attributes, data payloads — including the byte-identical yjs
    /// block/text values — and positions a0/a1), with the tail
    /// updateLeafletMetadata.
    #[test]
    fn minimal_record_reproduces_the_experiment02_batch() {
        let record = record_with(vec![
            block("header", json!({"level": 1, "plaintext": "Synthetic title"})),
            block("text", json!({"plaintext": "Synthetic body copy"})),
        ]);
        let options = fixture_options();
        let mut ids = id_pool();
        let authored = author(&record, &options, &mut *ids).unwrap();
        assert_eq!(authored.blocks_authored, 2);
        assert!(authored.warns.is_empty(), "{:?}", authored.warns);

        let fixture: Value = serde_json::from_str(exp02_fixture!("push-minimal-authoring.json")).unwrap();
        let fixture_mutations = fixture["request"]["pushRequest"]["mutations"].as_array().unwrap();
        // The fixture carries six mutations; ours adds a description to
        // the tail metadata mutation (the record has one).
        assert_eq!(authored.mutations.len(), fixture_mutations.len());
        for (i, mine) in authored.mutations.iter().enumerate() {
            let want = &fixture_mutations[i];
            assert_eq!(mine.id, want["id"], "mutation {i} id");
            assert_eq!(mine.name, want["name"], "mutation {i} name");
            assert_eq!(mine.client_id, want["clientID"], "mutation {i} clientID");
            assert_eq!(mine.timestamp, want["timestamp"], "mutation {i} timestamp");
            let mine_value = serde_json::to_value(mine).unwrap();
            if i == authored.mutations.len() - 1 {
                // Tail: title matches the fixture; description rides along.
                assert_eq!(mine_value["args"]["title"], want["args"]["title"]);
                assert_eq!(mine_value["args"]["description"], json!("Synthetic description"));
            } else {
                assert_eq!(mine_value["args"], want["args"], "mutation {i} args");
            }
        }
        // Positions ascending from empty: a0, a1.
        let positions: Vec<&str> = authored
            .mutations
            .iter()
            .filter(|m| m.name == "addBlock")
            .map(|m| m.args["position"].as_str().unwrap())
            .collect();
        assert_eq!(positions, vec!["a0", "a1"]);
        assert_eq!(authored.last_block_position.as_deref(), Some("a1"));
        // Ids strictly increasing from the base (replay-safe ordering).
        let mutation_ids: Vec<u64> = authored.mutations.iter().map(|m| m.id).collect();
        assert_eq!(mutation_ids, (1..=authored.mutations.len() as u64).collect::<Vec<_>>());
    }

    /// Authoring into a NON-empty draft appends after the existing last
    /// key — never touches or reorders what is there.
    #[test]
    fn appends_continue_from_the_drafts_last_position() {
        let record = record_with(vec![block("text", json!({"plaintext": "one"})), block("text", json!({"plaintext": "two"}))]);
        let mut options = fixture_options();
        options.last_block_position = Some("a1V".into());
        let mut ids = id_pool();
        let authored = author(&record, &options, &mut *ids).unwrap();
        let positions: Vec<&str> = authored
            .mutations
            .iter()
            .filter(|m| m.name == "addBlock")
            .map(|m| m.args["position"].as_str().unwrap())
            .collect();
        assert_eq!(positions, vec!["a2", "a3"], "appended strictly after a1V");
        assert!(positions.iter().all(|p| p.as_bytes() > b"a1V".as_slice()));
    }

    /// Facet authoring: the pushed block/text decodes back to exactly the
    /// record's plaintext + facets (link, bold, code over non-ASCII text).
    #[test]
    fn facet_blocks_encode_the_record_text_exactly() {
        let plaintext = "É see the Leaflet homepage — bold claim, inline code.";
        let record = record_with(vec![block(
            "text",
            json!({
                "plaintext": plaintext,
                "facets": [
                    {"index": {"byteStart": 2, "byteEnd": 24}, "features": [{"$type": "pub.leaflet.richtext.facet#link", "uri": "https://leaflet.pub"}]},
                    {"index": {"byteStart": 27, "byteEnd": 37}, "features": [{"$type": "pub.leaflet.richtext.facet#bold"}]},
                    {"index": {"byteStart": 39, "byteEnd": 44}, "features": [{"$type": "pub.leaflet.richtext.facet#code"}]},
                ]
            }),
        )]);
        let options = fixture_options();
        let mut ids = id_pool();
        let authored = author(&record, &options, &mut *ids).unwrap();
        let text_mutation = authored
            .mutations
            .iter()
            .find(|m| m.name == "assertFact" && m.args["attribute"] == "block/text")
            .expect("block/text asserted");
        assert_eq!(text_mutation.args["data"]["type"], json!("text"));
        let value = text_mutation.args["data"]["value"].as_str().unwrap();
        let decoded = decode_block_text(value).unwrap();
        assert_eq!(decoded.plaintext, plaintext);
        assert_eq!(decoded.facets.len(), 3);
        assert!(matches!(&decoded.facets[0].marks[..], [Mark::Link(uri)] if uri == "https://leaflet.pub"));
        assert_eq!(decoded.facets[0].byte_start, 2);
        assert!(matches!(&decoded.facets[1].marks[..], [Mark::Bold]));
        assert!(matches!(&decoded.facets[2].marks[..], [Mark::Code]));
    }

    /// List folding inverse: an orderedList with a start index, a checked
    /// second item, and a nested run — exact addBlock/assertFact sequence
    /// with the right parents, list args, and per-parent position chains.
    #[test]
    fn list_blocks_fold_back_into_card_block_children() {
        let record = record_with(vec![block(
            "orderedList",
            json!({
                "startIndex": 3,
                "children": [
                    {"content": {"$type": "pub.leaflet.blocks.text", "plaintext": "first"}},
                    {
                        "content": {"$type": "pub.leaflet.blocks.header", "plaintext": "second", "level": 2},
                        "checked": true,
                        "unorderedListChildren": {
                            "$type": "pub.leaflet.blocks.unorderedList",
                            "children": [{"content": {"$type": "pub.leaflet.blocks.text", "plaintext": "nested"}}]
                        }
                    },
                ]
            }),
        )]);
        let options = fixture_options();
        let mut ids = id_pool();
        let authored = author(&record, &options, &mut *ids).unwrap();
        assert_eq!(authored.blocks_authored, 4, "the list block + three items");

        let page = options.page_entity.as_str();
        let rows: Vec<(String, Value)> = authored.mutations.iter().map(|m| (m.name.clone(), m.args.clone())).collect();
        // Item 1: addBlock on the page with the ordered list arg.
        assert_eq!(rows[0].0, "addBlock");
        assert_eq!(rows[0].1["parent"], json!(page));
        assert_eq!(rows[0].1["position"], json!("a0"));
        assert_eq!(rows[0].1["list"], json!({"listStyle": "ordered"}));
        let first_entity = rows[0].1["newEntityID"].as_str().unwrap().to_string();
        // startIndex 3 lands on the run's first item only.
        assert_eq!(
            rows[1],
            (
                "assertFact".into(),
                json!({"entity": first_entity, "attribute": "block/list-number", "data": {"type": "number", "value": 3}})
            )
        );
        let (name, args) = (&rows[2].0, &rows[2].1);
        assert_eq!((name.as_str(), args["attribute"].as_str()), ("assertFact", Some("block/text")));
        // Item 2: header on the page, checklist arg carries the checked value.
        assert_eq!(rows[3].1["parent"], json!(page));
        assert_eq!(rows[3].1["position"], json!("a1"));
        assert_eq!(rows[3].1["list"], json!({"listStyle": "ordered", "checklist": true}));
        let second_entity = rows[3].1["newEntityID"].as_str().unwrap().to_string();
        assert_eq!(rows[4].1["attribute"], json!("block/heading-level"));
        assert_eq!(rows[4].1["data"], json!({"type": "number", "value": 2}));
        // Nested unordered item: parent is the SECOND ITEM's entity, fresh
        // position chain (a0), style flips.
        assert_eq!(rows[6].1["parent"], json!(second_entity));
        assert_eq!(rows[6].1["position"], json!("a0"));
        assert_eq!(rows[6].1["list"], json!({"listStyle": "unordered"}));
        // Positions ascending per parent.
        assert_eq!(authored.last_block_position.as_deref(), Some("a1"), "page-level chain");
    }

    /// Subpages: the card block in the parent flow, then createEntity,
    /// root/page (appended after the draft's existing pages), page/type,
    /// block/card — and the subpage's own blocks in a fresh chain. Orphan
    /// pages (no referencing card) warn; embeddedCanvas skips.
    #[test]
    fn subpages_author_with_root_page_links_and_orphans_warn() {
        let mut record = record_with(vec![block("page", json!({"id": "sub-1"})), block("embeddedCanvas", json!({"id": "canvas-9"}))]);
        record["content"]["pages"]
            .as_array_mut()
            .unwrap()
            .push(json!({"$type": "pub.leaflet.pages.linearDocument", "id": "sub-1", "blocks": [
                block("text", json!({"plaintext": "inner"}))
            ]}));
        record["content"]["pages"]
            .as_array_mut()
            .unwrap()
            .push(json!({"$type": "pub.leaflet.pages.linearDocument", "id": "orphan-2", "blocks": []}));

        let mut options = fixture_options();
        options.last_page_position = Some("a0".into());
        let mut ids = id_pool();
        let authored = author(&record, &options, &mut *ids).unwrap();

        let names: Vec<(&str, &Value)> = authored.mutations.iter().map(|m| (m.name.as_str(), &m.args)).collect();
        let expect = [
            "addBlock",     // the card
            "createEntity", // the subpage entity
            "assertFact",   // root/page (ordered-reference, appended)
            "assertFact",   // page/type doc
            "assertFact",   // block/card reference
            "addBlock",     // the inner text block (fresh chain)
            "assertFact",   // its block/text
            "updateLeafletMetadata",
        ];
        let got: Vec<&str> = names.iter().map(|(n, _)| *n).collect();
        assert_eq!(got, expect, "{got:?}");

        let page_entity = names[1].1["entityID"].as_str().unwrap();
        // root/page on the ROOT entity, appended after a0, with a fact id.
        assert_eq!(names[2].1["entity"], json!(options.root_entity));
        assert_eq!(names[2].1["attribute"], json!("root/page"));
        assert_eq!(names[2].1["data"], json!({"type": "ordered-reference", "value": page_entity, "position": "a1"}));
        assert!(names[2].1["id"].is_string(), "cardinality-many: caller-chosen fact id");
        assert_eq!(
            names[3].1,
            &json!({"entity": page_entity, "attribute": "page/type", "data": {"type": "page-type-union", "value": "doc"}})
        );
        assert_eq!(names[4].1["data"], json!({"type": "reference", "value": page_entity}));
        // The subpage's inner block: fresh position chain.
        assert_eq!(names[5].1["parent"], json!(page_entity));
        assert_eq!(names[5].1["position"], json!("a0"));
        assert_eq!(authored.last_page_position.as_deref(), Some("a1"));

        let reasons: Vec<&str> = authored.warns.iter().map(|w| w.reason.as_str()).collect();
        assert!(reasons.iter().any(|r| r.contains("embedded canvas")), "{reasons:?}");
        assert!(reasons.iter().any(|r| r.contains("no referencing card block")), "{reasons:?}");
        assert_eq!(authored.blocks_skipped, 1, "only the embeddedCanvas block skips");
    }

    /// The v0 skip set + poll resolution + bskyPost minimal blob.
    #[test]
    fn skips_warn_and_poll_resolves_through_the_caller_map() {
        let record = record_with(vec![
            block("image", json!({"image": {"ref": "bafy..."}})),
            block("drawing", json!({"strokes": []})),
            block(
                "poll",
                json!({"pollRef": {"uri": "at://did:plc:x/pub.leaflet.poll.definition/3poll", "cid": "bafy"}}),
            ),
            block(
                "bskyPost",
                json!({"postRef": {"uri": "at://did:plc:x/app.bsky.feed.post/3abc", "cid": "bafycid"}}),
            ),
        ]);
        let mut options = fixture_options();
        options
            .poll_definitions
            .insert("at://did:plc:x/pub.leaflet.poll.definition/3poll".into(), vec!["Yes".into(), "No".into()]);
        let mut ids = id_pool();
        let authored = author(&record, &options, &mut *ids).unwrap();
        assert_eq!(authored.blocks_skipped, 2);
        assert_eq!(authored.blocks_authored, 2, "poll + bskyPost author");
        // Poll: option entities + names + ordered poll/options facts.
        let poll_entity = authored.mutations[0].args["newEntityID"].as_str().unwrap().to_string();
        let names: Vec<&str> = authored.mutations.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "addBlock",
                "createEntity",
                "assertFact",
                "assertFact", // option 1 (name + options ref)
                "createEntity",
                "assertFact",
                "assertFact", // option 2
                "addBlock",
                "assertFact", // bsky post (thread blob)
                "updateLeafletMetadata",
            ],
            "{names:?}"
        );
        assert_eq!(authored.mutations[2].args["attribute"], json!("poll-option/name"));
        assert_eq!(authored.mutations[3].args["attribute"], json!("poll/options"));
        assert_eq!(authored.mutations[3].args["entity"], json!(poll_entity));
        assert_eq!(authored.mutations[3].args["data"]["position"], json!("a0"));
        assert_eq!(authored.mutations[6].args["data"]["position"], json!("a1"));
        // bskyPost: minimal thread blob.
        assert_eq!(
            authored.mutations[8].args["data"],
            json!({"type": "bluesky-post", "value": {"post": {"uri": "at://did:plc:x/app.bsky.feed.post/3abc", "cid": "bafycid"}}})
        );
        // Unresolved poll refs skip with a warn naming the uri.
        let record = record_with(vec![block(
            "poll",
            json!({"pollRef": {"uri": "at://did:plc:x/pub.leaflet.poll.definition/other", "cid": "bafy"}}),
        )]);
        let options = fixture_options();
        let mut ids = id_pool();
        let authored = author(&record, &options, &mut *ids).unwrap();
        assert_eq!(authored.blocks_skipped, 1);
        assert!(authored.warns[0].reason.contains("not resolved"), "{:?}", authored.warns[0]);
    }

    /// Target resolution against the export fixture's pulled fact graph:
    /// start page, existing block count, last positions.
    #[test]
    fn resolve_target_reads_the_pulled_draft_shape() {
        let fixture: Value = serde_json::from_str(exp02_fixture!("export-pull-doc.json")).unwrap();
        let snapshot = crate::pull::parse_pull_snapshot(&fixture["response"].to_string()).unwrap();
        let target = resolve_target("01999000-0000-7aaa-8000-000000000002", &snapshot.facts).unwrap();
        assert_eq!(target.page_entity, "01999000-0000-7aaa-8000-000000000003");
        assert_eq!(target.block_count, 2, "the fixture draft holds an h1 and a text block");
        assert_eq!(target.last_block_position.as_deref(), Some("a1"));
        assert_eq!(target.last_page_position.as_deref(), Some("a0"));
        assert_eq!(target.root_page_count, 1);
        // An empty page resolves with no positions; a root without pages
        // is a structural refusal.
        let empty_facts = vec![crate::facts::FactRow {
            id: "f1".into(),
            entity: "root-9".into(),
            attribute: "root/page".into(),
            data: crate::facts::FactData::parse(&json!({"type": "ordered-reference", "value": "page-9", "position": "a0"})),
        }];
        let empty = resolve_target("root-9", &empty_facts).unwrap();
        assert_eq!(empty.page_entity, "page-9");
        assert_eq!(empty.block_count, 0);
        assert_eq!(empty.last_block_position, None);
        assert!(matches!(resolve_target("no-such-root", &snapshot.facts), Err(Error::Structure(_))));
    }

    /// Structural refusals: no pages array, empty pages, invalid facet
    /// span — never a half-written batch.
    #[test]
    fn structural_errors_refuse_the_whole_batch() {
        let options = fixture_options();
        assert!(matches!(author(&json!({"content": {}}), &options, &mut *id_pool()), Err(Error::Structure(_))));
        assert!(matches!(
            author(&json!({"content": {"pages": []}}), &options, &mut *id_pool()),
            Err(Error::Structure(_))
        ));
        let bad_span = record_with(vec![block(
            "text",
            json!({"plaintext": "abc", "facets": [{"index": {"byteStart": 2, "byteEnd": 9}, "features": [{"$type": "pub.leaflet.richtext.facet#bold"}]}]}),
        )]);
        assert!(matches!(author(&bad_span, &options, &mut *id_pool()), Err(Error::Structure(_))));
    }
}
