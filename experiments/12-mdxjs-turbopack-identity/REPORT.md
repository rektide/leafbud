---
type: Experiment
title: "Vercel's mdxjs fork: current Serde, native Key, unchanged MDX boundaries"
description: "Exact Git/package identification and executable comparison of mdxjs-rs-turbopack against mdxjs 1.0.4, including native declarative Key markers, literal components, source positions, and retained code-ownership failures."
resource: /experiments/12-mdxjs-turbopack-identity/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-07
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e47fbf926ffeNzxvNxB16GDFRp"
    title: "Commissioned fork identity/build/behavior experiment, extended by user steering toward native Key rather than assumed comment adoption"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Single authoritative local source, literal components, ephemeral AST and native publication goal; no publication attempted"
  - resource: /design/authoring-parser-assessment0.gpt6a.md
    title: "Read provisional assessment and next-proof boundaries; this experiment supplies new evidence, not an accepted replacement ruling"
  - resource: /experiments/README.md
    title: "Tracked experimental evidence and no-live-write boundary"
  - resource: /Cargo.toml
    title: "Read production Serde/serde_json version requirements for the isolated compatibility comparison; not modified or integration-tested"
  - resource: /experiments/09-mdxjs-identity/REPORT.md
    title: "Stock mdxjs 1.0.4 baseline, old Serde failure, grammar and position constraints"
  - resource: /experiments/09-mdxjs-identity/src/lib.rs
    title: "Read full baseline projection/binder; new smaller comparison adapter follows its scoped comment policy"
  - resource: /experiments/09-mdxjs-identity/src/literal.rs
    title: "Read and reused strict JSON visitor design, including recursive duplicate-key rejection"
  - resource: /experiments/09-mdxjs-identity/tests/stock.rs
    title: "Read shared upstream syntax and code-interruption controls"
  - resource: /experiments/09-mdxjs-identity/tests/identity.rs
    title: "Read shared identity, code, container and position corpus"
  - resource: /experiments/09-mdxjs-identity/tests/components.rs
    title: "Read shared component/data and position corpus"
  - resource: /experiments/12-mdxjs-turbopack-identity/UPSTREAM.json
    title: "Recorded exact upstream Git tree, tar/selected-source SHA-256 values, markdown checksum and experiment lock checksum"
  - resource: /experiments/12-mdxjs-turbopack-identity/src/lib.rs
    title: "Executed actual fork parse-only entry point, native Key/comment binding and literal-only interpretation"
  - resource: /experiments/12-mdxjs-turbopack-identity/tests/key.rs
    title: "Executed native marker grouping, Key consumption, scoped identities, rejection and code-ownership controls"
  - resource: "https://github.com/vercel-labs/mdxjs-rs-turbopack/blob/700e192fbdbad5874208addfbf421dcf1eca7a20/Cargo.toml"
    title: "Inspected authoritative fork manifest: package still mdxjs 1.0.4, mandatory SWC 84, declared Rust 1.88"
  - resource: "https://github.com/vercel-labs/mdxjs-rs-turbopack/blob/700e192fbdbad5874208addfbf421dcf1eca7a20/src/lib.rs"
    title: "Read actual public parse-only function and compared unchanged file against upstream 1.0.4"
  - resource: "https://github.com/vercel-labs/mdxjs-rs-turbopack/blob/700e192fbdbad5874208addfbf421dcf1eca7a20/src/swc.rs"
    title: "Read private-module SWC expression validation and tree-dropping callback behavior"
  - resource: "https://github.com/vercel-labs/mdxjs-rs-turbopack/blob/700e192fbdbad5874208addfbf421dcf1eca7a20/readme.md"
    title: "Read inherited compiler purpose, parse-only markdown-rs recommendation, no-plugin statement and security boundary"
  - resource: "https://github.com/vercel/next.js/blob/8a54c08e5d6950729f63dfc6dc3fb7b62ac8d69c/Cargo.toml"
    title: "Existing local archived Next.js manifest identified the authoritative Git fork; this archive is not claimed as current Next.js HEAD"
  - resource: "https://github.com/wooorm/markdown-rs/blob/1506572f9b406431402928f3a8b3df0b4ae2d8f5/src/mdast.rs"
    title: "Read native JSX flow/text/attribute definitions directly: no self-closing flag or complete attribute position"
---

# Vercel mdxjs fork: what it actually fixes

## Verdict

**A genuine dependency-compatibility improvement, not a new authoring grammar.**
The fork builds unchanged with current production-compatible Serde versions,
without Turbopack. It does **not** remove SWC, fix the tested MDX code-span
interruptions, add `{#id}` grammar, or add attribute spans.

