//! Serial selected-file complete editor saves; never release or remote discovery.
use crate::{RemoteReader, config, inventory, native};
use anyhow::{Context, Result, ensure};
use leafbud_web::LeafletState;
use serde::Serialize;
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Default)]
pub struct Request {
    pub files: Vec<PathBuf>,
    pub all: bool,
    pub state: Option<LeafletState>,
    pub yes: bool,
    pub take_local: bool,
    pub recreate: BTreeSet<String>,
}
#[derive(Serialize)]
pub struct Report {
    pub version: u8,
    pub preview: bool,
    pub selected: usize,
    pub skipped: usize,
    pub items: Vec<Item>,
}
#[derive(Serialize)]
pub struct Item {
    pub path: PathBuf,
    #[serde(flatten)]
    pub result: native::SaveReport,
}
impl RemoteReader {
    pub async fn push(&self, context: &config::Context, request: Request) -> Result<Report> {
        ensure!(request.all ^ !request.files.is_empty(), "select FILE... or --all");
        ensure!(request.state.is_none() || request.all, "--state requires --all");
        ensure!(
            request.recreate.is_empty() || !request.all && request.files.len() == 1,
            "--recreate selects keys in exactly one explicit file"
        );
        let mut inventory = inventory::scan(&context.workspace_root, Some(self.client.base_url()))?;
        let paths = if request.all {
            inventory.paths.clone()
        } else {
            request
                .files
                .iter()
                .map(|p| {
                    std::fs::canonicalize(if p.is_absolute() { p.clone() } else { context.workspace_root.join(p) })
                        .with_context(|| format!("push operand {}", p.display()))
                })
                .collect::<Result<BTreeSet<_>>>()?
                .into_iter()
                .collect()
        };
        inventory.include_operands(&paths);
        let mut report = Report {
            version: 1,
            preview: !request.yes,
            selected: 0,
            skipped: 0,
            items: vec![],
        };
        for path in &paths {
            if request
                .state
                .is_some_and(|state| inventory.states.get(path).is_some_and(|s| s != state.as_str()))
            {
                report.skipped += 1;
                continue;
            }
            report.selected += 1;
            let result = self
                .save_native(
                    context,
                    path,
                    native::Selection { operands: paths.clone() },
                    native::SaveOptions {
                        yes: request.yes,
                        take_local: request.take_local,
                        recreate: request.recreate.clone(),
                    },
                )
                .await
                .unwrap_or_else(|error| native::SaveReport {
                    outcome: "failed",
                    effects: Default::default(),
                    message: Some(format!("{error:#}")),
                });
            report.items.push(Item { path: path.clone(), result });
        }
        Ok(report)
    }
}
