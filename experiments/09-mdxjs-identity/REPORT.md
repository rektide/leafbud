---
type: Experiment
title: "Native MDX comments and literal components through Rust mdxjs"
description: "Executable parse-only proof of optional local author keys, real JSX structure, restricted JSON properties, source offsets, and the costs and limits of mdxjs as a storage substrate."
resource: /experiments/09-mdxjs-identity/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-07
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4812756bffetb3uFNU3E1ERrt"
    title: "Commissioned fresh Sol experiment: authoritative source inspection, failed stock assumptions, native comment binding, literal validation, and offline checks"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Single authoritative local source and literal rich-component authoring; later user correction keeps prose ordinary Markdown"
  - resource: /experiments/README.md
    title: "Read the experiment scope and production no-write boundary; not modified"
  - resource: /experiments/06-rushdown-identity/REPORT.md
    title: "Earlier extension proof and explicit gap in real MDX component parsing"
  - resource: /experiments/06-rushdown-identity/tests/substrate.rs
    title: "Read code ownership controls and CommonMark versus JSX counterexamples"
  - resource: /experiments/06-rushdown-identity/tests/comments.rs
    title: "Read prior container-local identity and code-span expectations"
  - resource: /experiments/06-rushdown-identity/tests/attributes.rs
    title: "Read compact-syntax prose, code, reordering, and offset expectations for comparison"
  - resource: /experiments/09-mdxjs-identity/src/lib.rs
    title: "Executed public parse-only API, transient projection, comment binding, and component policy"
  - resource: /experiments/09-mdxjs-identity/src/literal.rs
    title: "Strict JSON property parser, including recursive duplicate object-key rejection"
  - resource: /experiments/09-mdxjs-identity/tests/stock.rs
    title: "Eight upstream controls, including surprising hash-expression acceptance and native flow interruption"
  - resource: /experiments/09-mdxjs-identity/tests/identity.rs
    title: "Eighteen author identity, container, prose, code, and position tests"
  - resource: /experiments/09-mdxjs-identity/tests/components.rs
    title: "Fifteen literal component, rejection, exact-text, and source-position tests"
  - resource: /experiments/09-mdxjs-identity/tests/cli.rs
    title: "Three executable stdout and exit-status controls"
  - resource: "https://github.com/wooorm/mdxjs-rs/blob/282729e6a0750c19dfd9217bc511971bf3295a29/readme.md"
    title: "Authoritative crate purpose, markdown-rs seam, lack of plugins, and security warning"
  - resource: "https://github.com/wooorm/mdxjs-rs/blob/282729e6a0750c19dfd9217bc511971bf3295a29/src/lib.rs"
    title: "Public parse-only API and hardcoded markdown-rs/SWC callback wiring"
  - resource: "https://github.com/wooorm/mdxjs-rs/blob/282729e6a0750c19dfd9217bc511971bf3295a29/src/configuration.rs"
    title: "MDX constructs and callback customization are not exposed as parser options"
  - resource: "https://github.com/wooorm/mdxjs-rs/blob/282729e6a0750c19dfd9217bc511971bf3295a29/src/swc.rs"
    title: "Expression and ESM validation parses SWC trees but returns signals; no execution"
  - resource: "https://github.com/wooorm/markdown-rs/blob/1506572f9b406431402928f3a8b3df0b4ae2d8f5/src/mdast.rs"
    title: "Real typed JSX nodes, string expression values, stops, and intentionally absent attribute positions"
  - resource: "https://github.com/wooorm/markdown-rs/blob/1506572f9b406431402928f3a8b3df0b4ae2d8f5/src/configuration.rs"
    title: "Underlying public MDX options and brace-only behavior without expression callbacks"
---

# Rust mdxjs: source-authoritative identity and literal components

## Verdict

**Directly implementable through public `mdxjs` APIs, without a fork, private
module access, preprocessing, HTML rendering, or JavaScript evaluation. Prefer
native preceding MDX comments for prose identities on this substrate.** Real
`Poll`/nested `Canvas` JSX parsing works; 44 offline tests pass.

The user's compact `{#id}` preference is not dismissed as inherently impossible.
The important correction is empirical: **this pinned parser accepts the user's
exact heading and paragraph examples as JavaScript-shaped expressions.** It does
not interpret them as metadata. Other otherwise-valid author keys fail the JS
grammar. We therefore chose a native comment AST transform, not a fragile
hash-expression convention or a disguised lexical adapter.

