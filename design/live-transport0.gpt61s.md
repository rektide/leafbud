---
type: ImplementationReport
title: "Production live transport checkpoint"
description: "Implemented cookie-free public bootstrap and local managed Phoenix transport, retained root invalidation, passive presence, lifecycle fixtures and a bounded deployed adapter check; watch integration remains separate."
resource: /design/live-transport0.gpt61s.md
status: implemented-transport-only
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e41995c2fffesHpONkYRgWfiZq"
    title: "Commissioned production web transport implementation, socket/bootstrap verification and receiving watch contract"
  - resource: /experiments/15-live-connection/live-connection0.gpt61s.md
    title: "Reviewed maintained-client choice, deployment bootstrap proof and latest-state transport handoff"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Accepted section 5 persistent connection, guarded catch-up and presence boundaries"
  - resource: /crates/leaflet/web/src/lib.rs
    title: "HTTP client conventions and new cookie-free live_bootstrap entry point"
  - resource: /crates/leaflet/web/src/live.rs
    title: "Implemented public state/control API and local socket/topic lifecycle"
  - resource: /crates/leaflet/web/src/live/bootstrap.rs
    title: "Validated explicit override and bounded same-origin asset discovery"
  - resource: /crates/leaflet/web/src/live/presence.rs
    title: "Transactional reference-aware presence reduction"
  - resource: /crates/leaflet/web/src/live/tests.rs
    title: "Production-adapter loopback, freshness, invalidation, redaction and bootstrap tests"
---

# Production live transport checkpoint

## Outcome and scope

**Implemented `leafbud_web::live`, not merely another probe.** One managed Phoenix
socket multiplexes explicitly selected native roots, preserves invalidation while
the controller is busy, maintains acknowledged subscription readiness and optional
passive presence, and leaves/recovers with bounded waits. Coordinated workspace
version and exact internal pins are **0.12.0**.

This is the independently commissioned transport checkpoint of
`leafbud-editor-follow`. It does **not** implement watch's filesystem/controller,
locks, source selection, HTTP refresh/adoption or CLI. No model, MDX, workflow,
CLI, README or accepted shared-design changes belong to this slice. The concurrent
foundation correction remains independently owned. The follow ticket stays open.

Checkpoints:

- `9f990516`: public bootstrap, managed transport/state, loopback tests and 0.12.0.
- `e2fc40e7`: explicit local Rustls provider with platform certificate verification,
  after the first production-adapter attempt exposed a mixed-provider panic.
- `bfac6d65`: consume the original channel stream instead of retaining an unused
  duplicate raw-payload queue; reject unverified opaque-key join configuration.
- `bd10ad8e`: uncertain-leave socket recovery, generic-close versus join-rejection
  classification, and additional gap/shutdown fixtures.

These are preservation/review checkpoints, not completion of the watch feature.

## Receiving API

| Surface | Location and contract |
| --- | --- |
| `LeafletClient::live_bootstrap(project)` | [HTTP facade](/crates/leaflet/web/src/lib.rs): independently resolves public config, never clones the authenticated HTTP transport |
| `PublicProject::new(project_origin, anon_key)` | [Bootstrap](/crates/leaflet/web/src/live/bootstrap.rs): validated explicit public anon-JWT override; hosted project/key ref mismatch is an error |
| `Bootstrap::{resolve, explicit, require_origin}` | Origin-bound configuration, no global cache or silent project fallback; call `require_origin` when reusing with a configured HTTP reader |
| `live::run(bootstrap, options, commands, state)` | [Transport](/crates/leaflet/web/src/live.rs): local future containing the continuously polled Phoenix driver; no dedicated thread or service |
| `Command::SetRoots(Vec<Uuid>)` | Replaces selection, deduplicates roots, retains unchanged channels and terminal failures; nil roots/over-limit selection fail explicitly |
| `Command::Stop` / sender closure | Bounded best-effort acknowledged leaves followed by socket shutdown; dropping the future closes locally but cannot promise acknowledged leaves |
| `State` | Latest socket `connection`, aggregate `health`, and selected `roots` only; `watch::Sender` retains the latest value even without a receiver |
| `RootState` | `subscription`, `observation_seq`, `join_epoch`, local `observed_at`, optional `PresenceState` |

The public bootstrap, command and state values/channels are `Send + Sync`; the
socket and driver deliberately remain local/`!Send`. Poll `run` alongside the
controller with `tokio::join!`/`select!`. Do **not** put the raw future into
`tokio::spawn`, await it before starting the controller, or stop polling it during
HTTP/file work. `LocalSet` is only needed if the caller chooses `spawn_local`;
the production check exercised direct concurrent polling without one.

