---
type: Design
title: "Keyed editing flows — create, change, drop and reorder"
description: "Case-by-case supplement and receiving-session handoff for native keyed authoring: identities, proposed mutations, Yrs editing, ordering options, read-back obligations and uncertain outcomes."
resource: /design/keyed-edit-flows0.gpt6a.md
status: proposed
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Operator requested concrete new/changed/dropped-block flows, reordering alternatives and a durable receiving-session seed"
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Connected these flows to the code-grounded build path, bounded fallbacks and persistent live watch proposal"
  - resource: /design/keyed-authoring0.gpt6a.md
    title: "Current complete-drafting design with stable keyed native objects"
  - resource: /crates/document/model/src/lib.rs
    title: "Current Document/State/Node shapes and scoped editor projection"
  - resource: /crates/leaflet/editor/src/lib.rs
    title: "Current page/block observation, occurrence-derived source keys and inactive-text association"
  - resource: /crates/leaflet/editor/src/text.rs
    title: "Current supported Yrs decode path"
  - resource: /crates/leaflet/web/src/pull.rs
    title: "Current full-snapshot reader with fixed read-only client group; raw acknowledgment data not yet a write-recovery API"
  - resource: /design/status-witnesses0.gpt61s.md
    title: "Implemented scoped receipts, identity correspondence and conservative legacy handling"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Read executed Yrs/Yjs editing, Unicode, marks, no-op, delete-only, merge and last-write-cache results"
  - resource: /design/native-edit0.gpt6a.md
    title: "Prior-wave runtime probe accounts: retained-text versus removed-child visibility, full-state/delta behavior and stale recompilation"
  - resource: /design/drafting-push0.gpt6a.md
    title: "Native creation/metadata/list research and rooted-visibility corrections; generated-body recommendation is not adopted"
  - resource: /design/drafting-push0.glm53m.md
    title: "Keyed mutation analysis and cross-review; absence-as-noncreation claim is not adopted"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "Re-read createEntity, assertFact, retractFact, addLastBlock and moveBlock argument shapes and behavior"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cachedServerMutationContext.ts
    title: "Re-read fact ID selection, cardinality-one reuse, permission skips, buffered writes and Yjs merge"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/push.ts
    title: "Re-read v1 request envelope, client mutation IDs, acknowledgment and skipped-mutation behavior"
---

# Keyed editing flows

## Read this first

This supplements [Keyed drafting](keyed-authoring0.gpt6a.md). It describes how to
implement its write path, not a capability already shipped. The delivery remains
complete drafting: new local documents and native first binding, new paragraphs
and headings, text/formatting changes, splits/joins, removals and ordering.

The central rule is **reuse the object named by a confirmed association; create
only genuinely new objects; remove placements deliberately**. We edit a changed
block's loaded CRDT rather than replace the document with a fresh body. Reordering
uses the outer fact graph, not Yrs text updates.

The [code-grounded build path](keyed-build-path0.gpt6a.md) maps these cases to
concrete module/crate boundaries and implementation batches, makes bulk fallbacks
explicit, and adds the operator's persistent live watch/sync direction. It also
records that local files are rebuildable and the format can be revised.

Evidence has different standing:

| Ground | Standing |
| --- | --- |
| Source parsing/rendering, native reads, scoped status and guarded pull | Implemented; receiving code linked below |
| Editing existing Yrs and consuming its updates with real Yjs | Executed offline in experiment 05 and later probes |
| Native mutation shapes and server processing | Source-inspected at Leaflet `600c7b5f…`; not a deployed-write proof |
| The composed create/change/drop/reorder saves below | Proposed implementation and acceptance cases |

Do not read the prior generated-body proposal or text-only checkpoint as current
direction. They remain useful sources for specific protocol findings, not the
product boundary of this supplement.

## 1. The coordinates each case uses

Consider one bound linear page containing A, B and C:

```text
source key     native entity     placement fact     rich-text fact
A              E_A               M_A                T_A
B              E_B               M_B                T_B
C              E_C               M_C                T_C

native root R --entry fact F--> page P
page P --card/block M_A, M_B, M_C--> E_A, E_B, E_C
E_B --block/text T_B--> one Yjs document
```

