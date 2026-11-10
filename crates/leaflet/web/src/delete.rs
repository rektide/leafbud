//! Literal `deleteLeaflet`/`deletePost` action contracts at Leaflet 600c7b5f.
//! This is a transport, NOT deletion authorization, binding verification or
//! local unbinding. The workflow owns those guards; nothing here retries.
//!
//! Upstream shape (pinned source, not a deployed probe): the home/dashboard
//! options menu calls `deleteLeaflet(permission_token)` with the observed
//! `get_leaflet_data` token object, and for a published document FIRST calls
//! `deletePost(document_uri)`, which deletes the PDS record from BOTH document
//! collections, the `documents` rows and `leaflets_in_publications` linkage
//! before the editor's entity set and permission token are removed.
use crate::{Error, LeafletClient, actions, transport};
use reqwest::Url;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

/// Current-origin/session discovery result. Preparing is read-only; invoking
/// either action is a permanent destructive effect.
pub struct DeletePreparation {
    base: String,
    session: Uuid,
    token: Uuid,
    page: Url,
    leaflet_reference: String,
    post_reference: Option<String>,
    actor: String,
}
impl DeletePreparation {
    pub fn actor(&self) -> &str {
        &self.actor
    }
    /// `deletePost` is only dispatched for a published document; a deployment
    /// chunk set without that reference still permits draft editor deletion.
    pub fn post_reference(&self) -> Option<&str> {
        self.post_reference.as_deref()
    }
}
impl std::fmt::Debug for DeletePreparation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeletePreparation").finish_non_exhaustive()
    }
}

impl LeafletClient {
    /// Read-only authenticated identity + same-origin /home chunk scan for
    /// the two native delete actions. No saved action hash, deployment
    /// probing or foreign asset fetch. `deleteLeaflet` must resolve
    /// unambiguously; `deletePost` may be absent (draft-only deployments).
    pub async fn prepare_delete(&self, token: Uuid) -> Result<DeletePreparation, Error> {
        if !self.has_cookie {
            return Err(Error::Native("delete requires cookie authentication"));
        }
        let response = self.http.get(self.base.join("home").expect("static path")).send().await.map_err(transport)?;
        let html = actions::bounded_text(response, "GET /home before delete", 4 * 1024 * 1024, &mut (4 * 1024 * 1024)).await?;
        let actor = crate::parse_home(&html)?
            .identity
            .did
            .ok_or(Error::Native("delete requires an authenticated DID"))?;
        let leaflet_reference = self.discover_action_in(&html, "deleteLeaflet").await?;
        let post_reference = self.discover_optional_action_in(&html, "deletePost").await?;
        Ok(DeletePreparation {
            base: self.base_url().into(),
            session: self.session,
            token,
            page: self.base.join("home").expect("static path"),
            leaflet_reference,
            post_reference,
            actor,
        })
    }

