---
type: Experiment
title: "Optional brace keys compose with native Satteri authoring"
description: "Native heading support and paragraph-tail limitations, followed by a small AST-guided byte-mask/reparse adapter that composes with unchanged experiment 13 and produces equivalent typed IR, Leaflet facts and fresh text."
resource: /experiments/14-satteri-authoring-attributes/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#xhigh
sources:
  - resource: "opencode://ses_e46bbf7f6ffeLNF9i7yGLd5Nx1"
    title: "Same commissioned Sol XHIGH follow-up: stock attribute controls, AST-guided brace-key adapter, real compiler/plan equivalence and recommendation without changing completed experiment 13"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Completed Satteri compiler/facts/Yrs adoption evidence, preserved unchanged"
  - resource: /experiments/13-satteri-authoring/src/author.rs
    title: "Read and reused the unchanged strict native-Key/component/Markdown compiler through a relative Cargo dependency"
  - resource: /experiments/13-satteri-authoring/fixtures/garden.mdx
    title: "Original native-Key fixture; adapted five declarations and compared with this exact source"
  - resource: /experiments/13-satteri-authoring/src/facts.rs
    title: "Unchanged deterministic entity/fact planner used for output equivalence, not reimplemented"
  - resource: /experiments/13-satteri-authoring/src/text.rs
    title: "Unchanged fresh Yrs encoder and semantic read-back; no new CRDT algorithm"
  - resource: /experiments/07-satteri-identity/tests/attributes.rs
    title: "Read earlier native heading/paragraph attributes and list-targeting observations before new stock controls"
  - resource: /design/authoring-parser-assessment1.gpt6a.md
    title: "Native Key preference and brace form as optional convenience, not a reason to reopen parser selection"
  - resource: /experiments/14-satteri-authoring-attributes/src/adapter.rs
    title: "Executed exact native expression discovery, byte-identity mask, strict compiler reparse, target matching and shared namespace"
  - resource: /experiments/14-satteri-authoring-attributes/tests/stock.rs
    title: "Direct stock parser tests distinguish default MDX, native heading flags and unsupported paragraph tails"
  - resource: /experiments/14-satteri-authoring-attributes/tests/adapter.rs
    title: "Executed supported forms, literal protection, scope/source mapping, diagnostics and full real fixture equivalence"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/lib.rs"
    title: "Read ENABLE_HEADING_ATTRIBUTES, ENABLE_ATTRIBUTES and ENABLE_MDX option definitions"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/firstpass.rs"
    title: "Read native heading attribute/MDX-expression precedence and setext error removal"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/block_attributes.rs"
    title: "Read Markdown block attribute targeting; native tight-list attachment differs from paragraph author identity"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/arena_build.rs"
    title: "Read native expression arena emission and returned MDX errors; executed recovery-node discovery"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/parse.rs"
    title: "Checked public ParserCallbacks: broken-link callback, not a replaceable MDX expression/parser callback"
---

# Optional `{#key}`: useful sugar, explicit adapter

## Verdict

