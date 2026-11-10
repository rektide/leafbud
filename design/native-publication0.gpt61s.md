---
type: DesignCheckpoint
title: "Native publication action contract and preflight foundation"
description: "Pinned action/wire/effect evidence, implemented read-only readiness and transport checkpoint, and a bounded handoff for fresh execution/recovery work."
resource: /design/native-publication0.gpt61s.md
status: implemented-foundation-awaiting-parent-review
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3983e092ffeuzOsaIZioZQNKF"
    title: "Added superseding deployed explicit-null witness diagnosis and final first/update/cleanup evidence pointer without rewriting the original foundation checkpoint"
  - resource: "opencode://ses_e39a14490ffevJd86pqM2pp36t"
    title: "Added current 0.25 execution/settlement and blocked-live-smoke follow-up pointer; original 0.22 evidence preserved"
  - resource: "opencode://ses_e3bb92bc4ffeN77oF8X2ayoH9a"
    title: "Added verification follow-up pointer and clarified publication-container-only gated cleanup scope; original 0.22 checkpoint preserved"
  - resource: "opencode://ses_e3bcb6540ffegZC39tN1E6wuTc"
    title: "Bounded release-native contract/preflight checkpoint: clean pinned source review, no-email action transport fixtures, read-only readiness and exact default-theme correction; no execution workflow or live effects"
  - resource: /doc/release/README.md
    title: "Maintained published/editor/public evidence boundaries"
  - resource: /design/published-lifecycle0.gpt61s.md
    title: "Receiving 0.21 native associations, independent release observation and honest CID limits"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted root publish saves then releases, independent names and normalized release settlement"
  - resource: /design/document-sync1.gpt61s.md
    title: "Actual ordinary action path/date constraints and uncertain publication recovery"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Author/tool ownership and missing-date preservation"
  - resource: /design/native-effects-review0.gpt6a.md
    title: "Durable dispatch-before-effect and explicit first-binding default adoption"
  - resource: /crates/leaflet/web/src/publication.rs
    title: "Implemented literal action contract, lookup, transport and bounded result decoder"
  - resource: /crates/workflow/src/publication.rs
    title: "Implemented no-write internal preview and conservative effect/identity readiness checks"
  - resource: /crates/leaflet/records/src/projection.rs
    title: "Shared full-body release preflight and exact native standalone default theme"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts
    title: "Authenticated action strips rkey, clamps explicit future date and adds analytics"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "Owner selection, public record construction, linking, notifications, newsletter and reply effects"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/factsToPagesRecord.ts
    title: "Page identity/traversal, list grouping, UTF-8 facets and richer resource branches"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts
    title: "Image/poll uploads and standalone theme including discriminator/default"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/gatedDocumentImages.ts
    title: "Unconditional publication stale-copy cleanup can delete historical gated assets"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/offloadPagesToBlob.ts
    title: "More than 100 KiB JSON pages triggers a public blob upload"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLock.ts
    title: "Busy refusal and five-minute process/Redis lock, not idempotency or input CAS"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/versioning/cutPublishVersion.ts
    title: "Conditional best-effort Pro publication snapshot"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_leaflet_data.ts
    title: "Available direct/sibling linkage, publication membership, schedule and reply evidence"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/%5Bleaflet_id%5D/publish/PublishPost.tsx
    title: "Native UI pushes first; sharing is a separate action; scheduling is not ordinary publication"
---

# Native publication contract and preflight checkpoint

## Standing and scope

**0.22 is an internal foundation, not published delivery.** Commit `87ec3c20`
adds transport and read-only preflight. Root `publish`, including `--yes`, still
refuses as unimplemented before effects. `leafbud-release-native` remains in
progress; delivery and explicitly authorized live first/update proof remain open.
No credentials, `~/l`, private test drafts, mint slots, live publication, uploads,
email, sharing or cleanup were used. Only loopback fixtures invoked the adapter.

The archive was clean at **600c7b5f1f7b31b66fb626c293cc104312a12caf** before this
review. All upstream behavior below refers to that revision, not a deployed probe.
The Next registration syntax reuses earlier observed mint discovery; the publish
export/response fixtures are synthetic, grounded in source—not evidence that a
current deployment has exposed or executed that action successfully.

## Actual action, authentication and wire

`publishToPublication(args)` is the ordinary public action for both contexts.
It obtains `getAuthIdentity().atp_did`; callers cannot choose the actor. A supplied
`rkey` is stripped, including runtime injection. An explicit parseable future
`publishedAt` is replaced with server now. Scheduling uses a different action.
The internal utility's trusted-import rkey argument is **not** this capability.

