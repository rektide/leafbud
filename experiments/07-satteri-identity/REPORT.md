---
type: Experiment
title: "Sätteri native MDX nodes for optional author keys"
description: "Offline Rust proof of preceding comment keys and literal component data, with native attribute/MDX conflicts and literal-code counterexamples retained as negative evidence."
resource: /experiments/07-satteri-identity/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-07
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4815c84cffewCcjzglN74Kqbv"
    title: "Commissioned independent Sätteri experiment: package discovery, native source inspection, attribute controls, comment lowering, literal component gate, and explicit unmet code invariants"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Source-authoritative local authoring and literal-data component requirements; not a commission for production or remote writes"
  - resource: /experiments/06-rushdown-identity/REPORT.md
    title: "Read previous experimental findings, README, attribute and substrate tests for comparison, not as a required architecture"
  - resource: /experiments/07-satteri-identity/src/lib.rs
    title: "Executed public parser/arena API, native expression binding, typed projection, native attribute controls and non-executing literal gate"
  - resource: /experiments/07-satteri-identity/tests/substrate.rs
    title: "Direct upstream AST controls separating parser behavior from our lowering"
  - resource: /experiments/07-satteri-identity/tests/comments.rs
    title: "Attachment, nested scope, formatting, diagnostics, code counterexample, move/reorder, Unicode/CRLF/BOM positions"
  - resource: /experiments/07-satteri-identity/tests/components.rs
    title: "Literal JSX-like component data and rejection controls"
  - resource: /experiments/07-satteri-identity/tests/attributes.rs
    title: "Markdown-only native attribute lowering and demonstrated limitations"
  - resource: /experiments/07-satteri-identity/tests/cli.rs
    title: "Executable success, exit-2 rejection, and exit-0 multiline-code misinterpretation controls"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/README.md"
    title: "Authoritative project identity and Rust/npm package map; located through the published satteri 0.2.17 README"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri/src/lib.rs"
    title: "Inspected high-level Rust facade: HTML rendering and MDX-to-JavaScript compilation, not a public AST return"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/arena_build.rs"
    title: "Inspected parse API, MDX_OPTIONS, native attributes, JSX codecs, source positions and BOM handling"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/block_attributes.rs"
    title: "Inspected native attribute attachment, tight list-item targeting, lazy container targeting and pre-inline processing"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/firstpass.rs"
    title: "Inspected intentional MDX indented-code disabling and native flow-expression paragraph interruption"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/parse.rs"
    title: "Inspected ParserCallbacks: broken-link callback, not arbitrary block/inline parser injection"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/tests/mdx_heading_attributes.rs"
    title: "Read upstream ATX/setext MDX heading-attribute examples; these do not establish paragraph-tail attribute support"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-ast/src/mdast/codec.rs"
    title: "Inspected typed code/link/list data and public JSX name, attribute-kind, expression-value decoders"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-arena/src/arena.rs"
    title: "Inspected public get_children/set_children, get_node_data/set_node_data, string resolution and original-source API"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-arena/src/line_index.rs"
    title: "Verified byte offsets versus UTF-16 column/JS offset conventions"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-plugin-api/src/plugin.rs"
    title: "Inspected Rust visitor/plugin interface; available but not used by this direct arena lowering"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/website/content/docs/entry-points.md"
    title: "Read documented TypeScript tree-returning entrypoints separately from compile/evaluate APIs"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/website/content/docs/plugins.md"
    title: "Read documented AST transformations and sibling restructuring examples"
---

# Sätteri author-key experiment

## Verdict

**Conditional fit, not ready to adopt unchanged as a Markdown-compatible authoring
substrate.** Sätteri's native MDX AST is substantially closer to our component/data
requirements than Rushdown's CommonMark AST. Preceding comment annotations bind
successfully without a fork, parser replacement or source preprocessor. However:

1. The requested paragraph-tail `{#id}` form does **not** compose with native MDX
   parsing, even when general attributes are enabled.
2. Native flow comments can steal annotation-shaped text from a **multiline
   backtick span**. The prototype then keys the wrong paragraph with **no error**.
3. MDX mode intentionally disables indented code.

