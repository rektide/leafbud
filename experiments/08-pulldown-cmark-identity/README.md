# pulldown-cmark: optional author keys through events

Offline, isolated Rust experiment. [REPORT.md](REPORT.md) records the verdict,
exact dependency/source pin, native controls and limits. No production integration,
Leaflet requests, source writes or maintained JSON companion.

```sh
cd experiments/08-pulldown-cmark-identity
cargo fetch --locked                                     # setup only
cargo run --locked --offline -- attributes fixtures/attributes.md
cargo run --locked --offline -- plain fixtures/components.mdx
cargo run --locked --offline -- native fixtures/attributes.md
cargo run --locked --offline -- strict fixtures/components.mdx # intentional exit 2
cargo run --locked --offline -- attributes fixtures/invalid.md # intentional exit 2
cargo nextest run --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

- `plain`: baseline CommonMark events and HTML.
- `native`: upstream `ENABLE_HEADING_ATTRIBUTES`; HTML IDs, not author keys.
- `attributes`: our event-layer dialect: heading suffix `{#heading}` and a
  paragraph's whole final line `{#paragraph}`. No preceding blank line.
- `strict`: attributes plus an intentionally overrestrictive component/HTML
  rejection control. **Not an MDX or JSON-property parser.**

Output is inspectable transient JSON containing untouched raw events, original
source, typed event-tree projection, internal keys, spans, diagnostics and HTML.
Any diagnostic means exit 2; partial keys/HTML are not safe compiled output.
The plain/attribute HTML renderer passes raw HTML through: do not treat its output
as sanitized or load it as an authoring preview without a separate policy.

Comment identities are not implemented. The requested MDX-shaped reserved
comment is diagnosed in attribute/strict modes, rather than silently binding a
later sibling. No JSX/JavaScript is executed.
