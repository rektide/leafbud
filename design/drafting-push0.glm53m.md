---
type: Design
title: "Normal drafting and push — keyed block operations with fresh generation as the creation case"
description: "Fresh g proposal for the corrected brief: author locally without native IDs, bind on first push, and express insert/edit/split/join/delete/reorder as one small keyed operation vocabulary on the live entry page; generated-body regeneration compared candidly and kept as degradation, not default."
resource: /design/drafting-push0.glm53m.md
status: proposal
generated:
  at: 2026-11-08
  by: model:zai-coding-plan/glm-5.3#max
updated:
  at: 2026-11-08
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e435692e5ffeDnxWPnxYBrl26q"
    title: "g stage-1 independent draft for the corrected drafting/push brief: prior-wave and code evidence re-verified, pull closure and mutation shapes read from archived Leaflet source, architecture reconsidered from the complete user stories"
  - resource: /design/drafting-push-brief0.gpt6a.md
    title: "Controlling corrected commission: complete user stories, architectural freedom and constraints"
  - resource: /AGENTS.md
    title: "Product-fit, session-sizing and completion-report cadence read before inquiry"
  - resource: /design/native-edit0.gpt6a.md
    title: "Prior Astra wave: identity budget, editing policy, alternatives table, executed concurrency/wire probes (prior-wave evidence, not a constraint)"
  - resource: /design/native-edit0.glm53m.md
    title: "Prior g wave and cross-review: keyed operations, corrections to concurrency/retry claims, revised stance"
  - resource: /design/native-edit-join0.gpt6a.md
    title: "Parent join: withdrawal of the text-only milestone and the corrected delivery boundary"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted CLI, crate map, conflict/effect contract, YAML corrections including pending-vs-confirmed node rows"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Governing keyed bidirectional addendum and unbound-draft binding sketch"
  - resource: /design/status-witnesses0.gpt61s.md
    title: "Landed scoped observation, editor receipts and Agreement classification reused by push"
  - resource: /README.md
    title: "Implemented read-only surface and pending write/create commands at inspected revision"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Live fact model, pull/push contracts, mutation vocabulary, replay and merge semantics"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Executed fresh compiler-to-facts-to-text pipeline and independent Yjs consumer"
  - resource: /crates/document/model/src/lib.rs
    title: "Current Document/State/Binding/Node/Sync shapes and editor_value scoped comparison input"
  - resource: /crates/document/mdx/src/lib.rs
    title: "Current parser invariants read directly: bound-draft requirement, Key markers, exact key/node coverage"
  - resource: /crates/leaflet/editor/src/lib.rs
    title: "observe() strict profile, synthetic key minting, single-entry and unrepresented-attribute refusals"
  - resource: /crates/leaflet/editor/src/text.rs
    title: "Production Yrs read path: single paragraph, strict marks, pending-state rejection"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Agreement classification and scoped local/remote change evidence"
  - resource: /crates/workflow/src/import.rs
    title: "Import binding construction, hash profiles and metadata sources"
  - resource: /crates/leaflet/web/src/pull.rs
    title: "Existing null-cookie full-read transport; no write transport exists"
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/supabase/migrations/20260721000000_add_pull_data_cvr.sql"
    title: "Read this session: pull full snapshot is a rooted recursive reference closure from the token's root entity — detached entities are absent"
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/src/replicache/mutations.ts"
    title: "Read this session: addBlock/moveBlock/assertFact/retractFact/createEntity/updateLeafletMetadata argument shapes and list-flag representation"
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/app/(app)/(identity)/new/route.ts"
    title: "Read in the 1.1 correction: GET /new is a plain force-dynamic GET route calling createNewLeaflet with redirectUser: true"
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/actions/createNewLeaflet.ts"
    title: "Read in the 1.1 correction: createLeaflet seed shape, auth_token homepage-link CTE that inserts nothing when the cookie is invalid, and the 307 Location carrying the fresh permission token id"
  - resource: https://github.com/rektide/leafspring/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/drafts/src/create.rs
    title: "Pre-reset mint client read in the 1.1 correction: GET /new with redirects disabled, liberal 3xx/Location harvest, auth-before-mint gating, offline loopback tests; live probe remained open"
  - resource: /design/draft-extraction.glm53m.md
    title: "Prior new-draft gap analysis: option (d) GET /new harvest verified in source; pre-reset client shipped it; live mint probe never executed"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Accepted envelope shape checked in the 1.1 correction: flat author fields, state plus optional binding group; tagged binding.kind was the rejected alternative my first sketch wrongly copied"
---

# Normal drafting and push — one keyed operation vocabulary

## Verdict

