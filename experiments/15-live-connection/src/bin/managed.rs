//! Same passive deployment probe through a maintained Phoenix lifecycle client.
use leafbud_live_probe::join;
use phoenix_channel_client::{ChannelEvent, Options, Socket, static_join_payload};
use phoenix_channel_runtime::ProtocolEvent;
use phoenix_channel_runtime_native::{NativeConnector, NativeTimer};
use serde_json::Value;
use std::{path::Path, time::Duration};
use tokio::time::{Instant, timeout};

#[tokio::main]
async fn main() {
    let result = tokio::task::LocalSet::new().run_until(run()).await;
    if let Err(category) = result {
        eprintln!("managed_probe_failed category={category}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), &'static str> {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/live-connection");
    let boot: Value =
        serde_json::from_slice(&tokio::fs::read(scratch.join("bootstrap.json")).await.map_err(|_| "bootstrap-file")?).map_err(|_| "bootstrap-json")?;
    let binding: Value =
        serde_json::from_slice(&tokio::fs::read(scratch.join("binding.json")).await.map_err(|_| "binding-file")?).map_err(|_| "binding-json")?;
    let key = boot["key"].as_str().ok_or("bootstrap-key")?;
    let mut url = url::Url::parse(boot["url"].as_str().ok_or("bootstrap-url")?).map_err(|_| "bootstrap-url")?;
    url.set_scheme("wss").map_err(|_| "bootstrap-scheme")?;
    url.set_path("/realtime/v1/websocket");
    url.query_pairs_mut().append_pair("apikey", key).append_pair("vsn", "2.0.0");
    let root = binding["root"].as_str().ok_or("binding-root")?;
    let (socket, driver) = Socket::new(
        NativeConnector::new(url.as_str()),
        NativeTimer,
        Options::default()
            .heartbeat_interval(Duration::from_secs(10))
            .heartbeat_timeout(Duration::from_secs(8)),
    );
    let driver = tokio::task::spawn_local(driver);
    let durable = socket
        .channel(
            format!("realtime:rootEntity:{root}"),
            static_join_payload(join("", "", key, false)["payload"].clone()),
        )
        .map_err(|_| "durable-channel")?;
    let yjs = socket
        .channel(format!("realtime:yjs:{root}"), static_join_payload(join("", "", key, true)["payload"].clone()))
        .map_err(|_| "yjs-channel")?;
    let mut durable_events = durable.events().map_err(|_| "durable-events")?;
    let mut yjs_events = yjs.events().map_err(|_| "yjs-events")?;
    durable.join().await.map_err(|_| "durable-join")?;
    yjs.join().await.map_err(|_| "yjs-join")?;
    for epoch in 1..=2 {
        let mut snapshots = 0;
        let mut count = 0;
        let mut rejoined = [epoch == 1; 2];
        let deadline = Instant::now() + Duration::from_secs(32);
        let mut ping = tokio::time::interval(Duration::from_secs(10));
        let mut acks = 0;
        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(deadline) => break,
                _ = ping.tick(), if rejoined.into_iter().all(|joined| joined) => {
                    socket.ping().await.map_err(|_| "heartbeat-ack")?;
                    acks += 1;
                }
                event = durable_events.next() => {
                    match event.ok_or("durable-events-ended")? {
                        ChannelEvent::Protocol(ProtocolEvent::Joined { .. }) => rejoined[0] = true,
                        ChannelEvent::Lagged { .. } => return Err("durable-events-lagged"),
                        _ => (),
                    }
                }
                event = yjs_events.next() => {
                    match event.ok_or("yjs-events-ended")? {
                        ChannelEvent::Protocol(ProtocolEvent::Joined { .. }) => rejoined[1] = true,
                        ChannelEvent::Protocol(ProtocolEvent::Message(frame)) if frame.event == "presence_state" => {
                            let payload: Value = frame.payload.deserialize().map_err(|_| "presence-payload")?;
                            count = payload.as_object().ok_or("presence-state")?.values().map(|entry| entry["metas"].as_array().map_or(0, Vec::len)).sum::<usize>();
                            snapshots += 1;
                        }
                        ChannelEvent::Lagged { .. } => return Err("yjs-events-lagged"),
                        _ => (),
                    }
                }
            }
        }
        if !rejoined.into_iter().all(|joined| joined) || snapshots == 0 || acks < 2 {
            return Err("managed-liveness-not-proven");
        }
        println!(
            "managed_epoch={epoch} protocol=2.0.0 root_join=ok yjs_join=ok heartbeat_acks={acks} presence_snapshots={snapshots} initial_connection_metas={count} track_sent=false"
        );
        if epoch == 1 {
            socket.reconnect().await.map_err(|_| "reconnect-request")?;
        }
    }
    timeout(Duration::from_secs(10), durable.leave())
        .await
        .map_err(|_| "durable-leave-timeout")?
        .map_err(|_| "durable-leave")?;
    timeout(Duration::from_secs(10), yjs.leave())
        .await
        .map_err(|_| "yjs-leave-timeout")?
        .map_err(|_| "yjs-leave")?;
    socket.shutdown().await.map_err(|_| "shutdown")?;
    driver.await.map_err(|_| "driver-join")?;
    println!("managed_leaves=ok managed_shutdown=ok leaflet_cookie_on_socket=false");
    Ok(())
}
