---
type: Index
title: Springleaf experiments
description: "Reading map for protocol evidence, parser alternatives, and the source-authoritative Satteri/Leaflet compiler experiments."
resource: /experiments/README.md
status: maintained
updated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4900b8d7ffeDT9QuFwmKRAB7t"
    title: "Commissioned and reviewed the experiments; assembled this reading map and the current recommendation with alternatives"
  - resource: /experiments/01-draft-list/REPORT.md
    title: "Read-only draft discovery evidence"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Fact graph, pull/push and text protocol evidence"
  - resource: /experiments/03-post-builder/REPORT.md
    title: "Record construction and explicitly unexecuted publication probe"
  - resource: /experiments/04-jacquard/REPORT.md
    title: "Rust stack and code-generation evaluation"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "CRDT interoperability scope and limits"
  - resource: /design/authoring-parser-assessment1.gpt6a.md
    title: "Comparative synthesis of parser experiments 06–10 and 12 before the Satteri compiler proof"
  - resource: /experiments/13-satteri-authoring/REPORT.md
    title: "Complete offline authoring slice, beyond parser-only evidence"
  - resource: /experiments/14-satteri-authoring-attributes/REPORT.md
    title: "Optional brace-key syntax composed with the same compiler"
---

# Springleaf experiments

Tracked, reproducible evidence for protocol and architecture decisions. Reports
state what ran, which source/dependency revisions were used, what failed, and
what remains unproved. They are not all implemented product capabilities.

## Start here

