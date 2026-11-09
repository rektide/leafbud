//! Real loopback websocket acceptance of maintained client lifecycle, NOT deployment.
use futures_util::{SinkExt, StreamExt};
use leafbud_live_probe::join;
use phoenix_channel_client::{ChannelEvent, ClientError, DisconnectReason, Options, Socket, SocketEvent, static_join_payload};
use phoenix_channel_runtime::ProtocolEvent;
use phoenix_channel_runtime_native::{NativeConnector, NativeTimer};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::{
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};

async fn read(socket: &mut WebSocketStream<TcpStream>) -> Value {
    loop {
        let frame = timeout(Duration::from_secs(3), socket.next()).await.unwrap().unwrap().unwrap();
        if let Message::Text(text) = frame {
            return serde_json::from_str(&text).unwrap();
        }
    }
}

async fn reply(socket: &mut WebSocketStream<TcpStream>, frame: &Value, status: &str) {
    socket
        .send(Message::text(
            json!([frame[0], frame[1], frame[2], "phx_reply", {"status":status,"response":{}}]).to_string(),
        ))
        .await
        .unwrap();
}

fn options() -> Options {
    Options::default()
        .heartbeat_interval(Duration::from_millis(100))
        .heartbeat_timeout(Duration::from_millis(100))
        .request_timeout(Duration::from_millis(300))
        .reconnect_delay(|_| Duration::from_millis(5))
        .rejoin_delay(|_| Duration::from_millis(10))
}

#[tokio::test]
async fn managed_rejoins_after_real_socket_loss_and_leaves_correctly() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                let mut references = vec![];
                for epoch in 0..2 {
                    let (stream, _) = listener.accept().await.unwrap();
                    let mut socket = accept_async(stream).await.unwrap();
                    let frame = read(&mut socket).await;
                    assert_eq!(frame[3], "phx_join");
                    assert_eq!(frame[4]["config"]["presence"]["enabled"], true);
                    references.push(frame[0].clone());
                    reply(&mut socket, &frame, "ok").await;
                    socket
                        .send(Message::text(json!([frame[0], null, frame[2], "presence_state", {}]).to_string()))
                        .await
                        .unwrap();
                    socket
                        .send(Message::text(
                            json!([null,null,frame[2],"broadcast",{"type":"broadcast","event":"poke","payload":{}}]).to_string(),
                        ))
                        .await
                        .unwrap();
                    if epoch == 0 {
                        // Drop TCP without a normal-close frame; driver must recover.
                        continue;
                    }
                    loop {
                        let frame = read(&mut socket).await;
                        assert!(matches!(frame[3].as_str(), Some("heartbeat" | "phx_leave")));
                        reply(&mut socket, &frame, "ok").await;
                        if frame[3] == "phx_leave" {
                            break;
                        }
                    }
                }
                assert_ne!(references[0], references[1]);
            });
            let (socket, driver) = Socket::new(NativeConnector::new(format!("ws://{addr}/socket/websocket?vsn=2.0.0")), NativeTimer, options());
            let driver = tokio::task::spawn_local(driver);
            let channel = socket
                .channel(
                    "realtime:rootEntity:synthetic-root",
                    static_join_payload(join("", "", "synthetic-key", true)["payload"].clone()),
                )
                .unwrap();
            let mut events = channel.events().unwrap();
            channel.join().await.unwrap();
            let mut joins = 0;
            let mut pokes = 0;
            let mut states = 0;
            timeout(Duration::from_secs(4), async {
                while joins < 2 || pokes < 2 || states < 2 {
                    match events.next().await.unwrap() {
                        ChannelEvent::Protocol(ProtocolEvent::Joined { .. }) => joins += 1,
                        ChannelEvent::Protocol(ProtocolEvent::Message(frame)) if frame.event == "presence_state" => states += 1,
                        ChannelEvent::Protocol(ProtocolEvent::Message(frame)) if frame.event == "broadcast" => pokes += 1,
                        _ => (),
                    }
                }
            })
            .await
            .unwrap();
            socket.ping().await.unwrap();
            channel.leave().await.unwrap();
            socket.shutdown().await.unwrap();
            driver.await.unwrap();
            server.await.unwrap();
        })
        .await;
}