There are **47 passing offline tests**, including tests that assert those negative
observations. That is not a claim that 47 desired authoring invariants hold. The
literal-code counterexample is an unmet invariant, not something the prototype
repairs. No production parser or syntax choice is accepted here.

All work lives in this new experiment space. The preceding
[Rushdown experiment](../06-rushdown-identity/REPORT.md) is untouched. There are no
resulting root production manifest/lock changes, Leaflet requests, remote writes,
credentials, source serializers, maintained JSON companions or delegated agents.
Package downloads and the upstream clone were setup traffic; ordinary checks run
offline afterward.

## Exact identity, dependencies and revision

The package is **Sätteri, `bruits/satteri`**, a Rust/TypeScript monorepo—not a
guessed crate alias. No existing archived checkout was initially available.
`cargo search satteri` and `cargo info satteri@0.2.17` found the published package;
its README links the authoritative repository. It was cloned to
`~/archive/bruits/satteri`. Its root AGENTS, contribution guide, README, website
docs, parser/arena/codec source, upstream tests and attribute examples were read.

| Item | Exact evidence |
| --- | --- |
| High-level facade inspected | `satteri` **0.2.17**; exposes `markdown_to_html` and feature-gated `compile_mdx`; not used as the runtime AST interface |
| Executed parser | **`satteri-pulldown-cmark = "=0.6.7"`**, features `std` (default), `mdx`, `serde` |
| Executed AST | **`satteri-ast = "=0.6.3"`**, `mdx` feature |
| Executed arena | **`satteri-arena = "=0.4.3"`** |
| Inspected Git revision | **`7d1297b7e4551a1c5124fa07bd2d1d05f66236e6`** |
| Published revision binding | All three executed crates and the facade have this SHA in their registry `.cargo_vcs_info.json`; the archive clone HEAD matches |
| Parser registry checksum | `4233d41e98cf2dffde8f8356a7e3d582688ce2e5c9192164193a2ae217faf697` |
| AST registry checksum | `ee1ee3790fa38c5604d9c986be504005123d1aa8c99ccbeeccc1d6833ff1698c` |
| Arena registry checksum | `b73729378f2c3e040b49e73cc7205fdf10dd19fc5f99217149e73437d6a28fbe` |
| Other direct pins | `serde` 1.0.229 with derive; `serde_json` 1.0.154 |
| Runtime checked | `rustc 1.101.0-nightly (822182db4 2026-10-13)` |

Dependencies were added through Cargo tooling. The isolated
[manifest](Cargo.toml) has its own `[workspace]`; its [lockfile](Cargo.lock) pins
all transitive packages, including OXC 0.121.0 for JavaScript **parsing**, not
execution. The experiment uses Rust in-process: **no Node.js or NAPI build is
needed**. The TypeScript API also documents `mdxToMdast`, but was not executed.

## Native architecture actually used

[Our code](src/lib.rs) calls
`satteri_pulldown_cmark::parse(source, options) -> (Arena<Mdast>, Vec<(usize,String)>)`.
The native parser already constructs `Heading`, `Paragraph`, `List`, `ListItem`,
`Blockquote`, inline marks/links/code, MDX expressions and JSX flow/text elements.

| Public facility | Executed responsibility |
| --- | --- |
| `MDX_OPTIONS` | Native Markdown + JSX/expression/ESM parsing; includes upstream default GFM/math/frontmatter flags |
| `MdastNodeType`, `get_node`, `get_children` | Inspect actual typed parser nodes and their real container/sibling relationships |
| `decode_expression_data`, `get_str` | Read the native comment-expression body; recognize the narrow reserved directive there |
| `set_children` | Detach annotation nodes from the transient tree after binding; source is never changed |
| `get_node_data`, `set_node_data` | Attach **`leafbudKey`** and **`leafbudKeySpan`** to the actual target parser node, independently of HTML `id` |
| Public MDAST codecs | Preserve heading depth, list/item properties, link destinations, code payload and native JSX attribute kinds/values |
| Typed inspection projection | Preserve children/inline structure and the optional key; emit transient JSON for tests/CLI inspection |

This is an ordinary **post-parse AST lowering**, not a new parser syntax hook.
Sätteri also supplies a Rust `Plugin` trait and JS MDAST visitors. Neither the
Rust plugin runner nor the JS bridge is needed for this small direct arena pass.
The inspected `ParserCallbacks` offers broken-link handling, not Rushdown-style
block/inline parser constructor injection. A plugin cannot restore code structure
already lost during native parsing without additional parsing/repair machinery.

