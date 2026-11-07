---
type: Experiment
title: "Sätteri mdxjs is the compiler over its pulldown arena, not another parser"
description: "Actual satteri-mdxjs execution, shared-arena identity and literal components, compiler normalization/metadata boundaries, and comparison with the earlier direct-mdxjs proof."
resource: /experiments/10-satteri-mdxjs-identity/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-07
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e48016574ffeTCuLLn14abM6ej"
    title: "Commissioned fresh Sol: exact package discovery, real compiler integration, guarded arena lowering, initial failed assumptions, and offline executable comparison"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Source-authoritative local file, transient compiler data and literal component requirements; no production implementation commissioned here"
  - resource: /experiments/README.md
    title: "Tracked experiment and synthetic/read-only scope"
  - resource: /experiments/07-satteri-identity/REPORT.md
    title: "Existing actual pulldown/arena execution, native attributes, metadata binding and retained silent multiline-code counterexample"
  - resource: /experiments/07-satteri-identity/src/lib.rs
    title: "Prior public arena/codecs and sibling-binding implementation; reused approach, not a runtime dependency"
  - resource: /experiments/07-satteri-identity/tests/comments.rs
    title: "Prior attachment, literal-code and byte-position corpus"
  - resource: /experiments/07-satteri-identity/tests/substrate.rs
    title: "Prior stock parser controls separating native behavior from identity policy"
  - resource: /experiments/09-mdxjs-identity/REPORT.md
    title: "Direct wooorm mdxjs/SWC proof, Serde pin burden, hash-expression accident and guarded comment policy"
  - resource: /experiments/09-mdxjs-identity/src/lib.rs
    title: "Read typed projection API and boundary design for comparison"
  - resource: /experiments/09-mdxjs-identity/src/literal.rs
    title: "Reused recursive duplicate-key-rejecting JSON visitor in an independent local copy"
  - resource: /experiments/09-mdxjs-identity/tests/stock.rs
    title: "Read direct mdxjs controls for accepted hash expressions, callbacks, native components and multiline-code interruption"
  - resource: /experiments/06-rushdown-identity/REPORT.md
    title: "Earlier custom-extension versus ready-native-JSX boundary"
  - resource: /experiments/08-pulldown-cmark-identity/REPORT.md
    title: "Earlier upstream event/offset seam and explicit absence of native MDX components"
  - resource: /experiments/10-satteri-mdxjs-identity/src/lib.rs
    title: "Executed actual compiler APIs, MDAST gate, projection and intended HAST integration"
  - resource: /experiments/10-satteri-mdxjs-identity/tests/integration.rs
    title: "Thirteen stock compiler/arena/options/position/normalization/metadata controls"
  - resource: /experiments/10-satteri-mdxjs-identity/tests/authoring.rs
    title: "Thirteen overlapping identity, nesting, code, component, literal and position tests"
  - resource: /experiments/10-satteri-mdxjs-identity/tests/cli.rs
    title: "Three actual executable success/rejection/control tests"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/README.md"
    title: "Read public package map: source folder satteri-mdxjs-rs links published satteri-mdxjs"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-mdxjs-rs/Cargo.toml"
    title: "Read exact package identity and direct pulldown/arena/OXC dependencies"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-mdxjs-rs/src/lib.rs"
    title: "Read public compile and compile_hast_arena APIs, internal same-parser call, tab expansion and error conversion"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-mdxjs-rs/src/configuration.rs"
    title: "Read compiler options and inherited parse fields; absence of their use was separately searched and controls executed"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-ast/src/convert.rs"
    title: "Inspected JSX metadata copy across MDAST/HAST; executed loss-of-prose-metadata control"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-arena/src/mdx_types.rs"
    title: "Read compiler Message/Place/Location types and UTF-16 conversion"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-plugin-api/src/plugin.rs"
    title: "Inspected Plugin trait and whole-root arena seam; no plugin runner was executed"
---

# Sätteri mdxjs / shared arena experiment

## Outcome and one-or-two decision

**One new experiment is sufficient.** `satteri-mdxjs-rs` is the repository folder;
the publicly published crate is **`satteri-mdxjs` 0.3.17**. It is an OXC-based fork
of mdxjs-rs, but **it no longer uses markdown-rs as its Markdown parser**. Its real
source compilation calls **`satteri-pulldown-cmark::parse`**, exactly the 0.6.7
parser already executed in [experiment 07](../07-satteri-identity/REPORT.md).

