//! Production controller + actual HTTP and Phoenix sockets. All data synthetic.
use super::*;
use crate::pull::tests::{self as fixtures, fixture, fixture_with_token, set_paragraph_text, yrs_payload};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::{Notify, mpsc},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

const TOKEN: &str = "01999000-0000-7aaa-8000-000000000001";
const TOKEN2: &str = "01999000-0000-7aaa-8000-000000000002";

struct Http {
    base: String,
    fixtures: Arc<RwLock<BTreeMap<String, Value>>>,
    hold: Arc<AtomicBool>,
    release: Arc<Notify>,
    reads: Arc<AtomicUsize>,
    fail: Arc<Mutex<Option<String>>>,
    task: tokio::task::JoinHandle<()>,
}
impl Http {
    async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let fixtures = Arc::new(RwLock::new(BTreeMap::from([
            (TOKEN.into(), fixture()),
            (TOKEN2.into(), fixture_with_token(TOKEN2)),
        ])));
        let hold = Arc::new(AtomicBool::new(false));
        let release = Arc::new(Notify::new());
        let reads = Arc::new(AtomicUsize::new(0));
        let fail = Arc::new(Mutex::new(None));
        let shared = (fixtures.clone(), hold.clone(), release.clone(), reads.clone(), fail.clone());
        let task = tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let (fixtures, hold, release, reads, fail) = shared.clone();
                tokio::spawn(async move {
                    let mut stream = BufReader::new(stream);
                    let mut first = String::new();
                    stream.read_line(&mut first).await.unwrap();
                    let path = first.split_whitespace().nth(1).unwrap().to_owned();
                    if path == "/" {
                        let _ = stream
                            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\nConnection: close\r\n\r\n<html></html>")
                            .await;
                        return;
                    }
                    let mut len = 0;
                    loop {
                        let mut line = String::new();
                        stream.read_line(&mut line).await.unwrap();
                        if line.trim().is_empty() {
                            break;
                        }
                        if let Some((key, value)) = line.split_once(':')
                            && key.eq_ignore_ascii_case("content-length")
                        {
                            len = value.trim().parse().unwrap();
                        }
                    }
                    let mut body = vec![0; len];
                    stream.read_exact(&mut body).await.unwrap();
                    let request: Value = serde_json::from_slice(&body).unwrap();
                    let token = request["token_id"].as_str().unwrap();
                    let data = fixtures.read().unwrap().get(token).cloned();
                    let failure = fail.lock().unwrap().as_deref() == Some(token);
                    let (status, response) = if failure {
                        (401, json!({}))
                    } else if let Some(data) = data {
                        if path.ends_with("/pull") {
                            reads.fetch_add(1, Ordering::SeqCst);
                            if hold.load(Ordering::SeqCst) {
                                release.notified().await;
                            }
                            (200, data["response"].clone())
                        } else {
                            (200, json!({"result":{"error":null,"data":data["leaflet_data"]}}))
                        }
                    } else {
                        (200, json!({"result":{"error":{"code":"PGRST116"},"data":null}}))
                    };
                    let response = response.to_string();
                    let head = format!(
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        response.len()
                    );
                    let _ = stream.write_all(head.as_bytes()).await;
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        });
        Self {
            base,
            fixtures,
            hold,
            release,
            reads,
            fail,
            task,
        }
    }
    fn reader(&self) -> RemoteReader {
        RemoteReader::new(leafbud_web::LeafletClient::new(&self.base, Some("synthetic")).unwrap())
    }
    fn seed(&self, root: &Path, token: &str, file: &str) -> PathBuf {
        let fixture = self.fixtures.read().unwrap()[token].clone();
        let doc = crate::import::build(
            &self.base,
            &fixtures::data(&fixture),
            &leafbud_web::parse_pull(fixture["response"].clone()).unwrap(),
        )
        .unwrap();
        let path = root.join(file);
        std::fs::write(&path, leafbud_mdx::render(&doc).unwrap()).unwrap();
        path
    }
    fn change(&self, token: &str, text: &str) {
        set_paragraph_text(self.fixtures.write().unwrap().get_mut(token).unwrap(), &yrs_payload(text, None));
    }
}
impl Drop for Http {
    fn drop(&mut self) {
        self.hold.store(false, Ordering::SeqCst);
        self.release.notify_waiters();
        self.task.abort();
    }
}

