---
type: Design
title: "Building keyed authoring and continuous inbound sync"
description: "Code-grounded domain boundaries, concrete interfaces, bounded coarse operations, revisable formats and a build path for complete drafting plus persistent live watch, with exploratory peer presence."
resource: /design/keyed-build-path0.gpt6a.md
status: accepted-direction
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
updated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Grounded implementation and rebuildable formats; operator clarified persistent live watch/sync and exploratory presence, then accepted the build direction and commissioned implementation"
  - resource: /design/keyed-edit-flows0.gpt6a.md
    title: "Primary create/change/drop/reorder cases and identity obligations"
  - resource: /design/keyed-authoring0.gpt6a.md
    title: "Complete drafting boundary and keyed native-object foundation"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Existing eight-crate decision, root commands, conflict policy and explicit recreation interface"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Author intent, compact binding/receipt ownership and missing-versus-null publication"
  - resource: /design/document-sync1.gpt61s.md
    title: "Current-endpoint resolution and separation of transport cursors from agreement evidence"
  - resource: /crates/document/model/src/lib.rs
    title: "Current mandatory bound State, Node and editor_value implementation"
  - resource: /crates/document/mdx/src/lib.rs
    title: "Current strict parser, block spans, checked rendering and whole-body refresh fallback"
  - resource: /crates/leaflet/editor/src/lib.rs
    title: "Strict fact scan, occurrence mapping, ordering and derived title"
  - resource: /crates/leaflet/editor/src/text.rs
    title: "Existing strict production Yrs reader"
  - resource: /crates/leaflet/web/src/pull.rs
    title: "Full snapshot only, fixed read-only client group and retained raw response"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Shared classification/adoption; imported UUID equality and discarded raw observation"
  - resource: /crates/workflow/src/pull.rs
    title: "Existing guarded per-document refresh and metadata recheck"
  - resource: /crates/workflow/src/files.rs
    title: "Actual byte recheck, fsync and atomic rename; no cross-process lock or filesystem CAS"
  - resource: /crates/workflow/src/inventory.rs
    title: "Local selection, duplicate detection and raw header indexing"
  - resource: /crates/workflow/src/import.rs
    title: "Import-specific identity/title construction and shared witness helpers"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Executed Yrs/Yjs interop and unsafe independent replacement/delta/retry evidence; its LCS is not the proposed production algorithm"
  - resource: /experiments/15-live-connection/live-connection0.gpt61s.md
    title: "Reviewed commissioned Sol probe: deployed passive joins/presence, maintained Phoenix client lifecycle evidence and production watch API handoff"
  - resource: /design/authoring-review0.gpt6a.md
    title: "Parent review of landed batch A and concrete correction/transport join conditions"
  - resource: /design/live-transport0.gpt61s.md
    title: "Production web transport checkpoint accepted after parent source review; workflow/CLI adoption remains the next join"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/index.tsx
    title: "Source-inspected rootEntity poke subscription, reconnect pull, poll backstop and browser push batching"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/serverPullData.ts
    title: "Source-inspected CVR incremental pull and null-cookie/cache-miss full-snapshot behavior"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cvr.ts
    title: "Source-inspected clear/put/del patches, unchanged cookies and mutation acknowledgment changes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/yjsRealtime.tsx
    title: "Source-inspected per-document Yjs channel, presence enablement/tracking, peer suppression, hello fallback, awareness and doc-update messages"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/TextBlock/useCollabText.tsx
    title: "Separate ephemeral live text path and debounced durable Replicache writes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/browserClient.ts
    title: "Browser realtime connection uses deployment public Supabase URL/key; CLI discovery/auth adaptation is unproven"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "Re-read membership-preserving moveBlock and addLastBlock"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/push.ts
    title: "Re-read request identity, skipped-but-acknowledged mutations and post-commit poke"
---

# Building keyed authoring and continuous inbound sync

## Recommendation

Build **one document synchronization workflow with two directions**: explicit
keyed saves to Leaflet, and a persistent live connection following Leaflet changes
into local files. Reuse guarded adoption as an internal operation, not as a
substitute for the live session. Keep the
existing source identities and native objects; bound complexity with whole-block
text interiors, whole-page ordering passes and whole-body local rendering where
those are adequate. The [flows](keyed-edit-flows0.gpt6a.md) remain the behavioral
examples; this document supplies the code boundaries and build sequence.

The operator explicitly allows format revision and rebuilding `~/l`. Recommend a
clean next source format rather than a migration framework for that disposable
corpus. Rebuildability removes a compatibility burden; ordinary future saves still
need stable identities and protection for newly authored work.

