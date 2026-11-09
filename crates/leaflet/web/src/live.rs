//! Receive-only, origin-scoped Leaflet realtime transport.
//!
//! Poll [`run`] concurrently with the controller (e.g. `tokio::select!`/`join!`).
//! Its Phoenix driver is intentionally local/`!Send`; no LocalSet or worker thread
//! is needed unless the caller chooses to spawn it. Control/state channels are
//! `Send`. Keep polling while HTTP/file work runs. A poke is untrusted invalidation,
//! never content, permission or agreement evidence.
//!
//! Read the latest state on attachment, then compare `observation_seq` at refresh
//! start/completion. Only mark the captured start sequence serviced. Coalesced
//! watch notifications cannot erase a poke that arrived during a refresh.

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
    sync::Arc,
    time::{Duration, SystemTime},
};

use futures_util::{StreamExt, future::LocalBoxFuture, stream::FuturesUnordered};
use phoenix_channel_client::{Channel, ChannelEvent, ClientError, DisconnectReason, ReconnectAction, Socket, SocketEvent, static_join_payload};
use phoenix_channel_runtime::{Payload, ProtocolEvent};
use phoenix_channel_runtime_native::{NativeConnector, NativeTimer, NativeTransportOptions};
use rustls_platform_verifier::BuilderVerifierExt;
use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot, watch};
use uuid::Uuid;

mod bootstrap;
mod presence;
pub use bootstrap::{Bootstrap, PublicProject};
use presence::Presence;

/// All diagnostics are fixed categories. Underlying library errors can contain
/// socket URLs, keys, topic IDs and payloads and deliberately never escape here.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("realtime origin must be HTTPS (or loopback HTTP), with no credentials, path, query or fragment")]
    Origin,
    #[error("realtime bootstrap belongs to a different Leaflet origin")]
    OriginMismatch,
    #[error("invalid realtime public key; require a public anon JWT for this join protocol")]
    PublicKey,
    #[error("realtime public key and project do not match")]
    ProjectMismatch,
    #[error("public realtime bootstrap request failed")]
    BootstrapTransport,
    #[error("public realtime bootstrap returned HTTP {0}")]
    BootstrapHttp(u16),
    #[error("public realtime bootstrap exceeded its deadline")]
    BootstrapTimeout,
    #[error("public realtime bootstrap exceeded a size/script limit")]
    BootstrapLimit,
    #[error("public realtime bootstrap has an unsupported response shape")]
    BootstrapShape,
    #[error("public realtime project not found; supply an explicit project override")]
    DiscoveryMissing,
    #[error("public realtime discovery is ambiguous; supply an explicit project override")]
    DiscoveryAmbiguous,
    #[error("invalid realtime options")]
    Options,
    #[error("realtime selection exceeds its root limit or contains a nil root")]
    Roots,
    #[error("realtime observation sequence exhausted; restart the session")]
    SequenceExhausted,
    #[error("realtime driver stopped unexpectedly")]
    DriverStopped,
    #[error("realtime channel could not be created")]
    Channel,
    #[error("realtime TLS configuration could not be initialized")]
    Tls,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub presence: bool,
    /// Must be nonzero and at most 25 seconds (Supabase's liveness requirement).
    pub heartbeat_interval: Duration,
    pub ack_timeout: Duration,
    pub connect_timeout: Duration,
    pub leave_timeout: Duration,
    pub backoff_min: Duration,
    pub backoff_max: Duration,
    pub max_roots: usize,
    pub event_capacity: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            presence: false,
            heartbeat_interval: Duration::from_secs(20),
            ack_timeout: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(10),
            leave_timeout: Duration::from_secs(3),
            backoff_min: Duration::from_millis(500),
            backoff_max: Duration::from_secs(15),
            max_roots: 256,
            event_capacity: 256,
        }
    }
}

