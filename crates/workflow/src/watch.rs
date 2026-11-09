//! Connected inbound follow. The local realtime future is polled concurrently
//! with one cancellable document transaction; no socket handles enter spawn.
use crate::{RemoteReader, config, refresh};
use anyhow::{Context, Result, ensure};
use leafbud_web::live;
use serde::Serialize;
use std::{
    collections::{BTreeMap, VecDeque},
    future::Future,
    path::PathBuf,
    pin::Pin,
    time::{Duration, SystemTime},
};
use tokio::{sync::watch, time::Instant};
use uuid::Uuid;

mod selection;
mod transport;
pub use leafbud_web::live::{Options as TransportOptions, PublicProject};

#[derive(Default)]
pub struct Request {
    pub all: bool,
    pub files: Vec<PathBuf>,
}

#[derive(Clone)]
pub struct Options {
    pub backstop: Duration,
    pub rescan: Duration,
    pub retry_min: Duration,
    pub retry_max: Duration,
    pub refresh_timeout: Duration,
    pub bootstrap_retry: Duration,
    pub bootstrap_timeout: Duration,
    pub project: Option<PublicProject>,
    pub transport: TransportOptions,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            backstop: Duration::from_secs(300),
            rescan: Duration::from_secs(5),
            retry_min: Duration::from_secs(1),
            retry_max: Duration::from_secs(60),
            refresh_timeout: Duration::from_secs(90),
            bootstrap_retry: Duration::from_secs(60),
            bootstrap_timeout: Duration::from_secs(90),
            project: None,
            transport: TransportOptions::default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Event {
    pub version: u8,
    pub observed_at: SystemTime,
    #[serde(flatten)]
    pub detail: Detail,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Detail {
    Connection {
        health: live::Health,
        message: Option<String>,
    },
    Inventory {
        selected: usize,
        subscribed_roots: usize,
        message: Option<String>,
    },
    Document {
        #[serde(flatten)]
        report: refresh::Report,
        acquisition: &'static str,
    },
    Presence {
        path: PathBuf,
        document_id: Option<Uuid>,
        #[serde(flatten)]
        state: live::PresenceState,
    },
    Subscription {
        path: PathBuf,
        document_id: Option<Uuid>,
        state: live::SubscriptionState,
    },
    Summary {
        #[serde(flatten)]
        summary: Summary,
    },
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Summary {
    pub selected: usize,
    pub refreshes: usize,
    pub writes: usize,
    pub cancelled_refreshes: usize,
    /// Latest per-file outcomes, not cumulative failures or content history.
    pub outcomes: BTreeMap<String, usize>,
    pub stopped: bool,
}

#[derive(Clone, Default)]
struct Transport {
    generation: u64,
    state: live::State,
    message: Option<String>,
}

struct Entry {
    selected: selection::Selected,
    serviced_seq: u64,
    local: u64,
    serviced_local: u64,
    sweep: u64,
    serviced_sweep: u64,
    retry: Instant,
    failures: u32,
    last: Option<(refresh::Outcome, Option<String>)>,
    presence: Option<live::PresenceState>,
    subscription: Option<live::SubscriptionState>,
}
impl Entry {
    fn new(selected: selection::Selected) -> Self {
        Self {
            selected,
            serviced_seq: 0,
            local: 1,
            serviced_local: 0,
            sweep: 0,
            serviced_sweep: 0,
            retry: Instant::now(),
            failures: 0,
            last: None,
            presence: None,
            subscription: None,
        }
    }
}

struct Job {
    path: PathBuf,
    identity: Option<refresh::Identity>,
    seq: u64,
    local: u64,
    sweep: u64,
    acquisition: &'static str,
    generation: u64,
}
type Work<'a> = Pin<Box<dyn Future<Output = refresh::Report> + 'a>>;

impl RemoteReader {
    /// `cancel` becoming true (or its sender closing) abandons HTTP work before
    /// any subsequent write, releases the operation lock and bounds socket leave.
    /// Emit must be quick: the CLI writes one transition line, never document bodies.
    pub async fn watch(
        &self,
        context: &config::Context,
        request: Request,
        options: Options,
        cancel: watch::Receiver<bool>,
        mut emit: impl FnMut(&Event) -> Result<()>,
    ) -> Result<Summary> {
        ensure!(request.all ^ !request.files.is_empty(), "select file operands or --all");
        ensure!(
            [
                options.backstop,
                options.rescan,
                options.retry_min,
                options.refresh_timeout,
                options.bootstrap_retry,
                options.bootstrap_timeout
            ]
            .iter()
            .all(|d| !d.is_zero() && *d <= Duration::from_secs(30 * 86400))
                && options.retry_max >= options.retry_min
                && options.retry_max <= Duration::from_secs(30 * 86400),
            "watch intervals must be positive, at most 30 days, and retry bounds ordered"
        );
        let root = std::fs::canonicalize(&context.workspace_root).context("resolving watch workspace")?;
        let selection = selection::Selection::new(root, self.client.base_url(), request)?;
        let (selected, roots) = selection.scan()?;
        ensure!(roots.len() <= options.transport.max_roots, "watch selection exceeds realtime root limit");
        let (roots_tx, roots_rx) = watch::channel(roots);
        let (state_tx, mut state_rx) = watch::channel(Transport::default());
        let (stop_tx, stop_rx) = watch::channel(false);
        let driver = transport::drive(&self.client, &options, roots_rx, state_tx, stop_rx);
        let controller = async {
            let result = self
                .watch_controller(context, &selection, selected, &options, cancel, &roots_tx, &mut state_rx, &mut emit)
                .await;
            stop_tx.send_replace(true);
            result
        };
        let (_, result) = tokio::join!(driver, controller);
        result
    }

    #[allow(clippy::too_many_arguments)]
    async fn watch_controller(
        &self,
        context: &config::Context,
        selection: &selection::Selection,
        selected: Vec<selection::Selected>,
        options: &Options,
        mut cancel: watch::Receiver<bool>,
        roots: &watch::Sender<Vec<Uuid>>,
        transport: &mut watch::Receiver<Transport>,
        emit: &mut impl FnMut(&Event) -> Result<()>,
    ) -> Result<Summary> {
        let mut entries: BTreeMap<_, _> = selected.into_iter().map(|s| (s.path.clone(), Entry::new(s))).collect();
        let mut queue: VecDeque<_> = entries.keys().cloned().collect();
        let mut summary = Summary::default();
        let mut work: Option<Work<'_>> = None;
        let mut job: Option<Job> = None;
        let mut next_scan = Instant::now() + options.rescan;
        let mut next_sweep = Instant::now() + options.backstop;
        let mut health = None;
        let mut generation = 0;
        let mut inventory_error = None;
        event(
            emit,
            Detail::Inventory {
                selected: entries.len(),
                subscribed_roots: roots.borrow().len(),
                message: entries.is_empty().then(|| "no managed files selected; waiting for inventory rescan".into()),
            },
        )?;
        loop {
            if *cancel.borrow() || cancel.has_changed().is_err() {
                break;
            }
            ensure!(transport.has_changed().is_ok(), "live transport stopped unexpectedly");
            let latest = transport.borrow_and_update().clone();
            if latest.generation != generation {
                generation = latest.generation;
                for entry in entries.values_mut() {
                    entry.serviced_seq = 0;
                }
            }
            let current_health = (latest.state.health, latest.message.clone());
            if health.as_ref() != Some(&current_health) {
                health = Some(current_health.clone());
                event(
                    emit,
                    Detail::Connection {
                        health: current_health.0,
                        message: current_health.1,
                    },
                )?;
            }
            for entry in entries.values_mut() {
                if entry.selected.root.is_some() {
                    let subscription = entry.selected.root.and_then(|r| latest.state.roots.get(&r)).map(|r| r.subscription).unwrap_or(
                        if latest.state.connection == live::Connection::Unavailable {
                            live::SubscriptionState::Unavailable(live::Unavailable::Closed)
                        } else {
                            live::SubscriptionState::Joining
                        },
                    );
                    if entry.subscription != Some(subscription) {
                        entry.subscription = Some(subscription);
                        event(
                            emit,
                            Detail::Subscription {
                                path: entry.selected.path.clone(),
                                document_id: entry.selected.id,
                                state: subscription,
                            },
                        )?;
                    }
                }
                if let Some(presence) = entry
                    .selected
                    .root
                    .and_then(|r| latest.state.roots.get(&r))
                    .and_then(|r| r.presence.clone())
                    .or_else(|| {
                        (options.transport.presence && entry.selected.root.is_some()).then_some(live::PresenceState {
                            subscription: live::SubscriptionState::Reconnecting,
                            evidence: live::PresenceEvidence::Unknown,
                            observed_at: None,
                        })
                    })
                    && entry.presence.as_ref() != Some(&presence)
                {
                    entry.presence = Some(presence.clone());
                    event(
                        emit,
                        Detail::Presence {
                            path: entry.selected.path.clone(),
                            document_id: entry.selected.id,
                            state: presence,
                        },
                    )?;
                }
            }
            let now = Instant::now();
            if now >= next_scan {
                next_scan = now + options.rescan;
                match selection.scan() {
                    Ok((selected, selected_roots)) if selected_roots.len() <= options.transport.max_roots => {
                        inventory_error = None;
                        let mut old = std::mem::take(&mut entries);
                        for selected in selected {
                            let path = selected.path.clone();
                            let mut entry = match old.remove(&path) {
                                Some(mut old) if old.selected.identity == selected.identity && old.selected.id == selected.id => {
                                    if old.selected.fingerprint != selected.fingerprint || old.selected.problem != selected.problem {
                                        old.local += 1;
                                        old.retry = now;
                                        old.failures = 0;
                                    }
                                    old.selected = selected;
                                    old
                                }
                                _ => Entry::new(selected),
                            };
                            // A new root identity starts a new selection even if
                            // it happens to reuse a pathname.
                            if entry.selected.problem.is_some() {
                                entry.serviced_seq = 0;
                            }
                            entries.insert(path.clone(), entry);
                            if !queue.contains(&path) {
                                queue.push_back(path);
                            }
                        }
                        for (path, entry) in old {
                            event(
                                emit,
                                Detail::Document {
                                    report: refresh::Report {
                                        path,
                                        document_id: entry.selected.id,
                                        outcome: refresh::Outcome::SourceLost,
                                        message: Some("source removed or no longer selected; not recreated".into()),
                                    },
                                    acquisition: "inventory",
                                },
                            )?;
                        }
                        queue.retain(|p| entries.contains_key(p));
                        if let Some(active) = &job
                            && entries
                                .get(&active.path)
                                .is_none_or(|e| e.selected.identity != active.identity || e.selected.problem.is_some())
                        {
                            work = None;
                            job = None;
                            summary.cancelled_refreshes += 1;
                        }
                        if *roots.borrow() != selected_roots {
                            roots.send_replace(selected_roots);
                            event(
                                emit,
                                Detail::Inventory {
                                    selected: entries.len(),
                                    subscribed_roots: roots.borrow().len(),
                                    message: None,
                                },
                            )?;
                        }
                    }
                    result => {
                        let message = match result {
                            Err(e) => format!("inventory rescan failed; adoption paused: {e:#}"),
                            _ => "inventory exceeds realtime root limit; adoption paused".into(),
                        };
                        if inventory_error.as_ref() != Some(&message) {
                            event(
                                emit,
                                Detail::Inventory {
                                    selected: entries.len(),
                                    subscribed_roots: roots.borrow().len(),
                                    message: Some(message.clone()),
                                },
                            )?;
                        }
                        inventory_error = Some(message);
                        if work.take().is_some() {
                            job = None;
                            summary.cancelled_refreshes += 1;
                        }
                    }
                }
            }
            if now >= next_sweep {
                next_sweep = now + options.backstop;
                for entry in entries.values_mut() {
                    entry.sweep += 1;
                }
            }
            if work.is_none() && inventory_error.is_none() {
                // Preserve the whole current selection, including external
                // operands and resolved rename targets, for the transaction's
                // post-await duplicate check (not just periodic inventory).
                let operands = entries.keys().cloned().collect::<Vec<_>>();
                for _ in 0..queue.len() {
                    let path = queue.pop_front().expect("queue length");
                    queue.push_back(path.clone());
                    let entry = entries.get_mut(&path).expect("selected queue");
                    if entry.retry > now {
                        continue;
                    }
                    let root_state = entry.selected.root.and_then(|r| latest.state.roots.get(&r));
                    let joined = root_state.is_some_and(|r| r.subscription == live::SubscriptionState::Joined && r.join_epoch != 0);
                    let seq = if joined { root_state.expect("joined").observation_seq } else { 0 };
                    let sweep = entry.sweep != entry.serviced_sweep;
                    let local = entry.local != entry.serviced_local;
                    if let Some((outcome, message)) = &entry.selected.problem {
                        if local {
                            emit_document(
                                emit,
                                entry,
                                refresh::Report {
                                    path,
                                    document_id: entry.selected.id,
                                    outcome: *outcome,
                                    message: Some(message.clone()),
                                },
                                "inventory",
                            )?;
                            entry.serviced_local = entry.local;
                        }
                        continue;
                    }
                    // Initial catch-up requires an acknowledged join. If no join
                    // is available, only an explicitly labeled slow polling
                    // backstop may acquire HTTP content, never a "live" claim.
                    if !(joined && (local || seq != entry.serviced_seq) || sweep) {
                        continue;
                    }
                    let active = Job {
                        path: path.clone(),
                        identity: entry.selected.identity.clone(),
                        seq,
                        local: entry.local,
                        sweep: entry.sweep,
                        acquisition: if joined { "live" } else { "degraded-poll" },
                        generation,
                    };
                    let policy = refresh::Policy {
                        follow: true,
                        expected: active.identity.clone(),
                        operands,
                        ..Default::default()
                    };
                    let timeout = options.refresh_timeout;
                    work = Some(Box::pin(async move {
                        match tokio::time::timeout(timeout, self.refresh_document(context, &path, policy)).await {
                            Ok(report) => report,
                            Err(_) => refresh::Report {
                                path,
                                document_id: None,
                                outcome: refresh::Outcome::Unavailable,
                                message: Some("refresh deadline exceeded; source and receipts preserved".into()),
                            },
                        }
                    }));
                    job = Some(active);
                    break;
                }
            }
            // Retry timer only for dirty entries, otherwise an old Instant would
            // create a busy loop. Rescan/backstop always supply a finite wakeup.
            let retry = entries.values().filter(|e| e.retry > now).map(|e| e.retry).min().unwrap_or(next_scan);
            let wake = next_scan.min(next_sweep).min(retry);
            tokio::select! {
                biased;
                _ = cancel.changed() => {},
                _ = transport.changed() => {},
                report = async { work.as_mut().expect("guarded work").await }, if work.is_some() => {
                    work = None;
                    let active = job.take().expect("active refresh");
                    summary.refreshes += 1;
                    if matches!(report.outcome, refresh::Outcome::Updated | refresh::Outcome::Converged | refresh::Outcome::ReceiptsOnly) { summary.writes += 1; }
                    if let Some(entry) = entries.get_mut(&active.path) {
                        // Only this start stamp is serviced. An event during
                        // the read remains dirty even if state is coalesced.
                        entry.serviced_seq = if active.generation == generation { active.seq } else { 0 };
                        entry.serviced_local = active.local;
                        entry.serviced_sweep = active.sweep;
                        if report.outcome.retry() {
                            let delay = options.retry_min.saturating_mul(1 << entry.failures.min(16)).min(options.retry_max);
                            entry.failures = entry.failures.saturating_add(1);
                            entry.retry = Instant::now() + delay;
                            // Retry independently of a further socket event.
                            entry.serviced_local = 0;
                        } else { entry.failures = 0; entry.retry = Instant::now(); }
                        emit_document(emit, entry, report, active.acquisition)?;
                    }
                },
                _ = tokio::time::sleep_until(wake) => {},
            }
        }
        if work.take().is_some() {
            summary.cancelled_refreshes += 1;
        }
        summary.selected = entries.len();
        for entry in entries.values() {
            *summary
                .outcomes
                .entry(entry.last.as_ref().map(|r| r.0.as_str()).unwrap_or("not-observed").into())
                .or_default() += 1;
        }
        summary.stopped = true;
        event(emit, Detail::Summary { summary: summary.clone() })?;
        Ok(summary)
    }
}

fn event(emit: &mut impl FnMut(&Event) -> Result<()>, detail: Detail) -> Result<()> {
    emit(&Event {
        version: 1,
        observed_at: SystemTime::now(),
        detail,
    })
}
fn emit_document(emit: &mut impl FnMut(&Event) -> Result<()>, entry: &mut Entry, report: refresh::Report, acquisition: &'static str) -> Result<()> {
    let state = (report.outcome, report.message.clone());
    if entry.last.as_ref() != Some(&state)
        || matches!(
            report.outcome,
            refresh::Outcome::Updated | refresh::Outcome::Converged | refresh::Outcome::ReceiptsOnly
        )
    {
        event(emit, Detail::Document { report, acquisition })?;
    }
    entry.last = Some(state);
    Ok(())
}

#[cfg(test)]
mod tests;
