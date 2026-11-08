---
type: Experiment
title: "Satteri is viable for source-authoritative Leaflet authoring"
description: "An offline native Key/Markdown/Page/Poll compiler slice produces typed authoring IR, real Leaflet fact shapes and fresh Yrs text consumed independently by Yjs, then exercises a bounded detached-generation replacement simulator."
resource: /experiments/13-satteri-authoring/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#xhigh
sources:
  - resource: "opencode://ses_e46bbf7f6ffeLNF9i7yGLd5Nx1"
    title: "Fresh commissioned Sol XHIGH: selected an adoption-oriented compiler experiment, implemented native Key/typed lowering/facts/fresh text and an independent consumer, then reviewed staging invariants and practical adoption limits"
  - resource: /design/authoring-parser-assessment1.gpt6a.md
    title: "Current modern-mdxjs recommendation, exact unproved Satteri Key distinction and proposed useful compiler slice; read before selecting scope"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "One authoritative source, fresh subtree under existing binding, Yrs initialization and narrow stable Poll identity"
  - resource: /experiments/README.md
    title: "Tracked synthetic experiment and no-production-write boundaries"
  - resource: /experiments/07-satteri-identity/REPORT.md
    title: "Existing direct parser/arena identity, native grammar and literal-code observations; not replaced or rerun"
  - resource: /experiments/10-satteri-mdxjs-identity/REPORT.md
    title: "Shared parser taxonomy and reasons to avoid unnecessary JS/HAST compilation"
  - resource: /experiments/10-satteri-mdxjs-identity/src/lib.rs
    title: "Read public parser/codecs, scoped sibling binder and literal-gate implementation"
  - resource: /experiments/10-satteri-mdxjs-identity/src/literal.rs
    title: "Independent attributed copy of the strict recursively unique JSON visitor, originating in 09"
  - resource: /experiments/12-mdxjs-turbopack-identity/REPORT.md
    title: "Fair existing modern-mdxjs evidence, Key and rich-component results; rival backend not rebuilt"
  - resource: /experiments/12-mdxjs-turbopack-identity/tests/key.rs
    title: "Read native next-block Key contract, scopes, source positions and negative observations"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Existing interoperable v1 XML/marks contract and wrong-old-entity replacement proof"
  - resource: /experiments/05-yrs-conversion/src/lib.rs
    title: "Read verified Yrs options, XML reading, UTF-16 and mark conventions; no incremental editor imported"
  - resource: /experiments/05-yrs-conversion/reference.mjs
    title: "Read independent Yjs/ProseMirror schema-subset consumer pattern reused here"
  - resource: /experiments/13-satteri-authoring/src/author.rs
    title: "Executed native Satteri Key consumption, component schemas, semantic lowering and page-reference validation"
  - resource: /experiments/13-satteri-authoring/src/facts.rs
    title: "Executed generated entities, real fact types, ordered memberships, key attribution and stable unpublished Poll IDs"
  - resource: /experiments/13-satteri-authoring/src/text.rs
    title: "Executed fresh full v1 updates and semantic read-back without retained CRDT state"
  - resource: /experiments/13-satteri-authoring/src/simulator.rs
    title: "Executed bounded detached staging, complete profile inventory verification, explicit entry update and repeat controls"
  - resource: /experiments/13-satteri-authoring/reference.mjs
    title: "Independent real Yjs/y-prosemirror consumer and ProseMirror validation of the emitted text payloads"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-ast/src/mdast/codec.rs"
    title: "Read exact typed codecs; verified selected installed registry source hash matches archived source"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/arena_build.rs"
    title: "Inspected native JSX/YAML building; selected installed source hash matches archived source"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-plugin-api/src/plugin.rs"
    title: "Read visitor and transform_root interfaces; intentionally not required or executed for this direct compiler"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/attributes.ts"
    title: "Read actual page/block/text/list/code/Poll fact names and discriminated data shapes"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts"
    title: "Read addBlock list facts and addPollOption entity/name/membership mapping"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cachedServerMutationContext.ts"
    title: "Read explicit many-fact ID updates and merging of existing text; simulator does not execute this server"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/factsToPagesRecord.ts"
    title: "Read block/card traversal and publication-time construction of Poll definitions from option facts"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts"
    title: "Verified native poll upload uses block entity ID as record key; no upload invoked"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Blocks/TextBlock/schema.ts"
    title: "Read schemaVersion 1 and the paragraph/marks/break subset actually validated by the independent consumer"
