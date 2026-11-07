---
type: Assessment
title: "Native MDX and Key: parser selection after the completed experiments"
description: "Revised comparative recommendation: pinned modern mdxjs fork, native next-block Key markers, and a small Leaflet compiler slice; corrected Satteri taxonomy and proportionate treatment of MDX limitations."
resource: /design/authoring-parser-assessment1.gpt6a.md
status: proposal
generated:
  at: 2026-11-07
  by: model:openai/gpt-6-astra#medium
sources:
  - resource: "opencode://ses_e48053fe8ffe3NnxwMn7hPx6TK"
    title: "Independent assessment of completed 06–09, continued with completed 10/12 source and test inspection, updated Key preference and recalibrated practical priorities"
  - resource: /design/authoring-parser-assessment0.gpt6a.md
    title: "Preserved predecessor at b11b21e6; detailed 06–09 implementation/test review and pinned source evidence"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Canonical local source, transient compiler tree, recreate under existing Leaflet binding, native publish"
  - resource: /experiments/10-satteri-mdxjs-identity/REPORT.md
    title: "Completed actual Satteri compiler/shared-parser experiment; implementation checkpoints 6649573a and 4556d72d"
  - resource: /experiments/10-satteri-mdxjs-identity/src/lib.rs
    title: "Read full direct arena lowering, guarded binding, literal gate and separate compiler controls"
  - resource: /experiments/10-satteri-mdxjs-identity/tests/integration.rs
    title: "Read shared parser/compiler comparison, lost metadata, fenced-tab normalization and coordinate controls"
  - resource: /experiments/10-satteri-mdxjs-identity/tests/authoring.rs
    title: "Read nested components, exact data, scoped identities, normal code and negative observations"
  - resource: /experiments/12-mdxjs-turbopack-identity/REPORT.md
    title: "Completed fork experiment through 85c53110; build, grammar, native Key and reported read-only stock comparison"
  - resource: /experiments/12-mdxjs-turbopack-identity/UPSTREAM.json
    title: "Read exact Git revision/tree, source and archive checksums, markdown identity and no-patch record"
  - resource: /experiments/12-mdxjs-turbopack-identity/Cargo.toml
    title: "Read exact fork Git rev and modern Serde pins in isolated workspace"
  - resource: /experiments/12-mdxjs-turbopack-identity/src/lib.rs
    title: "Read full actual parse-only call, Key recognition, consumed-marker paths, binding and literal component adapter"
  - resource: /experiments/12-mdxjs-turbopack-identity/tests/key.rs
    title: "Read native Key sibling/paired tests, container association, duplicates, source spans and normal-code controls"
  - resource: /experiments/12-mdxjs-turbopack-identity/tests/components.rs
    title: "Read nested Page/Canvas, exact Run literals, data rejection and source-mapping controls"
  - resource: /experiments/12-mdxjs-turbopack-identity/tests/cli.rs
    title: "Read actual executable clean/rejection controls and false-clean multiline-component observation"
  - resource: /experiments/12-mdxjs-turbopack-identity/fixtures/multiline-component.mdx
    title: "Inspected exact three-line single-backtick counterexample before assessing severity"
---

# Native MDX and Key: revised parser assessment

## Recommendation

**Use the pinned Vercel mdxjs fork's public parse-only API, with native
`<Key id="the-key" />` declarations naming the next block. Proceed to a small
source-to-Leaflet compiler slice rather than another parser bakeoff.** This is
a recommendation for user acceptance, not a production implementation decision
already made.

The exact candidate is Cargo package **`mdxjs` 1.0.4 from Git**, repository
`vercel-labs/mdxjs-rs-turbopack`, revision
`700e192fbdbad5874208addfbf421dcf1eca7a20`, with `markdown` 1.0.0. Call
`mdxjs::mdast_util_from_mdx`; consume the returned mdast directly. Turbopack,
Next.js, Node, React, JavaScript compilation and JavaScript execution are not
needed for this authoring path.

Why this choice:

- It demonstrates the desired combination: ordinary Markdown prose, nested
  Page/Canvas/Place/Image structure, literal Poll data, exact Run strings and
  optional native Key declarations. No prose wrappers are required.
- Experiment 12 builds unchanged with Serde **1.0.229** and serde_json
  **1.0.154**, removing the concrete old-SWC build failure that motivated my
  earlier recommendation to investigate a replacement callback first.
- The typed mdast and tested Key adapter give a direct compiler starting point.
  Keeping SWC as a dependency is presently a reasonable engineering tradeoff;
  removing it need not precede useful authoring work.

