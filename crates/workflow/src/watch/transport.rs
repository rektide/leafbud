//! Bounded bootstrap retry and concurrent polling of the accepted live adapter.
use super::*;
use tokio::sync::mpsc;

pub(super) async fn drive(
    client: &leafbud_web::LeafletClient,
    options: &Options,
    mut roots: watch::Receiver<Vec<Uuid>>,
    output: watch::Sender<Transport>,
    mut stop: watch::Receiver<bool>,
) {
    let mut generation = 0u64;
    loop {
        if *stop.borrow() || stop.has_changed().is_err() {
            return;
        }
        let bootstrap = tokio::select! {
            biased;
            _ = stop.changed() => return,
            result = tokio::time::timeout(options.bootstrap_timeout, client.live_bootstrap(options.project.clone())) => result,
        };
        let bootstrap = match bootstrap {
            Ok(Ok(bootstrap)) => bootstrap,
            error => {
                let message = match error {
                    Ok(Err(e)) => e.to_string(),
                    _ => "public realtime bootstrap exceeded watch deadline".into(),
                };
                output.send_replace(Transport {
                    generation,
                    state: live::State {
                        health: live::Health::Degraded,
                        connection: live::Connection::Unavailable,
                        roots: BTreeMap::new(),
                    },
                    message: Some(format!("{message}; bounded retry scheduled; slow polling is not a live connection")),
                });
                tokio::select! { _ = stop.changed() => return, _ = tokio::time::sleep(options.bootstrap_retry) => continue }
            }
        };
        if bootstrap.require_origin(client.base_url()).is_err() {
            return;
        }
        let Some(next) = generation.checked_add(1) else {
            return;
        };
        generation = next;
        let (sender, mut state) = watch::channel(live::State::default());
        let (commands, receiver) = mpsc::channel(1);
        let run = live::run(bootstrap, options.transport.clone(), receiver, sender);
        tokio::pin!(run);
        let mut pending = Some(roots.borrow_and_update().clone());
        output.send_replace(Transport {
            generation,
            state: state.borrow().clone(),
            message: None,
        });
        let result = loop {
            tokio::select! {
                biased;
                _ = stop.changed() => {
                    // Sender closure requests bounded leaves. Keep polling the
                    // local driver throughout shutdown; timeout bounds bugs too.
                    break None;
                },
                result = &mut run => break Some(result),
                _ = state.changed() => { output.send_replace(Transport { generation, state: state.borrow_and_update().clone(), message: None }); },
                result = roots.changed() => {
                    if result.is_err() { return; }
                    pending = Some(roots.borrow_and_update().clone());
                },
                permit = commands.reserve(), if pending.is_some() => {
                    match permit { Ok(permit) => permit.send(live::Command::SetRoots(pending.take().expect("pending selection"))), Err(_) => break Some(Err(live::Error::DriverStopped)) }
                }
            }
        };
        let Some(result) = result else {
            drop(commands);
            let _ = tokio::time::timeout(options.transport.leave_timeout + Duration::from_secs(1), &mut run).await;
            return;
        };
        output.send_replace(Transport {
            generation,
            state: live::State {
                health: live::Health::Degraded,
                connection: live::Connection::Unavailable,
                roots: BTreeMap::new(),
            },
            message: Some(format!("{}; retry scheduled", result.err().unwrap_or(live::Error::DriverStopped))),
        });
        tokio::select! { _ = stop.changed() => return, _ = tokio::time::sleep(options.bootstrap_retry) => {} }
    }
}
