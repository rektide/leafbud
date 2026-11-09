---
type: ImplementationReport
title: "Connected inbound watch delivery"
description: "Shared typed guarded refresh, stable-ID cooperative locks, actual live subscription-driven adoption, selection/rescan/backoff, CLI JSONL/cancellation and production-pipeline verification."
resource: /design/connected-watch0.gpt61s.md
status: implemented
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e416ba4e7ffeZ1ThfycXYgNh5S"
    title: "Commissioned connected-watch implementation through CLI delivery, guarded writer integration, actual HTTP/WebSocket verification and passive deployed scratch smoke"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Accepted persistent-live section 5 and batch B commission"
  - resource: /design/live-transport0.gpt61s.md
    title: "Accepted production web adapter and captured-start-stamp receiving obligations"
  - resource: /design/authoring-corrections0.glm53m.md
    title: "Accepted format-2 authored/projection/preparation/settlement foundation"
  - resource: /design/guarded-pull0.glm53m.md
    title: "Predecessor original-byte-guarded one-shot adoption contract"
  - resource: /crates/workflow/src/refresh.rs
    title: "Typed one-document transaction shared with pull"
  - resource: /crates/workflow/src/lock.rs
    title: "Nonblocking OS-released stable-ID and import-binding locks"
  - resource: /crates/workflow/src/watch.rs
    title: "Serial fair controller, retained invalidation and cancellation"
  - resource: /crates/workflow/src/watch/selection.rs
    title: "Identity-anchored selection, raw pending preservation and rescans"
  - resource: /crates/workflow/src/watch/transport.rs
    title: "Bounded bootstrap/restart wrapper around unchanged accepted adapter"
  - resource: /crates/workflow/src/watch/tests.rs
    title: "Real production HTTP/Phoenix controller/adoption verification"
  - resource: /crates/cli/tests/watch.rs
    title: "Actual subprocess JSONL, selection, interruption and cross-process locking"
---

# Connected inbound watch delivery

## Result

**`leafbud watch FILE...` / `leafbud watch --all` now maintains actual live
subscriptions and adopts supported durable remote edits into clean current-format
local drafts.** This completes commissioned `leafbud-editor-follow`, not another
transport-only milestone. Coordinated version and exact internal pins: **0.13.0**.
The accepted `leafbud_web::live` implementation is unchanged.

Code checkpoints (explicit owned paths; no squash/push):

- `8fc7babd`: shared typed refresh and cooperative writers, live controller,
  selection/CLI and initial production-pipeline tests.
- `3ed20112`: coordinated 0.13.0, restart-domain/lifetime/reporting hardening,
  expanded loopback/subprocess tests and explicit subscription transitions.
- `bd4ba589`: canonical workspace/operand identity and symlink-root regression.
- `fb40d32b`: actual controller witness-only settlement and unsupported-remote
  preservation proof.

