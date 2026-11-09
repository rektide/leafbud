---
type: Design
title: "Complete local drafting through verified generated bodies"
description: "Independent fresh Astra proposal: disk-first authoring, automatic keys, native draft minting, a single staged-body materializer, explicit identity effects, guarded pull and repeat no-op; concrete profile, implementation batches and protocol proof gaps."
resource: /design/drafting-push0.gpt6a.md
status: proposal
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
updated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e435692e7ffeGsnomW6T3dcfwp"
    title: "Independent first draft and authorized revision-bound peer cross-review; retained generation default, corrected inactive-data eligibility and verified publication/metadata/recovery distinctions; initial accidental search exposure disclosed"
  - resource: https://github.com/rektide/leafbud/blob/7c0e6f9501bf8ae52280aef9f37932804703902c/design/drafting-push0.glm53m.md
    title: "Authorized stage-two review of g's corrected independent draft, read with git show; no evolving peer cross-review read"
  - resource: https://github.com/rektide/leafbud/blob/b5e66475c66a36363d47e4f0c9e198287d59fcf9/design/drafting-push0.gpt6a.md
    title: "Preserved independent first argument; the appended cross-review states subsequent changes"
  - resource: /design/draft-checkout0.gpt61s.md
    title: "Actual imported-corpus receipt, including mixed JSX/inactive-text blockers resolved before 27-draft success"
  - resource: /crates/workflow/src/import/tests.rs
    title: "Committed synthetic inactive-code fixture proves current code projection and opaque remote witnessing"
  - resource: /crates/workflow/src/config.rs
    title: "Actual publication defaults remain configured authoring context, not only origin/auth"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Parent commission and operator correction: complete ordinary drafting is the delivery boundary, with coarse mechanisms permitted"
  - resource: /design/drafting-push-brief0.gpt6a.md
    title: "Controlling corrected brief, read first"
  - resource: /AGENTS.md
    title: "Low-volume complexity budget and workflow-sized acceptance"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted root verbs, automatic key preparation, state ownership and conflict contract; proposed body-continuity change identified explicitly below"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Earlier creation/materialization route and subsequent keyed bidirectional intent, reconsidered against the corrected brief"
  - resource: /design/native-edit0.gpt6a.md
    title: "Previous-wave Yrs API, child-deletion/concurrency and stale recompilation evidence; narrow delivery recommendation not adopted"
  - resource: /design/native-edit0.glm53m.md
    title: "Previous-wave keyed operations and corrected equality/retry arguments; no same-wave draft read"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Native entry-switch and public-identity reasoning, reconsidered as the one default engine"
  - resource: /design/status-witnesses0.gpt61s.md
    title: "Implemented scoped equality and conservative witnesses, not a structural planner"
  - resource: /design/human-diff0.gpt61s.md
    title: "Implemented external human comparison, kept independent of mutation authority"
  - resource: /Cargo.toml
    title: "Production Leafbud 0.9.0 dependencies at parent checkpoint 9a3a0dbb"
  - resource: /crates/document/model/src/lib.rs
    title: "Current bound-only State/Node and association-bearing editor_value"
  - resource: /crates/document/mdx/src/lib.rs
    title: "Actual Page/Key parser, exact key-table constraint, source spans and source-preserving update limits"
  - resource: /crates/leaflet/editor/src/lib.rs
    title: "Strict native graph observation, inactive-text locators and derived-title behavior"
  - resource: /crates/leaflet/editor/src/text.rs
    title: "Existing supported Yrs reader and mark profile"
  - resource: /crates/workflow/src/import.rs
    title: "Token-derived import identity, metadata precedence and receipt construction"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Current identity gate, association-first key reconciliation and endpoint classification"
  - resource: /crates/leaflet/web/src/pull.rs
    title: "Full-snapshot transport and currently fixed read-only client-group identity"
  - resource: /crates/leaflet/web/src/home.rs
    title: "Existing authenticated identity/discovery observation and bounded Flight extraction"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Native fact/pull/push source map and read-only deployment evidence"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Executed fresh text/compiler/list evidence and explicitly synthetic stage-switch proof"
  - resource: /experiments/13-satteri-authoring/src/facts.rs
    title: "Reusable mapping evidence, 62-sibling ordering limit and createEntity argument-shape mismatch found on inspection"
  - resource: /experiments/13-satteri-authoring/src/text.rs
    title: "Pinned Yrs fresh materialization and independent-consumer conventions"
  - resource: /experiments/13-satteri-authoring/fixtures/garden.mdx
    title: "Previously executed native Key spelling inside a Markdown list item"
  - resource: https://github.com/rektide/leafbud/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/drafts/src/create.rs
    title: "Recovered pre-reset GET /new transport; source-derived and loopback tested, not deployment proof"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/createPublicationDraft.ts
    title: "Inspected publication draft action: permission check, mint, association and contributor defaults; not public release"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/createNewLeaflet.ts
    title: "Inspected standalone creation and authentication/homepage distinction"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/new/route.ts
    title: "Inspected GET route invoking standalone mint; GET is an effect here"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/createLeaflet.ts
    title: "Inspected root/page and initial blank-block seed shapes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "Inspected createEntity array, addLastBlock, moveBlock, metadata updates and destructive removal differences"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cachedServerMutationContext.ts
    title: "Inspected buffered-read ordering, explicit fact IDs, permissions, merge and flush behavior"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/push.ts
    title: "Inspected push request, skipped-but-acknowledged mutations and transaction limits"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/getFactsForRoots.ts
    title: "Inspected explicit-root read with depth three"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/attributes.ts
    title: "Inspected list, text, code, image and comment attribute ownership"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/getBlocks.ts
    title: "Inspected list display-number computation and parent/child semantics"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publications/moveLeafletToPublication.ts
    title: "Stage-two source check: explicit existing-draft association action and permission checks"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/addDraftToPublication.ts
    title: "Stage-two source check: association insertion, creator byline, optional destructive IDs and returned error"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "Stage-two source check: optional prior draft linkage, existing public identity and release-time association upserts"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cvr.ts
    title: "Stage-two source check: publication metadata extras come from persisted rows, not client-local tx.set"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/migrations/20260721000000_add_pull_data_cvr.sql
    title: "Stage-two source check: rooted outgoing-reference closure, token-scoped metadata, same-snapshot fact/ack read"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/CodeBlock.tsx
    title: "Stage-two source check: active code/language rendering excludes latent block/text"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/ImageBlock.tsx
    title: "Stage-two source check: active image payload/alt/layout rendering excludes latent block/text"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/blockTypeOperations.ts
    title: "Stage-two source check: native text-type changes clear stale heading-level/text-size attributes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/getIdentityData.ts
    title: "Stage-two source check: homepage/contributor discovery is not mint-attempt attribution"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/TextBlock/schemaVersion.tsx
    title: "Stage-two source check: known meta.schemaVersion semantics, not blanket authority to drop unknown roots"
---

# Recommendation: generate a complete body, keep the document

**Use one source-authoritative body materializer for both first and later saves.**
Compile the complete desired supported body into a fresh, disconnected native
page graph; verify it; update the existing entry fact to that graph; verify the
active editor and settle the current associations. Automatic source keys, native
draft creation, ordinary structural edits, pull and no-op repeats are part of the
same delivery. A text-only save is not its first useful product.

This is a deliberate trade: **stable local document/source identities and native
document/public identity; replaceable native body page/block/text identities.**
It revises the older ordinary-save continuity recommendation. It is a proposal
under the corrected brief's architectural freedom, not a claim that the operator
already accepted this identity trade. If retained block/page anchors are required,
choose the complete keyed-block alternative in §9 instead; do not return to an
existing-text-only deliverable.

Inspection started in `/home/rektide/src/leafbud`, version **0.9.0**, clean
working copy `5571f343` on parent checkpoint **`9a3a0dbb`**. The incoming name is
**Leafbud**; another session owns the rename, configuration and existing-data
conversion. Code names cited here describe the inspected Leafbud revision;
example commands use the incoming name. Archived Leaflet HEAD was independently
checked as **`600c7b5f1f7b31b66fb626c293cc104312a12caf`**. No remote writes, live
user-file reads/transformations or production edits were performed for this design.

This is the independent **first draft** for `leafbud-editor-plan-design`.
Only previous-wave designs informed the proposal. **Independence note:** after
this draft and its recommendation were written and checked, a broad corpus search
unintentionally returned two creation-related lines from the concurrently appearing
same-wave g file. No full peer read, comparison or revision based on those lines
was performed. This incidental exposure is disclosed to the parent; deliberate
peer review awaits the explicit join.

## 1. The complete experience, twice

### Start with writing, not remote identities

`leafbud create essay.mdx --title "A garden notebook"` creates a local file with
a random local document ID, draft lifecycle, generated-body ownership and one
linear Page scaffold. Binding, nodes and sync receipt are absent/empty. The
author writes ordinary Markdown inside the page:

```mdx
<Page name="main" kind="linear">

## First observations

The **beds** are ready. See the [map](https://example.test/map).

Water after planting.

</Page>
```

