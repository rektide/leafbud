//! Yjs v1 update decoder for `block/text` values — plaintext + facets.
//!
//! A `block/text` fact holds base64 of a Yjs v1 update encoding a
//! ProseMirror doc: named root `prosemirror` → `paragraph` YXmlElements →
//! YXmlText → ContentString items; a second named root `meta` (YMap) may
//! carry `schemaVersion` (`experiments/02-draft-internals/REPORT.md` §4,
//! `fixtures/yjs-samples.md`).
//!
//! Three layers, each ported from a named source:
//!
//! 1. **lib0 primitives + struct walk** — port of
//!    `experiments/02-draft-internals/scripts/yjs_decode.py` (unsigned
//!    varuint, sign-magnitude varint, varstring, `writeAny` tags 116–127;
//!    item `info` bits, content refs 0–9, delete set).
//! 2. **Item integration** — port of yjs v13.6.20 `Item.integrate`
//!    (`src/structs/Item.js`): true order inside a YXmlText is the
//!    linked-list order defined by `origin`/`rightOrigin` pointers, not
//!    clock order. Items link into per-parent lists with the yjs conflict
//!    scan (same-origin ties by client id); origins pointing mid-item split
//!    the item first (`getItemCleanEnd`/`getItemCleanStart` semantics); the
//!    delete set splits and marks tombstones. Without this step, edited or
//!    tombstoned text orders wrong.
//! 3. **Delta reconstruction** — mirrors `YJSFragmentToFacets`
//!    (`factsToPagesRecord.ts:1087-1133`): ContentFormat items toggle mark
//!    runs (null value = close), ContentString runs accumulate text, and
//!    runs with active marks become facets with **UTF-8 byte offsets**.
//!    `hard_break` XmlElements render as `\n`; `footnote` renders `*`,
//!    `didMention`/`atMention` render their `text` attribute (plaintext
//!    parity with `YJSFragmentToString`), with their *facets* deferred
//!    (reported in [`DecodedText::notes`] — v0 scope per the design).
//!
//! Tolerance contract: the 3-byte `b5ec6d` non-update observed on empty
//! blocks and **any** undecodable value surface as `Err` here;
//! [`crate::extract`] turns that into empty plaintext + a warn — a text
//! block never fails extraction.

use std::collections::{BTreeMap, HashMap, HashSet};

use base64::Engine as _;

/// Decode failure (malformed update, truncated varint, unknown tag…).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("yjs decode error at byte {position}: {message}")]
pub struct DecodeError {
    pub position: usize,
    pub message: String,
}

type Result<T> = std::result::Result<T, DecodeError>;

// ---------------------------------------------------------------------------
// lib0 primitives (yjs_decode.py Reader)
// ---------------------------------------------------------------------------

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn u8(&mut self) -> Result<u8> {
        let byte = *self.data.get(self.pos).ok_or_else(|| self.err("unexpected end of input"))?;
        self.pos += 1;
        Ok(byte)
    }

    /// lib0 unsigned varuint: 7 bits/byte, LSB first, `0x80` continuation.
    fn varuint(&mut self) -> Result<u64> {
        let mut n: u64 = 0;
        let mut mult: u64 = 1;
        loop {
            let byte = self.u8()?;
            n = n
                .checked_add(u64::from(byte & 0x7f).checked_mul(mult).ok_or_else(|| self.err("varuint overflow"))?)
                .ok_or_else(|| self.err("varuint overflow"))?;
            if byte < 0x80 {
                return Ok(n);
            }
            mult = mult.checked_mul(128).ok_or_else(|| self.err("varuint overflow"))?;
        }
    }

    /// lib0 sign-magnitude varint (`writeVarInt`): first byte `0x40` sign +
    /// 6 payload bits, then 7 bits/byte.
    fn varint(&mut self) -> Result<i64> {
        let b0 = self.u8()?;
        let sign = if b0 & 0x40 != 0 { -1 } else { 1 };
        let mut n = i64::from(b0 & 0x3f);
        let mut mult: i64 = 64;
        let mut b = b0;
        while b & 0x80 != 0 {
            b = self.u8()?;
            n = n
                .checked_add(i64::from(b & 0x7f).checked_mul(mult).ok_or_else(|| self.err("varint overflow"))?)
                .ok_or_else(|| self.err("varint overflow"))?;
            mult = mult.checked_mul(128).ok_or_else(|| self.err("varint overflow"))?;
        }
        Ok(sign * n)
    }

    fn varstring(&mut self) -> Result<String> {
        let len = self.varuint()? as usize;
        let bytes = self.varbuf_bytes(len)?;
        // yjs writes UTF-8; tolerate invalid sequences (replacement chars)
        // rather than failing a whole block over one bad run.
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    fn varbuf(&mut self) -> Result<Vec<u8>> {
        let len = self.varuint()? as usize;
        Ok(self.varbuf_bytes(len)?.to_vec())
    }

    fn varbuf_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .filter(|end| *end <= self.data.len())
            .ok_or_else(|| self.err("buffer extends past end of input"))?;
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    /// lib0 `writeAny` tags: 116 bytes, 117 array, 118 object, 119 string,
    /// 120 true, 121 false, 122 bigint, 123 float64, 124 float32, 125 int,
    /// 126 null, 127 undefined.
    fn any(&mut self) -> Result<AnyValue> {
        let tag = self.u8()?;
        match tag {
            119 => Ok(AnyValue::Str(self.varstring()?)),
            120 => Ok(AnyValue::Bool(true)),
            121 => Ok(AnyValue::Bool(false)),
            125 => Ok(AnyValue::Int(self.varint()?)),
            124 => {
                let bytes = self.varbuf_bytes(4)?;
                Ok(AnyValue::Float(f32::from_le_bytes(bytes.try_into().unwrap())))
            }
            123 => {
                let bytes = self.varbuf_bytes(8)?;
                Ok(AnyValue::Double(f64::from_le_bytes(bytes.try_into().unwrap())))
            }
            122 => {
                let bytes = self.varbuf_bytes(8)?;
                Ok(AnyValue::BigInt(i64::from_le_bytes(bytes.try_into().unwrap())))
            }
            117 => {
                let len = self.varuint()? as usize;
                let mut items = Vec::with_capacity(len);
                for _ in 0..len {
                    items.push(self.any()?);
                }
                Ok(AnyValue::Array(items))
            }
            118 => {
                let len = self.varuint()? as usize;
                let mut entries = Vec::with_capacity(len);
                for _ in 0..len {
                    let key = self.varstring()?;
                    let value = self.any()?;
                    entries.push((key, value));
                }
                Ok(AnyValue::Object(entries))
            }
            116 => Ok(AnyValue::Bytes(self.varbuf()?)),
            126 => Ok(AnyValue::Null),
            127 => Ok(AnyValue::Undefined),
            other => Err(self.err(format!("unknown writeAny tag {other}"))),
        }
    }

    fn err(&self, message: impl Into<String>) -> DecodeError {
        DecodeError {
            position: self.pos,
            message: message.into(),
        }
    }
}

/// A `writeAny` value.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyValue {
    Null,
    Undefined,
    Bool(bool),
    Int(i64),
    Float(f32),
    Double(f64),
    BigInt(i64),
    Str(String),
    Bytes(Vec<u8>),
    Array(Vec<AnyValue>),
    Object(Vec<(String, AnyValue)>),
}

