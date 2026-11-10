//! Publish/recover through the actual CLI; independent native mutation store.
#![allow(clippy::disallowed_types)]
#[path = "../../leaflet/protocol/tests/support/store.rs"]
mod store;
#[path = "../../atproto/src/testserver.rs"]
#[allow(dead_code)]
mod testserver;
use leafbud_document::{BlockKind, Inline};
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use tokio::process::Command;
const OWNER: &str = "did:plc:abcdefghijklmnopqrstuvwx";
const URI: &str = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.document/post";
const PUB: &str = "at://did:plc:abcdefghijklmnopqrstuvwx/site.standard.publication/garden";

fn cli(base: &str, root: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_leafbud"));
    c.env_remove("HOME")
        .env_remove("LEAFLET_AUTH_TOKEN")
        .env_remove("LEAFLET_BASE_URL")
        .env("XDG_CONFIG_HOME", root.join(".config"))
        .args(["--base-url", base, "--pds", base, "--root", root.to_str().unwrap(), "--token", "synthetic"])
        .args(args);
    c
}
async fn invoke(base: &str, root: &Path, args: &[&str], code: i32) -> Value {
    let out = cli(base, root, args).output().await.unwrap();
    assert_eq!(
        out.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
#[tokio::test]
async fn preview_first_execute_update_noop_and_uncertain_recovery_cli_both_contexts() {
    for publication in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("doc.mdx");
        let source = path.clone();
        let mut native = store::Store::new(publication);
        native.data["reply_drafts"] = json!([]);
        native.data["publication_scheduled_posts"] = json!([]);
        if publication {
            native.extras.insert("draft_contributors".into(), json!([]));
            native.data["leaflets_in_publications"][0] = json!({"leaflet":store::TOKEN,"publication":PUB,"doc":null,"title":"Untitled","preferences":null,"publications":{"uri":PUB,"identity_did":OWNER,"publication_contributors":[]}});
        }
        let state = Arc::new(Mutex::new(native));
        let calls = Arc::new(AtomicUsize::new(0));
        let sent = calls.clone();
        let lose = Arc::new(AtomicBool::new(false));
        let loss = lose.clone();
        let server=testserver::TestServer::spawn_headers(Arc::new(move |r| {
            let mut state=state.lock().unwrap();
            let response=match r.path.as_str() {
                "/home"=>format!("<script>self.__next_f.push({})</script>",json!([1,format!("0:{}",json!({"atp_did":OWNER,"bsky_profiles":null,"permission_token_on_homepage":[],"contributor_leaflets":[]}))])),
                "/_next/static/publish.js"=>include_str!("../../leaflet/web/tests/fixtures/publish-reference.js").into(),
                "/_next/static/mint.js"=>include_str!("../../leaflet/web/tests/fixtures/mint-reference.js").into(),
                "/lish/did:plc:abcdefghijklmnopqrstuvwx/garden/dashboard" if r.method=="GET"=>"<script src=\"/_next/static/mint.js\"></script>".into(),
                "/lish/did:plc:abcdefghijklmnopqrstuvwx/garden/dashboard"=>return (200,vec![("Content-Type".into(),"text/x-component".into())],include_str!("../../leaflet/web/tests/fixtures/mint-result.flight").into()),
                "/new"=>return (307,vec![("Location".into(),format!("/{}",store::TOKEN))],String::new()),
                p if p.ends_with("/publish") && r.method=="GET"=>"<script src=\"/_next/static/publish.js\"></script>".into(),
                p if p.ends_with("/publish")=>{
                    sent.fetch_add(1,Ordering::SeqCst);
                    if loss.load(Ordering::SeqCst) { return (503,vec![],"lost".into()); }
                    let doc=leafbud_mdx::parse(&std::fs::read_to_string(&source).unwrap()).unwrap();
                    assert!(matches!(doc.state.publication_attempt.unwrap().stage,leafbud_document::PublicationStage::Dispatched{..}));
                    let args=r.body_json()[0].clone(); assert_eq!(args["sendEmail"],false);
                    let blocks:Vec<_>=doc.pages[0].blocks.iter().map(|b|{
                        let BlockKind::Paragraph{content}=&b.kind else{panic!("fixture paragraph")};
                        let text=content.iter().map(|i|match i{Inline::Run{text,..}=>text.as_str(),Inline::HardBreak=>"\n"}).collect::<String>();
                        json!({"$type":"pub.leaflet.pages.linearDocument#block","block":{"$type":"pub.leaflet.blocks.text","plaintext":text}})
                    }).collect();
                    let mut record=json!({"$type":"site.standard.document","site":if publication{PUB.to_owned()}else{format!("https://leaflet.pub/p/{OWNER}")},"title":args["title"],"description":args["description"],"tags":args["tags"],"path":"/post","publishedAt":"2026-11-01T00:00:00Z","content":{"$type":"pub.leaflet.content","pages":[{"$type":"pub.leaflet.pages.linearDocument","id":store::PAGE,"blocks":blocks}]}});
                    if publication{record["contributors"]=json!([{"did":OWNER}]);state.extras.insert("draft_contributors".into(),json!([OWNER]));}
                    let table=if publication{"leaflets_in_publications"}else{"leaflets_to_documents"};
                    let first=state.data[table][0][if publication{"doc"}else{"document"}].is_null();
                    if !publication{state.data[table]=json!([{"leaflet":store::TOKEN,"preferences":null}]);}
                    state.data[table][0][if publication{"doc"}else{"document"}]=json!(URI);
                    state.data[table][0]["documents"]=json!({"uri":URI,"data":record});
                    for key in ["title","description","tags"] {state.extras.insert(format!("publication_{key}"),args[key].clone());}
                    return (200,vec![("Content-Type".into(),"text/x-component".into())],format!("0:{{\"a\":\"$@1\"}}\n1:{}\n",json!({"success":true,"uri":URI,"rkey":"post","record":record,"firstPublish":first,"blocks":blocks.len()})));
                }
                p if p.starts_with("/xrpc/com.atproto.repo.getRecord")=>{
                    let record=&state.data[if publication{"leaflets_in_publications"}else{"leaflets_to_documents"}][0]["documents"]["data"];
                    json!({"uri":URI,"cid":leafbud_atproto::content::record_content_cid(record).unwrap(),"value":record}).to_string()
                }
                _=>state.rpc(&r.path,&r.body_json()).unwrap_or_else(||panic!("unexpected {} {}",r.method,r.path)).to_string(),
            }; (200,vec![],response)
        })).await;
        std::fs::write(&path,format!("---\nformat: leafbud-mdx/2\ntitle: CLI publish\npublication: {}\npublishedAt: 2026-11-01T00:00:00Z\n\"@leafbud\": {{id: \"{}\",state: draft}}\n---\n\nA paragraph.\n",if publication{format!("\"{PUB}\"")}else{"null".into()},leafbud_document::Uuid::new_v4())).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(
            invoke(&server.base_url, root.path(), &["publish", "doc.mdx", "--json"], 0).await["outcome"],
            "would-bind-and-publish"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), mtime);
        assert_eq!(
            invoke(&server.base_url, root.path(), &["publish", "doc.mdx", "--yes", "--json"], 0).await["outcome"],
            "published"
        );
        assert_eq!(
            invoke(&server.base_url, root.path(), &["publish", "doc.mdx", "--yes", "--json"], 0).await["outcome"],
            "unchanged"
        );
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, text.replace("A paragraph.", "New paragraph.\n\nAn additional paragraph.")).unwrap();
        assert_eq!(
            invoke(&server.base_url, root.path(), &["publish", "doc.mdx", "--yes", "--json"], 0).await["outcome"],
            "published"
        );
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, text.replace("New paragraph.", "Never confirmed.")).unwrap();
        lose.store(true, Ordering::SeqCst);
        assert_eq!(
            invoke(&server.base_url, root.path(), &["publish", "doc.mdx", "--yes", "--json"], 4).await["outcome"],
            "partial-unsupported"
        );
        assert_eq!(
            invoke(&server.base_url, root.path(), &["recover", "doc.mdx", "--yes", "--take", "local", "--json"], 4).await["outcome"],
            "partial-unsupported"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        assert_eq!(cli(&server.base_url, root.path(), &["publish"]).output().await.unwrap().status.code(), Some(2));
    }
}
