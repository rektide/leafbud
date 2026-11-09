---
type: ImplementationReceipt
title: "Complete keyed native saves"
description: "Version-0.16 production text/structure compiler, preview-first public push, guarded first/revised/repeat saves, independent Yjs and HTTP/Phoenix evidence, and create-free deployed P1 proof."
resource: /design/keyed-save0.gpt61s.md
status: implemented-awaiting-parent-review
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3d96851dffeS2cPD2vPHjGklF"
    title: "Commissioned complete keyed save implementation, native/compiler/CLI integration, offline and deployed proof, and receiving handoff"
  - resource: /AGENTS.md
    title: "Operator's complete drafting boundary, Sol-high selection and session rollover policy"
  - resource: /design/keyed-edit-flows0.gpt6a.md
    title: "Primary new/change/drop/order cases and full first/revised/repeat acceptance"
  - resource: /design/keyed-authoring0.gpt6a.md
    title: "Accepted keyed native-object foundation, complete supported profile and bounded Yrs editing"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Compiler/workflow/protocol ownership, native-effect receipts and live follower join"
  - resource: /design/native-effects-review0.gpt6a.md
    title: "Parent decisions on first-binding defaults, original-intent recovery and selected P1"
  - resource: /design/native-default-adoption0.gpt61s.md
    title: "Received 0.15 foundation, actual P1 settlement, exact description-clear/source-preservation obligations"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Independent Yrs/Yjs mechanics and negative replacement/delta evidence; not the production edit algorithm"
  - resource: /design/connected-watch0.gpt61s.md
    title: "Received follower, locking and full explicit-selection guards"
  - resource: /crates/leaflet/editor/src/write.rs
    title: "Pure complete compiler and source preflight"
  - resource: /crates/leaflet/editor/src/text/write.rs
    title: "Production loaded-state text writer and cumulative payload verification"
  - resource: /crates/workflow/src/native/save.rs
    title: "Public per-document guarded save orchestration"
  - resource: /crates/workflow/src/native/write.rs
    title: "Native request execution, compact detached verification roots and terminal settlement"
  - resource: /crates/cli/tests/push.rs
    title: "Executed complete CLI fixtures and opt-in create-free existing-P1 deployment proof"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "Re-read actual addLastBlock/moveBlock arguments, placement-ID continuity and ordering"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cachedServerMutationContext.ts
    title: "Re-read cardinality-one ID reuse, cardinality-many exact assertions, permission skips and cumulative text merge cache"
---

# Complete keyed native saves

## Outcome and standing

**Complete supported drafting is implemented**, not a text-helper-only or abstract
compiler checkpoint. Version **0.16.0** exposes preview-first `push FILE.../--all`,
`--yes`, deliberate `--take local`, selected `--recreate KEY`, state filtering and
versioned JSON. It composes local creation and ordinary Markdown preparation with
both first-binding contexts, the complete supported body compiler, native effects,
read-back, guarded tracking settlement, pull and unchanged repeat.

Implementation checkpoint: **`8e144b8b`**; deployed/follower integration and final
reporting/title hardening: **`33648dc4`**. Documentation is a separate commit. Parent
review/acceptance is distinct from this implementation/verification receipt.

The source-pinned Leaflet archive was clean at **`600c7b5f…`** during inspection.
No external source checkout was modified. No new dependency or crate was needed;
the existing shared native protocol is used downward by editor and web. All
workspace packages and exact internal pins are coordinated at 0.16.0.

## Domain boundaries and APIs

| Owner | Implemented receiving surface |
| --- | --- |
| `leaflet/editor::write` | `preflight(Document)` and `compile(desired, observed, facts, persisted_title, recreate) -> Compiled`; only native mutation arguments, proposed associations and allocated IDs |
| `editor::text/write` | `write_text(current?, desired) -> Option<cumulative-v1>`; `None` means semantic no-op |
| `editor::write/{blocks,pages,order,metadata}` | Owned scalar/text updates; keyed page/block creation; relationships and detachment; simple order normalization; context-correct metadata |
| `workflow::native::save` | `RemoteReader::save_native` with `SaveOptions`/`SaveReport`; one stable-ID lock across preflight, preparation, mint, effects and settlement |
| `workflow::native::write` | Existing exact-byte bounded request runner, now joined to complete compilation; terminal recovery and protected-scope checks |
| `workflow::push` | Serial explicit/bulk selection, state filter, full-selection late-duplicate guards and per-file reporting |
| `cli::push` | Thin argument/report/exit adapter; no compiler or transport policy |

There is no semantic operation AST, prose-similarity matching, LCS, fractional-index
port, new locking framework or whole-body native regeneration. The existing
Document model, MDX key preparation, native vocabulary and real web runner remain
the single path. Preview uses transient proposed keys and never writes source,
pending, mint or push. An unbound preview reports complete source preflight rather
than fabricating native mutation counts before a verified seed exists.

## Identity, text and structure

