---
type: Report
title: "Remote reads — first small implementation slice"
description: "Read-only remote list/show implementation, recovered protocol boundaries, offline receipts, limitations and next small seam."
resource: /design/remote-reads0.gpt61s.md
status: implemented
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e46368af7ffeGOrXJkxUfkr5a7"
    title: "Commissioned and implemented the bounded remote list/show slice; recovered evidence, offline transport/CLI verification and implementation handoff"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted CLI spelling, crate boundaries and output contract"
  - resource: /design/scaffold-reset0.glm53m.md
    title: "Starting checkpoint and selective recovery instructions"
  - resource: /experiments/01-draft-list/REPORT.md
    title: "Proven RSC extraction, merge/order rules, RPC envelopes and pagination uncertainty"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "RPC metadata and synthetic response fixtures, distinct from editor-content reads"
  - resource: /design/document-sync1.gpt61s.md
    title: "Precedent requiring honest discovery completeness and dependable native locator resolution"
  - resource: https://github.com/rektide/leafspring/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/drafts/src/lib.rs
    title: "Selectively recovered flight/brace extraction and protocol behavior, not obsolete lifecycle"
  - resource: https://github.com/rektide/leafspring/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/cli/src/main.rs
    title: "Recovered token/base URL precedence and rendering precedent"
  - resource: file:///home/rektide/archive/hyperlink-academy/leaflet/app/api/rpc/[command]/get_leaflet_data.ts
    title: "Local upstream RPC source verifies direct standalone document joins and response envelope"
  - resource: file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/getPublicationMetadataFromLeafletData.ts
    title: "Local upstream source verifies standalone linkage uses document; sibling-token lookup remains a known limitation"
---

# Remote reads — implementation handoff

## Delivered boundary

Commission: the first small list/get slice, not the whole authoring system.
Accepted spelling is **`remote list --state draft`** and **`remote show
LOCATOR`**. There is no new `get` alias. Root `list` remains local and
unimplemented. Starting parent: `6830f543`; implementation checkpoint:
`66d93a35` (**Implement read-only remote Leaflet discovery and inspection**).

| Owner | Implemented interface |
| --- | --- |
| [web client](../crates/leaflet/web/src/lib.rs) | `LeafletClient::new(base_url, token)`, `list()`, `get(Uuid)`, `resolve_locator()`, `parse_rpc()`; explicit auth/origin only |
| [home extractor](../crates/leaflet/web/src/home.rs) | `parse_home()`; recovered flight concatenation/brace extraction, homepage/contributor merge, row-date/UUIDv7 ordering and href drift alarm |
| [workflow](../crates/workflow/src/lib.rs) | `RemoteOptions::connect()`, reusable `RemoteReader::list(state)` / `show(locator)`, versioned reports and coverage |
| [CLI rendering](../crates/cli/src/remote.rs) | Human summaries and `--json`, with control-character-safe terminal text and errors on stderr |

Metadata is an intentionally partial typed view alongside exact raw current
response data. Show retains `data.raw`; list retains per-Leaflet
`raw_home_rows`, not the whole account identity/email object. Unknown nested
rights, record and linkage fields survive. This is not a full document model
or a persisted sync witness.

Auth resolution is flag > environment > XDG token file; base URL is flag >
environment > production. No TOML layer is used, so adding Figment would not
solve a real layering need in this slice. Secrets are omitted from option/client
Debug, validation diagnostics and RPC error bodies; cookie headers are marked
sensitive. Redirects are disabled, requests time out after 30 seconds, and
foreign-origin editor URLs are rejected rather than fetched. Explicit HTTP
origins remain available for loopback/custom servers.

Version **0.3.0** marks the new public library/CLI interfaces. All eight
packages retain the coordinated workspace version; internal dependency pins
are exact and the root lockfile is updated. The atproto/records implementations
and experiments were not changed.

The two local upstream files cited above were read at Leaflet revision
`600c7b5f1f7b31b66fb626c293cc104312a12caf`, the same pinned checkout used by
experiment 02; no new protocol probe was required.

## Semantics and limits

