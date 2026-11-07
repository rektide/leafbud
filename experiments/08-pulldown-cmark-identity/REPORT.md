---
type: Experiment
title: "pulldown-cmark offset events for optional author identities"
description: "Offline proof of heading/paragraph author keys using parser-owned events, with native heading-attribute controls, tight-list normalization, source offsets and an explicit MDX/component boundary."
resource: /experiments/08-pulldown-cmark-identity/REPORT.md
status: verified-experiment
generated:
  at: 2026-11-07
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4812759effeZjqLN3fwcfRD5B"
    title: "Commissioned independent pulldown-cmark experiment: pinned source inspection, event-layer implementation, offline tests and comparison against the preceding Rushdown experiment"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Source-authoritative local authoring and data-only rich-component purpose; no remote implementation attempted"
  - resource: /experiments/README.md
    title: "Experiment evidence and offline/no-production-write operating boundary"
  - resource: /experiments/06-rushdown-identity/REPORT.md
    title: "Read preceding 39-test Rushdown identity experiment, architectural findings and MDX limitation"
  - resource: /experiments/06-rushdown-identity/tests/attributes.rs
    title: "Read earlier syntax, nesting, malformed-key and source-position tests as an overlapping corpus reference"
  - resource: /experiments/06-rushdown-identity/tests/substrate.rs
    title: "Read preceding native controls and multiline-code/component counterexamples"
  - resource: /experiments/08-pulldown-cmark-identity/src/lib.rs
    title: "Executed raw event capture, transient typed projection, explicit tight-paragraph normalization and author-key transformation"
  - resource: /experiments/08-pulldown-cmark-identity/tests/identity.rs
    title: "20 offline identity, container, literal, diagnostic and offset tests"
  - resource: /experiments/08-pulldown-cmark-identity/tests/substrate.rs
    title: "14 native-library and rich-component boundary controls"
  - resource: /experiments/08-pulldown-cmark-identity/tests/cli.rs
    title: "3 executable JSON/output/exit-status tests"
  - resource: "https://github.com/raphlinus/pulldown-cmark/blob/014b61d38d235f382935bb676bab872b7680afa2/README.md"
    title: "Read pinned upstream CommonMark/event-transformation/offset architecture and examples"
  - resource: "https://github.com/raphlinus/pulldown-cmark/blob/014b61d38d235f382935bb676bab872b7680afa2/pulldown-cmark/src/lib.rs"
    title: "Inspected typed Tag/Event, heading fields, options and serde/static event APIs in the published source"
  - resource: "https://github.com/raphlinus/pulldown-cmark/blob/014b61d38d235f382935bb676bab872b7680afa2/pulldown-cmark/src/parse.rs"
    title: "Inspected public constructors, OffsetIter range production, omitted tight-paragraph events and event conversion"
  - resource: "https://github.com/raphlinus/pulldown-cmark/blob/014b61d38d235f382935bb676bab872b7680afa2/pulldown-cmark/src/firstpass.rs"
    title: "Inspected ATX heading and attribute extraction, last-ID-wins parsing and source-level restrictions"
  - resource: "https://github.com/raphlinus/pulldown-cmark/blob/014b61d38d235f382935bb676bab872b7680afa2/pulldown-cmark/src/utils.rs"
    title: "Read native text-merge iterator behavior; experiment instead preserves untouched unmerged events"
  - resource: "https://github.com/raphlinus/pulldown-cmark/blob/014b61d38d235f382935bb676bab872b7680afa2/pulldown-cmark/examples/string-to-string.rs"
    title: "Read pinned new_ext/options/HTML iterator example"
---

# pulldown-cmark author-identity experiment

## Verdict

**Conditional fit for ordinary Markdown plus optional internal author keys;
not a parser for the proposed MDX-shaped rich components.** The preferred
`{#id}` family is implementable without a fork, source preprocessing, or prose
wrappers. This experiment implements attributes, not comment identities.

