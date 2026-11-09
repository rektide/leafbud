---
type: Review
title: "Keyed-save integration review and terminal-recovery join"
description: "Parent review of the complete 0.16 save loop, accepted native/text/structure boundaries, a semantic no-op follow-up and the next fresh-Sol recovery/conflict batch."
resource: /design/keyed-save-review0.gpt6a.md
status: accepted-with-follow-up
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Reviewed complete compiler, native execution and deployed receipt; commissioned fresh Sol for bounded no-op correction and deliberate terminal recovery/conflicts"
  - resource: /design/keyed-save0.gpt61s.md
    title: "0.16 implementation, 282-test gate, independent Yjs and actual existing-P1 CLI proof"
  - resource: /design/keyed-edit-flows0.gpt6a.md
    title: "Primary complete drafting and native-identity cases"
  - resource: /crates/leaflet/editor/src/write.rs
    title: "Reviewed pure source preflight and complete native compiler composition"
  - resource: /crates/leaflet/editor/src/write/blocks.rs
    title: "Reviewed retained/shared/recreated occurrences, owned scalars and text handling"
  - resource: /crates/leaflet/editor/src/write/pages.rs
    title: "Reviewed page reachability, native-page reuse and incoming reference sets"
  - resource: /crates/leaflet/editor/src/write/order.rs
    title: "Reviewed membership detachment and changed-page end normalization"
  - resource: /crates/leaflet/editor/src/write/metadata.rs
    title: "Reviewed real persisted title, context-specific clears and explicit theme/byline policy"
  - resource: /crates/leaflet/editor/src/text/write.rs
    title: "Reviewed loaded-state Yrs splice/format/coarse-interior paths and cumulative payload"
  - resource: /crates/workflow/src/native/save.rs
    title: "Reviewed preview/execute, first-binding composition, pending handling and takeover gate"
  - resource: /crates/workflow/src/native/write.rs
    title: "Reviewed separate explicit verification view, planned membership table, terminal read-back and protected scope"
---

# Keyed-save integration review

## Accepted delivery

The complete supported save loop is implemented in 0.16.0 through `32f01aee`.
Parent reviewed the pure compiler's text/block/page/order/metadata boundaries,
public save orchestration and detached-state verification. The implementation
retains one source model and the native mutation vocabulary rather than adding
a parallel semantic change system. The loaded Yrs writer uses the agreed small
splice/format path and bounded paragraph-interior fallback.

Reported evidence is **282/282 regular workspace tests**, strict clippy and
formatting, **7/7 independent production-Yrs-to-actual-Yjs groups**, complete CLI/
HTTP first-mint fixtures in both contexts, and actual existing-P1 CLI baseline →
combined revision → read-back → unchanged pull/repeat. The HTTP/Phoenix follower
join also verifies pending preservation followed by later remote-save adoption.
These checks are trusted; parent source review answers architectural questions
rather than automatically rerunning the suite.

The planned-attachment and detached-visibility joins are present: membership
validation incorporates preceding native arguments, and explicit known-root rows
live in a separate verification view. Detached facts are neither mistaken for
deleted data nor fed as unrepresented leftovers to the active document observer.
Page reference locators are canonical sets. Ordinary saves retain existing keyed
objects; omission detaches placements, and selected recreation is deliberate.

Text/structure ticket closure is supported by this delivery. Bind/recovery remain
in progress and conflicts remains open for the next integration slice. Actual
first-unbound deployment composition in both contexts and the original lost-mint
attribution are not inferred from the separate proofs. No new draft was created;
the selected P1 was the sole deployed revision target.

## Bounded no-op follow-up

Code review identified a concrete edge for executable reproduction: retained
native objects can represent supported defaults without explicit facts or with
equivalent JSON spelling. Examples include blank rich text without a text fact,
default bullet style absent, `block/is-list=false`, empty code defaults, and
optional image fields omitted rather than null. The current builder compares
raw field presence/data and may initialize or normalize these while authored
meaning is unchanged.

The next Sol should test these cases and, where confirmed, fix the retained-block
semantic no-op boundary. This is not a minimum-edit engine: skip unsupported
normalization of already-equal supported meaning, retaining actual locators and
raw native data. Genuinely new objects still initialize. Metadata must separately
honor real persisted title—an `Untitled` display fallback must not suppress an
intentional first title write. The correction should not weaken read-back or
hide unknown semantics.

## Next major feature commissioned

Fresh **Sol high**, `opencode://ses_e3d6d728effe6RFnyZcrUtifT9`, owns
`leafbud-editor-recovery` + `leafbud-editor-conflicts`, then the
`leafbud-editor-bind` acceptance join. It also owns the narrow no-op follow-up
above. The predecessor finished about 276k; new work starts fresh, with a
committed handoff/rollover between 240–350k as requested.

The missing recovery boundary is **deliberate resolution of an acknowledged
terminal attempt**, distinct from observing that its original intent succeeded:

- Accept-current adopts a fresh complete supported native state locally while
  preserving local-only fields and source/binding guards.
- Replan-local resolves the old terminal result as partial/abandoned, then compiles
  current local intent against fresh native state using real existing/planned
  associations and new writer reservations. It cannot independently re-encode an
  old uncertain CRDT operation.
- Missing/lower acknowledgment is still uncertainty; take flags do not make a
  delayed request impossible. Incomplete unsupported graphs can remain precisely
  blocked rather than acquiring an automatic general repair engine.
- Preserve a bounded compact prior-resolution account: original attempt identity,
  fingerprints, writer/range/finality and the new decision's relation. No saved
  body, executable request journal or unbounded history is required.

The agent should choose a small explicit command/policy surface, document the
state transition, and prove crash/source/selection/follower behavior. `--yes`
executes; it does not select a conflict direction. Settling a prior operation
must not falsely report that a separately requested new recreation was performed.

This batch explicitly includes a **create-free deliberate terminal-resolution
proof on the known standalone verification draft**, after fixtures and the
resolution API are sound. It remains within the two existing disposable test
objects' authorized scope. The old partial must be resolved as a new acceptance
decision with preserved provenance, not have its old hashes rewritten as an
original success. No additional mint, publication, email or cleanup is authorized.

The user's continuing-work instruction permits the parent to join these results
and commission the next coherent fresh-Sol major batch. It does not remove the
specific external-effect and identity boundaries recorded above.