- Listing defaults to all discovered Leaflets, with exclusive draft/published
  filters. A publication-associated but unpublished item is a draft, not a
  third lifecycle. Direct publication `doc` or standalone `document` linkage
  marks publication; it does not prove current PDS availability.
- Discovery reads only one `/home` payload plus its contributor rows. Every
  report says completeness is **unknown**; a clean href cross-check is not
  account-wide completeness. No guessed page limit or pagination endpoint.
- Duplicate rows retain both source rows; homepage metadata takes precedence
  over duplicate contributor metadata. Classification uses direct linkage, not
  recursively linked sibling permission tokens. Sibling associations remain
  available in show JSON, not silently discarded or advertised as resolved.
- Locators: hyphenated UUID or same-origin native `/<uuid>` URL. Query/fragment
  do not alter the token. Document AT URIs return an explicit unsupported
  resolution error; no publication-discovery workaround or fabricated editor.
- Show fetches RPC metadata/rights/linkage only. Rights are reported, not a
  write probe. No fact/Yrs decoding, MDX import/pull, local document creation,
  sidecars, minting, save or publication. Other scaffold commands still fail
  honestly.
- HTTP errors, anonymous/malformed homepage bodies, malformed RPC envelopes,
  PostgREST errors inside HTTP 200 and returned-token mismatch are failures,
  not empty successes. Errors emit no success JSON. JSON reports use version 1
  and observed outcomes; raw output can contain private content and capability
  token identifiers, so users should not publish it indiscriminately.

## Verification receipts

Only offline synthetic fixtures and loopback HTTP were used. **No live
inspection, account credentials, private payload capture or remote writes.**

- `cargo fmt --all` — completed; final `cargo fmt --all --check` verifies it.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo nextest run --workspace` — **55/55 passed**, 9 binaries; prior baseline
  was 35 tests. The 20 added tests exercise actual code, not scaffold mirrors.
- [Web tests](../crates/leaflet/web/src/tests.rs): experiment-01/02 golden
  fixtures, escaped/chunked flight, legacy UUID dates, duplicate/contributor
  merging, direct lifecycle states including standalone linkage, unknown-field
  retention, malformed/anonymous bodies, HTTP/RPC errors, token mismatch,
  URL/cookie validation, refused redirects and transport failure.
- [CLI integration tests](../crates/cli/tests/remote.rs): real subprocesses
  against loopback prove human/JSON list and show, native URL resolution,
  flag/env/XDG precedence, errors without success JSON/secret echoes, local
  `list` remaining unimplemented and usage exit 2. The committed token fixture
  is explicitly synthetic (`synthetic-file-auth`), not an account credential.
- [Workflow tests](../crates/workflow/src/lib.rs): exclusive filtering with
  pre-filter discovery counts/unknown coverage, XDG path semantics and redaction.
- Loopback responders reuse the existing test-only atproto responder by source
  path; there is no new production `web → atproto` dependency.
- `rkt check README.md design/remote-reads0.gpt61s.md` and `rkt check --okf
  design/remote-reads0.gpt61s.md` — owned documentation/link/provenance checks.

## Next useful small seam

Recommend a separately commissioned **read-only editor observation**:
`get_leaflet_data` root/rights → proven facts/pull read → raw-payload-retaining
observation in `editor`. Start from experiment 02's fixtures and the accepted
Yrs promotion sources, not the removed hand-rolled codec. Do not call that MDX
import until the keyed model/writer join is proven. Alternatively, if bulk
discovery is the next user need, verify inventory completeness before promising
“all drafts”; that is a distinct discovery investigation, not automatic scope
growth here.

Connections: [accepted interface](authoring-interface0.gpt6a.md) owns the
destination; [reset handoff](scaffold-reset0.glm53m.md) explains selective
recovery; [experiment 01](../experiments/01-draft-list/REPORT.md) supplies the
protocol and its pagination uncertainty; [earlier sync design
§7](document-sync1.gpt61s.md#7-dates-alternate-encodings-and-commands) supplies the
completeness/locator caution. [README](../README.md#remote-reads) is the current
usage entrypoint and links back to this receipt.
