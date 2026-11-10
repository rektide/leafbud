---
type: VerificationCheckpoint
title: "Publication content addressing and native inline-size witnesses"
description: "Bounded pure verification prerequisites for a future native execution/recovery join, not publication delivery."
resource: /design/publication-verification0.gpt61s.md
status: implemented-verification-awaiting-parent-review
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
  - resource: /crates/workflow/src/publication/size.rs
    title: "Post-save cumulative-state/retained-native size witness using existing reader and page encoder"
  - resource: /experiments/16-publication-verification/README.md
    title: "Independent reproducible JS vectors and actual pinned native serialization harness"
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

Five independent fixed JS vectors include Unicode, length-first key ordering,
tag42/zero prefix, byte strings, plain strongRef CID strings, signed integer
boundaries (JS BigInt), nested null/false/empty values, a modern raw-CID blob
reference and mutation mismatches. Malformed modern blob wrappers refuse without
permissive inference or dropping extra fields; hashing a valid reference does not
verify referenced blob bytes or implement uploads/resources.
`experiments/16-publication-verification/cid.mjs` reproduces exact literal bytes/CIDs
without calling Rust. One relevant correction to loaded skill prose: DAG-CBOR
sorts **encoded** keys, including type/length, not lexical UTF-8 payload alone;
the actual specification, library encoder and independent vector agree.

## Native inline-size witness and derivation

`workflow::publication::size` evaluates the **existing** compiler's actual target
map, retained facts and cumulative `block/text` updates. The same strict Yrs reader
now exposes `editor::read_text_segments`: each active XmlText delta remains a
separate `Inline::Run`; ordinary `read_text` still normalizes exactly as before.
The transient desired document receives those real segments (checked against its
normalized intended meaning) and the compiled page targets. No new semantic model,
fact mutation simulator, fresh-doc replacement or publisher is added.

`records::projection::inline_pages_json_upper_bound` reuses the existing full
supported page/block/facet encoder **without** coalescing supplied segments. Its
derivation is:

1. Strict original observation/compiler and full release preflight exclude all
   unrepresented facts, resources, custom page/root themes, colored highlights,
   repeated/cyclic/unreachable graphs and unsupported marks/XML/causal state.
2. Source graph/order describes the compiler's post-save pages and flat list
   grouping. Every page ID/reference uses the actual retained or compiled UUID;
   new 36-byte UUIDs are explicitly provisional, never confirmed associations.
3. For active rich blocks only, the full cumulative assertion wins over retained
   fact content. Strict decoded delta/XML boundaries determine each native facet,
   its features/link bytes and UTF-8 offsets. Native ignores inactive `block/text`
   on code/rule/card: even malformed inactive text neither counts nor disappears.
   Native non-text list wrapping (not represented by the working model) refuses
   the size profile rather than assuming a plain wrapper.
4. The existing encoder counts every page/list/item/block/index/feature key and
   array/object delimiter. UTF-8 JSON escaping matches `JSON.stringify` for this
   string/small-integer/ordinary-map profile, including control escapes, quotes,
   backslashes, BMP/non-BMP, U+2028/U+2029 and IDs. Property order changes no size.
   One intentional conservative difference: absent code language is included as
   `language:""` (14 extra bytes), so the witness is an **upper bound**, not an
   unsupported claim to have reproduced the whole native record exactly.
5. At or below **102400** bytes the pinned native helper does not offload. Above
   the bound, no no-offload proof is offered (not necessarily proof upload must
   occur, since this is an upper bound). No blob support or policy exception.

Preview replaces its blanket size blocker only with a proven bound for actual
loaded/compiled state; reports scope, bytes/threshold, active segments, provisional
page count and the required fresh witness. A compile requiring saves is explicitly
conditional on those saves. Unbound/offline preview stays size-unwitnessed: it
does not invent a mint seed. Full source validation still happens before any
reads/effects. The future executor must validate first-binding actual seed/defaults
and evaluate its compiled desired result **before saving**, then repeat against
fresh fully saved facts before dispatch. No first mint/default claim is fabricated.