The [maintained usage guide](/README.md#connected-inbound-watch) owns flags,
JSONL and operational limits. New controller state lives only in memory; no body
history, raw CRDT overlay, on-disk replica, pending-attempt schema or auto-push
policy was introduced. Full metadata and null-cookie fact snapshots remain the
acquisition contract. Incremental CVR and adaptive presence scheduling stay deferred.

## Receiving interfaces and ownership

| API / file | Contract |
| --- | --- |
| `RemoteReader::refresh_document(context, path, refresh::Policy)` | Reusable typed guarded transaction; returns `refresh::Report` and `Outcome`, resolving relative paths against the workspace; no shell/CLI invocation |
| `refresh::Identity { id, binding }` | Captured expected local UUID and whole native binding; replacement paths/bindings cannot borrow the old selection |
| `refresh::Policy` | Explicit one-shot take/dry-run, follow preservation, optional expected identity/lifecycle and explicit-operand duplicate accounting |
| `pull.rs` | Existing one-shot report/count/exit mapping over the shared transaction; local-only remains one-shot `needs-decision` |
| `lock::{Store, Guard}` | Internal nonblocking OS file locks, stable local UUID key; a hashed origin/token key also serializes import discovery/creation |
| `RemoteReader::watch(context, request, options, cancel, emit)` | Local future polled alongside the socket driver; latest-state boolean cancellation and synchronous quick transition emitter; returns a final `Summary` |
| `watch::{Request, Options, Event, Detail, Summary}` | Selection, configurable intervals/deadlines/project/transport, versioned transition reports, current selected-file disposition totals |
| `watch/selection.rs` | Origin-scoped inventory; explicit operands anchored by local UUID through workspace renames; malformed/unbound/duplicate/pending cases remain loud |
| `watch/transport.rs` | Bounded public bootstrap retry and concurrently polled accepted adapter; new runs carry a new stamp domain |
| `cli/src/watch.rs` | Thin config/credentials/options, JSONL or concise transitions, terminal escaping, SIGINT/SIGTERM and report flushing |

Import and tidy now use the same lock store. Import reindexes **after** obtaining
its binding lock, locks an established local UUID before reading, and checks again
before no-clobber creation. Tidy rereads under the UUID lock and rejects duplicate
identities/bindings. Newly minted offline draft UUIDs need no shared predecessor
lock: creation is still random identity plus atomic no-clobber. Future push/edit
writers must join this cooperative lock boundary rather than add a second lock.

Lock files live beside the user config at `leafbud/locks`, independent of document
path/workspace. OS ownership is released on guard/process drop. Persistent empty
files are deliberately not deleted, interpreted as owners, or automatically
pruned. Separate user config homes are separate coordination domains. No content
is stored in a lock file. Small local inventory/parse/replace operations remain
synchronous; asynchronous HTTP and socket waits are concurrently polled.

## Correctness boundaries

### Read, classify, adopt

The shared transaction captures source identity, tries its operation lock without
waiting, rereads the original bytes and validates selected identity, binding,
origin, lifecycle and duplicates before observation. It retains those bytes
through every await. A non-null raw `@leafbud.pending` defers **before strict model
deserialization**; null/future unknown owned fields are rejected and preserved,
not removed to make parsing pass. There is no invented write-attempt schema.

Current endpoints are classified by the accepted receipt/projection implementation.
Remote-only changes adopt, equality can settle verified receipts, local-only is
normal watcher `local-dirty`, and divergence/no directional baseline is preserved
as `needs-decision`. New/unkeyed/retired authored objects remain local-dirty;
watch does not assign keys or materialize remote objects. Bound-no-receipt does
not imply history: only equal verified endpoints can settle automatically under
this explicit follow policy; differing endpoints stay untouched. A separate
one-shot pull makes a deliberate take-remote decision.

Metadata is reverified before writes; selection/duplicates are rescanned after
all network awaits, and the existing adjacent original-byte check guards atomic
replacement. Native association reconciliation retains manual source keys,
locally assigned document UUIDs and local-only author metadata. Equal bodies stay
byte-exact; equal whole files keep mtime. Removed paths are not recreated.

Advisory locking serializes cooperating Leafbud writers, **not arbitrary editors**.
The byte recheck detects saves across awaits but the final check/rename is not
filesystem CAS. Metadata/facts/reverification are not a server transaction or
remote write lock either. These limits remain explicit rather than claiming
absolute race exclusion.

### Connection, scheduling and selection

The controller attaches to latest state before driving the adapter. Initial live
catch-up requires actual `Joined` with nonzero epoch, not connect intent. Each
refresh captures the observation stamp at start; completion services only that
stamp. New pokes during a read require another pass, regardless of receiver
coalescing. A new transport run clears serviced stamps; completion from an older
run cannot service a numerically equal stamp in the new domain.

Only one refresh runs at a time. Round-robin selection and per-file capped retry
delays allow other documents to progress when a lock/HTTP request fails. Bursts
do not create one job per event; each document has at most one pending pass in
addition to an active cycle. The socket driver continues polling through HTTP,
retry and bounded shutdown. Socket gaps trigger rejoin catch-up; a configurable
slow sweep is the missed-poke/metadata backstop. If readiness is unavailable,
only that slow sweep can begin degraded polling, labeled `degraded-poll` and never
promoted to live health.

Inventory rescans update selected roots through `SetRoots`, following workspace
renames by stable local UUID. Duplicate UUIDs/bindings block every affected copy.
Changed/removed selections cancel obsolete jobs, and root remove/reselect cannot
inherit an old completed stamp. Recognizable roots keep poke subscriptions while
their source body is invalid, pending or newly authored; there is no presence-based
unsubscription. External explicit operands are checked, but rename discovery
outside the selected workspace is not recursive. New remote discovery stays import.

Bootstrap is scoped to the HTTP origin; errors are explicit with bounded waits
and fixed-delay retry (default 60 seconds). No deployed project/key is hardcoded.
Flags/environment can supply an explicit validated public project/anon JWT.
Cookie HTTP authority is independent of public topic access. Connection health,
per-file durable subscription status and adoption outcomes are separate reports.

Optional presence remains receive-only and advisory: unknown versus fresh
no-peers-observed versus observed connection metas, not people/idleness/permission.
Gaps clear evidence; freshness timestamps remain visible. No tracking, hello,
awareness, document update or fake peer is sent.

Cancellation drops active HTTP work and releases its guard before another write,
then bounds best-effort leaves/socket shutdown. Source/receipt state advances only
after a successful guarded local write. JSONL emits a final summary; gracefully
stopping a session does not imply every source is clean or freshly observed.

## Verification

**Final integrated gate:** `cargo nextest run --workspace --no-fail-fast` —
**221/221 passing** across 13 binaries; `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` pass. README and this
receipt pass Rekon's explicit Markdown/OKF link/provenance checker.
The prior accepted web 27-test/foundation 200-test receipts were trusted; they were
not mistaken for verification of this new controller. New tests exercise actual
production adoption and real loopback HTTP/Phoenix sockets, including:

- Poke → supported MDX update; repeat poke → exact bytes/mtime no-op; manual keys,
  local UUID and local-only metadata survive.
- Unacknowledged initial join prevents live reads; poke burst during a held first
  snapshot causes a second acquisition; heartbeats continue during the slow read.
- Socket reconnect catches a missed edit; passive empty presence remains advisory.
- Source save during acquisition survives; concurrent pull/tidy report busy;
  cancellation releases the writer lock without advancing receipts.
- Fair lock/error retries across documents; missing HTTP/auth outcomes preserve
  files and do not prune; two imports serialize one binding and recheck destination.
- Rename, duplicate introduction/removal, changed root during read, deselection
  and old-path loss preserve identity/selection and never recreate removed files.
- Local-dirty/diverged/unkeyed/unbound/invalid/pending/unknown owned state and
  bound-no-receipt unequal/equal current endpoints have distinct safe outcomes.
- A witness-only poke updates only the remote receipt line; subsequent unsupported
  remote data preserves those bytes and receipts exactly.
- Bootstrap failure is degraded and bounded, only slow sweep polls; cancellation
  during discovery, incomplete join and active refresh is finite.
- A new transport stamp domain with a colliding sequence cannot be serviced by
  an old in-flight read (state restart is injected; real HTTP/adoption still runs).
- Actual CLI process JSONL flushing, operand/empty selection, forbidden persistent
  take mode, SIGINT/SIGTERM/final summary, and cross-process advisory-lock release.
- A symlinked workspace root and a relative shared-refresh operand do not invent
  duplicate identities from canonical-versus-alias path spellings.

### Authorized passive deployment smoke

The **actual 0.13 CLI**, not an experiment-only socket client, freshly imported
one existing draft into ignored `.test-agent/connected-watch/workspace/` on
2026-11-09. Prior probe binding and normal cookie credentials were read privately;
no token, project key, native root or document body was emitted to logs.

The fresh current-format file then ran through `watch --presence --json`:

1. Public bootstrap succeeded; durable subscription was acknowledged and initial
   guarded HTTP catch-up reported unchanged.
2. Passive presence supplied a fresh **no-peers-observed** snapshot.
3. Live readiness persisted for a **23-second window**, crossing the default
   20-second heartbeat interval; SIGINT produced graceful exit and final summary.
4. **Zero local writes; exact byte and nanosecond-mtime preservation.**

No deployed poke or remote edit was observed. This proves deployed bootstrap,
connection/catch-up/no-op/interrupt integration, **not deployed poke-to-file
adoption**. Deliberate adoption events are loopback-proven. No remote mint/edit/
publish, fake presence or `~/l` conversion/deletion/write occurred. The scratch
README/script document the bounded opt-in proof without exposing private inputs.

## Remaining scope and road ahead

This delivery follows durable **draft** editor state only. Published-object
adoption, raw/live Yjs overlays, incremental CVR, automatic pushes, adaptive
presence scheduling and remote discovery/import while watching are not added.
Unknown pending state is preserved for future recovery; it is not executable
recovery. External editor/file-system and separately fetched remote snapshot
races retain the limits above. Old `~/l` data is intentionally untouched/stale.

Next proposed batch is **`leafbud-editor-bind` + `leafbud-editor-recovery`, Sol
high**, for the external native-effect/ambiguous-outcome boundary: current mint/
push protocol, writer identity, bounded requests, actual acknowledgments and
compact attempt evidence. This is an internal transport/recovery slice, not a
claim that first binding is complete before content materialization. Then
**`leafbud-editor-text` + `leafbud-editor-structure`, g**, can join the loaded-Yrs
text and straightforward keyed create/update/place/detach compiler to that seam
for the full drafting loop. Those four existing tickets remain open; follow does
not authorize implementing their remote effects or closing their completion
dependencies early. The parent owns their next commission and architecture join.

Connections: [accepted build path](/design/keyed-build-path0.gpt6a.md#5-continuous-inbound-sync-is-an-early-deliverable)
owns the capability; [transport receipt](/design/live-transport0.gpt61s.md) owns
socket/bootstrap proof; [foundation corrections](/design/authoring-corrections0.glm53m.md)
own parser/projection/settlement prerequisites; [guarded pull predecessor](/design/guarded-pull0.glm53m.md)
retains original one-shot evidence. This receipt records their integrated use.