The operator accepted this build direction on 2026-11-09 and commissioned the
next implementation work: g on `leafbud-author-create` + `leafbud-author-keys`,
then Sol high on connected watch/presence after parent integration review.
Exact API sketches remain implementation choices within these boundaries.
This design's source inspection is not runtime/deployed-write evidence. The local
Leaflet archive was clean at `600c7b5f…` during inspection. The existing interface's
eight-crate count and exact source schema are revisable; the shared protocol crate
is introduced when its write-transport consumers land, not as empty scaffolding.

**Operator correction:** the first checkpoint of this document proposed a
polling-first `pull --watch`. The operator clarified that the desired capability
is continuous watch/sync with a live Leaflet connection. This revision makes
subscription-driven watch the deliverable; polling becomes its backstop. Peer
presence is an exploratory scheduling opportunity, not a requirement to build a
multiplayer editor before watch is useful.

## 1. What the code actually needs

| Current seam | Concrete change | Why it matters |
| --- | --- | --- |
| `document/model::State` requires binding and sync | Allow unbound, bound-without-agreement and pending states; validate their combinations | First local draft and interrupted first save become expressible |
| `mdx::Reader::page` requires every Key; `parse_with_spans` requires exact key/table equality | Separate source validity, key preparation and confirmed-result validity | Adding and dropping paragraphs stops being a parse error |
| `Document::editor_value` assumes every key has a row | Define a new receipt interpretation for new/retired associations; never encode all missing rows as the same null identity | Structural edits can be classified without fabricated bindings |
| `workflow::observe_editor` returns only metadata, merged document and agreement | Retain the raw fact view alongside typed meaning | Write compilation must load actual current CRDT bytes |
| `same_identity` compares local UUID to a freshly imported token-derived UUID | Validate native binding independently; preserve local UUID | A locally created document can survive its first pull |
| `import::build` can derive title from first prose | Separate initial import's display-title choice from bound observation's persisted metadata | Inserting a heading cannot masquerade as a document rename |
| `mdx::update` combines metadata/body/state patching | Expose separate tool-state settlement and remote-content adoption operations | Successful push must not reformat the author's body |
| `web::pull` accepts only a clear-prefixed full snapshot and uses one fixed group | Add explicit client-group and acknowledgment interfaces; retain full reads | Recovery needs the writer's group, not the read-only group |
| Pull's `try_refresh` is a private command-shaped operation | Extract a reusable per-document cycle | One-shot pull and connected watch can share adoption decisions and writes |
| File writes re-read bytes then rename | Add cooperative operation locking and accurately describe the remaining race | Pull watcher and explicit push must not race each other |

These changes are more important than a clever diff. The existing pure text
reader, Markdown-first renderer, scoped agreement tests and import/pull fixtures
are useful foundations, not throwaway scaffolding.

## 2. Domain groups and dependency boundaries

### Recommended destination

Keep the present groups. Add **one small shared native-protocol crate when the
write transport lands**, so the HTTP client and pure editor use the same wire
types without depending on each other's implementations:

```text
crates/
  document/
    model/           current content, source keys, bindings, receipts, attempts
    mdx/             parse, prepare keys, render, adopt remote, settle tool state
  leaflet/
    protocol/        facts, native mutation arguments, pull/push envelopes
    editor/          observe graph, edit Yrs, compile native changes, verify
    web/             HTTP, auth, mint/actions, pull/push, explicit-ID reads
    records/         public-record types and projections
  atproto/           AT Protocol transport and identity
  workflow/          workspace IO and synchronization use cases
  cli/               arguments, report rendering, process lifetime
```

Consumers depend downward: MDX → model; editor → model + protocol + Yrs;
web → protocol; workflow → model + MDX + editor + web; CLI → workflow.
`protocol` depends on serialization/identifier libraries, not Yrs, HTTP, files
or the authoring model. It preserves unknown raw fact fields for witnessing;
typed access must not silently discard them. It models only the native messages
we actually consume or emit, not a second public lexicon system.

The concrete payoff is one definition of `FactId`, `NativeMutation`,
`PullResponse`, mutation IDs and acknowledgment data on both sides of the
compiler/transport seam. A native mutation enum is the server vocabulary, not
an invented semantic operation language. Keep JSON at genuinely open payload
boundaries; do not build a universal typed fact database before this save works.

### Modules before more crates

Split the growing implementation by responsibility as it is touched:

```text
document/model/src/  content.rs tracking.rs agreement.rs
document/mdx/src/    parse.rs keys.rs render.rs patch.rs view.rs
leaflet/editor/src/  graph.rs observe.rs text/{read,write}.rs
                    write/{blocks,pages,order,metadata}.rs verify.rs
leaflet/web/src/     pull.rs push.rs mint.rs actions.rs live.rs
workflow/src/
  workspace/        config.rs inventory.rs files.rs lock.rs
  editor/           observe.rs agreement.rs adopt.rs save.rs recovery.rs watch.rs
```