Readiness is `root.subscription == Joined`, with a nonzero `join_epoch`, not
socket connection intent. Presence readiness is **not** a prerequisite. A global
`Live` health requires connected socket and every durable root acknowledged;
presence failure has its own status and does not disable durable service.

### Retained invalidation rule

`observation_seq` uses checked **session-wide stamps**, strictly increasing for
each root, including after remove/reselect. It is not a per-root event count,
transport cookie, persistent cursor or receipt. `join_epoch` is the stamp of the
last acknowledged durable join. Gaps, pokes and joins advance invalidation; root
selection also allocates a fresh stamp but does **not** make the root ready.
Exhaustion fails the session rather than wrapping into an old equality.
A new `run` starts a new stamp domain; discard serviced transport stamps when
restarting it rather than carrying them across sessions.

The receiving controller should:

1. Attach/read the latest state and replace roots from validated inventory.
2. On joined readiness or a sequence different from the serviced sequence, queue
   a guarded refresh. Backstop/rescan can independently queue work.
3. At refresh **start**, capture the current sequence. Keep transport polling
   during all HTTP, lock and file awaits.
4. At completion, mark only that captured sequence serviced. If current differs,
   leave another pass pending; never clear new work merely because an older read
   completed. Coalesce bursts into one additional pass, not one HTTP read per poke.
5. Handle failures with bounded scheduling and continue other documents. Preserve
   dirty/divergent/invalid/pending sources through the corrected shared adoption
   contract. Keep slow reconciliation; polling fallback must report degraded,
   not a healthy live session.

No transport state goes into MDX. Joins/pokes only request reobservation; they do
not grant HTTP access, supply authoritative content or settle an agreement.

## Bootstrap, lifecycle and presence details

Discovery fetches the configured public landing page and directly referenced
same-origin `/_next/static/*.js` assets. It does not interpret bundles or follow
imports. Limits: 48 script references, 2 MiB landing page, 8 MiB each asset,
32 MiB total, 64 distinct project/key literals, 20-second request timeouts and a
90-second whole-discovery deadline. Redirects are rejected; MIME, path and origin
are checked. URL/key pairing is across the bounded asset set, filtered by public
`anon` role and matching hosted project ref. JWT decoding is filtering, **not
signature verification**; the server remains the authentication authority.

Explicit overrides use the verified anon-JWT join path. Opaque publishable keys
are not advertised or placed into a JWT `access_token` field without a verified
separate protocol adaptation. HTTPS origins are required except loopback HTTP.
Bootstrap/transport error and Debug values omit URLs, keys, root IDs and raw
private payloads. `State` serialization is an explicit caller/report boundary,
not an invitation to dump selection into tracing. No Leaflet cookie is held by
the bootstrap/socket path or sent to the external project host.

Managed dependencies are exact `phoenix-channel-client`, `phoenix-channel-runtime`
and `phoenix-channel-runtime-native` **0.1.1**, without tracing instrumentation.
Defaults: heartbeat every **20 seconds**, 10-second connect/join/heartbeat waits,
3-second leave waits, capped exponential 0.5–15-second recovery delays, 256 roots
and 256 events per bounded stream. Incoming frames/messages are capped at 2 MiB.
The socket reconnects after transient or ordinary clean closes; policy/auth close
codes stop it and expose unavailable state. No content/application sends exist.

`phx_error` uses the managed client's automatic rejoin. Unexpected `phx_close`
retains Leafbud selection and explicitly recreates that topic after backoff.
Ordinary close reasons are **not** terminal join rejections. Explicit auth/config
errors cancel channel intent and report unavailable; unknown join denials are
unavailable rather than retried forever, while known transient/rate/database
failures back off. A terminal root can be deliberately removed/reselected or the
session restarted after correcting configuration; unchanged selection does not
silently retry it.

Deselecting a root removes its public state immediately and cancels its worker.
Same-root reselect waits for the old topic worker to finish leaving before
registration. A timed-out dynamic leave cannot prove server membership is gone,
so the adapter resets the shared socket and catches up still-selected roots;
it does not leave a potentially joined ghost topic behind. Shutdown bounds all
pending leaves concurrently, then closes the socket. No retry can resurrect an
unselected root. Stale selection generations and old join-ref frames are ignored.

Lag on durable channels invalidates and requires a fresh acknowledged join;
lifecycle lag resets the socket. The upstream bounded event stream emits a
pending lag marker on a subsequent send after draining, not out of band. Surviving
burst pokes already retain invalidation; the controller's slow sweep remains the
backstop, not an assertion of lossless event history.

