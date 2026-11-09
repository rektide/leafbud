---
type: Design
title: "Local authoring tools: metadata, key names and inventory"
description: "Concrete commissioned next batch after 0.17: offline metadata edits, identity-preserving source-key rename and truthful local inventory, using current source patches and writer locks."
resource: /design/local-authoring-tools0.gpt6a.md
status: implemented
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
updated:
  at: 2026-11-10
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Commissioned local-authoring tools, wrote the compaction design, and accepted 0.18 after source/identity integration review"
  - resource: /design/local-authoring-tools0.gpt61s.md
    title: "Completed 0.18 implementation and 317-test/CLI/HTTP/Phoenix evidence"
  - resource: /AGENTS.md
    title: "Current fresh-Sol, rollover, bounded-complexity and source-ownership instructions"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted edit, keys rename and local list command purposes"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Author-field clear/date ownership and compact tool-state separation"
  - resource: /design/terminal-recovery0.gpt61s.md
    title: "Received 0.17 pending, lastResolution and explicit new-intent recovery boundaries"
  - resource: /design/authoring-corrections0.glm53m.md
    title: "Source-bound preparation, object/span correspondence and authored-versus-tracking projections"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Existing field/envelope patching and separate preparation, tracking settlement and remote adoption"
  - resource: /crates/workflow/src/inventory.rs
    title: "Existing current-file identity and duplicate inventory"
  - resource: /crates/workflow/src/lock.rs
    title: "Existing OS-released stable-ID coordination"
---

# Local authoring tools

**2026-11-10 parent acceptance:** delivered in 0.18.0 through `9dfd68d9`, with
317/317 tests and source/identity review complete. The
[implementation receipt](local-authoring-tools0.gpt61s.md) records actual APIs,
CLI spelling, remaining patch-shape refusals and verification. The design below
preserves the original commission; [next fronts](next-fronts0.gpt6a.md) is current.

## Commission and payoff

**Implementation owner:** fresh Sol high, `opencode://ses_e3d511cf0ffeKLT5XsWr37v0r6`.
**Tickets:** `leafbud-author-metadata`, `leafbud-author-rekey`,
`leafbud-inspect-list`; all three are now closed.

The 0.17 save/watch/recovery loop is implemented. This batch makes everyday local
work convenient: find a draft, edit selected metadata, and rename a meaningful
source key without turning it into a different native object. These operations
are offline. Their shared boundary is a current file, source-preserving patches,
identity validation and the existing cooperative lock—not another document store.

The concrete command surface follows the accepted interface:

```sh
leafbud list --json
leafbud edit drafts/essay.mdx --title "Garden notes"
leafbud edit drafts/essay.mdx --description "Updated introduction"
leafbud edit drafts/essay.mdx --published-at 2026-11-09T12:00:00Z
leafbud edit drafts/essay.mdx --now
leafbud keys rename drafts/essay.mdx b-old introduction
```

These are the commissioned destination, not a claim that the active agent has
finished them. Tag replacement/clearing needs a small explicit CLI spelling;
the implementation may choose that grammar and document it. Do not silently make
a replacement operation additive. `--published-at` and `--now` are exclusive.

## Ownership and implementation shape

| Domain | Work |
| --- | --- |
| `document/model` | Existing author/state distinctions and key grammar; narrow additional types only if actually useful |
| `document/mdx` | Pure selected-author-field patch and deliberate key/reference rename, with parser/reparse proofs |
| `workflow` | Local edit/rekey transaction over the existing lock, latest bytes, duplicate inventory and atomic byte guard; read-only list composition |
| `cli` | Thin argument/report/exit adapters |

No new crate is needed. MDX returns validated proposed bytes; filesystem effects
remain in workflow. Do not weaken `patch_tracking` so it can edit author fields,
or reuse remote adoption to install a reconstructed whole body for a label change.
The current module boundaries already support this small extension.

## 1. Selected metadata editing

The operation changes only requested title, description, ordered tags or
`publishedAt` intent. Ordinary updates preserve existing dates; `--now` changes
only the selected date field. Use Jiff to validate explicit timestamps and avoid
silently inventing a timezone. A publication date remains local release intent,
not a request to publish or an editor-save fact.

Keep the existing description rule: absence/null/empty string mean cleared;
nonempty whitespace remains meaningful. An omitted edit flag means **leave the
field alone**, unlike an omitted description in the desired document, which means
clear on save. Titles remain nonempty. Tags need explicit replace/clear behavior.