`mdxjs` is a credible MDX parser entry point but not an automatic winner for
**data-only storage**: its obligatory SWC compiler dependency stack and an actual
Serde compatibility failure are unnecessary costs if a smaller literal-only
parser seam can do the job. Compared with Rushdown, its major proven advantage
here is **real JSX structure already present**, not identity extensibility.

No files outside this experiment were changed. Parallel experiments 07/08 were
not read. No production tests, Leaflet requests/writes, Yrs work, source rewrite,
or synchronized JSON output were performed. Downloads were dependency/source setup;
the final execution/checks were offline.

## Exact libraries, inspected revisions, and setup failure

| Item | Executable pin / inspected source |
| --- | --- |
| Crate | `mdxjs = "=1.0.4"`, authoritative repository `wooorm/mdxjs-rs` |
| Published revision and inspected clone HEAD | `282729e6a0750c19dfd9217bc511971bf3295a29` |
| Registry checksum | `b307331572976abf2ddb519f45ecf210fe266e8c9d4200b32e288fd24cce7808` |
| Underlying crate | `markdown = "=1.0.0"`, authoritative repository `wooorm/markdown-rs` |
| Published revision and inspected clone HEAD | `1506572f9b406431402928f3a8b3df0b4ae2d8f5` |
| Registry checksum | `a5cab8f2cadc416a82d2e783a1946388b31654d391d1c7d92cc1f03e295b1deb` |
| Direct serialization dependencies | `serde = "=1.0.219"` with derive; `serde_json = "=1.0.140"` |
| SWC resolved through mdxjs | `swc_core` 27.0.6, `swc_common` 12.0.1, `swc_ecma_parser` 15.0.2 |
| Toolchain | `rustc 1.101.0-nightly (822182db4 2026-10-13)` |

Neither authoritative checkout was present initially. `cargo info mdxjs`
established the crate/repository; clones now live at
`~/archive/wooorm/mdxjs-rs` and `~/archive/wooorm/markdown-rs`. Registry
`.cargo_vcs_info.json` matches both inspected HEADs. No `AGENTS.md` or `llms.txt`
was found there. The previously discovered `satteri-mdxjs` manifest was only a
discovery clue; **it is not a dependency or the parser evaluated here**.

Dependencies were added using Cargo tooling. [Cargo.toml](Cargo.toml) has its own
`[workspace]`; [Cargo.lock](Cargo.lock) pins transitive dependencies. Root manifests
and locks remain untouched.

**The first build genuinely failed:** resolving current `serde` 1.0.229 and
`serde_json` 1.0.154 made `swc_common` 12.0.1 fail at
`pub use serde::__private as serde` (unresolved import). Pinning the older Serde
and JSON versions above allowed the unchanged published mdxjs/SWC sources to
build. No registry-source edits, patched SWC, or silent parser substitution were
used. This is a concrete dependency-maintenance cost, not a performance claim.

## The exact public seam exercised

Storage path:

```text
original &str
  -> mdxjs::mdast_util_from_mdx(source, &mdxjs::Options::default())
  -> markdown::mdast::Node
  -> experiment's typed Projected tree + diagnostics
  -> stdout for inspection (never a second maintained document)
```

`mdxjs::mdast_util_from_mdx` is public. Internally it constructs
`markdown::ParseOptions`, enables MDX flow/text expressions, JSX and ESM, disables
ordinary HTML/indented code/autolinks, and calls `markdown::to_mdast`. It installs
its private SWC expression and ESM callbacks. We call **none** of those private
functions ourselves.

The direct `markdown` dependency supplies the exact shared typed AST, `Node`
accessors, JSX attributes and positions. Its `serde` feature prints the stock
tree. A single explicit negative control calls public `markdown::to_mdast` with
`ParseOptions::mdx()` and **no callback**, showing it accepts `{a + }` while
`mdxjs` rejects it. That is not presented as equivalent mdxjs evaluation.

The public parse result contains MDX expression **strings and source mappings**,
not a reusable SWC/ESTree expression tree. SWC parses expressions for syntax
validation and returns signals, dropping that parsed tree. Our separate strict
JSON parser establishes the much smaller storage grammar; neither general JS
syntax acceptance nor the existence of an upstream parser proves data safety.

