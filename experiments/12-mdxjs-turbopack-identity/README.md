# Direct Rust test of Vercel's mdxjs fork

**Current Serde builds; Markdown grammar gotchas remain. Native `<Key id="…" />`
works beside ordinary Markdown.** See [the report](REPORT.md) for the conditional
verdict and deliberately retained negative observations.

The package is **`mdxjs` 1.0.4 from Git**, not a registry crate named
`mdxjs-rs-turbopack`. Exact source:
`vercel-labs/mdxjs-rs-turbopack@700e192fbdbad5874208addfbf421dcf1eca7a20`.
[UPSTREAM.json](UPSTREAM.json) records source hashes; [Cargo.lock](Cargo.lock)
pins the graph. No local dependency patch, Turbopack, Node, JavaScript execution,
Leaflet access, production manifest change, or earlier-experiment edit is involved.

## Run

```sh
cd experiments/12-mdxjs-turbopack-identity
# First setup, when dependency caches are empty:
cargo fetch --locked

# All execution/checks below are offline:
cargo nextest run --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
cargo tree --locked --offline --depth 2 -e normal

cargo run --locked --offline -- stock fixtures/key-candidate.mdx
cargo run --locked --offline -- check fixtures/key-candidate.mdx
cargo run --locked --offline -- check fixtures/document.mdx
```

`stock` prints the fork's native mdast. `check` prints that same ordered tree,
literal component data, marker-consumption paths, bindings, and diagnostics.
It exits 2 on upstream/adapter errors; inspection output is never a saved JSON
partner or permission to publish. There is no write path.

## Candidate Key rule, not an adoption decision

```mdx
<Key id="introduction" />
## Introduction

<Key id="central-argument" />
This is ordinary **formatted** prose.
```

The standalone self-closing marker names the immediate supported sibling in its
current container. No blank line is needed **after** it. The comparison adapter
requires a blank line **before** a marker after another sibling, to reject known
multiline-code interruptions. Quoted literal `id` only; duplicates/orphans diagnose.
The marker is declarative authoring data, not a runtime component.
Paired `<Key>…</Key>` parses but diagnoses because group semantics are not selected;
unclosed `<Key …>` fails natively. Comments and `{#id}` remain comparison controls,
not adopted recommendations.

## Do not confuse green observations with safe behavior

51 tests pass, including explicitly named negative observations. For example:

```sh
# exit 2: guard rejection, not native restoration of a code span
cargo run --locked --offline -- check fixtures/multiline-comment.mdx
# exit 0: deliberately recorded false-clean component/key interpretation
cargo run --locked --offline -- check fixtures/multiline-component.mdx
# exit 2: valid native paired JSX, deliberately undefined grouping policy
cargo run --locked --offline -- check fixtures/key-paired.mdx
```

The harness is a comparison adapter, **not a production-safe authoring compiler**.
MDX also treats four-space-indented JSX as active markup, not CommonMark code.
Native MDX+Key is feasible; this fork does not change that underlying dialect.

## Files

- [src/lib.rs](src/lib.rs): actual fork API, scoped marker binder and data views.
- [src/literal.rs](src/literal.rs): strict JSON visitor derived from experiment 09.
- [tests/stock.rs](tests/stock.rs): native grammar versus interpretation controls.
- [tests/key.rs](tests/key.rs): self-closing/paired/unclosed Key and attachment cases.
- [tests/identity.rs](tests/identity.rs): shared comments/code/hash comparison corpus.
- [tests/components.rs](tests/components.rs): nested Page/Canvas/Place and exact data.
- [tests/cli.rs](tests/cli.rs): executable output/exit controls, including false-clean evidence.
