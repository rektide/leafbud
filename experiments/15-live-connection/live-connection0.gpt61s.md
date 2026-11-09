---
type: Research
title: "Leaflet live connection: passive deployment proof and watch handoff"
description: "Verified public bootstrap, rootEntity/Yjs channel access and untracked presence; maintained Rust Phoenix recommendation and narrow transport/workflow contract."
resource: /experiments/15-live-connection/live-connection0.gpt61s.md
status: probe-complete-integration-uncommissioned
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e41f53633ffeI4wwnXfMRs1LeT"
    title: "Independent source/dependency inspection, passive deployed connections, executable lifecycle fixtures and integration recommendation"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Accepted section 5 connected watch and presence boundaries"
  - resource: /crates/leaflet/web/src/lib.rs
    title: "Cookie-scoped HTTP client and redacted errors"
  - resource: /crates/leaflet/web/src/pull.rs
    title: "Current full-snapshot read-only pull"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Shared guarded observation and endpoint classification before g's concurrent authoring revision"
  - resource: /crates/workflow/src/pull.rs
    title: "Single-file refresh is currently private and command-shaped"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/index.tsx
    title: "Poke subscription, reconnect pull and five-minute backstop"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/yjsRealtime.tsx
    title: "Explicit presence enablement, mounted editor gate and track/hello side effects"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/browserClient.ts
    title: "Public deployment project URL/key bootstrap"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/TextBlock/useCollabText.tsx
    title: "Ephemeral Yjs path is separate from debounced durable writes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/serverPullData.ts
    title: "Null-cookie full reads and CVR-backed delta path"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cvr.ts
    title: "Full versus delta payload semantics; an optimization, not a live transport"
  - resource: https://supabase.com/docs/guides/realtime/protocol
    title: "Read current v1/v2 wire, heartbeat, presence, close and join-error contract"
  - resource: https://github.com/refmdio/phoenix-channel-runtime/tree/07ea4e1322ce0ca624d770efbb3861996e91b99b
    title: "Inspected published 0.1.1 client/runtime/native sources and executed client against deployment and loopback fixtures"
  - resource: https://github.com/roberts-pumpurs/supabase-rs-rp/tree/bb56bdbc94367f2f33c9483b6a6243c7c3c6109f/crates/supabase-realtime
    title: "Inspected published rp-supabase-realtime 0.12.0 raw lifecycle, typed protocol and high-level auth coupling"
  - resource: https://github.com/Brainwires/rust-supabase-client/tree/79e4804541130e4677bf878169f7b2e05abe9737/crates/supabase-client-realtime
    title: "Inspected published 0.3.0; presence enabled flag is not serialized"
  - resource: https://github.com/Scaraude/supabase-realtime-rs/tree/1d16be5a32f467484079e4380ae499110127aa78
    title: "Inspected published 0.1.2 configuration and connection send boundary"
  - resource: https://github.com/joshuachp/phoenix-chan/tree/707e705f82a2f857a5f15959737fd235e022eb7c
    title: "Inspected published 0.4.6 join/leave/heartbeat behavior"
---

# Leaflet live connection: passive deployment proof and watch handoff

## Result and limits

**The actual persistent realtime seam is accessible.** On 2026-11-09, passive
connections to the current `https://leaflet.pub` deployment successfully joined
both topics for one existing validated bound document. The Leaflet HTTP cookie
was **not** sent to Supabase. Presence state arrived with `enabled: true` and
**without `track`, `hello` or awareness broadcasts**.

Recommend the maintained **`phoenix-channel-client = 0.1.1` +
`phoenix-channel-runtime-native = 0.1.1`**, encapsulated in `web::live`. It worked
against the deployment using protocol **2.0.0**. This avoids implementing a second
heartbeat/reconnect/join driver. Keep `tokio-tungstenite` as the tested low-level
fallback, not the first production choice.