---

# Satteri authoring, not another parser bakeoff

## Verdict

**Use Satteri's direct public parser → MDAST arena route for the next production
authoring compiler slice.** This new experiment supplies useful positive adoption
evidence: a realistic, ordinary MDX document compiles all the way to Leaflet-shaped
facts and interoperable fresh text, without a fork, JavaScript generation,
React, HTML reparsing or mandatory prose wrappers. It also demonstrates the
preferred native `<Key id="introduction" />` spelling on Satteri itself.

This is an implementation recommendation, **not production adoption or a live
save/publication claim**. Satteri is valid for this explicitly MDX-shaped,
source-authoritative workflow; it is not proved to be unchanged CommonMark,
a complete Leaflet compiler, or faster than the alternative.

The [previous assessment](../../design/authoring-parser-assessment1.gpt6a.md)
preferred modern mdxjs chiefly because exact Key behavior and a typed compiler
starting point had been proved there. **That distinction was mostly a proof gap,
not an inherent Satteri mismatch.** Here the gap closes: native JSX Key, ordinary
Markdown, literal rich data, typed lowering and real storage-boundary output work
together. The modern mdxjs path remains credible; I would not rebuild it again
before choosing Satteri for the user's preferred direction.

## Why this scope

[07](../07-satteri-identity/REPORT.md) and
[10](../10-satteri-mdxjs-identity/REPORT.md) already establish parser basics and
that the Satteri mdxjs compiler uses the same parser. Another inspection-AST-only
run would add little. I chose **Page + inline Run/HardBreak + an unpublished Poll**
instead of a broad Canvas/media inventory: it exercises nesting, literal fidelity,
forward reference resolution and a genuinely consequential stable interactive
identity, with real target fact shapes and no imaginary uploads.

The [recreation proposal](../../design/mdx-recreate0.gpt6a.md) supplies a useful
second boundary: stage fresh text/entities, verify, then replace the existing
entry edge. A small in-memory simulator tests that consequence without confusing
it with actual Leaflet authorization, transport or publication. No agents were
delegated, no credentials were read, and no Leaflet endpoint was called.

## Executed pipeline

```text
canonical fixtures/garden.mdx (bytes untouched)
  → satteri_pulldown_cmark::parse(source, MDX_OPTIONS)
  → Arena<Mdast>: native JSX/Markdown/YAML nodes
  → scoped sibling Key pass; actual marker nodes detached with set_children
  → strict properties + typed component/child schemas + named-page resolution
  → transient Document / BlockKind / Inline / Mark
  → generated page/block/option entities + Leaflet Fact/Data shapes
  → fresh Yrs prosemirror paragraph, full initial update v1 per text entity
  → Rust semantic read-back AND independent Yjs / y-prosemirror / ProseMirror
  → detached staging simulator → verify profile facts/text → root/page update
```

[author.rs](src/author.rs) uses real `MdastNodeType`, `get_children`, `get_node`,
native source ranges, string references and public `CodeData`, `ListData`,
`ListItemData`, `LinkData` and JSX-attribute codecs. Key associations live in a
typed side table keyed by arena node ID during lowering, rather than smuggling
identities through HTML attributes or generated JS. Marker removal really changes
the transient arena; original bytes remain borrowed and unchanged.

The inspected plugin API offers typed visitors and `transform_root`, but a plugin
runner is not needed to implement this compiler. **Extensibility here is an
ordinary public arena pass, not parser injection or an unexecuted plugin claim.**
Adding another supported component means a typed props/children schema, a typed IR
variant and an explicit target mapping. Unknown/unimplemented input cannot silently
fall through to prose or disappear. Native and semantic failures return no usable
Document; the CLI prints diagnostics to stderr, no partial plan, and exits 2.
Semantic validation currently stops at the first error; native parser errors are
returned together.

### What the representative source proves

[garden.mdx](fixtures/garden.mdx) yields **2 pages, 17 entities, 56 facts, 9 fresh
text payloads, 6 consumed Key declarations and 7 author-key attributions**. These
counts describe this fixture, not completeness or a ranking.

