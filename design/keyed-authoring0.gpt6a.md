---
type: Design
title: "Keyed drafting: complete documents, simple native updates"
description: "A complete local-drafting and native-push implementation that retains keyed blocks, uses ordinary Markdown and Yrs, and grows through small content mappings rather than whole-body replacement or structural optimization."
resource: /design/keyed-authoring0.gpt6a.md
status: proposed
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Operator rejected both the text-only milestone and generated-body default; commissioned this concrete medium-sized design building on durable keys and readable Markdown"
  - resource: /design/drafting-push-brief0.gpt6a.md
    title: "Required complete first/second-save user stories"
  - resource: /design/drafting-push0.glm53m.md
    title: "Complete keyed operation proposal and cross-review; publication, ordering and identity corrections reused, rooted-absence recovery overclaim excluded"
  - resource: /design/drafting-push0.gpt6a.md
    title: "Verified first-binding/metadata/list/operation shapes and cross-review evidence; whole-body generation recommendation rejected"
  - resource: /crates/document/model/src/lib.rs
    title: "Inspected Leafbud model, mandatory binding/sync and current scoped editor projection"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Inspected observation, identity checks and shared status/pull classification"
  - resource: /crates/leaflet/editor/src/text.rs
    title: "Inspected production Yrs rich-text decoder and supported marks/breaks"
  - resource: /design/markdown-roundtrip1.glm53m.md
    title: "Implemented readable Markdown, exact local fallbacks and source-preserving normalization"
  - resource: /design/human-diff0.gpt61s.md
    title: "Implemented external human comparison, independent of executable changes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "Re-read createEntity array, addLastBlock, moveBlock and persisted metadata writes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/createPublicationDraft.ts
    title: "Re-read authenticated publication draft creation, association insertion and contributor defaults"
---

# Keyed drafting: complete documents, simple native updates

The [case-by-case flow supplement](keyed-edit-flows0.gpt6a.md) traces new, changed,
dropped and reordered blocks through native identities, proposed writes and
verification. It also gives ordering alternatives and a receiving-session map.

## 1. The implementation we are building

**Keep the keyed document architecture and complete its write side.** The source
file describes the desired document. Its keys locate existing native objects;
new keys create new objects. A small reconciliation pass creates, updates,
places and detaches those objects. Existing pages and blocks survive ordinary
saves. Markdown stays readable, and Yrs handles native rich-text state.

The useful delivery is the whole loop: **create locally → write → push → add,
edit, split, join, delete and reorder → push again → pull → unchanged push**.
Inserting a paragraph or heading is ordinary input, not an unsupported structural
special case. This design replaces the rejected generated-body recommendation
and the earlier existing-text-only milestone.

We are choosing stable block identity without choosing an optimal-edit engine.
There is no prose-similarity matcher, minimum-move algorithm or second document
model. Coarser work happens *inside the affected object* or in a simple ordering
pass, rather than replacing the document's entire native body.

## 2. What writing and saving look like

`leafbud create essay.mdx --title "Garden notes"` makes an unbound local draft.
Ordinary Markdown is an implicit main page; explicit `Page` components remain
available for multi-page documents. Publication omission inherits configuration;
explicit null means standalone. Neither requires remote identifiers to write.

On `push essay.mdx --yes`, missing keys are inserted once, a native draft is
created if needed, and the complete supported document is saved. `tidy --keys`
performs the same preparation locally. Preview/check never insert keys or mint.

Suppose the first save contains keyed blocks **A, B, C**. The author inserts a
heading **H** before A, splits B into **B and D**, removes C, and moves A to the end:

```text
before: A B C
after:  H B D A
```

The next push creates H and D, edits B in its existing entity, detaches C's
placement, and puts the retained/new placements in that order. A keeps its entity
and text state. The document's token, root, entry and main page stay the same.
Another identical push writes nothing. Pull returns the same source identities.

Splitting retains the original key on one piece and assigns another to the new
piece. Joining retains one key and retires the other. Moving a block moves its
marker with it. Duplicated/orphan markers are diagnosed; an explicit key rename
updates source references and its binding together. We do not infer identity from
similar words or require authors to edit native UUID tables.

## 3. Make the source model capable of authorship

Extend the existing `Document`, rather than adding a parallel planning AST:

- **Unbound is valid:** `binding` and `sync` are optional coherent groups. Keep the
  local document UUID when binding; remove the current assumption that every local
  ID must equal import's token-derived UUID. Remote binding checks remain strict.
