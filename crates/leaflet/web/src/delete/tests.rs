use super::*;
use crate::testserver::TestServer;
use serde_json::json;
use std::sync::Arc;

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const DID: &str = "did:plc:abcdefghijklmnopqrstuvwx";
const DOCUMENT: &str = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/post";
const JS: &str = include_str!("../../tests/fixtures/delete-reference.js");
const VOID: &str = include_str!("../../tests/fixtures/delete-void.flight");

fn home() -> String {
    let identity = json!({"atp_did":DID,"bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]});
    format!("<script>self.__next_f.push({})</script>", json!([1, format!("0:{identity}")]))
}

fn observed() -> crate::LeafletData {
    crate::parse_rpc(
        &json!({"result":{"error":null,"data":{
            "id":TOKEN,
            "root_entity":"11111111-1111-4111-8111-111111111111",
            "permission_token_rights":[{"token":TOKEN,"entity_set":"44444444-4444-4444-8444-444444444444","read":true,"write":true,"created_at":"2026-01-01"}],
            "leaflets_in_publications":[],
            "leaflets_to_documents":[]
        }}})
        .to_string(),
    )
    .unwrap()
}

#[test]
fn void_and_post_result_contracts_are_literal_not_scrapes() {
    decode_void(VOID).unwrap();
    decode_void("0:{\"a\":null}\n").unwrap();
    for body in ["0:{\"a\":\"$@1\"}\n", "0:{\"a\":\"secret token\"}\n", "0:{\"a\":{\"success\":true}}\n"] {
        assert!(decode_void(body).is_err(), "{body}");
    }
    assert!(matches!(decode_post("0:{\"a\":{\"success\":true}}\n").unwrap(), PostOutcome::Deleted));
    for (kind, expected) in [
        ("oauth_session_expired", PostFailureKind::OauthSessionExpired),
        ("not_authorized", PostFailureKind::NotAuthorized),
    ] {
        let body = format!("0:{{\"a\":{{\"success\":false,\"error\":{{\"type\":\"{kind}\",\"message\":\"private server detail\",\"did\":\"\"}}}}}}\n");
        let PostOutcome::Failed(error) = decode_post(&body).unwrap() else {
            panic!("failure")
        };
        assert_eq!(error.kind, expected);
        assert!(!format!("{:?}", PostOutcome::Failed(error)).contains("private"));
    }
    for body in [
        "0:{\"a\":null}\n",
        "0:{\"a\":{\"success\":true,\"uri\":\"unverified\"}}\n",
        "0:{\"a\":{\"success\":false,\"error\":{\"type\":\"future_error\",\"message\":\"no\"}}}\n",
        "0:{\"a\":{\"success\":\"yes\"}}\n",
    ] {
        assert!(decode_post(body).is_err(), "{body}");
    }
}

#[test]
fn document_uris_are_exact_did_record_identities() {
    for uri in [DOCUMENT, "at://did:plc:abcdefghijklmnopqrstuvwx/pub.leaflet.document/3jz7ijj"] {
        assert!(super::validate_document_uri(uri).is_ok(), "{uri}");
    }
    for uri in [
        "https://leaflet.pub/x",
        "at://example.com/site.standard.document/post",
        // A minimal DID is syntactically acceptable to this gate (same family
        // as the mint dashboard validator); the workflow layer additionally
        // requires the exact observed binding URI before any dispatch.
        format!("at://{DID}/site.standard.publication/garden").as_str(),
        format!("at://{DID}/site.standard.document/").as_str(),
        format!("at://{DID}/site.standard.document/a?b").as_str(),
    ] {
        assert!(super::validate_document_uri(uri).is_err(), "{uri}");
    }
}

#[tokio::test]
async fn actual_next_request_carries_observed_token_shape_and_no_credentials() {
    let server = TestServer::spawn_headers(Arc::new(|r| {
        assert_eq!(r.header("cookie"), Some("auth_token=synthetic"));
        let body = match (r.method.as_str(), r.path.as_str()) {
            ("GET", "/home") => format!("<script src=\"/_next/static/delete.js\"></script>{}", home()),
            ("GET", "/_next/static/delete.js") => JS.into(),
            ("POST", "/home") => {
                assert_eq!(r.header("next-action"), Some("700000000000000000000000000000000000000003"));
                assert_eq!(r.header("next-router-state-tree"), Some(r#"["",{}]"#));
                assert!(r.header("origin").unwrap().starts_with("http://127.0.0.1:"));
                assert_eq!(r.header("accept"), Some("text/x-component"));
                assert_eq!(
                    r.body_json(),
                    json!([{"id":TOKEN,"root_entity":"11111111-1111-4111-8111-111111111111",
                        "permission_token_rights":[{"token":TOKEN,"entity_set":"44444444-4444-4444-8444-444444444444","read":true,"write":true,"created_at":"2026-01-01"}]}])
                );
                VOID.into()
            }
            _ => panic!("unexpected endpoint"),
        };
        (200, vec![("Content-Type".into(), "text/x-component".into())], body)
    }))
    .await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
    let preparation = client.prepare_delete(TOKEN.parse().unwrap()).await.unwrap();
    assert_eq!(preparation.actor(), DID);
    assert_eq!(preparation.post_reference(), Some("700000000000000000000000000000000000000004"));
    client.invoke_delete_leaflet(&preparation, &observed()).await.unwrap();
    let other = LeafletClient::new(&server.base_url, Some("other-session")).unwrap();
    assert!(other.invoke_delete_leaflet(&preparation, &observed()).await.is_err());
    let mut wrong = observed();
    wrong.metadata.id = Uuid::new_v4();
    assert!(client.invoke_delete_leaflet(&preparation, &wrong).await.is_err());
    assert_eq!(server.requests().iter().filter(|r| r.method == "POST").count(), 1);
}

#[tokio::test]
async fn delete_post_sends_exactly_one_document_uri() {
    let server = TestServer::spawn_headers(Arc::new(|r| match (r.method.as_str(), r.path.as_str()) {
        ("GET", "/home") => (200, vec![], format!("<script src=\"/_next/static/delete.js\"></script>{}", home())),
        ("GET", "/_next/static/delete.js") => (200, vec![], JS.into()),
        ("POST", "/home") => {
            assert_eq!(r.header("next-action"), Some("700000000000000000000000000000000000000004"));
            assert_eq!(r.body_json(), json!([DOCUMENT]));
            (
                200,
                vec![("Content-Type".into(), "text/x-component".into())],
                "0:{\"a\":{\"success\":true}}\n".into(),
            )
        }
        _ => panic!("unexpected endpoint"),
    }))
    .await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
    let preparation = client.prepare_delete(TOKEN.parse().unwrap()).await.unwrap();
    assert!(matches!(client.invoke_delete_post(&preparation, DOCUMENT).await.unwrap(), PostOutcome::Deleted));
    for uri in ["at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.publication/garden", "not-a-uri"] {
        assert!(client.invoke_delete_post(&preparation, uri).await.is_err());
    }
    assert_eq!(server.requests().iter().filter(|r| r.method == "POST").count(), 1);
}

#[tokio::test]
async fn lost_redirected_or_error_delete_responses_stay_uncertain_never_retried() {
    for case in 0..4 {
        let invoked = std::sync::atomic::AtomicUsize::new(0);
        let counter = Arc::new(invoked);
        let server = TestServer::spawn_headers(Arc::new(move |r| match (r.method.as_str(), r.path.as_str()) {
            ("GET", "/home") => (200, vec![], format!("<script src=\"/_next/static/delete.js\"></script>{}", home())),
            ("GET", "/_next/static/delete.js") => (200, vec![], JS.into()),
            ("POST", "/home") => {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                match case {
                    0 => (200, vec![("Content-Type".into(), "text/x-component".into())], "0:{\"a\":\"$@1\"}\n".into()),
                    1 => (303, vec![("x-action-redirect".into(), "/login".into())], String::new()),
                    2 => (500, vec![("Content-Type".into(), "text/x-component".into())], String::new()),
                    _ => (200, vec![("Content-Type".into(), "text/html".into())], "login".into()),
                }
            }
            _ => panic!("unexpected endpoint"),
        }))
        .await;
        let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
        let preparation = client.prepare_delete(TOKEN.parse().unwrap()).await.unwrap();
        let error = client.invoke_delete_leaflet(&preparation, &observed()).await.unwrap_err();
        if case == 0 {
            assert_eq!(error.into_action_response().unwrap().body(), "0:{\"a\":\"$@1\"}\n");
        }
        assert_eq!(server.requests().iter().filter(|r| r.method == "POST").count(), 1);
    }
}

#[tokio::test]
async fn missing_ambiguous_or_unauthenticated_discovery_never_dispatches() {
    for variant in 0..3 {
        let server = TestServer::spawn(Arc::new(move |r| match r.path.as_str() {
            "/home" => (
                200,
                match variant {
                    0 => format!(
                        "<script>self.__next_f.push({})</script>",
                        json!([
                            1,
                            format!(
                                "0:{}",
                                json!({"atp_did":DID,"bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]})
                            )
                        ])
                    ),
                    _ => format!("<script src=\"/_next/static/delete.js\"></script>{}", home()),
                },
            ),
            "/_next/static/delete.js" if variant == 1 => (200, format!("{JS}\n{}", JS.replace("000003\"", "000005\""))),
            "/_next/static/delete.js" if variant == 2 => (
                200,
                JS.replace("\"deleteLeaflet\"", "\"futureName\"").replace("\"deletePost\"", "\"futurePost\""),
            ),
            "/_next/static/delete.js" => (200, JS.into()),
            _ => (200, "<script src=\"/_next/static/delete.js\"></script>".into()),
        }))
        .await;
        let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
        let result = client.prepare_delete(TOKEN.parse().unwrap()).await;
        // No identity payload (0), ambiguous deleteLeaflet reference (1) and a
        // chunk set exposing no delete reference (2) all refuse before any
        // effect dispatch.
        assert!(result.is_err());
        assert!(server.requests().iter().all(|r| r.method == "GET"));
    }
    let server = TestServer::spawn(Arc::new(|r| match r.path.as_str() {
        "/home" => (200, format!("<script src=\"/_next/static/delete.js\"></script>{}", home())),
        "/_next/static/delete.js" => (
            200,
            // The deletePost reference's export-name pairing is gone; the
            // remaining createServerReference call is unrelated to it.
            JS.replace(",\"deletePost\")", ",\"unrelatedName\")"),
        ),
        _ => (200, "<script src=\"/_next/static/delete.js\"></script>".into()),
    }))
    .await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
    let preparation = client.prepare_delete(TOKEN.parse().unwrap()).await.unwrap();
    assert_eq!(preparation.post_reference(), None, "optional deletePost absence is not a refusal");
    let anonymous = LeafletClient::new(&server.base_url, None).unwrap();
    assert!(anonymous.prepare_delete(TOKEN.parse().unwrap()).await.is_err());
    assert!(server.requests().iter().all(|r| r.method == "GET"));
}
