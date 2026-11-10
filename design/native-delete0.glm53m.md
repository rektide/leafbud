---
type: ImplementationReceipt
title: "Deliberate native editor deletion: draft and published"
description: "Preview-first deleteLeaflet/deletePost transport, guarded workflow, CLI surface, verified published-record-deletion findings and the authorized disposable live smoke."
resource: /design/native-delete0.glm53m.md
status: implemented-awaiting-parent-review
generated:
  at: 2026-11-10
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e39c64844ffeh98er3US2A0gQC"
    title: "Implemented delete transport/workflow/CLI, verified the upstream published-record path, ran the authorized disposable live smoke and wrote this receipt"
  - resource: /design/native-publication0.gpt61s.md
    title: "Action discovery/transport structural template and verification discipline"
  - resource: /crates/leaflet/web/src/delete.rs
    title: "Literal deleteLeaflet/deletePost action adapter and wire fixtures"
  - resource: /crates/workflow/src/native/delete.rs
    title: "Guarded preview-first deletion workflow with observation-based verification"
  - resource: /crates/document/mdx/src/patch.rs
    title: "unbind_deleted local conversion"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/deleteLeaflet.ts
    title: "Queues image cleanup, deletes entity set and permission token in one transaction; void return"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/lish/[did]/[publication]/dashboard/deletePost.ts
    title: "Verified published-record deletion action: PDS deleteRecord on both document collections, documents rows, leaflets_in_publications linkage"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/(home-pages)/(writer)/home/LeafletList/LeafletOptions.tsx
    title: "UI invokes deletePost(documentUri) before deleteLeaflet(token) for published posts; both referenced from /home and dashboard chunks"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/index.tsx
    title: "PermissionToken wire type: id, root_entity, permission_token_rights"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/inngest/functions/delete_records_from_pds.ts
    title: "Operator inngest counterpart mirroring deletePost cleanup"
---

# Deliberate native editor deletion — draft and published

## Standing and scope

Version **0.24.0** adds `leafbud delete FILE [--yes] [--purge] [--json]`:
preview-first deletion of a bound native editor through Leaflet's own
`deleteLeaflet` action, extended under the 2026-11-10 operator scope expansion
("it should work for draft & published please") to full published deletion
through the native `deletePost` record-deletion action. All upstream claims
below were verified from the archive **clean at 600c7b5f** (worktree checked
before reading). The one-time live smoke was authorized the same day: one
disposable standalone draft, quiet, self-cleaning. No publish, email, share,
`~/l`, P1, the standalone verification draft or any pre-existing object was
touched; published deletion ran against loopback fixtures only.

## Upstream facts (pinned source)

`actions/deleteLeaflet.ts` takes one `PermissionToken` argument
(`src/replicache/index.tsx`: `{id, root_entity, permission_token_rights[]}` —
exactly the `get_leaflet_data` result object the UI passes from
`useLeafletPublicationStatus`). Server-side it re-reads everything from the
database and consumes only `id`: publication drafts require an owner/member
identity, standalone published docs require the document owner, and **a token
without a `write` right returns silently having done nothing** — which is why
our preflight refuses when the bound entity set has no verified writable
right. In one drizzle transaction it queues image-blob cleanup for the root's
image facts, deletes every entity in the token's entity set, and deletes the
permission token; rights/linkage rows cascade via schema FKs
(`drizzle/schema.ts`, `onDelete: "cascade"`). It does not delete
`documents` rows or PDS records. It returns void and calls `next/cache`
`refresh()`.

## Published-record deletion — the answer

**A verified, reachable native path exists; it is implemented.**
`app/(app)/(identity)/lish/[did]/[publication]/dashboard/deletePost.ts` is a
`"use server"` action the post menu invokes **before** `deleteLeaflet` for
published documents (`LeafletOptions.tsx` → `DeleteAreYouSureForm`: `deletePost
(documentUri)` first, then `deleteLeaflet(token)`). After an authority check
(PDS owner, or a confirmed contributor of a publication the document is linked
to) it restores the record owner's OAuth session server-side, revalidates the
document paths, and then deletes from **both** PDS collections through the
generated lexicon wrappers `agent.pub.leaflet.document.delete` /
`agent.site.standard.document.delete` (each `com.atproto.repo.deleteRecord`
under the hood, per-collection failures swallowed `.catch(() => {})` because a
document exists in only one), deletes the `documents` rows and the
`leaflets_in_publications` rows for that doc. Typed results:
`{success:true}` or `{success:false, error:{type:"oauth_session_expired"|"not_authorized",
…}}`. `unpublishPost` in the same module deletes the record while keeping the
draft editor; the operator-triggered inngest function
`app/api/inngest/functions/delete_records_from_pds.ts` mirrors the same
cleanup and cites deletePost as its model. `src/documentReplies.ts` and
`actions/replies.ts`/`questions.ts` delete other record types (replies,
questions) — not site documents. So the earlier open question resolves as:
**deletePost + deleteLeaflet, in that order, is the complete published-post
deletion the native UI itself performs.**

## Transport (`crates/leaflet/web/src/delete.rs`)