The tree names ownership; it is not an instruction to create every file empty.
Move hash/adoption/identity helpers out of `import.rs` as other callers need them.
Import then becomes an initial-adoption caller rather than the owner of all reads.
Rename `RemoteReader` to an operation-appropriate facade when writes arrive.

Two viable alternatives:

- **Keep eight crates:** place message types in `editor::wire` and let web depend
  on editor. Fewer packages, but the transport acquires the editor/Yrs dependency.
  Acceptable if the small protocol crate proves mostly passthrough boilerplate.
- **Separate local workspace and sync crates:** move filesystem/config/inventory
  into `workspace`, orchestration into `sync`. Useful if there is a second caller
  with a genuinely independent lifecycle. Today internal domain modules give the
  same ownership clarity without another pair of public APIs.

Recommend the first tree, not separate crates for each block type, diff strategy,
retry state or command. Extracting text alone is also unnecessary: its schema is
Leaflet-specific and its first consumers are the same editor read/write paths.

## 3. Source format and the few runtime values

### A next-format sketch

Use `leafbud-mdx/2` for the coherent authoring changes, with a new editor receipt
profile if canonical association meaning changes. This is a recommendation, not
a format already recognized by the binary. A minimal created document can be:

```mdx
---
format: leafbud-mdx/2
title: Garden notes
"@leafbud":
  id: "11111111-1111-4111-8111-111111111111"
  state: draft
---

# First observations

The seedlings are growing.
```

Ordinary top-level content is the main page. Explicit named pages remain available;
choose one unambiguous grammar for mixing main content and named definitions in
parser fixtures. Main-page identity is persisted during preparation, independent
of its first heading. Keep existing `<Key id="…" />` syntax initially; source
prettiness does not require another key grammar to ship this workflow.

Keep these state concepts, with no old body or saved CRDT:

- `id` and lifecycle: stable local document identity.
- `binding: Option<Binding>`: confirmed native document/root/entry/context.
- `nodes`: **confirmed** page/block associations, including rows retired locally
  but not yet detached remotely. New keys can be absent from this table.
- `sync: Option<EditorReceipt>`: last verified agreement, never merely an attempt.
- `pending: Option<Attempt>`: compact unresolved effect evidence.

Use explicit page/block locator variants rather than today's
`memberships: {fact_id: fact_id}` duplication. A block occurrence has an entity,
one placement fact and active/inactive text locators; a page has its entity and
known incoming reference facts. Multiple block keys may share an entity, but
still identify distinct occurrences. Parent/order are read from current facts,
not treated as immutable identity in a cached locator.

Give publication intent three states: absent/inherit, null/standalone, URI. A plain
`Option<String>` cannot retain all three through deserialization. Resolve this
intent only for first binding; confirmed association remains explicit thereafter.

Validation has three jobs on the same document model:

1. **Source:** valid supported syntax/references and unique supplied keys; missing
   keys and new/retired rows are ordinary authorship.
2. **Prepared:** every object needing identity has one persisted key; complete
   desired content and binding/default intent are ready for compilation.
3. **Settled:** every surviving authored object has its verified association;
   intended detachments are confirmed and the relevant receipt can advance.

The parser can generate proposed missing keys in an in-memory prepared document
plus parser-derived insertions. Only executed preparation persists them. Preview
IDs are provisional; check/status must not hash a new random allocation each run
and present it as durable identity. Unprepared input gets a preparation-needed
result. The byte edit/reparse proof uses the same proposed keys in both parses.

For receipt canonicalization, encode retained objects by confirmed native
association, new keyed objects by an explicit `new` identity, and locally retired
associations as explicit pending omissions. Source spelling remains irrelevant
to retained associations. Include fixtures for key rename, new empty blocks,
deleted last block and page links to new pages. Do not reinterpret old hashes
under this rule. Reimporting rebuildable data can establish the new profile.

### Runtime contracts, not another document tree

These are illustrative Rust API boundaries; names/types can be adjusted together:

```rust
// document/mdx: pure, retaining source bytes/spans at its boundary
prepare_keys(source, key_source) -> PreparedSource
patch_tracking(source, expected_id, tracking) -> String
adopt_remote(source, desired_document) -> String

// leaflet/editor: pure except allocating explicitly supplied IDs
observe(native_view, binding) -> ObservedEditor
compile(desired, confirmed, observed, allocation, policy) -> NativeSave
verify(desired, expected_associations, before, after) -> Verification

// workflow: owns acquisition, local files and execution
pull_one(context, path, policy) -> PullOutcome
push_one(context, path, policy) -> SaveOutcome
watch(context, selection, options, cancellation) -> WatchReport
```

