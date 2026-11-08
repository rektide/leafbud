---
type: Design
title: "Markdown-first rich text with checked exact fallbacks"
description: "Proposed bidirectional inline serialization, block-local fallback and deliberate source-preserving tidy for imported keyed MDX."
resource: /design/markdown-roundtrip0.gpt6a.md
status: proposal
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#medium
sources:
  - resource: "opencode://ses_e45c7f589ffeP5t82bqsQ23mlI"
    title: "Independent commissioned design; narrow redacted inspection of the user-indicated link, source review and proposed implementation slice"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted operation-specific preservation, scoped fingerprints and all-applicable tidy"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Controlling keyed bidirectional addendum and current-content versus CRDT-history distinction"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Single-file identities, bindings and compact independent witnesses"
  - resource: /design/draft-checkout0.gpt61s.md
    title: "Implemented import profile and private checkout receipt"
  - resource: /crates/document/model/src/lib.rs
    title: "Observed Inline, Mark, canonical push_run, keys, associations and semantic_value"
  - resource: /crates/document/mdx/src/lib.rs
    title: "Observed Satteri reader, whole-document renderer fallback and strict Run properties"
  - resource: /crates/leaflet/editor/src/text.rs
    title: "Observed Yrs mark decoder, href-only links and actual hard-break nodes"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Earlier executed ordinary Markdown/Run/Yrs evidence and dialect boundaries"
  - resource: /experiments/13-satteri-authoring/src/author.rs
    title: "Inline lowering precedent; explicit soft-wrap conversion differs from production"
  - resource: /experiments/14-satteri-authoring-attributes/REPORT.md
    title: "Earlier source-coordinate, literal protection and strict reparse evidence"
  - resource: /experiments/14-satteri-authoring-attributes/src/adapter.rs
    title: "AST-guided source edits and original-coordinate discipline, not a formatter to copy wholesale"
---

# Markdown-first rich text

## Recommendation and standing

**Prefer ordinary Markdown for supported inline meaning, then prove the emitted
block through the production compiler before using it.** Keep `Run` and typed
block components as exact fallbacks. Normalize existing source deliberately with
`tidy`; do not turn no-op pull into a formatter.

This is a proposal for parent review, not acceptance or implementation. The
guarded-pull implementation is concurrent work; integrate only after that build
finishes. Production observations below describe the code read from the checkout
whose starting parent was `67e66383`; receiving function names may change.
No production files or private documents were edited, no parser/build experiment
was run, and no remote calls were made by this design session.

The scope is Markdown **inside the managed MDX document**, retaining Page/Key and
the YAML envelope. A standalone plain-Markdown export that removes associations
is a different operation. There is no second source, formatting ledger, saved
base body or encoded CRDT history.

## Observed problem and existing capabilities

The user-indicated private document contains one `Run`: a standalone, href-only
HTTPS link whose label equals its destination, without boundary whitespace or
literal newlines. Structural inspection found no explicit Paragraph wrapper in
that document. Its prose, URL, filename and remote locators are intentionally
not reproduced here. A synthetic equivalent is:

```mdx
<Key id="reference" />
<Run text={"https://example.org/read?q=notes"} marks={[{"kind":"link","href":"https://example.org/read?q=notes"}]} />
```

Preferred output:

```mdx
<Key id="reference" />
[https://example.org/read?q=notes](https://example.org/read?q=notes)
```

The current [MDX reader](../crates/document/mdx/src/lib.rs) already maps Markdown
Link/Strong/Emphasis/Delete/InlineCode/Break into the same typed semantics as
Run/HardBreak. The writer emits **every marked run** as JSX, even this simple
link. It tries normal blocks, then falls back to literal wrappers for the entire
document if the first parse differs. This is a conservative writer preference,
not evidence that Satteri cannot parse ordinary links.

The [model](../crates/document/model/src/lib.rs) has text plus sets of Strong,
Em, Code, Strikethrough, Underline, Highlight(color), Link(href), and independent
HardBreak nodes. `push_run` sorts/deduplicates marks, drops empty text runs and
merges adjacent identical mark sets. These partitions carry no source or remote
identity; block keys and node bindings do. Empty paragraphs still need an exact
representation even though empty runs canonicalize away.

The [Yrs decoder](../crates/leaflet/editor/src/text.rs) accepts only `href` on
links; the model has no title field. Production rejects nonempty Markdown link
titles. Experiment 13 supports ordinary formatting and exact Runs, but is not a
drop-in writer: its Run schema and soft-wrap lowering differ from production.
Experiment 14 establishes source-range and strict-reparse discipline, not a
lossless CST formatter or permission to replace the parser.

