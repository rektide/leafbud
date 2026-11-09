//! Small read-only remote workflows. No local document lifecycle yet.
//!
//! Native credentials remain separate from restored TOML authoring defaults.

pub mod config;
pub mod diff;
pub mod editor_state;
mod files;
pub mod import;
mod inventory;
pub mod pull;
pub mod status;
pub mod tidy;

use std::path::PathBuf;

use leafbud_web::{DEFAULT_BASE_URL, LeafletClient};
pub use leafbud_web::{DiscoveredLeaflet, Identity, LeafletData, LeafletState};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Web(#[from] leafbud_web::Error),
    #[error("cannot locate auth-token file: set XDG_CONFIG_HOME or HOME, or supply --token/LEAFLET_AUTH_TOKEN")]
    NoConfigHome,
    #[error("{0} is not valid UTF-8")]
    Environment(&'static str),
    #[error("auth token from {0} is empty")]
    EmptyToken(&'static str),
    #[error("cannot read leafbud auth-token file")]
    TokenFile(#[source] std::io::Error),
}

/// The token is secret: neither this options value nor the client prints it.
#[derive(Default)]
pub struct RemoteOptions {
    pub base_url: Option<String>,
    pub token: Option<String>,
}

impl std::fmt::Debug for RemoteOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteOptions")
            .field("token_supplied", &self.token.is_some())
            .finish_non_exhaustive()
    }
}

impl RemoteOptions {
    /// Resolve only what the two remote reads need. Explicit values prevent
    /// reading lower-priority credential sources (including unreadable files).
    pub async fn connect(self) -> Result<RemoteReader, Error> {
        let base_url = match self.base_url {
            Some(url) => url,
            None => env_text("LEAFLET_BASE_URL")?.unwrap_or_else(|| DEFAULT_BASE_URL.into()),
        };
        let token = match self.token {
            Some(token) => Some(clean_token(token, "--token")?),
            None => match env_text("LEAFLET_AUTH_TOKEN")? {
                Some(token) => Some(clean_token(token, "LEAFLET_AUTH_TOKEN")?),
                None => {
                    let dir = config_home(std::env::var_os("XDG_CONFIG_HOME"), std::env::var_os("HOME"))?;
                    read_token_file(dir.join("leafbud/auth-token")).await?
                }
            },
        };
        Ok(RemoteReader::new(LeafletClient::new(&base_url, token.as_deref())?))
    }
}

fn env_text(name: &'static str) -> Result<Option<String>, Error> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(Error::Environment(name)),
    }
}

pub(crate) fn config_home(xdg: Option<std::ffi::OsString>, home: Option<std::ffi::OsString>) -> Result<PathBuf, Error> {
    if let Some(xdg) = xdg.filter(|v| !v.is_empty()).map(PathBuf::from).filter(|p| p.is_absolute()) {
        return Ok(xdg);
    }
    home.filter(|v| !v.is_empty())
        .map(|h| PathBuf::from(h).join(".config"))
        .ok_or(Error::NoConfigHome)
}

fn clean_token(token: String, source: &'static str) -> Result<String, Error> {
    let token = token.trim();
    if token.is_empty() {
        return Err(Error::EmptyToken(source));
    }
    Ok(token.into())
}

