---
type: VerificationHarness
title: "Independent publication verification fixtures"
description: "Reproducible JS DAG-CBOR/CID vectors and pinned native page-serialization evidence; no network effects."
resource: /experiments/16-publication-verification/README.md
status: active
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3bb92bc4ffeN77oF8X2ayoH9a"
    title: "Independent verification fixtures for content-CID and native inline-size prerequisites"
  - resource: /design/publication-verification0.gpt61s.md
    title: "Capability, proof derivation, limitations and receiving execution boundary"
---

# Independent publication verification

`pnpm install --frozen-lockfile && node cid.mjs` recomputes four fixed canonical
DAG-CBOR bytes/CIDs using `@ipld/dag-cbor` and `multiformats`; Rust is not called.
`node cid.mjs --write` deliberately regenerates the checked-in Rust test fixture.
Integer-limit inputs use JS BigInt rather than rounded JSON.parse numbers.

This directory contains only synthetic evidence. It does not load credentials or
contact a PDS/Leaflet deployment. See the [verification receipt](/design/publication-verification0.gpt61s.md).

`node native-size.mjs` additionally requires a clean archived Leaflet checkout at
`600c7b5f1f7b31b66fb626c293cc104312a12caf` (default
`~/archive/hyperlink-academy/leaflet`; override `LEAFLET_SOURCE`). It transpiles and
executes actual source modules/declarations, recording eight source hashes; it
does not copy their algorithms into a second publisher. Six fixed fixtures cover
retained Yjs fragmentation, all supported marks/links, Unicode JSON escaping,
lists/pages/hard breaks, ignored inactive text and exact native offload boundaries.
Resource hooks throw; upload is only an in-memory threshold counter, never HTTP.
`node native-size.mjs --write` deliberately regenerates the Rust fixture.

TypeScript 6 supplies `transpileModule`; latest 7 no longer exports that API, so
the harness deliberately uses 6. Existing native Yjs 13.6.15 baseline is pinned;
the old ordinary-writer harness is not rerun by these scripts.
