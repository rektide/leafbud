---
type: ImplementationReceipt
title: "Independent local naming reconciliation"
description: "Implemented 0.20 independent naming witnesses, offline reconciliation/read-only status, literal workspace mapping and recoverable no-clobber file moves."
resource: /design/naming0.gpt61s.md
status: implemented-awaiting-parent-review
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3c29df03ffeKTX7z5IHA3nup5"
    title: "Implemented independent naming, source-safe patches, guarded no-clobber moves and shared status/list evidence; CLI/HTTP/Phoenix and interruption/race verification"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted all-applicable tidy, explicit directions and independent name acceptance"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Optional workspace-relative name witnesses and author path intent"
  - resource: /design/document-sync1.gpt61s.md
    title: "Independent current-endpoint naming evidence and native path limits"
  - resource: /doc/mdx/README.md
    title: "Received source patch, comments and pending-state boundaries"
  - resource: /crates/workflow/src/tidy.rs
    title: "All-selector composition, full-selection late guards and per-item recoverable rename/source ordering"
  - resource: /crates/document/model/src/lib.rs
    title: "Typed independent pair, presence and validated reversible coordinate mapping"
  - resource: /crates/document/mdx/src/names.rs
    title: "Separate author path/naming patch authority and exact full-output/body proof"
  - resource: /crates/workflow/src/names.rs
    title: "Shared current naming classification and reconciliation plan"
  - resource: /crates/workflow/src/files.rs
    title: "Linux NOREPLACE primitive and file identity guards"
  - resource: /crates/workflow/src/tidy/tests.rs
    title: "Naming matrix, source/path/duplicate races, no-op, interruption and external save proofs"
  - resource: /crates/cli/tests/author.rs
    title: "Executed offline naming with deliberately bad credentials/origin"
  - resource: /crates/cli/tests/push.rs
    title: "Confirmed native naming-only no-effect push/pull and independent dirty editor proof"
  - resource: /crates/workflow/src/watch/tests.rs
    title: "Actual Phoenix watcher rescan after naming tidy and retained binding/names/comment adoption"
---

# Independent local naming reconciliation

## Outcome and checkpoints

Coordinated **0.20.0** implements `leafbud-author-names` for parent integration
review. It adds offline `tidy --from filename|path` and `--accept`, independent
read-only naming status/list, and all-applicable default reconciliation. It does
not implement public route mutation, release, names diff or automatic renaming by
push/pull/watch. The ticket remains open for parent acceptance/closure.

The initial architecture checkpoint is `6dde096c`; core implementation and initial
340-test gate are `7f3561cc`. Guard/failure corrections are `a0cb0383`; current-path,
preview destination-conflict and inventory corrections are `7799358b`. The final
owned documentation checkpoint retains the complete maintained usage/coverage
account. These checkpoints validate stored witness
coordinates, clarify post-write sync uncertainty and add direct no-replace and
post-rename concurrent-save fixtures.
No `~/l`, credentials, P1, standalone object or deployed remote was touched.
Both original mint slots remain consumed. Fixtures used only temporary workspaces
and loopback HTTP/Phoenix; the native/Yrs writer is unchanged.

## Mapping and evidence

Logical `/notes/Essay` maps exactly to workspace-relative `notes/Essay.mdx`.
The slash is a logical root, never an operating-system absolute destination.
Nested directories participate (they are not flattened or silently preserved).
Case and literal Unicode are retained without normalization. Exactly one final
lowercase `.mdx` extension is removed/added; `x.mdx.mdx` maps to `/x.mdx`.
Empty segments, `.`/`..`, backslashes, percent encodings, control characters and
URL/drive punctuation `:?#` are unsupported. No decoding, slugification or route
allocation occurs. Destination directories must already exist, be real directories
under the canonical workspace, and contain no symlink components. No directories
are created by preview or naming moves.

The compact optional `names` pair records the workspace-relative file including
extension and the actual author `path` presence (missing/null/string). No time is
needed: a changed pair is the settlement. There is no second body or name history.
External operands remain supported by key/Markdown passes; naming requires moving
the operand into the workspace explicitly first. Unsupported logical intent can
still be explicitly accepted as a mismatch: acceptance does not enact that intent.

Read-only classification and default tidy share one policy. Missing/null current
path is unassigned. Equal endpoints without a witness are equal; differing ones
are untracked and need choice. Against a witness, changed file only selects filename,
changed path only selects path; both changed and unequal needs choice. Equal changed
endpoints are converged. An unchanged mismatched pair is accepted-mismatch. Default
tidy settles equal/converged and reconciles only unambiguous one-sided changes.
Explicit direction selects the current endpoint, not a historical value. An
unassigned draft stays unassigned unless filename direction is explicitly chosen.

## Source and filesystem boundaries

MDX gets a separate naming patch authority for author `path` and `state.names`.
Tracking settlement does not gain arbitrary author-field authority. Complete
reparse/header/model/body checks prove the selected change; pending and future
owned state refuse. All editor receipts, native associations and bounded resolution
provenance remain unchanged. Naming is offline and does not implement native path
mutation, public identity selection, publishing or release equality.

