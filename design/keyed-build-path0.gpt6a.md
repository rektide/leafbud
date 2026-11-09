---
type: Design
title: "Building keyed authoring and continuous inbound sync"
description: "Code-grounded domain boundaries, concrete interfaces, bounded coarse operations, source-format changes and a dependency-ordered build path for complete drafting plus continuous pull."
resource: /design/keyed-build-path0.gpt6a.md
status: proposed
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Operator requested implementation grounding, domain/crate organization, right-sized bulk fallbacks and live inbound sync; confirmed local data is rebuildable and formats are revisable"
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
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/index.tsx
    title: "Source-inspected rootEntity poke subscription, reconnect pull, poll backstop and browser push batching"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/serverPullData.ts
    title: "Source-inspected CVR incremental pull and null-cookie/cache-miss full-snapshot behavior"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cvr.ts
    title: "Source-inspected clear/put/del patches, unchanged cookies and mutation acknowledgment changes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "Re-read membership-preserving moveBlock and addLastBlock"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/push.ts
    title: "Re-read request identity, skipped-but-acknowledged mutations and post-commit poke"
---

# Building keyed authoring and continuous inbound sync

## Recommendation

Build **one document synchronization workflow with two directions**: explicit
keyed saves to Leaflet, and continuous guarded adoption from Leaflet. Keep the
existing source identities and native objects; bound complexity with whole-block
text interiors, whole-page ordering passes and whole-body local rendering where
those are adequate. The [flows](keyed-edit-flows0.gpt6a.md) remain the behavioral
examples; this document supplies the code boundaries and build sequence.

The operator explicitly allows format revision and rebuilding `~/l`. Recommend a
clean next source format rather than a migration framework for that disposable
corpus. Rebuildability removes a compatibility burden; ordinary future saves still
need stable identities and protection for newly authored work.

