//! `POST /api/rpc/pull` — the read-only full-snapshot pull
//! (`experiments/02-draft-internals/REPORT.md` §2).
//!
//! Request: `{pullRequest: PullRequestV1, token_id}`; `cookie: null` (or a
//! legacy number) yields the **full-snapshot** `PullResponseOKV1`; a fresh
//! `clientGroupID` gets `lastMutationIDChanges: {}`. The response body is
//! the bare `PullResponseOKV1` (pull, unlike `get_leaflet_data`, does not
//! wrap itself in a `{result}` envelope).
//!
//! The patch stream is applied generically (`clear`/`put`/`del` into a map)
//! so both the full-snapshot and CVR-diff shapes decode; uuid-keyed `put`
//! values are fact rows, everything else (`initialized`,
//! `publication_{title,description,tags}`, `post_preferences`,
//! `draft_contributors`) lands in [`PullSnapshot::extras`] — tolerated and
//! read selectively, never fatal.

use serde::Deserialize;
use serde_json::Value;

use crate::facts::{FactData, FactRow};
use crate::{Error, LeafletClient};

/// One Replicache patch op (`clear` | `put` | `del`).
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum PatchOp {
    Clear,
    Put { key: String, value: Value },
    Del { key: String },
}

/// The bare `PullResponseOKV1` body.
#[derive(Debug, Deserialize)]
struct PullResponse {
    #[serde(default)]
    patch: Vec<PatchOp>,
}

/// An applied pull: fact rows plus the non-fact extras.
#[derive(Debug, Default)]
pub struct PullSnapshot {
    pub facts: Vec<FactRow>,
    /// Non-fact patch values keyed by their patch key (`initialized`,
    /// `publication_tags`, `draft_contributors`, …).
    pub extras: std::collections::BTreeMap<String, Value>,
}

impl PullSnapshot {
    /// `publication_tags` extra as a string list, when present and shaped.
    pub fn publication_tags(&self) -> Option<Vec<String>> {
        let value = self.extras.get("publication_tags")?;
        let items = value.as_array()?;
        let tags: Vec<String> = items.iter().filter_map(Value::as_str).map(str::to_string).collect();
        (items.len() == tags.len()).then_some(tags)
    }
}

/// Parse a full-snapshot (or CVR-diff) pull response body into fact rows +
/// extras.
pub fn parse_pull_snapshot(body: &str) -> Result<PullSnapshot, Error> {
    let response: PullResponse = serde_json::from_str(body).map_err(|e| Error::BadResponse {
        rpc: "pull".into(),
        detail: format!("invalid PullResponseOKV1: {e}"),
    })?;
    let mut map: std::collections::BTreeMap<String, Value> = Default::default();
    for op in response.patch {
        match op {
            PatchOp::Clear => map.clear(),
            PatchOp::Put { key, value } => {
                map.insert(key, value);
            }
            PatchOp::Del { key } => {
                map.remove(&key);
            }
        }
    }
    let mut facts = Vec::new();
    let mut extras = std::collections::BTreeMap::new();
    for (key, value) in map {
        match as_fact_row(&value) {
            Some(row) if uuid::Uuid::parse_str(&key).is_ok() && row.id == key => facts.push(row),
            _ => {
                extras.insert(key, value);
            }
        }
    }
    Ok(PullSnapshot { facts, extras })
}

/// Decode a patch value as a fact row (the `FactWithIndexes` shape minus
/// `indexes`, which is a client-side recompute and ignored).
fn as_fact_row(value: &Value) -> Option<FactRow> {
    let object = value.as_object()?;
    let id = object.get("id").and_then(Value::as_str)?;
    let entity = object.get("entity").and_then(Value::as_str)?;
    let attribute = object.get("attribute").and_then(Value::as_str)?;
    let data = object.get("data")?;
    Some(FactRow {
        id: id.to_string(),
        entity: entity.to_string(),
        attribute: attribute.to_string(),
        data: FactData::parse(data),
    })
}

