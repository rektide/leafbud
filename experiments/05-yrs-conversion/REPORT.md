---
type: Experiment
title: "Yrs conversion — identity-preserving edits against real Yjs"
description: "Offline Rust/Yjs interoperability, UTF-8 facets, incremental XML-text edits, a fact-merge round trip, and demonstrated concurrency limits."
resource: /experiments/05-yrs-conversion/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-07
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e48556aa8ffe9Fzbp0FmthuoDH"
    title: "Independent Astra inquiry upgraded by the user to an offline executable experiment; implementation and checks"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Prior protocol evidence at springleaf e48b62660d977e06de852d68bb1fc8a7a72b768a"
  - resource: /experiments/02-draft-internals/fixtures/yjs-samples.md
    title: "Frozen synthetic Yjs 13.6.18 inputs, same springleaf revision"
  - resource: /crates/drafts/src/yjs.rs
    title: "Existing hand codec and sample 1 hex, same springleaf revision"
  - resource: "https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/Cargo.toml"
    title: "Locally inspected Yrs 0.28.1 manifest; exact experiment dependency"
  - resource: "https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/transaction.rs"
    title: "Applied updates, state vectors and transaction-only v1 delta API, inspected locally"
  - resource: "https://github.com/y-crdt/y-crdt/blob/e0b476b0ce613453c70d0bb477b955808aa79a07/yrs/src/types/text.rs"
    title: "Text diff/format/remove/insert APIs and UTF-16 surrogate regression tests, inspected locally"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/TextBlock/schema.ts"
    title: "Inspected paragraph, inline and mark schema; subset reproduced by the JS reference"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cachedServerMutationContext.ts"
    title: "Inspected text merge and same-client last-write cache; simulated narrow behavior"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/package-lock.json"
    title: "Inspected exact Yjs, y-prosemirror and prosemirror-model versions used for final reference run"
  - resource: /experiments/05-yrs-conversion/reference.mjs
    title: "Executable assertions, fixture generator and tiny fact-graph scenario"
  - resource: /experiments/05-yrs-conversion/src/lib.rs
    title: "Experimental Rust projection and incremental editor"
---

# Yrs conversion experiment

## Verdict and standing

**Use Yrs for Leaflet's CRDT text payloads.** Against the source-pinned Leaflet
dependency versions, the prototype decodes the existing synthetic wire fixtures,
projects rich text to canonical UTF-8 facets, edits the existing CRDT, and sends
an incremental update that real Yjs applies correctly. Repeating the desired
document is a no-op. A surrounding fact fixture survives a text-only mutation
and simulated server merge without changing its entity, edge or comment facts.

This is a bounded interoperability proof, **not a complete Leaflet adapter**.
No production crate/manifest/lockfile was changed. No application endpoint was
called. Dependency downloads are setup traffic; all tests are offline.

The commission began as design-only and was explicitly upgraded to an experiment.
The useful scope is the difficult CRDT boundary plus one external fact edit, not
a CLI rewrite or a claim that all media/topology mappings are implemented.

## Revisions and execution receipts

Read local repositories before using network dependency resolution:

| Source | Exact inspected revision/version |
| --- | --- |
| `~/archive/y-crdt/y-crdt` | `e0b476b0ce613453c70d0bb477b955808aa79a07`, `yrs` **0.28.1** |
| `~/archive/hyperlink-academy/leaflet` | `600c7b5f1f7b31b66fb626c293cc104312a12caf` |
| Springleaf before this experiment | `e48b62660d977e06de852d68bb1fc8a7a72b768a` |
| Final JS reference | Yjs **13.6.15**, y-prosemirror **1.2.5**, prosemirror-model **1.25.0**, matching Leaflet's lockfile |
| Runtime used | Node **26.6.0**, pnpm **12.16.1**, cargo **1.101.0-nightly (29c5daa1a 2026-10-07)** |

The root Y-CRDT README's feature table still labels Yrs 0.21; its manifest is
the version authority here. `yrs` has empty default features; `weak`, `sync`,
and `small-client` are not enabled. The Git dependency fixes the exact inspected
source, including surrogate-split behavior, without editing the archive.
`Cargo.lock` and `pnpm-lock.yaml` pin transitive resolution.

Commands from this experiment directory:

