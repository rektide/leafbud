---
type: Report
title: "Batch A implementation: offline create and one-time source keys"
description: "Receipt for leafbud-author-create + leafbud-author-keys: format leafbud-mdx/2, implicit main pages, tri-state publication intent, typed occurrence locators, flat lists, preparation with proof, tracking-only settlement, and coherent create/check/status/pull/tidy surfaces."
resource: /design/author-create-keys0.glm53m.md
status: draft
generated:
  at: 2026-11-09
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e41f88509ffejyMz0pF3zg73r3"
    title: "Implemented batch A end to end: model/MDX rebuild, editor list observation, workflow create/tidy-keys, CLI surfaces, tests and this receipt"
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Parent commission: build batch A per the grounded build path, with concrete scope, edge cases and honest native-push boundary"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Primary build-path design; §§1–4 and batch A scope enacted here"
  - resource: /design/keyed-edit-flows0.gpt6a.md
    title: "Behavioral flows this batch's parse/preparation semantics support"
  - resource: /design/keyed-authoring0.gpt6a.md
    title: "Complete-drafting boundary and keyed native-object foundation"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted command meanings, tidy pass selection and YAML policy"
  - resource: /crates/document/model/src/lib.rs
    title: "Format-2 model, tri-state publication, typed occurrences, editor value v3"
  - resource: /crates/document/mdx/src/parse.rs
    title: "Implicit-main-page grammar, lists, unkeyed blocks, provisional identities"
  - resource: /crates/document/mdx/src/keys.rs
    title: "Preparation planning, insertion edits and applied proof"
  - resource: /crates/document/mdx/src/patch.rs
    title: "patch_tracking / adopt_remote split and envelope line insertion"
  - resource: /crates/leaflet/editor/src/lib.rs
    title: "Flat-list observation and typed locators over native facts"
  - resource: /crates/workflow/src/create.rs
    title: "Offline no-clobber draft creation"
  - resource: /crates/workflow/src/tidy.rs
    title: "Executed key preparation and pass selection"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "Re-read native list facts: block/is-list, block/list-style, block/check-list set in the same mutation as block/type"
---

# Batch A implementation: offline create and one-time source keys

Commissioned batch A of the [grounded build path](keyed-build-path0.gpt6a.md):
`leafbud-author-create` + `leafbud-author-keys`, executed as four commits
(model/MDX, editor observation, workflow/CLI, version/docs). This is an
offline create/check/tidy foundation — **not** a claim of delivered native
authoring: push, binding and native writes remain later batches.

## What landed

| Area | Change |
| --- | --- |
| Format | `leafbud-mdx/2` only. Format 1 has no compatibility surface; `~/l` is disposable and a fresh `import` replaces it. |
| Model | Optional binding/receipts (bound-without-receipt coherent, receipts-without-binding rejected), `mainPage` persisted identity, tri-state publication intent (absent/inherit, null/standalone, URI), typed `Occurrence::Block{placement, textFact, inactiveTextFact}` / `Occurrence::Page{references}` locators with strict node decoding, `ListItem` blocks. |
| Editor value | Profile `leaflet-editor/3`: retained rows by locator (spelling-free), new keys as explicit `{"kind":"new"}`, locally-retired rows as explicit pending omissions. Key renames and provisional identities do not churn it. |
| MDX crate | Split into `parse`/`render`/`keys`/`patch`/`view` modules. Implicit main page; named `Page` definitions anywhere at top level; forward page references; unkeyed blocks are new content with deterministic `b-auto-N` provisional keys; multi-item authored lists expand to one keyed block per item; task/nested lists diagnosed. |
| Preparation | `prepare_keys(source, KeySource)` proposes `b-<uuid4>` insertions (file-newline aware) plus a `p-<uuid4>` main-page key; `apply_key_preparation` proves the result through the production parser with the same proposed IDs and is an exact no-op when nothing is missing. Plans never mutate. |
| Patch split | `patch_tracking(source, next_state)` settles only the `@leafbud` envelope (first-binding `None→Some` allowed, rebinding refused, body bytes untouched) — the API future push uses so successful saves never reformat authored bodies. `adopt_remote` keeps the old author-field+body behavior for pull. |
| Editor | Native text blocks with `block/is-list` observe as `ListItem`s (bullet default, ordered via `block/list-style`); `block/check-list`, `block/list-number`, heading items and nested container content are loud diagnostics (facts re-read at Leaflet `600c7b5f`). |
| Workflow | Offline `create` (stable random v4 UUID, no-clobber, symlink-safe, empty implicit main page, tri-state intent via `--standalone`/global `--publication`), executed `tidy --keys` plus all-passes default (`pass: all|keys|markdown`), preparation-aware `check`, status outcomes `unbound`/`bound-no-receipt` with `needs_preparation` counts, import building format-2 files that keep locally assigned UUIDs and source spelling, pull/diff/status no longer require the token-derived UUID equality. |
| Version | Coordinated 0.10.0 → 0.11.0 with exact internal pins (public model/format API change). |