These are symbolic labels, not literal wire UUIDs. The source key is an authoring
identity. The entity is the native block. The placement is **one occurrence** of
that block in a parent. The text fact contains that block's CRDT state. A title or
filename does not select any of these identities.

The existing `nodes` table records confirmed associations, not a saved old body.
For planning we have three inputs: the desired parsed source, confirmed locators,
and a fresh native observation containing both typed meaning and raw text/facts.
An authored block may be new without a node row; a confirmed row may remain while
its source block has been removed. Today's strict parser rejects those states;
supporting them is required implementation work, not an optional convenience.

**Baseline for the examples:** local changes are allowed by the scoped agreement
policy, the selected binding/rights are valid, and the full desired document is
representable. Remote drift or unsupported input is resolved before effects.
`--yes` executes; it does not silently resolve a conflict. The same case logic
applies after an explicit take-local decision against a fresh observation.

## 2. The common save envelope

Every case uses this sequence:

1. Read source bytes, parse, validate references and determine missing key edits.
   Executed preparation inserts keys atomically and makes those prepared bytes the
   guarded input for this attempt. A preview leaves the source alone.
2. Obtain/verify the binding, current graph and current text states. Retain the raw
   observation in memory; a typed Markdown projection cannot reconstruct the CRDT.
3. Check the complete desired change and choose native mutations. No supported
   prefix is sent before discovering an unsupported remainder.
4. Record compact pending intent/attempt/target identifiers before native effects.
   Exact request bytes are retained in memory, not journaled as another document.
5. Execute ordered native mutations. New objects precede relationships that need
   them; placement changes refer to actual current parents and intended targets.
6. Fetch again and check desired meaning, associations/order and untouched scope.
   Acknowledgment alone is not achievement. A partially applied batch remains partial.
7. If verified and source bytes still match, promote confirmed rows, remove settled
   retired rows and update only the relevant receipts. Clear settled pending state.

This is one composition over the existing model, not a second semantic change AST.
Small functions can prepare block facts, text updates and placement arguments;
their actual choices also supply short human effect summaries.

## 3. New block — insert a paragraph or heading

**Example:** desired order becomes `[A, X, B, C]`, with a new paragraph X.

1. The parser sees an unkeyed paragraph. Preparation inserts one durable key X.
   It has no confirmed native row. Existing A/B/C associations remain unchanged.
2. Allocate a new entity `E_X` and placement `M_X` and record them as planned IDs.
   For its initial text, create a **new Yrs document**, with the supported
   `prosemirror` paragraph/text/break structure and desired marks.
3. Construct the native creation/fact arguments. Illustrative mutation arguments:

   ```text
   createEntity([{entityID: E_X, permission_set: S}])
   assertFact({entity: E_X, attribute: "block/type",
               data: {type: "block-type-union", value: "text"}})
   assertFact({entity: E_X, attribute: "block/text",
               data: {type: "text", value: fresh_initial_v1}})
   addLastBlock({parent: P, factID: M_X, entity: E_X})
   ```

   This is the native mutation **argument** shape, not a complete HTTP request.
   Fresh text is safe here because E_X is genuinely new. Heading creation uses the
   heading kind plus its owned level fact; flat list items add their list facts.
4. Appending yields `[A,B,C,X]` temporarily. The selected ordering pass establishes
   `[A,X,B,C]`. We need not invent a fractional position on the client.
5. Read-back must find X with exactly its intended type/text, one intended M_X
   placement and the requested order. Confirm its actual text fact ID `T_X`.
   Cardinality-one `assertFact` accepts an optional caller ID, but reuses an
   existing fact's ID if one exists. Either allocate a planned T_X or learn it
   from read-back; only the observed result becomes a confirmed node row.
6. Persist `X → E_X/M_X/T_X` after verification. Repeating this save must reuse it
   and emit no creation or text write.

Match read-back objects against existing **and pending** native IDs to restore
the author's source labels. The import observer currently derives labels from
native IDs; those temporary observed labels must not rename X or replace the
local document UUID during settlement.

