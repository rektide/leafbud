# Rushdown block author-key experiment

An offline, isolated Rust executable using **Rushdown's extension API**, not a
Markdown preprocessing pass. Production crates/manifests are unchanged. It makes
no network requests and performs no Leaflet writes.

From this directory:

```sh
cargo run --locked --offline -- attributes fixtures/attributes.md
cargo run --locked --offline -- plain fixtures/attributes.md
cargo run --locked --offline -- comments fixtures/comments.md
cargo run --locked --offline -- both fixtures/combined.md
cargo nextest run --locked --offline
```

Fetch dependencies once with `cargo fetch --locked` if the local Cargo cache is
empty. `attributes` enables optional author keys; `plain` demonstrates baseline
CommonMark interpretation. `comments` enables the narrow comment directive;
`both` enables both spellings with collisions rejected (neither wins).
The executable prints a transient JSON projection
(block kind, optional key, annotation byte span, available parser positions,
inline tree, children) and HTML for inspection. Diagnostics produce exit code 2.
There is no saved partner document or source rewriter.

## Phase A: attribute syntax

```md
## Introduction {#introduction}

This is an ordinary paragraph with **formatting**.
{#central-argument}
```

Keys are **internal author identities**, not HTML anchors or Yjs character IDs.
The custom extension records metadata on the actual block node and typed
projection, strips marker nodes before rendering, and leaves source bytes alone.
Headings (ATX/setext) accept a whitespace-separated final suffix; paragraphs
accept an entire final logical line, with no intervening blank line. A blank
line makes the attribute orphaned, not an instruction to search backwards.

A key on prose inside a list belongs to its paragraph, **not** its list item or
list. Quote/list marker lines require explicit container prefixes/indentation;
lazy unindented annotation continuation is rejected. Standalone malformed,
unbound and duplicate keys diagnose. Grammar: `[A-Za-z][A-Za-z0-9_.-]*`, unique
document-wide. Escape the opening brace to write a reserved attribute literally.
Code and ordinary non-reserved braces remain ordinary Markdown.

The [assessment and check receipts](REPORT.md) recommend attributes for selected
prose, and document the strict attachment rules, source-position limits and
unimplemented MDX/component grammar. There are 39 offline tests. The projection's
`content` is an inspection flattening: raw Text escapes/entities are not yet
semantic Leaflet text. `content_segments` are payload positions, not complete
rewriter ranges; no source reconstruction or source writer is supplied.

The published dependency is pinned exactly to `rushdown = "=0.18.0"`, matching
[`yuin/rushdown` revision e5eb4e4](https://github.com/yuin/rushdown/tree/e5eb4e4446541ea0ed53111c1b37e779283ff57c).
Extension constructors, `InlineParser`, `BlockParser`, temporary `KindData::Extension`
nodes and `AstTransformer` do the work; no fork or upstream patch is required.

## Phase B: preceding literal directives

```mdx
{/* leafbud:key introduction */}
## Introduction

{/* leafbud:key central-argument */}
This is ordinary Markdown with **formatting**.
```

Without the extension, Rushdown treats that MDX-shaped comment as visible prose.
The extension recognizes this exact single-line directive, **not general MDX**.
It binds the immediate next block sibling in the same container. Blank lines
between directive and target are allowed; a blank line is required before the
directive when preceding text is an open paragraph. The directive never skips
other directives, HTML/components or link-reference definitions to find prose.
An annotation before a list keys the list; `- {/* leafbud:key first */}`
followed by indented prose keys the paragraph **inside** that item, not the item.

Comments can also key a quote, fence or thematic break. An empty item/quote
annotation cannot escape its container. Both modes protect escaped braces,
fenced/indented/inline code (including multiline inline code). Each mode reserves
only its own syntax. Malformed reserved directives diagnose; unrelated comments
and expressions remain normal CommonMark input, not executable code.
