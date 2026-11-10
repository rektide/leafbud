---
type: ImplementationReceipt
title: "Published native adoption and independent release observation"
description: "0.21 verified published native adoption/refresh/save, explicit lifecycle transition and independent bounded public projection/status/diff; initial design evidence retained."
resource: /design/published-lifecycle0.gpt61s.md
status: implemented-awaiting-parent-review
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
updated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3c10b84effeXUnnnL5oZ5M7TJ"
    title: "Commissioned release-adopt then release-state; inspected clean pinned source, implemented native/public identity and independent evidence, verified CLI/HTTP/Phoenix and exact source/receipt boundaries"
  - resource: /doc/release/README.md
    title: "Maintained published/public capability and evidence contract"
  - resource: /doc/mdx/coverage.md
    title: "Named executable public/native/source cases and honest coverage limits"
  - resource: /crates/workflow/src/published.rs
    title: "Exact authoritative association and URI discovery candidate selection"
  - resource: /crates/workflow/src/release.rs
    title: "Credential-free public reads and independent versioned evidence"
  - resource: /crates/leaflet/records/src/projection.rs
    title: "Bounded native public wire-to-existing-document mapping with strict coverage"
  - resource: /crates/cli/tests/push.rs
    title: "Actual published import/edit/save/pull/public-only status/release diff CLI proof"
  - resource: /crates/workflow/src/watch/tests.rs
    title: "Actual Phoenix published poke adoption preserving URI/names/release witness"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted one-source, save versus release and current-endpoint diff contract"
  - resource: /design/document-sync1.gpt61s.md
    title: "Separate input/output evidence, date preservation and native path constraints"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Exclusive lifecycle, verified binding and unknown imported release baseline"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Historical native publication/linkage evidence; current pinned source controls adapters"
  - resource: /doc/mdx/README.md
    title: "Receiving selected-field ownership, comments and future/pending state refusal"
  - resource: /crates/workflow/src/import.rs
    title: "Current draft build, explicit native origin and guarded no-duplicate import"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Current scoped observation, association reconciliation and editor receipt settlement"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Separate tracking versus author-content patch authorities"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_leaflet_data.ts
    title: "Authoritative native token/root/rights and sibling linkage/document/publication joins"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/getPostEditLink.ts
    title: "Owner-gated edit capability; public page deliberately carries no editor token; publication-only resolver"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/migrations/20260721000000_add_pull_data_cvr.sql
    title: "Pull reads working linkage table metadata, publication first then standalone, independently of documents.data"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cvr.ts
    title: "buildExtras maps working linkage title/description/tags/preferences, not public record metadata"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "updatePublicationDraft updates both linkage-table contexts without publishing; loose metadata updates permission_tokens instead"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "Actual owner selection, stable URI/rkey, path/date normalization and independent public-record effects"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/factsToPagesRecord.ts
    title: "Native public block/page projection, facets and explicit image/poll resource effects"
---

# Published native adoption and independent release observation

## Initial checkpoint: selected receiving shape

This is an implementation design for `leafbud-release-adopt` then
`leafbud-release-state`, not a release-effect commission. The archive was clean
at **600c7b5f1f7b31b66fb626c293cc104312a12caf**. No deployed probing is needed.

Keep the existing complete native `Binding` group; add its optional exact public
URI and validate the lifecycle/group together. Draft bindings cannot contain a
public URI; published sources require a verified URI. Do not loosen draft guards
globally. URI-only local public adoption is **not selected** for this slice:
without a recoverable legitimate editor, import reports editing-unavailable and
writes nothing. Public observation remains independently useful without native
write authority. No synthetic token, editor mint or second local body is needed.

Native association comes from `get_leaflet_data`'s actual joined linkage rows,
including siblings only where the root/entity-set association is coherent.
Require one exact document identity and consistent joined document/publication
ownership, not the ambient account. URI lookup can select exact candidates from
authenticated `/home` discovery then reverify through the RPC; discovery omission
means editor unavailable through that inventory, **not public record missing**.
Do not search titles, paths or recency. The owner-only `getPostEditLink` action is
evidence of the capability boundary, not an excuse to guess its dynamic action ID
or to pretend it supports standalone documents. Missing/ambiguous/mismatched
linkage preserves all existing local source and receipts.

## Working metadata is not public metadata

The CVR SQL and `buildExtras` read `leaflets_in_publications`, falling back to
`leaflets_to_documents`. These rows remain working title/description/tags after
publication. `documents.data` is a separate released record; never use its stale
body/title as a replacement for current editor content. Published standalone
metadata must therefore use the linkage row (and `updatePublicationDraft` if safe
saving is included), **not** the loose-draft permission-token metadata operation.
The native mutation contains no public put/release/email effect. Existing native
write guards still need specific lifecycle/URI checks and HTTP proof before that
capability can be advertised. Read/refresh is the core acceptance boundary.

Ordinary observation cannot silently turn an existing draft binding into a post.
Report the observed lifecycle transition with an explicit adoption decision;
preserve UUID, source keys, all native locators, names and pending provenance.
Published refresh retains the established public URI and never substitutes
configuration defaults. Unknown native semantics stay zero-write refusals.

