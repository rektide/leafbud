---
type: Context
title: Leafspring work-session cadence
description: Project-local agent selection, session sizing and rolling ticket previews requested by the operator.
resource: /AGENTS.md
status: active
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Operator requested rolling 1–4-ticket previews, g by default, Sol high recommendations for higher-concern work and 160–350k-token agent sessions"
---

# Leafspring work-session cadence

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

- Operator target: **160–350k tokens per working agent session**. Budget toward a
  coherent result within that range; do not manufacture work to meet the lower
  bound or treat the upper bound as a goal. Use reported context when available.
- Continue a session when retained context is valuable and capacity permits.
  Rotate to fresh instances at coherent checkpoints as scope/context warrants.
  Before rotating, commit owned work and leave a concise handoff with code/design
  pointers, verification, unresolved decisions and the next useful slice.

## Completion reports and rolling preview

- Whenever a work session returns, give a quick result/verification summary and
  a **road ahead** preview of the next **1–4 tickets**.
- Name ticket IDs, intended payoff, proposed agent and useful batching/dependencies.
  Refresh the preview from actual outcomes and current Beads state; do not repeat
  the whole roadmap or automatically carry stale plans forward.
- Keep tracker pointers and material completion notes current, including actual
  session provenance. Preserve earlier work history when updating tickets.

Current capabilities and documentation map: [README](README.md).
Accepted product contract: [authoring interface](design/authoring-interface0.gpt6a.md).
Beads (`leafspring-*`) owns the roadmap, priorities and dependencies.
