use super::*;
use crate::inventory;
use sha2::{Digest, Sha256};

pub(super) struct Selection {
    root: PathBuf,
    base: String,
    all: bool,
    anchors: Vec<(PathBuf, Option<Uuid>)>,
}

#[derive(Clone)]
pub(super) struct Selected {
    pub path: PathBuf,
    pub id: Option<Uuid>,
    pub identity: Option<refresh::Identity>,
    pub root: Option<Uuid>,
    pub fingerprint: Vec<u8>,
    pub problem: Option<(refresh::Outcome, String)>,
}

impl Selection {
    pub fn new(root: PathBuf, base: &str, request: Request) -> Result<Self> {
        let mut anchors = vec![];
        for file in request.files {
            let path = if file.is_absolute() { file } else { root.join(file) };
            // Missing explicit operands remain loud and can appear later.
            let path = std::fs::canonicalize(&path).unwrap_or(path);
            let id = std::fs::read_to_string(&path).ok().and_then(|s| crate::lock::source_id(&s).ok());
            if !anchors.iter().any(|(p, _)| *p == path) {
                anchors.push((path, id));
            }
        }
        Ok(Self {
            root,
            base: base.into(),
            all: request.all,
            anchors,
        })
    }

    pub fn scan(&self) -> Result<(Vec<Selected>, Vec<Uuid>)> {
        let mut inventory = inventory::scan(&self.root, Some(&self.base))?;
        inventory.include_operands(&self.anchors.iter().map(|a| a.0.clone()).collect::<Vec<_>>());
        let mut paths = if self.all { inventory.paths.clone() } else { vec![] };
        if !self.all {
            for (original, id) in &self.anchors {
                if let Some(matches) = id.and_then(|id| inventory.ids.get(&id)) {
                    paths.extend(matches.iter().cloned());
                } else {
                    paths.push(original.clone());
                }
            }
        }
        paths.sort();
        paths.dedup();
        let mut selected = vec![];
        let mut roots = std::collections::BTreeSet::new();
        for path in paths {
            let mut item = Selected {
                path: path.clone(),
                id: None,
                identity: None,
                root: None,
                fingerprint: vec![],
                problem: None,
            };
            let result = (|| -> Result<()> {
                let bytes = std::fs::read(&path).context("local source missing or unreadable; not recreated")?;
                item.fingerprint = Sha256::digest(&bytes).to_vec();
                let source = String::from_utf8(bytes).context("local source is not UTF-8")?;
                let header = leafbud_mdx::header(&source).map_err(|_| anyhow::anyhow!("invalid managed frontmatter; preserved"))?;
                item.id = crate::lock::source_id(&source).ok();
                if !self.all && self.anchors.iter().any(|(p, id)| p == &path && id.is_some() && id != &item.id) {
                    item.problem = Some((
                        refresh::Outcome::SourceLost,
                        "original selected identity no longer exists at this path; replacement not selected".into(),
                    ));
                    return Ok(());
                }
                if inventory.duplicates.contains(&path) {
                    item.problem = Some((
                        refresh::Outcome::Ambiguous,
                        "duplicate local UUID or remote binding; all copies preserved".into(),
                    ));
                    return Ok(());
                }
                // A readable binding can keep its lightweight invalidation
                // subscription while the body is temporarily invalid/pending.
                // It still authorizes no HTTP adoption or receipt change.
                if let (Some(id), Ok(binding)) = (
                    item.id,
                    serde_json::from_value::<leafbud_document::Binding>(header["@leafbud"]["binding"].clone()),
                ) && binding.base_url == self.base
                {
                    let root: Uuid = binding.root.parse().context("native root is not a UUID; unsupported subscription")?;
                    ensure!(!root.is_nil(), "native root is nil; unsupported subscription");
                    item.root = Some(root);
                    item.identity = Some(refresh::Identity { id, binding });
                }
                if header["@leafbud"].get("pending").is_some_and(|v| !v.is_null()) {
                    item.problem = Some((refresh::Outcome::Pending, "unresolved pending state; preserved for explicit recovery".into()));
                    return Ok(());
                }
                let parsed =
                    leafbud_mdx::parse_with_spans(&source).map_err(|_| anyhow::anyhow!("unsupported/invalid local source or owned state; preserved"))?;
                let Some(binding) = parsed.document.state.binding.clone() else {
                    item.problem = Some((refresh::Outcome::Unbound, "unbound local draft; watch does not establish bindings".into()));
                    return Ok(());
                };
                ensure!(binding.base_url == self.base, "binding belongs to another origin; not selected");
                let root: Uuid = binding.root.parse().context("native root is not a UUID; unsupported subscription")?;
                ensure!(!root.is_nil(), "native root is nil; unsupported subscription");
                item.root = Some(root);
                item.identity = Some(refresh::Identity {
                    id: parsed.document.state.id,
                    binding,
                });
                if parsed.main_page_provisional() || !parsed.unkeyed().is_empty() {
                    // Keep root subscription even while authored content is dirty.
                    item.problem = Some((
                        refresh::Outcome::LocalDirty,
                        "unprepared authored content; watch never prepares keys or pushes".into(),
                    ));
                }
                Ok(())
            })();
            if let Err(error) = result {
                item.problem = Some((
                    if path.exists() {
                        refresh::Outcome::Invalid
                    } else {
                        refresh::Outcome::SourceLost
                    },
                    format!("{error:#}"),
                ));
            }
            if let Some(root) = item.root {
                roots.insert(root);
            }
            selected.push(item);
        }
        Ok((selected, roots.into_iter().collect()))
    }
}
