//! Offline production-adapter tests. No remote documents or fabricated peers on
//! the real service; all nonempty presence here is explicitly synthetic.
use super::*;
use futures_util::SinkExt;
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    task::JoinHandle,
    time::timeout,
};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        Message,
        handshake::server::{Request, Response},
    },
};

fn root(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn options() -> Options {
    Options {
        heartbeat_interval: Duration::from_millis(80),
        ack_timeout: Duration::from_millis(180),
        connect_timeout: Duration::from_millis(180),
        leave_timeout: Duration::from_millis(120),
        backoff_min: Duration::from_millis(20),
        backoff_max: Duration::from_millis(100),
        presence: true,
        ..Options::default()
    }
}

enum WireCommand {
    Send(Value),
    Burst(Vec<Value>),
    Drop,
}
struct Server {
    origin: String,
    command: mpsc::UnboundedSender<WireCommand>,
    frames: mpsc::UnboundedReceiver<(usize, Value)>,
    task: JoinHandle<()>,
    sent: Arc<Mutex<Vec<Value>>>,
}

impl Server {
    // Tungstenite's handshake callback requires its concrete HTTP error response.
    #[allow(clippy::result_large_err)]
    async fn new(reject_presence: bool, ignore_first_join: bool, heartbeat: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let (command, mut commands) = mpsc::unbounded_channel();
        let (frames_tx, frames) = mpsc::unbounded_channel();
        let sent = Arc::new(Mutex::new(Vec::new()));
        let recorded = sent.clone();
        let task = tokio::spawn(async move {
            let mut ignored = false;
            let mut epoch = 0;
            loop {
                let (tcp, _) = listener.accept().await.unwrap();
                epoch += 1;
                let mut socket = match accept_hdr_async(tcp, |request: &Request, response: Response| {
                    assert!(request.headers().get("cookie").is_none());
                    assert!(request.headers().get("authorization").is_none());
                    assert!(request.uri().path().ends_with("/realtime/v1/websocket"));
                    Ok(response)
                })
                .await
                {
                    Ok(socket) => socket,
                    Err(_) => continue,
                };
                loop {
                    tokio::select! {
                        next = socket.next() => {
                            let Some(Ok(Message::Text(text))) = next else { break };
                            let frame: Value = serde_json::from_str(&text).unwrap();
                            recorded.lock().unwrap().push(frame.clone());
                            assert!(matches!(frame[3].as_str(), Some("phx_join" | "phx_leave" | "heartbeat")), "passive adapter sent application data");
                            frames_tx.send((epoch, frame.clone())).unwrap();
                            if frame[3] == "phx_join" {
                                assert_eq!(frame[4]["config"]["presence"]["enabled"], frame[2].as_str().unwrap().contains(":yjs:"));
                                assert_eq!(frame[4]["config"]["private"], false);
                                if ignore_first_join && !ignored { ignored = true; continue; }
                            }
                            if frame[3] == "heartbeat" && !heartbeat { continue; }
                            let reject = reject_presence && frame[3] == "phx_join" && frame[2].as_str().unwrap().contains(":yjs:");
                            let response = json!([frame[0],frame[1],frame[2],"phx_reply",{"status":if reject {"error"} else {"ok"},"response":if reject {json!({"reason":"Invalid JWT private-payload"})} else {json!({})}}]);
                            if socket.send(Message::text(response.to_string())).await.is_err() { break; }
                        },
                        command = commands.recv() => match command {
                            Some(WireCommand::Send(frame)) => { if socket.send(Message::text(frame.to_string())).await.is_err() { break; } },
                            Some(WireCommand::Burst(frames)) => {
                                for frame in frames { if socket.feed(Message::text(frame.to_string())).await.is_err() { break; } }
                                if socket.flush().await.is_err() { break; }
                            },
                            Some(WireCommand::Drop) => break,
                            None => return,
                        }
                    }
                }
            }
        });
        Self {
            origin,
            command,
            frames,
            task,
            sent,
        }
    }

    fn bootstrap(&self) -> Bootstrap {
        Bootstrap::explicit(
            "https://leaflet.example",
            PublicProject::new(&self.origin, &bootstrap::test_key("synthetic")).unwrap(),
        )
        .unwrap()
    }
    async fn frame(&mut self, event: &str, topic: Option<&str>) -> (usize, Value) {
        timeout(Duration::from_secs(4), async {
            loop {
                let frame = self.frames.recv().await.unwrap();
                if frame.1[3] == event && topic.is_none_or(|topic| frame.1[2] == topic) {
                    return frame;
                }
            }
        })
        .await
        .unwrap()
    }
    fn send(&self, frame: Value) {
        self.command.send(WireCommand::Send(frame)).unwrap();
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

struct Adapter {
    command: mpsc::Sender<Command>,
    state: watch::Receiver<State>,
    task: JoinHandle<Result<(), Error>>,
}
impl Adapter {
    fn start(bootstrap: Bootstrap, options: Options) -> Self {
        let (command, commands) = mpsc::channel(8);
        let (sender, state) = watch::channel(State::default());
        let task = tokio::task::spawn_local(run(bootstrap, options, commands, sender));
        Self { command, state, task }
    }
    async fn roots(&self, roots: Vec<Uuid>) {
        self.command.send(Command::SetRoots(roots)).await.unwrap();
    }
    async fn wait(&mut self, predicate: impl Fn(&State) -> bool) -> State {
        timeout(Duration::from_secs(5), async {
            loop {
                let state = self.state.borrow_and_update().clone();
                if predicate(&state) {
                    return state;
                }
                self.state.changed().await.unwrap();
            }
        })
        .await
        .expect("adapter state deadline")
    }
    async fn stop(self) {
        self.command.send(Command::Stop).await.unwrap();
        timeout(Duration::from_secs(2), self.task).await.unwrap().unwrap().unwrap();
        assert_eq!(self.state.borrow().health, Health::Stopped);
    }
}

fn name(root: Uuid, presence: bool) -> String {
    format!("realtime:{}:{root}", if presence { "yjs" } else { "rootEntity" })
}
fn event(join: &Value, event: &str, payload: Value) -> Value {
    json!([join[0], null, join[2], event, payload])
}
fn poke(join: &Value) -> Value {
    event(
        join,
        "broadcast",
        json!({"event":"poke","type":"broadcast","payload":{"private":"not retained"}}),
    )
}

#[tokio::test]
async fn joins_poke_bursts_pending_receiver_and_root_replacement() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let mut server = Server::new(false, false, true).await;
            let mut adapter = Adapter::start(server.bootstrap(), options());
            adapter.roots(vec![root(1), root(1)]).await;
            let (_, join) = server.frame("phx_join", Some(&name(root(1), false))).await;
            let ready = adapter.wait(|s| s.health == Health::Live).await;
            let captured = ready.roots[&root(1)].observation_seq;
            assert!(ready.roots[&root(1)].join_epoch > 0);
            // Deliberately do not receive state while a hypothetical HTTP refresh is
            // pending. Three pokes must survive watch's latest-state coalescing.
            server.command.send(WireCommand::Burst(vec![poke(&join); 3])).unwrap();
            let dirty = adapter.wait(|s| s.roots[&root(1)].observation_seq >= captured + 3).await;
            assert!(dirty.roots[&root(1)].observation_seq > captured);
            assert_eq!(dirty.roots[&root(1)].join_epoch, ready.roots[&root(1)].join_epoch);
            adapter.roots(vec![root(2)]).await;
            server.frame("phx_leave", Some(&name(root(1), false))).await;
            server.frame("phx_join", Some(&name(root(2), false))).await;
            let state = adapter.wait(|s| s.health == Health::Live && s.roots.contains_key(&root(2))).await;
            assert!(!state.roots.contains_key(&root(1)));
            assert!(state.roots[&root(2)].observation_seq > captured);
            server.send(poke(&join)); // deselected old generation cannot reappear
            adapter.roots(vec![]).await;
            adapter.wait(|s| s.health == Health::Idle && s.roots.is_empty()).await;
            adapter.stop().await;
        })
        .await;
}

