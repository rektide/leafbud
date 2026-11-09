---
type: ImplementationReport
title: "Verified first-binding defaults and explicit locator recovery"
description: "Version-0.15 source-preserving default/publication preparation, caller-selected lost-locator recovery, synthetic complete lifecycle verification, and create-free P1 deployment evidence with an honest description-clear partial."
resource: /design/native-default-adoption0.gpt61s.md
status: implemented-checkpoint
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3dcb1f8effebiT0KtCRZB8dWf"
    title: "Implemented guarded default preparation and explicit selected-locator recovery, tested synthetic lifecycles, and exercised human-selected P1 without creation; retained terminal description partial"
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Parent commission and operator decisions: adopt omitted native defaults explicitly; preserve explicit choices; human-selected P1 is authorized as the existing recovery target"
  - resource: /design/native-effects-review0.gpt6a.md
    title: "Parent review, selected default policy and source/publication/recovery correction boundaries"
  - resource: /design/native-effects0.gpt61s.md
    title: "Preserved predecessor checkpoint, original 249-test evidence, two consumed creation slots and compiler receiving seam"
  - resource: /design/keyed-authoring0.gpt6a.md
    title: "Controlling keyed foundation and native metadata/contributor preservation"
  - resource: /design/keyed-edit-flows0.gpt6a.md
    title: "Complete materialization, planned associations, read-back and interruption obligations"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Batch C scope and full compiler completion dependencies"
  - resource: /design/connected-watch0.gpt61s.md
    title: "Shared locks, byte guards, full-selection revalidation and pending-preserving inbound adoption"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Narrow defaults preparation distinct from unchanged tracking/adoption guards"
  - resource: /crates/workflow/src/native.rs
    title: "Atomic binding/default/hash checkpoint, explicit locator selection and prospective duplicate checks"
  - resource: /crates/workflow/src/native/write.rs
    title: "Located-mint preparation, contributor/retarget diagnostics and strict desired verification"
  - resource: /crates/workflow/src/import.rs
    title: "Actual empty contributor-list preservation and current native description mapping"
  - resource: /crates/workflow/src/native/defaults_tests.rs
    title: "Synthetic defaults, nulls, races, selected-locator and full materialization/settlement/status/pull/no-op fixtures"
  - resource: /crates/workflow/src/native/probe.rs
    title: "Explicitly gated create-free P1 harness; no remint or restart push re-encoding"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cvr.ts
    title: "Read actual extras construction: draft contributors default to an empty array"
---

# Verified defaults and selected-locator recovery

## Outcome and standing

**The selected first-binding source preparation is implemented.** Omitted
theme/bylines are copied from the verified native draft, and confirmed publication
becomes explicit in source before push intent. Strict verification is retained;
there is no verifier-only publication normalization or default wildcard.

Coordinated workspace version/internal pins are **0.15.0**, rather than retaining
0.14: this slice adds the cross-crate MDX preparation API and an explicit workflow
locator-recovery API. There is no new CLI command, public partial `push`, complete
keyed compiler, new persistent base/body/update/request journal or cleanup policy.
Both `leafbud-editor-bind` and `leafbud-editor-recovery` remain **in progress** with
their completion dependencies intact.

Code checkpoints:

- `f184128e`: explicit source defaults/publication preparation and synthetic
  lifecycle coverage; coordinated minor/pins.
- `aa631ef7`: caller-selected existing locator recovery and prospective duplicate
  protection, including selected-locator complete synthetic lifecycle coverage.
- `d6c21d38`: gated create-free P1 harness, retained description-clear partial and
  read-only standalone evidence, plus final retarget/description/import regressions.

The predecessor's report remains preserved; this report supersedes its open
default-policy question, not its historical deployment account.

## APIs and preparation transaction

| API/home | Contract |
| --- | --- |
| `mdx::prepare_binding_defaults(source, verified)` | Pure narrow patch for a located mint with no writer/receipt; returns proposed source, never seed prose or tracking adoption |
| `native::Source::prepare_binding` | Internal explicit no-content-write transition: verify previous input/desired hashes, prepare source, bind, compute new mint hashes, and atomically write the coherent result under lock/byte/full-selection guards |
| `RemoteReader::stage_binding` | Returned locator persists first; actual seed/root/rights/context and rechecked metadata then authorize the same preparation checkpoint |
| `RemoteReader::recover_native(..., settle)` | Located mint with `settle=true` can prepare binding/defaults; still reports pending materialization, not a settled save. `false` is source-read-only. Push settlement remains tracking-only with terminal complete correspondence |
| `RemoteReader::select_mint_locator(context, path, selection, attempt_id, token)` | Explicit existing recovery object for the expected dispatched mint; no candidate discovery, mint, replacement of a known locator or original-result inference |
| `execute_compiled` | Still crate-private; reverify mint seed, join preparation, reject explicit publication retarget/contributor changes before compiler/effects, then record actual normalized desired push hashes |
| `verify_desired` | Strict equality with actual source publication, metadata, content/order and intended native associations; hashes the source it actually verified |