Preserve body bytes, unrelated YAML/comments, unknown author fields, UUID,
bindings, node rows, writer reservations, receipts and `lastResolution`. Reparse
the entire result and prove only selected fields changed. Do not refresh editor
receipts: title/description/tags changes should remain local changes, while a
date-only edit should not invent editor drift.

The simplest pending policy is to refuse the managed helper with a recovery hint.
If the implementation instead permits explicit author edits while retaining the
old attempt, it must demonstrate the resulting changed-source recovery semantics;
it may not rewrite original pending hashes. Unknown future owned state is
preserved through a clear refusal, not discarded to make an edit succeed.

## 2. Deliberate source-key rename

`OLD → NEW` is a reference/alias change, not a new object. It may affect a block
Key marker, a named Page, or the implicit main-page name in the tool envelope.
Update the corresponding current node-table key and every actual PageLink target.
Native entity/placement/text-fact IDs, body content/order and document UUID stay
the same. The native binding and historical `lastResolution` stay byte/meaning
equivalent; never rewrite historical attempt fingerprints to follow a new label.

Use parser-derived component/property ranges. An old word inside prose, code,
URLs or an unrelated author field is not a key reference. A bounded rewrite of an
owned component property is acceptable where a surgical token edit is awkward;
global string replacement and whole-body rendering are not the rename mechanism.

Validate the new spelling and collisions against **all** page/block names and
confirmed/retired node-table keys. Missing or ambiguous identities fail without
writes. Pending attempts should block rename: their input/target identity cannot
silently change. A retired-row-only label need not gain a resurrection mechanism.

For confirmed objects, `editor_value` is native-locator based: after rename the
editor projection and existing receipt remain identical. For new unconfirmed
objects the source key is identity; a rename can legitimately change that
projection, but must not accept unrelated dirty content by recomputing receipts.
`OLD == NEW` is a validated no-op, preserving bytes and mtime.

## 3. Truthful local inventory

Root `list` reads current managed files in the configured workspace. It reports
path, title, lifecycle, supported format/profile, local UUID and bound/unbound
status, with deliberate native editing locator reporting where useful. Human and
versioned JSON output should be deterministic. No credentials, remote requests,
key assignment or receipt updates are needed to list local work.

Reuse the inventory's header identity/duplicate accounting. A broken body can
still have useful header metadata, but that is not proof the document is valid.
Malformed/unsupported files, pending state, duplicate IDs/bindings and unbound
drafts remain explicit. Do full identity accounting before optional lifecycle
filtering; an unparseable state is not a reason to silently hide a broken file.
Lifecycle comes from content, not directory names. A published header can be
reported without claiming published-editor adoption is implemented.

## Write transaction and acceptance

Use the existing stable-UUID OS lock. Read identity, acquire, reread source and
selection, construct the pure patch, reparse/prove it, then write under the
adjacent original-byte guard. Duplicate and source-change outcomes remain
explicit. Locks coordinate Leafbud writers, not arbitrary editors; no new stale
lock-file owner/prune logic is required.

Acceptance fixtures must cover:

- Only chosen metadata fields change; clear/type/date-exclusion semantics work;
  no-op bytes/mtime, comments/unknown values and BOM/CRLF survive.
- Key rename preserves native IDs and confirmed editor receipts; multiple/forward
  links, interleaved pages and main-page names work; prose/code lookalikes do not
  change; collision/missing/pending cases produce no writes.
- Metadata changes classify honestly; confirmed rename followed by actual
  workflow/loopback push has no content mutations and retains correspondence.
- Inventory handles unbound, bound, malformed, unsupported and duplicate files,
  root/config precedence and renamed paths without network/auth dependency.
- Locks, intervening saves and unknown future owned state retain evidence and
  source rather than partially applying changes.

Run scoped checks followed by the appropriate final workspace nextest, fmt,
strict clippy and Markdown checks. No deployed test is needed for these local
features. Leave `~/l`, P1 and the standalone verification draft alone; temporary
fixtures are sufficient. Expected public-interface version is the next coordinated
minor after 0.17, with exact internal pins, subject to the actual landing.

On return, parent reviews the pure patch/identity boundaries and evidence before
accepting ticket closure. [Next fronts and compaction handoff](next-fronts0.gpt6a.md)
records the broader state and choices after this batch.
