//! Bounded literal-argument Next action adapters; not a Flight runtime.
use crate::{Error, LeafletClient, transport};
use regex::Regex;
use reqwest::Url;
use serde_json::Value;
use std::collections::BTreeSet;

const MAX_PAGE: usize = 4 * 1024 * 1024;
const MAX_ASSET: usize = 4 * 1024 * 1024;
const MAX_TOTAL: usize = 24 * 1024 * 1024;
const MAX_ASSETS: usize = 128;

pub struct ActionResponse(pub(crate) String);
impl ActionResponse {
    pub fn body(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for ActionResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionResponse").field("bytes", &self.0.len()).finish_non_exhaustive()
    }
}

impl LeafletClient {
    pub(crate) async fn discover_mint_action(&self, dashboard: &Url) -> Result<String, Error> {
        self.discover_action(dashboard, "createPublicationDraft").await
    }

    pub(crate) async fn discover_action(&self, page: &Url, name: &str) -> Result<String, Error> {
        let response = self.http.get(page.clone()).send().await.map_err(transport)?;
        let mut remaining = MAX_TOTAL;
        let html = bounded_text(response, "GET publication dashboard", MAX_PAGE, &mut remaining).await?;
        self.discover_action_in(&html, name).await
    }

    /// Scan one already-fetched page's same-origin static chunks for a
    /// required single action reference. Zero, ambiguous, changed or
    /// redirected discovery refuses.
    pub(crate) async fn discover_action_in(&self, html: &str, name: &str) -> Result<String, Error> {
        self.references_in(html, name)
            .await?
            .ok_or(Error::Native("current native action reference unavailable; no fallback"))
    }

    /// Same bounded scan for an optional action: absent (zero references) is
    /// `None`; two or more references are still an ambiguous refusal.
    pub(crate) async fn discover_optional_action_in(&self, html: &str, name: &str) -> Result<Option<String>, Error> {
        self.references_in(html, name).await
    }

