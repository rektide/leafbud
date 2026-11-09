---
type: ImplementationReport
title: "Native-effect foundation: mint, requests and conservative recovery"
description: "Version-0.14 transport/model/workflow checkpoint for bind and recovery, with actual loopback proofs, a bounded deployed standalone effect, an uncertain publication result and explicit compiler/default-policy join boundaries."
resource: /design/native-effects0.gpt61s.md
status: implemented-checkpoint
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4121e55dffeF009EbMiG1Xinz"
    title: "Commissioned native-effect foundation implementation, tests and commits; operator-authorized two-draft deployment probe and conservative partial/uncertain stop"
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Parent commission for bind/recovery batch C and relayed authorization for at most two new unpublished verification drafts"
  - resource: /design/keyed-edit-flows0.gpt6a.md
    title: "Controlling identity, complete-drafting, ordered effect and recovery obligations"
  - resource: /design/keyed-authoring0.gpt6a.md
    title: "Keyed native-object foundation; whole-body regeneration is not selected"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Accepted boundaries and internal native-effect batch C"
  - resource: /design/authoring-corrections0.glm53m.md
    title: "Authored projection, stable local identity and tracking-only settlement foundation"
  - resource: /design/connected-watch0.gpt61s.md
    title: "Shared operation locks, selection revalidation and pending-preserving inbound refresh"
  - resource: /crates/leaflet/protocol/src/lib.rs
    title: "Shared full views, acknowledgments and explicit-ID transport vocabulary"
  - resource: /crates/leaflet/protocol/src/push.rs
    title: "Actual native arguments, scope validation and immutable bounded request bytes"
  - resource: /crates/document/model/src/attempt.rs
    title: "Typed compact phases, writer reservations and lifecycle validation"
  - resource: /crates/workflow/src/native.rs
    title: "Guarded source transaction and first-binding staging"
  - resource: /crates/workflow/src/native/write.rs
    title: "Internal compiler receiving runner and public finite recovery observation/settlement"
  - resource: /crates/workflow/src/native/probe.rs
    title: "Explicit opt-in deployment harness and create-free inspection of privately recorded test drafts"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/getFactsForRoots.ts
    title: "Verified RPC takes roots and returns grouped descendant facts with server depth fixed at three"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/push.ts
    title: "Actual push-v1 envelope, client mutation dedup, skipped acknowledgments and buffered rollback behavior"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/createPublicationDraft.ts
    title: "Publication creation, association and contributor defaults; inspected clean pinned checkout"
---

# Native-effect foundation

## Outcome and standing

**Batch C has executable code/interfaces, not another architecture-only draft.**
Coordinated workspace version/internal pins: **0.14.0**. Both
`leafbud-editor-bind` and `leafbud-editor-recovery` remain **in progress** with
their original completion dependencies intact. Complete native drafting, a public
save command and first/second/repeat authoring acceptance are not delivered.

Code checkpoints:

- `c9d81b32`: small shared protocol crate, typed compact attempts, writer state,
  pending status/tidy integration and bounded request encoding.
- `fd68555b`: both native mint transports, guarded staging, raw observation,
  native request runner, finite recovery and loopback tests.
- `83048cce`: verified mint-to-compiler receiving seam, no-op tracking guard and
  explicitly gated deployment harness.
- `489f8e4a`: deployment-informed Flight hint tolerance/response retention,
  native preflight, protected-scope and source/selection regression coverage.
- `b2cbda02`: reverify native rights at the final metadata read before binding
  confirmation/recovery settlement, including a late-rights-change regression.

The local third-party checkout was clean at Leaflet
`600c7b5f1f7b31b66fb626c293cc104312a12caf`. Its server source is evidence of
protocol behavior, not a guarantee that every deployed operation was exercised.

## Receiving interfaces

| Home | Implemented boundary |
| --- | --- |
| `leafbud-protocol` | Native mutation names/arguments, writer/target IDs, complete null-cookie views with raw unknown fields, actual group acknowledgments, explicit entity-root reads, scope checks and exact immutable request bytes |
| `web::prepare_mint` / `mint` | Cookie/identity preflight; standalone GET effect without following redirects; publication dashboard/script discovery and one literal-argument Next action; no hardcoded action hash or standalone fallback |
| `RemoteReader::stage_binding` | Stable-ID lock, source-bound key preparation, prepared/dispatched/located checkpoints, prompt token persistence, rights/root/entity-set/entry/context/seed verification, binding without receipt or local UUID replacement |
| `RemoteReader::observe_editor` | Current typed projection plus original `PullSnapshot`, retaining current Yrs/fact bytes for compilation |
| `native::write::Compiled` / `execute_compiled` | **Crate-private** compiler receiving seam: native arguments and planned IDs only; complete compilation/encoding before effects, drift gates, bounded ordered requests, exact in-process retry and whole-current read-back |
| `RemoteReader::recover_native` | Finite observation and optional tracking-only settlement; no automatic repair, remint, stale re-encoding or restart resend |
| model/MDX | Explicit mint/push phases, writer high-water mark, planned associations and protected-scope fingerprints; existing full-binding/main-page/authored-byte guards remain |