Title remains an independent frontmatter field. Page scaffolding is structural;
there are no required Paragraph/Heading wrappers. A hand-written file with the
same managed envelope is equally valid. The first profile keeps the current
explicit Page syntax rather than making implicit and explicit entry-page selection
another design problem.

`check` accepts this as valid authoring input and reports three missing assignable
block keys. It does not require fake UUID rows or contact Leaflet. `push` without
`--yes` explains the resolved destination, three key insertions, draft creation
and generated-body effects. It neither inserts keys nor mints anything.

### First executed push

1. Validate the whole source/profile and publication intent before effects.
   Resolve flags > explicit unbound document intent > workspace > global;
   explicit `publication: null` means standalone. An unresolved required target
   is a diagnostic, never a reason to create the wrong kind of draft.
2. Under the existing byte-change guard, insert three keys, say `h`, `p`, `q`
   here for readability; real generated labels are opaque `b-<uuid>` strings.
   Persist the prepared source and a compact mint-attempt record. The page's
   `main` key already exists. **There are still no confirmed native node rows.**
3. Invoke the appropriate native mint path (§4), record the returned token as
   soon as possible, read its actual root/entity-set/entry/publication association,
   and establish the coherent binding. Keep the original local document ID.
4. The returned seed is an empty heading or paragraph in a single page, not the
   author's first paragraph. Verify that seed and its metadata/defaults; do not
   treat it as a keyed authored block. Stage generation `G1` under the returned
   entity set. Generate page `P1`, blocks `H1/B1/C1`, their memberships and fresh
   text facts. `h/p/q` identify the source; `G1` selects this native output.
5. Read the staged inventory explicitly. Require exact supported meaning,
   membership/order/reference completeness and valid text. Recheck the old
   active seed/entry. Switch the **existing** entry fact `E` from seed page to
   `P1`; apply supported title/metadata. Read active content and metadata again.
6. Only after verification, record `main→P1`, `h→H1`, `p→B1`, `q→C1` plus their
   actual membership/text fact locators; settle sync; clear pending. The seed is
   detached, not deleted. This remains an unpublished draft.

The initial binding can be saved before body success; `sync` cannot. Therefore
an interrupted first save never means “unbound: create another draft.”

### Second writing session and push

The author inserts a new heading and paragraph **before `h`**, edits the link and
marks in `p`, splits `p` with a blank line, removes `q` with its Key marker, and
moves `h` after the split paragraphs. The old `p` marker stays on the first piece;
the second piece has no key. No node table is manually edited.

Preparation supplies keys `lead`, `intro`, `p2`. Existing confirmed rows still
describe `G1`; `q` is now a retired source association awaiting save, and the three
new keys have no confirmed rows. This is valid desired input.

After the conflict gate, the same materializer emits `G2` in order
`[lead, intro, p, p2, h]`. It does not infer split/move/delete operations:
that order and those block boundaries are the desired body. All new text goes to
fresh `G2` entities. Verification precedes changing `E` from `P1` to `P2`.
The token, root, entity set, entry fact and publication association are unchanged.
Successful read-back replaces the current association table with `G2` rows;
the `q` row is removed. Old `G1` objects remain detached.

For a join, the author removes the second block's Key and paragraph boundary,
leaving one block under the first key. A remaining orphan Key is a useful source
error, not a reason to guess which identity to discard. Reordering means moving
a block together with its marker. Ordinary type changes, such as adding `##` to
a paragraph, need no native type-transition algorithm under this engine.

`pull essay.mdx` immediately afterward is a no-op. A later supported web edit is
decoded into the same current file with current `G2` associations and source labels
reconciled by those associations. Editing that pulled file and pushing produces
the next verified generation under the **same draft**.

Repeating an unchanged push does **not** create `G3`, mint another token, emit
new text, move memberships, or rewrite a clean local file. Equality is checked
against freshly observed current content, not merely the last local input hash.

## 2. Ownership and identity contract

The default for newly created local drafts is a **generated entry body**, recorded
in the owned envelope. One owned policy value is enough; it is not an engine
plugin framework. It identifies the selected entry closure and explains that
ordinary body saves regenerate its internal objects.

| Identity/content | Contract |
| --- | --- |
| Local document ID and source keys | Stable across pushes/pulls; new keys assigned once; key rename retains its current association when performed coherently. |
| Token, native root, entity set, entry fact ID and position | Retained after initial binding. Missing or retargeted bindings are not recreated automatically. |
| Publication container and eventual public document identity | Retained; no release/linkage rewrite is a save operation. This draft delivery does not claim published-editor support is already implemented. |
| Ordinary body pages, blocks, membership/text facts, Yrs items | Replaced on each meaningful body generation, including unchanged paragraphs within that body. |
| Links between named pages in the source | Resolved together to the new generation. Repeated references to one named page share the same new page. |
| Literal URLs containing old native page/block anchors | Remain literal URLs; they are not remapped. Recognizable references into the replaced graph should be diagnosed in preview. |
| External incoming anchors and open tabs pinned to an old page | Do not follow the replacement. Old entities remain, so such references can continue to show old content. There is no complete global incoming-reference discovery claim. |
| Asset URLs | Retained as resource locators. Regenerating an image block does not duplicate its binary or authorize cleanup. |
| Polls, questions, answers, RSVP, subscriptions and comment anchors | Outside the first writable profile. Their presence in current owned content blocks regeneration, including unchanged objects; no recreation/reinterpretation loophole. |

For an imported file without generated-body ownership, an equal push remains a
no-op. Its first body-changing push must make the ownership change explicit;
reuse the accepted `--recreate-body` selection to adopt this one policy. It does
not resolve remote drift. Subsequent normal pushes use the recorded policy. This
is one engine with a clear adoption boundary, not incremental plus recreation
implementations maintained in parallel.

The first supported graph has exactly one linear root entry, as the current
observer already requires. All locally defined pages must be reachable from it,
with no cycles. Multiple unrelated root/navigation entries are refused, rather
than broadening observation into a general ownership slicer for this delivery.
Known root watermark and metadata are handled explicitly. Unknown meaningful
facts, marks, extras or roots block the operation before writing.

**Generation eligibility is stricter than today's readable projection.** The
current reader permits an opaque inactive `block/text` on images/code and does
not exhaustively interpret other named Yrs roots. A body-changing generation
must refuse those unrepresented payloads unless they receive an explicit supported
mapping. Do not discard them because the visible `BlockKind` compares equal.
An unchanged push can still verify/no-op. New clean image/code blocks generated
by this profile do not carry such latent data. This limit must appear in the
profile diagnostics for imported files.

No entity deletion, `removeBlock`, recursive cleanup or automatic orphan pruning
is part of save. Accumulated detached generations are a real remote-storage cost,
not a local history feature or a free backup promise.

## 3. Actual first profile

Call the new writable mapping profile something distinct, e.g.
`leaflet-drafting/1`, and give its receipt interpretation a new version. Names
are illustrative; the rename owner supplies namespace spellings. “Whole document”
means **the complete supported document**, not silently saving a supported subset.

| Content | Delivered behavior |
| --- | --- |
| Title | Required nonblank author field, independent of first heading/paragraph; first save and subsequent changes applied natively and read back. |
| Paragraphs/headings 1–6 | Create anywhere, edit, split/join, remove, reorder and convert between supported forms. Blank paragraphs use an explicit empty Run when Markdown has no block. |
| Inline content | Existing strong/em/code/strike/underline/highlight/link marks, exact Run escapes and hard breaks; create/edit/remove them. Extend experiment 13's smaller encoder to match the production reader's full supported mark set. No title-bearing links, mentions, footnotes or comment marks. |
| Quotes | Existing single-paragraph quote profile, including marks/breaks: create/edit/delete/move. Multi-paragraph/nested quotes fail before writes. |
| Code | Exact code text/language facts; normal fences accepted with current parser semantics, `Code` retained for exact trailing newlines/awkward payloads. No fence metadata silently dropped. |
| Rules | Normal Markdown rule; create/delete/move. |
| Ordinary lists | **Add flat bullet and ordered lists**, one rich-text paragraph per item, ordinary Markdown input/output, arbitrary positive starting number within the bounded integer range. Insert/delete/reorder items and convert paragraphs/items. Nested, task and multi-block items explicitly outside this first profile. |
| Images | Existing literal Image component, remote HTTP(S) source/fallback and optional integer dimensions/alt: create/edit/delete/move. No new file/blob upload or inline Markdown image lowering claimed. No resource fetch during check/preview. |
| Page definitions/links | Existing linear Page/PageLink syntax; create/delete/reorder blocks and add/remove referenced pages. All targets defined, reachable, acyclic. Deleting a page still referenced is invalid. Definition layout beyond the entry is not reading order. |
| Watermark | Existing boolean theme field, applied to the retained root; absent on an unbound source adopts the actual native default. |
| Description/tags/contributors | Context-aware handling below. Preserve supported unchanged native values; never infer that a source field was applied when its target has no storage for it. |
| Other rich objects/layout | Explicit whole-operation refusal. In particular, no copying a poll's displayed text into a fresh poll or reusing its votes under changed options. |

### Lists are small additional modeling work, not already implemented

