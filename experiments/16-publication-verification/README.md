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
