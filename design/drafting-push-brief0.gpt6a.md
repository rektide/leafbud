---
type: Brief
title: "Fresh design commission — normal drafting and native push"
description: "Corrected product-first commission after rejection of the existing-text-only milestone: create, add paragraphs/headings, revise structure and push complete drafts through a right-sized implementation."
resource: /design/drafting-push-brief0.gpt6a.md
status: commissioned
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "User rejected the narrow existing-text scope, questioned the commissioning and requested a fresh design when the original ask missed the product need"
  - resource: /AGENTS.md
    title: "Complexity budget and useful-workflow delivery requirement"
  - resource: /design/native-edit-join0.gpt6a.md
    title: "Parent withdrawal of text-only milestone; previous evidence retained"
  - resource: /design/native-edit0.gpt6a.md
    title: "Earlier native/Yrs evidence and cross-review, available as research rather than a mandatory architecture"
  - resource: /design/native-edit0.glm53m.md
    title: "Earlier g operation analysis and corrected concurrency/retry claims"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Original whole-document goal, creation/staging evidence and keyed bidirectional intent"
  - resource: /design/status-witnesses0.gpt61s.md
    title: "Current observation and receipt capabilities"
  - resource: /design/human-diff0.gpt61s.md
    title: "Separate implemented human comparison tool integration"
---

# Fresh design: ordinary drafting must work

## Why restart

The previous briefs asked a broad technical question about native editing but
repeatedly emphasized a small first implementation. They did not require a
complete normal drafting workflow as the delivery acceptance condition. The
resulting existing-single-text-node proposal was inadequate, and the parent
endorsed that inadequacy. The operator rejected it.

**Keep the verified research; reset the product boundary and architecture choice.**
An existing-text writer may be an internal checkpoint, never the proposed useful
delivery by itself. Reducing complexity means choosing simpler mechanisms capable
of the user actions below, not excluding those actions.

## Required user stories

1. **Start on disk.** Create an ordinary local document and write a title, headings,
   paragraphs, inline formatting and links without obtaining native IDs by hand.
2. **First push.** Push that file into a new native draft. Resolve the publication
   context from explicit intent/config; record the actual returned binding. Save
   does not publish publicly.
3. **Continue drafting.** Insert paragraphs and headings anywhere, including before
   the old first block. Edit text/marks/breaks, split/join paragraphs, delete blocks
   and change their order through ordinary source edits.
4. **Push the revision.** The same bound draft now expresses the supported desired
   document. Automatic identity preparation handles new source nodes; no requirement
   that every source key already have a confirmed remote binding.
5. **Read and repeat.** Pull/read-back agrees with current authored meaning. A repeat
   unchanged push creates no second draft, duplicate body or unnecessary generation.
6. **Useful failure behavior.** Unsupported rich input, conflicting remote edits or
   uncertain writes are explicit. Existing unrelated content is not silently lost.

The whole-document profile must include ordinary paragraph/heading drafting and
their structure. Define handling of the already implemented quotes/code/rules/
images/page references and ordinary lists concretely: what is round-tripped,
preserved unchanged, newly enabled, or explicitly outside the first profile.
Do not hide the main user workflow behind exhaustive rich-type coverage, but
equally do not label paragraph/heading additions a future optimization.

## Architectural freedom and constraints

- Low volume, few users: minimum operation counts and character-level continuity
  are secondary to bounded conceptual complexity and useful coverage.
- Evaluate straightforward keyed block creation/update/detachment with a simple
  ordering pass, coarse block/page replacement, and generated-body materialization
  on equal terms. Choose a coherent default; do not build two broad engines to
  avoid choosing. Preserve a small seam for later improvement.
- Preserve native document/public identity. State clearly which page/block/text
  identities, references and anchors survive or are replaced by the chosen default.
  Coarser effects can be an explicit product trade. Interactive object semantics
  and unrelated content still require deliberate ownership.
- Reuse Yrs and native helpers; no new CRDT, minimum-edit optimizer, general
  structural comparison framework or parallel semantic document model by default.
- One current source and compact identities/receipts. No historical body, CRDT
  snapshot, alternate JSON master or persisted executable mutation journal.
- Keep the deployed-protocol proof separate from synthetic evidence; do not claim
  CAS or exactly-once semantics the server does not expose. Exact payload replay
  differs from independently compiling an uncertain stale edit.
- The incoming Leafbud rename/config/data conversion is independently owned.
  Human external-tool diff is already a separate implementation; it does not
  control executable changes or need redesign here.

## Design deliverables and independent join

Commission **fresh Astra high and fresh g** with this same brief. Each owns a new
model-suffixed `design/drafting-push0.*.md`; neither reads the other's same-wave
draft before both are committed. Prior-wave research is available. After the
independent join, each reads the peer and commits a cross-review addendum with its
revised stance. Parent/user then assess the architecture against the user stories.

Each proposal must contain:

- A concrete walk-through of initial local writing, first push, inserted heading/
  paragraph, deletion/reordering, second push and pull. Trace source identities,
  confirmed associations and native effects at each step.
- One recommended end-to-end implementation, alternatives with real complexity/
  identity tradeoffs, and supported tool/API evidence. No assumption that existing
  import-only validation or automatic key assignment already supports new nodes.
- The minimum needed source/model changes, key preparation, native creation/write
  transport, fact/text materialization, receipt handling and verification.
- Internal checkpoints and agent-sized batches that converge on the WHOLE useful
  deliverable. Allow multiple sessions if warranted; do not substitute the easiest
  checkpoint for completion. Avoid estimates and speculative frameworks.
- Executable acceptance scenarios for every required user story, concrete remaining
  protocol uncertainties, and what would change the chosen architecture.

Design only: no production edits, live native writes, user-file transformations or
rename work. New drafts and research receipts retain actual session provenance.