This experiment really builds and calls `satteri_mdxjs::compile` and
`compile_hast_arena`; it is not ordinary `mdxjs` renamed in a report. There is no
second independent Sätteri parser to compare. The useful missing follow-up controls
(shared corpus, guarded binding, native conversion, code/diagnostic normalization)
fit here. Creating experiment 11 would duplicate the same parser without a distinct
question. Experiments 07 and 09 remain unchanged, including their negative evidence.

**Verdict:** the common MDAST arena is useful for local semantic authoring. The
mdxjs compiler adds no needed capability to the proposed Leaflet data compiler;
keep Leaflet lowering on MDAST. Both Sätteri names inherit the same MDX grammar
hazards. Modern dependencies build cleanly, but the compiler introduces additional
normalization and position pitfalls described below. No production choice is accepted.

## Exact executable identity and setup

| Item | Pin / binding |
| --- | --- |
| Public registry discovery | `cargo search satteri-mdxjs`; `cargo info satteri-mdxjs@0.3.17` |
| Actual compiler | `satteri-mdxjs = "=0.3.17"` |
| Registry compiler checksum | `e91e3095ba2af2739ed44d82c545c0ecd5026306bfa9f772d4179b634cb326ff` |
| Parser | `satteri-pulldown-cmark = "=0.6.7"`; default `std`, `mdx` |
| Common AST / arena | `satteri-ast = "=0.6.3"` (default `mdx`), `satteri-arena = "=0.4.3"` |
| Serialization | `serde = "=1.0.229"` with derive; `serde_json = "=1.0.154"` |
| OXC | Resolved 0.121.0, including parser, AST, codegen, semantic and related crates |
| Inspected source | `~/archive/bruits/satteri`, HEAD `7d1297b7e4551a1c5124fa07bd2d1d05f66236e6` |
| Published revision | Compiler registry `.cargo_vcs_info.json` has that SHA and `crates/satteri-mdxjs-rs` path; matches archive HEAD and 07's pins |
| Toolchain executed | `rustc 1.101.0-nightly (822182db4 2026-10-13)` |

The existing archived checkout, its AGENTS, package map, relevant source and
prior experiment code were inspected before implementation. Dependencies were
added with Cargo tooling; [Cargo.toml](Cargo.toml) owns a separate `[workspace]`
and [Cargo.lock](Cargo.lock) pins the graph. No sibling experiment is a path
dependency. Downloads were setup traffic; final runs/checks are offline.

Unlike direct `mdxjs` 1.0.4 in [09](../09-mdxjs-identity/REPORT.md), **this stock
package compiles with current Serde without a source patch or historical pin**.
There is no SWC or markdown-rs dependency in this graph. No archived source,
registry source, root manifest/lock, production code, index, ticket or ignore file
outside this directory was edited. No Node/NAPI toolchain is needed.

## Actual public interfaces and architecture

```text
stock: source
  -> satteri_mdxjs::compile(source, &Options, pulldown_flags)
     -> expand leading tabs when any tab is present
     -> satteri_pulldown_cmark::parse -> Arena<Mdast>
     -> satteri_ast MDAST-to-HAST -> Arena<Hast>, set mdx flag
     -> compile_hast_arena -> OXC transformations/codegen -> JavaScript

authoring check: unchanged source
  -> satteri_pulldown_cmark::parse(MDX_OPTIONS) -> SAME Arena<Mdast>
  -> native-node sibling binder + literal gate -> transient projection

staged compiler control: validated/mutated Arena<Mdast>
  -> public MDAST-to-HAST conversion, explicitly hast.mdx = true
  -> satteri_mdxjs::compile_hast_arena
```

There is **no mdast-returning `mdast_util_from_mdx` API in this compiler crate**,
unlike direct mdxjs. Its intended arena integration is public and works: parse
through its parser dependency, mutate before conversion, compile the resulting
HAST. Recalling `compile(source, ...)` after changing an arena would discard the
changes by reparsing the source.

