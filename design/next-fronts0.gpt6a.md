---
type: Report
title: "Leafbud: current ground, next fronts and compaction handoff"
description: "Continuation account of accepted 0.21 published adoption/release evidence, active native-publication preflight checkpoint, remaining fronts and verification boundaries."
resource: /design/next-fronts0.gpt6a.md
status: current-handoff
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
updated:
  at: 2026-11-10
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Parent architecture and compaction report, acceptance through 0.21, native-publication checkpoint commission and correction after agent context overshoot"
  - resource: /design/published-lifecycle0.gpt61s.md
    title: "0.21 published adoption/editor save and independent release evidence, with 357-test gate"
  - resource: /doc/release/README.md
    title: "Maintained public meaning, lifecycle and evidence limits"
  - resource: /crates/workflow/src/published.rs
    title: "Reviewed authoritative native/public linkage and exact URI discovery"
  - resource: /crates/workflow/src/release.rs
    title: "Reviewed public acquisition, scoped current/unknown evidence and no receipt settlement"
  - resource: /crates/leaflet/records/src/projection.rs
    title: "Reviewed bounded working/public mapping and strict unsupported fields"
  - resource: /crates/leaflet/editor/src/write/metadata.rs
    title: "Reviewed published working-metadata routing independent of public actions"
  - resource: /design/naming0.gpt61s.md
    title: "0.20 literal path mapping, no-clobber move/witness ordering and 342-test gate"
  - resource: /crates/workflow/src/names.rs
    title: "Reviewed independent naming classification, explicit directions and destination validation"
  - resource: /crates/workflow/src/tidy.rs
    title: "Reviewed full-selection late guards, dry-run batch reservations and recoverable move/settlement ordering"
  - resource: /crates/workflow/src/files.rs
    title: "Reviewed actual Linux NOREPLACE primitive and guarded source replacement"
  - resource: /crates/document/mdx/src/names.rs
    title: "Reviewed separate naming patch authority and full source/state proof"
  - resource: /design/source-fidelity0.gpt61s.md
    title: "0.19 source/comment/flow delivery and 328-test gate"
  - resource: /doc/mdx/README.md
    title: "Accepted bounded source ownership and comment-anchor policy"
  - resource: /doc/mdx/coverage.md
    title: "Maintained capability/operation evidence and explicit limits"
  - resource: /crates/document/mdx/src/source.rs
    title: "Reviewed parser-paired flow collections and selected-entry surgery"
  - resource: /crates/document/mdx/src/render.rs
    title: "Reviewed original-source-bound tidy and raw comment preservation proof"
  - resource: /design/local-authoring-tools0.gpt61s.md
    title: "Completed 0.18 metadata/rekey/list, 317-test gate and actual CLI/HTTP/Phoenix evidence"
  - resource: /crates/document/mdx/src/author.rs
    title: "Reviewed selected field replacement, parser comment ranges and full-output proof"
  - resource: /crates/document/mdx/src/rekey.rs
    title: "Reviewed exact identity/reference rename and expected-document proof"
  - resource: /crates/document/mdx/src/parse.rs
    title: "Reviewed production JSX selection and bounded attribute-token extents"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Reviewed unchanged-state fast path after existing settlement guards"
  - resource: /crates/workflow/src/author.rs
    title: "Reviewed shared lock, duplicate checks and original-byte write guard"
  - resource: /crates/workflow/src/list.rs
    title: "Reviewed global identity accounting, partial header evidence and read-race reporting"
  - resource: /AGENTS.md
    title: "Active operator constraints, fresh-Sol selection and 220–320k rotation"
  - resource: /design/terminal-recovery0.gpt61s.md
    title: "Latest completed 0.17 delivery and 300-test/deployed standalone evidence"
  - resource: /design/keyed-save0.gpt61s.md
    title: "Complete 0.16 compiler/save, independent Yjs and actual P1 revision proof"
  - resource: /design/keyed-save-review0.gpt6a.md
    title: "Parent compiler review and terminal-recovery/no-op follow-up commission"
  - resource: /design/native-default-adoption0.gpt61s.md
    title: "Default/description semantics, selected P1 and original-intent settlement evidence"
  - resource: /design/connected-watch0.gpt61s.md
    title: "Persistent connected watch, source/selection/locking guards and live proof limits"
  - resource: /design/local-authoring-tools0.gpt6a.md
    title: "Completed local-authoring batch design"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Broader command meanings, naming practices and effect contracts"
  - resource: /design/document-sync1.gpt61s.md
    title: "Separate release evidence and current-endpoint resolution principles"
  - resource: /crates/workflow/src/native/resolve.rs
    title: "Parent reviewed actual explicit terminal resolution and observed-association reuse"
  - resource: /crates/document/model/src/attempt.rs
    title: "Parent reviewed bounded lastResolution provenance and lifecycle validation"