**Native MDX + a reserved `<Key id="introduction" />` marker is executable and
cleaner than inventing a custom brace-ID grammar.** It leaves headings/paragraphs
as ordinary Markdown, needs no JS execution, and does not require comment syntax.
This is a candidate matching the user's updated preference, **not adopted syntax**.
The self-closing form, paired form, and invalid unclosed form are distinguished
below. Key is an adapter-defined declaration carried by real native JSX nodes;
the parser does not itself know identity semantics.

Do not recommend this fork as though it cures the source of all previous gotchas.
If the product chooses an MDX dialect, the fork is a reasonable replacement for
stock mdxjs's broken dependency integration. If the requirement is unchanged
CommonMark code ownership, **neither stock nor this fork qualifies unchanged**.
The experiment deliberately retains false-clean negative evidence rather than
relabeling surprising interpretation as safety.

## Identity, lineage, distribution and reproducible build

The requested name was not assumed to be a crate. `cargo info mdxjs-rs-turbopack`
failed with “could not find”; Cargo search lists `mdxjs`, not a fork-named release.
The npm exact-name endpoint returned 404. These are discovery results, not proof
that no unrelated package can ever exist. The authoritative local clue was the
existing archived Next.js root manifest, which references:

```toml
mdxjs = { git = "https://github.com/vercel-labs/mdxjs-rs-turbopack.git", branch = "turbopack" }
```

That archive's lock uses `07688e832d27f04628f77793ad21b35021cd4efb` (SWC 79).
This experiment instead pins the fork's observed current default-branch tip:

| Item | Exact tested identity |
| --- | --- |
| Repository / branch | `vercel-labs/mdxjs-rs-turbopack`, `turbopack` |
| Git revision | `700e192fbdbad5874208addfbf421dcf1eca7a20` (`Bump to swc 84`) |
| Git tree | `3d5ac556760d4c1801f5c3c6e29a386827e56b4b` |
| Actual Cargo package/version | **`mdxjs` 1.0.4**, Git source, not crates.io stock 1.0.4 |
| Lineage baseline | upstream `wooorm/mdxjs-rs@282729e6a0750c19dfd9217bc511971bf3295a29` |
| Underlying parser | unchanged registry `markdown` 1.0.0 |
| Resolved SWC | `swc_core` 84.0.2, `swc_common` 27.0.0, `swc_ecma_parser` 47.0.0 |
| Direct serialization pins | `serde` 1.0.229, `serde_json` 1.0.154 |
| Declared fork MSRV | 1.88, raised from upstream 1.56; minimum/transitive MSRV not verified |
| Executed toolchain/host | `rustc 1.101.0-nightly (822182db4 2026-10-13)`, `x86_64-unknown-linux-gnu` |

The Vercel Labs archive was absent initially. After checking local archives, it
was cloned to `~/archive/vercel-labs/mdxjs-rs-turbopack`. No AGENTS.md or llms.txt
was found there. GitHub repository metadata identifies it as a fork of
`wooorm/mdxjs-rs`. Its inherited manifest/README still point to upstream's registry
and docs, so `cargo add mdxjs` alone **does not install the tested fork**.

[Cargo.toml](Cargo.toml) records the Git `rev`; [Cargo.lock](Cargo.lock) fixes
transitives. [UPSTREAM.json](UPSTREAM.json) records selected-source hashes and
the deterministic `git archive --format=tar REV` SHA-256:
`c3f7cdaf687fe875ec8b7507b2323ea02e44910441e73ce277bf214d1e048eee`.
This is a **Git archive checksum**, not an invented Cargo registry checksum;
Git dependencies have no registry checksum field. The unchanged markdown crate
has registry checksum
`a5cab8f2cadc416a82d2e783a1946388b31654d391d1c7d92cc1f03e295b1deb`.
There are **no local patches**, no edited registry sources and no vendored parser.

Dependencies were added with Cargo tooling. The first real fork build used modern
Serde and succeeded; no downgrade was needed. The direct version numbers match
production; the experiment does not exercise production's complete feature union.
This directly removes the old
experiment's `swc_common` 12.0.1 / `serde::__private` failure. Root production
manifests and locks were not touched; full production integration was not run.

## What changed upstream, and what did not

The complete baseline-to-fork diff changes 10 files, 75 insertions / 83 deletions.
Commits predominantly bump SWC and adapt its AST interfaces, plus lint fixes.
The manifest changes SWC **27 → 84** and declared Rust **1.56 → 1.88**.
`src/lib.rs`, `src/configuration.rs`, and `readme.md` are **byte-identical** to
the stock 1.0.4 revision (`git diff --exit-code … -- FILES` succeeded).
`src/swc.rs` itself changes only a format-string spelling; actual JavaScript
parser behavior may still change through SWC, so executable controls matter.