enum Wire {
    Poke,
    Drop,
    Join,
}
struct Socket {
    project: PublicProject,
    commands: mpsc::UnboundedSender<Wire>,
    joins: Arc<AtomicUsize>,
    beats: Arc<AtomicUsize>,
    leaves: Arc<AtomicUsize>,
    hold_join: Arc<AtomicBool>,
    task: tokio::task::JoinHandle<()>,
}
impl Socket {
    async fn new() -> Self {
        use base64::Engine;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let key = format!(
            "e30.{}.synthetic",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"role":"anon","ref":"synthetic"}"#)
        );
        let project = PublicProject::new(&url, &key).unwrap();
        let (commands, mut receiver) = mpsc::unbounded_channel();
        let (joins, beats, leaves) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
        let hold_join = Arc::new(AtomicBool::new(false));
        let shared = (joins.clone(), beats.clone(), leaves.clone(), hold_join.clone());
        let task = tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let Ok(mut socket) = accept_async(stream).await else {
                    continue;
                };
                let mut joined = BTreeMap::<String, Value>::new();
                let mut pending = vec![];
                loop {
                    tokio::select! {
                        frame = socket.next() => {
                            let Some(Ok(Message::Text(text))) = frame else { break; };
                            let frame: Value = serde_json::from_str(&text).unwrap();
                            match frame[3].as_str().unwrap() {
                                "phx_join" => {
                                    shared.0.fetch_add(1, Ordering::SeqCst);
                                    joined.insert(frame[2].as_str().unwrap().into(), frame.clone());
                                    if shared.3.load(Ordering::SeqCst) { pending.push(frame); continue; }
                                }
                                "phx_leave" => { shared.2.fetch_add(1, Ordering::SeqCst); joined.remove(frame[2].as_str().unwrap()); }
                                "heartbeat" => { shared.1.fetch_add(1, Ordering::SeqCst); }
                                other => panic!("unexpected application send {other}"),
                            }
                            let reply = json!([frame[0], frame[1], frame[2], "phx_reply", {"status":"ok","response":{}}]);
                            if socket.send(Message::text(reply.to_string())).await.is_err() { break; }
                            if frame[3] == "phx_join" && frame[2].as_str().unwrap().contains(":yjs:") {
                                let _ = socket.send(Message::text(json!([frame[0],null,frame[2],"presence_state",{}]).to_string())).await;
                            }
                        }
                        command = receiver.recv() => match command {
                            Some(Wire::Drop) => break,
                            Some(Wire::Poke) => {
                                for frame in joined.values().filter(|f| f[2].as_str().unwrap().contains(":rootEntity:")) {
                                    let poke = json!([frame[0],null,frame[2],"broadcast",{"event":"poke","type":"broadcast","payload":{}}]);
                                    let _ = socket.send(Message::text(poke.to_string())).await;
                                }
                            }
                            Some(Wire::Join) => {
                                for frame in pending.drain(..) { let _ = socket.send(Message::text(json!([frame[0],frame[1],frame[2],"phx_reply",{"status":"ok","response":{}}]).to_string())).await; }
                            }
                            None => return,
                        }
                    }
                }
            }
        });
        Self {
            project,
            commands,
            joins,
            beats,
            leaves,
            hold_join,
            task,
        }
    }
    fn options(&self) -> Options {
        Options {
            project: Some(self.project.clone()),
            rescan: Duration::from_millis(30),
            backstop: Duration::from_secs(20),
            retry_min: Duration::from_millis(60),
            retry_max: Duration::from_millis(200),
            bootstrap_retry: Duration::from_millis(100),
            transport: TransportOptions {
                heartbeat_interval: Duration::from_millis(50),
                ack_timeout: Duration::from_secs(2),
                connect_timeout: Duration::from_secs(1),
                leave_timeout: Duration::from_millis(100),
                backoff_min: Duration::from_millis(20),
                backoff_max: Duration::from_millis(100),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    fn poke(&self) {
        self.commands.send(Wire::Poke).unwrap();
    }
}
impl Drop for Socket {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn until(mut test: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !test() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("condition deadline");
}
async fn outcome(events: &mut mpsc::UnboundedReceiver<Event>, wanted: refresh::Outcome) -> refresh::Report {
    tokio::time::timeout(Duration::from_secs(6), async {
        loop {
            if let Detail::Document { report, .. } = events.recv().await.expect("watch events").detail
                && report.outcome == wanted
            {
                return report;
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("missing outcome {wanted:?}"))
}

#[tokio::test]
async fn actual_poke_adopts_and_duplicate_is_exact_noop_preserving_author_identity() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut doc = leafbud_mdx::parse(&source).unwrap();
    doc.state.id = Uuid::new_v4();
    let old = doc.pages[0].blocks[0].key.clone();
    let source = leafbud_mdx::render(&doc)
        .unwrap()
        .replace(&old, "manual-key")
        .replacen("\n\"@leafbud\":", "\ncustom: {keep: [1, 2]}\n\"@leafbud\":", 1);
    std::fs::write(&path, &source).unwrap();
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let (cancel, _) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancel.subscribe(),
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        outcome(&mut events, refresh::Outcome::Unchanged).await;
        http.change(TOKEN, "Durable remote edit");
        socket.poke();
        outcome(&mut events, refresh::Outcome::Updated).await;
        let bytes = std::fs::read(&path).unwrap();
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        let source = String::from_utf8(bytes.clone()).unwrap();
        assert!(source.contains("Durable remote edit") && source.contains("manual-key") && source.contains("custom: {keep: [1, 2]}"));
        assert_eq!(leafbud_mdx::parse(&source).unwrap().state.id, doc.state.id);
        socket.poke();
        outcome(&mut events, refresh::Outcome::Unchanged).await;
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    assert_eq!(report.unwrap().writes, 1);
}

// Event owns only small transition metadata, never remote content. This helper
// explicitly clones the few variants observed by these tests.
fn copy_event(event: &Event) -> Event {
    Event {
        version: event.version,
        observed_at: event.observed_at,
        detail: match &event.detail {
            Detail::Document { report, acquisition } => Detail::Document {
                report: report.clone(),
                acquisition,
            },
            Detail::Connection { health, message } => Detail::Connection {
                health: *health,
                message: message.clone(),
            },
            Detail::Inventory {
                selected,
                subscribed_roots,
                message,
            } => Detail::Inventory {
                selected: *selected,
                subscribed_roots: *subscribed_roots,
                message: message.clone(),
            },
            Detail::Presence { path, document_id, state } => Detail::Presence {
                path: path.clone(),
                document_id: *document_id,
                state: state.clone(),
            },
            Detail::Summary { summary } => Detail::Summary { summary: summary.clone() },
            Detail::Subscription { path, document_id, state } => Detail::Subscription {
                path: path.clone(),
                document_id: *document_id,
                state: *state,
            },
        },
    }
}

#[tokio::test]
async fn acknowledged_initial_join_and_poke_during_slow_read_queue_second_acquisition() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    http.hold.store(true, Ordering::SeqCst);
    socket.hold_join.store(true, Ordering::SeqCst);
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        until(|| socket.joins.load(Ordering::SeqCst) > 0).await;
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert_eq!(http.reads.load(Ordering::SeqCst), 0, "connection intent is not readiness");
        socket.commands.send(Wire::Join).unwrap();
        until(|| http.reads.load(Ordering::SeqCst) == 1).await;
        let beats = socket.beats.load(Ordering::SeqCst);
        http.change(TOKEN, "Edit after first snapshot");
        for _ in 0..12 {
            socket.poke();
        }
        until(|| socket.beats.load(Ordering::SeqCst) >= beats + 2).await;
        assert_eq!(http.reads.load(Ordering::SeqCst), 1, "serial refresh remains bounded");
        http.hold.store(false, Ordering::SeqCst);
        http.release.notify_waiters();
        outcome(&mut events, refresh::Outcome::Updated).await;
        assert!(std::fs::read_to_string(&path).unwrap().contains("Edit after first snapshot"));
        assert!(http.reads.load(Ordering::SeqCst) >= 2, "new start stamp not lost at completion");
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    assert!(report.unwrap().stopped);
}

#[tokio::test]
async fn reconnect_catches_missed_remote_edit_and_passive_presence_is_advisory() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let mut options = socket.options();
    options.transport.presence = true;
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        options,
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        let mut peer_snapshot = false;
        tokio::time::timeout(Duration::from_secs(4), async {
            while !peer_snapshot {
                if let Detail::Presence { state, .. } = events.recv().await.unwrap().detail
                    && state.evidence == live::PresenceEvidence::NoPeersObserved
                {
                    assert!(state.observed_at.is_some());
                    peer_snapshot = true;
                }
            }
        })
        .await
        .unwrap();
        until(|| http.reads.load(Ordering::SeqCst) >= 1).await;
        http.change(TOKEN, "Missed while disconnected");
        socket.commands.send(Wire::Drop).unwrap();
        outcome(&mut events, refresh::Outcome::Updated).await;
        assert!(std::fs::read_to_string(&path).unwrap().contains("Missed while disconnected"));
        assert!(socket.joins.load(Ordering::SeqCst) >= 4);
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    assert_eq!(report.unwrap().writes, 1);
}

#[tokio::test]
async fn source_save_during_await_is_preserved_and_cancellation_releases_writer_lock() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    http.change(TOKEN, "Remote edit must not overwrite later save");
    http.hold.store(true, Ordering::SeqCst);
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        until(|| http.reads.load(Ordering::SeqCst) == 1).await;
        let report = reader
            .pull(
                &context,
                crate::pull::Request {
                    files: vec![path.clone()],
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(report.failed, 1);
        assert!(report.items[0].error.as_deref().unwrap().contains("busy"));
        let tidy = crate::tidy::run(
            &context,
            crate::tidy::Request {
                files: vec![path.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(tidy.failed, 1);
        assert!(tidy.items[0].error.as_deref().unwrap().contains("busy"));
        let newer = std::fs::read_to_string(&path)
            .unwrap()
            .replace("Synthetic body copy", "Author save during HTTP");
        std::fs::write(&path, &newer).unwrap();
        http.hold.store(false, Ordering::SeqCst);
        http.release.notify_waiters();
        outcome(&mut events, refresh::Outcome::SourceChanged).await;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), newer);
        outcome(&mut events, refresh::Outcome::NeedsDecision).await;
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    report.unwrap();
    let id = crate::lock::source_id(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(crate::lock::Store::new(&context).unwrap().document(id).unwrap().is_some());
}

#[tokio::test]
async fn cancellation_during_refresh_is_finite_and_never_advances_receipts() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let before = std::fs::read(&path).unwrap();
    http.change(TOKEN, "Remote pending during cancellation");
    http.hold.store(true, Ordering::SeqCst);
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let (cancel, cancelled) = watch::channel(false);
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |_| Ok(()),
    );
    let actions = async {
        until(|| http.reads.load(Ordering::SeqCst) > 0).await;
        cancel.send_replace(true);
    };
    let (report, _) = tokio::time::timeout(Duration::from_secs(3), async { tokio::join!(run, actions) })
        .await
        .unwrap();
    assert_eq!(report.unwrap().cancelled_refreshes, 1);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let id = crate::lock::source_id(std::str::from_utf8(&before).unwrap()).unwrap();
    assert!(crate::lock::Store::new(&context).unwrap().document(id).unwrap().is_some());
}

#[tokio::test]
async fn busy_document_retries_without_stalling_other_documents_or_heartbeats() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "a.mdx");
    let other = http.seed(temp.path(), TOKEN2, "b.mdx");
    let context = fixtures::context(temp.path());
    let id = crate::lock::source_id(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut guard = crate::lock::Store::new(&context).unwrap().document(id).unwrap();
    http.change(TOKEN, "A becomes available");
    http.change(TOKEN2, "B progresses while A busy");
    let reader = http.reader();
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        outcome(&mut events, refresh::Outcome::Busy).await;
        let report = outcome(&mut events, refresh::Outcome::Updated).await;
        assert_eq!(report.path, other);
        until(|| socket.beats.load(Ordering::SeqCst) > 0).await;
        guard.take();
        let report = outcome(&mut events, refresh::Outcome::Updated).await;
        assert_eq!(report.path, path);
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    assert_eq!(report.unwrap().writes, 2);
}

#[tokio::test]
async fn rescan_follows_explicit_identity_rename_blocks_duplicates_and_never_recreates_removed_path() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let renamed = temp.path().join("renamed.mdx");
    let duplicate = temp.path().join("copy.mdx");
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            files: vec![path.clone()],
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        outcome(&mut events, refresh::Outcome::Unchanged).await;
        std::fs::rename(&path, &renamed).unwrap();
        let report = outcome(&mut events, refresh::Outcome::Unchanged).await;
        assert_eq!(report.path, renamed);
        std::fs::copy(&renamed, &duplicate).unwrap();
        outcome(&mut events, refresh::Outcome::Ambiguous).await;
        http.change(TOKEN, "Remote while duplicate exists");
        socket.poke();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!std::fs::read_to_string(&renamed).unwrap().contains("Remote while duplicate exists"));
        std::fs::remove_file(&duplicate).unwrap();
        outcome(&mut events, refresh::Outcome::Updated).await;
        assert!(std::fs::read_to_string(&renamed).unwrap().contains("Remote while duplicate exists"));
        std::fs::remove_file(&renamed).unwrap();
        outcome(&mut events, refresh::Outcome::SourceLost).await;
        socket.poke();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!renamed.exists() && !path.exists());
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    report.unwrap();
    assert!(socket.leaves.load(Ordering::SeqCst) >= 1);
}

