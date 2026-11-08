---
type: Report
title: "Transient human Markdown comparison through an external tool"
description: "Delivered current local/editor reading views and a foreground executable/argv/exit contract, without structural diff, edit planning, receipt settlement or source mutation. Synthetic verification and the separately owned live rename prerequisite remain explicit."
resource: /design/human-diff0.gpt61s.md
status: implemented
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e43700769ffery3DQFUKzsttVS"
    title: "Sol high implementation: comparison renderer, foreground process/config/CLI contract, synthetic tests, documentation and checkpoints"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Parent commission separating human diff from independent native-edit inquiry; subsequent Leafbud rename steering assigns migration to another session"
  - resource: /AGENTS.md
    title: "Low-volume bounded-complexity product fit and distinct human comparison purpose"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted root diff edge/editor default, read-only effects and endpoint meanings"
  - resource: /design/status-witnesses0.gpt61s.md
    title: "Shared current-editor observation, scoped receipt distinction and last verified live namespace/store gap"
  - resource: /design/markdown-roundtrip1.glm53m.md
    title: "Existing checked Markdown renderer and exact supported-meaning fallbacks reused here"
  - resource: /README.md#human-localeditor-diff
    title: "Maintained user-facing view, tool, exit, privacy and lifecycle contract"
---

# Human diff implementation handoff

## Outcome and checkpoints

`leafspring diff FILE [--edge editor]` now invokes a trusted user-selected
foreground diff/viewer with two transient normalized Markdown views. It reads
the current authored source and current **bound native editor**, not public output,
an ancestor or saved body. Different content is a successfully displayed comparison,
not a conflict-resolution decision. No custom line diff, alignment, semantic change
tree, edit planner, Yrs mutation, native save/publish or operation explanation was
implemented. The independent native-edit designers' same-wave drafts were not read.

- `87817057` — human reading-view renderer, shared checked block rendering and
  independent content/metadata/key/page/exact-fallback assertions.
- `51f0c31f` — shared observation → comparable views → configured foreground tool,
  direct argv, explicit exit modes, CLI/help/config and loopback/subprocess tests.
- `2a8d4b1e` — readable collision-safe code fences when exact parser validation
  succeeds; exact component fallback otherwise; canonical absolute temp operands.
- Final owned checkpoint adds the maintained contract, this receipt, coordinated
  **0.9.0**, exact internal pins and regenerated root lockfile.

The expected help was small and immediate: let authors use their existing reading
and comparison tools without exposing receipts as apparent writing changes. The
synthetics demonstrate this: literal MDX and Markdown serialization/key renames
produce equal views; actual prose/marks differ and arrive at the correctly labeled
tool operands. There is no live usability claim over the current personal corpus.

## Narrow interface and representation choice

