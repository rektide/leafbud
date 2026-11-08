use super::*;
use crate::testserver::TestServer;
use serde_json::json;
use std::sync::Arc;

const HOME: &str = include_str!("../../../../experiments/01-draft-list/fixture-home.html");
const DRAFT: &str = include_str!("../../../../experiments/02-draft-internals/fixtures/get_leaflet_data-pure-draft.json");
const PUBLISHED: &str = include_str!("../../../../experiments/02-draft-internals/fixtures/get_leaflet_data-pub-linked.json");
const ID: &str = "01999000-0000-7aaa-8000-000000000001";

fn html(identity: Value) -> String {
    format!("<script>self.__next_f.push({})</script>", json!([1, format!("0:{identity}")]))
}

fn identity() -> Value {
    json!({"permission_token_on_homepage": [], "contributor_leaflets": []})
}

fn row(id: &str) -> Value {
    json!({"archived": null, "created_at": null, "permission_tokens": {
        "id": id, "title": "title with }{ [ ] and a quote\"", "root_entity": null,
        "permission_token_rights": [], "leaflets_in_publications": [], "leaflets_to_documents": []
    }})
}

#[test]
fn recovered_home_fixture_orders_legacy_and_contributor_rows() {
    let list = parse_home(HOME).unwrap();
    assert_eq!(list.leaflets.len(), 5);
    assert_eq!(list.identity.handle.as_deref(), Some("example.bsky.social"));
    assert_eq!(list.fetched_at, Some(1_794_009_762_780));
    assert!(list.href_payload_mismatch.is_empty());
    assert_eq!(list.leaflets[0].metadata.id.to_string(), "01a19bc1-50a0-7bbb-8b00-02bbbb000000");
    assert_eq!(list.leaflets[4].metadata.id.get_version_num(), 4);
    assert!(list.leaflets[1].contributor && !list.leaflets[1].on_homepage);
    assert_eq!(list.leaflets[1].metadata.state(), LeafletState::Published);
    // Retain nested generated record/unknown response fields, not account email.
    let value = serde_json::to_value(list).unwrap();
    assert_eq!(
        value["leaflets"][1]["raw_home_rows"][0]["permission_tokens"]["leaflets_in_publications"][0]["tags"][0],
        "synthetic"
    );
    assert!(!value.to_string().contains("user@example.invalid"));
}

#[test]
fn rpc_fixtures_keep_full_unknown_nested_state() {
    let mut value: Value = serde_json::from_str(DRAFT).unwrap();
    value["result"]["data"]["future-field"] = json!({"nested": [1, "opaque"]});
    let data = parse_rpc(&value.to_string()).unwrap();
    assert_eq!(data.metadata.state(), LeafletState::Draft);
    assert_eq!(data.raw["future-field"]["nested"][1], "opaque");
    assert_eq!(data.raw["permission_token_rights"][0]["entity_sets"]["permission_tokens"][0]["id"], ID);
    assert_eq!(parse_rpc(PUBLISHED).unwrap().metadata.state(), LeafletState::Published);
}

#[test]
fn draft_in_publication_is_draft_and_standalone_is_published() {
    let mut value: Value = serde_json::from_str(DRAFT).unwrap();
    value["result"]["data"]["leaflets_in_publications"] = json!([{"doc":null,"publication":"at://synthetic/publication/one"}]);
    assert_eq!(parse_rpc(&value.to_string()).unwrap().metadata.state(), LeafletState::Draft);
    // Verified upstream getPublicationMetadataFromLeafletData.ts uses document.
    value["result"]["data"]["leaflets_to_documents"] = json!([{"document":"at://synthetic/document/one"}]);
    assert_eq!(parse_rpc(&value.to_string()).unwrap().metadata.state(), LeafletState::Published);
}

