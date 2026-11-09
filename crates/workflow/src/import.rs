//! Read native drafts into one current keyed MDX file per remote binding.
//! No remote mutations and no raw-JSON siblings/snapshots. Reimport is guarded:
//! equal observations are no-ops, differing files are preserved for explicit pull.
use crate::{RemoteReader, config};
use anyhow::{Context, Result, bail, ensure};
use leafbud_document::{Binding, Document, EDITOR_PROFILE, FORMAT, Node, Publication, State, Sync};
use leafbud_web::{LeafletData, LeafletState, PullSnapshot};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};
use uuid::Uuid;

#[derive(Default)]
pub struct Request {
    pub all: bool,
    pub locator: Option<String>,
    pub state: Option<LeafletState>,
    pub to: Option<PathBuf>,
}

#[derive(Serialize)]
pub struct Report {
    pub version: u8,
    pub workspace: PathBuf,
    pub coverage: &'static str,
    pub discovered: usize,
    pub selected: usize,
    pub imported: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub skipped: usize,
    pub items: Vec<Item>,
}
#[derive(Serialize)]
pub struct Item {
    pub token: Uuid,
    pub outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub(crate) fn hash(value: &Value) -> String {
    fn canonical(v: &Value) -> Value {
        match v {
            Value::Object(o) => Value::Object(
                o.iter()
                    .map(|(k, v)| (k.clone(), canonical(v)))
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .collect(),
            ),
            Value::Array(a) => Value::Array(a.iter().map(canonical).collect()),
            _ => v.clone(),
        }
    }
    format!("sha256:{:x}", Sha256::digest(serde_json::to_vec(&canonical(value)).expect("JSON value")))
}

pub(crate) fn association(data: &LeafletData) -> Result<Option<String>> {
    let mut associations = BTreeSet::new();
    for row in data.association_rows() {
        for link in row["leaflets_in_publications"].as_array().into_iter().flatten() {
            associations.insert(link["publication"].as_str().context("publication association has no URI")?.to_owned());
        }
    }
    ensure!(associations.len() <= 1, "multiple publication associations need explicit resolution");
    Ok(associations.into_iter().next())
}

pub(crate) fn remote_hash(data: &LeafletData, pull: &PullSnapshot) -> String {
    hash(&leafbud_editor::witness(
        &pull.facts,
        &pull.extras,
        json!({"root":data.metadata.root_entity,"title":data.metadata.title,"description":data.metadata.description,
            "associations":data.association_rows().iter().map(|r|json!({"publication":r["leaflets_in_publications"],"standalone":r["leaflets_to_documents"]})).collect::<Vec<_>>()}),
    ))
}

pub(crate) fn build(base: &str, data: &LeafletData, pull: &PullSnapshot) -> Result<Document> {
    ensure!(data.observed_state() == LeafletState::Draft, "published object is outside draft import profile");
    let root = data.metadata.root_entity.as_deref().context("missing native root entity")?;
    let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("{base}/{}", data.metadata.id).as_bytes());
    let content = leafbud_editor::observe_view(pull, root, id)?;
    let entity_sets: BTreeSet<_> = data
        .metadata
        .permission_token_rights
        .iter()
        .filter(|r| r.read == Some(true))
        .filter_map(|r| r.entity_set.as_ref())
        .collect();
    ensure!(entity_sets.len() == 1, "expected one readable native entity set");
    let publication = association(data)?;
    let mut extra = BTreeMap::new();
    let string = |key: &str| -> Result<Option<String>> {
        match pull.extras.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            _ => bail!("invalid {key} metadata"),
        }
    };
    let tags = match pull.extras.get("publication_tags") {
        None => vec![],
        Some(v) => serde_json::from_value(v.clone()).context("invalid publication tags")?,
    };
    for (key, value) in &pull.extras {
        match key.as_str() {
            "initialized" | "publication_title" | "publication_description" | "publication_tags" => {}
            "draft_contributors" => {
                let contributors: Vec<String> = serde_json::from_value(value.clone()).context("invalid draft contributors")?;
                if !contributors.is_empty() {
                    extra.insert("contributors".into(), json!(contributors));
                }
            }
            "post_preferences" => ensure!(
                value.is_null() || value == &json!({}),
                "post preferences require a supported author metadata mapping"
            ),
            _ => bail!("unsupported editor extra: {key}"),
        }
    }
    // Remote author fields, not ambient publication defaults. No draft date/path
    // or public record URI is invented merely to write a local source.
    let title = string("publication_title")?
        .filter(|s| !s.trim().is_empty())
        .or_else(|| data.metadata.title.clone().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "Untitled".into());
    let publication_uri = publication.clone();
    let publication = match publication {
        None => Publication::Standalone,
        Some(uri) => Publication::At(uri),
    };
    let nodes: BTreeMap<String, Node> = content.nodes;
    let main_page = content.pages.first().map(|page| page.key.clone()).context("entry page missing")?;
    let mut document = Document {
        format: FORMAT.into(),
        title,
        description: string("publication_description")?.or_else(|| data.metadata.description.clone()),
        tags,
        publication: publication.clone(),
        theme: content.theme,
        extra,
        pages: content.pages,
        state: State {
            writer: None,
            pending: None,
            id,
            state: "draft".into(),
            main_page: Some(main_page),
            binding: Some(Binding {
                base_url: base.into(),
                token: data.metadata.id,
                root: root.into(),
                entity_set: (*entity_sets.first().expect("validated entity set")).clone(),
                entry_fact: content.entry_fact,
                publication: publication_uri,
            }),
            nodes,
            sync: Some(Sync {
                profile: EDITOR_PROFILE.into(),
                local_hash: String::new(),
                remote_hash: remote_hash(data, pull),
            }),
        },
    };
    document.state.sync.as_mut().expect("just built").local_hash = hash(&document.editor_value());
    Ok(document)
}