async fn read_token_file(path: PathBuf) -> Result<Option<String>, Error> {
    match tokio::fs::read_to_string(path).await {
        Ok(token) => Ok(Some(clean_token(token, "XDG auth-token file")?)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(Error::TokenFile(err)),
    }
}

#[derive(Debug, Clone)]
pub struct RemoteReader {
    client: LeafletClient,
}

impl RemoteReader {
    /// Explicit-client entrypoint for library callers without environment or IO.
    pub fn new(client: LeafletClient) -> Self {
        Self { client }
    }

    pub async fn list(&self, state: Option<LeafletState>) -> Result<RemoteList, Error> {
        Ok(select_discovery(self.client.list().await?, state))
    }

    pub async fn show(&self, locator: &str) -> Result<RemoteShow, Error> {
        let id = self.client.resolve_locator(locator)?;
        let data = self.client.get(id).await?;
        Ok(RemoteShow {
            version: 1,
            outcome: "observed",
            editor_url: self.client.editor_url(id),
            state: data.observed_state(),
            data,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Coverage {
    pub scope: &'static str,
    pub completeness: &'static str,
    pub limitation: &'static str,
}

#[derive(Debug, Serialize)]
pub struct RemoteList {
    pub version: u8,
    pub outcome: &'static str,
    pub source: &'static str,
    pub identity: Identity,
    pub fetched_at: Option<i64>,
    pub coverage: Coverage,
    pub state_filter: Option<LeafletState>,
    pub discovered_count: usize,
    pub count: usize,
    pub href_payload_mismatch: Vec<String>,
    pub leaflets: Vec<RemoteListItem>,
}

#[derive(Debug, Serialize)]
pub struct RemoteListItem {
    pub outcome: &'static str,
    pub state: LeafletState,
    #[serde(flatten)]
    pub leaflet: DiscoveredLeaflet,
}

#[derive(Debug, Serialize)]
pub struct RemoteShow {
    pub version: u8,
    pub outcome: &'static str,
    pub editor_url: String,
    pub state: LeafletState,
    pub data: LeafletData,
}

fn select_discovery(discovery: leafbud_web::Discovery, state: Option<LeafletState>) -> RemoteList {
    let discovered_count = discovery.leaflets.len();
    let leaflets: Vec<_> = discovery
        .leaflets
        .into_iter()
        .filter(|l| state.is_none_or(|s| l.metadata.state() == s))
        .map(|leaflet| RemoteListItem {
            outcome: "observed",
            state: leaflet.metadata.state(),
            leaflet,
        })
        .collect();
    RemoteList {
        version: 1,
        outcome: "observed",
        source: "home-rsc",
        identity: discovery.identity,
        fetched_at: discovery.fetched_at,
        coverage: Coverage {
            scope: "homepage-and-contributor-payload",
            completeness: "unknown",
            limitation: "Only the observed /home payload; account-wide completeness and pagination are unverified.",
        },
        state_filter: state,
        discovered_count,
        count: leaflets.len(),
        href_payload_mismatch: discovery.href_payload_mismatch,
        leaflets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_is_exclusive_and_keeps_discovery_coverage() {
        let fixture = include_str!("../../../experiments/01-draft-list/fixture-home.html");
        for (filter, count) in [(None, 5), (Some(LeafletState::Draft), 3), (Some(LeafletState::Published), 2)] {
            let list = select_discovery(leafbud_web::parse_home(fixture).unwrap(), filter);
            assert_eq!(list.count, count);
            assert_eq!(list.discovered_count, 5);
            assert_eq!(list.coverage.completeness, "unknown");
            assert!(list.leaflets.iter().all(|l| filter.is_none_or(|s| l.state == s)));
            assert_eq!(serde_json::to_value(list).unwrap()["version"], 1);
        }
    }

    #[test]
    fn xdg_paths_and_secret_diagnostics() {
        assert_eq!(
            config_home(Some("/config".into()), Some("/home/example".into())).unwrap(),
            PathBuf::from("/config")
        );
        assert_eq!(
            config_home(Some("relative".into()), Some("/home/example".into())).unwrap(),
            PathBuf::from("/home/example/.config")
        );
        assert!(matches!(config_home(None, None), Err(Error::NoConfigHome)));
        assert_eq!(clean_token(" secret\n".into(), "test").unwrap(), "secret");
        assert!(clean_token("\n".into(), "test").is_err());
        let options = RemoteOptions {
            base_url: Some("https://user:secret@host".into()),
            token: Some("secret".into()),
        };
        assert!(!format!("{options:?}").contains("secret"));
    }
}