Prefer a flat `ListItem { content, marker }` variant in the **existing** BlockKind,
where marker is bullet or ordered with effective positive number. Every item has
one key and one native block/membership. A Markdown list wrapper has no native
entity and needs no key. This avoids importing an arbitrary recursive list tree
merely to implement flat lists.

Native mapping is `block/type=text`, `block/text`, `block/is-list=true`, optional
`block/list-style=ordered`, and `block/list-number` for ordered numbers. Emitting
each desired ordered item's number is acceptable coarse normalization. Bullet
items use the native absent/unordered style convention. Source list start plus
ordinal determines ordinary Markdown numbering; arbitrary explicit marker numbers
after the first do not acquire nonstandard Markdown semantics.

The decoder must compute effective ordered display numbers following native
`getBlocks.ts::computeDisplayNumbers`, including ordinary absent-number defaults,
before producing this model. It must reject nested children, check-list flags and
nonsequential resets that cannot be faithfully rendered by the first list writer.
Keep raw-number distinctions in the conservative witness. This extends production
observation, parsing and round-trip writing; experiment 13 supplies mapping evidence,
not these production capabilities ready-made.

Per-item key syntax is already experimentally demonstrated:

```mdx
- <Key id="measure" />
  Measure the sunny patch.
- <Key id="notes" />
  Keep **notes** in order.
```

Automatic insertion uses parser item/paragraph spans and the actual marker's
indentation. It must prove the completed list parses to the same content and item
boundaries; inserting standalone Keys between list items is not an equivalent
operation. Loose formatting with one paragraph per item is allowed; a second
paragraph is outside the profile.

### Metadata must follow the real destination

Publication-associated drafts use `updatePublicationDraft` for title, description
and tags. Standalone drafts use `updateLeafletMetadata` for title/description;
that helper does **not** store tags. Initially reject nonempty standalone tags
as unsupported application intent before mint, rather than claim they were saved.
For publication descriptions, map “no description” to the native empty-string
form and normalize it consistently in editor equality/pull. Standalone removal
uses the helper's supported null.

Contributors are meaningful identity/permission data. Adopt the actual server
default on creation when the source omits that field (the publication action can
add the creating contributor). Preserve and verify unchanged values thereafter;
explicit contributor-list changes are outside this writing profile. Do not clear
the server default by conflating omission with an explicit empty list.

Use authoritative stored metadata for bound authored titles. Current import falls
back to the first text/heading and treats the string `Untitled` as a placeholder;
that is unsuitable for verifying an explicitly authored title, particularly after
inserting a new first block. Keep derivation as an import fallback when metadata is
actually unavailable; do not make it the saved-title contract. A title-only save
updates metadata and verifies it **without generating another body**. This small
metadata path is not a second structural engine.

## 4. First native binding is essential, and still has protocol work

### Two destination adapters, one body engine

**Standalone:** the inspected `GET /new` route calls `createNewLeaflet`, returning
a redirect carrying the token. Recover the pre-reset client deliberately: do not
follow the redirect, accept only the expected same-origin editor locator, and
treat the GET as a write. Validate authenticated identity first; the native action
can mint even when a cookie is missing/invalid, because authentication chiefly
controls homepage attachment. Presence of a cookie alone is not authentication.

**Publication-associated:** call `createPublicationDraft(publication_uri, "doc",
{})`, the action used by the native dashboard. It checks owner/confirmed contributor,
mints a text-seeded Leaflet, inserts `leaflets_in_publications` with `doc:null`,
and may add a contributor. This is draft creation, not publication. Do not confuse
it with `createPublicationDraftLeaflet`, which creates a publication's homepage.
`GET /new` has no publication parameter that establishes this association.

The publication action's business semantics are source-verified. A production
HTTP adapter still needs **current Next action-reference discovery, request
encoding, return decoding and a deployed proof**. Reuse an established compatible
client/decoder if available during implementation; otherwise keep a bounded
adapter for this one literal-argument action. Do not build a general React Flight
runtime, hard-code a stale action ID, or claim the current homepage RSC extractor
already invokes actions. Discover from the actual build's client action reference;
an ambiguous/missing reference blocks mint. Exact discovery/response fixtures are
part of batch B, before authoring is called delivered.

Both paths then read `get_leaflet_data` and a full pull. Verify expected origin,
token/root/entity-set, readable **and writable** rights, exactly one entry and
actual publication association. Preserve a returned token even if later association
verification fails: native mint and publication-link insertion are not one proven
all-or-nothing operation. A null action result or missing link never authorizes
falling back to standalone creation.

The returned empty seed may be replaced only after checking its expected supported
empty content. If another writer already filled it, report remote content and
require the established content decision; the fact that mint was recent is not
proof the seed is still untouched.

### Local identity is not the token-derived import ID

Current `import::build` derives `state.id` from origin/token; `same_identity`
compares that generated ID to the local document's ID. A genuinely new local
random ID would fail its own post-save observation today.

Separate observation's **supplied local namespace** from import's initial identity
choice. Import can still choose a deterministic ID on first adoption. Existing
bound-file observation must use and retain its established local ID while checking
the actual origin/token/root/entity-set/entry/publication tuple. This is an identity
bug to fix for disk-first authoring, not permission to weaken remote binding checks.

## 5. Source/model changes: desired content is allowed to be ahead

Keep `Document` as the one current semantic model. Extend it and its validators;
do not introduce a planning document AST or JSON master.

1. **Binding and sync become optional with explicit invariants.** An unbound draft
   has a local ID, source and intent, but no fake native locator. A newly bound
   pending draft can have a verified locator without a sync receipt. Unknown or
   partial locator groups remain invalid. Lifecycle remains draft here.
2. **Publication intent distinguishes absent, null and URI.** The current `Option`
   collapses absent/null and the field comment assumes confirmed association.
   Parse presence explicitly. After binding, the actual association is authoritative;
   ambient defaults never retarget it. Changes require a separate identity decision.
3. **Keys may initially be missing.** Lowering records assignable nodes/spans rather
   than failing at `block has no source Key`. Either optional keys in the same model
   or a small parsing result with missing-key locations is sufficient; do not use
   temporary strings as confirmed node IDs. Prepared input has unique valid keys.
4. **Confirmed rows need not equal today's source key set.** Existing rows for
   retained source keys remain locators; new keys have no row; removed keys' old
   rows survive only until the operation resolves. This is the current accepted
   association set, not a saved body. Validate rows independently of desired
   presence. A supplied malformed/retargeted row is never classified as “new.”
5. **Successful observation uses a stronger check.** Every current source node has
   exactly its verified association; all required occurrences are accounted for;
   no retired row survives final settlement. Strict confirmed-output validation
   belongs here rather than in every authoring parse.
6. **Automatic keying is a focused byte patch.** Reuse parser source spans, retain
   author YAML/prose/existing markers and line endings, insert only missing markers,
   and strict-reparse before atomic file replacement. Duplicate/orphan keys fail;
   copied keys are never silently “fixed.” Confirmed push persists keys before any
   remote effect. `tidy --keys` exposes the same local helper.
7. **Receipt/binding patching must support state transitions.** Current `update`
   assumes both endpoints have the same binding token and parses under bound-only
   rules. Add narrow owned-envelope updates for unbound→pending→bound→verified,
   preserving body and unrelated YAML. Retain the local-byte guard after awaits.

### Equality and receipts without a new comparison model

Retain two questions: **what changed since last correspondence?** and **do today's
endpoints agree?** Scoped fingerprints answer them; neither reconstructs history.
Old receipts keep their original interpretation until a real supported settlement.
Adding list/presence/identity semantics warrants a new receipt profile, not silently
changing `leaflet-editor/2` in place.

The desired projection must encode new/unbound source identities explicitly,
e.g. a tagged local-key identity, rather than letting every missing row serialize
as null. Use confirmed associations where present. Ignore retired rows when
projecting desired content; retain them for current ownership validation until
settlement. This lets additions, removals and PageLink targets compare truthfully.

Use the current graph plus the current table to reconcile source labels. For a
generation, the materializer already returns the prospective new associations;
verify a clone of the same desired Document using those rows against decoded staged
content. After activation, compare that expected Document to the fresh active
observation. No arbitrary old/new tree matching or operation diff is needed.

Separate the body equality check from supported metadata equality so title-only
saves do not generate a body. A wholly equal current editor produces no mutations.
Unsupported raw state still blocks a body-changing operation even if visible text
compares equal. Unknown baseline plus differing endpoints uses the existing
explicit-decision policy, never a fabricated first-save baseline.

## 6. Materialization and transport: concrete native work

### Pure materializer

Inputs: prepared current Document, verified binding, fresh attempt/generation UUID,
resolved remote resource locators. Output: transient native mutations, generated
entity/fact inventory, prospective Node associations and a short effect summary.
No serialized executable output is stored.

Derive UUIDs from `(generation, source-key, role)`, using separate roles for page,
block, membership and attribute. Do not derive durable source keys from ordinals,
content or generation. Different generations have disjoint ordinary entity/fact
IDs; references inside one generation resolve in a first allocation pass.