- **Desired content may lead confirmed state:** a source key with no node row is
  new; a confirmed row absent from the body is retired; a key present in both is
  retained. Existing confirmed rows stay until their actual changes are verified.
  The parser's current exact body-key/node-table equality must become a save-result
  check, not a restriction preventing new paragraphs.
- **Missing keys are preparation work:** parser spans identify assignable blocks
  and list items. Insert `b-<uuid>` markers while preserving surrounding bytes,
  reparse for equivalence, and persist before remote effects. No per-character IDs.
- **Pending is compact:** planned new entity/membership IDs and attempt identifiers
  live in pending state until read-back promotes them. Never manufacture confirmed
  fact IDs, persist request bodies, or save a second copy of the document.

Keep source validation, prepared-input validation and confirmed-result validation
as ordinary functions over the same model. Existing receipt profiles retain their
meaning; new desired-node cases get explicit canonicalization fixtures. A failed
save or status observation cannot advance agreement merely to make hashes match.

## 4. One small reconciliation pass

Compare source keys and their associations with the fresh native observation.
Use equality of supported content to skip work; the external human diff is not
an input to mutation generation.

| Source change | Native action | Continuity |
| --- | --- | --- |
| New paragraph, heading, list item or other supported block | Create entity and owned facts, initialize fresh Yrs text where appropriate, attach membership | New identity for genuinely new content |
| Edit existing rich text | Edit its loaded Yrs document; assert one complete edited-state payload | Same block and text fact |
| Change heading level, code, image fields or supported block kind | Set the target kind's owned facts; clear incompatible owned scalars | Same block; no unrelated-field sweep |
| Move or reorder | Update existing membership placement through native helpers | Same block and membership |
| Remove a block | Retract the selected membership | Entity retained detached; no recursive deletion or asset cleanup |
| Add/remove a named page | Create or detach its identified relationships; validate remaining references | Existing other pages and targets retained |
| Change title/description/tags | Write through the actual metadata storage for this binding | Body untouched when only metadata changes |

**Ordering can be deliberately crude.** When a linear page's order differs, walk
its desired uniquely placed children and move each to the end in order. The native
`moveBlock` retains membership IDs and computes positions. New children use
`addLastBlock`; `createEntity` takes an array at the RPC boundary. No fractional
index port or minimum-move search is needed. Skip the pass when order already agrees.
Repeated references to a named page keep distinct link-block occurrences. A genuinely
ambiguous repeated placement of the same block needs exact-fact handling or a clear
diagnostic, never a guessed first match through the entity-addressed move helper.

First creation uses the same block builder and placement pass. Reuse the native
seed's main page and entry; detach its verified empty seed block after connecting
the authored content. It is not a separate generation/switch engine.

## 5. Yrs editing: modest policy, full supported text

Unchanged rich meaning produces no text write, regardless of serialization or XML
segmentation. For changed text, load the current complete Yrs state using the
verified UTF-16 editing convention; keep its paragraph, text fact and other roots.

For compatible text children, use a common-prefix/common-suffix splice and apply
desired supported marks, clearing removed marks. A formatting-only change need not
replace characters. This is a small helper with a native y-prosemirror precedent,
not an LCS engine. When inline topology changes—for example adding hard breaks—
rebuild the **affected paragraph interior** inside that loaded document. Keep the
outer native block and fact. This supported branch is part of the delivery, not a
reason to reject ordinary line breaks or shrink back to one-text-child editing.

Locally decode the result and require desired meaning, then send one complete
loaded-and-edited state per changed fact. Fresh independent Yrs state is used only
on genuinely new text entities. Both paths reuse one mark/inline encoder.

Coarse interior rebuilding trades some character/inline-anchor continuity; it
does not churn other blocks or the entire page. Removed children can hide unseen
concurrent edits, so read-back equality proves achieved content, not universal
concurrent-intent preservation. Unknown active marks/nodes remain unsupported.
Unchanged image/code blocks retain their known inactive text facts; editing their
active fields does not overwrite those facts. Supported type transitions update
owned scalars and record retained text as active/inactive as the native kind requires.

## 6. Native first binding and metadata are real parts of push

Resolve first-push target from explicit input, unbound author intent and existing
workspace/global defaults. Verify authentication before minting. Use the inspected
standalone `/new` route for standalone intent and `createPublicationDraft` for a
publication target. Publication creation needs bounded current Next-action discovery
and request/response decoding; proving that adapter is part of delivery. A browser
click or silently standalone draft does not fulfill configured publication intent.