**Strongest alternative: direct Satteri pulldown → MDAST arena.** Experiments
07/10 now demonstrate comparable rich structure, modern dependencies and a
guarded comment binder. It is a credible choice, not rejected for an unusual
code-span case. I favor 12 because it has the exact newly preferred Key contract
executed and an immediately usable typed tree. Satteri would require adapting
that small declaration policy; its native JSX support makes this plausible,
but its Key behavior has not been independently proved by these experiments.

Change the choice if actual workspace integration or deployment requirements
make the Git/SWC dependency unacceptable, or if Satteri's arena fits the compiler
substantially better in implementation. No measured performance or binary-size
comparison currently distinguishes them. Reopen Rushdown only if custom Markdown
grammar or unchanged CommonMark behavior becomes a product requirement.

## What changed since assessment 0

The [predecessor](authoring-parser-assessment0.gpt6a.md) remains the detailed
06–09 review, with source/test references. This revision changes its conclusions
in three important ways:

1. **Native Key is now the preferred syntax candidate.** Comments were never
   adopted. The user's preference for good MDX with Key supersedes the earlier
   comment-first recommendation. `{#id}` was never impossible: 06/08 proved it;
   it simply needs additional policy/grammar on the native MDX routes.
2. **The build blocker has a demonstrated remedy.** Stock registry mdxjs 1.0.4
   in 09 needed old Serde pins. Git mdxjs 1.0.4 in 12 does not. They share a
   package/version label but are different source identities. A normal registry
   `mdxjs = "1.0.4"` dependency would not select the tested fix.
3. **My earlier code-ownership framing gave a corner case too much decision
   weight.** The findings are real and worth retaining, but do not establish
   general code-example breakage or require a custom parser before adoption.
   Ordinary prose, fenced code and literal rich structure are the main workload.

The previous direct-markdown-rs callback proposal remains a possible dependency
optimization. It is no longer the recommended next gate. It would add a new
expression-boundary implementation to prove, without a present necessity.

## Corrected taxonomy and comparison

There are six completed experiments, but fewer independent parser architectures.
**Satteri's `satteri-mdxjs` 0.3.17** is published from the source directory
`satteri-mdxjs-rs`. Its `compile` calls **the same
`satteri-pulldown-cmark` 0.6.7 parser** exercised in 07, using AST 0.6.3 and arena
0.4.3. It is an OXC-based compiler over that parser, not a second Markdown
backend. Experiment 10 executes both compiler APIs and the direct arena route;
experiment 11 was therefore unnecessary.

| Route and evidence | Proven fit | Remaining practical cost / disposition |
| --- | --- | --- |
| Rushdown 0.18.0 — 06 | True block/inline extension hooks; `{#id}` heading suffix/paragraph tail and comment markers; tested CommonMark code ownership | JSON-prop components can become prose. Nested literal component grammar still needs building. Strong custom-dialect alternative, not shortest route to this goal |
| pulldown-cmark 0.13.6 — 08 | Offset-event transformation implements `{#id}`; semantic text and explicit tight-list paragraph reconstruction | No native JSX/data model. Strict rejection of `<` is not component parsing. Less compelling than Rushdown if new grammar must be owned |
| Satteri direct arena — 07 + 10 | Native JSX, nested Page/Canvas, precise literal strings; modern Serde/OXC; 10 adds guard and recursive duplicate-JSON rejection | Key binder adaptation, typed compiler/schema work; arena bytes/BOM conventions need care. Strongest alternative |
| Satteri compiler — 10 | Real `compile` / `compile_hast_arena` integration over that same parser | Adds no needed Leaflet storage capability. Keep lowering on MDAST; do not route internal identities through HAST or JS |
| Stock mdxjs + markdown — 09 | Native components, typed tree, comment identities, strict literals | Old SWC/Serde integration failure makes this exact dependency graph unattractive now that 12 exists |
| Modern Git mdxjs + markdown — 12 | Same useful native architecture, modern Serde build, executed self-closing Key binder, nested structure and exact data | Mandatory SWC still builds; Git revision/lock must be explicit. Full production feature-union and deployment integration remain to check. Recommended |

Satteri source is pinned at
`7d1297b7e4551a1c5124fa07bd2d1d05f66236e6`; 10 resolves OXC 0.121.0.
The modern fork resolves SWC core 84.0.2/common 27.0.0/parser 47.0.0, rather
than 09's old stack. Its declared Rust minimum is 1.88; the experiments ran on
1.101.0-nightly on Linux, not a verified minimum/cross-target matrix. Package
counts and public project names are not performance or maintenance guarantees.

Native parsing supplies structure, not our data language: **all** routes still
need component schemas and Leaflet lowering. Strict JSON acceptance/rejection,
unique author keys, allowed components and marker consumption are adapter policy,
not upstream features inferred from successful parsing.

