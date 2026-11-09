---
type: Context
title: Leafbud work-session cadence
description: Project-local agent selection, session sizing and rolling ticket previews requested by the operator.
resource: /AGENTS.md
status: active
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
updated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Operator set session cadence and complexity budget, then rejected generated-body default in favor of complete drafting on the keyed native-object foundation"
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Operator permits rebuildable formats, commissions live watch and native-default adoption, and sets future agent rollover between 240–350k while allowing active agents to finish"
---

# Leafbud work-session cadence

## Product fit and complexity

- Leafbud is a low-volume tool with few expected users. Prioritize bounded
  conceptual complexity, correctness and useful authoring over minimizing every
  structural edit or remote operation. Prefer existing tools and verified Yrs
  capabilities to a parallel bespoke comparison/planning model.
- Coarser edits that rewrite more of a block/page are product-acceptable candidates
  when their identity/content effects are understood and explicit. Evaluate them
  alongside simple identity-preserving wins; keep modest room to improve without
  building a general structural optimization framework in advance.
- Preserve the keyed native-object foundation in ordinary saves. The operator
  rejected whole-body regeneration as the default: simplicity should come from
  straightforward keyed create/update/place/detach operations and bounded text
  editing, not discarding page/block continuity. Current write-side design:
  [keyed authoring](design/keyed-authoring0.gpt6a.md).
- Human comparison and enacting edits have distinct purposes. The selected human
  direction is comparable Markdown views passed to a user-provided diff tool.
  The architecture inquiry concerns how to enact/plan CRDT and fact-graph changes;
  diff algorithms are incidental means, not the central product requirement.
- Size user-facing deliveries around useful workflows. For drafting/push, adding
  paragraphs and headings, first binding, and ordinary structural revisions are
  basic requirements. A text-only existing-block proof can be an internal commit,
  but must not be presented as a sufficient authoring milestone. Reduce mechanism
  complexity rather than excluding the normal user actions that make it useful.
- Local content currently on this computer, including `~/l`, is rebuildable.
  Formats and earlier design choices may be revised when useful; avoid building
  migration machinery solely to preserve this disposable corpus.
- Include **persistent live watch/sync from Leaflet** in the build direction.
  Repeated one-shot pulls alone do not fulfill that request. Reuse guarded
  adoption inside a connected subscription-driven session; polling/full reads
  can backstop missed events and reconnects. Detecting other attached clients
  is exploratory, not a settled requirement or proof that an editor is idle.
  Code-grounded proposal: [build path and live sync](design/keyed-build-path0.gpt6a.md).
- At first binding, adopt verified native theme/contributor defaults for omitted
  fields and record them explicitly in local source. Preserve explicit authored
  choices; do not hide missing-field differences by weakening verification.
  [Default-adoption decision and recovery boundary](design/native-effects-review0.gpt6a.md).

## Commission and agent selection

- Default implementation agent: **g**. Recommend **Sol high** when a slice has
  higher architectural concern, unresolved semantics or substantially unpaved
  implementation work; explain briefly why the extra judgment would help.
  The operator may explicitly select Sol high. Follow explicit selections and
  inherited fallback rules.
- Plan coherent batches of **1–4 upcoming tickets**. A preview is a recommendation;
  keep commissioned implementation distinct from work merely queued for discussion.
- Parent review owns architecture, evidence and integration; implementation agents
  carry through their coherent slice, checks, commits and ticket updates.

## Session size and rotation

- For future working-agent sessions, **roll over at a coherent checkpoint between
  240–350k reported context tokens**. Prepare the handoff as the session approaches
  that window and complete rollover by 350k; nominal model capacity is not the
  operating budget. Smaller completed slices may end normally—never manufacture
  work to reach the lower bound. Use reported context, not cumulative lifetime
  token usage, when available.
- Continue a session when retained context is valuable within that rollover
  budget. Rotate earlier when scope or a natural completion warrants it.
  Before rotating, commit owned work and leave a concise handoff with code/design
  pointers, verification, unresolved decisions and the next useful slice.

## Completion reports and rolling preview

- Whenever a work session returns, give a quick result/verification summary and
  a **road ahead** preview of the next **1–4 tickets**.
- Name ticket IDs, intended payoff, proposed agent and useful batching/dependencies.
  Refresh the preview from actual outcomes and current Beads state; do not repeat
  the whole roadmap or automatically carry stale plans forward.
- Make the preview narrative: explain what the next batch makes possible, why its
  tickets belong together, the main judgment involved and the boundary of the
  slice. A bare ticket/agent table is not enough. Keep it a quick readable account.
- Keep tracker pointers and material completion notes current, including actual
  session provenance. Preserve earlier work history when updating tickets.

Current capabilities and documentation map: [README](README.md).
Accepted product contract: [authoring interface](design/authoring-interface0.gpt6a.md).
Beads (`leafbud-*`) owns the roadmap, priorities and dependencies.
