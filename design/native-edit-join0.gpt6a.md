---
type: Assessment
title: "Native edit design join — useful drafting and push as the delivery boundary"
description: "Parent assessment and user correction: existing-text edits are an internal checkpoint; usable drafting requires first binding and paragraph/heading creation, editing, deletion and ordering in one coherent push loop."
resource: /design/native-edit-join0.gpt6a.md
status: recommendation
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "User's complexity-first direction, independent-wave join, rejection of the text-only milestone and correction to a useful complete drafting/push delivery"
  - resource: /AGENTS.md
    title: "Low-volume product fit and bounded conceptual complexity"
  - resource: /design/native-edit0.gpt6a.md
    title: "Astra independent draft 4ab29da3 and revised single-container stance at 0147be35, including executed Yrs/Yjs concurrency and wire probes"
  - resource: /design/native-edit0.glm53m.md
    title: "g independent draft ff2f4a7b and cross-review corrections/revised stance at 515d3fd2"
  - resource: /design/status-witnesses0.gpt61s.md
    title: "Implemented scoped receipts and shared read-only observation, 137-test receipt and live rename gap"
  - resource: /design/human-diff0.gpt61s.md
    title: "Delivered normalized Markdown views and direct external-tool invocation, 148-test receipt"
  - resource: /crates/workflow/src/diff.rs
    title: "Reviewed small human view/process adapter and its no-source-write boundary"
  - resource: /crates/cli/src/diff.rs
    title: "Reviewed thin foreground CLI rendering"
---

# Native edit design join

## Controlling correction: deliver normal drafting

The user rejected the parent recommendation below as an inadequate product slice:
editing only existing single-container text does not support drafting a document,
adding a paragraph or inserting a heading. **That narrow delivery recommendation
is withdrawn.** The probe/algorithm findings remain useful implementation evidence.

The next product milestone is **write locally, push a draft, revise its ordinary
document structure, and push again**. Minimum end-to-end acceptance:

1. Create a local draft without remote IDs, then push it to a newly bound native
   editor using the configured publication context without publishing it.
2. Add paragraphs and headings anywhere, including a heading before the previous
   first block; assign required source IDs automatically once.
3. Edit prose/formatting/links, split/join paragraphs, remove blocks and reorder
   ordinary supported content. New nodes need not already have confirmed bindings.
4. Push the complete supported desired document into the same bound editor, with
   identity/ownership effects stated and ordinary conflict handling intact.
5. Read back/pull equivalent content and associations; a repeated unchanged push
   is a no-op rather than a second draft or duplicate body.
6. Preserve unsupported/unrelated existing content by refusing an unsafe save,
   rather than silently dropping it. A declared rich-content limit is different
   from excluding basic paragraph/heading authoring from the milestone.

The implementation can have internal commits for parser state, key assignment,
Yrs editing, creation and transport. Completion of one such mechanism is not the
claimed completion of this user workflow. Coordinate `author-create`, `author-keys`,
`editor-text`, `editor-structure`, `editor-bind` and the essential conflict/recovery
work as one delivery goal instead of making every mechanism a separate product
stop. Existing tickets remain useful ownership/checkpoint units.

**Simple structure is in scope; structural optimization is not required.** Use
source keys to recognize existing objects, create genuinely new blocks, detach
removed memberships and set a straightforward desired order. A changed-page
ordering pass or coarse text/topology update is acceptable when simpler. If
generated page/body materialization is materially simpler for this complete loop,
evaluate it on that product payoff and explicit identity effects. Do not make
minimal moves, character continuity or an elaborate tree matcher prerequisites
for adding a paragraph. The exact write strategy remains to be selected against
this corrected delivery scope; the rejected scope is not a reason to restart all
the parser/protocol research.

## Earlier parent recommendation — superseded as a delivery boundary

**Land an existing-bound-text save, not a general structural planner.** Keep the
current block and its single native text container, make one simple text splice,
apply desired supported marks, and let Yrs encode the edited state. Pair this with
essential drift checks, pending-attempt evidence and read-back. Leave topology
changes, new blocks, moves and first binding for separately chosen slices.

This is a parent recommendation following both requested independent drafts and
their cross-reviews. It is not a new implementation commission or human acceptance
of the broader structural architecture. The incoming name is Leafbud; another
session owns the code/config/data rename. Paths here identify the inspected code.

## Completed work and architectural separation

The two initial proposals were committed before either author read the other.
Each subsequent addendum preserves its original argument and states a revised
position. Astra's stage-two draft reviewed g's initial draft, and vice versa;
neither relied on the other's evolving addendum to manufacture agreement.

The [human diff](human-diff0.gpt61s.md) is implemented independently: current local
and bound-editor content are rendered through the same Markdown view and handed
to a user-selected foreground executable. Temporary operands are local first,
editor second; receipt/binding/key noise is omitted; exact supported content
fallbacks remain. There is no custom diff algorithm or executable edit plan.
The implementation reported 148 passing tests and clean formatting/Clippy.
Live comparison awaits the independently owned namespace/store integration.

The [status work](status-witnesses0.gpt61s.md) supplies current observations and
whole-document equality/change direction. It does not supply a free structural
planner, nor does external-view equality prove receipt agreement. These remain
appropriately small, distinct capabilities.