## Independent release evidence

Use a named/versioned public-meaning projection in the records domain. It excludes
source labels, names/trivia, writer and editor receipts; includes only verified
public content, metadata and explicit release intent. Exact facets, page IDs,
path/date normalization and resource boundaries must follow pinned native mapping.
Unsupported output fields/resources must refuse a complete comparison rather than
vanish. In particular URL images do not prove the bytes/blob identity a publisher
would upload; offloaded/gated content is not an empty body.

`released` is optional compact **confirmed correspondence**, not every public read:
profile, input hash, actual output CID/path/date and linkage. Imported CID alone
does not prove which editor input produced it. Initial imported baseline remains
unknown; read-only status/diff never settle it. Fetch public records using the
existing unauthenticated ATProto reader, independently of native editor rights.
Known absence, unavailable fetch, unsupported projection, no request and offline
are different evidence. A public CID changed out of band remains visible even
when current editor receipts match. No historical body or new state store.

Preserve observed publication date on initial adoption without interpreting it as
`--now`. Later editor pull keeps local date/path intent and the old release witness.
Release diff passes comparable current local-public and actually fetched public
Markdown to the existing foreground external tool; it does not compare a stored
historical body or silently drop unrepresentable public fields.

## Implementation and verification join

Land the model/association/read vertical with literal independent HTTP fixtures,
then public projection/observation/status/diff. Operation-specific MDX authorities
must retain full-output reparsing, body/comment and source-only-field rules.
Duplicate public URI accounting joins UUID/native binding accounting. Locks,
canonical operands, late full-selection checks and exact byte guards remain.
Unknown future owned state and pending attempts block mutation.

Prove current unpublished editor content differs from public body during adoption,
refresh/watch retains it, missing/ambiguous/wrong-owner/origin/permission cases
refuse, and read-only public observation needs no editor authorization. Any included
published editor save must show unchanged public record/action counters and release
witness. No live corpus, credentials, new mint, release/share/email/upload/cleanup.
Coordinated minor version and final workspace tests/fmt/strict clippy/docs follow.

At this initial checkpoint no implemented public projection or published workflow
was claimed. A consequential contradiction would have returned to the parent
before a broader replacement model. Implementation results follow separately.

## Implementation return — coordinated 0.21.0

Implemented the commissioned adoption/state pair without a new store, historical
body, general evidence framework or publication effect. Initial design checkpoint
**8adbd74a** preceded native adoption/save **fece404a** and independent release/
explicit lifecycle implementation **1661e914**. Review hardening **27b6f89f** adds
published inventory/legacy-profile validation, scoped endpoints, stale-source
evidence and real-patch preview checks. Later owned documentation records
the maintained receiving contract. Parent review/acceptance is separate.

### Receiving APIs and behavior

- `Binding.uri` is optional only for draft; parser validates complete native group
  plus exact DID-based supported public URI for published sources. Existing draft
  shape remains valid without a shim. New local URI-only adoption is deliberately
  unsupported and reports editing-unavailable, not a made-up capability.
- `published::association` verifies actual native/sibling root, unique public URI,
  joined document/type/site/publication and real owner identity. URI import uses
  exact `/home` candidates then RPC verification; omitted/ambiguous/native mismatch
  is not a public-body fallback or authorization to mint. `import --all --state
  published` and explicit tokens work; reimport reuses guarded refresh at the known
  path. Import JSON is version 2, with a separate `refreshed` count.
- `observe_editor` supports either established lifecycle, and includes public URI
  in identity checks. Published remote hashes witness current facts/extras and
  native/public identity—not joined released bodies, dates or public metadata.
  Public-only appview changes therefore leave editor agreement clean.
- Explicit `pull --adopt-published`/`refresh::Policy.adopt_published` uses the separate
  pure `mdx::adopt_published` authority. Ordinary tracking/adoption cannot transition
  lifecycle. Only URI/lifecycle change on an otherwise identical native binding;
  omitted/null date/path adopt actual values, other source-only choices/names/history
  survive. Pending/conflicts retain ordinary guards; preview writes nothing.
  Prospective URI duplicates are checked even before a URI exists in old source,
  under the existing OS-lock mechanism keyed by public identity.
- Published editor-only `push` reuses the native keyed compiler/runner and terminal
  recovery. Published standalone title/description/tags use linkage-table mutation
  and persisted pull values, not loose-draft token metadata. Correct context is
  selected in the compiler and native scope. No public action is invoked.
- `State.released: Option<Released>` is compact confirmed profile/input hash/CID/
  path/date/time; unknown input baseline is absence, not CID-only pseudo-success.
  Current parser rejects future receipt fields/profiles. Editor tracking/adoption
  require exact receipt equality; saves, pulls, watch, status and diff never advance it.
- `records::projection::{working,from_record}` provides `leaflet-public/1`; generated
  validation precedes strict mapping coverage. Actual native block/page/facet rules
  lower into the existing `Document`, not a parallel prose comparison model.
  Source keys, names, writer/resolution/trivia are excluded. Dates normalize by
  instant; unknown/richer fields and unresolved resources refuse instead of vanish.
