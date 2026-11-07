//! `POST /api/rpc/push` — the write half of the Replicache surface
//! (`experiments/02-draft-internals/REPORT.md` §3).
//!
//! Request: `{pushRequest: PushRequestV1, rootEntity, token: {id}}` —
//! mutations carry monotonically increasing ids per `clientID` within a
//! `clientGroupID`; the server skips any mutation with
//! `id <= lastMutationID` (replay protection), so a retried push with the
//! same ids is idempotent. Response: `{"result": null}` on success; the
//! familiar `{result: {error …}}` envelope rides HTTP 200 on failure.
//!
//! The body builder and response parser are pure and offline-testable;
//! [`LeafletClient::push`] is the thin live composition (the same split
//! `pull` uses). The CLI's `drafts author` command wraps its transport in
//! an injected closure (`PushFn`) so the full flow is tested offline — no
//! test executes a push against any real draft.

use serde_json::{Value, json};

use crate::Error;
use crate::LeafletClient;
use crate::author::AuthorMutation;

/// The all-zero profile id Replicache clients send when no profile is
/// registered (the value leaflet's own client and experiments/02 use).
pub const ZERO_PROFILE_ID: &str = "00000000-0000-0000-0000-000000000000";

/// Build the exact `POST /api/rpc/push` body for a mutation batch
/// (`fixtures/push-minimal-authoring.json` shape): `pushVersion` 1, empty
/// `schemaVersion`, the caller's `clientGroupID`, and the mutations in
/// order.
pub fn build_push_body(mutations: &[AuthorMutation], client_group_id: &str, root_entity: &str, token_id: &str) -> Value {
    json!({
        "pushRequest": {
            "pushVersion": 1,
            "schemaVersion": "",
            "profileID": ZERO_PROFILE_ID,
            "clientGroupID": client_group_id,
            "mutations": mutations,
        },
        "rootEntity": root_entity,
        "token": {"id": token_id},
    })
}

/// Parse a push response body. Success is `{"result": null}` (or an absent
/// result member); errors ride HTTP 200 in the same `result.error`
/// envelope as the other RPCs.
pub fn parse_push_response(http_status: u16, body: &str) -> Result<(), Error> {
    let value: Value = serde_json::from_str(body).map_err(|e| Error::BadResponse {
        rpc: "push".into(),
        detail: format!("invalid JSON (HTTP {http_status}): {e}"),
    })?;
    let Some(result) = value.get("result") else {
        return Err(Error::BadResponse {
            rpc: "push".into(),
            detail: "no `result` envelope".into(),
        });
    };
    if let Some(error) = result.get("error").filter(|e| !e.is_null()) {
        let code = error.get("code").and_then(Value::as_str).unwrap_or("UNKNOWN");
        let message = error.get("message").and_then(Value::as_str).unwrap_or_default();
        let result_status = result.get("status").and_then(Value::as_i64).unwrap_or(0);
        if code == "PGRST116" || result_status == 406 {
            return Err(Error::UnknownDraft(String::new()));
        }
        return Err(Error::Rpc {
            rpc: "push".into(),
            code: code.into(),
            message: message.into(),
        });
    }
    Ok(())
}

