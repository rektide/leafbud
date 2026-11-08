//! Bounded import profile: current content and compact identity bindings.
//! No network, parser arenas, raw snapshots or serialized CRDT history.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
pub use uuid::Uuid;

pub const PROFILE: &str = "leaflet-import/1";

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub format: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Explicit confirmed association; null means standalone, never inherited.
    pub publication: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
    #[serde(rename = "@springleaf")]
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
#[serde(rename_all = "camelCase")]
pub struct State {
    pub id: Uuid,
    pub state: String,
    pub binding: Binding,
    pub nodes: BTreeMap<String, Node>,
    pub sync: Sync,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Binding {
    pub base_url: String,
    pub token: Uuid,
    pub root: String,
    pub entity_set: String,
    pub entry_fact: String,
    pub publication: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Node {
    pub entity: String,
    /// Occurrence/role keyed membership, not an array index as identity.
    pub memberships: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_fact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inactive_text_fact: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sync {
    pub profile: String,
    pub local_hash: String,
    pub remote_hash: String,
}

impl Document {
    /// Current semantic projection only. Excludes binding bookkeeping and all
    /// receipt hashes; includes source identities/order and represented meaning.
    pub fn semantic_value(&self) -> Value {
        serde_json::json!({"profile":PROFILE,"title":self.title,"description":self.description,"tags":self.tags,
            "publication":self.publication,"theme":self.theme,"extra":self.extra,"pages":self.pages,"nodes":self.state.nodes})
    }
}
