---
type: Design
title: "Preliminary design and implementation handoff — local MDX authoring"
description: "Preliminary Satteri/MDX handoff with a subsequent keyed bidirectional/Yrs addendum: stable current-content identities, automatic key assignment, modest synchronization, and explicit recreation fallback."
resource: /design/authoring-handoff0.gpt61s.md
status: preliminary
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
updated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4900b8d7ffeDT9QuFwmKRAB7t"
    title: "User's authoring/storage direction and experiments; preliminary handoff followed by clarification that MDX keys should retain Yrs/Leaflet identity and enable some bidirectional conversion"
  - resource: /design/authoring-recommendation0.gpt61s.md
    title: "Current Satteri recommendation with alternatives and adoption boundaries"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Native publication and stable-Leaflet/fresh-subtree replacement recipe"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Executed compiler, independent consumer, and bounded staging proof"
  - resource: /experiments/13-satteri-authoring/src/model.rs
    title: "Actual transient document/inline/block types and limited current profile"
  - resource: /experiments/13-satteri-authoring/src/simulator.rs
    title: "Actual stage/verify/switch and interrupted-write model; not the real server"
  - resource: /experiments/14-satteri-authoring-attributes/REPORT.md
    title: "Optional brace-key syntax, compiler equivalence, and explicit two-parse adapter obligation"
  - resource: /experiments/14-satteri-authoring-attributes/src/lib.rs
    title: "Actual compile/prepare exports and reuse boundary"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Verified text wire semantics, no-op and wrong-existing-root replacement evidence"
  - resource: /design/yrs-conversion0.gpt6a.md
    title: "Detailed identity-preserving Yrs edits and current-block/resource/fact mapping; revisited for the bidirectional addendum"
  - resource: /Cargo.toml
    title: "Current production crate/dependency layout, separate from experimental workspaces"
  - resource: /crates/drafts/src/lib.rs
    title: "Existing web client and create/pull/push/author/extract/text modules"
  - resource: /crates/cli/src/config.rs
    title: "Existing Figment configuration precedence and XDG storage"
---

# Preliminary design and implementation handoff

## Read this first

**Goal: let someone author a document locally in MDX and make Leaflet match it,
then publish or update it through Leaflet.**