The common arena/codecs materially enable the same identity/component lowering
without a translation to another parser tree. Internal key data is written to
real MDAST nodes with `set_node_data`; directives are detached via `set_children`.
This is a post-parse pass, not parser injection. The inspected Rust `Plugin` trait
also accepts MDAST and offers root transformation, but **neither its runner nor
the JS plugin bridge is executed**. Adding that packaging would not repair a
native parse defect or create a second parser. No HTML serialization/reparse occurs.

The compiler additionally exposes `parse_expression_to_estree_json` and
`parse_esm_to_estree_json`; both are executed in controls. They parse, not execute,
arbitrary JavaScript. This differs from 09's expression-validation seam, which
returns strings/stops rather than a reusable SWC tree. They are unnecessary for
the smaller strict-JSON document property policy.

### Exact compiler controls and surprising boundaries

[integration.rs](tests/integration.rs) executes these properties:

1. For the comment, nested component, ordinary Markdown and multiline-code
   fixtures **without tabs**, stock `compile` and direct parser → unmodified
   native HAST → `compile_hast_arena` produce identical JavaScript. This proves the
   intended integration and shared parser, not broad cross-parser equivalence.
2. `Options.parse.constructs.heading_atx/attention = false` changes neither tested
   construct. The compiler uses the separate pulldown flag argument instead;
   searching its source found no operative use of these inherited parse fields.
   Changing the explicit flags does change the grammar. Do not configure this fork
   as if it were upstream mdxjs's markdown-rs parser.
3. Stock codegen discards identity **comment text**, leaving empty JSX expressions
   in JSX-retaining mode. Initial tests wrongly expected preservation and failed;
   corrected controls retain this discovery. Our binder removes those nodes before
   conversion. Stock compilation is not identity extraction.
4. Prose `springleafKey` metadata **does not survive** tested heading conversion
   to HAST, while JSX node metadata does. Generated JavaScript is not a transport
   for internal prose identity. Do not key via HTML `id` just to recover it.
5. Stock compilation accepts and emits an unresolved import, `danger()` and spread
   props, without resolving the nonexistent module. Our `staged` gate rejects them.
   JavaScript is never evaluated. Both JSX-retaining and default JSX-compiling
   codegen are executed as controls, not an authoring runtime.
6. **`compile(source, ...)` expands a leading tab in fenced code to spaces.** The
   direct arena route retains that tab. This extra source-compiler normalization
   is outside 07's direct parser path and matters for exact code payloads. It is
   not a reason to preprocess author source or to treat generated JS as storage.

## Author identities and native grammar limits

The optional syntax selected is the user's permitted comment alternative:

```mdx
{/* springleaf:key introduction */}
## Introduction

{/* springleaf:key central-argument */}
An ordinary paragraph with **formatting**.
```

The binder follows 07's native sibling policy with 09's **blank-before guard**:

- Keys match `[A-Za-z][A-Za-z0-9_.-]*`, case-sensitive and document-wide unique.
  Components' optional `key="NAME"` joins that same namespace. Neither HTML anchors,
  remote IDs nor Yjs identities are created.
- Only standalone native flow comments bind; the immediate next supported sibling
  in the same native container is the target. No skipping definitions/comments/
  unsupported nodes. Targets: heading, paragraph, list, quote, code, thematic break
  and flow component. A marker before a list keys the list; one inside an item keys
  that item's paragraph, not the item itself. Nested quote/list/Page scopes are tested.
- A preceding sibling requires a blank line before the marker. Blank lines after
  a marker are fine. This avoids the tested silent multiline-backtick capture.
- Malformed/misplaced/unbound/unsupported/duplicate/colliding keys diagnose;
  duplicates clear all accepted instances. Unterminated syntax can fail upstream.
  Any diagnostics mean CLI exit 2 and reject the output; inspectable partial trees
  are not publishable results. Ordinary prose/headings need no annotations.

**The exact paragraph-tail `{#central-argument}` remains incompatible with this
stock MDX grammar.** Enabling native attributes recognizes the heading ID but the
paragraph tail is a flow expression rejected by OXC. This is reproduced through
actual compiler execution, not inferred from 07. Direct mdxjs/SWC in 09 accidentally
accepts the user's exact names as expressions; that is neither shared behavior nor
a real attribute feature. We did not implement another attribute dialect here.

