---
type: Design
title: Canonical on-disk format, revision 2 — the leafspring.toml shape
description: Answers records-canonical's open leafspring.toml item; flat keys resolved as figment2 layers (flag > workspace toml > global toml) with a loud malformed-config posture, replacing the hand parser.
resource: design/records-canonical2.glm53m.md
status: draft
generated:
  at: 2026-11-07
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e48d80516ffeSrmXtCzOWCWBwE"
    title: "Session implementing the figment2 layered config this revision records"
  - resource: "opencode://ses_e4ceef2e9ffeMaBQB1Yc3bcyMD"
    title: "Session holding the protocol map; operator's canonical-format direction recorded in the original doc"
  - resource: "file:///home/rektide/src/leafspring/experiments/03-post-builder/REPORT.md"
    title: "Proof that a lexicon-valid record + putRecord posting path works"
---

# Canonical on-disk format, revision 2 — the leafspring.toml shape

Revision of [`design/records-canonical.glm53m.md`](records-canonical.glm53m.md).
Only the config items change; the layout, sidecar, and sync semantics
are unchanged from the original.

## The `leafspring.toml` open item, answered

The original's open items asked after "the `leafspring.toml` shape and
whether publication default belongs there (yes, per experiment 03's
pick-first-publication lesson)". Answer, implemented in
`crates/cli/src/config.rs` with figment2 (the operator's preferred
config library) replacing the hand parser:

- **Shape: flat keys, one file name, two locations.** The known key
  set is `workspace`, `handle`, `pds`, `publication`; unknown keys are
  ignored. A global file at `$XDG_CONFIG_HOME/leafspring/
  leafspring.toml` (else `~/.config/leafspring/`) and a workspace file
  at `<root>/leafspring.toml`.
- **Layering, per key** — stacked figment providers, flags fed in
  programmatically as the top layer (not env-var magic):
  - `publication`: `--publication` > workspace toml > global toml.
    The operator's explicit rule: a workspace publication overrides
    the top-level one. (Before this revision the workspace toml was
    the only toml source and the global `publication` key had no
    consumer.)
  - `handle`: `--handle` > workspace toml > global toml >
    `$ATPROTO_HANDLE`, preserving sync's original chain.
  - `workspace`: `--root` (a path, not a value) > the global key >
    cwd — necessarily global-only: a workspace toml cannot be found
    before the workspace is known. Relative values resolve against
    the config file's directory (the hand parser's rule, kept).
- **Loud malformed posture** — a deliberate change from the hand
  parser, which never failed (absent/malformed → silently ignored):
  an absent file or key is simply unset, but a present file that
  fails to parse or types a known key wrongly aborts with the file
  path and the parser's error. Silent fallback on a typo'd config is
  a footgun in a publishing tool. Explicitly empty values count as
  unset (the hand parser's `!value.is_empty()` rule, kept).
- **`pubs list` consistency**: it now resolves its default handle
  through the same workspace + global layers (workspace found via the
  global `workspace` key) minus the env fallback — previously it read
  the global toml only, ignoring the workspace. One identity default
  everywhere; `pubs list` stays a casual public read, so it does not
  inherit sync's `$ATPROTO_HANDLE`.

Tests stay offline and machine-independent: pure layering functions,
`TempDir` workspaces, explicit paths, and `$XDG_CONFIG_HOME` injection
under an env guard — never the real config dir.

The operator-facing precedence table lives in the README's
Configuration section, which also covers the `config show` /
`config set` command (toml_edit write path, comment-preserving;
figment stays read-only). Remaining open items from the original
(multi-publication drafts, CLI flag ordering) are unchanged.