## Verification

- `cargo nextest run`: **175 tests, all passing** across the workspace
  (38 document/mdx incl. the round-trip corpus, 3 editor observation, 59
  workflow lib incl. loopback pull/status/import/tidy, 23 CLI integration,
  plus atproto/web/records regression suites). `cargo fmt` and
  `cargo clippy --workspace --all-targets` are clean.
- Behavioral edges covered by tests: missing keys/new empty blocks/retired
  last block (empty main page valid), split/join/order without identity
  inference, forward page references, list key syntax and list-break
  continuation indents, unbound tri-state through render/parse/adoption,
  no-op/dry-run/source-race guards, UUID independence under observation,
  renamed retained keys leaving editor identity unchanged, CRLF/BOM
  preparation and tidy, multi-item list splitting, first-binding settlement
  and rebinding refusal, unknown profile/field refusal, and the full
  existing read workflow (import/pull/status/diff) regressions through the
  format-2 model.
- No live remote writes; no credentials touched; `~/l` untouched.

## Judgment calls the parent should review

1. **Multi-item authored lists parse as per-item new blocks** rather than
   erroring: `- a\n- b` without markers yields two provisional-keyed items
   needing preparation. The alternative (error with a tidy hint) was
   rejected because unkeyed prose must parse; splitting is the same case.
2. **Main-page key lives in the envelope** (`@leafbud.mainPage`), assigned
   at preparation; the body never carries a main-page marker. Hand-authored
   files can omit it until their first preparation.
3. **Provisional identities are deterministic** (`b-auto-N`, `p-auto`) and
   excluded from editor value spelling, so check/status never hash random
   allocations as durable identity; preparation replaces them one-time.
4. **List-number display overrides** (`block/list-number`) are rejected on
   observation, and source ordered markers normalize to `1.`: the native
   model sequences display numbering itself, and no override mapping is
   modeled yet.
5. **`render_comparison` accepts new-keyed documents** (associations render
   as unconfirmed rather than erroring) since unbound drafts are now valid
   diff operands conceptually; diff still requires a binding to compare.

## Known gaps (deliberate)

- No naming tidy (`--from`/`--accept` still rejected honestly), no `keys
  rename`, no push/publish, no `list`/`edit`/`export`/`pubs`/`auth`.
- Nested/task lists and heading list items remain diagnostics in both
  directions; native `group`/canvas blocks unchanged (unsupported).
- `block/list-number` overrides make a document unrepresentable; if real
  drafts commonly carry them, a display-override mapping is a small follow-up.
- Preparation assigns keys only for the main page and blocks; named pages
  already carry explicit names (no assignment needed).
- The `~/l` 27-file corpus is stale by format; nothing migrated it.

## Handoff to the next session

Batch B (connected watch) and batch C (native transport) build on:
`prepare_keys`/`apply_key_preparation` as the push-side preparation API,
`patch_tracking` for post-save settlement, `Document::editor_value()` (v3)
as the agreement baseline, and `leaflet/editor::observe` with typed
locators as the observation input. The shared protocol crate remains
uncreated per the build-path plan (no empty scaffolding).
