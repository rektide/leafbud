---
type: ImplementationReceipt
title: "Save-first native publication and observation-only recovery"
description: "0.25 quiet publication, durable editor/dispatch checkpoints, verified release settlement, fixture gates and the blocked one-shot live smoke."
resource: /design/native-publish0.gpt61s.md
status: implemented-awaiting-parent-review-and-live-proof
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3983e092ffeuzOsaIZioZQNKF"
    title: "Read-only deployed witness investigation corrects missing-field diagnosis: explicit null one-to-one joins match the pinned schema; guarded repair and separately authorized final proof follow"
  - resource: "opencode://ses_e39a14490ffevJd86pqM2pp36t"
    title: "Implemented and fixture-verified 0.25 save-first publish/recovery and separate settlement; one fresh live standalone draft saved, publication refused missing effect evidence before dispatch, draft deleted and token observed gone"
  - resource: /design/native-publication0.gpt61s.md
    title: "Complete action/effect/preflight and execution receiving brief"
  - resource: /design/publication-verification0.gpt61s.md
    title: "Content-CID and cumulative-native inline-size prerequisites"
  - resource: /design/native-delete0.glm53m.md
    title: "Observation-based native deletion and published record→editor cleanup path"
  - resource: /doc/release/README.md
    title: "Maintained release/editor identity and independent evidence contract"
  - resource: /crates/workflow/src/native/publication.rs
    title: "Locked save/dispatch, declared normalization, exact association/content verification and finite recovery"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Distinct loss-checked settle_release authority"
  - resource: /crates/workflow/src/native/publication_tests.rs
    title: "Actual HTTP/native-mutation-store execution, uncertainty, crash, byline and saved-fragmentation fixtures"
  - resource: /crates/cli/tests/publish.rs
    title: "Real binary preview/first/update/no-op/uncertain-recovery fixtures in both contexts"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "Re-read native contributor upsert: applies to every publication-container publish, not only first release"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_leaflet_data.ts
    title: "Pinned RPC explicitly selects reply_drafts and publication_scheduled_posts; deployed omission is not empty evidence"
---

# Native publish — 0.25

## Standing

Commit `1fed32fc` implements execution/recovery and coordinated **0.25.0**;
this is not ticket acceptance or live first/update proof. `leafbud-release-native`
remains in progress. No delivery, newsletter/share controls, resource upload,
custom rkey/path mutation, cleanup framework or migration was added.

`leafbud publish FILE [--yes] [--json] [--take local]` previews by default.
Exit 0 means successful preview/no-op/settlement; 4 means blocked/pending/
uncertain/partial-unsupported/source-changed; 1 is operational failure; 2 usage.
The CLI is single-file deliberately; bulk publish was scaffold-only, not an
implemented compatibility obligation. `push` remains editor-only.

## Save and dispatch boundaries

Full source/release-body/path/date preflight precedes remote reads/effects.
The native save body is factored into `save_locked`; publication composes it
without recursively acquiring the stable-ID source lock. Mint/default adoption,
existing drift decision rules and keyed CRDT/fact identity remain unchanged.
Post-seed/pre-save checks verify authority, effect fields and the actual byline;
compiler sizing proves no offload before save. A second size witness uses freshly
observed **actually saved** cumulative native facts before dispatch. Over-bound
state refuses rather than discovering an upload by sending publication.

The save receipt checkpoint atomically adds a separate compact
`@leafbud.publicationAttempt` in `editorAchieved` phase. There is no crash gap
between editor success and that continuation. A crash earlier remains ordinary
editor mint/push recovery: no release was sent. Editor attempts and publication
attempts cannot coexist in persisted state. The separate slot avoids pretending
publication uses Replicache writer acknowledgments or widening editor effects.

Immediately before invoking `invoke_publication`, persist `dispatched`: attempt
UUID, unchanged authored/editor hashes, owner, argument fingerprint, complete
supported public-core hash, exact old URI/CID for updates, date policy and the
one permitted native actor-byline addition. No body or executable journal.
The marker means **may have been sent**, including the pre-send crash window.
No automatic retry, even on typed `publish_failed`, busy, timeout or lost response.
Native publish locking is neither idempotency nor input CAS.

