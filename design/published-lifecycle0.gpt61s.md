---
type: Design
title: "Published native adoption and independent release observation"
description: "Initial code-grounded checkpoint for verified published bindings, current working metadata and separate public evidence; no publication effects."
resource: /design/published-lifecycle0.gpt61s.md
status: implementation-design
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3c10b84effeXUnnnL5oZ5M7TJ"
    title: "Commissioned release-adopt then release-state; inspected clean pinned native archive and receiving source/observation/effect boundaries"
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

This checkpoint does not yet claim an implemented public projection or published
workflow. A consequential contradiction of the one-source/editor/public-role
contract returns to the parent before a broader replacement model is built.