The [multiline-code fixture](fixtures/multiline-code.mdx) still loses both native
inline-code spans in stock MDX, just as 07 and 09 report. Stock compiler codegen
does not fix it. This binder rejects both accidental key directives with
`key-boundary` and assigns neither; **the original code AST is not restored**.

Indented code is also still disabled by MDX. A four-space-indented marker can
silently key following prose in this experiment too; a passing test explicitly
records that unmet CommonMark/code-is-literal expectation. The blank-line guard
is not comprehensive Markdown compatibility. Same-line inline code, fenced code
and escaped openings protect the tested directive-shaped literals. Choose a
fence-required MDX dialect deliberately, or require a parser correction before
adoption. No parser patch or lexical repair is hidden in this experiment.

## Literal component fidelity and position contract

[components.mdx](fixtures/components.mdx) contains the exact Poll arrays, nested
Page/Canvas/Place/Image, Markdown children and inline Run/HardBreak. Native JSX
flow/text kinds, names, ordered attribute kinds/raw expression strings and ordered
children are retained. Quoted entity text becomes `A & B`; the exact Run JSON string
retains spaces, newline, special characters and Unicode. No wrapper is required
around ordinary Markdown blocks.

The local name registry is Poll, Canvas, Page, Place, Image, Run and HardBreak.
Unknown/lowercase/fragment names diagnose instead of disappearing. Braced values
must be **one strict JSON value**, including recursively unique object keys; the
visitor is an independent attributed copy of 09's gate. Calls, arithmetic, member
access, arrows, templates, identifiers, non-JSON object syntax, comments, trailing
commas, spreads, ESM and body expressions are rejected. JSX duplicate properties
diagnose. Native parser syntax validation is separate from this smaller policy.

This is not a complete secure storage/compiler schema: required fields, property
types/names, child models, URLs/assets, poll semantics, geometry precision/ranges,
resource budgets and all unsupported Markdown node lowering remain open. The
projection is an inspection tree, not a complete Leaflet IR. `staged` only proves
compiler interoperability; source-authoritative Leaflet work needs neither its
JavaScript nor HAST.

| Position/source interface | Executed contract |
| --- | --- |
| Native MDAST node range | Half-open UTF-8 bytes, BOM-stripped source; Unicode/CRLF and nested slices tested |
| Native columns | One-based UTF-16 code units; heading with Café/🦀 tested |
| Comment annotation range | Exact native expression span stored on target metadata, excluding quote/list prefix and newline |
| JSX individual attribute range | Not supplied by these inspected codecs; raw expression strings and element spans retained, no fabricated token range |
| Initial BOM | Explicit `position_source_start = 3`; add to byte offsets before slicing original source |
| `parse_error_to_message` | Converts a byte offset to `Message` whose `Point.offset` is **UTF-16 units**, unlike arena byte offsets; tested 8-byte versus 5-unit case |
| `compile` error positions with tabs | Refer to its tab-expanded source: tested offset shifts by three relative to direct original-source conversion |
| This check CLI parser errors | Retain native byte offset as zero-width span, zero line/column sentinels; do not conflate with compiler helper positions |

No-op preservation means retaining the exact borrowed original bytes; tested with
BOM/Unicode/CRLF. There is no body writer, serializer, synchronized JSON or arbitrary
source-edit contract. Reordering annotated blocks retains keys and changes spans.
Whole-node source ranges are not lossless semantic-to-source reconstruction.

## Comparison and recommendation for the assessor

| Concern | Sätteri pulldown / 07 + this run | Direct mdxjs / 09 |
| --- | --- | --- |
| Parser identity | Same actual pulldown 0.6.7 arena for both Sätteri names | markdown 1.0.0 with mdxjs 1.0.4/SWC callbacks |
| Native rich structure | Poll/nested Page/Canvas and inline JSX demonstrated here | Poll/nested Canvas and inline JSX demonstrated in 09 |
| Comment keys | Immediate-sibling arena metadata; this run adds explicit boundary guard | Typed-node transform with boundary guard |
| Apparent multiline code | Same native interruption, guard mitigates key capture only | Same observed interruption and limitation |
| Compact requested attributes | Heading native flag works; paragraph MDX expression errors | Exact examples accepted accidentally as expressions, other keys error |
| Current Serde | Builds unchanged with 1.0.229 | 09 required 1.0.219/JSON 1.0.140 for old SWC |
| Parse-only seam | Public pulldown parse already returns common arena; mdxjs compiler unnecessary | Public mdxjs parse exists but mandatory compiler deps build; direct markdown callback seam remains prospective |
| Extra compiler behavior | Tab-normalized source/code payloads, UTF-16 diagnostic offsets, inherited parse fields ignored | Those specific fork behaviors not tested for upstream; do not assume equivalence |

