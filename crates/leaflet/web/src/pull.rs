//! Proven null-cookie native read, with full raw current observations retained.
use crate::{Error, LeafletClient, transport};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct PullSnapshot {
    pub raw: Value,
    pub facts: Vec<Value>,
    pub extras: BTreeMap<String, Value>,
}

pub fn parse_pull(raw: Value) -> Result<PullSnapshot, Error> {
    let patch = raw.get("patch").and_then(Value::as_array).ok_or(Error::Pull("missing patch array"))?;
    if patch.first().and_then(|p| p["op"].as_str()) != Some("clear") {
        return Err(Error::Pull("not a full snapshot (initial clear missing)"));
    }
    let mut map = BTreeMap::new();
    for op in patch.iter().skip(1) {
        match op["op"].as_str() {
            Some("put") => {
                let key = op["key"].as_str().ok_or(Error::Pull("invalid patch key"))?;
                let value = op.get("value").ok_or(Error::Pull("missing patch value"))?;
                if map.insert(key.to_owned(), value.clone()).is_some() {
                    return Err(Error::Pull("duplicate full-snapshot key"));
                }
            }
            _ => return Err(Error::Pull("unexpected patch operation in full snapshot")),
        }
    }
    if map.get("initialized") != Some(&Value::Bool(true)) {
        return Err(Error::Pull("editor not initialized/access unavailable"));
    }
    let mut facts = vec![];
    let mut extras = BTreeMap::new();
    for (key, value) in map {
        if Uuid::parse_str(&key).is_ok() {
            if value["id"].as_str() != Some(&key) || !value["entity"].is_string() || !value["attribute"].is_string() || !value["data"].is_object() {
                return Err(Error::Pull("malformed fact row"));
            }
            facts.push(value);
        } else {
            extras.insert(key, value);
        }
    }
    Ok(PullSnapshot { raw, facts, extras })
}

impl LeafletClient {
    pub async fn pull(&self, id: Uuid) -> Result<PullSnapshot, Error> {
        let response = self
            .http
            .post(self.base.join("api/rpc/pull").expect("static path"))
            .json(&serde_json::json!({"token_id":id,"pullRequest":{
                "pullVersion":1,"schemaVersion":"","profileID":"leafspring-read-only",
                "cookie":null,"clientGroupID":"leafspring-read-only"
            }}))
            .send()
            .await
            .map_err(transport)?;
        if !response.status().is_success() {
            return Err(Error::Http {
                operation: "POST pull",
                status: response.status().as_u16(),
            });
        }
        let text = response.text().await.map_err(transport)?;
        let raw = serde_json::from_str(&text).map_err(|_| Error::Pull("invalid JSON"))?;
        parse_pull(raw)
    }
}
