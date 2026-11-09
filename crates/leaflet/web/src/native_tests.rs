use super::*;
use crate::testserver::TestServer;
use leafbud_protocol::*;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const URI: &str = "at://did:plc:synthetic/site.standard.publication/garden";
const JS: &str = include_str!("../tests/fixtures/mint-reference.js");
fn home() -> String {
    let identity = json!({"atp_did":"did:plc:synthetic","bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]});
    format!("<script>self.__next_f.push({})</script>", json!([1, format!("0:{identity}")]))
}

#[test]
fn bounded_action_reference_and_return_slot_not_a_uuid_scrape() {
    assert_eq!(actions::mint_references(JS).len(), 1);
    assert!(actions::mint_references(r#"createPublicationDraft(\"stale-id\")"#).is_empty());
    assert_eq!(
        actions::mint_result(include_str!("../tests/fixtures/mint-result.flight")).unwrap(),
        Some(TOKEN.parse().unwrap())
    );
    assert_eq!(actions::mint_result("0:{\"a\":null,\"f\":[]}\n").unwrap(), None);
    for bad in [
        format!("0:{{\"f\":[\"{TOKEN}\"]}}\n"),
        "0:{\"a\":\"$@1\"}\n1:E{\"digest\":\"private\"}\n".into(),
        "0:{\"a\":\"$@1\"}\n".into(),
        format!("0:{{\"a\":\"{TOKEN}\"}}\n0:null\n"),
    ] {
        assert!(actions::mint_result(&bad).is_err());
    }
}

#[tokio::test]
async fn publication_uses_current_dashboard_action_and_literal_args_only() {
    let server = TestServer::spawn_headers(Arc::new(|request| {
        assert_eq!(request.header("cookie"), Some("auth_token=synthetic"));
        let (body, mime) = match request.path.as_str() {
            "/home" => (home(), "text/html"),
            "/lish/did:plc:synthetic/garden/dashboard" if request.method == "GET" => (
                "<script src=\"/_next/static/mint.js\"></script><script src=\"https://foreign.test/inject.js\"></script>".into(),
                "text/html",
            ),
            "/_next/static/mint.js" => (JS.into(), "application/javascript"),
            "/lish/did:plc:synthetic/garden/dashboard" => {
                assert_eq!(request.header("next-action"), Some("700000000000000000000000000000000000000001"));
                assert_eq!(request.header("accept"), Some("text/x-component"));
                assert_eq!(request.body_json(), json!([URI, "doc", {}]));
                (include_str!("../tests/fixtures/mint-result.flight").into(), "text/x-component")
            }
            _ => panic!("unexpected native endpoint"),
        };
        (200, vec![("Content-Type".into(), mime.into())], body)
    }))
    .await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
    let preparation = client.prepare_mint(MintTarget::Publication(URI.into())).await.unwrap();
    assert!(server.requests().iter().all(|r| r.method == "GET"));
    let other = LeafletClient::new(&server.base_url, Some("other-session")).unwrap();
    assert!(other.mint(&preparation).await.is_err());
    assert_eq!(client.mint(&preparation).await.unwrap(), Some(TOKEN.parse().unwrap()));
    assert_eq!(server.request_count("/new"), 0);
    assert_eq!(server.requests().len(), 4);
}

#[tokio::test]
async fn standalone_get_is_effect_and_redirect_never_followed() {
    let server = TestServer::spawn_headers(Arc::new(|r| match r.path.as_str() {
        "/home" => (200, vec![], home()),
        "/new" => (307, vec![("Location".into(), format!("/{TOKEN}?focusFirstBlock&addToHome"))], String::new()),
        _ => panic!("redirect followed"),
    }))
    .await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
    let preparation = client.prepare_mint(MintTarget::Standalone).await.unwrap();
    assert_eq!(server.request_count("/new"), 0);
    assert_eq!(client.mint(&preparation).await.unwrap(), Some(TOKEN.parse().unwrap()));
    assert_eq!(server.request_count("/new"), 1);
    let anonymous = LeafletClient::new(&server.base_url, None).unwrap();
    assert!(anonymous.prepare_mint(MintTarget::Standalone).await.is_err());
}

#[tokio::test]
async fn missing_action_fails_closed_without_standalone_fallback_or_mint() {
    let server = TestServer::spawn(Arc::new(|r| {
        if r.path == "/home" {
            (200, home())
        } else {
            (200, "<html>no usable mint reference</html>".into())
        }
    }))
    .await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
    assert!(client.prepare_mint(MintTarget::Publication(URI.into())).await.is_err());
    assert!(
        client
            .prepare_mint(MintTarget::Publication("at://did:plc:x/site.standard.publication/../new".into()))
            .await
            .is_err()
    );
    assert_eq!(server.request_count("/new"), 0);
    assert!(server.requests().iter().all(|r| r.method == "GET"));
}

#[tokio::test]
async fn exact_push_replay_actual_group_acks_and_explicit_detached_reads() {
    let writer = Writer {
        client_group_id: Uuid::new_v4(),
        client_id: Uuid::new_v4(),
    };
    let target = Target {
        token: TOKEN.parse().unwrap(),
        root: Uuid::new_v4(),
        entity_set: Uuid::new_v4(),
    };
    let detached = Uuid::new_v4();
    let fact = Uuid::new_v4();
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let server=TestServer::spawn(Arc::new(move |r|match r.path.as_str(){
        "/api/rpc/push"=>{if count.fetch_add(1,Ordering::SeqCst)==0{(503,"{}".into())}else{(200,"{}".into())}},
        "/api/rpc/pull"=>{
            assert_eq!(r.body_json()["pullRequest"]["clientGroupID"],writer.client_group_id.to_string());
            (200,json!({"patch":[{"op":"clear"},{"op":"put","key":"initialized","value":true}],"lastMutationIDChanges":{writer.client_id.to_string():17}}).to_string())
        }
        "/api/rpc/getFactsForRoots"=>{
            assert_eq!(r.body_json(),json!({"roots":[detached]}));
            (200,json!({"result":{"facts":{detached.to_string():[{"id":fact,"root_id":detached,"entity":detached,"attribute":"block/type","data":{"type":"block-type-union","value":"text"},"future":true}]}}}).to_string())
        }
        _=>panic!("unexpected native endpoint"),
    })).await;
    let client = LeafletClient::new(&server.base_url, Some("synthetic")).unwrap();
    let requests = encode_pushes(
        target,
        writer,
        17,
        42,
        &[NativeMutation::UpdateLeafletMetadata(LeafletMetadata {
            title: Some("Persisted title".into()),
            ..Default::default()
        })],
        &WriteScope::default(),
        BatchLimits::default(),
    )
    .unwrap();
    assert!(client.send_push(&requests[0]).await.is_err());
    client.send_push(&requests[0]).await.unwrap();
    assert_eq!(server.requests()[0].body, server.requests()[1].body);
    let view = client.pull_writer(target.token, writer).await.unwrap();
    assert_eq!(view.acknowledged(writer.client_id), Some(17));
    assert!(view.facts.is_empty(), "acknowledged but skipped is not achievement");
    assert_eq!(
        client.facts_for_entities(&[detached]).await.unwrap().fact(detached, fact).unwrap()["future"],
        true
    );
}
