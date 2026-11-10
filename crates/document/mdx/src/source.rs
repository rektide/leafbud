//! Bounded source coordinates from the existing YAML parser, not a second
//! YAML grammar. Used only by explicit local authoring helpers.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_saphyr::Spanned;

#[derive(Debug)]
pub(crate) enum Node {
    Map(Vec<(Spanned<String>, Spanned<Node>)>),
    Other,
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Node;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("ordinary YAML")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> std::result::Result<Node, A::Error> {
                let mut entries = vec![];
                while let Some(entry) = map.next_entry()? {
                    entries.push(entry);
                }
                Ok(Node::Map(entries))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> std::result::Result<Node, A::Error> {
                while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                Ok(Node::Other)
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> std::result::Result<Node, E> {
                Ok(Node::Other)
            }
            fn visit_bool<E: serde::de::Error>(self, _: bool) -> std::result::Result<Node, E> {
                Ok(Node::Other)
            }
            fn visit_i64<E: serde::de::Error>(self, _: i64) -> std::result::Result<Node, E> {
                Ok(Node::Other)
            }
            fn visit_u64<E: serde::de::Error>(self, _: u64) -> std::result::Result<Node, E> {
                Ok(Node::Other)
            }
            fn visit_f64<E: serde::de::Error>(self, _: f64) -> std::result::Result<Node, E> {
                Ok(Node::Other)
            }
            fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Node, E> {
                Ok(Node::Other)
            }
        }
        d.deserialize_any(Visitor)
    }
}

impl Node {
    pub fn entry(&self, name: &str) -> Option<&(Spanned<String>, Spanned<Node>)> {
        match self {
            Self::Map(entries) => entries.iter().find(|(key, _)| key.value == name),
            _ => None,
        }
    }
}

pub(crate) fn range<T>(value: &Spanned<T>) -> Result<std::ops::Range<usize>> {
    let span = value.referenced.span();
    let start = span.byte_offset().context("YAML parser did not supply byte coordinates")? as usize;
    let len = span.byte_len().context("YAML parser did not supply byte extent")? as usize;
    Ok(start..start + len)
}

pub(crate) struct Header {
    pub node: Node,
    pub offset: usize,
    pub yaml: String,
    /// Complete extents of flow collections, paired from parser start/end
    /// events. Serde's collection span alone is only the opening delimiter.
    flows: Vec<std::ops::Range<usize>>,
    comments: Vec<std::ops::Range<usize>>,
    root_flow: Option<std::ops::Range<usize>>,
}

pub(crate) fn header(source: &str) -> Result<Header> {
    let (lines, _) = super::patch::split_managed(source)?;
    let yaml = lines.join("");
    let node = super::frontmatter(&yaml)?;
    let offset = source.find('\n').context("frontmatter fence missing")? + 1;
    use serde_saphyr::granit_parser::{Event, Parser, StructureStyle};
    let mut stack = vec![];
    let mut flows = vec![];
    let mut comments = vec![];
    let mut root_start = None;
    for event in Parser::new_from_str(&yaml) {
        let (event, span) = event?;
        let start = span.start.byte_offset().context("YAML event coordinates absent")?;
        let end = span.end.byte_offset().context("YAML event extent absent")?;
        match event {
            Event::MappingStart(style, _, _) | Event::SequenceStart(style, _, _) => {
                if stack.is_empty() && style == StructureStyle::Flow {
                    root_start = Some(start);
                }
                stack.push((start, style));
            }
            Event::MappingEnd | Event::SequenceEnd => {
                let (start, style) = stack.pop().context("unpaired YAML collection")?;
                if style == StructureStyle::Flow {
                    flows.push(start..end);
                }
            }
            Event::Comment(_, _) => comments.push(start..end),
            _ => {}
        }
    }
    let root_flow = flows.iter().find(|r| Some(r.start) == root_start).cloned();
    Ok(Header {
        node,
        offset,
        yaml,
        flows,
        comments,
        root_flow,
    })
}

impl Header {
    pub fn flow_root(&self) -> bool {
        self.root_flow.is_some()
    }

    pub fn flow_range(&self, value: &Spanned<Node>) -> Result<std::ops::Range<usize>> {
        let start = range(value)?.start;
        self.flows.iter().find(|r| r.start == start).cloned().context("unsupported flow mapping extent")
    }

    fn complete_value(&self, value: &Spanned<Node>) -> Result<std::ops::Range<usize>> {
        let range = range(value)?;
        if matches!(self.yaml.as_bytes().get(range.start), Some(b'[' | b'{')) {
            self.flows
                .iter()
                .find(|r| r.start == range.start)
                .cloned()
                .context("flow collection extent absent")
        } else {
            Ok(range)
        }
    }

    fn retained_comments(&self, range: &std::ops::Range<usize>, newline: &str) -> String {
        self.comments
            .iter()
            .filter(|r| r.start >= range.start && r.end <= range.end)
            .map(|r| format!("{}{newline}", &self.yaml[r.clone()]))
            .collect()
    }