impl Options {
    fn validate(&self) -> Result<(), Error> {
        if self.heartbeat_interval.is_zero()
            || self.heartbeat_interval > Duration::from_secs(25)
            || [self.ack_timeout, self.connect_timeout, self.leave_timeout, self.backoff_min]
                .into_iter()
                .any(|v| v.is_zero() || v > Duration::from_secs(120))
            || self.backoff_max < self.backoff_min
            || self.backoff_max > Duration::from_secs(120)
            || self.max_roots == 0
            || self.max_roots > 4096
            || self.event_capacity == 0
            || self.event_capacity > 4096
        {
            return Err(Error::Options);
        }
        Ok(())
    }

    fn delay(&self, attempt: u32) -> Duration {
        self.backoff_min.saturating_mul(1u32 << attempt.min(16)).min(self.backoff_max)
    }
}

pub enum Command {
    /// Replace the explicit native-root selection; duplicates collapse. An
    /// unchanged root retains its subscription (and terminal failure, if any).
    SetRoots(Vec<Uuid>),
    Stop,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetRoots(roots) => f.debug_tuple("SetRoots").field(&roots.len()).finish(),
            Self::Stop => f.write_str("Stop"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Idle,
    Connecting,
    Live,
    Reconnecting,
    Degraded,
    Stopped,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Connection {
    Connecting,
    Connected,
    Reconnecting,
    Unavailable,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unavailable {
    Rejected,
    AuthOrConfig,
    Protocol,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionState {
    Joining,
    Joined,
    Reconnecting,
    Unavailable(Unavailable),
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceEvidence {
    Unknown,
    NoPeersObserved,
    ObservedPeers(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PresenceState {
    pub subscription: SubscriptionState,
    pub evidence: PresenceEvidence,
    /// Local evidence time, not a server clock or a peer's activity time.
    pub observed_at: Option<SystemTime>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RootState {
    pub subscription: SubscriptionState,
    /// Session-wide stamps strictly increasing for this root, including after
    /// deselection/reselection. They are not event counts or persistent cursors.
    pub observation_seq: u64,
    pub join_epoch: u64,
    pub observed_at: Option<SystemTime>,
    pub presence: Option<PresenceState>,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct State {
    pub health: Health,
    pub connection: Connection,
    pub roots: BTreeMap<Uuid, RootState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            health: Health::Idle,
            connection: Connection::Connecting,
            roots: BTreeMap::new(),
        }
    }
}

// Deliberate: serialization is the controller's explicit report boundary; Debug
// must not accidentally expose a selected private root in an error/tracing span.
impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("health", &self.health)
            .field("connection", &self.connection)
            .field("root_count", &self.roots.len())
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Durable,
    Presence,
}
type Key = (Uuid, Kind);

struct Model {
    state: State,
    sender: watch::Sender<State>,
    generations: BTreeMap<Uuid, u64>,
    sequence: u64,
}

impl Model {
    fn stamp(&mut self) -> Result<u64, Error> {
        self.sequence = self.sequence.checked_add(1).ok_or(Error::SequenceExhausted)?;
        Ok(self.sequence)
    }

    fn publish(&mut self) {
        self.state.health = if self.state.roots.is_empty() {
            Health::Idle
        } else if self.state.connection == Connection::Unavailable
            || self.state.roots.values().any(|r| matches!(r.subscription, SubscriptionState::Unavailable(_)))
        {
            Health::Degraded
        } else if self.state.connection == Connection::Reconnecting {
            Health::Reconnecting
        } else if self.state.connection == Connection::Connected && self.state.roots.values().all(|r| r.subscription == SubscriptionState::Joined) {
            Health::Live
        } else {
            Health::Connecting
        };
        self.sender.send_replace(self.state.clone());
    }

    fn update(&mut self, key: Key, generation: u64, update: Update) -> Result<(), Error> {
        if self.generations.get(&key.0) != Some(&generation) {
            return Ok(());
        }
        let update = if self.state.connection == Connection::Unavailable {
            Update::Gap(SubscriptionState::Unavailable(Unavailable::Closed))
        } else {
            update
        };
        let stamp = if key.1 == Kind::Durable && matches!(update, Update::Joined | Update::Gap(_) | Update::Poke) {
            Some(self.stamp()?)
        } else {
            None
        };
        let root = self.state.roots.get_mut(&key.0).expect("selected generation");
        if let Some(stamp) = stamp {
            root.observation_seq = stamp;
            root.observed_at = Some(SystemTime::now());
        }
        match (key.1, update) {
            (Kind::Durable, Update::Joined) => {
                root.subscription = SubscriptionState::Joined;
                root.join_epoch = stamp.expect("join stamp");
            }
            (Kind::Durable, Update::Gap(status)) => root.subscription = status,
            (Kind::Presence, Update::Joined) => {
                if let Some(presence) = &mut root.presence {
                    presence.subscription = SubscriptionState::Joined;
                    presence.evidence = PresenceEvidence::Unknown;
                    presence.observed_at = None;
                }
            }
            (Kind::Presence, Update::Gap(status)) => {
                if let Some(presence) = &mut root.presence {
                    presence.subscription = status;
                    presence.evidence = PresenceEvidence::Unknown;
                    presence.observed_at = None;
                }
            }
            (Kind::Presence, Update::Peers(count)) => {
                if let Some(presence) = &mut root.presence {
                    presence.evidence = if count == 0 {
                        PresenceEvidence::NoPeersObserved
                    } else {
                        PresenceEvidence::ObservedPeers(count)
                    };
                    presence.observed_at = Some(SystemTime::now());
                }
            }
            _ => (),
        }
        self.publish();
        Ok(())
    }
}

enum Update {
    Joined,
    Gap(SubscriptionState),
    Poke,
    Peers(usize),
}
type Shared = Rc<RefCell<Model>>;
struct Slot {
    generation: u64,
    cancel: Option<oneshot::Sender<()>>,
    attempt: u32,
    running: bool,
}
enum End {
    Retired,
    Retry,
    Terminal,
}
type Jobs = FuturesUnordered<LocalBoxFuture<'static, (Key, u64, Result<End, Error>)>>;

/// Run until Stop, command-channel closure, or a fatal local protocol/counter
/// failure. Leaves and shutdown are bounded. Drop is also safe (closes the local
/// driver) but cannot promise an acknowledged graceful leave.
pub async fn run(bootstrap: Bootstrap, options: Options, commands: mpsc::Receiver<Command>, state: watch::Sender<State>) -> Result<(), Error> {
    options.validate()?;
    let retry = options.clone();
    let rejoin = options.clone();
    let phoenix = phoenix_channel_client::Options::default()
        .heartbeat_interval(options.heartbeat_interval)
        .heartbeat_timeout(options.ack_timeout)
        .connect_timeout(options.connect_timeout)
        .join_timeout(options.ack_timeout)
        .leave_timeout(options.leave_timeout)
        .event_capacity(options.event_capacity)
        .command_capacity(options.max_roots * 4 + 64)
        .reconnect_policy(move |context| match context.reason {
            DisconnectReason::Requested | DisconnectReason::DriverStopped => ReconnectAction::Stop,
            DisconnectReason::Closed(close) if matches!(close.code, Some(1008 | 4001 | 4003)) => ReconnectAction::Stop,
            _ => ReconnectAction::RetryAfter(retry.delay(context.attempt)),
        })
        .rejoin_delay(move |attempt| rejoin.delay(attempt));
    let connector = NativeConnector::new(bootstrap.endpoint()).options(
        NativeTransportOptions::default()
            .tls_config(tls_config()?)
            .max_message_size(Some(2 * 1024 * 1024))
            .max_frame_size(Some(2 * 1024 * 1024)),
    );
    let (socket, driver) = Socket::new(connector, NativeTimer, phoenix);
    let shared = Rc::new(RefCell::new(Model {
        state: State::default(),
        sender: state,
        generations: BTreeMap::new(),
        sequence: 0,
    }));
    shared.borrow_mut().publish();
    tokio::pin!(driver);
    let shutdown_deadline = options.leave_timeout + Duration::from_millis(200);
    let session = session(&socket, bootstrap, options, commands, shared.clone());
    tokio::pin!(session);
    // The driver remains polled during every join, retry delay and graceful leave.
    let result = tokio::select! {
        result = &mut session => result,
        _ = &mut driver => {
            // shutdown() acknowledges just before the driver returns. Its ready
            // future can win this select during normal cancellation; let the
            // controller consume that acknowledgement (or its Closed event).
            tokio::time::timeout(shutdown_deadline, &mut session).await.unwrap_or(Err(Error::DriverStopped))
        }
    };
    let mut model = shared.borrow_mut();
    model.state.connection = Connection::Stopped;
    model.state.health = if result.is_ok() { Health::Stopped } else { Health::Failed };
    for root in model.state.roots.values_mut() {
        root.subscription = SubscriptionState::Stopped;
        if let Some(presence) = &mut root.presence {
            presence.subscription = SubscriptionState::Stopped;
            presence.evidence = PresenceEvidence::Unknown;
            presence.observed_at = None;
        }
    }
    model.sender.send_replace(model.state.clone());
    result
}

fn tls_config() -> Result<Arc<rustls::ClientConfig>, Error> {
    // reqwest and the native Phoenix adapter can enable different Rustls
    // providers in the same binary. Never rely on automatic feature inference
    // (which panics with both), or mutate the embedding process's global choice.
    // Match the HTTP client's platform trust verification, with a local provider.
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .map_err(|_| Error::Tls)?
        .with_platform_verifier()
        .map_err(|_| Error::Tls)?
        .with_no_client_auth();
    Ok(Arc::new(config))
}

async fn session(socket: &Socket, bootstrap: Bootstrap, options: Options, mut commands: mpsc::Receiver<Command>, model: Shared) -> Result<(), Error> {
    let mut events = socket.events().map_err(|_| Error::DriverStopped)?;
    let mut slots: BTreeMap<Key, Slot> = BTreeMap::new();
    let mut jobs = Jobs::new();
    let result = async {
        loop {
            tokio::select! {
                command = commands.recv() => match command {
                    Some(Command::SetRoots(roots)) => {
                        let roots: BTreeSet<_> = roots.into_iter().collect();
                        if roots.len() > options.max_roots || roots.iter().any(Uuid::is_nil) { return Err(Error::Roots); }
                        {
                            let mut model = model.borrow_mut();
                            model.state.roots.retain(|root, _| roots.contains(root));
                            model.generations.retain(|root, _| roots.contains(root));
                            for root in roots {
                                if model.state.roots.contains_key(&root) { continue; }
                                let generation = model.stamp()?;
                                model.generations.insert(root, generation);
                                model.state.roots.insert(root, RootState {
                                    subscription: SubscriptionState::Joining, observation_seq: generation, join_epoch: 0, observed_at: None,
                                    presence: options.presence.then_some(PresenceState { subscription: SubscriptionState::Joining, evidence: PresenceEvidence::Unknown, observed_at: None }),
                                });
                            }
                            model.publish();
                        }
                        // Do not register the same topic twice during leave/reselect.
                        for (key, slot) in &mut slots {
                            if model.borrow().generations.get(&key.0) != Some(&slot.generation)
                                && let Some(cancel) = slot.cancel.take() { let _ = cancel.send(()); }
                        }
                        slots.retain(|key, slot| slot.running || model.borrow().generations.get(&key.0) == Some(&slot.generation));
                        reconcile(socket, &bootstrap, &options, &model, &mut slots, &mut jobs)?;
                    }
                    Some(Command::Stop) | None => return Ok(()),
                },
                event = events.next() => {
                    let connection = match event {
                        Some(SocketEvent::Connected) => Connection::Connected,
                        Some(SocketEvent::Connecting { .. }) => if model.borrow().state.connection == Connection::Reconnecting { Connection::Reconnecting } else { Connection::Connecting },
                        Some(SocketEvent::Disconnected { .. } | SocketEvent::ReconnectScheduled { .. }) => Connection::Reconnecting,
                        Some(SocketEvent::ReconnectStopped { .. }) => {
                            for slot in slots.values_mut() {
                                if let Some(cancel) = slot.cancel.take() { let _ = cancel.send(()); }
                            }
                            Connection::Unavailable
                        },
                        Some(SocketEvent::Lagged { .. }) => {
                            // A lifecycle gap cannot establish readiness. Restart the
                            // socket; per-topic gaps/joins will dirty every selected root.
                            tokio::time::timeout(options.leave_timeout, socket.reconnect()).await.map_err(|_| Error::DriverStopped)?.map_err(|_| Error::DriverStopped)?;
                            Connection::Reconnecting
                        }
                        Some(SocketEvent::Closed) | None => return Err(Error::DriverStopped),
                    };
                    let mut model = model.borrow_mut();
                    model.state.connection = connection;
                    if connection == Connection::Unavailable {
                        let selected: Vec<_> = model.generations.iter().map(|(root, generation)| (*root, *generation)).collect();
                        for (root, generation) in selected {
                            model.update((root, Kind::Durable), generation, Update::Gap(SubscriptionState::Unavailable(Unavailable::Closed)))?;
                            if options.presence { model.update((root, Kind::Presence), generation, Update::Gap(SubscriptionState::Unavailable(Unavailable::Closed)))?; }
                        }
                    }
                    model.publish();
                },
                completed = jobs.next(), if !jobs.is_empty() => {
                    let (key, generation, end) = completed.expect("nonempty jobs");
                    let end = end?;
                    let Some(slot) = slots.get_mut(&key) else { continue };
                    if slot.generation != generation { continue; }
                    slot.cancel = None;
                    slot.running = false;
                    if (matches!(end, End::Terminal) || model.borrow().state.connection == Connection::Unavailable) && model.borrow().generations.get(&key.0) == Some(&generation) { continue; }
                    let attempt = slot.attempt.saturating_add(1);
                    slots.remove(&key);
                    start(socket, &bootstrap, &options, &model, &mut slots, &mut jobs, key, attempt)?;
                }
            }
        }
    }.await;
    for slot in slots.values_mut() {
        if let Some(cancel) = slot.cancel.take() {
            let _ = cancel.send(());
        }
    }
    // All jobs already in graceful leave run concurrently, not N * leave_timeout.
    let _ = tokio::time::timeout(options.leave_timeout + Duration::from_millis(100), async {
        while jobs.next().await.is_some() {}
    })
    .await;
    // Drop any timed-out jobs BEFORE shutdown so their handles cannot resurrect.
    drop(jobs);
    let _ = tokio::time::timeout(options.leave_timeout, socket.shutdown()).await;
    result
}

fn reconcile(socket: &Socket, bootstrap: &Bootstrap, options: &Options, model: &Shared, slots: &mut BTreeMap<Key, Slot>, jobs: &mut Jobs) -> Result<(), Error> {
    let roots: Vec<_> = model.borrow().state.roots.keys().copied().collect();
    for root in roots {
        start(socket, bootstrap, options, model, slots, jobs, (root, Kind::Durable), 0)?;
        if options.presence {
            start(socket, bootstrap, options, model, slots, jobs, (root, Kind::Presence), 0)?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn start(
    socket: &Socket,
    bootstrap: &Bootstrap,
    options: &Options,
    model: &Shared,
    slots: &mut BTreeMap<Key, Slot>,
    jobs: &mut Jobs,
    key: Key,
    attempt: u32,
) -> Result<(), Error> {
    if slots.contains_key(&key) {
        return Ok(());
    }
    let Some(generation) = model.borrow().generations.get(&key.0).copied() else {
        return Ok(());
    };
    if model.borrow().state.connection == Connection::Unavailable {
        model
            .borrow_mut()
            .update(key, generation, Update::Gap(SubscriptionState::Unavailable(Unavailable::Closed)))?;
        return Ok(());
    }
    let name = format!("realtime:{}:{}", if key.1 == Kind::Durable { "rootEntity" } else { "yjs" }, key.0);
    let payload = json!({"config":{"broadcast":{"ack":false,"self":false},"presence":{"key":"","enabled":key.1 == Kind::Presence},"postgres_changes":[],"private":false},"access_token":bootstrap.project.key});
    let channel = socket.channel(name, static_join_payload(payload)).map_err(|_| Error::Channel)?;
    let (cancel, cancellation) = oneshot::channel();
    slots.insert(
        key,
        Slot {
            generation,
            cancel: Some(cancel),
            attempt,
            running: true,
        },
    );
    let options = options.clone();
    let model = model.clone();
    jobs.push(Box::pin(async move {
        (key, generation, topic(channel, key, generation, options, model, cancellation, attempt).await)
    }));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn topic(
    channel: Channel,
    key: Key,
    generation: u64,
    options: Options,
    model: Shared,
    mut cancellation: oneshot::Receiver<()>,
    attempt: u32,
) -> Result<End, Error> {
    let mut channel = channel;
    let mut joined_ref = None;
    let mut presence = Presence::default();
    let mut terminal = None;
    // The handle's original bounded stream already captures Joined/pokes while
    // this first acknowledgement is pending. Use that stream directly instead
    // of creating a second subscription and retaining an unused raw-payload
    // queue for the whole connection. Other topics and the driver keep polling.
    let initial = tokio::select! {
        biased;
        _ = &mut cancellation => Some(End::Retired),
        result = channel.join() => match result {
            Ok(_) | Err(ClientError::AlreadyJoined(_)) => None,
            Err(ClientError::JoinRejected { response, .. }) => {
                terminal = join_rejection(&response);
                Some(if terminal.is_some() { End::Terminal } else { End::Retry })
            }
            Err(ClientError::DriverStopped) => return Err(Error::DriverStopped),
            Err(_) => Some(End::Retry),
        }
    };
    let end = if let Some(end) = initial {
        end
    } else {
        loop {
            tokio::select! {
                biased;
                _ = &mut cancellation => break End::Retired,
                event = channel.next_event() => {
                    match event {
                        Some(ChannelEvent::Protocol(ProtocolEvent::Joined { reference, .. })) => {
                            if joined_ref.as_ref() == Some(&reference) { continue; }
                            joined_ref = Some(reference);
                            presence.gap();
                            model.borrow_mut().update(key, generation, Update::Joined)?;
                        }
                        Some(ChannelEvent::Protocol(ProtocolEvent::Message(frame))) if joined_ref.is_some() => {
                            if frame.join_ref.is_some() && frame.join_ref != joined_ref { continue; }
                            if frame.event == "system" {
                                let payload: Value = frame.payload.deserialize().unwrap_or(Value::Null);
                                if payload["status"] == "error" {
                                    terminal = rejection(&frame.payload);
                                    break if terminal.is_some() { End::Terminal } else { End::Retry };
                                }
                            } else if key.1 == Kind::Durable && frame.event == "broadcast" {
                                let payload: Value = match frame.payload.deserialize() { Ok(value) => value, Err(_) => break End::Retry };
                                if payload["event"] == "poke" { model.borrow_mut().update(key, generation, Update::Poke)?; }
                            } else if key.1 == Kind::Presence && matches!(frame.event.as_str(), "presence_state" | "presence_diff") {
                                let count = frame.payload.deserialize::<Value>().map_err(|_| ()).and_then(|value| {
                                    if frame.event == "presence_state" { presence.snapshot(&value).map(Some) } else { presence.diff(&value) }
                                });
                                match count {
                                    Ok(Some(count)) => model.borrow_mut().update(key, generation, Update::Peers(count))?,
                                    Ok(None) => (),
                                    Err(_) => break End::Retry, // reset + fresh snapshot, no partially applied count
                                }
                            }
                        }
                        Some(ChannelEvent::Protocol(ProtocolEvent::ChannelClosed { payload, .. })) => { terminal = rejection(&payload); break if terminal.is_some() { End::Terminal } else { End::Retry }; }
                        Some(ChannelEvent::Protocol(ProtocolEvent::JoinError { response, .. })) => {
                            if let Some(reason) = join_rejection(&response) { terminal = Some(reason); break End::Terminal; }
                            presence.gap(); joined_ref = None;
                            model.borrow_mut().update(key, generation, Update::Gap(SubscriptionState::Reconnecting))?;
                        }
                        Some(ChannelEvent::Protocol(ProtocolEvent::ChannelError { payload, .. })) => {
                            if let Some(reason) = rejection(&payload) { terminal = Some(reason); break End::Terminal; }
                            presence.gap(); joined_ref = None;
                            model.borrow_mut().update(key, generation, Update::Gap(SubscriptionState::Reconnecting))?;
                        }
                        Some(ChannelEvent::Disconnected) => {
                            presence.gap(); joined_ref = None;
                            model.borrow_mut().update(key, generation, Update::Gap(SubscriptionState::Reconnecting))?;
                        }
                        Some(ChannelEvent::Lagged { .. }) => break End::Retry,
                        Some(ChannelEvent::JoinPayloadError(_)) => { terminal = Some(Unavailable::AuthOrConfig); break End::Terminal; }
                        None => { terminal = Some(Unavailable::Closed); break End::Terminal; }
                        // StaleMessage and UnmatchedReply never change retained state.
                        _ => (),
                    }
                }
            }
        }
    };
    model.borrow_mut().update(
        key,
        generation,
        Update::Gap(if let Some(reason) = terminal {
            SubscriptionState::Unavailable(reason)
        } else {
            SubscriptionState::Reconnecting
        }),
    )?;
    let _ = tokio::time::timeout(options.leave_timeout, channel.leave()).await;
    // Dispose the registered topic before the central loop is allowed to recreate.
    drop(channel);
    if matches!(end, End::Retry) {
        tokio::select! { _ = &mut cancellation => return Ok(End::Retired), _ = tokio::time::sleep(options.delay(attempt)) => () }
    }
    Ok(end)
}

fn rejection(payload: &Payload) -> Option<Unavailable> {
    let value: Value = payload.deserialize().unwrap_or(Value::Null);
    if value == json!({}) || value.is_null() {
        return None;
    } // normal close / unspecified transient error
    let text = value.to_string().to_ascii_lowercase();
    if [
        "unauthorized",
        "not authorized",
        "forbidden",
        "invalid jwt",
        "jwt expired",
        "invalid token",
        "invalid api",
        "invalid signature",
        "invalid config",
        "tenant not found",
        "permission denied",
    ]
    .iter()
    .any(|part| text.contains(part))
    {
        Some(Unavailable::AuthOrConfig)
    } else if [
        "timeout",
        "rate limit",
        "too many",
        "database",
        "restart",
        "shutdown",
        "unavailable",
        "overload",
    ]
    .iter()
    .any(|part| text.contains(part))
    {
        None
    } else {
        Some(Unavailable::Rejected)
    }
}

fn join_rejection(payload: &Payload) -> Option<Unavailable> {
    let value: Value = payload.deserialize().unwrap_or(Value::Null);
    if value.is_null() || value == json!({}) {
        Some(Unavailable::Rejected)
    } else {
        rejection(payload)
    }
}

#[cfg(test)]
mod tests;