`ObservedEditor` retains the current raw fact map and typed projection together.
`NativeSave` contains ordered native arguments, allocated target IDs and small
effect counters; exact encoded requests exist only in memory during execution.
`Verification` checks desired meaning, retained/new associations, order and
untouched scope. None is an alternate mutable document representation.

Keep helpers ordinary functions and explicit structs. One transport seam for
loopback/fake responses is sufficient; there is no need for a backend trait per
command or a generic graph-reconciliation framework.

The live transport stays in `web::live`: connection/join/heartbeat/reconnect and
typed incoming events. `workflow::editor::watch` owns subscriptions selected from
the local inventory, coalescing and scheduling, adoption and reports. Neither
the MDX parser nor the pure native compiler owns a socket. A separate realtime
crate is unnecessary for this one backend.

## 4. A bounded fallback ladder

**Choose a coarser strategy during compilation when its preconditions fit. A
failed or uncertain remote execution is not permission to increase its scope.**

| Problem | First policy / bounded fallback | Preserved and traded |
| --- | --- | --- |
| Supported content already equal | Emit no operation | All native state retained |
| Compatible changed text | One Unicode-safe common-prefix/suffix splice and supported-mark pass in loaded Yrs | Paragraph/text objects and unchanged characters retained where straightforward |
| Breaks or awkward supported inline segmentation | Rebuild the affected paragraph interior inside the loaded Yrs document | Native block, text fact, paragraph and unrelated roots retained; inline/character anchors may change |
| Page order differs | Move all desired uniquely placed children to end in desired order | Page, block and membership IDs retained; extra position writes accepted |
| Remote body must be adopted locally | Checked rendering of the whole local body, preserve local-only frontmatter and reconciled keys | Supported meaning/identities retained; body formatting normalized |
| Incremental observation has no usable base | Fresh full snapshot | More transfer/decoding; no content mutation |
| Known supported block deliberately needs fresh identity | Explicit `push --recreate KEY`, reusing new-block construction and selected-placement replacement | Source key/occurrence can remain; old entity/text anchors are not transferred |
| Local checkout is obsolete | Deliberate fresh import into a chosen clean destination | No compatibility engine needed for disposable local files |
| Timeout, partial effects or unexpected read-back | Observe/settle or stop with pending evidence | No automatic broader rewrite or repeated attempt to force equality |

The coarse text branch is a normal supported path, not a catch-all error handler.
Malformed/incomplete CRDT, unknown active semantics and ambiguous memberships are
diagnostics. They do not silently become fresh content. Avoid adopting experiment
05's bounded LCS machinery as a production requirement: reuse its interop fixtures
and Yrs mechanics with the much smaller policy above.

The existing `mdx::update` already bulk-renders the body when pages change.
Continuous pull can use that behavior. Whole-body **local serialization** does not
replace the corresponding **remote graph**. Byte patching every changed local
paragraph is optional improvement, not a prerequisite. Unknown source constructs
still fail parsing; recognized authored trivia must either be preserved or have
an explicit normalization contract rather than disappearing accidentally.

### Explicit recreation without a second engine

For a selected, understood block occurrence B: allocate E′/T′, build it with the
ordinary new-block helpers, then reassert B's identified membership M with E′ at
the observed parent/position. Verify the new target and untouched occurrences;
only then promote B's new association. The old entity remains detached or remains
reachable elsewhere. No recursive deletion is involved. This exact-membership
assertion reuses an observed position, not a newly ported fractional-index system.

Start this escape with **representable supported blocks and resolved attempts**.
Do not advertise it as repair for an arbitrary unreadable document or unknown
interactive object. If another occurrence shares the old entity, recreation forks
only the selected occurrence and the preview says so. A failed request retains
the planned new IDs; it cannot allocate another replacement on each retry.

Recommend including selected-block recreation with the structural compiler once
its common helpers work. Whole-page/body recreation remains a separate explicit
extension: it requires reference/anchor and root-switch handling and is not needed
for normal drafting. Its accepted interface can remain unimplemented honestly.

## 5. Continuous inbound sync is an early deliverable

### User behavior

Proposed spelling: **`leafbud watch --all`**, or selected file operands. It opens
and maintains a live connection, subscribes to the selected documents, catches
up their current state and applies subsequent remote changes to local files.
The session reports connecting/live/reconnecting/degraded state separately from
whether an individual file is clean, dirty or conflicted. One-shot `pull` remains
useful, but repeatedly calling it on a timer does not complete this capability.

The first delivery follows durable editor changes through live notifications.
Explicit push remains available while watching. Automatically pushing local file
changes is a further direction policy over the same session, not implied by the
inbound request. The lower-latency Yjs stream is a concrete extension described
below; durable changes and ephemeral keystrokes must have different receipt rules.

Both directions use the same classification,
identity reconciliation and per-document operation lock. Live watch is useful
before the full push implementation lands; do not leave it until publishing.