#[tokio::test]
async fn disconnect_close_error_and_stale_frames_recover_with_new_join_epochs() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let mut server = Server::new(false, false, true).await;
            let mut adapter = Adapter::start(server.bootstrap(), Options { presence: false, ..options() });
            adapter.roots(vec![root(3)]).await;
            let mut join = server.frame("phx_join", None).await.1;
            let mut previous = adapter.wait(|s| s.health == Health::Live).await.roots[&root(3)].clone();
            for close in [Some("phx_error"), Some("phx_close"), None] {
                if let Some(close) = close {
                    server.send(event(&join, close, json!({})));
                } else {
                    server.command.send(WireCommand::Drop).unwrap();
                }
                let old = join.clone();
                join = server.frame("phx_join", None).await.1;
                assert_ne!(old[0], join[0]);
                let state = adapter
                    .wait(|s| s.health == Health::Live && s.roots[&root(3)].join_epoch > previous.join_epoch)
                    .await;
                previous = state.roots[&root(3)].clone();
                server.send(poke(&old)); // library reports StaleMessage, never a poke
                // Ordered heartbeat acknowledgement gives the adapter a chance to
                // drain the preceding stale frame before we inspect retained state.
                server.frame("heartbeat", None).await;
                tokio::time::sleep(Duration::from_millis(10)).await;
                assert_eq!(adapter.state.borrow().roots[&root(3)].observation_seq, previous.observation_seq);
                server.send(poke(&join));
                adapter.wait(|s| s.roots[&root(3)].observation_seq > previous.observation_seq).await;
            }
            adapter.stop().await;
            server.frame("phx_leave", None).await;
        })
        .await;
}