impl LeafletClient {
    /// Execute a mutation batch against a draft (the write RPC). Live
    /// pushes are operator-gated — see the README procedure; tests
    /// exercise this exclusively through the injected-transport path.
    pub async fn push(&self, token_id: &str, root_entity: &str, mutations: &[AuthorMutation], client_group_id: &str) -> Result<(), Error> {
        let url = self.url_for("api/rpc/push");
        let body = build_push_body(mutations, client_group_id, root_entity, token_id);
        tracing::debug!(url = %url, token_id = %token_id, mutations = mutations.len(), "POST push");
        let response = self.http.post(&url).json(&body).send().await?;
        let status = response.status();
        let text = response.text().await?;
        tracing::debug!(status = %status, "POST push done");
        if !status.is_success() {
            return Err(Error::HttpStatus { url, status: status.as_u16() });
        }
        parse_push_response(status.as_u16(), &text).map_err(|err| match err {
            // The envelope's error body need not echo the id; we know it.
            Error::UnknownDraft(_) => Error::UnknownDraft(token_id.to_string()),
            other => other,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::author::AuthorMutation;

    macro_rules! exp02_fixture {
        ($file:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../experiments/02-draft-internals/fixtures/",
                $file
            ))
        };
    }

    fn sample_mutations() -> Vec<AuthorMutation> {
        vec![
            AuthorMutation {
                id: 1,
                name: "addBlock".into(),
                client_id: "springleaf-client-1".into(),
                timestamp: 1_794_000_000_000,
                args: json!({"parent": "p", "permission_set": "ps", "factID": "f", "type": "heading",
                             "newEntityID": "e", "position": "a0"}),
            },
            AuthorMutation {
                id: 2,
                name: "updateLeafletMetadata".into(),
                client_id: "springleaf-client-1".into(),
                timestamp: 1_794_000_000_001,
                args: json!({"title": "Synthetic title"}),
            },
        ]
    }

    /// Wire shape parity with experiments/02's committed push fixture:
    /// same envelope members, same PushRequestV1 fields, same mutation
    /// row shape, same constant values.
    #[test]
    fn body_shape_matches_the_experiment02_fixture() {
        let fixture: Value = serde_json::from_str(exp02_fixture!("push-minimal-authoring.json")).unwrap();
        let body = build_push_body(
            &sample_mutations(),
            "springleaf-author-group-1",
            "01999000-0000-7aaa-8000-000000000002",
            "01999000-0000-7aaa-8000-000000000001",
        );
        // Envelope keys and constant fields, exactly.
        let request = &fixture["request"];
        assert_eq!(body["rootEntity"], request["rootEntity"]);
        assert_eq!(body["token"], request["token"]);
        assert_eq!(body["pushRequest"]["pushVersion"], request["pushRequest"]["pushVersion"]);
        assert_eq!(body["pushRequest"]["pushVersion"], json!(1));
        assert_eq!(body["pushRequest"]["schemaVersion"], request["pushRequest"]["schemaVersion"]);
        assert_eq!(body["pushRequest"]["schemaVersion"], json!(""));
        assert_eq!(body["pushRequest"]["profileID"], request["pushRequest"]["profileID"]);
        assert_eq!(body["pushRequest"]["profileID"], json!(ZERO_PROFILE_ID));
        assert_eq!(body["pushRequest"]["clientGroupID"], request["pushRequest"]["clientGroupID"]);
        // Mutation rows serialize to the fixture's key set, camelCase.
        let row = &body["pushRequest"]["mutations"][0];
        let fixture_row = &request["pushRequest"]["mutations"][0];
        let keys: Vec<&str> = row.as_object().unwrap().keys().map(String::as_str).collect();
        let fixture_keys: Vec<&str> = fixture_row.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, fixture_keys);
        assert_eq!(keys, vec!["id", "name", "clientID", "timestamp", "args"]);
        // The whole body round-trips through JSON untouched (no
        // serialization surprises via the Value path the client posts).
        assert_eq!(serde_json::from_str::<Value>(&serde_json::to_string(&body).unwrap()).unwrap(), body);
    }

    #[test]
    fn response_parsing_all_three_shapes() {
        assert!(parse_push_response(200, r#"{"result": null}"#).is_ok());
        assert!(parse_push_response(200, r#"{"result": {"error": null}}"#).is_ok());
        let err = parse_push_response(200, r#"{"result": {"error": {"code": "PGRST-2", "message": "write denied"}, "status": 500}}"#).unwrap_err();
        assert!(matches!(&err, Error::Rpc { code, .. } if code == "PGRST-2"), "{err}");
        assert!(matches!(parse_push_response(200, "not json"), Err(Error::BadResponse { .. })));
        assert!(matches!(parse_push_response(200, "{}"), Err(Error::BadResponse { .. })));
    }
}
