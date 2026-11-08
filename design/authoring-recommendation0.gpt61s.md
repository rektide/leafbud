---
type: Recommendation
title: "Recommend Satteri for local MDX authoring, with explicit alternatives"
description: "Post-experiment recommendation for direct Satteri MDAST, optional Key/brace identities, fresh Leaflet body generation and native publication; alternatives and actual adoption gates."
resource: /design/authoring-recommendation0.gpt61s.md
status: proposal
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4900b8d7ffeDT9QuFwmKRAB7t"
    title: "User-directed authoring redesign, experiment commissions, reviewed reports, and requested recommendation with alternatives"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Real compiler slice, independent Yjs consumption, and bounded replacement simulator"
  - resource: /experiments/14-satteri-authoring-attributes/REPORT.md
    title: "Optional brace-key compiler equivalence and explicit adapter cost"
  - resource: /experiments/12-mdxjs-turbopack-identity/REPORT.md
    title: "Strong alternative with modern dependency graph, native Key and literal components"
  - resource: /experiments/10-satteri-mdxjs-identity/REPORT.md
    title: "Satteri compiler taxonomy and avoidable JS/HAST compilation route"
  - resource: /experiments/09-mdxjs-identity/REPORT.md
    title: "Stock mdxjs dependency friction and direct markdown-rs seam"
  - resource: /experiments/08-pulldown-cmark-identity/REPORT.md
    title: "Offset-event identity alternative without native components"
  - resource: /experiments/06-rushdown-identity/REPORT.md
    title: "Custom parser extension alternative and unimplemented MDX grammar boundary"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Independent wire evidence and why fresh text must be placed on fresh entities"
  - resource: /design/authoring-parser-assessment1.gpt6a.md
    title: "Earlier mdxjs preference, its explicit proof-gap rationale, and proportionate limitation weighting"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Source-authoritative workflow and stable-Leaflet/fresh-body architecture"
  - resource: /design/yrs-conversion0.gpt6a.md
    title: "Full working-content requirements and the distinction between Yjs text and the external fact graph"
---

# Recommend Satteri for local MDX authoring

## Recommendation and standing

**Build the authoring path on Satteri's direct parser → MDAST arena route.
Keep the local MDX file authoritative; compile a fresh Leaflet body under a
stable Leaflet binding, and publish through Leaflet's native lifecycle.**

Use ordinary Markdown for ordinary content. Support native `<Key id="…" />`
for optional author identity and include the tested `{#key}` heading/paragraph
convenience through a contained adapter. Rich components are declarative typed
data, not React components or arbitrary executable JavaScript.

This is a recommendation, not accepted production architecture or an instruction
to implement it. Production remains the current CLI documented in the
[root README](../README.md). No remote writes or new test executions occurred
while writing this synthesis; reported checks are attributed to their experiments.

### Why this is the current lean

1. [Experiment 13](../experiments/13-satteri-authoring/REPORT.md) now demonstrates
   the actual desired path, not merely an inspectable parser AST: normal prose,
   keys, named pages and a Poll become typed content, Leaflet-shaped facts, and
   fresh text independently consumed by Yjs/ProseMirror.
2. [Experiment 14](../experiments/14-satteri-authoring-attributes/REPORT.md)
   composes the optional brace syntax with that unchanged compiler and obtains
   equivalent document/fact semantics. We need not choose between native MDX and
   convenient prose identifiers.
3. Direct public arena passes support binding, validation and lowering without
   a JS compiler, HTML round-trip, framework runtime or parser fork. Modern
   serialization dependencies build in the tested isolated workspace.
4. The user prefers exploring Satteri. The new evidence supports that preference;
   the earlier assessment's decisive native-Key/compiler proof gap is closed.

This is not a claim that Satteri is faster, smaller or inherently more reliable
than every alternative. There is no comparable performance/build-cost benchmark
or full head-to-head Leaflet compiler. Satteri's MDX path still uses OXC; avoiding
SWC alone is not evidence of a cheaper dependency graph.

## The architecture being recommended

```text
one authoritative .mdx file
  → direct Satteri parse / optional brace adapter
  → consume author keys; validate components and references
  → transient typed authoring document
  → resolve required resources; generate facts and fresh Yrs text
  → stage and verify a new authored subtree
  → update the existing Leaflet entry reference; verify
  → optional native publication / update of the same published identity
```

**Storage:** MDX source plus compact owned frontmatter metadata in the same file.
No saved base body, shadow JSON, serialized CRDT state or maintained merge copy.
Typed JSON inspection output is fine; it is not another source of truth.

