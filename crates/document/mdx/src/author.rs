//! Explicit author metadata edits, separate from adoption/receipt settlement.
use super::{TidyEdit, source};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

/// Selected replacements only. Description `Some("")` clears; an already
/// absent/null/empty description is unchanged. Tags replace one ordered list.
#[derive(Default, Clone, Debug)]
pub struct MetadataEdit {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    /// Explicit offset-bearing datetime; a caller resolves `--now` once.
    pub published_at: Option<String>,
}

/// Validate before planning; return proposed bytes only. Future owned state,
/// invalid source and pending attempts block instead of being normalized away.
pub fn edit_metadata(input: &str, edit: &MetadataEdit) -> Result<String> {
    let existing = super::parse(input).context("unsupported managed source; nothing was changed")?;
    ensure!(
        existing.state.pending.is_none() && existing.state.publication_attempt.is_none(),
        "pending native attempt; run recover before managed metadata edits"
    );
    let header = super::header(input)?;
    if let Some(date) = header.get("publishedAt").filter(|v| !v.is_null()) {
        let date = date.as_str().context("publishedAt must be a datetime string or null")?;
        date.parse::<jiff::Timestamp>()
            .context("publishedAt requires a valid datetime with explicit offset")?;
    }
    let mut expected = header.clone();
    let mut updates = vec![];
    if let Some(title) = &edit.title {
        ensure!(!title.trim().is_empty(), "title must not be empty");
        if title != &existing.title {
            updates.push(("title", json!(title)));
        }
    }
    if let Some(description) = &edit.description
        && leafbud_document::canonical_description(Some(description)) != leafbud_document::canonical_description(existing.description.as_deref())
    {
        updates.push(("description", json!(description)));
    }
    if let Some(tags) = &edit.tags {
        // Empty tags explicitly request [], even if previously absent.
        if header.get("tags") != Some(&json!(tags)) {
            updates.push(("tags", json!(tags)));
        }
    }
    if let Some(date) = &edit.published_at {
        let date = date
            .parse::<jiff::Timestamp>()
            .context("--published-at requires a valid datetime with explicit offset")?
            .to_string();
        if header.get("publishedAt") != Some(&json!(date)) {
            updates.push(("publishedAt", json!(date)));
        }
    }
    if updates.is_empty() {
        return Ok(input.to_owned());
    }
    let out = patch_fields(input, &updates)?;
    for (name, value) in updates {
        expected[name] = value;
    }
    ensure!(
        super::header(&out)? == expected,
        "metadata patch shape is unsupported; unrelated YAML would change; nothing was written"
    );
    let parsed = super::parse(&out).context("edited source failed validation; nothing was written")?;
    ensure!(
        parsed.state == existing.state && parsed.pages == existing.pages,
        "metadata edit changed tracking or body meaning"
    );
    ensure!(
        super::patch::split_managed(input)?.1 == super::patch::split_managed(&out)?.1,
        "metadata edit changed body bytes"
    );
    Ok(out)
}

/// Source adapter only; callers own authority and exact full-output proof.
pub(crate) fn patch_fields(input: &str, updates: &[(&str, Value)]) -> Result<String> {
    let positions = source::header(input)?;
    let mut edits = vec![];
    let mut inserts = String::new();
    for (name, value) in updates {
        if positions.flow_root() {
            edits.extend(positions.flow_edit(name, Some(value))?);
        } else if positions.node.entry(name).is_some() {
            edits.push(field_edit(input, &positions, name, value)?);
        } else {
            let newline = if input.starts_with("---\r\n") || input.starts_with("\u{feff}---\r\n") {
                "\r\n"
            } else {
                "\n"
            };
            inserts.push_str(&format!("{}: {}{newline}", super::json(name), super::json(&value)));
        }
    }
    if !inserts.is_empty() {
        let (key, _) = positions.node.entry("@leafbud").context("reserved envelope missing")?;
        ensure!(
            key.referenced.column() == 1,
            "metadata insertion requires a block-style top-level mapping; rewrite that shape explicitly first"
        );
        let at = positions.offset + source::range(key)?.start;
        edits.push(TidyEdit {
            start: at,
            end: at,
            text: inserts,
        });
    }
    source::splice(input, edits)
}

/// Rewrite only the selected entry in a block top-level mapping. Key and
/// comment locations come from the parser; comment-looking scalar content
/// cannot masquerade as YAML comments. Unsupported layouts fail closed.
fn field_edit(input: &str, header: &source::Header, name: &str, value: &Value) -> Result<TidyEdit> {
    use serde_saphyr::granit_parser::{Event, Parser};
    let source::Node::Map(entries) = &header.node else {
        anyhow::bail!("top-level mapping required");
    };
    ensure!(
        entries.iter().all(|(k, _)| k.referenced.column() == 1),
        "metadata edits require block-style top-level fields; rewrite that shape explicitly first"
    );
    let (key, _) = header.node.entry(name).context("field missing")?;
    let key_range = source::range(key)?;
    let start = header.offset + key_range.start;
    let next = entries
        .iter()
        .filter_map(|(k, _)| source::range(k).ok())
        .filter(|r| r.start > key_range.start)
        .map(|r| r.start)
        .min();
    let (lines, _) = super::patch::split_managed(input)?;
    let yaml = lines.join("");
    let end = header.offset + next.unwrap_or(yaml.len());
    let newline = if input[..end].contains("\r\n") { "\r\n" } else { "\n" };
    // Preserve comments from the old entry. Full-line comments retain all
    // indentation/trivia; comments on removed values retain their text as
    // standalone comments. Blank separator lines also survive.
    let comments: Vec<_> = Parser::new_from_str(&yaml)
        .filter_map(|event| match event {
            Ok((Event::Comment(_, _), span)) => Some(Ok(span)),
            Err(e) => Some(Err(e)),
            _ => None,
        })
        .collect::<std::result::Result<_, _>>()?;
    let mut retained = String::new();
    let mut cursor = start;
    for line in input[start..end].split_inclusive('\n') {
        if line.trim().is_empty() {
            retained.push_str(line);
        } else if let Some(span) = comments.iter().find(|s| {
            s.start.byte_offset().is_some_and(|at| {
                let at = at + header.offset;
                at >= cursor && at < cursor + line.len()
            })
        }) {
            let at = span.start.byte_offset().context("comment span absent")? + header.offset;
            if input[cursor..at].trim().is_empty() {
                retained.push_str(line);
            } else {
                retained.push_str(&input[at..cursor + line.len()]);
            }
        }
        cursor += line.len();
    }
    Ok(TidyEdit {
        start,
        end,
        text: format!("{}: {}{newline}{retained}", &input[start..header.offset + key_range.end], super::json(value)),
    })
}