Field ownership is deliberately bounded:

- Header **presence**, not `Option::None`, determines omission. Absent theme and
  contributors copy actual verified values. Explicit watermark true/false and
  supplied contributor lists, including `[]`, stay authored intent.
- `theme: null` and `contributors: null` have no established supported native
  meaning. They fail before authentication/mint, not silently become omission.
  Malformed contributor values fail there too.
- Contributors are observed author metadata, not a license to edit native
  permission/contributor records. A different explicit list diagnoses unsupported
  native save vocabulary before any push; it is not overwritten or ignored.
- Actually observed empty contributor arrays remain explicit through import and
  preparation. An absent native field does not invent a DID or an empty array.
  Older sources that omitted an actually present empty array can now show unequal
  endpoints; no migration or automatic receipt acceptance hides that correction.
- Publication omission resolves from the already pinned mint context, then is
  written as the confirmed URI or standalone null. Later ambient config cannot
  retarget it. An explicit conflicting publication is a diagnostic.
- Description/tags retain their clear semantics. Neither is copied as another
  default. Title, local-only metadata and desired body are never seed-adopted.

The MDX helper changes only selected top-level fields. Local UUID, main-page/key
identity, pages/prose, unknown/local-only metadata and bytes outside owned field
spans stay intact, including comments and explicit-field syntax. `patch_tracking`
still forbids authored-value changes; `adopt_remote` is not used to install a seed.

There is **one atomic source write** for defaults plus coherent binding/mint
hashes. The intermediate proposed string exists only in memory. Before that write,
a crash leaves the original located mint recoverable; after it, new fields and
hashes agree. No remint is possible from either state. Source edits or newly
ambiguous full selections across awaits preserve source and known locator.

An already dispatched **push** cannot enter this transition. Its hashes are not
rewritten to pretend newly adopted intent was originally dispatched.

## Explicit selected-locator recovery

The caller supplies both expected attempt ID and chosen token. Source hashes must
still match that attempt. Only dispatched mint or the same already located token
is eligible. Prepared mint, stale attempt, source edits, other origin, token
replacement and prospective workspace/external-operand duplicates stop before
native observation.

The selected locator is persisted **before** network verification, retaining old
attempt ID and hashes. Wrong rights/context/seed or a local save then leaves known,
unverified pending evidence rather than an unbound file or automatic next candidate.
Normal finite mint recovery verifies get/pull/get and performs the default checkpoint.
Reports explicitly distinguish a selected recovery target from proven attribution.

## Automated verification

Regular workspace gate: **262/262 tests passing**, with **three opt-in deployment
tests ignored**; formatting and strict all-target Clippy pass. Scoped native tests
also passed before the create-free effect. A red-before-fix regression failed on
the missing theme adoption before production changes were applied.

New coverage includes:

- Omitted theme/bylines copied once; explicit true/false, empty list and comments
  retained; actual empty bylines recorded; null/malformed fields fail before network.
- Pure patch preserves BOM/body/CRLF existing bytes, author syntax/custom metadata,
  local UUID/main page and tracking; description/tags are not adopted.
- Located-mint restart before/after the preparation checkpoint, unchanged mtime on
  repeat, source saves during observation and adjacent checkpoint, plus external
  full-selection ambiguity at verification.
- Original mint hashes remain until verified preparation; normalized hashes agree
  afterward. Dispatched old push/terminal partial cannot acquire the new policy.
- Both inherited-publication/configured and omitted-publication/no-config standalone
  lifecycles: bind → controlled complete fixture materialization → source comment
  save during read-back → fresh-reader terminal recovery/settlement → clean status,
  equal bound observation, unchanged pull and exact byte/mtime repeat no-op.
  Explicit theme false is actually written/read back, not rejected at seed creation.
- The same complete synthetic lifecycle through explicit selected existing locators
  in both contexts, without any creation call.