| Source | Typed/target result actually checked |
| --- | --- |
| YAML title/description/tags/date intent | Typed metadata, separate from body heading; retained author intent, not an actual scheduling or root-metadata write |
| Normal heading and paragraph | Heading level and text facts; no Heading/Paragraph components required |
| Strong/emphasis/code/link, escapes/entities, é/emoji | Native decoded inline values and corresponding Yjs marks; independent expected prose assertion |
| Soft wrap / Markdown hard break | Soft wrap becomes a space; explicit break becomes `hard_break` XML, not a text newline |
| Native Run and HardBreak in Page prose | Exact spaces, newline, tab, angle/braces/ampersand and Unicode; Run's newline remains inside text, distinct from XML hard break |
| Bullet and ordered lists starting at 3 | Ordered `card/block` memberships plus real `block/is-list`, `block/list-style` and `block/list-number` facts |
| Quote with an inner Key | Container-local binding and a real blockquote/text mapping for one paragraph |
| Fenced MDX example with a leading tab | Exact semantic code payload retains the tab and literal Key/Poll text; no unnecessary compiler tab expansion |
| Forward PageLink then named Page definition | Definition is not a main-body block; resolve `block/card` to a generated `page/type=doc` entity; missing/duplicate/cyclic references fail |
| Poll with literal option array | Fresh unpublished option entities, exact `poll-option/name` strings and ordered `poll/options`; no fabricated public URI/CID |

Native Satteri trims the tested final fence payload newline, as previously known;
code semantics follow that parser value. This is not a lossless CST round-trip.

### Native Key contract

- Standalone self-closing flow JSX; exactly one **quoted literal** `id` matching
  `[A-Za-z][A-Za-z0-9_.-]*`, case-sensitive. Multiline opening tags work.
- The immediate next supported sibling in the **same root, quote, list item or
  Page scope** is named. Optional blanks after Key do not change association.
  A blank line is required before Key after an earlier sibling. No skipping
  comments, definitions, other declarations or closed containers.
- Inline, malformed, duplicate, orphan and paired-Key uses diagnose. Paired Key
  group semantics are deliberately not invented. Component `key` shares the same
  namespace; Key plus the target's own key is an error even if equal.
- Keys are author identities, not anchors, remote IDs or character continuity.
  Prose entities are fresh each generation, even if keyed; moves keep the author
  name and change source attribution. A list key names the typed collection, with
  **no fabricated remote group entity**. A one-paragraph quote and its paragraph
  share one output entity if both have different valid author names.
- Positions are absolute UTF-8 byte ranges in original source. Add the native
  BOM base once; Unicode/CRLF/BOM and whole Key slices are tested. Component-key
  diagnostics use the real element range, not an invented property-token span.

### Fresh text and recreation controls

[text.rs](src/text.rs) reuses 05's verified **conventions**, not its incremental
editor: `OffsetKind::Utf16`, schemaVersion 1, `prosemirror` XmlFragment, paragraph,
XmlText marks and actual `hard_break` elements. Yrs is pinned to the same exact
revision. Each new entity receives a full initial v1 update; there are no saved
bases, state files, LCS edits or merge-with-old-text workflow.

[reference.mjs](reference.mjs) independently applies every emitted text fact in
**Yjs 13.6.15**, converts with **y-prosemirror 1.2.5**, and validates with
**prosemirror-model 1.25.0**, matching the inspected Leaflet direct versions.
It compares the actual consumer tree to independently constructed expectations
from typed inline data, and separately asserts exact known fixture prose, Run
newlines/break topology, Poll values and Page references. This is stronger evidence
than a Rust encoder decoding only its own output. The oracle's copied schema is
the exercised subset, not the full application/editor runtime.

[simulator.rs](src/simulator.rs) and [pipeline tests](tests/pipeline.rs) prove,
**within the model**:

1. Detached staged facts are not reachable through the old entry.
2. Interrupt at **every one of 56 fact boundaries**, resume the same generation,
   verify, then switch; the old entry stays active until successful verification.
3. Missing/corrupt text, unexpected profile memberships and duplicate profile facts
   fail verification before switching. Verification covers the entire emitted
   profile inventory, references and decoded text, not merely a subset.
4. The update preserves the existing root, entry fact ID/position, entity set,
   synthetic token locator/public-linkage witness and unrelated navigation edge.
5. Exact repeat and a simulated lost switch response do not append. Reconstructed
   equal text with a different Yrs client ID is left untouched, not reasserted.
6. A source edit creates a fresh page/prose generation; the old generation is
   detached, not automatically deleted. Named Poll entity/options remain stable.