impl LeafletClient {
    /// Read-only full-snapshot pull for a draft token id: every fact of the
    /// entity set plus the client extras.
    pub async fn pull(&self, token_id: &str) -> Result<PullSnapshot, Error> {
        let url = self.url_for("api/rpc/pull");
        let body = serde_json::json!({
            "pullRequest": {
                "pullVersion": 1,
                "schemaVersion": "",
                "profileID": "00000000-0000-0000-0000-000000000000",
                "cookie": null,
                "clientGroupID": concat!("springleaf/", env!("CARGO_PKG_VERSION")),
            },
            "token_id": token_id,
        });
        tracing::debug!(url = %url, token_id = %token_id, "POST pull");
        let response = self.http.post(&url).json(&body).send().await?;
        let status = response.status();
        tracing::debug!(status = %status, "POST pull done");
        if !status.is_success() {
            return Err(Error::HttpStatus { url, status: status.as_u16() });
        }
        let text = response.text().await?;
        parse_pull_snapshot(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    macro_rules! exp02_fixture {
        ($file:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../experiments/02-draft-internals/fixtures/",
                $file
            ))
        };
    }

    #[test]
    fn parses_the_experiment02_full_snapshot_fixture() {
        let envelope: Value = serde_json::from_str(exp02_fixture!("pull-full-snapshot.json")).unwrap();
        let body = serde_json::to_string(&envelope["response"]).unwrap();
        let snapshot = parse_pull_snapshot(&body).unwrap();
        assert_eq!(snapshot.facts.len(), 2, "both fact rows decode");
        assert_eq!(snapshot.facts[0].id, "01999000-0000-7aaa-8000-000000000021");
        assert_eq!(snapshot.facts[0].attribute, "root/page");
        assert_eq!(
            snapshot.facts[0].data,
            FactData::OrderedReference { value: "01999000-0000-7aaa-8000-000000000003".into(), position: "a0".into() }
        );
        assert_eq!(snapshot.facts[1].attribute, "theme/page-leaflet-watermark");
        // Extras: initialized + draft_contributors, not facts.
        assert_eq!(snapshot.extras.get("initialized"), Some(&json!(true)));
        assert_eq!(snapshot.extras.get("draft_contributors"), Some(&json!([])));
        assert_eq!(snapshot.extras.len(), 2);
        assert_eq!(snapshot.publication_tags(), None);
    }

    #[test]
    fn cvr_diff_shape_and_del_ops_apply() {
        // A diff pull (no clear): put + del against the running map.
        let body = json!({
            "cookie": {"order": 1, "cvrID": "x"},
            "lastMutationIDChanges": {"c1": 5},
            "patch": [
                {"op": "put", "key": "01999000-0000-7aaa-8000-000000000021",
                 "value": {"id": "01999000-0000-7aaa-8000-000000000021",
                           "entity": "e1", "attribute": "page/type",
                           "data": {"type": "page-type-union", "value": "doc"}}},
                {"op": "put", "key": "publication_tags", "value": ["news", "meta"]},
                {"op": "del", "key": "initialized"}
            ]
        })
        .to_string();
        let snapshot = parse_pull_snapshot(&body).unwrap();
        assert_eq!(snapshot.facts.len(), 1);
        assert_eq!(snapshot.extras.get("publication_tags"), Some(&json!(["news", "meta"])));
        assert_eq!(snapshot.publication_tags(), Some(vec!["news".to_string(), "meta".to_string()]));
        assert!(snapshot.extras.get("initialized").is_none());
    }

    #[test]
    fn malformed_body_is_a_bad_response() {
        assert!(matches!(
            parse_pull_snapshot("not json"),
            Err(Error::BadResponse { rpc, .. }) if rpc == "pull"
        ));
        // A patch value that is not a fact row object degrades to an extra,
        // never an error.
        let body = json!({"patch": [{"op": "put", "key": "weird", "value": 3}]}).to_string();
        let snapshot = parse_pull_snapshot(&body).unwrap();
        assert!(snapshot.facts.is_empty());
        assert_eq!(snapshot.extras.get("weird"), Some(&json!(3)));
    }
}