This independent probe did **not** implement watch, change production code, alter
`~/l`, exercise a deployed document edit or observe a real durable poke. There
were no observed peers in the sampled window. Do not close `leafbud-editor-follow`
or claim its file-adoption acceptance on this evidence. Parent review of g's
author-create/author-keys revision remains the integration join point.

## What was actually exercised

| Layer | Executed result | What it does not establish |
| --- | --- | --- |
| Public discovery | Unauthenticated landing page, 11 referenced script assets inspected; matching project URL and public JWT found | No stable official Leaflet bootstrap API |
| Existing binding | Read-only local selection and authenticated `get_leaflet_data`; root matched | No new discovery, permission grant or mutation |
| Minimal protocol v1 | Two fresh socket epochs, both topic joins acknowledged each time, 3 heartbeat acks per epoch, both leaves acknowledged | Not an automatic reconnect implementation |
| Maintained client v2 | Initial join and explicit socket reconnect, both channels rejoined, 4 explicit heartbeat acks per epoch, passive snapshot each epoch, acknowledged leave and shutdown | No forced deployed network failure or server restart |
| Passive presence | Initial `presence_state` received in all four successful connected epochs, 0 connection metas | No proof that all editors are absent; nonempty deployed diffs not observed |
| Durable/Yjs traffic | 0 root pokes and 0 Yjs broadcasts during the minimal-protocol sample | No deployed content-change propagation proof |
| Offline real sockets | Automatic reconnect after TCP loss, heartbeat timeout recovery, `phx_error` rejoin, adapter recovery after `phx_close`, join rejection/timeout and proper leaves | Loopback events are synthetic, not deployment evidence |
| Pure fixtures | Event-during-read coalescing and presence snapshot/ref/diff/gap rules pass | Not filesystem locking, inventory rescan or adoption integration |

Credentials, document IDs and raw deployment data remain only under ignored
`.test-agent/live-connection/`; neither findings nor fixtures reproduce them.

Two probe defects were corrected before the successful results: the low-level
Rustls connection needed an explicit crypto provider, and an explicit managed
`ping()` immediately after requesting reconnect raced an unconnected socket.
The latter reinforces the API distinction: a reconnect request is not rejoin
readiness. No values from failing responses were logged.

## Bootstrap and authorization

The [browser initializer](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/browserClient.ts)
uses `NEXT_PUBLIC_SUPABASE_API_URL` and `NEXT_PUBLIC_SUPABASE_ANON_KEY`. Their
compiled values are obtainable from current same-origin public Next assets.
The discovered JWT's role is `anon`; it is not a Leaflet login or service-role key.
The exact URL/key are saved in ignored `bootstrap.json`, not hardcoded here.

The verified wire endpoint is derived as:

```text
wss://<public-project-host>/realtime/v1/websocket?apikey=<public-key>&vsn=2.0.0
topic: realtime:rootEntity:<validated-binding-root>
topic: realtime:yjs:<validated-binding-root>
```

The Supabase JS helper prepends `realtime:`; Leafbud's raw Phoenix adapter must
include it exactly once. Join configuration uses `private: false`, empty Postgres
changes, broadcast `{ack:false,self:false}`, and `presence.enabled: false` for
durable pokes / `true` for the optional Yjs observer. The public key is supplied
as the join `access_token`; no Supabase user sign-in or refresh session is needed
for the proven public-topic access. No Leaflet cookie crosses that origin boundary.

**Join success is not document-read authorization.** Topic access and the
cookie/native-token HTTP read are separate boundaries. Treat a poke as untrusted
invalidation, never authoritative content, permission or receipt evidence. Keep
root identifiers and full socket URLs out of diagnostic errors. This probe did
not enumerate other roots, test private channels or assess deployment security.

