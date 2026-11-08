---
type: Design
title: "Preliminary design and implementation handoff — local MDX authoring"
description: "A source-authoritative Satteri compiler, fresh Leaflet body recreation under stable identity, native publication, experimental reuse map and staged implementation gates."
resource: /design/authoring-handoff0.gpt61s.md
status: preliminary
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4900b8d7ffeDT9QuFwmKRAB7t"
    title: "User's authoring/storage/lifecycle direction, experiment commissions and request for this preliminary design and handoff"
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
    title: "Detailed current-block, resource, fact and inline mapping obligations; fine-grained editing not required here"
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

Recommended path: Satteri's direct parser/MDAST arena, a small typed compiler,
fresh generated Leaflet content, Yrs for fresh text encoding, verified replacement
under an existing Leaflet binding, and native publication.

This is a preliminary implementation brief, not an accepted schema or permission
to execute remote writes. The user requested a document, not implementation in
this pass. Proposed command names, storage details and behavior choices are
identified below. No production code changed or live requests were made.

For a short decision account and alternatives, read the
[recommendation](authoring-recommendation0.gpt61s.md). For exact experimental
receipts, use the [experiment index](../experiments/README.md). Start substantive
code work with experiments **13 and 14**, not the earliest parser prototypes.

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
