---
type: Review
title: "Native-effect checkpoint review: defaults and uncertain creation"
description: "Parent review of the 0.14 native-effect foundation, deployed partial/uncertain results, the operator-selected explicit default-adoption rule and the bounded next commissions."
resource: /design/native-effects-review0.gpt6a.md
status: correction-in-progress
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Reviewed native-effect code and deployment outcomes; after restart recovered unanswered decisions, obtained explicit default-adoption and read-only candidate-inspection selections, and dispatched independent follow-ups"
  - resource: /design/native-effects0.gpt61s.md
    title: "Implementation checkpoint, 249-test gate and exact deployed evidence limits"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Accepted build sequence and two-draft verification authorization"
  - resource: /design/keyed-edit-flows0.gpt6a.md
    title: "Required complete authoring, confirmed-versus-pending identity and recovery behavior"
  - resource: /crates/document/model/src/attempt.rs
    title: "Reviewed compact typed mint/push phases, writer reservation and lifecycle checks"
  - resource: /crates/leaflet/protocol/src/push.rs
    title: "Reviewed native vocabulary, scope validation and immutable bounded push requests"
  - resource: /crates/leaflet/web/src/mint.rs
    title: "Reviewed actual preparation and standalone/publication request transports"
  - resource: /crates/workflow/src/native.rs
    title: "Reviewed source lock/checkpoint, key preparation, early locator persistence and verified seed/binding"
  - resource: /crates/workflow/src/native/write.rs
    title: "Reviewed internal compiler runner, finality/read-back and temporary publication normalization in verification"
  - resource: /crates/workflow/src/native/probe.rs
    title: "Reviewed guarded test-draft budget and create-free inspection harness"
---

# Native-effect checkpoint review

## Accepted ground and delivery boundary

The 0.14.0 checkpoint through `3e3c39f7` supplies real protocol, transport and
workflow code. Parent reviewed immutable bounded requests, writer reservations,
typed attempts, lock/source/selection checks, early locator persistence and finite
recovery. The reported regular verification is **249/249 workspace tests**, with
two deployment-only tests ignored, plus formatting, strict clippy and document
checks. Those reported checks are trusted rather than repeated after restart.

This remains an internal prerequisite to complete keyed authoring. Both
`leafbud-editor-bind` and `leafbud-editor-recovery` stay in progress; full compiler
and first/revised/repeat/pull acceptance remain open. The internal `Compiled`
callback must receive full supported source compilation, not become a public
metadata-only or existing-text-only save surface.

The compiler join must also extend the simple footprint preparation to recognize
membership facts introduced by earlier planned attachments before ordering new
blocks. That is an ID correspondence over actual native arguments, not a new
structural optimization engine.

## What happened on the deployment

The operator authorized at most two new unpublished verification drafts. Both
creation-call slots are consumed, and the private budget markers remain intact.

- **Standalone:** token/root/rights/seed verified; controlled text and title writes
  were acknowledged and observed through ordinary and explicit known-fact reads.
  Complete desired verification stopped as **resolved-partial** because the source
  omitted theme while native creation seeded watermark true. No successful-save
  receipt was fabricated.
- **Publication:** one action call ran, but the original decoder rejected its
  response before recovering a locator. The attempt remains **mint-uncertain**.
  Repeated preload hints are now supported, and future undecoded replies are
  retained in memory, but the original discarded response cannot be reconstructed
  by that fix. No second action call or recency/title-based binding was made.

The test-draft source, locators and private observations remain under ignored
`.test-agent/native-effects/deployed/`. This review intentionally contains no
capability URLs, publication URI, credentials or private response bodies. No
public release, email, existing-document edit or cleanup was performed.

## Operator-selected default rule

After the restart, the operator answered the previously interrupted question:
**first binding should fill omitted theme and contributor fields from the verified
native draft and record them explicitly in the local file. Explicit authored
values remain authoritative.** This is explicit source adoption of actual native
defaults, not an instruction to ignore arbitrary differences in verification.

The receiving implementation should:

1. Distinguish omission from explicit field presence. In particular, explicit
   `false`, contributor `[]`, or a supplied null must not accidentally become
   omission through deserialization. If null has no supported defined meaning,
   diagnose it before mint rather than inventing one.
2. Read and verify actual native defaults with binding/rights/context. Do not
   hardcode a contributor identity or overwrite an explicit authored choice.
3. Patch only the selected absent author fields, preserving desired prose,
   source keys, local UUID, unrelated frontmatter and comments. This is its own
   preparation transition; keep tracking-only settlement's authored-data guard.
4. Persist the coherent normalized source and pending intent under the existing
   lock/byte/selection guard, before the content-write attempt's fingerprints.
   Immediate mint and resumed located-mint paths must agree. Every persisted
   intermediate state remains recoverable without reminting.
5. Keep omission-as-clearing for fields already defined that way, such as
   description/tags. Do not generalize default adoption into a universal merge
   policy or introduce a new saved document model.

Parent review found a related first-binding equality gap: `verify_desired`
normalizes publication from the binding only in a temporary expected document,
then returns tracking state and fingerprints the original source. An inherited
publication could therefore pass that verification yet disagree with the next
bound observation's explicit publication. Prefer materializing the verified
resolved association into the source at the same narrow preparation boundary.
Explicit retarget intent must still be diagnosed. Test first binding through
settlement, status/pull correspondence and repeat no-op in both contexts.

An already-dispatched push is different: the new policy must not retroactively
rewrite its old fingerprints and claim that revised intent was originally saved.
The known standalone partial remains evidence until a legitimate terminal
resolution/replan or explicit adoption transition is available. No additional
creation is required to investigate it.

## Publication candidate recovery

The operator selected **read-only candidate inspection for human selection**.
The earlier native-effect Sol session retains deployment context and is inspecting
the exact configured publication. A shortlist may include creation time, title,
association and observable seed state, with the limitations stated. Temporal
proximity or a single plausible empty draft does not establish original-attempt
attribution. The human identifies the intended recovery target before binding.

Do not retry creation, silently select a candidate, overwrite the uncertain local
attempt, or mutate a candidate during this inspection. Exact retained original
response evidence, if unexpectedly found, would have different standing from a
candidate list and must be identified as such.

## Current commissions and next join

- **Default adoption/corrective join:** fresh Sol high,
  `opencode://ses_e3dcb1f8effebiT0KtCRZB8dWf`, owns production code/tests and a new
  implementation receipt. Existing bind/recovery tickets remain the work anchors.
- **Read-only candidate inspection:** retained native-effect Sol,
  `opencode://ses_e4121e55dffeF009EbMiG1Xinz`, owns private scratch findings only.
  It will return a bounded shortlist or explain why selection cannot be grounded.

These tasks are independent. Parent joins the corrected first-binding semantics
and the human's recovery selection before further deployed testing. The next
proposed implementation batch remains `leafbud-editor-text` +
`leafbud-editor-structure` with g, followed by complete bind/recovery acceptance
with Sol. A source/transport checkpoint is not a substitute for that full loop.