#[test]
fn chunk_concatenation_escaping_and_bootstrap() {
    let source = format!("0:{}", json!({"noise": "}{[\\\"", "wrapper": identity()}));
    let split = source.len() / 2;
    let html = format!(
        "<script>self.__next_f.push([0])</script><script>self.__next_f.push({})</script><script>self.__next_f.push({})</script>",
        json!([1, &source[..split]]),
        json!([1, &source[split..]])
    );
    assert_eq!(home::flight_stream(&html).unwrap(), source);
    assert_eq!(parse_home(&html).unwrap().leaflets.len(), 0);
    let object = home::enclosing_object(
        r#"0:{"noise":"}{[}","wrapper":{"permission_token_on_homepage":[]}}"#,
        "permission_token_on_homepage",
    )
    .unwrap();
    assert_eq!(object, r#"{"permission_token_on_homepage":[]}"#);
}

#[test]
fn anonymous_and_malformed_home_are_not_empty_successes() {
    for anonymous in ["", "<html>login</html>", r#"<script>self.__next_f.push([1,"0:{\"landing\":true}"])</script>"#] {
        assert!(matches!(parse_home(anonymous), Err(Error::NoHomePayload)));
    }
    assert!(matches!(parse_home("<script>self.__next_f.push([1,42])</script>"), Err(Error::Home(_))));
    for bad in [
        json!({"permission_token_on_homepage":null}),
        json!({"permission_token_on_homepage":[], "contributor_leaflets":null}),
    ] {
        assert!(parse_home(&html(bad)).is_err());
    }
    let mut bad = identity();
    bad["permission_token_on_homepage"] = json!([{"permission_tokens":{"id":ID}}]);
    assert!(parse_home(&html(bad)).is_err());
    assert!(parse_home(r#"<script>self.__next_f.push([1,"0:{\"permission_token_on_homepage\":[]"])</script>"#).is_err());
    assert!(parse_home(&html(identity())).unwrap().leaflets.is_empty());
}

#[test]
fn duplicates_merge_flags_keep_raw_and_report_dom_mismatch() {
    let mut ident = identity();
    ident["permission_token_on_homepage"] = json!([row(ID)]);
    ident["contributor_leaflets"] = json!([row(ID)]);
    let list = parse_home(&html(ident)).unwrap();
    assert_eq!(list.leaflets.len(), 1);
    let leaflet = &list.leaflets[0];
    assert!(leaflet.on_homepage && leaflet.contributor);
    assert_eq!(leaflet.raw_home_rows.len(), 2);
    assert_eq!(leaflet.created_at_source, Some("uuidv7"));
    assert_eq!(list.href_payload_mismatch, [ID]);
}

#[test]
fn row_timestamps_sort_by_instant_and_uuid_v4_without_date_is_retained() {
    let mut older = row(ID);
    older["created_at"] = json!("2026-01-01T02:00:00+03:00");
    let mut newer = row("01999000-0000-7aaa-8000-000000000004");
    newer["created_at"] = json!("2026-01-01T00:00:00Z");
    let mut ident = identity();
    ident["permission_token_on_homepage"] = json!([older, newer.clone(), row("12345678-1234-4abc-9def-123456789abc")]);
    let list = parse_home(&html(ident)).unwrap();
    assert_eq!(list.leaflets[0].metadata.id.to_string(), newer["permission_tokens"]["id"].as_str().unwrap());
    assert_eq!(list.leaflets[2].created_at, None);
}

#[test]
fn malformed_and_error_rpc_envelopes_are_failures_without_body_leaks() {
    for body in [
        "not JSON",
        "{}",
        r#"{"result":{"data":null,"error":null}}"#,
        r#"{"result":{"data":{},"error":null}}"#,
        r#"{"result":{"data":{}}}"#,
    ] {
        assert!(parse_rpc(body).is_err(), "{body}");
    }
    assert!(matches!(
        parse_rpc(r#"{"result":{"error":{"code":"PGRST116"},"status":406,"data":null}}"#),
        Err(Error::NotFound)
    ));
    let error = parse_rpc(r#"{"result":{"error":{"code":"reflected-secret","message":"secret"},"status":500,"data":null}}"#).unwrap_err();
    assert!(!format!("{error:?} {error}").contains("secret"));
    assert!(matches!(
        parse_rpc(r#"{"result":{"error":null,"status":500,"data":{}}}"#),
        Err(Error::Rpc { .. })
    ));
}

#[test]
fn base_url_cookie_and_locator_validation() {
    for base in [
        "file:///home/example",
        "not a URL",
        "https://user:secret@host",
        "https://host/base",
        "https://host/?secret",
        "https://host/#secret",
        "",
    ] {
        let error = LeafletClient::new(base, None).unwrap_err();
        assert!(matches!(error, Error::BadBaseUrl));
        assert!(!format!("{error:?} {error}").contains("secret"));
    }
    for token in ["", "bad\ncookie", "bad;cookie", "bad cookie", "bad,cookie", "bad\\cookie", "bad\"cookie"] {
        assert!(matches!(LeafletClient::new(DEFAULT_BASE_URL, Some(token)), Err(Error::BadToken)));
    }
    let client = LeafletClient::new("https://example.invalid/", Some("secret-token")).unwrap();
    assert!(!format!("{client:?}").contains("secret-token"));
    assert_eq!(client.resolve_locator(ID).unwrap().to_string(), ID);
    assert_eq!(
        client
            .resolve_locator(&format!("https://example.invalid/{ID}?panel=inspect#top"))
            .unwrap()
            .to_string(),
        ID
    );
    assert!(matches!(
        client.resolve_locator("at://did:plc:x/site.standard.document/one"),
        Err(Error::UnsupportedAtUri)
    ));
    for locator in [
        format!("https://foreign.invalid/{ID}"),
        format!("http://example.invalid/{ID}"),
        format!("https://user:secret@example.invalid/{ID}"),
        format!("https://example.invalid/{ID}/extra"),
        "garbage".into(),
    ] {
        assert!(matches!(client.resolve_locator(&locator), Err(Error::BadLocator)));
    }
}

#[tokio::test]
async fn loopback_reads_send_only_expected_cookie_and_get_request() {
    let server = TestServer::spawn(Arc::new(|request| match request.path.as_str() {
        "/home" => (200, HOME.into()),
        "/api/rpc/get_leaflet_data" => (200, DRAFT.into()),
        _ => (404, "unexpected".into()),
    }))
    .await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic-auth")).unwrap();
    assert_eq!(client.list().await.unwrap().leaflets.len(), 5);
    assert_eq!(client.get(Uuid::parse_str(ID).unwrap()).await.unwrap().metadata.id.to_string(), ID);
    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[1].method, "POST");
    assert_eq!(requests[1].body_json(), json!({"token_id":ID}));
    for request in requests {
        assert_eq!(request.header("cookie"), Some("auth_token=synthetic-auth"));
        assert!(request.query.is_empty());
    }
    assert_eq!(server.request_count("/home"), 1);
}

#[tokio::test]
async fn loopback_http_rpc_malformed_and_mismatched_token_fail() {
    for (status, body) in [(401, "secret"), (500, DRAFT), (302, "redirect"), (200, "not JSON")] {
        let server = TestServer::spawn(Arc::new(move |_| (status, body.into()))).await;
        let client = LeafletClient::new(&server.base_url, None).unwrap();
        let error = client.get(Uuid::parse_str(ID).unwrap()).await.unwrap_err();
        assert!(!format!("{error:?} {error}").contains("secret"));
        if status != 200 {
            assert!(matches!(error, Error::Http { status: s, .. } if s == status));
        }
    }
    let server = TestServer::spawn(Arc::new(|_| (200, DRAFT.into()))).await;
    let client = LeafletClient::new(&server.base_url, None).unwrap();
    assert!(matches!(client.get(Uuid::nil()).await, Err(Error::Response(_))));
    let anonymous = TestServer::spawn(Arc::new(|_| (200, "<html>login</html>".into()))).await;
    assert!(matches!(
        LeafletClient::new(&anonymous.base_url, None).unwrap().list().await,
        Err(Error::NoHomePayload)
    ));
}

#[tokio::test]
async fn redirect_is_not_followed_and_transport_errors_remain_errors() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let target = TestServer::spawn(Arc::new(|_| (200, HOME.into()))).await;
    let location = format!("{}/home", target.base_url);
    let reply = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        {
            let mut reader = BufReader::new(&mut stream);
            loop {
                let mut line = String::new();
                assert_ne!(reader.read_line(&mut line).await.unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
            }
        }
        stream
            .write_all(format!("HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes())
            .await
            .unwrap();
    });
    let client = LeafletClient::new(&base, Some("synthetic-secret")).unwrap();
    assert!(matches!(client.list().await, Err(Error::Http { status: 302, .. })));
    reply.await.unwrap();
    assert!(target.requests().is_empty());
    // Listener has been dropped; this connection must fail, not return an empty list.
    assert!(matches!(client.list().await, Err(Error::Transport(_))));
}
