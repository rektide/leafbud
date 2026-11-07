---
type: Design
title: "Yrs-backed Leaflet conversion and identity-preserving editing"
description: "Experiment-backed text substrate, transient fact reconciliation, identity bookkeeping, full-content obligations and implementation gates without saved bases."
resource: /design/yrs-conversion0.gpt6a.md
status: proposal
generated:
  at: 2026-11-07
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e48556aa8ffe9Fzbp0FmthuoDH"
    title: "Independent Astra design and user-commissioned offline Yrs/Yjs experiment; source research, implementation and verification"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Executed interoperability proof, initially committed as e2b22fb6"
  - resource: /README.md
    title: "Current append-only authoring and production layout at e48b62660d977e06de852d68bb1fc8a7a72b768a"
  - resource: /crates/drafts/src/yjs.rs
    title: "Existing handwritten CRDT integration and fresh-content encoder at the same starting revision"
  - resource: /crates/drafts/src/facts.rs
    title: "Current lossy fact parser and scan index, same revision"
  - resource: /crates/drafts/src/pull.rs
    title: "Full pull shape and discarded row metadata, same revision"
  - resource: /crates/drafts/src/author.rs
    title: "Current append builder, facet translation and incomplete reverse mapping, same revision"
  - resource: /crates/drafts/src/extract.rs
    title: "Existing projection, tolerant empty-text fallback and type coverage fixtures, same revision"
  - resource: /crates/drafts/src/push.rs
    title: "Existing transport envelope and response parser, same revision"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Prior read/source protocol evidence; not a live-write proof"
  - resource: /design/draft-extraction.glm53m.md
    title: "Mapped types and prior implementation outcomes; preserved as earlier evidence"
  - resource: /design/document-sync1.gpt61s.md
    title: "Snapshot-free complete-file proposal; push/publish judgment remains outside this design"
  - resource: /design/document-sync1.gpt6a.md
    title: "Compact witnesses and complete adapter proposal; refined here at conversion boundary"
  - resource: "https://github.com/y-crdt/y-crdt/tree/e0b476b0ce613453c70d0bb477b955808aa79a07"
    title: "Local source inspection: root/yrs READMEs, manifests, doc/block/store/transaction/decoder/text/XML source and tests, ywasm Doc binding"
  - resource: "https://github.com/hyperlink-academy/leaflet/tree/600c7b5f1f7b31b66fb626c293cc104312a12caf"
    title: "Local source inspection: schema/version/editor/Yjs realtime, fact attributes/mutations/server merge, factsToPagesRecord/getBlocks/list folding, image transport and package lock"
---

# Yrs-backed Leaflet conversion and editing

## Recommendation and evidence boundary

**Replace the hand-written Yjs codec with a Yrs-backed text adapter, and keep
fact-graph reconciliation as a separate conversion layer in `crates/drafts`.**
Load fresh pulled text state into memory, retain its CRDT identities, apply
specific edits, and emit one cumulative v1 delta per existing text fact. Build
non-text changes from explicit entity/edge correspondence. Verify the resulting
remote document by reading it back.

This now has an executable proof:
[experiment 05](../experiments/05-yrs-conversion/REPORT.md), initially committed
as **`e2b22fb6`**, passes **13 real-Yjs interoperability groups and two Rust tests**.
It covers ordinary marks, Unicode, hard breaks, deletion-only updates, no-op and
repeat behavior, competing updates and one fact-graph text mutation. It does not
implement the full structural/media adapter described below.

The file remains a complete current document with top-level `@springleaf`, in
one exclusive draft/published state. There is **no saved historical document,
fact graph, CRDT state, shadow copy, sidecar or serialized mutation batch**.
Transient remote state is an input to this attempt, discarded afterwards.
This proposal does not choose between the two round-one push/publish semantics.

**Evidence labels in this document:** executed claims refer to experiment 05;
source-verified claims refer to the exact revisions below; proposed contracts
and remaining gates are explicitly identified. No live write was performed.

## 1. What Leaflet and Yrs actually provide

Source pins: **L** = Leaflet `600c7b5f1f7b31b66fb626c293cc104312a12caf`;
**Y** = y-crdt `e0b476b0ce613453c70d0bb477b955808aa79a07`. Links below bind to
these revisions; deployed Leaflet parity has not been checked.

### Leaflet's shared state is text inside facts