#[tokio::test]
async fn presence_is_passive_ref_aware_transactional_and_independent() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let mut server = Server::new(false, false, true).await;
            let mut adapter = Adapter::start(server.bootstrap(), options());
            adapter.roots(vec![root(4)]).await;
            let (_, yjs) = server.frame("phx_join", Some(&name(root(4), true))).await;
            adapter.wait(|s| s.health == Health::Live).await;
            assert_eq!(
                adapter.state.borrow().roots[&root(4)].presence.as_ref().unwrap().evidence,
                PresenceEvidence::Unknown
            );
            server.send(event(
                &yjs,
                "presence_diff",
                json!({"joins":{"same":{"metas":[{"phx_ref":"unready"}]}},"leaves":{}}),
            ));
            server.send(event(&yjs, "presence_state", json!({"same":{"metas":[{"phx_ref":"a"},{"phx_ref":"b"}]}})));
            adapter
                .wait(|s| s.roots[&root(4)].presence.as_ref().unwrap().evidence == PresenceEvidence::ObservedPeers(2))
                .await;
            server.send(event(
                &yjs,
                "presence_diff",
                json!({"joins":{"same":{"metas":[{"phx_ref":"c","phx_ref_prev":"a"}]}},"leaves":{"same":{"metas":[{"phx_ref":"b"}]}}}),
            ));
            let state = adapter
                .wait(|s| s.roots[&root(4)].presence.as_ref().unwrap().evidence == PresenceEvidence::ObservedPeers(1))
                .await;
            let durable = state.roots[&root(4)].observation_seq;
            assert!(state.roots[&root(4)].presence.as_ref().unwrap().observed_at.is_some());
            // A valid join followed by malformed leaves must not publish peers(2).
            server.send(event(
                &yjs,
                "presence_diff",
                json!({"joins":{"same":{"metas":[{"phx_ref":"d"}]}},"leaves":{"broken":{"metas":[{}]}}}),
            ));
            adapter
                .wait(|s| s.roots[&root(4)].presence.as_ref().unwrap().evidence == PresenceEvidence::Unknown)
                .await;
            assert_eq!(adapter.state.borrow().roots[&root(4)].observation_seq, durable);
            let (_, fresh) = server.frame("phx_join", Some(&name(root(4), true))).await;
            assert_ne!(fresh[0], yjs[0]);
            server.send(event(&fresh, "presence_state", json!({})));
            adapter
                .wait(|s| s.roots[&root(4)].presence.as_ref().unwrap().evidence == PresenceEvidence::NoPeersObserved)
                .await;
            assert_eq!(adapter.state.borrow().health, Health::Live);
            adapter.stop().await;
        })
        .await;
}

