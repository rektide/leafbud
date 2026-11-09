---
type: Design
title: "Native edit strategy — keyed block operations with block-local text rewrite"
description: "Right-sized design for enacting desired local MDX into existing Yrs text and the Leaflet fact graph: per-key typed comparison, coarse same-entity text rewrite, targeted fact operations, fresh-subtree only as deliberate fallback and first generation."
resource: /design/native-edit0.glm53m.md
status: proposal
generated:
  at: 2026-11-08
  by: model:zai-coding-plan/glm-5.3#max
updated:
  at: 2026-11-08
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e437006ffffetjLhrTerTYFAE4"
    title: "g stage-1 independent draft; then stage-2 cross-review of the peer draft and its executed probes, producing the revised-stance addendum"
  - resource: /design/native-edit0.gpt6a.md
    title: "Peer Astra-high stage-1 draft at commit 4ab29da3de51 (initial independent version, read for cross-review before any peer addendum)"
  - resource: /.test-agent/native-edit-astra/README.md
    title: "Peer-executed offline probe scope: container-vs-character replacement and delta-vs-full wire forms; scratch is untracked, findings recorded here and in the peer draft"
  - resource: /.test-agent/native-edit-astra/wire-probe.mjs
    title: "Executed assertions read for this review: concurrent-insert visibility by edit granularity, self-containedness, and stale-base regeneration duplicating content"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Parent/user authority: commission, updated low-volume complexity-first stance, and mid-session Leafbud rename steering"
  - resource: /AGENTS.md
    title: "Operator product-fit and complexity direction controlling this design"
  - resource: /design/status-witnesses0.gpt61s.md
    title: "Landed witnesses/status, editor profile leaflet-editor/2, agreement classification and reuse seams"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted push/pull/publish contract, conflict table, --take/--recreate spelling and exit semantics"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Keyed bidirectional addendum whose fine-grained aspirations this design reweighs; stage/switch fallback retained"
  - resource: /design/yrs-conversion0.gpt6a.md
    title: "Verified Yrs surface, update/delta distinctions, server merge constraint and fact mutation ownership"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Fresh-subtree stage/verify/switch recipe reassessed here as fallback rather than default"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Executed Yrs/Yjs edit mechanics: transaction deltas including deletes, no-op semantics, wrong-root replacement negative control, concurrency survival"
  - resource: /experiments/05-yrs-conversion/src/lib.rs
    title: "Actual load/transaction/edit/delta-encode composition this design reuses without its LCS"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Source-mapped push RPC contract, mutation vocabulary, replay idempotency and never-executed-live status"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Executed compiler-to-facts-to-fresh-text pipeline and stable Poll key policy"
  - resource: /experiments/13-satteri-authoring/src/simulator.rs
    title: "Executed detached staging, inventory verification and explicit entry switch model"
  - resource: /crates/document/model/src/lib.rs
    title: "Current typed document model, node association rows and editor_value/semantic_value split at inspected revision"
  - resource: /crates/leaflet/editor/src/lib.rs
    title: "observe() keyed scan, strict profile and unrepresented-attribute refusal at inspected revision"
  - resource: /crates/leaflet/editor/src/text.rs
    title: "Production Yrs decode path promoted from experiments 05/13"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Agreement classification and adoption logic the push guard/verify loop reuses"
  - resource: /crates/leaflet/web/src/lib.rs
    title: "Read-only web client: existing reads, absent write transport at inspected revision"
  - resource: /Cargo.toml
    title: "Production Yrs/Satteri pins at inspected revision"
---

# Native edit strategy — how local intent becomes CRDT and fact changes

## Verdict

**Enact desired content through keyed per-block operations: compare typed,
keyed current meaning on both sides; leave unchanged blocks unwritten; rewrite
a changed block's text coarsely on the same entity in one Yrs transaction;
express structure through a handful of explicit fact mutations; keep the
fresh-subtree stage/switch path as deliberate fallback and first generation.**
Do not build the character-level edit compiler or any diff/alignment engine
now. No new planning document model, no saved bases, no parallel comparison
tree.

