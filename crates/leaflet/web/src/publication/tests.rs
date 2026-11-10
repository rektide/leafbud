use super::*;
use crate::testserver::TestServer;
use std::sync::Arc;

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const DID: &str = "did:plc:abcdefghijklmnopqrstuvwx";
const JS: &str = include_str!("../../tests/fixtures/publish-reference.js");
const SUCCESS: &str = include_str!("../../tests/fixtures/publish-result.flight");
fn home() -> String {
    let identity = json!({"atp_did":DID,"bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]});
    format!("<script>self.__next_f.push({})</script>", json!([1, format!("0:{identity}")]))
}
fn args() -> Arguments {
    Arguments {
        root_entity: Uuid::nil(),
        leaflet_id: TOKEN.parse().unwrap(),
        publication_uri: None,
        title: "Title".into(),
        description: "".into(),
        tags: vec!["ordered".into()],
        published_at: None,
    }
}

#[test]
fn literal_success_update_and_failure_contracts_not_release_receipts() {
    let Outcome::Published(published) = decode(SUCCESS).unwrap() else {
        panic!("success")
    };
    assert!(published.first_publish);
    assert_eq!(published.rkey, "post");
    assert!(published.reply.is_none());
    assert!(published.record.get("cid").is_none(), "native success supplies no output CID");
    let Outcome::Published(updated) = decode(&SUCCESS.replace("\"firstPublish\":true", "\"firstPublish\":false")).unwrap() else {
        panic!("update")
    };
    assert!(!updated.first_publish);
    assert_eq!(updated.uri, published.uri);
    for (kind, expected) in [
        ("oauth_session_expired", FailureKind::OauthSessionExpired),
        ("publish_in_progress", FailureKind::PublishInProgress),
        ("publish_failed", FailureKind::PublishFailed),
    ] {
        let body = format!("0:{{\"a\":{{\"success\":false,\"error\":{{\"type\":\"{kind}\",\"message\":\"private server detail\",\"did\":\"{DID}\"}}}}}}\n");
        let outcome = decode(&body).unwrap();
        assert!(!format!("{outcome:?}").contains("private"));
        let Outcome::Failed(error) = outcome else { panic!("failure") };
        assert_eq!(error.kind, expected);
    }
    for body in [
        "0:{\"a\":null}\n",
        "0:{\"a\":\"$@1\"}\n1:E{\"digest\":\"secret\"}\n",
        "0:{\"a\":\"$@1\"}\n",
        "0:{\"a\":{\"success\":true,\"uri\":\"unverified\"}}\n",
        "0:{\"a\":{\"success\":false,\"error\":{\"type\":\"future_error\",\"message\":\"no\"}}}\n",
    ] {
        assert!(decode(body).is_err(), "{body}");
    }
}

#[test]
fn observed_deployed_empty_iterable_and_undefined_reply_decode_without_record_coercion() {
    // Reduced, anonymized literal fixture. X/C framing and optional reply's
    // "$undefined" spelling came from the retained deployed update, not guesses.
    let fixture = include_str!("../../tests/fixtures/publish-empty-iterable.flight");
    let Outcome::Published(published) = decode(fixture).unwrap() else {
        panic!("success")
    };
    assert!(!published.first_publish && published.reply.is_none());
    assert_eq!(published.blocks, 1);
    assert_eq!(published.record["title"], "Updated disposable fixture");
    let body = fixture.replace("Updated disposable fixture", "$undefined");
    let Outcome::Published(published) = decode(&body).unwrap() else {
        panic!("success")
    };
    assert_eq!(published.record["title"], "$undefined", "do not coerce authored values");
    assert!(decode(&fixture.replace("\"reply\":\"$undefined\"", "\"reply\":\"$@12\"")).is_err());
    assert!(decode(&fixture.replace("\"reply\":\"$undefined\"", "\"reply\":{}")).is_err());
}

#[tokio::test]
async fn actual_next_request_forces_no_email_and_never_sends_key_path_or_deletion() {
    let server = TestServer::spawn_headers(Arc::new(|r| {
        assert_eq!(r.header("cookie"), Some("auth_token=synthetic"));
        let body = match (r.method.as_str(), r.path.as_str()) {
            ("GET", "/home") => home(),
            ("GET", "/_next/static/publish.js") => JS.into(),
            ("GET", _) => "<script src=\"/_next/static/publish.js\"></script><script src=\"https://foreign.test/_next/static/inject.js\"></script>".into(),
            ("POST", path) => {
                assert_eq!(path, format!("/{TOKEN}/publish"));
                assert_eq!(r.header("next-action"), Some("700000000000000000000000000000000000000002"));
                assert_eq!(r.header("next-router-state-tree"), Some(r#"["",{}]"#));
                assert!(r.header("origin").unwrap().starts_with("http://127.0.0.1:"));
                assert_eq!(r.header("accept"), Some("text/x-component"));
                assert_eq!(
                    r.body_json(),
                    json!([{"root_entity":Uuid::nil(),"leaflet_id":TOKEN,
                    "publication_uri":"at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.publication/garden",
                    "title":"Title","description":"","tags":["ordered"],"publishedAt":"2026-10-01T10:00:00Z","sendEmail":false}])
                );
                SUCCESS.into()
            }
            _ => panic!("unexpected endpoint"),
        };
        (200, vec![("Content-Type".into(), "text/x-component".into())], body)
    }))
    .await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
    let preparation = client.prepare_publication(TOKEN.parse().unwrap()).await.unwrap();
    assert_eq!(preparation.actor(), DID);
    assert!(server.requests().iter().all(|r| r.method == "GET"));
    let mut args = args();
    args.publication_uri = Some(format!("at://{DID}/site.standard.publication/garden"));
    args.published_at = Some("2026-10-01T10:00:00Z".into());
    let other = LeafletClient::new(&server.base_url, Some("other-session")).unwrap();
    assert!(other.invoke_publication(&preparation, &args).await.is_err());
    let mut wrong = args.clone();
    wrong.leaflet_id = Uuid::new_v4();
    assert!(client.invoke_publication(&preparation, &wrong).await.is_err());
    assert!(matches!(client.invoke_publication(&preparation, &args).await.unwrap(), Outcome::Published(_)));
    assert_eq!(server.requests().iter().filter(|r| r.method == "POST").count(), 1);
    assert_eq!(server.request_count("/new"), 0);
}

#[tokio::test]
async fn undecoded_lost_redirected_server_errors_are_uncertain_never_retried() {
    for case in 0..5 {
        let server = TestServer::spawn_headers(Arc::new(move |r| match (r.method.as_str(), r.path.as_str()) {
            (_, "/home") => (200, vec![], home()),
            (_, "/_next/static/publish.js") => (200, vec![], JS.into()),
            ("GET", _) => (200, vec![], "<script src=\"/_next/static/publish.js\"></script>".into()),
            _ => match case {
                0 => (200, vec![("Content-Type".into(), "text/x-component".into())], "0:{\"a\":\"$@1\"}\n".into()),
                1 => (303, vec![("x-action-redirect".into(), "/login".into())], String::new()),
                2 => (500, vec![("Content-Type".into(), "text/x-component".into())], String::new()),
                3 => (200, vec![("x-nextjs-action-not-found".into(), "1".into())], String::new()),
                _ => (200, vec![("Content-Type".into(), "text/html".into())], "login".into()),
            },
        }))
        .await;
        let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
        let preparation = client.prepare_publication(TOKEN.parse().unwrap()).await.unwrap();
        let error = client.invoke_publication(&preparation, &args()).await.unwrap_err();
        if case == 0 {
            assert_eq!(error.into_action_response().unwrap().body(), "0:{\"a\":\"$@1\"}\n");
        }
        assert_eq!(server.requests().iter().filter(|r| r.method == "POST").count(), 1);
    }
}

#[tokio::test]
async fn missing_ambiguous_or_unauthenticated_discovery_never_dispatches() {
    for ambiguous in [false, true] {
        let server = TestServer::spawn(Arc::new(move |r| {
            (
                200,
                match r.path.as_str() {
                    "/home" => home(),
                    "/_next/static/publish.js" if ambiguous => format!("{JS}\n{}", JS.replace("000002\"", "000003\"")),
                    "/_next/static/publish.js" => "unrelated".into(),
                    _ => "<script src=\"/_next/static/publish.js\"></script>".into(),
                },
            )
        }))
        .await;
        let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
        assert!(client.prepare_publication(TOKEN.parse().unwrap()).await.is_err());
        let anonymous = LeafletClient::new(&server.base_url, None).unwrap();
        assert!(anonymous.prepare_publication(TOKEN.parse().unwrap()).await.is_err());
        assert!(server.requests().iter().all(|r| r.method == "GET"));
    }
}
