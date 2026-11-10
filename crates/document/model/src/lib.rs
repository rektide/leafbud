//! Pure document model: current content, source keys, bindings, receipts.
//! No network, parser arenas, raw snapshots, CRDT history or saved bodies.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
pub use uuid::Uuid;
mod attempt;
pub use attempt::*;

/// Source format this model reads and writes (`leafbud-mdx/2`). Local data is
/// rebuildable; there is no compatibility surface for the retired format 1.
pub const FORMAT: &str = "leafbud-mdx/2";
/// Editor agreement receipt profile; independent of serialization fidelity.
/// Profile 3 canonicalizes desired-new and locally-retired associations
/// explicitly instead of requiring exact body/key/table equality.
pub const EDITOR_PROFILE: &str = "leaflet-editor/3";
pub const EDITOR_OWNED_EXTRA: &[&str] = &["contributors"];

/// Shared semantic clear representation. Do not trim: every nonempty string,
/// including whitespace, is meaningful and remains byte-exact. Authored values
/// and conservative native witnesses retain their original representation.
#[doc(hidden)]
pub fn canonical_description(description: Option<&str>) -> Option<&str> {
    description.filter(|description| !description.is_empty())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Inline {
    Run { text: String, marks: Vec<Mark> },
    HardBreak,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Mark {
    Strong,
    Em,
    Code,
    Strikethrough,
    Underline,
    Highlight { color: String },
    Link { href: String },
}
pub fn push_run(out: &mut Vec<Inline>, text: &str, marks: &[Mark]) {
    if text.is_empty() {
        return;
    }
    let mut marks = marks.to_vec();
    marks.sort();
    marks.dedup();
    if let Some(Inline::Run {
        text: previous,
        marks: previous_marks,
    }) = out.last_mut()
        && previous_marks == &marks
    {
        previous.push_str(text);
    } else {
        out.push(Inline::Run { text: text.into(), marks });
    }
}

/// Flat list marker style. Numbering is display-derived: the native model
/// sequences ordered items itself, and source spelling normalizes to `1.`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ListStyle {
    Bullet,
    Ordered,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub key: String,
    #[serde(flatten)]
    pub kind: BlockKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BlockKind {
    Paragraph {
        content: Vec<Inline>,
    },
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Quote {
        content: Vec<Inline>,
    },
    /// One flat list item: a text block occurrence with native list facts.
    /// Consecutive items form a list; there is no wrapper object or identity.
    ListItem {
        style: ListStyle,
        content: Vec<Inline>,
    },
    Code {
        language: String,
        text: String,
    },
    Rule,
    Image {
        src: String,
        fallback: Option<String>,
        width: Option<u64>,
        height: Option<u64>,
        alt: Option<String>,
    },
    PageLink {
        page: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub key: String,
    pub blocks: Vec<Block>,
}

/// Publication intent, retaining absent-versus-null through deserialization.
/// `Inherit` (field absent) defers to configuration while unbound; `Standalone`
/// (explicit null) selects standalone; `At` selects a publication target.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Publication {
    #[default]
    Inherit,
    Standalone,
    At(String),
}

impl Publication {
    /// Canonical JSON spelling used by editor/receipt projections.
    pub fn value(&self) -> Value {
        match self {
            Self::Inherit => serde_json::json!({"kind":"inherit"}),
            Self::Standalone => serde_json::json!({"kind":"standalone"}),
            Self::At(uri) => serde_json::json!({"kind":"uri","uri":uri}),
        }
    }
}

impl Serialize for Publication {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Standalone => serializer.serialize_none(),
            Self::At(uri) => serializer.serialize_str(uri),
            // Absent-by-contract: the document field skips serialization.
            Self::Inherit => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for Publication {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = Publication;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("null or a publication URI string")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Publication::At(value.to_owned()))
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Publication::Standalone)
            }
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Publication::Standalone)
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub format: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Publication::is_inherit")]
    pub publication: Publication,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
    #[serde(rename = "@leafbud")]
    pub state: State,
    #[serde(skip)]
    pub pages: Vec<Page>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Theme {
    pub leaflet_watermark: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct State {
    pub id: Uuid,
    pub state: String,
    /// Source key of the implicit main page. Absent until preparation
    /// persists it; independent of the first heading's spelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_page: Option<String>,
    /// Confirmed native association; absent for local-only drafts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<Binding>,
    /// Confirmed page/block occurrence associations. The table may lag desired
    /// content: keys without rows are new; rows without content are retired.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub nodes: BTreeMap<String, Node>,
    /// Last verified agreement; never an attempt. Coherent without a binding
    /// is impossible, so sync requires binding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync: Option<Sync>,
    /// Stable native request identity/next reserved sequence, never a Yrs ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writer: Option<WriterState>,
    /// Unresolved effect evidence, never a body, request journal or receipt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<Attempt>,
    /// Last deliberate terminal resolution; provenance, never a receipt or journal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_resolution: Option<Resolution>,
    /// Accepted local file/path pair, independent of native/editor evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub names: Option<Names>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Names {
    /// Workspace-relative, slash-separated coordinate, including `.mdx`.
    pub file: String,
    pub path: NamePath,
}

impl Names {
    /// Literal, reversible logical path → workspace-relative MDX coordinate.
    /// The leading slash is logical, never an OS absolute pathname.
    pub fn file_for_path(path: &str) -> Result<String, &'static str> {
        let relative = path
            .strip_prefix('/')
            .ok_or("logical path must begin with one slash (for example /notes/Essay)")?;
        name_segments(relative)?;
        Ok(format!("{relative}.mdx"))
    }

    pub fn path_for_file(file: &str) -> Result<String, &'static str> {
        name_segments(file)?;
        let stem = file.strip_suffix(".mdx").ok_or("naming requires the lowercase .mdx extension")?;
        name_segments(stem)?;
        Ok(format!("/{stem}"))
    }

    /// An accepted mismatch may contain unmappable author intent, but its
    /// file witness must still be a supported coordinate, not a future schema.
    pub fn validate(&self) -> Result<(), &'static str> {
        Self::path_for_file(&self.file).map(|_| ())
    }
}