#[tokio::test]
async fn terminal_presence_rejection_does_not_disable_durable_subscription() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let mut server = Server::new(true, false, true).await;
            let mut adapter = Adapter::start(server.bootstrap(), options());
            adapter.roots(vec![root(5)]).await;
            let (_, durable) = server.frame("phx_join", Some(&name(root(5), false))).await;
            let state = adapter
                .wait(|s| {
                    s.roots.get(&root(5)).is_some_and(|r| {
                        r.subscription == SubscriptionState::Joined && matches!(r.presence.as_ref().unwrap().subscription, SubscriptionState::Unavailable(_))
                    })
                })
                .await;
            assert_eq!(state.health, Health::Live);
            let sequence = state.roots[&root(5)].observation_seq;
            server.send(poke(&durable));
            adapter.wait(|s| s.roots[&root(5)].observation_seq > sequence).await;
            tokio::time::sleep(Duration::from_millis(250)).await;
            let yjs_joins = server
                .sent
                .lock()
                .unwrap()
                .iter()
                .filter(|frame| frame[3] == "phx_join" && frame[2] == name(root(5), true))
                .count();
            assert_eq!(yjs_joins, 1, "terminal rejection must cancel generic-client retries");
            adapter.stop().await;
        })
        .await;
}

#[tokio::test]
async fn timed_out_join_and_heartbeat_timeout_recover_without_false_readiness() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let mut server = Server::new(false, true, true).await;
            let mut adapter = Adapter::start(server.bootstrap(), Options { presence: false, ..options() });
            adapter.roots(vec![root(6)]).await;
            server.frame("phx_join", None).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
            assert_ne!(adapter.state.borrow().health, Health::Live);
            assert_eq!(adapter.state.borrow().roots[&root(6)].join_epoch, 0);
            server.frame("phx_join", None).await;
            adapter.wait(|s| s.health == Health::Live).await;
            adapter.stop().await;

            let mut server = Server::new(false, false, false).await;
            let mut adapter = Adapter::start(server.bootstrap(), Options { presence: false, ..options() });
            adapter.roots(vec![root(6)]).await;
            let first = adapter.wait(|s| s.health == Health::Live).await.roots[&root(6)].join_epoch;
            server.frame("phx_join", None).await;
            let (epoch, _) = server.frame("phx_join", None).await;
            assert!(epoch > 1);
            adapter.wait(|s| s.health == Health::Live && s.roots[&root(6)].join_epoch > first).await;
            adapter.stop().await;
        })
        .await;
}

#[tokio::test]
async fn immediate_reselection_waits_for_leave_and_preserves_monotonicity() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let mut server = Server::new(false, false, true).await;
            let mut adapter = Adapter::start(server.bootstrap(), Options { presence: false, ..options() });
            adapter.roots(vec![root(7)]).await;
            server.frame("phx_join", None).await;
            let old = adapter.wait(|s| s.health == Health::Live).await.roots[&root(7)].observation_seq;
            adapter.roots(vec![]).await;
            adapter.roots(vec![root(7)]).await;
            server.frame("phx_leave", None).await;
            server.frame("phx_join", None).await;
            adapter.wait(|s| s.health == Health::Live && s.roots[&root(7)].observation_seq > old).await;
            adapter.stop().await;
        })
        .await;
}

#[tokio::test]
async fn lag_requires_new_acknowledged_join_and_deselection_cancels_close_retry() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let mut server = Server::new(false, false, true).await;
            let mut adapter = Adapter::start(
                server.bootstrap(),
                Options {
                    presence: false,
                    event_capacity: 2,
                    ..options()
                },
            );
            adapter.roots(vec![root(9)]).await;
            let (_, join) = server.frame("phx_join", None).await;
            let epoch = adapter.wait(|s| s.health == Health::Live).await.roots[&root(9)].join_epoch;
            server.command.send(WireCommand::Burst(vec![poke(&join); 512])).unwrap();
            // The upstream bounded-stream implementation reports accumulated loss
            // on the next send after the receiver drains, not out of band.
            tokio::time::sleep(Duration::from_millis(50)).await;
            server.send(poke(&join));
            // Overflow the real library's bounded stream. The adapter must not just
            // count the few surviving messages and silently remain on that epoch.
            let (_, joined_again) = server.frame("phx_join", None).await;
            let refreshed = adapter.wait(|s| s.health == Health::Live && s.roots[&root(9)].join_epoch > epoch).await;
            assert!(refreshed.roots[&root(9)].observation_seq > epoch);
            server.send(event(&joined_again, "phx_close", json!({})));
            adapter.wait(|s| s.roots[&root(9)].subscription == SubscriptionState::Reconnecting).await;
            adapter.roots(vec![]).await;
            adapter.wait(|s| s.roots.is_empty()).await;
            tokio::time::sleep(Duration::from_millis(250)).await;
            while let Ok((_, frame)) = server.frames.try_recv() {
                assert_ne!(frame[3], "phx_join", "deselected retry resurrected a root");
            }
            adapter.stop().await;
        })
        .await;
}