## What the cross-review established

| Question | Joined evidence and parent judgment |
| --- | --- |
| Do we need a text diff library or tree matcher now? | Neither design needs one. A common-prefix/common-suffix splice follows an existing y-prosemirror/lib0 precedent and is much smaller than the earlier LCS ambition. |
| What do we retain? | Bound document, existing block/text fact, paragraph and single text child. Character continuity is limited to retained ranges, not a guarantee for every comment or annotation. |
| What payload? | One complete state of the **loaded-and-edited** Yrs document per changed text fact is a reasonable initial native-shaped choice. A single transaction delta is also valid with its causal base; payload optimization can remain private. An independently generated replacement document is not equivalent. |
| Does exact read-back prove no concurrent edit was lost? | No. The offline probes show child removal can hide concurrent text while reading exactly as desired. Retaining the child and editing characters has different visibility behavior. Neither supplies server CAS. |
| Can uncertain writes simply be recompiled? | No. Two independently compiled stale-base edits can yield duplicated text. Retry identical encoded bytes/IDs in-process; after interruption, resolve the earlier attempt from acknowledgments/current state or stop uncertain. |
| Is broader generation a bad product choice? | No. It genuinely simplifies some heterogeneous authoring. It also needs staging, rebinding, reference/interactive-object policy and explicit anchor effects. Keep it a credible future choice instead of implementing two engines now. |

g corrected its concurrency/retry overclaims and adopted splice retention and
full edited-state transport. Astra conceded the validity of a single delta and
the genuine additional machinery required for keyed structure. Those corrections
make this join stronger than agreement on a generic “incremental editing” slogan.

## Earlier proposed next batch — internal evidence only

Use `leafbud-editor-text` plus the essential portions of
`leafbud-editor-conflicts`. **Sol high is recommended** for this first native
write boundary: the editing algorithm is deliberately modest, but protocol,
identity, uncertainty and verification still need careful integration.

The first delivery would let an author change words or ordinary formatting in an
already imported eligible paragraph/heading/quote and save to the same draft
editor. Several eligible blocks can be changed together. The operation should:

1. Require an existing verified binding and confirmed source associations.
2. Permit changed blocks with one existing XmlText child and supported text/marks;
   preserve unchanged other supported content, including blocks with breaks.
3. Compare characters, retain a common prefix/suffix, replace the middle once,
   then apply/clear supported marks over desired ranges. A mark-only edit retains
   characters. No scalar-plus-mark matcher or optimal edit script is required.
4. Refuse the whole operation before effects if desired changes include unsupported
   metadata, structure, resources, type or inline topology. Include the existing
   derived-title interaction in fixtures rather than silently accepting it.
5. Keep a small in-file pending-attempt record, recheck relevant evidence and
   submit one payload per changed fact. No persistent request/body/history store.
6. Read back supported meaning and preserved identities/scope. Settle only verified
   receipts under the local original-byte guard. Report partial/diverged/uncertain
   results; never replan repeatedly until desired words appear.

A flat list of target text facts and prepared updates, with concise effect labels,
is sufficient. The human message can say: “save two text blocks; retain their block
and text containers; public content unchanged.” No second semantic change tree or
general operation framework is justified by this slice.

Offline Yrs/real-Yjs fixtures and loopback request/recovery tests precede any
separately authorized deployed write probe. Existing-text work does not depend on
automatic key assignment, which remains accepted but unimplemented. It must not
close the broader structure/recovery tickets merely by delivering this narrow loop.

## Earlier staging choices — reconsider under the corrected product boundary

- **If the splice/mark implementation grows disproportionate**, all-character
  replacement inside the retained text child remains an acceptable simpler option.
  It trades character continuity, not correctness of desired supported meaning.
- **Later inline topology:** coarse child rebuilding versus explicit recreation
  still needs a product decision informed by the documented concurrent-visibility
  effect. Keeping it out of the first slice avoids that decision now.
- **Later page structure:** g prefers targeted fractional moves; Astra permits
  moving all desired unique children into order. Prefer whichever is simpler with
  verified native helpers; low operation count is not an independent requirement.
- **Broader generated-body ownership** remains credible if actual authoring needs
  make it a better fit. Do not make every unsupported case imply an already-built
  recreation engine or an elaborate preservation optimizer.

The immediate road is therefore one useful save loop, followed by evidence-driven
selection of topology/structure scope. Local inventory remains an independent,
small g task (`leafbud-inspect-list`) if native-write work is not the next choice.

## Connections

- [Astra design and addendum](native-edit0.gpt6a.md#10-cross-review-and-revised-stance--after-the-independent-join):
  narrow first-slice recommendation and executed probe accounts.
- [g design and addendum](native-edit0.glm53m.md#addendum--cross-review-of-the-astra-draft-and-revised-stance):
  keyed operation proposal, corrections and remaining structure preferences.
- [Human diff receipt](human-diff0.gpt61s.md): already delivered presentation
  capability, independent of the executable write strategy.
- [Scoped status receipt](status-witnesses0.gpt61s.md): the observation/equality
  foundation and the separate live workspace rename limitation.