#[tokio::test]
async fn rejected_and_unacknowledged_joins_are_not_ready() {
    tokio::task::LocalSet::new()
        .run_until(async {
            for reject in [true, false] {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let addr = listener.local_addr().unwrap();
                let server = tokio::spawn(async move {
                    let (stream, _) = listener.accept().await.unwrap();
                    let mut socket = accept_async(stream).await.unwrap();
                    let frame = read(&mut socket).await;
                    if reject {
                        reply(&mut socket, &frame, "error").await;
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                });
                let (socket, driver) = Socket::new(
                    NativeConnector::new(format!("ws://{addr}/socket/websocket?vsn=2.0.0")),
                    NativeTimer,
                    options().heartbeat_interval(Duration::from_secs(5)),
                );
                let driver = tokio::task::spawn_local(driver);
                let channel = socket.channel("realtime:synthetic", static_join_payload(json!({}))).unwrap();
                let result = channel.join().await;
                if reject {
                    assert!(matches!(result, Err(ClientError::JoinRejected { .. })));
                } else {
                    assert!(matches!(result, Err(ClientError::Timeout { .. })));
                }
                socket.shutdown().await.unwrap();
                driver.await.unwrap();
                server.abort();
            }
        })
        .await;
}

#[tokio::test]
async fn missing_heartbeat_ack_triggers_reconnect() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut first = accept_async(stream).await.unwrap();
                let join = read(&mut first).await;
                reply(&mut first, &join, "ok").await;
                assert_eq!(read(&mut first).await[3], "heartbeat");
                // Do not acknowledge; keep the socket open until a reconnect arrives.
                let (stream, _) = listener.accept().await.unwrap();
                let mut second = accept_async(stream).await.unwrap();
                loop {
                    let frame = read(&mut second).await;
                    reply(&mut second, &frame, "ok").await;
                    if frame[3] == "phx_leave" {
                        break;
                    }
                }
            });
            let (socket, driver) = Socket::new(NativeConnector::new(format!("ws://{addr}/socket/websocket?vsn=2.0.0")), NativeTimer, options());
            let driver = tokio::task::spawn_local(driver);
            let mut lifecycle = socket.events().unwrap();
            let channel = socket.channel("realtime:synthetic", static_join_payload(json!({}))).unwrap();
            let mut events = channel.events().unwrap();
            channel.join().await.unwrap();
            timeout(Duration::from_secs(3), async {
                loop {
                    if matches!(
                        lifecycle.next().await,
                        Some(SocketEvent::Disconnected {
                            reason: DisconnectReason::HeartbeatTimeout
                        })
                    ) {
                        break;
                    }
                }
                let mut joins = 0;
                while joins < 2 {
                    if matches!(events.next().await, Some(ChannelEvent::Protocol(ProtocolEvent::Joined { .. }))) {
                        joins += 1;
                    }
                }
            })
            .await
            .unwrap();
            channel.leave().await.unwrap();
            socket.shutdown().await.unwrap();
            driver.await.unwrap();
            server.await.unwrap();
        })
        .await;
}

#[tokio::test]
async fn channel_error_rejoins_but_clean_server_close_needs_adapter_rejoin() {
    tokio::task::LocalSet::new()
        .run_until(async {
            for event in ["phx_error", "phx_close"] {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let addr = listener.local_addr().unwrap();
                let server = tokio::spawn(async move {
                    let (stream, _) = listener.accept().await.unwrap();
                    let mut socket = accept_async(stream).await.unwrap();
                    let first = read(&mut socket).await;
                    reply(&mut socket, &first, "ok").await;
                    socket
                        .send(Message::text(json!([first[0], null, first[2], event, {}]).to_string()))
                        .await
                        .unwrap();
                    let second = read(&mut socket).await;
                    assert_eq!(second[3], "phx_join");
                    assert_ne!(second[0], first[0]);
                    reply(&mut socket, &second, "ok").await;
                    loop {
                        let frame = read(&mut socket).await;
                        reply(&mut socket, &frame, "ok").await;
                        if frame[3] == "phx_leave" {
                            break;
                        }
                    }
                });
                let (socket, driver) = Socket::new(NativeConnector::new(format!("ws://{addr}/socket/websocket?vsn=2.0.0")), NativeTimer, options());
                let driver = tokio::task::spawn_local(driver);
                let channel = socket.channel("realtime:synthetic", static_join_payload(json!({}))).unwrap();
                let mut events = channel.events().unwrap();
                channel.join().await.unwrap();
                timeout(Duration::from_secs(3), async {
                    let mut joins = 0;
                    while joins < 2 {
                        match events.next().await.unwrap() {
                            ChannelEvent::Protocol(ProtocolEvent::Joined { .. }) => joins += 1,
                            ChannelEvent::Protocol(ProtocolEvent::ChannelClosed { .. }) => {
                                assert_eq!(event, "phx_close");
                                // Leafbud owns whether this unexpected close merits
                                // retry/backoff; generic Phoenix treats it as final.
                                channel.join().await.unwrap();
                            }
                            _ => (),
                        }
                    }
                })
                .await
                .unwrap();
                channel.leave().await.unwrap();
                socket.shutdown().await.unwrap();
                driver.await.unwrap();
                server.await.unwrap();
            }
        })
        .await;
}
