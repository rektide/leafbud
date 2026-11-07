---
type: Experiment
title: "Rushdown extensions for optional Markdown block identities"
description: "Offline executable proof of attribute and MDX-shaped comment author keys, container-local binding, preserved inline structure, source positions, and the boundary between CommonMark and MDX."
resource: /experiments/06-rushdown-identity/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-07
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4829b52cffeyefZc8je6I5xb4"
    title: "Commissioned Sol HIGH offline experiment: source inspection, attribute phase, comment phase, executable controls and implementability assessment"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Source-authoritative local authoring context; this experiment follows the later correction to keep ordinary prose in Markdown"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Read prior interoperability scope and receipts; Yrs work was not rerun or expanded"
  - resource: /experiments/06-rushdown-identity/src/extension.rs
    title: "Executed Rushdown ParserExtension, block/inline hooks, custom marker nodes and AST binding"
  - resource: /experiments/06-rushdown-identity/src/projection.rs
    title: "Typed transient block/inline projection and honest source-position exposure"
  - resource: /experiments/06-rushdown-identity/tests/attributes.rs
    title: "16 attribute syntax tests"
  - resource: /experiments/06-rushdown-identity/tests/comments.rs
    title: "15 comment and combined-mode tests"
  - resource: /experiments/06-rushdown-identity/tests/substrate.rs
    title: "8 controls distinguishing upstream behavior, failed approaches, positions and missing MDX semantics"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/README.md"
    title: "Locally read authoritative repository, extension documentation, and CommonMark scope"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/src/parser/mod.rs"
    title: "Inspected registration constructors, parser ordering, BlockParser, InlineParser, AstTransformer, and source-position assignment"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/tests/extension.rs"
    title: "Inspected UserMention custom node/inline parser and Admonition transformer patterns"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/src/parser/heading.rs"
    title: "Inspected ATX/setext parsing, built-in heading attributes and auto IDs"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/src/parser/code_span.rs"
    title: "Inspected multiline code ownership before choosing inline suffix recognition"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/src/ast.rs"
    title: "Inspected arena extension nodes, attributes, node starts, source segments and Text indices"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/src/text.rs"
    title: "Inspected BasicReader/BlockReader and byte-based Segment/Index APIs, including padding and force_newline"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/src/parser/link_ref.rs"
    title: "Inspected reference-definition sibling insertion to avoid skipping invisible intervening content"
  - resource: "https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/src/renderer/html.rs"
    title: "Inspected text rendering, raw HTML policy and attribute filtering; projection deliberately retains raw Text escapes"
---

# Optional block identities using Rushdown extensions

## Verdict

**Implementable without a Rushdown fork or a regex preprocessor. Prefer the
`{#introduction}` family for selected ordinary prose, following the user's light
lean.** Both requested syntaxes work through actual Rushdown extension hooks;
39 offline tests pass. No final production syntax is accepted by this experiment.

The important boundary: **Rushdown is a good extensible Markdown substrate, not
a ready-made MDX parser.** A narrow literal directive is straightforward; literal
components with JSON-shaped properties need additional grammar and an explicit
handoff. Do not treat this identity proof as a complete authoring compiler.

This experiment does not touch production crates, manifests or lockfiles, call
Leaflet, create partner documents, implement synchronization, or rewrite source.
Dependency/source downloads were setup traffic; execution and checks were offline.

## Exact dependency and source evidence

| Item | Pin / observation |
| --- | --- |
| Crate | **`rushdown` 0.18.0**, `rushdown = "=0.18.0"` |
| Authoritative repository | `https://github.com/yuin/rushdown` (not a crate named `rushdown-rs`) |
| Published source revision | **`e5eb4e4446541ea0ed53111c1b37e779283ff57c`** |
| Registry archive checksum | `57358ca9b61ea373ec6bf1c9e916f521f19a919587d9179624d47c40da2ffc5d` |
| Inspected local clone | `~/archive/yuin/rushdown`, same HEAD as the published `.cargo_vcs_info.json` |
| Enabled features | Defaults: `std`, `html-entities`; no MDX feature exists in the inspected manifest |
| Other direct dependencies | `serde` 1.0.229 with derive; `serde_json` 1.0.154, resolved by the isolated lockfile |
| Runtime | `rustc 1.101.0-nightly (822182db4 2026-10-13)`; nextest 0.9.122 |

