//! Literal `publishToPublication` contract at Leaflet 600c7b5f. This is a
//! transport, NOT publication authorization, input verification or recovery.
//! The guarded workflow owns save-first execution, dispatch evidence and recovery.
use crate::{Error, LeafletClient, actions, transport};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

/// Only the initial no-email/no-Bluesky profile. There is intentionally no
/// caller rkey, path, actorDid, entitiesToDelete or delivery flag escape hatch.
#[derive(Clone)]
pub struct Arguments {
    pub root_entity: Uuid,
    pub leaflet_id: Uuid,
    pub publication_uri: Option<String>,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub published_at: Option<String>,
}
impl Arguments {
    pub fn wire(&self) -> Value {
        let mut value = json!({"root_entity":self.root_entity,"leaflet_id":self.leaflet_id,
            "title":self.title,"description":self.description,"tags":self.tags,"sendEmail":false});
        if let Some(publication) = &self.publication_uri {
            value["publication_uri"] = json!(publication);
        }
        if let Some(date) = &self.published_at {
            value["publishedAt"] = json!(date);
        }
        // Preferences/discovery are inherited by the real action. The workflow
        // must verify their supported values; omission is not a quiet publish.
        json!([value])
    }
}

/// Current-origin/session lookup only. Actor identity does not prove native
/// editor write rights, publication membership, or restored owner OAuth health.
pub struct Preparation {
    base: String,
    session: Uuid,
    token: Uuid,
    page: Url,
    reference: String,
    actor: String,
}
impl Preparation {
    pub fn actor(&self) -> &str {
        &self.actor
    }
}
impl std::fmt::Debug for Preparation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublicationPreparation").finish_non_exhaustive()
    }
}

impl LeafletClient {
    /// Read-only authenticated identity + same-origin /<token>/publish chunks.
    /// No saved action hash, deployment probing, dashboard retarget or mint.
    pub async fn prepare_publication(&self, token: Uuid) -> Result<Preparation, Error> {
        if !self.has_cookie {
            return Err(Error::Native("publication requires cookie authentication"));
        }
        let response = self.http.get(self.base.join("home").expect("static path")).send().await.map_err(transport)?;
        let html = actions::bounded_text(response, "GET /home before publication", 4 * 1024 * 1024, &mut (4 * 1024 * 1024)).await?;
        let actor = crate::parse_home(&html)?
            .identity
            .did
            .ok_or(Error::Native("publication requires an authenticated DID"))?;
        let page = self.base.join(&format!("{token}/publish")).expect("UUID path");
        let reference = self.discover_action(&page, "publishToPublication").await?;
        Ok(Preparation {
            base: self.base_url().into(),
            session: self.session,
            token,
            page,
            reference,
            actor,
        })
    }

    /// EFFECT. Caller MUST persist dispatched uncertainty before polling this,
    /// verify the complete saved input and all native side effects, and observe
    /// output/linkage afterward. No automatic retries, even on publish_failed.
    /// Production callers compose this through the guarded publication workflow.
    pub async fn invoke_publication(&self, preparation: &Preparation, arguments: &Arguments) -> Result<Outcome, Error> {
        if self.base_url() != preparation.base || self.session != preparation.session || !self.has_cookie || arguments.leaflet_id != preparation.token {
            return Err(Error::Native("publication preparation belongs to another origin/session/token"));
        }
        let response = self
            .http
            .post(preparation.page.clone())
            .header("Next-Action", &preparation.reference)
            .header("Next-Router-State-Tree", r#"["",{}]"#)
            .header("Origin", self.base_url())
            .header("Accept", "text/x-component")
            .header("Content-Type", "text/plain;charset=UTF-8")
            .body(arguments.wire().to_string())
            .send()
            .await
            .map_err(transport)?;
        if response.headers().contains_key("x-action-redirect") || response.headers().get("x-nextjs-action-not-found").is_some_and(|v| v == "1") {
            return Err(Error::Native("publication action redirected/not found; outcome uncertain"));
        }
        if !response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("text/x-component"))
        {
            return Err(Error::Native("publication action did not return Flight; outcome uncertain"));
        }
        let body = actions::bounded_text(response, "POST publishToPublication", 1024 * 1024, &mut (1024 * 1024)).await?;
        decode(&body).map_err(|error| match error {
            Error::Native(reason) => Error::ActionResponse {
                reason,
                response: actions::ActionResponse(body),
            },
            other => other,
        })
    }
}

/// Return value is merely a server report. It contains NO output CID, and its
/// inflated `record` can differ from the PDS's offloaded/truncated record.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Published {
    pub rkey: String,
    pub uri: String,
    pub record: Value,
    pub first_publish: bool,
    pub blocks: u64,
    pub reply: Option<Reply>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub uri: String,
    pub subject: String,
    pub document: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    OauthSessionExpired,
    PublishInProgress,
    PublishFailed,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    #[serde(rename = "type")]
    pub kind: FailureKind,
    pub message: String,
    pub did: Option<String>,
}
pub enum Outcome {
    Published(Published),
    Failed(Failure),
}
impl std::fmt::Debug for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Neither arbitrary server error prose nor returned content/capability
        // identifiers belong in incidental diagnostics.
        match self {
            Self::Published(_) => f.write_str("PublicationOutcome::Published(unverified)"),
            Self::Failed(error) => f.debug_tuple("PublicationOutcome::Failed").field(&error.kind).finish(),
        }
    }
}

/// Bounded literal/promise action slot, not a scrape for an unrelated URI.
pub fn decode(body: &str) -> Result<Outcome, Error> {
    let mut value = actions::action_result(body)?;
    let success = value
        .as_object_mut()
        .and_then(|object| object.remove("success"))
        .ok_or(Error::Native("publication result missing success"))?;
    match success {
        Value::Bool(true) => {
            // The actual native result includes `reply: undefined` when no
            // reply was submitted. Flight serializes that optional field as
            // "$undefined". Normalize ONLY this known optional result slot;
            // authored record strings and unknown fields remain untouched.
            if value.get("reply").is_some_and(|reply| reply == "$undefined") {
                value.as_object_mut().expect("success object").remove("reply");
            }
            let result: Published = serde_json::from_value(value).map_err(|_| Error::Native("unsupported publication success shape"))?;
            if result.rkey.is_empty() || result.uri.is_empty() || !result.record.is_object() {
                return Err(Error::Native("incomplete publication success"));
            }
            Ok(Outcome::Published(result))
        }
        Value::Bool(false) => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Envelope {
                error: Failure,
            }
            let envelope: Envelope = serde_json::from_value(value).map_err(|_| Error::Native("unsupported publication error shape"))?;
            if envelope.error.kind == FailureKind::OauthSessionExpired && envelope.error.did.is_none() {
                return Err(Error::Native("OAuth publication failure missing DID"));
            }
            Ok(Outcome::Failed(envelope.error))
        }
        _ => Err(Error::Native("invalid publication result discriminator")),
    }
}

#[cfg(test)]
mod tests;
