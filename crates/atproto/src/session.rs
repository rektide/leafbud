//! Session persistence and the resume-or-create flow.
//!
//! `SessionStore` persists an app-password session to
//! `$XDG_CONFIG_HOME/leafspring/session.json` (else `~/.config/leafspring/`)
//! with private permissions: the file 0600, the parent directory 0700
//! (directories need the execute bit to be traversable — 0700 is the
//! directory analog of a 0600 file). JWTs are secrets: `Debug` redacts
//! them and nothing here logs, prints, or embeds them in errors.
//!
//! [`resume_or_create_session`] is the publish flow's entry point: a
//! stored session whose `accessJwt` is still valid (JWT `exp` claim,
//! 60-second margin) is reused; an expired one is refreshed via
//! `refreshSession` against the PDS that issued it; anything else falls
//! back to `createSession` with the app password (which is used once and
//! dropped). `exp` parsing is scheduling only — no signature is verified.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::xrpc::{Session, XrpcClient};

/// How many seconds before its `exp` an access JWT is treated as expired
/// (avoids racing a put through a token that lapses mid-flight).
const ACCESS_MARGIN_SECS: i64 = 60;

/// A persisted session. JWT fields are secrets — `Debug` redacts them.
#[derive(Serialize, Deserialize, Clone)]
pub struct PersistedSession {
    pub did: String,
    pub handle: String,
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
    /// The PDS base URL that issued the session (refreshes must return
    /// to the same endpoint).
    pub pds: String,
    /// Unix seconds, informational.
    pub created_at: i64,
}

impl std::fmt::Debug for PersistedSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistedSession")
            .field("did", &self.did)
            .field("handle", &self.handle)
            .field("pds", &self.pds)
            .field("created_at", &self.created_at)
            .finish_non_exhaustive() // never the JWTs
    }
}

impl PersistedSession {
    fn into_session(self) -> Session {
        Session {
            access_jwt: self.access_jwt,
            refresh_jwt: self.refresh_jwt,
            did: self.did,
            handle: self.handle,
        }
    }
}

/// The JWT `exp` claim (unix seconds), if present and parseable.
/// Scheduling only — the signature is not verified.
pub fn jwt_exp(token: &str) -> Option<i64> {
    use base64::engine::Engine as _;
    let payload = token.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    match value.get("exp") {
        Some(serde_json::Value::Number(n)) => n.as_i64(),
        Some(serde_json::Value::String(s)) => s.parse().ok(),
        _ => None,
    }
}

/// Whether a JWT's `exp` (if any) is still at least `margin` seconds away.
fn jwt_live(token: &str, now: i64, margin: i64) -> bool {
    jwt_exp(token).is_none_or(|exp| exp > now.saturating_add(margin))
}

/// File-backed session store. One JSON file, private permissions.
#[derive(Debug, Clone)]
pub struct SessionStore {
    path: PathBuf,
}

impl SessionStore {
    /// A store at an explicit path (tests).
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The default store: `$XDG_CONFIG_HOME/leafspring/session.json`,
    /// else `~/.config/leafspring/session.json`.
    pub fn xdg() -> Result<Self, Error> {
        let dir = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(xdg) if !xdg.is_empty() => PathBuf::from(xdg),
            _ => {
                let home = std::env::var_os("HOME").ok_or_else(|| Error::Session("session store: $HOME unset and no $XDG_CONFIG_HOME".into()))?;
                PathBuf::from(home).join(".config")
            }
        };
        Ok(Self::at(dir.join("leafspring").join("session.json")))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load the persisted session, if one exists. A corrupt file is an
    /// error (not silently discarded): the operator removes it on purpose.
    pub fn load(&self) -> Result<Option<PersistedSession>, Error> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => {
                let session: PersistedSession = serde_json::from_str(&text).map_err(|e| {
                    Error::Session(format!(
                        "session store {}: decode failed: {e} (remove the file to re-auth)",
                        self.path.display()
                    ))
                })?;
                Ok(Some(session))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(Error::Session(format!("session store {}: {err}", self.path.display()))),
        }
    }

    /// Persist a session: parent directories created 0700, the file
    /// written 0600 (temp file + rename so no partially-written store).
    pub fn save(&self, session: &PersistedSession) -> Result<(), Error> {
        use std::os::unix::fs::PermissionsExt;
        let parent = self.path.parent().unwrap_or(Path::new("."));
        match std::fs::create_dir_all(parent) {
            Ok(()) => inherit_or_restrict(parent)?, // created or already there — always re-assert 0700
            Err(err) => return Err(Error::Session(format!("session store: creating {}: {err}", parent.display()))),
        }
        let mut tmp = self.path.clone().into_os_string();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);
        let body = serde_json::to_vec_pretty(session).map_err(|e| Error::Session(format!("session store: encode failed: {e}")))?;
        std::fs::write(&tmp, &body)
            .and_then(|_| {
                // Restrict before the rename so the file is never world-readable at rest.
                std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))
            })
            .map_err(|err| Error::Session(format!("session store: writing {}: {err}", tmp.display())))?;
        std::fs::rename(&tmp, &self.path).map_err(|err| Error::Session(format!("session store: renaming into {}: {err}", self.path.display())))?;
        Ok(())
    }
}