## Key contract: next-block declaration first

Recommended authoring surface:

```mdx
<Key id="introduction" />
## Introduction

An ordinary paragraph needing no stable identity.

<Key id="central-argument" />
This is ordinary **formatted** prose with optional continuity.

<Poll key="topic" options={["Gardens", "Maps"]} />
```

Experiment 12 proves Key and the following heading/paragraph are separate native
flow siblings, including optional blank lines after the marker and multiline
opening tags. **Key is not fork-specific syntax**: it uses existing JSX grammar.
Its meaning is supplied by our binder. In 12 the raw inspection AST remains
unchanged and `consumed_annotations` records marker paths; the future compiler
must actually omit those markers when constructing its authoring document.

Keep the demonstrated small association contract:

- A standalone, self-closing Key with exactly one quoted literal `id`; the
  demonstrated alphabet is `[A-Za-z][A-Za-z0-9_.-]*`, case-sensitive.
- Name the immediate next supported sibling in the same root, quote, list-item
  or component container. Do not skip another declaration, comment or definition.
  A marker before a list names the list; a marker inside an item names the next
  paragraph. Container closure cannot carry a pending name outside.
- Retain the adapter's blank-before rule after an earlier sibling; no blank
  line is required between Key and its target. This is an explicit authoring
  convention, not native JSX's own requirement.
- Duplicate names and marker/component-key collisions diagnose; neither wins.
  Orphan, malformed and inline Key uses diagnose. Only validated documents reach
  lowering. Optional keys share a document-wide namespace and survive moves by
  name, not by AST path. They are not public anchors or Yjs identifiers.

The `/>` spelling matters. **`<Key id="intro">` alone is an unclosed JSX tag**,
not a standalone marker. A paired form is different:

```mdx
<Key id="section">

## A heading

Following prose.

</Key>
```

Native parsing supports those ordered Markdown children; the present adapter
deliberately rejects the paired form. A wrapper could usefully name a group, or
make a single block's extent visually explicit. It must first specify which
object has identity: one child, a new group, or something else. Assigning the
same identity to all descendants would be ambiguous. Both forms could coexist
if a real group concept warrants it; **start with next-block Key**, which meets
the current wrapper-free need without creating that concept.

Comments and custom `{#id}` remain alternatives, not mandatory companion syntax.
The experiments' comparison modes need not all become production features.

## Findings weighted by actual usage

| Finding | Practical weight | Treatment |
| --- | --- | --- |
| 09 cannot build unchanged with current Serde | Real integration blocker for that exact graph | 12's unchanged modern-version build resolves it; still check the actual workspace feature union when integrating |
| 06/08 lack semantic rich-component parsing | Central missing capability for this workflow | Prefer ready native MDX; do not treat HTML/prose fallback or strict rejection as successful composition |
| 10/12 preserve ordinary Markdown, nested literal data and exact strings | Primary positive evidence | Build the compiler on it; no need to execute a JSX runtime |
| Unknown components, invalid fields, duplicate keys and malformed source | Ordinary compiler correctness | Reject invalid authoring documents with useful diagnostics; full schemas remain to implement |
| Four-space indentation is not a code block in MDX | Documented dialect restriction, relevant when copying Markdown | Use fenced blocks for multiline examples; clearly name the source format MDX-shaped |
| Flow JSX/comment line inside a multiline single-backtick span becomes active syntax | Narrow, demonstrated interpretation surprise | Document with its exact example; normal one-line inline code/fences work. Not a prerequisite for a replacement lexer |
| 10 compiler expands leading tabs even inside a fence; drops tested heading key metadata in MDAST→HAST | Real behavior on an unnecessary compiler route | Lower directly from arena/mdast. Do not attribute this fenced-tab change to Satteri's direct parser |
| Missing complete JSX attribute spans | Diagnostic polish, not a body-read blocker | Use real element/Key ranges and available expression stops; no fabricated ranges |
| SWC dependency size, runtime speed, deployment targets | Dependency obligation / unmeasured properties | Pin source and lock, verify real target builds; optimize only against evidence |

### The code-span example, in proportion

The exact new [component fixture](../experiments/12-mdxjs-turbopack-identity/fixtures/multiline-component.mdx)
is only this:

````text
Code `first
<Poll key="accidental" options={["Gardens", "Maps"]} />
last`.
````

It opens a **single-backtick inline span**, carries it across lines, and puts a
standalone JSX element at the beginning of an intervening line. Native MDX splits
the apparent span into paragraphs and a component. Experiment 12's CLI accepts
that Poll and key without diagnostics; its report also records reproduction in
09. The Key/comment boundary guard does not cover ordinary components' own keys.
This is a potential unintended output component if the author meant a literal
example. **It is not JavaScript execution**, and the fixture does not show a
fenced example being interpreted as Poll.