There is no HTML rendering or HTML reparsing in the executed path. Escapes and
entities are already decoded in native text values; inline/container nodes remain
structured. The projection is a small typed inspection tree, **not** a complete
Leaflet IR or semantic compiler for every upstream node type.

## Why comments, rather than forcing attributes

Native Markdown-only `Options::ENABLE_ATTRIBUTES` successfully handles:

```md
## Introduction {#introduction}

Ordinary **formatted** prose.
Another line.
{#central-argument}
```

The identity control reads native `data.hProperties.id`, validates/transfers it to
internal key metadata and removes the HTML `id`. No attribute range is invented.
Sätteri's native grammar is broader/different from experiment 06: same-line
paragraph attributes work; a tail inside a tight list keys the **list item**, not
its paragraph; native indentation can target containers. An orphan yields an empty
paragraph which our control rejects. Multiple IDs on one node are already collapsed
to the last ID, and malformed attributes can remain plain text without diagnostics.
Those are explicit limitations of this **control mode**, not silently completed
identity policies.

Combining `MDX_OPTIONS | ENABLE_ATTRIBUTES` preserves the heading ID, but the
paragraph's own-line `{#central-argument}` becomes `MdxFlowExpression` and produces
an OXC parse error. [The direct upstream test](tests/substrate.rs) proves this
before our binder runs. The upstream MDX heading-attribute tests cover headings,
not a promise that all block attribute forms compose with MDX.

Thus `author` deliberately uses native MDX **without** general attributes and
selects the user's alternative:

```mdx
{/* leafbud:key introduction */}
## Introduction

{/* leafbud:key central-argument */}
This is an ordinary paragraph with **formatting**.
```

### Attachment contract implemented

- The single comment body must be exactly `/* leafbud:key NAME */`.
  Keys match `[A-Za-z][A-Za-z0-9_.-]*`, case-sensitive, document-wide unique.
  Unicode prose is supported; Unicode author keys are intentionally rejected.
- Only native **flow** expression nodes bind. Inline/same-line/heading-contained
  annotations diagnose `misplaced-key`; reserved malformed comments diagnose.
  Unterminated comments produce native parse errors. Ordinary single block-comment
  expressions remain inert `Comment` nodes, not keys or executable code.
- Bind to the immediate next supported sibling **in the same native container**.
  Blank lines between marker and target are allowed. Supported targets are
  heading, paragraph, list, quote, code, thematic break and flow component.
- A comment before a list keys the list; a comment inside an item followed by
  indented prose keys that paragraph, not the item. Nested quotes/items are tested.
  No direct list-item comment spelling is demonstrated; native attributes have
  different item-targeting behavior.
- Empty-container/trailing annotations cannot escape into root prose or another
  item. Reference definitions and consecutive marker expressions are not skipped.
  A first consecutive annotation diagnoses; the second may bind its following
  block in inspectable partial output, but the result is invalid as a whole.
- Duplicate keys diagnose both and clear their accepted keys. Component `key`
  properties share the prose-key namespace. A comment plus component `key` is a
  collision even when equal; neither wins. No HTML anchor, array-position identity
  or Yjs character identity is implied.
- Unlike experiment 06, a native MDX flow comment **can interrupt preceding prose
  without a blank line**. That upstream grammar choice also causes the code hazard
  below; it is not merely a convenient binding feature.

Any diagnostic produces CLI exit **2**, while retaining partial projection for
inspection. Future consumers must reject diagnostic-bearing output; no save or
publication consumer is implemented here.

## Literal component control: a real native MDX advantage

[The fixture](fixtures/components.mdx) includes:

```mdx
<Poll key="next-topic" options={["Gardens", "Maps"]} />

<Image src="./assets/diagram.png" width={640} metadata={{"alt":"Diagram","decorative":false}} />
```

These are native **`MdxJsxFlowElement` nodes**, with component names, distinct
literal/expression/boolean/spread attribute kinds and expression source strings.
Inline components become `MdxJsxTextElement`; multiline attributes and Markdown
children are also demonstrated. They are not opaque HTML or fallback prose.