    /// Selected flow-root entry only. Unrelated spelling/comments and body
    /// remain exact; comments inside a replaced value precede its new value.
    pub fn flow_edit(&self, name: &str, value: Option<&serde_json::Value>) -> Result<Vec<super::TidyEdit>> {
        let root = self.root_flow.as_ref().context("flow root required")?;
        let newline = if self.yaml.contains("\r\n") { "\r\n" } else { "\n" };
        let Some((key, old)) = self.node.entry(name) else {
            return Ok(value.map(|v| self.insert_member(root, name, v)).into_iter().collect());
        };
        let old = self.complete_value(old)?;
        let retained = self.retained_comments(&old, newline);
        if let Some(value) = value {
            return Ok(vec![super::TidyEdit {
                start: self.offset + old.start,
                end: self.offset + old.end,
                text: retained + &super::json(value),
            }]);
        }
        let key = range(key)?;
        let Node::Map(entries) = &self.node else { anyhow::bail!("mapping required") };
        let next = entries
            .iter()
            .filter_map(|(k, _)| range(k).ok())
            .filter(|r| r.start > old.end)
            .map(|r| r.start)
            .min()
            .unwrap_or(root.end - 1);
        // Only separators between parser-selected entries are inspected; a
        // comma inside a comment is trivia, never a mapping separator.
        let comma = |from: usize, to: usize| (from..to).find(|i| self.yaml.as_bytes()[*i] == b',' && !self.comments.iter().any(|r| r.contains(i)));
        let separator = comma(old.end, next).or_else(|| {
            let prev = entries
                .iter()
                .filter_map(|(_, v)| self.complete_value(v).ok())
                .filter(|r| r.end <= key.start)
                .map(|r| r.end)
                .max()
                .unwrap_or(root.start + 1);
            comma(prev, key.start)
        });
        let retained = self.retained_comments(&(key.start..old.end), newline);
        let mut edits = vec![super::TidyEdit {
            start: self.offset + key.start,
            end: self.offset + old.end,
            text: retained,
        }];
        if let Some(at) = separator {
            edits.push(super::TidyEdit {
                start: self.offset + at,
                end: self.offset + at + 1,
                text: String::new(),
            });
        }
        Ok(edits)
    }

    pub fn insert_member(&self, mapping: &std::ops::Range<usize>, name: &str, value: &serde_json::Value) -> super::TidyEdit {
        let at = self.offset + mapping.start + 1;
        super::TidyEdit {
            start: at,
            end: at,
            text: format!("{}: {}, ", super::json(name), super::json(value)),
        }
    }
}

pub(crate) fn patch_flow_root(input: &str, name: &str, value: Option<&serde_json::Value>) -> Result<Option<String>> {
    let header = header(input)?;
    if !header.flow_root() {
        return Ok(None);
    }
    splice(input, header.flow_edit(name, value)?).map(Some)
}

/// Apply descending, disjoint edits in the exact input. Never filesystem IO.
pub(crate) fn splice(source: &str, mut edits: Vec<super::TidyEdit>) -> Result<String> {
    edits.sort_by_key(|e| e.start);
    ensure!(edits.windows(2).all(|p| p[0].end <= p[1].start), "overlapping source edits");
    let mut out = source.to_owned();
    for e in edits.into_iter().rev() {
        ensure!(
            e.start <= e.end && e.end <= source.len() && source.is_char_boundary(e.start) && source.is_char_boundary(e.end),
            "invalid source coordinates"
        );
        out.replace_range(e.start..e.end, &e.text);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_spans_cover_values() {
        let yaml = "title: 'héllo' # keep\ntags: [a, b]\nblock: |\n  hello\n  world\nmap:\n  foo: bar\n";
        let node: Node = super::super::frontmatter(yaml).unwrap();
        let (key, value) = node.entry("title").unwrap();
        assert_eq!(&yaml[range(key).unwrap()], "title");
        assert_eq!(&yaml[range(value).unwrap()], "'héllo'");
        // Collections provide their start, not a complete value extent. Local
        // field rewrites therefore use parser key boundaries and comments.
        assert_eq!(&yaml[range(&node.entry("tags").unwrap().1).unwrap()], "[");
    }

    #[test]
    fn flow_entry_removal_keeps_separators_and_comments_not_string_lookalikes() {
        for yaml in [
            "{description: 'old', custom: 'a,b'}",
            "{custom: 'a,b', description: # between,\n 'old'}",
            "{custom: [a, {x: 1}], description: 'old', # after,\n}",
            "{description: 'old' # after,\n}",
        ] {
            let source = format!("---\n{yaml}\n---\nbody\n");
            let before: serde_json::Value = super::super::frontmatter(yaml).unwrap();
            let out = patch_flow_root(&source, "description", None).unwrap().unwrap();
            let mut expected = before;
            expected.as_object_mut().unwrap().remove("description");
            let (lines, body) = super::super::patch::split_managed(&out).unwrap();
            assert_eq!(super::super::frontmatter::<serde_json::Value>(&lines.join("")).unwrap(), expected);
            assert_eq!(body, "body\n");
            for comment in ["# between,", "# after,"] {
                if yaml.contains(comment) {
                    assert!(out.contains(comment));
                }
            }
        }
    }
}
