---
type: Report
title: "Scoped editor witnesses and read-only status"
description: "Delivered separate editor receipt scope and read-only status without diff/planning APIs; records safe legacy interpretation, synthetic verification and the actual live namespace/store gap."
resource: /design/status-witnesses0.gpt61s.md
status: implemented
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e438cd26fffedgKcnaMrNRqncp"
    title: "Commissioned Sol high implementation, receipt-scope checkpoint, status integration, synthetic checks and read-only verification of all 27 legacy live files"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Parent commission: witnesses/status only, no diff or operation-explanation work; sequential fresh Astra medium designs follow parent review"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted ownership, root status, YAML and current-endpoint effect contract"
  - resource: /design/guarded-pull0.glm53m.md
    title: "Predecessor guarded refresh and the whole-extra false-conflict limitation corrected here"
  - resource: /design/draft-checkout0.gpt61s.md
    title: "Existing import receipts and no-snapshot live checkout"
  - resource: /design/markdown-roundtrip1.glm53m.md
    title: "Production parser/writer/tidy and unknown-owned-state refusal preserved"
  - resource: /README.md#read-only-localeditor-status
    title: "Current CLI, JSON schema, outcome/freshness and exit contract"
---

# Witnesses and status handoff

## Delivered checkpoints

- `61aed9f6` — separate editor input from full-document fidelity; shared
  observation/classification used by guarded pull; safe legacy interpretation.
- `ac98904f` — read-only status workflow/CLI, per-file inventory diagnostics,
  conservative unsupported evidence, rename-gap guards; coordinated **0.8.0**
  and exact internal pins/root lockfile. Later owned documentation/tracker
  checkpoint completes the receipt.
- `acc4e438` — review correction: match native associations before key
  spelling, reserve all matched labels before naming new nodes, and prove
  label permutations/new-node collisions cannot retarget or drop associations.

No diff, alignment, semantic change tree, operation plan/explanation API,
published import/save/release, sidecar, ancestor or CRDT snapshot was added.
The upcoming designers retain architectural choice.

## Scope and receipt choice

`Document::semantic_value` remains the full fidelity comparison used by MDX
round-trip verification. `Document::editor_value` is separate. Profile
**`leaflet-editor/2`** fingerprints title, description, tags, publication
association, watermark, contributors, supported block/inline meaning, main
page, block order, page targets and remote object/occurrence associations.
It substitutes association rows for author-facing source labels and sorts
non-entry page definitions. A key rename with its references/node row, layout
normalization or a custom/date/delivery/naming-intent field is not an editor edit.
Source associations changing under identical prose are still editor input.
Full serialization fidelity continues to retain all local-only fields and keys.

The editor's raw witness excludes only known transport timestamps, row version
and indexes; it retains all other fact fields/payloads and extras, including
opaque/inactive text, with native root/title/description/association inputs
supplied by workflow. Raw evidence is hashed **before** conversion. Unknown
meaningful content/owned fields block adoption, not silently typed-drop.
Row/extra transport and representation scope are not publication receipts.

`leaflet-import/1` retains its old hash algorithm. Its local hash matching
proves the full input unchanged; a mismatch cannot distinguish local-only from
editor changes. Differing current endpoints therefore remain **unverified**,
not guessed remote-only or both-changed. Equal supported current endpoints
can be observed as **converged**; an explicit guarded `pull` establishes new
current correspondence and writes only receipts. No historical body is
reconstructed. `status` never performs that settlement. Explicit `--take remote`
is still the separate content decision for supported differing endpoints;
unknown profiles/owned-state semantics remain blocked. Malformed hashes give
unknown evidence, not invented change direction.

The bound association must be explicit null/URI; absence is refused rather
than silently treated as standalone. Binding/token/root/entity-set/entry and
deterministic imported-document identity guards remain distinct from hashes.
No native identity is inferred from a filename or ambient publication default.

## Receiving code and behavior

- [Model](../crates/document/model/src/lib.rs): pure scoped input and ownership.
- [Editor](../crates/leaflet/editor/src/lib.rs): conservative raw witness,
  strict representation capability; the Yrs decoder is unchanged.
- [Workflow editor state](../crates/workflow/src/editor_state.rs): current
  acquisition, identity checks, existing-key reconciliation/adoption and scalar
  agreement classifications. Transport remains in web. This is deliberately
  not a reusable comparison tree or planner.
- [Status](../crates/workflow/src/status.rs) and
  [inventory](../crates/workflow/src/inventory.rs): local selection, explicit
  per-file uncertainty/errors, freshness and read-race detection. Offline
  takes no client or credentials. Naming/release remain independently unknown.
- [Pull](../crates/workflow/src/pull.rs): maps the same observation into its
  existing effect policy. Original-byte atomic write guard, identity recheck,
  dry-run, exact no-op bytes/mtime, local-only YAML and key spelling survive.
- [CLI status](../crates/cli/src/status.rs): thin human/JSON rendering and
  accepted status exit semantics. [README](../README.md#read-only-localeditor-status)
  owns the version-1 report schema and full outcome/freshness vocabulary.

## Verification and live limitation

`cargo fmt --all`, strict workspace/all-target Clippy (`-D warnings`), workspace
nextest: **137/137 pass** (115 baseline, 22 additional tests; existing conflict
regression now includes a genuine local body edit rather than relying on key
spelling/local-only metadata to create a false conflict). Existing fixtures,
real Yrs payload helpers and the shared loopback responder were reused.
Coverage includes isolated text/marks/moves/association changes, page targets
and definition layout, local-only metadata plus remote edits, true conflicts,
legacy settlement and unknown/malformed receipts, offline versus fresh evidence,
convergence without writes, no-op and local race guards, malformed files and
missing operands, duplicate document IDs/bindings (including external operands), missing
native objects, unknown opaque fields, origin/rights/lifecycle limits, JSON,
human/privacy/exit behavior and rename duplicate prevention.

**Live facts differ from the commission's assumed rename completion.** All 27
files in `~/l` still have `springleaf-mdx/1` and `@springleaf`. Global config and
the existing credential file are still in `~/.config/springleaf/`; the renamed
default-root invocation therefore selects the code checkout, not `~/l`.
No store or source conversion was assumed or executed. Explicit-root offline
and online status each accounted for **27 unsupported, 0 failed**, remote
freshness respectively **offline** and **not-requested**. The format gate
prevented native fetching; **fresh remote editor status was not live-verified**.
All 27 file-byte SHA-256 values and nanosecond mtimes remained identical.
No live receipt was settled, no remote write occurred, and no private body,
filename, capability locator, credential or raw response was committed/printed.

Old-namespace files are diagnostic detections, not a compatibility read/write
path. Import refuses duplication alongside them. The parent should obtain the
explicit namespace/store-conversion choice separately; conversion is not
permission to rebaseline editor receipts. Current-namespace configured-default
routing and actual online classifications are covered synthetically.

## Parent join and road ahead

Review the receipt boundary and the live rename gap first. The commissioned
sequence remains **`leafspring-inspect-compare-design`**, a fresh Astra medium
design grounded in the actual receiving code, then **another fresh Astra
medium** for **`leafspring-inspect-operation-design`** using that design as
input. Join their outcomes with the user before selecting further architecture.
Neither is delegated or designed here; `leafspring-inspect-diff` remains
deferred. Current typed content, native association rows, raw witnesses and
`Agreement` booleans are evidence available to those designers—not a mandate
for structural diff or a new parallel planning representation.
