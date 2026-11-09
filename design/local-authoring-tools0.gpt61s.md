---
type: ImplementationReceipt
title: "Offline authoring tools without new remote authority"
description: "Version-0.18 selected metadata editing, native-preserving source-key rename and truthful local inventory; source-coordinate boundaries, lock/receipt policy and executable verification."
resource: /design/local-authoring-tools0.gpt61s.md
status: implemented-awaiting-parent-review
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3d511cf0ffeKLT5XsWr37v0r6"
    title: "Fresh Sol commissioned offline metadata/rekey/list batch, pure source patching, local locks/identity inventory, actual CLI and HTTP/Phoenix verification; no live corpus or remote effects"
  - resource: /AGENTS.md
    title: "Bounded complexity, native continuity, fresh-Sol and rollover policy"
  - resource: /design/local-authoring-tools0.gpt6a.md
    title: "Parent commissioned design and acceptance boundaries, received during implementation"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted command purposes and operation-specific source ownership"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Author-field presence/clear/date and deliberate alias semantics"
  - resource: /design/terminal-recovery0.gpt61s.md
    title: "Received pending, writer and lastResolution protections; no-op/default evidence"
  - resource: /design/keyed-save0.gpt61s.md
    title: "Receiving complete native save/compiler and confirmed source-key-independent editor projection"
  - resource: /design/connected-watch0.gpt61s.md
    title: "Shared stable-ID locks, duplicate selection and follower preservation boundary"
  - resource: /design/authoring-corrections0.glm53m.md
    title: "Source-bound edits, interleaved identity allocation and actual parser spans"
  - resource: /design/native-default-adoption0.gpt61s.md
    title: "Exactly absent/null/empty description clear semantics, not whitespace trimming"
  - resource: /crates/document/mdx/src/author.rs
    title: "Pure selected-field metadata patch and whole-output proof"
  - resource: /crates/document/mdx/src/rekey.rs
    title: "Current explicit source-identity rename, native association continuity and full reparse"
  - resource: /crates/document/mdx/src/source.rs
    title: "Existing serde-saphyr key spans and descending validated byte edits"
  - resource: /crates/document/mdx/src/parse.rs
    title: "Production arena selection of actual JSX properties and bounded opening-tag token extents"
  - resource: /crates/workflow/src/author.rs
    title: "Shared locked offline transaction, original-byte guard and late duplicate check"
  - resource: /crates/workflow/src/list.rs
    title: "Read-only inventory report and partial header evidence"
  - resource: /crates/cli/tests/push.rs
    title: "Actual rename/no-effect push/clean status, metadata scopes and pending/provenance fixtures"
  - resource: /crates/workflow/src/watch/tests.rs
    title: "Actual HTTP/Phoenix renamed-key adoption, helper lock contention and dirty metadata preservation"
---

# Offline authoring tools

## Outcome and receiving surfaces

Coordinated **0.18.0** implements the three commissioned tickets:
`leafbud-author-metadata`, `leafbud-author-rekey`, `leafbud-inspect-list`.
Implementation checkpoint **`b5c2aa5e`**; expanded watch/source tests and final
documentation/verification follow separately. Parent acceptance is not implied
by implementation or tracker closure.