#[tokio::test]
async fn durable_terminal_system_error_is_unavailable_without_rejoin_storm() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let mut server = Server::new(false, false, true).await;
            let mut adapter = Adapter::start(server.bootstrap(), Options { presence: false, ..options() });
            adapter.roots(vec![root(10)]).await;
            let (_, join) = server.frame("phx_join", None).await;
            adapter.wait(|s| s.health == Health::Live).await;
            server.send(event(&join, "system", json!({"status":"error","message":"Invalid JWT private-payload"})));
            adapter
                .wait(|s| s.health == Health::Degraded && s.roots[&root(10)].subscription == SubscriptionState::Unavailable(Unavailable::AuthOrConfig))
                .await;
            // Idempotent selection does not implicitly retry a terminal config.
            adapter.roots(vec![root(10)]).await;
            tokio::time::sleep(Duration::from_millis(250)).await;
            while let Ok((_, frame)) = server.frames.try_recv() {
                assert_ne!(frame[3], "phx_join");
            }
            adapter.stop().await;
        })
        .await;
}

#[tokio::test]
async fn cancellation_during_handshake_needs_no_localset_and_is_bounded() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let project = PublicProject::new(&format!("http://{}", listener.local_addr().unwrap()), &bootstrap::test_key("synthetic")).unwrap();
    let bootstrap = Bootstrap::explicit("https://leaflet.example", project).unwrap();
    let (commands, receiver) = mpsc::channel(4);
    let (state, observed) = watch::channel(State::default());
    let controller = async {
        commands.send(Command::SetRoots(vec![root(8)])).await.unwrap();
        let (_tcp, _) = listener.accept().await.unwrap(); // deliberately no upgrade response
        commands.send(Command::Stop).await.unwrap();
    };
    let (result, ()) = timeout(Duration::from_secs(2), async {
        tokio::join!(run(bootstrap, options(), receiver, state), controller)
    })
    .await
    .unwrap();
    result.unwrap();
    assert_eq!(observed.borrow().health, Health::Stopped);
}

#[test]
fn counters_stale_selection_presence_transaction_and_diagnostics_are_bounded() {
    let (sender, _) = watch::channel(State::default());
    let mut model = Model {
        state: State::default(),
        sender,
        generations: BTreeMap::new(),
        sequence: u64::MAX,
    };
    assert_eq!(model.stamp(), Err(Error::SequenceExhausted));
    model.update((root(99), Kind::Durable), 42, Update::Poke).unwrap(); // stale selection ignored even at exhaustion
    assert_eq!(model.sequence, u64::MAX);
    let mut presence = Presence::default();
    presence.snapshot(&json!({"k":{"metas":[{"phx_ref":"a"}]}})).unwrap();
    assert!(
        presence
            .diff(&json!({"joins":{"k":{"metas":[{"phx_ref":"b"}]}},"leaves":{"x":{"metas":[{}]}}}))
            .is_err()
    );
    assert_eq!(presence.diff(&json!({"joins":{},"leaves":{}})).unwrap(), Some(1));
    presence.gap();
    assert_eq!(presence.diff(&json!({"joins":{},"leaves":{}})).unwrap(), None);
    let project = PublicProject::new("https://private-project.supabase.co", &bootstrap::test_key("private-project")).unwrap();
    let boot = Bootstrap::explicit("https://private-leaflet.example", project.clone()).unwrap();
    for debug in [
        format!("{project:?}"),
        format!("{boot:?}"),
        format!("{:?}", Command::SetRoots(vec![root(99)])),
        format!("{:?}", State::default()),
    ] {
        assert!(!debug.contains("private-project"));
        assert!(!debug.contains("private-leaflet"));
        assert!(!debug.contains("eyJ"));
        assert!(!debug.contains(&root(99).to_string()));
    }
    assert_eq!(boot.require_origin("https://other.example"), Err(Error::OriginMismatch));
    assert_eq!(
        PublicProject::new("https://other.supabase.co", &bootstrap::test_key("private-project")).unwrap_err(),
        Error::ProjectMismatch
    );
    assert_eq!(PublicProject::new("https://example.invalid/path?secret", "secret").unwrap_err(), Error::Origin);
    assert!(
        Options {
            heartbeat_interval: Duration::from_secs(30),
            ..Options::default()
        }
        .validate()
        .is_err()
    );
    fn send_sync<T: Send + Sync>() {}
    send_sync::<State>();
    send_sync::<Command>();
    send_sync::<Bootstrap>();
}