**Insertion before the first block** follows exactly this case. Document title is
independent metadata; the observer must not derive a different saved title merely
because the author inserted a new first heading.

**Partial creation:** E_X and text may exist while attachment fails. They are then
absent from the active rooted pull. Use pending IDs and explicit native root reads
to inspect them; absence from the active body is not proof they never existed.
Do not mint X again or independently re-encode initial text on the same E_X while
an earlier attempt remains unresolved.

## 4. Changed block — edit the same object

**Example:** B changes from `Red boats drift.` to `Blue boats drift.`

1. X is not involved: B still maps to E_B/M_B/T_B. Verify those associations in
   the current observation and compare supported inline meaning.
2. Load T_B's current binary update into Yrs. Reject unresolved causal state. Use
   UTF-16 editing offsets, while choosing string boundaries on valid Unicode scalars.
3. For a compatible text child, retain common prefix/suffix characters. Here the
   suffix ` boats drift.` survives; replace the initial word once. Apply desired
   marks and clear removed supported marks. A mark-only change retains characters.
4. Decode the edited in-memory result with the production reader and require the
   desired rich meaning before sending anything.
5. Encode **one complete loaded-and-edited state** for T_B and submit one
   `assertFact` for `(E_B, "block/text")`. The native cardinality-one rule retains
   the existing T_B. Server merging combines this with existing state.
6. Read-back verifies B's meaning, E_B/M_B/T_B continuity and the unchanged other
   blocks. Source/table IDs do not churn simply because text changed.

The payload is complete state for **one existing block's Yjs document**, not a new
independent document and not a regenerated Leaflet body. A single transaction delta
is another valid encoding with its prerequisites, but full edited state is the
initial native-shaped default. Never send a chain of prerequisite-dependent deltas
to the server's same-client last-write cache for one fact.

### Breaks, formatting and kinds are part of normal editing

- **Hard-break/topology change:** retain the native block, text fact and paragraph;
  rebuild that affected paragraph's children when straightforward child retention
  does not fit. This is the bounded coarse branch, not a first-slice refusal of
  ordinary breaks. Other blocks remain untouched. Removed children can hide unseen
  concurrent insertions, so read-back is not proof of universal intent preservation.
- **Paragraph becomes heading:** retain E_B/M_B/T_B, write `block/type` and level;
  write text only if meaning changed. Reverting to paragraph removes incompatible
  owned level/list scalars according to the chosen mapping.
- **Code or image fields change:** write the respective scalar/resource facts.
  Code text is not Yjs prose. Leave its known inactive rich-text fact untouched;
  read-back maintains the active/inactive locator distinction.
- **Shared entity:** two source occurrences addressing E_B must request one
  consistent value for its shared text. An occurrence-specific divergent edit
  requires an explicit fork/new block, not two competing writes to T_B.

Unknown active semantics are diagnosed before writes. Unrelated state carried in
the loaded Yrs document is not erased merely to make the desired paragraph simpler.

## 5. Dropped block — remove its placement, not its entity

**Example:** C is removed, so desired order is `[A,B]`.

1. C's confirmed row remains available for planning even though C is absent from
   the source. Resolve M_C and verify it is the owned occurrence under P.
2. Check that the omission is coherent: no orphan source Key, unresolved local
   page target or contradictory shared-object intent. A remote-only new object
   is not automatically a locally requested deletion; use the drift/ownership gate.
3. Send `retractFact({factID: M_C})`. Do not send `removeBlock`, `deleteEntity`,
   text replacement or asset cleanup to represent this ordinary source omission.
4. Verify the selected membership is absent and P's remaining order/content is
   correct. E_C/T_C can remain stored, or reachable through a different reference;
   that is not a failed deletion of this placement.
5. Remove C's confirmed source row only after the removal is settled. Unchanged
   next push must not try to retract it again. Any retained local occurrence still
   pointing at E_C keeps its own mapping.

For a dropped named page, validate its local references first, then detach the
owned links/placements. Do not recursively destroy its graph to implement omission.
The seeded blank block in a newly minted draft uses the same membership-detachment
primitive, but only after verifying that it is the expected empty seed.

