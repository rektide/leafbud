//! Real Satteri parser -> scoped Key consumption -> typed, validated authoring IR.
//! The semantic schema is ours; native JSX parsing alone is not data validation.
use std::collections::{BTreeMap, BTreeSet, HashMap};

use satteri_arena::{Arena, Mdast, decode_string_ref_data};
use satteri_ast::mdast::*;
use serde::Deserialize;
use serde_json::Value;

use crate::{literal, model::*};

type Checked<T> = Result<T, Diagnostic>;

struct Lower<'a> {
    arena: Arena<Mdast>,
    source: &'a str,
    base: usize,
    keys: HashMap<u32, AuthorKey>,
    namespace: BTreeSet<String>,
    consumed: usize,
}

fn valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|c| c.is_ascii_alphabetic()) && bytes.all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}

impl Lower<'_> {
    fn kind(&self, id: u32) -> MdastNodeType {
        MdastNodeType::from_u8(self.arena.get_node(id).node_type).expect("native node kind")
    }
    fn span(&self, id: u32) -> Span {
        let n = self.arena.get_node(id);
        Span {
            start: self.base + n.start_offset as usize,
            end: self.base + n.end_offset as usize,
        }
    }
    fn error(&self, id: u32, code: &'static str, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            code,
            message: message.into(),
            span: self.span(id),
        }
    }
    fn value(&self, id: u32) -> &str {
        self.arena.get_str(decode_string_ref_data(self.arena.get_type_data(id)))
    }
    fn name(&self, id: u32) -> &str {
        self.arena.get_str(decode_mdx_jsx_element_name(self.arena.get_type_data(id)))
    }
    fn jsx(&self, id: u32) -> bool {
        matches!(self.kind(id), MdastNodeType::MdxJsxFlowElement | MdastNodeType::MdxJsxTextElement)
    }
    fn is_key(&self, id: u32) -> bool {
        self.jsx(id) && self.name(id) == "Key"
    }
    fn register(&mut self, key: AuthorKey) -> Checked<AuthorKey> {
        if !valid_name(&key.name) {
            return Err(Diagnostic {
                code: "invalid-key",
                message: "expected [A-Za-z][A-Za-z0-9_.-]*".into(),
                span: key.span,
            });
        }
        if !self.namespace.insert(key.name.clone()) {
            return Err(Diagnostic {
                code: "duplicate-key",
                message: format!("duplicate author key {:?}", key.name),
                span: key.span,
            });
        }
        Ok(key)
    }
    fn bind(&mut self, parent: u32) -> Checked<()> {
        let children = self.arena.get_children(parent).to_vec();
        // Do not recurse into a reserved paired Key and invent group semantics.
        for &child in &children {
            if !self.is_key(child) {
                self.bind(child)?;
            }
        }
        let mut keep = vec![];
        for (i, &id) in children.iter().enumerate() {
            if !self.is_key(id) {
                keep.push(id);
                continue;
            }
            if self.kind(id) != MdastNodeType::MdxJsxFlowElement {
                return Err(self.error(id, "misplaced-key", "Key must be standalone flow JSX"));
            }
            let span = self.span(id);
            if !self.source[span.start..span.end].trim_end().ends_with("/>") {
                return Err(self.error(
                    id,
                    "paired-key-unsupported",
                    "use self-closing <Key id=\"name\" />; paired groups are outside this slice",
                ));
            }
            let bytes = self.arena.get_type_data(id);
            if decode_mdx_jsx_attr_count(bytes) != 1 {
                return Err(self.error(id, "malformed-key", "Key requires exactly one quoted id"));
            }
            let (kind, name, value) = decode_mdx_jsx_attr(bytes, 0);
            if kind != MDX_ATTR_LITERAL_PROP || self.arena.get_str(name) != "id" {
                return Err(self.error(id, "malformed-key", "Key requires exactly one quoted id"));
            }
            let name = self.arena.get_str(value).to_owned();
            if i > 0 {
                let previous = self.arena.get_node(children[i - 1]);
                let marker = self.arena.get_node(id);
                if previous.end_line.saturating_add(1) >= marker.start_line {
                    return Err(self.error(id, "key-boundary", "put a blank line before Key after an earlier sibling"));
                }
            }
            let Some(&next) = children.get(i + 1) else {
                return Err(self.error(id, "orphan-key", "no next sibling in this container"));
            };
            use MdastNodeType as M;
            if self.is_key(next)
                || !matches!(
                    self.kind(next),
                    M::Paragraph | M::Heading | M::Code | M::List | M::Blockquote | M::ThematicBreak | M::MdxJsxFlowElement
                )
            {
                return Err(self.error(
                    id,
                    "orphan-key",
                    "immediate next sibling is not a supported block; declarations never skip siblings",
                ));
            }
            let key = self.register(AuthorKey { name, span })?;
            self.keys.insert(next, key);
            self.consumed += 1;
        }
        self.arena.set_children(parent, &keep);
        Ok(())
    }
    fn props<T: for<'de> Deserialize<'de>>(&self, id: u32) -> Checked<T> {
        let bytes = self.arena.get_type_data(id);
        let mut map = serde_json::Map::new();
        for i in 0..decode_mdx_jsx_attr_count(bytes) {
            let (kind, name, raw) = decode_mdx_jsx_attr(bytes, i);
            let name = self.arena.get_str(name).to_owned();
            let raw = self.arena.get_str(raw);
            let value = match kind {
                MDX_ATTR_LITERAL_PROP => Value::String(raw.into()),
                MDX_ATTR_BOOLEAN_PROP => Value::Bool(true),
                MDX_ATTR_EXPRESSION_PROP => literal::parse(raw).map_err(|e| self.error(id, "nonliteral-prop", format!("{name}: {e}")))?,
                _ => return Err(self.error(id, "spread-prop", "spread/unknown attribute kinds are not literal data")),
            };
            if map.insert(name.clone(), value).is_some() {
                return Err(self.error(id, "duplicate-prop", format!("duplicate property {name}")));
            }
        }
        serde_json::from_value(Value::Object(map)).map_err(|e| self.error(id, "component-schema", e.to_string()))
    }
    fn leaf(&self, id: u32) -> Checked<()> {
        if !self.arena.get_children(id).is_empty() {
            return Err(self.error(id, "component-children", "this component requires no children"));
        }
        Ok(())
    }
    fn component_key(&mut self, id: u32, own: Option<String>) -> Checked<Option<AuthorKey>> {
        let marker = self.keys.get(&id).cloned();
        if own.is_some() && marker.is_some() {
            return Err(self.error(id, "key-collision", "both Key and a component key supplied; neither takes precedence"));
        }
        match own {
            Some(name) => self.register(AuthorKey { name, span: self.span(id) }).map(Some),
            None => Ok(marker),
        }
    }
    fn inlines(&self, parent: u32, marks: &[Mark], out: &mut Vec<Inline>) -> Checked<()> {
        use MdastNodeType as M;
        for &id in self.arena.get_children(parent) {
            match self.kind(id) {
                M::Text => push_run(out, &self.value(id).replace(['\r', '\n'], " "), marks),
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
                            if !self.arena.get_str(data.title).is_empty() {
                                return Err(self.error(id, "link-title-unsupported", "link titles have no mapping in this slice"));
                            }
                            let href = self.arena.get_str(data.url);
                            if !href.starts_with("https://") && !href.starts_with("http://") && !href.starts_with("mailto:") {
                                return Err(self.error(id, "link-scheme", "this slice permits http, https and mailto links only"));
                            }
                            Mark::Link { href: href.into() }
                        }
                    });
                    self.inlines(id, &next, out)?;
                }
                M::Break => out.push(Inline::HardBreak),
                M::MdxJsxTextElement => match self.name(id) {
                    "Run" => {
                        let props: RunProps = self.props(id)?;
                        self.leaf(id)?;
                        push_run(out, &props.text, marks);
                    }
                    "HardBreak" => {
                        let _: EmptyProps = self.props(id)?;
                        self.leaf(id)?;
                        out.push(Inline::HardBreak);
                    }
                    name => return Err(self.error(id, "unsupported-inline-component", format!("{name} is not an implemented inline component"))),
                },
                _ => return Err(self.error(id, "unsupported-inline", format!("{} has no inline mapping", self.kind(id).name()))),
            }
        }
        Ok(())
    }
    fn blocks(&mut self, parent: u32) -> Checked<Vec<Block>> {
        self.arena.get_children(parent).to_vec().into_iter().map(|id| self.block(id)).collect()
    }
    fn block(&mut self, id: u32) -> Checked<Block> {
        use MdastNodeType as M;
        let mut key = self.keys.get(&id).cloned();
        let kind = match self.kind(id) {
            M::Paragraph | M::Heading => {
                let mut content = vec![];
                self.inlines(id, &[], &mut content)?;
                if self.kind(id) == M::Heading {
                    BlockKind::Heading {
                        level: self.arena.get_type_data(id)[0],
                        content,
                    }
                } else {
                    BlockKind::Paragraph { content }
                }
            }
            M::Code => {
                let data = CodeData::from_bytes(self.arena.get_type_data(id));
                if !self.arena.get_str(data.meta).is_empty() {
                    return Err(self.error(id, "code-meta-unsupported", "fence metadata is outside this compiler slice"));
                }
                BlockKind::Code {
                    language: self.arena.get_str(data.lang).into(),
                    text: self.arena.get_str(data.value).into(),
                }
            }
            M::ThematicBreak => BlockKind::Rule,
            M::List => {
                let data = ListData::from_bytes(self.arena.get_type_data(id));
                let mut items = vec![];
                for item in self.arena.get_children(id).to_vec() {
                    if self.kind(item) != M::ListItem || ListItemData::from_bytes(self.arena.get_type_data(item)).checked != 2 {
                        return Err(self.error(item, "task-list-unsupported", "only ordinary list items are implemented"));
                    }
                    let blocks = self.blocks(item)?;
                    if blocks.len() != 1 || !matches!(blocks[0].kind, BlockKind::Paragraph { .. }) {
                        return Err(self.error(
                            item,
                            "list-shape-unsupported",
                            "this slice requires one paragraph per item; nested/loose multi-block items are not flattened",
                        ));
                    }
                    items.push(blocks);
                }
                BlockKind::List {
                    ordered: data.ordered,
                    start: data.start,
                    items,
                }
            }
            M::Blockquote => {
                let mut blocks = self.blocks(id)?;
                if blocks.len() != 1 || !matches!(blocks[0].kind, BlockKind::Paragraph { .. }) {
                    return Err(self.error(id, "quote-shape-unsupported", "this slice maps one paragraph per blockquote"));
                }
                BlockKind::Quote {
                    paragraph: Box::new(blocks.remove(0)),
                }
            }
            M::MdxJsxFlowElement => match self.name(id) {
                "Poll" => {
                    let props: PollProps = self.props(id)?;
                    self.leaf(id)?;
                    key = self.component_key(id, props.key)?;
                    if key.is_none() {
                        return Err(self.error(id, "poll-key-required", "Poll requires a stable author key, directly or from Key"));
                    }
                    if !(2..=8).contains(&props.options.len())
                        || props.options.iter().any(|o| o.trim().is_empty())
                        || props.options.iter().collect::<BTreeSet<_>>().len() != props.options.len()
                    {
                        return Err(self.error(id, "poll-options", "expected 2–8 distinct nonempty option strings"));
                    }
                    BlockKind::Poll { options: props.options }
                }
                "PageLink" => {
                    let props: PageLinkProps = self.props(id)?;
                    self.leaf(id)?;
                    key = self.component_key(id, props.key)?;
                    BlockKind::PageLink { page: props.page }
                }
                "Run" => {
                    let props: RunProps = self.props(id)?;
                    self.leaf(id)?;
                    let mut content = vec![];
                    push_run(&mut content, &props.text, &[]);
                    BlockKind::Paragraph { content }
                }
                "Canvas" | "Place" | "Image" | "Page" => {
                    return Err(self.error(
                        id,
                        "unimplemented-component",
                        format!("{} has no block mapping here (Page definitions are root-only)", self.name(id)),
                    ));
                }
                name => return Err(self.error(id, "unknown-component", format!("unknown flow component {name:?}"))),
            },
            _ => return Err(self.error(id, "unsupported-block", format!("{} has no block mapping", self.kind(id).name()))),
        };
        Ok(Block {
            span: self.span(id),
            key,
            kind,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PollProps {
    #[serde(default)]
    key: Option<String>,
    options: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageLinkProps {
    #[serde(default)]
    key: Option<String>,
    page: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageProps {
    name: String,
    #[serde(default)]
    kind: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunProps {
    text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyProps {}

fn references(blocks: &[Block]) -> Vec<(&str, Span)> {
    let mut result = vec![];
    for b in blocks {
        match &b.kind {
            BlockKind::PageLink { page } => result.push((page.as_str(), b.span)),
            BlockKind::List { items, .. } => {
                for item in items {
                    result.extend(references(item));
                }
            }
            BlockKind::Quote { paragraph } => result.extend(references(std::slice::from_ref(paragraph))),
            _ => {}
        }
    }
    result
}

fn validate_references(doc: &Document<'_>) -> Checked<()> {
    let pages: BTreeMap<_, _> = doc.pages.iter().map(|p| (p.name.as_str(), p)).collect();
    fn visit<'a>(blocks: &'a [Block], pages: &BTreeMap<&'a str, &'a Page>, stack: &mut Vec<&'a str>) -> Checked<()> {
        for (name, span) in references(blocks) {
            let page = pages.get(name).ok_or_else(|| Diagnostic {
                code: "missing-page",
                message: format!("undefined page {name:?}"),
                span,
            })?;
            if stack.contains(&name) {
                return Err(Diagnostic {
                    code: "page-cycle",
                    message: format!("cyclic page reference {name:?}"),
                    span,
                });
            }
            stack.push(name);
            visit(&page.blocks, pages, stack)?;
            stack.pop();
        }
        Ok(())
    }
    visit(&doc.main, &pages, &mut vec![])?;
    for page in &doc.pages {
        visit(&page.blocks, &pages, &mut vec![page.name.as_str()])?;
    }
    Ok(())
}

/// No typed document is returned on any native or semantic diagnostic.
pub fn compile(source: &str) -> Result<Document<'_>, Vec<Diagnostic>> {
    let base = if source.starts_with('\u{feff}') { 3 } else { 0 };
    let (arena, errors) = satteri_pulldown_cmark::parse(source, satteri_pulldown_cmark::MDX_OPTIONS);
    if !errors.is_empty() {
        return Err(errors
            .into_iter()
            .map(|(at, message)| Diagnostic {
                code: "parse",
                message,
                span: Span {
                    start: base + at,
                    end: base + at,
                },
            })
            .collect());
    }
    let mut lower = Lower {
        arena,
        source,
        base,
        keys: HashMap::new(),
        namespace: BTreeSet::new(),
        consumed: 0,
    };
    fn finish<'a>(lower: &mut Lower<'a>) -> Checked<Document<'a>> {
        lower.bind(0)?;
        let children = lower.arena.get_children(0).to_vec();
        let Some(&frontmatter) = children.first().filter(|&&id| lower.kind(id) == MdastNodeType::Yaml) else {
            return Err(lower.error(0, "frontmatter-required", "start with YAML frontmatter (format/title/tags/date intent)"));
        };
        let metadata: Metadata = serde_saphyr::from_str(lower.value(frontmatter)).map_err(|e| lower.error(frontmatter, "frontmatter", e.to_string()))?;
        if metadata.format != "springleaf-mdx/1" || metadata.title.trim().is_empty() {
            return Err(lower.error(frontmatter, "frontmatter", "expected format springleaf-mdx/1 and a nonempty title"));
        }
        let mut main = vec![];
        let mut pages = vec![];
        let mut names = BTreeSet::new();
        for id in children.into_iter().skip(1) {
            if lower.jsx(id) && lower.name(id) == "Page" {
                let props: PageProps = lower.props(id)?;
                if !valid_name(&props.name) || props.name == "main" || !names.insert(props.name.clone()) {
                    return Err(lower.error(id, "page-name", "page names must be valid, unique, and not main"));
                }
                if props.kind.as_deref().is_some_and(|k| k != "linear") {
                    return Err(lower.error(id, "page-kind", "only linear Page definitions are implemented"));
                }
                if lower.keys.contains_key(&id) {
                    return Err(lower.error(id, "definition-key", "Page name identifies a definition; Key names a block, not a definition"));
                }
                pages.push(Page {
                    name: props.name,
                    span: lower.span(id),
                    blocks: lower.blocks(id)?,
                });
            } else {
                main.push(lower.block(id)?);
            }
        }
        let doc = Document {
            original_source: lower.source,
            metadata,
            main,
            pages,
            consumed_keys: lower.consumed,
        };
        validate_references(&doc)?;
        Ok(doc)
    }
    finish(&mut lower).map_err(|d| vec![d])
}