- `release::PublicReader::{new,with_endpoint,fetch,comparison}` uses the existing
  resolver/XRPC public client and URI's actual repo DID, with no sessions/cookies/
  bearer tokens. `release::observe` independently classifies input/output/current
  correspondence and confidence; no receipt adoption policy is enabled.
  Bound-owner resolution ignores ambient PDS/account defaults; only explicit
  `--pds`/library override selects another endpoint, with that scope reported.
- `status::inspect_with_public` adds independent per-item release evidence; status
  JSON is version 3. Normal CLI status requests both remote edges; `--public-only`
  bypasses the native credential store entirely, and offline requests neither.
  `diff --edge release` fetches a current record, renders matching current public
  metadata plus the existing human Markdown body, and invokes the existing waiting
  external tool. No historical body, invented missing view or source write.
- Existing ATProto client origin validation now rejects userinfo/path/query/fragment
  and lookalike HTTP loopback hosts; requests have a bounded timeout and no redirect
  following. This is read-path hardening of the reused client, not new auth logic.

### Bounded judgments and limitations

The [maintained public contract](/doc/release/README.md#public-meaning-profile)
names the supported mapping and refusals. Images need verified blob-byte identity;
offloaded/gated/richer output is not an empty public body. Multiple working
contributors cannot claim publication order from unordered native pull. Repeated
page links cannot claim deduplication when native projection repeats subpages.
These and colored theme-dependent highlights, richer lists, legacy release
comparison and unresolved date/path remain honest unsupported projections.

Imported current public/editor equality remains equal-untracked, not a confirmed
release baseline. Known evidence separately reports input/output/both changes and
converged-unaccepted. Out-of-band output CID changes remain visible even when
projection is unsupported or the editor is unchanged. CID evidence is explicitly
**PDS-reported, syntax/profile validated, not rehashed or signature verified**;
comparison also requires complete supported current meaning. A claimed unchanged
CID with inconsistent meaning is not clean. Future native publication owns actual
input/output verification, normalization settlement, pending effects and delivery.

No deployed published proof, `~/l` rewrite, account probing, credential edit, new
mint, public release/update/delete, upload, email/share or cleanup occurred. All
new effects were temporary files and loopback native/PDS/Phoenix fixtures.

### Executable evidence

Final regular workspace gate: **357/357 tests, six opt-in deployment skips**
(run `998ddeea-657c-4e5a-b2cc-bee2f3ffa8ae`). Formatting and strict workspace/
all-target Clippy pass. Markdown/OKF checks cover README, both maintained MDX
documents, the public contract and this receipt.
The [maintained matrix](/doc/mdx/coverage.md) A1/A2/S1/S2/C7 links exact cases:

- Native/public literal disagreement through both import contexts; missing,
  ambiguous, wrong-owner/join/site/root, unreadable and malformed metadata refuse.
  Exact URI discovery, all-published selection, duplicate public identities across
  origins and explicit/prospective lifecycle adoption are covered.
- Actual CLI import → BOM/CRLF/custom/comment source → explicit edit/new paragraph
  → preview/save/readback/no-op pull/repeat → later native refresh. UUID, native
  node/placement/text associations, actual date and public identity remain.
- Both-context native HTTP editor saves retain public record content and older
  release witness. Published standalone never emits loose metadata operations.
  Requests contain only native editor RPCs, not mint/release/upload actions.
- Actual Phoenix poke-to-file runs draft and published variants; refreshed source
  retains manual labels, comments, names, native URI and older release evidence.
- Independent literal public facets/UTF-8 offsets, scalar kinds/bytes, list styles,
  page identity/references and date/default expectations supplement roundtrip controls.
  Unknown/richer fields, invalid facets and unresolved image identity refuse.
- Public HTTP state matrix covers unknown baseline, input/output/both/convergence,
  missing/unavailable/mismatch/linkage/unsupported/offline/not-requested. Actual CLI
  public-only status and fetched diff run with a deliberately unusable native
  credential store/origin; public requests carry no cookie/bearer/native RPC.
  Status/diff preserve source bytes/mtime and old evidence.
  A public-await source race explicitly invalidates clean/directional/equality
  claims while retaining the fetched CID's old bound-URI scope. Unknown future
  release fields/profile block parse and managed patches without dropping data.

The Yrs text implementation is unchanged; earlier seven-group independent Yjs
evidence is retained, not rerun or newly claimed. Existing full-selection source,
lock, pending/future-state and conflict tests remain part of the passing regular
gate. No server/filesystem CAS or independent publisher execution is claimed.

### Road ahead

`leafbud-release-adopt` and `leafbud-release-state` are closed with their implemented
bounded acceptance evidence and current pointers/notes; parent integration
acceptance remains separate. Next **fresh Sol high** should design/implement
`leafbud-release-native` after parent integration: verify native action discovery,
preserve exact output identity/date, settle accepted normalization and recover
uncertain effects without blind replay. The open
`leafbud-release-delivery` follows it, not this batch: explicitly selected email/
share outcomes must not be inferred from CID equality or retried automatically.
No live publication/delivery proof is authorized by this implementation return.