- **Implementation handoff:** [preliminary local-MDX design](../design/authoring-handoff0.gpt61s.md)
  connects the recommended workflow to module boundaries, reusable experimental
  code, remaining decisions and production/remote verification gates. Its
  [latest addendum](../design/authoring-handoff0.gpt61s.md#keyed-bidirectional-addendum)
  restores identity-preserving MDX/Yrs edits and a modest bidirectional target;
  the combined path still needs its own proof. The
  [CLI design arena](../design/authoring-handoff0.gpt61s.md#remake-and-cli-arena)
  inventories commands, effects, bulk workflows and choices to settle before
  remaking the public interface.
- **Current recommendation:** [local MDX authoring with Satteri, with
  alternatives](../design/authoring-recommendation0.gpt61s.md). This incorporates
  the later compiler evidence, not just the initial parser comparison.
- **What is stored in the file?** The [YAML frontmatter proposal](../design/frontmatter0.gpt61s.md)
  gives concrete author/state examples and alternatives for identity, sync,
  publication receipts and source writing. It is not implemented by the small
  experimental metadata parser yet.
- **Does Satteri actually fit the workflow?** Start with
  [13 — source to Leaflet facts and interoperable text](13-satteri-authoring/REPORT.md).
- **Can we also write `{#key}`?** Read
  [14 — optional brace keys](14-satteri-authoring-attributes/REPORT.md).
- **What is the intended workflow?**
  [Local MDX → replace the Leaflet body → publish](../design/mdx-recreate0.gpt6a.md)
  explains the earlier recreation proposal. The handoff addendum above retains
  recreation as a fallback rather than discarding existing identities on each edit.
- **Why were other parsers considered?** The
  [comparative assessment](../design/authoring-parser-assessment1.gpt6a.md)
  records the earlier choice and alternatives. Its mdxjs preference predates 13/14.

## Protocol and Rust foundations

| Experiment | Evidence | Boundary |
| --- | --- | --- |
| [01 — draft discovery](01-draft-list/REPORT.md) | Live read-only `/home`/metadata probes, RSC extraction and synthetic fixtures | Observed listing size is not proof of complete pagination |
| [02 — draft internals](02-draft-internals/REPORT.md) | Read-only probes plus source inspection of the fact graph, pull/push and Yjs | Mutation vocabulary/source semantics are not executed write verification |
| [03 — post construction](03-post-builder/REPORT.md) | Canonical record building, validation, identity/publication reads and URL derivation | The included live publication script was not executed |
| [04 — Jacquard](04-jacquard/REPORT.md) | Rust library/code-generation evaluation against pinned source | Not a production rewrite onto the entire library stack |
| [05 — Yrs conversion](05-yrs-conversion/REPORT.md) | Real-Yjs interoperability for decoding, rich text, incremental edits, repeats and one synthetic fact merge | Not full structure/media coverage or multiplayer guarantees; recreation needs only part of this machinery |

## Markdown/MDX substrate experiments

These runs address related questions with different interfaces and scopes.
Passing test counts include negative controls; they are receipts, not rankings.

| Experiment | Actual route and useful result | Limitation or disposition |
| --- | --- | --- |
| [06 — Rushdown](06-rushdown-identity/REPORT.md) | Parser extensions implement `{#id}` and comment keys on ordinary Markdown | Rich literal component grammar is still our work; custom-dialect alternative |
| [07 — Satteri identities](07-satteri-identity/REPORT.md) | Direct Satteri pulldown → MDAST arena; native components and comment binding | Earlier binder had a silent multiline-inline-code counterexample; not the final authoring compiler |
| [08 — pulldown-cmark](08-pulldown-cmark-identity/REPORT.md) | Offset-event transformation implements heading/paragraph `{#id}` | No semantic MDX/component parser; Markdown-first alternative |
| [09 — stock mdxjs](09-mdxjs-identity/REPORT.md) | Native MDX tree, comments and literal components | Its old SWC graph required historical Serde pins; 12 demonstrates a remedy |
| [10 — Satteri compiler](10-satteri-mdxjs-identity/REPORT.md) | Executes `satteri-mdxjs` and verifies its shared parser/arena architecture | It is a JS compiler over the parser in 07, not another parser backend; lower directly from MDAST instead |
| [12 — modern mdxjs fork](12-mdxjs-turbopack-identity/REPORT.md) | Parse-only Rust API, modern Serde build, native `<Key />` and rich components | Strong fallback; exact Git pin and SWC remain dependency obligations |

There is no experiment 11: investigation in 10 established that the proposed
second Satteri parser backend was the same parser under its compiler. The gap
is intentional, not missing evidence.

## Authoring compiler and optional syntax

| Experiment | What it demonstrates | What it does not |
| --- | --- | --- |
| [13 — Satteri authoring](13-satteri-authoring/REPORT.md) | Ordinary Markdown, native `<Key />`, named Pages and Polls → typed document → Leaflet-shaped facts and fresh Yrs text; independent Yjs/ProseMirror consumption; simulated staging/replacement/recovery | Complete block/media coverage, production dependency integration, deployed permissions, actual staging/switch or native publication |
| [14 — Satteri brace keys](14-satteri-authoring-attributes/REPORT.md) | Optional heading/paragraph `{#key}` alongside `<Key />`, producing equivalent document and fact plans through unchanged 13 | Native paragraph attributes: this is an explicit two-parse adapter using pinned recovery nodes, not a new upstream grammar feature |

**Taxonomy matters:** `satteri-mdxjs-rs` is a source-directory name for the
`satteri-mdxjs` compiler. Its parser is `satteri-pulldown-cmark`. Neither is the
ordinary `pulldown-cmark` package in 08. Likewise, the Git mdxjs fork in 12 is not
selected by installing registry `mdxjs` with the same version number.

## Reproduce useful results

Follow each experiment's own README or report for dependency setup and exact
pins. Later Rust experiments are isolated workspaces; root `cargo test` does
not run them. Dependency downloads are setup traffic; their ordinary tests are
offline after setup. The independent JS consumer is a test oracle, not a proposed
JavaScript runtime dependency for springleaf.

From the repository root, a useful first run is:

```sh
cargo fetch --locked --manifest-path experiments/13-satteri-authoring/Cargo.toml
cargo run --locked --offline --manifest-path experiments/13-satteri-authoring/Cargo.toml -- demo experiments/13-satteri-authoring/fixtures/garden.mdx
```

Then use [13's reproduction instructions](13-satteri-authoring/README.md) for
the independent Yjs consumer, or [14's comparison](14-satteri-authoring-attributes/README.md)
to compare brace and native-Key spellings. Experiment 14 intentionally depends
on 13; reproduce both directories from the same checkout.

## Experiment practice

1. **Prefer a new experiment space for a new question.** Use the next unused
   `NN-topic/` directory with a report, runnable probe and synthetic fixtures.
   Keep earlier results and failures inspectable. A focused follow-up may reuse
   another experiment explicitly; record that dependency rather than concealing it.
2. **Keep prototypes isolated.** Use their own manifests/locks and local ignored
   build outputs. Do not silently turn experimental code into production code.
3. **No credentials in evidence.** Read secrets from the configured auth store or
   environment at runtime only when needed; never print, log or commit them.
   Real response bodies stay private; committed examples are synthetic/redacted.
4. **No remote writes without explicit operator approval.** Classify by behavior,
   not HTTP method: `GET /new` creates a real draft. A source-backed write recipe,
   simulator or successful encoder is not authorization or proof of a live write.
5. **State the evidence boundary.** Separate live reads, source inspection,
   independent interoperability tests and simulated server behavior. Report failed
   assumptions and exact dependency/source identities; avoid unsupported claims
   about performance, complete coverage or deployed parity.
6. **Promote deliberately.** Recommendations synthesize findings into `design/`;
   adopting code or executing an operator-gated probe is a separate decision.
