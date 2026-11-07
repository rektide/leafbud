# Yrs conversion experiment

Offline Rust ↔ real Yjs/y-prosemirror prototype. Production crates are untouched.
Results and limits: [REPORT.md](REPORT.md).
Proposed full adapter: [Yrs conversion design](../../design/yrs-conversion0.gpt6a.md).

## Run

From this directory, with Rust (edition 2024), Node.js 22+ and pnpm installed:

```sh
pnpm install --frozen-lockfile
cargo nextest run --locked
pnpm test
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Dependency downloads are needed on first setup; the tests themselves use only
local subprocesses and files. No authentication or application configuration is
read. `cargo test --locked` also runs the Rust tests when nextest is unavailable.

`Cargo.toml` starts an isolated workspace; both lockfiles belong only to this
experiment. Yrs is pinned to the inspected upstream Git commit, not a mutable
path or branch. The JS dependency versions are exact. No browser/DOM is required:
the reference uses the model/conversion functions of y-prosemirror with a declared
subset of Leaflet's schema. It does not mount Leaflet's editor.

```sh
# Rebuild deterministic synthetic wire/fact fixtures deliberately:
pnpm fixtures
# Decode a committed sample, using the already built Rust binary:
node --input-type=module -e 'import {readFile} from "node:fs/promises"; const f=JSON.parse(await readFile("fixtures/wire.json")); console.log(JSON.stringify({update:f.samples[2].update}));' | target/debug/yrs-conversion-experiment
```

Rust requests are `{update: base64V1, desired?: {plaintext, facets}, client?: number}`.
Responses include the before/after projection and XML structure, a nullable
incremental update, and full state for **test peer exchange only**. `null` means
no mutation. The runner keeps responses in memory; it writes no working-document
snapshots or mutation batches. The committed synthetic fixtures are test inputs.

## Scope

- One `prosemirror` paragraph; XmlText and `hard_break` children.
- UTF-8 canonical facets; scalar-boundary edits mapped to UTF-16 Yrs positions.
- Seven ordinary marks, highlight slot 1, overlapping facet normalization.
- One transaction per edit; retained text/node identities and comment marks.
- A tiny fact-graph scenario in `reference.mjs` constructs one text assertion,
  simulates `Y.mergeUpdates`, and verifies read-back and a repeated no-op.

Mentions, footnotes, arbitrary XML/embeds, colored highlights and multiple
paragraphs are explicit errors in this prototype. The fact harness handles one
linear text block, not a full adapter. See the report for identity limitations
when splitting nodes, malformed-input behavior and concurrency boundaries.