| Observed situation | Follow action |
| --- | --- |
| Clean | Nothing; preserve bytes and mtime |
| Remote-only supported edit | Adopt through existing guarded pull/rendering |
| Equal current endpoints or supported witness-only change | Settle verified receipt without needless body rewrite |
| Local-only edit | Report `local-dirty`, preserve, keep observing; this is normal authoring |
| Divergent edits | Report `needs-decision`, preserve, continue other documents |
| Pending push/mint | Defer adoption to recovery; do not erase attempt evidence |
| Local parser error while editing | Preserve and retry after a later valid file state |
| Missing/inaccessible remote or new unsupported content | Report blocked/unavailable; retain local file |
| Temporary transport failure | Reconnect with backoff, mark stale; catch up after rejoin |

Reject persistent `--take remote` with watch in the initial interface. A one-shot
resolution is a deliberate decision about current differences; continuously
discarding every future local edit would be a different product mode. Polling
backstops and subscriptions continue after conflicts, but receipt baselines never
advance merely because a new remote snapshot was fetched.

Workspace selection is rescanned periodically; filename changes follow the
same document/binding identity. Duplicate bindings stay loud. `--all` follows
existing local bindings. New remote document discovery remains `import`; a future
opt-in discover-and-import mode can compose it without conflating the two jobs.
Missing local files are not silently recreated and remote omissions do not prune.

Report transitions rather than printing every notification or clean catch-up.
Machine watch output
should be versioned JSON Lines with document/path, outcome and observation time;
one-shot pull retains its existing JSON report. Cancellation finishes or abandons
the current read safely, emits a final summary and releases the lock. No daemon,
service manager or persistent event queue is needed for the first version.

### What upstream provides, and what to adopt when

Pinned Leaflet implements more than our current reader uses:

- `index.tsx` subscribes to Supabase `rootEntity:<root>` broadcast `poke`, calls
  pull on notification and reconnect, and uses a five-minute poll backstop.
- `yjsRealtime.tsx` has a separate `yjs:<root>` topic with `doc-update`,
  `awareness`, `hello` and presence events. It carries faster text/cursor state,
  while Replicache remains the durable source.
- `serverPullData.ts` + `cvr.ts` implement a server-side client-view record (CVR).
  The returned opaque cookie selects a base; changed facts/extras arrive as
  `put`/`del`, and an unchanged view can produce an empty patch.
- Null/old/unavailable cookies yield `clear` plus a full view. The server's fact
  and acknowledgment query uses one SQL snapshot. Separately fetched metadata
  does not acquire that transactional guarantee.

The browser comment saying every pull is full is stale relative to those
implementation files. This is source evidence, not a verified deployment contract.

### First implementation: a connected session

**Probe completed after commissioning:** [experiment 15](../experiments/15-live-connection/live-connection0.gpt61s.md)
verified public bootstrap discovery, acknowledged root/Yjs topic joins, passive
presence snapshots without tracking, heartbeat and reconnect/rejoin. It did not
observe nonempty deployed presence or a durable poke/file-adoption loop. Parent
review accepts its recommended `phoenix-channel-client` and native runtime
0.1.1 as the integration starting point. Keep its local/`!Send` driver isolated,
use the specified heartbeat interval and explicit unexpected-close recovery, and
follow the report's latest-state/monotonic-sequence handoff. The scope below is
the production work now equipped by that evidence, not a claim it has landed.

1. Resolve the deployment's realtime URL/public project key and actual channel
   access requirements. The browser uses `NEXT_PUBLIC_SUPABASE_API_URL` and
   `NEXT_PUBLIC_SUPABASE_ANON_KEY`; Leafbud's existing cookie HTTP client does not
   yet implement this bootstrap. Verify a bounded discovery/configuration adapter,
   rather than hardcoding an assumed production project or claiming the cookie
   alone is sufficient.
2. Maintain an origin-scoped connection, multiplexing document topics where the
   chosen client supports it. Subscribe to `rootEntity:<root>` and wait for join
   confirmation before initial catch-up. Events received while that read runs
   mark the document dirty for another pass, closing the subscribe/read race.
3. Coalesce pokes per document into one pending refresh and allow only one read
   cycle per document at a time. An event arriving during that cycle queues one
   more pass; it is not discarded when the earlier request completes. Begin with
   bounded serial work across documents rather than a sophisticated scheduler.
4. Acquire the current facts and metadata, classify and adopt with the existing
   per-file rules. Full snapshots are an acceptable first content acquisition
   mechanism: the connection and event lifecycle are still continuously live.
5. Rejoin and catch up after every connection gap. Use a slow, configurable
   reconciliation sweep for missed pokes and metadata changes that may not emit
   them. An unavailable subscription reports degraded state; polling fallback
   must not be labeled a healthy live connection.