The analogous comment/Key cases are guarded rejections in 09/10/12, whereas 07
silently bound the comment. Those distinctions remain useful regression evidence.
But the ordinary alternatives—one-line inline code, fenced multiline examples,
or exact Run string data—already work in the tested corpus. Frequency in real
documents was not measured; structurally this is an unusual authoring pattern,
not evidence that routine code documentation is broken.

I therefore **do not carry forward experiment 12's recommendation to require a
uniform flow-JSX rejection repair before adopting the parser**. For the chosen
MDX authoring language, document the behavior and use fences in examples. A
later targeted lint for unexpectedly interrupting flow JSX could help if users
encounter it, but should be assessed for false positives on legitimate MDX and
need not become a custom lexer. If unchanged CommonMark interpretation becomes
an explicit requirement, that would genuinely reopen parser selection.

## Next step: one useful compiler slice

The [recreation design](mdx-recreate0.gpt6a.md) remains the destination: one local
authoritative file, a transient typed document, fresh generated Leaflet content
under the existing binding, then native publication. No saved base, shadow JSON,
historical snapshots or live collaborative-editing machinery is required.

**Recommend commissioning an offline source-to-Leaflet compiler slice on the
selected fork**, rather than another ID/parser experiment:

1. Integrate the exact Git source and test the real workspace dependency/features.
   Parse frontmatter and MDX; preserve original body bytes. Consume next-block Key
   declarations into optional internal identity. Keep parsing errors and semantic
   diagnostics separate, returning a usable typed document only on success.
2. Lower one small document with headings, paragraphs/marks, a hard break, list,
   fenced code, keyed Poll and a named Page containing Canvas/Place/Image. Include
   an exact Run string. Validate the actual supported fields/child models and
   resolve the page reference. Use a synthetic/already-resolved image for this
   offline slice rather than taking on upload transport simultaneously.
3. Produce transient planned Leaflet facts and fresh Yrs text updates using the
   existing wire evidence. Check expected text, geometry, names and key mapping;
   reordering selected keyed prose must preserve the key. Ordinary unkeyed blocks
   remain valid. Unsupported nodes/components diagnose rather than disappearing.

Reuse the meaningful existing controls for code literals, malformed Key,
duplicates and container-local association. Add tests for the new **compiler
mapping**, not a broad re-run of parser comparisons. No automatic body rewriting
is needed: owned frontmatter patching and byte-preserved body are a separate,
small source-preservation concern.

This is a bounded implementation recommendation, not permission to implement
in this assessment. Complete Leaflet block/media coverage remains the long-term
obligation. Source parsing and key binding do not yet prove remote continuity,
entry-point replacement, asset upload or authenticated native publication. The
recreation proposal's staging/read-back/switch proof and commissioned transport
work remain the subsequent pipeline gates.

## Evidence and review receipt

Earlier reports remain the comparison record:
[06 Rushdown](../experiments/06-rushdown-identity/REPORT.md) explains grammar hooks;
[08 pulldown](../experiments/08-pulldown-cmark-identity/REPORT.md) explains event
transformation; [07 Satteri](../experiments/07-satteri-identity/REPORT.md) and
[09 mdxjs](../experiments/09-mdxjs-identity/REPORT.md) establish the native baselines.
Their code/tests were inspected in this same assessment session for revision 0.

For this revision I read the complete [10 report](../experiments/10-satteri-mdxjs-identity/REPORT.md),
`src/lib.rs`, integration and authoring tests; and the complete
[12 report](../experiments/12-mdxjs-turbopack-identity/REPORT.md), `src/lib.rs`,
Key/component/CLI tests, manifest, `UPSTREAM.json` and exact multiline-component
fixture. Critical anchors are 10's `lower`/`staged` and metadata/tab tests;
12's `Adapter::visit`, `key_component`, `consumed_annotations`, native sibling
tests and false-clean CLI control. The latter prevents overclaiming that the
Key guard covers all components. Neither adapter is already a Leaflet compiler.

10 reports **29** passing tests; 12 reports **51**, including negative
observations. These are receipts, not rankings or counts of fulfilled product
requirements. Their nextest/clippy/fmt results and the new fork build are trusted
reported executions; no parser tests, new reproductions or dependency operations
were necessary in this review. Upstream lineage/build-history claims here are
attributed to those reports and recorded identities, not newly replicated.

The continued review began on clean working copy `ccde16d2`, parent `85c53110`.
Only this new assessment is owned; revision 0 and experiment evidence remain
preserved. Validate this document with:

```sh
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf design/authoring-parser-assessment1.gpt6a.md
```
