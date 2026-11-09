---
type: Report
title: "Batch A review corrections: spans, identity, preparation, settlement, status"
description: "Receipt for the focused parent-review correction pass over document/model, document/mdx and workflow/status: seven review rows reproduced as failing tests, fixed in six coherent commits plus rustfmt."
resource: /design/authoring-corrections0.glm53m.md
status: complete
generated:
  at: 2026-11-09
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e419a7411ffe4oaxpvanZ9R4hO"
    title: "This correction pass: reproduced each review row as a failing test, landed six correction commits plus rustfmt, updated tickets and this receipt"
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Parent review commission with the concrete defect list and join conditions"
  - resource: /design/authoring-review0.gpt6a.md
    title: "Parent architectural review whose correction table this pass enacted"
  - resource: /design/author-create-keys0.glm53m.md
    title: "Batch A receipt under review (its 175-test claim versus the 173-test handoff noted there)"
  - resource: /crates/document/model/src/lib.rs
    title: "authored_value projection and keyed new-object identity"
  - resource: /crates/document/mdx/src/parse.rs
    title: "Document-order spans, identity reservation, provisional main-page skipping"
  - resource: /crates/document/mdx/src/keys.rs
    title: "Claimed identities, plan-source binding, total apply, link retargets"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Syntax-based envelope insertion; settlement guards over authored data"
  - resource: /crates/workflow/src/status.rs
    title: "Typed receipt branches, reachable bound-no-receipt, duplicates before unbound"
---

# Batch A review corrections

Commissioned pass over the [parent review](authoring-review0.gpt6a.md)
correction table for the batch A foundation at `6aee3dd9`. Every row was
first reproduced as a failing test, then fixed; each coherent correction is
its own commit. No redesign, no new capability, no version bump (internal
corrections; the concurrent transport session owns the coordinated version
in the shared working copy). The committed predecessor is preserved; the
exact regression test for the envelope-shape row was also run against an
extract of the pre-correction sources and panics there at the former
`expect`.

## Corrections landed

| Review row | Commit | Failure reproduced | Fix |
| --- | --- | --- | --- |
| Spans and object order | `7ad81dab` | Interleaved main/ABOUT/main crossed span assignment; preparation failed its reparse proof | Reader records per-block span indices; spans/keyed are regrouped alongside the final pages composition (main blocks, then definitions), so flattened index *i* always addresses its own block |
| New graph identity | `1c561dbd` | PageLink retarget between two distinct new pages changed no editor value | `editor_value` new associations are `{"kind":"new","key":<source key>}`; confirmed associations keep spelling-free locators; profile-3 fixtures updated in place (no shim) |
| Provisional/assigned names + envelope shape | `03e777b7` | Explicit `b-auto-1` after an unkeyed block false-collided; retired rows were borrowed as hidden associations; flow+blank-line and four-space envelopes panicked at the reparse `expect` | Pre-scan reserves every Key id, Page name and node-table key before allocation (`p-auto` yields to explicit spellings); claimed set covers page names and table rows and validates the main-page proposal; `insert_state_line` detects flow/block shape from the envelope line's own syntax, splices at measured indentation, edits flow JSON in place — no intermediate document parse |
| Preparation input | `b5102f87` | Stale plans spliced silently; out-of-bounds/non-UTF-8 edits panicked in `replace_range`; reparse `expect` panicked | Plans bind their exact source (in-memory, never persisted); apply validates operand equality, edit bounds, UTF-8 alignment, ordering/overlap; reparse proof is a contextual error; no-op holds only for the matching source |
| Tracking settlement + binding continuity | `5f146b5a` | First bind with node promotion and repeat settlements failed as "authored content" changes; token-only binding comparison | New `Document::authored_value()` (metadata, pages, source identities) replaces the semantic-value comparison; established bindings preserved whole, never dropped; main-page identity changes refused in both directions |
| Status states | `b65f1903` | `bound-no-receipt` unreachable (reported `unsupported`); duplicate unbound UUIDs silently `unbound` | Receipts read from the typed parse (explicit `sync: null` = absent); duplicates precede every early return; bound-no-receipt reports baseline `none` with no fetch; offline never claims clean |
| Flow byte preservation | `ab84ca8a` | — (refinement of the envelope row) | Flow-envelope member insertion preserves the mapping's own separator style instead of re-serializing; CLI fixtures unchanged |

