//! Keyed MDX writer, semantic validation and bounded source-preserving updates
//! using real Satteri. Plain prose stays Markdown; exact rich runs/whitespace
//! use proven Run JSX. Updates patch owned fields and the reserved envelope;
//! unrelated author text, comments and unknown fields survive.
use anyhow::{Context, Result, bail, ensure};
use satteri_arena::{Arena, Mdast, decode_string_ref_data};
use satteri_ast::mdast::*;
use serde_json::{Value, json};
use springleaf_document::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

fn json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).expect("serializable import model")
}

fn frontmatter<T: serde::de::DeserializeOwned>(source: &str) -> Result<T> {
    let options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
        merge_keys: serde_saphyr::MergeKeyPolicy::Error,
        reject_unsupported_tags: true,
        strict_booleans: true,
        no_schema: true,
        budget: serde_saphyr::budget! { max_anchors:0, max_aliases:0 },
    };
    serde_saphyr::from_str_with_options(source, options).map_err(|_| anyhow::anyhow!("invalid/restricted managed frontmatter"))
}

/// Emit ordinary, explicitly typed YAML, quoting every key/string (including
/// UUIDs and @springleaf). No aliases/tags/duplicate keys are generated.
fn yaml(value: &Value, indent: usize, out: &mut String) {
    if let Some(map) = value.as_object() {
        for (key, value) in map {
            let _ = write!(out, "{}{}:", " ".repeat(indent), json(key));
            if value.as_object().is_some_and(|v| !v.is_empty()) {
                out.push('\n');
                yaml(value, indent + 2, out);
            } else {
                let _ = writeln!(out, " {}", json(value));
            }
        }
    }
}

fn inline(content: &[Inline]) -> String {
    if content.is_empty() {
        return "<Run text={\"\"} />".into();
    }
    let mut out = String::new();
    for value in content {
        match value {
            Inline::HardBreak => out.push_str("<HardBreak />"),
            Inline::Run { text, marks } => {
                if !marks.is_empty() || text.starts_with(char::is_whitespace) || text.ends_with(char::is_whitespace) || text.contains(['\n', '\r', '\t']) {
                    let _ = write!(out, "<Run text={{{}}} marks={{{}}} />", json(text), json(marks));
                } else {
                    for c in text.chars() {
                        if c.is_ascii_punctuation() {
                            out.push('\\');
                        }
                        out.push(c);
                    }
                }
            }
        }
    }
    out
}

fn render_frontmatter(document: &Document) -> Result<String> {
    let mut out = "---\n".to_owned();
    yaml(&serde_json::to_value(document)?, 0, &mut out);
    out.push_str("---\n");
    Ok(out)
}

fn render_body_mode(document: &Document, literal_blocks: bool) -> String {
    // The blank line after the closing fence is part of the body part, so
    // `frontmatter + body` reproduces the original single-writer format.
    let mut out = "\n".to_owned();
    for page in &document.pages {
        let _ = writeln!(out, "<Page name={} kind=\"linear\">\n", json(&page.key));
        for block in &page.blocks {
            let _ = writeln!(out, "<Key id={} />", json(&block.key));
            if literal_blocks {
                let tag = match &block.kind {
                    BlockKind::Paragraph { .. } => Some("Paragraph"),
                    BlockKind::Heading { .. } => Some("Heading"),
                    BlockKind::Quote { .. } => Some("Quote"),
                    _ => None,
                };
                if let Some(tag) = tag {
                    let (content, level) = match &block.kind {
                        BlockKind::Paragraph { content } | BlockKind::Quote { content } => (content, None),
                        BlockKind::Heading { content, level } => (content, Some(level)),
                        _ => unreachable!(),
                    };
                    let _ = writeln!(out, "<{tag}{}>\n", level.map(|l| format!(" level={{{l}}}")).unwrap_or_default());
                    for run in content {
                        match run {
                            Inline::Run { text, marks } => {
                                let _ = writeln!(out, "<Run text={{{}}} marks={{{}}} />\n", json(text), json(marks));
                            }
                            Inline::HardBreak => out.push_str("<HardBreak />\n\n"),
                        }
                    }
                    let _ = writeln!(out, "</{tag}>\n");
                    continue;
                }
            }
            match &block.kind {
                BlockKind::Paragraph { content } => {
                    let _ = writeln!(out, "{}", inline(content));
                }
                BlockKind::Heading { level, content } => {
                    let _ = writeln!(out, "{} {}", "#".repeat((*level).into()), inline(content));
                }
                BlockKind::Quote { content } => {
                    let _ = writeln!(out, "> {}", inline(content));
                }
                // Exact trailing whitespace/newline semantics; no fence normalization.
                BlockKind::Code { language, text } => {
                    let _ = writeln!(out, "<Code language={{{}}} text={{{}}} />", json(language), json(text));
                }
                BlockKind::Rule => out.push_str("***\n"),
                BlockKind::PageLink { page } => {
                    let _ = writeln!(out, "<PageLink page={} />", json(page));
                }
                BlockKind::Image {
                    src,
                    fallback,
                    width,
                    height,
                    alt,
                } => {
                    let _ = writeln!(
                        out,
                        "<Image src={{{}}} fallback={{{}}} width={{{}}} height={{{}}} alt={{{}}} />",
                        json(src),
                        json(fallback),
                        json(width),
                        json(height),
                        json(alt)
                    );
                }
            }
            out.push('\n');
        }
        out.push_str("</Page>\n\n");
    }
    out
}