fn local_index(root: &Path, base: &str) -> Result<BTreeMap<Uuid, Vec<PathBuf>>> {
    fn visit(path: &Path, base: &str, index: &mut BTreeMap<Uuid, Vec<PathBuf>>) -> Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_dir() {
                visit(&path, base, index)?;
            } else if kind.is_file() && path.extension().is_some_and(|e| e == "mdx") {
                let source = std::fs::read_to_string(&path)?;
                let header = match leafbud_mdx::header(&source) {
                    Ok(header) => header,
                    Err(_) if !source.contains("@leafbud") && !source.contains("@springleaf") => continue,
                    Err(error) => return Err(error).with_context(|| format!("cannot safely index managed file {}", path.display())),
                };
                ensure!(
                    header.get("@springleaf").is_none(),
                    "legacy springleaf managed source requires explicit namespace conversion before import (duplicate checkout refused)"
                );
                let binding = &header["@leafbud"]["binding"];
                if let Some((origin, token)) = crate::inventory::native_locator(&header["@leafbud"]) {
                    if origin == base {
                        index.entry(token).or_default().push(path);
                    }
                } else if binding.is_object() && binding["baseUrl"] == base {
                    bail!("managed file token missing/invalid");
                }
            }
        }
        Ok(())
    }
    let mut index = BTreeMap::new();
    visit(root, base, &mut index)?;
    Ok(index)
}

#[cfg(test)]
pub(crate) mod tests;

/// Offline validation outcome for one managed file.
pub struct Check {
    pub blocks: usize,
    /// Blocks without a durable source key; tidy --keys (or executed push)
    /// assigns them. Check itself never writes.
    pub needs_preparation: usize,
    /// None when no receipt exists (unbound/never-settled documents).
    pub local_change: Option<bool>,
}

pub fn check(path: &Path) -> Result<Check> {
    let source = std::fs::read_to_string(path).context("reading MDX source")?;
    let parsed = leafbud_mdx::parse_with_spans(&source)?;
    let blocks = parsed.document.pages.iter().map(|p| p.blocks.len()).sum();
    let needs_preparation = parsed.unkeyed().len() + usize::from(parsed.main_page_provisional());
    let local_change = match &parsed.document.state.sync {
        Some(sync) => Some(hash(&parsed.document.editor_value()) != sync.local_hash.as_str()),
        None => None,
    };
    Ok(Check {
        blocks,
        needs_preparation,
        local_change,
    })
}

fn destination(root: &Path, title: &str, id: Uuid, to: Option<&Path>) -> Result<PathBuf> {
    let path = match to {
        Some(to) if to.is_absolute() => to.to_owned(),
        Some(to) => root.join(to),
        None => {
            let mut slug = String::new();
            for c in title.chars() {
                if c.is_ascii_alphanumeric() {
                    slug.push(c.to_ascii_lowercase());
                } else if !slug.ends_with('-') {
                    slug.push('-');
                }
                if slug.len() >= 70 {
                    break;
                }
            }
            let slug = slug.trim_matches('-');
            root.join("drafts").join(format!("{}-{id}.mdx", if slug.is_empty() { "draft" } else { slug }))
        }
    };
    ensure!(path.extension().is_some_and(|e| e == "mdx"), "import destination must end in .mdx");
    let relative = path.strip_prefix(root).context("destination must be inside document workspace")?;
    ensure!(relative.components().all(|c| matches!(c, Component::Normal(_))), "unsafe destination path");
    let mut cursor = root.to_path_buf();
    for part in relative.components() {
        cursor.push(part);
        if let Ok(metadata) = std::fs::symlink_metadata(&cursor) {
            ensure!(!metadata.file_type().is_symlink(), "destination contains a symlink");
        }
    }
    Ok(path)
}

/// Initial filename/display convenience only. Never substitutes a first heading
/// for persisted author-title metadata in a bound observation or save read-back.
fn display_title(document: &Document) -> String {
    if document.title != "Untitled" {
        return document.title.clone();
    }
    document
        .pages
        .first()
        .and_then(|p| {
            p.blocks.iter().find_map(|b| match &b.kind {
                leafbud_document::BlockKind::Heading { content, .. } | leafbud_document::BlockKind::Paragraph { content } => {
                    let text: String = content
                        .iter()
                        .map(|i| match i {
                            leafbud_document::Inline::Run { text, .. } => text.as_str(),
                            leafbud_document::Inline::HardBreak => "\n",
                        })
                        .collect();
                    (!text.trim().is_empty()).then_some(text)
                }
                _ => None,
            })
        })
        .unwrap_or_else(|| document.title.clone())
}