---

# Current ground and next fronts

## Resume here after compaction

**Latest accepted baseline: 0.26.0, through `954cd9f1`** (plus the parent's
tag-length preflight below). Native publishing is live-proven end-to-end (0.25:
quiet publish + same-URI update + full deletion on a disposable draft), and
publication preferences are fully authorable (0.26: tri-state frontmatter,
`edit --preference/--clear-preferences`, meaning-based settlement). The
**author, editor and inspect epics are all closed**. Open: `leafbud-release`
(delivery child) and `leafbud-mdx` (content families) plus the parked P3
direct-PDS tickets.

**Cleanup policy DECIDED (operator, 2026-11-10): "let Leaflet be Leaflet" — option A.**
Publishing from Leafbud into a publication accepts Leaflet's own publish-time
behavior, including its per-post image housekeeping, exactly as the web UI's
publish button does. The preview discloses it plainly ("old image copies may
be deleted; count unknown"). Scoped acceptance of native behavior only — no
Leafbud-side deletion machinery. Two independent explainers converged:
[g](native-cleanup-question0.glm53m.md), [Sol](native-cleanup-question0.gpt61s.md).

**Delivery research returned (g `ses_e3917df05ffexDi7k5XmXAi7Lj`,
[doc](newsletter-delivery0.glm53m.md), `f786423f`) — implementation awaits an
explicit operator commission.** Headline findings: quiet publishing is an
**active protective write** (a `skipped` send-slot claim that outruns a
firehose-driven auto-sender), with a **crash-window hazard** — a quiet publish
dying between record write and slot claim leaves the slot free and subscribers
can be emailed despite quiet intent (native Leaflet behavior, live for today's
forced-quiet publishes; disclosed in the design with an observe-and-report
recovery rule, never re-invocation). `sendEmail` defaults **true** natively, so
the future wire must always send an explicit boolean. "Sent" means Postmark
batch acceptance only — per-subscriber receipts exist server-side but are
invisible to every RPC Leafbud can invoke. **No resend primitive exists**;
the design refuses to fake one. Proposed surface: `delivery.email` frontmatter,
missing = quiet default unchanged; delivery receipts separate from publication
receipts and never gate settlement. Test-email and post-hoc Bluesky share are
separate later slices.

**Handoff note from the operator (2026-11-10):** content expansion (front D)
is to be **handed to a new session** — that is the next session's starting
commission. Housekeeping facts verified: the native lexicon caps each tag at
50 **characters** and has **no tag-count limit** (the "up to 5" recollection
matches no pinned source); preflight now refuses over-length tags before any
effect (parent fix, tested in save_tests).

Rotate implementation agents at major task boundaries or **220–320k reported
context**, superseding the older 240–350k window. Completed checks need not
repeat merely because context was compacted.

## 1. What has actually landed

| Capability | Current ground | Receiving home |
| --- | --- | --- |
| Local draft source | `leafbud-mdx/2`, implicit main page, ordinary Markdown, explicit named pages, one-time durable keys, new/retired associations | `document/model`, `document/mdx`; [foundation corrections](authoring-corrections0.glm53m.md) |
| Read/compare/adopt | Current native reads, scoped status, normalized external-tool diff, guarded import/pull | `workflow`; [watch receipt](connected-watch0.gpt61s.md) links predecessors |
| Persistent inbound sync | Real Phoenix/Supabase subscriptions, catch-up/reconnect, coalescing, optional passive presence, slow backstop | `leaflet/web::live`, `workflow::watch`; [connected watch](connected-watch0.gpt61s.md) |
| Native effect foundation | Both mint transports, exact bounded requests, writer acknowledgments, explicit known-ID reads, compact pending attempts | `leaflet/protocol`, `leaflet/web`, `workflow::native`; [native effects](native-effects0.gpt61s.md) |
| Complete supported saves | Preview-first public push; first binding; text/marks/breaks; new/drop/move/reorder; flat lists, code/images, named pages, selected recreation | `editor::write`, `native::save`; [keyed saves](keyed-save0.gpt61s.md) |
| Deliberate terminal recovery | Original-success settlement separately from explicit accept-current/replan; bounded `lastResolution`; no uncertain replay/remint | `native::resolve`; [terminal recovery](terminal-recovery0.gpt61s.md) |
| Offline authoring tools | Selected metadata edits, identity-preserving key rename, truthful local inventory | `mdx::author/rekey`, `workflow::author/list`; [local tools](local-authoring-tools0.gpt61s.md) |
| Source fidelity and coverage | Whole-flow headers, broad preparation spellings, same-page block comments; explicit operation evidence | [Source contract](/doc/mdx/README.md), [coverage](/doc/mdx/coverage.md) |
| Filename/path reconciliation | Independent `names`, explicit/default tidy reconciliation, read-only classification, no-clobber move with unconfirmed-settlement reporting | [Naming receipt](naming0.gpt61s.md), `workflow::names/tidy`, `mdx::patch_names` |
| Published editor and public evidence | Verified published binding/import/refresh/save; explicit lifecycle adoption; separate public status/diff with unknown imported baselines | [Public contract](/doc/release/README.md), [receipt](published-lifecycle0.gpt61s.md) |
| Native publication foundation | Pinned `publishToPublication` contract, read-only readiness preview, content-CID verification, proven inline-size witness; execution not wired | [Action/preflight](native-publication0.gpt61s.md), [verification](publication-verification0.gpt61s.md) |
| Deliberate native deletion | Preview-first `delete` for draft and published; record-first native order; observation-verified; clean local unbinding | [Delete receipt](native-delete0.glm53m.md), `web::delete`, `native::delete` |
| Publish execution and recovery | Live-proven quiet first publish + same-URI update with content-verified settlement; finite recovery; explicit-null evidence guards | [Publish receipt](native-publish0.gpt61s.md), `native::publish`, `mdx::settle_release` |
| Publication preferences | Tri-state authorable visibility preferences; whole-object native replacement; meaning-based settlement; dispatch-only Discover prediction | [Preferences receipt](release-preferences0.gpt61s.md), [options design](publish-options0.glm53m.md) |

The supported native editor lifecycles are **draft and published**. Canvas,
nested/task lists, richer interactive objects, uploads and native public-release
execution remain separate work. Public comparison is a narrower explicit profile
than editor support. There is no whole-body regeneration default, automatic
cleanup, saved historical body or executable request journal.

The domain grouping is now nine crates: document **model/MDX**, Leaflet
**protocol/editor/web/records**, **ATProto**, **workflow**, and **CLI**.
Protocol types are consumed by web/editor; pure editor logic does not depend on
HTTP or workflow. Local file effects and orchestration stay in workflow.

### Verification to carry forward

- Latest regular gate: **378/378 workspace tests**, six opt-in deployment tests
  skipped; formatting, strict all-target clippy and document checks passed.
- Production Yrs → independent actual Yjs: **7/7 groups**. The terminal-recovery
  slice did not change the writer; its receipt correctly reuses that evidence.
- Both first-mint contexts pass complete CLI/HTTP fixtures through first save,
  revision, pull and repeat. This is not a deployed first-unbound composer claim.
- Actual existing **P1** passed complete bound CLI baseline → combined revision →
  read-back → clean status → unchanged pull/repeat, preserving IDs and no-op mtime.
- Actual existing **standalone test draft** passed explicit terminal accept-current,
  retaining original partial provenance, then clean status and unchanged pull/
  repeat. No new native content effect was needed for that resolution.
- HTTP/Phoenix fixtures prove follower pending preservation and later remote-save
  adoption. The passive deployed watch smoke proved connection/catch-up/no-op/
  presence/interrupt, not an observed deployed poke-to-file event.
- Actual CLI/HTTP confirmed rekey → preview/repeat push preserves empty effects,
  receipts, source bytes and mtime. HTTP/Phoenix tests cover renamed-key adoption,
  shared-lock contention and preservation of dirty local metadata. These 0.18
  checks used temporary fixtures; no live corpus or draft was touched.
- Version 0.19 CLI/HTTP and Phoenix fixtures cover whole-flow edit/default/save/
  refresh, comment preservation and refusal when a remote comment anchor disappears.
  Native/Yrs writer unchanged; prior seven-group Yjs evidence remains applicable.
- Version 0.20 tests add actual NOREPLACE collisions, post-rename interruption/
  external saves, independent dirty-editor naming acceptance, content-no-op CLI/
  HTTP push/pull and Phoenix rescan at the new filename with comments intact.
- Version 0.21 verifies both published editor contexts against HTTP fixtures,
  explicit lifecycle adoption, scoped public reads/diff without native credentials,
  independent unknown/changed release evidence and published Phoenix adoption.
  No deployed published proof or new independent Yjs execution is claimed.
- Version 0.22/0.23 add literal native publish-action transport fixtures, read-only
  readiness guards, five independent fixed JS content-CID vectors and six fixtures
  executing the actual pinned native serializer/offload helper (fragmented text
  161205 bytes correctly refuses inline; ordinary two-page post 6551 bytes passes;
  exact 102399/102400/102401 boundary verified). All loopback/synthetic; the
  unchanged ordinary Yrs writer keeps its received seven-group evidence.

Trust these reported checks. Add tests for new changes or a specific unresolved
review question; do not rerun everything to reconstruct confidence after compaction.

## 2. Decisions that must survive

- **Fresh Sol high for major features.** Continue through coherent batches and
  parent reviews. Rotate at major task boundaries or a coherent checkpoint
  **between 220–320k reported context**, completing rollover by 320k. Smaller
  finished slices can end earlier.
- **Low volume, bounded complexity.** Simple keyed create/update/place/detach;
  compatible text splice/marks with loaded-paragraph-interior fallback; changed
  page end-normalization. No general structural optimizer.
- **Current source and durable identity.** Normal saves retain native objects.
  Drops detach placements. Raw current Yrs stays in memory; no saved body/history.
- **Local data is rebuildable.** `~/l` remains untouched and old-format. A deliberate
  fresh import can rebuild it; do not build a compatibility framework just for it.
- **First-binding defaults are explicit.** Copy verified omitted theme/contributors
  and confirmed publication into source; preserve explicit choices. Null theme/
  contributors are not silently omission. Do not weaken verification.
- **Description clear equivalence:** exactly absent/null/empty string; no trimming
  of nonempty values. Preserve source spelling and conservative raw native witness.
- **Live means connected.** Subscriptions are primary, polling/full reads recover
  missed events. Presence is advisory observed connections, never proof of idle
  state or a reason to disable quiet-document subscriptions.
- **Resolution is a new decision.** Unacknowledged old work cannot be overridden
  by `--take` or independently re-encoded. A terminal partial can be accepted or
  replanned explicitly, retaining old ID/hashes/finality in bounded provenance.
- **Locks and scope:** use existing stable-ID OS locks, exact source guards and
  all explicit operands in late duplicate checks. Atomic rename is not filesystem
  CAS against unrelated editors. Native server locking is not caller CAS.
- **One secret; Leaflet brokers all writes.** Leafbud holds only the Leaflet
  website session cookie. Public-record reads hit the PDS unauthenticated; every
  PDS write happens inside Leaflet's server via the owner's stored OAuth link.
  No ATProto credentials are minted, and failures never fall back to raw PDS
  writes. Researched and confirmed: [auth architecture](auth-architecture0.glm53m.md).

## 3. Accepted return — local authoring ergonomics

**Standing: implemented and parent-accepted.** All three tickets are closed:
`leafbud-author-metadata`, `leafbud-author-rekey`, `leafbud-inspect-list`.
Commits: `b5c2aa5e` implementation, `10ae22b5` receipt/usage, `9dfd68d9` expanded
source/watch verification. [Design](local-authoring-tools0.gpt6a.md) and
[implementation receipt](local-authoring-tools0.gpt61s.md) remain distinct.

This batch makes the implemented system easier to operate: list local drafts,
edit selected title/description/tags/date fields, and rename block/page keys while
keeping native bindings. It reuses source-aware patches, inventory and locks.
Its important judgment is keeping author edits, alias edits and receipt settlement
distinct. Confirmed rekeying should leave the editor witness unchanged; an author
metadata edit must not hide itself by advancing receipts.

The pure APIs are `MetadataEdit`, `edit_metadata` and `rename_key`; workflow owns
locked `author::edit/rename` and read-only `list::inspect`. Repeatable `--tag`
replaces the ordered list; `--clear-tags` writes `[]`. Pending/future owned state
blocks helpers safely. Selected-field comments may move to standalone lines,
while unrelated source/body bytes remain intact. Whole-flow-root metadata edits
subsequently landed in 0.19; block-scalar main-page rename still refuses explicitly.

Review accepts the bounded source adapters: production parser nodes choose edits,
and full reparse equality proves their scope. The new equal-state return in
`patch_tracking` runs after existing identity/writer/binding/pending guards, so a
renamed node-map's textual order no longer causes an empty push to rewrite bytes.

## 4. Current and next fronts after 0.21

Fronts A/B and front C's adoption/state pair are accepted. Native publication is
now a staged implementation commission; live effects remain separately authorized.
Use **fresh
Sol high**, selecting coherent batches of 1–4 tickets.

### A. Source fidelity and executable coverage — accepted

**Tickets:** `leafbud-mdx-source` + `leafbud-mdx-coverage`.

**Delivered and parent-accepted:** Sol `ses_e3c420e5affeLoQfS0oTd9wu5b`, through
`aa13bed1`. Both tickets are closed. [Receipt](source-fidelity0.gpt61s.md) records
commits and 328/328 tests; [source contract](/doc/mdx/README.md) and
[coverage matrix](/doc/mdx/coverage.md) are maintained homes.

Parent expressly accepts the bounded comment policy: standalone source comments
attach to the following same-page block; retained block reorder carries them.
Remote deletion/cross-page movement refuses adoption, including `--take remote`,
preserving source and requiring explicit author placement/removal. Inline/orphan
comments and native discussion remain unsupported. This makes comment ownership
predictable without a saved body, sidecar or general source tree.

Flow collection extents come from paired parser events rather than incomplete
serde spans. Existing pure-operation reparse checks constrain each patch; only
the tool envelope may normalize during settlement. Markdown tidy now binds exact
original bytes and proves comment anchors/raw text as well as content/state.

Coverage distinguishes parser validity, native writability and actual workflow
evidence. Source-valid cyclic PageLinks still refuse before effects. Independent
native kind/resource expectations supplement the existing complete-save tests;
the matrix names shared versus dedicated cases and prior deployed/Yjs boundaries.
Parent trusts the reported checks and found no blocking architectural issue.

### B. Naming and path intent — accepted local authoring completion

**Ticket:** `leafbud-author-names`.

**Delivered and accepted:** fresh Sol `ses_e3c29df03ffeKTX7z5IHA3nup5`, through
`875c01b6`; [receipt](naming0.gpt61s.md). Parent accepts ticket closure and the
local authoring epic: create, keys, metadata, rekey and naming children are complete.

**Payoff:** reconcile intentional filename/path choices without confusing them
with native identity. This completes the remaining child of the author epic.

**Bounded design:** implement deliberate `tidy --from filename|path` and `--accept`
using independent naming witnesses. Normal push never renames files. A pathname
change keeps local UUID and native binding; collision and conflicting direction
require an explicit choice. Local path intent does not reserve or change a native
public route. Preserve the established global/workspace selection conventions.

Accepted mapping: logical `/notes/Éssay` ↔ workspace `notes/Éssay.mdx`, literal
case/Unicode, exactly one suffix, existing real directories only. No decoding,
slugification or traversal. External operands retain key/Markdown passes but do
not settle workspace names. An explicit accepted mismatch need not enact its path.

`State.names` is an independent pair with missing/null/value path presence, no
timestamp/history. Status/list share tidy's classification. Source patching keeps
editor receipts, native IDs, writer, historical resolution and body/comment bytes.

Parent accepts the failure ordering: actual Linux NOREPLACE rename first, guarded
source/witness replacement second. A second-phase error reports destination and
**unconfirmed** settlement, whose bytes may be original, patched or externally
edited. No rollback overwrites/deletes the good current file. Full-selection late
duplicate checks, source inode/bytes, operand routes and destination checks guard
cooperative writes; no filesystem CAS is claimed. Preview reserves colliding batch
destinations in memory; execution relies on the real no-replace primitive.

This can follow the new field/key patch helpers. Keep it separate from remote
slug support: current source evidence says ordinary native publication chooses its
first public key/path. Do not invent custom-rkey behavior by sending ignored fields.

### C. Published lifecycle and release — larger next product loop

**Accepted tickets:** `leafbud-release-adopt` and `leafbud-release-state`, through
`5f7a666f`, Sol `ses_e3c10b84effeXUnnnL5oZ5M7TJ`. Both are closed. Parent accepts
verified linkage, current-editor adoption, explicit `pull --adopt-published`,
editor-only saves and independent read-only release status/diff. Imported public
baselines stay unknown; no operation here creates a confirmed release receipt.
`leafbud-release-delivery` remains explicit newsletter/share behavior.

**Payoff:** continue editing the same native document after publication and
distinguish saved editor changes from released public output. This is a new
lifecycle/evidence boundary, not a rename of `push`.

**Design needed before effects:** represent the actual public binding/record and
separate released-input/output witnesses; lift draft-only observation restrictions
only where verified native linkage exists; preserve local/native identity across
release. Read-only status should show unreleased changes and out-of-band public
record drift without accepting them. Preview publishing should explain native
path/date and delivery choices. A lost release result needs observation-based
recovery, not a blind publish retry.

The delivered batch keeps current native working content distinct from fetched
public output. A URI/CID alone does not prove what editor input was released;
unknown imported release baselines remain unknown. Import with no legitimate
editor linkage reports editing-unavailable, never guesses or mints a replacement.
Current-endpoint release diff reuses the existing human Markdown/external-tool
contract where conversion is faithful. Ordinary editor agreement cannot silently
advance release receipts; naming/comment/source protections remain intact.

Published editor-only saves use working linkage-table metadata, including standalone
published documents. Parent checked upstream `updatePublicationDraft`: it updates
those working rows without calling public release. Release projection refuses
unresolved images/resources, contributor ordering, ambiguous page graphs, legacy
comparison and richer output fields rather than hiding them. CID observations are
PDS-reported and syntax-validated, **not rehashed**; status labels that evidence and
requires supported current meaning for equality. This is accepted for the current
read-only contract, not sufficient by itself to claim verified future publication.

**Foundation accepted; execution next — `leafbud-release-native`:** two bounded
checkpoints are parent-accepted. Sol `ses_e3bcb6540ffegZC39tN1E6wuTc` (0.22,
~223k) delivered the pinned action contract, forced no-email transport, read-only
`workflow::publication::preview` and the exact standalone default-theme correction.
Sol `ses_e3bb92bc4ffeN77oF8X2ayoH9a` (0.23, ~241k) delivered record content-CID
verification (`atproto::content`, `fetch_content_verified`) and the proven native
inline-size witness through the actual pinned serializer. Root `publish`/`--yes`
still refuses; no workflow invokes the effect adapter.

The remaining slice is one fresh Sol high: locked save-first execution, durable
dispatch-before-effect uncertainty, fresh saved-state size witness, content-verified
exact-URI output plus native authoritative association, complete meaning comparison
with declared path/date normalization, a distinct loss-checked release-patch
authority and finite recovery. Never blind-retry uncertain dispatch. The receiving
briefs in both receipts equip it.

**DECIDED — gated-image cleanup (operator 2026-11-10, "let Leaflet be Leaflet"):**
publishing into a publication accepts the native document-scoped stale-copy
cleanup (`gatedImages.removeStale`, no suppression flag; standalone never
constructs it) as disclosed behavior, identical to the web UI's publish. The
execution slice removes the publication-scoped blocker and replaces it with a
plain disclosure: "old image copies may be deleted; count unknown." No
Leafbud-side deletion, storage access or cleanup retry joins it. Two
independent explainers ([g](native-cleanup-question0.glm53m.md),
[Sol](native-cleanup-question0.gpt61s.md)) verified the bounds and converged
on this lean: effect limited to the document's own publish-owned folder,
fails safe on listing errors, capped at 1000 objects per sweep, unreachable
for Leafbud-authored content, and a clean public record cannot prove the
folder empty (D illusory). Scoped acceptance of native behavior only — this
decides preview disclosure, not live authorization.

Other automatic native effects are enumerated and preflight-guarded: byline upsert
on first publication, schedule deletion, newsletter skipped-slot claim on first
newsletter-enabled publish, path revalidation/analytics and conditional version
snapshots. Nondefault inherited preferences, reply drafts and schedules refuse.

All verification uses fixtures/loopback; no live corpus/test-draft
mutation or real publication, upload, email/share or cleanup is commissioned.

Reuse existing native web actions and public record/protocol crates where their
actual behavior fits; inspect current upstream capabilities before choosing the
adapter. No parallel PDS publication lifecycle, copied source body, or automatic
email delivery. **A deployed publish/email/share proof needs a new explicit
operator instruction:** current live authorization covers unpublished test drafts.

### D. Content/profile expansion — designated next-session commission

**Tickets:** `leafbud-mdx-structure`, `leafbud-mdx-rich`, `leafbud-mdx-media`.

**Operator designation (2026-11-10):** this front is handed to a new parent
session as its starting commission. Ask the operator which family to build
first — do not pick for them — with this framing: **media** (image/cover
resources — also the prerequisite for lifting the cover-upload refusal on
publish), **rich blocks** (one concrete richer kind, e.g. embeds/polls/buttons,
chosen by user need), or **nested structures** (nesting/task lists). Each is
carried model/parse/render/observe/write/read-back together on the keyed
foundation, with shared/interactive identity consequences declared and explicit
refusals retained for the rest.

**Payoff:** adopt and edit more real Leaflet content. Pick a concrete fixture
family from actual user need and carry it end-to-end. Uploading bytes is a
different capability from preserving a remote image URL; the current profile
supports URL images only, and `members-only/gated` content remains web-UI-only.

Do not commission all three as an abstract "support everything" batch. Choose
one family with its native source evidence (verify the pinned archive revision
first) and a complete round-trip case. `leafbud-mdx-source`/`-coverage`
(the epic's completed children) established the pattern: bounded source forms
plus the maintained [coverage matrix](/doc/mdx/coverage.md) naming every
supported/refused cell.

### E. Live-sync efficiency — ready seam, lower immediate payoff

**Existing capability anchor:** completed `leafbud-editor-follow`; additional work
should get deliberate scope/tracking if selected.

**Options:** incremental CVR pulls, or advisory presence-based prioritization.
The pinned upstream supports cookie-based `put`/`del` patches with full-reset
fallback. A process-local replica must advance its base map and cookie together;
restart starts fresh, and neither belongs in document receipts. Empty delta is
not an empty document. Keep the same connected lifecycle and adoption policy.

Measure request volume/latency first. Low volume favors the current full snapshots
and simple fair scheduler. Presence can prioritize observed activity but cannot
prove nobody is attached, replace permission checks, or unsubscribe quiet roots.
No adaptive controller or live unsaved-Yjs overlay is required by current usage.

### F. Explicit PDS tools — retain as separate deferred work

`leafbud-pds-publish` and `leafbud-pds-sync` remain P3. Native editor/release
semantics must not silently switch to a separate PDS-managed identity/lifecycle.
Reassess these only for an explicit direct-PDS use case. The
[auth research](auth-architecture0.glm53m.md) (2026-11-10) confirmed the
Leaflet-brokered design is sound for current needs and found a complete dormant
direct-PDS session stack already in `atproto::session` — reviving these tickets
would start from tested code, with the capability ceiling (custom rkeys,
scheduling, gated content) as the actual driver, not auth availability.

## 5. Tracker and session state

At this update, metadata, rekey and local list are **closed**. The text/structure,
binding, recovery, conflicts, follow and earlier authoring-foundation children
are closed. The `leafbud-editor` parent epic remains open for its final scope/
administrative review; do not infer published lifecycle completion from the
draft acceptance. The author epic is complete with naming acceptance. Inspect
still needs its parent-level scope review, not duplicate implementation tickets.

| Session | Disposition |
| --- | --- |
| `ses_e3bb92bc4ffeN77oF8X2ayoH9a` | Completed 0.23 content-CID/inline-size verification at ~241k; parent accepted |
| `ses_e3bcb6540ffegZC39tN1E6wuTc` | Completed 0.22 action contract/preflight at ~223k; parent accepted |
| `ses_e3c10b84effeXUnnnL5oZ5M7TJ` | Completed 0.21 published adoption/state; ~422k exceeded rotation limit, retired |
| `ses_e3c29df03ffeKTX7z5IHA3nup5` | Completed naming at 214,041 reported tokens; parent accepted 0.20 |
| `ses_e3c420e5affeLoQfS0oTd9wu5b` | Completed source fidelity/coverage at 239,394 measured tokens; parent accepted 0.19 |
| `ses_e3d511cf0ffeKLT5XsWr37v0r6` | Completed local metadata/rekey/list at ~215k; parent accepted 0.18 |
| `ses_e3d6d728effe6RFnyZcrUtifT9` | Completed terminal recovery/no-op correction at ~263k; rotate rather than continue major work |
| `ses_e3d96851dffeS2cPD2vPHjGklF` | Completed keyed save loop at ~276k; retired from major continuation |
| `ses_e3dcb1f8effebiT0KtCRZB8dWf` | Completed defaults/description/P1 settlement at ~256k; retired |
| `ses_e4121e55dffeF009EbMiG1Xinz` | Completed native effects and later candidate inspection at ~310k; retired |
| `ses_e4217c526ffeXeHByo5ZswnB3F` | Parent reviewed through 0.23; execution/recovery dispatch awaits the cleanup-policy decision |

Historical independent waves and earlier agents are linked from the receipts.
Their old g recommendations or polling-first descriptions are superseded by
current AGENTS and the operator's explicit decisions; do not relitigate them.

## 6. Private deployed objects and effect boundaries

The user authorized **at most two new unpublished verification drafts**; both
creation slots are consumed. **2026-11-10 update:** the operator relaxed this —
a **modest disposable live-test loop** is now permitted: freshly minted test
drafts may be created, verified against, and **deleted afterward** once the
delete capability lands, at low volume (markers under `.test-agent` prevent
accidental repeats). Quiet publish (forced `sendEmail:false`) suppresses
subscriber email only, **not public record visibility**, and the native
`deleteLeaflet` action deletes the editor rather than the public record —
verify a published-record deletion path exists upstream before any test
publish. Existing P1 and the standalone draft have been used only within the
authorized test scope; neither was published or emailed, and no unrelated
document or asset was cleaned up.

- **P1:** user explicitly selected the candidate after the original publication
  action response decoder lost its locator. Its original-action attribution is
  still unproven. It is now settled and has passed complete bound-save proof.
- **Standalone:** known original locator; its old theme-omission partial was
  deliberately accepted with `originalNotVerified` provenance. It is now clean.
- Private mappings, sources, comparisons, budget and started-proof markers live
  under ignored `.test-agent/native-effects/`. Presence/watch probes have separate
  ignored scratch homes. Do not print credentials or copy private bodies into docs.
- Do not blindly rerun ignored deployment tests or erase their started markers.
  They intentionally prevent repeating creation or reconstructing an old edit.
  Inspect the current recorded state and the exact test's effect before use.
- The completed local-authoring batch used fixtures/temporary workspaces and
  left `~/l` and both test drafts alone.

## 7. Minimal continuation checklist

1. Read this report, [AGENTS](/AGENTS.md), and the completed
   [local-tool receipt](local-authoring-tools0.gpt61s.md). No need to reload every
   historical design or rerun completed checks.
2. Await the active native-publication preflight Sol return; do not duplicate or poll.
   Review action/effect contract and recovery design before fresh execution dispatch.
3. Parent integration review through 0.21 is complete. Trust reported tests; add
   targeted regressions only for new changes or actual unresolved concerns.
4. Preserve predecessor commits before corrections. Use explicit owned jj paths;
   never `jj edit`, squash or push. Shared `.beads/issues.jsonl` changes may include
   multiple sessions' legitimate ticket updates—inspect before checkpointing.
5. Update current capability/docs and ticket pointers from the result. Give the
   user a short outcome plus next 1–4-ticket narrative.
6. Select the next front deliberately. Continue major implementation in a fresh
   Sol high session; external release effects retain their separate authorization.

This file records accepted work and proposed fronts; every listed front is not
selected. The [README](/README.md) remains the maintained
capability/usage home, and Beads remains the current work-state authority.
