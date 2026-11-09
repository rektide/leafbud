---
type: Design
title: "Native editing with stable outer objects and deliberately modest text edits"
description: "Independent Astra proposal for Leafbud: retain existing keyed blocks, edit loaded Yrs state with a small splice policy, allow explicit coarse effects, and keep staging/recreation as a bounded alternative rather than a general structural optimizer."
resource: /design/native-edit0.gpt6a.md
status: proposal
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
updated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e43700700ffezpESdrhZMD98I1"
    title: "Independent stage-one design and probes, followed by authorized review of g's committed initial draft, wire/replay probe and separately committed revised stance; original argument preserved"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "User authority via parent commission: low-volume product, bounded complexity, independent Astra/g drafts then cross-review; separate external human diff; subsequent Leafbud rename steering"
  - resource: https://github.com/rektide/leafbud/blob/ff2f4a7b858e99625d3496e55aa97f32559bde0c/design/native-edit0.glm53m.md
    title: "Stage-two review target: g's initial independent draft, read through git show at this exact commit, not its concurrent cross-review addendum"
  - resource: /AGENTS.md
    title: "Current product-fit and collaboration instructions read before inquiry"
  - resource: /design/status-witnesses0.gpt61s.md
    title: "Delivered scoped observation/status and current implementation limits"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted commands, ownership, YAML effects, conflict and release boundaries"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Original recreation route and subsequent keyed bidirectional requirements, reassessed against current operator priorities"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Earlier fresh-subtree alternative, entry switching and stable publication identity"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Existing Yrs/Yjs edit, no-op, identity, concurrency and last-write-cache evidence"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Fresh text/fact generation and bounded staging model, not deployment evidence"
  - resource: /experiments/14-satteri-authoring-attributes/REPORT.md
    title: "Key/compiler equivalence and source adapter boundaries, not an editing engine"
  - resource: /Cargo.toml
    title: "Inspected production 0.8.0 dependencies at Leafbud revision 39814abe57645e88583e2ca1c7f5ba16ee46b4d3"
  - resource: /crates/document/model/src/lib.rs
    title: "Existing current Document, associations, scoped editor value and receipt types"
  - resource: /crates/document/mdx/src/lib.rs
    title: "Current parser requires draft binding and exact source-key/node-table coverage"
  - resource: /crates/leaflet/editor/src/lib.rs
    title: "Conservative raw witness, keyed native observation and loss-checking"
  - resource: /crates/leaflet/editor/src/text.rs
    title: "Current Yrs reader and supported inline/mark profile"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Endpoint classification, association-first label reconciliation and identity gates"
  - resource: /crates/workflow/src/import.rs
    title: "Import title derivation, supported extras, draft-only checks and raw witness assembly"
  - resource: /crates/leaflet/web/src/pull.rs
    title: "Existing null-cookie full-read transport; does not retain mutation acknowledgments as a typed recovery API"
  - resource: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/types/text.rs
    title: "Locally read Text insert/remove/format/apply_delta/diff implementations"
  - resource: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/types/xml.rs
    title: "Locally read existing XML child mutation APIs"
  - resource: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/transaction.rs
    title: "Locally read transaction delta versus full-state/state-vector encoding contracts"
  - resource: https://cdn.jsdelivr.net/npm/y-prosemirror@1.2.5/src/plugins/sync-plugin.js
    title: "Installed experiment-05 package source inspected locally: updateYText uses simpleDiff; updateYFragment is marked private/unstable"
  - resource: https://cdn.jsdelivr.net/npm/lib0@0.2.119/diff.js
    title: "Installed dependency source inspected locally: common-prefix/suffix single replacement, including surrogate boundary adjustment"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "Locally read moveBlock, assertFact, retractFact and destructive removeBlock behavior"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cachedServerMutationContext.ts
    title: "Locally read cardinality/author checks, per-client text merge, flush and deletion behavior"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/push.ts
    title: "Locally read request transaction and per-mutation skip/ack semantics; no expected-witness CAS"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/migrations/20260721000000_add_pull_data_cvr.sql
    title: "Locally read rooted reference closure and same-snapshot facts/mutation acknowledgments"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/getFactsForRoots.ts
    title: "Locally read explicit staged-root read route with depth three"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts
    title: "Locally verified poll entity ID becomes public poll record key"
---

# Recommendation

**Keep the keyed native blocks and edit their loaded Yrs text. Use one simple
replacement span per retained text node, not a general tree matcher. Permit
coarser inline replacement when necessary, with its effects stated.** Preserve
unchanged blocks exactly. Add straightforward native fact operations afterward;
whole-body staging/switch is a genuine alternative, but not the smallest next
step from today's imported, bound documents.

The incoming product name is **Leafbud**. Inspection began in Leafbud 0.8.0
at `39814abe57645e88583e2ca1c7f5ba16ee46b4d3`; historical package/profile spellings
below identify that inspected revision. Another session owns renaming and data
conversion. This proposal neither performs those changes nor infers their
completion. It owns only this document and isolated synthetic scratch.

This is **stage one**, independently written without the same-wave peer draft.
The parent must join the two committed drafts before commissioning cross-review.
Ticket: `leafbud-editor-plan-design`; this proposal does not select an
architecture on the user's behalf.