Production recommendation: a small, origin-scoped bootstrap resolver with explicit
project URL/public-key override plus a bounded same-origin asset discovery path.
Discovery must fail clearly if no unambiguous pair exists; never silently use a
different deployment's cached project. Avoid a general browser-bundle interpreter.
The [helper](bootstrap.py) demonstrates the bounded path but is not production
configuration code. Restrict redirects, assets, sizes and script count; verify
explicit inputs independently. Decode JWT claims only for discovery filtering,
not as cryptographic verification or proof of authorization.

## Dependency decision, from inspected sources

Published crate source archives were downloaded and read; versions below are
actual registry releases, not imagined APIs. Source revision links are in OKF.

| Candidate | Concrete fit / obstacle | Decision |
| --- | --- | --- |
| `phoenix-channel-client` + native `0.1.1` | Multiplexed channels, acknowledged join/leave, automatic heartbeat/reconnect/rejoin, bounded events with explicit lag, presence reset/resync, separate channel/socket states; deployed v2 proof passed | **Recommend**, with the small adaptation below |
| `rp-supabase-realtime 0.12.0` | Strong raw reconnect driver and explicit presence enablement; high-level API is auth-session oriented. Raw `RealtimeBaseConnection` avoids login and supports multiplexing. However `ProtocolPayload` has `PhxClose` but no `PhxLeave`; registry treats outbound `PhxClose` as removal | Not selected: clean dynamic unsubscribe needs library work or socket replacement, not an invented `leave()` API |
| `supabase-client-realtime 0.3.0` | Maintained heartbeat/reconnect and correct `phx_leave`. `ChannelBuilder` records `presence_enabled`, but `PresenceConfig` contains only `key` and `subscribe()` never serializes `enabled` | Not selected for this presence requirement without upstream correction; do not mistake callback registration for enablement |
| `supabase-realtime-rs 0.1.2` | Explicit enabled flag exists; reconnect/heartbeat facilities exist. `ConnectionManager::send_message` returns `Ok(())` with no writer, weakening disconnected-send evidence | Not preferred when the first candidate has a clearer lifecycle and executed proof |
| `phoenix-chan 0.4.6` | Correct join/leave sends; `join_with_payload` returns a sent message ID, not acknowledgment. Caller must assemble readiness/recovery beyond its basic receive/heartbeat API | Less useful than the managed client for this task |
| `tokio-tungstenite 0.30.0` | Minimal protocol v1 control probe works; established transport, not a Phoenix lifecycle client | Acceptable fallback only if managed integration has a real blocker |

### Managed-client adaptations to make explicit

1. **Runtime locality:** handles/driver use `Rc` and local futures, not `Send`.
   The probe uses Tokio `LocalSet`/`spawn_local`. Production can instead poll the
   driver and adapter/controller concurrently on the current task. Do not put
   these handles into `tokio::spawn` or add a dedicated service/thread by habit.
   Keep public Leafbud control/state channels `Send`; keep the local driver hidden.
2. **Heartbeat:** default is 30 seconds; current Supabase guidance requires at
   least every 25. Set 20 seconds, with a finite acknowledgment timeout and bounded
   handshake/join/leave waits. The probe used 10/8 for bounded evidence.
3. **Join readiness:** `Channel::join().await` awaits the server reply; socket
   `connect()`/`reconnect()` only accepts connection intent. Observe each
   `ProtocolEvent::Joined` to schedule catch-up, including automatic rejoins.
4. **Unexpected `phx_close`:** generic Phoenix treats it as final, clearing desired
   intent. `phx_error` automatically rejoins. Leafbud must retain its selected
   root intent and explicitly rejoin after an unexpected close, with bounded
   backoff and preceding `system`/auth/rate-limit classification. The executable
   test demonstrates rejoining the same channel handle after `ChannelClosed`.
5. **Join rejection:** the generic client schedules retries. On terminal invalid
   auth/config errors, cancel that channel's intent (`leave`/drop) and expose
   degraded/unavailable, rather than permitting an endless rejection loop. Transient
   restart/database/rate errors get backoff. Do not log raw server payloads.