No HTTP, Yrs, local file or semantic document model enters the protocol crate.
Web and pure editor both consume it. There is no second semantic change AST or
whole-body generation policy. The internal compiler callback is a receiving seam,
not the missing complete compiler: that compiler must establish full desired
representability and select the actual native arguments before returning.

Requests default to **250 mutations, 1 MiB/request, 128 requests/attempt**. Every
request is encoded before any is sent; a late invalid/oversized argument rejects
the complete attempt. Envelopes cannot be injected through caller-supplied bytes;
token/root/group/client/range come from typed preparation. Native references,
permission sets, fact IDs and metadata storage context are checked. Generated
text facts must pass the production reader before effects. Existing-block causal
editing itself remains with `leafbud-editor-text`.

Writer groups/clients are stable per binding, with increasing reserved IDs;
they are not Yrs client IDs. Reservations survive prepared-work abandonment.
The in-process runner can retry the identical immutable request once. There is
no disk request/body/update journal. An ack beyond the persisted reservation is
an explicit identity/sequence problem, not permission to reuse skipped IDs.

## What recovery can conclude

| Observation | Result |
| --- | --- |
| Prepared checkpoint, never dispatched | `prepared-not-sent`; deliberate settlement can clear the attempt without a new receipt; reserved IDs remain consumed |
| Mint dispatched, no locator | `mint-uncertain`; no automatic second call or homepage-title matching |
| Token returned but verification fails | Token remains in pending state; it does not revert to unbound |
| Token verified/seed present | `pending-materialization`; an empty native draft is not a completed authoring save |
| Push has no/lower ack | `uncertain`, even if desired visible content is already present; a delayed request may still land |
| Sent prefix terminal, reserved later requests unsent | `resolved-partial`; keep IDs/evidence for a deliberate next operation |
| Last dispatched request terminal, desired/associations or protected scope differs | `resolved-partial`; acknowledgment is processing evidence, not achievement |
| Entire desired current meaning/associations and protected scope verified, last request terminal | `desired-verified`; optional guarded tracking settlement promotes rows/receipt and clears pending |
| Source changed across a network await or since prepared authorship | Preserve it and return `source-changed`; successful native effects can remain locally unsettled |

Explicit `getFactsForRoots` reads are actually `{roots:[entity UUIDs]}` to the
RPC endpoint, **not a fabricated Next action or fact-ID endpoint**. The server
walks descendants to depth three. Known fact IDs are selected under their known
owning entities; pending created entities can be inspected while detached.
Empty rooted or explicit results never prove entity noncreation.

Pending state is tool-owned compact evidence: identifiers, hashes, mutation
range, phases, proposed association IDs and protected fact/entity/metadata-field
scope. It contains neither source/body copies nor encoded binary text/requests.
Raw current snapshots and exact request bytes live only in process. Undecodable
mint replies now retain a redacted-debug `ActionResponse` in memory, skipped from
`BindingReport` serialization, so an explicit private investigation need not lose
the sole returned locator evidence. This prevention cannot recreate a response
discarded by the earlier deployed decoder.

Pull/watch still defer any raw non-null pending field before strict parsing,
including unknown future shapes. Tidy refuses pending work. Typed status reports
prepared/uncertain/materialization/partial states without fetching or advancing
agreement. Null now means no typed attempt. Pending locators also enter duplicate
binding indexing before a complete binding exists, including import and explicit
external operands. All writes join the existing OS-released stable-ID locks and
adjacent byte guard; neither that guard nor the native server lock is a CAS.

## Verification

Final automated gate: **249/249 workspace tests passing**, with **two opt-in
deployment tests ignored**. `cargo fmt --all --check` and strict all-target
Clippy pass. Regular tests use actual loopback HTTP, not only encoding mirrors:

- Both mint contexts, action reference/result-slot parsing, repeated Flight
  preload rows, wrong seed/rights/context, token-before-failed-verification and
  dispatched-before-effect persistence.
- Lost mint response, source race retaining the returned locator in the report,
  and an external selected operand acquiring the UUID during the mint wait.
- Stable request identities and byte-identical replay, ordered multi-batch
  failure, no/lower restart ack, prepared-unsent settlement with reserved IDs,
  acked skip versus verified achievement and explicit detached-ID inspection.
- Whole-request late invalidity and final pre-effect drift stop with zero pushes;
  equal visible meaning cannot hide a protected native fact change.