The public event/offset interface is sufficient. It does not need Rushdown-style
parser hooks to be useful. However, simply enabling native heading attributes is
**not** our author-key policy: they feed HTML IDs, accept broader metadata, choose
the last ID silently, do not handle paragraphs, and differ on escapes and closing
hash placement. The implemented dialect therefore transforms **plain CommonMark
events**, keeping native heading attributes as a separately executable control.

**37 offline tests pass**: 20 identity tests, 14 substrate controls and 3 CLI tests.
No production crates/manifests/lockfile, experiment index or parallel experiment
were changed. No Leaflet calls, CRDT operations, persistent companion document or
source rewriter exist here. Dependency and source downloads were setup traffic.

## Exact substrate and architecture

| Item | Pin / observation |
| --- | --- |
| Crate | `pulldown-cmark = "=0.13.6"`, defaults plus `serde` |
| Repository from Cargo metadata | `https://github.com/raphlinus/pulldown-cmark` (current repository links use `pulldown-cmark/pulldown-cmark`) |
| Published `.cargo_vcs_info.json` revision | `014b61d38d235f382935bb676bab872b7680afa2`, crate path `pulldown-cmark` |
| Registry checksum | `3db2be46835f013dd3f4fa7f5f7402c537787b54ebb6fe2b94bbdd7e2669bb7b` |
| Local archive | `~/archive/raphlinus/pulldown-cmark`; initially absent, cloned after `cargo info` established identity |
| Other direct dependencies | `serde` 1.0.229 with derive; `serde_json` 1.0.154; exact resolution in local lockfile |
| Runtime | rustc 1.101.0-nightly (`822182db4`, 2026-10-13); nextest 0.9.122 |

The clone's initial HEAD was newer (`f7c6deaa2c75c3ecf04ce04711dd5882d3e29617`).
Executable/source claims here bind the **published revision**, not that moving
HEAD: implementation internals were read in the 0.13.6 registry source; README and
example were also read using `git show` at the published revision. No archive
`AGENTS.md` or `llms.txt` was found. [Cargo.toml](Cargo.toml) owns an isolated
`[workspace]`; dependencies were added with Cargo tooling and [Cargo.lock](Cargo.lock)
is confined to this experiment.

Relevant actual APIs and boundaries:

- `Parser::new_ext(source, Options::empty()).into_offset_iter()` yields typed
  `(Event, Range<usize>)`. `Event::into_static()` makes the inspection projection
  owned. `Tag::Heading` retains heading level; `Tag::List` retains ordered start;
  `Tag::Link` retains destination/title/link kind. These are not HTML reparses.
- `Options::ENABLE_HEADING_ATTRIBUTES` and `Tag::Heading { id, classes, attrs }`
  implement native heading metadata, exercised in `native` mode only.
- Public constructors/options and the broken-link callback do not provide custom
  block/inline parser injection. The intended extension seam used here is the
  **output iterator**, as upstream's own transformation examples describe.
- `OffsetIter::next` explicitly suppresses `TightParagraph` Start/End events.
  Treating the nearest surviving `Item` tag as the prose block would key the wrong
  object. We group direct inline runs within each item into transient Paragraph
  nodes, labelled `derived-tight-inline-hull`. They are not invented native nodes.
  Their virtual tags are omitted when regenerating HTML, preserving tight-list
  rendering. Loose-list paragraphs retain upstream tags/ranges.
- Consecutive Text events are allowed. The MDX-shaped comment arrives as separate
  `"{/"`, `"*"`, content, `"*"`, `"/}"` events. The first unsupported-comment test
  failed because it assumed the directive occupied one event; the correction
  checks a parser-owned text opening against original source. No text-merge claim
  or regex preprocessing is hidden in this correction.

## Chosen identity dialect

```md
## Introduction {#introduction}

This is an ordinary paragraph with **formatting**.
{#central-argument}
```

[The implementation](src/lib.rs) captures untouched `raw_events`, builds a small
event tree, normalizes tight paragraphs, recognizes metadata only in the relevant
prose events, validates/binds keys and projects typed output. **Source is never
modified or reparsed.** This is our event-layer dialect, not native paragraph
attribute support and not a native parser plugin.