This is a proposed implementation design, grounded in inspected code. No new
runtime/deployed-write evidence is claimed. The local Leaflet archive was clean
at `600c7b5f…` during inspection. The existing interface's eight-crate count and
exact source schema are revisable; the ninth crate below is a recommendation.

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
| Pull's `try_refresh` is a private command-shaped operation | Extract a reusable per-document cycle | One-shot pull and continuous pull can share decisions and writes |
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
leaflet/web/src/     pull.rs push.rs mint.rs actions.rs
workflow/src/
  workspace/        config.rs inventory.rs files.rs lock.rs
  editor/           observe.rs agreement.rs adopt.rs save.rs recovery.rs follow.rs
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
follow(context, selection, options, cancellation) -> FollowReport
```

`ObservedEditor` retains the current raw fact map and typed projection together.
`NativeSave` contains ordered native arguments, allocated target IDs and small
effect counters; exact encoded requests exist only in memory during execution.
`Verification` checks desired meaning, retained/new associations, order and
untouched scope. None is an alternate mutable document representation.

Keep helpers ordinary functions and explicit structs. One transport seam for
loopback/fake responses is sufficient; there is no need for a backend trait per
command or a generic graph-reconciliation framework.

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

Proposed spelling: **`leafbud pull --watch --all`**, or selected file operands.
`--interval` configures polling; begin with a conservative interval such as 30s,
one in-flight document cycle, and per-origin backoff. It means saved editor state
arrives locally without rerunning a command. It is not character-level live
co-editing, and it does not automatically push local file changes.

Manual push can coexist with it. Both directions use the same classification,
identity reconciliation and per-document operation lock. Pull-follow is useful
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
| Temporary transport failure | Back off, mark stale; resume from fresh observation |

Reject persistent `--take remote` with watch in the initial interface. A one-shot
resolution is a deliberate decision about current differences; continuously
discarding every future local edit would be a different product mode. Polling
continues after conflicts, but receipt baselines never advance merely because a
new remote snapshot was fetched.

Workspace selection is rescanned between rounds; filename changes follow the
same document/binding identity. Duplicate bindings stay loud. `--all` follows
existing local bindings. New remote document discovery remains `import`; a future
opt-in discover-and-import mode can compose it without conflating the two jobs.
Missing local files are not silently recreated and remote omissions do not prune.

Report transitions rather than printing every clean poll. Machine watch output
should be versioned JSON Lines with document/path, outcome and observation time;
one-shot pull retains its existing JSON report. Cancellation finishes or abandons
the current read safely, emits a final summary and releases the lock. No daemon,
service manager or persistent event queue is needed for the first version.

### What upstream provides, and what to adopt when

Pinned Leaflet implements more than our current reader uses:

- `index.tsx` subscribes to Supabase `rootEntity:<root>` broadcast `poke`, calls
  pull on notification and reconnect, and uses a five-minute poll backstop.
- `serverPullData.ts` + `cvr.ts` implement a server-side client-view record (CVR).
  The returned opaque cookie selects a base; changed facts/extras arrive as
  `put`/`del`, and an unchanged view can produce an empty patch.
- Null/old/unavailable cookies yield `clear` plus a full view. The server's fact
  and acknowledgment query uses one SQL snapshot. Separately fetched metadata
  does not acquire that transactional guarantee.

The browser comment saying every pull is full is stale relative to those
implementation files. This is source evidence, not a verified deployment contract.

**First implementation:** run the existing full-snapshot acquisition on a timer.
Extract the per-file cycle instead of repeatedly invoking the CLI or building a
second synchronization model. Fetch metadata too: standalone title and lifecycle
must not be skipped because the fact payload appears unchanged. Bound network
work and measure request volume over the actual selected corpus.

**Next transport improvement:** add `RemoteView { rows, cookie, acknowledgments }`
in memory, one per origin/token/group. Apply each response to a candidate map,
validate it, then advance map and cookie together. Empty delta means unchanged
facts, not an empty document; acknowledgment deltas merge into their map. `clear`
resets the fact view. Unsupported but valid raw facts may be cached while local
adoption remains blocked. Invalid protocol state discards the candidate and
requests a full read; repeated invalid full reads fail rather than loop forever.
Process restart starts with no cookie and no replica. A cookie is never saved
without its base map, and neither is an agreement receipt or an MDX sidecar.

**Then, if latency warrants it:** verified Supabase notification subscription
wakes the same cycle; it carries no authoritative content. Reconnect pulls and
polling remain. Verify channel/auth discovery and metadata-only notification
coverage before relying on it. No need to implement browser Yjs broadcasts or
local CRDT editing sessions to keep saved remote documents refreshed.

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

### B. Keep the local checkout current — proposed ticket, g

Proposed **`leafbud-editor-follow`** (not yet created): extract one-file adoption,
add cooperative local locking and `pull --watch`, first using full snapshots.
This can land after A while write transport is being prepared, because it only
depends on observation/adoption. It must not wait for native push or publication.

Acceptance: browser-like fixture edits appear locally; local dirty/conflicted
files survive; equal polls preserve mtime; rename/duplicate/source-race/reconnect
cases behave as above. Incremental transport and notifications are later
improvements to this same behavior, not prerequisites hidden in the ticket.

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
| Follow dirty/converged/unsupported | Correct disposition without refreshing the agreement baseline on conflict |
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
  and deliberate recreation; proposes `pull --watch` and a ninth wire-only crate.
- [Frontmatter](frontmatter0.gpt61s.md): preserves ownership and compact evidence,
  proposes a clean next format and more explicit occurrence locators.
- [Earlier sync design](document-sync1.gpt61s.md#4-shared-classification-and-current-endpoint-resolution):
  current-endpoint conflict decisions remain applicable; a transport replica in
  memory is not a historical merge base or new on-disk source of truth.

Recommended next commission is **A: `leafbud-author-create` +
`leafbud-author-keys`, g**, with the above parser/model seam as its scope. Agree
the format/locator and protocol-crate direction at that boundary. This document
does not turn its proposed ticket or the rest of the build path into an automatic
implementation dispatch.