**Lifecycle:** a file is draft or published, never both. Editing a published
file leaves it published with unreleased edits. Directories are default
placement, not identity. A convenient filename does not choose the remote rkey.

**Identity:** preserve the Leaflet token/root/entry/publication binding. Ordinary
generated blocks can receive new entity IDs on replacement. Author keys label
content, but currently do not promise preservation of every keyed prose entity,
comment anchor or Yjs item. Use stable semantic keys where identity actually
matters, such as a Poll; changing vote meanings needs a deliberate new identity.

**Publication:** save means update the web editor; publish means release the
selected file version, saving first when needed. Both meanings remain the same
before and after first publication. This save/release distinction is recommended
here, not represented as an already accepted CLI change.

**Recreation policy:** local source owns the selected generated body. An explicit
replace operation may overwrite web edits there; other navigation roots and
unrelated state remain outside its scope. This is a proposed simplification from
earlier drift-aware synchronization, not assumed human acceptance. A preflight
report can expose web differences without requiring a merge engine.

## Authoring surface

````mdx
---
format: springleaf-mdx/1
title: An essay
tags: [notes]
---

## Introduction {#introduction}

This is ordinary Markdown with **formatting**.
{#central-argument}

<Key id="example" />

```rust
let answer = 42;
```

<Poll key="next-topic" options={["Gardens", "Maps"]} />
````

This expresses the recommendation, not final component-schema approval.

### Identity rules and cost

- A self-closing Key names the immediate next supported block in its own
  container. It is consumed during compilation, not emitted as a Leaflet block.
- A heading may end in `{#name}`. A paragraph may have `{#name}` on its immediate
  following line, without a blank before it; follow with a blank, container
  closure or EOF. Both compose with lists/quotes under the tested scoped rules.
- Keys share one namespace with component keys. Duplicate or conflicting
  annotations diagnose rather than selecting a winner. They are internal author
  identities, not automatic public anchors or remote UUIDs.
- Key is the general form, including blocks without paragraph/heading suffix
  syntax. Paired Key groups are not needed for this first contract.

**Be honest about brace support:** heading attributes are natively available;
paragraph-tail support in 14 is an AST-guided discovery/mask/strict-reparse
adapter. It costs two parses and depends on the pinned recovery-tree shape.
Keep that module and its upgrade corpus explicit. If its maintenance becomes
unreasonable, native Key remains useful without changing the document model.
We are not adopting every syntax from the exploratory experiments.

### Rich content and current-document fidelity

Use typed literal components for named pages, canvas layout, images, drawings,
interactive objects and exact text that Markdown cannot express conveniently.
Quoted strings and validated JSON-literal properties are sufficient for this
data language; imports, calls, spreads and a JSX runtime are not necessary.

The transient document represents authoring semantics, not just a publication
record. Unpublished Poll definitions, exact geometry and editor-only fields need
real representations. Native publication is a separate projection and can
normalize or omit some of them. Do not invent published resource references to
make an unpublished source look wire-valid.

All current Leaflet block/media types remain the target. Experiment 13 proves
a selected profile, not complete coverage. Until a type is implemented, return
a clear compilation error before remote effects; do not silently drop it.
Full web-editor-to-MDX reconstruction is not a prerequisite for local authorship.

## Alternatives and when they would win

| Alternative | Why it remains credible | What would make us choose it instead |
| --- | --- | --- |
| **Modern mdxjs Git fork**, parse-only | [12](../experiments/12-mdxjs-turbopack-identity/REPORT.md) proves native Key, nested components, literal data and a modern Serde build; typed mdast is a clean compiler input | Actual workspace integration or compiler ergonomics are better than Satteri; choose its exact tested Git revision, not registry mdxjs with the same version label |
| **Rushdown with our dialect extensions** | [06](../experiments/06-rushdown-identity/REPORT.md) proves real parser hooks and both identity styles with ordinary Markdown | Custom syntax/CommonMark behavior becomes central enough to justify owning rich component/property grammar rather than using MDX |
| **Direct markdown-rs with literal expression handling** | Source exposes a parse-to-mdast route and expression callbacks; ordinary runtime dependencies do not require mdxjs's SWC compiler stack | A concrete build/dependency constraint justifies proving and owning that narrower expression boundary; it has not had its own executable experiment here |

The strongest **demonstrated** fallback is the modern mdxjs fork. It does not
require Turbopack, Next.js, Node or React at runtime. It does build SWC machinery,
and the tested source is Git-pinned. Stock registry mdxjs in
[09](../experiments/09-mdxjs-identity/REPORT.md) needed old Serde pins; the fork
fixes that practical failure. Do not describe the modern fork as still broken.

Direct `satteri-mdxjs` is **not another parser alternative**: [10](../experiments/10-satteri-mdxjs-identity/REPORT.md)
shows it is a JS compiler over the same Satteri parser. We need neither its
generated JS nor its MDAST→HAST route. Those routes' metadata/tab-normalization
findings are reasons to avoid unnecessary compilation, not faults in direct
Satteri authoring.

Plain pulldown-cmark in [08](../experiments/08-pulldown-cmark-identity/REPORT.md)
is useful for Markdown/event transforms but supplies no semantic MDX components.
For a custom rich grammar, Rushdown's demonstrated extension seam is the more
interesting alternative. Do not implement all the candidates in production.

## What matters before adoption

### Practical gates

1. **Real receiver integration.** Check selected Satteri/Yrs dependencies and
   features against the application workspace and intended build toolchain.
   Isolated experiment builds do not establish that feature union, target matrix
   or dependency cost. Record findings rather than guessing from compiler names.
2. **Promote one vertical slice.** Extract the parser-independent authoring model
   and small Satteri lowering seam from 13/14 deliberately, with ordinary prose,
   native Key, optional braces, one named page and an interactive object. Do not
   blindly copy the simulator or test-only transport into production.
3. **Verify actual remote mechanisms separately.** Staging/reading an unpublished
   subtree, switching the entry reference, media upload and native publication
   need real transport evidence under explicit operator authorization. Until then
   offline replacement does not prove deployed permissions or publication.
4. **Expand content by capability.** Prove each new block/asset mapping through
   actual target semantics and read-back. Unsupported input fails early. Keep
   source schemas, resource preparation, facts and text encoding separate enough
   to extend without rebuilding the parser.

The selected body is recreated, not a multiplayer editor. Fresh Yrs full updates
belong on fresh text entities; unrelated freshly encoded text merged onto an
existing entity does not replace it. Stable Polls and interrupted staged writes
need the explicit read-before-reuse rules demonstrated by 13.

Do not automatically delete old generations or assets. Replacing the owned
entry detaches old content; accumulated remote data is a known cost until a
separate cleanup policy is deliberately chosen. Old block comments/anchors are
not transferred to regenerated body entities. Stable document identity is the
promise; fine-grained collaborative continuity is not.

### Limitations in proportion

Use fenced blocks for multiline code. The unusual multiline-single-backtick
example with flow JSX is a documented MDX interpretation surprise, not evidence
that routine fenced technical writing is broken. Retain its regression evidence;
do not make universal CommonMark repair a prerequisite for useful authoring.

Exact spans for every JSX property and a general source formatter are not
required for a reader/compiler that leaves the body alone. Useful diagnostics
and owned frontmatter patching do matter. No-op operations should leave source
bytes unchanged.

No performance winner is established. Both Satteri/OXC and mdxjs/SWC have
dependency obligations. Investigate real constraints if they arise; another
parser bakeoff or speculative benchmark ranking is not the recommended next step.

## Evidence and how this updates earlier recommendations

[13](../experiments/13-satteri-authoring/REPORT.md) reports 29 Rust tests and four
independent Yjs consumer groups. [14](../experiments/14-satteri-authoring-attributes/REPORT.md)
reports 21 tests and equivalent typed IR/fact plans. These are scoped receipts,
not more points than another library's test total. The simulator remains a model;
neither experiment executed Leaflet writes or native publication.

[Astra's preceding assessment](authoring-parser-assessment1.gpt6a.md) correctly
called Satteri the strongest alternative and said the exact Key/compiler proof
was missing. It predates 13/14. This recommendation updates that judgment with
new evidence rather than rewriting the earlier record.

The [experiment index](../experiments/README.md) supplies the complete reading
map. The [recreation proposal](mdx-recreate0.gpt6a.md) remains the lifecycle
precedent; the [Yrs conversion design](yrs-conversion0.gpt6a.md) remains useful for
wire/inline/content obligations, not a requirement to implement character-level
synchronization first.

**Recommended next decision:** select Satteri and this source-authoritative
authoring direction for an explicitly commissioned integration slice, with the
modern mdxjs fork as the bounded fallback if that integration reveals a concrete
problem. This document does not itself authorize that implementation or a live
probe.
