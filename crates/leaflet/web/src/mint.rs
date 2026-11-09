use crate::{Error, LeafletClient, actions, transport};
use reqwest::Url;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MintTarget {
    Standalone,
    Publication(String),
}

/// Authenticated, current-origin discovery result. No arbitrary action ID or
/// endpoint injection. Preparing is read-only; invoking `mint` is an effect.
pub struct MintPreparation {
    base: String,
    session: Uuid,
    target: MintTarget,
    action: Option<(Url, String)>,
}
impl std::fmt::Debug for MintPreparation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MintPreparation")
            .field("publication", &self.action.is_some())
            .finish_non_exhaustive()
    }
}
impl MintPreparation {
    pub fn target(&self) -> &MintTarget {
        &self.target
    }
}

impl LeafletClient {
    pub async fn prepare_mint(&self, target: MintTarget) -> Result<MintPreparation, Error> {
        if !self.has_cookie {
            return Err(Error::Native("mint requires cookie authentication"));
        }
        // Authenticated identity payload, not merely a supplied cookie. Do not
        // turn an inaccessible dashboard into a standalone document.
        let response = self.http.get(self.base.join("home").expect("static path")).send().await.map_err(transport)?;
        let html = actions::bounded_text(response, "GET /home before mint", 4 * 1024 * 1024, &mut (4 * 1024 * 1024)).await?;
        let identity = crate::parse_home(&html)?.identity;
        let action = match &target {
            MintTarget::Standalone => None,
            MintTarget::Publication(uri) => {
                if identity.did.is_none() {
                    return Err(Error::Native("publication mint requires an authenticated DID"));
                }
                let dashboard = dashboard(&self.base, uri)?;
                let reference = self.discover_mint_action(&dashboard).await?;
                Some((dashboard, reference))
            }
        };
        Ok(MintPreparation {
            base: self.base_url().into(),
            session: self.session,
            target,
            action,
        })
    }

    /// Effect: caller MUST persist a dispatched attempt before polling this.
    /// Any error/lost response can leave a real draft; never auto-call again.
    pub async fn mint(&self, preparation: &MintPreparation) -> Result<Option<Uuid>, Error> {
        if self.base_url() != preparation.base || self.session != preparation.session || !self.has_cookie {
            return Err(Error::Native("mint preparation belongs to another origin/session"));
        }
        if let Some((dashboard, reference)) = &preparation.action {
            let MintTarget::Publication(uri) = &preparation.target else {
                return Err(Error::Native("incoherent publication mint preparation"));
            };
            let response = self
                .http
                .post(dashboard.clone())
                .header("Next-Action", reference)
                .header("Next-Router-State-Tree", r#"["",{}]"#)
                .header("Origin", self.base_url())
                .header("Accept", "text/x-component")
                .header("Content-Type", "text/plain;charset=UTF-8")
                .body(serde_json::json!([uri, "doc", {}]).to_string())
                .send()
                .await
                .map_err(transport)?;
            if response.headers().contains_key("x-action-redirect") || response.headers().get("x-nextjs-action-not-found").is_some_and(|v| v == "1") {
                return Err(Error::Native("mint action redirected/not found; outcome uncertain"));
            }
            if !response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with("text/x-component"))
            {
                return Err(Error::Native("mint action did not return Flight; outcome uncertain"));
            }
            let body = actions::bounded_text(response, "POST createPublicationDraft", 1024 * 1024, &mut (1024 * 1024)).await?;
            actions::mint_result(&body)
        } else {
            // GET /new creates a document. Redirects are intentionally NOT
            // followed, including a forged foreign-origin Location.
            let response = self.http.get(self.base.join("new").expect("static path")).send().await.map_err(transport)?;
            if !matches!(response.status().as_u16(), 303 | 307) {
                return Err(Error::Native("standalone mint missing native redirect; outcome uncertain"));
            }
            let location = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or(Error::Native("standalone mint has no locator"))?;
            let url = self.base.join(location).map_err(|_| Error::Native("invalid mint redirect"))?;
            self.resolve_locator(url.as_str()).map(Some)
        }
    }
}

/// Deliberately narrow native dashboard context: DID-based publication record
/// only, no query/fragment/percent-encoded path injection or mutable handles.
fn dashboard(base: &Url, uri: &str) -> Result<Url, Error> {
    let parts: Vec<_> = uri
        .strip_prefix("at://")
        .ok_or(Error::Native("publication must be a DID-based AT-URI"))?
        .split('/')
        .collect();
    if parts.len() != 3
        || !matches!(parts[1], "site.standard.publication" | "pub.leaflet.publication")
        || !parts[0].starts_with("did:")
        || parts[0].len() > 2048
        || !parts[0].bytes().all(|b| b.is_ascii_alphanumeric() || b"-._:".contains(&b))
        || parts[0].split(':').count() < 3
        || parts[0].split(':').any(str::is_empty)
        || parts[2].is_empty()
        || parts[2].len() > 512
        || matches!(parts[2], "." | "..")
        || !parts[2].bytes().all(|b| b.is_ascii_alphanumeric() || b"._:~-".contains(&b))
    {
        return Err(Error::Native("unsupported publication AT-URI for native dashboard"));
    }
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|_| Error::Native("invalid base"))?
        .clear()
        .extend(["lish", parts[0], parts[2], "dashboard"]);
    Ok(url)
}
