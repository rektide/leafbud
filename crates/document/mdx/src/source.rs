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
}

pub(crate) fn header(source: &str) -> Result<Header> {
    let (lines, _) = super::patch::split_managed(source)?;
    let yaml = lines.join("");
    let node = super::frontmatter(&yaml)?;
    let offset = source.find('\n').context("frontmatter fence missing")? + 1;
    Ok(Header { node, offset })
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
}