6. **Lag/staleness:** bounded `ChannelEvent::Lagged` is explicit. A durable-topic
   lag must dirty that document and require reobservation; a lifecycle-event lag
   should conservatively rejoin/catch up. Presence lag/decode/gap becomes unknown
   until a new snapshot. Never label an empty stale map as ready zero.
7. **Wire scope:** use text JSON v2 for these existing Leaflet messages. The client
   is a generic Phoenix codec, not evidence for every newer Supabase binary/replay
   extension. Pokes and pinned Leaflet Yjs payloads are JSON; no live Yrs overlay
   or arbitrary binary broadcast capability is being promised.

## Exact narrow integration recommendation

Names below are proposed Leafbud interfaces, **not already implemented APIs**.
Do not add a realtime crate or put sockets in MDX/editor/compiler code.

### `leafbud_web::live`

```rust,ignore
struct Bootstrap { /* validated project origin + redacted public key */ }
struct Options { heartbeat, ack_timeout, backoff, presence: bool }
enum Command { SetRoots(Vec<Uuid>), Stop }
struct State { health: Health, roots: BTreeMap<Uuid, RootState> }
struct RootState {
    subscription: SubscriptionState,
    // Increment after every acknowledged join AND every durable poke/lag.
    observation_seq: u64,
    join_epoch: u64,
    presence: PresenceEvidence, // Unknown | NoPeersObserved | ObservedPeers(n)
    // Local observation/freshness time; no peer payload or identity retained.
}
async fn run(
    bootstrap: Bootstrap,
    options: Options,
    commands: mpsc::Receiver<Command>,
    state: watch::Sender<State>,
) -> Result<()>; // local future; poll alongside controller, not tokio::spawn
```

Use a **latest state channel with monotonic per-root observation sequences**, not
one queued event for every poke. This is sufficient for invalidation: coalescing
notifications cannot lose a required refresh because the latest sequence remains
different from the controller's serviced sequence. No persistent event queue,
historical event replay or generic realtime abstraction is required. State map
size is bounded by selected roots. Treat counter exhaustion as session failure,
not wraparound equality.

`web::live` owns one socket per configured origin/project, unique root topics,
join payloads/acks, heartbeat, retry/close classification, stale-generation
filtering and optional passive presence state. Its own processing must stay live
while HTTP/file work runs. It must not await a document refresh, parse MDX, fetch
HTTP editor facts or write files. Quiet documents retain durable subscriptions.
Presence failure must not take a healthy durable topic out of service.

### `workflow::editor::watch`

```rust,ignore
async fn watch(context, selection, options, cancellation, report_sink)
    -> Result<WatchSummary>;

// Shared with one-shot pull, using the revised model/adoption contract after g:
async fn refresh_one(context, document_identity, current_path, policy)
    -> Result<RefreshOutcome>;
```

`watch` owns inventory/root/path selection, duplicate detection, periodic rescans,
serial refresh scheduling, cooperative document locks, adoption and transition
reports. The shared `refresh_one` must report `local-dirty` as normal preserved
authorship, not the current one-shot resolution suggestion as a repeated failure.
Use guarded default policy; persistent `--take remote` is not accepted. Pending,
invalid, missing, unsupported and divergent sources remain preserved.

Concrete race rule per document:

1. Subscribe; only an acknowledged `Joined` increments `observation_seq` and makes
   the live subscription healthy. Read current state after attaching the receiver.
2. If current sequence differs from the serviced value (or a backstop/rescan
   requires work), mark pending. Acquire the stable-ID lock and read latest local
   bytes/path/inventory; initial selection must not become a stale permanent path.
3. **At read start**, capture the current sequence and clear pending. Start at
   most one `refresh_one` for that document; continue consuming transport state
   while its HTTP/file future runs. The transport driver must also keep polling.
