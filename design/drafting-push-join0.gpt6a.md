---
type: Assessment
title: "Complete drafting design join — choose one body engine"
description: "Parent join of the fresh whole-workflow drafts and cross-reviews: both satisfy normal drafting, with a generated-body recommendation under the user's complexity budget and explicit body-identity tradeoffs."
resource: /design/drafting-push-join0.gpt6a.md
status: withdrawn
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
updated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Parent join and subsequent operator rejection of the generated-body recommendation; replaced by complete keyed authoring design"
  - resource: /design/drafting-push-brief0.gpt6a.md
    title: "Controlling product acceptance after the rejected text-only recommendation"
  - resource: /design/drafting-push0.gpt6a.md
    title: "Astra first draft b5e66475 and cross-review 4883ff58: complete generated-body approach and corrected corpus eligibility"
  - resource: /design/drafting-push0.glm53m.md
    title: "g draft 6985d01f, first-binding correction 7c0e6f95 and cross-review 367aed31: complete keyed engine and publication/protocol corrections"
  - resource: /design/native-edit-join0.gpt6a.md
    title: "Previous narrow recommendation explicitly withdrawn; prior mechanical evidence remains useful"
  - resource: /design/draft-checkout0.gpt61s.md
    title: "Actual 27-draft import and known inactive-text corpus evidence; initial failures had more than one cause"
  - resource: /design/human-diff0.gpt61s.md
    title: "Delivered separate human comparison capability"
---

# Complete drafting: parent design join

**Withdrawn after operator review, 2026-11-09.** The generated-body default below
discarded too much of the keyed continuity and extensible foundation the project
had developed. The receiving design is now
[Keyed drafting: complete documents, simple native updates](keyed-authoring0.gpt6a.md).
The previous assessment remains below as decision history, not implementation
direction. Complete drafting remains the delivery boundary.