## 6. Reordering — choose a small policy, preserve the same objects

Membership facts carry fractional positions. The existing observer orders them by
`(position, fact ID)`. Native `moveBlock` changes a membership's parent/position in
place and computes its position; it does not rewrite the block's text.

There are three reasonable strategies:

| Option | Mechanism | Tradeoff |
| --- | --- | --- |
| **A. Normalize a changed page's order** | Skip if order already agrees; otherwise move each desired child to `end`, in desired order | Smallest planning policy; may write every membership position on that page |
| **B. Fix the desired prefix** | Walk desired order left-to-right; move an out-of-place child to `first` or `after` the already-placed predecessor; simulate the resulting order locally | Still simple, usually fewer writes; needs a mutable order list, not a minimum-edit algorithm |
| **C. Write explicit positions by membership ID** | Compute positions with a verified compatible routine; assert selected ordered-reference facts | Addresses repeated/ambiguous occurrences precisely, but adds position-generation/validation responsibility |

**Start with A**, as proposed in the main design: low volume makes extra position
writes acceptable, while all native block/text/membership identities are retained.
B is a straightforward alternative if its small simulation is clearer in the
receiving implementation. Neither needs LCS or a globally minimum move sequence.
Keep this choice behind one placement-preparation function so it can change later.

Worked example, current `[A,B,C,D]`, desired `[C,A,D,B]`:

```text
A: move C to end → A B D C
   move A to end → B D C A
   move D to end → B C A D
   move B to end → C A D B

B: move C first  → C A B D
   A already next
   move D after A → C A D B
   B already next
```

Both preserve all four objects and text facts. A single request may carry these
ordered native mutations; source inspection says buffered reads see prior writes.
Read-back, not the simulated list or acknowledgment, proves deployed order.

**Occurrence constraint:** native `moveBlock` locates its membership by block
entity within `oldParent`, taking the first match. A/B therefore require that
target to be unambiguous. Distinct PageLink blocks sharing a page target are fine;
repeated placements of the *same block entity in one parent* need C or an explicit
unsupported-case diagnostic. Do not silently move whichever occurrence is found.
Cross-page movement also verifies the actual old parent and destination existence.

New blocks can be appended first, dropped memberships detached, then desired order
established. That intermediate order is not the final result being promised. A
failed reorder reports partial state; it does not trigger a whole-page replacement.

## 7. One combined ordinary revision

Start with `[A,B,C]`. The user inserts heading H before A, splits B into B and D,
removes C, and moves A last. Desired order is `[H,B,D,A]`.

| Work | Decision and expected result |
| --- | --- |
| Preparation | Assign H/D once. B retains its key on the retained split piece. C's old association stays available until detach is verified. |
| H/D | Create new entities/facts and fresh Yrs text; attach their memberships. |
| B | Update the existing text state to its retained piece; keep E_B/M_B/T_B. |
| C | Detach M_C, retain the old entity without global cleanup. |
| Order | Apply A or B above to obtain `[H,B,D,A]`; A's text is never rewritten. |
| Verify/settle | Confirm the whole desired content, retained/new associations and metadata; promote H/D rows, retire C's row, preserve A/B IDs and source spelling. |

Joining is the inverse source action: combine the words under one retained key,
remove the other marker/block and detach that other placement. It is not inferred
from text similarity. A subsequent pull must preserve the chosen source keys;
an unchanged push emits no mutations. This combined case is a required fixture,
not four separately claimed product deliveries.

## 8. First push is the same cases after establishing a binding

Before the new-block case can run for an unbound source, resolve omitted/null/URI
publication intent through config, authenticate, and create the correct native
draft. Standalone uses the inspected `/new` route; publication-associated creation
uses the native publication draft action. Keep this inside the delivery; a manual
browser seed is not a substitute for the required local-create/first-push story.

Persist the returned token promptly and verify root, rights, entity set, entry,
association and seed. Keep the locally minted document UUID. Map the source's main
page to that native main page; then create its authored blocks, detach the verified
empty seed, order, apply context-correct metadata and verify. A failure after mint
retains the binding/pending outcome, rather than pretending the file is unbound.

