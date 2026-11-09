use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

// Store only connection refs, never awareness/client/editor payloads. Limits also
// bound derived memory across a malicious sequence of individually small diffs.
const MAX_REFS: usize = 4096;
const MAX_STRING: usize = 512;

#[derive(Clone, Default)]
pub(super) struct Presence {
    ready: bool,
    refs: BTreeMap<String, BTreeSet<String>>,
}

impl Presence {
    pub fn gap(&mut self) {
        *self = Self::default();
    }

    pub fn snapshot(&mut self, payload: &Value) -> Result<usize, ()> {
        let mut next = Self::default();
        next.add(payload)?;
        next.ready = true;
        let count = next.count()?;
        *self = next;
        Ok(count)
    }

    pub fn diff(&mut self, payload: &Value) -> Result<Option<usize>, ()> {
        // Validate even when unready, but never promote diffs to a snapshot.
        let mut next = self.clone();
        next.add(payload.get("joins").ok_or(())?)?;
        for (key, refs) in decode(payload.get("leaves").ok_or(())?)? {
            if let Some(current) = next.refs.get_mut(&key) {
                for (reference, _) in refs {
                    current.remove(&reference);
                }
                if current.is_empty() {
                    next.refs.remove(&key);
                }
            }
        }
        let count = next.count()?;
        if !self.ready {
            return Ok(None);
        }
        *self = next;
        Ok(Some(count))
    }

    fn add(&mut self, payload: &Value) -> Result<(), ()> {
        for (key, refs) in decode(payload)? {
            let current = self.refs.entry(key).or_default();
            for (reference, previous) in refs {
                if let Some(previous) = previous {
                    current.remove(&previous);
                }
                current.insert(reference);
            }
        }
        self.refs.retain(|_, refs| !refs.is_empty());
        self.count()?;
        Ok(())
    }

    fn count(&self) -> Result<usize, ()> {
        let count: usize = self.refs.values().map(BTreeSet::len).sum();
        if count > MAX_REFS { Err(()) } else { Ok(count) }
    }
}

type Decoded = BTreeMap<String, Vec<(String, Option<String>)>>;

fn decode(payload: &Value) -> Result<Decoded, ()> {
    let object = payload.as_object().ok_or(())?;
    if object.len() > MAX_REFS {
        return Err(());
    }
    let mut decoded = BTreeMap::new();
    let mut count = 0;
    for (key, value) in object {
        if key.len() > MAX_STRING {
            return Err(());
        }
        let mut refs = Vec::new();
        for meta in value.get("metas").and_then(Value::as_array).ok_or(())? {
            count += 1;
            if count > MAX_REFS {
                return Err(());
            }
            let reference = meta
                .get("phx_ref")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty() && s.len() <= MAX_STRING)
                .ok_or(())?;
            let previous = match meta.get("phx_ref_prev") {
                None | Some(Value::Null) => None,
                Some(Value::String(s)) if !s.is_empty() && s.len() <= MAX_STRING => Some(s.clone()),
                _ => return Err(()),
            };
            refs.push((reference.to_owned(), previous));
        }
        decoded.insert(key.clone(), refs);
    }
    Ok(decoded)
}
