use leafbud_live_probe::{Dirty, Presence, control, join};
use serde_json::{Value, json};

#[test]
fn passive_join_has_no_application_broadcast_or_presence_tracking() {
    let durable = join("realtime:rootEntity:synthetic-root", "1", "synthetic-key", false);
    let yjs = join("realtime:yjs:synthetic-root", "2", "synthetic-key", true);
    assert_eq!(durable["event"], "phx_join");
    assert_eq!(durable["payload"]["config"]["presence"]["enabled"], false);
    assert_eq!(yjs["payload"]["config"]["presence"]["enabled"], true);
    assert_eq!(yjs["payload"]["config"]["private"], false);
    assert_eq!(control("realtime:yjs:synthetic-root", "phx_leave", "3", Some("2"))["event"], "phx_leave");
}

#[test]
fn passive_presence_waits_for_state_counts_refs_and_resets_on_gap() {
    let fixtures: Value = serde_json::from_str(include_str!("../fixtures/events.json")).unwrap();
    let mut presence = Presence::default();
    assert!(!presence.ready);
    assert_eq!(presence.diff(&fixtures["diff"][4]).unwrap(), None);
    assert_eq!(presence.snapshot(&fixtures["state"][4]).unwrap(), 2);
    assert_eq!(presence.diff(&fixtures["diff"][4]).unwrap(), Some(1));
    // Repeated diff is idempotent and does not delete unrelated metas.
    assert_eq!(presence.diff(&fixtures["diff"][4]).unwrap(), Some(1));
    presence.gap();
    assert!(!presence.ready);
    assert_eq!(presence.diff(&fixtures["diff"][4]).unwrap(), None);
    assert_eq!(presence.snapshot(&json!({})).unwrap(), 0);
    assert!(presence.ready); // No peers observed, NOT proof nobody is editing.
}

#[test]
fn event_during_read_survives_finish_and_reconnect_forces_catchup() {
    let mut dirty = Dirty::default();
    dirty.wake(); // confirmed initial subscription
    assert!(dirty.start());
    dirty.wake(); // poke racing the HTTP snapshot
    dirty.wake(); // repeated pokes coalesce
    assert!(!dirty.start());
    dirty.finish();
    assert!(dirty.start()); // follow-up was not lost
    dirty.finish();
    assert!(!dirty.start());
    dirty.wake(); // every confirmed rejoin, even with no observed poke
    assert!(dirty.start());
}
