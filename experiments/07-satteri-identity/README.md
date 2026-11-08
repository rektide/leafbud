# Sätteri author-key experiment

[Assessment, exact source pins, test receipts and adoption caveats](REPORT.md).

Isolated offline Rust evidence, using `bruits/satteri`'s published parser/arena
crates. No production integration, Leaflet requests, source rewriting, or saved
JSON companion. The high-level `satteri` crate only offers HTML/JS output; this
probe uses the underlying public Rust AST API directly.

Initial setup: `cargo fetch --locked`. Then, from this directory:

```sh
cargo nextest run --locked --offline
cargo run --locked --offline -- attributes fixtures/attributes.mdx
cargo run --locked --offline -- author fixtures/comments.mdx
cargo run --locked --offline -- author fixtures/components.mdx
cargo run --locked --offline -- author fixtures/negative.mdx  # intentional exit 2
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

The selected authoring spelling uses native MDX comment expression nodes:

```mdx
{/* leafspring:key introduction */}
## Introduction

{/* leafspring:key central-argument */}
This is an ordinary paragraph with **formatting**.

<Poll key="next-topic" options={["Gardens", "Maps"]} />
```

`author` binds each reserved comment to its immediate next block sibling in the
same container, removes the marker node through the public arena API, and adds
internal key metadata. Blank lines between marker and target are allowed.
Supported targets: heading, paragraph, list, quote, code, rule, flow component.
Inside a list item the comment keys its following paragraph, not the item.
Unbound/malformed/misplaced/duplicate keys diagnose; keys match
`[A-Za-z][A-Za-z0-9_.-]*`, case-sensitive and document-wide unique. No HTML anchor
is generated. Source bytes remain unchanged; the JSON projection is transient.

The component control recognizes only `Poll` and `Image`, preserves native
component/attribute AST structure, and reads braced attribute values as strict
JSON. It rejects unknown component names, arbitrary expressions, spreads and
duplicate properties. It does not render components or validate their complete
property schemas. No JavaScript is executed or generated.

## Important limitations

- The exact paragraph-tail `{#id}` request works in Markdown-only attribute mode
  but conflicts with native MDX flow expressions. Even enabling both flags does
  not solve it. `attributes` is a separate native-facility control, **not** a second
  completed authoring dialect; malformed text can fall back to prose and repeated
  IDs on one node are already collapsed by upstream metadata.
- **Native MDX flow comments interrupt multiline backtick spans.** The fixture
  below is ordinary inline code in `plain`, but a comment and two paragraphs in
  `mdx`; `author` mistakenly keys the trailing paragraph without a diagnostic.
  This is a reproduced unmet invariant, not a passing protection claim:

  ```sh
  cargo run --locked --offline -- plain fixtures/multiline-code.mdx
  cargo run --locked --offline -- mdx fixtures/multiline-code.mdx
  cargo run --locked --offline -- author fixtures/multiline-code.mdx
  ```

- MDX mode intentionally has no indented code. Single-line inline code and fenced
  code protect annotation-shaped contents in the tested corpus. Use fences for
  robust literal examples; do not assume complete CommonMark semantics.
- Native spans are UTF-8 byte offsets (half-open); line/column coordinates are
  one-based with UTF-16 columns. Offsets exclude an initial BOM; add the printed
  `position_source_start` to slice the retained original. Comment annotations have
  their own exact span. Native attributes/JSX property codecs expose no individual
  key-token range here. No edited-source round-trip is implemented or claimed.

`plain` is CommonMark with no option flags; `mdx` is upstream `MDX_OPTIONS` without
our lowering; `attributes` adds native general attributes without MDX; `author`
uses `MDX_OPTIONS` plus AST binding/literal checks, **without** general attributes.
Any diagnostic causes CLI exit 2, although partial output is printed for inspection.

Prerequisites: Rust 2024-capable toolchain (tested on rustc 1.101.0-nightly), Cargo,
and cargo-nextest for the recommended test command. `cargo test --locked --offline`
is an alternative without nextest. No Node.js, browser, upstream clone or network
is required after dependency setup. The isolated manifest has its own workspace
and exact direct pins; its lockfile pins transitives. `target/` is ignored locally.