fn name_segments(value: &str) -> Result<(), &'static str> {
    if value.is_empty() {
        return Err("empty name coordinate");
    }
    for part in value.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err("empty/traversal name segment is unsupported");
        }
        if part.chars().any(|c| c.is_control() || "\\%:?#".contains(c)) {
            return Err("encoded, control, URL or drive punctuation in name is unsupported; use literal Unicode and slash-separated segments");
        }
    }
    Ok(())
}

/// Preserve actual author-field presence, including missing versus null.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase", deny_unknown_fields)]
pub enum NamePath {
    Missing,
    Null,
    Value(String),
}

impl NamePath {
    pub fn from_author(extra: &BTreeMap<String, Value>) -> Result<Self, &'static str> {
        match extra.get("path") {
            None => Ok(Self::Missing),
            Some(Value::Null) => Ok(Self::Null),
            Some(Value::String(value)) => Ok(Self::Value(value.clone())),
            _ => Err("author path must be a string or null"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Binding {
    /// Exact verified public identity. Absent only for a draft editor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    pub base_url: String,
    pub token: Uuid,
    pub root: String,
    pub entity_set: String,
    pub entry_fact: String,
    /// Confirmed association; explicit null means standalone, never inherited.
    pub publication: Option<String>,
}

/// Narrow native publication identity profile, using the existing protocol
/// library's syntax checks rather than a second AT-URI parser. Public identities
/// persist a DID, never a mutable handle, collection root or fragment.
pub fn validate_document_uri(value: &str) -> Result<(), &'static str> {
    let uri = jacquard_common::types::string::AtUri::new(value).map_err(|_| "invalid document AT URI")?;
    if !uri.authority().as_str().starts_with("did:") || value.contains(['?', '#']) || uri.rkey().is_none() {
        return Err("document identity requires a complete DID-based record URI without query or fragment");
    }
    match uri.collection().as_ref().map(|collection| collection.as_str()) {
        Some("site.standard.document" | "pub.leaflet.document") => Ok(()),
        _ => Err("unsupported public document collection"),
    }
}

impl State {
    pub fn validate_lifecycle(&self) -> Result<(), &'static str> {
        match (self.state.as_str(), self.binding.as_ref().and_then(|binding| binding.uri.as_deref())) {
            ("draft", None) => Ok(()),
            ("published", Some(uri)) => validate_document_uri(uri),
            ("draft", Some(_)) => Err("draft cannot carry a public document URI"),
            ("published", None) => Err("published requires an exact public identity and complete native editor binding; URI-only adoption is unavailable"),
            _ => Err("unsupported managed lifecycle"),
        }
    }
}
/// One typed native occurrence locator. Multiple source keys may share an
/// entity but always identify distinct occurrences.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum Occurrence {
    /// A block occurrence: its entity, one placement fact, and rich-text
    /// locators. At most one of the text facts is set for a given kind.
    Block {
        placement: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text_fact: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inactive_text_fact: Option<String>,
    },
    /// A page occurrence: its entity and known incoming reference facts
    /// (entry and card references pointing at this page).
    Page { references: Vec<String> },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub entity: String,
    #[serde(flatten)]
    pub occurrence: Occurrence,
}

