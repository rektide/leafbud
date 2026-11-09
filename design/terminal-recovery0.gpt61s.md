---
type: ImplementationReceipt
title: "Terminal save recovery without retroactive success"
description: "Version-0.17 explicit accept-current/replan API, bounded provenance and atomic pending transitions; retained native-default no-op correction and conflict/follower acceptance."
resource: /design/terminal-recovery0.gpt61s.md
status: implemented-awaiting-parent-review
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3d6d728effe6RFnyZcrUtifT9"
    title: "Commissioned recovery/conflicts and bind acceptance join; retained-default correction, explicit terminal API/state policy, 300-test gate and actual create-free standalone accept-current/status/pull/no-op proof"
  - resource: /AGENTS.md
    title: "Sol-high commission, keyed continuity, low-volume complexity and live-watch requirements"
  - resource: /design/keyed-save0.gpt61s.md
    title: "Received complete0.16 compiler/save, 282-test and independent Yjs evidence, actual P1 and remaining recovery boundary"
  - resource: /design/keyed-edit-flows0.gpt6a.md
    title: "Required complete workflows and confirmed-versus-pending associations"
  - resource: /design/keyed-authoring0.gpt6a.md
    title: "Supported meaning skips work; simple create/update/place/detach and no saved body"
  - resource: /design/native-default-adoption0.gpt61s.md
    title: "Original-success P1 versus old standalone theme-partial distinction and exact clear/default rules"
  - resource: /design/native-effects-review0.gpt6a.md
    title: "Parent prohibition on retroactive old-intent hash rewriting and extra minting"
  - resource: /design/keyed-save-review0.gpt6a.md
    title: "Parent accepted 0.16 shape and explicitly commissioned semantic no-op correction plus create-free terminal recovery/conflict/bind join"
  - resource: /design/connected-watch0.gpt61s.md
    title: "Stable-ID locks, byte guards, full-selection duplicate checks and pending follower contract"
  - resource: /crates/workflow/src/native/resolve.rs
    title: "Explicit terminal API, observation-bound new decision and observed association reuse"
  - resource: /crates/workflow/src/native/write.rs
    title: "Finality separate from original achievement, atomic new pending/provenance and bounded native runner"
  - resource: /crates/document/model/src/attempt.rs
    title: "Validated compact last-resolution provenance and retained writer reservations"
  - resource: /crates/workflow/src/native/recovery_tests.rs
    title: "Executed production HTTP recovery, conflict, source/identity guard and Phoenix follower tests"
  - resource: /crates/cli/tests/push.rs
    title: "Actual CLI/HTTP recovery, uncertainty and complete both-context save fixtures"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/push.ts
    title: "Re-read actual duplicate suppression and acceptance of monotonic IDs with reserved gaps"
---

# Terminal save recovery

## Outcome and API

Coordinated **0.17.0** adds `recover FILE [--take remote|local] [--yes] [--json]`
and `RemoteReader::recover_editor(..., RecoverOptions) -> ResolveReport`.
No direction is inferred from execution approval. Normal push cannot bypass an
outstanding effect with `--take local` or `--recreate`. `recover_native` remains the
finite original-intent observer/settler; the explicit API composes it, not another
effect engine. Source/transport/model ownership remains unchanged.

