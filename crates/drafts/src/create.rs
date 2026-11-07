//! `GET /new` — minting a fresh editor draft by harvesting the redirect
//! (`design/draft-extraction.glm53m.md` §"The new-draft gap", option d).
//!
//! Draft creation is a server-action-only path — no Replicache mutation
//! mints `entity_sets`/`permission_tokens`
//! (`experiments/02-draft-internals/REPORT.md` §3, the crucial finding
//! behind the old "pick a throwaway draft in the web UI" step). But
//! leaflet's `app/(app)/(identity)/new/route.ts` is a plain GET route
//! that runs the `createNewLeaflet` server action with
//! `redirectUser: true`, and the action ends in
//! `redirect("/${permTokenId}?focusFirstBlock…&addToHome")` — a 307
//! whose Location header carries the fresh permission token id in its
//! first path segment. One authenticated GET mints a real doc draft (h1
//! seed block, watermark fact, homepage link) and hands us the id.
//!
//! **Assumption flagged for the first gated live run:** the Location
//! header shape is verified against the leaflet source checkout
//! (`actions/createNewLeaflet.ts` @`600c7b5f`), not against a captured
//! live response — experiments/02 holds no `/new` redirect fixture (its
//! probes were read-only). The parser is deliberately liberal (any 3xx;
//! relative or absolute Location; query ignored) so the exact
//! parameter suffix does not matter, and the first operator-approved
//! `--yes` run is the acceptance probe.
//!
//! This is a write even when it fails to authenticate: the insert itself
//! never consults the cookie — only the homepage-link CTE does — so a
//! missing/invalid token still mints an orphan draft nobody lists. The
//! CLI gates the request behind `--yes` exactly like the push.
//!
//! Redirect handling is explicit: reqwest follows redirects by default,
//! and here the redirect target *is* the payload, so [`LeafletClient`]
//! keeps a second inner client with redirects disabled
//! ([`LeafletClient::create_draft`]) and reads the 307 itself.

use uuid::Uuid;

use crate::Error;
use crate::LeafletClient;

/// A fresh draft minted by `GET /new`: its permission token id plus the
/// verbatim redirect target it was harvested from. The Location carries
/// only a path and query — never a secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewDraft {
    pub token_id: String,
    pub location: String,
}

/// Parse a `GET /new` outcome: a 3xx (the source emits 307 via
/// next/navigation `redirect()`) whose Location names the new draft.
/// Pure — offline-testable without a transport.
pub fn parse_new_redirect(http_status: u16, location: Option<&str>) -> Result<NewDraft, Error> {
    if !(300..400).contains(&http_status) {
        return Err(Error::BadResponse {
            rpc: "GET /new".into(),
            detail: format!("expected a 3xx redirect (307), got HTTP {http_status} — the /new route's shape may have changed"),
        });
    }
    let location = location.ok_or_else(|| Error::BadResponse {
        rpc: "GET /new".into(),
        detail: "redirect carries no Location header".into(),
    })?;
    Ok(NewDraft {
        token_id: parse_new_location(location)?,
        location: location.to_string(),
    })
}

/// Harvest the token id from the redirect target: the first path segment
/// must be a uuid (`/<permTokenId>?focusFirstBlock&addToHome` per the
/// source; the query is ignored). Relative and absolute Location forms
/// both parse — the header is resolved against a dummy base first.
pub fn parse_new_location(location: &str) -> Result<String, Error> {
    let segment = first_path_segment(location).unwrap_or_default();
    match Uuid::parse_str(&segment) {
        Ok(id) => Ok(id.to_string()),
        Err(_) => Err(Error::BadResponse {
            rpc: "GET /new".into(),
            detail: format!(
                "redirect target {location:?} carries no token id (first path segment {segment:?} is not a uuid) — \
                 the request may have been redirected to a non-draft route (anonymous token?), or the /new route changed"
            ),
        }),
    }
}

/// First path segment of a Location value, resolving relative forms
/// against a placeholder base so `/x?y` and `https://host/x?y` behave
/// identically.
fn first_path_segment(location: &str) -> Option<String> {
    let base = reqwest::Url::parse("https://harvest.invalid/").ok()?;
    let url = base.join(location).ok()?;
    url.path_segments()?.next().map(str::to_string)
}

