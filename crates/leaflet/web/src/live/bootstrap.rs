//! Deliberately small public Next-asset discovery, not a JavaScript interpreter.
use std::{collections::BTreeSet, fmt, net::IpAddr, time::Duration};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use regex::Regex;
use reqwest::Url;
use serde_json::Value;

use super::Error;

const MAX_SCRIPTS: usize = 48;
const MAX_PAGE: usize = 2 * 1024 * 1024;
const MAX_ASSET: usize = 8 * 1024 * 1024;
const MAX_TOTAL: usize = 32 * 1024 * 1024;
const MAX_CONFIG_VALUES: usize = 64;

/// An explicit public project override using the verified anon-JWT join path.
/// Private/service-role JWTs and opaque keys are rejected, not sent as JWTs.
#[derive(Clone)]
pub struct PublicProject {
    origin: Url,
    pub(super) key: String,
}

impl fmt::Debug for PublicProject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PublicProject([redacted])")
    }
}

impl PublicProject {
    pub fn new(project_origin: &str, public_key: &str) -> Result<Self, Error> {
        let origin = origin(project_origin)?;
        if public_key.len() > 8192 || public_key.is_empty() || !public_key.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)) {
            return Err(Error::PublicKey);
        }
        let claims = anon_claims(public_key).ok_or(Error::PublicKey)?;
        if let Some(reference) = project_ref(&origin)
            && claims["ref"].as_str() != Some(reference)
        {
            return Err(Error::ProjectMismatch);
        }
        Ok(Self {
            origin,
            key: public_key.to_owned(),
        })
    }
}

/// A public project bound to one validated Leaflet origin. No HTTP auth is held.
#[derive(Clone)]
pub struct Bootstrap {
    leaflet: Url,
    pub(super) project: PublicProject,
}

impl fmt::Debug for Bootstrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Bootstrap([origin and project redacted])")
    }
}

impl Bootstrap {
    /// Explicit configuration is an operator selection, not a discovered fallback.
    pub fn explicit(leaflet_origin: &str, project: PublicProject) -> Result<Self, Error> {
        Ok(Self {
            leaflet: origin(leaflet_origin)?,
            project,
        })
    }

    pub async fn resolve(leaflet_origin: &str, project: Option<PublicProject>) -> Result<Self, Error> {
        if let Some(project) = project {
            return Self::explicit(leaflet_origin, project);
        }
        let leaflet = origin(leaflet_origin)?;
        tokio::time::timeout(Duration::from_secs(90), discover(leaflet))
            .await
            .map_err(|_| Error::BootstrapTimeout)?
    }

    /// Call when reusing a bootstrap with a configured HTTP reader. No cross-origin
    /// cache fallback is permitted; a mismatch requires fresh resolution.
    pub fn require_origin(&self, leaflet_origin: &str) -> Result<(), Error> {
        if self.leaflet.origin() != origin(leaflet_origin)?.origin() {
            return Err(Error::OriginMismatch);
        }
        Ok(())
    }

    pub(super) fn endpoint(&self) -> String {
        let mut endpoint = self.project.origin.clone();
        endpoint
            .set_scheme(if self.project.origin.scheme() == "https" { "wss" } else { "ws" })
            .expect("validated HTTP origin");
        endpoint.set_path("/realtime/v1/websocket");
        endpoint.query_pairs_mut().append_pair("apikey", &self.project.key).append_pair("vsn", "2.0.0");
        endpoint.into()
    }
}

fn origin(input: &str) -> Result<Url, Error> {
    let url = Url::parse(input).map_err(|_| Error::Origin)?;
    let loopback = url
        .host_str()
        .is_some_and(|host| host == "localhost" || host.trim_matches(['[', ']']).parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback()));
    if !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(Error::Origin);
    }
    Ok(url)
}

fn project_ref(url: &Url) -> Option<&str> {
    url.host_str()?
        .strip_suffix(".supabase.co")
        .filter(|reference| !reference.is_empty() && !reference.contains('.'))
}

// Discovery filtering only. This is NOT JWT signature verification or permission.
fn anon_claims(key: &str) -> Option<Value> {
    let parts = key.split('.').collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        return None;
    }
    let claims: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).ok()?).ok()?;
    (claims["role"] == "anon").then_some(claims)
}