Rules actually selected and tested:

- Identity is optional, internal metadata on the actual Heading/Paragraph node.
  It is separate from public HTML anchors, remote block IDs, Yjs identities and
  array positions. Projected heading `id` stays `None`; HTML gets no author ID.
- Keys match `[A-Za-z][A-Za-z0-9_.-]*`, case-sensitive and document-wide unique.
  Unicode prose remains supported; Unicode key names are rejected.
- Headings use a whitespace-separated suffix. Both ATX and setext are tested.
  With ATX closing hashes, use `## Heading {#key} ##`. A direct-text `{#` suffix
  reserves the rest of that heading content: malformed syntax/trailing junk errors.
  Markers inside heading code, emphasis or link payloads stay literal.
- A paragraph marker is an **entire final logical line**, with preceding prose in
  the same paragraph and no blank before it. Its preceding soft/hard break is
  removed from semantic content. Same-line `Paragraph {#id}` remains literal.
- Blank-before-marker, standalone markers, heading-following markers and markers
  following a reference-definition-only input are unbound. A middle marker errors.
  There is no backward sibling search and no skipping arbitrary intervening nodes.
- In nested lists and quotes, the key belongs to the paragraph, not list/item/quote.
  Quote prefixes must be explicit at every depth. A marker's indentation must reach
  the paragraph's initial visual column (tabs count at width four). Lazy unindented
  continuations are rejected. This intentionally strict authoring policy is not a
  general replacement for CommonMark's container parser.
- Inline/fenced/indented code, multiline code spans, escaped openings and raw HTML
  payloads are not scanned as annotations. Baseline HTML equality tests protect
  these cases. Ordinary braces remain ordinary CommonMark text.
- Multiple suffix keys on a heading or marker lines on a paragraph produce
  `key-collision`; none wins. Document-wide `duplicate-key` invalidates every
  occurrence. Malformed/unbound/misplaced errors include source-byte ranges.
- Requested `{/* springleaf:key introduction */}` is visible prose in baseline
  CommonMark. Attribute/strict modes diagnose the unescaped reserved opening as
  `unsupported-comment`, not as identity. Ordinary HTML comments stay raw HTML.
  Uniform preceding annotations for lists/quotes/code are **not implemented**.

Diagnostic-bearing output is deliberately inspectable partial output, with exit
code **2**. It must never be accepted as compiled authoring data: duplicate syntax
may already have been removed from semantic text, and an unrelated valid key can
still appear alongside an error. The probe does not pretend to recover a valid
document from errors.

### Why native heading attributes are not enabled in the dialect

Executable controls show:

| Input | Native behavior | Event-layer author-key behavior |
| --- | --- | --- |
| `## H {#heading}` | HTML `id="heading"` | Internal key; no HTML ID |
| `Prose.` then `{#paragraph}` | Visible paragraph text | Paragraph key |
| `## H {#first #123 .class custom=x}` | Last ID `123`, class/attr retained, no diagnostic | Invalid author syntax |
| `## H \{#literal}` | Still consumes suffix and emits ID `literal` | Escaped literal preserved |
| `## H {#key} ##` | Attribute remains text | Internal key |
| `## H ## {#key}` | Native ID, closing hashes stripped | Not the chosen closing-hash spelling |

One could remap native `id` to internal metadata and clear it before rendering for
a weaker dialect. That alone loses duplicate-ID evidence and does not solve
escaped-literal or paragraph policy. This experiment instead leaves the flag off
so original event ownership remains available. The native feature is useful,
but **not adequate unchanged for the selected internal-identity semantics**.

## Source ownership and positions

JSON is stdout inspection output, not a saved second document. `original_source`
retains exact input bytes, tested under Unicode and CRLF. Moving the two keyed
blocks changes their ranges/order but retains the same key/content pairs.