    async fn references_in(&self, html: &str, name: &str) -> Result<Option<String>, Error> {
        // The page fetch already consumed its bytes from this shared budget.
        let mut remaining = MAX_TOTAL.saturating_sub(html.len());
        let tags = Regex::new(r#"(?is)<script\b[^>]*\bsrc\s*=\s*["']([^"']+)["'][^>]*>"#).expect("static regex");
        let mut urls = BTreeSet::new();
        for capture in tags.captures_iter(html) {
            let path = capture[1].replace("&amp;", "&");
            if path.len() > 2048 {
                return Err(Error::Native("action discovery asset path limit"));
            }
            let url = self.base.join(&path).map_err(|_| Error::Native("invalid action asset URL"))?;
            if url.origin() == self.base.origin()
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
                && url.path().starts_with("/_next/static/")
                && url.path().ends_with(".js")
            {
                urls.insert(url);
            }
            if urls.len() > MAX_ASSETS {
                return Err(Error::Native("action discovery asset count limit"));
            }
        }
        let mut references = BTreeSet::new();
        for url in urls {
            let response = self.http.get(url).send().await.map_err(transport)?;
            let text = bounded_text(response, "GET action asset", MAX_ASSET, &mut remaining).await?;
            references.extend(action_references(&text, name));
            if references.len() > 1 {
                return Err(Error::Native("ambiguous native action reference"));
            }
        }
        Ok((references.len() == 1).then(|| references.into_iter().next().expect("one reference")))
    }
}

#[cfg(test)]
pub(crate) fn mint_references(text: &str) -> BTreeSet<String> {
    action_references(text, "createPublicationDraft")
}

fn action_references(text: &str, name: &str) -> BTreeSet<String> {
    // Observed 2026-11-09 on the actual dashboard's same-origin client chunk.
    // Pin the function/export pairing, not a deployment's generated hash.
    let pattern = Regex::new(&format!(r#"createServerReference\)\(\s*"([0-9a-f]{{40,42}})"\s*,\s*([A-Za-z_$][A-Za-z0-9_$]*)\.callServer\s*,\s*void 0\s*,\s*[A-Za-z_$][A-Za-z0-9_$]*\.findSourceMapURL\s*,\s*"{}"\s*\)"#, regex::escape(name))).expect("escaped regex");
    pattern.captures_iter(text).map(|c| c[1].to_owned()).collect()
}

pub(crate) async fn bounded_text(mut response: reqwest::Response, operation: &'static str, max: usize, remaining: &mut usize) -> Result<String, Error> {
    if !response.status().is_success() {
        return Err(Error::Http {
            operation,
            status: response.status().as_u16(),
        });
    }
    if response.content_length().is_some_and(|len| len > max.min(*remaining) as u64) {
        return Err(Error::Native("native response size limit"));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(transport)? {
        if chunk.len() > max.saturating_sub(bytes.len()).min(*remaining) {
            return Err(Error::Native("native response size limit"));
        }
        *remaining -= chunk.len();
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| Error::Native("invalid native response UTF-8"))
}

/// Decode just the action return slot, either literal or a single promise row.
/// Unrelated render payloads are not interpreted as mint results.
pub(crate) fn mint_result(body: &str) -> Result<Option<uuid::Uuid>, Error> {
    match action_result(body)? {
        Value::Null => Ok(None),
        Value::String(token) => uuid::Uuid::parse_str(&token)
            .map(Some)
            .map_err(|_| Error::Native("action returned invalid permission token")),
        _ => Err(Error::Native("unsupported action result")),
    }
}

pub(crate) fn action_result(body: &str) -> Result<Value, Error> {
    if body.len() > 1024 * 1024 {
        return Err(Error::Native("action response size limit"));
    }
    let mut root = None;
    let mut rows = std::collections::BTreeMap::new();
    let mut empty_iterables = std::collections::BTreeMap::new();
    for (index, line) in body.lines().take(4097).enumerate() {
        if index >= 4096 {
            return Err(Error::Native("action row count limit"));
        }
        let Some((id, value)) = line.split_once(':') else {
            return Err(Error::Native("invalid action row"));
        };
        // Next emits repeated no-model resource-hint rows. They are not
        // promise/model IDs and cannot select an action return. This form was
        // also observed in the read-only deployed dashboard Flight stream.
        if id.is_empty() && value.starts_with("HL") {
            let hint: Value = serde_json::from_str(&value[2..]).map_err(|_| Error::Native("invalid Flight preload hint"))?;
            if !hint.is_array() {
                return Err(Error::Native("invalid Flight preload hint"));
            }
            continue;
        }
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::Native("invalid/duplicate action row ID"));
        }
        // Observed in the retained 2026-11-10 publication response: an unrelated
        // empty async iterable emits `12:X` then `12:C`. React Flight uses X for
        // an async iterable and C for its close. This is NOT two model rows.
        // Admit only that complete empty pair, never values, arbitrary duplicate
        // models, a stream as root/action result, or a general streaming runtime.
        if value == "X" && id != "0" && !rows.contains_key(id) && !empty_iterables.contains_key(id) {
            empty_iterables.insert(id, false);
            continue;
        }
        if value == "C" && empty_iterables.get(id) == Some(&false) {
            empty_iterables.insert(id, true);
            continue;
        }
        if matches!(value, "X" | "C") || empty_iterables.contains_key(id) || rows.insert(id, value).is_some() {
            return Err(Error::Native("invalid/duplicate action row ID"));
        }
        if id == "0" {
            root = Some(value);
        }
    }
    if empty_iterables.values().any(|closed| !closed) {
        return Err(Error::Native("incomplete action response async iterable"));
    }
    let value: Value =
        serde_json::from_str(root.ok_or(Error::Native("action return root missing"))?).map_err(|_| Error::Native("invalid action return root"))?;
    let mut result = value.get("a").ok_or(Error::Native("action return slot missing"))?.clone();
    if let Some(reference) = result.as_str().and_then(|s| s.strip_prefix("$@")) {
        let row = rows.get(reference).ok_or(Error::Native("action result promise unresolved"))?;
        result = serde_json::from_str(row).map_err(|_| Error::Native("action result failed or unsupported"))?;
    }
    Ok(result)
}
