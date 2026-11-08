---
type: Report
title: "Guarded pull delivered: bound drafts refresh with stable identities and explicit decisions"
description: "Implemented the commissioned guarded refresh/pull for imported draft MDX: source-preserving MDX updates, receipt-based classification, identity reconciliation, thin CLI with dry-run/take-remote, offline verification and a live 27-file no-op."
resource: /design/guarded-pull0.glm53m.md
status: implemented
generated:
  at: 2026-11-08
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e45caa8e0ffeARkDD15IcPc24x"
    title: "Fresh g session implementing guarded pull end to end: mdx update writer, workflow pull, CLI wiring, tests, live dry-run and guarded no-op on ~/l"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Parent commission, boundary contract and review; owns the accepted interface and sequencing"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted conflict/effect contract, exit codes, YAML preservation and pull semantics this implements"
  - resource: /design/draft-checkout0.gpt61s.md
    title: "Prior import/config implementation whose readers, hashes and live workspace this builds on"
  - resource: /design/markdown-roundtrip0.gpt6a.md
    title: "Approved follow-on Markdown-first body rendering that integrates against these seams next"
---

# Guarded pull for bound drafts

## Outcome and checkpoints

The user commissioned guarded refresh/pull for the existing imported draft
MDX files. Code remains in `~/src/springleaf`; **`~/l` is the document
workspace** and only contains the 27 real drafts plus this capability's
receipts.

- `tvplrskqw` — **Add source-preserving MDX update for guarded refresh**:
  the single writer split into verified frontmatter/body parts, plus
  `springleaf_mdx::update` patching owned author fields, regenerating the
  reserved `@springleaf` envelope, keeping unrelated author bytes and
  rewriting the body only when page content changed. `Uuid` re-exported from
  the document model.
- `xyqypkkt` — **Add guarded pull workflow**: `RemoteReader::pull` with
  workspace scan, identity/rights/lifecycle/association guards, key
  reconciliation, remote-owned merge preserving local-only metadata,
  receipt-based classification and guarded atomic writes.
- `vkyusotn` — **Wire the CLI and bump to 0.5.0**: `springleaf pull FILE...
  [--all] [--state draft|published] [--take remote] [--dry-run] [--json]`,
  accepted exit codes, coordinated version bump with exact internal pins.

Authorized live result: a full **dry-run** over the real workspace reported
**27 selected, 27 unchanged, 0 needing decision, 0 failed** (exit 0), and the
subsequent normal guarded pull reported the same classification with
**bytes and nanosecond mtimes identical for all 27 files** — an exact no-op.
No live conflict existed to preserve; no remote mutation was invoked; private
text, tokens, titles and filenames are omitted here on purpose.

## Capability contract as implemented

`pull --all` selects **existing bound local files** for this origin (optionally
by local `--state`; managed-but-unreadable files stay loud rather than being
hidden by a filter). `import --all` remains remote discovery/adoption. The
two vocabularies do not mix: pull never creates a file, import delegates
existing bindings to preservation behavior.

Per file, pull reads the original bytes **before** any await, parses with the
current profile, fetches the same bound editor's metadata and content, and
validates: draft lifecycle, document-id correspondence to the binding
derivation, root entity, single readable entity set (rights), entry placement
fact, and publication association. Any mismatch is an explicit
`needs-decision` limitation (file preserved), never a silent rebinding.
Missing remotes (`PGRST116`) are a distinct failure from unmanaged operands
and from files bound to another origin.

Classification uses **current endpoints and receipts only** — there is no
stored base document, CRDT history or server CAS:

| situation | outcome |
| --- | --- |
| both sides match receipts | `unchanged` — no write at all |
| endpoints semantically equal but receipts moved (local edit equals remote edit, or local-only `extra` metadata) | `converged` — receipts settle, body untouched |
| remote moved, local clean, projection unchanged (e.g. latent inactive text) | `receipts-only` — witness settled, only the `remoteHash` line changes |
| remote moved, local clean, projection changed | `updated` — adopted through the source-preserving writer |
| local moved, remote clean | `needs-decision` — preserved |
| both moved, different | `needs-decision` conflict; `--take remote` adopts remote editor-owned content while local-only metadata, comments and renames survive |

`--take remote` is the only path that replaces differing local editor-owned
content. It never touches bindings, never drops unknown fields, and unsupported
remote content still blocks the item.