| Position | What is actually available |
| --- | --- |
| Native Start/End tags | Both carry upstream node ranges; e.g. heading `0..32` includes `##`, suffix and line ending in the fixture. End is not merely a closing-delimiter range. |
| Native text/code/raw HTML | Original parser event byte ranges. Semantic Text may decode entities/escapes; Code may normalize whitespace. A range is source ownership, not equality with current text. |
| Tight paragraph | Derived first-inline-start to last-inline-end hull, explicitly labelled; no universal native tight-paragraph boundary is claimed. |
| Author key | Exact half-open UTF-8 range including `{#...}`, excluding quote/list prefixes and line ending. |
| Modified heading Text | Original event range retained even though semantic text loses suffix/separator. It is not an invented exact range for the shortened string. |

For [attributes.md](fixtures/attributes.md), the first heading's native range is
`0..32`, original text-event range `3..31`, and exact key range `16..31`.
The tight first item's synthetic paragraph is `141..163`, while the item itself
has the upstream range `139..164`. Conflating those ranges would lose the
distinction between prose content, annotation and list-item syntax.

Upstream offsets are more directly useful for source ownership than Rushdown's
start-plus-payload-segment interface in the preceding probe. They still do not
establish edit-safe concrete syntax for every construct: container ranges can
include gaps/prefixes/blank lines, and upstream itself has a list-end TODO. No
round-trip serializer, byte-to-semantic-character map or source-edit algorithm
is implemented or claimed. No-op preservation here means **retaining the source**,
not reconstructing it from events. Native HTML equality under projection is tested
separately from source retention.

## Rich components: explicit boundary, not accidental success

[components.mdx](fixtures/components.mdx) reproduces the important split:

| Source | Actual plain-parser result |
| --- | --- |
| `<Poll key="topic" options={["Gardens", "Maps"]} />` | Paragraph containing several Text events; no component/JSON-property structure |
| `<Poll key="topic" />` | HtmlBlock with raw HTML payload |
| `Text with <Image src="x" /> and {expression}.` | Paragraph containing InlineHtml and ordinary expression-shaped text |
| `{/* springleaf:key introduction */}` | Visible paragraph text, not native MDX comment |

No component is executed, but **pulldown-cmark's HTML renderer passes raw HTML
through**, unlike Rushdown's default renderer in experiment 06. HTML output here
is not sanitized or a safe preview policy. The JSON-shaped form is escaped prose;
the HTML-shaped form can become real HTML if a consumer inserts that output.

The identity-only mode has the same semantic trap as Rushdown: a JSON-shaped
component followed by `{#looks-like-prose}` receives a paragraph key. That exact
counterexample is tested. Identity recognition does not establish component intent.

`strict` is an executable **rejection control**: reject all raw HTML plus every
unescaped prose `<`, including malformed/multiline component candidates that
CommonMark calls text. It rejects the valid JSON example, a function-call property,
a spread, multiline properties and an inline JSON-shaped component. It intentionally
also rejects `a < b`, while code/escaped `<` remain allowed. It is **not a JSX or
JSON validator**, has no structured properties, and accepts `{danger()}` as inert
CommonMark prose. Partial keys can remain in strict output; any diagnostic blocks
acceptance. No security claim for a future MDX runtime follows from this guard.

A real compiler needs a deliberate component grammar/region handoff that preserves
Markdown ownership, parses JSON-only property values, whitelists names/props, and
rejects spreads/imports/arbitrary expressions before lowering. Neither a full MDX
parser nor that narrow literal grammar was built here. Regex preprocessing would
not close this gap honestly.

## Comparison and next recommendation

- **Keep `{#id}` as the syntax lean for selected ordinary prose.** Both this and
  the preceding Rushdown experiment demonstrate it without wrappers or a saved AST.
  This is evidence, not acceptance of a production spelling or parser choice.
- **pulldown-cmark is viable if the destination is Markdown-first plus a separately
  specified literal-component layer.** Offset events and native typed link/heading
  data are attractive. The event seam suffices for identities; no fork is needed.