**Yes: both requested brace forms work alongside native `<Key />`, and the
convenience is worth including in the authoring-language design now.** Native Key
remains the default/full-block candidate; braces are optional heading/paragraph
sugar. This does not weaken [experiment 13's Satteri recommendation](../13-satteri-authoring/REPORT.md)
or require a parser fork, replacement Markdown lexer or JS runtime.

Be precise about ownership: **heading attributes are supported natively with a
flag; the complete heading-plus-paragraph convenience here is an adapter.** Native
MDX+attributes does not by itself compile the requested paragraph tail. The
adapter is small enough to be reasonable, but it costs an extra native parse and
depends on tested error-recovery nodes at an exact pinned parser revision.
Production adoption/integration remains uncommissioned.

## Stock behavior proved first

[Four direct stock controls](tests/stock.rs) ran before implementation:

| Stock configuration | Executed observation |
| --- | --- |
| `MDX_OPTIONS` | Both `{#introduction}` and `{#central-argument}` become native MDX expression nodes and report expression errors; their exact values/ranges remain in the recovered arena |
| MDX + `ENABLE_HEADING_ATTRIBUTES` | The formatted/link-bearing heading works; native `hProperties.id` holds the ID; the paragraph tail still errors as a flow expression |
| MDX + `ENABLE_ATTRIBUTES` | Same useful heading result, but still not a paragraph-tail solution |
| Markdown-only `ENABLE_ATTRIBUTES` | Tight-list tail attaches to **ListItem**, not the paragraph identity desired here |
| Native code and component attribute strings | No body-key expression is produced inside tested inline code, fences, Run strings or Poll option JSON |

This uses the same public parser as 13/07/10, not a second Satteri backend.
Inspected `ParserCallbacks` has broken-link handling, not a public expression
grammar override. Post-parse plugins can transform arenas but do not themselves
make original malformed MDX valid. No plugin runner or grammar hook is claimed.

## The actual adapter seam

[adapter.rs](src/adapter.rs) deliberately reuses the unchanged 13 compiler:

1. Parse original bytes with Satteri's ordinary `MDX_OPTIONS`. Treat its recovered
   arena as **discovery only**, not a validated document.
2. Select only native MDX expression nodes whose exact source/value is `{#name}`.
   Names use the same case-sensitive `[A-Za-z][A-Za-z0-9_.-]*` alphabet as Key.
   Identify the actual Heading parent or immediate preceding Paragraph sibling
   in the same native container. No nearest-text/backwards search.
3. Store each annotation's exact original range and native target start/kind.
   Replace **only those ASCII annotation bytes** with same-length spaces in a
   transient parser-input copy. No lines, prefixes or arbitrary text are removed;
   no synthetic Key tags or length-changing insertion is used.
4. Run 13's unmodified strict compiler on that amended source. All remaining
   syntax, component/child schemas and reference validation must pass. Unrelated
   parse errors, arbitrary JS expressions and unknown components still fail.
5. Match each final typed block by **the exact original target start and kind**.
   Require one match, attach its author key, enforce the shared Key/Poll namespace
   and reject dual annotation on a block. Rebind the returned Document's borrowed
   source to the untouched original bytes; all semantic IR content is owned.

This is **AST-guided preprocessing plus typed binding**, not “native paragraph
attributes.” Uniform handling avoids mixing native HTML-ID metadata, its broader
attribute grammar and its different list-target semantics into author identity.
No archive/registry source was patched. Two native parses, one transient source
copy and the annotation table are the explicit implementation cost. Native-Key-only
input also takes this wrapper's discovery pass; no performance claim is made.

## Supported contract

```mdx
## Introduction {#introduction}

An ordinary paragraph with **formatting**.
{#central-argument}
```

- **Heading:** one final brace key after whitespace. Formatted headings, links and
  the tested setext form retain their semantic children. Native Key remains valid.
- **Paragraph:** a standalone suffix on its immediately following line, **no blank
  before it**. Follow with a blank line, EOF or container closure. An immediately
  following content line diagnoses `attribute-not-tail` rather than guessing.
- **Scopes:** in a quote repeat `>`; in a list indent into the same item. The key
  names that paragraph, not the quote/list/item. Named Page children follow the same
  rule. Missing container prefixes, blanks and closed containers cannot reach back.
- **Identity:** native Key, brace keys and component keys share one namespace.
  Duplicate names and both forms on one block fail; neither spelling wins. These
  are internal author keys, not HTML anchors, remote UUIDs or CRDT character IDs.
- **Literals:** native fenced/single-line inline code, escaped opening braces,
  Run literal strings and component JSON remain literal. Mixed real keys plus
  `{#not-a-key}` string data are explicitly tested. Plain Markdown quotation marks
  do not create an MDX string literal; escape literal opening braces or use Run.
- **Outside the slice:** inline/same-line paragraph suffixes, classes, arbitrary
  attributes, several keys on one heading, spaced `{ #name }` and malformed names
  fail. Unterminated syntax remains a native parse error. Full native attribute
  grammar is intentionally not imported.

Every masked byte retains its original offset; line endings and Unicode bytes
outside ASCII keys remain unchanged. BOM/Unicode/CRLF tests check byte identity,
precise annotation slices and actual paragraph ranges. Whole block ranges remain
native semantic spans, not a lossless CST/editing contract. The discovery target
range is explicitly labelled separately from the final typed block range.
Discovery-only errors are also rebased to original absolute bytes. The raw
`stock` inspection instead labels its native BOM base explicitly.

## Real compiler integration, not just AST output

[garden-attributes.mdx](fixtures/garden-attributes.mdx) adapts the actual
[13 native-Key fixture](../13-satteri-authoring/fixtures/garden.mdx): five brace
keys replace five native markers; the code block keeps native Key and Poll keeps
its component key. Ordinary prose, marks, link, hard break, Unicode, list order,
quote, fenced tabs, named Page/PageLink and exact Run/Poll values are unchanged.

`compare` executes both compilers and verifies:

- Equivalent typed Document, excluding only source ranges and marker counts.
- The same seven author names, target entities, page references, memberships and
  literal values in the Leaflet plan.
- **2 pages, 17 entities, 56 facts and 9 fresh Yrs text payloads**, with each payload
  decoded and compared semantically. Both builds inject the same generation/IDs;
  source spelling does not sneak extra entities or path changes into the result.
- Equivalent full plan after normalizing only source attribution and Yrs encoding
  representation bytes. Text/marks/breaks/order/keys are not erased for comparison.
- Successful staging/verification through 13's offline model, with original source
  bytes unchanged and no companion file emitted.

No new text encoder, fact planner or staging model was copied/reimplemented.
13's independent Yjs/ProseMirror receipt remains its evidence; that suite and all
other previous experiment/production suites were **not rerun**. This follow-up
adds no new claim about deployed Leaflet, publication, interaction continuity or
CRDT synchronization. Everything remains offline and in-memory.

## Costs and recommendation boundary

The new adapter is one focused Rust module, **including** comparison helpers and
source/binding types, rather than a new Markdown parser. Its real maintenance
obligation is dependence on the pinned parser retaining useful expression nodes
after validation errors. That recovery shape is executed evidence, not an assumed
stable upstream extension contract. Version updates must rerun these positive and
literal-boundary controls. Strict final compilation and target start/kind checks
prevent accepting the discovery tree as valid output.

Keep ordinary MDX limitations in proportion. The unusual multiline single-backtick
brace control is rejected because content immediately follows the suffix; its
CommonMark code structure is not restored. A flow Poll in an apparent multiline
inline-code span keeps 13's known active-MDX interpretation. Use fenced multiline
examples; this follow-up does not add a universal lexer or claim unchanged
CommonMark compatibility. Input-size/depth hardening, performance, full schema
coverage and production feature integration remain unproved as in 13.

**Include the optional convenience when the production authoring compiler is
commissioned**, with the small contract above and Key as the default/general
escape hatch. It does not justify delaying the viable Satteri route or forcing
all of Satteri's broader native attribute syntax into MDX. If future recovery
behavior becomes costly to maintain, dropping paragraph sugar would not invalidate
Satteri or native Key; native headings alone remain a demonstrated fallback.

## Reproduction, exact source and receipts

[Cargo.toml](Cargo.toml) has its own `[workspace]`. Its explicit relative dependency
on 13 is intentional, not a hidden import or machine-specific path: reproduce
this repository with both directories. The reused source is 13 at
`24b6dfc7c38fd3d190c83d9b296f1b453b9e8e50`; actual runtime/fixture hashes and the
new lock checksum are in [UPSTREAM.json](UPSTREAM.json). A path dependency is **not
a Cargo Git pin**; the receipt records the checkout used.

Satteri parser **0.6.7**, AST **0.6.3**, arena **0.4.3** and core published revision
**`7d1297b7e4551a1c5124fa07bd2d1d05f66236e6`** are unchanged. Archive HEAD was verified
again. Direct Serde **1.0.229**, serde_json **1.0.154**, anyhow **1.0.103** and all
transitives have the same package identities/sources/checksums as 13; the only new
lock package identity is this experiment. Its inherited Yrs pin remains **0.28.1**
at `e0b476b0ce613453c70d0bb477b955808aa79a07`. Dependencies were added through Cargo,
only in this new directory. Rust/Cargo 1.101 nightly on Linux x86-64 were used;
no new MSRV/cross-target claim is made.

See [README.md](README.md). After `cargo fetch --locked`, from this directory:

```sh
cargo run --locked --offline -- compare fixtures/garden-attributes.mdx ../13-satteri-authoring/fixtures/garden.mdx
cargo run --locked --offline -- stock fixtures/garden-attributes.mdx
cargo run --locked --offline -- check fixtures/rejected.mdx # expected exit 2, no plan
cargo nextest run --locked --offline --no-fail-fast
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

Results: **21/21 focused tests passed, zero skipped**, nextest run
`444cae52-8913-485b-b6a6-def88e87d0fa`: 4 direct stock controls, 15 adapter/integration
tests and 2 actual CLI tests. Formatting and Clippy with warnings denied passed.
Passing negative controls record native limitations, not fulfilled compatibility
invariants. The actual `compare` executable emitted all equivalence/verification
flags true. Semantic diagnostics exit 2 with original-source spans and no stdout
plan; `stock` is explicitly diagnostic-bearing inspection, never valid output.

Checkpoints, explicit new-directory paths only:

- `2172f01d` — direct stock behavior and literal controls before adaptation.
- `28201d19` — adapter, real fixture equivalence and all 20 passing focused tests.
- `9883a485` — original-byte discovery error rebasing/BOM labelling and the final
  21-test source-position control.

Document checks from the repository root:

```sh
cargo run --locked --offline --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf experiments/14-satteri-authoring-attributes/REPORT.md
cargo run --locked --offline --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check experiments/14-satteri-authoring-attributes/README.md
```

Both passed: report **1 document, 8 links, 16 sources** (including actual native
session provenance); README **1 document, 3 links**. `git diff --check` passed;
all recorded reused 13 source/fixture hashes and the 14 lock checksum matched.

Experiment 13 and all prior evidence are preserved. No production/root manifest,
lock, ignore, index or beads changes, upstream edits, subagents, credentials or
live requests/writes were involved. Root indexes and any promotion remain with
the commissioning parent; this session owns only the new experiment directory.