`mdxjs::compile` is called once in a stock **test control** to inspect generated
JavaScript containing `#id`; nothing executes it. The storage/CLI `check` path
does not call compile, HAST conversion, JSX rewrite, codegen or a runtime.
The compiler stack still builds because it is not optional in mdxjs's manifest.
Upstream's README itself recommends markdown-rs for parse-only use and says this
project has no plugin support. Its public options do not expose replacement
expression callbacks or toggles to disable the mandatory MDX constructs.

## Stock hash-attribute behavior: corrected, not assumed

The initial test expected strict rejection and **failed**. The retained controls
now document the actual behavior:

| Source | Stock `mdast_util_from_mdx` result |
| --- | --- |
| `## Introduction {#introduction}` | Heading with Text plus `MdxTextExpression { value: "#introduction" }` |
| `Prose.` then `{#central-argument}` | Paragraph, followed by a **sibling** `MdxFlowExpression { value: "#central-argument" }` |
| `{#trailing.}` or `{#two..dots}` | SWC parse error |
| Escaped `\{#id}`, inline code or fenced code | Literal content, not metadata |

SWC's private-name-shaped expression handling accepts the exact example. Even
`mdxjs::compile` carries `#id` into generated JavaScript; no internal author-key
semantics appear. This is **not** an implemented attribute extension or a general
license to put the author-key grammar into braces.

A transform could reinterpret the accepted expression subset, but it would tie
identity acceptance to incidental JS grammar, including punctuation-dependent
failures **before** the transform. Turning off expressions globally is not an
available mdxjs option and would sacrifice literal component expression syntax.
Using markdown-rs's configurable callback, changing mdxjs, or building a
context-aware lexical adapter are possible different seams, **not implemented
proofs here**. No regex or line replacement is used anywhere in this experiment.

## Selected identity grammar

```mdx
{/* leafspring:key introduction */}
## Introduction

{/* leafspring:key central-argument */}
This is an ordinary paragraph with **formatting**.
```

- The exact single-line comment value is `/* leafspring:key NAME */`.
  Keys match `[A-Za-z][A-Za-z0-9_.-]*`, case-sensitive, document-wide unique.
  Unicode prose is supported; Unicode key names diagnose. Periods need not
  satisfy JS member-expression grammar; comment keys `trailing.` and `two..dots`
  are deliberately tested.
- A metadata comment must be a standalone **flow expression**, not an inline
  comment in prose. It binds the immediate next supported sibling in its current
  Root, Blockquote, ListItem or flow-component children. Blank lines **after**
  the directive are allowed. Definitions, ordinary comments, unsupported nodes,
  and another directive are barriers, not things to skip.
- Supported targets: heading, paragraph, fenced code, list, quote, thematic break,
  and flow component. A preceding comment keys a list as a whole;
  `- {/* leafspring:key first */}` followed by indented prose keys that item's
  **paragraph**, not the ListItem. No direct list-item identity spelling is defined.
- If there is an earlier sibling, require at least one blank line **before** the
  identity comment. This is checked using actual node line positions, not a
  guessed source scanner. Consecutive directives never merge; the first diagnoses
  unbound, and the second must also satisfy the blank-boundary rule.
- Malformed reserved `leafspring:` comments, orphan/misplaced comments and duplicate
  keys diagnose. Unterminated MDX syntax can instead fail upstream first. A comment
  in an empty quote/item/component cannot escape into outside prose or another item.
- Component `key="topic"` is optional internal identity too. It shares the global
  key namespace with prose comments. Comment plus component key on one target gives
  two collision diagnostics and neither identity wins. Reusing one key on different
  nodes diagnoses/removes both bindings. All diagnostics make the CLI exit **2**;
  partial inspectable output is never authorization to compile/publish it.

Keys are not HTML anchors, array positions, remote entity IDs or Yjs identities.
They are optional author names carried by this transient tree. No identity
allocation or persistence policy beyond source annotations is implemented.

### Native MDX's multiline-code trap

Another initial test failed because this source acquired a key:

````mdx
Code `first
{/* leafspring:key accidental */}
last`.
````

