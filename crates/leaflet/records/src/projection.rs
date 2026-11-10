//! Bounded native public-meaning mapping, independent of editor agreement.
//! No upload, offload, public write, historical body or prose matching engine.
use anyhow::{Context, Result, bail, ensure};
use leafbud_document::{
    Block, BlockKind, Document, Inline, ListStyle, Mark, Node, Occurrence, Page, Publication, RELEASE_PROFILE, canonical_description, push_run,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub struct Projection {
    /// Canonical supported public meaning, including public page identities.
    pub value: Value,
    /// Transient document for the existing human Markdown comparison writer.
    /// Source keys are presentation-only; they are not part of `value`.
    pub document: Document,
    pub metadata: Value,
}

fn fields(value: &Value, supported: &[&str]) -> Result<()> {
    let object = value.as_object().context("expected public object")?;
    for name in object.keys() {
        ensure!(
            supported.contains(&name.as_str()),
            "unsupported public field {name}; complete comparison refused"
        );
    }
    Ok(())
}
fn text(value: &Value, name: &str) -> Result<String> {
    Ok(value[name].as_str().with_context(|| format!("missing public {name}"))?.to_owned())
}
fn date(value: &Value) -> Result<String> {
    Ok(value
        .as_str()
        .context("publication date unavailable; no current time is invented")?
        .parse::<jiff::Timestamp>()?
        .to_string())
}

/// Convert supported current source to the actual native public record shape,
/// then use the same public decoder/canonicalization as the fetched endpoint.
pub fn working(source: &Document) -> Result<Projection> {
    let binding = source.state.binding.as_ref().context("public projection requires a bound document")?;
    let uri = leafbud_atproto::AtUri::parse(binding.uri.as_deref().context("public URI is not established")?)?;
    ensure!(
        uri.collection == "site.standard.document",
        "release comparison for legacy collection is unavailable"
    );
    supported_metadata(source)?;
    let page_id = |key: &str| -> Result<String> {
        let node = source
            .state
            .nodes
            .get(key)
            .context("new page has no native public identity yet; save the editor first")?;
        ensure!(
            matches!(node.occurrence, Occurrence::Page { .. }),
            "public page is not associated with a native page"
        );
        Ok(node.entity.clone())
    };
    let pages = encode_pages(source, page_id)?;
    let contributors = contributors(source)?;
    let mut record = json!({"$type":"site.standard.document","title":source.title,"description":canonical_description(source.description.as_deref()),
        "tags":source.tags,"site":binding.publication.clone().unwrap_or_else(||format!("https://leaflet.pub/p/{}",uri.authority)),
        "path":source.extra.get("path").context("public path intent unavailable")?,
        "publishedAt":source.extra.get("publishedAt").context("publication date intent unavailable")?,
        "contributors":contributors.into_iter().map(|did|json!({"did":did})).collect::<Vec<_>>(),
        "content":{"$type":"pub.leaflet.content","pages":pages}});
    if record["description"].is_null() {
        record.as_object_mut().expect("record").remove("description");
    }
    from_record(&record, source)
}

fn supported_metadata(source: &Document) -> Result<()> {
    for name in ["cover", "coverImage", "delivery", "preferences"] {
        ensure!(!source.extra.contains_key(name), "release intent {name} has no supported projection");
    }
    Ok(())
}

fn contributors(source: &Document) -> Result<Vec<String>> {
    let contributors: Vec<String> = source
        .extra
        .get("contributors")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()?
        .unwrap_or_default();
    ensure!(
        contributors.len() <= 1,
        "multi-contributor publication order is unavailable from the unordered native pull; complete input projection refused"
    );
    Ok(contributors)
}

/// Pure full-body release validation BEFORE mint/save. Symbolic source page
/// labels validate reachability only; they are NEVER confirmed native IDs or a
/// receipt/hash. The executor must project the actual saved native pages again.
pub fn preflight(source: &Document) -> Result<usize> {
    supported_metadata(source)?;
    contributors(source)?;
    let pages = encode_pages(source, |key| Ok(key.to_owned()))?;
    // Validate every encoded block using the existing strict public decoder.
    let ids: BTreeMap<_, _> = source.pages.iter().map(|page| (page.key.clone(), page.key.clone())).collect();
    for page in &pages {
        for block in page["blocks"].as_array().expect("encoded blocks") {
            decode_block(&block["block"], &ids, &mut vec![])?;
        }
    }
    Ok(source
        .pages
        .iter()
        .filter(|page| {
            !source
                .state
                .nodes
                .get(&page.key)
                .is_some_and(|node| matches!(node.occurrence, Occurrence::Page { .. }))
        })
        .count())
}

fn encode_pages(source: &Document, page_id: impl Fn(&str) -> Result<String>) -> Result<Vec<Value>> {
    let mut reached = BTreeSet::new();
    let mut pending = vec![source.pages.first().context("missing working entry page")?.key.as_str()];
    while let Some(key) = pending.pop() {
        ensure!(
            reached.insert(key),
            "repeated/cyclic page references need native publication traversal support; projection unavailable"
        );
        let page = source.pages.iter().find(|page| page.key == key).context("unresolved source page reference")?;
        pending.extend(page.blocks.iter().filter_map(|block| {
            if let BlockKind::PageLink { page } = &block.kind {
                Some(page.as_str())
            } else {
                None
            }
        }));
    }
    ensure!(reached.len() == source.pages.len(), "unreachable working page definitions cannot be projected");
    let mut pages = vec![];
    for page in &source.pages {
        let mut blocks: Vec<Value> = vec![];
        for block in &page.blocks {
            let rich = |kind: &str, content: &[Inline]| -> Result<Value> {
                let mut text = String::new();
                let mut facets = vec![];
                for item in content {
                    match item {
                        Inline::HardBreak => text.push('\n'),
                        Inline::Run { text: run, marks } => {
                            let start = text.len();
                            text.push_str(run);
                            let features: Vec<_> = marks.iter().map(feature).collect::<Result<_>>()?;
                            if !features.is_empty() && !run.is_empty() {
                                facets.push(json!({"index":{"byteStart":start,"byteEnd":text.len()},"features":features}));
                            }
                        }
                    }
                }
                let mut value = json!({"$type":format!("pub.leaflet.blocks.{kind}"),"plaintext":text});
                if !facets.is_empty() {
                    value["facets"] = json!(facets);
                }
                Ok(value)
            };
            let body = match &block.kind {
                BlockKind::Paragraph { content } => rich("text", content)?,
                BlockKind::Heading { level, content } => {
                    let mut value = rich("header", content)?;
                    value["level"] = json!(level);
                    value
                }
                BlockKind::Quote { content } => rich("blockquote", content)?,
                BlockKind::Rule => json!({"$type":"pub.leaflet.blocks.horizontalRule"}),
                BlockKind::Code { language, text } => json!({"$type":"pub.leaflet.blocks.code","language":language,"plaintext":text}),
                BlockKind::PageLink { page } => json!({"$type":"pub.leaflet.blocks.page","id":page_id(page)?}),
                BlockKind::Image { .. } => {
                    bail!("URL images do not establish public blob bytes/identity; release projection unavailable (no fetch/upload or lossy omission)")
                }
                BlockKind::ListItem { style, content } => {
                    let kind = match style {
                        ListStyle::Bullet => "unorderedList",
                        ListStyle::Ordered => "orderedList",
                    };
                    let child = json!({"$type":format!("pub.leaflet.blocks.{kind}#listItem"),"content":rich("text",content)?});
                    if let Some(last) = blocks.last_mut().filter(|last| last["block"]["$type"] == format!("pub.leaflet.blocks.{kind}")) {
                        last["block"]["children"].as_array_mut().expect("built list").push(child);
                        continue;
                    }
                    let mut value = json!({"$type":format!("pub.leaflet.blocks.{kind}"),"children":[child]});
                    if *style == ListStyle::Ordered {
                        value["startIndex"] = json!(1);
                    }
                    value
                }
            };
            blocks.push(json!({"$type":"pub.leaflet.pages.linearDocument#block","block":body}));
        }
        pages.push(json!({"$type":"pub.leaflet.pages.linearDocument","id":page_id(&page.key)?,"blocks":blocks}));
    }
    Ok(pages)
}

/// JSON UTF-8 upper bound for THIS transient supported view, preserving its
/// supplied inline segmentation. This alone is NOT evidence of native size:
/// callers must supply actual active loaded/cumulative Yrs segments, validated
/// facts and actual or bounded provisional UUID page IDs. Semantic runs alone
/// can underestimate facets. Native optional absent code language can make this
/// larger (we include language:""), never smaller. Key order is size-neutral.
pub fn inline_pages_json_upper_bound(source: &Document) -> Result<usize> {
    preflight(source)?;
    let pages = encode_pages(source, |key| {
        let node = source.state.nodes.get(key).context("size witness needs native/bounded page identity")?;
        ensure!(matches!(node.occurrence, Occurrence::Page { .. }), "size witness page occurrence mismatch");
        Ok(node.entity.clone())
    })?;
    // Supported JSON contains only strings, finite small integers, arrays and
    // ordinary maps. serde_json matches JSON.stringify escaping/UTF-8 lengths:
    // quotes, slash/backslash, C0 controls, BMP/non-BMP and U+2028/U+2029.
    Ok(serde_json::to_vec(&pages)?.len())
}

fn feature(mark: &Mark) -> Result<Value> {
    let kind = match mark {
        Mark::Strong => "bold",
        Mark::Em => "italic",
        Mark::Code => "code",
        Mark::Strikethrough => "strikethrough",
        Mark::Underline => "underline",
        Mark::Highlight { color } => {
            ensure!(color == "1", "colored highlight requires native theme-color resolution; projection unavailable");
            "highlight"
        }
        Mark::Link { href } => return Ok(json!({"$type":"pub.leaflet.richtext.facet#link","uri":href})),
    };
    Ok(json!({"$type":format!("pub.leaflet.richtext.facet#{kind}")}))
}

fn rich(value: &Value) -> Result<Vec<Inline>> {
    let plaintext = text(value, "plaintext")?;
    let mut ranges = vec![];
    let mut boundaries = BTreeSet::from([0, plaintext.len()]);
    if let Some(facets) = value.get("facets") {
        for facet in facets.as_array().context("invalid public facets")? {
            fields(facet, &["$type", "index", "features"])?;
            if let Some(kind) = facet.get("$type") {
                ensure!(kind == "pub.leaflet.richtext.facet", "unsupported facet discriminator");
            }
            fields(&facet["index"], &["byteStart", "byteEnd"])?;
            let start = usize::try_from(facet["index"]["byteStart"].as_u64().context("invalid facet start")?)?;
            let end = usize::try_from(facet["index"]["byteEnd"].as_u64().context("invalid facet end")?)?;
            ensure!(
                start < end && end <= plaintext.len() && plaintext.is_char_boundary(start) && plaintext.is_char_boundary(end),
                "facet is outside UTF-8 text boundaries"
            );
            let mut marks = vec![];
            for feature in facet["features"].as_array().context("invalid facet features")? {
                let kind = text(feature, "$type")?;
                let mark = match kind.as_str() {
                    "pub.leaflet.richtext.facet#bold" => Mark::Strong,
                    "pub.leaflet.richtext.facet#italic" => Mark::Em,
                    "pub.leaflet.richtext.facet#code" => Mark::Code,
                    "pub.leaflet.richtext.facet#underline" => Mark::Underline,
                    "pub.leaflet.richtext.facet#strikethrough" => Mark::Strikethrough,
                    "pub.leaflet.richtext.facet#highlight" => Mark::Highlight { color: "1".into() },
                    "pub.leaflet.richtext.facet#link" => Mark::Link { href: text(feature, "uri")? },
                    _ => bail!("unsupported public rich-text feature {kind}"),
                };
                fields(feature, if matches!(mark, Mark::Link { .. }) { &["$type", "uri"] } else { &["$type"] })?;
                marks.push(mark);
            }
            ensure!(!marks.is_empty(), "empty facet has no supported meaning");
            ranges.push((start, end, marks));
            boundaries.extend([start, end]);
        }
    }
    let boundaries: Vec<_> = boundaries.into_iter().collect();
    let mut runs = vec![];
    for pair in boundaries.windows(2) {
        let marks: Vec<_> = ranges
            .iter()
            .filter(|(start, end, _)| *start <= pair[0] && *end >= pair[1])
            .flat_map(|(_, _, marks)| marks.iter().cloned())
            .collect();
        push_run(&mut runs, &plaintext[pair[0]..pair[1]], &marks);
    }
    Ok(runs)
}

/// Strict *mapping* coverage after generated lexicon validation. Unknown or
/// richer fields never disappear just because codegen tolerates open unions.
pub fn from_record(record: &Value, template: &Document) -> Result<Projection> {
    crate::record::validate_document_record(record)?;
    ensure!(record["$type"] == "site.standard.document", "unsupported public record type");
    fields(
        record,
        &[
            "$type",
            "title",
            "description",
            "tags",
            "site",
            "path",
            "publishedAt",
            "contributors",
            "content",
            "theme",
        ],
    )?;
    let site = text(record, "site")?;
    // Pinned extractThemeFromFacts always includes $type, so the publisher's
    // Object.keys(extracted).length > 1 retains even this exact default theme.
    // No arbitrary theme, page color, image or nondefault property is ignored.
    if let Some(theme) = record.get("theme") {
        ensure!(
            theme == &json!({"$type":"pub.leaflet.publication#theme","showPageBackground":true}),
            "nondefault public theme requires a supported mapping"
        );
    }
    let path = text(record, "path")?;
    ensure!(path.starts_with('/'), "invalid public path");
    let published_at = date(&record["publishedAt"])?;
    let tags: Vec<String> = record.get("tags").map(|v| serde_json::from_value(v.clone())).transpose()?.unwrap_or_default();
    let description = record.get("description").map(|v| v.as_str().context("invalid description")).transpose()?;
    let mut contributors = vec![];
    if let Some(value) = record.get("contributors") {
        for contributor in value.as_array().context("invalid contributors")? {
            fields(contributor, &["$type", "did"])?;
            if let Some(kind) = contributor.get("$type") {
                ensure!(kind == "site.standard.document#contributor", "unsupported contributor discriminator");
            }
            contributors.push(text(contributor, "did")?);
        }
    }
    fields(&record["content"], &["$type", "pages"])?;
    ensure!(
        record["content"]["$type"] == "pub.leaflet.content",
        "offloaded/unknown public content is not an empty body"
    );
    let pages = record["content"]["pages"].as_array().context("public pages unavailable")?;
    ensure!(!pages.is_empty(), "public document has no entry page");
    let mut ids = BTreeMap::new();
    for (index, page) in pages.iter().enumerate() {
        let id = text(page, "id")?;
        ensure!(ids.insert(id, format!("p-{index}")).is_none(), "duplicate public page identity");
    }
    let mut document = template.clone();
    document.title = text(record, "title")?;
    ensure!(
        !document.title.trim().is_empty(),
        "empty public title cannot be represented faithfully by this source profile"
    );
    document.description = canonical_description(description).map(str::to_owned);
    document.tags = tags;
    document.theme = None; // native publication ignores editor watermark, not a theme wildcard
    document.extra = BTreeMap::from([
        ("contributors".into(), json!(contributors)),
        ("path".into(), json!(path)),
        ("publishedAt".into(), json!(published_at)),
    ]);
    document.publication = if site.starts_with("at://") {
        Publication::At(site.clone())
    } else {
        Publication::Standalone
    };
    document.state.nodes.clear();
    document.pages.clear();
    for (page_index, page) in pages.iter().enumerate() {
        fields(page, &["$type", "id", "blocks"])?;
        ensure!(page["$type"] == "pub.leaflet.pages.linearDocument", "unsupported public page type/theme");
        let id = text(page, "id")?;
        let key = ids[&id].clone();
        document.state.nodes.insert(
            key.clone(),
            Node {
                entity: id,
                occurrence: Occurrence::Page { references: vec![] },
            },
        );
        let mut blocks = vec![];
        for wrapper in page["blocks"].as_array().context("invalid public page blocks")? {
            fields(wrapper, &["$type", "block"])?;
            ensure!(wrapper["$type"] == "pub.leaflet.pages.linearDocument#block", "unsupported block wrapper");
            decode_block(&wrapper["block"], &ids, &mut blocks)?;
        }
        for (index, block) in blocks.iter_mut().enumerate() {
            block.key = format!("b-{page_index}-{index}");
        }
        document.pages.push(Page { key, blocks });
    }
    document.state.main_page = Some(document.pages[0].key.clone());
    let metadata = json!({"title":document.title,"description":document.description,"tags":document.tags,"site":site,"path":path,"publishedAt":published_at,"contributors":contributors});
    let mut projected_pages: Vec<_> = document
        .pages
        .iter()
        .map(|page| {
            let blocks: Vec<_> = page
                .blocks
                .iter()
                .map(|block| {
                    let mut kind = serde_json::to_value(&block.kind).expect("block JSON");
                    if let BlockKind::PageLink { page } = &block.kind {
                        kind["page"] = json!(document.state.nodes[page].entity);
                    }
                    kind
                })
                .collect();
            json!({"id":document.state.nodes[&page.key].entity,"blocks":blocks})
        })
        .collect();
    // Native emits repeated subpages for repeated cards. This bounded profile
    // rejects duplicates above; definition layout/order is not public meaning.
    projected_pages[1..].sort_by_key(|page| page["id"].to_string());
    Ok(Projection {
        value: json!({"profile":RELEASE_PROFILE,"metadata":metadata,"pages":projected_pages}),
        document,
        metadata,
    })
}

fn decode_block(value: &Value, ids: &BTreeMap<String, String>, out: &mut Vec<Block>) -> Result<()> {
    let kind = text(value, "$type")?;
    let block = match kind.as_str() {
        "pub.leaflet.blocks.text" | "pub.leaflet.blocks.header" | "pub.leaflet.blocks.blockquote" => {
            fields(
                value,
                if kind.ends_with("header") {
                    &["$type", "plaintext", "facets", "level"]
                } else {
                    &["$type", "plaintext", "facets"]
                },
            )?;
            let content = rich(value)?;
            match kind.as_str() {
                "pub.leaflet.blocks.header" => {
                    let level = value["level"].as_u64().context("invalid header level")?;
                    ensure!((1..=6).contains(&level), "unsupported header level");
                    BlockKind::Heading { level: level as u8, content }
                }
                "pub.leaflet.blocks.blockquote" => BlockKind::Quote { content },
                _ => BlockKind::Paragraph { content },
            }
        }
        "pub.leaflet.blocks.horizontalRule" => {
            fields(value, &["$type"])?;
            BlockKind::Rule
        }
        "pub.leaflet.blocks.code" => {
            fields(value, &["$type", "language", "plaintext"])?;
            BlockKind::Code {
                language: value
                    .get("language")
                    .map(|v| v.as_str().context("invalid code language"))
                    .transpose()?
                    .unwrap_or_default()
                    .into(),
                text: text(value, "plaintext")?,
            }
        }
        "pub.leaflet.blocks.page" => {
            fields(value, &["$type", "id"])?;
            BlockKind::PageLink {
                page: ids.get(&text(value, "id")?).context("public page reference is unresolved")?.clone(),
            }
        }
        "pub.leaflet.blocks.orderedList" | "pub.leaflet.blocks.unorderedList" => {
            let ordered = kind.ends_with(".orderedList");
            ensure!(
                !out.last()
                    .is_some_and(|block| matches!(&block.kind,BlockKind::ListItem{style,..} if (*style==ListStyle::Ordered)==ordered)),
                "adjacent separate same-style lists have an unsupported restart/boundary meaning"
            );
            fields(
                value,
                if ordered {
                    &["$type", "children", "startIndex"]
                } else {
                    &["$type", "children"]
                },
            )?;
            ensure!(
                !ordered || value.get("startIndex").is_none_or(|start| start == 1),
                "ordered list start override unsupported"
            );
            let children = value["children"].as_array().context("invalid list children")?;
            ensure!(!children.is_empty(), "empty public list has no faithful flat source representation");
            for child in children {
                fields(child, &["$type", "content"])?;
                ensure!(child["$type"] == format!("{kind}#listItem"), "unsupported list item");
                fields(&child["content"], &["$type", "plaintext", "facets"])?;
                ensure!(child["content"]["$type"] == "pub.leaflet.blocks.text", "non-text list content unsupported");
                out.push(Block {
                    key: format!("b-{}", out.len()),
                    kind: BlockKind::ListItem {
                        style: if ordered { ListStyle::Ordered } else { ListStyle::Bullet },
                        content: rich(&child["content"])?,
                    },
                });
            }
            return Ok(());
        }
        _ => bail!("unsupported public block {kind}; no partial comparison produced"),
    };
    out.push(Block {
        key: format!("b-{}", out.len()),
        kind: block,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn template() -> Document {
        serde_json::from_value(json!({"format":"leafbud-mdx/2","title":"T","@leafbud":{"id":"11111111-1111-4111-8111-111111111111","state":"published","binding":{"uri":"at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/post","baseUrl":"https://leaflet.pub","token":"22222222-2222-4222-8222-222222222222","root":"r","entitySet":"s","entryFact":"e","publication":null}}})).unwrap()
    }
    fn record() -> Value {
        json!({"$type":"site.standard.document","site":"https://leaflet.pub/p/did:plc:abcdefghijklmnopqrstuvwx","title":"Title","path":"/post","publishedAt":"2026-10-01T10:00:00.000Z","content":{"$type":"pub.leaflet.content","pages":[{"$type":"pub.leaflet.pages.linearDocument","id":"native-page","blocks":[{"$type":"pub.leaflet.pages.linearDocument#block","block":{"$type":"pub.leaflet.blocks.text","plaintext":"Hi 🦋 café","facets":[{"index":{"byteStart":3,"byteEnd":7},"features":[{"$type":"pub.leaflet.richtext.facet#bold"},{"$type":"pub.leaflet.richtext.facet#link","uri":"https://example.com"}]}]}}]}]}})
    }
    #[test]
    fn pinned_standalone_publisher_default_theme_is_exact_not_an_arbitrary_wildcard() {
        let plain = from_record(&record(), &template()).unwrap();
        let mut native = record();
        native["theme"] = json!({"$type":"pub.leaflet.publication#theme","showPageBackground":true});
        assert_eq!(from_record(&native, &template()).unwrap().value, plain.value);
        native["theme"]["showPageBackground"] = json!(false);
        assert!(from_record(&native, &template()).is_err());
        native["theme"]["showPageBackground"] = json!(true);
        native["theme"]["headingFont"] = json!("serif");
        assert!(from_record(&native, &template()).is_err());
    }
    #[test]
    fn explicit_utf8_facets_date_defaults_and_spelling_free_public_meaning() {
        let projected = from_record(&record(), &template()).unwrap();
        assert_eq!(projected.metadata["publishedAt"], "2026-10-01T10:00:00Z");
        assert_eq!(projected.metadata["tags"], json!([]));
        assert_eq!(projected.metadata["contributors"], json!([]));
        assert_eq!(
            projected.document.pages[0].blocks[0].kind,
            BlockKind::Paragraph {
                content: vec![
                    Inline::Run {
                        text: "Hi ".into(),
                        marks: vec![]
                    },
                    Inline::Run {
                        text: "🦋".into(),
                        marks: vec![
                            Mark::Strong,
                            Mark::Link {
                                href: "https://example.com".into()
                            }
                        ]
                    },
                    Inline::Run {
                        text: " café".into(),
                        marks: vec![]
                    }
                ]
            }
        );
        let working = working(&projected.document).unwrap();
        assert_eq!(working.value, projected.value);
        let mut changed = projected.document.clone();
        changed.extra.insert("customAuthor".into(), json!({"retained":"not applied"}));
        changed.state.names = Some(leafbud_document::Names {
            file: "other.mdx".into(),
            path: leafbud_document::NamePath::Missing,
        });
        changed.state.id = uuid::Uuid::new_v4();
        let mut page = changed.pages.remove(0);
        let old = page.key.clone();
        page.key = "friendly".into();
        page.blocks[0].key = "friendly-block".into();
        let node = changed.state.nodes.remove(&old).unwrap();
        changed.state.nodes.insert(page.key.clone(), node);
        changed.pages.push(page);
        changed.state.main_page = Some("friendly".into());
        assert_eq!(
            working.value,
            super::working(&changed).unwrap().value,
            "source labels/names/custom metadata do not change public meaning"
        );
        changed.extra.insert("publishedAt".into(), json!("2026-10-01T12:00:00+02:00"));
        assert_eq!(working.value, super::working(&changed).unwrap().value);
        changed.title = "Edited".into();
        assert_ne!(working.value, super::working(&changed).unwrap().value);
    }
    #[test]
    fn public_unknown_richer_resources_and_bad_facet_boundaries_are_not_stripped() {
        for case in 0..7 {
            let mut record = record();
            let block = &mut record["content"]["pages"][0]["blocks"][0]["block"];
            match case {
                0 => record["futureField"] = json!(true),
                1 => record["content"]["pages"][0]["theme"] = json!({"$type":"pub.leaflet.theme.page"}),
                2 => block["facets"][0]["index"]["byteEnd"] = json!(4),
                3 => block["facets"][0]["features"][0]["$type"] = json!("pub.leaflet.richtext.facet#footnote"),
                4 => block["textSize"] = json!("large"),
                5 => record["preferences"] = json!({"$type":"pub.leaflet.publication#preferences","showInDiscover":false}),
                _ => record["content"] = json!({"$type":"pub.leaflet.content","pages":[],"pagesBlob":{"$type":"blob"}}),
            }
            assert!(from_record(&record, &template()).is_err(), "case={case}");
        }
        let mut doc = from_record(&record(), &template()).unwrap().document;
        doc.pages[0].blocks[0].kind = BlockKind::Image {
            src: "https://example.com/image.png".into(),
            fallback: None,
            width: None,
            height: None,
            alt: None,
        };
        assert!(working(&doc).is_err());
    }

    #[test]
    fn native_public_scalar_lists_and_unique_page_links_have_explicit_meaning() {
        let wrap = |block: Value| json!({"$type":"pub.leaflet.pages.linearDocument#block","block":block});
        let mut record = record();
        record["content"]["pages"] = json!([
            {"$type":"pub.leaflet.pages.linearDocument","id":"main-id","blocks":[
                wrap(json!({"$type":"pub.leaflet.blocks.header","level":2,"plaintext":"A heading"})),
                wrap(json!({"$type":"pub.leaflet.blocks.blockquote","plaintext":"Quoted\nline"})),
                wrap(json!({"$type":"pub.leaflet.blocks.code","language":"rust","plaintext":"let x = 1;\n"})),
                wrap(json!({"$type":"pub.leaflet.blocks.horizontalRule"})),
                wrap(json!({"$type":"pub.leaflet.blocks.unorderedList","children":[{"$type":"pub.leaflet.blocks.unorderedList#listItem","content":{"$type":"pub.leaflet.blocks.text","plaintext":"A"}},{"$type":"pub.leaflet.blocks.unorderedList#listItem","content":{"$type":"pub.leaflet.blocks.text","plaintext":"B"}}]})),
                wrap(json!({"$type":"pub.leaflet.blocks.orderedList","startIndex":1,"children":[{"$type":"pub.leaflet.blocks.orderedList#listItem","content":{"$type":"pub.leaflet.blocks.text","plaintext":"C"}}]})),
                wrap(json!({"$type":"pub.leaflet.blocks.page","id":"child-id"}))
            ]},
            {"$type":"pub.leaflet.pages.linearDocument","id":"child-id","blocks":[wrap(json!({"$type":"pub.leaflet.blocks.text","plaintext":""}))]}
        ]);
        let projection = from_record(&record, &template()).unwrap();
        let blocks = &projection.document.pages[0].blocks;
        assert_eq!(blocks.len(), 8);
        assert!(matches!(blocks[0].kind, BlockKind::Heading { level: 2, .. }));
        assert_eq!(
            blocks[2].kind,
            BlockKind::Code {
                language: "rust".into(),
                text: "let x = 1;\n".into()
            }
        );
        assert_eq!(blocks[3].kind, BlockKind::Rule);
        assert!(matches!(blocks[4].kind, BlockKind::ListItem { style: ListStyle::Bullet, .. }));
        assert!(matches!(blocks[6].kind, BlockKind::ListItem { style: ListStyle::Ordered, .. }));
        assert_eq!(blocks[7].kind, BlockKind::PageLink { page: "p-1".into() });
        assert_eq!(projection.value["pages"][0]["blocks"][7]["page"], "child-id");
        assert_eq!(projection.value, working(&projection.document).unwrap().value);
        let mut repeated = projection.document.clone();
        repeated.pages[0].blocks.push(blocks[7].clone());
        assert!(
            working(&repeated).is_err(),
            "native repeats subpages for repeated links; do not invent a deduplicated public output"
        );
        record["content"]["pages"][0]["blocks"][5]["block"]["startIndex"] = json!(2);
        assert!(
            from_record(&record, &template()).is_err(),
            "flat source numbering cannot erase public list-start intent"
        );
    }
}