Use the existing Yrs pin **0.28.1 / `e0b476b0…`** for fresh text: UTF-16 offsets,
`prosemirror` fragment, one paragraph, supported attributed text and actual
`hard_break` elements, compatible schema metadata, one full initial v1 payload per
new text entity. Decode it with the production reader before transport. Production
uses Yrs, not the older hand-written binary encoder.

Crucially, **fresh text is sent only to an absent generated text fact**. Reencoding
the same desired text after restart creates new CRDT identity; it is not a replay.
An existing equal staged text is left unwritten after resolution of earlier
requests. Different/incomplete existing staged text blocks activation; do not
merge an independently regenerated paragraph into it.

### Native operation sequence

1. `createEntity` for all planned entities in bounded requests. **Its RPC mutation
   takes an array**: `args: [{entityID, permission_set}, ...]`. The context method
   takes one object; experiment 13's `staged_mutations()` accidentally uses that
   context shape at the RPC boundary. Correct it when promoting the idea.
2. `assertFact` for `page/type`, block type/payload/scalars and page references,
   with deterministic explicit fact IDs. One payload per generated text fact.
3. For each fresh page, append its desired children in order using native
   **`addLastBlock { parent, factID, entity }`**. The native helper uses
   `generateKeyBetween`; buffered reads see previous writes. This avoids promoting
   the experiment's 62-sibling `a<digit>` helper or writing a local fractional-index
   implementation. Verify returned order, not fabricated expected position strings.
4. Read all planned entities via `getFactsForRoots` in bounded root batches.
   Query every generated entity as an explicit root; a depth-three page read is
   insufficient for arbitrary referenced-page depth. Deduplicate returned rows by
   fact ID and reject contradictory duplicate observations.
5. Verify the complete generated inventory: expected entity attributes and cardinality,
   exact membership IDs/targets, order, page links, text meaning and absence of
   unexpected content on generated entities. Do not call the root-oriented
   `observe()` directly on a staged graph lacking a `root/page`; factor a small
   page-closure reader/validator from it or explicitly supply a transient entry
   context. A synthetic entry is not a remotely confirmed fact.
6. Immediately before activation, re-read active editor/binding and confirm the
   preflight witness still applies. Verify staged state remains complete. Update
   exactly existing entry `E` with `assertFact`, retaining its root and position,
   pointing at the new main page. Apply supported metadata/root-field changes.
7. Full pull plus metadata read verifies the actual active body, association,
   metadata and protected root/publication state. Only then settle the source.

Native `assertFact` supports explicit cardinality-many fact updates, so the entry
switch changes one existing row. The server request transaction and token lock
do **not** make the whole save atomic: individual mutations can be skipped and
acknowledged, and `runOnServer` metadata effects use a separate Supabase client.
Metadata/title may be partially applied even when body activation or its response
fails. Verify the combined result and keep uncertainty visible.

For retries, append helpers are not intrinsically repeat-neutral: calling them
again with new mutation IDs can change positions. Native request IDs plus exact
payload reuse handle same-process replay; recovery must observe, not rerun the
materializer's append sequence indiscriminately.

### Minimum receiving seams

```text
document/model: optional coherent binding/sync; source keys; pending identity data;
                existing BlockKind + flat list items; scoped equality
document/mdx:   permissive authoring identity validation -> precise key patches;
                checked round-trip writer; owned-envelope transition patch
leaflet/editor: fresh text/facts; staged inventory validation; strict eligibility;
                page-closure decoding + current associations
leaflet/web:    mint adapters; push; explicit-root reads; acknowledgment-aware pull
workflow:      create; resolve target; prepare; observe/gate; stage/activate/verify;
                settle or expose pending; existing guarded pull
cli:           accepted create/check/tidy/push verbs and concise effects/outcomes
```

The important future seam is `materialize/prepare_body(desired, observed, binding)`.
It returns transient native work plus expected associations. A later keyed writer
could replace the body implementation without changing source, metadata, transport,
conflict or receipt ownership. Do not build strategy traits, two engines, a general
ScopeOp algebra or an executable plan file to preserve that option now.

## 7. Failure and concurrency behavior

Before the first remote **effect**, persist compact **identities and evidence**, not
operations: attempt ID, resolved target, source-input hash, mapping/compiler profile,
phase, known token/binding, generation and old/new entry targets where known, request
client/group IDs and highest issued mutation ID. Generated inventory IDs can be
derived from the still-matching source, profile and generation. A changed or
unavailable compiler profile blocks reconstruction. Retain only this unsettled
attempt and current confirmed associations; no successful-generation history list
is necessary. Read-only observation and previews do not write pending markers.

Use one outstanding write request at a time. Keep exact bytes/IDs in memory for
an exact retry. Advance the durable phase/request high-water mark before sending
its effects; a crash there conservatively means “may have been sent.” Pull must
accept the attempt's client group and expose typed `lastMutationIDChanges`, instead
of always using `leafbud-read-only` as it does now.

| Observed interruption/outcome | Behavior |
| --- | --- |
| Mint response lost, no token known | Stop with uncertain creation. Do not retry mint automatically or choose the newest discovered draft by title. Discovery can supply candidates; explicit selection establishes a binding decision, not proof of exactly-once creation. |
| Token returned, binding read or local write fails | Retain/report the known locator and attempt; never return to ordinary unbound mint logic. If local bytes changed, report the locator without overwriting the new file. |
| Stage response lost | Resolve issued request IDs through acknowledgments and explicit generated-root reads. An absent/lower ack is not proof the request will never arrive. Do not reencode fresh text while it may still arrive. |
| All stage requests processed, inventory incomplete | Report failed/partial staging; old body still selected. With earlier requests conclusively resolved and no activation issued, a later deliberate push can abandon that generation and stage a new one. No deletion of the old stage. |
| Stage complete and all prior requests resolved | Revalidate source, active witness and staged inventory before issuing activation. Recovered equal staged text is not rewritten. |
| Activation response lost | Inspect actual entry and acknowledgments. Desired target plus verified full current content/metadata can settle. An old target with unresolved ack remains uncertain; it is not permission to switch another generation. |
| Activation processed but skipped/partial/diverged | Report current state. Processed does not mean applied; do not resend acknowledged IDs expecting different application. A new deliberate attempt requires resolved prior requests and a new current conflict decision. |
| Source changed during an await | Do not write old receipts over it. Keep/report pending evidence; resolve the remote attempt before taking new content as another save. |

Initial recovery may stop on ambiguous cases. It must support automatic observation
and settlement of the common “request completed, response lost” case when the
source still matches and all issued writes are resolved. Cross-process replay of a lost executable payload is not
promised. A small explicit binding-recovery UI for unknown mint outcome needs a
receiving CLI decision; no algorithm can infer that missing token safely from a
local hash because the native mint action accepts no correlation/idempotency key.

No source-only method can prevent a web edit arriving after the final preflight.
Switching a page can leave that edit on the old now-detached graph. Comparing the
old graph again after activation can detect some such races, but cannot close the
window or prove there were none. Do not make that additional read a false guarantee.
The truthful contract is drift checks, explicit take-local choice for known drift,
and verification of **achieved current meaning**, not preservation of every unseen
concurrent intent. Users needing active shared editing should choose the keyed
alternative and still face non-CAS limits; it is not solved by full-state encoding.

Previous-wave executed probes specifically show that deleting an XmlText child
can hide a concurrent edit while read-back equals desired, and independently
compiling twice from a stale base can duplicate text. Those results invalidate
both “equality proves no loss” and “same intention is an idempotent replay.” This
design does not rerun those probes or claim generation eliminates the analogous
entry-switch race.

## 8. Acceptance: executable cases, not a text-only substitute

Implement these as offline CLI/workflow fixtures over a source-faithful loopback
native service plus real Yrs/Yjs text consumption. The test double must model
permissions, explicit IDs/cardinality, text merging, skipped acknowledgments and
rooted visibility; a map that simply overwrites text would prove the wrong thing.
Use the existing independent Yjs/y-prosemirror consumer for wire compatibility.