#[tokio::test]
async fn local_dirty_diverged_pending_unknown_unbound_and_no_receipt_have_honest_preserving_outcomes() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let original = std::fs::read_to_string(&path).unwrap();
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        outcome(&mut events, refresh::Outcome::Unchanged).await;
        let dirty = original.replace("Synthetic body copy", "Local work");
        std::fs::write(&path, &dirty).unwrap();
        outcome(&mut events, refresh::Outcome::LocalDirty).await;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), dirty);
        http.change(TOKEN, "Remote divergence");
        socket.poke();
        outcome(&mut events, refresh::Outcome::NeedsDecision).await;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), dirty);
        for (addition, wanted) in [
            ("pending: {future: [1, 2], attempt: unknown}", refresh::Outcome::Pending),
            ("futureOwned: true", refresh::Outcome::Invalid),
        ] {
            let changed = original.replacen("\"@leafbud\":\n", &format!("\"@leafbud\":\n  {addition}\n"), 1);
            assert_ne!(changed, original);
            std::fs::write(&path, &changed).unwrap();
            outcome(&mut events, wanted).await;
            assert_eq!(std::fs::read_to_string(&path).unwrap(), changed);
        }
        let mut no_receipt = leafbud_mdx::parse(&original).unwrap();
        no_receipt.state.sync = None;
        let changed = leafbud_mdx::render(&no_receipt).unwrap();
        std::fs::write(&path, &changed).unwrap();
        outcome(&mut events, refresh::Outcome::NeedsDecision).await;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), changed, "no historical direction inferred");
        // Equal endpoints with absent receipts may explicitly settle the current
        // verified observation. Watch's policy permits this, never differing data.
        *http.fixtures.write().unwrap().get_mut(TOKEN).unwrap() = fixture();
        socket.poke();
        outcome(&mut events, refresh::Outcome::Converged).await;
        assert!(leafbud_mdx::parse(&std::fs::read_to_string(&path).unwrap()).unwrap().state.sync.is_some());
        let unkeyed = format!("{original}\nNew authored paragraph without a key\n");
        std::fs::write(&path, &unkeyed).unwrap();
        outcome(&mut events, refresh::Outcome::LocalDirty).await;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), unkeyed);
        crate::create::run(
            &context,
            crate::create::Request {
                file: "unbound.mdx".into(),
                title: "Local draft".into(),
                publication: leafbud_document::Publication::Inherit,
            },
        )
        .unwrap();
        outcome(&mut events, refresh::Outcome::Unbound).await;
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    report.unwrap();
}