impl<'de> Deserialize<'de> for Node {
    /// Strict decode: exactly `entity`, a `kind` discriminator, and the
    /// fields of that occurrence variant. Unknown/future node fields are
    /// rejected rather than silently dropped (the flatten on `Occurrence`
    /// cannot carry `deny_unknown_fields` itself).
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let mut map: BTreeMap<String, serde_json::Value> = BTreeMap::deserialize(deserializer)?;
        let take_string = |map: &mut BTreeMap<String, serde_json::Value>, key: &str| -> Result<Option<String>, D::Error> {
            match map.remove(key) {
                None | Some(serde_json::Value::Null) => Ok(None),
                Some(value) => Ok(Some(String::deserialize(value).map_err(D::Error::custom)?)),
            }
        };
        let entity = take_string(&mut map, "entity")?.ok_or_else(|| D::Error::custom("node entity must be a non-null string"))?;
        let kind = take_string(&mut map, "kind")?.ok_or_else(|| D::Error::custom("node kind must be a non-null string"))?;
        let occurrence = match kind.as_str() {
            "block" => {
                let placement = take_string(&mut map, "placement")?.ok_or_else(|| D::Error::custom("block occurrence requires a placement"))?;
                let text_fact = take_string(&mut map, "textFact")?;
                let inactive_text_fact = take_string(&mut map, "inactiveTextFact")?;
                Occurrence::Block {
                    placement,
                    text_fact,
                    inactive_text_fact,
                }
            }
            "page" => {
                let references = match map.remove("references") {
                    None => vec![],
                    Some(value) => Vec::<String>::deserialize(value).map_err(D::Error::custom)?,
                };
                Occurrence::Page { references }
            }
            other => return Err(D::Error::custom(format!("unknown node occurrence kind {other:?}"))),
        };
        if let Some(key) = map.keys().next() {
            return Err(D::Error::custom(format!("unknown node field {key:?}")));
        }
        Ok(Node { entity, occurrence })
    }
}
impl Node {
    /// Canonical locator JSON for receipt projections. Spelling-free: the
    /// source key addressing this row is not part of the association.
    pub fn locator_value(&self) -> Value {
        match &self.occurrence {
            Occurrence::Block {
                placement,
                text_fact,
                inactive_text_fact,
            } => serde_json::json!({
                "kind":"block","entity":self.entity,"placement":placement,
                "textFact":text_fact,"inactiveTextFact":inactive_text_fact
            }),
            Occurrence::Page { references } => {
                let references: BTreeSet<_> = references.iter().collect();
                serde_json::json!({"kind":"page","entity":self.entity,"references":references})
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sync {
    pub profile: String,
    pub local_hash: String,
    pub remote_hash: String,
}

impl Publication {
    pub fn is_inherit(publication: &Publication) -> bool {
        matches!(publication, Publication::Inherit)
    }
}

impl Document {
    /// Source keys referenced by current content: every page and block.
    /// PageLink targets are page keys and validated to resolve.
    pub fn content_keys(&self) -> BTreeSet<String> {
        let mut keys = BTreeSet::new();
        for page in &self.pages {
            keys.insert(page.key.clone());
            for block in &page.blocks {
                keys.insert(block.key.clone());
            }
        }
        keys
    }

    /// Supported editor meaning and object/occurrence associations. Source-key
    /// spelling, non-entry page definition order and local-only author
    /// metadata are not editor changes. Confirmed objects carry their native
    /// locator; desired-new objects carry an explicit `new` identity that
    /// includes their source key, so distinct new objects never collapse into
    /// one marker (a link retarget between two new pages is a real change).
    /// Locally-retired associations are explicit pending omissions rather
    /// than dropped rows.
    pub fn editor_value(&self) -> Value {
        let association = |key: &str| match self.state.nodes.get(key) {
            Some(node) => node.locator_value(),
            None => serde_json::json!({"kind":"new","key":key}),
        };
        let page = |page: &Page| {
            let blocks: Vec<_> = page
                .blocks
                .iter()
                .map(|block| {
                    let mut kind = serde_json::to_value(&block.kind).expect("block JSON");
                    if let BlockKind::PageLink { page } = &block.kind {
                        kind["page"] = association(page);
                    }
                    serde_json::json!({"association":association(&block.key),"content":kind})
                })
                .collect();
            serde_json::json!({"association":association(&page.key),"blocks":blocks})
        };
        let mut definitions: Vec<_> = self.pages.iter().skip(1).map(page).collect();
        definitions.sort_by_key(Value::to_string);
        let retired: Vec<_> = {
            let content = self.content_keys();
            let mut rows: Vec<_> = self
                .state
                .nodes
                .iter()
                .filter(|(key, _)| !content.contains(*key))
                .map(|(_, node)| node.locator_value())
                .collect();
            rows.sort_by_key(Value::to_string);
            rows
        };
        let extra: BTreeMap<_, _> = self.extra.iter().filter(|(key, _)| EDITOR_OWNED_EXTRA.contains(&key.as_str())).collect();
        serde_json::json!({"profile":EDITOR_PROFILE,"title":self.title,"description":canonical_description(self.description.as_deref()),
            "tags":self.tags,"publication":self.publication.value(),"theme":self.theme,"extra":extra,
            "main":self.pages.first().map(page),"definitions":definitions,"retired":retired})
    }

    /// Current semantic projection only. Excludes binding bookkeeping and all
    /// receipt hashes; includes source identities/order and represented meaning.
    pub fn semantic_value(&self) -> Value {
        serde_json::json!({"format":self.format,"title":self.title,"description":canonical_description(self.description.as_deref()),"tags":self.tags,
            "publication":self.publication.value(),"theme":self.theme,"extra":self.extra,
            "mainPage":self.state.main_page,"pages":self.pages,"nodes":self.state.nodes})
    }

    /// Authored projection: editor-owned meaning plus current content with
    /// source identities, excluding everything tracking settlement may
    /// legitimately change (binding, nodes, receipts) and the main-page
    /// persistence boundary owned by preparation. A settlement must leave
    /// this value untouched.
    pub fn authored_value(&self) -> Value {
        serde_json::json!({"format":self.format,"title":self.title,"description":self.description,"tags":self.tags,
            "publication":self.publication.value(),"theme":self.theme,"extra":self.extra,"pages":self.pages})
    }
}