/// For each existing ancestor of the store (up to and including the
/// leafspring dir), assert the private 0700 mode. Only rewrites modes we
/// created this call; existing directories that are already private are
/// left alone, group/other-readable ones are tightened (the store is a
/// secret — better loud than leaky).
fn inherit_or_restrict(dir: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(dir).map_err(|err| Error::Session(format!("session store: stat {}: {err}", dir.display())))?;
    let mode = meta.permissions().mode();
    if mode & 0o077 != 0 {
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(mode & 0o700))
            .map_err(|err| Error::Session(format!("session store: tightening {} to private: {err}", dir.display())))?;
    }
    Ok(())
}

/// Resume a session or create a fresh one (the publish flow's only auth
/// surface). Order:
///
/// 1. stored session for this identifier with a live `accessJwt` → reuse;
/// 2. expired access but live `refreshJwt` → `refreshSession` at the
///    issuing PDS, persist, use; a failed refresh falls through to 3;
/// 3. `createSession` with the app password at `auth_pds`, persist, use.
///
/// `auth_pds` defaults to `https://bsky.social` at the call site (the
/// social-cli convention); the stored `pds` field keeps refreshes bound
/// to the actual issuer.
pub async fn resume_or_create_session(store: &SessionStore, identifier: &str, app_password: &str, auth_pds: &str) -> Result<Session, Error> {
    let now = unix_now();
    if let Ok(Some(persisted)) = store.load()
        && (persisted.handle == identifier || persisted.did == identifier)
    {
        if jwt_live(&persisted.access_jwt, now, ACCESS_MARGIN_SECS) {
            tracing::debug!(did = %persisted.did, "reusing stored session");
            return Ok(persisted.into_session());
        }
        if jwt_live(&persisted.refresh_jwt, now, 0) {
            let issuer = XrpcClient::new(&persisted.pds)?;
            match issuer.refresh_session(&persisted.refresh_jwt).await {
                Ok(fresh) => {
                    store.save(&persisted_with(&fresh, &persisted.pds, now))?;
                    return Ok(fresh);
                }
                Err(err) => tracing::warn!("refresh failed ({err}); falling back to createSession"),
            }
        }
    }
    let client = XrpcClient::new(auth_pds)?;
    let fresh = client.create_session(identifier, app_password).await?;
    store.save(&persisted_with(&fresh, auth_pds, now))?;
    Ok(fresh)
}