This revisits the [keyed bidirectional
addendum](authoring-handoff0.gpt61s.md#keyed-bidirectional-addendum) under
the operator's updated stance (low volume, few users, bounded conceptual
complexity over structural efficiency, coarse operations acceptable). The
addendum's *identity goals* survive: durable source keys binding blocks to
remote entities, structure explicit in the source. Its *mechanism ambition*
—the bounded-LCS scalar/child matching with split/merge policy — is dropped
from the first build. Character-identity continuity inside an edited block is
the specific thing we trade, honestly and reversibly.

Revision bindings: all `leafbud-*` crate names and `@leafbud` field
spellings refer to leafbud at commit **`39814abe5764`** (inspected working
copy; the operator's in-flight rename to Leafbud is owned by a separate
session and changes names, not this design's substance). Leaflet source
claims bind to `600c7b5f1f7b31b66fb626c293cc104312a12caf`; Yrs claims bind to
`e0b476b0ce613453c70d0bb477b955808aa79a07` (**0.28.1**, the production pin).
Deployment parity of the Leaflet server remains unverified; see [Evidence
standing](#evidence-standing).

## What we compare, why, and who controls what

The edit engine's input is three current values and one correspondence, never
history:

| Concern | Representation | Controlling actor | Role in the engine |
| --- | --- | --- | --- |
| Desired content | Parsed MDX → typed `Document` (pages, `BlockKind`, `Inline`) | Local author's file | Target of every operation |
| Current content | Fresh pull → `observe()` → `Content` + raw witness | Remote editor (and its collaborators) | Base of every text delta; authority for what "is" |
| Correspondence | `state.nodes`: source key → entity/membership/text-fact | The binding, maintained by tool operations | Decides *which* remote object an edit addresses |
| Conflict decisions | `Agreement` classification (existing `editor_state`) | The user, via the accepted conflict contract | Gate on what push may do without `--take` |
| Public output | Release/publication state | Separate domain (`publish`, release tickets) | Explicitly not touched by push |

Two consequences follow, and they are the core of the right-sizing:

1. **Comparison is struct equality plus key lookup, not text diff.** Both
   sides already lower to the same typed model (`Inline`/`BlockKind` with
   source keys) at the inspected revision. A block is unchanged, changed, new,
   or gone by comparing keyed typed values. The only place an algorithmic
   diff could appear is *inside* one changed text block — and the
   recommendation is not to diff there at all (next section).
2. **Identity decisions come from the association map, never from content
   similarity.** Content matching cannot distinguish move-and-rewrite from
   delete-and-create ([documented ambiguity](yrs-conversion0.gpt6a.md#5-stable-identity-without-saved-content));
   keys make every such decision explicit and previewable. Splits and joins
   need no inference policy: MDX block structure **is** desired remote
   structure. One source paragraph with one key is one block; two paragraphs
   are two blocks; merging is an authoring act (write one paragraph), not an
   engine act.

The comparison layer is not optional even for a coarse strategy, because the
accepted [conflict contract](authoring-interface0.gpt6a.md#conflict-and-effect-contract)
(unchanged → no write; local-only → apply; remote-only → report; both →
conflict) requires exactly this local/remote typed comparison. The existing
`editor_value()` / `Agreement` machinery from
[status-witnesses](status-witnesses0.gpt61s.md) already computes it; push
reuses it as its drift gate. So the real strategic choice is only about the
**write** side, compared below.

## The recommended shape

### Text changes: block-local coarse rewrite on the loaded Yrs doc

For a keyed block whose typed content differs:

1. Load the block's current `block/text` update into a fresh Yrs `Doc`
   (exactly the production [read path](../crates/leaflet/editor/src/text.rs):
   `apply_update`, reject pending state, `OffsetKind::Utf16` conventions from
   the same promoted experiment).
2. In **one transaction**: remove all currently-seen paragraph children and
   characters (`remove_range` / child removal — the operations experiment 05
   executed), then insert the desired children (`XmlTextPrelim`,
   `hard_break` elements, `insert_with_attributes` with the desired mark
   sets). No matching, no LCS, no split policy.
3. Semantically re-read the transaction's result in memory; require equality
   with the desired typed content (the experiment's `read == normalized` gate).
4. Emit **one cumulative v1 delta** — `TransactionMut::encode_update_v1` of
   the edit transaction only, base64 — as the asserted `block/text` value.

What this preserves and costs, stated plainly:

- **Survives:** block entity, membership fact, text fact identity, paragraph
  root, Yrs document/causal continuity of the block, all other blocks'
  character identity and anchors. Concurrent unseen edits still survive
  (deletes target only items present in our pulled base — the executed
  `abc→ac` vs concurrent `R` → `aRc` behavior; unseen content is never
  destroyed by our delete set).
- **Traded:** character-level item identity *within the edited block*.
  Comment marks ride characters, so comments anchored in a rewritten block
  lose their anchor (the thread entity is not deleted). The plan preview
  says which blocks lose anchors. Unchanged blocks keep every anchor.
- **Not traded silently, ever:** unrelated content. Only keys the source
  still names are touched; unmatched/unrepresentable remote content still
  hard-refuses (existing `observe` posture).

Why coarse-instead-of-LCS is sound CRDT usage, not a hack: deletion of seen
items plus insertion in one transaction is the same operation vocabulary the
fine-grained compiler emits; we merely always choose delete-all-seen +
insert-desired rather than a minimal script. The server merge cache
constraint — one cumulative delta per `(entity, attribute, client)` per
push, because the retained last-write cache drops prerequisite chunks
([verified failure mode](../experiments/05-yrs-conversion/REPORT.md)) — is
satisfied identically. The two tempting shortcuts remain forbidden, with
executed negative evidence: asserting a fresh independent document over an
existing text entity **merges into two paragraphs**; retracting and
re-asserting the text fact discards causality and can strand collaborators.
`push --take local` is therefore implemented as *re-observe fresh, then
coarse-rewrite from that base* — explicit last-writer-wins expressed as CRDT
operations, not as fact identity churn.

A new block's text (and any first generation) uses the proven fresh path: a
new Yrs doc per new entity, full initial v1 update
([experiment 13](../experiments/13-satteri-authoring/REPORT.md); a fresh
standalone update is a first-class merge input on a fresh entity — verified).

### Structure changes: a small explicit fact-operation vocabulary

| Plan operation | Native writes (Leaflet push mutations) | Identity/effect notes |
| --- | --- | --- |
| Create block/page | `addBlock` (caller-generated `factID`/`newEntityID`, explicit position) + `assertFact` for scalars; `createEntity` where a bare entity is needed | New entity, fresh full text update. Positions are caller-chosen fractional keys. |
| Rewrite text (existing) | `assertFact {entity,"block/text", delta}` | Same fact (cardinality-one update-in-place); server merges the delta. |
| Change scalar fields (heading level, code text/language, image payload) | `assertFact` on the same entity/attribute | Entity and text fact retained; heading-level changes don't touch text. |
| Change block type (text↔heading↔quote) | `assertFact block/type` (+ level) on the same entity | Same-entity type change preserves entity, text fact and anchors — mirrors upstream `blockTypeOperations`. Unsupported transitions (e.g. into canvas/list families outside profile) refuse and offer `--recreate KEY`. |
| Move/reorder block | `moveBlock` (overwrites the existing membership edge's parent/position) | Membership fact identity kept. |
| Delete block | `retractFact` on the membership fact | Entity and text detached, not deleted: comments, answers and incoming references survive on the detached entity. Never `removeBlock`/`deleteEntity` in ordinary push (descendant deletion, image cleanup, reference destruction). |
| Add/delete page + references | `addBlock`/`retractFact` for `card` blocks and page entities; guard: refuse page deletion while any remaining reference names it | Non-entry page definition order is source layout, not editor state (already true of `editor_value`). |
| Root/document metadata (title, description, tags, watermark) | `updateLeafletMetadata` / root fact asserts under explicit ownership | Publication association and release state are *not* push concerns. |

Positions: mint fractional midpoint keys between neighbors for inserts and
moves only; never renumber unchanged memberships. Renumber-all was considered
and rejected: it is not simpler to *reason* with (every preview says "12
memberships rewritten" for one moved block) and it needlessly breaks the
untouched-blocks story. Tie-breaking by `(position, factID)` exists upstream
but is not relied upon.

### Drift guard, execution, read-back

`push FILE` (no `--yes`) is exactly the same pipeline stopping before
transport: observe fresh (existing `observe_editor`), classify
(`Agreement`), build the plan, render the structured preview. `--yes`
executes it. The loop:

1. **Guard:** refuse on `both-changed`/`unverified`/unsupported/identity
   drift (existing classifications); `--take local` re-observes and proceeds
   as above.
2. **Execute:** one push call, mutations in dependency order (entities →
   facts → text), caller-generated IDs throughout. Replay is idempotent
   upstream (`lastMutationID`); re-asserted equal values are no-ops; a
   duplicate text delta does not duplicate text (verified).
3. **Verify:** fresh pull → `observe()` → compare typed content per scope
   against desired. Report achieved / partial / diverged per block; settle
   `sync` receipts only from the verified result (existing hash machinery,
   profile `leaflet-editor/2`).
4. **Recover:** any interruption re-pulls and replans from current state;
   the same plan converges (operations are idempotent by explicit ID, text
   deltas converge rather than append). A compact pending receipt (input
   hash + phase) is the separate `leafbud-editor-recovery` ticket's
   seam; this design adds no persistent plan or generation store.

Concurrency honesty: the server's token-scoped advisory lock is not caller
CAS; a web edit landing between pull and push merges per CRDT rules and may
make read-back differ from desired (`diverged`, exit 4). We report; we never
loop-push until the file "wins".

## Alternatives weighed

**A — Fine-grained character/child LCS (the addendum's full ambition).**
Keeps comment anchors across ordinary edits and minimizes deltas. Costs: the
LCS engine plus a declared split/merge correspondence policy — the exact
unresolved ambiguity the experiment flagged (repeated text, node splits
recreate characters) — plus more preview complexity. Under the operator's
stance its payoff is optional continuity; its cost is the largest single
chunk of conceptual complexity in the whole editor domain. **Deferred** with
a deliberate seam (below), not abandoned: experiment 05's compiler is
preserved, promotable behind the same `EditText` plan operation later.

**C — Fresh-subtree default (mdx-recreate0 as the ordinary save).** One
mechanism for everything; experiment 13's simulator already models it;
per-block matching becomes unnecessary. But: every save detaches *every*
block's entity — comments anchored even in untouched blocks lose their
anchors, which reads as silent destruction of other people's contributions;
detached generations accumulate per save; previews can only say "body
replaced"; and the accepted conflict contract still forces the typed
local/remote comparison, so the comparison code is not actually saved. Its
true incremental saving over the recommendation is small (the op builder),
while its product costs are permanent. **Kept** as the explicit
`--recreate KEY` / `--recreate-body` fallback, and as the generation
mechanism for first binding — where nothing exists to preserve and its
simplicity is fully appropriate.

**B — this recommendation.** Targeted keyed operations; coarse within-block
text. Incremental code over C ≈ a small op builder + preview, on top of
comparison machinery the conflict contract already requires. Untouched
blocks stay untouched (anchors, receipts, witnesses stable); writes are
proportional to intent; previews are honest ("2 blocks rewritten, 1 added, 1
removed; anchors in rewritten blocks are lost").

## Evidence standing

What is actually verified, source-mapped, or open:

- **Executed (offline, real Yjs/Yrs):** loading existing text updates,
  single-transaction delete/insert/format edits, cumulative edit-transaction
  deltas *including* delete-only changes, semantic (not state-vector) no-op
  detection, duplicate-delta idempotency, concurrent-insert survival,
  fresh-initial-update encoding, and simulated server `mergeUpdates(base,
  delta)` behavior (experiments 05/02/13; cross-language Yjs oracle).
- **Source-mapped, never executed live by this project:** the push RPC
  itself — `POST /api/rpc/push` with `{pushRequest, rootEntity}`,
  `pushVersion` 1, ordered mutations, `lastMutationID` replay, cardinality-one
  update-in-place, caller-generated IDs, and the 44-mutation vocabulary
  including `addBlock`/`assertFact`/`retractFact`/`moveBlock`/
  `updateLeafletMetadata` (experiment 02, Leaflet pinned revision). The
  pre-reset client that used it is recoverable at `80436b61`. **The first
  live push (throwaway draft, operator-gated) remains the open acceptance
  step for the write slice.**
- **Distinguished, not conflated:** Yrs state-vector diff/`encode_diff`
  APIs emit missing structs plus delete sets between *CRDT states* — they
  are irrelevant to producing edits from *desired content*, which is why the
  engine never touches them. Yrs does not and cannot reconstruct local
  offline history; nothing here asks it to.
- **Deployment semantics:** everything above binds to inspected source
  revisions; no live server behavior is claimed. Experimental-model evidence
  (simulator) is not server proof — acknowledged where relied upon.
- **Diff libraries:** no text-diff dependency is needed by this design. If
  fine-grained `EditText` is later built, `similar` (2.x, patience-lineage)
  or `imara-diff` are the obvious candidates; neither is in the workspace
  lockfile today, and adding one should wait for that need.

## Architecture and minimal interfaces

No new crate, no new model types beyond the plan enum. The engine lives at
the existing seams:

```text
leafbud-editor (crates/leaflet/editor):
  edit_text(base_update: &str, desired: &[Inline]) -> Result<TextEdit>
    // TextEdit::Fresh(full_initial_update) for new entities
    // TextEdit::Delta(cumulative_v1)       for existing entities
    // (no-op handled earlier by typed comparison)

leafbud-workflow (crates/workflow), new plan.rs alongside editor_state.rs:
  plan_push(desired: &Document, observed: &Content) -> Result<PushPlan>
    // PushPlan: Vec<ScopeOp> + expected effects + per-block preview lines

leafbud-web (crates/leaflet/web):
  push(root_entity, mutations, client_id) -> PushReceipt   // new transport;
    // shape per experiment 02; port deliberately from 80436b61, re-verify

push flow: observe_editor (existing) -> Agreement gate (existing)
  -> plan_push -> [preview | execute -> fresh observe -> verify -> receipts]
```

`ScopeOp` is a flat enum mirroring the operation table above — operations,
not a document: no content copies, no CRDT state, nothing persisted. This
*is* the "small base for iteration": the future fine-grained engine replaces
the body of `edit_text`'s existing-entity branch and nothing else; structure
operations, transport, guard, verify and receipts are unaffected. That one
function boundary is the entire preserved seam — no generic plugin
abstraction.

Reuse from inspected state: `observe`, `read_text`, node association rows,
`editor_value`/`Agreement`/`classify`, hashes and receipts, keyed MDX writer
and automatic key assignment (accepted interface: keys are assigned by
`tidy --keys`/executed push before planning identity-sensitive work).
Source-mapping/formatting comments inside those files are contextual; no
refactor is proposed.

## Scope boundaries

- **Deletion:** membership retraction only; detached entities accumulate
  until a separately commissioned cleanup policy exists. No automatic
  pruning (standing rule).
- **Moves:** membership position updates; keys ride the block, so a moved
  keyed block keeps entity/facts/anchors.
- **References:** page-link deletions guarded against remaining referents;
  reference occurrence identities follow the existing node rows.
- **Assets:** existing remote image sources are preserved/updated as plain
  facts; new local asset upload is *not* in this design (unverified path) —
  the plan reports it as an unmet preparation requirement rather than
  fabricating a resource.
- **Interactive objects (polls/questions):** outside import profile 1
  (`observe` refuses them today); when added, their identity policy is
  stable-key-derived entity IDs with explicit option-inventory guards
  (experiment 13's simulator models the guard; never silently reinterpret
  votes).
- **Publish vs save:** push touches the editor edge only; publish/release,
  delivery and released-state tracking are the separate release domain.
- **Human display diff:** Sol's `leafbud-inspect-diff` Markdown views +
  user-provided external tool are independent of this engine. Push previews
  are structured operation lists; optionally they may later reuse the same
  Markdown renderer for per-block before/after display, as presentation
  only — no coupling in either direction.
- **Published documents:** current observation supports drafts only;
  extending editor operations to bound published documents is the release
  domain's boundary to draw, not silently assumed here.
- **First binding:** generation into an operator-created (web UI) empty
  draft adopted via existing `import` is the recommended first-push route;
  headless `createNewLeaflet` (server-action probe, Next-Action discovery)
  is a separate explicitly-gated capability.

## First push slices and acceptance fixtures

Sequenced coherent slices (capability gates, not authorization):

1. **Plan engine (offline).** `edit_text` in leafbud-editor; `plan_push`
   in workflow; pure fixtures → plans. *Fixtures:* per-block no-op/change/
   create/delete/move/type-change; split (two paragraphs, one key + one
   auto-key) and join (author merged in source) cases; anchor-loss preview
   wording; unsupported-type refusal offering `--recreate KEY`; deleted-page
   reference guard; image-requirement reporting; repeated planning is
   deterministic and produces no writes. Existing status/pull fixtures are
   reused for observed inputs.
2. **Write transport (offline + gated probe).** Web-crate `push` per the
   experiment-02 contract, ported deliberately from `80436b61`; loopback
   responder tests (existing harness pattern) for ordering, replay,
   throw-skip-ack, partial batches; then one operator-gated throwaway-draft
   live push proving deployment semantics.
3. **`push` command end-to-end (draft scope).** observe → guard → plan →
   preview/`--yes` → execute → verify → receipts; exit codes 0/3/4 per the
   accepted contract; `--take local`; `--recreate KEY`. *Fixtures:*
   re-push no-op writes nothing and settles receipts; concurrent-divergence
   simulation reports `diverged` without re-push looping; interrupted
   execution recovers by re-pull/re-plan.
4. **Bind-on-first-push.** Import-adopted empty-draft generation first;
   headless creation deferred as its own probe.

## Unresolved questions and what would change this stance

- **Live push semantics** (mutation acceptance, merge behavior on real
  `cachedServerMutationContext`, position scheme tolerance): the gated probe
  decides; a failure here changes the *transport* slice, not the strategy.
- **Comment-anchor prevalence in actual workflows:** if anchored comments
  prove common in edited blocks, fine-grained `EditText` via the preserved
  seam becomes worth building — the single most likely trigger to revisit.
- **Multi-writer reality:** if the web editor is routinely open while
  pushing, `diverged` outcomes may frustrate; the answer stays honest
  reporting plus explicit `--take`, never silent re-push loops.
- **Published-document updates:** if the release domain requires
  editor-equivalent operations on published leaflets, profile support must
  widen deliberately (status currently refuses); this design's per-block
  strategy extends, the fallback path does not automatically.
- **Position key degradation:** fractional midpoints lengthen under
  adversarial orderings; at this project's volume irrelevant, but a
  deliberate compaction pass could later join `tidy` if ever observed.
- What would *not* change it: operation-count efficiency, delta byte size,
  or parser comparison concerns — explicitly out of scope by the operator's
  stance.

## Cross-review status

Independent first draft (this document), stage 1 of the commissioned
parallel wave. The peer Astra-high draft (`design/native-edit0.gpt6a.md`)
has deliberately not been read; a cross-review addendum with any revised
stance will follow on explicit parent dispatch, after both drafts are
committed, as a separately committed addendum preserving this text unchanged.

<a id="cross-review-addendum-1"></a>

## Addendum — cross-review of the Astra draft and revised stance

Stage 2, dispatched by the parent after both drafts were committed. The text
above is my unchanged stage-1 argument; this addendum supersedes it where
stated. Reviewed: the peer's initial draft at commit **`4ab29da3de51`** (no
peer addendum existed when I began; I did not read one before committing
this), its cited Yrs/lib0/y-prosemirror source readings, and its executed
offline probes in `.test-agent/native-edit-astra/` (Rust source, `probe.mjs`,
`wire-probe.mjs` — read and checked against their assertions).

### Corrections to this document, first

Reading the peer's probe evidence and its wire-form companion surfaced three
errors or overclaims in my stage-1 text. Correcting mine before criticizing
theirs:

1. **The concurrency claim was half wrong.** I wrote that unseen concurrent
   content "survives" coarse replacement, citing experiment 05's
   character-deletion case — while my prescribed mechanism was
   *child-level* removal ("remove all currently-seen paragraph children and
   characters"). The peer's probe separates the granularities, and I verified
   its assertions: deleting and reinserting **characters inside a retained
   XmlText** leaves a concurrent insert visible (both orders converge to
   `Rnew`; read-back then *differs* from desired and is honestly reported as
   diverged); deleting the **XmlText child itself** hides a concurrent insert
   living inside it — the merged result reads exactly `new`, so read-back
   *equality* is satisfied while the concurrent intent is silently invisible.
   The precise rule: our delete set never tombstones unseen items, but
   visibility follows the *container*; content inside a removed child
   disappears from the rendered sequence without ever being "deleted" by us.
   **Read-back equality proves achieved meaning, not absence of lost
   concurrent intent.** My stage-1 wording is corrected accordingly.
2. **The recovery claim was over-broad.** "Any interruption re-pulls and
   replans; the same plan converges" is unsafe in one window: the peer's
   wire probe shows two *independently compiled* edits of the same stale
   base merge into `newnew`, not an idempotent retry. Re-planning is safe
   only from an observation that *resolves the prior attempt* (it landed →
   semantic no-op; or its absence is established via acknowledgment reads
   and fresh state). In-flight/lost-response uncertainty must stop and
   report, never regenerate a payload and call it a replay. Exact-bytes
   retry (same request re-sent) remains harmless — experiment 05's duplicate
   delivery evidence covers that case only.
3. **A dependency mislabel.** I implied automatic key assignment exists to
   reuse; the accepted interface settled its behavior but the current
   baseline has no key-assignment pass. Corrected below in the revised slice
   scoping.

### The five joined tensions

**1. Splice-retain vs coarse same-block overwrite — I move; the peer is
right, and the decisive precedent is one I under-weighted.** My stage-1
rejection bundled "prefix/suffix splice" with the LCS/split-merge machinery
I was actually rejecting. They are not the same thing: a single
common-prefix/suffix splice is what production y-prosemirror does on every
keystroke (`updateYText` → lib0 `simpleDiff`, per the peer's installed-source
reading), it is ~100–150 lines with tests, and the peer's design never
proposes cross-node relocation, child-kind LCS, or repeated-text
disambiguation — the parts that made experiment 05's compiler expensive stay
rejected, by both of us. What the splice buys is concrete: comment anchors
and character identity in the untouched ends of an edited block — the
typical shape of a text edit — and, per the probe above, *visible divergence*
instead of silent hiding for concurrent edits inside retained containers.
Refinements I would add to the peer's policy: extend prefix/suffix only
while `(scalar, marks)` pairs are equal (retained ranges then never need
`format`), and treat chars-equal-marks-differ as a separate `format`-only
path so a mark-only edit never rewrites characters. Topology changes fall
back to the coarse paragraph-interior rebuild, now with its concurrency cost
stated in the effect summary. The empty-paragraph rule (keep the single
text node, delete its characters) is adopted as-is.

**2. Full edited-state payload vs edit-transaction delta — I concede the
wire form; the probe settles it as a robustness question, not semantics.**
The peer's wire probe (assertions read) shows delta and full forms produce
the *same* concurrent merge semantics; the difference is that the
full-state update of the loaded-and-edited document is **self-contained**
(applies standalone with no pending structs) while the transaction delta
dangles without its causal base. Since the server merge cache composes our
payload with *its* captured base (`Y.mergeUpdates([latest, base])`), a
self-contained payload is strictly more robust if that base is lost or
stale on the server side (e.g. another writer retracted/recreated the text
fact between our pull and push), and it is exactly what the native browser
writes send — least novelty against the one code path we cannot test
offline. Payload size is irrelevant at this volume. **First transport sends
the complete edited state; the transaction delta stays as a test artifact
and later optimization.** Two distinctions from the peer are worth
restating sharply because both drafts converged on them independently: a
full state of the *loaded-and-edited* doc is not the negative-control
"independently generated fresh document" (which merges into two
paragraphs), and retract-then-reassert of the text fact remains forbidden
either way.

**3. Granularity, not wire form, decides concurrent visibility — corrected
above.** To delimit exactly, in the revised policy: the paragraph element
is never removed; XmlText children are removed **only** when inline
topology changes, and the effect summary then says concurrent web edits
inside rebuilt inline containers may be hidden — undetectable by read-back.
Concurrent content in retained containers surfaces as visible divergence.
Concurrent *children* inserted into the paragraph by a collaborator are not
in our delete set and remain visible under either granularity. None of this
is multiplayer support, and no server CAS exists or is claimed; preflight
narrows the race, read-back verifies achieved meaning, and the user-facing
wording must not advertise more.

**4. Does the equality gate make per-block planning mandatory? No — I
overstated the "free rider" argument; the case for per-block is semantic,
and the marginal-cost comparison is closer than I presented.** The conflict
gate needs document-level typed equality only; per-block planning adds a
comparator (~50 lines) plus an operation builder (~200–300), while
fresh-subtree-every-save adds a generator (~300, experiment 13 is
promotable), staged reads, and whole-binding rewrites. The machinery costs
are comparable; neither is free. What actually decides it is product
semantics: per-block leaves untouched blocks unwritten (anchors, receipts
and witnesses stable, previews say "2 blocks changed"), while a
regenerated body detaches every block's anchors on every save — including
reader comment anchors on blocks the author never touched — and
accumulates detached generations. Under the operator's "no silent
unrelated-content destruction" line, that is the difference. Where the
peer's candid table is right and I keep it: fresh-body staging remains a
credible *product* selection if the user one day declares internal anchors
disposable wholesale; until then per-block with `--recreate KEY`/
`--recreate-body` as the explicit escape preserves that option without
maintaining two engines.

**5. First-slice scope — converged shape, with my corrections folded in.**
Both drafts now point at the same first coherent slice; I adopt the peer's
sharper boundaries: existing **bound draft** text only (today's parse
already requires every source key to have a confirmed node row, which this
slice satisfies without new machinery); **atomic scope refusal** — a desired
change outside the slice's supported scope (title, images, structure)
refuses the whole push rather than saving part of the document; a compact
**pending-attempt marker** before effects (intent/preflight hashes, attempt
and client IDs, touched native targets — a small owned-state extension);
acknowledgment reads as recovery evidence with **processed ≠ applied**; and
the derived-title interaction (first-block edits versus stored title) as an
explicit fixture. Key assignment stays labeled accepted-not-implemented:
new-block creation waits for it plus the desired-source/association
validation extension, and belongs to the structure slice.

### Adopted, kept, and rejected

**Adopted from the peer:** splice-with-coarse-fallback text policy (with my
marks refinements); full-state wire form; retain-container rule; atomic
scope refusal; pending marker field sketch; acknowledgment-based recovery
with the regeneration prohibition; shared-entity consistent-intent rule;
rooted-closure/staged-read and multiple-root-refusal boundaries; the
fixture list merged with mine (concurrency-visibility cases added).

**Kept from mine:** keyed per-block operations as the default save path
with fresh-subtree as fallback/first generation only; targeted fractional
moves; the `edit_text`-style single seam where any future finer engine
plugs in (now the splice helper, not a diff library); no diff dependency;
drift-guard/receipts reuse; draft-first scope; import-adopted empty draft
as the first-binding route with headless creation as its own probe.

**Rejected or challenged in the peer:** "move every desired child to the
end in desired order" — it rewrites every membership position on any
reorder, churning witnesses and previews for unchanged blocks; targeted
moves to fractional positions are equally simple and honest (and the peer's
own reading says `moveBlock` uses Leaflet's position generator, so the
server side is exercised either way). End-normalization remains acceptable
as a deliberate fallback if targeted placement misbehaves against the real
deployment. The peer's suggestion of adopting y-prosemirror's
`updateYFragment` in Rust is declined for this project's shape (private/
unstable API, JS runtime, schema bridge) — it stays the independent test
consumer, as both drafts otherwise agree.

### Revised recommendation (operative summary)

Keyed per-block operations; per changed text block, splice retained
containers (prefix/suffix on `(scalar, marks)`, format-only for
chars-equal) and rebuild paragraph children only when inline topology
changes, with that hiding effect stated; one complete edited-state payload
per changed text fact; targeted fact operations for structure per my
stage-1 table; fresh-subtree staging reserved for explicit recreation and
first generation; recovery that never regenerates payloads and re-plans
only from resolving observations. First slice per §5 above: offline
plan/verify engine for existing-bound draft text (fixtures first), then
the write transport with pending marker, preflight, atomic scope refusal
and full read-back — gated live probe before any claim of a working save.

### Still unresolved, and what would change this stance

- The **gated live probe** remains the arbiter of deployed merge/mutation
  behavior; a surprising server response (payload rejection, merge
  poisoning, position scheme) changes the transport slice, not the
  strategy.
- **Anchor prevalence**: if comments/annotations on *edited middles* turn
  out common, the next step is a better interior matcher behind the splice
  seam — still not a general engine.
- **Whether the user ever declares body anchors disposable wholesale** —
  that single product statement would flip the default to fresh-body
  staging per tension 4, and nothing else in either draft would need to
  survive unchanged.
- **Key assignment timing** (push-time insertion vs `tidy --keys` habit)
  affects only the structure slice's UX, not architecture.
- Peer's Sol-high recommendation for the first write boundary is
  reasonable and remains the parent's call; my stage-1 slice sequencing
  (offline fixtures first) stands as the safer prefix either way.
