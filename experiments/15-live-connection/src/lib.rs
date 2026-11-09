//! Synthetic protocol and scheduling models for the experiment, NOT production.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub fn join(topic: &str, reference: &str, key: &str, presence: bool) -> Value {
    json!({"topic":topic,"event":"phx_join","ref":reference,"join_ref":reference,
        "payload":{"config":{"broadcast":{"ack":false,"self":false},
            "presence":{"key":"","enabled":presence},"postgres_changes":[],"private":false},
            "access_token":key}})
}

pub fn control(topic: &str, event: &str, reference: &str, join_ref: Option<&str>) -> Value {
    json!({"topic":topic,"event":event,"payload":{},"ref":reference,"join_ref":join_ref})
}

/// Count connection metas, not authenticated people. A snapshot is mandatory;
/// state is unknown after a gap even if the old count was zero.
#[derive(Default)]
pub struct Presence {
    pub ready: bool,
    pub refs: BTreeMap<String, BTreeSet<String>>,
}

impl Presence {
    pub fn gap(&mut self) {
        self.ready = false;
        self.refs.clear();
    }

    pub fn snapshot(&mut self, payload: &Value) -> Result<usize, &'static str> {
        let mut candidate = Self::default();
        candidate.add(payload)?;
        candidate.ready = true;
        *self = candidate;
        Ok(self.count())
    }

    pub fn diff(&mut self, payload: &Value) -> Result<Option<usize>, &'static str> {
        if !self.ready {
            return Ok(None);
        }
        self.add(&payload["joins"])?;
        for (key, entry) in payload["leaves"].as_object().ok_or("invalid leaves")? {
            for meta in entry["metas"].as_array().ok_or("invalid leave metas")? {
                let reference = meta["phx_ref"].as_str().ok_or("invalid leave ref")?;
                if let Some(refs) = self.refs.get_mut(key) {
                    refs.remove(reference);
                    if refs.is_empty() {
                        self.refs.remove(key);
                    }
                }
            }
        }
        Ok(Some(self.count()))
    }

    fn add(&mut self, payload: &Value) -> Result<(), &'static str> {
        for (key, entry) in payload.as_object().ok_or("invalid presence map")? {
            for meta in entry["metas"].as_array().ok_or("invalid presence metas")? {
                let reference = meta["phx_ref"].as_str().ok_or("invalid presence ref")?;
                let refs = self.refs.entry(key.clone()).or_default();
                if let Some(previous) = meta["phx_ref_prev"].as_str() {
                    refs.remove(previous);
                }
                refs.insert(reference.to_owned());
            }
        }
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.refs.values().map(BTreeSet::len).sum()
    }
}

/// Lossless dirty/coalescing rule. Clear pending when STARTING, never at finish.
#[derive(Default)]
pub struct Dirty {
    pub pending: bool,
    pub in_flight: bool,
}

impl Dirty {
    pub fn wake(&mut self) {
        self.pending = true;
    }
    pub fn start(&mut self) -> bool {
        if self.in_flight || !self.pending {
            return false;
        }
        self.pending = false;
        self.in_flight = true;
        true
    }
    pub fn finish(&mut self) {
        self.in_flight = false;
    }
}