7. Changing or removing stable Poll option meanings is rejected **before writes**;
   a deliberately new key makes a new Poll. Native publisher source really uses
   the Poll entity ID as its record key, which motivates this exception.

This is not a new atomic-swap API. The emitted switch is the actual low-level
`assertFact` argument shape for the existing ordered `root/page` fact. Initial
`stagedMutations` output is a **fresh-empty-generation plan**, not a safe blindly
replayable push batch: a resumed writer must inspect existing text and skip equal
payloads. A negative test retains the real CRDT rule: merging two independently
encoded old/new paragraphs on one entity does not replace its contents.

## Exact dependency/source identity

[Cargo.toml](Cargo.toml) owns `[workspace]`; [Cargo.lock](Cargo.lock) pins transitives.
All dependency operations used Cargo tooling within this directory.
[UPSTREAM.json](UPSTREAM.json) records checksums and selected source hashes.

| Item | Executed pin / checked binding |
| --- | --- |
| Satteri parser | Registry `satteri-pulldown-cmark =0.6.7`, default std + mdx + serde |
| AST / arena | Registry `satteri-ast =0.6.3` (default mdx), `satteri-arena =0.4.3` |
| Core published source + inspected archive HEAD | `7d1297b7e4551a1c5124fa07bd2d1d05f66236e6`, verified from actual HEAD and all three registry `.cargo_vcs_info.json` files |
| Transitive property-info | 0.2.1, published SHA `3498bad3f330de74c355637426ad165da8af9ee0`; not falsely attributed to the later core revision |
| OXC / serialization | OXC 0.121.0; Serde 1.0.229, serde_json 1.0.154; no legacy Serde downgrade |
| Fresh Yrs | 0.28.1, Git `e0b476b0ce613453c70d0bb477b955808aa79a07`, default features off |
| Other direct dependencies | serde-saphyr 2.1.1, uuid 1.23.0 (v5/serde), anyhow 1.0.103, base64 0.22.1; exact pins |
| Leaflet source contract | Local archive HEAD verified as `600c7b5f1f7b31b66fb626c293cc104312a12caf`; source inspected, not server executed |
| Independent JS consumer | Exact direct versions above; [pnpm-lock.yaml](pnpm-lock.yaml) pins peer/transitive resolution, which is not claimed identical to Leaflet's whole lock |
| Executed host/toolchain | Linux x86-64; Rust 1.101.0-nightly (822182db4), Cargo 1.101.0-nightly (29c5daa1a), Node 26.6.0, pnpm 12.16.1 |

This deliberately executes the **same pinned core parser as 07/10**, not an
unannounced newer release. The verified archive HEAD is not a claim to have
fetched today's latest remote tip. Selected installed parser/codec/arena source
hashes match that archive; no registry/archive patch was applied. There is no
machine-specific dependency. No Satteri mdxjs compiler, SWC, Node/NAPI or React is
required for the Rust path; OXC still builds for native MDX expression parsing.
The optional Node program is a test oracle, not an MDX execution runtime.

## Honest limitations and material remaining costs

- This is **not all Leaflet types**. Canvas/Place/Image receive explicit
  unimplemented diagnostics; unknown names fail. Nested/multi-block list items,
  tasks, multi-paragraph quotes, tables, mentions, math, title-bearing links,
  fence metadata and other unsupported nodes fail rather than flatten/drop.
  The bounded ordering helper supports 62 siblings; production should use the
  existing general fractional-order implementation. Poll's 2–8 distinct nonempty
  options rule is this experiment's profile, not a complete native schema claim.
- YAML author metadata is parsed, not written to a remote root. `publishedAt` is
  retained as an opaque author-intent string, not validated/scheduled. CLI-owned
  compact linkage/witness parsing/patching and draft-or-published lifecycle are
  deferred; the simulator injects its synthetic binding and generation IDs. Semantic
  input hashing/no-op receipts and durable cross-process recovery are not built.
  No body rewriting occurs.
- The simulator is not the real push handler: no authorization/SQL, mutation IDs,
  acknowledgments, depth-three/batched staging reads, concurrency/CAS, metadata
  actions, upload, publication, delivery, crashes across processes or browser tabs.
  Its final Poll type fact marks a complete model definition so partial new Polls
  can resume; that is not an upstream transaction/receipt guarantee. Verification
  owns the implemented profile's fact attributes, not arbitrary uncompiled fields.