## 1. What controls a save

| Question | Controlling input and reason |
| --- | --- |
| What should the editor mean? | Parsed local MDX and editor-owned author fields, after an allowed local-only edit or explicit take-local choice. Current `Document` expresses this meaning; it is not historical evidence. |
| Which editor and objects may change? | Verified binding plus current remote facts and the source-to-native node associations. Filenames, prose similarity and source-label spelling do not choose a target. |
| Has either side changed since accepted correspondence? | Existing scoped local/remote witnesses. They establish direction/uncertainty, never reconstruct a base body or unseen local operations. |
| How should text be changed? | Actual current XML/Yrs topology plus the small editing policy below. MDX does not identify historical characters. |
| Did save succeed? | Fresh supported editor meaning, intended associations and unchanged-outside-scope checks, not HTTP success or a CRDT state vector alone. |
| What is publicly released? | Actual native publication linkage/output and separate release evidence. Push changes the working editor; it cannot certify public equality or delivery. |
| What should a person inspect? | The external Markdown diff and a short operation-effects summary. Neither controls executable mutation generation. |

Use the existing [scoped classification](status-witnesses0.gpt61s.md): clean and
converged endpoints need no content writes; local-only changes may proceed;
remote-only/both-changed require the established choice. Unknown/legacy evidence
must retain its uncertainty. `--yes` executes; `--take local` selects differing
current content; recreation selects identity effects. These are different choices.

Only writes supported by the selected slice may execute. A text-only first slice
must reject a simultaneous unsupported title, image or structure change before
writing any text, rather than save part of the desired document and call it done.
Local-only YAML remains outside that test. A title derived from the first block
is an existing import rule, not permission to rename the document when editing
that block: verification must catch that interaction or implement the native title
mapping in the same slice.

## 2. The useful identity budget

The goal is neither perfect continuity nor indiscriminate regeneration:

- **Document/public identity:** keep token, root, rights/entity set, selected entry
  and established publication linkage. A new body is not a new public document.
- **Block/page identity:** retain existing source associations and native entities
  when easy. This supports pull, page references and block-level links. Reordering
  known blocks does not require recreating their contents.
- **Placement identity:** a membership is an occurrence, distinct from its target
  entity. Keep its fact ID on a move. Repeated occurrences cannot be distinguished
  by target entity alone.
- **Character/XML identity:** keep untouched nodes, and retain prefix/suffix items
  within matched text nodes. A rewritten middle can lose surviving-but-interior
  character anchors; a coarse child replacement loses the replaced children's
  anchors. Do not promise comment/range continuity merely because the block ID stays.
- **Interactive identity:** poll entity/option meanings are externally consequential.
  The native publisher uses the poll entity ID as its record key. Reuse unchanged
  polls, including across a recreated surrounding page; never reinterpret existing
  votes through a new option meaning under the same identity.

