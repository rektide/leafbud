---
type: Report
title: "Markdown round-trip delivered: checked Markdown-first rich blocks, exact fallbacks and deliberate tidy"
description: "Implemented the approved bounded slice: ordinary links/marks/code render as proven Markdown with block-local exact fallback, imports and changed pulls adopt it, and tidy FILE.../--all --markdown normalizes existing files through parser-derived source spans preserving trivia and witnesses. Includes the parent-commissioned State deny_unknown_fields correction."
resource: /design/markdown-roundtrip1.glm53m.md
status: implemented
generated:
  at: 2026-11-08
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e45962104ffetgOyiGVSJHaXiM"
    title: "g implementation session: State correction, Markdown-first renderer, tidy pass and CLI, tests, live read-only dry-run evidence"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Parent commission, State-gap code review, implementation review and correction of the test-count account"
  - resource: /design/markdown-roundtrip0.gpt6a.md
    title: "Approved design and bounded first-implementation slice this delivers"
  - resource: /design/guarded-pull0.glm53m.md
    title: "Finished guarded-pull seams integrated against (update body policy, classification, atomic writes)"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted tidy semantics and selectors this tidy pass specializes"
  - resource: /crates/document/mdx/src/lib.rs
    title: "Candidate renderer, per-block validation, tidy planning and tests as landed"
  - resource: /crates/workflow/src/tidy.rs
    title: "Workspace tidy workflow with dry-run and atomic writes as landed"
---

# Markdown round-trip implementation

## Outcome and checkpoints

The user commissioned g implementation of
[the approved design](markdown-roundtrip0.gpt6a.md) after the guarded-pull
build returned and the parent reviewed its seams. All work stayed in
`~/src/springleaf`; `~/l` (the document workspace) was only ever read.

- `qtzmvxtw` — **Correction (parent code review)**: `State` gained
  `deny_unknown_fields` (like `Binding`/`Node`/`Sync`). Unknown top-level
  `@springleaf` fields were previously **silently dropped** by typed
  reserialization in `mdx::update` — a real data-loss path the guarded-pull
  handoff had misreported as already blocked. Regression: a synthetic
  `futurePolicy` field makes full parse and `update` refuse while untyped
  `header` inspection still reads the raw state; the handoff's wording was
  corrected with provenance.
- `vpllonmn` — **Markdown-first renderer**: per-block candidates with
  production-parser validation, exact atoms and literal-wrapper fallback;
  `parse_with_spans` (BOM/CRLF-aware parser byte ranges) plus
  `plan_markdown_tidy`/`apply_markdown_tidy`.
- `wsulwwrx` — **tidy workflow + CLI**: `workflow::tidy` (operands or
  `--all`, dry-run, atomic original-byte writes, reason-code reports) and
  `springleaf tidy FILE.../--all [--markdown] [--dry-run] [--json]` with
  honest exit-1 rejection of the unimplemented `--keys/--from/--accept`
  selectors.
- `roswolus` — **Pull adoption proof**: real-Yrs link-mark regression — a
  genuinely changed remote body adopts as `see the [docs](…) now` and the
  repeat pull is a byte- and mtime-exact no-op.
- `kpyoslvs` — **Version 0.6.0**: coordinated minor bump with exact internal
  pins (public API additions: tidy CLI surface, `parse_with_spans`,
  tidy planning).

## Capability contract as implemented

**Writer (import and genuinely changed pull bodies).** Each
Paragraph/Heading/Quote renders a deterministic Markdown-first candidate:
`[label](destination)` links (bare destinations when URL-safe, angle-bracket
destinations otherwise, exact `Run` for control-character hrefs), nested
strong/em/strikethrough wrappers ordered for delimiter stability across
crossing run boundaries, backtick code spans with fences longer than any
internal backtick run, backslash-newline hard breaks (`>` prefixes on quote
continuation lines; typed `<HardBreak />` atoms in headings and trailing
position). Ineligible runs stay exact `<Run>` atoms *inside* the surrounding
markup: underline/highlight, conflicting double links, control/tab/LF text,
line-leading or block-final whitespace, and bare-URL text the GFM
autolink-literal post-pass would link (probing showed escape-resolved text
still autolinks, so such segments are split out as atoms while neighboring
prose stays Markdown). Text escaping is context-aware — emphasis/bracket/
tilde/backtick/dollar/brace/entity-like-ampersand and MDX-significant
characters, block-leading constructs at every emitted line start, ordered
list delimiters escaped rather than digits, `#` always escaped in headings —
while ordinary punctuation stays readable (the motivating URL label emits
unescaped inside its brackets, where autolinks cannot form).