Same discipline as `publication.rs`. `prepare_delete` requires the cookie,
reads authenticated `/home` once (identity DID + same-origin `/_next/static`
chunk scan), and resolves exactly one `createServerReference(…,"deleteLeaflet")`
pairing (required) plus `deletePost` (optional; absent only disables the
published path). Zero/ambiguous references refuse; no saved hash, foreign
asset or fallback. `invoke_delete_leaflet` POSTs to the same-origin `/home`
with the literal Next action headers and one argument mirroring the declared
`PermissionToken` shape built from the verified observation (`{id, root_entity,
permission_token_rights}`); the server reads only `id`, so the narrower-but-
declared shape is faithful, bounded and avoids dumping unrelated linkage rows.
`invoke_delete_post` POSTs exactly `[document_uri]` after a strict DID-record
URI validation (both document collections only). Void success decodes only
`null`/Flight `$undefined`; `deletePost` decodes only the literal success/error
union with `deny_unknown_fields`. Redirect/not-found/non-Flight/undecodable
responses are `ActionResponse`/transport errors meaning **uncertain**, and
nothing retries. Diagnostics never include credentials, tokens or server prose.

## Workflow (`crates/workflow/src/native/delete.rs`)

`delete_editor` holds the stable-ID lock plus the origin/token binding lock,
refuses pending attempts with a `recover` hint, and requires the live editor
to match the binding exactly: token, root, a **writable** right for the bound
entity set (the silent-no-op guard), publication association, lifecycle, and
for published the exact document URI via the authoritative association. A
published document additionally preflights the public record through the
independent `PublicReader`; a record already missing while the editor resolves
is divergence and blocks. Preview lists exactly what dies — entity set,
permission token with cascaded rows, queued image-blob cleanup (advisory local
image-block count; the server's entity scan decides), and for published the
exact record URI, documents rows, linkage rows, native order
(record → editor), the plainly-larger public visibility, and appview lag.
Execution (`--yes`) mirrors the native order: `deletePost`, then — only after
the record is observed missing (immediately on success, or by observation
after an uncertain response, with a coherent post-record editor recheck) —
`deleteLeaflet`. Final verification is **by observation**: the token must 404
and the record must go missing. Typed refusals are `refused` (exit 1 via CLI
failure contract: preserved, nothing concluded); uncertain outcomes are
`uncertain` (exit 4) and never retried. An editor already deleted out of band
reports `remote-deleted`; `--yes` then performs only the local conversion
(published requires the record gone too). After verified deletion the local
file converts through `leafbud_mdx::unbind_deleted` under the held lock and
original-byte guard: binding/node associations/sync/writer/pending/last
resolution/release evidence are dropped; identity, main-page identity, names,
authored content, keys, comments and exact body bytes are preserved and the
result reparses. `--purge` removes the file instead. No other file is touched;
full-selection duplicate checks run as usual.

## CLI, version, and evidence

`leafbud delete FILE [--yes] [--purge] [--json]`: exit **0** preview/deleted/
local-unbind, **4** pending/blocked/uncertain/source-changed, **1** refused or
operational failure, **2** usage. README documents the surface. Coordinated
workspace bump to **0.24.0** (all nine crates, exact internal pins, lock
updated) per the pre-alpha versioning practice.

- Web transport: literal wire/fixture contracts, request-shape assertions,
  uncertain-case non-retry, discovery guards (missing/ambiguous/anonymous,
  optional deletePost) — `crates/leaflet/web/src/delete/tests.rs`.
- Workflow guards: preview no-touch, execution observation, purge, pending
  block, wrong origin, server-side drift (root, lost write right), uncertain
  both ways, already-deleted recovery incl. published divergence, published
  preview/execution order, typed refusal, uncertain deletePost continuation,
  record-divergence block — `crates/workflow/src/native/delete_tests.rs`.
- Local conversion: draft and published sources preserve authored material
  while dropping exactly the dead tracking; unbound/pending refuse —
  `crates/document/mdx/src/tests.rs`.
- CLI through the real binary: preview/execute/purge/unbound-operand/mismatch
  with honest exit codes — `crates/cli/tests/delete.rs`.

## Authorized live smoke (one time)

`LEAFBUD_NATIVE_DELETE_PROBE=authorized-disposable-draft` ran the marker-gated
probe once (`.test-agent/native-delete/`, budget marker created before mint):
a **fresh standalone disposable draft** was minted through the normal push
first-binding path, its controlled save verified (`saved`), then deleted via
the new capability — outcome `deleted`, token independently re-observed 404,
local source cleanly unbound with authored content and UUID intact. Private
receipt: `draft-smoke.result.json`. **Published deletion was not exercised
live**; test-publish authorization is explicitly deferred pending operator
confirmation of this record-deletion answer.

## Limits

- The void `$undefined`/`null` decode and the action-argument shapes are
  protocol-grounded synthetic fixtures, not a deployed probe of those two
  actions; the live smoke exercised the real deployed `deleteLeaflet` for the
  draft path end to end.
- Publication-draft deletion additionally requires server-side membership;
  Leafbud verifies what `get_leaflet_data` exposes and the server enforces the
  rest, surfacing as a typed refusal or an uncertain-then-observed outcome.
- `deletePost` deletes both collections blind server-side (per-collection
  errors swallowed); our post-observation verifies the exact bound URI went
  missing, which is the authoritative PDS read, while appview/search lag
  remains disclosed, not verified.
- Only standalone-draft binding was live-proven; the published path and the
  in-publication draft path carry loopback coverage only.
- `unbind_deleted` keeps the adopted `theme`/`contributors`/`publishedAt`
  author fields — they are authored/adopted metadata, not dead server
  references; a future first binding re-verifies defaults.