The [attribute vocabulary][attributes] defines `block/text` as cardinality-one
`{type:"text", value:base64}`. Text, headings and quotes use that attribute;
footnote/comment entities can have their own independent text documents. Code
and math use **plain string facts**, `block/code` and `block/math`, not Yjs.
Pages, list nesting, image positions and all other topology remain external EAV
facts over UUID entities. A Yrs document is not the whole Leaflet document.

The [editor hook][collab] applies `Y.applyUpdate`, binds
`getXmlFragment("prosemirror")`, and persists `Y.encodeStateAsUpdate(ydoc)` as
full **v1** state. [createYjsText][seed] constructs a `paragraph` XmlElement
containing XmlText under that root. [Realtime][realtime] also uses v1 update
merging/application, with schema version in its message envelope. No v2 or
awareness/WebSocket protocol is required for the batch conversion adapter.

`meta` is a Y.Map; `schemaVersion` currently equals **1** for editor-written
documents. Absence means version 0. The [schema gate][schema-version] explains
why unknown nodes must not reach an older y-prosemirror schema: it can delete
unrepresentable content from the shared document. New supported writers should
stamp the version they actually implement when required, never downgrade it or
use an unstamped document to bypass compatibility. A no-op must not stamp.

The [schema][schema] specifies one paragraph (`doc.content = "block"`), XmlText
mark runs and inline `hard_break`, `didMention`, `atMention`, and `footnote`
nodes. A separate `multiBlockSchema` permits multiple paragraphs in other uses.
The [publication converter][pages] reads only the first paragraph. Ordinary
editing must not silently delete or ignore extra root children merely to mimic
that publication projection.

Named root types are interpreted by contract: the editor uses XmlFragment,
while some readers call `getXmlElement("prosemirror")` over the same sequence.
An update does not carry a complete root constructor declaration. Use
`get_xml_fragment`, inspect children/attributes, and report shape mismatches;
do not claim a successful root getter proves the original constructor.

### Verified Yrs surface