The new lock has 120 package entries versus 171 in experiment 09, including each
experiment root. That is a directly counted lock-graph observation, **not a
benchmark** or proof of runtime/binary-size improvement. SWC parsing, AST, codegen,
visit and transform crates still build on the parse-only route. No optional
parse-only feature removes them. Turbopack/Next.js/Node are **not** dependencies
of this direct Rust route. No performance ranking is inferred.

## Public API actually used

```text
unchanged input &str
  → fork mdxjs::mdast_util_from_mdx(source, &mdxjs::Options::default())
  → markdown::mdast::Node
  → scoped declarative-marker binder + strict JSON property validation
  → ephemeral inspection data on stdout
```

The fork's public function exists independently of compilation/Turbopack. The
experiment never calls `compile`, renders HTML, evaluates JS or invokes a React
runtime. A direct `markdown` dependency names/serializes the same returned typed
AST; it is not secretly substituted for the fork. CommonMark/no-callback calls
appear only as explicitly labelled controls.

Internally the fork still hardcodes MDX expressions/JSX/ESM on, ordinary HTML and
indented code off, with its private SWC expression/ESM callbacks. Public options
do not replace those callbacks or supply a general plugin interface. The SWC
syntax tree is discarded by the parse callback: public mdast expression values
are **strings plus source stops**, not a reusable JS AST. Strict JSON policy
therefore remains adapter work, independent of the fork's syntax acceptance.

## Native Key: implemented candidate, not comments by inertia

The user's steering changed the syntax emphasis. This exact form is now tested:

```mdx
<Key id="introduction" />
## Introduction

<Key id="central-argument" />

This is ordinary **formatted** prose.
```

Native mdast represents each marker as an **empty `MdxJsxFlowElement` named Key**,
with an `id` literal attribute, followed by a sibling Heading/Paragraph. Zero,
one or multiple blank lines **after** the marker preserve that grouping. A
multiline opening tag also works; this is not a line-regex recognizer.
[key-candidate.mdx](fixtures/key-candidate.mdx) yields five bindings, five consumed
marker paths and just one ordinary component, Poll. Headings, Strong inlines and
quote/list prose stay ordinary Markdown nodes; no Paragraph/Heading wrappers.

Candidate adapter contract:

- Key is reserved declarative source data, consumed before future Leaflet lowering.
  `consumed_annotations` records its native tree paths; the inspection AST retains
  raw nodes for evidence. No renderer/runtime component is invoked.
- Self-closing, standalone **flow** node; exactly one property named `id`, whose
  upstream value is a quoted literal string. Braced strings/expressions, spreads,
  duplicate/extra props, booleans, absent/invalid IDs diagnose. Single/double JSX
  quotes are equivalent; upstream entity decoding applies. IDs use the same
  case-sensitive `[A-Za-z][A-Za-z0-9_.-]*` namespace as comparison forms.
- Attach to the **immediate supported sibling in the same container**, never skip
  comments, definitions or another marker. Valid targets include heading, paragraph,
  fenced code, list, quote, rule and flow component. Inside a list item, the next
  paragraph is named, not the list item. Empty quote/item/Canvas cannot leak a name.
- Blank line **before** a marker after an earlier sibling is required by this
  adapter's inherited safety guard. After a marker no blank line is required.
  Adjacent JSX on the next line is the target; an unknown target still diagnoses.
  Inline `<Key … />` inside prose, including marker plus prose on one line, diagnoses.
- Duplicate IDs across Key/comments/component keys remove both bindings. A Key
  marker plus a target's own `key` is a collision, not a precedence convention.
- Whole native marker ranges provide exact source diagnostics. mdast has no
  self-closing boolean, so the adapter checks `/>` at the end of the native node's
  own source range. It does not parse/preprocess JSX or fabricate attribute spans.

### Paired and unclosed forms are different

`<Key id="intro">` without a closing tag is an upstream syntax error, **not**
a valid shorthand for a zero-width declaration.
[key-paired.mdx](fixtures/key-paired.mdx) is valid native MDX: the paired Key has
real Heading then Paragraph children when authored on block lines with blanks.
The candidate adapter returns `paired-key-unsupported`, including for an empty
paired Key. It does not silently decide whether `id` names a group, wrapper,
first block or every descendant. Paired single-block/group semantics remain an
explicit user choice; they are not necessary for the proven wrapper-free option.

