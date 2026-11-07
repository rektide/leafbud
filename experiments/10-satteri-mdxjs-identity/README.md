# Sätteri MDX compiler / arena authoring experiment

**One new experiment, not two new parser backends.** The public crate is
`satteri-mdxjs` 0.3.17; its source directory is `satteri-mdxjs-rs`. It compiles the
same `satteri-pulldown-cmark` 0.6.7 arena already exercised in experiment 07.
See [REPORT.md](REPORT.md) for the evidence, negative controls and recommendation.

This isolated Rust workspace has its own lockfile and no sibling path dependencies.
It never contacts Leaflet, executes JavaScript, writes source, or publishes anything.
Generated JSON/JavaScript is stdout inspection output, not a maintained document.

## Run

Use the toolchain recorded in the report (Rust 1.101 nightly was exercised).
Download dependencies once with `cargo fetch --locked`. Thereafter, from here:

```sh
cargo nextest run --locked --offline --no-fail-fast
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
cargo tree --locked --offline --depth 2 -e normal

cargo run --locked --offline -- check fixtures/comments.mdx
cargo run --locked --offline -- check fixtures/components.mdx
cargo run --locked --offline -- stock fixtures/components.mdx
cargo run --locked --offline -- staged fixtures/components.mdx
```

- `check`: actual native MDAST → optional comment keys → strict literal gate → JSON.
- `stock`: actual `satteri_mdxjs::compile` → JavaScript with JSX retained.
- `staged`: validated/mutated MDAST → native HAST conversion → actual
  `satteri_mdxjs::compile_hast_arena`. This is a compiler integration **control**,
  not the recommended Leaflet compiler path or an execution step.

Expected negative/control pairs:

```sh
cargo run --locked --offline -- stock fixtures/rejected.mdx  # exit 0, emits unsafe-for-storage JS
cargo run --locked --offline -- check fixtures/rejected.mdx  # exit 2
cargo run --locked --offline -- staged fixtures/rejected.mdx # exit 2
cargo run --locked --offline -- stock fixtures/multiline-code.mdx # exit 0, code semantics lost
cargo run --locked --offline -- check fixtures/multiline-code.mdx # exit 2, guard NOT parser repair
cargo run --locked --offline -- stock fixtures/attributes.mdx # exit 2, native grammar conflict
```

Optional identities use preceding `{/* springleaf:key NAME */}` or component
`key="NAME"`, in one document-wide namespace. Binding is immediate-next-sibling,
container-local. A blank line is required **before** a marker after an earlier
sibling; blank lines after it are allowed. Duplicate/malformed/unbound keys diagnose.
Ordinary Markdown needs no identity annotation or JSX wrapper.

Known limitations are deliberate receipts: MDX loses the tested multiline backtick
span; the guard rejects its accidental key but does not restore the code. MDX
disables indented code, and an indented marker can still bind. Use fences if
accepting that dialect. Literal schemas, resource limits, frontmatter and Leaflet
lowering are not implemented. All diagnostic-bearing output must be rejected.