impl AnyValue {
    /// Object member by key.
    fn member(&self, key: &str) -> Option<&AnyValue> {
        match self {
            AnyValue::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            AnyValue::Str(s) => Some(s),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Update structs (yjs_decode.py decode_update)
// ---------------------------------------------------------------------------

/// A struct id: client + clock (clock = the item's *start* clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    pub client: u64,
    pub clock: u64,
}

/// Item content, refs 0–9.
#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Gc { len: u64 },
    Deleted { len: u64 },
    Json(String),
    Binary(Vec<u8>),
    Str(String),
    Embed(String),
    Format { key: String, value: AnyValue },
    Type { type_ref: u8, name: Option<String> },
    Any(Vec<AnyValue>),
    Doc(String),
}

impl Content {
    /// Length in clock units. Strings count UTF-16 code units (JS `.length`
    /// — the unit yjs clock arithmetic runs on).
    fn clock_len(&self) -> u64 {
        match self {
            Content::Gc { len } | Content::Deleted { len } => *len,
            Content::Str(s) => s.encode_utf16().count() as u64,
            Content::Any(values) => values.len() as u64,
            _ => 1,
        }
    }
}

/// Content type refs (`ContentType` payload).
pub mod type_refs {
    pub const XML_ELEMENT: u8 = 3;
    pub const XML_TEXT: u8 = 6;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Parent {
    Named(String),
    Id(Id),
}

/// One decoded struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: Id,
    pub info: u8,
    pub origin: Option<Id>,
    pub right_origin: Option<Id>,
    /// `None` when the wire omits the parent (origin/rightOrigin set) —
    /// resolved from a neighbour during integration.
    pub parent: Option<Parent>,
    pub parent_sub: Option<String>,
    pub content: Content,
}

/// A decoded update: structs grouped by client (wire order) + delete set.
#[derive(Debug, Clone, Default)]
pub struct Update {
    pub clients: Vec<ClientStructs>,
    pub delete_set: Vec<(u64, Vec<(u64, u64)>)>,
}

#[derive(Debug, Clone)]
pub struct ClientStructs {
    pub client: u64,
    pub items: Vec<Item>,
}

/// Parse the update wire format (see `yjs_decode.py`'s layout docstring).
pub fn decode_update(bytes: &[u8]) -> Result<Update> {
    let mut r = Reader::new(bytes);
    let num_clients = r.varuint()? as usize;
    let mut clients = Vec::with_capacity(num_clients);
    for _ in 0..num_clients {
        let num_structs = r.varuint()? as usize;
        let client = r.varuint()?;
        let mut clock = r.varuint()?;
        let mut items = Vec::with_capacity(num_structs);
        for _ in 0..num_structs {
            let info = r.u8()?;
            let ref_ = info & 0x1f;
            let origin = (info & 0x80 != 0).then(|| read_id(&mut r)).transpose()?;
            let right_origin = (info & 0x40 != 0).then(|| read_id(&mut r)).transpose()?;
            let parent = if info & 0xc0 == 0 {
                if r.varuint()? == 1 {
                    Some(Parent::Named(r.varstring()?))
                } else {
                    Some(Parent::Id(read_id(&mut r)?))
                }
            } else {
                None
            };
            let parent_sub = (info & 0x20 != 0).then(|| r.varstring()).transpose()?;
            let content = read_content(&mut r, ref_)?;
            let item = Item {
                id: Id { client, clock },
                info,
                origin,
                right_origin,
                parent,
                parent_sub,
                content,
            };
            clock += item.content.clock_len();
            items.push(item);
        }
        clients.push(ClientStructs { client, items });
    }
    let num_ds_clients = r.varuint()? as usize;
    let mut delete_set = Vec::with_capacity(num_ds_clients);
    for _ in 0..num_ds_clients {
        let client = r.varuint()?;
        let num_ranges = r.varuint()? as usize;
        let mut ranges = Vec::with_capacity(num_ranges);
        for _ in 0..num_ranges {
            let clock = r.varuint()?;
            let len = r.varuint()?;
            ranges.push((clock, len));
        }
        delete_set.push((client, ranges));
    }
    Ok(Update { clients, delete_set })
}

fn read_id(r: &mut Reader) -> Result<Id> {
    Ok(Id {
        client: r.varuint()?,
        clock: r.varuint()?,
    })
}

fn read_content(r: &mut Reader, ref_: u8) -> Result<Content> {
    match ref_ {
        0 => Ok(Content::Gc { len: r.varuint()? }),
        1 => Ok(Content::Deleted { len: r.varuint()? }),
        2 => Ok(Content::Json(r.varstring()?)),
        3 => Ok(Content::Binary(r.varbuf()?)),
        4 => Ok(Content::Str(r.varstring()?)),
        5 => Ok(Content::Embed(r.varstring()?)),
        6 => {
            let key = r.varstring()?;
            let value = r.any()?;
            Ok(Content::Format { key, value })
        }
        7 => {
            let type_ref = r.varuint()? as u8;
            let name = (type_ref == type_refs::XML_ELEMENT).then(|| r.varstring()).transpose()?;
            Ok(Content::Type { type_ref, name })
        }
        8 => {
            let len = r.varuint()? as usize;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(r.any()?);
            }
            Ok(Content::Any(values))
        }
        9 => Ok(Content::Doc(r.varstring()?)),
        other => Err(r.err(format!("unknown content ref {other}"))),
    }
}

// ---------------------------------------------------------------------------
// Integration (yjs v13.6.20 Item.integrate)
// ---------------------------------------------------------------------------

/// Identifies a Yjs type: a named root (`prosemirror`, `meta`) or an item
/// id (the ContentType struct that owns the nested type).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TypeKey {
    Named(String),
    Id(Id),
}

#[derive(Debug, Clone)]
struct ItemExt {
    item: Item,
    left: Option<u32>,
    right: Option<u32>,
    deleted: bool,
}

/// An integrated update: items linked into per-type lists, tombstones
/// marked.
#[derive(Debug, Clone)]
pub struct Integrated {
    items: Vec<ItemExt>,
    /// Per type + optional map key: the first item of the list.
    starts: HashMap<(TypeKey, Option<String>), u32>,
    /// Per client: start clock → item index (for id resolution).
    by_clock: HashMap<u64, BTreeMap<u64, u32>>,
}

impl Integrated {
    /// Integrate every struct of `update` (wire order, with a pending queue
    /// for cross-client dependencies — the same retry shape yjs applies
    /// out-of-order structs with).
    pub fn integrate(update: &Update) -> Self {
        let mut doc = Integrated {
            items: Vec::new(),
            starts: HashMap::new(),
            by_clock: HashMap::new(),
        };
        for client in &update.clients {
            let clock_map = doc.by_clock.entry(client.client).or_default();
            for item in &client.items {
                let idx = doc.items.len() as u32;
                clock_map.insert(item.id.clock, idx);
                doc.items.push(ItemExt {
                    item: item.clone(),
                    left: None,
                    right: None,
                    deleted: false,
                });
            }
        }
        // Wire order; an item integrates once its origin/rightOrigin
        // neighbours (when present) are themselves integrated.
        let mut integrated: HashSet<u32> = HashSet::new();
        let mut pending: Vec<u32> = (0..doc.items.len() as u32).collect();
        loop {
            let before = pending.len();
            let mut remaining = Vec::new();
            for idx in pending {
                if doc.neighbours_ready(idx, &integrated) {
                    doc.integrate_item(idx);
                    integrated.insert(idx);
                } else {
                    remaining.push(idx);
                }
            }
            pending = remaining;
            if pending.is_empty() || pending.len() == before {
                break;
            }
        }
        // Unresolvable leftovers (circular/garbage references): best-effort
        // integration with missing neighbours treated as absent.
        for idx in pending {
            doc.integrate_item(idx);
        }
        for (client, ranges) in &update.delete_set {
            for &(clock, len) in ranges {
                doc.mark_deleted(Id { client: *client, clock }, len);
            }
        }
        doc
    }

    fn neighbours_ready(&self, idx: u32, integrated: &HashSet<u32>) -> bool {
        let item = &self.items[idx as usize].item;
        [&item.origin, &item.right_origin]
            .iter()
            .filter_map(|id| **id)
            .all(|id| self.containing(id).is_some_and(|n| integrated.contains(&n) || n == idx))
    }

    /// The item whose clock range contains `id.clock`.
    fn containing(&self, id: Id) -> Option<u32> {
        let clocks = self.by_clock.get(&id.client)?;
        let (&start, &idx) = clocks.range(..=id.clock).next_back()?;
        (id.clock < start + self.items[idx as usize].item.content.clock_len()).then_some(idx)
    }

    /// Split `idx` so a piece starts exactly at `at_clock`; returns the
    /// right piece's index. Mid-surrogate-pair string splits clamp to the
    /// enclosing char (rendering-equivalent).
    fn split(&mut self, idx: u32, at_clock: u64) -> u32 {
        let item = self.items[idx as usize].item.clone();
        let start = item.id.clock;
        let len = item.content.clock_len();
        debug_assert!(at_clock > start && at_clock < start + len, "split inside the item");
        let left_len = at_clock - start;
        let right_content = match item.content {
            Content::Str(ref s) => {
                // Cut at the UTF-16 boundary `left_len`; a mid-surrogate-pair
                // index clamps to the enclosing char (rendering-equivalent).
                let units = left_len as usize;
                let mut cut = s.len();
                let mut seen = 0usize;
                for (i, ch) in s.char_indices() {
                    if seen >= units {
                        cut = i;
                        break;
                    }
                    seen += ch.len_utf16();
                }
                let (left_str, right_str) = s.split_at(cut);
                self.items[idx as usize].item.content = Content::Str(left_str.to_string());
                Content::Str(right_str.to_string())
            }
            other => {
                let right = match other {
                    Content::Deleted { .. } => Content::Deleted { len: len - left_len },
                    Content::Gc { .. } => Content::Gc { len: len - left_len },
                    _ => unreachable!("only strings/deleted/gc span multiple clocks"),
                };
                self.items[idx as usize].item.content = match right {
                    Content::Deleted { .. } => Content::Deleted { len: left_len },
                    Content::Gc { .. } => Content::Gc { len: left_len },
                    _ => unreachable!(),
                };
                right
            }
        };
        let right_idx = self.items.len() as u32;
        let right = ItemExt {
            item: Item {
                id: Id {
                    client: item.id.client,
                    clock: at_clock,
                },
                info: item.info,
                origin: Some(Id {
                    client: item.id.client,
                    clock: at_clock - 1,
                }),
                right_origin: item.right_origin,
                parent: item.parent.clone(),
                parent_sub: item.parent_sub.clone(),
                content: right_content,
            },
            left: Some(idx),
            right: self.items[idx as usize].right,
            deleted: false,
        };
        self.items.push(right);
        self.by_clock.entry(item.id.client).or_default().insert(at_clock, right_idx);
        // Relink.
        let old_right = self.items[idx as usize].right;
        self.items[idx as usize].right = Some(right_idx);
        if let Some(next) = old_right {
            self.items[next as usize].left = Some(right_idx);
        }
        right_idx
    }

    /// Item ending exactly at `id.clock` (splits first when needed) —
    /// `getItemCleanEnd`.
    fn resolve_clean_end(&mut self, id: Id) -> Option<u32> {
        let idx = self.containing(id)?;
        let start = self.items[idx as usize].item.id.clock;
        let len = self.items[idx as usize].item.content.clock_len();
        if id.clock + 1 < start + len {
            let _ = self.split(idx, id.clock + 1);
        }
        Some(idx)
    }

    /// Item starting exactly at `id.clock` (splits first when needed) —
    /// `getItemCleanStart`.
    fn resolve_clean_start(&mut self, id: Id) -> Option<u32> {
        let idx = self.containing(id)?;
        let start = self.items[idx as usize].item.id.clock;
        if id.clock > start {
            return Some(self.split(idx, id.clock));
        }
        Some(idx)
    }