#[tokio::test]
async fn http_auth_failure_retries_fairly_and_missing_remote_never_prunes() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "a.mdx");
    let other = http.seed(temp.path(), TOKEN2, "b.mdx");
    *http.fail.lock().unwrap() = Some(TOKEN.into());
    http.change(TOKEN2, "Other still progresses");
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        outcome(&mut events, refresh::Outcome::Unavailable).await;
        assert_eq!(outcome(&mut events, refresh::Outcome::Updated).await.path, other);
        *http.fail.lock().unwrap() = None;
        outcome(&mut events, refresh::Outcome::Unchanged).await;
        http.fixtures.write().unwrap().remove(TOKEN);
        socket.poke();
        outcome(&mut events, refresh::Outcome::Unavailable).await;
        assert!(path.exists());
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    report.unwrap();
}

#[tokio::test]
async fn bootstrap_failure_is_degraded_bounded_and_slow_backstop_is_not_live() {
    let http = Http::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    http.change(TOKEN, "Slow fallback adoption");
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let options = Options {
        rescan: Duration::from_millis(20),
        backstop: Duration::from_millis(180),
        bootstrap_retry: Duration::from_secs(30),
        ..Default::default()
    };
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        options,
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        tokio::time::sleep(Duration::from_millis(70)).await;
        assert_eq!(
            http.reads.load(Ordering::SeqCst),
            0,
            "failed bootstrap does not bypass initial readiness as live"
        );
        let mut degraded = false;
        tokio::time::timeout(Duration::from_secs(4), async {
            loop {
                match events.recv().await.unwrap().detail {
                    Detail::Connection { health, message } => {
                        assert_ne!(health, live::Health::Live);
                        if health == live::Health::Degraded {
                            assert!(message.is_some());
                            degraded = true;
                        }
                    }
                    Detail::Document { report, acquisition } if report.outcome == refresh::Outcome::Updated => {
                        assert_eq!(acquisition, "degraded-poll");
                        break;
                    }
                    _ => {}
                }
            }
        })
        .await
        .unwrap();
        assert!(degraded);
        assert!(std::fs::read_to_string(&path).unwrap().contains("Slow fallback adoption"));
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    report.unwrap();
}