Our deliberately bounded gate recognizes `Poll` and `Image`. Unknown names,
lowercase elements and fragments are rejectable by their AST kind/name rather
than text heuristics. Braced property values must parse as **one strict JSON
value** via `serde_json`; arrays, objects, numbers, strings, booleans and null fit
that grammar. Calls, functions, shorthand objects, single-quoted JS strings,
trailing-comma JS arrays, arbitrary expressions and spreads are rejected.
Standalone expressions/ESM are rejected; top-level JSX property duplicates diagnose.

The parser itself accepts arbitrary JavaScript expressions. **Parsing is not a
literal-only security policy.** This experiment never invokes `compile_mdx`,
`mdxToJs`, `evaluate` or any component runtime. It does not validate complete Poll/
Image property schemas, required fields, unknown properties, nested JSON duplicate
keys, resource limits, URLs or numeric precision policy. That work belongs in the
eventual typed document compiler, not an assertion that JSX parsing makes storage
safe. This is several bounded successful/negative controls, not full MDX support.

## Source preservation and positions

`original_source()` retains the exact input bytes, including CRLF and an initial
BOM. There is no source write/serialize path; no-op retention is tested directly,
not claimed as edited-AST round-trip reconstruction.

| Position | What this probe actually exposes |
| --- | --- |
| Native node range | Half-open UTF-8 byte offsets from each arena node; heading/fence syntax slices, Unicode/CRLF boundaries and child ranges are tested |
| Native line/column | One-based; columns count **UTF-16 code units**, confirmed by Unicode tests and `LineIndex` source |
| Initial BOM | Native parser strips it before positioning. Printed `position_source_start` is 3 for a UTF-8 BOM, otherwise 0; add it when slicing the retained original |
| Comment key annotation | Exact native expression range, including `{/* ... */}` but excluding quote/list prefixes and its trailing line ending; `key_span` is copied from that node |
| Native attribute/JSX key token | No individual token range exposed through the inspected metadata/codecs here; `key_span` stays absent instead of guessing |
| Native parse error | Upstream exposes a byte offset/message; represented as a zero-width range with zero line/column sentinels |

Whole-node ranges are more directly available than Rushdown's starts/payload
segments, but they are still **semantic AST positions, not a lossless CST or a
verified arbitrary-edit rewriter contract**. Removing a marker from the transient
tree does not shrink an ancestor's original source range. Native code values also
normalize away the final payload newline in the tested fence. Source stays intact;
reconstructing source from those normalized values would not be a no-op.

## Important failing requirement retained, not repaired

[This fixture](fixtures/multiline-code.mdx) is the decisive counterexample:

```text
Code `first
{/* leafbud:key literal */}
last`.
```

| Mode | Executed observation |
| --- | --- |
| `plain` | One paragraph containing a multiline `InlineCode`; annotation-shaped text is literal |
| `mdx` | Two paragraphs separated by a native flow comment; no `InlineCode`; no parse error |
| `author` | The comment keys the trailing `last\`\.` paragraph as `literal`; **exit 0 with no diagnostic** |

The same break is reproduced inside a quote. The native first pass's
`mdx_block_interrupts` treats flow expressions as paragraph interrupts; the
single-line/code-span safeguards do not preserve this example. This report does
not establish whether every other MDX implementation agrees—it establishes this
pinned implementation's behavior and its mismatch with the desired code-literal
expectation. Separately, firstpass explicitly disables indented code for MDX, so
four leading spaces do **not** protect a key comment there.

Same-line inline code, escaped opening braces and fenced code do protect the
tested key-shaped text. Fenced examples are a viable authoring restriction, but
that is a human grammar choice, not a repair already supplied by this prototype.
No regex preprocessor, backtick scanner or second parser is hidden in the binder.

## Comparison and recommendation