After action discovery preview re-reads raw native metadata **and facts/extras**;
a changed witness invalidates the whole preview. Final future dispatch must also
recheck fresh native input. This is not a server CAS, publication lock, persistent
authorization or guarantee against a later concurrent writer.

### Independent actual native serializer evidence

`native-size.mjs` requires the clean exact archive revision, records eight source
SHA-256s, and transpiles/executes the **actual** native serializer, text converter,
block/list grouping and offload helper. Only schema constants, the synchronous
scan declaration and the relevant pure modules load. Unsupported resource/theme
branches throw; image/poll hooks cannot execute. An in-memory upload stub only
checks which boundary the actual offload helper selects; there is no RPC/network.
Yjs 13.6.15 and actual `@atproto/api` UnicodeString produce the input/offsets.

Six fixed fixtures agree with Rust's witness: 1600 adjacent same-bold XmlTexts
produce **161205 bytes**, while normalized semantic sizing falls below threshold;
ordinary two-page mixed headings/quotes/lists/marks/links/hard breaks and escaped
Unicode is **6551 bytes** and passes; inactive invalid text/absent code language
is 256 native bytes versus the conservative 270. Actual helper tests at **102399,
102400, 102401** show inline, inline, offload respectively. Rust also verifies
post-save shorter cumulative text plus new blocks/page IDs, retained causal clocks,
code transition ignoring latent text, unsupported non-text lists and a native-fact
race after size computation. No unchanged ordinary-writer seven-group harness was
gratuitously rerun; this new independent proof addresses publication fragmentation.

## Checkpoint evidence

Targeted CID/strict-JSON/stronger-read gate: **4/4**, nextest run
`f0d5f59b-d384-4d1e-86c8-33c634b8ace2`; independent JS CID vectors pass. Measured
context at this boundary: **184103** (lagging provider report). Final workspace
verification follows the size checkpoint. Targeted publication gate: **11/11**,
run `b3ee79ee-1472-4097-be23-621ba3192272`; independent native fixture reproduction
passes. The complete initial verification gate was **377/377**, six deployment
skips, run `8db766ff-8500-4fea-9c78-eba8ed7afbc4`; strict all-target Clippy, fmt and
four owned Markdown/OKF documents passed. A final modern-blob strictness refinement
adds one test and an independent vector before the final gate below.
Reported context at this checkpoint: **222935**; scope is complete, so
only final gates/documentation/handoff remain in this session.

### Final gate and commits

Final regular workspace gate: **378/378**, six opt-in deployment skips, nextest
run `2402d841-6530-41eb-abfc-11c45cdfd12a`. Strict workspace/all-target Clippy,
`cargo fmt --all --check`, five owned Markdown/OKF documents, frozen pnpm install,
five independent CID vectors and all six actual-native size fixtures pass.
The independent harness runs are offline with respect to Leaflet/PDS; the only
upload in those fixtures is an in-memory counter. No deployed proof is implied.

Coherent code checkpoints: `12164f92` (separate canonical content addressing,
coordinated 0.23) and `460320fc` (actual cumulative/native size bound plus preview
fact-race invalidation). Final strict modern-blob refinement is a separate correction,
not a changed publication resource profile. Last measured context before final
receipt/handoff: **237667**; rotation occurs here at the completed major boundary.

## Fresh execution receiving brief

Parent integration review owns acceptance and the publication-container gated-image
cleanup question. Native constructs gatedImages **only when publication_uri is
present**; its removeStale is then unconditional. Standalone is not broadened into
that refusal. This checkpoint neither accepts nor changes that policy blocker.

Next fresh Sol high continues **the same `leafbud-release-native` ticket**: compose
locked save-first execution with durable dispatch-before-effect uncertainty,
current-owner/context/effect checks and fresh saved-native size witness; acquire
exact-URI content-verified output plus native authoritative association; compare
complete supported meaning with declared path/date normalization; use a distinct
loss-checked release-patch authority and finite recovery, never blind replay.
First byline upsert/order remains separate metadata work, not a size wildcard.
Root publish stays unsupported until that integration. Delivery/live proof remains
uncommissioned and the ticket remains in progress. No agent in this checkpoint
implements that next slice or closes a task.
