---
type: VerificationCheckpoint
title: "Publication content addressing and native inline-size witnesses"
description: "Bounded pure verification prerequisites for a future native execution/recovery join, not publication delivery."
resource: /design/publication-verification0.gpt61s.md
status: in-progress
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3bb92bc4ffeN77oF8X2ayoH9a"
    title: "Bounded content-CID and native inline-size verification commission; synthetic/loopback only, no publication execution or cleanup-policy decision"
  - resource: /design/native-publication0.gpt61s.md
    title: "Receiving pinned action/effect and preflight foundation; execution remains open"
  - resource: /crates/atproto/src/content.rs
    title: "Strict AT data conversion and canonical library-owned DAG-CBOR content addressing"
  - resource: /crates/workflow/src/release.rs
    title: "Separate content-verified acquisition; ordinary status retains reported evidence"
  - resource: https://atproto.com/specs/data-model
    title: "Signed 64-bit integers, no floats, singleton links/bytes and standard base64"
  - resource: https://ipld.io/specs/codecs/dag-cbor/spec/
    title: "Canonical map keys include major type/length: length-first, then bytes; tag42 with zero prefix"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/factsToPagesRecord.ts
    title: "Actual native facet fragmentation, scalar/list/page wrappers and themes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/offloadPagesToBlob.ts
    title: "Upload only above 102400 UTF-8 bytes of JSON.stringify(pages)"
---

# Publication verification checkpoint

## Boundary

Coordinated **0.23.0** adds verification APIs, not an execution workflow. Root
`publish` remains unsupported. `leafbud-release-native` stays in progress. No
live corpus/auth changes, mint, PDS write/upload, publication, email/share or cleanup
is authorized or performed. The publication-container gated-image policy question
belongs to the parent/operator and does not block this independent work. The pinned
archive is clean at `600c7b5f1f7b31b66fb626c293cc104312a12caf`.

## Record content addressing

`atproto::content::{record_content_cid,verify_record_content}` builds only validated
Jacquard 0.12.1 `RawData` variants and uses `RawData::to_dag_cbor` (resolved
`serde_ipld_dagcbor` 0.6.4), SHA-256 and `cid` 0.11.3. It does **not** deserialize
through permissive RawData inference: invalid numbers/data could serialize as
bytes, and string-only CID wrappers could serialize as text. Singleton `$link`
becomes a parsed CID/tag42, `$bytes` becomes bytes; ordinary strings remain strings.
Floats/out-of-signed-64-bit integers, malformed special maps/bytes/CIDs, unknown
reserved fields and excessive nesting produce typed refusals. Unknown normal map
fields remain hashed; no generated record struct can drop them. The XRPC JSON
reader now rejects duplicate keys recursively rather than losing them before
verification. This is data-representation verification, not a lexicon validator.

`PublicReader::fetch_content_verified` is a separate stronger read. It requires
an actual reported CID and binds immutable returned content to `VerifiedContent`.
Old `fetch`, status/diff and historical observations are **not** retrospectively
rehash-verified. Missing/unavailable/URI-type-syntax mismatch still pass through
their existing acquisition boundary; digest mismatch and absent CID are separate
stronger-API errors. Content addressing proves no actor/authorship, time, native
association, semantic correspondence, repository signature or delivery.

Four independent fixed JS vectors include Unicode, length-first key ordering,
tag42/zero prefix, byte strings, plain strongRef CID strings, signed integer
boundaries (JS BigInt), nested null/false/empty values and mutation mismatches.
`experiments/16-publication-verification/cid.mjs` reproduces exact literal bytes/CIDs
without calling Rust. One relevant correction to loaded skill prose: DAG-CBOR
sorts **encoded** keys, including type/length, not lexical UTF-8 payload alone;
the actual specification, library encoder and independent vector agree.

## Next prerequisite

Native size evidence must use actual active loaded/cumulative Yrs segmentation,
not normalized semantic runs. Reuse the existing inline/block/page encoder for a
transient supported view retaining those boundaries; count UTF-8 JSON escaping,
all facet/key/wrapper overhead, list grouping and native page IDs. New compiled
UUIDs are bounded proposals, not confirmed associations. Recheck fresh saved native
facts before any future dispatcher; a preview is never server CAS. Unknown
resources/themes/graphs or unsupported text still refuse. No general publisher or
blob upload capability is commissioned here.

## Checkpoint evidence

Targeted CID/strict-JSON/stronger-read gate: **4/4**, nextest run
`f0d5f59b-d384-4d1e-86c8-33c634b8ace2`; independent JS CID vectors pass. Measured
context at this boundary: **184103** (lagging provider report). Final workspace
verification and size-proof return follow in a separate coherent checkpoint.