#[tokio::test]
async fn cancellation_during_bootstrap_and_unacknowledged_join_is_finite() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let before = std::fs::read(&path).unwrap();
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    socket.hold_join.store(true, Ordering::SeqCst);
    let (cancel, cancelled) = watch::channel(false);
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |_| Ok(()),
    );
    let actions = async {
        until(|| socket.joins.load(Ordering::SeqCst) > 0).await;
        cancel.send_replace(true);
    };
    let (report, _) = tokio::time::timeout(Duration::from_secs(3), async { tokio::join!(run, actions) })
        .await
        .unwrap();
    assert_eq!(report.unwrap().refreshes, 0);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    // A landing server which never responds exercises dropping discovery itself.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let reader = RemoteReader::new(leafbud_web::LeafletClient::new(&format!("http://{}", listener.local_addr().unwrap()), None).unwrap());
    let (cancel, cancelled) = watch::channel(false);
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        Options::default(),
        cancelled,
        |_| Ok(()),
    );
    let actions = async {
        let (_tcp, _) = listener.accept().await.unwrap();
        cancel.send_replace(true);
    };
    let (report, _) = tokio::time::timeout(Duration::from_secs(2), async { tokio::join!(run, actions) })
        .await
        .unwrap();
    report.unwrap();
}

#[tokio::test]
async fn changing_root_during_read_cancels_old_selection_then_reselect_catches_up() {
    let http = Http::new().await;
    let socket = Socket::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let original = std::fs::read_to_string(&path).unwrap();
    http.hold.store(true, Ordering::SeqCst);
    http.change(TOKEN, "Adopt only after original root returns");
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    let run = reader.watch(
        &context,
        Request {
            all: true,
            ..Default::default()
        },
        socket.options(),
        cancelled,
        |e| {
            tx.send(copy_event(e)).unwrap();
            Ok(())
        },
    );
    let actions = async {
        until(|| http.reads.load(Ordering::SeqCst) == 1).await;
        let mut doc = leafbud_mdx::parse(&original).unwrap();
        doc.state.binding.as_mut().unwrap().root = Uuid::new_v4().to_string();
        let changed = leafbud_mdx::render(&doc).unwrap();
        std::fs::write(&path, &changed).unwrap();
        until(|| socket.joins.load(Ordering::SeqCst) >= 2).await;
        http.hold.store(false, Ordering::SeqCst);
        http.release.notify_waiters();
        outcome(&mut events, refresh::Outcome::NeedsDecision).await;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), changed);
        std::fs::write(&path, &original).unwrap();
        outcome(&mut events, refresh::Outcome::Updated).await;
        assert!(std::fs::read_to_string(&path).unwrap().contains("Adopt only after original root returns"));
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    assert!(report.unwrap().cancelled_refreshes >= 1);
}