**Author locally with optional keys and an implicit main page; on first
push, mint a fresh native draft headlessly (authenticated `GET /new`
harvest), bind it, and generate the whole desired body with caller-minted
native IDs; on later pushes, express the supported
desired document through one small keyed operation vocabulary on the live
entry page — create, update-in-place, detach membership, normalize order,
write editor-owned metadata — verifying by fresh read-back and promoting
pending node rows only then.** Fresh generation is the *creation case* of
this vocabulary (a new block is a create op whose text happens to be a fresh
initial Yjs update), not a second engine. Wholesale body regeneration and
whole-page rewrite are kept as named degradations with their identity costs
stated, not as parallel implementations. No stage/switch machinery ships in
the default path; `--recreate-body` remains an explicitly separate future
escape.

This is stage one, written without reading the same-wave peer draft. Ticket:
`leafspring-editor-plan-design` (parent owns coordination). Revision
bindings: inspected working copy at parent checkpoint `9a3a0dbb`
(`qqumsozq`, "Commission fresh whole-document drafting and push designs");
crate/CLI names below are that revision's `leafspring-*` spellings. The
incoming **Leafbud** rename/config/data conversion is owned by a separate
session; nothing here performs or assumes it. This document owns only
itself.

The rejected prior delivery was "edit one existing XmlText, ship it."
Nothing below treats that as a milestone: the delivery unit is the union of
user stories 1–6 in the [brief](drafting-push-brief0.gpt6a.md).

## 1.1 — Parent correction (stage 1.1, before peer review)

The parent review found my stage-one Story 2 in direct violation of the
brief: I required the user to create an empty draft in the web UI and pass
`--to <locator>`, demoting headless creation to a "separately gated
enhancement." That repeats the exact failure the user rejected — removing a
basic normal workflow to simplify implementation. Required story 2 is
**push a locally written file into a NEW native draft with no native IDs
obtained by hand**; automatic native creation and binding are part of the
milestone. This revision corrects Story 2 around the source-verified
`GET /new` harvest route, fixes uncertain-mint handling and publication
deliberateness, and replaces a superseded frontmatter sketch (`binding:
unlinked-draft`, the [rejected
alternative](frontmatter0.gpt61s.md)) with the accepted envelope shape. The
original stage-one text remains evidence at commit `6985d01f`
(jj change `kkpyzwwx`); the sections below are the operative stance. The
peer draft remains unread.

## 1. What exists today, verified against the code

Read directly this session, not assumed from prior waves:

- **Parsing is import-only.** `parse_with_spans` requires a bound draft:
  explicit `binding.publication`, `state: draft`, full
  `Binding{base_url,token,root,entity_set,entry_fact}`, a `<Key id>` marker
  before every block, and **exact equality of source keys and node rows**
  (`crates/document/mdx/src/lib.rs`, coverage check at the parse tail). An
  unbound local document or an authored paragraph without a node row cannot
  parse today. `create`, `push` and `tidy --keys` are unimplemented (README
  table; `tidy --keys` is rejected honestly, exit 1).
- **Observation is strict and association-based.** `observe()` mints
  synthetic keys (`b-<uuidv5(namespace, membership-id)>`), requires exactly
  one `root/page` entry, and refuses every unrepresented fact attribute.
  `editor_value()` compares scoped editor meaning including associations;
  `Agreement` classification (`clean`/`local-changed`/`remote-changed`/
  `both-changed`/`converged`/`unverified`) already exists and is reusable.
- **Yrs handling is read-only in production.** `text.rs` decodes one
  paragraph with strict marks; experiments 05/13 plus the prior wave's probes
  verified the write mechanics offline: load base, edit in one transaction,
  one complete edited-state payload per changed text fact, fresh initial
  updates on new entities, duplicate-payload idempotency, and the two
  concurrency hazards (child removal can hide concurrent inserts; two
  independently compiled stale-base edits merge into duplicated text).
- **No write transport exists.** `leafspring-web` has reads only.

New this session, from the archived Leaflet checkout (`600c7b5f`):

- **The pull full snapshot is a rooted closure.**
  `pull_data_cvr` (and the legacy `pull_data` it replicates) starts from
  facts whose entity is the token's root entity and recursively follows only
  reference-typed facts to their targets. Consequences that shape this
  design: (a) **detached entities disappear from pull**, so retract-only
  deletion keeps future status/pull working instead of tripping the
  unrepresented-attribute refusal; (b) definition pages referenced by
  `block/card` facts are reachable; (c) absence from pull is usable
  recovery evidence that a creation never landed; (d) comments and other
  entities that *reference* blocks but are not referenced by anything
  reachable are invisible to pull — preserved by never touching them, never
  witnessed by us.
