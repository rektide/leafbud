---
type: Report
title: "Leafbud: current ground, next fronts and compaction handoff"
description: "Continuation account of accepted 0.19 source fidelity/coverage, active filename/path reconciliation, remaining fronts and exact verification boundaries."
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
    title: "Parent architecture and compaction report, 0.18/0.19 integration acceptance, continuing Sol commissions and 220–320k rotation policy"
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

**Latest accepted baseline: 0.19.0, through `aa13bed1`.** The draft workflow now
includes whole-flow author headers, parser-selected preparation spellings and
source-only block-anchored comments. Parent reviewed flow collection pairing,
comment selection/attachment/adoption, source-bound tidy and maintained coverage;
the bounded contract and reported 328-test gate are accepted (section 4A).

**Active agent:** fresh Sol high `ses_e3c29df03ffeKTX7z5IHA3nup5`, implementing
**filename/path reconciliation** (`leafbud-author-names`, section 4B). The source/
coverage predecessor returned at 239,394 measured tokens; it is finished. Rotate
at major task boundaries or **220–320k reported context**, superseding 240–350k.

Predecessor `ses_e3d511cf0ffeKLT5XsWr37v0r6` completed all three local-tool tickets
at about 215k context. Its [receipt](local-authoring-tools0.gpt61s.md) records APIs
and source-shape limits. Do not duplicate the active implementation or poll its
progress; await the harness return and review the actual result.

The earlier compaction-report request is fulfilled. The latest request is to
continue chasing the next moves using Sol high agents with the revised rotation
window. Completed checks need not repeat merely because context was compacted.

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

The supported native lifecycle is **draft**. Canvas, nested/task lists, richer
interactive objects, uploads, published-editor adoption and public release are
still separate work. There is no whole-body regeneration default, automatic
cleanup, saved historical body or executable request journal.

The domain grouping is now nine crates: document **model/MDX**, Leaflet
**protocol/editor/web/records**, **ATProto**, **workflow**, and **CLI**.
Protocol types are consumed by web/editor; pure editor logic does not depend on
HTTP or workflow. Local file effects and orchestration stay in workflow.

### Verification to carry forward

- Latest regular gate: **328/328 workspace tests**, six opt-in deployment tests
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

## 4. Current and next fronts after 0.19

Front A is accepted; front B is commissioned and active after integration review.
Later fronts remain options. Use **fresh
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

### B. Naming and path intent — small independent authoring completion

**Ticket:** `leafbud-author-names`.

**Active:** fresh Sol `ses_e3c29df03ffeKTX7z5IHA3nup5`, under the operator's
continuing-work instruction. Owns implementation, tests, maintained usage/coverage
and its own receipt; parent owns this report and integration review.

**Payoff:** reconcile intentional filename/path choices without confusing them
with native identity. This is the remaining open child of the author epic.

**Bounded design:** implement deliberate `tidy --from filename|path` and `--accept`
using independent naming witnesses. Normal push never renames files. A pathname
change keeps local UUID and native binding; collision and conflicting direction
require an explicit choice. Local path intent does not reserve or change a native
public route. Preserve the established global/workspace selection conventions.

The implementation must define filename/logical-path mapping and workspace versus
external coordinates explicitly, use independent compact `names` witnesses, and
expose read-only naming classification in status. Selectors compose as a union;
dry-run has no source/rename/witness effects. Pending attempts block managed naming.

Consequential review: no-clobber move plus witness/source update needs an honest
failure ordering, not a claim of multi-file atomicity. Carry all explicit external
operands through late duplicate checks; current tidy's single-operand early check
must be strengthened for the new move path. Preserve native IDs, editor receipts,
historical resolution, body comments and unrelated author bytes. A name settlement
cannot conceal editor drift. The agent should checkpoint a material coordinate or
transaction-policy fork before building a general mechanism.

This can follow the new field/key patch helpers. Keep it separate from remote
slug support: current source evidence says ordinary native publication chooses its
first public key/path. Do not invent custom-rkey behavior by sending ignored fields.

### C. Published lifecycle and release — larger next product loop

**Tickets:** start design with `leafbud-release-state` +
`leafbud-release-adopt`; then join `leafbud-release-native`.
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

Reuse existing native web actions and public record/protocol crates where their
actual behavior fits; inspect current upstream capabilities before choosing the
adapter. No parallel PDS publication lifecycle, copied source body, or automatic
email delivery. **A deployed publish/email/share proof needs a new explicit
operator instruction:** current live authorization covers unpublished test drafts.

### D. Content/profile expansion — choose one vertical family

**Tickets:** `leafbud-mdx-structure`, `leafbud-mdx-rich`, `leafbud-mdx-media`.

**Payoff:** adopt and edit more real Leaflet content. Pick a concrete fixture family
from actual user need—such as nested structures, one richer block, or media
resources—and carry model/parse/render/observe/write/read-back together. Keep the
keyed foundation and declare shared/interactive identity consequences. Uploading
bytes is a different capability from preserving a remote image URL.

Do not commission all these tickets as an abstract “support everything” batch.
Choose the next useful family with its native source evidence and a complete
round-trip case; retain explicit unsupported diagnostics for the rest.

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
Reassess these only for an explicit direct-PDS use case.

## 5. Tracker and session state

At this update, metadata, rekey and local list are **closed**. The text/structure,
binding, recovery, conflicts, follow and earlier authoring-foundation children
are closed. The `leafbud-editor` parent epic remains open for its final scope/
administrative review; do not infer published lifecycle completion from the
draft acceptance. Author/inspect parent epics likewise need their remaining
children reviewed, not duplicate implementation tickets.

| Session | Disposition |
| --- | --- |
| `ses_e3c29df03ffeKTX7z5IHA3nup5` | **Active:** filename/path reconciliation; 220–320k rotation window |
| `ses_e3c420e5affeLoQfS0oTd9wu5b` | Completed source fidelity/coverage at 239,394 measured tokens; parent accepted 0.19 |
| `ses_e3d511cf0ffeKLT5XsWr37v0r6` | Completed local metadata/rekey/list at ~215k; parent accepted 0.18 |
| `ses_e3d6d728effe6RFnyZcrUtifT9` | Completed terminal recovery/no-op correction at ~263k; rotate rather than continue major work |
| `ses_e3d96851dffeS2cPD2vPHjGklF` | Completed keyed save loop at ~276k; retired from major continuation |
| `ses_e3dcb1f8effebiT0KtCRZB8dWf` | Completed defaults/description/P1 settlement at ~256k; retired |
| `ses_e4121e55dffeF009EbMiG1Xinz` | Completed native effects and later candidate inspection at ~310k; retired |
| `ses_e4217c526ffeXeHByo5ZswnB3F` | Parent resumed after compaction, reviewed 0.18 and refreshed this handoff |

Historical independent waves and earlier agents are linked from the receipts.
Their old g recommendations or polling-first descriptions are superseded by
current AGENTS and the operator's explicit decisions; do not relitigate them.

## 6. Private deployed objects and effect boundaries

The user authorized **at most two new unpublished verification drafts**. Both
creation slots are consumed. There is no new mint budget. Existing P1 and the
standalone draft have been used only within the authorized test scope; neither
was published or emailed, and no unrelated document or asset was cleaned up.

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
2. Await the active naming Sol return; do not duplicate its work or poll.
   Review coordinate mapping, move/witness failure ordering and independent status.
3. Parent integration review through 0.19 is complete. Trust reported tests; add
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