This joins the [corrected product brief](drafting-push-brief0.gpt6a.md),
[Astra's proposal and cross-review](drafting-push0.gpt6a.md), and
[g's proposal and cross-review](drafting-push0.glm53m.md).

## Recommendation and decision standing

**Recommend one generated-body engine for the complete ordinary drafting loop.**
This recommendation follows the operator's preference for bounded conceptual
complexity, coarse writes where useful and full drafting rather than an optimized
existing-text checkpoint. It is not yet operator acceptance of the body-identity
trade, and no new write implementation is dispatched by this assessment.

The incoming product name is Leafbud. Another session owns the code/config/data
rename; paths refer to the current checkout. Preserve both authors' independent
arguments and cross-reviews as evidence; this join does not overwrite them.

## What the corrected wave accomplished

Both proposals now design the required workflow:

> Create locally → write normal Markdown → native first push in the selected
> publication context → insert/edit/split/join/delete/reorder → second push to the
> same draft → pull/read-back → unchanged push with no new remote content.

Paragraphs and headings are basic supported input. Both revised profiles also
include flat ordinary lists, the existing rich-text marks/breaks, quotes, code,
rules, remote-URL images and linear page references. Broader layout, nested/task
lists, local binary upload and interactive content are named separate capabilities,
not silently discarded content.

Native creation is part of delivery. G's initial browser-created-empty-draft
shortcut was rejected and corrected before the join. Its later standalone-only
creation policy was corrected during cross-review: the selected publication
context must become an actual native draft association, not unfulfilled future
intent left in the file.

## The two complete engines

| Concern | Generated body (Astra) | Keyed live blocks (g) |
| --- | --- | --- |
| Desired structural edits | Materialize the entire supported target graph; no split/join/move inference | Match keys/associations; create new, update retained, detach removed and normalize order |
| Text | Fresh valid Yrs state on new entities | Edit loaded Yrs on retained entities, fresh state on new entities |
| Activation | Stage disconnected content, verify, switch existing entry, verify active result | Mutate live graph, then verify; partial results can be visible |
| Preserved identity | Local document/source keys and native token/root/entry/public identity | Those plus retained native blocks/pages/text facts and some text continuity |
| Main additional machinery | Staging/explicit-root reads, generation binding updates and activation recovery | Existing-state text editing, per-kind scalar cleanup, shared occurrence and type-transition policies |
| Remote residue | Old whole bodies remain detached after meaningful body saves | Detached objects primarily arise from actual removals/replacements |

Neither is free or proven smaller in lines of code. The recommendation is about
the **shape of the reasoning**: construction from desired source gives insertions,
splits, joins, ordering and type changes the same path. Staging/recovery is real
work, but avoids growing an existing-graph transition policy per content family.
This fits the stated coarse-effect tolerance better than optimizing preservation
first. If native body anchors matter materially, the complete keyed engine is a
credible alternative; it must retain the full drafting acceptance boundary.

## Identity and current-data effects to accept deliberately

Generation keeps the same native draft and eventual public document. A meaningful
body save replaces its internal native page/block/text objects. Local source keys
remain stable and their compact native association rows are updated after verified
activation. An unchanged save creates no generation.

External links/comments tied to old body objects do not automatically follow the
new body. There is no complete incoming-reference discovery promise. Native editor
comments reachable through `block/comment` are outside the first writable profile
and must not disappear behind an apparently successful supported save.

Known inactive `block/text` on currently image/code blocks is different from
unknown active meaning. Under explicitly selected generated-body ownership, its
old payload remains on detached old entities; current code/images/prose are
recreated from represented content. Switching the new code/image back to text
will not resurrect the old inactive prose. This is an explicit continuity trade,
not a blanket blocker on ordinary editing of the imported corpus. The checkout
receipt does not establish a per-cause count for its initial 12 failures.

Newly created sources can declare generated-body ownership. Existing imported
documents need an explicit first adoption of that policy (the proposed existing
`--recreate-body` selector can serve); later saves use the recorded policy. There
is no need for a new per-field confirmation ceremony or two parallel engines.

## Shared corrections that must survive either selection

- First creation resolves absent/null/URI publication intent through existing
  config precedence. Use `/new` for standalone and a source-backed publication
  association path for publication drafts; direct `createPublicationDraft` is the
  recommended adapter. Never silently substitute standalone creation.
- Source investigation found a later `moveLeafletToPublication` path too; g's
  assertion that association can only ever be created at initial mint is too strong.
  This does not change the requirement to realize context in the first push.
- Apply metadata through its durable native storage. Client-local optimistic
  extras do not prove saved title/tags/date; pure standalone tags lack a verified
  editor storage route. Keep unbound omission distinct from explicit null/defaults.
- A local UUID must survive binding; do not require it to equal import's
  token-derived initial UUID. New source nodes need no fake confirmed remote rows.
- `createEntity` takes an array at the RPC boundary. Use native append helpers
  such as `addLastBlock` where verified, rather than porting a fractional optimizer.
- Absence from active rooted pull **does not prove creation never landed**.
  Partially created objects can be detached; requests can remain in flight. Use
  known-ID explicit reads plus acknowledgments/current state, or report uncertainty.
- A new homepage candidate is not proof of attribution to an uncertain mint.
  Exact encoded request replay differs from compiling a new stale attempt.
- Both independent source review and synthetic probes remain distinct from the
  deployed first-create, write/read-back and two-save proof.

## Road ahead: one delivery, internal implementation batches

The proposed batching keeps agents within useful working sessions without calling
a partial mechanism the product:

1. **g — source authorship and preparation:** `author-create` + `author-keys`,
   coordinated with the bounded profile work. Optional coherent bindings/receipts,
   presence-aware intent, stable local IDs, source-span key insertion, valid desired
   nodes ahead of confirmed associations, and flat-list representation.
2. **Sol high — actual native binding/write boundary:** `editor-bind` and essential
   uncertainty handling. Standalone and publication mint adapters, auth/target
   checks, token persistence, correct mutation shapes and explicit-root/ack reads.
3. **g — complete body save:** `editor-structure` plus fresh-text materialization
   from `editor-text`. One emitter, native ordering helpers, staged verification,
   entry activation, proper metadata destination and protected source writes.
4. **Sol high — joined product proof:** `editor-conflicts` + `editor-recovery` core
   cases, guarded pull/receipt settlement and the full first/second/unchanged save
   scenarios. Essential uncertainty handling is built throughout, not bolted on
   after unguarded writes ship.

Only the complete drafting loop earns the delivery claim. These are proposed
batches, not implementation dispatches. Exact ticket scopes/dependencies should
be reconciled to the selected engine before starting; in particular, the old
existing-state-only text ticket wording must not force a second engine.