This deliberately revises the older aspiration to maximize surviving text/XML
items. That aspiration need not force experiment 05's two-level LCS or a better
optimizer. It preserves the useful keyed editing goal in the
[handoff addendum](authoring-handoff0.gpt61s.md#keyed-bidirectional-addendum).

## 3. What existing tools actually solve

### Yrs is the editing and encoding engine

The production pin is Yrs **0.28.1**, Git
`e0b476b0ce613453c70d0bb477b955808aa79a07`, also the inspected archive HEAD and
experiment-05 pin. Its verified APIs supply:

- Load v1 state with `Update::decode_v1` / `apply_update`; reject unresolved causal
  state. Use `OffsetKind::Utf16` for editing positions. The current read-only
  decoder's default byte offsets are not an editing configuration to copy.
- Edit an existing `XmlTextRef` with `remove_range`, `insert_with_attributes`,
  `format` or `apply_delta`; edit XML children through `XmlFragment` methods.
- Encode a transaction's inserted/deleted items, or encode the full edited loaded
  document with `encode_state_as_update_v1(&StateVector::default())`.

`Text::diff` returns current formatted runs. `apply_delta` consumes edits supplied
by the caller. State-vector diff finds missing **CRDT structs/deletes between
replicas with history**. None translates arbitrary desired Markdown into edits,
infers which earlier remote text the author saw, or compares the outer fact graph.
Deletion-only changes can leave the state vector unchanged. Compare meaning for
no-op; do not use vector equality as the decision.

**Prefer one full edited-state payload per changed existing text fact for the
initial transport.** It is derived from the loaded document, not independently
encoded replacement text. This follows native full-state writes and avoids making
delta accumulation another protocol obligation. Payload size is not the priority.
Transaction deltas remain useful for tests and a later internal optimization.
The server merges existing text and keeps only the latest same-client payload per
entity/attribute in a batch; never submit a chain of dependent standalone deltas.
Do not recycle an old Yrs client ID to pretend new prose has old item identities.

### An existing editor uses a suitably modest text algorithm

Locally installed **y-prosemirror 1.2.5**, matching the archived Leaflet lock and
experiment-05 oracle, is informative: `updateYText` uses lib0's **single common-
prefix/common-suffix replacement**, then applies formatting ranges. Installed
lib0 **0.2.119** handles surrogate boundaries explicitly. This is strong precedent
for not implementing a minimum edit script. Rust can do the equivalent small
scalar-boundary calculation and count UTF-16 units for Yrs.

Using y-prosemirror itself is a real alternative: its exported `updateYFragment`
reconciles a ProseMirror tree into existing Yjs state. However, the implementation
labels it private/unstable, contains mapping/child heuristics, and requires a JS
runtime plus the ProseMirror representation/schema bridge. That expands this
Rust application's operational surface. Keep it as the independent test consumer
and a candidate if full ProseMirror coverage becomes the main difficulty.

No generic Rust text-diff dependency is present in the inspected production
lockfile. A Myers/LCS library could later replace the single-splice helper; it
would not solve marks, XML identity, association ownership or native transport.
Adding one now buys little. This is not a claim that no suitable library exists.

### Native fact operations are already available

Archived Leaflet `600c7b5f1f7b31b66fb626c293cc104312a12caf` supplies:

- `assertFact`: cardinality-one updates reuse the existing fact; cardinality-many
  updates require explicit IDs. Author/permission checks can silently skip writes.
- `moveBlock`: updates an existing membership fact's parent/position, preserving
  the fact ID and using Leaflet's own fractional-position generator. It chooses
  the occurrence by target entity, so it is appropriate only when unambiguous.
- `retractFact`: can detach one selected occurrence. `removeBlock` is much broader:
  entity deletion, some recursive deletion and image-cleanup side effects. It is
  not the ordinary local-delete primitive recommended here.
- `createEntity`, fresh facts and an existing `root/page` fact update provide the
  staging/switch route. `getFactsForRoots` reads explicit roots, capped at depth
  three; staged inventory must be queried explicitly rather than assumed visible.

The inspected push handler uses a database transaction and token-derived lock,
but catches/acknowledges individual failed mutations and has no expected-witness
CAS. This is useful native implementation evidence, not an application-level
all-or-nothing save guarantee or a verified deployment contract.

## 4. Recommended small editing policy

### First: existing bound text, unchanged outer topology

1. Parse local input; acquire current metadata plus full pull; check identity,
   rights, representation and scoped change permission. Keep the raw current
   snapshot in memory for text planning and verification.
2. Match existing source associations to current facts, not names or similar
   prose. If several occurrences share a text entity, require one consistent
   desired value for all of them. A divergent per-occurrence edit requires an
   explicit fork/new object, not whichever write happens last.
3. Equal supported inline meaning emits no text write, even when XML segmentation
   differs. Never normalize a no-op's native shape.
4. For changed text with the same simple child-kind sequence (XmlText/break),
   pair by position. In each retained XmlText, preserve common prefix/suffix
   characters and replace the middle once. Apply desired owned marks by ranges;
   mark-only edits retain characters. Do not equate a literal text newline with
   an XML hard break. Keep the paragraph and existing compatible children.
5. For changed child topology, use a **coarse paragraph-interior edit**: retain
   the paragraph, delete its observed children, insert the desired text/break
   children within the loaded Yrs document. No XML LCS, cross-node character
   relocation, or prose-based identity guessing. The explanation says which
   block's inline anchors are being replaced. An empty single-XmlText paragraph
   should keep that text node and delete its characters, as y-prosemirror does.
6. Read the locally edited Yrs document back through the existing decoder before
   sending it. Send one complete edited state per fact, then verify fresh remote
   meaning and associations. A second push must be a semantic no-op.

Step 5 is a **proposed ordinary-save policy for supported inline changes**, not
permission to recreate a native block/page silently. It needs parent/user
selection against the earlier stronger preservation language. If not selected,
ship the same-topology path first and make break/topology edits explicitly blocked
until that choice. Do not invent a configurable family of optimizer modes.

Unknown comments/marks/inline types are refused by today's decoder. Preserve that
boundary; a coarse edit is not a loophole for dropping unrepresented material.
An unchanged opaque root outside `prosemirror` can survive through the loaded Yrs
document; changing or claiming to interpret it is a separate capability.

### Then: straightforward structure, with coarse ordering acceptable

Keep the current key/entity/occurrence table. New source IDs get new bindings;
removed IDs mean selected occurrences are detached, not global entity destruction.
A manual label rename with its binding retained is not a remove/add. Split/join
policy can be simple: the retained key owns the retained block, other pieces get
new keys or are detached. No content-based identity recovery is required.

For ordinary single-placement linear blocks, native `moveBlock` is a small useful
win. On a changed page, moving every desired child to the end in desired order is
acceptable: it yields the requested order while preserving objects/memberships,
using the existing server ordering logic. Skip the whole sequence when order is
already right. This is intentionally not a minimal move algorithm. Repeated
occurrences need explicit fact-ID placement writes and a verified compatible
position routine, or remain outside this first structure slice.

Stage new blocks/resources before connecting them. Reuse unchanged pages and page
targets; a removed page definition still referenced by local content is invalid.
Detach only identified memberships; retain entities/assets for deliberate cleanup.
Do not call recursive `removeBlock` to implement a local omission. Shared objects
outside the selected body must not be mutated through one alias by accident.

Image URLs/locators can stay unchanged. Local asset upload needs its own resolved
resource path before facts reference it; no upload during preview. No automatic
blob pruning. Poll editing is outside today's model: when added, initially permit
unchanged stable polls and deliberately new polls, not definition changes that
reinterpret votes. Recreating their surrounding prose must preserve those stable
objects and option meanings, not manufacture a new poll from its displayed text.

## 5. Coarse alternatives, candidly compared

| Shape | What survives | Implementation/product tradeoff |
| --- | --- | --- |
| **Recommended keyed blocks + simple text splices/coarse interior** | Document, page, block, membership and text-fact identities; some XML/character continuity | Builds on current imports and Yrs reader. Small text policy, direct native fact writes. Partial save possible; topology replacement has explicit anchor/concurrency cost. |
| **Always rewrite all characters in existing compatible XmlText nodes** | Same outer identities and text-node containers; rewritten characters do not survive | Even smaller text policy; adequate for low-volume authoring. Useful fallback if splice/format code becomes disproportionate. More lost character anchors, but still avoids resetting containers. |
| **Fresh block, switch its existing membership** | Document/page and selected membership, but not old block/text identity | Avoids editing that block's old Yrs shape. Needs staging/read-back, rebinding, and handling incoming references. Other aliases may still show the old object. Better explicit `--recreate KEY` than normal text save. |
| **Fresh page/body generation, switch selected entry** | Token/root/public linkage and selected entry; stable polls can be exceptions | Simplifies heterogeneous body materialization and preserves old active body until switch. Loses ordinary page/block anchors and may leave old aliases pointing at old pages. Requires staging discovery/recovery, new binding rows and stable-object exceptions. Credible local-source-first product, not a free simplification. |

The [old recreation design](mdx-recreate0.gpt6a.md) remains attractive if the user
prefers “this entire body is generated; internal links/comments need not follow.”
Then make that ownership contract the default consciously, reuse experiment 13's
materialization approach, and omit the incremental engine rather than maintain
two equally elaborate engines. Keep the same Leaflet/public identity, current-
endpoint conflict choices and useful pull. No need to resurrect its older
recreation-only refusal of reverse conversion.

For either staging alternative: allocate generation/target IDs before effects,
create disconnected content, verify complete emitted inventory/text, recheck the
bound entry, switch exactly that existing fact, then verify active content. Old
content remains detached; no cleanup is implied. Reusing an existing page that
other people reference is different from switching an entry to a new page.

The full-pull SQL walks **rooted reference closure**, not the entire entity set.
Detached old/staged objects therefore need explicit reads; absence from active
pull is not proof of deletion. Conversely, current `editor::observe` refuses
every returned unrepresented fact and multiple root entries. Do not weaken that
to silently drop unrelated root/navigation content to make recreation pass.
A broader owned-entry projection is real additional work if required.

## 6. Concurrency and recovery: small, honest obligations

### Verified narrow replacement hazard

This session ran an isolated offline probe in
`.test-agent/native-edit-astra/`: Yjs 13.6.15 made `old` in one XmlText; pinned Yrs
loaded it and generated a transaction update replacing it with `new`. Yjs applied
the update twice and combined it with a concurrent insertion of `R` into the old
text, in both arrival orders.

| Yrs edit | Verified result |
| --- | --- |
| Delete old characters, insert new characters in the **same XmlText** | Paragraph and text node retained; duplicate update harmless; both orders converge to `Rnew`. |
| Delete the **XmlText child**, insert a new child under the same paragraph | Paragraph retained, text child replaced; both orders converge to exactly `new`; concurrent `R` is no longer visible. |

Commands: `cargo build --offline` then `node probe.mjs`; both completed, all
assertions passed. This is synthetic engine evidence, **not** a Leaflet server or
browser test. Scratch is not a shipped implementation or durable test suite.
Experiment 05 supplies the broader existing interoperability/no-op/marks evidence.

Consequently, **read-back equality proves achieved meaning, not absence of lost
concurrent intent**. Coarse subtree edits can conceal concurrent changes inside
removed containers. Fine text edits can instead yield a visibly divergent merge.
No strategy here supplies uninterrupted multiplayer protection; preflight narrows
the race but cannot close it. The explanation must not advertise verified equality
as proof every unseen web edit survived.

### Execution and interruption

- Re-read source bytes and remote witness immediately before effects. Changed
  evidence invalidates the prepared operation. Use current supported state and
  stop on unknown/ambiguous associations; do not bypass representation checks with
  `--take local` or `--recreate-body`.
- Preserve source keys and a compact **pending attempt marker before writing**.
  First text-only push needs only intent/preflight hashes, attempt/protocol IDs and
  touched native targets. Later staging adds generation/known target IDs. This
  requires a small owned-state extension: today's `State` has no pending field.
  No body, binary CRDT, serialized mutation batch or alternate source is persisted.
- Keep exact request bytes/IDs in memory during execution. A same-process transport
  retry may resend that exact request; never regenerate independent text updates
  and call that a replay. Yrs client IDs and Replicache request client IDs are
  different identities.
- After interruption, read actual current state before doing anything. If desired
  meaning/associations and protected scope are verified, settle. If the result is
  partial, divergent or unverifiable, preserve pending evidence and report it.
  A process restart has lost the exact payload: do not silently replan and send
  another update until the previous attempt is resolved. A matching old content
  hash does not prove a timed-out request cannot still arrive.
- Native pull exposes mutation acknowledgments; a small typed acknowledgment read
  can establish whether an attempt was processed, but **processed is not applied**.
  Acknowledged-skipped mutations require observed-state resolution, not blind
  replay of already acknowledged IDs. Unknown in-flight state stays uncertain.
- Verify full supported desired meaning, intended identities, and unchanged
  out-of-scope observations. Report convergence/divergence/partial/unknown as
  observed. Never repeatedly edit until the local file “wins.” Do not roll back
  with invented inverse edits that might delete newer work.
- Patch successful receipts only if the local source still matches the input for
  that attempt. A local race can leave a verified remote save with an unsettled
  receipt; it must not overwrite the newer file. Push never touches release/naming
  receipts merely because the editor converged.

These checks belong in the first remotely executable slice. Recovery can initially
stop and explain; automatic resumability is not required to make uncertainty safe.

## 7. Minimal architecture and explanation

Keep the accepted crate boundaries and current document model. Add ordinary
functions/modules, not a comparison framework or another planning document AST:

```text
MDX bytes -> existing Document
native get + full pull -> raw observation + existing decoded Document
                 | identity / witness / supported-scope decision
                 v
editor::prepare_save(desired, current_raw, bindings)
    -> native mutations + concise effects + required target IDs
                 | same-observation/source check; compact pending marker
                 v
web::push(request) -> transport result
fresh native read -> decode -> verify desired + preserved scope
                 v
workflow: settle receipts or report partial/diverged/uncertain
```

`prepare_save` is a conceptual seam, not a mandated public type/schema. It can
return a small transient batch whose effect annotations are made **where mutations
are chosen**. Verification reuses `Document` and raw observations; new confirmed
bindings can be applied to a clone of that same model for comparison. Do not build
a second semantic change tree just to print the operation.

Current `Observed` retains metadata/merged content but not the pulled raw snapshot;
the executing workflow must retain that input rather than decode twice or pretend
the merged typed view contains CRDT state. Current parse requires every source key
to have a confirmed node row. Existing-text edits fit it; insertion/deletion and
first binding need a deliberate desired-source/association validation extension.
Do not invent fake confirmed remote IDs to squeeze new nodes through today's
import-only invariants.

The user needs to know **destination, scope, identity effects, blockers and release
effect**, not every Yjs instruction. For example:

> Save to the bound working editor: edit 2 text blocks; keep their block IDs.
> `intro` keeps its text container; `notes` rebuilds its inline children, so
> character anchors there may no longer follow. One native block is moved.
> Public content is unchanged. Current editor was checked; intervening edits
> remain possible. Read-back will verify the resulting content.

For recreation:

> Replace the selected body under the same Leaflet/public identity. Ordinary
> page/block anchors will change; unchanged polls retain their objects. Old body
> entities are detached, not deleted. This save does not publish.

Count semantic effects, not remote operations. The external Markdown diff can be
invoked for prose inspection, but its output/exit status is never an edit program.
Preview is read-only and can state missing keys/assets as requirements. Executed
push assigns missing keys only when that capability exists; it does not run tidy
or rewrite prose as a side effect. Existing spans are enough for block diagnostics;
property-level source mapping and formatter improvements are not prerequisites.

## 8. Acceptance fixtures and next coherent slices

**First batch: `leafbud-editor-text` with the necessary part of
`leafbud-editor-conflicts`.** Recommend Sol high for this first write boundary:
the new judgment is native execution/uncertainty, not parser work. Deliver only
existing bound draft text; the current production workflow explicitly refuses
published editors. Add native writer transport, small pending marker, scoped
preflight and full read-back together. Preserve that lifecycle limit visibly
until native published binding support is commissioned.

Required fixtures, using synthetic data and the independent Yjs consumer:

1. Existing imported keyed text -> edit -> prepare -> apply -> read-back -> pull;
   same entity, membership, text fact, paragraph; second push no content write.
2. Mark-only change, removed mark/link, empty text, Unicode emoji/combining text,
   hard break versus literal newline, unchanged alternate XML segmentation.
   Check retained prefix/suffix items; do not assert optimal interior preservation.
3. Changed hard-break topology uses the selected coarse policy and reports its
   anchor effect. Include both concurrent cases above; equal output must not claim
   absence of lost intervening intent.
4. Unsupported inline/extra fields, mixed unsupported desired changes, missing or
   retargeted binding, author-protected facts and inconsistent shared-entity intent
   fail before writes. Include derived-title interaction with first-block edits.
5. Remote-only/both-changed, stale preflight, source race, ignored acknowledged
   write, lost response and restart-pending state. No automatic destructive replan;
   never mark failure/partial as synchronized. No release or unrelated file writes.

**Next: `leafbud-editor-recovery`, then the bounded part of
`leafbud-editor-structure`.** Recommend Sol high for completing observation-
based recovery; g is suitable for the subsequently specified native move/simple
fact slice. Exercise move-all-in-order, detach-only deletion, staging a new block,
explicit target reads, shared-page references and repeat/no duplicate behavior.
Keep creation, local assets, polls and body recreation separate capability additions
unless the chosen first structure profile actually requires them. This is a
proposed batch sequence; the parent owns ticket changes and commissioning.

Publication is a later joined operation: verify editor save first, invoke native
release deliberately, and verify actual public output/linkage separately. Neither
this design nor experiments 05/13/14 prove deployment writes or native publication.
An authorized throwaway-editor integration probe remains necessary before claiming
the complete live loop; no such writes were performed here.

## 9. Open choices and what would change the recommendation

- **Accept ordinary coarse inline rebuilding?** My lean is yes for the supported
  no-comment profile, with explicit effect reporting and the concurrency limitation
  above. If users value character annotations, keep simple splices but gate topology
  changes, then consider the existing y-prosemirror reconciler or a focused better
  matcher. Do not front-load that complexity today.
- **Is generated-body ownership preferable?** If internal anchors/web edits are
  explicitly disposable and heterogeneous content is the primary near-term need,
  I would switch the default recommendation to fresh body staging/switch. It is a
  viable product selection, especially for wholly new local documents. Its remote
  staging/recovery/association obligations still need implementing.
- **How often are entities shared or interactive?** Current associations permit
  repeated references; transport rights do not imply exclusive semantic ownership.
  More sharing makes broad in-place replacement less attractive. Polls require a
  real stable-definition/option policy; experiment 13's simulator completeness
  flag is not a server capability.
- **What does deployed Leaflet do?** Source proves intended native semantics, not
  availability/auth/normalization in the actual deployment. Failure to verify
  existing-state saves or staged-root reads can change which slice lands first.
- **How much continuity is worth paying for?** The recommendation keeps easy outer
  identity and single-splice wins. If even that costs disproportionate code, retain
  containers and rewrite their text; if richer editing dominates, reuse a mature
  reconciler. Operation-count optimization alone would not change this lean.

The architectural center is a small adapter from **desired current content plus
fresh native state** into verified native edits. Its improvement seams are the
text-edit helper and supported block mappings, not a new general planning system.

## 10. Cross-review and revised stance — after the independent join

**Review target:** g's initial draft at
[`ff2f4a7b858e99625d3496e55aa97f32559bde0c`](https://github.com/rektide/leafbud/blob/ff2f4a7b858e99625d3496e55aa97f32559bde0c/design/native-edit0.glm53m.md),
read with `git show`, compared with my initial `4ab29da3`. This addendum preserves
sections 1–9 as independent evidence. I have not read g's stage-two addendum.
The stance below controls my recommendation where it narrows the original.

### Revised recommendation: land a single-container save first

**Start with existing bound draft text whose changed native paragraph contains
one XmlText child, and whose desired content contains only supported text/marks.**
Retain that paragraph and XmlText; use a single prefix/suffix splice plus desired
formatting. Preserve unchanged other blocks, including supported blocks with hard
breaks. Permit multiple eligible changed blocks in one document, but reject a
mixed unsupported edit before any write. Do not implement general per-block
create/delete/move/type-change planning or XML child matching in this first slice.

This narrows my initial same-topology pairing/coarse-topology branch. G's emphasis
on a simple mechanism is persuasive; however, deleting whole XML children is not
merely a simpler implementation of character overwrite with identical effects.
Removing that branch from the first landing avoids both a topology matcher and
an additional replacement-semantics decision. Hard-break/topology editing remains
a credible next coarse capability, not a requirement for a general diff engine.

I retain the complete edited-existing-state wire default, but now state more
precisely that **g's single-transaction delta is also technically valid** under
the verified merge assumptions. I retain a stop-and-observe recovery boundary;
I reject automatic re-pull/replan as a sufficient idempotency argument.

### A. Single splice versus coarse overwrite: real costs and gains

There are three distinct mechanisms to compare:

| Mechanism | Extra work / continuity |
| --- | --- |
| Delete all characters in the retained XmlText; insert desired runs with attributes | Simplest retained-container writer. No old/new text correspondence or format clearing pass. All replaced character anchors are traded, even for a mark-only change. |
| Single splice in that XmlText; apply desired formatting ranges | Adds common prefix/suffix calculation, UTF-16 position conversion and clearing of obsolete owned marks. Retains prefix/suffix character items; mark-only edits retain all character items. |
| Delete paragraph children and rebuild them | Simple fresh child emitter, but deletes containers as well as characters. Distinct anchor/concurrency effects; not equivalent to either row above. |

My splice recommendation is **not free**. Reusing characters means old marks must
be removed where absent from the desired runs. A small implementation can set the
supported mark keys to their desired values or `Any::Null` over each desired
range, without optimizing unchanged formatting operations. Coarse insertion
instead supplies the complete desired attributes on newly inserted runs. Both
need the same mark-value encoding and semantic read-back; only the retained path
needs the clearing logic. Current unknown-mark refusal remains in force, so this
is not permission to erase unknown annotations under a broad mark reset.

The correspondence calculation itself is small: compare Unicode scalars from
both ends without overlapping the retained ranges; convert their lengths to
UTF-16 for Yrs. It needs no tokenization, LCS, grapheme normalization, source-span
alignment or per-character persisted IDs. Literal whitespace/newlines stay exact
within XmlText. MDX soft-wrap normalization already occurred during parsing;
hard-break XML is outside this first changed-block profile. Formatting boundaries
do not decide character correspondence: compare strings, then apply marks.

This preserves useful continuity for everyday edits but not maximum continuity.
Two distant edits replace the whole intervening middle, including unchanged words
between them. Equal-looking repeated text does not prove intended historical
correspondence. Explain the guarantee as retained outer text/container identity
and limited character retention, never “all comments survive.” Current production
does not support comment marks anyway.

**New lean:** the modest splice is worth its bounded helper and format pass; the
existing y-prosemirror `updateYText`/lib0 approach corroborates that scale. If the
implementation reveals substantially more machinery than this, use all-character
overwrite **within the same XmlText** for the first slice. That is an acceptable
product trade, not a correctness failure. Neither option requires an XML matcher.

### B. Delta versus complete edited state: valid alternatives, not fresh replacement

Both proposals load the real base and edit it through Yrs. An independently
generated fresh document is a different operation and remains wrong for an
existing text fact: merging it can add another paragraph rather than replace one.

I re-read the pinned server's `assertFact` and flush implementation. It captures
the stored base, retains the last submitted value per request client for the
entity/attribute, and merges those values with `Y.mergeUpdates`. Therefore:

- One edit transaction's `encode_update_v1` includes that transaction's insertions
  and deletions. Given the real prerequisite base at the receiver, **one such
  payload per changed fact/client is sufficient**. G is right about this. My
  original suggestion that full state uniquely avoids accumulation work was too
  strong for this one-transaction/one-payload design: there is no delta chain to
  accumulate in that case.
- Several incremental deltas sent as separate writes from the same client in one
  request can lose earlier prerequisites through the last-write cache. Both
  proposals should simply emit one payload per fact. A transaction delta is
  cumulative over its own transaction, not automatically over earlier transactions.
- Full edited state includes loaded prerequisites and the document delete set;
  transaction encoding includes the edit's deletes. Delete-only changes work with
  both. Neither should use state-vector equality as a no-op test.
- Complete state matches native full-state convention and can be decoded without
  separately supplying the base. It retransmits more accumulated history, which
  can matter for a long-lived heavily edited block even with few users. It does
  not recover local offline history or strengthen conflict resolution.

The stage-two scratch probe used the same pinned Yrs and real Yjs `mergeUpdates`:
for both retained-XmlText and replaced-child edits, delta and full edited-state
payloads produced equal concurrent results after merging with the base. Full
state decoded on its own; the delta had pending prerequisites until the base was
applied. Duplicate delivery of either exact payload was harmless. Commands:
`cargo build --offline`, `node wire-probe.mjs`; all assertions passed. This is
synthetic engine evidence, not deployed-server verification.

**New lean:** keep full edited state as a simple native-shaped default, without
selling it as necessary for correctness. Encoding choice is one private helper,
not a public mode or separate plan type. A measured payload/history problem or
stronger deployed delta evidence could switch it without changing the edit
architecture. Missing/deleted remote facts still require an identity decision;
self-contained state is not permission to resurrect them. The server's malformed-
merge fallback is not a recovery mechanism either format may rely on.

### C. Concurrency and retry claims that need correction

G's claim that unseen concurrent content survives its stated **child deletion**
algorithm is contradicted by the stage-one probe. The exact distinction is:

- Retain paragraph **and XmlText**; delete only observed characters: a concurrent
  insertion into that retained text can survive, as experiment 05 and my `Rnew`
  probe show. This does not protect against an independent deletion of a parent.
- Retain paragraph but delete its **XmlText child**: a concurrent insertion inside
  that child can become invisible when the child is removed. My probe converged
  to `new`, exactly the desired content, despite the lost visible `R`.
- Delete a paragraph or replace a larger body: the same container-removal concern
  extends to that larger scope. Detached old content remaining in storage does
  not mean its annotations/edits remain attached to the visible replacement.

The delete set need not list the unseen character for its visible contribution
to disappear. Thus “we delete only seen items” is an insufficient survival proof.
Read-back can detect a visible merged divergence; it cannot establish that no
concurrent content became hidden under a deleted container. Preflight checks
remain best effort, with no invented caller CAS or lock across the read/write
gap. This is a limit to describe, not a reason to build multiplayer machinery.

G's recovery sentence—re-pull/replan because the same plan converges—also needs a
boundary. **An identical encoded update/request is idempotent; an independently
compiled edit expressing the same desired prose need not be.** The stage-two
probe compiled `old → new` twice from the same base using fresh Yrs clients.
Applying both yielded `newnew`, with either retained-text or replaced-child
editing. That models the essential hazard when a timed-out request remains in
flight while a new attempt reads a not-yet-updated server and recompiles.

Fresh observation after a completed save can of course discover equality and
produce no write. That is different from proving a pending request can no longer
arrive. Retain exact bytes/IDs for same-process retries; after restart inspect
attempt acknowledgments and current content, settle if established, otherwise
report unresolved/partial. Do not repeatedly overwrite merged divergence. Both
full state and deltas share this obligation; neither encoding solves recovery.

### D. Equality is reusable; a structural planner is additional machinery

I reject the peer's claim that the conflict contract already requires “exactly”
the per-key comparison needed for all mutations, making the incremental cost
merely an op builder. `editor_value`/`Agreement` answer whole-document equality
and change direction. A generated-body implementation can use that gate without
matching every old block to its replacement or deriving moves/deletions.

Per-block editing additionally needs correct association lookup, desired/current
field ownership, handling of shared entities and occurrences, retained versus new
bindings, and validation/read-back of each operation family. Existing structures
make these jobs feasible; they do not make them implemented or costless. The
current parser's exact source-key/node-table coverage also excludes casually
adding/removing blocks. A flat operation enum does not eliminate those semantics.

Consequently fresh-subtree generation deserves a fairer comparison. It can
substantially simplify broad heterogeneous authoring, at the cost of staging,
rebinding, visible-anchor changes and stable interactive-object exceptions.
It does not automatically destroy comment entities, and its explanation can name
preserved public identity, replaced pages/blocks and retained polls rather than
only say “body replaced.” If the user selects generated-body ownership, those
effects are an explicit trade, not inherently silent destruction.

I still prefer the small existing-text path **for today's bound imports**, not
because a full structural planner is almost free. I also retract any implication
that every later unsupported operation must immediately offer an implemented
recreation fallback. An unsupported operation can stop with an accurate reason.
Building both general keyed updates and general stage/switch now doubles the
important paths before either has deployment evidence.

For later ordering, g's “never renumber unchanged memberships” is stronger than
the user's product requirement. Moving all desired unique occurrences in order
can be acceptable. A human summary should report the semantic reorder, not expose
every incidental position write as a separate user change. Choose minimal moves
only if the existing native helpers make that simpler; do not build a fractional
indexing optimizer to improve operation counts. Repeated/shared occurrences still
need explicit handling regardless of which order algorithm wins.

### E. Concrete revised first slice and flat explanation

Join `leafbud-editor-text` with its essential conflict/read-back protections
in one coherent end-to-end delivery, rather than first building a general planner
for create/delete/move/type-change. Preconditions:

1. Existing verified draft binding, complete confirmed node rows, one unambiguous
   target per changed text entity, supported current/desired representation.
2. Each changed block has an existing text fact, paragraph and single XmlText;
   desired change is text/marks only. No create/delete/type/placement/metadata
   change may ride along. If derived-title behavior would alter document metadata,
   block that case unless its native mapping is handled in this slice.
3. Reuse scoped drift classification, recheck source/remote evidence, compile one
   edit per target, locally decode/verify, issue one native request, and read back
   desired full supported meaning plus protected identities/scope.
4. Persist a small pending marker before effects; an uncertain outcome may stop
   for observation/recovery. No autonomous replan-until-equal behavior. Patch only
   verified receipts while guarding local file bytes.

**Automatic key assignment is accepted interface intent, not an implemented
dependency to assume.** This slice uses already imported/keyed/bound files; it
does not need to implement key insertion or pretend an empty imported draft can
already accept newly authored blocks. New-source parsing, missing-key assignment,
new associations and first binding are subsequent work. G's first-generation
suggestion remains useful once those actual dependencies exist.

The first preparation result can be a flat list of target text facts with encoded
updates and brief effect labels, plus borrowed desired/observed inputs for
verification. No generic `ScopeOp` covering future structure is needed. Preview:
“Save 2 text blocks to the existing draft; retain their block and text containers;
public content unchanged.” Include blockers/uncertainty when present. Optional
human Markdown diff remains independent. No runtime toggle between splice and
overwrite, no second comparison tree, no persisted plan.

Retain the initial fixtures for marks, Unicode, exact no-op, unsupported mixed
edits, stale observation, ignored writes, local race and uncertain retry; replace
the first-slice topology fixture with **refusal before writes**, and retain the
coarse concurrency probe as evidence for the later topology decision. Do not
describe a closed text-only loop as full structural authoring.

### Decisions still belonging to the parent/user

- Select single-splice retention versus simpler all-character overwrite inside
  the retained XmlText. My new recommendation remains the splice; a materially
  larger implementation than the bounded calculation/mark pass would change it.
- Decide whether later hard-break/topology changes normally rebuild children or
  require deliberate recreation. The prototype demonstrates the trade; it cannot
  decide the user's acceptable continuity policy.
- Select broad keyed structure versus generated-body ownership when expanding
  beyond existing text. Measured implementation burden, shared references/polls
  and actual desired authoring scope should decide, not edit counts or presumed
  free reuse of equality.
- Verify deployed native writes/read-back and request recovery in an authorized
  throwaway editor. Source inspection and these synthetic probes do not establish
  deployment parity. Full edited state is my default, with delta a valid alternate
  encoding rather than a competing architecture.

This revised stance **keeps** keyed outer identity, existing Yrs, small splice and
full-state defaults; **changes** the first landing to a narrower single-container
operation and grants genuine savings to the generation alternative; **rejects**
universal unseen-edit survival, automatic replan idempotency, free structural
planning and automatic-key-assignment-as-existing-code claims. No implementation
or remote write is commissioned by this review.