- Selected locator checkpoint before observation; changed/non-seed candidates and
  source races retain pending; stale/prepared/replacement/duplicate selection refuses.
- Explicit bound retarget/inherit stops before compiler/network; acknowledged
  native empty-string description cannot be silently adopted or masked as absence.
- Existing pending status/pull/watch/tidy, source/rights guards, protected native
  scope and request/acknowledgment regressions remain passing.

## Create-free deployed P1 proof — bounded partial

The operator explicitly selected **P1** through parent session
`ses_e4217c526ffeXeHByo5ZswnB3F`. It is the **human-selected existing verification
object**, not a proven reconstruction of the original lost mint response. No
additional approval ceremony or candidate-ranking mechanism was added.

This correction made **zero creation calls**. Both original budget markers remain
consumed. No release, email, unrelated-document mutation or cleanup occurred.
Private locators, original and selected reports/comparisons remain under ignored
`.test-agent/native-effects/`; none are published in this report.

Actual executed layers:

1. The explicit selection API moved the existing publication attempt from
   dispatched to located, retaining its original ID/hashes until verified source
   preparation. Current root, writable rights, publication association and empty
   seed were checked normally.
2. Verified watermark and contributor defaults were recorded in local source.
   The controlled internal compiler persisted title plus the single verification
   paragraph, with actual writer acknowledgment **2** and current read-back.
3. Strict whole-desired verification stopped **resolved-partial**, solely on
   **description**: authored absence is `null`, while native publication metadata
   returns an empty string. The theme/contributor/publication/content/association
   correction is not the remaining mismatch.
4. Create-free restart observations did **not** re-encode/resend the push, copy the
   native description into source, rewrite its old hashes or advance a receipt.
   Explicit known-text-fact read and protected-scope comparison are recorded
   privately. The ignored completion harness intentionally fails its complete-save
   gate on this partial; it is not reported as a passing deployed authoring proof.
5. The known standalone original was inspected with `settle=false`, keeping source
   bytes identical. It remains terminal partial at acknowledgment **2** under its
   original intent. No retroactive default adoption was performed there.

This proves selected-locator recovery and actual default adoption with controlled
publication draft writes; it does **not** prove original mint attribution, completed
deployed save/receipt/pull/no-op, native second-save editing or the full keyed compiler.
Those successful complete lifecycle properties are synthetic-only in this receipt.

## Remaining decisions and compiler receiving assumptions

**Native description clearing needs a projection decision, not default adoption.**
For P1, native empty string versus authored absence is now the only difference.
Choose/implement the native clear representation at the metadata projection/compiler
boundary, preserving explicit authored behavior and honest old intent. A bounded
candidate is to interpret the actual native empty publication description as the
defined clear meaning; verify that mapping rather than inserting a temporary
verifier exception or copying another default. The old source/desired hashes must
not be retroactively changed. Reobserve the existing terminal attempt after the
mapping is correct; never repeat the mint or initialize an independent CRDT.

**The old standalone needs deliberate terminal resolution**, not first-binding
preparation. The missing surface is a source-safe, acknowledged-terminal
accept-current/replan operation which explicitly accepts verified native
theme/bylines now, verifies actual planned associations/protected scope, retains
old attempt provenance and establishes newly accepted correspondence without
claiming the old omitted-field intent was originally saved. No such general
terminal accept/replan API is delivered here. Until selected/implemented, retain
its old pending attempt. No remote content effect is needed merely to make a
newly accepted local default explicit.

Next proposed implementation remains **`leafbud-editor-text` +
`leafbud-editor-structure`, g**, receiving normalized first-binding source and
actual raw native snapshots. The compiler must own all ordinary keyed
create/update/place/detach, full supported text, context-correct metadata and
complete source preflight. Contributor changes remain unsupported rather than
becoming permission edits. Recognize prior planned attachments when preparing
later moves, protect unowned native defaults, and do not substitute the controlled
single-seed test compiler for complete authoring.

Then finish **bind/recovery, Sol high**, with the description-clear and terminal
resolution joins plus full first/revised/repeat/pull/follower acceptance. Neither
ticket is closed by this correction. Maintained capability boundary:
[README](/README.md#native-effect-foundation-internal-checkpoint). Controlling
review: [native-effect review](native-effects-review0.gpt6a.md); preserved prior
evidence: [native effects](native-effects0.gpt61s.md); full delivery contract:
[keyed flows](keyed-edit-flows0.gpt6a.md).