// A dedicated HTTP fixture keeps MIME and redirects explicit without changing
// the shared JSON-only server owned by other test suites.
struct Assets {
    origin: String,
    requests: Arc<Mutex<Vec<String>>>,
    task: JoinHandle<()>,
}
impl Assets {
    async fn start(page: String, scripts: BTreeMap<String, (u16, String, String)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let task = tokio::spawn(async move {
            loop {
                let (tcp, _) = listener.accept().await.unwrap();
                let mut stream = BufReader::new(tcp);
                let mut line = String::new();
                stream.read_line(&mut line).await.unwrap();
                let path = line.split_whitespace().nth(1).unwrap().to_owned();
                loop {
                    line.clear();
                    stream.read_line(&mut line).await.unwrap();
                    assert!(!line.to_ascii_lowercase().starts_with("cookie:"));
                    if line == "\r\n" {
                        break;
                    }
                }
                recorded.lock().unwrap().push(path.clone());
                let (status, mime, body) = if path == "/" {
                    (200, "text/html".to_owned(), page.clone())
                } else {
                    scripts.get(&path).cloned().unwrap_or((404, "text/plain".to_owned(), String::new()))
                };
                let response = format!(
                    "HTTP/1.1 {status} Test\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nLocation: https://foreign.invalid/private\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.get_mut().write_all(response.as_bytes()).await.unwrap();
            }
        });
        Self { origin, requests, task }
    }
}
impl Drop for Assets {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[tokio::test]
async fn public_bootstrap_is_cookie_free_same_origin_bounded_and_unambiguous() {
    let key = bootstrap::test_key("synthetic-project");
    let page = "<script src='/_next/static/a.js'></script><script src=\"/_next/static/b.js\"></script><script src='https://foreign.invalid/_next/static/never.js'></script><script src='/not-next/never.js'></script>";
    // URL and key may be in separate directly referenced assets. Matching is
    // across the bounded set; discovery never executes or follows JS imports.
    let server = Assets::start(
        page.to_owned(),
        BTreeMap::from([
            (
                "/_next/static/a.js".to_owned(),
                (200, "text/javascript".to_owned(), "https://synthetic-project.supabase.co".to_owned()),
            ),
            ("/_next/static/b.js".to_owned(), (200, "application/javascript".to_owned(), key)),
        ]),
    )
    .await;
    let client = crate::LeafletClient::new(&server.origin, Some("private-cookie")).unwrap();
    let boot = client.live_bootstrap(None).await.unwrap();
    boot.require_origin(&server.origin).unwrap();
    assert_eq!(server.requests.lock().unwrap().len(), 3);
    assert_eq!(boot.project.key, bootstrap::test_key("synthetic-project"));
    drop(server);

    for (script, status, expected) in [
        (
            format!(
                "https://one.supabase.co {} https://two.supabase.co {}",
                bootstrap::test_key("one"),
                bootstrap::test_key("two")
            ),
            200,
            Error::DiscoveryAmbiguous,
        ),
        ("no public configuration".to_owned(), 200, Error::DiscoveryMissing),
        ("redirect body private".to_owned(), 302, Error::BootstrapHttp(302)),
        ("x".repeat(8 * 1024 * 1024 + 1), 200, Error::BootstrapLimit),
    ] {
        let assets = Assets::start(
            "<script src='/_next/static/a.js'></script>".to_owned(),
            BTreeMap::from([("/_next/static/a.js".to_owned(), (status, "text/javascript".to_owned(), script))]),
        )
        .await;
        assert_eq!(Bootstrap::resolve(&assets.origin, None).await.unwrap_err(), expected);
    }
    let assets = Assets::start("<script src='/_next/static/a.js'></script>".repeat(49), BTreeMap::new()).await;
    assert_eq!(Bootstrap::resolve(&assets.origin, None).await.unwrap_err(), Error::BootstrapLimit);
    assert_eq!(assets.requests.lock().unwrap().len(), 1);
}