fn persisted_with(session: &Session, pds: &str, now: i64) -> PersistedSession {
    PersistedSession {
        did: session.did.clone(),
        handle: session.handle.clone(),
        access_jwt: session.access_jwt.clone(),
        refresh_jwt: session.refresh_jwt.clone(),
        pds: pds.to_string(),
        created_at: now,
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("leafspring-atproto-session-{label}-{}-{}", std::process::id(), uuid_counter()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn uuid_counter() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        N.fetch_add(1, Ordering::Relaxed)
    }

    fn sample(pds: &str, exp_access: i64, exp_refresh: i64) -> PersistedSession {
        let mk = |exp: i64, sub: &str| {
            use base64::engine::Engine as _;
            let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256"}"#);
            let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!(r#"{{"sub":"{sub}","exp":{exp}}}"#));
            format!("{header}.{payload}.sig")
        };
        PersistedSession {
            did: "did:plc:zjbq26wybii5ojoypkso2mso".into(),
            handle: "jauntywunderkind.bsky.social".into(),
            access_jwt: mk(exp_access, "access"),
            refresh_jwt: mk(exp_refresh, "refresh"),
            pds: pds.into(),
            created_at: 0,
        }
    }

    fn file_mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn store_roundtrip_with_private_permissions() {
        let dir = unique_dir("roundtrip");
        let store = SessionStore::at(dir.join("leafspring").join("session.json"));
        let session = sample("https://pds.example", 4_000_000_000, 4_000_000_000);
        store.save(&session).unwrap();
        let file = store.path();
        assert_eq!(file_mode(file), 0o600, "session file must be 0600");
        assert_eq!(file_mode(file.parent().unwrap()), 0o700, "parent dir must be 0700");
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.did, session.did);
        assert_eq!(loaded.pds, "https://pds.example");
        // Debug redacts both JWTs.
        let dbg = format!("{loaded:?}");
        assert!(!dbg.contains("access"), "{dbg}");
        assert!(!dbg.contains("refresh"), "{dbg}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_missing_is_none_and_corrupt_is_error() {
        let dir = unique_dir("load");
        let store = SessionStore::at(dir.join("leafspring").join("session.json"));
        assert!(store.load().unwrap().is_none());
        std::fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        std::fs::write(store.path(), "{not json").unwrap();
        let err = store.load().unwrap_err().to_string();
        assert!(err.contains("decode failed"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn jwt_exp_parses_both_exp_shapes() {
        let far = sample("p", 4_000_000_000, 4_000_000_000);
        assert_eq!(jwt_exp(&far.access_jwt), Some(4_000_000_000));
        assert!(jwt_live(&far.access_jwt, 1_700_000_000, 60));
        // Expired tokens.
        let old = sample("p", 1_000, 1_000);
        assert!(!jwt_live(&old.access_jwt, 1_700_000_000, 60));
        // No exp claim → treated as live (conservative for scheduling).
        assert!(jwt_live("a.b.c", 1_700_000_000, 60));
        assert_eq!(jwt_exp("a.b.c"), None);
    }

    // ── resume-or-create flow against the local test server ───────────────

    use crate::testserver::TestServer;
    use std::sync::Arc;

    /// A JWT-shaped token with an explicit exp (test-issued, unsigned).
    fn jwt_with_exp(exp: i64) -> String {
        use base64::engine::Engine as _;
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256"}"#);
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":{exp}}}"#));
        format!("{header}.{payload}.sig")
    }

    fn session_json(access_exp: i64, refresh_exp: i64) -> String {
        format!(
            r#"{{"accessJwt":"{}","refreshJwt":"{}","did":"did:plc:zjbq26wybii5ojoypkso2mso","handle":"jauntywunderkind.bsky.social"}}"#,
            jwt_with_exp(access_exp),
            jwt_with_exp(refresh_exp)
        )
    }

    #[tokio::test]
    async fn live_stored_session_is_reused_without_network() {
        let server = TestServer::spawn(Arc::new(|_req| panic!("no request expected"))).await;
        let dir = unique_dir("reuse");
        let store = SessionStore::at(dir.join("leafspring").join("session.json"));
        store.save(&sample(&server.base_url, 4_000_000_000, 4_000_000_000)).unwrap();
        let session = resume_or_create_session(&store, "jauntywunderkind.bsky.social", "app-password", &server.base_url)
            .await
            .expect("reuse succeeds");
        assert_eq!(session.did, "did:plc:zjbq26wybii5ojoypkso2mso");
        assert_eq!(server.requests().len(), 0, "live access token means zero wire calls");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn expired_access_refreshes_and_persists_the_new_tokens() {
        let server = TestServer::spawn(Arc::new(|req| {
            if req.path == "/xrpc/com.atproto.server.refreshSession" {
                (200, session_json(4_000_000_000, 4_000_000_000))
            } else {
                panic!("unexpected request {}", req.path);
            }
        }))
        .await;
        let dir = unique_dir("refresh");
        let store = SessionStore::at(dir.join("leafspring").join("session.json"));
        // Access long expired, refresh still live.
        store.save(&sample(&server.base_url, 1_000, 4_000_000_000)).unwrap();
        let session = resume_or_create_session(&store, "jauntywunderkind.bsky.social", "app-password", "https://auth.example")
            .await
            .expect("refresh path succeeds");
        assert_eq!(jwt_exp(&session.access_jwt), Some(4_000_000_000), "new access token in play");
        assert_eq!(server.request_count("/xrpc/com.atproto.server.refreshSession"), 1);
        assert_eq!(
            server.request_count("/xrpc/com.atproto.server.createSession"),
            0,
            "no app-password burn when refresh works"
        );
        // The store now holds the refreshed JWTs.
        let persisted = store.load().unwrap().unwrap();
        assert_eq!(jwt_exp(&persisted.access_jwt), Some(4_000_000_000));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn dead_refresh_falls_back_to_create_session() {
        let server = TestServer::spawn(Arc::new(|req| match req.path.as_str() {
            "/xrpc/com.atproto.server.refreshSession" => (400, r#"{"error":"ExpiredTokenError","message":"token expired"}"#.into()),
            "/xrpc/com.atproto.server.createSession" => (200, session_json(4_000_000_000, 4_000_000_000)),
            other => panic!("unexpected request {other}"),
        }))
        .await;
        let dir = unique_dir("fallback");
        let store = SessionStore::at(dir.join("leafspring").join("session.json"));
        // Both tokens expired; the createSession endpoint is the store's pds.
        store.save(&sample(&server.base_url, 1_000, 2_000)).unwrap();
        let session = resume_or_create_session(&store, "jauntywunderkind.bsky.social", "app-password", &server.base_url)
            .await
            .expect("create fallback succeeds");
        assert_eq!(jwt_exp(&session.access_jwt), Some(4_000_000_000));
        assert_eq!(server.request_count("/xrpc/com.atproto.server.createSession"), 1);
        let create = server
            .requests()
            .into_iter()
            .find(|r| r.path == "/xrpc/com.atproto.server.createSession")
            .unwrap();
        assert_eq!(create.body_json()["identifier"], "jauntywunderkind.bsky.social");
        // The app password traveled once, in the request body only — and is
        // not persisted.
        assert_eq!(create.body_json()["password"], "app-password");
        assert!(!serde_json::to_string(&store.load().unwrap().unwrap()).unwrap().contains("app-password"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn no_store_or_foreign_identity_creates_fresh() {
        let server = TestServer::spawn(Arc::new(|req| {
            assert_eq!(req.path, "/xrpc/com.atproto.server.createSession");
            (200, session_json(4_000_000_000, 4_000_000_000))
        }))
        .await;
        let dir = unique_dir("fresh");
        let store = SessionStore::at(dir.join("leafspring").join("session.json"));
        // Empty store → create.
        resume_or_create_session(&store, "jauntywunderkind.bsky.social", "pw", &server.base_url)
            .await
            .expect("fresh create succeeds");
        assert_eq!(server.request_count("/xrpc/com.atproto.server.createSession"), 1);
        // A stored session for a *different* identity must not be reused
        // even when its tokens are live.
        let mut foreign = sample(&server.base_url, 4_000_000_000, 4_000_000_000);
        foreign.handle = "someone-else.example".into();
        store.save(&foreign).unwrap();
        resume_or_create_session(&store, "jauntywunderkind.bsky.social", "pw", &server.base_url)
            .await
            .expect("identity mismatch forces create");
        assert_eq!(
            server.request_count("/xrpc/com.atproto.server.createSession"),
            2,
            "foreign stored session was not reused"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