| Capability | Evidence and intended use |
| --- | --- |
| Version/features | [Manifest][yrs-cargo] is **0.28.1**, default features empty. No `weak`, `sync` or `small-client` needed. Root README's old feature-table version is not the dependency pin. |
| v1/v2 updates | [Decoder][decoder] has `Update::decode_v1/v2`; [transactions][txn] provide corresponding encoders. Use v1 explicitly for this Leaflet profile. The experiment proves v1 interoperability; it does not prove generic version sniffing. |
| Integration | `Doc::with_options`, `transact_mut().apply_update(...)` integrate deletes, concurrent structs and formatting. Inspect `ReadTxn::store().pending_update()` and `pending_ds()` to reject a supposed full pull with missing causal state. |
| XML and formatting | [XML types][xml] provide XmlFragment/Element/Text references, children, attributes and preliminary nodes; [Text][text] gives `diff`, `insert_with_attributes`, `remove_range`, `format`. Executed against real Yjs/y-prosemirror. |
| Delta generation | `TransactionMut::encode_update_v1()` emits edits made in that transaction, including deletes. `encode_diff_v1(&state_vector)` emits missing structs **plus the whole delete set**. `encode_state_as_update_v1` also includes pending state. These are not interchangeable no-op checks. |
| Index units | [Options][options] defaults to **bytes**. Set `OffsetKind::Utf16`, as [ywasm's constructor][ywasm] does. CRDT clocks are UTF-16 even when a Yrs API is configured for byte indexes. |
| Client IDs | [ClientID][block] defaults to random JS-safe **53-bit** IDs. Explicit IDs must be unique within the document's causal history; a collision can corrupt state, not merely change order. Use fresh IDs for a new editing attempt; deterministic injected IDs only in tests. |
| Snapshots | `ReadTxn::snapshot()` captures state vector plus delete set. Historical encoding [requires retained GC state][store] (`skip_gc`); a vector alone cannot restore deleted text. Neither snapshots nor persistent undo history are needed here. |

Use the exact Y Git revision for the next implementation proof, as the experiment
does. A published crate release can replace that dependency only after verifying
it contains the inspected behavior and passes the corpus. No path dependency on
the operator's archive and no permanent dual-engine compatibility layer.

## 2. Small, explicit conversion boundaries

Proposed modules inside `springleaf-drafts`, preserving existing generated record
types in `springleaf-leaflet`:

```text
conversion/
  text.rs       Yrs loading, inline projection, index map, edit compilation
  graph.rs      typed current facts, reachability, entity/edge correspondence
  document.rs   complete logical JSON projection and its editable field ownership
  plan.rs       pure desired/current comparison -> operations and checks
  resources.rs  resolved media/poll/reference inputs and acquisition requirements
```

These are logical boundaries; begin with a few files, not new service/framework
layers. `pull.rs` supplies transport, `push.rs` executes an already validated
plan, and CLI code supplies user decisions. Markdown may feed the same logical
document later, without a second synchronization model.

Conceptual APIs (proposed, not implemented production signatures):

```rust
decode_text(full_v1, schema_profile) -> Result<LoadedText>
LoadedText::project(resolvers) -> Result<RichText>
LoadedText::edit(desired_richtext) -> Result<Option<TextDelta>>
project(current_facts, envelope, resources) -> Result<WorkingDocument>
plan(current_facts, desired_document, bindings, resources, ids) -> Result<Plan>
verify(plan_expectations, freshly_pulled_facts, resources) -> Verification
```

`LoadedText` owns its `yrs::Doc`; internal XML references and scalar/byte/UTF-16
maps are ephemeral. `Plan` owns an in-memory mutation batch, explicit preconditions,
affected entity/edge identities and expected meaning. `ids` and timestamps are
injected for deterministic offline planning. No plan object needs disk storage.

The current `facts.rs` drops unknown payloads to a type label and discards useful
fields; `pull.rs` discards row metadata. Replace that in the conversion path with
a **raw-value-preserving current fact view** plus validated typed accessors.
Keep unknown facts in memory for conservative witnesses/preservation. Do not
serialize the graph under `@springleaf`. Cardinality violations and malformed
known payloads are errors, not arbitrary `[0]` decisions during editing.

Once acceptance gates pass, remove the handwritten varints, Item integration,
tombstone splitting and fresh-content text encoder from production `yjs.rs`.
Retain existing frozen input fixtures/reference scripts as evidence. Semantic
equivalence is the replacement gate; new Yrs struct layout need not reproduce
the old encoder's bytes. Domain-specific mark/inline conversion still belongs to
springleaf and should not be confused with CRDT integration supplied by Yrs.

## 3. Rich text projection and Unicode contract

### Forward projection

Load each **complete current** text update once; share that in-memory instance
through projection and planning. Inspect `meta`, roots and supported schema first.
Walk XML children, rather than rendering XML markup with `get_string`:

| Upstream | Logical value and reverse obligation |
| --- | --- |
| XmlText string delta | Append plaintext; convert attributes into canonical UTF-8 facet intervals. Preserve unowned comment attributes in the transient source. |
| `strong`, `em`, `code`, `strikethrough`, `underline` | `bold`, `italic`, `code`, `strikethrough`, `underline` features. Actual y-prosemirror zero-attribute marks use `{}`; older handwritten inputs may use `true`. |
| `link{href}` | `link{uri}` facet; a URL change is a changed value under the same mark key, not just “link remains present.” |
| `highlight{color:"1"|"2"|"3"}` | Slot 1 default; slots 2/3 resolve through root theme/defaults per `resolveHighlightColors`. Preserve slot identity through current style intent; RGB alone cannot uniquely recover it. |
| `hard_break` | One plaintext `\n`, represented as an XML child on write. It is not an XmlText character offset. |
| `didMention{did,text}` | Display `text`, with a mention facet covering those UTF-8 bytes; reverse reconstructs an atom, not plain marked text. |
| `atMention{atURI,text,href,icon}` | Display/facet/reference and optional presentation data. Upstream publication omits `icon`; a complete editor document must retain current icon intent outside the lossy publication projection. |
| `footnote{footnoteEntityID}` | `*` with `footnoteId`, `contentPlaintext`, `contentFacets`, resolving that entity's separate `block/text`. Keep shared identity; reject conflicting definitions for one ID. |
| `comment{commentID}` mark | Preserve on retained CRDT characters; ordinary body editing does not own comment records. Whole-range deletion may leave an unanchored comment, which is a reportable effect, not permission to delete the thread. |

This matches the directions in [YJSFragmentToFacets][pages], but extends the
editing contract beyond its lossy publication behavior. It currently flattens
only the first paragraph and omits comment anchors from facets. Full conversion
needs explicit handling of all supported nodes; unknown nodes/attributes or a
newer schema block mutation rather than becoming empty strings.

### Index mapping

The public JSON contract uses **UTF-8 byte offsets**, while all Yrs text operations
in this adapter use **UTF-16 units**. For every scalar boundary in each live text
node, construct `(byte_offset, utf16_offset)`. Keep a separate document-offset
map accounting for XML atoms and breaks. Example: `é😀é` has **9 bytes**, **5
UTF-16 units**, and **4 Unicode scalars**. A mention may display many bytes while
occupying one XML child; paragraph child indexes must never be text indexes.

Validate `0 <= byteStart < byteEnd <= plaintext.len()` and Rust char boundaries.
Normalize overlapping ordinary marks into nonoverlapping constant-mark runs;
reject conflicting values of the same key over an overlap. Atom spans must cover
their exact display ranges and not partially overlap other atoms. Preserve empty
mention/footnote cases deliberately rather than using an invalid zero-length
ordinary facet as an accidental encoding; their working-body representation is
part of the inline gate.

Diff Unicode scalars (or a production grapheme-aware algorithm that still maps to
scalars), never UTF-8 bytes or individual surrogate halves. Do not NFC-normalize
content implicitly. Combining-mark-only edits are legal and change exact text.
The experiment covers accented/astral/combining strings and invalid byte ranges.

Rust cannot hold isolated surrogates as a String. Reject such local JSON; don't
silently “repair” the intended document. Existing Yjs wire strings may already
contain U+FFFD after JS encoding. Preserve that actual wire value; the original
surrogate is irrecoverable. Yrs's [split implementation][block] and its tests
explicitly replace broken pairs with U+FFFD, matching Yjs; the experiment confirms
the JS-originated case. Our own compiled edits only target scalar boundaries.

## 4. Identity-preserving text edits

For an existing mapped entity:

1. Decode its fresh full update; reject incomplete/corrupt input. A missing
   `block/text` fact is different from corrupt bytes such as `b5ec6d`. A verified
   missing fact permits creation; corruption never authorizes silent replacement.
2. Project current and desired text with all semantic fields. If equivalent,
   return no mutation. Do not use update byte equality or state-vector equality
   as the semantic/no-op criterion: delete-only changes do not advance a vector.
3. Match retained paragraphs/inline atoms/text regions. Prefer explicit reference
   IDs and unchanged nodes; align text scalars inside retained XmlText. Produce
   a deterministic edit script. The prototype's bounded child-kind/scalar LCS is
   evidence for the operations, not the final large-document matching strategy.
4. In one new transaction, remove only selected known characters with
   `remove_range`; insert at mapped UTF-16 boundaries with
   `insert_with_attributes`; change only differing owned attributes with `format`.
   `Any::Null` closes/removes a mark. Explicit insertion attributes avoid accidental
   inheritance of nearby links/comments. Do not clear all formatting globally.
5. Preserve paragraph and surviving XML node identities. Add/remove actual XML
   break/mention/footnote nodes through `XmlFragment` operations. Update atom
   attributes in place when its identity remains. Do not reinsert an already
   integrated Yrs child under a different parent: cross-parent split/merge may
   require new child identities; record exactly what anchors cannot survive.
6. Project again in memory and require desired semantics. Encode **only the edit
   transaction** using `encode_update_v1`; separate the initial apply transaction.
   If multiple transactions are necessary, merge all their updates into one
   cumulative delta before exposing the mutation plan.
7. Emit `assertFact {entity, attribute:"block/text", data:{type:"text",value:b64}}`.
   There is exactly one final payload per entity/attribute per push attempt.

For a **new entity or genuinely absent text**, initialize a new Yrs paragraph and
emit its full initial update. For an existing entity, a fresh independent doc
with replacement text is wrong: the executed negative control produces two
paragraphs. Retraction/reassertion of the text fact to evade merging is also not
the normal edit algorithm; it discards causality and can strand collaborators.

The experiment proves preservation of an existing paragraph, both text nodes,
hard break, relative position and retained comment mark in its mixed edit. It
does **not** prove optimal identity retention across paragraph/text splits.
Production must minimize replacement and enumerate unavoidable reference/anchor
effects, particularly for repeated text and identical inline nodes.

### Server-specific delta constraint

[cachedServerMutationContext][server] stores a captured base and the last supplied
payload per `(entity,attribute,Replicache clientID)`, then calls
`Y.mergeUpdates([...latestValues, base])`. This cache assumes cumulative browser
writes. Our **one cumulative delta** satisfies its causal need when the current
base exists, as demonstrated in the simulated merge. Separate incremental chunks
under the same client in one push can lose prerequisites; the experiment proves
that failure. Replicache clientID is not the Yjs client ID.

There is an additional server limit: if merge decoding fails, it logs and writes
the latest unmerged value. Do not rely on that recovery path. If another writer
removes/recreates the text fact between pull and push, an incremental payload may
land without its causal base. Preflight/read-back are required; full-state
transmission is not a substitute for preserving entity meaning or server CAS.

## 5. Stable identity without saved content

Content-only record JSON has no unique identity for ordinary text blocks.
Page `id`, footnote IDs and reply/questions `key` already provide some identities;
array position, title, and content hashes do not provide all of them. Equal
paragraphs or “move and rewrite both” are fundamentally ambiguous without a
carried association. Yrs cannot solve this outer-graph correspondence problem.

**Proposed compact bookkeeping**, entirely inside the single top-level envelope:

```text
@springleaf.binding                 exclusive lifecycle + target locator/root
@springleaf.conversion.profile      leaflet schema/mapping profile
@springleaf.conversion.nodes[]      localId, entityId, current JSON pointer,
                                   optional relationship factId, fingerprint
@springleaf.assets[]                resource identity/hash + source locator
@springleaf.agreement               current accepted local/remote fingerprints
```

Each row names a node/edge; it contains no previous text, fields, XML or CRDT
bytes. A fingerprint is only an identity/drift hint, never a recoverable ancestor.
Multiple references to one page carry separate edge associations and one page
entity. List wrappers are derived grouping, while each actual list item has its
own association. Mapped drawing strokes, gallery images and poll options need
their own identities if edits/reordering are to preserve them.

Structured local edits maintain pointers with their nodes atomically. For manual
JSON edits, a pointer is an **association claim**, not a proof of identity:
unchanged uniquely fingerprinted nodes can be relocated automatically; duplicate
matches, changed/reordered ambiguous nodes, stale pointers or reused IDs need an
explicit mapping decision. Show the current proposed entity correspondence before
editing; never use similarity matching to silently decide an ambiguous deletion.
Even equal-length reorder-and-rewrite can be undetectable from content alone.
If the operator keeps metadata pointers unchanged, that means editing the entities
at those associations, not a guarantee the tool inferred the intended move.
New unbound nodes receive IDs once the plan is selected; repeated successful
operations reuse the returned associations instead of appending again.

All outgoing records strip `@springleaf`; fact mutations are constructed from
typed fields and verified IDs, not by forwarding the local envelope. Fingerprints
exclude transport/client counters but include meaningful unknown current state
for drift detection. Metadata read-back must not overwrite a concurrently edited
local file. There is no old JSON to perform a three-way merge against.

## 6. Fact graph ↔ complete working document

### A representational gate the earlier proposals leave open

**Pure published-record JSON is not an invertible editor format.** At L,
`factsToPagesRecord` explicitly omits `datetime`, `rsvp`, `mailbox`; rounds drawing
and canvas coordinates; separates list children by style; converts images to
blobs and polls to externally published records; omits some inline/editor data.
Simply applying Yrs to text does not remove these losses.

Recommended contract: the working JSON is a **record-shaped complete current
document**, with a documented local content profile for editor-only blocks,
unpublished resource definitions and editor intent not expressible in the public
record schema. Those are **current content in the body**, not hidden historical
payloads under `@springleaf`. Prefer existing canonical fields wherever expressive;
add the smallest typed extensions for the remaining meanings. The exact extension
names/schema need acceptance before implementing full coverage. Do not invent
fake poll AT-URIs, blob availability, or registered lexicons.

Consequences: stripping `@springleaf` is always required, but is not sufficient
to make every unpublished working body a valid published record. Native editor
mutations can represent those blocks; publication has a separate documented
projection. A record-only serializer must validate/materialize that projection
and identify unsupported editor-only fields, not silently upload local extension
data. If “body must be exactly the public lexicon at every moment” is instead a
hard constraint, full current-editor support is impossible without changing that
constraint or the upstream lexicons. This is a concrete representation decision,
not a reason to leave permanent skips in the converter.

### Topology and mutation ownership

Read a full `cookie:null` pull and envelope into memory. Select the explicitly
bound document/start page; preserve other publication navigation roots. The
upstream converter uses `root/page[0]` when no start page is supplied, whereas
current springleaf sorts it; do not treat either shortcut as universal ownership.

| Concern | Proposed round-trip and mutation policy |
| --- | --- |
| Pages/references | Map linear/canvas page IDs explicitly. `card`/`embedded-canvas` use `block/card` and canonical page references. Serialize a page once and allow multiple incoming references; validate dangling refs and detect cycles without silently omitting content. Preserve root/nav title/route/external-link facts unless the selected scope owns them. |
| Linear order | `card/block` is a cardinality-many edge with its own fact ID. Order by `(position,factID)` as [getBlocks][get-blocks] does. Keep unaffected keys; mint fractional keys only for additions/moves. Equal concurrent keys are possible and have a tiebreak, not a uniqueness guarantee. |
| Moves | Preserve entity and edge IDs. [moveBlock][mutations] overwrites the existing edge's parent/position. Pure offline plans can use equivalent explicit `assertFact` writes with reviewed fractional keys. Same-cardinality edge moves preserve IDs; linear↔canvas conversion changes the attribute and must deliberately retract/create the relationship because server upsert does not update its `attribute` column. |
| Lists | `block/is-list`, `block/list-style`, `block/list-number`, `block/check-list` and nested `card/block` encode items. Canonical list wrappers/runs are derived, not new entities. Preserve item IDs, nested cross-style order and explicit start/check states. Public lists split same/different-style children into separate fields: complete working order must not disappear through that projection. |
| Canvas/groups | `canvas/block` carries x/y; entity facts carry width/rotation/stack order; groups contain linear `card/block` children. Include fixed dimensions, mobile view, zoom lock, background/pattern and nested page links. Keep precise current geometry in working content; round only at publication projection. |
| Additions | Create page/block/resource entities before references and attributes; use caller-generated entity IDs and explicit IDs for many-valued facts. One attribute registry supplies cardinality/type validation. Do not regenerate existing IDs to simplify conversion. |
| Deletions | Explicit local removal from an owned document produces removal of that membership edge. Do not garbage-collect the entire pulled set. Keeping an entity detached preserves comments and incoming references; any requested actual entity deletion lists its consequences separately. |
| Type changes | Text↔heading↔quote should preserve entity, `block/text` and comments; update `block/type` and owned heading/size fields as [blockTypeOperations][type-change] illustrates. Other type transitions reconcile only relevant fields/children. An unavoidable entity replacement exposes old/new mapping and incoming-reference effects before execution. |
| Themes/metadata | Alignment belongs on block wrappers; page themes and root themes have distinct owners. Preserve HSBA/default/slot intent where RGB projection loses it. `theme/code-theme` is root-wide, so contradictory per-code-block themes cannot silently choose a winner. Metadata mutations are separate from facts; `updatePublicationDraft.localPublishedAt` is client-only at L. |

Do not call `removeBlock` indiscriminately for document replacement: it can
delete group descendants/question answers and enqueue image cleanup.
`deleteEntity` also removes incoming reference facts; `removeGalleryImage`
enqueues cleanup. Those are user-visible semantic effects. A normal body edit
must preserve unrelated comments, answer drafts, publication navigation and assets.
No automatic orphan pruning is proposed.

### Current block and resource coverage contract

The registry must account for **all 30 current block/type values**, both page
kinds, derived lists and all combinations. These are implementation requirements,
not capabilities already proven by the text experiment:

| Families (all enum values accounted for) | Required mapping/evidence |
| --- | --- |
| `text`, `heading`, `blockquote` | Full inline adapter, text size/level, alignment, comment-preserving edits. |
| `code`, `math` | Plain string facts, language and root code theme. No accidental Yrs wrapping. |
| `horizontal-rule`, `signup`, `post-header` | Correct type plus compact flags; no fabricated empty text document. |
| `button`, `link`, `embed`, `html` | Text/URL/title/description/preview; iframe-vs-legacy-HTML precedence, dimensions/aspect ratio. Resolve preview resources before mutation. |
| `bluesky-post`, `standard-site-post`, `standard-site-publication` | Reference identity, view/host/size/theme flags. Retain or resolve the full editor thread payload for Bluesky; a minimal `{post:{uri,cid}}` is not proven render-complete. |
| `members-only-delimiter`, `posts-list`, `recommended-pubs` | Audience/tier membership, filter tags, limit/view/reader controls and flags, preserving explicit defaults. Content after a delimiter remains in the working document. |
| `reply`, `questions` | Preserve `key`=entity identity on the same target, UI text/audience/theme flags. Answer/interactions are associated data, not disposable body children. Retargeting cannot claim to transfer them merely by copying a key. |
| `card`, `embedded-canvas`, `group` | Shared referenced pages, inline canvas dimensions and alt, group linear children and spatial container, list composition. |
| `image`, `image-gallery` | Source bytes/locator ↔ blob identity, alt/name/dimensions/width/full-bleed, ordered gallery entities and format/gap/width. Preserve asset entities on ordinary changes. |
| `drawing` | View box, strokes, pressure, fill membership/order and precision. Publication rounds and derives fills, so exact inverse needs working semantics, not invented reconstruction of lost fractions. |
| `poll` | Editable inline option definitions/identities plus actual external pollRef when one exists. `poll/options` is ordered; the current publication converter does not explicitly sort it. Verify ordering against the editor and fixture, not a map's iteration order. |
| `datetime`, `rsvp`, `mailbox` | Explicit current-content representation and native fact mappings, despite no published block counterpart. Date needs timezone/dateOnly; mailbox preserves archive/draft references (its UI is deprecated). RSVP field/interaction mapping remains a source/prototype gate. |

Resource interfaces should separate **read resolution**, **required preparation**,
and **execution receipts**. A pulled draft image can have a content descriptor
and compact source locator without uploading anything. Actual bytes can be read
transiently to verify digest/dimensions; do not claim that computing a CID means
the blob already exists in a PDS. A new/unpublished poll needs current definition
data, not a fake reference. Native publication may later materialize it.

[addImage][image] demonstrates `minilink-user-assets`, prepared URL/dimensions/
thumbhash, upload and finalization; it is source evidence, not a verified CLI
auth/transport recipe. Keep `resolve_asset`, `prepare_editor_asset`,
`resolve_poll`, and `resolve_embed` as explicit interfaces with fixtures until
transport/auth is verified. Cover images, page backgrounds, wordmarks and
website previews use the same boundary. Missing required bytes/reference data
fails the plan before structural mutations. No uploaded-asset cleanup is implicit.

## 7. Plan, execute, read-back and honest concurrency

**Plan (pure once inputs are supplied):** validate the full working profile and
bindings; inspect fresh fact state; classify witness drift; resolve resource
requirements; compare semantic content; select scoped text/field/edge operations.
Inject ID/time sources, preserve untouched fields, list deletion/replacement
effects and unknown-state blockers. Deterministic means the same logical inputs
produce the same operation choices; unordered internal CRDT encoding need not
yield byte-identical payloads. No network side effects occur inside planning.

**Execute:** ensure current local hash and reviewed remote witnesses still match;
otherwise replan/review. Supply resources first, then dependency-ordered entity,
attribute and edge changes. One final text delta per fact/client per batch.
Retain exact mutation IDs/body only in memory for transport retry. Source
`push.ts` skips already acknowledged IDs, but a throwing mutation is also
acknowledged after its buffered writes roll back; some authorization failures
return without writing. HTTP success is not semantic success.

**Read-back:** full fresh pull → same projection. Verify both intended semantics
and preservation obligations: identities, references, unmodified fields and
reachable resources. Report `achieved`, `partial`, `concurrent divergence`, or
`unverifiable`; advance accepted hashes/associations only for the actual verified
result. A second invocation against achieved state must plan zero additions and
zero text changes. After a crash, pull and replan from current state; a compact
pending operation ID/hash can explain uncertainty but cannot regenerate a lost
batch from absent historical inputs.

Yjs merging preserves CRDT intent encoded against observed items: a deletion
targets those known items, while an unseen concurrent insert can survive. The
executed `abc → ac` plus concurrent `R` yields **`aRc`** in both arrival orders.
Do not automatically replan until `R` disappears. Canonical JSON is a desired
current value, not an offline CRDT operation log; it does not reveal whether an
absent collaborator paragraph was intentionally deleted or never seen. Drift
checks and explicit current-endpoint decisions remain necessary without a base.

Structure is weaker still: field and edge assertions are ordinary fact writes,
not a Yrs tree. The [push handler][push-handler] takes a **token-scoped** advisory
lock, not a caller-supplied revision precondition. It does not establish global
CAS, an atomic successful batch, or serial ordering against every possible writer.
Distinct tokens pointing at the same entities may not share that lock; the
read-base/merge/upsert source alone therefore cannot prove absence of lost updates
across those paths. This race is not exercised by the experiment. Native
`Y.mergeUpdates` convergence assumes all required updates are retained/delivered;
it cannot repair a server-side lost update it never receives.

## 8. Capability stages and acceptance gates

| Stage | Coherent work and required proof |
| --- | --- |
| **Text substrate — partial proof landed** | Promote the exact dependency/API choice only after replacing handwritten codec tests with semantic corpus tests. Experiment 05 establishes ordinary marks/UTF-8/UTF-16/hard breaks/deltas/no-op/competing insert and real-Yjs application. Keep its negative controls. |
| **Full inline profile** | Mentions, footnotes including shared references/cycles, all highlight slots/defaults, comments, empty nodes, extra paragraphs, malformed/truncated/trailing bytes, explicit size/depth limits. Compare both Yjs and Leaflet-derived schema model, then actual editor mounting in an offline browser fixture. No silent first-paragraph-only output. |
| **Identity and edit compiler** | Production-sized deterministic matching; retained characters and anchors under middle edits, repeated text, splits/merges and changed atom attrs. Invalid inputs yield contextual errors with no executable partial plan. Test a meaningful no-op with tombstones and format-only and delete-only changes. |
| **Fact planner** | Read-only complete current facts; page/ref identity; scalar fields; add/move/remove/type-change/list/canvas scenarios; comments and unrelated nav/answers preserved. Apply plans to a simulator faithful to cardinality, reference removal and failure acknowledgment, not only a map that always succeeds. |
| **Complete document/resource profile** | Accept the representation gap described above, then prove every current block/page/field and combinations, including draft-only values and unresolved-to-resolved media/poll transitions. Complete current content cannot pass with warning-and-skip. Transport gates are separately named; no live probe in this experiment. |
| **Integration readiness** | Pure planning from fixtures, repeated push/read-back no append, drift before/after planning, partial acknowledgments, remote-delete/recreate, multi-token races, dropped responses and local edits during await. Verify metadata stripping and no historical state files. Only then replace production append authoring for synchronization. |

Run focused nextest/Clippy checks and real-Yjs interoperability in the converter
test surface. The eventual ordinary network transport needs deployment evidence;
an operator-authorized live probe is separate from this completed offline work.

## Connections and remaining decisions

- [Draft extraction](draft-extraction.glm53m.md) supplied the type inventory and
  older byte-parity evidence. Its hand codec, append-only reverse path,
  warn-and-skip policy and empty-on-decode-error behavior are not the destination.
- [Sol round one](document-sync1.gpt61s.md) and
  [Astra round one](document-sync1.gpt6a.md) agree on snapshot-free current files
  and complete coverage. This proposal supplies their missing CRDT edit mechanism
  and identifies the non-invertible public-record projection as a concrete gate.
  It leaves their save/release disagreement untouched.
- [Experiment 05](../experiments/05-yrs-conversion/REPORT.md) is reproducible
  evidence for the narrow substrate, including failures and negative controls;
  this broader adapter remains proposed.

The next consequential decisions are the complete working-body profile and the
explicit binding policy for ambiguous manual reorders. The next proof work is
full inline/reference preservation, followed by structural/resource coverage.
None requires keeping a historical document on disk.

[attributes]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/attributes.ts
[collab]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/TextBlock/useCollabText.tsx
[seed]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/createYjsText.ts
[realtime]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/yjsRealtime.tsx
[schema-version]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/TextBlock/schemaVersion.tsx
[schema]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/TextBlock/schema.ts
[pages]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/factsToPagesRecord.ts
[yrs-cargo]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/Cargo.toml
[decoder]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/updates/decoder.rs
[txn]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/transaction.rs
[xml]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/types/xml.rs
[text]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/types/text.rs
[options]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/doc.rs
[ywasm]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/ywasm/src/doc.rs
[block]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/block.rs
[store]: https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/store.rs
[server]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cachedServerMutationContext.ts
[get-blocks]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/getBlocks.ts
[mutations]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
[type-change]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/blockTypeOperations.ts
[image]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/addImage.ts
[push-handler]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/push.ts