A small session state holds connection status, per-document subscription state,
dirty/in-flight flags, current observation and last result. No persistent event
queue is needed. Reuse a maintained Supabase/Phoenix client if it proves the join,
heartbeat, broadcast, presence and reconnect cases; otherwise isolate a narrow
adapter over an established WebSocket library. Dependency selection is a concrete
implementation check, not evidence that a suitable Rust client is already wired.

Fetch metadata too: standalone title and lifecycle must not be skipped because
facts appear unchanged. Slow sweep intervals can differ from event-driven reads.
Measure actual requests/subscriptions before adding adaptive rate machinery.

**Next transport improvement:** add `RemoteView { rows, cookie, acknowledgments }`
in memory, one per origin/token/group. Apply each response to a candidate map,
validate it, then advance map and cookie together. Empty delta means unchanged
facts, not an empty document; acknowledgment deltas merge into their map. `clear`
resets the fact view. Unsupported but valid raw facts may be cached while local
adoption remains blocked. Invalid protocol state discards the candidate and
requests a full read; repeated invalid full reads fail rather than loop forever.
Process restart starts with no cookie and no replica. A cookie is never saved
without its base map, and neither is an agreement receipt or an MDX sidecar.

### Other attached clients: a promising, bounded experiment

The operator's speculative presence idea has a source-backed foothold:

- Leaflet joins `yjs:<root>` only after an editor registers a text document.
  It enables presence explicitly and tracks a random connection key with an
  `awarenessClientID`. Presence sync/join/leave exposes peers on that topic.
- It suppresses text/awareness broadcasts when no peer is observed. Recent remote
  broadcasts and `hello` provide a fallback for older clients or broken presence.
- These are editor connections, not authenticated unique people or a complete
  account-wide inventory of open tabs. A page without a mounted text editor may
  have no Yjs presence. Disconnect or missing presence means unknown, not zero.

Expose **`observed-peers(n)` / `no-peers-observed` / `unknown`**, with freshness,
as advisory session information. Presence must not grant write authority, clear
a conflict, disable change subscriptions or prove it is safe to overwrite.

The receive-only probe has now established initial passive presence snapshots
without tracking a Leafbud peer or publishing cursor awareness. Experiment 15
also checks reference-aware synthetic diffs; nonempty deployed diffs remain
unverified. Tracking our watcher would make the existing browser count it as a
peer and could enable broadcasts it otherwise suppresses; that is a real change
in traffic, not passive observation. Do not fabricate an awareness client or send
`hello` just to obtain a connected-user count.

When adopting advisory presence, start with a small scheduling rule: retain lightweight
`rootEntity` subscriptions for all selected documents; prioritize catch-up and
optionally keep richer in-memory state for documents with peers or recent
activity. Quiet documents still receive pokes and periodic catch-up. Presence
inspection itself requires per-document subscriptions; it is not a free global
"which documents are active?" directory. Avoid making an adaptive controller a
dependency of the first watch implementation.

### Faster live text is a distinct extension seam

`doc-update` contains entity ID, base64 Yjs delta and schema version. We could
load the block's durable baseline into Yrs and maintain an in-memory live view
from those messages. `useCollabText.tsx` shows a separate debounced write into
Replicache, so a received delta is not proof of persisted content. It can arrive
before the outer graph references its entity or before durable acknowledgment.

Recommend event-driven durable-file adoption first, and assess a live text overlay
next if that latency matters. Such an overlay needs bounded buffering, schema and
causal checks, gap recovery and an explicit policy for whether ephemeral content
ever reaches the file. It must not advance the durable agreement receipt or echo
received deltas back as locally authored edits. Full reobservation remains the
coarse recovery path. This preserves a route to tighter live sync without making
cursor collaboration or an unsaved-text authority model prerequisites.

## 6. Save and follow share one correctness boundary

The save cycle is concrete and finite:

1. Acquire the document's cooperative lock; read bytes and prepare missing keys.
2. Persist preparation under the byte guard, then use those bytes as input.
3. Fetch metadata + complete current facts; check binding, rights, agreement and
   absence/resolution of an earlier attempt. Validate the whole desired document.
4. Allocate new target IDs once and compile native mutations. Choose coarse
   branches now. Recheck current witnesses before effects; drift returns a result.
5. Persist compact pending evidence before mint/push. First mint is its own phase:
   persist the returned locator promptly, verify the seed/context, then compile
   against that real graph. A failed later step stays bound/pending.
6. Execute ordered requests; reobserve after completion or transport uncertainty.
7. Verify actual meaning, associations/order and protected scope. Patch only
   tracking state on successful push, guarded against intervening source changes.