impl RemoteReader {
    pub async fn import(&self, context: &config::Context, request: Request) -> Result<Report> {
        ensure!(request.all ^ request.locator.is_some(), "select one locator or --all");
        ensure!(!request.all || request.to.is_none(), "--to is only valid for one locator");
        let root = std::fs::canonicalize(&context.workspace_root).context("resolving document workspace")?;
        let base = self.client.base_url();
        local_index(&root, base)?;
        let ids = if request.all {
            self.client.list().await?.leaflets.into_iter().map(|l| l.metadata.id).collect::<Vec<_>>()
        } else {
            vec![self.client.resolve_locator(request.locator.as_deref().expect("validated locator"))?]
        };
        let mut report = Report {
            version: 1,
            workspace: root.clone(),
            coverage: if request.all {
                "Observed /home+contributor inventory; source query has no explicit pagination/limit; backend/account-wide completeness unverified."
            } else {
                "Explicit locator only"
            },
            discovered: ids.len(),
            selected: 0,
            imported: 0,
            unchanged: 0,
            failed: 0,
            skipped: 0,
            items: vec![],
        };
        for id in ids {
            let result: Result<Option<(PathBuf, &'static str)>> = async {
                let store = crate::lock::Store::new(context)?;
                let _binding_guard = store.binding(base, id)?.context("binding busy in another Leafbud import; retry later")?;
                // A concurrent import may have established the binding while
                // this request was being selected. Index only after locking.
                let index = local_index(&root, base)?;
                let data = self.client.get(id).await?;
                let state = data.observed_state();
                if request.state.is_some_and(|s| s != state) {
                    return Ok(None);
                }
                report.selected += 1;
                ensure!(state == LeafletState::Draft, "published-object import is not implemented in this profile");
                let pull = self.client.pull(id).await?;
                let document = build(base, &data, &pull)?;
                if let Some(paths) = index.get(&id) {
                    ensure!(paths.len() == 1, "duplicate existing local bindings require resolution");
                    let path = &paths[0];
                    if let Some(to) = request.to.as_deref() {
                        ensure!(
                            destination(&root, &document.title, id, Some(to))? == *path,
                            "binding already exists at another local path"
                        );
                    }
                    let (_guard, _, source) = crate::lock::read(path, &store)?;
                    let existing = leafbud_mdx::parse(&source).context("existing document has unsupported/invalid local edits; preserved")?;
                    ensure!(
                        existing.state.sync.as_ref().is_some_and(|sync| sync.remote_hash == remote_hash(&data, &pull)),
                        "remote changed since import; explicit pull/refresh support required (existing file preserved)"
                    );
                    let value = existing.editor_value();
                    let outcome = if hash(&value) == existing.state.sync.as_ref().expect("checked above").local_hash {
                        "unchanged"
                    } else {
                        "kept-local-changes"
                    };
                    return Ok(Some((path.clone(), outcome)));
                }
                let path = destination(&root, &display_title(&document), id, request.to.as_deref())?;
                let _document_guard = store
                    .document(document.state.id)?
                    .context("document busy in another Leafbud operation; retry later")?;
                let source = leafbud_mdx::render(&document)?;
                // Recheck native identity/association after content acquisition; no
                // promise of a remote transaction/CAS, and no files overwritten.
                let verified = self.client.get(id).await?;
                let confirmed = match &document.publication {
                    Publication::Standalone => None,
                    Publication::At(uri) => Some(uri.clone()),
                    Publication::Inherit => unreachable!("imports always carry the confirmed association"),
                };
                ensure!(
                    verified.metadata.root_entity == data.metadata.root_entity && verified.observed_state() == state && association(&verified)? == confirmed,
                    "remote identity/lifecycle changed during import"
                );
                ensure!(
                    !local_index(&root, base)?.contains_key(&id),
                    "binding appeared during import; existing file preserved"
                );
                crate::files::write(&path, None, source.as_bytes())?;
                Ok(Some((path, "imported")))
            }
            .await;
            match result {
                Ok(None) => {
                    report.skipped += 1;
                    report.items.push(Item {
                        token: id,
                        outcome: "filtered",
                        path: None,
                        error: None,
                    });
                }
                Ok(Some((path, outcome))) => {
                    if outcome == "imported" {
                        report.imported += 1;
                    } else {
                        report.unchanged += 1;
                    }
                    report.items.push(Item {
                        token: id,
                        outcome,
                        path: Some(path),
                        error: None,
                    });
                }
                Err(error) => {
                    report.failed += 1;
                    report.items.push(Item {
                        token: id,
                        outcome: "failed",
                        path: None,
                        error: Some(format!("{error:#}")),
                    });
                }
            }
        }
        Ok(report)
    }
}