4. At completion, mark only the captured sequence serviced. If a poke or rejoin
   incremented it during the cycle, leave pending and run another pass. Never
   clear new pending work at completion. Coalesce a burst into one extra pass.
5. Failures get bounded retry/backoff; do not spin a failing document simply
   because its old sequence remains unserviced. Continue other documents.
6. Every gap marks stale; every confirmed rejoin dirties again. Full metadata +
   null-cookie facts are the first acquisition path. Fetch metadata even when
   facts compare equal. A slow configurable sweep (upstream uses five minutes)
   repairs missed pokes and metadata-only changes. Fallback polling is degraded,
   never described as a healthy live session.

Keep transport sequence/cookie/cache out of MDX receipts. A full read is transport
recovery, not permission to advance agreement on conflict. Later CVR deltas can
replace snapshot acquisition without changing this lifecycle; cookie and full
base map must advance together in memory, never cookie-only on disk.

## Presence: useful evidence, limited meaning

The passive probe answers the narrow open question: **tracking is unnecessary to
receive an initial presence state on this deployment**. Enabling presence on join
does not itself add our watcher as a tracked editor: the received state remained
empty. The probe did not manufacture a client ID, hello or awareness payload.

Source explains why restraint matters: Leaflet opens `yjs:<root>` after an editor
registers a text doc; it tracks a random connection key with `awarenessClientID`.
Its join callback can announce awareness to a newly tracked peer. Tracking our
watcher would therefore alter browser send gating/traffic. Do not do it merely
to obtain a count.

Report `unknown` until a fresh `presence_state`; after that report
`no-peers-observed` or `observed-peers(n)` with observation time and generation.
Count **connection metas** by `phx_ref`, not unique people. Pinned Leaflet normally
uses a random key per connection, but same-key multiple metas are legal; diffs
must remove only the departing refs. On gap/lag/rejoin reset readiness and state.
The synthetic fixture covers same-key refs and a replacement ref; nonempty
deployed presence/diffs remain unverified.

No observed peers does not prove no Leaflet users: unmounted/no-text/read-only
contexts and older clients may not track; stale/disconnected observers know less.
There is no account-wide active-document directory here. Subscribe per selected
root; do not enumerate guessed channels. Presence is optional advisory priority
for catch-up, never authority, conflict resolution, or a reason to disable quiet
root subscriptions. Do not make an adaptive scheduler necessary for first watch.

## Verification and next join

Experiment checks: **7/7 nextest tests**, all-target clippy with warnings denied,
Rust formatting and Python/fixture parse checks. Successful deployed v1 and managed
v2 results are listed above. These are receive-only bounded proofs, not soak tests.

**Next: `leafbud-editor-follow`, Sol high**, after parent accepts g's model/adoption
handoff. Implement the small `web::live` adapter and shared guarded refresh/lock
boundary, then the serial watch controller in the same coherent ticket. The main
judgment is preserving continuously serviced live invalidation while source/file
decisions remain conservative. g can carry fixture expansion once this seam and
the revised state model are integrated. No new ticket is needed for the probe.

Still to prove in that commissioned integration: file adoption after browser-like
edits, pokes during reads, real local dirty/conflict/pending guards, equal mtime,
rename/duplicate/rescan, lock interaction with explicit writers and cancellation.
A later operator/browser-driven deployed edit can verify actual poke-to-file
delivery; this probe's passive scope did not authorize manufacturing one.

Connections: the [accepted build path](/design/keyed-build-path0.gpt6a.md#5-continuous-inbound-sync-is-an-early-deliverable)
owns the product direction; this report supplies missing deployment/library evidence.
The [draft internals experiment](/experiments/02-draft-internals/REPORT.md) supplies
earlier native pull grounding; a persistent socket adds lifecycle, not another
content authority. Shared indexes/designs were deliberately not edited during
this independent probe; parent can link this evidence after review.