- The one unusual multiline **single-backtick** control remains: a flow Poll
  between its apparent inline-code lines becomes active MDX and compiles. The Key
  version is rejected by the blank-before rule; neither restores a CommonMark code
  AST. Indented code is not MDX's code-block form. Use fences for multiline examples.
  This known dialect limit is recorded, not elevated above the realistic successful
  fixture or mislabeled as arbitrary JS execution. Unchanged CommonMark semantics
  would be a different requirement and reopen parser choice.
- The literal gate is not a hardened arbitrary-input sandbox. No depth/size/fuzz,
  exhaustive URL validation, strict binary-framing, deployment/MSRV or performance
  matrix is claimed. Raw arena codecs are lower-level than mdxjs's owned enum tree;
  they require a small maintained adapter and coordinated AST/arena pins. This run
  proves that adapter practical, not intrinsically cheaper to maintain.
- Production workspace feature-union integration was deliberately **not run**.
  Modern dependencies work together in this real isolated compiler, but that is
  not proof of every production feature or platform. No production code, root
  manifest/lock/index/ignore or beads file was modified. Other experiments remain intact.

The smallest immediate engineering gate is **integrate this direct parser/typed
adapter in the actual production dependency/features context and connect the
verified fresh-generation plan to the existing transport boundary**. A separately
authorized synthetic live staging/read-back/entry-switch exercise then tests what
this simulator cannot. Assets and authenticated native publication remain distinct
external gates, not reasons for another parser bakeoff. No production adoption or
live writes are commissioned by this verdict.

## Reproduce and check receipts

[README.md](README.md) gives setup and CLI behavior. From this directory, after
`cargo fetch --locked` and `pnpm install --frozen-lockfile`, checks are offline:

```sh
cargo nextest run --locked --offline --no-fail-fast
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
cargo build --locked --offline
pnpm test
cargo run --locked --offline -- check fixtures/garden.mdx
cargo run --locked --offline -- demo fixtures/garden.mdx
cargo run --locked --offline -- check fixtures/rejected.mdx # expected exit 2
cargo tree --locked --offline --depth 2 -e normal
```

Recorded Rust result: **29/29 tests pass, zero skipped**, nextest run
`f7434c03-88fe-4564-b1e4-7af47853a671`: 15 authoring controls, 11 pipeline tests and
3 actual CLI tests. One control explicitly asserts the known negative MDX
interpretation; the count is not 29 universally fulfilled product invariants.
Clippy passed with warnings denied; formatting passed. The independent consumer
passed **4 groups**, including all 9 real text payloads and actual CLI execution.
Node prints an environmental experimental localStorage warning; it did not affect
the assertions or create browser persistence.

Initial Rust tests passed 27/27. The first JS oracle run exposed an expected-JSON
representation mismatch: the real converter includes `attrs: {}` on zero-attribute
marks. Normalize only that empty-object/order/partition representation for semantic
comparison; preserve link attrs and exact text/break checks. Self-review also
strengthened staging from subset to entire profile inventory and added all-boundary
interruption recovery. These corrections are separately checkpointed:

- `771d3302` — native authoring/facts/fresh text, initial 27 tests and initial JS
  oracle mismatch preserved before correction.
- `8e1d2bcd` — independent consumer correction plus complete profile-inventory and
  partial-Poll recovery controls; 29 tests and clean Rust/JS checks.

Document checks from the repository root:

```sh
cargo run --locked --offline --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf experiments/13-satteri-authoring/REPORT.md
cargo run --locked --offline --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check experiments/13-satteri-authoring/README.md
```

Both passed: report **1 document, 15 links, 27 sources**, including actual native
session provenance; README **1 document, 1 link**. `git diff --check` also passed.
The frozen pnpm install was checked offline and the final built CLI's independent
consumer rerun passed all four groups. The recorded Rust-lock SHA-256 in
`UPSTREAM.json` was checked against the actual lockfile. No untouched production
or sibling experiment test suite was run.

Connections stay explicit: the assessment supplied the proof gap, 07/10 own the
earlier Satteri observations, 12 remains the modern-mdxjs alternative, 05 supplies
the fresh-wire foundation, and the recreation proposal owns the wider intended
workflow. This experiment adds executed compiler/consumer/model evidence rather
than silently replacing those records. Root index/reciprocal changes are left to
the commissioning parent because this session owns only this new directory.