- **Mutation shapes are exact.** `addBlock {parent, permission_set, factID,
  type, newEntityID, position}` (caller-minted entity + membership fact +
  fractional position); `moveBlock {oldParent, block, newParent, position:
  first|end|before|after}` (overwrites the existing membership fact in
  place, server-mints the fractional key, chooses the occurrence by target
  entity); `assertFact` with cardinality-one attributes updating in place
  and cardinality-many requiring explicit fact IDs; `retractFact {factID}`;
  `createEntity`; `updateLeafletMetadata {title?, description?}` writing
  `permission_tokens` server-side. Lists are ordinary blocks carrying
  `block/is-list` / `block/list-style` / `block/check-list` flags — known
  shape, deliberately outside the first profile (§5).

## 2. The complete walkthrough

Concrete trace of every required story. Native IDs are minted as UUIDv4;
source keys are minted once as `b-<uuidv4>` (human names still available via
`{#name}` / `<Key id>`).

### Story 1 — start on disk

```sh
leafspring create drafts/garden.mdx --title "Garden notes"
```

writes an unbound file — no binding, nodes, or sync rows; `publication:
null` is the explicit standalone intent (a URI would declare future
association intent):

```mdx
---
format: leafspring-mdx/1
title: Garden notes
publication: null

"@leafspring":
  id: 9af05c1e-…
  state: draft
---

# Garden notes

## Compost

The pile is **hot**. See [the guide](https://example.com/compost).
```

Per the [accepted envelope](frontmatter0.gpt61s.md): flat author fields,
`state: draft`, and **no binding group** — remote fields appear only after
binding. `publication: null` is the explicit standalone intent (a URI would
declare future association intent).

The authoring parse profile accepts this: an implicit main page when no
`<Page>` wrapper is present, `<Key>` markers optional (a block without one
is simply a new block), ordinary Markdown inline content per the existing
Markdown-first surface. `check` validates the supported profile offline —
no network, no IDs. Nothing remote exists yet; nothing needs it.

### Story 2 — first push (browser-free, REQUIRED)

**Product criterion: an initial push with no locator, no browser, and no
hand-obtained native IDs is a required part of this milestone, not an
enhancement.** The ordinary command is:

```sh
leafspring push drafts/garden.mdx --yes
```