Passive presence uses only `presence.enabled: true` on `yjs:<root>` join. There is
no track, hello, awareness or doc-update send. Until a fresh snapshot, evidence is
`Unknown`; after it, `NoPeersObserved` or `ObservedPeers(n)` with local observation
time. Count connection metas by `phx_ref`, including multiple refs under one key
and `phx_ref_prev` replacement. Snapshot/diff decoding is transactional. Gaps,
lag or malformed input clear readiness and resubscribe for a fresh snapshot;
malformed leaves cannot expose partially applied joins. Derived presence memory
also has fixed ref/string limits. Count means observed connections, not people,
permission, document idleness or a reason to unsubscribe quiet roots.

## Verification

### Offline production adapter

`cargo nextest run -p leafbud-web`: **27/27 passed**, including existing web tests.
Strict all-target web Clippy with warnings denied and scoped formatting pass.
The new tests execute the production adapter over real loopback sockets/HTTP:

- Actual joins versus connect intent; duplicate root selection and graceful
  replacement; repeated poke burst while the receiver is not serviced.
- TCP loss, heartbeat timeout, generic `phx_error`/`phx_close`, automatic recovery
  with new join stamps, stale join-ref rejection, join timeout and terminal
  presence/root-system failure without a rejoin storm.
- Actual bounded-stream lag followed by rejoin; deselection during close retry;
  immediate remove/reselect; uncertain dynamic leave resets the socket without
  resurrecting the removed root; shutdown bounds unacknowledged leaves.
- Passive presence initial snapshot, same-key refs, ref replacement, partial
  departure, transactional malformed diff, gap-to-unknown and fresh snapshot
  recovery; presence failure leaves the durable poke channel healthy.
- Cancel during an incomplete handshake by directly polling the run future, no
  LocalSet; counter exhaustion/stale selection, public `Send + Sync`, redacted
  diagnostics, explicit provider without global installation, project/origin
  mismatch, cookie-free same-origin bootstrap, ambiguity, redirects and caps.

One workspace regression run compiled the coordinated 0.12.0 graph and passed
**189/191**. Its two failures were the concurrently edited MDX preparation tests:
`preparation_proposals_check_page_names_and_table_rows` and
`preparation_retargets_links_to_the_provisional_main_page`. They were left
untouched, not attributed to a transport regression or silently fixed here.
The parent owns the final integrated foundation/workspace gate.

### Separately authorized passive deployment check

The **production APIs**, not the old experimental socket client, ran against
`https://leaflet.pub` on 2026-11-09. The ignored scratch executable is
`.test-agent/production-live/` with its README. It privately revalidated the
existing binding via HTTP metadata, used production public discovery, then:

1. Joined the selected durable and presence topics; both became acknowledged and
   a fresh passive presence snapshot arrived.
2. Deselected and reselected that same root; a new acknowledged epoch and fresh
   passive snapshot arrived through production selection/leave handling.
3. Retained `Live` health through a **23-second window**, crossing the default
   20-second heartbeat interval, then completed shutdown. This aggregate state
   check is not a packet-level heartbeat-ack count or long-running soak proof.

Observed presence was **no-peers-observed**. No deployed poke or nonempty peer
diff/file-adoption claim is made. No fake peer, remote document edit/mint/publish,
imported-workspace modification or `~/l` write occurred. The initial TLS panic and
its correction are preserved above; the final check passed after stream/lifecycle
hardening as well. Exact project/key/root/cookie values remain private.

## Next commissioned join

Continue **`leafbud-editor-follow`, Sol high**, after parent review joins the
concurrent foundation correction. One coherent receiving slice should share
guarded one-document refresh with pull, add stable-ID cooperative locking for
Leafbud writers, and build the serial watch controller, selection rescan and CLI
transitions/backstop over this transport. Its payoff is actual supported remote
edits appearing in clean local files while local dirty/conflicted/pending input
survives—not another socket-only milestone.

The judgment is at source/adoption and cancellation/locking boundaries, not a new
realtime crate or general scheduler. Use this transport unchanged unless an
integration-specific property fails. No new ticket is needed; native push,
incremental CVR deltas, adaptive presence scheduling and live Yjs overlays remain
outside that first connected inbound delivery.

Connections: the [accepted build path](/design/keyed-build-path0.gpt6a.md#5-continuous-inbound-sync-is-an-early-deliverable)
owns the product direction; [experiment 15](/experiments/15-live-connection/live-connection0.gpt61s.md)
retains dependency/deployment investigation and its unmodified fixture evidence.
This isolated report records production implementation and does not replace
either source or edit their shared indexes during concurrent work.