**Read the [keyed bidirectional addendum](#keyed-bidirectional-addendum) before
implementing.** The original handoff below emphasized fresh-body recreation.
The user's subsequent clarification changes the target: preserve associations
between MDX blocks and existing Leaflet/Yrs content, generating durable keys
automatically when needed. Recreation remains a deliberate fallback, not the
default for every edit. Earlier sections are retained to make that change clear.

This is a preliminary implementation brief, not an accepted schema or permission
to execute remote writes. The user requested a document, not implementation in
this pass. Proposed command names, storage details and behavior choices are
identified below. No production code changed or live requests were made.

For a short decision account and alternatives, read the
[recommendation](authoring-recommendation0.gpt61s.md). For exact experimental
receipts, use the [experiment index](../experiments/README.md). Start substantive
code work with experiments **05, 13 and 14**, connecting the existing-state Yrs
proof to the MDX/keying compiler rather than promoting only fresh encoding.

## 1. Direction, assumptions, and boundaries

### User-established constraints

- The local file is authoritative current content. MDX is storage, not an export
  view backed by a second JSON file.
- A document is **draft or published**, never both. A published document may
  have unreleased edits without becoming a draft again.
- Local metadata lives in the same file under `@springleaf`; no separate sidecar,
  saved base document, hidden snapshot store, persisted Yjs history or shadow JSON.
- Ordinary headings, paragraphs and lists remain Markdown. Optional identifiers
  are welcome; mandatory Paragraph/Heading component wrappers are not.
- Native Leaflet lifecycle is the primary desired route. Direct PDS operations
  remain a separately tracked alternative, not an implicit fallback.
- Filenames and `drafts/`/`posted/` directories are convenient defaults, not
  document identity. Filename/path tidying must be optional.
- No multiplayer/live-editor architecture is required. Preserve future options
  where cheap; do not make them prerequisites for local authorship.
- Complete current-block/media support is the destination. Unsupported input
  must fail visibly, not disappear during an apparently successful save.
- No migration or compatibility machinery is needed for valuable legacy data.
  Do not invent such obligations. Existing experiment evidence stays preserved.

### Recommended working assumptions, still reviewable

1. **Save versus release:** `save` changes the editor; `publish` saves as needed
   and releases publicly. Both verbs have the same meaning for drafts and posts.
2. **Explicit replacement:** source owns a selected generated body. Confirmed
   replacement can overwrite web edits there, without automatically merging them.
   This simplifies the earlier drift-aware proposals; it is not an already
   accepted instruction to discard arbitrary web changes.
3. **Fresh subtree:** keep the Leaflet identity/root/entry relationship; recreate
   ordinary body entities instead of compiling character-level CRDT edits.
4. **Parser:** adopt the proven Satteri route and optional brace adapter, subject
   to real workspace integration. Keep the modern mdxjs fork as the fallback.

### Explicit non-goals for the first implementation

Full editor-to-MDX import, automatic local/web merging, character-level anchor
preservation, general MDX formatting, JSX execution, React/HTML rendering,
publication scheduling, automatic orphan cleanup, and simultaneous PDS/native
management. None should block a useful local authoring slice.

## 2. Intended workflow and command sketch

These commands are proposals, not the existing implemented surface:

```sh
springleaf create essay.mdx --title "An essay" # local only
springleaf check essay.mdx                    # compile; no remote writes
springleaf save essay.mdx --replace           # preview replacement plan
springleaf save essay.mdx --replace --yes     # save to Leaflet editor
springleaf publish essay.mdx --replace --yes  # save if needed, then publish
springleaf edit essay.mdx --published-at TIME # local metadata edit
springleaf edit essay.mdx --now               # local date edit
```

The normal short loop is **write → publish → edit → publish again**. Save is
available for web preview/review without going public. A published file remains
published after either local edits or an editor-only save.

Without `--yes`, remote-write commands must not create a draft, upload assets,
stage facts, switch the body or publish. Read-only inspection may improve their
plan, but local `check` should remain usable without auth or network access.
An unresolved remote asset can be listed as a preparation requirement rather
than fabricated as an already uploaded resource.

Use explicit file paths as the unambiguous interface. Friendly workspace lookup
is convenience; filename stem must never choose a published identity. CLI naming,
the exact `--replace` confirmation policy, and whether `push` is offered instead
of `save` need one deliberate choice before the public interface is landed.

## 3. Canonical source and compact metadata

An authoring file has document-level frontmatter and an MDX-shaped body:

````mdx
---
format: springleaf-mdx/1
title: An essay
description: Written locally
tags: [notes]
---

## Introduction {#introduction}

Ordinary **Markdown** with a [link](https://example.com).
{#central-argument}

<Key id="example" />

```rust
let answer = 42;
```

<Poll key="next-topic" options={["Gardens", "Maps"]} />
````

Frontmatter also owns optional publication target, date, theme/cover intent and
delivery preferences as those fields are implemented. A body heading is not
automatically removed and treated as document metadata.

### Local state shape — preliminary

Use a versioned typed structure, not arbitrary fields copied from old sidecars:

```text
@springleaf
  version, localId
  binding: unlinked-draft | leaflet-draft | leaflet-post
    baseUrl, token, rootEntity, entitySet, entryFact   when linked
    documentUri                                    only when published
  applied
    inputHash, generation, compilerProfile, confirmedAt
  released                                         only when published
    inputHash, cid, publishedAt, confirmedAt
  names                                           optional tidy witness
    acceptedFile, acceptedPath, acceptedAt
  assets                                          compact locators/digests only
  pending                                         only while unsettled
    kind, phase, generation, inputHash, knownTargetIds
```

The exact YAML schema is not frozen. The important distinction is **current
author content versus operation receipts**. No receipt contains another body,
full fact graph, encoded Yjs history or serialized mutation batch.

The tagged binding prevents simultaneous draft/post lifecycles. A post retains
the token/root because native updates use that same Leaflet; that is its editor
locator, not another independently maintained draft. Keep the actual returned
AT URI, never derive it from path, title, filename or a default account.

Preserve the source body byte-for-byte. No-op operations do not rewrite the
file. A real metadata update must preserve unknown author frontmatter values;
frontmatter formatting/comment preservation is a separate implementation choice,
not a requirement to build a general MDX rewriter. Define and test that behavior
before shipping automatic state writes. Use atomic file replacement and check
that the source did not change during a network await before finalizing receipts.

### Dates, target defaults, names, and paths

- New local drafts need not have a publication date or assigned public path.
- Resolve an unbound document's target from explicit input, source intent and
  existing workspace/global defaults. Proposed precedence is command input >
  document intent > workspace > global. Once bound, defaults cannot silently
  retarget it; a conflicting target needs a separate explicit operation.
- Preserve the actual publication date on updates unless the author sets another
  one or chooses `--now`. When source date is absent, the release receipt can
  retain the actual date for subsequent native publication arguments.
- Native ordinary publication currently chooses the first rkey and assigns
  `/rkey`. Trusted import has more control; an ordinary custom-slug setter has
  not been verified. Report this before publication; do not silently use PDS.
- Local filename/path tidy remains an optional two-way operation with compact
  pair witnesses. Renaming a file leaves its binding intact. Tidying path from
  filename expresses local intent; it does not bypass native path normalization.
  A server-assigned path must not automatically rename or move the file.

These preserve earlier naming requirements without making a naming-sync engine
the prerequisite for the first compiler/transport slice.

## 4. Language and compiler contract

### Parser and identities

Use direct `satteri_pulldown_cmark::parse` and public MDAST arena/codecs. Do not
route authoring through generated JS, HAST or HTML reparsing. A reserved native
self-closing Key names the next supported sibling inside its own container;
components may also have explicit author keys. Duplicate/conflicting/orphan
names are diagnostics. Keys are not HTML anchors or guarantees that a prose
entity/comment anchor survives recreation.

Optional `{#key}` is the small tested heading/paragraph dialect from 14. Isolate
its recovery-tree discovery, byte-preserving mask and strict-reparse adapter.
Never treat an error-recovery tree as a validated document. Keep original byte
positions for diagnostics. A dependency update must rerun this adapter's corpus.

### Typed transient document

The public compiler boundary should not expose Satteri's arena as the domain
model. Lower to typed pages, blocks, inline nodes, reference names, author keys,
source spans and resource requests. This is **current authoring content**, not
merely published-record JSON. Retain layout precision, theme-slot intent and
unpublished resource definitions; native publication is a separate projection.

Rich components use declared props/child schemas and literal values. Unknown or
unimplemented components, invalid references and invalid fields return useful
diagnostics and no executable partial document. No imports, arbitrary calls,
computed execution, or spreads are needed for the first data language.

A first useful profile can mirror 13: normal headings/prose/marks/links/breaks,
simple lists, a bounded quote form, code/rules, named linear Pages/PageLinks,
Run/HardBreak and a keyed Poll. Explicitly identify that profile in outputs.
Canvases, drawings, images/galleries, themes, mentions/footnotes and the rest of
the current Leaflet vocabulary must be added by capability, not assumed covered.

### Text and stable objects

Yrs supplies valid fresh v1 text documents: `prosemirror` fragment, paragraph,
marks and actual XML hard-break nodes, using the verified UTF-16 convention.
Public-record UTF-8 facets are an interchange concern, not stored authoring
structure. Code/math are plain facts where Leaflet expects them, not Yjs text.

Generate a full initial update **on a new text entity**. Never assert unrelated
fresh text over an existing text fact and assume overwrite: Leaflet merges it.
No character-LCS editing engine is needed for this workflow.

Ordinary output IDs derive from document identity, generation and deterministic
compiler paths. A meaningful author key can survive source moves without
implying permanent prose entity identity. Persistent interactive objects such as
Polls need stable key-derived identity outside the generation seed. A change
that reinterprets existing votes requires a new key/object or an explicit future
policy, not automatic reuse. Internal page/footnote references resolve together
within the newly generated graph.

## 5. Suggested production boundaries

Recommended layout; module names can adjust to the first coherent slice:

```text
crates/authoring/       new pure local-authoring domain
  source               frontmatter, original bytes, owned metadata updates
  parse                Satteri adapter and optional brace syntax
  model                parser-independent current document and diagnostics
  resources            asset/reference requests and resolved inputs
  fingerprint          versioned semantic input hash

crates/drafts/         existing Leaflet web domain
  materialize          typed document -> entities, facts, metadata operations
  text                 Yrs fresh encoding and semantic verification
  stage                observe, stage, verify, switch, recover
  publication          native publish action adapter and outcome checks
  create/pull/push      existing transport, reused deliberately

crates/cli/            commands, config layering, confirmation, output
crates/leaflet/        public record types/projection/validation as needed
crates/atproto/        existing public reads and explicit PDS operations
```

Keep dependency direction simple: drafts may consume authoring types; authoring
does not depend on the network client or Leaflet transport. A new domain crate
is a recommendation, not a framework: a small pure module boundary is acceptable
if it demonstrably avoids unnecessary crate machinery. Do not put new parser
semantics into the CLI entrypoint or create a dependency cycle.

Conceptual data flow, not frozen Rust signatures:

```text
compile(source, profile) -> CheckedDocument | Diagnostics
prepare_resources(document, supplied_inputs) -> ResolvedInputs | Requirements
materialize(document, binding, generation, resources) -> Materialization
observe(binding) -> CurrentRemote
plan_resume(materialization, current_remote) -> Operations
execute(operations, transport) -> Attempt
verify(materialization, fresh_reads) -> Achieved | Partial | Unverifiable
publish(binding, selected_metadata, delivery) -> PublicationOutcome
```

Compilation is pure; acquisition and remote effects have explicit boundaries.
Inject ID/time sources in planning tests. The in-memory materialization may
contain entities, facts, expected text, source attributions and ownership scopes.
It is not saved as a companion file. The renderer/parser can be replaced at the
small lowering seam if integration makes the demonstrated mdxjs fallback better.

## 6. Remote save and publication algorithm

1. **Compile before effects.** Validate all supported source and references;
   resolve required local/media inputs. A compile error produces no staging plan
   or remote writes. Preparing uploads requires execution confirmation too.
2. **Establish or verify binding.** Create a Leaflet once when unlinked; validate
   authentication first because `/new` can create anonymously. Persist its
   returned locator immediately. For existing documents verify rights/entity set
   and the selected `root/page` fact; do not guess among navigation roots.
3. **Plan the generation.** Derive fresh ordinary entity/fact IDs and source-owned
   metadata changes. Reuse stable semantic objects only under their explicit
   identity/content policy. Resolve assets before facts refer to them.
4. **Stage disconnected content.** Create entities and facts in the existing
   entity set without first replacing the active body. Respect dependency order
   and transport batch limits. One fresh text payload per new text entity.
5. **Read back and verify.** Compare the complete owned inventory, relationships
   and decoded text for the supported profile, not merely a successful response
   or a subset of expected facts. Read explicit staged entities in bounded sets;
   a depth-three read of one root does not prove an arbitrary nested graph.
6. **Switch the existing entry.** Update the explicitly bound `root/page` fact's
   target, retaining its fact ID/position and the Leaflet root/token. Apply
   source-owned root metadata/theme changes under explicit ownership. Verify
   active reachability and the combined state before declaring save successful.
7. **Publish only when requested.** Call Leaflet's native publication on that
   same bound Leaflet, with explicit metadata/date/delivery arguments. First
   publication records the actual returned URI/path/date and changes the tag to
   published. Updates require the same existing URI. Verify linkage and outcome;
   saving content alone does not prove publication or delivery.
8. **Finalize locally.** Recheck unchanged source, update compact receipts and
   clear settled pending state. No automatic source-body rewrite or relocation.

The entry update is an ordinary fact write, not an atomic compare-and-swap API.
Root metadata and stable-object writes may become visible separately. The
server may acknowledge failed mutations. Preflight/read-back are essential;
neither the simulator nor native publication lock supplies global atomicity.

**Do not blindly promote `simulator.rs`.** It has no auth, SQL, real cardinality
rules, concurrency or publication implementation. Its Poll completeness signal
is explicitly model-only. Carry its desired invariants into real tests without
treating its map insertion behavior as the server protocol.

## 7. Repeats, failure recovery, and replacement effects

A semantic input fingerprint covers normalized typed content, relevant source
metadata, compiler profile and asset digests. It excludes source spans, formatting
trivia and owned operation receipts. Preserve meaningful text/ordering/keys;
do not hash random CRDT encoding bytes as desired content.

An unchanged local hash is not proof the web body is unchanged. Before skipping
save, use the last applied generation to inspect the current owned output against
today's compiled source. Otherwise an explicit replacement may regenerate. Neither
path appends another visible body or creates another public document.

Write compact pending generation/hash/phase before effects. On restart:

- If source/profile inputs still match, inspect staged facts and resume only
  compatible work. Existing equal text is left alone, not freshly re-encoded and
  merged again. Missing text may be initialized; different/corrupt staged text
  requires a new generation or explicit error.
- If source changed, plan a new generation. Keep the known binding. Do not
  reconstruct a lost historical payload or blindly replay acknowledged failures.
- A lost switch response is resolved by checking the entry's actual target.
- A lost publish response is resolved through actual linkage/output before retry;
  post existence alone need not prove every email/share effect completed.
- A lost mint response is an uncertain creation, not permission to create again
  automatically. Show recovery instructions and any known identifiers.

Replacing the selected entry retires old generated body membership. Do not run
`removeBlock`, `deleteEntity`, asset cleanup or global pruning automatically.
Detached generations may accumulate remotely until a deliberate cleanup policy
is commissioned. Old block/page anchors and comments do not automatically attach
to the new generation; warn concisely before replacement. Document/public URI
continuity, not multiplayer character continuity, is the intended promise.

## 8. Experimental reuse map

| Read/reuse as evidence | Promote deliberately | Do not assume |
| --- | --- | --- |
| [13 `author.rs`](../experiments/13-satteri-authoring/src/author.rs) and [`model.rs`](../experiments/13-satteri-authoring/src/model.rs) | Key consumption, native typed access, literal/component schemas, inline lowering and page-reference validation | Its bounded model represents every Leaflet type or accepts owned `@springleaf` state already |
| [14 `adapter.rs`](../experiments/14-satteri-authoring-attributes/src/adapter.rs) | Optional brace discovery/mask/reparse, source coordinates and identity collision corpus | Recovery-node shape is a stable upstream extension promise |
| [13 `facts.rs`](../experiments/13-satteri-authoring/src/facts.rs) | Real fact/data shapes, deterministic generation, membership/reference and stable-Poll rules | Test fixture IDs or fresh-empty mutation batches are safe crash-recovery transport code |
| [13 `text.rs`](../experiments/13-satteri-authoring/src/text.rs) and [`reference.mjs`](../experiments/13-satteri-authoring/reference.mjs) | Fresh v1 Yrs profile and independent Yjs/ProseMirror oracle | All mentions, themes, footnotes and text shapes are covered |
| [13 `simulator.rs`](../experiments/13-satteri-authoring/src/simulator.rs) | Stage/verify/switch/repeat/interruption acceptance cases | Deployed server/cardinality/auth semantics or atomic publication |
| [05 report](../experiments/05-yrs-conversion/REPORT.md) | Wire constraints, Unicode and existing-root negative controls | Its incremental edit compiler is required for recreation |
| Existing `drafts` create/pull/push transport | URL/auth handling, explicit transport seams and known read shapes | Current append authoring is the replacement algorithm or HTTP success proves achievement |

Known starting pins: `satteri-pulldown-cmark` **0.6.7**, `satteri-ast` **0.6.3**,
`satteri-arena` **0.4.3**, published source
`7d1297b7e4551a1c5124fa07bd2d1d05f66236e6`; Yrs **0.28.1** at
`e0b476b0ce613453c70d0bb477b955808aa79a07`. Check manifests/receipts in 13/14 before
integration. Registry/Git pins identify tested behavior, not an instruction to
assume minimum-Rust or all deployment targets work. The root workspace currently
has no production Satteri/Yrs dependency.

## 9. Landing sequence and acceptance

These are capability gates, not time estimates or authorization to begin:

| Stage | Coherent delivery | Required evidence |
| --- | --- | --- |
| **A — local compiler** | Production dependency integration, parser-independent model, native Key/optional braces, selected supported profile, check output | Root feature/dependency build; no auth/network needed; normal prose/components/keys; invalid input has no usable plan; exact original body retained |
| **B — local source lifecycle** | Local create, embedded state, source-relative assets, owned metadata updates, date helpers | Draft/post tag invariants; moved filename keeps binding; unknown author values preserved; no-op byte identity; concurrent local edit not overwritten |
| **C — remote staging** | Resource preparation interfaces, observe/plan/execute/verify and explicit entry switch | Offline transport/cardinality/failure tests; then separately approved throwaway-draft probe demonstrating staged reads, rights and actual switch behavior |
| **D — native release** | Native action discovery/auth, first publish/update and outcome verification | Offline transport tests; then separately approved live first/update proof with stable document URI, date/path normalization and explicit delivery effects |
| **E — content expansion** | Canvas/media/theme/inline and remaining block mappings; optional tidy/status conveniences | Capability-specific semantic fixtures/read-back; full coverage inventory; no warn-and-skip success; no automatic cleanup |

Do not hold all useful prose authoring hostage to every future component, but
label the implemented profile and reject unsupported source. Stages can be
adjusted at architectural checkpoints; staged remote proof is a more valuable
next experiment than another general parser comparison.

Production Rust changes should run formatting, workspace Clippy and nextest,
plus the independent Yjs consumer where text encoding changes. Existing isolated
experiments remain evidence, not production path dependencies. Preserve their
fixtures and selected regression cases when promoting code; do not erase them
because a new implementation is accepted.

Existing `drafts author` should become explicitly named `append` if retained;
it must not be mistaken for the new replacement path. Pre-1.0 changes do not
need compatibility aliases, but decide the receiving CLI surface explicitly.
Current JSON export/build/PDS tools need not be deleted merely because MDX is
the new authoring direction. Their long-term modes and boundaries are separate
decisions, not an excuse for silent native-to-PDS fallback.

## 10. Decisions to settle at implementation kickoff

1. Accept source-authoritative, explicitly confirmed replacement as the normal
   save policy, with concise web-change reporting rather than mandatory merging.
2. Settle save/publish names and execution confirmation; keep public-release
   visibility predictable across both lifecycles.
3. Approve the minimal frontmatter/owned-metadata schema and patching behavior,
   including effective target/date persistence. No saved-content field is allowed.
4. Select the initial content profile and resource boundary; define the component
   vocabulary without inventing fake public refs for unpublished content.
5. Verify actual detached-entity reads/switch and native-action invocation before
   promising the production path. Source inspection and a model are not deployment
   evidence. All live writes require separate explicit operator authorization.

Other details can evolve within these boundaries. Neither an obscure MDX
multiline-inline-code case, a missing property-level source span, nor a theoretical
parser-speed difference should reopen the entire design without an actual need.

## 11. Related and parked work

- [Current recommendation and alternatives](authoring-recommendation0.gpt61s.md):
  why Satteri now has sufficient positive evidence; modern mdxjs remains the
  demonstrated fallback, Rushdown/custom grammar and direct markdown-rs are
  conditional alternatives rather than parallel production implementations.
- [Recreation design](mdx-recreate0.gpt6a.md): detailed native-linkage and entry
  replacement source reasoning. This handoff adds concrete experimental reuse
  and production gates without replacing that evidence.
- [Yrs conversion design](yrs-conversion0.gpt6a.md): full inline/fact/resource
  obligations and optional future fine-grained editing, not the initial workflow.
- [Earlier sync designs](document-sync1.gpt61s.md) and
  [Astra's independent version](document-sync1.gpt6a.md): preserve the naming,
  identity and compact-witness discussion. Their broader synchronization choices
  are not prerequisites for local-source recreation.
- Existing work items `leafspring-pds-publish`, `leafspring-pds-sync`, and
  `leafspring-mdx` hold alternative PDS operations and authoring exploration.
  This document does not close or expand them automatically.

### Handoff for the next contributor

Read this document, recommendation, 13/14 reports and their code. Inspect current
workspace changes and dependencies. Return a proposed first coherent integration
slice and any concrete contradiction found; do not start by re-running every
historical parser experiment or implementing a live merge engine. Keep source,
compiler, materializer and transport boundaries inspectable, commit explicit
owned stages, and stop before any unapproved remote write.

<a id="keyed-bidirectional-addendum"></a>

## 12. Addendum — keyed bidirectional MDX and Yrs editing

### Correction of intent

The user clarified that authoring keys were meant to retain the associations
with Yjs/Yrs content, not merely label disposable generated output. They want
to draft locally and push, with at least some useful conversion in both
directions. They permit whatever identifiers the representation needs;
human-friendly naming is optional. Unnamed blocks must not therefore be treated
as permanently unidentifiable.

**That is feasible as a structured, editable projection of current content.**
The earlier recreation-only emphasis simplified away too much of this goal.
This addendum controls where it conflicts with the earlier handoff:

| Earlier emphasis | Revised intended direction |
| --- | --- |
| Fresh ordinary blocks on each meaningful save | Retain mapped blocks and relationships; create only new objects and deliberately replace unsupported cases |
| Yrs used only to initialize fresh text | Also load current remote text and apply identity-preserving edits |
| Keys optional labels without prose continuity | Stable source-to-remote association, automatically assigned where the author supplied none |
| Reverse editor-to-MDX conversion entirely outside the first direction | Prove a small bidirectional profile; grow coverage without first implementing every rich block |
| Stage/switch is the main edit algorithm | Stage/switch remains useful for initial creation and explicit recreation fallback |

No live multiplayer UI, stored ancestor, persistent CRDT history or automatic
three-way merge is added. MDX is still the one local current document, and the
draft/published lifecycle remains exclusive.

### What is encoded, and what is not

Leaflet has an outer fact graph for pages, blocks, order and references. Each
ordinary rich-text block has a relatively shallow **separate** Yjs document:
the named `prosemirror` fragment, paragraph, text runs/marks and inline nodes.
Yrs operates on that text state; it is not a CRDT for the entire fact graph.

The file should encode **current meaning and durable associations**:

```text
MDX block identity
  -> existing Leaflet block entity and membership fact
    -> existing block/text fact and Yjs document
      -> current XML/text items loaded through Yrs for this edit
```

Yjs internal items use numeric client/clock identities. An author key such as
`introduction` is not a replacement spelling for those identities. The key
locates the same remote object; loading and editing its actual CRDT state
retains surviving item identities and allocates new ones for new content.
Choosing the same numeric client ID for independently regenerated text does
not recreate the old history safely and must not be the implementation.

Two round-trip claims must not be confused:

1. **Current-content round trip:** editor facts/Yrs -> MDX -> parsed current
   document preserves supported text, marks, layout and reference meanings,
   with durable node associations. This is the desired storage/conversion goal.
2. **Exact CRDT-state reconstruction from MDX alone:** reproduce tombstones,
   causal history, every text-item identity and binary updates without reading
   the remote state. Ordinary Markdown plus block IDs does not supply that
   information. This is not promised and does not justify stashing a full CRDT
   snapshot or adding per-character annotations to normal prose.

An existing bound document can obtain current CRDT state by a fresh pull when
applying changes. A new unbound document initializes fresh state. If the remote
state has disappeared, the file still contains its current authored meaning,
but cannot resurrect lost causal history merely from identifiers.

### Automatic identity, optional human names

Every source node needing independent remote continuity needs a durable logical
identifier. Authors may choose useful names with the existing Key/brace syntax;
otherwise the tool assigns one. For example:

```mdx
<Key id="b-6d247168-3a91-4fd2-856e-225d028bfa68" />
An ordinary paragraph whose identity was assigned by the tool.

## Introduction {#introduction}
```

Both identify current content; one is machine-generated, the other friendly.
The concrete generated spelling remains preliminary. The tested alphabet allows
the `b-` prefix; these strings are not assumed to be remote UUIDs or Yjs IDs.
Do not turn a name edit into an accidental delete/recreate: key renaming/rekeying
must either retain the binding through an explicit operation or be diagnosed.

**Persist the association with the block in the source.** A frontmatter-only
table keyed by array position, source offset or content hash cannot reliably
recognize a paragraph that was both moved and rewritten. A compact table may
map the source ID to remote entity/fact IDs; it must not be the only record of
which source block owns that identity. Copying a keyed block produces a duplicate
identity until a new one is assigned; it is not an implicit move.

This revises the previous blanket avoidance of body writes: a focused local
preparation step may insert missing identity annotations, while preserving all
other source bytes. It can be an explicit prepare/key operation or part of a
confirmed first save; settle that interface before implementation. Do not mint
fresh identities on every parse or save. Unchanged keyed input is byte-untouched.
Keep ordinary Markdown content and optional human naming; no mandatory prose
wrappers and no visible per-character identity ledger.

The exact association rules for list items, derived list wrappers, multiple
references and split/join operations still need the combined experiment. Give
IDs to meaningful editable nodes and relationship occurrences, not blindly to
every parser AST node. Already named Page/Poll/reference objects also need a
consistent binding policy.

Suggested compact extension of `@springleaf`, not a finalized schema:

```text
nodes[sourceId]
  entityId
  membershipFactId(s)
  textFactId                         where applicable
  acceptedLocalHash, acceptedRemoteWitness
  mappingProfile, confirmedAt
```

These are identifiers and witnesses, not previous text or CRDT state. If a
meaningful inline object requires a separate locator, retain its association
too. Do not annotate every Yjs character unless a concrete requirement proves
that necessary; the fresh remote CRDT already has those identities.

### A modest bidirectional save/pull path

For the initially supported profile:

1. Parse MDX, validate identities and load the current bound Leaflet graph and
   text states. Resolve content/resource references explicitly.
2. Match by durable identity. Unchanged objects produce no writes; moved objects
   retain entity identity and update membership/order where supported. New
   objects receive new bindings and fresh initial text.
3. For a changed existing text block, load its current update into Yrs. Compile
   edits to existing text/formatting/XML nodes, preserving surviving identities;
   emit one cumulative delta per existing text fact/Replicache client as required
   by the server cache. Do not substitute a new independent full document.
4. Handle structural/metadata changes through explicit fact operations. An
   unsupported identity-preserving change can offer deliberate recreation with
   its effects stated; it must not silently select that fallback.
5. Push and read back actual meaning and preserved associations. CRDT convergence
   does not guarantee exact equality with the requested file under concurrent
   edits. Report divergence rather than repeatedly applying edits until local
   text wins. Update witnesses only after the result is understood/verified.
6. In the other direction, decode current Leaflet/Yrs content and serialize the
   supported MDX profile, carrying the same logical IDs. Ordinary content stays
   Markdown; explicit components/Run escapes preserve structure that plain
   Markdown cannot express. A guarded pull may update the one file; it does not
   create a second maintained source. A no-change operation must not normalize
   or rewrite the body needlessly.

This is the missing connection between experiment 05 and experiments 13/14.
It is **not yet proven end-to-end**. In particular, existing XML-node identity
matching under splits/joins is not solved by simply adding block keys.

### Synchronization evidence without stored bases

Retain compact accepted local fingerprints and remote witnesses per useful
scope. Status can distinguish unchanged, local-changed, remote-changed,
both-changed, untracked and missing without a saved historical document.
Hashes need not reconstruct old content. A conservative editor witness must
include meaningful state that the current serializer cannot interpret yet.

When both sides changed, compare today's endpoints and ask for an explicit
choice/manual edit; automatic historical merge is not promised. Preserve the
separate optional filename/path witness. Renaming the file must not break node
bindings, and updating text must not clear unrelated naming or publication drift.

Without a common local CRDT history, applying a locally edited paragraph to a
freshly changed remote paragraph cannot reveal which remote characters were
never seen by the author. The user does not need a multiplayer editor, but
preflight drift reporting and a clear overwrite choice are still useful.
The existing creation/release and no-unapproved-write boundaries remain intact.

### Revised next experiment and acceptance

Do not start another parser comparison. Connect the existing proofs in a new
focused experiment space, without changing their preserved evidence:

- A small real Yjs/fact fixture -> MDX with automatic and friendly IDs -> parse
  back with the same current-content semantics and reference associations.
- Edit one paragraph, change a mark and move a keyed block. Plan against the
  same remote objects; apply the emitted updates to real Yjs and a scoped fact
  model. Check unchanged block/paragraph/XML identities and retained text items
  in the exercised cases, not only the final words.
- New local prose receives identities once and creates new remote objects once.
  A second save is a no-op; deleting/reordering blocks does not append duplicates.
- Exercise generated-ID source insertion, duplicate-ID copying and a deliberate
  rekey. Show that machine IDs do not require the author to name each paragraph.
- Pull a web edit into the same MDX file, preserve its IDs, edit again, push again.
  No saved base body or binary CRDT copy is introduced.
- Include a bounded concurrent-edit case and an unsupported structural case;
  report the outcome and explicit recreation option instead of hidden replacement.

Start with ordinary rich text, breaks and a small block graph. Full inline/media
coverage remains a capability progression. A reliable narrow bidirectional loop
is more valuable now than either a full multiplayer architecture or silently
discarding all existing CRDT identities through recreation.

This addendum authorizes no implementation or live write by itself. The next
handoff should be an explicitly commissioned combined experiment/integration
slice using the corrected target above.
