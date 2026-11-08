---
type: ExperimentGuide
title: "mdxjs parse-only identity and literal-component experiment"
description: "Isolated offline CLI, fixtures, and checks for native MDX comments as optional author keys and strict JSON component properties."
resource: /experiments/09-mdxjs-identity/README.md
status: verified-experiment
generated:
  at: 2026-11-07
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4812756bffetb3uFNU3E1ERrt"
    title: "Commissioned independent mdxjs experiment: executable adapter, controls, and reproduction guide"
  - resource: /experiments/09-mdxjs-identity/REPORT.md
    title: "Results, exact seams, syntax tradeoff, and limitations"
---

# mdxjs identity experiment

**Read [REPORT.md](REPORT.md) for the verdict.** This tests the public Rust
`mdxjs` API, not JavaScript evaluation or a substituted parser.

From this directory:

```sh
cargo fetch --locked                                      # setup only
cargo run --locked --offline -- stock fixtures/attribute-conflict.mdx
cargo run --locked --offline -- check fixtures/comments.mdx
cargo run --locked --offline -- check fixtures/components.mdx
cargo run --locked --offline -- check fixtures/rejected.mdx # intentional exit 2
cargo nextest run --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

`stock` prints upstream mdast. `check` prints a **transient** typed projection
and diagnostics to stdout; any diagnostic or upstream syntax error exits 2.
There is no source writer, renderer, JavaScript runtime, synchronized JSON file,
network operation, or Leaflet/Yrs integration. The original `.mdx` stays authoritative.

Selected optional prose identity syntax:

```mdx
{/* leafspring:key introduction */}
## Introduction

{/* leafspring:key central-argument */}
Ordinary prose with **formatting**.

<Poll key="topic" options={["Gardens", "Maps"]} />
```

Identity comments require a blank line after an earlier sibling, stay in their
own container, and bind only the immediate next supported block. Component
braces accept strict JSON, not arbitrary JavaScript. This is a small grammar
probe, **not** complete component schemas or a production security boundary.

The user's `{#id}` example is a deliberately retained control: this pinned
upstream unexpectedly accepts its expressions, but does not give them author-key
semantics. The selected adapter rejects them; the report explains why comments
are the safer direct-mdxjs choice.