    /// EFFECT, permanent and server-verified-authoritative. The action returns
    /// void, so only observation (token no longer resolving) can confirm it.
    /// Any lost/ambiguous response is UNCERTAIN; this adapter never retries.
    pub async fn invoke_delete_leaflet(&self, preparation: &DeletePreparation, observed: &crate::LeafletData) -> Result<(), Error> {
        if self.base_url() != preparation.base || self.session != preparation.session || !self.has_cookie || observed.metadata.id != preparation.token {
            return Err(Error::Native("delete preparation belongs to another origin/session/token"));
        }
        let root = observed
            .metadata
            .root_entity
            .as_ref()
            .filter(|root| !root.is_empty())
            .ok_or(Error::Native("observed token lacks a root entity; refusing to serialize"))?;
        let rights = observed
            .raw
            .get("permission_token_rights")
            .filter(|rights| rights.is_array())
            .ok_or(Error::Native("observed token rights are not a list; refusing to serialize"))?;
        let body = json!([{ "id": observed.metadata.id, "root_entity": root, "permission_token_rights": rights }]).to_string();
        let response = self
            .http
            .post(preparation.page.clone())
            .header("Next-Action", &preparation.leaflet_reference)
            .header("Next-Router-State-Tree", r#"["",{}]"#)
            .header("Origin", self.base_url())
            .header("Accept", "text/x-component")
            .header("Content-Type", "text/plain;charset=UTF-8")
            .body(body)
            .send()
            .await
            .map_err(transport)?;
        let body = read_action_response(response, "POST deleteLeaflet").await?;
        decode_void(&body).map_err(|error| match error {
            Error::Native(reason) => Error::ActionResponse {
                reason,
                response: actions::ActionResponse(body),
            },
            other => other,
        })
    }

    /// EFFECT for a PUBLISHED document: delete the public PDS record (the
    /// server attempts BOTH document collections and swallows per-collection
    /// errors), the `documents` rows and publication linkage. Typed failures
    /// (`oauth_session_expired`, `not_authorized`) are clean refusals that
    /// precede the record effects in inspected native branches; anything else
    /// is uncertain and must be resolved by observing the record.
    pub async fn invoke_delete_post(&self, preparation: &DeletePreparation, document_uri: &str) -> Result<PostOutcome, Error> {
        let reference = preparation.post_reference.as_deref().ok_or(Error::Native("deletePost action unavailable on this origin; published deletion unsupported"))?;
        if self.base_url() != preparation.base || self.session != preparation.session || !self.has_cookie {
            return Err(Error::Native("delete preparation belongs to another origin/session"));
        }
        validate_document_uri(document_uri)?;
        let response = self
            .http
            .post(preparation.page.clone())
            .header("Next-Action", reference)
            .header("Next-Router-State-Tree", r#"["",{}]"#)
            .header("Origin", self.base_url())
            .header("Accept", "text/x-component")
            .header("Content-Type", "text/plain;charset=UTF-8")
            .body(json!([document_uri]).to_string())
            .send()
            .await
            .map_err(transport)?;
        let body = read_action_response(response, "POST deletePost").await?;
        decode_post(&body).map_err(|error| match error {
            Error::Native(reason) => Error::ActionResponse {
                reason,
                response: actions::ActionResponse(body),
            },
            other => other,
        })
    }
}

async fn read_action_response(response: reqwest::Response, operation: &'static str) -> Result<String, Error> {
    if response.headers().contains_key("x-action-redirect") || response.headers().get("x-nextjs-action-not-found").is_some_and(|v| v == "1") {
        return Err(Error::Native("delete action redirected/not found; outcome uncertain"));
    }
    if !response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/x-component"))
    {
        return Err(Error::Native("delete action did not return Flight; outcome uncertain"));
    }
    actions::bounded_text(response, operation, 1024 * 1024, &mut (1024 * 1024)).await
}

/// Void actions resolve their slot to Flight's `undefined` token; `null` is
/// accepted as the same no-value result. Anything else is an unsupported
/// (uncertain) shape, never a fabricated success.
pub fn decode_void(body: &str) -> Result<(), Error> {
    match actions::action_result(body)? {
        Value::Null => Ok(()),
        Value::String(token) if token == "$undefined" => Ok(()),
        _ => Err(Error::Native("delete action returned a value; unsupported shape")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostFailureKind {
    OauthSessionExpired,
    NotAuthorized,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostFailure {
    #[serde(rename = "type")]
    pub kind: PostFailureKind,
    pub message: String,
    pub did: Option<String>,
}
pub enum PostOutcome {
    Deleted,
    Failed(PostFailure),
}
impl std::fmt::Debug for PostOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Server error prose is not incidental diagnostic content.
        match self {
            Self::Deleted => f.write_str("PostOutcome::Deleted"),
            Self::Failed(error) => f.debug_tuple("PostOutcome::Failed").field(&error.kind).finish(),
        }
    }
}

/// Bounded literal/promise action slot for `deletePost`'s success/error union.
pub fn decode_post(body: &str) -> Result<PostOutcome, Error> {
    let mut value = actions::action_result(body)?;
    let success = value
        .as_object_mut()
        .and_then(|object| object.remove("success"))
        .ok_or(Error::Native("deletePost result missing success"))?;
    match success {
        Value::Bool(true) => {
            if !value.as_object().is_some_and(|object| object.is_empty()) {
                return Err(Error::Native("unsupported deletePost success shape"));
            }
            Ok(PostOutcome::Deleted)
        }
        Value::Bool(false) => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Envelope {
                error: PostFailure,
            }
            let envelope: Envelope = serde_json::from_value(value).map_err(|_| Error::Native("unsupported deletePost error shape"))?;
            Ok(PostOutcome::Failed(envelope.error))
        }
        _ => Err(Error::Native("invalid deletePost result discriminator")),
    }
}

/// Exact DID-based document record identity, mirroring the collections the
/// native action deletes from. No handle, query, fragment or foreign path.
fn validate_document_uri(value: &str) -> Result<(), Error> {
    let parts: Vec<_> = value
        .strip_prefix("at://")
        .ok_or(Error::Native("document must be a DID-based AT-URI"))?
        .split('/')
        .collect();
    if parts.len() != 3
        || !parts[0].starts_with("did:")
        || parts[0].len() > 2048
        || !parts[0].bytes().all(|b| b.is_ascii_alphanumeric() || b"-._:".contains(&b))
        || parts[0].split(':').count() < 3
        || parts[0].split(':').any(str::is_empty)
        || !matches!(parts[1], "site.standard.document" | "pub.leaflet.document")
        || parts[2].is_empty()
        || parts[2].len() > 512
        || matches!(parts[2], "." | "..")
        || !parts[2].bytes().all(|b| b.is_ascii_alphanumeric() || b"._:~-".contains(&b))
    {
        return Err(Error::Native("unsupported document AT-URI for native deletion"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