    /// The yjs integrate: conflict scan + linked-list insertion.
    fn integrate_item(&mut self, idx: u32) {
        let item = self.items[idx as usize].item.clone();
        let left = item.origin.and_then(|o| self.resolve_clean_end(o));
        let right = item.right_origin.and_then(|o| self.resolve_clean_start(o));

        // Resolve the parent from a neighbour when the wire omitted it.
        if self.items[idx as usize].item.parent.is_none() {
            let parent = left.or(right).and_then(|n| self.items[n as usize].item.parent.clone());
            self.items[idx as usize].item.parent = parent;
        }
        let Some(parent) = self.items[idx as usize].item.parent.clone() else {
            return; // orphan: no parent on the wire and none resolvable
        };
        let key = (
            match parent {
                Parent::Named(n) => TypeKey::Named(n),
                Parent::Id(id) => TypeKey::Id(id),
            },
            item.parent_sub.clone(),
        );

        // Conflict scan (Item.integrate): walk right from the left
        // neighbour (or the list start) past items that must stay before us.
        let mut l = left;
        let mut o = match l {
            Some(li) => self.items[li as usize].right,
            None => self.starts.get(&key).copied(),
        };
        let mut conflicting: HashSet<u32> = HashSet::new();
        let mut before_origin: HashSet<u32> = HashSet::new();
        while let Some(oi) = o {
            if Some(oi) == right {
                break;
            }
            before_origin.insert(oi);
            conflicting.insert(oi);
            let o_item = self.items[oi as usize].item.clone();
            if item.origin == o_item.origin {
                if o_item.id.client < item.id.client {
                    l = Some(oi);
                    conflicting.clear();
                } else if item.right_origin == o_item.right_origin {
                    break;
                }
            } else if let Some(o_origin) = o_item.origin
                && let Some(containing) = self.containing(o_origin)
                && before_origin.contains(&containing)
            {
                if !conflicting.contains(&containing) {
                    l = Some(oi);
                    conflicting.clear();
                }
            } else {
                break;
            }
            o = self.items[oi as usize].right;
        }

        // Link.
        if let Some(li) = l {
            let old_right = self.items[li as usize].right;
            self.items[idx as usize].left = Some(li);
            self.items[idx as usize].right = old_right;
            self.items[li as usize].right = Some(idx);
        } else {
            let old_start = self.starts.get(&key).copied();
            self.items[idx as usize].left = None;
            self.items[idx as usize].right = old_start;
            self.starts.insert(key, idx);
        }
        if let Some(ri) = self.items[idx as usize].right {
            self.items[ri as usize].left = Some(idx);
        }
    }

    /// Mark `[clock, clock+len)` deleted, splitting at the range boundaries
    /// first so tombstones align with future clean splits.
    fn mark_deleted(&mut self, start: Id, len: u64) {
        let Some(first) = self.resolve_clean_start(start) else { return };
        let Some(last) = self.resolve_clean_end(Id {
            client: start.client,
            clock: start.clock + len.saturating_sub(1),
        }) else {
            return;
        };
        let mut current = Some(first);
        while let Some(idx) = current {
            self.items[idx as usize].deleted = true;
            if idx == last {
                break;
            }
            current = self.items[idx as usize].right;
        }
    }

    /// Non-deleted array children of a type, in integrated order.
    fn children(&self, key: &TypeKey) -> Vec<u32> {
        self.walk(&(key.clone(), None))
    }

    fn walk(&self, key: &(TypeKey, Option<String>)) -> Vec<u32> {
        let mut out = Vec::new();
        let mut current = self.starts.get(key).copied();
        while let Some(idx) = current {
            out.push(idx);
            current = self.items[idx as usize].right;
        }
        out
    }

    /// Current value of a map entry (parentSub list tail; deleted ⇒ none).
    fn map_value(&self, key: &TypeKey, name: &str) -> Option<&Content> {
        let list = self.walk(&(key.clone(), Some(name.to_string())));
        let tail = list.last()?;
        (!self.items[*tail as usize].deleted).then_some(&self.items[*tail as usize].item.content)
    }
}

// ---------------------------------------------------------------------------
// Delta reconstruction (YJSFragmentToFacets parity)
// ---------------------------------------------------------------------------

/// One facet feature in converter-neutral form (mapped to the generated
/// lexicon facets by the extraction layer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    Bold,
    Italic,
    Code,
    Strikethrough,
    Underline,
    Highlight,
    Link(String),
}

/// A facet run: byte span over the paragraph plaintext + active marks.
/// Byte offsets count UTF-8 bytes (experiment 03 §4 arithmetic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacetRun {
    pub byte_start: usize,
    pub byte_end: usize,
    pub marks: Vec<Mark>,
}

/// A decoded `block/text` value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DecodedText {
    /// First paragraph's plaintext (the `nodes[0]` parity rule).
    pub plaintext: String,
    pub facets: Vec<FacetRun>,
    /// Non-deleted children of the `prosemirror` root.
    pub paragraph_count: usize,
    /// `meta.schemaVersion`, when stamped.
    pub schema_version: Option<i64>,
    /// Deferred content (mentions, footnotes, embeds) — the caller warns.
    pub notes: Vec<String>,
}

/// Decode a base64 `block/text` value end-to-end.
pub fn decode_block_text(value: &str) -> std::result::Result<DecodedText, DecodeError> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(value.trim()).map_err(|e| DecodeError {
        position: 0,
        message: format!("invalid base64: {e}"),
    })?;
    decode_block_text_bytes(&bytes)
}

/// Decode raw update bytes end-to-end.
pub fn decode_block_text_bytes(bytes: &[u8]) -> std::result::Result<DecodedText, DecodeError> {
    let update = decode_update(bytes)?;
    if update.clients.iter().all(|c| c.items.is_empty()) && update.delete_set.is_empty() {
        return Err(DecodeError {
            position: 0,
            message: "empty update".to_string(),
        });
    }
    let doc = Integrated::integrate(&update);
    let schema_version = doc
        .map_value(&TypeKey::Named("meta".into()), "schemaVersion")
        .and_then(content_any_first)
        .and_then(|v| if let AnyValue::Int(i) = v { Some(*i) } else { None });
    let mut out = DecodedText {
        schema_version,
        ..DecodedText::default()
    };

    let paragraphs: Vec<u32> = doc
        .children(&TypeKey::Named("prosemirror".into()))
        .into_iter()
        .filter(|idx| !doc.items[*idx as usize].deleted)
        .collect();
    out.paragraph_count = paragraphs.len();
    let Some(first) = paragraphs.first() else {
        return Ok(out); // empty doc: empty plaintext, no facets
    };

    let mut runs: Vec<(String, Vec<Mark>)> = Vec::new();
    render_element(&doc, *first, &mut runs, &mut out.notes);
    let mut offset = 0usize;
    let mut plaintext = String::new();
    for (text, marks) in runs {
        let start = offset;
        plaintext.push_str(&text);
        offset += text.len();
        if !marks.is_empty() {
            out.facets.push(FacetRun {
                byte_start: start,
                byte_end: offset,
                marks,
            });
        }
    }
    out.plaintext = plaintext;
    Ok(out)
}

fn content_any_first(content: &Content) -> Option<&AnyValue> {
    match content {
        Content::Any(values) => values.first(),
        _ => None,
    }
}

/// Render an XmlElement's children (a paragraph or any nested element).
fn render_element(doc: &Integrated, item_idx: u32, runs: &mut Vec<(String, Vec<Mark>)>, notes: &mut Vec<String>) {
    let children = doc.children(&TypeKey::Id(doc.items[item_idx as usize].item.id));
    for idx in children {
        if doc.items[idx as usize].deleted {
            continue;
        }
        match &doc.items[idx as usize].item.content {
            Content::Type { type_ref, .. } if *type_ref == type_refs::XML_TEXT => {
                render_xml_text(doc, idx, runs, notes);
            }
            Content::Type { type_ref, .. } if *type_ref == type_refs::XML_ELEMENT => {
                render_inline_element(doc, idx, runs, notes);
            }
            // Wire-level tombstones (ContentDeleted/Gc): list members that
            // hold positions but no content — skipped silently, exactly
            // like yjs's delta walk.
            Content::Deleted { .. } | Content::Gc { .. } => {}
            other => notes.push(format!("unexpected {} in element position", content_name(other))),
        }
    }
}

/// An XmlElement wherever it appears (paragraph child or inline in text):
/// hard_break → `\n`, mentions/footnotes → their plaintext contribution +
/// a deferral note, anything else → recurse.
fn render_inline_element(doc: &Integrated, idx: u32, runs: &mut Vec<(String, Vec<Mark>)>, notes: &mut Vec<String>) {
    let item = &doc.items[idx as usize].item;
    let name = match &item.content {
        Content::Type { name: Some(name), .. } => name.clone(),
        _ => return,
    };
    match name.as_str() {
        "hard_break" => runs.push(("\n".to_string(), Vec::new())),
        "footnote" => {
            runs.push(("*".to_string(), Vec::new()));
            notes.push("footnote facet deferred (content resolution is v0.5)".to_string());
        }
        "didMention" | "atMention" => {
            let key = TypeKey::Id(item.id);
            let text = doc.map_value(&key, "text").and_then(map_content_string).unwrap_or_default();
            runs.push((text, Vec::new()));
            notes.push(format!("{name} facet deferred"));
        }
        _ => render_element(doc, idx, runs, notes),
    }
}