### What Key improves and what it cannot improve

Key avoids JavaScript-private-name-shaped `{#id}` and comment-namespace spelling.
It uses existing JSX attribute grammar and an ordinary literal ID. The same native
JSX capability already exists in stock mdxjs: **Key is not a new fork feature**.
The semantic binder is small and operates after parsing, but cannot repair code
ownership lost before that stage. The remaining boundary rule is explicit, not
evidence that all Markdown documents are accepted unchanged.

## Common corpus: native preservation, guarded rejection, false-clean behavior

| Input/concern | Fork native result | Comparison adapter result |
| --- | --- | --- |
| `## Introduction {#introduction}` / paragraph tail `{#central-argument}` | Same expressions as stock, not metadata | `executable-expression`; no keys |
| `{#trailing.}` / `{#two..dots}` | SWC error, as stock | Cannot recover before parse failure |
| Valid preceding comment | Native flow expression, as stock | Scoped binding for comparison; not assumed adopted |
| Self-closing Key then normal Markdown | Native Key flow node + separate block | Reserved marker consumed; block identity bound |
| Root/quote/list apparent multiline code with comment/Key-leading line | Native paragraph interrupted; marker becomes a flow node | Blank-before guard diagnoses `misplaced-key`; **no key**, but code semantics not restored |
| Real single-line inline code, fenced code, non-flow-leading multiline span, escaped braces/angle syntax | Literal native content | No accidental Key/comment/component binding |
| Four-space-indented comment/Key then paragraph | Active flow node, **not code** in MDX | Can bind the following prose cleanly; retained negative observation under CommonMark expectations |
| Apparent multiline code containing `<Poll key="accidental" … />` | Native active component between paragraphs | Baseline-style component binding remains **false-clean**, exit 0 |
| Duplicate/orphan/malformed IDs, closed-container escape | Native tree still preserves evidence (or syntax error first) | Diagnostics, no cross-container capture; duplicate bindings removed |
| Valid paired Key | Native wrapper with ordered Markdown children | Deliberately unsupported grouping, exit 2 |

The important extended counterexample is
[multiline-component.mdx](fixtures/multiline-component.mdx). Not only comments can
interrupt an apparent code span: native flow JSX does too. Our comment/Key marker
guard does not automatically protect ordinary components' `key` attributes.
The previously built **stock experiment 09 executable** was run read-only against
this new fixture and also emitted component key `accidental`, no diagnostics,
exit 0. No old source was changed or old suite rerun. This is inherited behavior,
not evidence of a new fork regression.

Six tests are explicitly named `negative_observation`; other controls also record
constraints. **51 green tests do not mean 51 safe-authoring invariants.** The CLI's
false-clean observation is retained to prevent a production-readiness inference.

MDX deliberately differs from CommonMark here. If authors truly mean MDX, four
spaces are not a code fence. If they mean ordinary CommonMark code, silently
treating those bytes as identity/components is unacceptable. Naming the dialect
does not repair a product expectation mismatch.

## Rich literal data and positions

[document.mdx](fixtures/document.mdx) executes Poll arrays, nested **Page → Canvas →
Place → Image**, ordered Markdown children, geometry and exact Run string data.
Raw typed mdast retains nesting, text/Strong/Link nodes and source ranges. Ordered
component/property data views retain native paths and expression strings/stops.
The Page test establishes parsing, not page-reference resolution or Leaflet schema.

Strict braced-property JSON supports strings, finite numbers, bool/null, arrays
and recursive objects; it rejects duplicate object keys, non-JSON JS expressions,
calls, spreads, templates, identifiers, unquoted keys, comments and trailing commas.
Imports/exports/body expressions and unknown components diagnose. Unknown names,
children and literal data remain inspectable, not silently dropped. The JSON
visitor is derived from experiment 09, **not attributed to upstream**. JSON number
precision is serde_json's normal integer/f64 representation; semantic object key
order is not promised. Bare component attributes are true; quoted strings use
native entity decoding. Exact Run spaces/newlines/tabs/Unicode survive as data.

CRLF/Unicode tests slice original UTF-8 bytes at element and marker/comment ranges,
and verify relative-expression/absolute-source `stops`. Complete JSX attribute
spans remain absent in the same `markdown` AST; component key diagnostics cannot
pretend to point to a precise property range. Element/Key-marker spans are real.
`original_source` borrows unchanged input and tests compare original bytes: no
serializer, body rewriter, AST sidecar or saved base is implemented.

