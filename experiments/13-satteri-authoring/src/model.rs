use serde::{Deserialize, Serialize};

/// Absolute UTF-8 bytes in the retained source, including an initial BOM.
/// Whole-node/element spans only: no invented individual JSX attribute ranges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Metadata {
    pub format: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Author intent only; this experiment neither schedules nor publishes.
    #[serde(default)]
    pub published_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuthorKey {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Mark {
    Strong,
    Em,
    Code,
    Strikethrough,
    Link { href: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Inline {
    Run { text: String, marks: Vec<Mark> },
    HardBreak,
}

/// The canonical inline stream coalesces adjacent identically marked runs.
/// Literal newlines inside Run remain text; only HardBreak is an XML element.
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

pub fn plaintext(content: &[Inline]) -> String {
    content
        .iter()
        .map(|n| match n {
            Inline::Run { text, .. } => text.as_str(),
            Inline::HardBreak => "\n",
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Block {
    pub span: Span,
    pub key: Option<AuthorKey>,
    pub kind: BlockKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BlockKind {
    Paragraph {
        content: Vec<Inline>,
    },
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Code {
        language: String,
        text: String,
    },
    Rule,
    List {
        ordered: bool,
        start: u32,
        items: Vec<Vec<Block>>,
    },
    /// Bounded Leaflet mapping: one ordinary paragraph in a quote.
    Quote {
        paragraph: Box<Block>,
    },
    Poll {
        options: Vec<String>,
    },
    PageLink {
        page: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Page {
    pub name: String,
    pub span: Span,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Serialize)]
pub struct Document<'a> {
    #[serde(skip)]
    pub original_source: &'a str,
    pub metadata: Metadata,
    pub main: Vec<Block>,
    pub pages: Vec<Page>,
    pub consumed_keys: usize,
}