No archived checkout was initially present. `cargo search` / `cargo info
rushdown@0.18.0` established the repository, then it was cloned to the standard
archive location. No `AGENTS.md` or `llms.txt` was found there. Source inspection
used the pinned checkout and registry source, not guessed API names.

Dependencies were added with Cargo tooling. [Cargo.toml](Cargo.toml) has its own
`[workspace]`; [Cargo.lock](Cargo.lock) pins transitive dependencies. The registry
pin plus checksum identifies the executable dependency; the Git revision binds
the inspected upstream evidence.

### Extension architecture actually exercised

Upstream's [extension documentation](https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/README.md#extending-rushdown)
and [custom-node tests](https://github.com/yuin/rushdown/blob/e5eb4e4446541ea0ed53111c1b37e779283ff57c/tests/extension.rs)
provide the pattern. [Our extension](src/extension.rs) registers constructors
inside `parser_extension(...)`, supplied to `Parser::with_extensions(...)`:

| Hook | Actual responsibility |
| --- | --- |
| `add_block_parser`, priority `PRIORITY_PARAGRAPH - 10` | `LineMarkerParser`: preceding comment directives at block boundaries and orphan attribute lines; `{` trigger; **cannot interrupt an open paragraph** |
| `add_inline_parser`, priority `PRIORITY_EMPHASIS + 10` | `SuffixParser`: heading suffixes and entire final paragraph lines; code-span parsing has already claimed its contents; misplaced directives diagnose |
| `KindData::Extension`, `NodeKind`, `PrettyPrint` | Temporary typed marker nodes, leaf-block or inline as appropriate; exact annotation byte ranges travel with them |
| `add_ast_transformer`, priority 100 | `BindKeys`: resolve actual parents/siblings, reject invalid/duplicate/colliding keys, attach internal metadata and remove marker nodes before rendering |
| `Node::attributes_mut()` | Store **`springleaf-key`**, not `id`, on the actual target block; expose the same identity on the typed projection |

Upstream `parser/mod.rs` runs AST transformers **after** block parsing and inline
parsing. Suffixes attach to their actual containing heading/paragraph; directives
attach to the immediate next sibling. No global line substitution, second Markdown
parse or search for a vaguely matching block is involved.

Rushdown already has `Options { attributes: true, .. }` for headings. The control
test proves it produces `<h2 id="introduction">` but leaves paragraph `{#body}`
visible. It is not the author-key policy we want. Both built-in heading attributes
and automatic slug IDs stay off here. Rendered HTML deliberately has **no generated
anchor IDs**. Stable author identity, HTML navigation anchors, block array indices
and remote/Yjs identities remain separate concepts.

## Phase A — attributes, implemented first

```md
## Introduction {#introduction}

This is an ordinary paragraph with **formatting**.
{#central-argument}
```

Rules selected and tested, rather than claimed as a universal attribute standard:

- Keys match `[A-Za-z][A-Za-z0-9_.-]*`, are case-sensitive and document-wide unique.
  Unicode prose is supported; Unicode key names are intentionally rejected.
- ATX headings accept a whitespace-separated suffix at the end of their logical
  content. Optional closing hashes go **after** the suffix:
  `## Introduction {#introduction} ##`.
- Setext headings accept the suffix on the content line, not the underline:
  `Introduction {#introduction}` followed by `---` or `===`.
- Paragraphs require a **whole final logical line** `{#central-argument}`, with
  no intervening blank line. Same-line prose `Paragraph {#id}` remains literal.
  A marker in the middle of a multiline paragraph is an error, not an instruction
  to split the paragraph or search backwards.
- A blank line before the attribute creates an orphan and diagnoses. A standalone
  attribute after a heading is not an alternative spelling for its heading suffix.
  Reference definitions alone do not count as preceding prose after Rushdown strips
  them from a paragraph; a regression test rejects that otherwise-empty target.
- Inside a quote/list, prose keys belong to the **paragraph**, not the containing
  quote, list or list item. Nested examples are tested. Container-level attribute
  syntax and direct list-item keys are not implemented.
- Annotation continuations must be explicit: all quote levels need `>` prefixes;
  within lists the annotation's column must reach at least the paragraph's initial
  content column (tabs count at width four). This intentionally strict check
  rejects lazy unindented annotation continuations. It can reject unusual but
  valid CommonMark indentation; it is an authoring rule, not a general container
  syntax recognizer.
- Escape the opening brace for a literal reserved marker. Ordinary `{braces}`,
  completed attribute-shaped text within a heading's emphasis/link label, and
  inline/fenced/indented code remain content.

**One attribute vocabulary is viable, but not one identical placement rule.**
Both working forms share the inline parser; headings use a suffix and paragraphs
use a final whole line. The block hook catches otherwise orphaned marker lines.
This avoids broad “attributes belong to whatever was before them” semantics.

### Failed approach retained as an executable control

A tempting block parser with `can_interrupt_paragraph() == true` breaks this:

````md
Code `first
{#literal}
last`.
````

The block stage cannot know that the line belongs to an inline code span.
`eager_block_interruption_really_destroys_multiline_inline_code` injects precisely
that wrong extension and proves it produces a thematic break instead of a code
span. The real extension produces the same HTML as baseline CommonMark for this
input. This is why the final paragraph-tail hook is inline, not interrupting block
syntax. The negative control uses Rushdown's extension API too.

## Phase B — preceding comment annotations

```mdx
{/* springleaf:key introduction */}
## Introduction

{/* springleaf:key central-argument */}
This is an ordinary paragraph with **formatting**.
```

Baseline Rushdown emits the first annotation as **visible paragraph text**.
The experiment implements the requested literal spelling through the block
extension, not a substituted HTML-comment spelling and not an MDX runtime.

- Only the exact single-line `{/* springleaf:key NAME */}` directive is accepted
  (trailing whitespace allowed). The reserved `{/* springleaf:` namespace gives
  loud errors for wrong names, malformed keys or unterminated/multiline directives.
  Other comment-like text and `{expression}` are ordinary CommonMark content.
- A preceding directive must begin a block. If earlier prose is still an open
  paragraph, insert a blank line before the directive; the inline hook diagnoses
  the standalone directive otherwise. This preserves multiline code instead of
  making comments interrupt prose invisibly.
- Blank lines **between** directive and target are allowed. It binds the immediate
  next supported sibling in **its own container**, never a later arbitrary block.
  Supported targets: paragraph, heading, list, quote, code block, thematic break.
- A directive before a list keys the list. `- {/* springleaf:key first */}` followed
  by indented prose keys the paragraph inside that item, **not the item**. There is
  no direct list-item key spelling in this experiment.
- An annotation at the end of an empty quote/item is unbound; it cannot escape to
  root prose or another item. Consecutive directives do not silently merge: the
  first encounters another directive and diagnoses.
- HTML nodes, link-reference definitions and other unsupported siblings block
  attachment; the binder does not skip them. **This is AST-kind-based, not a
  complete MDX-component detector**; the JSON-property counterexample below matters.
- All malformed, unbound, duplicate and collision diagnostics include annotation
  byte ranges. The executable prints inspectable partial output but exits **2**;
  a future compiler must refuse any diagnostic-bearing result.

### Independent and combined modes

`plain`, `attributes`, `comments`, `both` are separately runnable. Disabled syntax
retains its baseline Markdown interpretation. In combined mode, either syntax can
key separate blocks. Two annotations on the same block produce `key-collision`
for both, even if their strings are equal; **neither wins and the block gets no
key**. Reusing a key on different blocks diagnoses both as `duplicate-key`.
There is no hidden “comments override suffix” policy.

## Projection and source positions

The [executable](src/main.rs) prints transient JSON, not a second authoritative
document. For the first block of [attributes.md](fixtures/attributes.md), the
projection includes:

```json
{
  "kind": "Heading",
  "key": "introduction",
  "key_span": { "start": 16, "end": 31 },
  "start": 0,
  "content_segments": [
    { "start": 3, "end": 31, "padding": 0, "force_newline": false }
  ],
  "content": "Introduction"
}
```

`inlines` additionally retains Text/Strong/Emphasis/Link/CodeSpan structure, link
destinations, line-break flags and available text indices. A later Leaflet compiler
can consume the key plus actual inline/container structure, without rendering to
HTML and reparsing it. The convenience `content` string is an inspection flattening,
**not a finished semantic-text compiler**: raw Text leaves still contain Markdown
escapes/entities. A downstream compiler must decode those correctly and preserve
code's distinct semantics. Heading levels, list ordering and complete Leaflet facet
lowering are also outside this minimal projection; the upstream typed AST retains
the relevant node data.

Available versus unavailable positions are deliberately distinguished:

| Data | Actual availability |
| --- | --- |
| Annotation | Exact half-open UTF-8 byte range captured by our parser hook; excludes quote/list prefixes and trailing line ending |
| Block start | Rushdown `Node::pos()`; **13/13** blocks in the attribute fixture have starts |
| Payload segments | `Block::source()` for prose/headings, `CodeBlock`/`HtmlBlock` `Lines::Segments` for raw payloads; **9/13** fixture blocks have payload segments |
| Containers | Lists, list items and quotes have starts but no own payload segments in this fixture; child segments do not constitute full container ranges |
| Key positions | **5/5** keyed fixture blocks carry exact annotation spans; no key span is fabricated for an unkeyed block |
| Text leaves | Exposed `Text::index()` start/end when retained; trimming separator whitespace preserves that index |
| General inline/block end | No universal concrete-syntax end range exposed by this probe; code-span normalized text and its start are exposed, not invented delimiter-end spans |

Payload segments can still include the original annotation bytes, because the
extension removes marker **AST nodes**, not source. They also omit heading markers,
setext underlines and fence delimiters; `padding` and `force_newline` can describe
virtualized whitespace. Treating a segment hull as a full rewriter range would be
incorrect. CRLF/Unicode annotation slicing and byte-for-byte source retention are
tested for both syntax families.

**No-op source preservation is trivial here:** parsing borrows the source, the
result retains its original bytes, and there is no source serializer or write
path. Tests compare `original_source().as_bytes()` directly. This is not proof of
round-trip reconstruction after edits, and none is needed for source-authoritative
compilation. Moving keyed blocks changes positions while retaining author keys;
both families have deterministic reorder tests.

## MDX/component boundary — demonstrated limitation

[components.mdx](fixtures/components.mdx) and the substrate tests show three
different CommonMark interpretations:

| Input | Observed Rushdown node |
| --- | --- |
| `<Poll key="topic" options={["A", "B"]} />` | **Paragraph of text**, because quoted JSON property contents fail ordinary HTML attribute grammar |
| `<Poll key="topic" />` | **HtmlBlock** |
| `Text with <Image src="x" /> and {expression}.` | Paragraph with **RawHtml** plus ordinary expression-shaped text |

The first control initially expected the JSON-shaped component to be an HTML
block and failed; the corrected test preserves the observed distinction. The
default HTML renderer escapes the first form and suppresses raw HTML in the other
forms; it never executes a component.

A comment before a JSON-shaped `<Poll ...>` can therefore bind a **paragraph**
without diagnosing: the identity extension does not understand that it was meant
as a component. This is explicitly tested, not hidden by a claim that all components
are rejected. A production authoring dialect needs a component parser extension
that claims its syntax before paragraph/HTML fallback, or a deliberate outer
grammar that composes Markdown regions. Recognition order, braces, multiline
properties and nesting would then need their own tests. **That is hypothetical
follow-on work, not proved here.**

If a full MDX parser is later used, `{#id}` competes with expression syntax, while
`{/* ... */}` already resembles native MDX comments. Conversely, Rushdown alone
needs only these small exact recognizers. This experiment says nothing about
arbitrary JSX/JavaScript, imports, evaluation, component children or general MDX
compatibility. GFM tables/tasks, frontmatter integration, extension composition
with third-party plugins, fuzzing and performance were not tested.

## Recommendation and implementability judgment

1. **Use the attribute family for optional selected prose IDs** if ordinary
   Markdown plus a restricted rich-component dialect remains the destination.
   It is compact, keeps identity next to the prose, matches the user's light
   preference, and needs no `<Paragraph>`/`<Heading>` wrappers. Keys remain
   optional; ordinary blocks do not need IDs just to compile fresh content.
2. **Keep the final-line paragraph rule explicit.** A heading suffix and a
   paragraph's own attribute line share a familiar vocabulary without promising
   uniform attachment to lists/items/fences. Avoid magical backwards scanning.
3. **Comments are feasible, not technically required.** Their uniform preceding
   form handles container targets neatly and may compose more naturally with a
   future full MDX parser, but costs a line and has the block-boundary rule shown
   above. Do not maintain two production spellings merely because both probes work.
4. **Rushdown fits the Markdown half well.** Public constructor injection, typed
   custom nodes, inlines and post-parse transformation are sufficient and coherent.
   The multiline-code trap is manageable with inline recognition. Pin the version
   and retain the corpus: these are pre-1.0 public APIs, not a stability promise.
5. **Do not promote this whole probe as production code.** Container indentation
   policy is intentionally strict; binding/projection use simple recursive walks
   and some linear searches; error output is inspectable partial data; semantic
   text/component lowering is unfinished. No throughput, pathological-depth or
   broad Markdown-plugin guarantees are made.

Future remote identity allocation could use these author keys as one input, but
no such allocation or continuity policy is implemented or recommended as mandatory.
The proven result is only stable optional author IDs carried into a typed parse.

## Reproduction and check receipts

From `experiments/06-rushdown-identity/`:

```sh
# Initial setup only, if dependencies are not cached:
cargo fetch --locked

# All of these runs are offline:
cargo run --locked --offline -- attributes fixtures/attributes.md
cargo run --locked --offline -- comments fixtures/comments.md
cargo run --locked --offline -- both fixtures/combined.md
cargo run --locked --offline -- plain fixtures/comments.md
cargo run --locked --offline -- both fixtures/components.mdx
cargo run --locked --offline -- both fixtures/collision.md  # intentional exit 2
cargo nextest run --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

Recorded Rust check results: **39/39 tests passed**, zero skipped (16 attributes,
15 comment/combined, 8 substrate controls); nextest run
`4d6d9f75-45c3-4627-896c-747e27ceba9a`. Clippy passed with warnings denied; formatting
check passed. The positive attribute executable returned clean metadata and HTML;
the collision executable emitted two `key-collision` diagnostics and exited 2.

Owned code stages were checkpointed separately:

- `03907ae2` — attribute phase, initially 12 passing tests.
- `da737eed` — comment/combined phase, 27 passing tests.
- `27a8f483` — final position exposure and source-derived controls, 38 passing tests.
- `9bec2a7e` — reject reference-definition-only paragraph tails, 39 passing tests.

Document verification from the repository root:

```sh
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf experiments/06-rushdown-identity/REPORT.md
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check experiments/06-rushdown-identity/README.md experiments/README.md
```

Document checks passed: report **1 document, 12 links, 17 sources**, with OKF
provenance; READMEs **2 documents, 3 links**. The executable smoke matrix returned
5 attribute keys, 7 comment keys and 4 combined keys, all without diagnostics;
plain/component controls returned no keys; the collision fixture returned no keys,
two diagnostics and exit 2.

### Connections

- [The source-authoritative authoring recommendation](../../design/mdx-recreate0.gpt6a.md)
  supplies the build-fresh-content direction. This experiment refines its prose
  surface in response to the later correction: ordinary prose stays Markdown;
  optional identity does not require literal wrapper components.
- [The Yrs experiment](../05-yrs-conversion/REPORT.md) addresses a different layer:
  encoding/interoperability. These author keys do not identify Yjs characters,
  establish CRDT continuity, or reopen bidirectional synchronization as a goal.
- [Experiment entry point](README.md) gives the small runnable surface;
  [the experiment directory guide](../README.md) retains the offline/no-write
  evidence boundary shared with the earlier protocol probes.