| Case | Required assertions |
| --- | --- |
| **A — local start** | `create` needs no auth/network; edit title and add unkeyed heading/prose/marks/link. `check` accepts missing assignable keys; preview leaves bytes/mtime and remote unchanged. |
| **B — first save, publication** | Explicit/config target resolved once; exactly one native mint; actual association/rights/entry returned; no fake nodes before verification; independent local ID retained; full active body matches and public document link remains null. |
| **C — first save, standalone** | Explicit null overrides configured publication; `/new` redirect is not followed; title persists independently of body; unsupported standalone tags diagnosed before mint. |
| **D — insert at every boundary** | Add paragraphs/headings before first, between existing blocks and after last; automatic keys inserted once; second save retains token/root/entry but selects new complete generation. |
| **E — ordinary revisions together** | Edit text/link/marks/breaks, split, join, delete and reorder in one edit. One complete desired body is activated; removed keys retired; no visible duplicate old body. |
| **F — read and repeat** | Immediate pull and push do no writes; same remote token/generation and exact local bytes/mtime. A supported web edit pulls to same source labels, then can be edited and pushed again. |
| **G — profile content** | Quotes, exact code, rules, URL images, PageLinks and shared named pages round-trip; flat bullet/ordered lists including start 3, item insertion/reorder and >62 siblings preserve order. Unicode, empty text, Run newlines versus hard breaks and underline/highlight survive the independent consumer. |
| **H — source/model validity** | Missing keys/new rows/retired rows accepted at appropriate stages; duplicate/orphan keys, dangling/unreachable/cyclic pages and retargeted existing rows fail. BOM/CRLF, Unicode offsets and key insertion in list items preserve unrelated bytes. |
| **I — identity/metadata** | Title-only change creates no generation; inserted first block does not rename title, including explicit `Untitled`; metadata defaults settle once; key rename/definition layout do not cause a body save; changing ambient target cannot retarget a bound draft. |
| **J — unsupported/protected** | Poll/comment/task/nested-list/canvas/unknown attribute/extra or latent unmapped text blocks the entire body write. No partial supported subset is sent. No entity/blob cleanup; old generation and unrelated protected root/publication state survive. |
| **K — drift** | Remote-only/both-changed needs decision; explicit take-local reobserves; late entry/rights/association change blocks activation when observed. Simulate an unseen old-body edit and assert the report never claims CAS or no lost concurrent intent. |
| **L — failures** | Interrupt mint/stage/activation/receipt phases; model throw-skip-ack, ignored writes, lost response, delayed request, changed source and process restart. Known binding reused; no blind remint, stale recompile or premature sync receipt. Verify completed lost-response settlement and unresolved-state stopping. |
| **M — exact no-op under native variation** | Equal supported text encoded differently does not generate; local formatting/key spelling alone does not generate; raw unsupported drift is not silently accepted as safe regeneration. |

The **deployed acceptance** is a separately authorized throwaway draft in each
supported creation context, executing A→B/C→D/E→F, with native browser inspection
of body order/marks/title and repeat no-op. This design performs no such writes.
Offline evidence can justify the implementation; only the deployed probe can
establish that the actual mint/action/stage/read/switch path works there.

## 9. Why this default; the serious alternative

All strategies pay for local authoring validation, key preparation, target/mint
handling, supported mapping, conflict evidence, pending attempts, native transport
and read-back. Generation is not “just emit JSON,” and keyed editing is not “just
add an op builder to equality.” Their additional mechanisms differ:

| Strategy | Additional machinery beyond shared obligations | Consequence |
| --- | --- | --- |
| **Selected: whole owned-body generation** | One fresh emitter; explicit staged inventory reads/checks; one entry activation; complete association rebinding; body-continuity ownership policy | All supported structural edits follow from emission. Old active body survives staging failure. Body anchors replaced and detached storage accumulates. |
| **Complete keyed blocks with coarse text** | Retained/new/removed occurrence matching; changed field ownership and stale-attribute removal; load/edit existing Yrs; missing-text/type-change cases; membership movement/detachment; aliases/shared-entity constraints; partial active graph verification | Better page/block continuity, unchanged objects remain untouched. Useful and feasible, but several operation families need semantics and tests beyond whole-document equality. |
| **Stable pages, replace all their children** | Per-page association matching; per-page membership retirement and reattachment; multiple visible page transitions; same fresh emitter | Keeps page references, loses blocks, but lacks the single entry boundary. Not an obvious complexity win over either coherent endpoint. |
| **Full character/XML/tree reconciliation** | Correspondence heuristics and retention policies on top of keyed structure | More continuity than this low-volume drafting contract requires. Not selected. |

This is a conceptual inventory, not invented line counts or timing estimates.
Generation's distinctive cost is staging/activation/rebinding and broad identity
churn. Keyed editing's distinctive cost is several correctly owned mutation
families and their interactions. Neither one removes the first-binding protocol
gap or the source-validation work.

I favor generation here because the mandatory product is **full desired-body
creation and subsequent ordinary revision**, the existing fresh text/fact evidence
already covers a useful body profile, and the operator expressly permits coarse
page effects. It avoids making new headings, list items, type changes and joins
each wait for a separate mutation family. The strict no-interactive/no-comment
profile makes the initial anchor trade explicit and bounded rather than promising
to preserve objects the model cannot represent.

### If block continuity wins the selection

The credible alternative is a **complete** keyed save, not the withdrawn narrow
proposal. Retain pages/block entities; create missing nodes; detach omitted
memberships; update supported scalar/type fields; load each changed existing text
through Yrs and rewrite its supported content coarsely, retaining compatible
containers where straightforward. Use one full loaded-and-edited payload per fact.
For changed page order, move desired unique occurrences to the end in source order
with native helpers, or use fact-ID `addLastBlock` reparenting where verified.
Skip ordering entirely when equal. No minimal move optimizer is necessary.

That alternative needs explicit supported type transitions and inactive-field
policy, new empty-text initialization versus existing-state handling, occurrence
ownership and shared-entity checks. Repeated aliases cannot receive contradictory
desired values. It can use the same create/key/mint/receipt batches below, replacing
the body-engine batch. It should ship the **same acceptance matrix**, especially
new first blocks, split/join/delete/reorder, and first native binding. Do not build
it beside generation now merely to postpone the decision.

I would change my recommendation if durable native page/block anchors are a real
near-term requirement, if the source-owned body policy is unacceptable, if detached
generation growth proves material, or if deployed explicit staged-root reads fail
while ordinary block operations work reliably. An interactive-object requirement
also reopens the choice: stable reuse exceptions are possible, but a growing
exception registry would erode the simplicity that motivates generation.

## 10. Implementation batching toward one delivered workflow

These are proposed batches for parent coordination, **not new tickets created by
this session**. Existing IDs below are the commission/prior design's vocabulary;
the parent should reconcile their descriptions with live Beads before assignment.
Four batches can span multiple sessions. Their integration exit is A→B/C→D/E→F,
not whichever internal checkpoint lands first.

### A — Make a draft genuinely local and editable

Bundle local `create`, the authoring State/validation extension, tri-state intent,
flat-list model/parser/reader/writer and automatic `tidy --keys` preparation.
Adapt imported/bound observation to retain a supplied local document namespace.
Tests prove unbound files and bound files with new/removed blocks are valid without
manufactured rows. **Recommend g** for this specified source/model work, with parent
review at the desired-versus-confirmed boundary. This is usable local groundwork,
not native drafting delivery.

### B — Prove first binding and the actual write protocol

Bundle both mint destination adapters, push/explicit-root/ack reads and loopback
contract tests. Read the pinned helpers as production contracts rather than
promoting simulator output blindly. Settle the publication action-reference/Flight
adapter and prove the returned association. **Recommend Sol high** for this batch:
the substantive judgment is uncertain native action transport, not routine CLI
plumbing. An authorized protocol probe can happen here, with user approval outside
this design. If it fails, resolve it; manually creating an empty draft is a useful
diagnostic technique, not an acceptable replacement for the commissioned first push.

### C — Complete the generated-body save

Recast the body implementation scope behind `leafbud-editor-structure` together
with the needed fresh-text work formerly associated with `leafbud-editor-text`.
Build the full profile emitter, native append ordering, staged verification and
entry activation. Include title/metadata ownership. **Recommend g** once B has
verified wire shapes; parent reviews staging/ownership before remote execution.
An offline text encoder checkpoint is fine, but paragraph/heading addition,
deletion, order and PageLink/list handling must land in this same delivery arc.

### D — Close the lifecycle and demonstrate the product

Join `leafbud-editor-conflicts` and `leafbud-editor-recovery` with the push
workflow/CLI, source-preserving binding/receipt settlement, pull integration and
the complete acceptance matrix. **Recommend Sol high** for the uncertainty and
receipt boundary, with g suitable for specified CLI/rendering fixtures if assigned
in a later session. A verified completed request with lost response must settle;
an unresolved in-flight request must stop honestly. Parent review then commissions
the throwaway **create → push → structural revision → push → pull → no-op** run.
Only that joined loop warrants saying normal drafting/native push is delivered.

No batch implements public release, automatic cleanup, local image upload or a
general multiplayer editor. Those boundaries do not exclude any mandatory
paragraph/heading drafting action in the corrected brief.

## 11. Remaining decisions and proof points

1. **Product selection:** accept generated-body ownership for newly created local
   drafts, including replaced internal anchors, or select the broad keyed alternative.
   This is the main parent/user architecture decision; do not quietly inherit the
   old continuity promise while shipping generation.
2. **Publication mint invocation:** exact current build reference discovery and
   response decoding are not yet exercised. Source shows the necessary action,
   arguments and semantics. Completing this is on the critical delivery path.
3. **Deployed stage/read/entry behavior:** native sources support it; experiment
   13's simulator does not establish authorization, read completeness, partial
   metadata outcomes or real browser behavior. Check those on the authorized probe.
4. **Profile refinements:** native list-number/display parity, remote image handling,
   metadata-null normalization and strict latent/unknown-content eligibility need
   executable fixtures. These are mapping proofs, not invitations to another parser
   comparison or structural framework.
5. **Unknown mint recovery interface:** choose the smallest explicit adoption path
   for a user-selected candidate. Preserve the uncertain attempt and explain that
   candidate selection is not a retroactive exactly-once guarantee.