Persist a returned token immediately, even if later linkage/content work fails.
Read and verify root, entity set, writable rights, entry, seed and actual association.
Keep the local UUID. A failure after mint is a partially bound attempt, never an
excuse to create a second draft automatically. After binding, defaults cannot retarget it.

Use `updateLeafletMetadata` for standalone title/description and
`updatePublicationDraft` for associated title/description/tags. Verify persisted
rows through reads, not browser-local optimistic extras. Pure standalone tags
have no established editor storage: diagnose that field before effects rather than
claim it was saved. Preserve actual contributor defaults and keep publication dates
as separate author/release intent. An authored title is independent of the first
paragraph, so inserting a new first heading cannot silently rename the document.

## 7. Save, observe, settle

The workflow is a direct composition of the existing libraries:

```text
parse and prepare keys → acquire current native state → classify drift
  → validate the entire desired change → prepare native writes and effects
  → persist compact attempt → execute → read back → settle bindings/receipts
```

Prepare the whole supported change before sending it. The transient result is
native mutation arguments, target IDs and a few effect labels—not another document
or generic plan language. The summary can say “add a heading and two paragraphs;
edit one block; remove one placement; reorder this page; save without publishing.”

Create prerequisites before attaching them; retain explicit IDs for new entities
and relationships. Query current facts/known IDs to complete confirmed node rows.
Verify the entire desired supported document, intended associations and protected
scope. Native mutations can be acknowledged yet skipped; HTTP success is insufficient.
Partial writes may be visible, and should be reported as partial rather than hidden.

Use scoped drift evidence and explicit take-local for conflicting remote changes.
Recheck input/current state before effects and guard source bytes across awaits.
Only verified correspondence advances receipts; naming/release state is independent.

Retry the identical encoded request in-process. After interruption, use attempt
acknowledgments, current content and explicit reads of known created IDs to resolve
it before compiling another write. Rooted-pull absence does not prove noncreation,
and a homepage candidate does not identify a lost mint. Stopping with a precise
uncertain result is enough initially; autonomous repair is not required. Pending
state contains identifiers/hashes, never binary text history or a serialized batch.

## 8. Coverage, growth and delivery

The first writable profile includes paragraphs, headings, supported inline marks/
links/breaks, single-paragraph quotes, code, rules, flat bullet/ordered lists,
remote-URL images and linear named pages/links. Flat list items are keyed text
blocks with native list facts; list wrappers need no remote identity. Nested/task
lists, canvas, uploads and interactive objects extend the relevant mapping later.
Unsupported content gets a precise diagnostic before effects, not a truncated save.

Growth follows the existing eight crates: model for meaning, MDX for parsing/key
preparation, editor for content-to-native writes, web for transport, workflow for
coordination. A better text splice replaces one helper. Another supported block
adds its model/parser/observe/write mapping. A cheaper ordering strategy replaces
the ordering pass. None requires replacing source identity, receipts or the UI diff.

Implement through these checkpoints as **one authoring delivery**:

1. **Source and identity — g:** `author-create` + `author-keys`; valid unbound and
   structurally edited documents, automatic IDs, flat-list/source mappings and
   source-preserving preparation. Existing imported files remain usable.
2. **Binding and native transport — Sol high:** `editor-bind` plus essential
   recovery; both mint contexts, metadata destinations, exact mutation/ack/read
   interfaces and early token/attempt persistence.
3. **Complete reconciliation — g:** `editor-text` + `editor-structure`; new and
   retained blocks, all supported text topology, scalar/type changes, detachment
   and simple order normalization. No whole-body replacement default.
4. **Joined proof — Sol high:** `editor-conflicts` + `editor-recovery`; finish
   uncertainty handling and exercise the complete native first/second-save loop.

Acceptance must demonstrate a new local file, browser-free first push under the
configured publication, then a combined revision with an inserted first heading,
new paragraph, split/join, formatting/break change, list edit, removal and reorder.
Existing retained block/entity IDs must survive; new blocks must acquire real
bindings. Pull must agree; repeat push must be a no-op. Include an imported code/
image block with inactive text, local edit during network work, skipped mutation,
partial creation and uncertain retry. Synthetic fixtures precede an authorized
deployed proof; none of the current source-mapped write paths is claimed live-tested.

The unit of usefulness is that complete loop. The internal checkpoints organize
implementation; they do not redefine ordinary drafting out of scope.

Evidence and receiving context: [complete-workflow brief](drafting-push-brief0.gpt6a.md),
[keyed operation research](drafting-push0.glm53m.md),
[native creation/metadata and review findings](drafting-push0.gpt6a.md),
and [existing observation/receipt implementation](status-witnesses0.gpt61s.md).