| Concern | Experiment 06 / Rushdown | This Sätteri run |
| --- | --- | --- |
| Optional keys | Custom block/inline hooks + AST transform; both requested spellings | Native Markdown attributes as controls; native MDX comments + AST lowering selected |
| Components with JSON props | Can fall back to prose; no semantic MDX component AST | Native flow/text JSX nodes, attribute kinds and names; literal-only gate demonstrated |
| `{#id}` + rich MDX | Future component grammar still needed | Heading works with combined flags; exact paragraph-tail form conflicts |
| Multiline code protection | Preserved in experiment 06's selected extension; eager-interruption negative control | Native MDX flow grammar reproduces the interruption problem; not fixed here |
| Positions | Starts and payload segments; no universal end range | Native start/end node ranges, UTF-16 columns, exact comment range; BOM base must be explicit |
| Inline semantic values | Inspection projection retains raw escapes/entities | Native text values decode tested escapes/entities |
| Rust integration | In-process public extension constructors | In-process public parser/arena/codecs; facade alone is insufficient; OXC dependency footprint, not benchmarked |

**Recommendation:** keep Sätteri as a serious **MDX-first conditional candidate**,
not a proven drop-in replacement for the lightweight Markdown dialect. Native
component recognition is its strongest advantage, and the optional identity layer
is small once the native grammar has made the right tree. The exact `{#id}` family
is not a free native MDX solution.

Before adoption, deliberately choose between accepting the demonstrated MDX
literal-code restrictions (with fences and a clear authoring contract) and requiring
a parser-level correction/customization for the unmet invariant. The inspected
public callbacks do not offer the same grammar injection as Rushdown. Do not turn
this experiment into a replacement parser to conceal that tradeoff. If unrestricted
ordinary Markdown code semantics are required, **do not adopt this run unchanged**.

Remaining compiler work includes complete typed component schemas, all supported
Markdown node lowering, frontmatter interpretation, text-break policy, resources,
diagnostic boundaries and real Leaflet generation. Native GFM/math/frontmatter
flags are enabled in MDX defaults but their complete compilation was not tested.
No fuzzing, performance, pathological-depth, browser runtime or plugin-composition
guarantees are made. No production implementation is commissioned by these findings.

## Reproduction and receipts

See [README](README.md) for prerequisites, modes and CLI examples. Setup once:
`cargo fetch --locked`. Then, from this directory, offline:

```sh
cargo nextest run --locked --offline --no-fail-fast
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
cargo run --locked --offline -- author fixtures/comments.mdx
cargo run --locked --offline -- author fixtures/components.mdx
cargo run --locked --offline -- author fixtures/negative.mdx  # expected exit 2
cargo run --locked --offline -- plain fixtures/multiline-code.mdx
cargo run --locked --offline -- author fixtures/multiline-code.mdx  # known semantic failure, exit 0
```

Executed results: **47/47 tests passed, zero skipped**, nextest run
`8ad9ee5b-b3ca-4f18-95f2-0b77584b3c42`: 22 comment/identity tests, 8 component
tests, 8 direct upstream controls, 6 native-attribute controls, 3 CLI tests.
Clippy passed with warnings denied; formatting passed. CLI tests exercise the
real executable: clean comment fixture, rejection fixture with two duplicate-key
diagnostics plus unknown-component/nonliteral/spread errors, and the falsely clean
multiline-code result. All of those tests run offline.

Checkpoints (owned explicit paths only):

- `ab28cda1` — native facility/grammar discovery, first 7 upstream controls.
- `1d2338ad` — comment lowering, component gate, 44 tests and documented counterexamples.
- `818448ed` — executable CLI success/rejection/counterexample controls, 47 tests.

Document checks:

```sh
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf experiments/07-satteri-identity/REPORT.md
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check experiments/07-satteri-identity/README.md
```

Both checks passed: report **1 document, 11 links, 22 sources**, including native
session provenance; README **1 document, 1 link**. No production test suite was
rerun for this isolated experiment.

### Connections

- [The source-authoritative direction](../../design/mdx-recreate0.gpt6a.md) supplies
  the authoritative-file/transient-AST boundary and literal-data component need.
  Optional keys here refine prose identity without requiring wrapper components
  or reviving synchronization machinery.
- [Experiment 06](../06-rushdown-identity/REPORT.md) supplies the preceding evidence
  and overlapping corpus. The architectures differ deliberately: this run tests
  native MDX rather than rebuilding its syntax on CommonMark.
- [The experiment guide](../README.md) supplies the isolated/synthetic/no-write
  evidence boundary. This new directory preserves the earlier run rather than
  replacing it; sibling experiment integration/index ownership remains with the
  commissioning parent session.