## API changes the watch implementer must know

- **`KeyPreparation` gained `pub source: String`** and is bound to it:
  `apply_key_preparation` rejects any other operand (including
  same-length alterations) with a re-plan error, and no-op semantics hold
  only for the matching source. Build → apply must pass the same bytes.
- **`KeyPreparation.edits` may contain retarget replacements**
  (`start < end`) for page links addressing the provisional main page, not
  only Key-marker insertions. Supported spellings: `page={json}`,
  `page="…"`, `page='…'`; anything else fails planning loudly.
- **`Parsed::provisional_main_key() -> Option<&str>`** is the way to learn
  the provisional main-page spelling; the `PROVISIONAL_PAGE` crate alias is
  gone, and the provisional main key can be `p-auto-2`, `p-auto-3`, … when
  `p-auto` is explicitly owned elsewhere.
- **`prepare_keys` validates the main-page proposal** (spelling, collisions
  with page names, keyed blocks and confirmed/retired node rows) exactly
  like block proposals.
- **`Document::authored_value()`** is the settlement comparison surface;
  `patch_tracking` now preserves an established binding whole
  (origin/token/root/entity set/entry fact/publication), refuses dropping
  it, and refuses changing `mainPage` in either direction (preparation
  owns persisting it).
- **`editor_value` new associations carry the source key**, so receipt
  hashes change for any document with unconfirmed objects. Renaming a
  *new* (unrowed) key churns the value — accepted, per the review's
  "explicit source identity for prepared-new objects"; retained-key
  renames remain churn-free.
- **`insert_state_line` no longer parses the document**: flow envelopes
  keep their spelling and spacing (one member inserted), block envelopes
  keep every line byte-for-byte. Flow values that are not literal JSON on
  their own line (trailing comments) fail honestly.
- **`status` on a bound document without receipts** now yields
  `bound-no-receipt` (baseline `none`, no remote fetch); duplicate
  identities are `ambiguous` even while unbound.

## Verification

- Full workspace `cargo nextest run`: **200 tests, all passing** — this
  includes the concurrent `web::live` transport tests present in the
  shared working copy (not mine). The crates this pass owns: 51
  document/mdx, 61 workflow, 22 CLI integration tests (pull/remote/tidy),
  3 editor observation. `cargo fmt` and `cargo clippy --workspace
  --all-targets` are clean.
- The predecessor baseline before any change was 99 passing tests across
  document/model, document/mdx and workflow. The batch A receipt's
  durable claim was 175 workspace tests; the prior session's final chat
  said 173. This pass reports its own actual counts above.
- Each correction row has a regression test that fails on the reviewed
  code (demonstrated before fixing where feasible; the envelope-shape
  test was additionally run against extracted pre-correction sources and
  panics at the former `expect` on `keys.rs:159`).

## Remaining concrete concerns

- PageLink retargeting recognizes exactly three attribute spellings
  (`page={json}`, `page="…"`, `page='…'`, no whitespace around `=`); other
  spellings fail preparation planning with an explicit error rather than
  guessing. The reparse proof gates any misreplacement.
- A flow envelope followed by a trailing comment on the same line fails
  honestly (`not literal JSON`) instead of regenerating the envelope;
  regenerating would discard the author's flow spelling, which this pass
  deliberately preserves.
- `status` does not observe the remote for bound-no-receipt documents
  (matching the original outcome design); the watch integration should
  treat it as its own state rather than deriving direction from absence.
- Renaming a new, unconfirmed key still churns `editor_value` (its
  identity *is* the source key until confirmed). `keys rename` (later
  batch) is the deliberate surface for that transition.