Workflow retains the stable-UUID OS lock. All explicit operands participate in
initial and immediately-pre-effect duplicate checks, even outside the workspace.
Source bytes and file identity, canonical operand route, destination parent and
destination absence are checked again before effects. Each batch item is independent;
two items targeting the same file cannot overwrite it (the later item refuses).
Copies sharing identity are ambiguous even if the destination copy has identical
bytes. Symlink operands resolve to their real file; naming follows that canonical
coordinate, while route changes during planning refuse.

For a move: validate/compose everything first; use an actual no-replace rename;
then write the prepared source/witness under the original-byte guard at the new
path. Interruption before this write leaves the good original at the destination
with old naming evidence. An error after replacement but before directory sync
may instead leave the complete patched source; an unrelated save can leave its
own bytes. Reports therefore say settlement **not confirmed**, name the destination
and request inspection/retry rather than deleting it or rolling over a concurrent
editor. Retry tidy there: a stale
pair truthfully reports the move; equal endpoints can settle. Native identity and
pending evidence are never forgotten. No journal or generic transaction manager.

This is not an atomic multi-file CAS. Cooperative operations serialize by UUID;
noncooperating filesystem writers can still race the final check/syscall window.
Cross-device moves and unsupported no-replace platforms refuse. No fallback copy,
overwrite rename, auto cleanup or persistent staged duplicate is introduced.

## Receiving APIs and reports

- Model: optional `State.names: Option<Names>`; `Names { file, path: NamePath }`.
  `NamePath` preserves missing/null/value, and the same pure `Names::file_for_path`
  / `path_for_file` policy validates historical file coordinates. No migration:
  old current-format files simply omit this optional pair.
- MDX: `patch_names(input, optional_path_edit, pair)` patches selected author path
  and tool envelope only, with exact full-header/document/raw-body proof. Existing
  selected-field source adapter is reused; tracking/adoption cannot change names.
- Workflow: `names::inspect` supplies one classification to tidy/status/list;
  evidence includes current coordinate/presence, accepted pair, grounded independent
  change flags, mapped filename, unsupported mapping and native-path limitation.
  `tidy::Request` adds `from`/`accept`; selectors union, while `--all` selects files.
- Reports: tidy/status/list JSON are **version 2**. Tidy adds destination,
  pre-operation naming evidence, selected `proposed_names`, action, needs-choice
  count and partial renamed-settlement-unconfirmed outcome. A naming decision
  blocks that item's whole composed rewrite; independent neighbors still run.
  Dry run composes/reparses the real source and validates destination without any
  source/rename/witness write. It can acquire the existing cooperative UUID lock.

## Verification and review judgments

The maintained [coverage matrix](/doc/mdx/coverage.md) N1–N3/L2 names every new
fixture and its level. Tests exercise pair classification, missing/null, accepted
mismatch exact repeat bytes/mtime, explicit directions, nested literal Unicode and
extension mapping, invalid paths, external policy, selector union, preview and
workspace selection. Pure block/flow/BOM/CRLF patches keep unknown author bytes,
selected path comments and exact body/comment anchors.

Filesystem tests cover identical/dangling destinations, symlink aliases/parents,
common batch destinations, all external copied operands, late duplicates, replaced
inodes, path retarget, source save and target creation. A direct syscall fixture
proves NOREPLACE independently of preflight. Injected pre-witness interruption
retains the original at its reported new path and retry settles truthful convergence;
a new external save after rename wins over the witness patch. No copy/rollback or
cleanup manager was introduced.

Actual CLI/HTTP verifies name-only changes retain UUID/native nodes/binding/writer/
receipts, status does not accept name drift, and subsequent push/pull is empty-effect
with exact bytes/mtime. Actual Phoenix rescan adopts later remote content at the
new filename with names/binding/comment preserved. Existing terminal-recovery and
typed-pending fixtures now exercise naming explicitly: pending refuses, and historical
resolution provenance remains exact. Future unknown owned state/coordinates refuse.
Offline CLI uses a deliberately invalid credential/origin environment.

The post-core review tightened these boundaries: malformed stored file coordinates
must not supply direction evidence; a post-replacement directory-sync error cannot
truthfully claim the original source is necessarily still present. These are bounded
corrections, not a filesystem CAS claim or additional native semantics.
An unsupported current author-path type also needs a choice/refusal; it is never
reported as a missing/unassigned author path or silently skipped by default tidy.
Dry-run batches reserve earlier planned destinations in memory so two absent-target
plans cannot both advertise success; execution still relies on no-clobber effects.
Workspace file selection now reuses the shared inventory, keeping invalid UTF-8
candidates as per-item failures rather than aborting the whole tidy batch.

Final regular gate: **342/342 workspace nextest tests**, six opt-in deployment
tests skipped. `cargo fmt --all -- --check`, strict workspace/all-target Clippy
and Markdown/OKF checks of this receipt, README and both maintained MDX documents
passed. No deployed or independent-Yjs rerun is claimed; the native writer was
not changed. Parent acceptance is separate from these executable checks.

Parent owns integration acceptance and the next release-front assessment. The
recommended next design pair is `leafbud-release-adopt` then
`leafbud-release-state` (the latter currently depends on the former): a fresh Sol
high can establish actual public/native identity and independent release evidence
before `leafbud-release-native` effects. That is a recommendation, not authorization
to publish/email/share or a continuation of this implementation session.