- Pending survives status/pull/tidy, old receipt/binding continuity remains,
  generated text is locally decoded, and unknown raw witness fields survive.

### Authorized deployed check — bounded, incomplete outcomes

The operator authorized **at most two newly created unpublished drafts**, one
standalone and one in the actually configured publication, with no emails,
existing-document edits, public release or automatic deletion. This session used
that budget for **one standalone GET mint and one publication mint action call**.
An initial harness-source parse failure happened before any HTTP/mint; its
reserved marker and invalid local source are preserved as pre-send evidence.

Private locators, source files, per-role consumed-budget markers and sanitized
inspection results are under ignored `.test-agent/native-effects/deployed/`.
No credentials, capability URLs, publication URI or bodies are published here.
`~/l` was not changed. The creation harness refuses existing budget markers;
the separate inspection test creates nothing.

**Standalone:** real `/new` returned a token, verified native root/rights/seed
and persisted a complete binding while retaining the local UUID. A controlled
single-seed text initialization and `updateLeafletMetadata` wrote the clearly
labeled Leafbud verification title/text. Actual writer-group ack reached 2.
Subsequent normal and explicit known-entity/known-text-fact reads confirmed the
new text and persisted title, with native identity retained and lifecycle draft.

Whole-current verification correctly stopped **resolved-partial**, not saved:
the source omitted theme, while native creation seeds the watermark to true.
The only differing projection field was `theme`. No receipt advanced and the
planned IDs remain. Repeated create-free recovery confirmed this same terminal
partial state. The generic exact-replay/full-success restart proofs remain
loopback-only; the deployed harness did not reach those later steps.

**Publication:** configuration and authenticated identity were resolved, the
current dashboard action reference was discovered, and exactly one action POST
was made. Its response failed the original decoder's model-row test, leaving
**mint-uncertain with no known token**. It was **not retried**. No candidate was
chosen by title, creation order or dashboard recency. If a draft was created,
it may still carry the native default title because its locator was not recovered;
that limitation must not be hidden behind the intended local verification title.

Read-only dashboard Flight inspection found repeated no-ID `:HL` preload hints.
The decoder now handles that observed form and has loopback fixtures, plus
in-memory retention for future unsupported replies. The discarded action response
is unavailable, so this fix is **not a retrospective deployed publication
mint/result/read-back proof**. Both budget slots are consumed for automatic
creation. An explicit human-approved candidate recovery decision, not reminting,
is the next publication verification step.

## Compiler join and road ahead

This returns a useful **internal checkpoint with two surfaced boundaries**, not
closed bind/recovery tickets:

1. **Default omission needs an owner decision before full compilation.** Native
   mint supplies watermark true and may supply a contributor byline. Recommend
   preserve-on-omission while witnessing actual remote defaults; an alternative
   is explicit source adoption of defaults before the save attempt. The current
   strict whole-current verifier stops rather than silently choosing either.
   Title metadata is now persisted-author title, independent of headings;
   standalone/publication metadata operations use their real distinct storage
   and do not offer a contributor-clearing shortcut or invented standalone tags.
2. **Publication response loss remains uncertain.** The corrected bounded parser
   and retained-response seam are ready; no new remote creation is authorized by
   this checkpoint. Recover the existing candidate only through an explicit
   human decision or exact retained evidence, neither of which exists here yet.

Next proposed coherent implementation batch is **`leafbud-editor-text` +
`leafbud-editor-structure`, g**, after parent chooses the default-omission rule.
It owns full keyed create/update/place/detach, loaded Yrs edits, ordering and the
complete source compiler; join it to `Compiled` instead of exposing raw narrow
push as the product. The current footprint preflight locates moves in the current
snapshot: it must also resolve preceding planned attachments when newly created
objects enter that ordering pass. That is a bounded native-ID join, not a request
for a general structural optimizer.

Then **`leafbud-editor-bind` + `leafbud-editor-recovery`, Sol high**, finish the
first/second/repeat and actual complete materialization acceptance, default
bylines/themes, source races, skipped/detached partials and follower interaction.
Both tickets retain their original completion dependencies; this pass neither
removed those edges nor closed a binding-only/metadata-only demo as authoring.

Connections: [keyed edit flows](keyed-edit-flows0.gpt6a.md) owns behavior;
[build path batch C](keyed-build-path0.gpt6a.md#c-establish-reliable-native-effects--two-existing-tickets-sol-high)
owns this slice; [connected watch](connected-watch0.gpt61s.md) owns the shared
lock/selection/adoption contract; the [usage guide](../README.md#native-effect-foundation-internal-checkpoint)
records the maintained capability boundary. Whole-body regeneration, public
publication, automatic force-local repair and a saved request journal were not
introduced.