- Retained occurrences preserve block/entity/placement/text-fact IDs. New source
  keys allocate genuinely new objects. Split/join identity follows the author's
  retained marker, not inferred textual similarity.
- Compatible single-text-child changes use a scalar-safe common prefix/suffix,
  UTF-16 offsets and a supported mark pass. Formatting-only changes retain
  characters. Semantic equality skips writes regardless of XML segmentation.
- Supported break/topology changes choose paragraph-interior rebuilding before
  execution. The loaded native paragraph, text fact, block and unrelated roots
  remain. This is not a fresh independent replacement document. Unknown active
  semantics and unresolved causal state refuse; they cannot trigger escalation.
- Every changed fact gets one **complete cumulative loaded-and-edited v1 payload**,
  locally decoded by the production reader before effects. Independent fresh Yrs
  documents are used only on genuinely new text entities.
- Type/scalar transitions manage only the supported owned fields and retain known
  inactive text facts. URL images, code, quotes, rules, headings and flat lists
  share the same block builder. No entities/assets are deleted for omission.
- Page relationships are prepared with real IDs; unreachable definitions, dangling
  references and cycles refuse before effects. Incoming reference locators are
  canonical sets, so traversal/link order does not churn page identity.
- New placements append, then a changed page moves desired uniquely placed children
  to native `end` in order. Identical order skips the pass. Cross-page moves use the
  actual old parent. Repeated same-entity placements in one parent are diagnosed.
- `--recreate KEY` forks one understood block, reasserts its exact membership at
  observed placement and leaves old/shared occurrences intact. It cannot resolve
  an outstanding attempt or recreate an arbitrary unknown object.

Two real defects were exposed by independent checks, not hidden by test changes:

1. `XmlText::get_string` includes XML mark serialization. Offset arithmetic now
   collects actual string inserts from Yrs diff, avoiding marked-text deletion
   lengths derived from markup. The real Yjs mark-removal group caught this.
2. Entity-only key fallback could let a newly observed shared occurrence sorting
   first steal an existing source key. Reconciliation now reserves **all exact
   occurrences first**, then considers fallback. The shared drop/recreation fixture
   passed **20 stress iterations** after correction.

The image protocol validator also stopped indexing absent `data.value`; image
payloads use native `src`/dimensions fields instead. The full profile HTTP fixture
exercises that executable path.

Metadata compilation receives the actual persisted title separately from the
reader's display fallback. A local title of `Untitled` is really stored even when
an absent native title would display the same fallback; both contexts have a
regression and then repeat without writes. Description clearing retains the
received exact None/empty-only canonical rule and native context-specific encoding.

## Native effects and detached protection

The pre-save snapshot alone cannot resolve a later move of a newly attached block.
Runner preflight now maintains a small membership-ID → `(parent, entity)` table
from current facts and preceding actual native arguments. Add/assert/retract/move
update it in request order; ambiguous/missing entity-addressed moves fail before
effects. This is placement validation, not an optimizer or another document model.

Dropping C hides its retained text from rooted pull without deleting it. Compact
`Protection.verificationRoots` stores only IDs for explicit current reads. Before
effects and during read-back, those rows are kept in a **separate verification
view**, never passed as detached rows to the strict active graph observer. Active
and explicit facts are deduplicated and checked for consistency; the SQL `root_id`
traversal annotation is removed, while stored fact evidence remains witnessed.
The hash protects both retained detached facts and untouched active facts. A
hostile detached-text change correctly produces terminal partial, not success.

Raw identity/rights observation is intentionally independent of complete typed
body decoding: an acknowledged partial graph can lack a block type and must still
be reported **partial** with its attempt retained. Compilation and achievement
verification separately require complete supported meaning. Transport/read-back
failure after dispatch reports uncertainty, rather than pretending no effects
occurred. Source byte guards, full external-operand selection checks, writer-group
acknowledgments, bounded requests and exact in-process replay remain in force.

Only whole achieved meaning/order/metadata/associations and protected scope advance
receipts. Preview cannot settle. On execution, already-achieved terminal pending
work may settle tracking without another write. Unacknowledged or terminal-partial
work remains pending; neither `--take local` nor recreation independently re-encodes
it. Source comments saved during a request survive later settlement; changed author
intent stops it. General deliberate terminal accept-current/replan is still open.

## Executed verification

Final regular gate: **282/282 workspace nextest tests**, **five deployment tests
ignored**, formatting and strict workspace/all-target Clippy. The complete CLI
fixtures exercise both contexts through actual HTTP, not only compiler projection:

- local create → ordinary unkeyed Markdown → zero-byte/zero-effect preview → full
  first mint/save → `[A,B,C] → [H,B,D,A]` → read-back/pull → unchanged repeat;
- retained/new IDs, independent first-heading/title, split and join, Unicode,
  marks/breaks, flat lists, code/image scalars and inactive text;
- named pages, repeated page links, link ordering, page retirement, cross-page
  moves, shared occurrence drop and selected recreation;
