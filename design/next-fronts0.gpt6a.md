---
type: Report
title: "Leafbud: current ground, next fronts and compaction handoff"
description: "Compaction-ready account of the reviewed 0.17 delivery, active fresh-Sol local-authoring design, remaining work fronts, exact verification boundaries and safe continuation instructions."
resource: /design/next-fronts0.gpt6a.md
status: current-handoff
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Long-running parent session: grounded architecture, reviewed implementations and corrections, commissioned fresh Sol batches, and wrote this user-requested report before compaction"
  - resource: /AGENTS.md
    title: "Active operator constraints, fresh-Sol selection and 240–350k rollover"
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
    title: "Concrete design of the active next local-authoring batch"
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

**Latest completed baseline: 0.17.0, through `c1a2ce11`.** The complete supported
draft authoring/save/watch/recovery loop is implemented. Parent has read the
terminal resolution/provenance code and retained-default no-op correction and
accepts their architecture and reported verification. Later files in the shared
checkout may belong to the active next agent; do not mistake them for abandoned
work or overwrite them.

**One agent is currently active:** fresh Sol high,
`ses_e3d511cf0ffeKLT5XsWr37v0r6`, implementing:

- `leafbud-author-metadata`
- `leafbud-author-rekey`
- `leafbud-inspect-list`

Its concrete receiving design is [local authoring tools](local-authoring-tools0.gpt6a.md).
It owns code/tests, relevant README changes and its own implementation receipt.
Parent owns this handoff and parent design/review documents. **Do not spawn a
duplicate implementation or poll its progress.** Await the harness completion
notification, then review its result. If it needs rollover, use its committed
handoff in a fresh Sol high session.

User's latest request: write the current report and next-front design in Markdown
so this parent session can compact. No completed implementation/test/deployed
probe needs repeating merely because context was compacted.

## 1. What has actually landed

| Capability | Current ground | Receiving home |
| --- | --- | --- |
| Local draft source | `leafbud-mdx/2`, implicit main page, ordinary Markdown, explicit named pages, one-time durable keys, new/retired associations | `document/model`, `document/mdx`; [foundation corrections](authoring-corrections0.glm53m.md) |
| Read/compare/adopt | Current native reads, scoped status, normalized external-tool diff, guarded import/pull | `workflow`; [watch receipt](connected-watch0.gpt61s.md) links predecessors |
| Persistent inbound sync | Real Phoenix/Supabase subscriptions, catch-up/reconnect, coalescing, optional passive presence, slow backstop | `leaflet/web::live`, `workflow::watch`; [connected watch](connected-watch0.gpt61s.md) |
| Native effect foundation | Both mint transports, exact bounded requests, writer acknowledgments, explicit known-ID reads, compact pending attempts | `leaflet/protocol`, `leaflet/web`, `workflow::native`; [native effects](native-effects0.gpt61s.md) |
| Complete supported saves | Preview-first public push; first binding; text/marks/breaks; new/drop/move/reorder; flat lists, code/images, named pages, selected recreation | `editor::write`, `native::save`; [keyed saves](keyed-save0.gpt61s.md) |
| Deliberate terminal recovery | Original-success settlement separately from explicit accept-current/replan; bounded `lastResolution`; no uncertain replay/remint | `native::resolve`; [terminal recovery](terminal-recovery0.gpt61s.md) |

The supported native lifecycle is **draft**. Canvas, nested/task lists, richer
interactive objects, uploads, published-editor adoption and public release are
still separate work. There is no whole-body regeneration default, automatic
cleanup, saved historical body or executable request journal.

The domain grouping is now nine crates: document **model/MDX**, Leaflet
**protocol/editor/web/records**, **ATProto**, **workflow**, and **CLI**.
Protocol types are consumed by web/editor; pure editor logic does not depend on
HTTP or workflow. Local file effects and orchestration stay in workflow.

### Verification to carry forward

- Latest regular gate: **300/300 workspace tests**, six opt-in deployment tests
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

Trust these reported checks. Add tests for new changes or a specific unresolved
review question; do not rerun everything to reconstruct confidence after compaction.

## 2. Decisions that must survive

- **Fresh Sol high for major features.** Continue through coherent batches and
  parent reviews. Rollover at a coherent checkpoint **between 240–350k reported
  context**, completing it by 350k. Smaller finished slices can end earlier.
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

## 3. Next front — local authoring ergonomics

**Standing: commissioned and active.** Three tickets, fresh Sol high, design in
[local authoring tools](local-authoring-tools0.gpt6a.md).

This batch makes the implemented system easier to operate: list local drafts,
edit selected title/description/tags/date fields, and rename block/page keys while
keeping native bindings. It reuses source-aware patches, inventory and locks.
Its important judgment is keeping author edits, alias edits and receipt settlement
distinct. Confirmed rekeying should leave the editor witness unchanged; an author
metadata edit must not hide itself by advancing receipts.