**Block-local fallback.** Every candidate is validated through the
production lowerer inside a synthetic single-block envelope with the real
Page/Key/node rows; a failing block keeps the exact literal wrapper without
downgrading neighbors. The composed document must still parse back to the
same semantics and state or the all-literal body is used, and if even that
fails nothing is written. No-op pulls, receipts-only and converged cases
keep old body bytes by the existing `update` contract (only genuinely
changed pages re-serialize).

**Tidy.** `plan_markdown_tidy` walks parser-derived block spans and proposes
edits only where a validated candidate differs from the current bytes;
`apply` splices descending and re-parses the complete result requiring
semantic and state equality. Frontmatter bytes, YAML/body comments outside
replaced spans, Key markers and Page boundaries are preserved (a body MDX
comment or unknown component makes the whole file fail loudly — never a
partial rewrite); inserted syntax keeps the file's own line-ending
convention; BOM-prefixed files work; witnesses and stale receipts are never
touched, so local unaccepted edits survive formatting. Idempotent by
construction (a second tidy plans nothing). Writes reuse the workflow's
atomic original-byte guard; dry-run composes and validates but never writes.
Reports carry per-file outcomes, block counts and reason codes
(`candidate-mismatch`, `unsupported-mark`, `whitespace-or-control-text`,
`autolink-text`) without private text. `--all` selects managed files only;
`--keys` remains keys-only by contract and is rejected as unimplemented,
exactly like `--from`/`--accept` — no false claim of the full accepted tidy
destination.

## Verification

- `cargo fmt --all` / `--check`; `cargo clippy --workspace --all-targets --
  -D warnings` — clean; `cargo nextest run --workspace` — **115/115 passed**
  (the guarded-pull handoff reported 84 tests; 101 was an intermediate count
  during this implementation, not the preceding slice's baseline). Coverage
  includes MDX reader/writer and tidy cases, workflow tidy, CLI subprocesses
  and the pull adoption regression; all offline.
- Reader/writer synthetics assert **independent expected typed values**, not
  only round-trip equality: the motivating href-only link and link-in-prose
  cases, strong/em/strike/code/nested/crossing marks, a destination matrix
  (parens/brackets/ampersands/percent/Unicode/fragment/empty/backslash/
  spaces via angle brackets; control-char href stays exact), atom fallbacks
  (underline/highlight/edge/control/double-link/autolink/email splits), a
  break matrix (leading/consecutive/trailing/heading/quote/LF-vs-HardBreak-
  vs-soft-wrap), a 21-entry escaping matrix plus heading-hash protection,
  block isolation, empty-paragraph exactness, no-op and changed-body update
  byte behavior, and tidy span/trivia/witness/CRLF/BOM/idempotence/
  refuse-unparsable cases.
- Workflow tests: comments/customField/witness byte preservation with a
  local semantic edit present; dry-run writes nothing; `--all` ignores
  unmanaged and non-mdx files; broken bodies fail loud without writes;
  operand validation; the atomic writer refuses stale original bytes.
- CLI subprocess tests: JSON and human output shapes, idempotent repeat,
  honest selector rejection with the file untouched, usage exit 2, lone
  failure exit 1, mixed batch exit 4.
- **Live (read-only)**: `tidy --all --markdown --dry-run` over the real
  27-file workspace predicted **26 normalizable files (281 blocks), 1
  already clean, 0 failures**; one block is kept exact as a
  `candidate-mismatch`. The user-indicated motivating file (single
  href-only link whose label equals its destination) was copied to a
  scratch directory and tidied there: the `Run` became bracket Markdown
  with frontmatter bytes, Key markers, Page tags and semantics unchanged,
  `check` passes, and a second tidy reports unchanged. **No live file was
  rewritten**; the real workspace only ever saw dry-runs. No remote calls.

## Boundaries and next seams

Reader semantics, the Yrs decoder, model marks, fingerprints and the hash
schema are untouched; link titles remain rejected by the reader and
unsupported by the model (no silent dropping or title invention). The
conservative whole-file full-`extra` conflict conservatism noted in the
guarded-pull handoff is unchanged. Tidy's pass set is honest about being
Markdown-only: key assignment and naming reconciliation stay separate
slices, and body-comment/unknown-component files still refuse rather than
partially rewrite. A standalone plain-Markdown export remains a different
operation. Smallest useful next slices: `status`/`diff --edge editor`
reusing pull's read-only classification, and a source-preserving
opaque-node/comment mode if the strict compiler ever grows one.