The operator accepted native publication-container image housekeeping. Preview
now says **old image copies may be deleted; count unknown**. Image-free source
is not an empty-history witness. Leafbud adds no storage access/deletion/retry.
The low-level argument type still forces email false and exposes no actor,
rkey/path, entitiesToDelete or sharing escape hatch.

## Normalization and settlement

Only declared fields substitute: first URI/rkey is learned from authoritative
native association (never title/recency search); updates require the old exact
URI and `/rkey`. Explicit nonfuture dates remain the same instant and preserve
their authored spelling/trivia. An omitted update date uses the observed native
date, even if postdated. First missing/null date and explicit future-clamp use
native now, bounded below by dispatch and above by observation; this verifies
current correspondence, not authenticity of a server clock or invocation time.
The complete remaining public meaning—including site/owner, native page IDs,
all supported metadata and body—is hashed/compared without wildcards.

Pinned source upserts the actor on **every** publication-container publish.
The actual post-save contributors plus actor must fit the existing one-byline
projection; a second contributor refuses before publication rather than sorting
unordered rows. Verified native addition is adopted explicitly at settlement.

Success requires the bounded action result (when available), authoritative native
association and exact-URI `fetch_content_verified` PDS content to agree through
the complete supported projection. Wrong CID, path/date, URI/owner, body/metadata,
unknown output or changed native editor cannot advance `released`. Content
rehashing reuses 0.23; no new CBOR/CID implementation or signature claim.

New `leafbud_mdx::settle_release` is distinct from `patch_tracking`,
`adopt_remote` and `adopt_published`; those keep their old release restrictions.
It loss-checks only lifecycle/URI, permitted path/date/contributor values,
verified editor receipt, release receipt and removal of the settled continuation.
UUID, keys/native associations, names witness, writer/resolution provenance,
unrelated author fields/comments and exact body bytes survive. A public-URI lock
and full-selection prospective URI duplicate check guard first settlement.
Clean repeats rehash/observe and return empty effects with stable bytes/mtime.

## Finite recovery

`recover` routes publication continuations to a sibling observation-only path.
An editor-achieved/undispatched continuation can be cleared deliberately; that
call never publishes. Repeat `publish` is a separate explicit action. A dispatched
attempt discovers first identity only by association or reads the exact old URI;
absent association is not negative finality. Supported correspondence can settle;
wrong/unsupported partial output stays explicit. Source changes retain the old
fingerprints/uncertainty and return the actually observed locator without rewriting
new intent as the original success. `--take local` never overrides publication
uncertainty. Push, delete, managed metadata/rekey/naming and inbound refresh preserve
or block publication continuations. Status names undispatched versus uncertain.

## Evidence and live result

Initial full fixture gate: **405/405 regular tests**, nine opt-in skips, nextest
`26c68322-5f79-4ba9-8f41-351d100fa8c5`; strict all-target Clippy passed. Final
metadata-before-effect regression and final formatting/document gates follow.
Tests include both first-binding contexts, heading/paragraph additions, same-URI
updates, no-op bytes/mtime, lost responses both ways, pre-send/after-editor/
after-dispatch windows, drift decisions, source changes, wrong meaning/CID/date/
path, byline refusal, separate patch authority, and actual post-save fragmentation
above 100 KiB despite a smaller pre-save compiler estimate. Earlier independent
native serializer/CID vector proof was trusted, not gratuitously rerun.

The authorized one-shot smoke consumed `.test-agent/native-publish/smoke.started`
**before mint**. Normal push created and saved exactly one fresh standalone draft.
`publish --yes` then **refused before dispatch** because deployed `get_leaflet_data`
omitted `reply_drafts`. The pinned RPC selects it; missing evidence cannot be
treated as an empty array. No publication attempt was dispatched, public record,
email or share created. The fresh draft was deleted through the native capability,
its token observed gone and local file unbound. Private push/publish/delete
receipts are under that marker directory. Budget remains consumed: no rerun.
This is an honest blocked publication smoke with verified draft cleanup,
**not** live public-record or published-deletion proof.

## Parent steering: undecoded mint evidence

The concurrent publication-draft delete smoke reported `invalid/duplicate action
row ID`, but its own private README confirms the body was lost. No retained
payload exists from which to derive a new Flight grammar. This session therefore
does **not** guess a decoder relaxation or claim the deployed mint gap fixed.