**Expected review gates:** exact unrelated bytes/comments preserved; no network
or credential dependency; current/pending/future-owned state respected; native
IDs and confirmed receipts survive rekey; malformed/duplicate/unbound inventory
is truthful; actual workflow/CLI no-op and classification fixtures pass.

The agent may choose exact tag flag spelling and narrow patch API names. It may
not turn this into arbitrary property editing, publication retargeting, a second
parser, or a new tracking database. Public additions likely land as 0.18, but
that version and capability count are unverified until the agent returns.

## 4. Fronts after the active batch

These are recommendations/options, not claims of commissioned implementation.
Use **fresh Sol high**, selecting coherent batches of 1–4 tickets.

### A. Source fidelity and executable coverage — recommended next foundation

**Tickets:** `leafbud-mdx-source` + `leafbud-mdx-coverage`.

**Payoff:** fewer ordinary authoring spellings cause refusal, and future changes
have a visible operation/content coverage contract. The current code has disclosed
edges around flow-style YAML, trailing comments and some property spellings;
body comment support also needs deliberate source ownership. The active local
tool batch may remove some edges—refresh this scope from its actual results.

**Bounded design:** choose a small corpus of currently failing valid source forms.
Extend parser-derived patch ranges or use an explicit owned-region rewrite where
that is simpler, preserving unrelated bytes. Reuse the real production parser and
existing test servers. Maintain a compact matrix of source form/content kind ×
create/change/drop/reorder/pull/recovery, linking fixtures and naming unsupported
cells. Do not build a generic lossless formatter, test framework or all-format
converter first. Unknown content remains a clear refusal, never a truncated save.

**Acceptance:** each chosen form works through the actual command/workflow with
semantic equivalence and no-op/race preservation; unsupported cases remain explicit.
This improves the most-used path without adding another remote effect surface.

### B. Naming and path intent — small independent authoring completion

**Ticket:** `leafbud-author-names`.

**Payoff:** reconcile intentional filename/path choices without confusing them
with native identity. This is the remaining open child of the author epic outside
the active batch at handoff.

**Bounded design:** implement deliberate `tidy --from filename|path` and `--accept`
using independent naming witnesses. Normal push never renames files. A pathname
change keeps local UUID and native binding; collision and conflicting direction
require an explicit choice. Local path intent does not reserve or change a native
public route. Preserve the established global/workspace selection conventions.

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

At handoff, **in progress**: metadata, rekey, local list. The text/structure,
binding, recovery, conflicts, follow and earlier authoring-foundation children
are closed. The `leafbud-editor` parent epic remains open for its final scope/
administrative review; do not infer published lifecycle completion from the
draft acceptance. Author/inspect parent epics likewise need their remaining
children reviewed, not duplicate implementation tickets.

| Session | Disposition |
| --- | --- |
| `ses_e3d511cf0ffeKLT5XsWr37v0r6` | **Active:** local metadata/rekey/list; await result |
| `ses_e3d6d728effe6RFnyZcrUtifT9` | Completed terminal recovery/no-op correction at ~263k; rotate rather than continue major work |
| `ses_e3d96851dffeS2cPD2vPHjGklF` | Completed keyed save loop at ~276k; retired from major continuation |
| `ses_e3dcb1f8effebiT0KtCRZB8dWf` | Completed defaults/description/P1 settlement at ~256k; retired |
| `ses_e4121e55dffeF009EbMiG1Xinz` | Completed native effects and later candidate inspection at ~310k; retired |
| `ses_e4217c526ffeXeHByo5ZswnB3F` | This parent session, now preparing to compact; sources retain this actual ID |

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
- The active local-authoring batch has **no live-object edits** in its scope.
  It uses fixtures/temporary workspaces and leaves `~/l` and both test drafts alone.

## 7. Minimal continuation checklist

1. Read this report, [AGENTS](/AGENTS.md), and the active
   [local-tool design](local-authoring-tools0.gpt6a.md). No need to reload every
   historical design or rerun completed checks.
2. Await the active Sol's completion. Respect its file ownership and any coherent
   rollover handoff. Do not poll or launch a second writer for the same batch.
3. Read its receipt and the important changed patch/identity/lock boundaries.
   Trust reported tests; request targeted regressions for actual concerns.
4. Preserve predecessor commits before corrections. Use explicit owned jj paths;
   never `jj edit`, squash or push. Shared `.beads/issues.jsonl` changes may include
   multiple sessions' legitimate ticket updates—inspect before checkpointing.
5. Update current capability/docs and ticket pointers from the result. Give the
   user a short outcome plus next 1–4-ticket narrative.
6. Select the next front deliberately. Continue major implementation in a fresh
   Sol high session; external release effects retain their separate authorization.

This file is a checkpoint, not a claim that the active batch has finished or that
every listed front is selected. The [README](/README.md) remains the maintained
capability/usage home, and Beads remains the current work-state authority.