The [README](/README.md#explicit-save-recovery) owns command usage and reports.
This is not general repair, creation discovery, automatic cleanup or native CAS.

Implementation checkpoint: **`514bdae0`**, following no-op correction **`d4fda273`**.
Final provenance/future-state tests and deployment/acceptance receipt are a separate
verification checkpoint. No production text writer or deployed P1 was changed.

| Owner | Receiving surface |
| --- | --- |
| `workflow::native` | `recover_editor`, `RecoverOptions`, `ResolveReport`; preview/read-only by default, explicit new direction separate from execution |
| `native::resolve` | Fresh terminal witness, actual confirmed/pending correspondence and compact `Replan` input to the existing runner |
| `native::write` | `recover_native`/`recover_locked` original settlement, `execute_resolving_locked` atomic successor checkpoint; no separate effect engine |
| `document::State` | Optional validated `last_resolution: Resolution`, serialized as `lastResolution`; bounded provenance only |
| `editor` | `observe_detached_block` and `write::compile_reusing` for complete fresh explicit facts of authorized old allocations; ordinary `compile` supplies none |
| CLI | `recover` adapter and distinct `push` `settled-prior` result; `--take local` on ordinary push still cannot bypass pending |

## Chosen bounded state policy

- **Finality is separate from achievement.** Push observations now inspect the
  actual writer group even if current authored/desired hashes differ. No/lower ack
  remains uncertain regardless of direction. A terminal changed-source attempt
  reports original achievement unverified; old fingerprints are never rewritten.
- Exact original intent plus terminal acknowledgment and protected-scope equality
  still settles automatically without a new native request. Public push reports
  `settled-prior`, expressly not a newly requested recreation/new save.
- **Remote** accepts one freshly observed complete supported native document,
  reconciled against confirmed and pending associations. Existing source UUID,
  whole binding, writer/high-water mark and local-only metadata survive. Actual
  theme/bylines are adopted now, not claimed to have fulfilled old omission intent.
- **Local** compiles CURRENT source against CURRENT complete native state. Reused
  association rows come from actual observation, not assumed old success. Known
  detached allocated blocks require a complete supported explicit read; the same
  entity/text/placement IDs are reused. Missing, ambiguous, unsupported or changed
  allocations block rather than silently allocate again. Detached named-page/card
  graph repair is not implemented. Intentional genuinely new source keys still
  create new objects.
- One `lastResolution` records old ID/input+desired hashes, writer/range/through/
  acknowledgment, selected direction, conservative active+explicit observation
  hash, `originalNotVerified` disposition and optional successor attempt ID. It is
  bounded provenance, not a saved old body or executable mutation history.
- **No forgotten-old-attempt window:** remote acceptance writes accepted source,
  new receipts, cleared pending and old provenance atomically. Local replan keeps
  the old pending attempt while validating/compiling/rechecking; only the ordinary
  prepared-new-attempt checkpoint replaces it, atomically with provenance and the
  new ID link. A verified empty replan records correspondence/provenance without
  reserving an attempt. Later crashes retain prepared/dispatched new evidence plus
  the prior summary; failures never fabricate a receipt.
- Writer identity never resets. Unsent reserved suffixes stay consumed. The pinned
  server skips IDs at/below its acknowledgment and accepts larger IDs without a
  consecutive-ID requirement; new work starts at the persisted exclusive high-water
  mark. Old delivered duplicates cannot be reinterpreted as new work.

This avoids a resolution journal or transitional state machine: one old attempt
until an ordinary atomic successor checkpoint, plus one bounded last decision.
Fresh source/native witnesses are rechecked across awaits, including explicit
detached facts. All local writes use the existing UUID lock, canonical operands,
late full-selection duplicate checks and adjacent byte guard. Cancellation drops
the lock; it does not clear evidence. Unknown future owned state is rejected and
preserved by existing parse/refresh boundaries.

## Retained-default no-op correction

Commit **`d4fda273`** reproduced and fixed the parent review edge independently of
the new public feature/version bump. An unchanged imported body with native blank
text absence, bullet-style absence, explicit `is-list: false`, absent empty code
fields and omitted image optionals proposed **seven fact writes and one clear**.

Retained supported block equality now skips field construction, preserving native
representation and inactive data. Page links compare their target entity, not
temporary observed labels. New objects and intentional changes still construct or
clear owned fields. This is not blanket snapshot normalization or whole-document
display-equality skipping. Persisted title remains a separate compiler input:
display `Untitled` cannot mask absent title metadata. The production HTTP fixture
checks empty effects, stable fact/node/writer identities and exact bytes/mtime.

## Verification

- **60/60 native workflow tests** pass, including 15 new recovery groups and the
  retained-default regression. **4/4 regular CLI push/recovery groups** pass,
  including complete first/revised/pull/repeat saves in both contexts.
- Changed-source terminal inspection; explicit remote versus local; no/lower ack;
  original success versus new decision; current-intent no-effect replan; actual
  detached allocation reuse without entity/text reinitialization; reserved unsent
  suffix and old duplicate delivery; unsupported/unknown allocation refusal.
- Source edits, external selected-operand duplicates, rights changes, native drift
  across resolution and final pre-effect awaits preserve old pending. Failed new
  effects retain prior summary plus new pending and old receipts. Repeated
  resolution/no-op and separately requested recreation have truthful outcomes.
- Actual HTTP/Phoenix follower stays pending with unchanged receipts, then resumes
  adoption after explicit accepted resolution. Cancellation during HTTP recovery
  releases the UUID lock without source mutation; a concurrent cooperating writer
  cannot enter first.
- Known resolution provenance survives tidy/adoption; unknown future resolution
  state is preserved loudly by recovery, refresh, status and tidy. Definitely unsent
  prepared work can be abandoned even after source intent changes, without a
  native read or ID reuse; it is never reported as achieved content.

Final regular gate: **300/300 workspace nextest tests across 15 binaries**, with
**six deployment-only tests skipped**. Strict workspace/all-target Clippy passes;
formatting and README/receipt Markdown/OKF checks pass. The Yrs editing implementation is unchanged;
the received **7/7 independent actual Yjs groups** remain the relevant writer
evidence, rather than claiming these native-store tests are independent Yjs.

## Actual existing-standalone proof

The ignored actual-CLI test
`authorized_existing_standalone_terminal_accept_current_cli` **passed** on the
already-created operator-authorized standalone draft. No candidate discovery or
new creation was needed. Executed layers:

1. Checked the existing bound token against its private original locator and
   required the original terminal-partial phase/sent-through 2, no receipt and no
   prior resolution. Preserved previous source/binding/attempt privately before any
   acceptance.
2. Read-only public `recover` reported **resolved-partial**, **acknowledgment 2**,
   **terminal true**, **source matches original true**. The old theme-omission
   intent was still NOT achieved; this was not merely a changed-source mismatch.
3. Public `recover --take remote` preview was source-byte-exact. Public
   `recover --take remote --yes` then reported **accepted-remote**, empty native
   effects, actual watermark/default adoption and newly accepted correspondence.
   Old attempt ID/input/desired hashes and writer reservation were preserved in
   `lastResolution`, with **originalNotVerified**, remote direction, acknowledgment
   2 and no successor attempt. No old fingerprint was rewritten as success.
4. Actual CLI status **clean**, pull **unchanged**, repeat push **unchanged** with
   empty effects; repeat recovery **no-attempt**. Source bytes/mtime remained exact
   through these repeats. UUID, whole binding and writer were retained.
5. **P1 source bytes/mtime and both consumed creation-budget files were identical.**
   Zero draft-creation calls, native content writes, replay/re-encoding, release,
   email, unrelated-document mutation or cleanup. The standalone remained a verified
   draft; P1 was left untouched, not re-probed or revised.

Private before/command/proof outputs stay ignored under
`.test-agent/native-effects/deployed/standalone.terminal-*`. The opt-in proof refuses
blind reuse of its started marker; inspect current accepted state instead of
reconstructing old intent. No capability URLs, roots, credentials or bodies are
published in this receipt.

## Acceptance join and next receiving batch

The [complete keyed save receipt](/design/keyed-save0.gpt61s.md) supplies actual
existing-P1 revision/pull/repeat proof and both-context first-binding fixtures;
[native default adoption](/design/native-default-adoption0.gpt61s.md) supplies
original-intent P1 settlement. This receipt adds the deliberate partial boundary,
not original lost-mint attribution or a deployed first-unbound composer claim.
The bind acceptance join must retain those distinctions.

`leafbud-editor-recovery`, `leafbud-editor-conflicts` and `leafbud-editor-bind` now
have the complete implemented/verified commissioned slice. Ticket closure is the
implementation/acceptance-evidence account, not a substitute for parent review.
The editor epic remains for the parent to review; public release and unsupported
content are not inferred complete.

Recommended next **fresh Sol high** batch: `leafbud-author-metadata` plus
`leafbud-author-rekey`, optionally `leafbud-inspect-list`. These finish useful
local explicit authoring and discovery around the now-complete save loop, sharing
source-preserving field/key patches, current identity checks and existing locks.
Keep receipt/pending ownership honest without a migration framework. This is a
preview, not another implementation commission. Native release remains a later
separate batch with its own release-state and explicit live-effect authorization.
