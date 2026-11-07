# Rushdown block author-key experiment

An offline, isolated Rust executable using **Rushdown's extension API**, not a
Markdown preprocessing pass. Production crates/manifests are unchanged. It makes
no network requests and performs no Leaflet writes.

From this directory:

```sh
cargo run --locked --offline -- attributes fixtures/attributes.md
cargo run --locked --offline -- plain fixtures/attributes.md
cargo nextest run --locked --offline
```

Fetch dependencies once with `cargo fetch --locked` if the local Cargo cache is
empty. `attributes` enables optional author keys; `plain` demonstrates baseline
CommonMark interpretation. The executable prints a transient JSON projection
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

The published dependency is pinned exactly to `rushdown = "=0.18.0"`, matching
[`yuin/rushdown` revision e5eb4e4](https://github.com/yuin/rushdown/tree/e5eb4e4446541ea0ed53111c1b37e779283ff57c).
Extension constructors, `InlineParser`, `BlockParser`, temporary `KindData::Extension`
nodes and `AstTransformer` do the work; no fork or upstream patch is required.
