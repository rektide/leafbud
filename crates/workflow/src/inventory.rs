//! Local selection and identity ambiguity, not the user-facing list command.
use anyhow::Result;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Default)]
pub(crate) struct Inventory {
    pub paths: Vec<PathBuf>,
    pub index: BTreeMap<(String, Uuid), Vec<PathBuf>>,
    pub states: BTreeMap<PathBuf, String>,
    pub duplicates: BTreeSet<PathBuf>,
    pub unmanaged: usize,
    ids: BTreeMap<Uuid, Vec<PathBuf>>,
}

pub(crate) fn scan(root: &Path, base: Option<&str>) -> Result<Inventory> {
    fn visit(path: &Path, base: Option<&str>, inventory: &mut Inventory) -> Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_dir() {
                visit(&path, base, inventory)?;
            } else if kind.is_file() && path.extension().is_some_and(|e| e == "mdx") {
                let source = match std::fs::read_to_string(&path) {
                    Ok(source) => source,
                    Err(_) => {
                        inventory.paths.push(path);
                        continue;
                    }
                };
                let header = match leafbud_mdx::header(&source) {
                    Ok(header) if header.get("@leafbud").is_some() || header.get("@springleaf").is_some() => header,
                    _ if !source.contains("@leafbud") && !source.contains("@springleaf") => {
                        inventory.unmanaged += 1;
                        continue;
                    }
                    _ => {
                        inventory.paths.push(path);
                        continue;
                    }
                };
                let state = &header["@leafbud"];
                let binding = &state["binding"];
                let origin = binding.get("baseUrl").and_then(Value::as_str);
                if let Some(id) = state["id"].as_str().and_then(|s| Uuid::parse_str(s).ok()) {
                    inventory.ids.entry(id).or_default().push(path.clone());
                }
                if base.is_some() && origin.is_some() && origin != base {
                    continue;
                }
                inventory.paths.push(path.clone());
                if let Some(local_state) = state["state"].as_str() {
                    inventory.states.insert(path.clone(), local_state.into());
                }
                // Only bound documents participate in binding identity;
                // unbound local drafts have no (origin, token) pair yet.
                if let (Some(origin), Some(token)) = (origin, binding.get("token").and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok())) {
                    inventory.index.entry((origin.to_owned(), token)).or_default().push(path.clone());
                }
            }
        }
        Ok(())
    }
    let mut inventory = Inventory::default();
    visit(root, base, &mut inventory)?;
    inventory.find_duplicates();
    inventory.paths.sort();
    Ok(inventory)
}

impl Inventory {
    /// Explicit operands may be outside the workspace. Account for their
    /// identities against each other and the workspace without following any
    /// new directory tree or inventing a managed-file lifecycle.
    pub fn include_operands(&mut self, paths: &[PathBuf]) {
        for path in paths.iter().filter(|p| !self.paths.contains(p)) {
            let Some(header) = std::fs::read_to_string(path).ok().and_then(|s| leafbud_mdx::header(&s).ok()) else {
                continue;
            };
            let state = &header["@leafbud"];
            let binding = &state["binding"];
            if let (Some(base), Some(token)) = (
                binding.get("baseUrl").and_then(Value::as_str),
                binding.get("token").and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok()),
            ) {
                self.index.entry((base.into(), token)).or_default().push(path.clone());
            }
            if let Some(id) = state["id"].as_str().and_then(|s| Uuid::parse_str(s).ok()) {
                self.ids.entry(id).or_default().push(path.clone());
            }
        }
        self.find_duplicates();
    }

    fn find_duplicates(&mut self) {
        for paths in self.index.values().chain(self.ids.values()).filter(|paths| paths.len() > 1) {
            self.duplicates.extend(paths.iter().cloned());
        }
    }

    pub fn token(&self, path: &Path) -> Option<Uuid> {
        self.index
            .iter()
            .find(|(_, paths)| paths.iter().any(|p| p == path))
            .map(|((_, token), _)| *token)
    }
}