The [main design's binding/metadata section](keyed-authoring0.gpt6a.md#6-native-first-binding-and-metadata-are-real-parts-of-push)
owns publication defaults, standalone tags, contributors and independent titles.
Those rules are prerequisites for a truthful first-save/read-back result, not a
separate future release feature. Save still does not publish publicly.

## 9. Interruption and settlement: what can actually be known

| Observation | Permitted conclusion/action |
| --- | --- |
| Exact request still available in-process | Resend identical bytes/client/mutation IDs if retry policy allows; do not recompile and call it the same request |
| Mutation acknowledged, desired facts absent | Processed does not mean applied; diagnose skipped/partial state and inspect known IDs |
| New block absent from active pull | It may exist detached, or an earlier request may remain in flight; inspect explicit IDs/acknowledgments |
| Current desired meaning and intended associations verified; attempt resolved | Settle the relevant node rows and receipts under the local byte guard |
| Local file changed while network work ran | Preserve the newer source. Remote success may exist with an unsettled local receipt; report that accurately |
| Prior attempt unresolved after restart | Keep compact pending evidence and stop uncertain; do not independently encode the stale edit again |
| Mint response lost; a plausible homepage draft appears | Candidate for explicit recovery, not proof it belongs to this attempt; never choose by newest title or auto-remint |

Yrs client IDs and Replicache request client/group IDs are different identities.
The current `web::pull` uses a fixed read-only group; write recovery needs reads
for the attempt's actual group plus a typed view of mutation acknowledgments.
The raw response already retains evidence, but that recovery interface is not
implemented. Native request locking is not a caller-supplied expected-version CAS.

The pending format and exact canonical desired-node fingerprints remain concrete
implementation work. They must preserve existing receipt interpretation, carry
planned identities without claiming success, and avoid storing old bodies or
binary updates. Automatic repair is not a requirement for reporting uncertainty.

## 10. Receiving-session map and acceptance

| Home | Already present | Work this supplement asks it to own |
| --- | --- | --- |
| `document/model` | Current content, confirmed node rows, scoped editor projection | Valid unbound/new/retired node states and compact attempt data |
| `document/mdx` | Satteri reader/writer, Markdown normalization and byte spans | Authoring validation, one-time key insertion and confirmed-state patching |
| `leaflet/editor` | Typed graph observation and Yrs reader | Fresh block facts/text, loaded-text editing and native placement preparation |
| `leaflet/web` | Metadata/full-pull readers | Correct mint/push/explicit-ID/acknowledgment transport |
| `workflow` | Config, inventory, agreement, guarded pull and file writes | Whole desired-document validation, attempt orchestration and settlement |
| `cli` | Read/inspect/tidy/diff surfaces | Thin create/push/key-preparation commands and concise effect/outcome reporting |

Begin with the [current keyed design](keyed-authoring0.gpt6a.md), this flow account,
the receiving code, and [experiment 05's executable evidence](../experiments/05-yrs-conversion/REPORT.md).
Use [Astra's native-protocol cross-review](drafting-push0.gpt6a.md#12-authorized-cross-review-and-revised-stance)
and [g's keyed operation research](drafting-push0.glm53m.md#addendum--cross-review-of-the-astra-draft-and-revised-stance)
for details, with the corrections about rooted absence and confirmed identity above.

The first receiving session should pin down these choices in executable fixtures:

1. Source-only/new/retained/retired states; automatic keys; no mutation on preview.
2. New paragraph and inserted first heading; real confirmed IDs after read-back.
3. Text/marks/break changes on retained objects, plus code/image scalar changes.
4. Dropped membership with retained entity and independent remaining occurrence.
5. Both ordering examples; selected policy preserves native IDs and repeats cleanly.
6. The combined revision in §7, first native creation in both supported contexts,
   immediate pull and unchanged push.
7. Skipped mutation, detached partial creation, source race, exact replay and
   uncertain stale attempt. Include Unicode and native mark/topology edge cases.

Carry these across the implementation checkpoints in the main design. No source-
mapped native write is called deployment-verified until the actual first/second-save
loop passes. No narrow helper completion replaces the complete drafting goal.