Pending needs an attempt ID, phase, prepared-input/desired fingerprints, origin
and known locator, writer group/client and allocated mutation range, plus planned
new object/placement IDs. Keep expected target associations sufficient to match
read-back with author keys. A small typed attempt struct belongs in model; native
request bytes and binary text do not belong in frontmatter.

Reuse a writer identity and monotonically allocated mutation sequence for its
binding; keep it distinct from ephemeral read-only follower state and Yrs client
IDs. Acknowledgment is evidence of processing, not desired application. Resolve
terminal acknowledged/skipped operations by inspecting current facts, including
known detached IDs where needed. An unacknowledged request after restart can
remain uncertain; no exact-request journal means automatic exact replay is not
promised. Successful desired content alone does not settle an outstanding request
that might still land later. Explicit-ID access is a transport fixture/probe gate,
not an endpoint to invent from rooted-pull absence.

The browser limits each push slice to 250 mutations. Use bounded request sizing
and dependency order; do not assume a document fits one unlimited request. Persist
allocated ranges before sending; retain exact bytes for in-process retries. Whole
desired-document preflight still happens before the first content mutation.
Requests may be partially successful and intermediate state may be visible.

**Local coordination:** use an OS-released advisory lock keyed by stable local
document identity, with duplicate binding checks before work. All Leafbud writers
participate, including pull, push, tidy and metadata edits; the watcher releases
between document cycles. This prevents cooperating processes from racing and
does not become persistent content storage. Start with serial watch cycles.

The existing file writer's byte recheck followed by atomic rename has a narrow
check-to-rename window against an unrelated editor. An atomic rename is not a
conditional filesystem write. Keep the recheck adjacent to replacement, test
edits across awaits, and describe the guarantee honestly; advisory locks cannot
make arbitrary editors participate. Likewise the server lock is not a remote CAS.
These limits argue for conservative conflict behavior, not a larger merge engine.

## 7. Build path and useful checkpoints

The useful authoring delivery remains first binding plus the full ordinary
revision loop. The inbound watcher is a separately useful early capability.
Essential pending/drift/read-back protections belong in each executable write
slice, even while their broader tracker items remain open.

### A. Make authorship valid — two existing tickets, g

`leafbud-author-create` + `leafbud-author-keys`: implement the next source/state
format, implicit main page, missing/new/retired keys, flat list representation,
source-preserving preparation and tracking-only settlement. Update import,
status/diff/pull and inventory together for the new model. Existing code contains
no list BlockKind, so this is a real mapping addition, not just a CLI switch.

Acceptance: a complete unbound draft checks offline; additions, deletions and
split/join source actions parse; preparation repeats without changes. Fixture
imports use the same format. No migration subsystem is required for `~/l`;
a deliberate fresh checkout can replace its obsolete format when wanted.

### B. Establish connected watch — proposed ticket, Sol high then g

**`leafbud-editor-follow`**: extract one-file adoption,
add cooperative local locking and `watch`, with actual realtime subscriptions,
initial catch-up, event coalescing, reconnect and a slow reconciliation backstop.
Sol high is recommended for the connection/bootstrap and optional presence probe;
g can carry the well-defined loop and fixture integration once that seam is
verified. This can land after A while write transport is being prepared, because
it depends on observation/adoption. It must not wait for native push or publication.

Acceptance: browser-like fixture edits appear locally; local dirty/conflicted
files survive; equal refreshes preserve mtime; events during catch-up are retained;
rename/duplicate/source-race/reconnect cases behave as above. A connection probe
must establish deployment bootstrap/channel access before claiming live watch.
Incremental pulls and presence-based prioritization can follow; the persistent
subscription is in this ticket's initial delivery, not postponed behind polling.

### C. Establish reliable native effects — two existing tickets, Sol high

Work essential portions of `leafbud-editor-bind` + `leafbud-editor-recovery`:
protocol types/crate, both mint contexts, current action transport, writer IDs,
bounded requests, acknowledgment/explicit-ID reads and compact attempts. Sol's
extra judgment helps at this external protocol and ambiguous-outcome boundary.
Make first-binding protocol fixtures executable before piling on compiler work.

First binding cannot close before the full compiler materializes the desired
document. Current Beads dependencies describe **completion** dependencies:
`bind` depends on structure/recovery. They do not prohibit developing its transport
seam now. A binding-only checkpoint is internal work, not delivered authoring.

### D. Complete ordinary saves — two existing tickets, g

`leafbud-editor-text` + `leafbud-editor-structure`: implement block/scalar builders,
loaded Yrs edit with coarse interior fallback, keyed page/block create/update/
detach, simple order normalization and selected supported-block recreation.
Use the same helpers for first materialization and later saves. Join them with C's
transport and essential conflict/settlement checks as one working save operation.