The [README](/README.md#offline-metadata-key-names-and-local-inventory) owns usage,
tag spelling, clear/date/filter/exit policy and source-shape limitations. These are
local commands, not a remote publication, authentication or release capability.

| Owner | API / implementation |
| --- | --- |
| MDX | `MetadataEdit`, `edit_metadata(source, &edit) -> Result<String>` |
| MDX | `rename_key(source, old, new) -> Result<String>` |
| MDX source adapter | Existing serde-saphyr `Spanned` key/value locations; its reexported granit parser supplies actual comment events. No new YAML grammar/dependency |
| MDX parser | Actual Satteri JSX nodes/decoded string property meaning choose replacements; bounded opening-tag token scanner locates the selected value only |
| Workflow | `author::edit` / `author::rename` return path, UUID and changed/no-op; existing locks/files/inventory own effects |
| Workflow | `list::inspect(context, Request { state }) -> Report`, version 1, per-file partial header evidence/diagnostics |
| CLI | Thin offline `edit`, `keys rename`, `list --state/--json` adapters; `--tag` repeats replace one list, `--clear-tags` writes `[]` |

Workspace version, exact internal dependency pins and root lock are coordinated.
Installed `cargo bump` 1.1.0 explicitly panics on workspaces; the root workspace
version/pins were updated directly and Cargo refreshed the lock. No migration,
new crate, arbitrary property setter or new comparison/planning framework exists.

## Metadata: intentionally selected scope

Omitted flags mean leave alone. Changed title/description/tags are editor inputs;
`publishedAt` is local release intent only. Dates require an explicit offset and
are normalized through Jiff; `--now` samples Jiff once, and Clap excludes the two
date alternatives. No publication target/path/unknown property is exposed.

Description clear is exactly absence/null/empty, never trimming. A helper clear
on any already-cleared spelling is a byte no-op; clearing a nonempty description
writes the explicitly selected empty string. Empty tag replacement explicitly
writes `[]`, even from absence. Tag order/duplicates/whitespace are not normalized.
Invalid existing/selected known field types or datetimes block before writes.

The source adapter gets key positions from the existing YAML parser, not line
regex guesses. Scalar/collection spans do **not** all cover full source values:
the tested sequence span is its opening bracket, and a block scalar span omits
its indicator. The helper therefore replaces a selected block-mapping entry
between parser key boundaries, retaining parser-identified comments and blank
separators. Unknown multiline author content cannot impersonate a key/comment.
Changed-entry inline comments may become standalone lines; unselected fields,
comments, exact body, BOM and fence newline style survive.

This is deliberately not a lossless whole-YAML editor. A flow-style whole root
or other unsupported patch shape is refused with no partial rewrite. Inserting
an absent selected field requires the envelope's block-style top-level key.
Every proposed output reparses and its header equals exactly the original plus
selected replacements; state/pages and exact body bytes are independently checked.
The no-op branch also validates the actual source and pending/known-date policy.

## Rename: alias edits do not accept new content

One explicit current block Key, named Page or persisted `mainPage` changes label,
along with actual PageLink targets and its confirmed node-map key. Missing,
provisional-only or retired-only old identities are not resurrected. New spelling
uses the receiving ASCII key grammar; collision checks cover all current pages/
blocks and confirmed/retired rows. `OLD == NEW` still validates source, pending,
old existence and spelling, then returns exact bytes.

Satteri selects actual JSX elements and decoded property values. The token-range
adapter stays inside those validated opening tags, handles literal quotes, JSON
expressions, escaped string values and whitespace around properties, and changes
only the selected attribute value. It does not search prose, code, metadata, text
facts or arbitrary components for the old word. YAML map-key and mainPage value
edits use parser spans; flow-style envelopes/node maps work without normalization.
Block-scalar `mainPage` requires explicit reshaping first. If provisional identity
allocation would shift, reparse fails closed with a `tidy --keys` hint.

The reparsed model must equal exactly the expected renamed document/state. Native
entities, placement/reference facts, active/inactive text facts, UUID, whole binding,
writer, sync and historical `lastResolution` remain unchanged. Confirmed native
associations keep the editor projection, but a new unconfirmed key's spelling is
its identity and can change the projection. No receipt is recomputed to hide drift.

**Integration correction:** an existing unchanged tracking settlement always
serialized its envelope, so surgical rename's retained textual row order caused
an otherwise empty push to rewrite bytes. `patch_tracking` now returns original
source when the validated old/new states are equal, **after all its existing
identity/binding/writer/pending/main-page guards**. This does not broaden settlement
authority. The actual CLI/HTTP rename → preview/repeat push → clean status fixture
checks empty effects, stable receipts/writer and exact source bytes/mtime.

## Local effects and inventory evidence

Both helpers resolve canonical operands, read UUID, acquire the existing shared
OS lock, reread the file/identity, inspect full workspace plus external-operand
duplicates, compute/validate proposed bytes, repeat duplicate/path checks and use
the existing adjacent original-byte atomic write guard. Arbitrary editors are not
made participants in a filesystem CAS; intervening bytes are preserved. Actual
no-ops do not replace the file. Watch/push/pull use the same UUID lock.

**Pending policy:** both helpers refuse any pending attempt with a recovery hint.
They never rewrite an old input/desired hash, target map, pending disposition or
resolution provenance. Unknown future owned semantics remain a loud preserved
refusal, not an excuse to drop the state. Manual source editing remains possible.

List reuses `inventory::scan(root, None)`, including global identity accounting
before filtering; no second recursive scanner or database. Sorted canonical-root
regular `.mdx` candidates are reported; symlink files/directories are not followed.
Broken/unsupported/ambiguous candidates remain visible under state filters.
Unmanaged sources are counted, not converted. Published headers remain unsupported
for managed authoring and cannot claim release adoption.

Header title/UUID/lifecycle/format/profile and native locator can still be useful
when body or future owned state is invalid. `valid` describes only a full current
supported parse, not remote state or uniqueness. Separate duplicate-ID/binding
flags and outcomes expose ambiguity. `binding` is explicitly header evidence;
`editing_locator` distinguishes header binding from a located mint's pending
evidence. Pending kind/phase and resolution presence/direction are bounded
summaries, not raw bodies/credentials/provenance logs. Read-only list creates no
lock/config/key/source state and does not resolve native credentials at all.

## Verification and limits

All fixtures are synthetic temporary sources or loopback HTTP/Phoenix. No live
`~/l`, P1, standalone draft, deployed credentials or creation budget was touched.
No new live or independent-Yjs claim: that unchanged writer evidence remains in
the [received save receipt](/design/keyed-save0.gpt61s.md).

- Pure MDX: each selected field alone, ordered/clear tags, semantic description
  no-op/whitespace, title/type/date validation, exact unrelated fields/body,
  comments/unknown YAML, BOM/CRLF, flow-root metadata refusal, main/named/block
  renames, forward/multiple links, interleaved definitions, escaped expression
  strings, prose/code lookalikes, inactive text, retired collisions and flow state.
- Workflow: byte/mtime no-ops, shared stable lock, source changed during prepare,
  duplicate introduced after prepare, external copied UUID and symlink aliases;
  deterministic/read-only list, malformed/legacy/unsupported/duplicate headers,
  body-invalid header evidence, global duplicates across state filter and file moves.
- CLI: repeatable tags versus clear, date exclusion/offset/now, usage/refusal exits,
  config workspace selection and list with deliberately invalid token/origin env;
  absence of list lock/config writes; versioned JSON and mixed human reports.
- Actual CLI/HTTP: confirmed block/page/main renames → unchanged preview/push and
  clean current status with no native effects or byte/mtime change; each editor
  metadata field reports local-changed without accepting it, date-only stays clean.
  Actual partial pending blocks helpers with a recovery hint; accepted resolution
  survives edit/rename, while future unknown resolution blocks without writes.
- Actual HTTP/Phoenix: the real rename helper preserves its label through later
  remote adoption; in-flight watch rejects both helpers as busy; local metadata
  is reported LocalDirty then NeedsDecision on remote drift, with zero source writes.

Final gate: **317/317 regular workspace nextest tests pass**, six explicitly
opt-in deployment tests skipped; `cargo fmt --all --check`, strict workspace
all-target Clippy and README/receipt Markdown + OKF checks pass (2 documents,
49 links, 31 sources). Commit references are recorded in the completion report/tickets.
The supported profile/lifecycle and source-shape limits are not widened by listing.

## Parent join and road ahead

Parent review should focus on selected-entry/comment ownership, bounded JSX token
extents and the unchanged-state settlement fast path, rather than re-running
already received live/CRDT evidence. Relevant current homes: the [README](/README.md),
the [commissioned parent design](local-authoring-tools0.gpt6a.md) and its
[next-fronts handoff](next-fronts0.gpt6a.md). This receipt refines those design choices
with actual API names, source-shape refusals and executable evidence.

Recommended next **fresh Sol high** batch: `leafbud-mdx-source` plus
`leafbud-mdx-coverage`. Together they can broaden a deliberately selected small
corpus (not a general formatter), and make source/content × operation coverage
visible. This batch removed whitespace/escaped JSX rename and trailing-comment
hazards for these local helpers, but whole-flow-root metadata edits, old preparation
matchers, body comments and wider patch fidelity remain separate seams. Then
`leafbud-author-names` can use these local transaction/presence patterns to finish
independent filename/path-intent reconciliation. These are recommendations, not a
new dispatch; release/email/upload scopes still require their own commission.