The comparison adapter is **not a secure storage schema/compiler**. It does not
validate all property names/types, child models, URLs, assets, geometry ranges,
Leaflet kinds, named-page references, depth/size limits or downstream rendering.
No JS execution occurs; that does not make arbitrary downstream use safe.

## Recommendation and next decisive boundary

1. **Do not say brace identities were abandoned because they are impossible.**
   Their exact examples still parse, but this stock/fork API assigns expression
   semantics and punctuation-dependent failures. No custom `{#id}` extension was
   implemented here. Earlier Rushdown/event-layer alternatives remain separate
   evidence; this experiment does not rerun or supersede them.
2. **For the user's updated direction, prefer the demonstrated native self-closing
   Key candidate over comments.** It is a declarative adapter on good native JSX,
   not custom Markdown grammar. Confirm that spelling/next-block semantics before
   calling it adopted; a paired wrapper would be a different group contract.
3. **Treat the fork as a dependency refresh.** It removes the concrete old-Serde
   failure and needs no Turbopack infrastructure. That may be sufficient reason to
   use it if choosing mdxjs. It is not evidence of better code ownership, lower
   runtime cost, new extensibility or complete source mapping.
4. **Do not ship the false-clean component case.** A narrow, uniform rejection
   boundary for flow JSX interrupting paragraph content could extend the existing
   marker guard before any lowering; this experiment proves marker rejection only,
   not that broader repair. It must cover components as well as annotation syntax.
   Restoring CommonMark code ownership instead requires a parser-boundary change;
   a post-AST identity pass cannot restore bytes already classified differently.
   Do not respond by piling on special marker spellings or claiming fences fix
   already misinterpreted input automatically.

The immediate syntax question is **self-closing next-block Key or a paired
block/group wrapper?** Recommend the self-closing candidate demonstrated here;
the user's updated native-MDX preference is real steering, not comment adoption.
For that direction the remaining decisive engineering gate is fail-closed
treatment of code-shaped flow JSX across **ordinary components too**, rather
than another custom ID grammar. If unchanged CommonMark code semantics are also
required, that instead changes the parser boundary; this fork does not deliver
it. Direct markdown-rs remains the assessor's smaller callback candidate for
removing SWC, but **was not built as a substitute fork result** and does not
inherently fix MDX flow ownership. No additional experiment is silently
commissioned by this report.

## Verification, checkpoints and scope

Final offline commands in this isolated workspace:

```sh
cargo nextest run --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
cargo tree --locked --offline --depth 2 -e normal
```

**51/51 passed, none skipped**: 8 stock controls, 13 shared identity cases,
9 component/data cases, 17 native Key cases and 4 CLI cases. Nextest run ID:
`261d581c-b91f-480f-ad61-278468316f20`. Clippy warnings-denied and formatting checks
passed after the Key extension. The earlier 33-test comparison checkpoint also
passed (`cff7daf8-1391-4b20-ac4d-d6a51ba5f7df`). No runtime/performance benchmark.

Owned coherent checkpoints:

- `28bc475d` — exact Git dependency and successful current-Serde smoke build.
- `1558c3d4` — native grammar/data comparison, shared controls and negative evidence.
- `6265b2e6` — user-steered native Key candidate and 51-test corpus.

Document verification from the repository root:

```sh
cargo run --quiet --offline --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf experiments/12-mdxjs-turbopack-identity/REPORT.md
cargo run --quiet --offline --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check experiments/12-mdxjs-turbopack-identity/README.md
```

Both document checks passed, including report provenance and local destinations.

Initial status showed unfamiliar experiment 10 work; it was left untouched.
Experiment 12's path was absent before creation. Unfinished 10/11 were not read.
Only this new folder was edited/committed. Dependency downloads/source inspection
were setup; final tests/checks run offline. No production tests, Leaflet/PDS writes,
production API calls, root index/tickets/ignore changes, pushes or cleanup occurred.

### Connections

- [The provisional assessment](../../design/authoring-parser-assessment0.gpt6a.md)
  supplies the comparison question; this evidence narrows the fork's actual win
  and responds to the later Key preference without rewriting that assessment.
- [Experiment 09](../09-mdxjs-identity/REPORT.md) supplies the baseline and strict
  literal policy; new code-shaped component evidence limits its comment guard too.
- [The recreation design](../../design/mdx-recreate0.gpt6a.md) remains the downstream
  goal: one source and a transient tree. This parser proof establishes neither its
  Leaflet compiler nor save/native-publication transport.
- [Experiment practice](../README.md) keeps these independent offline receipts
  outside production implementation; earlier backend reports are not generalized
  to unexamined Satteri work.