/// Choose and verify the body representation against the composed source.
/// Standard Markdown may introduce automatic links or split a flow JSX run
/// from a paragraph; explicit typed block/run boundaries are the fallback.
/// Both representations must prove exact semantic equality through Satteri.
fn verified_body(document: &Document, frontmatter: &str) -> Result<String> {
    let mut body = render_body_mode(document, false);
    if !parse(&format!("{frontmatter}{body}")).is_ok_and(|p| p.semantic_value() == document.semantic_value()) {
        body = render_body_mode(document, true);
    }
    let parsed = parse(&format!("{frontmatter}{body}")).context("generated MDX validation")?;
    ensure!(
        parsed.semantic_value() == document.semantic_value() && parsed.state == document.state,
        "MDX semantic round-trip would lose content/associations"
    );
    Ok(body)
}

pub fn render(document: &Document) -> Result<String> {
    let frontmatter = render_frontmatter(document)?;
    let body = verified_body(document, &frontmatter)?;
    Ok(frontmatter + &body)
}

struct Reader {
    arena: Arena<Mdast>,
    keys: BTreeSet<String>,
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
    fn register(&mut self, key: String) -> Result<String> {
        let mut bytes = key.bytes();
        ensure!(
            bytes.next().is_some_and(|b| b.is_ascii_alphabetic()) && bytes.all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b)),
            "invalid source key spelling"
        );
        ensure!(self.keys.insert(key.clone()), "duplicate source key");
        Ok(key)
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
                if matches!(self.name(id), "Paragraph" | "Heading" | "Quote") {
                    let mut content = vec![];
                    self.literal_rich(id, &mut content)?;
                    let kind = match self.name(id) {
                        "Paragraph" => BlockKind::Paragraph { content },
                        "Quote" => BlockKind::Quote { content },
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
    fn page(&mut self, id: u32) -> Result<Page> {
        let mut props = self.props(id)?;
        let name: String = self.prop(&mut props, "name")?;
        let kind: String = self.prop(&mut props, "kind")?;
        ensure!(props.is_empty() && kind == "linear", "unsupported page properties");
        let key = self.register(name)?;
        let children = self.arena.get_children(id).to_vec();
        let mut blocks = vec![];
        let mut pending = None;
        for id in children {
            if self.kind(id) == MdastNodeType::MdxJsxFlowElement && self.name(id) == "Key" {
                ensure!(pending.is_none(), "duplicate Key marker");
                let mut props = self.props(id)?;
                let key: String = self.prop(&mut props, "id")?;
                ensure!(props.is_empty() && self.arena.get_children(id).is_empty(), "invalid Key marker");
                pending = Some(self.register(key)?);
            } else {
                blocks.push(self.block(id, pending.take().context("block has no source Key")?)?);
            }
        }
        ensure!(pending.is_none(), "orphan Key marker");
        Ok(Page { key, blocks })
    }
}

pub fn parse(source: &str) -> Result<Document> {
    let (arena, errors) = satteri_pulldown_cmark::parse(source, satteri_pulldown_cmark::MDX_OPTIONS);
    ensure!(errors.is_empty(), "invalid native MDX syntax ({} diagnostics)", errors.len());
    let mut reader = Reader { arena, keys: BTreeSet::new() };
    let children = reader.arena.get_children(0).to_vec();
    let first = *children.first().context("empty managed document")?;
    ensure!(reader.kind(first) == MdastNodeType::Yaml, "YAML frontmatter required");
    let mut document: Document = frontmatter(reader.value(first))?;
    ensure!(
        document.format == "springleaf-mdx/1" && document.state.sync.profile == PROFILE && document.state.state == "draft",
        "unsupported managed format/profile/lifecycle"
    );
    ensure!(!document.title.trim().is_empty(), "empty document title");
    for &id in children.iter().skip(1) {
        ensure!(
            reader.kind(id) == MdastNodeType::MdxJsxFlowElement && reader.name(id) == "Page",
            "expected Page definition"
        );
        document.pages.push(reader.page(id)?);
    }
    ensure!(!document.pages.is_empty(), "missing main page");
    ensure!(
        reader.keys.len() == document.state.nodes.len() && reader.keys.iter().all(|k| document.state.nodes.contains_key(k)),
        "source keys and node bindings differ"
    );
    let page_names: BTreeSet<_> = document.pages.iter().map(|p| &p.key).collect();
    for page in &document.pages {
        for block in &page.blocks {
            if let BlockKind::PageLink { page } = &block.kind {
                ensure!(page_names.contains(page), "unresolved page link");
            }
        }
    }
    Ok(document)
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

/// Split a managed source into its frontmatter lines (fences excluded) and
/// the exact body text following the closing fence. The MDX frontmatter
/// block ends at the first `---` line, matching the native parser's view.
fn split_managed(source: &str) -> Result<(Vec<String>, String)> {
    ensure!(source.starts_with("---\n"), "managed frontmatter opening fence missing");
    let mut lines = Vec::new();
    let mut cursor = "---\n".len();
    loop {
        let Some(nl) = source[cursor..].find('\n') else {
            bail!("managed frontmatter closing fence missing");
        };
        let line = &source[cursor..cursor + nl + 1];
        if line == "---\n" {
            return Ok((lines, source[cursor + line.len()..].to_owned()));
        }
        lines.push(line.to_owned());
        cursor += line.len();
    }
}

/// Top-level mapping key of a column-0 line, quoted or plain. Comment lines,
/// nested content and flow continuations (always indented in valid block
/// mappings) yield None; a multiline scalar therefore cannot impersonate a key.
fn column_key(line: &str) -> Option<&str> {
    let line = line.strip_suffix('\n').unwrap_or(line);
    if line.is_empty() || line.starts_with([' ', '\t', '#']) {
        return None;
    }
    if let Some(q) = line.chars().next().filter(|c| *c == '"' || *c == '\'') {
        let close = line[1..].find(q)? + 1;
        let name = &line[1..close];
        line[close + 1..].trim_start().starts_with(':').then_some(name)
    } else {
        let colon = line.find(':')?;
        let name = line[..colon].trim_end();
        (!name.is_empty() && !name.starts_with('[') && !name.starts_with('{')).then_some(name)
    }
}

/// Locate the single column-0 definition of `key`.
fn find_key(lines: &[String], key: &str) -> Result<Option<usize>> {
    let mut found = None;
    for (index, line) in lines.iter().enumerate() {
        if column_key(line) == Some(key) {
            ensure!(found.replace(index).is_none(), "duplicate frontmatter key {key:?} cannot be patched");
        }
    }
    Ok(found)
}

/// Exclusive end of a key's value extent: the key line plus following indented
/// or blank lines, with trailing blank separators excluded.
fn value_extent(lines: &[String], key_line: usize) -> usize {
    let mut end = key_line + 1;
    while end < lines.len() && (lines[end].starts_with([' ', '\t']) || lines[end].trim().is_empty()) {
        end += 1;
    }
    while end > key_line + 1 && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    end
}

fn key_lines(key: &str, value: &Value) -> Vec<String> {
    let mut out = String::new();
    yaml(&json!({ key.to_owned(): value }), 0, &mut out);
    out.split_inclusive('\n').map(str::to_owned).collect()
}

/// Replace, insert or remove one owned top-level key. An unchanged value
/// touches nothing; a removed value (null) deletes the extent; an absent key
/// with a value is inserted before the reserved envelope.
fn patch_key(lines: &mut Vec<String>, key: &str, new: &Value) -> Result<()> {
    match find_key(lines, key)? {
        Some(index) => {
            let end = value_extent(lines, index);
            if new.is_null() {
                lines.splice(index..end, []);
            } else {
                lines.splice(index..end, key_lines(key, new));
            }
        }
        None if new.is_null() => {}
        None => {
            let at = find_key(lines, "@springleaf")?.context("reserved @springleaf envelope not found for source-preserving patch")?;
            lines.splice(at..at, key_lines(key, new));
        }
    }
    Ok(())
}

/// Source-preserving update for guarded refresh. `document` is the reviewed
/// next state of THIS document (same format, id and binding token). Owned
/// author fields that changed are rewritten in normalized form; unchanged
/// fields, comments, blank lines and unknown/local-only fields keep their
/// bytes; the reserved `@springleaf` envelope is regenerated. The body is
/// re-serialized only when page content actually changed. The result is
/// verified to parse back to exactly `document`.
pub fn update(source: &str, document: &Document) -> Result<String> {
    let existing = parse(source)?;
    ensure!(existing.format == document.format, "update cannot change the document format");
    ensure!(
        existing.state.id == document.state.id && existing.state.binding.token == document.state.binding.token,
        "update cannot change document identity"
    );
    let (mut lines, body) = split_managed(source)?;
    for (key, old, new) in [
        ("title", json!(existing.title), json!(document.title)),
        ("description", json!(existing.description), json!(document.description)),
        ("tags", json!(existing.tags), json!(document.tags)),
        ("publication", json!(existing.publication), json!(document.publication)),
        ("theme", json!(existing.theme), json!(document.theme)),
        (
            "contributors",
            json!(existing.extra.get("contributors")),
            json!(document.extra.get("contributors")),
        ),
    ] {
        if old == new {
            continue;
        }
        patch_key(&mut lines, key, &new)?;
    }
    let envelope = find_key(&lines, "@springleaf")?.context("reserved @springleaf envelope not found for source-preserving patch")?;
    let end = value_extent(&lines, envelope);
    let mut regenerated = String::new();
    yaml(&json!({"@springleaf": &document.state}), 0, &mut regenerated);
    lines.splice(envelope..end, regenerated.split_inclusive('\n').map(str::to_owned));
    let frontmatter = "---\n".to_owned() + &lines.join("") + "---\n";
    let body = if existing.pages == document.pages {
        body
    } else {
        verified_body(document, &frontmatter)?
    };
    let result = frontmatter + &body;
    let parsed = parse(&result).context("source-preserving update validation")?;
    ensure!(
        parsed.semantic_value() == document.semantic_value() && parsed.state == document.state,
        "source-preserving update would lose content/associations"
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn node(entity: &str) -> Node {
        Node {
            entity: entity.into(),
            memberships: BTreeMap::new(),
            text_fact: None,
            inactive_text_fact: None,
        }
    }

    fn document() -> Document {
        let state = State {
            id: Uuid::nil(),
            state: "draft".into(),
            binding: Binding {
                base_url: "https://leaflet.pub".into(),
                token: Uuid::nil(),
                root: "e-root".into(),
                entity_set: "es".into(),
                entry_fact: "f-entry".into(),
                publication: None,
            },
            nodes: BTreeMap::from([("p-main".into(), node("e-main")), ("b-1".into(), node("e-b1"))]),
            sync: Sync {
                profile: PROFILE.into(),
                local_hash: "sha256:old-local".into(),
                remote_hash: "sha256:old-remote".into(),
            },
        };
        Document {
            format: "springleaf-mdx/1".into(),
            title: "T".into(),
            description: None,
            tags: vec![],
            publication: None,
            theme: None,
            extra: BTreeMap::new(),
            pages: vec![Page {
                key: "p-main".into(),
                blocks: vec![Block {
                    key: "b-1".into(),
                    kind: BlockKind::Paragraph {
                        content: vec![Inline::Run {
                            text: "hello".into(),
                            marks: vec![],
                        }],
                    },
                }],
            }],
            state,
        }
    }

    /// Hand-shaped variant of a rendered file: author comments, an unquoted
    /// key, blank separators and an unknown local-only field.
    fn authored(source: &str) -> String {
        source
            .replacen("\"title\": \"T\"", "title: T # chosen by the author", 1)
            .replacen("\"format\": \"springleaf-mdx/1\"\n", "\"format\": \"springleaf-mdx/1\"\n# author note\n", 1)
            .replacen("\n\"@springleaf\":", "\ncustomField: {\"meaning\": \"retained\"}\n\n\"@springleaf\":", 1)
    }

    #[test]
    fn update_settles_receipts_preserving_author_text_and_body() {
        let original = authored(&render(&document()).unwrap());
        let mut next = parse(&original).unwrap();
        next.state.sync.remote_hash = "sha256:new-remote".into();
        let updated = update(&original, &next).unwrap();
        assert!(updated.contains("title: T # chosen by the author"));
        assert!(updated.contains("# author note"));
        assert!(updated.contains("customField: {\"meaning\": \"retained\"}"));
        assert!(updated.contains("sha256:new-remote"));
        assert!(!updated.contains("sha256:old-remote"));
        assert_eq!(body_of(&updated), body_of(&original));
        assert_eq!(parse(&updated).unwrap(), next);
    }

    #[test]
    fn update_patches_changed_fields_and_rewrites_only_changed_bodies() {
        let original = authored(&render(&document()).unwrap());
        let mut next = parse(&original).unwrap();
        next.title = "Remote title".into();
        next.description = Some("From the editor".into());
        next.tags = vec!["one".into()];
        let updated = update(&original, &next).unwrap();
        assert!(updated.contains("\"title\": \"Remote title\""));
        assert!(!updated.contains("chosen by the author"));
        assert!(updated.contains("\"description\": \"From the editor\""));
        assert!(updated.contains("customField: {\"meaning\": \"retained\"}"));
        assert_eq!(body_of(&updated), body_of(&original));
        // A truly changed body is re-serialized; unchanged frontmatter lines stay.
        next.pages[0].blocks[0].kind = BlockKind::Paragraph {
            content: vec![Inline::Run {
                text: "edited remotely".into(),
                marks: vec![],
            }],
        };
        let updated = update(&updated, &next).unwrap();
        assert!(updated.contains("edited remotely"));
        assert_ne!(body_of(&updated), body_of(&original));
        assert!(updated.contains("customField: {\"meaning\": \"retained\"}"));
        let mut removed = next.clone();
        removed.description = None;
        removed.tags = vec![];
        let updated = update(&updated, &removed).unwrap();
        assert!(!updated.contains("\"description\""));
        assert!(updated.contains("\"tags\": []"));
    }

    #[test]
    fn update_refuses_foreign_identity_and_verifies_output() {
        let original = render(&document()).unwrap();
        let mut foreign = document();
        foreign.state.id = Uuid::new_v5(&Uuid::NAMESPACE_URL, b"foreign");
        assert!(update(&original, &foreign).is_err());
        // A body change that cannot round-trip (duplicate key) is caught.
        let mut broken = document();
        broken.pages[0].blocks.push(Block {
            key: "b-1".into(),
            kind: BlockKind::Rule,
        });
        assert!(update(&original, &broken).is_err());
    }

    fn body_of(source: &str) -> &str {
        source.split_once("---\n\n").expect("fence").1
    }
}