- **Rushdown has the better proven seam for adding grammar during parsing.** Its
  custom block/inline parsers and AST transformers can claim new syntax before
  prose/HTML fallback; this experiment cannot infer that capability from an event
  iterator. Rushdown's preceding probe also implements comment/container keys,
  which this narrower probe does not. Neither proves MDX/JSON-component support.
- **Do not choose on ID support alone.** The next discriminating proof is a narrow
  rich-literal grammar composed with Markdown: multiline JSON props, nested children,
  code/HTML ownership, malformed input and exact source handoff. Only then decide
  whether pulldown's output-layer composition or Rushdown's parser hooks are the
  cleaner compiler boundary. No implementation or parallel-session result is
  presumed by this recommendation.

Known gaps: comment binding, container/item-level keys, metadata/frontmatter,
GFM extensions/footnotes/tables/tasks, broad CommonMark corpus testing, fuzzing,
pathological-depth handling, performance/streaming memory measurements and full
Leaflet block/facet lowering. The small tree/projection owns/clones input/events
for inspection; this is not a minimal-allocation performance implementation.

## Reproduction and receipts

See [README.md](README.md) for the runnable modes. From this experiment directory:

```sh
cargo fetch --locked  # initial setup only
cargo run --locked --offline -- attributes fixtures/attributes.md
cargo run --locked --offline -- native fixtures/attributes.md
cargo run --locked --offline -- plain fixtures/components.mdx
cargo run --locked --offline -- strict fixtures/components.mdx # intentional exit 2
cargo run --locked --offline -- attributes fixtures/invalid.md # intentional exit 2
cargo nextest run --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

The valid fixture projects **6 author keys**; the native control exposes HTML
heading IDs and zero author keys. The invalid fixture exits 2 with **4 diagnostics**
(two duplicate occurrences, one collision, one unbound marker). Strict components
exit 2; plain components exit 0 as a baseline, not a validation success.

The first complete 33-test pass used nextest run
`0d8f19d2-f9fa-476d-9dca-c33f0b6d6d7d`; the final suite adds the identity/component
counterexample and three executable tests. Clippy's initial single-range-vector
style warning was corrected without changing grammar.

Final receipts:

- Nextest run `a465db6f-ad45-42cd-8332-35a1e9027ae2`: **37/37 passed**, zero skipped,
  all `--locked --offline`.
- Warnings-denied clippy for all targets passed; `cargo fmt --check` passed.
- Report OKF check passed: **1 document, 10 links, 16 sources**; ordinary README
  check passed: **1 document, 1 link**.
- Executable smoke matrix: attributes fixture exit 0 / 6 keys / 0 diagnostics;
  native attributes exit 0 / 0 author keys / 0 diagnostics; plain components
  exit 0 / 0 keys / 0 diagnostics; attributes components exit 2 / 0 keys /
  1 unsupported-comment diagnostic; strict components exit 2 / 0 keys /
  4 diagnostics; invalid attributes exit 2 / 0 keys / 4 diagnostics.

Owned checkpoint:

- `527c09fa` — event-layer implementation, isolated dependencies, fixtures and
  initial 33-test corpus; formatting and warnings-denied clippy passed.

The subsequent owned documentation/control checkpoint adds this report, entry
point, the component-key counterexample and executable tests. Its immutable ID
is supplied in the experiment's session handoff rather than self-cited here.

Document checks, from the repository root:

```sh
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf experiments/08-pulldown-cmark-identity/REPORT.md
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check experiments/08-pulldown-cmark-identity/README.md
```

### Connections

- [The source-authoritative authoring proposal](../../design/mdx-recreate0.gpt6a.md)
  supplies the purpose and JSON-only rich-property constraint. This experiment
  addresses parsing/optional identity only, not its remote replacement pipeline.
- [The earlier Rushdown report](../06-rushdown-identity/REPORT.md) is the preceding
  independent parser-hook proof and the sole implementation comparison here.
- [The experiment operating guide](../README.md) explains the tracked evidence
  boundary; [this experiment's entry point](README.md) is the reproduction surface.