Creation route: the **`GET /new` harvest**, verified in upstream source and
previously implemented with offline loopback tests in the pre-reset client
([create.rs at `80436b61`](https://github.com/rektide/leafspring/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/drafts/src/create.rs),
per the [new-draft gap analysis](draft-extraction.glm53m.md#the-new-draft-gap)
option (d)):

- `app/(app)/(identity)/new/route.ts` is a **plain `force-dynamic` GET
  route** — no Next-Action id, no body — that calls `createNewLeaflet({
  pageType: "doc", redirectUser: true, addToHome: true })`.
- `actions/createNewLeaflet.ts` mints the full draft (entity set, entities,
  permission token with full rights, h1 seed block, watermark root fact)
  via one SQL statement and ends in
  `redirect("/${permTokenId}?focusFirstBlock&addToHome")` — a **307 whose
  Location header carries the fresh permission token id** in its first
  path segment.
- The client sends one authenticated GET with redirects disabled (the
  Location *is* the payload) and harvests the token UUID with a liberal
  parser (any 3xx; relative or absolute Location; query ignored). The
  pre-reset implementation, its parser and its loopback tests are portable
  to `leaflet/web` deliberately.
- **Live-verification standing:** the Location shape is source-verified,
  not live-captured; the pre-reset live probe never executed before the
  scaffold reset. The first operator-gated live run is therefore an
  explicit implementation probe *inside* this delivery (batch 2, §9) — not
  a reason to omit creation from the milestone.

Steps, with identities:

1. **Auth before mint.** The insert itself never consults the cookie — only
   the homepage-link CTE does — so an unauthenticated `GET /new` would
   still mint an orphan draft nobody lists. Push therefore performs an
   authenticated discovery read first (the existing homepage `remote list`
   surface) and refuses to mint when credentials are missing or invalid.
2. **Prepare.** Parse desired document; validate profile. Mint source keys
   and insert `<Key id="b-K1"/> …` markers byte-preservingly (existing span
   machinery); re-read and re-parse to prove the plan input is unchanged.
   Write nothing else to the body, ever.
3. **Mint and record immediately.** Persist a pending marker (phase
   `minting`), then `GET /new` → token `T`. Record `T` in the pending
   envelope field immediately (atomic write under the original-byte guard)
   *before* any content mutations, so a crash after minting never loses the
   identity of the draft we created.
4. **Resolve the new draft.** `get_leaflet_data(T)` → verify observed state
   is a draft, our token holds `write` on exactly the entity set, and the
   editor is seed-only (the h1 seed block and watermark fact; anything else
   is not a fresh mint — refuse with guidance rather than clobbering, per
   story 6). Collect root `R`, entity set `S`, entry fact `F`, entry page
   `P`.
5. **Mint and record pending node IDs.** For each desired block mint entity
   `E_i` and membership fact `M_i`; record them in the `pending` envelope
   field (accepted interface correction 5: planned creation IDs live in
   `pending` until verified, then promote to `nodes`). Text fact IDs are
   *not* minted: `block/text` is cardinality-one, the server assigns the
   fact on first assert, and we learn it from read-back. No fake confirmed
   native IDs anywhere.
6. **Execute one push RPC** (mutations in dependency order):
   `addBlock(E1,M1,P,"heading",pos)` + `assertFact(E1, block/text, fresh1)`
   + `assertFact(E1, block/heading-level, 1)`; likewise for the `##`
   heading and the paragraph; `updateLeafletMetadata {title}`; then
   `retractFact(M0)` detaching the seed h1's membership (its entity and
   facts persist detached; no deletion). `fresh_i` are fresh initial v1
   updates on each new entity (experiment-13-verified encoding).
7. **Verify and settle.** Fresh pull → `observe` → per-block typed equality
   against desired; collect the observed `block/text` fact IDs `T_i`; promote
   pending rows into `nodes: {b-K1: {E1, {M1}, T1}, …}`; promote the pending
   binding into the actual `Binding` group (base URL, token `T`, root `R`,
   entity set `S`, entry fact `F`) and observed publication association;
   settle `sync` receipts (profile `leaflet-editor/2`). Envelope write is
   atomic against the original bytes.

Exit 0. Save is not publish: no release receipts, no public output, the
draft stays a draft. A second `push` now classifies `clean` and exits 0
with zero mutations (story 5).

**Publication intent versus observed binding, deliberately.** A freshly
minted draft is a pure draft: no publication linkage. The file's declared
`publication` (null or URI) is *retained as release-domain intent* — never
silently retargeted by ambient config, never conflated with the observed
null association recorded in the binding. Association happens at publish
time under the release domain's rules. Config (base URL, auth) resolves the
*native context* — which Leaflet instance and identity mint the draft; it
never chooses a publication for the file.

**Uncertain mint responses — never auto-remint.** A timed-out or
unparseable `GET /new` response may still have minted a draft we cannot
identify. Rules: (a) never automatically re-GET `/new` (each retry risks
another orphan); (b) because auth was verified first and `addToHome` is
true, a successfully minted draft appears on the operator's homepage —
recovery lists drafts and looks for a fresh pure-seed draft; exactly one
candidate → propose adopting it explicitly (recorded, user-visible);
ambiguous or none → report the uncertain mint with guidance (adopt by
locator via `--to`, or authorize one deliberate re-mint), naming the orphan
cost honestly. The same resolution covers a crash between mint and pending
persist. This is the same resolve-don't-regenerate discipline as §7, applied
to the mint itself.

**`--to <locator>` remains optional convenience functionality** — adopting
a draft that already exists (e.g. created in the browser, or resolving an
uncertain mint) — with the same verify-seed/emptiness gate and the same
binding promotion. It is not the required path and never was the story.

### Story 3 — continue drafting

Ordinary source edits, no IDs to think about:

```mdx
---
… binding now leaflet-draft; nodes b-K1..b-K3 confirmed …

Cold nights are coming.          ← new paragraph before the old first block

# Garden notes                    {#b-K1 unchanged}

## Winter prep                    ← new heading inserted mid-document

The pile is **hot** and steaming. ← b-K3: text edited, split follows below

Mulching matters too.             ← second half of the split (new block)

## Compost                        ← b-K2 deleted by the author
```

plus a join elsewhere (two paragraphs merged into one) and a reorder (the
Winter prep pair moved above another section).

### Story 4 — push the revision

Per-key disposition from comparing desired blocks against the fresh pull,
matched through node rows (`b-K1 → E1/M1/T1`), never by prose similarity:

| Source situation | Native operations | Identity effect |
| --- | --- | --- |
| `b-K1` heading, unchanged | none | entity, membership, text fact, all anchors untouched |
| new intro paragraph | mint `E4/M4`, key `b-K4` inserted at push; `addBlock` at position minted before the old first position; fresh text update | new objects, first-position membership |
| new `## Winter prep` | same creation path mid-document | new objects |
| `b-K3` edited text, same inline topology | load `T3`'s update into Yrs; one transaction: single common-prefix/suffix splice, apply desired marks; emit **one complete edited-state payload**; `assertFact(E3, block/text, payload)` | entity/membership/text fact and container retained; prefix/suffix character items retained; concurrent unseen edits surface as visible divergence, not silent hiding |
| split: `b-K3` keeps first half (edited), second half is new | `b-K3` path above + creation for the new block | deterministic rule: the first piece retains the key; `keys rename` adjusts deliberately if the author meant otherwise |
| join: earlier keys `b-KA`,`b-KB` merged | retained key gets the edited-text path; vanished key's membership `retractFact` | merged block keeps one identity; other entity persists detached |
| `b-K2` deleted | `retractFact(M2)` only — never `removeBlock`/`deleteEntity` | entity, comments referencing it, and incoming references survive detached |
| reorder | `moveBlock` `after`-chain over desired order, **skipped entirely when order already matches** | membership facts updated in place (fact IDs preserved); untouched blocks untouched |
| heading level change | `assertFact block/heading-level` | text fact untouched |
| type change text↔heading↔quote | `assertFact block/type` (+level) | same entity and text fact; Yrs continuity across the type change |

Editor-owned metadata: title/description changes via
`updateLeafletMetadata`; watermark via `theme/*` assert on the root. Tags
are publication-domain (`publication_tags` extras exist only under
publication linkage): a local tags change is carried as release intent and
reported in the plan ("applied at publish"), never silently dropped or
faked — see §8 for the receipt seam this leaves.

The whole desired document is expressed or the push refuses (atomic scope
refusal): an unsupported change riding along (a list, a poll, an image
upload) blocks *before any write*, rather than saving part of the document.

### Story 5 — read and repeat

`pull` after push classifies clean and rewrites nothing; a repeat unchanged
`push` sends zero mutations and settles nothing new — no second draft, no
duplicate body, no new generation, because the equality gate fires before
planning and the pull closure contains exactly the reachable body.

### Story 6 — useful failure

- A list appears in the source → explicit profile refusal naming lists and
  their seam (§5), exit 3, no writes.
- A web edit landed meanwhile → `both-changed`/`remote-changed` → exit 3
  with pull guidance; `--take local` re-observes fresh and proceeds with
  the same keyed vocabulary (targeted, not wholesale).
- An unrepresented remote attribute (e.g. a poll) → observation refusal,
  exit 3; unrelated native content is never edited around.
- Interrupted/uncertain push → compact pending marker, stop-and-report;
  resolution rules in §7. Existing unrelated content is structurally safe:
  the vocabulary never deletes entities and never writes outside the keys
  the source names.

## 3. The engine, precisely

One planner function at one seam (workflow layer, alongside
`editor_state.rs`; no new crate, no parallel semantic tree):

```text
plan_push(desired: &Document, observed: &Content, nodes: &BTreeMap<String,Node>)
  -> Result<PushPlan>            // flat list of native mutations + effect labels + minted IDs
edit_text(base_update: &str, desired: &[Inline]) -> Result<Payload>
  // CompleteEdit(state) for existing facts; Fresh(initial) for new entities
```

Operation families (each maps to verified native mutations):

| Family | Native writes | Notes |
| --- | --- | --- |
| Create block | `addBlock` + `assertFact block/text` (fresh initial update) + scalar asserts | caller-minted entity/membership/position; positions minted with a ported `generateKeyBetween` (small, property-tested) |
| Create definition page + link | `createEntity` + `assertFact page/type` + blocks + card block's `block/card` reference | shape-verified; reached by pull closure via the referencing card fact |
| Edit text (existing) | `assertFact block/text` with one complete edited-state payload per changed fact | load from the pull we already hold; local semantic read-back before send; never a chain of dependent deltas (server last-write cache drops prerequisites); never an independently generated fresh document over an existing text fact (merges into two paragraphs — executed negative evidence); never retract-then-reassert |
| Change scalar / type | `assertFact` cardinality-one on the same entity | heading level, code text/language, image payload, block/type |
| Detach block | `retractFact` membership | entities persist; absent from future pulls by rooted closure |
| Reorder | `moveBlock` after-chain in desired order, skipped when ordered | server mints positions; no client renumbering of unchanged memberships |
| Metadata | `updateLeafletMetadata`, `theme/*` asserts | tags excepted (§8) |

**Text policy inside an existing block** (joined prior-wave position,
kept): same inline topology → single prefix/suffix splice in the retained
`XmlText`, desired marks applied over ranges (mark-only edits retain all
characters); changed topology (breaks added/removed) → rebuild paragraph
children in the loaded document, with the effect label stating that
concurrent web edits inside rebuilt containers can be hidden (read-back
equality proves achieved meaning, not absence of lost concurrent intent).
Named degradation if the splice helper grows disproportionate:
delete-all-seen-characters + insert-desired inside the retained `XmlText`
(container identity and visible-divergence behavior preserved). Named
degradation if per-block field wiring proves disproportionate: treat every
block as changed — whole-page rewrite on the *same* page entity via the
same create/detach vocabulary (identity cost: all block identities die
every save). These are policy switches inside one engine, not a second
implementation.

**No-op discipline:** document equality (existing `editor_value` gate)
precedes planning; per-block no-ops emit no writes; order normalization is
skipped when order matches; a semantically equal push never normalizes
native shape.

## 4. Minimum source/model changes

1. **Envelope follows the accepted interface, not the superseded sketch.**
   Established: the [accepted
   envelope](frontmatter0.gpt61s.md)/[interface](authoring-interface0.gpt6a.md)
   uses flat author fields, a readable `state: draft|published`, and a
   validated optional `binding` group that appears only once linked (a
   partial group is not a usable binding); today's production `State`
   struct requires binding/nodes/sync because parsing is import-only.
   *Proposed by this design:* make binding/nodes/sync serialize only when
   linked (the unbound local file carries just `id` + `state`), and add the
   accepted interface's `pending` field (creation IDs, attempt hashes,
   phase) existing only while unsettled. No saved body, no CRDT snapshot,
   no JSON master, no persisted mutation journal — receipts and IDs only.