- whole unsupported remainder before mint, skipped mutations, detached creation,
  malformed intermediate graph, source races, terminal recovery, visible desired
  content without writer finality, drift decision and take-local against current
  complete state;
- public exit/report contracts, preview no source/pending/key/native writes,
  unchanged bytes/mtime and empty operation sets;
- existing exact-request retry, bounded multi-request uncertainty, rights/selection
  guards and follower source/pending tests remain passing.

The complete multi-block first-save request also passed response-loss replay with
byte-identical Yrs/envelope/ID arguments and no duplicate objects, not just the
earlier metadata-only exact-retry fixture.

The connected join uses **actual loopback HTTP + Phoenix sockets**: a real native
save remains unacknowledged/pending; the follower preserves it; terminal observation
settles without another push; a subsequent native remote save emits a poke and is
adopted with retained IDs. It passed **five stress iterations**. Busy one-shot
operations honestly report lock contention; the test retries only this explicit
pre-effect busy result, not uncertain native effects.

Independent actual Yjs/y-prosemirror consumption uses experiment 05's pinned JS
dependencies but calls the **production** writer through an offline example:

```sh
cargo build -p leafbud-editor --example text-write
node crates/leaflet/editor/tests/text-interop.mjs
```

**7/7 groups passed:** scalar-safe Unicode/suffix anchor, all seven marks and
mark-only removal/change, deletion-only state-vector behavior, coarse breaks/empty
text with retained paragraph/other roots, segmented/tombstone no-op, fresh-new and
cumulative last-payload independence, and unknown/incomplete-state refusal.
These are neither same-implementation round trips nor the experimental quadratic
LCS. Node emits the existing dependency localStorage warning; no browser storage
or application endpoint is involved in these tests.

## Actual existing-P1 proof — no new draft

The separate opt-in CLI integration test
`authorized_selected_p1_complete_bound_cli_save` **passed** against the actual
human-selected existing P1. It refused unbound/pending input and checked the exact
selected catalogue token before work. The standalone partial was not selected.

Executed layers:

1. Original settled P1 → CLI preview → three-block baseline. Original A/main/root/
   local UUID and binding remained; B/C gained actual new native associations.
2. CLI preview → combined `[H,B,D,A]` revision. H/D were new, B retained its block/
   text fact while changing Unicode/marks/breaks, C detached, A moved without a
   text rewrite, title remained independent of the inserted first heading.
3. Actual native read-back and protected detached verification settled normally.
   Actual CLI pull was **unchanged**, status **clean**, repeat push **unchanged**
   with empty effects and byte/mtime equality. Writer reservation did not advance.
4. **Zero draft-creation calls**, no release/email/cleanup/unrelated document edit.
   Both consumed budget files and standalone source bytes/mtime remained identical.
   P1 remains an unpublished draft. New block entities inside P1 were ordinary
   editing, not new document creation.

Private source/binding/body/operation evidence is under ignored
`.test-agent/native-effects/deployed/`, with `publication.keyed-before.private.json`
and `publication.keyed-proof.private.json` plus command outputs. The test deliberately
refuses rerunning over its started proof marker: inspect current state rather than
reconstructing old intent. No tokens, roots or private bodies are committed here.

This is actual **complete bound compiler/CLI save** proof, not a live first-unbound
mint claim assembled from unrelated layers. Complete first mint in **both contexts**
is loopback-proven. Original lost publication-mint attribution remains unestablished;
P1 is human-selected. The old standalone requires separate deliberate recovery.

## Remaining boundaries and road ahead

`leafbud-editor-text` and `leafbud-editor-structure` have complete implementation
and their first/revised/repeat verification. Essential portions of conflicts,
binding, recovery and follower interaction advanced with this delivery; those
broader tickets are not declared complete by this receipt.

The next coherent **fresh Sol high** batch is `leafbud-editor-recovery` with
`leafbud-editor-conflicts`: add a deliberate source-safe terminal accept/replan
boundary, retaining prior attempt provenance and never calling new intent the old
intent's success. This can then finish `leafbud-editor-bind` acceptance around the
complete ordinary save loop. The existing standalone partial is a concrete recovery
case only if the operator commissions its resolution; do not mutate it incidentally.

Canvas, nested/task lists, interactive content, uploads, public release, whole-body
recreation, remote CAS and automatic cleanup are not included. Code/image fields
and rich text are bounded supported mappings, not an arbitrary editor graph writer.
Known concurrency limits of rebuilding paragraph interiors remain explicit.

Connections: [keyed flows](keyed-edit-flows0.gpt6a.md) supply the behavioral contract;
[native defaults](native-default-adoption0.gpt61s.md) supply the received preparation
and original-intent boundary; [connected watch](connected-watch0.gpt61s.md) supplies
the cooperating follower. The [README](../README.md#keyed-editor-saves) is the
maintained user-facing command/capability home. Parent-owned build-path/review
documents can link this receipt after integration review.