The implemented adapter reads authenticated `/home`, then `/<token>/publish`
and bounded same-origin `/_next/static/*.js` scripts. Exactly one literal
`createServerReference(...,"publishToPublication")` pairing must be found; zero,
ambiguous, changed or redirected discovery refuses. No remembered deployment hash,
title lookup, foreign asset, guessed standalone fallback or direct PDS write.
Conflicting authenticated/profile DIDs now refuse instead of choosing one.

The effect transport POSTs to that same native publish page with `Next-Action`,
`Next-Router-State-Tree: ["",{}]`, same-origin `Origin`, `Accept: text/x-component`,
and `Content-Type: text/plain;charset=UTF-8`. Its body is one literal argument:

```json
[{"root_entity":"<verified UUID>","leaflet_id":"<verified token UUID>","publication_uri":"<verified publication, omitted for standalone>","title":"Current title","description":"","tags":[],"publishedAt":"<explicit date, otherwise omitted>","sendEmail":false}]
```

`Arguments::wire` cannot supply `actorDid`, path, rkey, `entitiesToDelete`, email
true or a sharing action. Empty description/tags explicitly clear old values.
Omitted preferences/discovery **inherit native state**; omission does not mean
quiet publishing. Current preflight refuses nondefault inherited preferences and
unsupported current record fields rather than silently overwriting/ignoring them.

`Preparation` binds current origin, client session and token, but is not authority
to publish. `invoke_publication` is an explicit low-level effect API, unused by
CLI/workflows in this checkpoint. Future callers must checkpoint dispatch first.

Return `success:true` contains rkey, URI, inflated record, firstPublish, blocks and
optional reply. **No output CID is returned.** The actual PDS record can differ
after gating/offload. Bounded Flight parsing selects only the action return slot,
literal or one promise, not unrelated render URIs. Unknown/error/promise shapes
retain a bounded undecoded body in memory without diagnostic content leakage.

Typed failures are `oauth_session_expired` (with DID), `publish_in_progress`, and
`publish_failed`. OAuth refusal occurs before content effects in inspected native
branches; busy means another publication may already be running. `publish_failed`
does **not** establish no effects: a PDS write/link failure can leave partial
state. HTTP errors, redirect/not-found, truncated/lost response and server throws
are uncertain after dispatch. No automatic invocation retry is implemented.

## Owner, identity and readiness

Publication records live in the **joined publication's identity_did**, using its
restored OAuth session. Actor must be that owner or a confirmed contributor.
Membership does not imply owner OAuth health; editor token write rights do not
imply publication authority. Standalone first release uses authenticated actor;
standalone update additionally requires actor equal to the established URI owner.
No ambient handle/PDS/publication default retargets a binding.

The action queries the direct token's linkage row. Existing sibling-only native
adoption is insufficient proof that this lookup addresses the right draft.
Preflight requires coherent direct row, token, joined publication owner/membership,
complete unchanged native binding and actual read/write rights. Existing public
URI/collection/owner and native association must agree; legacy publication profile
refuses. Reply/schedule evidence must be present, not assumed empty from omission.
Nonempty replies/schedules block this initial profile.

`workflow::publication::preview` validates the **entire** desired source through
existing key preparation/editor preflight and shared records release-body encoding
before remote reads. New headings/paragraphs/pages are allowed; missing source keys
and native page identities are counts, not durable invented associations. Bound
previews reuse current editor observation/compiler to report save effects and drift
decisions. Unbound previews report bind/save/default/owner work without minting.
Unsupported images/marks/graphs or metadata fail before any future save effects.

No preview writes bytes, mtime, keys, defaults, attempts, names or receipts, opens
a stable-ID change lock, or calls save/mint/publish. Source bytes and full-selection
UUID/native/public duplicates are checked after awaits; raw native metadata is
rechecked after action discovery so schedules/replies/current output are not hidden
by the deliberately narrower editor hash. It is an internal readiness summary,
**never a ready-to-execute authorization**. Execution/size coverage remain blockers.

## Normalization and projection

- First ordinary release chooses a TID server-side; URI/rkey and `/rkey` path are
  unknown until returned and authoritatively associated. Custom author path is
  disclosed as normalized, never sent as a target selector.
- Updates keep the exact existing collection/rkey/URI and force path `/rkey`.
  Existing nonstandard path intent is a disclosed normalization, not a new post.
- Missing/null first-release date means server now, not an invented local time.
  Missing/null update date must be **omitted** so native preserves the actual
  existing publishedAt, including a postdated value. Explicit authored date is
  passed unchanged; future explicit date is disclosed as clamped, not scheduled.
  Execution must re-evaluate against current evidence; preview's `now` is not CAS.