It fixes the evidence loss: an already-bounded undecoded mint body is written
before returning uncertainty to a create-new private receipt beside global config:
`action-responses/<attempt-id>.flight` (directory 0700; file 0600; at most 1 MiB;
non-symlink private directory required; fsynced; never overwritten/pruned).
The save/push report carries only its path, never body/token-bearing server prose;
no opaque body enters source state. The existing in-memory `BindingReport` body
remains available. Retention failure is explicit and cannot trigger another mint.
A literal unknown-row fixture proves preservation through the save facade.
The next actually observed payload can support a bounded decoder regression;
there is no authorization to remint merely to obtain one.

## Review and road ahead

Parent review owns acceptance. Next fresh Sol high continues **leafbud-release-native**:
verify a deployed authoritative effect-field witness/capability without weakening
absence guards; inspect a genuinely retained mint response before changing its
decoder; obtain explicit authorization for live first/update acceptance separately
from this consumed smoke. Then consider **leafbud-release-delivery** only after
its prerequisite and operator commission: explicit observable newsletter/share
behavior is distinct from content correspondence. No ticket was closed.

Reported context at the code/gate checkpoint was **239401**; scope closed near
221k. Final documentation/gates/handoff finish this session; no adjacent feature
or further live publication attempt is commissioned here.

### Final gate

**406/406 regular tests**, nine opt-in skips: nextest run
`3d32df4e-653f-453f-a4c0-c4fee41d4871`. Strict workspace/all-target Clippy and
`cargo fmt --all --check` pass. Four owned Markdown/OKF documents passed
structure, links/anchors and session provenance checks (67 links, 70 sources
before this final evidence paragraph). Last reported context: **256101**;
rotation/handoff follows this completed implementation-and-evidence boundary.

## Deployed witness correction — 2026-11-10

Follow-up session `ses_e3983e092ffeuzOsaIZioZQNKF` inspected only the already
authorized P1 and standalone verification drafts: one read-only
`get_leaflet_data` RPC and one read-only publish-page GET per target. No source,
binding or remote object was changed. Private bounded bodies and sanitized key/
shape summaries are retained in `.test-agent/native-publish-delta/` (0700/0600).
The archive is still clean at `600c7b5f1f7b31b66fb626c293cc104312a12caf`.

**The earlier assertion that deployed `reply_drafts` was omitted is not supported.**
The old error said “evidence missing” for *every non-array*, including null; the
prior smoke retained that diagnostic, not the raw metadata. Today's direct RPC
evidence contains both `reply_drafts: null` and `publication_scheduled_posts: null`
on both objects. The pinned `supabase/database.types.ts` marks both leaflet
foreign keys `isOneToOne: true` (reply lines 2293–2298; schedule 2109–2114).
PostgREST's selected empty one-to-one join is explicit null, not an empty array.
The pinned publish page also reads `data.reply_drafts?.subject` and supplies the
selected token; the deployed page token's reply witness is null, consistently.

The complete direct RPC key inventory on both targets is:
`blocked_by_admin`, `custom_domain_routes`, `description`, `id`,
`leaflets_in_publications`, `leaflets_to_documents`, `permission_token_rights`,
`publication_scheduled_posts`, `publications`, `reply_drafts`, `root_entity`,
`title`. P1 has one direct publication draft row; standalone has neither direct
linkage row. P1's row includes `preferences: null`, `email_only: false`, null
`doc`/`documents`, and exact-token/publication/owner joins. The publication join
includes `identity_did`, `uri`, `record`, `publication_contributors` (empty),
newsletter/membership settings and membership tiers, matching the pinned selection.
Rights contain token/entity-set/read/write plus the sibling entity-set join.
There is no observed missing or renamed field used by publication preflight.

This is compatibility evidence for the inspected contract, **not a deployed commit
identification**. It establishes neither an older nor newer deployment and does
not justify a re-pin. The authoritative reply/schedule witness already exists in
the same RPC; another RPC, page scraper or fresh-mint provenance escape hatch is
unnecessary. Repair will distinguish a present null from an absent key, refuse
non-null relations and malformed shapes, and continue requiring complete evidence
for both adopted/pre-existing and transaction-minted documents. The original
smoke's refusal and cleanup remain real; its inferred deployed-delta explanation
above is superseded by this correction.