The maintained [README contract](../README.md#human-localeditor-diff) is normative
for the delivered capability. Receiving seams:

- [MDX `render_comparison`](../crates/document/mdx/src/view.rs): one public human
  view renderer; no second semantic document model. It validates the complete
  supported typed input with the existing round-trip writer and reuses the same
  checked block serializer. Header includes title, description, ordered tags,
  confirmed publication, watermark and contributors. Unsupported Markdown meaning
  remains exact typed components, rather than being flattened silently. Code uses
  a collision-safe fence only when the same parser proves exact text/language;
  unrepresentable info/line endings retain the `Code` component. This presentation
  choice does not change the import/pull writer's exact code-component policy.
- [Workflow `diff`](../crates/workflow/src/diff.rs): one command adapter plus small
  tool/options/result types. Local selection/index guards reuse inventory; remote
  acquisition, lifecycle/identity checks and supported mapping reuse
  [`observe_editor`](../crates/workflow/src/editor_state.rs). Both endpoint models
  use the same view writer. The adopted remote model's carried local-only fields
  are intentionally excluded by the view writer, not misreported as native data.
- [Config](../crates/workflow/src/config.rs): existing Figment2 layers and
  toml_edit preservation, with `diff-program`, `diff-args` and `diff-exit`. The
  argument array replaces lower-layer arrays, including explicit empty arrays.
  `config set diff-args` uses JSON-array input; it does not parse shell syntax.
- [CLI](../crates/cli/src/diff.rs): foreground adapter, help/example and a short
  stderr execution summary. No `--json`: arbitrary inherited tool output cannot
  truthfully be a structured Leafspring diff report.

The entire tool envelope, source labels and local-only dates/delivery/naming/custom
fields are absent from **both** views. They survive unchanged in the source.
Entry page is `page-1`; other definitions sort by native association and receive
ordinal presentation labels, also used by page links. Native sorting coordinates
are never printed. These labels are neither saved nor durable; page addition/removal
can renumber them. Block order, page boundaries and link targets remain visible.
Authored URIs/DIDs/image URLs are content and remain visible, unlike tool bindings.

Human-view equality deliberately excludes native object associations, inactive
native text and receipt state. Equal writing can coexist with a status drift
report. This adapter never settles that report or converts legacy receipts.

## Tool/process choice

The contract is `PROGRAM ARG... LOCAL.md EDITOR.md`, always local left/editor
right. There is no shell, placeholder language, tool-name inference or plugin API.
Spaces/punctuation remain literal argv entries. The process runs in the configured
workspace with inherited stdin/stdout/stderr/environment; users must trust the tool
and the effective workspace configuration. It receives writing, not credentials
or bindings embedded in the view (normal inherited environment remains its own
trust boundary).

Exit modes are deliberately explicit:

- `success` (default): tool exit 0 accepted; useful for interactive viewers.
- `diff`: POSIX diff convention 0/1 accepted, meaning equal/different in that tool.
- Other tool codes, signal, spawn/wait or cleanup errors: Leafspring exit 1.
- Either accepted tool code yields Leafspring exit 0; CLI usage errors give 2.
  Leafspring's own equality summary is based on rendered strings, not tool exits.

There is no default executable. Configure one or supply `--tool`; help demonstrates
`diff -u`. `--tool-arg` replaces saved args when present; changing only `--tool`
retains saved args. `--tool-exit` overrides the configured convention.

`tempfile` owns a private directory; the absolute operands end in `local.md` and
`editor.md`. Tokio waits for the process with kill-on-drop for ordinary future
cancellation. Normal completion explicitly removes the directory, including tool
error exits; RAII covers ordinary early errors. Detached GUI launchers are not
supported—select the viewer's wait/foreground option. Hard termination can leave
temporary files, and viewers can create their own backups/caches or arbitrary
effects. No automatic cleanup/pruning machinery was added. A view edit is discarded,
never a save/merge into source or native content.

## Verification and live boundary

Workspace fmt, strict all-target Clippy and nextest: **148/148 tests pass**
(137 baseline plus 11 new tests). Existing pull/status regression remains intact;
one old test expecting the diff scaffold was updated to expect missing tool
configuration instead. Checks cover:

- Independent expected text/marks/metadata, headings, breaks, code/image/empty-block
  exactness; no receipts/keys/native bindings/local-only values in the view.
- Equal views across source-key spelling, literal-vs-Markdown serialization and
  non-entry page-definition ordering; correctly rewritten presentation page links.
- Equal, differing and converged current endpoint views with unchanged source
  bytes and mtimes, thus unchanged bindings/receipts. No native mutation route.
- Missing/inaccessible remote, unsupported/opaque content or lifecycle, unknown
  profile/namespace, incoherent/foreign/duplicate/external binding guards and local
  network-read race; refused views/errors do not reflect private source/response.
- Synthetic executables and paths/args containing spaces and shell punctuation,
  local/editor operand order, inherited stdout/stderr, workspace cwd, explicit
  0/1/2 modes, missing executable, child signal and temp deletion after success or
  tool-error exit. CLI configuration/overrides, no fake JSON, usage errors and
  explicit unsupported names/release edges.

All exercises used synthetic current-namespace models/fixtures and loopback HTTP.
No real document, credential store or native editor was written or migrated.
The predecessor status receipt's last live observation found all 27 `~/l` files
in the old Springleaf namespace/stores. This session did **not** re-read or convert
that corpus. The user subsequently announced another session's ongoing **Leafbud**
code/config/existing-data rename. That session owns the migration; live comparison
remains unverified until the selected binary, namespace, configured root and stores
agree. This implementation neither provides compatibility shims nor authorizes
receipt rebaselining. Parent/rename owner should integrate the few newly introduced
names/help strings as needed, without treating these historical receipts as current
runtime configuration.

## Parent review and road ahead

First review the small human view/tool contract independently from edit architecture.
For **`leafspring-editor-plan-design`**, join the commissioned Astra-high/g independent
drafts and cross-reviews before selecting or commissioning native-write work; no
recommendation here presumes their result. After that choice,
**`leafspring-editor-text`** is the first small existing-block write seam, proposed
Sol high if identity/Yrs policy remains unresolved, otherwise g for a concrete
approved slice. Pairing it later with **`leafspring-editor-conflicts`** would equip
guarded preview/read-back rather than silently broadening this comparison API.
Separately, **`leafspring-inspect-list`** is a bounded g task reusing local inventory,
useful without any native-write decision. These are recommendations, not new
implementation commissions; live rename integration remains owned elsewhere.