Identity policy: the merged document keeps the existing document id and
binding; fresh deterministic keys are first-association choices. Key
reconciliation keeps the existing local key whenever the fresh binding
identifies the same remote object (same entity, exact placement match, or a
unique entity when placements grew), so manual renames survive refresh and
unrenamed files are never rekeyed. Ambiguous matches fall back to the derived
key rather than guessing.

Writing is `springleaf_mdx::update`: changed owned fields (`title`,
`description`, `tags`, `publication`, `theme`, `contributors`) are rewritten in
normalized form at their existing location; unchanged fields, comments, blank
lines and unknown/local-only fields keep their exact bytes; the reserved
envelope is regenerated; the body is re-serialized (with the existing
dual-mode semantic verification) only when page content actually changed. The
composed result must parse back to exactly the reviewed document or nothing is
written. Before writing, remote identity/lifecycle/association is re-fetched
and re-checked; the atomic writer re-checks the retained original bytes, so a
concurrent local edit aborts with `changed during pull` and wins. Dry-run
performs the reads and classification but never writes receipts or files.

## Hash-scope decisions and one known conservative seam

The receipts keep the existing full scope (`semantic_value` includes `extra`);
no hashes were rewritten or blindly re-acknowledged. The `extra` merge takes
every key from the fresh remote observation plus every existing key the
profile does not treat as remote-owned (`contributors` today), so:

- A local-only metadata change with an **unchanged** remote cannot become an
  editor conflict: the endpoints compare equal and receipts settle
  (`converged`), honoring "don't let local-only metadata changes become editor
  conflict by accident" without changing any stored hash.
- When **both** sides have moved, a local-only metadata change is
  indistinguishable from a local editor change (there is no stored base to
  reconstruct the receipt-time content), so the item is conservatively
  reported as a conflict. `--take remote` resolves it while preserving that
  metadata. This is a deliberate, reported limitation, not a silent overwrite.

Envelope round-tripping goes through the typed model, but the patcher replaces
only the envelope's own extent, so future unknown envelope fields survive in
the file only insofar as they are representable; unknown fields inside
`State`/`Sync` are `deny_unknown_fields` errors today and remain explicit
blockers rather than dropped data.

## Verification

- `cargo fmt --all` / `--check`; `cargo clippy --workspace --all-targets --
  -D warnings` — clean; `cargo nextest run --workspace` — **84/84 passed**
  (67 pre-existing + 17 new; all offline, synthetic fixtures and loopback
  responses only).
- New mdx tests: receipt-only patch preserves author comments/body bytes;
  changed-field patch and absent-field insertion/removal; foreign identity and
  unrepresentable results refused.
- New workflow tests (real fixture → Yrs → MDX): exact no-op byte+mtime
  equality with request-count assertions; remote text+strong-mark+block-move+
  title adoption with the same key set, new mark runs, adopted order and
  receipts matching a fresh build; witness-only inactive-text change settling
  exactly one `remoteHash` line; local edits preserved with needs-decision
  (also dry-run); conflict → `--take remote` preserving a manual rename, an
  author comment and `customField`; convergence settling receipts without body
  rewrite followed by a no-op repeat; missing/published/unsupported remotes;
  local-edit-during-network refusal; `--all` selection with unmanaged files
  ignored, broken managed files loud, and `--state` filtering; duplicate
  bindings, foreign origins, unmanaged operands; dry-run preview performing
  the adoption only when run for real.
- New CLI subprocess tests: human and JSON output, conflict exit 3, dry-run
  JSON with `dry_run`/`needs_decision`, `--take remote` exit 0, usage errors
  exit 2, lone failure exit 1 versus mixed batch exit 4.
- Live: dry-run then guarded pull over `~/l` as recorded above. `--take
  remote` was never run against real files; destructive scenarios used
  synthetic workspaces only.
- Owned Markdown checked with `rkt check` / `rkt check --okf`.

## Remaining boundaries and the next small seam

No push/publish/editor writes; no published-lifecycle refresh (explicit
per-file decision); entry-placement/root/entity-set/association changes are
decisions, not adoptions; body rendering keeps the conservative whole-block
Run/typed fallback. The approved
[Markdown-first body rendering](markdown-roundtrip0.gpt6a.md) lands next
against these exact seams: `render_body_mode`/`verified_body` for its
block-local candidate checks, `update`'s body-rewrite-only-when-changed
scope, and pull's classification for its no-op guarantees. After that, the
smallest useful slice is `status`/`diff --edge editor` reusing pull's
read-only classification, so divergence can be inspected before any decision.