**Stock mdxjs itself** parses Paragraph / MdxFlowExpression / Paragraph here,
not one multiline code span. The adapter's blank-boundary rule now rejects the
directive without assigning identity. It does not pretend to restore CommonMark
code semantics. Inline code on one line, multiline code with a non-flow-leading
continuation, fenced code, and escaped opening braces retain their native literal
interpretation and are tested. MDX also disables ordinary indented-code parsing;
use fences. This matters when migrating ordinary Markdown, independently of keys.

## Real JSX and a restricted literal data grammar

[components.mdx](fixtures/components.mdx) exercises actual MDX nodes, not raw HTML
or a string that merely looks like a component:

```mdx
<Poll key="topic" options={["Gardens", "Maps"]} config={{"anonymous": true, "limit": 2}} />

<Canvas width={800} height={400}>
  <Place x={12.25} y={40.5} rotation={-2.75}>
    <Image src="./map.png" alt="A &amp; B" />
  </Place>

  Markdown **inside** a canvas.
</Canvas>

Exact <Run text={"  literal\ntext <>{}"} /> then <HardBreak />end.
```

The projection retains ordered children, real Markdown inlines, names, flow/text
distinction and ordered properties. Quoted attributes are strings with upstream
entity decoding (`A &amp; B` becomes `A & B`); bare attributes are `true`.
**Braced properties must be strict JSON:** strings, finite representable numbers,
booleans, null, arrays and objects. Recursive duplicate object keys diagnose rather
than silently overwrite. Attribute duplicates diagnose too. JSON object key order
is not promised by the semantic `Value`; `raw_expression` and the original source
remain available. Numbers use serde_json's normal integer/f64 representation, not
an arbitrary-precision geometry guarantee.

Tests reject syntactically valid JavaScript calls, property access, arithmetic,
arrows, templates, identifiers, undefined/NaN/Infinity, parenthesized objects,
unquoted/single-quoted object keys, comments, trailing commas and spread data.
Imports/exports, body expressions and spread attributes diagnose. Key-shaped text
inside JSX expression strings/arrays is untouched, proving no global brace/comment
replacement has interfered. Exact spaces, Unicode, tabs and newlines in the `Run`
JSON string survive as data.

The name registry here recognizes only Poll, Canvas, Place, Image, Run and HardBreak.
**Unknown components retain names, valid data and children, but diagnose**, so a
compiler cannot silently omit them. Syntax errors remain upstream errors.

This is **not a finished secure storage schema**. It does not validate per-component
property names/types/required fields, event-like names, child models, URL protocols,
asset access, poll option semantics, coordinate ranges, depth/size budgets or all
Leaflet kinds. Literal data is never executed here, but a future renderer/compiler
must still impose those rules. Do not pass this output into a general JSX runtime
or declare arbitrary downstream handling safe because the JSON parser accepted it.

## Compiler-facing projection and honest positions

For the first block of [comments.mdx](fixtures/comments.mdx), the executable emits
this structure (omitting null/empty fields):

```json
{
  "kind": "Heading",
  "span": { "start": 36, "end": 51 },
  "key": "introduction",
  "key_origin": "Comment",
  "key_span": { "start": 0, "end": 35 },
  "details": { "type": "Heading", "depth": 2 },
  "children": [{
    "kind": "Text",
    "span": { "start": 39, "end": 51 },
    "text": "Introduction"
  }]
}
```

[src/lib.rs](src/lib.rs) exposes typed Kind/Details/Projected/Property structures.
Paragraph/Heading children retain Strong/Emphasis/Link/InlineCode/Break structure;
list numbering, heading depth, code language/meta, image/link data and reference
information remain explicit. Soft wraps retain upstream text newlines; downstream
semantic whitespace/facet lowering is not implemented. Text is already decoded for
Markdown escapes/entities, unlike Rushdown's raw text segments in experiment 06.

| Position | Actual result |
| --- | --- |
| Node ranges | mdast `position.start/end.offset`; half-open **UTF-8 byte** offsets in the exercised Unicode/CRLF cases |
| Identity comment | Exact source expression span including `{/* ... */}`, not quote/list prefix or trailing newline |
| Headings, prose, code, JSX elements | Upstream start/end regions, including delimiters where the node owns them; not a lossless CST |
| JSX property / quoted value | **No complete attribute span** in mdast; `MdxJsxAttribute.position` is commented out upstream |
| Braced JSX values | `raw_expression` plus upstream `stops` (relative-value-byte → absolute-source-byte mapping); not fabricated property delimiter ranges |
| Component `key` attribute | Key value/origin retained; `key_span: null`; diagnostics point to the element region |

