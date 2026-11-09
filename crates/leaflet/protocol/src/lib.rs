//! Native Leaflet wire vocabulary, not a semantic change language or replica.
//! No HTTP, files, authoring model or Yrs. Unknown read fields remain in `raw`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

mod push;
pub use push::*;

pub const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;

#[derive(Debug, thiserror::Error)]
#[error("invalid native protocol: {0}")]
pub struct Error(pub &'static str);

/// A fresh null-cookie full view. Ack values are processing evidence only.
#[derive(Debug, Clone)]
pub struct FullView {
    pub raw: Value,
    pub facts: Vec<Value>,
    pub extras: BTreeMap<String, Value>,
    pub acknowledgments: BTreeMap<String, u64>,
}

impl FullView {
    pub fn parse(raw: Value) -> Result<Self, Error> {
        let patch = raw.get("patch").and_then(Value::as_array).ok_or(Error("missing patch array"))?;
        if patch.first().and_then(|p| p["op"].as_str()) != Some("clear") {
            return Err(Error("not a full snapshot (initial clear missing)"));
        }
        let mut map = BTreeMap::new();
        for op in patch.iter().skip(1) {
            if op["op"] != "put" {
                return Err(Error("unexpected full-snapshot operation"));
            }
            let key = op["key"].as_str().ok_or(Error("invalid patch key"))?;
            let value = op.get("value").ok_or(Error("missing patch value"))?;
            if map.insert(key.to_owned(), value.clone()).is_some() {
                return Err(Error("duplicate full-snapshot key"));
            }
        }
        if map.get("initialized") != Some(&Value::Bool(true)) {
            return Err(Error("editor not initialized/access unavailable"));
        }
        let mut facts = vec![];
        let mut extras = BTreeMap::new();
        for (key, value) in map {
            if Uuid::parse_str(&key).is_ok() {
                validate_fact(&value)?;
                if value["id"] != key {
                    return Err(Error("fact key/id mismatch"));
                }
                facts.push(value);
            } else {
                extras.insert(key, value);
            }
        }
        let acknowledgments = match raw.get("lastMutationIDChanges") {
            None => BTreeMap::new(),
            Some(value) => {
                let map = value.as_object().ok_or(Error("invalid acknowledgment map"))?;
                let mut acks = BTreeMap::new();
                for (client, id) in map {
                    let id = id
                        .as_u64()
                        .filter(|id| *id <= MAX_SAFE_INTEGER)
                        .ok_or(Error("invalid mutation acknowledgment"))?;
                    acks.insert(client.clone(), id);
                }
                acks
            }
        };
        Ok(Self {
            raw,
            facts,
            extras,
            acknowledgments,
        })
    }

    pub fn acknowledged(&self, client: Uuid) -> Option<u64> {
        self.acknowledgments.get(&client.to_string()).copied()
    }
}

/// Validate the narrow fact envelope while retaining every unknown row/data field.
pub fn validate_fact(value: &Value) -> Result<(), Error> {
    if !value.is_object()
        || ["id", "entity"].iter().any(|key| value[*key].as_str().is_none_or(str::is_empty))
        || value["attribute"].as_str().is_none_or(str::is_empty)
        || !value["data"].is_object()
    {
        return Err(Error("malformed fact row"));
    }
    Ok(())
}

/// getFactsForRoots is a depth-3 descendant read, not an entity existence API.
/// Empty rows mean no observable facts; they never prove noncreation.
#[derive(Debug)]
pub struct ExplicitFacts {
    pub raw: Value,
    pub roots: BTreeMap<Uuid, Vec<Value>>,
}

impl ExplicitFacts {
    pub fn parse(raw: Value, requested: &[Uuid]) -> Result<Self, Error> {
        let map = raw["result"]["facts"].as_object().ok_or(Error("missing explicit facts result"))?;
        let requested: BTreeSet<_> = requested.iter().copied().collect();
        let mut roots = requested.iter().map(|id| (*id, vec![])).collect::<BTreeMap<_, _>>();
        for (root, rows) in map {
            let root = Uuid::parse_str(root).map_err(|_| Error("invalid explicit root"))?;
            if !requested.contains(&root) {
                return Err(Error("unexpected explicit root"));
            }
            let rows = rows.as_array().ok_or(Error("invalid explicit fact rows"))?;
            let mut ids = BTreeSet::new();
            let mut unique = Vec::new();
            for row in rows {
                validate_fact(row)?;
                if row.get("root_id").is_some_and(|id| id.as_str() != Some(root.to_string().as_str())) {
                    return Err(Error("explicit row/root mismatch"));
                }
                // The SQL recursive walk may reach one fact at several depths.
                if ids.insert(row.to_string()) {
                    unique.push(row.clone());
                }
            }
            roots.insert(root, unique);
        }
        Ok(Self { raw, roots })
    }

    /// Exact known fact lookup inside the explicitly selected entity's rows.
    /// No guessed fact-ID endpoint is used.
    pub fn fact(&self, entity: Uuid, id: Uuid) -> Option<&Value> {
        self.roots
            .get(&entity)?
            .iter()
            .find(|row| row["entity"] == entity.to_string() && row["id"] == id.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Writer {
    pub client_group_id: Uuid,
    pub client_id: Uuid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub token: Uuid,
    pub root: Uuid,
    pub entity_set: Uuid,
}

#[cfg(test)]
mod tests;