## Semantic contract

For each supported block `B`, require:

```text
lower(parse(render_preferred(B), actual_container_context)) == canonical(B)
```

Equality includes block kind/heading level, exact Unicode text, all mark values,
HardBreak positions, keys, page order and reference targets. Only existing
`push_run` normalization is allowed: adjacent equal run partitions and mark-set
order are immaterial. Do not trim, normalize Unicode, collapse whitespace,
normalize URLs, replace literal newlines with breaks, or ignore a missing mark
to make a candidate pass. Also validate the final whole document and state.

This promises **current-content roundtrip**, not reproduction of delimiter
choice, line wrapping, YAML quoting, original JSX spelling, or Yjs item history.
An authored Markdown edit lowers through Satteri into current typed content;
future editor updates use the retained block association and current remote Yrs
state, as specified by the [bidirectional handoff](authoring-handoff0.gpt61s.md#keyed-bidirectional-addendum).
Formatting alone must not cause a text-fact update or recreate an entity.

Keep the existing scoped fingerprint definitions. Changing the writer does not
change semantic profiles. A formatting-only tidy must produce identical current
editor projection and bindings and leave stored localHash, remoteHash, profile,
acceptance times, release and naming witnesses unchanged. In particular, do not
replace an old accepted localHash with today's hash if the user has local edits:
formatting does not accept those edits. Missing-key assignment is separately
semantic identity preparation, not grounds to settle editor agreement.

## Rendering algorithm and seams

Keep this in `document/mdx`; do not introduce Markdown rendering into the CLI,
workflow or Yrs decoder. Suggested internal boundaries, not public API mandates:

```text
render_inline_candidate(canonical_runs, block_context) -> candidate | reason
render_block_exact(block) -> exact_component_source
validate_block_candidate(source, expected_block, container) -> checked | reason
plan_markdown_tidy(parsed_source) -> nonoverlapping_byte_edits + report
```

1. Canonicalize through the existing model normalization. Traverse typed runs,
   never search/replace JSX text with a regular expression. A literal example of
   `<Run .../>` inside a code span, JSON string or escaped text is not a component.
2. Build a small temporary inline grouping tree. Prefer Link outermost, then
   Strong, Em, Strikethrough, with Code at the leaf. At each run boundary retain
   the common prefix of that ordered mark stack, close the rest and open the new
   suffix. This groups a link across differently emphasized label runs. Crossing
   mark intervals can split/reopen wrappers; semantic equality ignores that
   partition, but no markup nesting may alter the covered characters.
3. Emit context-specific escaped text, delimiters, destinations and code spans.
   Reject ineligible spans into exact Run atoms rather than approximating them.
   Multiple distinct Link marks on one interval cannot become nested Markdown
   links; retain the whole marked run exactly. HardBreak is an independent atom:
   close wrappers before it and reopen afterward, matching the unmarked model.
4. Render Paragraph, Heading and single-paragraph Quote using normal Markdown.
   Include the actual heading/quote prefix, Page context, Key and blank-line
   boundaries in validation. A quote with a break needs `>` on each emitted line.
5. If a mixed Markdown/Run candidate is not exact, use a literal typed wrapper
   **for that block**, with exact Run/HardBreak children. Do not downgrade every
   other block. Do not merge/split keyed blocks to avoid a parse error.
6. Assemble the final source and compile it once in full. Require expected
   semantic document and unchanged intended state. If even exact output differs,
   return a structured error and write nothing; no warn-and-skip success.

Start with a bounded, deterministic candidate per block plus exact fallback;
do not search every delimiter permutation or repeatedly parse the whole file
for every run. A block validator can use a tiny synthetic managed envelope with
matching Page/Key/node rows and the same production lowerer; the final full-file
validation remains necessary for neighboring boundaries and references. For
tidy, actual-source splicing also proves that unchanged neighboring bytes still
compile. Validation is linear in total emitted source for bounded attempts,
plus the final document parse; no network or Yrs decoding is needed per candidate.
Reuse parsed source/metadata rather than repeatedly parsing the original file.

## Preference rules and exact boundaries

| Meaning | Preferred spelling | Exact fallback / constraint |
| --- | --- | --- |
| Plain text | Normal prose with required escaping | Run for whitespace/control or parser interactions not faithfully representable |
| Link | `[label](destination)` | Run for a destination/label that cannot roundtrip exactly, conflicting link marks or empty semantic label |
| Strong / Em / Strike | `**text**`, `*text*`, `~~text~~` | Run when flanking/delimiter interactions change text or marks |
| Code mark | Backtick span with a safe delimiter length | Run for newline/whitespace normalization or incompatible nesting |
| Underline / Highlight | Exact Run with original marks | No invented HTML/CSS or loss of color intent |
| HardBreak | Backslash + source newline where valid | `<HardBreak />`; literal block if required by flow parsing |
| Empty rich block | Exact empty Paragraph/Heading/Quote representation | Never omit its Key or invent visible placeholder text |

### Escaping, destinations and titles

Use a dedicated **text** escaper, distinct from destination and code escaping.
Protect backslashes, brackets, emphasis/strike delimiters, backticks, `<`, `&`
when entity-like, and MDX `{`/`}`. Protect block-leading constructs (`#`, `>`,
list markers, thematic breaks) and heading suffixes in their actual context.
Keep ordinary punctuation such as sentence periods readable when safe; escaping
every ASCII punctuation character is a valid conservative starting point but
need not be the final authoring style. A plain URL without a Link mark must stay
plain text even if the parser enables autolinks; escaping or Run plus validation
prevents an accidental semantic link. Avoid raw HTML/autolink generation in the
first slice: explicit bracket links are more predictable in MDX.

Prefer bare parenthesized destinations for a simple safe URL; use an angle-bracket
destination when needed, with proper Markdown escapes. Do not percent-encode,
decode, lowercase, resolve relative paths, or otherwise transform the stored
href merely to satisfy syntax. The decoder's URL must equal the original string,
including query punctuation, ampersands, percent escapes and Unicode. Destinations
with newlines/control characters or unproved escape behavior retain Run. The
production reader currently does not share experiment 13's limited scheme check;
this writer change must not silently add a different URL policy.

Do not emit or strip titles. Nonempty authored titles remain a diagnosed unsupported
feature because neither the current typed Link nor its editor mapping can carry
them. If upstream title support is later verified, add an optional title through
model, decoder, writer and fingerprint fixtures together, then test quoting and
escaping. Do not hide titles in unknown props. Explicit empty-title spelling is
style only under today's parser value, but no nonempty value may disappear.

### Whitespace, marks and code spans

An unmarked space between marked words can remain a literal space. A marked
leading/trailing space cannot simply move outside its delimiters: that changes
the mark coverage. Keep the entire problematic span in Run initially. Similarly,
all-whitespace marked runs, tabs, CR, repeated edge spaces and literal newlines
are exact-fallback cases until specific tested encodings exist.

Nested Markdown lowers to the union of inherited marks; nesting order is not
semantic. Adjacent links to different destinations remain separate. Adjacent
equal links/marks may coalesce under `push_run`. Delimiter collisions such as
an Em interval immediately adjoining Strong, punctuation-flanked spans or an
intraword mark require the full candidate check; do not insert spaces or zero-width
characters to make them parse. An unsupported mark combination can remain one Run
without preventing other safe Markdown in the same block, if flow validation passes.

Choose code-span delimiters longer than the longest contiguous backtick sequence
in the text. Add the standard boundary padding only when parser stripping gives
back **exactly** the original string. Never backslash-escape code-span contents
as ordinary prose. Code can be enclosed by a link or emphasis wrapper to represent
combined marks; Markdown-looking text inside the code span remains literal. For
the first slice use Run for all code text containing newlines and uncertain edge
whitespace. Experiment 13 records a multiline single-backtick MDX collision;
there is no need to solve that by changing parsers.

A source soft wrap, a literal LF inside Run, and HardBreak are three distinct
inputs. Preserve production's current lowering for each. Unlike experiment 13's
explicit newline-to-space conversion for Text, production presently passes
native Text values through unchanged. Do not introduce reflow or change this
rule as part of serialization. Emit no soft wraps in generated prose; characterize
LF/CRLF parser behavior in tests before proposing any separate language change.
Two-space hard-break syntax remains accepted where the reader supports it;
prefer backslash-newline when generating because trailing spaces are fragile.
At block edges, consecutive breaks or headings, use HardBreak/typed block if the
ordinary spelling cannot preserve topology.

### MDX flow and unknown components

Run at the start of a line can be parsed as flow JSX and separate following prose
into another block. This is why validation must surround the candidate with its
real block/container shape. Never add dummy text to force a paragraph. The exact
wrapper is the safe local fallback. A Paragraph wrapper's current reader accepts
literal Run/HardBreak children, not arbitrary Markdown children; removing the
wrapper is an all-block candidate transformation, not changing only its inner Runs.

Known nonrepresentable content stays exact. Unknown components remain untouched
source, never flattened to their children or encoded as guessed Run meaning.
Today's strict compiler rejects unknown components: initially, tidy must refuse
such a file without writing it. A later source-preserving opaque-node mode could
normalize independent known spans while retaining unknown bytes, but this design
does not claim the current model can validate that. Managed pull must likewise
report unsupported content rather than choose lossy downconversion.

## Tidy and lifecycle integration

Follow the [accepted interface](authoring-interface0.gpt6a.md#tidy-all-by-default-selectors-narrow-the-passes):

```sh
springleaf tidy essay.mdx --dry-run   # all applicable implemented passes
springleaf tidy essay.mdx             # includes Markdown normalization
springleaf tidy essay.mdx --markdown  # proposed selector: this pass only
springleaf tidy essay.mdx --keys      # only missing keys, never Markdown formatting
```

`--markdown` is a proposed pass selector, not an export format or a demand to
erase components. Multiple selectors select their union; no selectors means all
applicable supported passes. Keep `--all` for workspace file selection. If key
assignment is not implemented by the receiving baseline, do not label Markdown-only
work as implementing the entire accepted tidy destination.

For this pass, locate supported blocks and native components through parser
source ranges and retained syntax information. Replace only candidate body spans;
retain Key annotations, Page boundaries, frontmatter bytes and comments. Do not
call the whole-document `render` on the parsed model for tidy: it would regenerate
YAML and lose source trivia. Exclude identity-marker ranges from replacement;
reparse to prove they still bind to the same nodes. Preserve original line-ending
convention for inserted syntax; literal strings retain their own exact values.

Comments omitted by semantic lowering are still owned source. Retain them at
their exact locations by narrower edits where possible; skip a block if a proposed
full-block rewrite would swallow or relocate a comment. Report that skip. If the
current strict compiler cannot accept a comment form, report unsupported input
and make no write rather than remove it to make parsing succeed. Native ranges
alone are not proof of trivia coverage: inspect the exact replaced slice.

Construct edits against one source revision, reject overlaps, apply in descending
byte order, compile the complete result, and recheck original bytes before the
existing atomic writer installs it. A second tidy must produce no edits. On an
error, leave the whole file unchanged; report per-file outcomes for batches.
Dry-run can report keys and reason codes/counts without dumping private text or
destinations. Normal exact fallbacks are successful preservation, not failures;
provide details such as `marked-edge-whitespace`, `unsupported-mark`,
`flow-boundary`, `comment-preservation`, or `candidate-mismatch` when requested.
Malformed input and failed final validation are errors with source positions.

| Operation | Effect of this proposal |
| --- | --- |
| New import | Prefer checked Markdown for newly generated body |
| Remote content genuinely changes, guarded pull permits adoption | Use improved renderer for the body scope that pull already owns; retain its source-preserving metadata policy |
| No-op pull / repeated import | Keep original bytes and mtime, including old Runs |
| Receipt-only update / current endpoints converge | No body normalization |
| Explicit tidy | Normalize selected supported body spans, preserving current semantics, bindings and witnesses |
| Ordinary push | Consume either spelling identically; do not format the source implicitly |

## Tests and success criteria

Use synthetic fixtures and independent expected typed values, not the private
document. Parser/writer agreement alone can hide a shared misunderstanding;
assert known text, mark coverage and break nodes as well as roundtrip equality.

1. **The motivating case:** standalone href-equals-label HTTPS link with query
   punctuation becomes bracket Markdown, compiles identically, retains its Key
   and full binding row. Also test a link embedded between prose runs.
2. **Common author edits:** Markdown strong/em/strike/code/link and nested marked
   link labels lower to explicit expected runs. Changing the Markdown label or
   href changes only that typed content; unchanged block/entity/textFact remain.
3. **Boundary matrix:** before/after punctuation, intraword marks, adjacent distinct
   links, crossing mark sets, strong-to-em adjacency, marked/unmarked spaces,
   Unicode/combining characters, literal brackets/braces/backslashes/entities.
   Every case either emits exact Markdown or an explicit exact fallback.
4. **URLs:** parentheses, brackets, ampersands, percent sequences, spaces, Unicode,
   relative and fragment destinations, plus empty/control/newline cases. No href
   normalization. Nonempty title is an error, not stripped output.
5. **Break/code negatives:** literal LF versus HardBreak versus soft wrap; CRLF;
   consecutive/leading/trailing breaks; tabs; all-space code; backticks; code with
   combined marks; MDX-looking literals. Assert exact expected distinctions.
6. **Block isolation:** one unsupported-mark or flow-collision paragraph must not
   wrap unrelated clean paragraphs/headings/quotes. Exact wrapper can simplify
   back to ordinary prose only when its whole-block comparison passes. Empty
   paragraph remains present. Duplicate/orphan Keys still fail.
7. **Source preservation:** tidy a file containing YAML comments/styles, author
   extras, state witnesses, keys, source comments and an existing local semantic
   edit. Only eligible body ranges change; stored witnesses are byte-untouched.
   Unknown components or unsupported comment forms yield no write. Include
   byte-range guards for UTF-8, BOM and CRLF.
8. **Workflow:** no-op pull/reimport does not rewrite old Runs; changed remote body
   can adopt Markdown under existing guards; `--keys` does not normalize Runs;
   `--markdown` does not mint keys; default tidy runs all implemented applicable
   passes. Dry-run/race refusal/idempotent repeat have zero write effects.

Success is useful ordinary output for the common cases, exact semantics for all
accepted cases, local fallbacks, stable identities and no spurious drift. Do not
make success depend on eliminating every Run. No live content rewrite is needed
to prove this first implementation.

## Bounded first implementation for g

After guarded-pull lands and the parent approves:

1. Inspect the finished MDX source-span/body replacement seams and fingerprint
   APIs. Add a pure Markdown-first block renderer for existing Paragraph/Heading/
   Quote and current inline marks; simple href links are the must-pass case.
   Use existing reader semantics unchanged, deterministic nested marks, safe
   single-line code, and conservative exact fallbacks. Retain exact Code/Image/
   PageLink serialization. No title/model expansion or new content types.
2. Replace whole-document fallback with block-local checked fallback and final
   document validation. Route new import and genuinely changed pull through it,
   preserving the already-built no-op and metadata patch contracts.
3. Expose source-range Markdown normalization through `tidy --markdown` and
   include it in default all-applicable tidy. Reuse the workflow atomic/source
   revision guard. If comments or span ownership cannot yet be preserved, skip
   those blocks or refuse unsupported files with a precise report; do not widen
   rewrites to compensate. Keep `--keys` semantics strict.
4. Add the synthetic reader/writer and workflow regressions above; run normal
   formatting, Clippy and appropriate nextest checks. No Yrs encoder change is
   planned, so no new CRDT algorithm or remote-write experiment is required.
   Record supported ordinary cases and conservative fallbacks in the receiving
   documentation after this independent design has been reviewed.

This is one coherent authoring improvement, not a general Markdown beautifier.
An initial implementation may conservatively retain mixed/awkward Runs rather
than implement an optimal interval-tree serializer; it must still handle the
motivating link and ordinary single-mark text well and prove exact fallback.

## Alternatives and remaining uncertainty

- **Link-only special case:** smallest immediate fix and useful first checkpoint,
  but leaves ordinary emphasis noisy. If chosen, retain the same typed traversal,
  block validation and tidy seams so it can grow without regex migration work.
- **Run everywhere:** exact and easy, but fails the requested authoring experience;
  keep it only as fallback, not the preferred serialization.
- **Generic Markdown formatter/stringifier:** may improve style coverage, but
  needs a typed adapter and the same MDX/context/equality tests. Adding a parser
  or formatter dependency is not justified by this link case alone.
- **Always reserialize whole files on pull/tidy:** cheap implementation but violates
  no-op/source-comment/state preservation. Changed-body pull may own a larger
  rewrite; deliberate body tidy still must preserve unrelated source bytes.

Remaining evidence gaps are narrow: exact Satteri delimiter and destination escape
behavior on the proposed matrix, soft-wrap/CRLF values in production, and the
source-range/comment facilities delivered by the concurrent guarded-pull build.
The equality gate safely bounds serialization while those fixtures are added;
it is not evidence of universal Markdown representability or proof of future
editor mutation correctness. The parent should review selector spelling and the
scope of the first slice, then commission implementation separately.

## Connections

- [Authoring interface](authoring-interface0.gpt6a.md): controlling lifecycle,
  tidy semantics, operation ownership and scoped hashes; this proposal specializes
  its supported body normalization.
- [Frontmatter design](frontmatter0.gpt61s.md): identities and witnesses stay in
  the one file; a different inline spelling does not accept new endpoint state.
- [Draft checkout receipt](draft-checkout0.gpt61s.md): explains the conservative
  import writer this proposal improves, without invalidating its exactness proof.
- [Experiment 13](../experiments/13-satteri-authoring/REPORT.md): prior positive
  Markdown/Run compiler evidence and multiline-code limits.
- [Experiment 14](../experiments/14-satteri-authoring-attributes/REPORT.md): precedent
  for syntax-aware source ranges and strict validation after edits, not a new
  requirement to implement brace keys in this slice.

Reciprocal links and index updates are left to the parent after the concurrent
build joins; this commission owns this new design file only.
