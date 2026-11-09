use crate::remote::text;
use leafbud_workflow::{
    RemoteOptions,
    config::{self, Config},
    watch::{self, Detail},
};
use std::{
    io::{self, Write},
    path::Path,
    time::Duration,
};

pub struct Settings {
    pub json: bool,
    pub backstop: u64,
    pub rescan: u64,
    pub presence: bool,
    pub project_url: Option<String>,
    pub anon_key: Option<String>,
}

pub async fn run(
    remote: RemoteOptions,
    root: Option<&Path>,
    flags: &Config,
    request: watch::Request,
    settings: Settings,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = config::load(root, flags, None)?;
    let project_url = settings.project_url.or(std::env::var("LEAFBUD_REALTIME_PROJECT_URL").ok());
    let anon_key = settings.anon_key.or(std::env::var("LEAFBUD_REALTIME_ANON_KEY").ok());
    let project = match (project_url, anon_key) {
        (None, None) => None,
        (Some(url), Some(key)) => Some(watch::PublicProject::new(&url, &key)?),
        _ => {
            return Err(
                "realtime override requires both project URL and public anon JWT (flags or LEAFBUD_REALTIME_PROJECT_URL/LEAFBUD_REALTIME_ANON_KEY)".into(),
            );
        }
    };
    let options = watch::Options {
        project,
        backstop: Duration::from_secs(settings.backstop),
        rescan: Duration::from_secs(settings.rescan),
        transport: watch::TransportOptions {
            presence: settings.presence,
            ..Default::default()
        },
        ..Default::default()
    };
    let reader = remote.connect().await?;
    let (cancel, cancelled) = tokio::sync::watch::channel(false);
    let stopped = cancel.subscribe();
    let signal = async {
        let mut stopped = stopped;
        tokio::select! {
            _ = interrupt() => { cancel.send_replace(true); },
            _ = stopped.changed() => {},
        }
    };
    let following = async {
        let stdout = io::stdout();
        let mut output = stdout.lock();
        let result = reader
            .watch(&context, request, options, cancelled, |event| {
                if settings.json {
                    serde_json::to_writer(&mut output, event)?;
                    writeln!(output)?;
                } else {
                    match &event.detail {
                        Detail::Connection { health, message } => writeln!(
                            output,
                            "connection {health:?}{}",
                            message.as_ref().map(|s| format!(": {}", text(s))).unwrap_or_default()
                        )?,
                        Detail::Inventory {
                            selected,
                            subscribed_roots,
                            message,
                        } => writeln!(
                            output,
                            "inventory {selected} files; {subscribed_roots} roots{}",
                            message.as_ref().map(|s| format!(": {}", text(s))).unwrap_or_default()
                        )?,
                        Detail::Document { report, acquisition } => writeln!(
                            output,
                            "{} {} [{acquisition}]{}",
                            report.outcome.as_str(),
                            text(&report.path.display().to_string()),
                            report.message.as_ref().map(|s| format!(": {}", text(s))).unwrap_or_default()
                        )?,
                        Detail::Presence { path, state, .. } => writeln!(
                            output,
                            "presence {} {:?}; observed {:?} (advisory only)",
                            text(&path.display().to_string()),
                            state.evidence,
                            state.observed_at
                        )?,
                        Detail::Subscription { path, state, .. } => writeln!(output, "subscription {} {state:?}", text(&path.display().to_string()))?,
                        Detail::Summary { summary } => writeln!(
                            output,
                            "stopped: {} selected; {} refreshes; {} writes; {} cancelled; {:?}",
                            summary.selected, summary.refreshes, summary.writes, summary.cancelled_refreshes, summary.outcomes
                        )?,
                    }
                }
                output.flush()?;
                Ok(())
            })
            .await;
        cancel.send_replace(true);
        result
    };
    let (_, result) = tokio::join!(signal, following);
    result?;
    Ok(())
}

async fn interrupt() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match (signal(SignalKind::interrupt()), signal(SignalKind::terminate())) {
            (Ok(mut int), Ok(mut term)) => {
                tokio::select! { _ = int.recv() => {}, _ = term.recv() => {} }
            }
            _ => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