#[tokio::test]
async fn two_imports_share_binding_lock_and_recheck_selection_after_release() {
    let http = Http::new().await;
    let temp = tempfile::tempdir().unwrap();
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    http.hold.store(true, Ordering::SeqCst);
    let first = reader.import(
        &context,
        crate::import::Request {
            locator: Some(TOKEN.into()),
            to: Some("one.mdx".into()),
            ..Default::default()
        },
    );
    let other = async {
        until(|| http.reads.load(Ordering::SeqCst) == 1).await;
        let report = reader
            .import(
                &context,
                crate::import::Request {
                    locator: Some(TOKEN.into()),
                    to: Some("two.mdx".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(report.failed, 1);
        assert!(report.items[0].error.as_deref().unwrap().contains("busy"));
        http.hold.store(false, Ordering::SeqCst);
        http.release.notify_waiters();
    };
    let (report, _) = tokio::join!(first, other);
    assert_eq!(report.unwrap().imported, 1);
    let report = reader
        .import(
            &context,
            crate::import::Request {
                locator: Some(TOKEN.into()),
                to: Some("two.mdx".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(report.failed, 1);
    assert!(!temp.path().join("two.mdx").exists());
    assert!(temp.path().join("one.mdx").exists());
}

#[tokio::test]
async fn refresh_preserves_raw_future_pending_and_owned_state_without_network() {
    let http = Http::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let original = std::fs::read_to_string(&path).unwrap();
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    for (extra, wanted) in [
        ("pending: {unknownVersion: 99, evidence: [private]}", refresh::Outcome::Pending),
        ("pending: null", refresh::Outcome::Invalid),
        ("futureOwned: {doNotDrop: true}", refresh::Outcome::Invalid),
    ] {
        let changed = original.replacen("\"@leafbud\":\n", &format!("\"@leafbud\":\n  {extra}\n"), 1);
        assert_ne!(changed, original);
        std::fs::write(&path, &changed).unwrap();
        let report = reader
            .refresh_document(
                &context,
                &path,
                refresh::Policy {
                    take_remote: true,
                    ..Default::default()
                },
            )
            .await;
        assert_eq!(report.outcome, wanted);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), changed);
    }
    assert_eq!(http.reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_new_transport_stamp_domain_cannot_be_serviced_by_an_old_inflight_read() {
    let http = Http::new().await;
    let temp = tempfile::tempdir().unwrap();
    let path = http.seed(temp.path(), TOKEN, "doc.mdx");
    let reader = http.reader();
    let context = fixtures::context(temp.path());
    let selection = selection::Selection::new(
        temp.path().into(),
        &http.base,
        Request {
            all: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (selected, roots) = selection.scan().unwrap();
    let root = roots[0];
    let (roots_tx, _) = watch::channel(roots);
    let joined = live::State {
        health: live::Health::Live,
        connection: live::Connection::Connected,
        roots: BTreeMap::from([(
            root,
            live::RootState {
                subscription: live::SubscriptionState::Joined,
                observation_seq: 1,
                join_epoch: 1,
                observed_at: Some(SystemTime::now()),
                presence: None,
            },
        )]),
    };
    let (state, mut receiver) = watch::channel(Transport {
        generation: 1,
        state: joined.clone(),
        message: None,
    });
    let (cancel, cancelled) = watch::channel(false);
    let (tx, mut events) = mpsc::unbounded_channel();
    http.hold.store(true, Ordering::SeqCst);
    let options = Options {
        rescan: Duration::from_millis(20),
        ..Default::default()
    };
    let mut emit = |e: &Event| {
        tx.send(copy_event(e)).unwrap();
        Ok(())
    };
    // The restart stamp collision is injected at the state seam; the document
    // transaction, HTTP acquisition and adoption remain the production path.
    let run = reader.watch_controller(&context, &selection, selected, &options, cancelled, &roots_tx, &mut receiver, &mut emit);
    let actions = async {
        until(|| http.reads.load(Ordering::SeqCst) == 1).await;
        http.change(TOKEN, "New run has a distinct stamp domain");
        state.send_replace(Transport {
            generation: 2,
            state: joined,
            message: None,
        });
        tokio::time::sleep(Duration::from_millis(30)).await;
        http.hold.store(false, Ordering::SeqCst);
        http.release.notify_waiters();
        outcome(&mut events, refresh::Outcome::Updated).await;
        assert!(http.reads.load(Ordering::SeqCst) >= 2);
        assert!(std::fs::read_to_string(&path).unwrap().contains("New run has a distinct stamp domain"));
        cancel.send_replace(true);
    };
    let (report, _) = tokio::join!(run, actions);
    report.unwrap();
}