2. **Authoring parse profile** (same parser, new mode): implicit main page
   without `<Page>` wrappers; `<Key>` markers optional; keys without node
   rows parse as pending; body blocks without keys parse as new. The
   current exact-coverage invariant becomes a three-way disposition
   (bound / pending-new / vanished) computed by the planner, not the
   parser. Round-trip: pull continues writing the canonical keyed form, so
   imported files' bytes are unaffected; authored files stay Markdown-first.
3. **Key minting and insertion**, once, at executed push and in
   `tidy --keys`: `b-<uuidv4>` markers inserted byte-preservingly; existing
   keys never re-minted; duplicates diagnosed. `check` reports missing keys
   as preparation state, not errors, for unbound documents.
4. **`editor_value` pending handling:** pending nodes contribute content
   entries with null associations for local comparison; promotion replaces
   them after verified read-back. (The current implementation
   `expect`s a node row per key — that panic path is removed by the
   three-way disposition.)
5. **Fresh-text encoder promotion** from experiment 13 into
   `leaflet/editor` (paragraph, marks, hard breaks, Unicode; Yjs-oracle
   tests), plus the `edit_text` existing-state writer and a small
   fractional-key port with property tests.
6. **Write transport** in `leaflet/web`: `push(root_entity, token, clientID,
   mutations)` per the experiment-02 contract; loopback responder tests
   (ordering, replay via `lastMutationID`, throw-skip-ack partial batches,
   lost responses); per-attempt stable client UUID + monotonic mutation ids
   so an in-process retry resends exact bytes/ids. Plus the **mint
   transport**: port `create_draft` (GET /new, redirects disabled, liberal
   Location harvest) with its loopback tests from the pre-reset client, and
   the auth-before-mint gate.