Shared projection covers exact title/description, ordered tags, supported bylines,
native page IDs, reachable unique linear links, grouping of flat adjacent list
items, text/scalar block order and UTF-8 facets/marks. Canonical comparison ignores
facet segmentation/feature ordering and non-entry page definition order, not public
meaning. Publisher traversal can repeat subpages; repeated/cyclic/unreachable
graphs refuse. Multiple byline ordering is unavailable from unordered pull.

A concrete correction landed: `extractThemeFromFacts` includes `$type` as well as
showPageBackground; therefore its key-count test retains even the exact standalone
default theme. Projection now accepts only
`{"$type":"pub.leaflet.publication#theme","showPageBackground":true}` as the
same default meaning as absence. False, images, font/colors or any extra property
still refuse. This is not arbitrary theme suppression or editor-support reduction.

URL images, cover/background resources, polls, custom/native page themes, colored
highlights, offloaded/gated output and unknown record fields remain unsupported
release profiles. Existing native observer coverage rejects unrepresented facts;
existing output projection also catches inherited cover/share/preferences.
The native offload threshold is **100 KiB of JSON pages**, triggering upload. A
semantic re-encoding's size is not sufficient: retained Yrs fragmentation can
produce more facets than normalized runs. Before execution, conservatively bound
actual native text/fact expansion or obtain an exact no-effect native projection;
do not discover this upload only after saving or dispatching.

## Native effects that flags do not suppress

The initial contract forces `sendEmail:false` and never invokes `publishPostToBsky`.
That does not turn native publication into a bare putRecord:

- Public putRecord, appview document/linkage upserts, path revalidation and publish
  analytics. Owner OAuth restore may refresh server-side credentials.
- First publication upserts actor as a byline contributor, then orders contributors
  by created_at. Actor addition must be included in desired output before dispatch;
  current preview blocks until post-save set/order is fully known.
- Every publication deletes schedules, even when not scheduling; initial profile
  refuses nonempty observed schedules. Reply drafts trigger another PDS reply record
  and cleanup; current profile refuses them. Plain supported hyperlink facets are
  not DID/AT mention facets; actual mention/embed branches remain unsupported.
- First release creates mention/embed notifications when corresponding richer
  facets exist. First newsletter-enabled publication claims a `skipped` send slot
  with email disabled, inhibiting later firehose broadcast; that is a real native
  effect, not proof that no newsletter state changed. Link/write failures still
  prevent a guarantee of global no-email behavior from flags alone.
- Pro owner publication version snapshots are conditional/best effort.
- **Every publication runs gatedImages.removeStale**, even image-free updates;
  historical gated copies not referenced today can be deleted. There is no action
  flag to suppress this. Current preflight explicitly blocks it: no verified empty
  historical-storage witness is available through the inspected native RPC.
- Native first-record linkage failure tries compensating PDS delete. This is
  server behavior, not a Leafbud cleanup authorization or proof rollback succeeded.

Parent judgment needed: retain the publication refusal until an upstream no-cleanup
capability/authoritative empty-storage witness exists, or seek explicit operator
acceptance of that unavoidable native lifecycle effect. Do not remove the blocker
merely because today's source contains no images. No delivery ticket implementation
or general resource framework is commissioned here.

## Proposed execution/recovery join — not implemented

Keep the existing keyed writer and single source. A fresh Sol high should first
settle size/effect policy, then compose the existing locked save path; do not call
public `save_native` recursively while holding its stable-ID lock. Prepare complete
release input before mint/save, then save first only when needed, keeping compact
durable progress between editor and publication phases. A partially saved editor
is not a published receipt, and `push` must remain editor-only.

Do not freeze a final state enum from this prose. Proposed compact evidence is
attempt ID, original authored hash, saved-editor/native witness, resolved owner/
context, expected old URI/CID for updates, normalization policy, complete desired
public-core hash after actual page IDs exist, dispatch phase, and optional returned
URI/record digest/date. No body, executable request journal or unbounded history.
Before network dispatch persist that it **may have been sent**, including the
pre-send crash window; publish lock is neither idempotency nor input compare/swap.

Normalization must be specific, not arbitrary wildcards. Hash the fully resolved
desired projection after save with only *declared server-assigned fields* represented
by compact policy: first key/path from authoritative new association, first missing
date or explicit future-clamp from actual native result/association. Updates require
the exact old URI and `/old-rkey`; explicit nonfuture date remains exact. On result,
derive candidates exclusively from those permitted fields, require action record,
native association and fetched PDS meaning to agree, and compare every remaining
public field/page identity. Unknown fields/resources never join that substitution.
On lost response, authoritative token/root association supplies candidate identity,
not title/recency search. If evidence cannot bind the policy's exact resolved values
and full original meaning, retain uncertain/partial rather than accept any date/body.

