---
type: SourceContract
title: "Managed MDX source fidelity"
description: "Current source ownership, bounded YAML and body-comment patching, identity continuity and refusal boundaries."
resource: /doc/mdx/README.md
status: active
updated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3c420e5affeLoQfS0oTd9wu5b"
    title: "Implemented parser-selected preparation, whole-flow author headers and block-anchored source comments; defined preservation/refusal contracts and executable evidence"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted operation-specific ownership and native keyed continuity"
  - resource: /crates/document/mdx/src/source.rs
    title: "Serde key spans and granit-paired flow collection extents"
  - resource: /crates/document/mdx/src/parse.rs
    title: "Real Satteri traversal, decoded JSX selection and source-only block comment anchors"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Separate tracking/default preparation/adoption guards and comment-anchor refusal"
  - resource: /crates/document/mdx/src/render.rs
    title: "Source-bound, loss-checked Markdown tidy"
---

# Managed MDX source fidelity

Current contract: **Leafbud 0.19**, `leafbud-mdx/2`, draft lifecycle,
`leaflet-editor/3` receipts. [Usage](/README.md) and
[executable content/operation coverage](coverage.md) are companion homes.
This is a bounded source adapter, not an arbitrary YAML setter or lossless MDX CST.

## Operation ownership

| Operation | May change | Must preserve |
| --- | --- | --- |
| `tidy --keys`, executed push preparation | Missing block/main keys; actual PageLinks to the provisional main key | Existing/retired identities, unrelated bytes, author metadata and receipts |
| `tidy --markdown` | Selected rich block spans, after checked Markdown rendering | Keys, Page boundaries, comments, header and semantic meaning |
| `edit` | Selected title/description/tags/explicit publication date | Exact body; unrelated author fields, state, historical attempt hashes and receipts |
| `keys rename` | One current label, its actual references and current node-map key | Native IDs, binding, writer sequence, receipts and historical provenance |
| Tracking settlement | Reserved `@leafbud` state after verification | Author fields and exact body; established binding and monotonic writer reservations |
| First-binding default preparation | Explicit verified publication; omitted native theme/contributors | Prose and explicit author choices; original dispatched pushes are never reinterpreted |
| Pull/watch/remote recovery adoption | Changed editor-owned metadata/body and verified state | Local-only author fields, matched labels, UUID/binding; comments under the rule below |

Pending attempts block managed edit/rename/tidy and ordinary inbound adoption;
explicit recovery owns their resolution. Unknown future owned state blocks rather
than being dropped. A helper never refreshes receipts to conceal dirty content.
Original-byte filesystem guards remain in workflow, not the MDX adapter.

## YAML shapes

Block top-level mappings and **whole-flow mappings** are supported through edit,
key preparation, first-binding defaults, tracking and adoption. Quoted keys,
nested flow collections, multiline flow collections and comments are selected
through the existing serde-saphyr/granit parser. Whole-flow changes patch only
selected values/entries; unrelated entries retain their exact source spelling.
The reserved envelope is tool-owned and may be regenerated during settlement.

Serde collection spans can be only the opening delimiter. Flow extents therefore
pair parser start/end events; they are not guessed from scalar spans or braces
inside strings. Selected-value comments survive replacement (they may precede
the normalized new value as standalone comments). In block mappings, selected
metadata entries use parser key boundaries, retaining comments and separators;
multiline scalars keep unrelated indicator/content bytes intact.

Anchors, aliases, merge keys, unsupported tags and duplicate keys remain rejected
by the managed YAML profile. Unknown **author** values are retained; unknown
**owned state** is not silently accepted. A block-scalar `mainPage` rename still
requires explicit single-line reshaping. Exotic unsupported layouts fail the
full-output proof before writes; accepting a YAML parse is not a promise that
every source surgery is possible.

Description clear means exactly absent/null/empty string, never trimmed nonempty
whitespace. Explicit tag replacement preserves order/duplicates; empty tags write
`[]`. Date-only changes remain local intent. An unchanged operation returns exact
bytes and the workflow does not replace the file.

## Body comments: source-only, block-anchored

Supported syntax is a standalone MDX flow expression containing **one** JavaScript
block comment, with optional surrounding whitespace:

```mdx
{/* local drafting note */}

<Key id="intro" />
An introduction.
```

It may also appear between the Key and its block. Multiple standalone comments
retain their order. Multiline/Unicode comments work. A comment attaches to the
**next block in that same container/page**, including the first item of an
unkeyed list; key preparation makes that identity durable. A comment before a
named Page definition is ambiguous and refused: put it before a block inside
that Page instead. Trailing/orphan comments, inline comments, HTML comments and
non-comment expressions are not supported. No expression is evaluated.

Comments are **not native Leaflet comments**, collaborator discussion or content.
They do not enter semantic/editor fingerprints, native effects or human editor
comparison views. Source-byte guards still detect an intervening comment edit.

- Preparation, rename, selected metadata edits and tracking keep comment bytes.
- Markdown tidy operates outside comment spans. Its plan binds the exact original
  source, validates disjoint UTF-8 ranges and proves unchanged comment anchors/raw
  text as well as semantic meaning.
- Equal-body adoption/no-op saves keep the exact original body, not reconstructed
  comments. No-op bytes/mtime and writer sequence remain protected.
- When remote content changes, comments precede their retained block after
  rendering. Their raw comment bytes remain exact; surrounding blank layout may
  normalize. Reordering **within the same page** follows the block.
- If an anchor is removed or moves to another page, adoption refuses before the
  local write—even with `pull --take remote`. Source/receipts remain untouched.
  Explicitly move the comment to its intended surviving block, or remove it, then
  repeat the deliberate adoption. There is no automatic relocation/discard flag.

This policy uses transient parser spans and source keys only. No second body,
saved history, comment sidecar, general document diff or persistent trivia model
was added. [Comment fixtures](/crates/document/mdx/src/comment_tests.rs),
[actual CLI/HTTP loop](/crates/cli/tests/push.rs) and
[HTTP/Phoenix adoption](/crates/workflow/src/watch/tests.rs) carry executable proof.

## Safety and limits

Pure patches always reparse/prove the full output. Workflow retains stable-UUID
OS locks, canonical operands, duplicate checks and original-byte atomic-write
guards; atomic rename is not filesystem CAS against noncooperating editors.
Preview changes no source/native state. Unknown source/native semantics are
explicit refusals, never a truncated save.

Parser validity and native writability differ: provisional-main PageLinks can
prepare correctly while cyclic page graphs still fail native save preflight.
The [coverage matrix](coverage.md) separates those layers, resource restrictions,
test evidence and unimplemented richer/native-public features. The
[implementation receipt](/design/source-fidelity0.gpt61s.md) records this delivery
and its architectural review boundary.