### Connections and evidence standing

The [corrected brief](drafting-push-brief0.gpt6a.md) controls delivery scope. The
[accepted interface](authoring-interface0.gpt6a.md) supplies commands and ownership;
this proposal changes its earlier default internal-continuity aspiration explicitly.
The [handoff](authoring-handoff0.gpt61s.md) and
[recreation design](mdx-recreate0.gpt6a.md) are generation precedents, while the
previous [Astra](native-edit0.gpt6a.md) and [g](native-edit0.glm53m.md) designs supply
useful native/Yrs and retry evidence, not scope constraints.

[Experiment 13](../experiments/13-satteri-authoring/REPORT.md) is executed compiler
and independent-consumer evidence plus a bounded simulation. This session inspected
its implementation and found promotion details needing correction; it did not rerun
the experiment or claim its simulator is the server. The native source links above
are revision-bound source evidence. The current [status receipt](status-witnesses0.gpt61s.md)
and [human diff receipt](human-diff0.gpt61s.md) establish reuse boundaries; neither
claims a write planner already exists. This draft owns no production adoption,
live result or peer consensus.

## 12. Authorized cross-review and revised stance

**Review target:** g's corrected independent draft at
[`7c0e6f9501bf8ae52280aef9f37932804703902c`](https://github.com/rektide/leafbud/blob/7c0e6f9501bf8ae52280aef9f37932804703902c/design/drafting-push0.glm53m.md),
read through `git show`, compared with my first draft at
[`b5e66475c66a36363d47e4f0c9e198287d59fcf9`](https://github.com/rektide/leafbud/blob/b5e66475c66a36363d47e4f0c9e198287d59fcf9/design/drafting-push0.gpt6a.md).
The peer's browser-creation requirement was already corrected before this review;
I assess its corrected automatic `/new` path, not that superseded requirement.
No parallel evolving peer addendum was read. The checkout still resolved as
`/home/rektide/src/leafbud` during this review. Sections 1–11 remain the original
argument; **this addendum controls where it changes that argument**.

### 12.1 Revised verdict: keep generation; correct corpus eligibility

**I retain one generated-body engine as the recommended default**, with actual
publication-context binding on first push and automatic source identity preparation.
I withdraw the blanket generation blocker for known inactive `block/text` on
currently image/code blocks. Generated-body ownership should deliberately carry
the **current supported body**, while leaving that known inactive conversion
residue on the old detached entities. The preview and first imported-document
adoption must say so. Unknown active semantics and interactive relationships still
block a save. This correction makes generation usable for the already-imported
kind of documents, instead of holding ordinary prose editing hostage to latent data
the native renderer itself does not display.

The peer offers a credible alternative: create all new blocks, rewrite changed
existing text using loaded Yrs, detach omitted memberships and normalize order.
It can deliver every mandatory paragraph/heading action. **I do not claim that
generation is the only broad solution, or necessarily fewer lines overall.** Its
advantage is one body-construction algorithm independent of the old graph's text
topology/type-transition cases, paid for by explicit staging, activation and
rebinding. That remains a good fit for the stated low-volume, coarse-effect-tolerant
product. If preserving native body anchors outweighs that simplicity, select the
complete keyed alternative, not a narrower text checkpoint or two default engines.

The original first and second save walkthroughs still apply. The changes are:
publication intent is resolved and realized before calling first binding complete;
imported image/code conversion residue is explicitly retired rather than blocking;
request resolution belongs in the basic save lifecycle; metadata/read-back use the
actual destination storage; and source/default adoption must not quietly invent
new author intent. Flat lists remain in my proposed delivery profile.

### 12.2 Publication context: the peer's new mint is necessary but insufficient

G now correctly makes browser-free native minting required. However, its corrected
Story 2 says config never chooses a publication and retains a source publication URI
only as release intent while recording `binding.publication=null`. That is a new
product policy, not the existing configured-context contract:

- The corrected brief requires resolving publication context from explicit
  intent/config on first push.
- The accepted interface distinguishes absent unbound publication (inherit), null
  (standalone) and URI (selected publication); once bound, the actual association
  wins over defaults.
- The shipped config and checkout receipt explicitly retain publication defaults.
  Import uses the observed association, rather than retargeting imported files;
  this does not imply new drafts ignore the default.

`GET /new` has no publication argument and produces a standalone draft. It cannot
by itself fulfill a selected publication context. A source URI retained as “future
intent” is truthful only if the user accepts that changed behavior explicitly.
It does not satisfy the present commission merely because origin/auth were honored.
Also, `create` should not unconditionally insert `publication:null`: doing so
would suppress configured publication inheritance before push even runs.

**Recommended resolution remains:** preserve omission in an unbound source,
resolve the target once at execution using the accepted precedence, record that
resolution in pending, and call `createPublicationDraft(uri, "doc", {})` for a
publication target; use `/new` for resolved standalone. Read actual linkage and
persist it as the binding. An inherited target can be materialized into the author
field when binding completes under the documented first-binding field update; an
explicit authored null/URI must never be silently overwritten. After binding,
ambient config cannot retarget the file.

#### A later association path exists, but it does not remove the action work

I checked the exact alternative rather than assuming association must occur only
during mint. Native `actions/publications/moveLeafletToPublication.ts` accepts
`(leaflet_id, publication_uri, {title, description}, entitiesToDelete)`; after
publication owner/contributor checks it calls `addDraftToPublication`:

1. Insert `leaflets_in_publications` with `doc:null`, title and description.
2. Add the actor to the draft byline, including an owner.
3. Delete `entitiesToDelete` when nonempty.

Thus `/new` followed by this action **during the same first push** is a genuine
association route. It must pass `[]` for deletions, retain the minted token on any
later failure, verify actual association, and account for its byline defaults.
The wrapper does not return the helper's insert error as a useful result, making
read-back especially important. It still needs a Next server-action adapter and
introduces an extra externally visible transition. `createPublicationDraft` is
closer to the native new-publication-draft workflow and adds only a non-owner
contributor by default; I keep it as the selected adapter. Do not build both paths
speculatively.

The native publisher also reads an optional existing draft link and later upserts
`leaflets_in_publications` after successful public output. Therefore I do **not**
claim publication can never establish association later. That path crosses the
public-release boundary and leaves the working draft unassociated in the meantime.
It is a different save/release contract, not proof `/new` already implemented
configured draft context. If a new action adapter is difficult, it remains required
implementation/proof work rather than a reason to demote publication context.

### 12.3 Metadata: separate persisted rows from local browser cache writes

The source confirms the following receiving table:

| Native context | Durable write/read route | Consequence for push |
| --- | --- | --- |
| Pure standalone draft | `updateLeafletMetadata` updates title/description on permission tokens sharing the root; `get_leaflet_data` observes those fields. | Title/description supported; no native tags storage established here. |
| Publication-associated draft | `updatePublicationDraft` updates its `leaflets_in_publications` row; full pull's `publication_*` extras are built from that row. | Title, description and tags are real editor metadata before public release. |
| Standalone published editor | `updatePublicationDraft` falls back to `leaflets_to_documents`; pull can expose its tags/title/description too. | A distinct lifecycle, outside this draft delivery; “tags exist only under publication linkage” is too broad. |
| Contributors | `leaflet_contributors`, with explicit permission checks on addition; pull returns `draft_contributors`. | Native creation defaults are actual state, not a local formatting field. Preserve/adopt them deliberately; no arbitrary byline edits in this profile. |

`updatePublicationDraft` has a `runOnClient` branch calling `tx.set` for title,
description, tags, local published date and preferences. The server context's
`runOnClient` does nothing. In particular, sending that mutation on a pure draft
with neither linkage row cannot create durable tags merely because a browser would
show optimistic `publication_tags`. Nor does its client-only
`publication_local_published_at` assignment establish server-side date storage.
The SQL pull obtains metadata rows from `leaflets_in_publications`, falling back
to `leaflets_to_documents`; `buildExtras` then exposes them. These are source facts,
not simulated `tx.set` promises.

G's sole `updateLeafletMetadata` route is insufficient for a publication-associated
import: an existing publication title can continue to override the changed token
title in the observation. Its suggestion to accept permanent `local-changed` for
unsaved tags also violates the complete save/read/repeat-no-op contract if those
tags remain part of editor equality. Either apply supported fields in their actual
context or define them as genuinely non-editor intent in a new explicit profile.
Do not settle a successful editor receipt while differing declared editor content
is left for future publication.

I keep my narrower initial policy: apply tags on associated drafts; diagnose
nonempty tags on pure standalone drafts before mint/write. This is not a reason
to block paragraph/headings in an otherwise valid standalone document. A future
explicit release-only tag field is possible, but silently reclassifying the current
flat field is not this implementation. Title/description/watermark have native
write mappings in both relevant draft contexts.

One further implementation condition: publication mutation and pull SQL address
the current permission-token ID. Existing metadata discovery can see association
on sibling tokens. A sibling association alone is not proof that a metadata write
through the supplied token will hit it. First mint returns the associated token;
for imports, resolve the actual association-bearing writer token with verified
rights or report the unsupported identity case. Do not update a shadow token title
and call it a publication-title save. This belongs in binding/read-back tests.

### 12.4 Existing corpus: known inactive state is not unknown active meaning

The [checkout receipt](draft-checkout0.gpt61s.md) records 15 initial successes and
12 initial failures involving **both** exact JSX boundary handling and inactive
text semantics; after fixes, all 27 observed drafts imported. It does not give a
per-cause count. I cannot infer that all 12, or all 27, contain latent text. It does
establish that the issue is real in the existing corpus, not a hypothetical edge.
My first-draft eligibility rule would block a body-changing push in affected files,
even when the only authored edit was adding a paragraph elsewhere. **I withdraw
that rule for the precisely known inactive case.** No private files were inspected.

The production fixture `visible_code_ignores_but_witnesses_native_inactive_richtext`
converts a text block to code, retains its old `block/text`, displays only
`block/code`, keeps `inactiveTextFact`, and proves that changing even opaque latent
bytes changes the remote witness. Native `CodeBlock` reads code/language;
`ImageBlock` reads image/alt/layout. Neither uses `block/text` as its current
displayed content. This is enough to distinguish:

| Category | Revised generation policy |
| --- | --- |
| Current represented code/image content | Recreate exactly from the source's supported active fields. |
| Known inactive `block/text` on a currently code/image entity | Keep it untouched on the old entity; omit it from the new entity. Report its retirement as part of generated-body ownership. No decoding requirement solely to establish that this renderer does not use it. |
| Active rich-text state, including known schema metadata | Decode/validate supported meaning, then emit fresh supported state. Do not erase meaningful active nodes/marks through normalization. |
| Unknown attribute, unknown meaningful active XML/root/meta state, comment or interactive relation | Fail before generation until its role has a supported mapping or an explicit separately selected treatment. “Not displayed by today's projection” alone is insufficient. |

The old inactive payload may represent words the author could recover by converting
the **old** block back to text. This proposal does not promise that resurrection
behavior on the new image/code entity. That is the actual loss of continuity;
“the bytes still exist remotely” does not make it disappear. The source owns the
current body, not all former renderings of each native object. The first imported
adoption preview should say, for example:

> Replace this entry's body under the same draft. Native page/block anchors change.
> Two image/code blocks carry inactive native text; it stays on their detached old
> entities and is not carried into the new body. Current code, images and prose are
> preserved. Save does not publish.

The existing `--recreate-body` adoption boundary records acceptance of that body
policy; no additional per-stale-field approval dialog is needed. Subsequent saves
retain the policy and show relevant effects. A latent remote edit still changes
the conservative witness and participates in drift decisions before retirement.
The new current Node table has no inactive locator for facts not present in the
new generation; the verified receipt reflects that deliberate state, not an
unnoticed omission.

I considered copying opaque inactive payloads onto new generated blocks. It is
technically possible using transient current observations, without local history.
It nevertheless adds old-key carry-forward rules and perpetuates unrepresented
conversion state in an otherwise source-generated body. It is not the chosen
default. If future type-toggle restoration becomes important, prefer retaining
the actual keyed objects rather than silently growing a generic opaque-state
copy mechanism. Do not create a second engine solely for latent text.

For unknown roots in **active** text, native source inspected here establishes
`prosemirror` and `meta.schemaVersion`, not a universal inertness rule for other
roots. The strict active-state boundary remains. There is no receipt evidence
here that unknown active roots affect the 27 files, so neither assert zero impact
nor reject every imported document preemptively. Add explicit fixtures and report
the concrete unsupported root if encountered.

### 12.5 Comments and rooted visibility: two corrections to the peer

The SQL full pull follows **outgoing** reference/ordered-reference/spatial-reference
edges from the root. G correctly uses that to explain why detached objects normally
leave the active pull. Two further claims need correction:

1. **Absence from rooted pull does not prove a creation never landed.** An entity
   can exist with its type/text facts while its parent membership was skipped,
   or a page-reference write may have failed. Inspect its known ID through explicit
   root reads and resolve request acknowledgments. Even explicit absence cannot
   prove that an unresolved in-flight request will never arrive.
2. **Native editor comments are not generally invisible incoming references.**
   `createEditorComment` writes `block/comment` from the block to the comment;
   replies use `comment/reply`. These are ordered references and therefore reachable
   from an active block in the same full-pull closure. Both current proposals'
   strict observer should reject them, rather than claim they are invisibly
   preserved as an ordinary supported imported case. Truly external incoming-only
   references can remain undiscovered; that is a separate limitation.

Generation already needs explicit-root reads for staging; a keyed writer also
needs them for reliable partial-creation diagnosis. This reduces the apparent
transport gap between the strategies, although generation additionally verifies
the complete staged graph before activation. SQL's fact/ack snapshot is useful
evidence; it does not close later read/write races or make external metadata writes
part of the same transaction.

### 12.6 The keyed alternative's real scope, and protocol improvements worth keeping

The peer's creation case is not inherently a second engine: “new block” can use
fresh encoding while “existing block” uses loaded-state editing in the same keyed
planner. I accept that description. Its split/join rules are ordinary source
identity rules, not a generalized structural matcher. Its willingness to normalize
order rather than minimize moves is also aligned with the corrected brief.

I challenge four remaining simplifications:

- **Type/scalar operations include removal semantics.** Heading→paragraph cannot
  merely assert a new type and leave heading-level facts unaccounted for. Native
  `setTextBlockStyle` clears stale heading-level/text-size fields, and the current
  strict observer would reject an unused heading-level fact. Optional image fields,
  removed code language and PageLink retargeting need similarly explicit ownership.
  The operation vocabulary is bounded, but those cases are real work.
- **Whole-page replacement is not a costless policy toggle.** Reusing create/detach
  primitives still needs a complete new identity assignment, desired-to-observed
  verification and partial replacement semantics. Naming it a degradation is fine;
  claiming it costs nothing risks hiding a second behavior before it is needed.
- **Character continuity is useful but not the delivery axis.** A single splice
  is a small plausible helper, supported by prior Yrs evidence. Its exact retained
  ranges/mark clearing need not determine when paragraph additions or splits ship.
  All-character overwrite in a retained compatible XmlText is an honest alternative;
  coarse child rebuilding has the already-proven concurrent-visibility cost.
- **Recovery is part of basic successful execution.** The peer calls the milestone
  complete through batch 3 and batch 4 “hardening,” while putting pending lifecycle,
  ack reads and delayed/lost-request tests there. Those minimal uncertainty controls
  must land before the remote write loop is presented as useful/safe. Extensive
  autonomous repair can be later; honest stop-and-resolve cannot.

Native helpers permit simplification in either architecture:

| Boundary | Verified shape and recommended use |
| --- | --- |
| RPC `createEntity` | Array of `{entityID, permission_set}`; the context method takes one object. Keep these distinct. |
| `addBlock` | Creates entity/type/membership but requires caller position and can infer list status from a block parent. Suitable when that full behavior is intended. |
| `addLastBlock` | `{parent, factID, entity}` only; assumes entity already exists, computes an end position and writes the specified membership ID. Useful for generation order and for simple fact-ID-aware reparent/normalize passes. |
| `moveBlock` | Uses target entity to choose an occurrence; requires unambiguous occurrences if used. An after-chain still must ensure the first desired block actually becomes first. |
| `retractFact` | RPC arguments are `{factID}`; ordinary source deletion detaches the selected membership, not the entity. |
| `assertFact` for new text | `id` is optional, not forbidden, for cardinality-one attributes. A supplied explicit ID can be used on a fresh entity; an existing cardinality-one fact retains its existing ID. |

Therefore g's server-minted text-fact IDs are a valid choice, but not a native
requirement. My generation chooses explicit derived IDs for its entire planned
inventory, then verifies actual rows. The keyed alternative may omit new text IDs
and learn them by `(entity, attribute)` read-back, promoting only actual IDs. Both
must reject ambiguity rather than call a planned ID confirmed. No fake native IDs
are needed in ordinary authoring source under either approach.

For ordering, native `addLastBlock` means a local fractional-index port is optional,
not a prerequisite for inserting before the old first block. Create new entities
and payloads, then place/normalize the desired membership order. Bounded extra writes
are acceptable. Preserve native helpers' exact semantics and prove >62 siblings;
do not inherit experiment 13's toy limit. Retrying append/move under fresh mutation
IDs is still a new effect, not an intrinsically harmless replay.

### 12.7 Source identity, coverage and receipts: shared work neither engine avoids

Keep the desired-versus-confirmed model changes in §5. The peer agrees on new keys
without node rows and vanished rows awaiting settlement; that is essential. Its
phrase “null association” is too weak for the canonical desired projection.
Two new PageLinks to different unbound page keys would otherwise both reduce their
target to null. Use a tagged local identity for an unconfirmed target, and confirmed
native associations where available. This is a projection of the existing model,
not a new diff model.

A small factual correction: current `editor_value` serializes
`self.state.nodes.get(key)` as an Option; its `expect` concerns JSON serialization,
not missing-node lookup. A missing row would currently become null if parsing
allowed it. The problem is the parser invariant and the meaning of missing rows,
not an existing lookup panic. Fix those semantics deliberately.

The peer does not address the token-derived imported-document-ID assertion.
Retain my supplied-local-namespace change: a random ID minted by local `create`
must survive first binding and every later observation. Import's deterministic
ID is an initial-adoption convention, not a second identity to impose after save.
The independently verified native tuple remains the remote identity check.

Both source forms are feasible: g's implicit main-page syntax is more ergonomic;
my explicit Page scaffold keeps the existing container grammar. This is independent
of the engine decision. I retain the explicit scaffold in this proposal's minimum
source change, supplied by `create` so authors do not invent page/native IDs.
An implicit-main convenience can be selected if it is given a stable local page
identity and unambiguous interaction with explicit Page definitions. Neither form
requires prose wrappers or manual block identities.

**Flat lists remain included in my proposed first product profile.** Their absence
does not logically prevent the brief's mandatory paragraph/heading operations;
the brief explicitly permits a declared list exclusion. Thus g's list deferral
is not equivalent to its earlier unacceptable paragraph-addition deferral.
Nevertheless, simple bullet/ordered lists are common drafting, the mapping is
known, and my recommendation budgets the small representation/read/write work
instead of claiming it already exists. Nested/tasks/multi-block items remain
explicitly outside. Quotes/code/rules/images/PageLinks and hard breaks retain the
concrete coverage in §3, with the inactive-text correction above.

Finally, unchanged source formatting and coherent key renames must not cause a
body generation, while title-only saves may update metadata without one. A successful
push must settle a receipt profile whose editor fields actually agree; permanent
false drift is not an acceptable placeholder for pending metadata integration.
Receipt changes must preserve old profile interpretation, not reinterpret previously
stored hashes under new list/presence/identity semantics.

### 12.8 Unknown mint, exact replay and minimum recovery

I reject guaranteed homepage recovery. `createNewLeaflet`'s **`addToHomepage`**
default controls the auth-conditioned CTE; `addToHome` controls a redirect query
flag. A valid preflight cookie is not a guarantee it remains valid at mint. The
identity query's lack of an explicit limit is not verified global completeness.
Even one new seed candidate is not unique attempt attribution: another tab or
actor could have created it, and the native mint accepts no client correlation key.

After a lost response, listing candidates is useful. Explicitly adopting a selected
candidate is a new identity decision; it is not evidence the original attempt
succeeded exactly once. No match does not prove no mint. Keep pending uncertainty,
never auto-remint, and preserve a returned token before further effects when local
byte guards allow it. A crash between server creation and durable token recording
is an unavoidable uncertainty window in this protocol.

The peer's “absent from pull → rederive with same IDs” and “partial → assert missing
facts” recipes are unsafe without resolving issued requests first. Same entity IDs
do not make a newly encoded fresh Yrs payload identical to the lost payload.
Require acknowledgments/observed effects to resolve prior requests, then inspect
the relevant entities explicitly. Recovered equal text is not reasserted. Unknown
late-arrival possibility stops; it does not become a blind repair loop.

For generation, use the same rule before abandoning a stage or issuing another
activation. A late old activation could select an obsolete body even though later
generation IDs are different. Keep exact requests in memory for same-process retry;
persist only identity/evidence. After restart, observe/settle completed effects or
stop. These constraints are required for both designs; generation is not an
exactly-once escape hatch.

### 12.9 Updated first/second-save checks and implementation map

The walkthrough in §1 remains the selected algorithm. These additional scenarios
make the review's substantive differences executable:

1. **Configured publication:** unbound source omits publication; config selects P.
   First push calls publication mint, observes binding P and `doc:null`, applies
   title/description/tags to the association row, then body/read-back succeeds.
   It never claims success with null binding and unsaved release-intent tags.
2. **Explicit standalone:** authored null overrides config P, chooses `/new`, keeps
   local UUID, writes token metadata, and repeats without mint or false drift.
   No unconditional null is inserted by `create` when intent was absent.
3. **Already-imported document:** committed synthetic code/image fixture contains
   known inactive text. Add a heading and paragraph before the first block. On
   explicit generated-body adoption, current code/image/prose survives, old hidden
   text remains only on old entities, the new association table reflects reality,
   and immediate pull/repeat push are no-ops. Unknown active fields still fail.
4. **Second save combined:** add heading/paragraph, edit marks/link/breaks, split,
   join, delete, reorder and change heading level in one source revision. One new
   body is verified/activated under the same token/root/entry. Include flat lists
   and a repeated named-page target in the fixture; neither needs fake rows.
5. **Interrupted create/placement:** entity/text creation succeeds but membership
   fails. Active pull omits it; explicit-root read finds it. The workflow must not
   conclude nonexistence or send independently regenerated fresh text while an
   earlier request is unresolved. Include delayed activation and lost mint with
   a plausible but unattributable homepage candidate.
6. **Metadata and comments:** optimistic client-only extras cannot satisfy server
   read-back; linked draft title uses its actual row; direct native `block/comment`
   and reply fixtures are observed and rejected as unsupported, not called invisible.

Keep four coherent batches, with essential failure handling integrated rather than
deferred as post-delivery hardening. Parent owns actual ticket updates/assignment:

- **A — local authorship and model**, under `leafbud-author-create` and
  `leafbud-author-keys` if retained in the tracker: g implements optional coherent
  binding/sync, intent presence, local-ID stability, precise key preparation and the
  complete chosen source/read-back profile, including flat lists. This makes normal
  structural drafts valid before remote execution.
- **B — native first binding and write boundary**, under
  `leafbud-editor-bind`: recommend Sol high for publication-action invocation,
  standalone mint, exact protocol shapes, pending-token lifecycle and typed ack/
  explicit-root reads. Essential lost/uncertain mint and request handling ships here.
  Resolve actual publication context; don't accept a standalone substitute.
- **C — full generated-body save**, under `leafbud-editor-structure` with the
  fresh-text work from `leafbud-editor-text`: g can implement the now-specified
  emitter, helper-driven ordering, staged verification, entry activation, context-
  correct metadata and known-inactive retirement policy. It includes source-byte
  guards and read-back; no independently executable “safe later” write shortcut.
- **D — joined conflict/recovery/product proof**, under
  `leafbud-editor-conflicts` and `leafbud-editor-recovery`: recommend Sol high
  for the final receipt/uncertainty integration and complete matrix. This completes
  stop/settle recovery, guarded pull and the authorized full two-save native loop.
  Extensive autonomous repair is optional; the required workflow is not declared
  complete until this batch's core cases pass.

These are recommendations, not newly created work or a one-session-per-batch
constraint. If batch B needs multiple sessions for real action transport, use them.
Do not shrink first binding or later structural drafting to fit a convenient
session boundary. The prior experiments need not all be rerun; new tests should
target the receiving code and the concrete differences above.

### 12.10 Final choice, tradeoffs and remaining deployment evidence

**Kept:** one generation engine; explicit current-body ownership; native
document/public identity; automatic source keys; actual publication-context mint;
strict unknown-active/interactive boundaries; flat lists; no stored body/history/
plan; native helper ordering; complete two-save acceptance.

**Changed:** known inactive image/code text no longer blocks generation; its
retirement is a documented ownership effect. The imported-corpus receipt and native
renderers justify that narrower treatment. I sharpened metadata storage, association-
token handling, optional fact-ID semantics and essential recovery timing.

**Accepted from the peer:** keyed create/update/detach/order is one plausible
complete engine, not necessarily a second broad implementation; ordinary split/join
can follow source keys without a structural diff; no minimal-move requirement;
headless native creation is mandatory. Its preservation of unchanged keyed objects
is a genuine advantage, including inactive state, rather than mere operation savings.

**Rejected:** postponing selected publication context until release by default;
calling permanent unsaved-tag drift a completed save; rooted absence as noncreation
proof; guaranteed homepage mint attribution; blanket comment invisibility; and
portraying whole-page replacement/recovery as free switches or later-only polish.

The human decision is still substantive: **accept changed native body anchors and
retired inactive conversion state in exchange for source-complete generation**, or
choose the complete keyed engine to retain them. My recommendation remains the
former under the expressed priorities. If the operator wants the latter, it must
retain all required drafting actions and the binding/metadata/recovery corrections
above. There is no evidence-based reason to retreat to existing-one-XmlText scope.

Deployment uncertainties remain concrete: action-reference discovery and result
decoding for publication mint; auth/redirect behavior for `/new`; actual returned
association and contributor defaults; array-shaped creation and ordered buffered
mutations; explicit-root completeness and stage visibility; entry update/metadata
partial outcomes; browser rendering of lists/marks/images/title; and request-ack
recovery on the deployed server. Source inspection supports implementation choices,
and existing probes support Yrs mechanics. Neither substitutes for the authorized
**local create → first native push → structural revision → second push → pull →
unchanged push** deployment proof in both supported creation contexts.

No new runtime experiment was needed to resolve the reviewed contradictions: native
source, current production code and committed corpus fixtures suffice. This stage
performs no production, private-data or remote changes, and does not claim peer
consensus before the parent joins both separately committed addenda.