async fn discover(leaflet: Url) -> Result<Bootstrap, Error> {
    // Never reuse LeafletClient.http: its default Cookie belongs only to Leaflet.
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("leafbud/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| Error::BootstrapTransport)?;
    let mut remaining = MAX_TOTAL;
    let page = get(&http, leaflet.clone(), MAX_PAGE, &mut remaining, true).await?;
    let scripts = scripts(&leaflet, &page)?;
    let mut urls = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let url_pattern = Regex::new(r"https://[a-zA-Z0-9-]+\.supabase\.co\b").expect("static regex");
    let key_pattern = Regex::new(r"eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").expect("static regex");
    for asset in scripts {
        let text = get(&http, asset, MAX_ASSET, &mut remaining, false).await?;
        for matched in url_pattern.find_iter(&text) {
            if text
                .as_bytes()
                .get(matched.end())
                .is_some_and(|b| b.is_ascii_alphanumeric() || b".-_".contains(b))
            {
                continue;
            }
            urls.insert(matched.as_str().to_owned());
            if urls.len() > MAX_CONFIG_VALUES {
                return Err(Error::BootstrapLimit);
            }
        }
        for matched in key_pattern.find_iter(&text) {
            if matched.len() <= 8192 {
                keys.insert(matched.as_str().to_owned());
            }
            if keys.len() > MAX_CONFIG_VALUES {
                return Err(Error::BootstrapLimit);
            }
        }
    }
    let mut pairs = Vec::new();
    for url in urls {
        let parsed = origin(&url)?;
        for key in &keys {
            if anon_claims(key).is_some_and(|claims| claims["ref"].as_str() == project_ref(&parsed)) {
                pairs.push(PublicProject::new(&url, key)?);
            }
        }
    }
    if pairs.len() != 1 {
        return Err(if pairs.is_empty() {
            Error::DiscoveryMissing
        } else {
            Error::DiscoveryAmbiguous
        });
    }
    Ok(Bootstrap {
        leaflet,
        project: pairs.remove(0),
    })
}

fn scripts(base: &Url, html: &str) -> Result<BTreeSet<Url>, Error> {
    let tags = Regex::new(r"(?is)<script\b[^>]*>").expect("static regex");
    let src = Regex::new(r#"(?i)\bsrc\s*=\s*["']([^"']+)["']"#).expect("static regex");
    let mut scripts = BTreeSet::new();
    let mut count = 0;
    for tag in tags.find_iter(html) {
        let Some(capture) = src.captures(tag.as_str()) else { continue };
        count += 1;
        if count > MAX_SCRIPTS {
            return Err(Error::BootstrapLimit);
        }
        let path = capture[1].replace("&amp;", "&");
        if path.len() > 2048 {
            return Err(Error::BootstrapLimit);
        }
        let url = base.join(&path).map_err(|_| Error::BootstrapShape)?;
        if url.origin() == base.origin()
            && url.username().is_empty()
            && url.password().is_none()
            && url.fragment().is_none()
            && url.path().starts_with("/_next/static/")
            && url.path().ends_with(".js")
        {
            scripts.insert(url);
        }
    }
    Ok(scripts)
}

async fn get(http: &reqwest::Client, url: Url, limit: usize, remaining: &mut usize, page: bool) -> Result<String, Error> {
    let mut response = http.get(url).send().await.map_err(|_| Error::BootstrapTransport)?;
    if !response.status().is_success() {
        return Err(Error::BootstrapHttp(response.status().as_u16()));
    }
    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim();
    if if page {
        mime != "text/html"
    } else {
        !matches!(mime, "application/javascript" | "text/javascript" | "application/x-javascript")
    } {
        return Err(Error::BootstrapShape);
    }
    if response.content_length().is_some_and(|length| length > limit.min(*remaining) as u64) {
        return Err(Error::BootstrapLimit);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| Error::BootstrapTransport)? {
        if chunk.len() > limit.saturating_sub(bytes.len()).min(*remaining) {
            return Err(Error::BootstrapLimit);
        }
        *remaining -= chunk.len();
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| Error::BootstrapShape)
}

#[cfg(test)]
pub(super) fn test_key(reference: &str) -> String {
    format!(
        "eyJhbGciOiJIUzI1NiJ9.{}.synthetic",
        URL_SAFE_NO_PAD.encode(serde_json::json!({"role":"anon","ref":reference}).to_string())
    )
}
