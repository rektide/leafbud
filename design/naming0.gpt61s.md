---
type: Design
title: "Independent local naming reconciliation"
description: "Bounded 0.20 mapping, witness ownership and recoverable no-clobber move ordering for leafbud-author-names."
resource: /design/naming0.gpt61s.md
status: implementation-design
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3c29df03ffeKTX7z5IHA3nup5"
    title: "Commissioned independent naming implementation; selected reversible workspace mapping and rename-before-witness ordering"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted all-applicable tidy, explicit directions and independent name acceptance"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Optional workspace-relative name witnesses and author path intent"
  - resource: /design/document-sync1.gpt61s.md
    title: "Independent current-endpoint naming evidence and native path limits"
  - resource: /doc/mdx/README.md
    title: "Received source patch, comments and pending-state boundaries"
  - resource: /crates/workflow/src/tidy.rs
    title: "Received local pass composition and duplicate-check receiving boundary"
---

# Independent local naming reconciliation

## Mapping and evidence

Logical `/notes/Essay` maps exactly to workspace-relative `notes/Essay.mdx`.
The slash is a logical root, never an operating-system absolute destination.
Nested directories participate (they are not flattened or silently preserved).
Case and literal Unicode are retained without normalization. Exactly one final
lowercase `.mdx` extension is removed/added; `x.mdx.mdx` maps to `/x.mdx`.
Empty segments, `.`/`..`, backslashes, percent encodings, control characters and
URL/drive punctuation `:?#` are unsupported. No decoding, slugification or route
allocation occurs. Destination directories must already exist, be real directories
under the canonical workspace, and contain no symlink components. No directories
are created by preview or naming moves.

The compact optional `names` pair records the workspace-relative file including
extension and the actual author `path` presence (missing/null/string). No time is
needed: a changed pair is the settlement. There is no second body or name history.
External operands remain supported by key/Markdown passes; naming requires moving
the operand into the workspace explicitly first. Unsupported logical intent can
still be explicitly accepted as a mismatch: acceptance does not enact that intent.

Read-only classification and default tidy share one policy. Missing/null current
path is unassigned. Equal endpoints without a witness are equal; differing ones
are untracked and need choice. Against a witness, changed file only selects filename,
changed path only selects path; both changed and unequal needs choice. Equal changed
endpoints are converged. An unchanged mismatched pair is accepted-mismatch. Default
tidy settles equal/converged and reconciles only unambiguous one-sided changes.
Explicit direction selects the current endpoint, not a historical value. An
unassigned draft stays unassigned unless filename direction is explicitly chosen.

## Source and filesystem boundaries

MDX gets a separate naming patch authority for author `path` and `state.names`.
Tracking settlement does not gain arbitrary author-field authority. Complete
reparse/header/model/body checks prove the selected change; pending and future
owned state refuse. All editor receipts, native associations and bounded resolution
provenance remain unchanged. Naming is offline and does not implement native path
mutation, public identity selection, publishing or release equality.

Workflow retains the stable-UUID OS lock. All explicit operands participate in
initial and immediately-pre-effect duplicate checks, even outside the workspace.
Source bytes and file identity, canonical operand route, destination parent and
destination absence are checked again before effects. Each batch item is independent;
two items targeting the same file cannot overwrite it (the later item refuses).
Copies sharing identity are ambiguous even if the destination copy has identical
bytes. Symlink operands resolve to their real file; naming follows that canonical
coordinate, while route changes during planning refuse.

For a move: validate/compose everything first; use an actual no-replace rename;
then write the prepared source/witness under the original-byte guard at the new
path. Failure after rename leaves the only good original file at the destination
with old naming evidence. Report that destination and explicit recovery rather
than deleting it or rolling over a concurrent editor. Retry tidy there: the stale
pair truthfully reports the move; equal endpoints can settle. Native identity and
pending evidence are never forgotten. No journal or generic transaction manager.

This is not an atomic multi-file CAS. Cooperative operations serialize by UUID;
noncooperating filesystem writers can still race the final check/syscall window.
Cross-device moves and unsupported no-replace platforms refuse. No fallback copy,
overwrite rename, auto cleanup or persistent staged duplicate is introduced.

## Delivery boundary

Implementation, fixtures, maintained source/coverage/usage and completion evidence
follow in this session. Parent owns acceptance and the next release-front design;
this checkpoint is not evidence that the design is already implemented.
