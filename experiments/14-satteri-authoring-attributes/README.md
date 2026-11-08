# Optional `{#key}` on Satteri authoring

A focused follow-up to unchanged experiment 13. See [REPORT.md](REPORT.md) for
stock behavior, the explicit adapter seam, comparison receipts and recommendation.

```mdx
## Introduction {#introduction}

An ordinary paragraph with **formatting**.
{#central-argument}

<Key id="another-block" />
Ordinary native-Key prose still works.
```

- Heading key: one final `{#name}`, separated from content by whitespace.
- Paragraph key: its own immediately following line, **no blank before it**.
  Follow with a blank line, container closure or EOF. In a quote repeat `>`;
  in a list indent the suffix into the same item. These name the paragraph,
  not a nearby list/quote/item container.
- Keys share the native Key/component-key namespace. Duplicates, both forms on
  one block, malformed and orphan keys fail. These are author identities, not
  HTML anchors or Yjs character identities.
- Native code/escaped text/component string values are not annotations. Use
  fences for multiline code and `\{#literal}` for literal braces in prose.
  Same-line paragraph suffixes, classes and arbitrary attributes are outside
  this small experiment.

From this directory:

```sh
cargo fetch --locked
cargo run --locked --offline -- compare fixtures/garden-attributes.mdx ../13-satteri-authoring/fixtures/garden.mdx
cargo run --locked --offline -- check fixtures/garden-attributes.mdx
cargo run --locked --offline -- stock fixtures/garden-attributes.mdx
cargo run --locked --offline -- check fixtures/rejected.mdx # expected exit 2
cargo nextest run --locked --offline --no-fail-fast
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

`stock` prints native MDX and MDX+attributes observations, including native errors;
it is inspection, not a validated compiler result. `check` emits transient typed
IR and a fresh-generation fact plan. `compare` verifies actual typed/Leaflet plan
equivalence against the original Key fixture and stages/reads the fresh text in
the offline simulator. No source/sidecar/state file is written. No credentials,
application endpoint, JS execution or publication transport is present.

The adapter uses recovered native expression nodes, masks only exact annotation
bytes with same-length spaces, then requires the unchanged compiler's strict
reparse and schemas. **This is adapter support, not native paragraph attributes.**
Original bytes are retained and every byte offset stays unchanged.

The relative Cargo dependency on [experiment 13](../13-satteri-authoring/README.md)
is deliberate: reproduce with both directories in this repository. Its exact
source receipt and the new isolated lock are recorded in [UPSTREAM.json](UPSTREAM.json).
No previous experiment test suite needs rerunning. Tested with Linux/Rust 1.101
nightly; no new minimum/cross-platform claim is made.