```sh
pnpm install --frozen-lockfile
pnpm fixtures
cargo nextest run --locked
pnpm test
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Recorded results: **2/2 Rust tests passed** (nextest run
`316ccfe6-4372-4446-9198-a1023aec7a4d`); **13/13 cross-language groups passed**;
Clippy passed with warnings denied. The rich-text edit emitted a **226-byte
delta**, versus **469-byte full state** in this run. Bytes are diagnostic, not
a contractual compression target or byte-identical serialization requirement.
Formatting and Markdown checks are recorded in the session/commit receipts.

For clean checkout setup follow [README.md](README.md). Fixture generation is
deliberate; ordinary tests regenerate in memory and compare with the committed
files without rewriting them. The prior experiment's two wire examples and
push text payloads are also exercised directly. Its “40 bytes” description for
sample 1 is stale: the committed bytes are **63**. Sample 2's actual hex, not its
prose size claim, is the input.

Local-source execution is possible with Cargo's command-line patch override:

```sh
cargo test --offline --config 'patch."https://github.com/y-crdt/y-crdt".yrs.path="/home/rektide/archive/y-crdt/y-crdt/yrs"'
```

This optional command was **not** used for the recorded checks. It changes
dependency resolution/lock source, so use a disposable copy of the experiment
and verify the local revision first. The committed Git pin is the portable
reproduction path; there is no machine-specific dependency in the manifest.

## What each group establishes

| Group | Executed assertion |
| --- | --- |
| Frozen wire and roots | Both annotated examples and the old push fixture decode in Yrs; agree with Yjs. Rich fixture has named `meta` and `prosemirror`, one paragraph, XmlText/break/XmlText children. First text is 11 UTF-16 units, not its UTF-8 byte length. |
| Incremental edit | JSON changes characters, removes bold, adds italic, changes an existing link URL, appends text. Yjs receives the delta and agrees with Yrs. Existing paragraph, both text nodes and hard-break objects survive; a relative position remains valid. A retained comment mark survives. ProseMirror conversion validates against the copied schema subset. |
| No-op and repeat | Original content, including tombstones, produces no mutation. Duplicate delivery of the same delta does not duplicate text. Freshly load the merged peer and request the same JSON: again no mutation. |
| Overlapping facets | Separate bold/italic ranges become normalized runs; their union and removal boundaries survive Yjs read-back. |
| Ordinary marks | All seven keys (`strong`, `em`, `code`, `strikethrough`, `underline`, `highlight`, `link`) generated by y-prosemirror project correctly and can all be removed. Highlight coverage is slot 1 only. |
| Delete-only | `abc → ac` produces an effective update while the Yjs state vector is unchanged. An unchanged vector is not a no-op test. |
| Hard breaks/Unicode | Add, remove and repeat actual `hard_break` elements; start/end breaks, empty paragraph, accented letters, emoji and combining characters work in both engines and ProseMirror's model. |
| Competing insert | From `abc`, remote inserts `R` before `b`; local deletes `b`. Both arrival orders yield `aRc`, and Yrs decodes that result. This is convergence, not equality with desired `ac`. |
| Wrong replacement | Merging independent `old` and `new` documents yields **two paragraphs**. New-client replacement content does not overwrite an existing root. The prototype refuses this unsupported shape. |
| Surrogates | A JS insertion inside an emoji pair yields `a�Z�b`; Yrs reads the same wire result. An unpaired JS surrogate is replaced on wire encoding. Literal unpaired-surrogate JSON is rejected by Rust's parser. |
| Invalid/unsupported | Corrupt `b5ec6d`, off-boundary facets, schema version 2, mention nodes, unknown marks, a v2 fixture sent to the v1 endpoint, and a standalone delta missing its base all fail. |
| Fact round trip | Full synthetic pull patch → one record-shaped JSON text block → one `assertFact` → actual `Y.mergeUpdates(base, delta)` → Yrs JSON. Original fact ID/entity/edge/comment rows remain; `@springleaf` never enters the mutation. Repeating produces no delta. |
| Last-write cache | Two separate same-client deltas are not safe if only the last is retained: the last depends on the missing first. Merging them into one cumulative delta works. This reproduces the relevant server cache assumption without a database. |

## Experimental algorithm

Rust loads base64 with `Update::decode_v1`, applies it in a dedicated transaction
to `Doc::with_options` with **`OffsetKind::Utf16`**, and rejects unresolved
`pending_update`/`pending_ds`. The editor client ID is separate from incoming
IDs (fixed injected IDs in tests, JS-safe 53-bit bound). `meta.schemaVersion`
above 1 is an error. Existing meta/other root state is retained by the document.

The root type is interpreted from Leaflet's contract as an XmlFragment, not
inferred from a binary type declaration: named root type constructors are not
fully conveyed by the update. The editor uses `getXmlFragment`; some Leaflet
readers use `getXmlElement` over the same root sequence.

Projection walks actual XML children. `XmlTextRef::diff(..., YChange::identity)`
returns runs; it is **not** the edit diff algorithm. The prototype makes Unicode
scalar tokens carrying owned marks, flattens a hard break to `\n`, and constructs
facets using Rust string byte lengths. It accepts historical `true` and
ProseMirror `{}` values for zero-attribute marks. Comment marks are deliberately
outside the document-owned facet projection, but remain in retained CRDT text.

Editing uses deterministic, bounded LCS twice: XML child **kind** correspondence,
then Unicode-scalar correspondence inside matched XmlText nodes. Retained
characters keep their CRDT identity. Deleted characters call `remove_range`;
inserts call `insert_with_attributes`; format differences call `format`, with
`Any::Null` removing a mark. Unchanged marks are not rewritten. Positions advance
by `char.len_utf16()`, never by UTF-8 bytes or grapheme count. Hard breaks are XML
children, not inserted newline characters. Only the edit transaction is encoded
with `TransactionMut::encode_update_v1`, so deletes are included and pulled
history is not retransmitted as fresh content.

Before editing, normalized desired text/facets are compared to current meaning.
Equality returns `None`; no synthetic schema stamp, new paragraph, or meaningless
assertion is emitted. Full state in the response exists to exchange test peers
in memory, not as a proposed saved snapshot. Test fixtures are fixed synthetic
evidence, not application persistence.

## Limits, failures and important negative evidence

- **Identity across node splits is not maximized.** Child-kind LCS retains matched
  nodes, but turning one text node into two can recreate characters in the new
  sibling. Correspondence is especially ambiguous with repeated text/breaks.
  Production needs content/anchor-aware matching and a declared split/merge policy;
  this prototype does not prove all comment ranges survive those transformations.
- The scalar LCS is quadratic and rejects matrices over four million cells.
  This is a bounded proof algorithm, not a production large-document diff choice.
  Grapheme sequences are not normalized: editing a combining scalar is allowed;
  decomposed and composed spellings remain distinct.
- Input validation is for the supported profile, not a hardened arbitrary binary
  parser. A v2 sample actually passed the initial v1 decoder and failed later as
  missing causal state. **Do not autodetect version by “decode succeeded.”** Leaflet's
  source contract selects v1. Strict framing/trailing-byte and resource-limit
  checks remain production work.
- First test runs caught an incorrect expected UTF-16 length (12 instead of 11)
  and overly narrow expected error messages for malformed surrogate JSON and v2.
  Assertions were corrected to actual behavior. No CRDT operation failure was
  hidden or turned into empty text. Clippy requested one collapsed conditional;
  it was fixed before the clean run.
- Initial runs used Yjs 13.6.18/y-prosemirror 1.2.15. After inspecting Leaflet's
  lockfile, dependencies were pinned to its exact older versions and the complete
  suite rerun successfully. The older frozen samples still originate in 13.6.18.
- Node 26 printed an experimental `localStorage` availability warning from the
  JS dependency environment. It did not affect assertions or create browser state.
- The full editor schema contains mentions and footnotes; this executable
  explicitly refuses them, colored highlights, unexpected XML and multiple
  paragraphs. Literal newline content already inside XmlText is projected as
  newline and retained on semantic no-op; no-op does not normalize its topology.
- The graph harness is **not** the actual push handler: no authorization, SQL,
  metadata actions, failed-mutation acknowledgments, movement/deletion, media,
  Replicache scheduling or live browser effects were tested. Its locator uses a
  JSON pointer suitable for this one unmoved block; arbitrary reordering/editing
  cannot infer stable identity from such a pointer alone.
- The three layers are distinct: Yrs integrates text items; springleaf must map
  the external fact graph; Leaflet must accept the mutations. CRDT convergence
  supplies neither fact-graph CAS nor user-intent merge from an old JSON file.
  Replanning `ac` after observing concurrent `aRc` would delete `R`; do not loop
  until the file “wins.” Read-back must surface the concurrent result.

## Production gates left open

Full inline/mark semantics including references and colored highlights;
identity-aware splits/merges; all current block/page/list/canvas/theme mappings;
fresh-state resource resolution and uploads; complete local content representation
for editor-only types and unpublished polls; stable mapping under manual JSON
reordering; malformed-input hardening; and actual deployment verification.

The experiment supports replacing the hand-written CRDT machinery once those
adapter-specific obligations are implemented. It does not justify permanently
warning-and-skipping any currently supported Leaflet block type.