Semantic equivalence is **bounded**, not universal: both families preserve tested
ordinary marks/container structure and native literal components; their expression
acceptance, options, defaults, positions and compilation machinery differ. This
run does not rerun 09 or claim byte-identical ASTs across its typed tree and the
arena. Sources/tests were read and shared input forms exercised here.

**Favor direct Sätteri pulldown → MDAST over Sätteri mdxjs for the Leaflet path.**
The compiler is good executable evidence that this fork has a real integration,
not an extra parser capability or a necessary abstraction layer. Its current
dependency compatibility is better than the pinned direct mdxjs/SWC graph, but
that does not resolve the common grammar mismatch. If a ready native JSX grammar
and fenced-code MDX restrictions are acceptable, Sätteri remains credible. If
ordinary CommonMark literal-code expectations are non-negotiable, **neither
Sätteri name is proved suitable unchanged**. Rushdown/upstream pulldown have
stronger demonstrated attribute/code paths but still lack ready semantic JSX in
06/08; this new compiler run does not alter that tradeoff.

Exact remaining assessor questions:

1. Accept explicit MDX/fences plus blank-before annotations, or require a native
   code-span/indented-code correction? This is a grammar decision, not adapter polish.
2. Is paragraph-tail `{#id}` important enough to commission a new parser seam or
   fork? No compatible general native Sätteri MDX form was demonstrated.
3. Choose direct Sätteri arena versus a bounded markdown-rs callback experiment
   before importing either compiler stack into production. No such new callback
   experiment or upstream patch was commissioned/executed here.
4. Define fail-closed supported nodes, per-component schemas, frontmatter ownership,
   soft-wrap rules and limits. Do not use success of JSON parsing or JavaScript
   compilation as complete Leaflet validation.
5. Keep compiler diagnostic coordinates separate from MDAST/original byte spans.
   No full source formatter is needed for the planned frontmatter-owning CLI.

## Reproduction, receipts and connections

[README.md](README.md) has commands and exit expectations. Final offline checks:

```sh
cargo nextest run --locked --offline --no-fail-fast
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
cargo tree --locked --offline --depth 2 -e normal
```

**29/29 passed, zero skipped**: 13 authoring, 13 integration, 3 executable CLI
tests. Nextest run `9a71d0d9-9cc3-4983-a528-7c5930886dfc`. Clippy passed with
warnings denied and formatting passed. Passing negative observations are not
capability successes: indented key capture, lost code AST, metadata loss and tab
normalization remain limitations. No fuzzing/performance claim is made.

Implementation checkpoints (explicit owned paths only):

- `6649573a` — actual compiler/arena integration, first 27 controls; 25 passed,
  two wrong comment-output assumptions preserved before correction.
- `4556d72d` — corrected comment controls plus normalization/default-codegen probes;
  all 29 tests and offline Rust checks pass.

Document checks from the repository root:

```sh
cargo run --locked --offline --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf experiments/10-satteri-mdxjs-identity/REPORT.md
cargo run --locked --offline --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check experiments/10-satteri-mdxjs-identity/README.md
```

Both checks passed offline: report **1 document, 13 links, 24 sources**, including
native session provenance; README **1 document, 1 link**.

The [source-authoritative design](../../design/mdx-recreate0.gpt6a.md) supplies the
single-file/transient-compiler boundary, not authority for remote writes. The
[07 report](../07-satteri-identity/REPORT.md) owns the prior pulldown proof;
[09](../09-mdxjs-identity/REPORT.md) owns the distinct upstream-mdxjs proof and the
reused literal/boundary ideas. [06](../06-rushdown-identity/REPORT.md) and
[08](../08-pulldown-cmark-identity/REPORT.md) remain the earlier Markdown/attribute
alternatives. Root indexes and reciprocal updates are left to the commissioning
parent to avoid touching other sessions' ownership. No live writes, credentials,
new tickets, agents, production implementation or production test runs occurred.