/// A map entry value as a plain string.
fn map_content_string(content: &Content) -> Option<String> {
    match content {
        Content::Str(s) => Some(s.clone()),
        Content::Any(values) => match values.first() {
            Some(AnyValue::Str(s)) => Some(s.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Render a YXmlText's delta: ContentFormat toggles the active marks
/// (null value = close), ContentString accumulates runs, inline elements
/// contribute their plaintext.
fn render_xml_text(doc: &Integrated, idx: u32, runs: &mut Vec<(String, Vec<Mark>)>, notes: &mut Vec<String>) {
    let key = TypeKey::Id(doc.items[idx as usize].item.id);
    let mut active: BTreeMap<String, AnyValue> = BTreeMap::new();
    for child in doc.children(&key) {
        if doc.items[child as usize].deleted {
            continue;
        }
        match &doc.items[child as usize].item.content {
            Content::Format { key, value } => {
                if *value == AnyValue::Null || *value == AnyValue::Undefined {
                    active.remove(key);
                } else {
                    active.insert(key.clone(), value.clone());
                }
            }
            Content::Str(s) => {
                let marks = active_marks(&active, notes);
                runs.push((s.clone(), marks));
            }
            Content::Type { type_ref, .. } if *type_ref == type_refs::XML_ELEMENT => {
                render_inline_element(doc, child, runs, notes);
            }
            Content::Type { .. } => {
                render_xml_text(doc, child, runs, notes);
            }
            // Wire-level tombstones: skip silently (see render_element).
            Content::Deleted { .. } | Content::Gc { .. } => {}
            Content::Embed(_) => notes.push("embedded content deferred".to_string()),
            other => notes.push(format!("unexpected {} in text position", content_name(other))),
        }
    }
}

/// Snapshot the active formats as marks, in `YJSFragmentToFacets` feature
/// order (strikethrough, code, highlight, underline, bold, italic, link).
fn active_marks(active: &BTreeMap<String, AnyValue>, notes: &mut Vec<String>) -> Vec<Mark> {
    let mut marks = Vec::new();
    if active.contains_key("strikethrough") {
        marks.push(Mark::Strikethrough);
    }
    if active.contains_key("code") {
        marks.push(Mark::Code);
    }
    if let Some(highlight) = active.get("highlight") {
        marks.push(Mark::Highlight);
        let slot = highlight.member("color").and_then(AnyValue::as_str).unwrap_or("1");
        if slot != "1" {
            notes.push(format!("highlight slot {slot} color deferred (needs theme facts)"));
        }
    }
    if active.contains_key("underline") {
        marks.push(Mark::Underline);
    }
    if active.contains_key("strong") {
        marks.push(Mark::Bold);
    }
    if active.contains_key("em") {
        marks.push(Mark::Italic);
    }
    if let Some(href) = active.get("link").and_then(|link| link.member("href")).and_then(AnyValue::as_str) {
        marks.push(Mark::Link(href.to_string()));
    }
    marks
}

fn content_name(content: &Content) -> &'static str {
    match content {
        Content::Gc { .. } => "gc struct",
        Content::Deleted { .. } => "deleted content",
        Content::Json(_) => "json content",
        Content::Binary(_) => "binary content",
        Content::Str(_) => "string content",
        Content::Embed(_) => "embed content",
        Content::Format { .. } => "format content",
        Content::Type { .. } => "type content",
        Content::Any(_) => "any content",
        Content::Doc(_) => "subdoc content",
    }
}

// ---------------------------------------------------------------------------
// Encoder — the yjs_encode_text.py reference port, extended with marks
// ---------------------------------------------------------------------------

/// Encode failure (a facet span the text cannot carry).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("facet span {byte_start}..{byte_end} is out of bounds or off a char boundary of the {len}-byte text")]
pub struct EncodeError {
    pub byte_start: usize,
    pub byte_end: usize,
    pub len: usize,
}

/// A mark's yjs representation: the ContentFormat key + `writeAny` value
/// the decoder's [`Mark`] recovery reads back — the exact shapes Leaflet's
/// editor writes (`strong`/`em`/`code`/`strikethrough`/`underline` as
/// `true`, `link` as `{href}`, `highlight` as `{color:"1"}` slot 1).
fn mark_format(mark: &Mark) -> (String, AnyValue) {
    match mark {
        Mark::Strikethrough => ("strikethrough".into(), AnyValue::Bool(true)),
        Mark::Code => ("code".into(), AnyValue::Bool(true)),
        Mark::Highlight => ("highlight".into(), AnyValue::Object(vec![("color".into(), AnyValue::Str("1".into()))])),
        Mark::Underline => ("underline".into(), AnyValue::Bool(true)),
        Mark::Bold => ("strong".into(), AnyValue::Bool(true)),
        Mark::Italic => ("em".into(), AnyValue::Bool(true)),
        Mark::Link(href) => ("link".into(), AnyValue::Object(vec![("href".into(), AnyValue::Str(href.clone()))])),
    }
}

/// Canonical mark order — the decoder's `active_marks` emission order.
/// Encoding target sets in this order makes `decode(encode(x)) == x`
/// hold exactly (order included) for run-normalized facet lists.
fn mark_rank(mark: &Mark) -> u8 {
    match mark {
        Mark::Strikethrough => 0,
        Mark::Code => 1,
        Mark::Highlight => 2,
        Mark::Underline => 3,
        Mark::Bold => 4,
        Mark::Italic => 5,
        Mark::Link(_) => 6,
    }
}

/// One content item of the XmlText, in emission order.
enum EncodedItem {
    Str(String),
    Format { key: String, value: AnyValue },
}

impl EncodedItem {
    /// Length in clock units (strings count UTF-16 code units — the unit
    /// the decoder's clock arithmetic runs on).
    fn clock_len(&self) -> u64 {
        match self {
            EncodedItem::Str(s) => s.encode_utf16().count() as u64,
            EncodedItem::Format { .. } => 1,
        }
    }

    fn content_ref(&self) -> u8 {
        match self {
            EncodedItem::Str(_) => 4,
            EncodedItem::Format { .. } => 6,
        }
    }
}

/// Split `text` into maximal segments of constant mark set, validating
/// every facet span (byte bounds + char boundaries). Facets may overlap;
/// the segmentation merges them — the decoder round-trip therefore yields
/// the *normalized* facet list (one run per segment, features merged),
/// which is exactly the shape both the markdown converter and extraction
/// emit.
fn segments(text: &str, facets: &[FacetRun]) -> std::result::Result<Vec<(usize, usize, Vec<Mark>)>, EncodeError> {
    for facet in facets {
        let ok = facet.byte_start <= facet.byte_end
            && facet.byte_end <= text.len()
            && text.is_char_boundary(facet.byte_start)
            && text.is_char_boundary(facet.byte_end);
        if !ok {
            return Err(EncodeError {
                byte_start: facet.byte_start,
                byte_end: facet.byte_end,
                len: text.len(),
            });
        }
    }
    let mut points: std::collections::BTreeSet<usize> = [0, text.len()].into_iter().collect();
    for facet in facets {
        if facet.byte_start < facet.byte_end {
            points.insert(facet.byte_start);
            points.insert(facet.byte_end);
        }
    }
    let mut bounds: Vec<usize> = points.into_iter().collect();
    bounds.dedup();
    let mut out = Vec::new();
    for pair in bounds.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        let mut marks: Vec<Mark> = Vec::new();
        for facet in facets {
            if facet.byte_start <= start && end <= facet.byte_end {
                for mark in &facet.marks {
                    if !marks.contains(mark) {
                        marks.push(mark.clone());
                    }
                }
            }
        }
        marks.sort_by_key(mark_rank);
        out.push((start, end, marks));
    }
    Ok(out)
}

/// lib0 writer subset — the exact inverse of [`Reader`]'s primitives.
struct Writer {
    out: Vec<u8>,
}

impl Writer {
    fn u8(&mut self, byte: u8) {
        self.out.push(byte);
    }

    /// Unsigned varuint: 7 bits/byte, LSB first, `0x80` continuation.
    fn varuint(&mut self, mut n: u64) {
        while n > 0x7f {
            self.u8(0x80 | (n & 0x7f) as u8);
            n >>= 7;
        }
        self.u8(n as u8);
    }

    /// Sign-magnitude varint (`writeVarInt`): first byte `0x40` sign +
    /// 6 payload bits, then 7-bit groups.
    fn varint(&mut self, n: i64) {
        let sign = if n < 0 { 0x40u8 } else { 0 };
        let mut rest = n.unsigned_abs();
        let mut byte = (rest & 0x3f) as u8 | sign;
        rest >>= 6;
        if rest != 0 {
            byte |= 0x80;
        }
        self.u8(byte);
        while rest >= 0x80 {
            self.u8(((rest & 0x7f) as u8) | 0x80);
            rest >>= 7;
        }
        if rest != 0 {
            self.u8(rest as u8);
        }
    }

    fn varstring(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.varuint(bytes.len() as u64);
        self.out.extend_from_slice(bytes);
    }

    /// lib0 `writeAny` — the tag set [`Reader::any`] decodes (116–127).
    fn write_any(&mut self, value: &AnyValue) {
        match value {
            AnyValue::Bytes(bytes) => {
                self.u8(116);
                self.varuint(bytes.len() as u64);
                self.out.extend_from_slice(bytes);
            }
            AnyValue::Array(items) => {
                self.u8(117);
                self.varuint(items.len() as u64);
                for item in items {
                    self.write_any(item);
                }
            }
            AnyValue::Object(entries) => {
                self.u8(118);
                self.varuint(entries.len() as u64);
                for (key, value) in entries {
                    self.varstring(key);
                    self.write_any(value);
                }
            }
            AnyValue::Str(s) => {
                self.u8(119);
                self.varstring(s);
            }
            AnyValue::Bool(true) => self.u8(120),
            AnyValue::Bool(false) => self.u8(121),
            AnyValue::BigInt(i) => {
                self.u8(122);
                self.out.extend_from_slice(&i.to_le_bytes());
            }
            AnyValue::Double(d) => {
                self.u8(123);
                self.out.extend_from_slice(&d.to_le_bytes());
            }
            AnyValue::Float(f) => {
                self.u8(124);
                self.out.extend_from_slice(&f.to_le_bytes());
            }
            AnyValue::Int(i) => {
                self.u8(125);
                self.varint(*i);
            }
            AnyValue::Null => self.u8(126),
            AnyValue::Undefined => self.u8(127),
        }
    }
}

/// Encode a ProseMirror text block as a Yjs v1 update — the Rust port of
/// `experiments/02-draft-internals/scripts/yjs_encode_text.py`, extended
/// to carry marks as interleaved ContentFormat items (origin-chained like
/// real editor updates; the plain-text form is byte-identical to the
/// python reference and to yjs 13.6.18 `encodeStateAsUpdate` with the
/// same clientID). The doc shape is the decoder's domain: `prosemirror`
/// root → `paragraph` XmlElement → XmlText → items. No `meta.schemaVersion`
/// is stamped (unstamped = version 0, editable by every client — the safe
/// choice, experiment 02 §4).
///
/// Facet spans are UTF-8 byte offsets over `text`, matching the decoder's
/// recovery arithmetic. Plaintext containing `\n` is encoded as a literal
/// newline (not a `hard_break` XmlElement): it decodes back identically,
/// though the editor may render it differently than an authored break.
pub fn encode_block_text(text: &str, facets: &[FacetRun], client_id: u32) -> std::result::Result<String, EncodeError> {
    let bytes = encode_block_text_bytes(text, facets, client_id)?;
    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

/// [`encode_block_text`] returning raw update bytes.
pub fn encode_block_text_bytes(text: &str, facets: &[FacetRun], client_id: u32) -> std::result::Result<Vec<u8>, EncodeError> {
    // Build the content items from the mark segmentation.
    let mut items: Vec<EncodedItem> = Vec::new();
    let mut active: Vec<(String, AnyValue)> = Vec::new();
    for (start, end, marks) in segments(text, facets)? {
        let target: Vec<(String, AnyValue)> = marks.iter().map(mark_format).collect();
        let contains = |list: &[(String, AnyValue)], key: &str| list.iter().any(|(k, _)| k == key);
        // Close marks leaving the run (reverse open order), then open the
        // new ones (null value = close, per the delta contract).
        for (key, _) in active.iter().rev() {
            if !contains(&target, key) {
                items.push(EncodedItem::Format {
                    key: key.clone(),
                    value: AnyValue::Null,
                });
            }
        }
        for (key, value) in &target {
            if !contains(&active, key) {
                items.push(EncodedItem::Format {
                    key: key.clone(),
                    value: value.clone(),
                });
            }
        }
        active = target;
        if end > start {
            items.push(EncodedItem::Str(text[start..end].to_string()));
        }
    }

    let mut w = Writer { out: Vec::new() };
    w.varuint(1); // numClients
    w.varuint((2 + items.len()) as u64); // numStructs (2 when empty — yjs skips the empty insert)
    let client = u64::from(client_id);
    w.varuint(client);
    w.varuint(0); // startClock
    // struct 0: paragraph XmlElement in root "prosemirror"
    w.u8(0x07); // info: ContentType, no origin/rightOrigin/parentSub
    w.varuint(1); // parentInfo: parent is a root-type name
    w.varstring("prosemirror");
    w.varuint(3); // typeRef: YXmlElement
    w.varstring("paragraph");
    // struct 1: XmlText child of the paragraph (clock 1)
    w.u8(0x07);
    w.varuint(0); // parentInfo: parent is an ID
    w.varuint(client);
    w.varuint(0); // …the paragraph's clock
    w.varuint(6); // typeRef: YXmlText
    // content items (clock from 2): the first names its parent (the
    // XmlText at clock 1); every later one chains to its predecessor's
    // last clock via origin — the shape real editor updates carry.
    let mut clock = 2u64;
    for (i, item) in items.iter().enumerate() {
        if i == 0 {
            w.u8(item.content_ref());
            w.varuint(0); // parentInfo: parent is an ID
            w.varuint(client);
            w.varuint(1); // …the XmlText's clock
        } else {
            w.u8(0x80 | item.content_ref()); // origin set
            w.varuint(client);
            w.varuint(clock - 1);
        }
        match item {
            EncodedItem::Str(s) => w.varstring(s),
            EncodedItem::Format { key, value } => {
                w.varstring(key);
                w.write_any(value);
            }
        }
        clock += item.clock_len();
    }
    w.varuint(0); // deleteSet: empty
    Ok(w.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Sample 1 (`fixtures/yjs-samples.md`): `createYjsText("Hello world")`
    /// with client 979896792 — byte-identical to yjs 13.6.18 (regenerated
    /// with the committed `scripts/yjs_encode_text.py`; 63 bytes).
    const SAMPLE1_HEX: &str = "0103d893a0d3030007010b70726f73656d6972726f7203097061726167726170680700d893a0d30300060400d893a0d303010b48656c6c6f20776f726c6400";

    /// Sample 2: edited block with `meta.schemaVersion` and a tombstone —
    /// 5 structs + delete set; plaintext must converge to
    /// "Synthetic sample text".
    const SAMPLE2_HEX: &str = "0105cdf8acd1010007010b70726f73656d6972726f7203097061726167726170680700cdf8acd10100060400cdf8acd101011553796e7468657469632073616d706c65207465787481cdf8acd10116082801046d6574610d736368656d6156657273696f6e017d0101cdf8acd101011708";

    fn bytes_of(hex_str: &str) -> Vec<u8> {
        (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).unwrap())
            .collect()
    }

    // ── struct decode parity with yjs_decode.py ──────────────────────────

    #[test]
    fn sample1_structs_match_the_python_decoder_walkthrough() {
        let update = decode_update(&bytes_of(SAMPLE1_HEX)).unwrap();
        assert_eq!(update.clients.len(), 1);
        let client = &update.clients[0];
        assert_eq!(client.client, 979896792);
        assert_eq!(client.items.len(), 3);
        let info = |i: usize| client.items[i].info;
        let clock = |i: usize| client.items[i].id.clock;
        assert_eq!((clock(0), info(0)), (0, 0x07));
        assert_eq!(client.items[0].parent, Some(Parent::Named("prosemirror".into())));
        assert_eq!(
            client.items[0].content,
            Content::Type {
                type_ref: 3,
                name: Some("paragraph".into())
            }
        );
        assert_eq!((clock(1), info(1)), (1, 0x07));
        assert_eq!(client.items[1].parent, Some(Parent::Id(Id { client: 979896792, clock: 0 })));
        assert_eq!(client.items[1].content, Content::Type { type_ref: 6, name: None });
        assert_eq!((clock(2), info(2)), (2, 0x04));
        assert_eq!(client.items[2].content, Content::Str("Hello world".into()));
        assert!(update.delete_set.is_empty());
    }

    #[test]
    fn sample2_structs_tombstone_meta_and_delete_set() {
        let update = decode_update(&bytes_of(SAMPLE2_HEX)).unwrap();
        let client = &update.clients[0];
        assert_eq!(client.client, 439041101);
        assert_eq!(client.items.len(), 5);
        // clock 23: ContentDeleted len 8, origin = ID(c, 22) (info 0x81).
        assert_eq!(client.items[3].id.clock, 23);
        assert_eq!(client.items[3].info, 0x81);
        assert_eq!(client.items[3].origin, Some(Id { client: 439041101, clock: 22 }));
        assert_eq!(client.items[3].content, Content::Deleted { len: 8 });
        // clock 31: ContentAny [1] under meta.schemaVersion (info 0x28).
        assert_eq!(client.items[4].id.clock, 31);
        assert_eq!(client.items[4].info, 0x28);
        assert_eq!(client.items[4].parent, Some(Parent::Named("meta".into())));
        assert_eq!(client.items[4].parent_sub.as_deref(), Some("schemaVersion"));
        assert_eq!(client.items[4].content, Content::Any(vec![AnyValue::Int(1)]));
        // delete set: client 439041101, one range clock 23 len 8.
        assert_eq!(update.delete_set, vec![(439041101, vec![(23, 8)])]);
    }

    #[test]
    fn sample2_renders_edited_text_and_schema_version() {
        let decoded = decode_block_text_bytes(&bytes_of(SAMPLE2_HEX)).unwrap();
        assert_eq!(decoded.plaintext, "Synthetic sample text");
        assert_eq!(decoded.schema_version, Some(1));
        assert_eq!(decoded.paragraph_count, 1);
        assert!(decoded.facets.is_empty());
        assert!(decoded.notes.is_empty());
    }

    #[test]
    fn sample1_renders_plain_text() {
        let decoded = decode_block_text_bytes(&bytes_of(SAMPLE1_HEX)).unwrap();
        assert_eq!(decoded.plaintext, "Hello world");
        assert_eq!(decoded.paragraph_count, 1);
        assert_eq!(decoded.schema_version, None);
    }

    #[test]
    fn experiment02_doc_fixture_texts_decode() {
        // The h1-title + body-copy pair from getFactsForRoots-doc.json,
        // as regenerated with the committed encoder (the fixture file's own
        // base64 payloads carry a corrupted numStructs byte — the python
        // reference decoder throws on them too; see the tolerance test).
        let title = "AQPYk6DTAwAHAQtwcm9zZW1pcnJvcgMJcGFyYWdyYXBoBwDYk6DTAwAGBADYk6DTAwEPU3ludGhldGljIHRpdGxlAA==";
        let body = "AQPYk6DTAwAHAQtwcm9zZW1pcnJvcgMJcGFyYWdyYXBoBwDYk6DTAwAGBADYk6DTAwETU3ludGhldGljIGJvZHkgY29weQA=";
        assert_eq!(decode_block_text(title).unwrap().plaintext, "Synthetic title");
        assert_eq!(decode_block_text(body).unwrap().plaintext, "Synthetic body copy");
    }

    #[test]
    fn b5ec6d_non_update_is_an_error_never_a_panic() {
        // The 3-byte value observed on empty blocks: first varuint decodes
        // to 1,799,733 "clients" — Y.applyUpdate throws; so do we, and the
        // caller tolerates.
        let bytes = [0xb5, 0xec, 0x6d];
        assert!(decode_block_text_bytes(&bytes).is_err());
        // And as it would arrive (base64 of those bytes):
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        assert!(decode_block_text(&b64).is_err());
        // Empty-ish inputs are errors too, not panics.
        assert!(decode_block_text_bytes(&[]).is_err());
        assert!(decode_block_text("!!!not base64!!!").is_err());
        // The corrupt block/text payloads inside the committed
        // getFactsForRoots-doc.json fixture error identically (the python
        // reference decoder throws on them too — parity on the shared
        // fixtures, and exactly the tolerance path extraction needs).
        assert!(decode_block_text("AQDYk6DTAwAHAQtwcm9zZW1pcnJvcgMJcGFyYWdyYXBoBwDYk6DTAwAGBADYk6DTAwEPU3ludGhldGljIHRpdGxlAA==").is_err());
    }

    #[test]
    fn empty_paragraph_doc_renders_empty() {
        // createYjsText("") shape: 2 structs, no string item.
        let empty = [
            0x01, 0x02, 0x2a, 0x00, 0x07, 0x01, 0x0b, b'p', b'r', b'o', b's', b'e', b'm', b'i', b'r', b'r', b'o', b'r', 0x03, 0x09, b'p', b'a', b'r', b'a',
            b'g', b'r', b'a', b'p', b'h', 0x07, 0x00, 0x2a, 0x00, 0x06, 0x00,
        ];
        let decoded = decode_block_text_bytes(&empty).unwrap();
        assert_eq!(decoded.plaintext, "");
        assert_eq!(decoded.paragraph_count, 1);
        assert!(decoded.facets.is_empty());
    }

    // ── item integration: order is origin-linked, not clock order ────────

    /// Hand-built update: client 42 writes "AB" (clocks 0–1), then inserts
    /// "x" between them (clock 2, origin = ID(42,0), rightOrigin =
    /// ID(42,1)). Clock order is A,B,x — document order must be A,x,B.
    #[test]
    fn middle_insert_orders_by_origin_links() {
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
        bytes.push(1); // numClients
        bytes.push(4); // numStructs
        varuint(&mut bytes, 42); // client
        varuint(&mut bytes, 0); // start clock
        // struct 0: paragraph element in root "prosemirror"
        bytes.push(0x07);
        bytes.push(1);
        varstring(&mut bytes, "prosemirror");
        varuint(&mut bytes, 3);
        varstring(&mut bytes, "paragraph");
        // struct 1: XmlText child of the paragraph (clock 1)
        bytes.push(0x07);
        bytes.push(0);
        varuint(&mut bytes, 42);
        varuint(&mut bytes, 0);
        varuint(&mut bytes, 6);
        // struct 2: "AB" (clock 2) child of the XmlText
        bytes.push(0x04);
        bytes.push(0);
        varuint(&mut bytes, 42);
        varuint(&mut bytes, 1);
        varstring(&mut bytes, "AB");
        // struct 3: "x" (clock 4) with origin ID(42,2) — end of the left
        // "A" piece — and rightOrigin ID(42,3) — start of the "B" piece.
        bytes.push(0xc4); // 0x80 | 0x40 | 0x04
        varuint(&mut bytes, 42);
        varuint(&mut bytes, 2);
        varuint(&mut bytes, 42);
        varuint(&mut bytes, 3);
        varstring(&mut bytes, "x");
        bytes.push(0); // empty delete set

        let decoded = decode_block_text_bytes(&bytes).unwrap();
        assert_eq!(decoded.plaintext, "AxB", "origin/rightOrigin linking, not clock order");
    }

    /// Two clients append concurrently at the same tail (both origin = the
    /// "AB" item): the lower client id lands first (yjs conflict rule).
    #[test]
    fn concurrent_tail_appends_order_by_client_id() {
        fn build() -> Vec<u8> {
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
            // numClients 2, DESCENDING client order (wire convention).
            bytes.push(2);
            // client 100 (one struct: the "!" tail append, origin = ID(7,3)
            // — the last clock of client 7's "AB" item)
            bytes.push(1);
            varuint(&mut bytes, 100);
            varuint(&mut bytes, 0);
            bytes.push(0x84); // origin set, ContentString
            varuint(&mut bytes, 7);
            varuint(&mut bytes, 3);
            varstring(&mut bytes, "!");
            // client 7 (three structs: paragraph, XmlText, "AB")
            bytes.push(3);
            varuint(&mut bytes, 7);
            varuint(&mut bytes, 0);
            bytes.push(0x07);
            bytes.push(1);
            varstring(&mut bytes, "prosemirror");
            varuint(&mut bytes, 3);
            varstring(&mut bytes, "paragraph");
            bytes.push(0x07);
            bytes.push(0);
            varuint(&mut bytes, 7);
            varuint(&mut bytes, 0);
            varuint(&mut bytes, 6);
            bytes.push(0x04);
            bytes.push(0);
            varuint(&mut bytes, 7);
            varuint(&mut bytes, 1);
            varstring(&mut bytes, "AB");
            bytes.push(0);
            bytes
        }
        let decoded = decode_block_text_bytes(&build()).unwrap();
        // The pending queue must resolve client 100's origin (later in wire
        // order, pointing into client 7's structs); same-origin conflict
        // resolves by client id: 7 < 100, so "AB" precedes "!".
        assert_eq!(decoded.plaintext, "AB!");
    }

    /// A partially deleted string: delete set splits "Hello worldX" so the
    /// trailing X is a tombstone; plaintext drops it (sample-2 shape, but
    /// with the deletion mid-item to exercise clean splits).
    #[test]
    fn partial_delete_splits_and_tombstones() {
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
        bytes.push(3);
        varuint(&mut bytes, 9);
        varuint(&mut bytes, 0);
        bytes.push(0x07);
        bytes.push(1);
        varstring(&mut bytes, "prosemirror");
        varuint(&mut bytes, 3);
        varstring(&mut bytes, "paragraph");
        bytes.push(0x07);
        bytes.push(0);
        varuint(&mut bytes, 9);
        varuint(&mut bytes, 0);
        varuint(&mut bytes, 6);
        bytes.push(0x04);
        bytes.push(0);
        varuint(&mut bytes, 9);
        varuint(&mut bytes, 1);
        varstring(&mut bytes, "Hello worldX");
        // delete set: client 9, one range: clock 13 len 1 (the trailing X —
        // string index 11 + the string's start clock 2).
        bytes.push(1);
        varuint(&mut bytes, 9);
        varuint(&mut bytes, 1);
        varuint(&mut bytes, 13);
        varuint(&mut bytes, 1);

        let decoded = decode_block_text_bytes(&bytes).unwrap();
        assert_eq!(decoded.plaintext, "Hello world");
    }

    // ── marks / facets / byte offsets ────────────────────────────────────

    /// Format items toggling bold/link around runs, with a hard_break and a
    /// non-ASCII prefix to force byte-offset arithmetic.
    #[test]
    fn marks_render_as_byte_offset_facet_runs() {
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
        // struct 0 paragraph, struct 1 XmlText, then (all appended at the
        // XmlText tail in typing order, so each carries origin = the
        // previous item's last clock — the shape real updates have):
        //   "É " (clock 2, len 2)
        //   format strong true (clock 4, origin ID(5,3))
        //   "bold " (clock 5, origin ID(5,4))
        //   format link {href} (clock 10, origin ID(5,9))
        //   "text" (clock 11, origin ID(5,10))
        //   format link null (clock 15, origin ID(5,14))
        //   format strong null (clock 16, origin ID(5,15))
        //   "tail" (clock 17, origin ID(5,16))
        bytes.push(1);
        bytes.push(10); // numStructs
        let client: u64 = 5;
        varuint(&mut bytes, client);
        varuint(&mut bytes, 0);
        // 0: paragraph
        bytes.push(0x07);
        bytes.push(1);
        varstring(&mut bytes, "prosemirror");
        varuint(&mut bytes, 3);
        varstring(&mut bytes, "paragraph");
        // 1: XmlText (clock 1)
        bytes.push(0x07);
        bytes.push(0);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 0);
        varuint(&mut bytes, 6);
        // 2: "É " (clock 2, len 2) — first item of the XmlText: parentInfo
        bytes.push(0x04);
        bytes.push(0);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 1);
        varstring(&mut bytes, "É ");
        // 3: format strong true (clock 4, origin ID(5,3))
        bytes.push(0x86);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 3);
        varstring(&mut bytes, "strong");
        bytes.push(120); // writeAny true
        // 4: "bold " (clock 5, origin ID(5,4))
        bytes.push(0x84);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 4);
        varstring(&mut bytes, "bold ");
        // 5: format link {href} (clock 10, origin ID(5,9))
        bytes.push(0x86);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 9);
        varstring(&mut bytes, "link");
        // writeAny object {href: "https://x.example"}: tag 118, len 1,
        // key "href" (varstring), value string tag 119.
        bytes.push(118);
        varuint(&mut bytes, 1);
        varstring(&mut bytes, "href");
        bytes.push(119);
        varstring(&mut bytes, "https://x.example");
        // 6: "text" (clock 11, origin ID(5,10))
        bytes.push(0x84);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 10);
        varstring(&mut bytes, "text");
        // 7: format link null (clock 15, origin ID(5,14))
        bytes.push(0x86);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 14);
        varstring(&mut bytes, "link");
        bytes.push(126); // null
        // 8: format strong null (clock 16, origin ID(5,15))
        bytes.push(0x86);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 15);
        varstring(&mut bytes, "strong");
        bytes.push(126);
        // 9: "tail" (clock 17, origin ID(5,16)) — tail append
        bytes.push(0x84);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 16);
        varstring(&mut bytes, "tail");
        bytes.push(0);

        let decoded = decode_block_text_bytes(&bytes).unwrap();
        assert_eq!(decoded.plaintext, "É bold texttail");
        // Byte offsets: "É " is 3 bytes (É=2) → bold run starts at 3.
        // Runs stay unmerged (one facet per delta run, Leaflet parity):
        // "bold " spans 3..8, "text" (4 bytes) spans 8..12.
        assert_eq!(
            decoded.facets,
            vec![
                FacetRun {
                    byte_start: 3,
                    byte_end: 8,
                    marks: vec![Mark::Bold]
                },
                FacetRun {
                    byte_start: 8,
                    byte_end: 12,
                    marks: vec![Mark::Bold, Mark::Link("https://x.example".into())]
                },
            ]
        );
        // Byte-slicing round-trips each marked span.
        let bytes = decoded.plaintext.as_bytes();
        assert_eq!(&bytes[3..8], b"bold ");
        assert_eq!(&bytes[8..12], b"text");
    }

    #[test]
    fn multi_paragraph_warns_via_paragraph_count() {
        // Two paragraph elements in the fragment: only nodes[0] renders
        // (Leaflet parity); the count exposes the rest for the warn.
        let mut bytes: Vec<u8> = Vec::new();
        #[allow(unused_variables)]
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
        // Clocks: para1(0), XmlText1(1), "one"(2, len 3 → 2..4),
        // para2(5, origin = ID(3,0) — tail append in the FRAGMENT list,
        // whose last item is para1), XmlText2(6, parent Id(3,5)),
        // "two"(7, parent Id(3,6)).
        bytes.push(1);
        bytes.push(6);
        varuint(&mut bytes, 3);
        varuint(&mut bytes, 0);
        // para1
        bytes.push(0x07);
        bytes.push(1);
        varstring(&mut bytes, "prosemirror");
        varuint(&mut bytes, 3);
        varstring(&mut bytes, "paragraph");
        // xmltext1 (child of para1)
        bytes.push(0x07);
        bytes.push(0);
        varuint(&mut bytes, 3);
        varuint(&mut bytes, 0);
        varuint(&mut bytes, 6);
        // "one" (first item of xmltext1)
        bytes.push(0x04);
        bytes.push(0);
        varuint(&mut bytes, 3);
        varuint(&mut bytes, 1);
        varstring(&mut bytes, "one");
        // para2 (clock 5, origin = ID(3,0): tail append in the fragment)
        bytes.push(0x87);
        varuint(&mut bytes, 3);
        varuint(&mut bytes, 0);
        varuint(&mut bytes, 3);
        varstring(&mut bytes, "paragraph");
        // xmltext2 (child of para2 = Id(3,5))
        bytes.push(0x07);
        bytes.push(0);
        varuint(&mut bytes, 3);
        varuint(&mut bytes, 5);
        varuint(&mut bytes, 6);
        // "two" (first item of xmltext2)
        bytes.push(0x04);
        bytes.push(0);
        varuint(&mut bytes, 3);
        varuint(&mut bytes, 6);
        varstring(&mut bytes, "two");
        bytes.push(0);
        let decoded = decode_block_text_bytes(&bytes).unwrap();
        assert_eq!(decoded.plaintext, "one");
        assert_eq!(decoded.paragraph_count, 2);
    }

    /// A wire-level tombstone (ContentDeleted struct) inside an XmlText
    /// with NO delete-set range: skipped silently (live-draft shape — the
    /// server's merged values carry structural tombstones).
    #[test]
    fn wire_level_tombstones_skip_silently() {
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
        let client: u64 = 17;
        // para(0), xmltext(1), "AB"(2..3), Deleted{len:2}(4..5, origin
        // ID(c,3)), "C"(6, origin ID(c,5)).
        bytes.push(1);
        bytes.push(5);
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
        bytes.push(0x04); // "AB", first item of the XmlText: parentInfo
        bytes.push(0);
        varuint(&mut bytes, client);
        varuint(&mut bytes, 1);
        varstring(&mut bytes, "AB");
        bytes.push(0x81); // origin + ContentDeleted(2)
        varuint(&mut bytes, client);
        varuint(&mut bytes, 3);
        varuint(&mut bytes, 2);
        bytes.push(0x84); // "C" after the tombstone
        varuint(&mut bytes, client);
        varuint(&mut bytes, 5);
        varstring(&mut bytes, "C");
        bytes.push(0); // empty delete set
        let decoded = decode_block_text_bytes(&bytes).unwrap();
        assert_eq!(decoded.plaintext, "ABC", "tombstone holds its position but renders nothing");
        assert!(decoded.notes.is_empty(), "no warn for structural tombstones: {:?}", decoded.notes);
    }

    #[test]
    fn lib0_varint_sign_magnitude_vectors() {
        let mut r = Reader::new(&[0x00]);
        assert_eq!(r.varint().unwrap(), 0);
        let mut r = Reader::new(&[0x40]);
        assert_eq!(r.varint().unwrap(), -0);
        let mut r = Reader::new(&[0x3f]);
        assert_eq!(r.varint().unwrap(), 63);
        let mut r = Reader::new(&[0x7f]);
        assert_eq!(r.varint().unwrap(), -63); // sign bit + 63, no continuation
        // +63 with a redundant continuation byte: payload 63, b1 = 0.
        let mut r = Reader::new(&[0xbf, 0x00]);
        assert_eq!(r.varint().unwrap(), 63);
        // 123456 = 0b1_1110_0010_0100_0000: low 6 bits 0; >>6 = 1929
        // (low 7 = 9, cont); >>7 = 15 (stop). +: [0x80, 0x89, 0x0f];
        // -: sign bit adds 0x40 to b0.
        let mut r = Reader::new(&[0x80, 0x89, 0x0f]);
        assert_eq!(r.varint().unwrap(), 123456);
        let mut r = Reader::new(&[0xc0, 0x89, 0x0f]);
        assert_eq!(r.varint().unwrap(), -123456);
    }

    #[test]
    fn varuint_and_string_round_trips() {
        let mut bytes = Vec::new();
        for value in [0u64, 1, 0x7f, 0x80, 0x3fff, 979_896_792, u32::MAX as u64, 1_799_733] {
            let mut n = value;
            while n > 0x7f {
                bytes.push((n & 0x7f) as u8 | 0x80);
                n >>= 7;
            }
            bytes.push(n as u8);
        }
        let mut r = Reader::new(&bytes);
        for value in [0u64, 1, 0x7f, 0x80, 0x3fff, 979_896_792, u32::MAX as u64, 1_799_733] {
            assert_eq!(r.varuint().unwrap(), value);
        }
        // Truncated continuation is an error, not a panic.
        let mut r = Reader::new(&[0x80]);
        assert!(r.varuint().is_err());
        let mut r = Reader::new(&[0x05, b'h']);
        assert!(r.varstring().is_err()); // length 5, only 1 byte left
    }

    #[test]
    fn write_any_object_graph() {
        // {a: [1, "two", true, null], b: {c: 3.5}} — object keys are plain
        // varstrings; only values carry writeAny tags.
        let mut bytes = vec![118u8]; // object
        #[allow(unused_variables)]
        let varuint = |bytes: &mut Vec<u8>, mut n: u64| {
            while n > 0x7f {
                bytes.push((n & 0x7f) as u8 | 0x80);
                n >>= 7;
            }
            bytes.push(n as u8);
        };
        bytes.push(2); // two members
        bytes.extend_from_slice(&[1, b'a']); // key "a" (varstring)
        bytes.push(117); // array
        bytes.push(4);
        bytes.extend_from_slice(&[125, 0x01]); // int 1
        bytes.extend_from_slice(&[119, 3, b't', b'w', b'o']); // string "two"
        bytes.push(120); // true
        bytes.push(126); // null
        bytes.extend_from_slice(&[1, b'b']); // key "b"
        bytes.push(118); // nested object
        bytes.push(1);
        bytes.extend_from_slice(&[1, b'c']); // key "c"
        bytes.extend_from_slice(&[124]); // float32
        bytes.extend_from_slice(&3.5f32.to_le_bytes());
        let mut r = Reader::new(&bytes);
        let value = r.any().unwrap();
        assert_eq!(
            value,
            AnyValue::Object(vec![
                (
                    "a".into(),
                    AnyValue::Array(vec![AnyValue::Int(1), AnyValue::Str("two".into()), AnyValue::Bool(true), AnyValue::Null])
                ),
                ("b".into(), AnyValue::Object(vec![("c".into(), AnyValue::Float(3.5))])),
            ])
        );
    }

    #[test]
    fn hex_of_samples_is_stable_for_fixture_cross_checks() {
        // Guard the guard: the constants above really are the fixture bytes.
        assert_eq!(SAMPLE1_HEX.len(), 126, "63 bytes of hex");
        assert_eq!(SAMPLE2_HEX.len(), 226, "113 bytes of hex");
        assert_eq!(hex(&bytes_of("b5ec6d")), "b5ec6d");
    }

    // ── encoder: byte parity with the python reference ───────────────────

    /// Golden bytes produced by running the committed
    /// `experiments/02-draft-internals/scripts/yjs_encode_text.py`
    /// (`yjs_encode_text.py <text> --client <id> --hex`); regenerate
    /// against that script if this table ever disagrees.
    #[test]
    fn encoder_is_byte_identical_to_the_python_reference() {
        let vectors: &[(&str, u32, &str)] = &[
            (
                "Hello world",
                979_896_792,
                "0103d893a0d3030007010b70726f73656d6972726f7203097061726167726170680700d893a0d30300060400d893a0d303010b48656c6c6f20776f726c6400",
            ),
            ("", 42, "01022a0007010b70726f73656d6972726f72030970617261677261706807002a000600"),
            (
                "Synthetic title",
                1_000_000_000,
                "01038094ebdc030007010b70726f73656d6972726f72030970617261677261706807008094ebdc03000604008094ebdc03010f53796e746865746963207469746c6500",
            ),
            (
                "Émigré — cafés of Montréal",
                u32::MAX,
                "0103ffffffff0f0007010b70726f73656d6972726f7203097061726167726170680700ffffffff0f00060400ffffffff0f0120c3896d696772c3a920e2809420636166c3a973206f66204d6f6e7472c3a9616c00",
            ),
            ("A", 1, "0103010007010b70726f73656d6972726f720309706172616772617068070001000604000101014100"),
            (
                "multi paragraph\nsecond line",
                7,
                "0103070007010b70726f73656d6972726f7203097061726167726170680700070006040007011b6d756c7469207061726167726170680a7365636f6e64206c696e6500",
            ),
        ];
        for (text, client, expect_hex) in vectors {
            let bytes = encode_block_text_bytes(text, &[], *client).unwrap();
            assert_eq!(&hex(&bytes), expect_hex, "text {text:?} client {client}");
        }
        // The committed push fixture's two block/text values: same client
        // (979896792), same texts — the encoder reproduces them verbatim.
        assert_eq!(
            encode_block_text("Synthetic title", &[], 979_896_792).unwrap(),
            "AQPYk6DTAwAHAQtwcm9zZW1pcnJvcgMJcGFyYWdyYXBoBwDYk6DTAwAGBADYk6DTAwEPU3ludGhldGljIHRpdGxlAA=="
        );
        assert_eq!(
            encode_block_text("Synthetic body copy", &[], 979_896_792).unwrap(),
            "AQPYk6DTAwAHAQtwcm9zZW1pcnJvcgMJcGFyYWdyYXBoBwDYk6DTAwAGBADYk6DTAwETU3ludGhldGljIGJvZHkgY29weQA="
        );
    }

    #[test]
    fn encoded_plain_text_decodes_back_exactly() {
        for (text, client) in [("Hello world", 979_896_792u32), ("", 7), ("É — ünïcode ✓", 1234)] {
            let value = encode_block_text(text, &[], client).unwrap();
            let decoded = decode_block_text(&value).unwrap();
            assert_eq!(decoded.plaintext, text);
            assert!(decoded.facets.is_empty(), "{text:?}");
            assert_eq!(decoded.paragraph_count, 1);
            assert_eq!(decoded.schema_version, None, "unstamped = version 0 (the safe choice)");
        }
    }

    /// The round-trip property: decode(encode(text, facets)) == (text,
    /// facets) for run-normalized facet lists — one facet per marked
    /// segment with merged features, the exact shape both the markdown
    /// converter and extraction emit. Overlapping inputs normalize.
    #[test]
    fn encoded_marks_round_trip_through_the_decoder() {
        let run = |start: usize, end: usize, marks: &[Mark]| FacetRun {
            byte_start: start,
            byte_end: end,
            marks: marks.to_vec(),
        };
        // The v0 mark set, exercised one at a time over non-ASCII text so
        // byte-offset arithmetic must stay honest (é is 2 bytes, — is 3).
        let text = "É precursor plain bold tail — underline code strike highlight link end";
        let cases: Vec<FacetRun> = vec![
            run(13, 18, &[Mark::Bold]),
            run(25, 34, &[Mark::Underline]),
            run(36, 40, &[Mark::Code]),
            run(41, 48, &[Mark::Strikethrough]),
            run(49, 59, &[Mark::Highlight]),
            run(60, 64, &[Mark::Link("https://example.com".into())]),
        ];
        let value = encode_block_text(text, &cases, 42).unwrap();
        let decoded = decode_block_text(&value).unwrap();
        assert_eq!(decoded.plaintext, text);
        assert_eq!(decoded.facets, cases, "exact run list (order included) for normalized input");

        // Overlapping bold+link on one span merges into a single run with
        // both marks (canonical order: bold before link).
        let text = "prefix marked suffix";
        let merged = vec![run(7, 13, &[Mark::Bold, Mark::Link("https://x.example".into())])];
        let value = encode_block_text(text, &merged, 42).unwrap();
        let decoded = decode_block_text(&value).unwrap();
        assert_eq!(decoded.facets, merged);

        // A partially-overlapping input (bold and italic spans) normalizes
        // to per-segment runs — the property is equality after the same
        // normalization, not echo of the input.
        let text = "abcdef";
        let raw = vec![run(0, 3, &[Mark::Bold]), run(2, 5, &[Mark::Italic])];
        let value = encode_block_text(text, &raw, 42).unwrap();
        let decoded = decode_block_text(&value).unwrap();
        assert_eq!(decoded.plaintext, text);
        assert_eq!(
            decoded.facets,
            vec![run(0, 2, &[Mark::Bold]), run(2, 3, &[Mark::Bold, Mark::Italic]), run(3, 5, &[Mark::Italic]),]
        );

        // Facet directly adjacent to another (close then reopen) and a
        // facet at the very start/end of the text.
        let text = "ab";
        let edge = vec![run(0, 1, &[Mark::Code]), run(1, 2, &[Mark::Code])];
        let value = encode_block_text(text, &edge, 42).unwrap();
        assert_eq!(decode_block_text(&value).unwrap().facets, edge);

        // Empty facet spans are dropped by normalization.
        let value = encode_block_text("ab", &[run(1, 1, &[Mark::Bold])], 42).unwrap();
        assert!(decode_block_text(&value).unwrap().facets.is_empty());
    }

    /// The format items are origin-chained exactly like the hand-built
    /// fixture in `marks_render_as_byte_offset_facet_runs` — the wire
    /// shape real editor updates carry, not a decoder-only convenience.
    #[test]
    fn encoded_wire_bytes_carry_origin_chained_format_items() {
        use std::collections::BTreeMap;
        let text = "bold";
        let facets = [FacetRun {
            byte_start: 0,
            byte_end: 4,
            marks: vec![Mark::Bold],
        }];
        let bytes = encode_block_text_bytes(text, &facets, 5).unwrap();
        let update = decode_update(&bytes).unwrap();
        let items = &update.clients[0].items;
        assert_eq!(items.len(), 4, "paragraph, XmlText, format strong, string");
        // struct 2: format strong (clock 2, first item — parentInfo present)
        assert_eq!(items[2].id.clock, 2);
        assert_eq!(items[2].info, 0x06);
        assert_eq!(items[2].origin, None);
        assert_eq!(items[2].parent, Some(Parent::Id(Id { client: 5, clock: 1 })));
        assert_eq!(
            items[2].content,
            Content::Format {
                key: "strong".into(),
                value: AnyValue::Bool(true)
            }
        );
        // struct 3: the string (clock 3), origin = the format's clock.
        assert_eq!(items[3].id.clock, 3);
        assert_eq!(items[3].info, 0x84);
        assert_eq!(items[3].origin, Some(Id { client: 5, clock: 2 }));
        assert_eq!(items[3].content, Content::Str("bold".into()));
        // UTF-16 clock arithmetic: bold over "a" in "a—b" makes the items
        // format(2), "a"(3), close-format(4), "—b"(5) — close formats are
        // items with clocks like any other, and the em-dash is one UTF-16
        // unit.
        let bytes = encode_block_text_bytes(
            "a—b",
            &[FacetRun {
                byte_start: 0,
                byte_end: 1,
                marks: vec![Mark::Bold],
            }],
            9,
        )
        .unwrap();
        let items = &decode_update(&bytes).unwrap().clients[0].items;
        assert_eq!(items.len(), 6);
        assert_eq!(items[3].id.clock, 3); // "a" (len 1)
        assert_eq!(items[4].id.clock, 4); // format strong null (close)
        assert_eq!(items[4].origin, Some(Id { client: 9, clock: 3 }));
        assert_eq!(
            items[4].content,
            Content::Format {
                key: "strong".into(),
                value: AnyValue::Null
            }
        );
        assert_eq!(items[5].id.clock, 5); // "—b" (— is 1 utf-16 unit + b)
        assert_eq!(items[5].origin, Some(Id { client: 9, clock: 4 }));
    }

    #[test]
    fn encoder_rejects_bad_spans() {
        let err = encode_block_text(
            "abc",
            &[FacetRun {
                byte_start: 2,
                byte_end: 9,
                marks: vec![Mark::Bold],
            }],
            1,
        )
        .unwrap_err();
        assert_eq!(err.len, 3);
        // Mid-character boundary (é is 2 bytes: offset 1 is inside it).
        assert!(
            encode_block_text(
                "éx",
                &[FacetRun {
                    byte_start: 1,
                    byte_end: 3,
                    marks: vec![Mark::Bold]
                }],
                1
            )
            .is_err()
        );
        // Inverted span.
        assert!(
            encode_block_text(
                "abc",
                &[FacetRun {
                    byte_start: 2,
                    byte_end: 1,
                    marks: vec![Mark::Bold]
                }],
                1
            )
            .is_err()
        );
    }
}