Acceptance is the flow document's `[A,B,C] → [H,B,D,A]` plus marks/breaks, lists,
named page references, code/image inactive-text preservation and metadata in
both first-binding contexts. It must include second push, pull and repeat no-op.

### E. Join both directions and settle the delivery — Sol high

Finish `leafbud-editor-conflicts`, `leafbud-editor-recovery`,
`leafbud-editor-bind` and the proposed follow ticket's push-interaction proof.
Exercise response loss, skipped mutation, source changes, detached creation,
recreation interruption and pending attempt while follow is running. Use the
existing code/fixture evidence to select a disposable deployed first/second-save
proof when remote-write execution is commissioned.

Only then describe complete native authoring as delivered. Transport optimizations
can follow measured need; nested/task lists, canvas, uploads, public release and
whole-body recreation remain separately scoped growth. Ordinary paragraph/heading
insertion, split/join/delete/order and continuous inbound refresh are in this path.

## 8. Fixtures that hold the design together

| Fixture | Independent assertion |
| --- | --- |
| New local → bind → pull | Local UUID and authored keys survive token-derived import observation |
| Combined flow | Desired H/B/D/A order; A/B native IDs retained; H/D confirmed; C placement absent |
| Text splice and coarse branch | Real Yjs consumes both; Unicode/marks/breaks correct; unrelated roots retained |
| No-op | No mutations, no rekeying, unchanged local bytes/mtime |
| Recreate one shared occurrence | Selected placement targets new entity; other occurrence still targets old entity |
| Partial/acknowledged failure | No false success receipt; planned IDs retained; rooted absence not treated as noncreation |
| Watch dirty/converged/unsupported | Correct disposition without refreshing the agreement baseline on conflict |
| Subscribe/catch-up/reconnect | Event during an in-flight read causes another pass; reconnect repairs missed events |
| Presence experiment | Observed peer is distinguishable from unavailable presence; zero observation does not suppress pokes |
| Full/delta/reset (later optimization) | Same complete current projection; empty patch not interpreted as empty content |
| Writer + follower overlap | Cooperative serialization; pending save is not adopted away |
| Local editor saves during acquisition | New source preserved; successful remote effect can remain locally unsettled |

Run scoped nextest, fmt and clippy for code slices; retain cross-language Yrs/Yjs
checks for changes to encoding/editing. Offline protocol fixtures establish local
contracts; they do not certify current deployment action identifiers or endpoint
behavior. Document the deployed proof separately when performed.

## Connections and decisions still needing selection

- [Keyed edit flows](keyed-edit-flows0.gpt6a.md): behavioral cases this build must
  enact; this document adds code seams, inbound operation and sequencing.
- [Keyed authoring](keyed-authoring0.gpt6a.md): retains its complete-drafting and
  object-continuity direction; adds the explicit early follow capability.
- [Accepted interface](authoring-interface0.gpt6a.md): retains command meanings
  and deliberate recreation; proposes persistent `watch` and a ninth wire-only crate.
- [Frontmatter](frontmatter0.gpt61s.md): preserves ownership and compact evidence,
  proposes a clean next format and more explicit occurrence locators.
- [Earlier sync design](document-sync1.gpt61s.md#4-shared-classification-and-current-endpoint-resolution):
  current-endpoint conflict decisions remain applicable; a transport replica in
  memory is not a historical merge base or new on-disk source of truth.

**Commissioned:** A, `leafbud-author-create` + `leafbud-author-keys`, with g in
`opencode://ses_e41f88509ffejyMz0pF3zg73r3`. Parent review then hands the revised
model/adoption interfaces to Sol high for connected watch. Later native-write
batches retain their described sequence and receive concrete commissions at their
integration checkpoints; this acceptance does not claim them implemented.

The independent connection/presence probe ran alongside A in Sol session
`opencode://ses_e41f53633ffeI4wwnXfMRs1LeT` and returned at `44447c73`.
Parent reviewed its source, lifecycle tests and handoff, accepting the transport
recommendation while retaining the explicit nonempty-presence and deployed
poke-to-file proof limits. Watch integration follows the model/adoption join.

Batch A landed through `6aee3dd9`; [parent review](authoring-review0.gpt6a.md)
accepts its core choices and commissions focused corrections to preparation,
identity projection, tracking settlement and status before adoption integration.
A fresh g correction pass and an independent Sol production transport checkpoint
run in parallel, with separate owned files and an explicit parent join.

The [production transport](live-transport0.gpt61s.md) has now returned through
`224f08f8` at coordinated version 0.12.0. Parent source review accepts its
`web::live` boundary and the reported scoped/passive verification. Connected
watch remains open until the corrected foundation joins shared refresh/locking,
selection, scheduler and CLI integration; the transport-only return is an
internal checkpoint, not the user's completed live-sync workflow.