CRLF/Unicode tests slice original bytes at node/comment ranges and verify expression
stops. A container's contiguous region can include intervening indentation/quote
prefixes; semantic text does not reconstruct those bytes. This is sufficient for
compiler diagnostics and lowering, **not proof of an edit-preserving formatter**.

No-op preservation is explicit but modest: `Parsed.original_source` borrows the
unchanged input; tests compare original bytes. There is no serializer/write path.
Moving annotated blocks changes spans while retaining keys. No AST or JSON is
maintained alongside source, and there is no HTML rendering/reparse step.

## Fit versus the earlier Rushdown experiment

1. **Identity alone:** Rushdown has the stronger demonstrated extension seam for
   the user's compact attribute family. mdxjs has no equivalent general plugin hook;
   native comments compose coherently with its existing AST. This is a syntax
   tradeoff, not proof that the user must abandon attributes in every substrate.
2. **Rich components:** mdxjs solves the concrete Rushdown gap: JSON-like JSX props,
   nested elements and Markdown children are real typed nodes immediately. No
   ordinary prose wrapper components are needed.
3. **Source diagnostics:** mdast exposes useful whole-node end positions and
   semantic text directly. Attribute position precision remains missing, and neither
   experiment establishes a production source rewriter.
4. **Storage cost:** native MDX expression semantics are broader than the requested
   data language. SWC validation, ESM and the compile pipeline are mostly unnecessary
   for data-only authoring; the Serde pin failure is a real integration warning.
   Prose also inherits MDX's brace/JSX rules: expression-shaped text is not simply
   ordinary literal Markdown. Escape literal braces or use code/literal data.
5. **Recommendation:** retain this as a positive direct-mdxjs proof, with comment
   annotations if choosing mdxjs. For production selection, compare its ready JSX
   grammar against a smaller parse-only/literal seam before accepting the compiler
   dependency cost. A markdown-rs-only callback adapter is a plausible separate
   investigation, **not built or benchmarked here**. No parser winner or final syntax
   is declared accepted by this experiment.

Remaining work includes schemas and compiler lowering, frontmatter policy (default
options here leave it off), GFM/plugin composition, resource limits, fuzzing,
performance and complete Leaflet mappings. This experiment deliberately does not
expand into remote writes, Yrs integration, or whole-application construction.

## Reproduction and receipts

See [README.md](README.md) for the runnable command list. Final Rust checks:

```sh
cargo nextest run --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
cargo tree --locked --offline --depth 2 -e normal
```

**44/44 tests passed, zero skipped:** 8 stock, 18 identity, 15 component/data,
3 actual CLI controls. Final nextest run:
`88d56d26-95ab-4598-adcf-4cc2a6ddb096`. Clippy passed with warnings denied;
formatting passed. CLI controls verify clean comments/components exit 0, the
stock attribute example exits 0, the selected dialect's attribute example exits 2,
and rejected data emits five diagnostics and exits 2. A separate CLI smoke run
printed the heading projection above.

Owned implementation checkpoints:

- `dc71aa22` — pinned mdxjs baseline, seven passing initial controls.
- `f3fb4377` — native comment binding, strict literals, typed projection and 44 tests.

Document checks from the repository root:

```sh
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf experiments/09-mdxjs-identity/README.md experiments/09-mdxjs-identity/REPORT.md
```

Document verification passed for both owned documents, including local destinations,
anchors, frontmatter and native session provenance.

### Connections and scope

- [The source-authoritative design](../../design/mdx-recreate0.gpt6a.md) motivates
  a transient compiler tree and richer literal components. This experiment follows
  the later user correction: normal prose stays Markdown, not Paragraph/Heading JSX.
- [Rushdown's report](../06-rushdown-identity/REPORT.md) is the prior identity
  baseline and the JSX gap this pass concretely fills. We read its relevant tests;
  no current parallel experiment was read or incorporated.
- [Experiment guidance](../README.md) supplies the no-production-write boundary.
  Root indexes, manifests, locks, ignore rules and ticket state were left untouched.
