//! Bounded passive deployed protocol probe. No track, hello, content or cursor sends.
use futures_util::{SinkExt, StreamExt};
use leafbud_live_probe::{Presence, control, join};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeSet, path::Path, time::Duration};
use tokio::time::{Instant, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Deserialize)]
struct Bootstrap {
    url: String,
    key: String,
}
#[derive(Deserialize)]
struct Binding {
    root: String,
}

#[tokio::main]
async fn main() {
    // tokio-tungstenite's rustls transport does not select a crypto provider.
    // Respect a provider already installed by the embedding application.
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    // Deliberately redact underlying URL/message-bearing errors at this boundary.
    if let Err(category) = run().await {
        eprintln!("probe_failed category={category}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), &'static str> {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/live-connection");
    let bootstrap: Bootstrap =
        serde_json::from_slice(&tokio::fs::read(scratch.join("bootstrap.json")).await.map_err(|_| "bootstrap-file")?).map_err(|_| "bootstrap-json")?;
    let binding: Binding =
        serde_json::from_slice(&tokio::fs::read(scratch.join("binding.json")).await.map_err(|_| "binding-file")?).map_err(|_| "binding-json")?;
    let mut url = url::Url::parse(&bootstrap.url).map_err(|_| "bootstrap-url")?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("bootstrap-origin");
    }
    url.set_scheme("wss").map_err(|_| "websocket-scheme")?;
    url.set_path("/realtime/v1/websocket");
    url.query_pairs_mut().append_pair("apikey", &bootstrap.key).append_pair("vsn", "1.0.0");
    let topics = [format!("realtime:rootEntity:{}", binding.root), format!("realtime:yjs:{}", binding.root)];
    // One bounded fresh connection then another fresh connection: tests actual
    // rejoin/bootstrap reuse, not automatic reconnect scheduling or content repair.
    for epoch in 1..=2 {
        let (mut socket, _) = timeout(Duration::from_secs(15), connect_async(url.as_str()))
            .await
            .map_err(|_| "handshake-timeout")?
            .map_err(|_| "handshake")?;
        let mut reference = 0u64;
        let mut next_ref = || {
            reference += 1;
            reference.to_string()
        };
        let mut join_refs = vec![];
        for (index, topic) in topics.iter().enumerate() {
            let r = next_ref();
            socket
                .send(Message::text(join(topic, &r, &bootstrap.key, index == 1).to_string()))
                .await
                .map_err(|_| "join-write")?;
            join_refs.push(r);
        }
        let mut joined = BTreeSet::new();
        let mut presence = Presence::default();
        let mut presence_frames = 0;
        let mut pokes = 0;
        let mut broadcasts = 0;
        let mut heartbeat: Option<(String, Instant)> = None;
        let mut heartbeat_acks = 0;
        let start = Instant::now();
        let mut next_heartbeat = start + Duration::from_secs(10);
        let until = start + Duration::from_secs(36);
        loop {
            let now = Instant::now();
            if now >= until {
                break;
            }
            if joined.len() < 2 && now.duration_since(start) > Duration::from_secs(10) {
                return Err("join-ack-timeout");
            }
            if heartbeat.as_ref().is_some_and(|(_, sent)| now.duration_since(*sent) > Duration::from_secs(8)) {
                return Err("heartbeat-ack-timeout");
            }
            if now >= next_heartbeat && heartbeat.is_none() {
                let r = next_ref();
                socket
                    .send(Message::text(control("phoenix", "heartbeat", &r, None).to_string()))
                    .await
                    .map_err(|_| "heartbeat-write")?;
                heartbeat = Some((r, now));
                next_heartbeat = now + Duration::from_secs(10);
            }
            let frame = match timeout(Duration::from_millis(200), socket.next()).await {
                Err(_) => continue,
                Ok(Some(Ok(Message::Text(text)))) => text,
                Ok(Some(Ok(Message::Ping(_)))) => {
                    socket.flush().await.map_err(|_| "pong-write")?;
                    continue;
                }
                Ok(Some(Ok(Message::Close(_)))) | Ok(None) => return Err("unexpected-close"),
                Ok(Some(Err(_))) => return Err("socket-read"),
                _ => continue,
            };
            let message: Value = serde_json::from_str(&frame).map_err(|_| "frame-json")?;
            let topic = message["topic"].as_str().ok_or("frame-topic")?;
            let event = message["event"].as_str().ok_or("frame-event")?;
            let r = message["ref"].as_str();
            match event {
                "phx_reply" if topic == "phoenix" => {
                    if heartbeat.as_ref().is_some_and(|(pending, _)| Some(pending.as_str()) == r) && message["payload"]["status"] == "ok" {
                        heartbeat = None;
                        heartbeat_acks += 1;
                    }
                }
                "phx_reply" => {
                    if let Some(index) = topics.iter().position(|t| t == topic)
                        && r == Some(join_refs[index].as_str())
                    {
                        if message["payload"]["status"] != "ok" {
                            return Err("join-rejected");
                        }
                        joined.insert(index);
                    }
                }
                "presence_state" if topic == topics[1] => {
                    presence.snapshot(&message["payload"])?;
                    presence_frames += 1;
                }
                "presence_diff" if topic == topics[1] => {
                    presence.diff(&message["payload"])?;
                    presence_frames += 1;
                }
                "broadcast" if topic == topics[0] && message["payload"]["event"] == "poke" => pokes += 1,
                "broadcast" if topic == topics[1] => broadcasts += 1,
                "phx_error" | "phx_close" => return Err("channel-closed"),
                _ => (),
            }
        }
        if joined.len() != 2 || heartbeat_acks < 2 {
            return Err("liveness-not-proven");
        }
        // Correct Phoenix client leave, not server-originated phx_close.
        let mut leave_refs = BTreeSet::new();
        for (index, topic) in topics.iter().enumerate() {
            let r = next_ref();
            socket
                .send(Message::text(control(topic, "phx_leave", &r, Some(&join_refs[index])).to_string()))
                .await
                .map_err(|_| "leave-write")?;
            leave_refs.insert(r);
        }
        let leave_deadline = Instant::now() + Duration::from_secs(5);
        while !leave_refs.is_empty() {
            let frame = timeout(leave_deadline.saturating_duration_since(Instant::now()), socket.next())
                .await
                .map_err(|_| "leave-ack-timeout")?
                .ok_or("leave-socket-closed")?
                .map_err(|_| "leave-read")?;
            if let Message::Text(text) = frame {
                let message: Value = serde_json::from_str(&text).map_err(|_| "leave-json")?;
                if message["event"] == "phx_reply"
                    && message["payload"]["status"] == "ok"
                    && let Some(r) = message["ref"].as_str()
                {
                    leave_refs.remove(r);
                }
            }
        }
        socket.close(None).await.map_err(|_| "close-write")?;
        println!(
            "epoch={epoch} root_join=ok yjs_join=ok heartbeat_acks={heartbeat_acks} passive_presence_ready={} observed_connection_metas={} presence_frames={presence_frames} durable_pokes={pokes} yjs_broadcasts={broadcasts} leaves=ok leaflet_cookie_on_socket=false track_sent=false",
            presence.ready,
            presence.count()
        );
        presence.gap();
    }
    Ok(())
}
