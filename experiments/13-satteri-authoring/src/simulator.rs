//! Explicitly NOT Leaflet's push handler: no SQL/auth/concurrency/publication.
//! Models detached immutable staging and one explicit-ID root/page fact update.
use crate::{
    facts::{Data, Fact, Plan},
    text,
};
use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Binding {
    pub token_locator: String,
    pub root: String,
    pub permission_set: String,
    pub entry_fact: String,
    pub document_uri: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Store {
    pub binding: Binding,
    pub entities: BTreeSet<String>,
    pub facts: BTreeMap<String, Fact>,
}

impl Store {
    pub fn fixture() -> Self {
        let root = Uuid::from_u128(10).to_string();
        let old = Uuid::from_u128(11).to_string();
        let entry_fact = Uuid::from_u128(12).to_string();
        let entry = Fact {
            id: entry_fact.clone(),
            entity: root.clone(),
            attribute: "root/page".into(),
            data: Data::OrderedReference {
                value: old.clone(),
                position: "a0".into(),
            },
        };
        let nav = Fact {
            id: Uuid::from_u128(13).to_string(),
            entity: root.clone(),
            attribute: "root/page".into(),
            data: Data::OrderedReference {
                value: Uuid::from_u128(14).to_string(),
                position: "a1".into(),
            },
        };
        Self {
            binding: Binding {
                token_locator: "fixture-only:no-credential".into(),
                root: root.clone(),
                permission_set: Uuid::from_u128(2).to_string(),
                entry_fact,
                document_uri: "fixture://existing-public-document".into(),
            },
            entities: BTreeSet::from([root, old, Uuid::from_u128(14).to_string()]),
            facts: BTreeMap::from([(entry.id.clone(), entry), (nav.id.clone(), nav)]),
        }
    }
    pub fn active_page(&self) -> &str {
        self.facts[&self.binding.entry_fact].data.reference().expect("entry reference")
    }
    pub fn reachable(&self, root: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut pending = vec![root.to_owned()];
        while let Some(entity) = pending.pop() {
            if !seen.insert(entity.clone()) {
                continue;
            }
            for fact in self.facts.values().filter(|f| f.entity == entity) {
                if let Some(id) = fact.data.reference() {
                    pending.push(id.into());
                }
            }
        }
        seen
    }
    fn equivalent(a: &Fact, b: &Fact) -> Result<bool> {
        if a.id != b.id || a.entity != b.entity || a.attribute != b.attribute {
            return Ok(false);
        }
        match (&a.data, &b.data) {
            (Data::Text { value: a }, Data::Text { value: b }) => Ok(text::read(a)? == text::read(b)?),
            _ => Ok(a.data == b.data),
        }
    }
    pub fn stage(&mut self, plan: &Plan, fail_after: Option<usize>) -> Result<()> {
        ensure!(plan.permission_set == self.binding.permission_set, "wrong entity set");
        // Preflight every existing staged/stable fact before any write. In particular,
        // changing options on a stable Poll must not silently reinterpret old votes.
        for fact in &plan.facts {
            if let Some(old) = self.facts.get(&fact.id) {
                ensure!(
                    Self::equivalent(old, fact)?,
                    "existing generated/poll fact differs; use fresh generation or new Poll key"
                );
            }
        }
        // Also reject removed Poll options, not only changed surviving slots.
        for entity in plan.polls.keys() {
            let old: BTreeSet<_> = self
                .facts
                .values()
                .filter(|f| &f.entity == entity && f.attribute == "poll/options")
                .map(|f| f.id.as_str())
                .collect();
            let next: BTreeSet<_> = plan
                .facts
                .iter()
                .filter(|f| &f.entity == entity && f.attribute == "poll/options")
                .map(|f| f.id.as_str())
                .collect();
            ensure!(old.is_empty() || old == next, "stable Poll option inventory changed; use a new key");
        }
        self.entities.extend(plan.entities.iter().cloned());
        for (i, fact) in plan.facts.iter().enumerate() {
            if fail_after == Some(i) {
                bail!("injected stage interruption");
            }
            // Never reassert independently encoded text over an existing entity.
            self.facts.entry(fact.id.clone()).or_insert_with(|| fact.clone());
        }
        Ok(())
    }
    pub fn verify(&self, plan: &Plan) -> Result<()> {
        ensure!(plan.entities.is_subset(&self.entities), "staged entity missing");
        for expected in &plan.facts {
            let actual = self.facts.get(&expected.id).context("staged fact missing")?;
            ensure!(Self::equivalent(actual, expected)?, "staged fact mismatch");
        }
        for (entity, expected) in &plan.expected_text {
            let Some(Data::Text { value }) = self
                .facts
                .values()
                .find(|f| &f.entity == entity && f.attribute == "block/text")
                .map(|f| &f.data)
            else {
                bail!("missing text fact")
            };
            ensure!(&text::read(value)? == expected, "decoded text mismatch");
        }
        for fact in &plan.facts {
            if let Some(id) = fact.data.reference() {
                ensure!(plan.entities.contains(id), "dangling generated reference");
            }
        }
        Ok(())
    }
    pub fn switch(&mut self, plan: &Plan) -> Result<Value> {
        self.verify(plan)?;
        let entry = self.facts.get_mut(&self.binding.entry_fact).context("lost entry fact")?;
        ensure!(entry.entity == self.binding.root && entry.attribute == "root/page", "entry binding changed");
        let Data::OrderedReference { value, .. } = &mut entry.data else {
            bail!("entry is not ordered-reference")
        };
        *value = plan.main_page.clone();
        Ok(json!({"name":"assertFact","args":entry}))
    }
    pub fn apply(&mut self, plan: &Plan) -> Result<()> {
        self.stage(plan, None)?;
        self.switch(plan)?;
        Ok(())
    }
}