## 5. Profile statement (honest, per the brief)

- **Round-tripped today (import) and newly enabled for authoring:**
  paragraphs, headings (level/text), quotes, code (text/language), rules,
  images with existing remote sources, page links plus definition pages,
  strong/em/code/strikethrough marks, links, hard breaks; underline/highlight
  via exact `Run` components as today; title/description/watermark.
- **Preserved unchanged, never interpreted:** latent/inactive text facts on
  type-converted blocks (witnessed opaque), detached entities (absent from
  pull), comments/replies (invisible to the rooted closure), publication
  association and release state (release domain).
- **Explicitly outside the first profile, refused with named seams:**
  Markdown/native lists (source refusal naming `mdx-structure`; native shape
  is known — `block/is-list`/`list-style`/`check-list` flags on ordinary
  blocks — so the extension is a representation addition, not an engine
  change); polls/questions and all interactive objects (unchanged-stable
  policy from prior waves: never reinterpret votes; key-derived stable
  identities when added); local asset upload (`mdx-media`); canvas, groups,
  galleries, footnotes, mentions (`mdx-rich`).
- Paragraph/heading creation, editing, splitting, joining, deletion and
  reordering are **mandatory core**, never "future optimization".

## 6. Verification and receipts

Execution loop: re-read source bytes and re-pull immediately before effects
(changed evidence invalidates the prepared plan) → pending marker → one
push RPC → fresh pull → per-block typed equality + association match +
out-of-scope untouched → promote pending rows → settle `sync` under the
original-byte guard → report achieved/partial/diverged with effect labels
("2 blocks edited, containers retained; 1 rebuilt — concurrent edits there
could be hidden; 3 created; 1 detached; order normalized; public content
unchanged"). Receipts patch only when the local file still matches the
attempt input. `push` never touches release/naming receipts.

## 7. Uncertainty and recovery — small, honest obligations

No server CAS exists or is claimed; the token-scoped advisory lock is not
one. From the corrected prior-wave evidence: an identical encoded
payload/request is idempotent (resend exact bytes/ids in-process); an
*independently compiled* edit of a stale base duplicates text — so after
interruption, **resolve from observation, never regenerate-and-replay**:
re-pull; creations visible with desired content → promote; visible
partially (entity present, facts missing — per-mutation rollback makes
partial batches possible) → repair by asserting the missing facts; absent
→ re-derive the plan (same minted IDs, fresh mutations) since absence from
the rooted pull means the creation never landed; in-flight/ambiguous →
stop, report pending state with IDs. `lastMutationIDChanges` from pull
responses provide acknowledgment evidence where present — processed is not
applied. Uncertain outcomes never loop-push until the file "wins".

## 8. Open seams and decisions for the parent/user

1. **Mint route live verification:** the `GET /new` harvest is
   source-verified and pre-implemented offline, but never executed live
   (the pre-reset probe stayed open). The **browser-free initial push is a
   required product criterion**; the live mint is an implementation probe
   inside batch 2, not optional scope. If the deployed `/new` route shape
   diverges (non-3xx, different Location), fallback routes are the
   draft-extraction options: Next-Action POST (fragile, last resort) or —
   only if creation were *impossible* with supported auth — an explicit
   blocker report with proof, never a silent redefinition of story 2 as a
   browser click.
2. **Tags on drafts:** local tags cannot be written natively on a pure
   draft; push carries them as release intent. The editor receipt profile
   includes tags, so such a file reports `local-changed` until publish.
   Options: accept the signal, or scope tags out of the editor profile
   (receipt churn for existing files). Decide before freezing receipts.
3. **Derived-title interaction:** first-block edits vs stored
   `permission_tokens.title` — fixture required (prior wave's flag; kept).
4. **Deployed parity** of addBlock/moveBlock/metadata acceptance, seed
   detach, and position tolerance: the two gated live probes (§9) settle
   transport; a surprising server response changes the transport slice,
   not the engine.
5. **Pull closure deployment parity:** the rooted-closure behavior is
   migration-source-verified and consistent with live full snapshots; if
   production pull ever returned detached facts, `observe` would refuse —
   the fix is an owned-projection allowance naming our own detached
   generations, engine unchanged.

## 9. Batching that converges on the whole deliverable

Existing tickets as ownership units; the **product milestone is the union
through batch 3**; batch 4 hardens story 6. Each batch = one agent session
(160–350k), committed code + fixtures + ticket notes; internal checkpoints
are never presented as the product.

- **Batch 1 — source authoring foundation** (`leafspring-author-create`,
  `leafspring-author-keys`; agent g): `create`, unbound envelope, authoring
  parse profile, key mint/insert, pending-node model, profile refusals.
  All offline; fixtures: create/parse/assign/re-parse identity, unkeyed
  docs, split/join parse shapes, list refusal, implicit-page round-trip.
- **Batch 2 — write transport + first generation** (`leafspring-editor-bind`
  + creation half of `leafspring-editor-structure`; Sol high recommended —
  first native write boundary, identity/protocol judgment): web push
  transport + loopback tests; mint transport ported from the pre-reset
  client with auth-before-mint; fresh-text encoder promotion; fact builder
  for a full desired document; first-binding flow end-to-end (mint via
  `GET /new`, seed detach, read-back, promotion, receipts, uncertain-mint
  resolution rules). **Gated live probe #1** on an operator-authorized
  throwaway run completes it: browser-free `push --yes` from an unbound
  local file through mint, generation and verified read-back.
- **Batch 3 — revision engine** (`leafspring-editor-text` + remainder of
  `leafspring-editor-structure` + essential `leafspring-editor-conflicts`;
  g or Sol high): `edit_text` splice path with degradation seam, per-block
  compare + op builder, ordering pass, type/scalar/reference ops, conflict
  gate + `--take local`, atomic scope refusal, full verification.
  **Gated live probe #2**: second push exercising insert-before-first,
  split, join, delete, reorder, pull agreement, repeat no-op.
- **Batch 4 — uncertainty hardening** (`leafspring-editor-recovery` +
  remainder of conflicts; g): pending marker lifecycle, exact-bytes retry,
  resolution-from-observation repair, acknowledgment reads, failure
  fixtures (lost response, restart-pending, stale preflight, local race,
  remote divergence), receipt edge cases (tags seam, derived title).

## 10. Acceptance scenarios (executable)

Offline: nextest + committed Yjs/fact fixtures + real-Yjs oracle (existing
harness patterns) + loopback push responder. Live: the two gated probes.
Every required story gets at least one automated scenario:

1. **Start on disk:** create → author title/headings/paragraphs/marks/links
   as plain Markdown → `check` passes offline; no network, no IDs anywhere
   in the file.
2. **First push (browser-free, required):** `push --yes` with **no locator
   and no browser** → auth-before-mint gate, `GET /new` harvest (loopback
   responder asserts the 307/Location contract), pending token recorded
   before content mutations, seed-verified, generation executed → exit 0;
   file gains keys (byte-diff: Key markers only), binding, confirmed nodes
   with real text-fact IDs; pull → equal; second push → zero mutations
   (loopback capture asserts an empty mutation list), no second draft
   (same token/root), no release receipts; observed publication recorded
   null while the file's declared intent is retained unchanged. Include:
   unauthenticated run refuses *before* any mint request (loopback asserts
   no GET /new was sent); lost mint response → no auto-remint, homepage
   list-discovery resolution or explicit user decision.
3. **Continue drafting:** insert paragraph before old first block, insert
   heading mid-document, edit text + mark, split (first piece retains key),
   join (vanished key detached), delete, reorder, heading level change,
   type change → single push expresses all; loopback fact state shows
   created/updated/detached/moved exactly per disposition table; unchanged
   blocks' facts byte-identical.
4. **Push revision mechanics:** new blocks' keys inserted exactly once;
   pending rows contain minted IDs and no fabricated confirmed IDs;
   interrupted-mid-batch simulation (responder drops after k mutations) →
   repair-by-observation converges without duplicate entities.
5. **Read and repeat:** pull output equals authored meaning (typed
   equality per block); repeat unchanged push → clean exit 0, zero
   mutations, receipts untouched, body bytes untouched.
6. **Useful failure:** list in source → exit 3 naming lists; remote edit
   meanwhile → exit 3 with pull guidance, `--take local` proceeds; poll
   present remotely → observation refusal, no writes to unrelated content;
   lost response → pending state reported, next invocation resolves or
   stops with explanation; concurrent-insert visibility fixture for the
   rebuilt-container case asserts the honest report wording.

## 11. Alternatives weighed candidly

The brief asks for equal-terms comparison, without assuming keyed structure
is nearly free. Honest machinery accounting for the recommendation beyond
the existing equality gate: association lookup and single-placement
occurrence checks; per-kind desired/current field ownership; three-way key
disposition; per-op-family validation and read-back; parser surgery on the
coverage invariant. Roughly a per-block comparator, an op builder, the
text-edit helper, and the fractional-key port — each bounded, each with
typed-equality and observation reuse, none free.

- **Generated-body stage/switch as the default save** (fresh page/body
  every save, entry-fact switch): one mechanism for all content; drops the
  existing-text edit path, per-block compare, moves and detaches. But it
  adds staging verification reads (`getFactsForRoots` route), entry-switch
  recovery, full node-table rebinding every save, internal-reference
  re-resolution, and stable-object exceptions (polls re-linked by hand);
  and it pays permanent product costs: every save detaches every block —
  anchors and associations churn even for untouched paragraphs —
  generations accumulate, concurrent web edits are orphaned wholesale
  invisibly, and the association-based architecture the codebase *is*
  (status/pull/diff/receipts) becomes write-only churn. Simpler write
  engine, not simpler verification or recovery. **Kept** as the explicit
  unimplemented `--recreate-body` escape and as the honest answer if the
  user one day declares body anchors disposable wholesale.
- **Whole-page rewrite every save** (same page entity, recreate all
  blocks): the coarse middle — drops edit+compare while keeping page/entry
  identity. Expressible *inside this engine* as the treat-everything-changed
  policy switch, so it costs nothing to keep available; as a default it
  still kills all block identities every save and recreates N blocks per
  save for a one-word edit. **Named degradation**, not default.
- **Fine-grained character/child matcher (LCS):** rejected by both prior
  waves; nothing in the new brief resurrects it. The splice is the whole
  retained continuity story.
- **y-prosemirror's reconciler in Rust:** declined (private/unstable API,
  JS runtime, schema bridge); stays the independent test consumer.

## 12. What would change this architecture

- Deployed `/new` mint divergence (non-3xx, changed Location): transport
  fallback per draft-extraction options (Next-Action POST, last resort);
  if creation proves impossible with supported auth, an explicit blocker
  report with proof — never silently shrinking story 2.
- Live probe rejections of our batches (payload handling, position
  tolerance, metadata acceptance) → transport fixes only.
- Pull observed to include detached facts → owned-projection allowance in
  `observe`; engine unchanged.
- Splice/mark machinery growing disproportionate → all-character overwrite
  in the retained `XmlText` (named, bounded).
- Per-block field wiring growing disproportionate → whole-page-rewrite
  policy switch (named, bounded, same engine).
- Authoring reality dominated by whole-body rewrites, or the user declaring
  internal anchors disposable → reconsider stage/switch as default (§11).
- Comments/annotations becoming core → continuity revisit behind the
  `edit_text` seam only.
- Lists demanded → representation addition (is-list flags), engine
  unchanged.

## Cross-review status

Stage-one draft corrected once (§1.1) on parent review for a direct brief
violation in Story 2; the correction is committed separately with its delta
stated, and the original remains evidence in history. The peer's
`design/drafting-push0.gpt6a.md` has not been read. A cross-review
addendum with revised stance follows on explicit parent dispatch after both
drafts are committed, as a separately committed addendum preserving this
text unchanged.