impl LeafletClient {
    /// Mint a fresh editor draft via `GET /new` — the one gated write
    /// probe of the new-draft gap resolution. Redirects are disabled for
    /// this request (the 307's Location header is the payload); the auth
    /// cookie rides the client's default headers and is never logged.
    pub async fn create_draft(&self) -> Result<NewDraft, Error> {
        let url = self.url_for("new");
        // Only the URL is logged; never headers (the cookie lives there).
        tracing::debug!(url = %url, "GET /new (redirects disabled)");
        let response = self.http_no_redirect.get(&url).send().await?;
        let status = response.status();
        tracing::debug!(status = %status, "GET /new done");
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        parse_new_redirect(status.as_u16(), location.as_deref())
    }
}

// ---------------------------------------------------------------------------
// Tests (offline; the assumed wire shape is asserted against the
// source-derived fixture strings, and the live method against a local
// loopback responder — no test touches leaflet.pub)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Synthetic ids in the 01999000-… space the other crates' tests use.
    const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";

    /// The assumed Location shape, derived from the leaflet source
    /// (@`600c7b5f`): the route calls
    /// `createNewLeaflet({pageType: "doc", redirectUser: true, addToHome: true})`,
    /// and the action redirects to
    /// `/${permTokenId}?focusFirstBlock` + `&addToHome` (with
    /// `&welcomeModal` inserted when the route's query carried it). Not a
    /// live capture — the first gated `--yes` run is the probe.
    #[test]
    fn assumed_location_shape_from_the_leaflet_source() {
        let draft = parse_new_redirect(307, Some(&format!("/{TOKEN}?focusFirstBlock&addToHome"))).unwrap();
        assert_eq!(draft.token_id, TOKEN);
        assert_eq!(draft.location, format!("/{TOKEN}?focusFirstBlock&addToHome"));
        // ?welcomeModal inserts itself between the two fixed params.
        let draft = parse_new_redirect(307, Some(&format!("/{TOKEN}?focusFirstBlock&welcomeModal&addToHome"))).unwrap();
        assert_eq!(draft.token_id, TOKEN);
        // Any 3xx harvests the same way (the status is next.js's choice).
        for status in [301u16, 302, 303, 307, 308] {
            let draft = parse_new_redirect(status, Some(&format!("/{TOKEN}?focusFirstBlock"))).unwrap();
            assert_eq!(draft.token_id, TOKEN, "status {status}");
        }
    }

    #[test]
    fn absolute_and_bare_locations_parse() {
        // An absolute Location (some proxies rewrite relative ones).
        let draft = parse_new_redirect(307, Some(&format!("https://leaflet.pub/{TOKEN}?focusFirstBlock&addToHome"))).unwrap();
        assert_eq!(draft.token_id, TOKEN);
        // No query at all still harvests.
        let draft = parse_new_redirect(307, Some(&format!("/{TOKEN}"))).unwrap();
        assert_eq!(draft.token_id, TOKEN);
    }

    #[test]
    fn error_shapes_refuse_loudly() {
        // Non-3xx: the route rendered instead of redirecting.
        let err = parse_new_redirect(200, Some(&format!("/{TOKEN}"))).unwrap_err();
        assert!(err.to_string().contains("expected a 3xx redirect"), "{err}");
        assert!(err.to_string().contains("200"), "{err}");
        assert!(matches!(parse_new_redirect(500, None), Err(Error::BadResponse { .. })));
        // 3xx without a Location header.
        let err = parse_new_redirect(307, None).unwrap_err();
        assert!(err.to_string().contains("no Location header"), "{err}");
        // Non-uuid first segment: e.g. an anonymous redirect to /login.
        let err = parse_new_redirect(307, Some("/login?next=/new")).unwrap_err();
        assert!(err.to_string().contains("not a uuid"), "{err}");
        assert!(err.to_string().contains("anonymous"), "{err}");
        // Malformed / empty targets.
        assert!(matches!(parse_new_redirect(307, Some("")), Err(Error::BadResponse { .. })));
        assert!(matches!(parse_new_redirect(307, Some("https://leaflet.pub/")), Err(Error::BadResponse { .. })));
    }

    // ── live method against a loopback responder ──────────────────────────

    /// Minimal HTTP/1.1 loopback responder (the atproto `testserver`
    /// pattern, extended with response headers — here the reply *is* the
    /// headers, not a body). Answers every connection with the canned
    /// `(status, headers, body)`; records requests until dropped.
    mod probe_server {
        use std::sync::{Arc, Mutex};
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::{TcpListener, TcpStream};

        #[derive(Debug, Clone)]
        pub struct Recorded {
            pub method: String,
            pub path: String,
            pub headers: Vec<(String, String)>,
        }

        impl Recorded {
            pub fn header(&self, name: &str) -> Option<&str> {
                let name = name.to_ascii_lowercase();
                self.headers.iter().find(|(n, _)| n == &name).map(|(_, v)| v.as_str())
            }
        }

        type Reply = (u16, Vec<(&'static str, String)>, String);
        /// Test-supplied reply function: → (status, headers, body).
        pub type Responder = std::sync::Arc<dyn Fn() -> Reply + Send + Sync>;

        pub struct ProbeServer {
            pub base_url: String,
            requests: Arc<Mutex<Vec<Recorded>>>,
            task: tokio::task::JoinHandle<()>,
        }

        impl ProbeServer {
            pub async fn spawn(reply: Responder) -> Self {
                let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind loopback");
                let port = listener.local_addr().expect("local addr").port();
                let requests: Arc<Mutex<Vec<Recorded>>> = Arc::new(Mutex::new(Vec::new()));
                let task = {
                    let requests = requests.clone();
                    tokio::spawn(async move {
                        loop {
                            let (stream, _) = match listener.accept().await {
                                Ok(pair) => pair,
                                Err(_) => return,
                            };
                            let requests = requests.clone();
                            let reply = reply.clone();
                            tokio::spawn(async move {
                                if let Ok((stream, recorded)) = read_request(stream).await {
                                    requests.lock().expect("requests lock").push(recorded);
                                    write_response(stream, &reply()).await.ok();
                                }
                            });
                        }
                    })
                };
                Self {
                    base_url: format!("http://127.0.0.1:{port}"),
                    requests,
                    task,
                }
            }

            pub fn requests(&self) -> Vec<Recorded> {
                self.requests.lock().expect("requests lock").clone()
            }
        }

        impl Drop for ProbeServer {
            fn drop(&mut self) {
                self.task.abort();
            }
        }

        async fn read_request(stream: TcpStream) -> std::io::Result<(TcpStream, Recorded)> {
            let mut reader = BufReader::new(stream);
            let mut method = String::new();
            let mut path = String::new();
            let mut headers = Vec::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).await? == 0 {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no request"));
                }
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if method.is_empty() {
                    let mut parts = line.split_whitespace();
                    method = parts.next().unwrap_or_default().to_string();
                    path = parts.next().unwrap_or_default().to_string();
                    if let Some((p, _)) = path.split_once('?') {
                        path = p.to_string();
                    }
                } else if let Some((name, value)) = line.split_once(':') {
                    headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
                }
            }
            Ok((reader.into_inner(), Recorded { method, path, headers }))
        }

        async fn write_response(mut stream: TcpStream, reply: &Reply) -> std::io::Result<()> {
            let (status, headers, body) = reply;
            let mut head = format!("HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n", body.len());
            for (name, value) in headers {
                head.push_str(&format!("{name}: {value}\r\n"));
            }
            head.push_str("\r\n");
            stream.write_all(head.as_bytes()).await?;
            stream.write_all(body.as_bytes()).await?;
            stream.shutdown().await
        }
    }

    #[tokio::test]
    async fn live_probe_harvests_the_307_without_following_it() {
        let token = TOKEN.to_string();
        let reply: probe_server::Responder = {
            let token = token.clone();
            Arc::new(move || (307, vec![("Location", format!("/{token}?focusFirstBlock&addToHome"))], String::new()))
        };
        let server = probe_server::ProbeServer::spawn(reply).await;
        let client = LeafletClient::new(&server.base_url, Some("synthetic-cookie".into())).unwrap();

        let draft = client.create_draft().await.expect("harvest");

        assert_eq!(draft.token_id, TOKEN);
        assert_eq!(draft.location, format!("/{TOKEN}?focusFirstBlock&addToHome"));
        let requests = server.requests();
        // Exactly one request: the 307 must NOT be followed (following it
        // would GET the new draft page and lose the id).
        assert_eq!(requests.len(), 1, "the redirect must not be followed: {requests:?}");
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].path, "/new");
        // The auth cookie rode the request's default headers.
        assert_eq!(requests[0].header("cookie"), Some("auth_token=synthetic-cookie"));
    }

    #[tokio::test]
    async fn live_probe_surfaces_a_non_redirect_answer() {
        // The route rendered a page instead of redirecting — the shape
        // change the parser must alarm on.
        let reply: probe_server::Responder = Arc::new(|| (200, Vec::new(), "<html>landing</html>".to_string()));
        let server = probe_server::ProbeServer::spawn(reply).await;
        let client = LeafletClient::new(&server.base_url, None).unwrap();
        let err = client.create_draft().await.unwrap_err();
        assert!(err.to_string().contains("expected a 3xx redirect"), "{err}");
    }
}
