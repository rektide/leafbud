# Sätteri author-key experiment

Isolated offline Rust evidence, using `bruits/satteri`'s published parser/arena
crates. No production integration, Leaflet requests, source rewriting, or saved
JSON companion. The high-level `satteri` crate only offers HTML/JS output; this
probe uses the underlying public Rust AST API directly.

Initial setup: `cargo fetch --locked`. Then, from this directory:

```sh
cargo nextest run --locked --offline
cargo run --locked --offline -- attributes fixtures/attributes.mdx
cargo run --locked --offline -- author fixtures/attributes.mdx
```

At the initial checkpoint, the last command deliberately demonstrates a native
grammar gap: paragraph-tail attributes become MDX expressions and exit 2.
Seven baseline tests distinguish upstream parsing from our subsequent lowering.
The experiment is proceeding with native MDX comments instead of forcing that
attribute syntax into the parser.