Before any confirmed receipt, add **record-content CID verification**, independently
of status's current honest PDS-reported CID. Recommended receiving implementation:
reuse already-resolved Jacquard 0.12.1 `Data`/`RawData::to_dag_cbor` canonical encoder,
with actual AT JSON `$link`/`$bytes` conversion, SHA-256 and standard CIDv1 library;
enforce dag-cbor/SHA-256/base32lower/32-byte profile. Inspect and fixture-test that
conversion before choosing it; do not encode `serde_json::Value` blindly as maps or
hand-roll CBOR/tag 42. Independent literal vectors must include links, UTF-8, key
ordering, integer limits and mismatched digest. This checkpoint loaded CID guidance
and inspected that encoder but **did not implement or claim rehash verification**.
Rehash proves content addressing, not actor, time, authorship or delivery. Still
require exact URI/owner, authoritative native linkage and complete semantic match.

After dispatch, timeout/lost response/publish_failed never permits blind retry.
Finite recovery reads current linkage/editor/output and distinguishes: no effect
prepared; editor achieved but release undispatched; dispatch uncertain; public
correspondence verified; partial/unsupported; or source changed. First release may
need to discover its URI only through verified native linkage. Updates inspect the
exact old URI/CID, never fall back to a new first publication. Current content equal
to old input is not proof that this invocation caused it or delivered notifications.

Use a separate loss-checked `mdx` release authority to adopt draft→published URI,
accepted normalized date/path and `Released` only after full verification. Preserve
UUID, keyed native continuity, raw author trivia/comments, old pending fingerprints,
names and their independent witness. Ordinary tracking/adopt_published must **not**
gain permission to advance release evidence. If source intent changed during awaits,
do not rewrite it or retroactively call new intent the original success; retain
compact uncertainty/actual locator. Unknown future owned state remains a refusal.

## Verification and next slice

Regular workspace nextest: **368/368**, six opt-in deployment skips, run
`bd6f27a2-988f-4225-8e3c-fefa9887682b`. Strict workspace/all-target Clippy passed;
formatting and owned Markdown checks accompany this receipt. Existing native Yrs
writer is unchanged; no independent Yjs/deployed probe was rerun.
Named proof: [transport fixtures](/crates/leaflet/web/src/publication/tests.rs),
[readiness/guards](/crates/workflow/src/publication/tests.rs), and exact-default-theme
projection test. The root unsupported-command regression still passes.

**Road ahead:** fresh Sol high continues `leafbud-release-native` only after parent
review: settle unavoidable cleanup/size policy; implement content-CID verification,
locked save→dispatch→exact observation and separate release patch/recovery authority
as one coherent batch. `leafbud-release-delivery` remains dependent and uncommissioned.
No live proof is authorized; ticket acceptance cannot be closed from fixtures alone.
This checkpoint measured about 200k context before final documentation/handoff and
stops at the major boundary rather than continuing execution in this session.

## Verification follow-up — 0.23

The bounded [content-CID and native inline-size checkpoint](/design/publication-verification0.gpt61s.md)
implements the two independent verification prerequisites described above, with
fixed independent JS vectors and actual pinned native/Yjs size fixtures. It does
not implement publication execution/recovery, release patching, delivery or live
proof. Root `publish` remains unsupported and `leafbud-release-native` in progress.

Scope clarification for the hidden-effect prose: `publishLeaflet` constructs
`gatedImages` **only when `publication_uri` is present**; removeStale then runs for
every such publication-container action. Standalone is not subject to that
gated-image cleanup branch. The 0.22 preview already scoped its blocker correctly;
this clarification does not resolve or relax the parent/operator policy question.

## Execution follow-up — 0.25

The [native publish receipt](/design/native-publish0.gpt61s.md) implements the
save→durable-dispatch→content-verified-observation→separate-settlement join and
finite recovery. The operator accepted native publication-container housekeeping;
preview discloses old image copies may be deleted, count unknown, rather than
retaining the historical policy blocker above. A one-shot live standalone smoke
refused missing deployed `reply_drafts` evidence before dispatch; its fresh draft
was deleted and token observed gone. Fixture success is not live public proof.

## Deployed follow-up — 2026-11-10

The [current receipt's deployed correction](/design/native-publish0.gpt61s.md#deployed-witness-correction--2026-11-10)
supersedes the prior omission diagnosis: the inspected API returns explicit null
for both selected one-to-one relations, matching the pin. The guard now
distinguishes null from an absent/malformed/nonempty field in both adopted and
minted scopes, without an exemption or re-pin. One separately authorized final
standalone disposable first/update settled with full content-CID correspondence,
then record/editor deletion was observed and the local file unbound. Actual
retained update Flight evidence grounded the bounded decoder repair; first-response
bytes remain unavailable. Parent review owns acceptance, not the old blocked-smoke
or the new artifact completion alone.
