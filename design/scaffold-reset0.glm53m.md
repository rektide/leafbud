---
type: Report
title: "Scaffold reset — eight-crate workspace, retention inventory, and build handoff"
description: "Executed workspace reset to the accepted eight-crate scaffold: what was retained from the stage-5 client, what was removed and where it re-homes, dependency and version policy, verification, and the recommended next integration slice."
resource: /design/scaffold-reset0.glm53m.md
status: final
generated:
  at: 2026-11-08
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e464771dfffe6czx6QEC8Qryue"
    title: "Commissioned scaffold/reset execution: crate moves, retention decisions, CLI skeleton, verification"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Parent session accepting the authoring interface and commissioning the reset"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted destination: root verbs, tidy semantics, eight-crate map, YAML preservation"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Keyed bidirectional addendum and CLI capability arena consulted for re-homing decisions"
  - resource: /design/frontmatter0.gpt61s.md
    title: "YAML envelope that replaces the reset sidecar lifecycle"
  - resource: /Cargo.toml
    title: "Reset workspace manifest: eight members, version 0.2.0, trimmed dependency pins"
  - resource: /crates/leaflet/records/LEAFLET-PIN.md
    title: "Generated-tree pin procedure updated for the records crate path"
---

# Scaffold reset

## Commission and starting point

The operator commissioned the scaffold/reset stage of the accepted
[authoring interface](authoring-interface0.gpt6a.md) on 2026-11-08:
build the eight-crate workspace with clean boundaries, remove the
obsolete production implementation, and keep honest status — no
business workflows, no speculative type hierarchies, no pretend-success
stubs. This document records what was done so the next slice starts
from verified ground.

**Pre-reset revision: `80436b61`** ("Accept authoring interface and
unify local normalization under tidy"). Every deleted file below is
recoverable there; `jj diff -r 80436b61` shows the full stage-5 client.

## Workspace shape (as built)

Exactly the accepted destination's eight crates:

| Crate | Package | Status |
| --- | --- | --- |
| `crates/document/model` | `springleaf-document` | stub (crate docs only) |
| `crates/document/mdx` | `springleaf-mdx` | stub; Satteri dependency deferred |
| `crates/leaflet/editor` | `springleaf-editor` | stub; Yrs dependency deferred |
| `crates/leaflet/web` | `springleaf-web` | stub; reqwest dependency deferred |
| `crates/leaflet/records` | `springleaf-records` | **real**: generated pinned types + wire helpers |
| `crates/atproto` | `springleaf-atproto` | **real**: unchanged client surface |
| `crates/workflow` | `springleaf-workflow` | stub |
| `crates/cli` | binary `springleaf` | skeleton: full accepted surface, every command honestly not-implemented |

Stub crates deliberately declare **no dependencies**: an edge that no
code uses yet would only draw the design graph (the accepted design's
dependency direction is recorded in each stub's crate docs). Workspace
version bumped 0.1.0 → 0.2.0: pre-alpha policy marks the public-interface
change (new crate set, new CLI surface) in one coordinated step.

## Retained components and why

| Retained | Home | Rationale |
| --- | --- | --- |
| Entire `springleaf-atproto` (identity, DNS/DID resolution, XRPC, sessions, test server) | `crates/atproto` | Matches its destination ownership verbatim: "existing public reads, identity, explicitly selected PDS writes". Only change: a doc-comment reference to the renamed records crate. |
| Generated jacquard lexicon tree (`com_atproto`, `pub_leaflet`, `site_standard`, `builder_types`, features) | `crates/leaflet/records` | The committed tree **is** the lexicon pin; regenerating or rewriting it would manufacture churn, not clarity. |
| `validate_document_record` + `reinject_type_discriminators` | `records/src/record.rs` | Records' named "validation" ownership, and the wire-shape adaptation every future document serializer needs (single-variant unions drop `$type`). |
| `standalone_site_did` + `derive_document_url` | `records/src/record.rs` | Proven public-URL rules (experiment 03 §6) — publication-domain, `/lish` deep-link, and standalone `/p/{did}` forms. Introduces the one real `records → atproto` edge (`AtUri` parsing); no cycle (atproto stays lexicon-agnostic). |
| LEAFLET-PIN.md | `crates/leaflet/records/` | Regeneration procedure, updated for the new path and the retained test. |

The old `build_record` golden test was replaced by an equivalent
**pin-drift alarm**: `built-record.json` from
[experiment 03](../experiments/03-post-builder/REPORT.md) must decode
through the generated type, pass lexicon validation, and round-trip
exactly after discriminator re-injection. Regeneration checklist step 3
in LEAFLET-PIN.md now names this test.

## Reset components and re-homing

| Removed (at `80436b61`) | Lines | Re-home when needed |
| --- | --- | --- |
| Old CLI command tree (`main.rs` modules: publish, sync, author, create, export, workspace, config*) | ~3,800 | `config.rs`/`config_cmd.rs` (figment2 + toml_edit layering) → `workflow`'s first slice; the rest dies with the sidecar lifecycle. |
| `springleaf-drafts` web client (`lib.rs`, `create.rs`) | ~1,580 | `LeafletClient` core (`/home` RSC parse, `get_leaflet_data`, `/new`) → `web` when discovery/`remote list` lands. |
| Yjs codec (`yjs.rs`), fractional positioning (`fractional.rs`) | ~2,520 | Proven byte-parity wire codec → `editor`'s first slice (text decoding/encoding); re-evaluate against Yrs-generated updates then. |
| Fact decoding (`facts.rs`) | ~490 | `editor`, but **not as-is**: `FactData::Other { kind }` drops payloads; the accepted design requires raw-payload-retaining observations. |
| Extraction ⇄ authoring converters (`extract.rs`, `author.rs`, `pull.rs`, `push.rs`) | ~4,050 | Not recovered: they convert between records and the editor via the reset lifecycle. The new path is document model ↔ editor facts. The Replicache push transport shape returns to `web` for `remote append`. |
| Markdown→blocks converter, frontmatter parser, record builder, slug policy (`markdown.rs`, `record.rs` builder half) | ~1,150 | Not recovered: local record assembly is replaced by the MDX document model and a future document-to-public projection. |
| Publication-record fetch helper (`publication.rs`) | ~110 | `web` (public reads) when publication discovery lands; too small to keep an ambiguous records→network helper warm. |
| `uuid` dependency, `anyhow`/`figment2`/`jiff`/`sha2`/`toml_edit`/`tracing-subscriber` workspace pins | — | Re-added by the crates that actually use them (`uuid` was only the reset builder's page-id minting). |

Experiments were verified to have **no path dependencies** on
`crates/*` (each is an independent Cargo project); `experiments/`,
`design/`, `.beads/`, and root metadata are untouched. Historical
citations of `crates/drafts`/`crates/leaflet` in earlier design
documents and reports intentionally remain: they are revision-bound
evidence, recoverable at `80436b61`.

## CLI skeleton contract

`crates/cli/src/main.rs` parses the accepted command map — root verbs,
`keys rename` only (no `keys assign`; manual keying is `tidy FILE
--keys`), `tidy` with all-applicable default and `--keys`/`--from`/
`--accept` narrowing, `remote`/`pubs`/`auth`/`config` groups, bulk
`--all` + state filters, and `--take local`/`--take remote` spelled per
command. Every arm funnels to one `not_implemented` path: message on
stderr naming the command, exit **1** (operational failure). Clap usage
errors exit **2**, `--version`/`--help` exit **0** — consistent with
the accepted exit-code contract. Flag spellings beyond the map (e.g.
exact `--dry-run` coverage per command) remain provisional until real
implementations fix them; nothing implies an operation ran.

## Verification

- `cargo build --workspace` — clean.
- `cargo fmt --check` — clean.
- `cargo clippy --workspace --all-targets` — clean, zero warnings
  (including the unused-workspace-dependency check after dropping the
  `springleaf-records` workspace entry until a consumer exists).
- `cargo nextest run --workspace` — **35/35 passed** (32 atproto
  offline wire/session tests; 3 records tests: fixture round-trip drift
  alarm, URL derivation forms, standalone-site near-misses).
- Manual CLI probes: `--version` → `springleaf 0.2.0`; `tidy … --keys`,
  `push … --take local` → not-implemented, exit 1; `--take bogus` →
  clap usage error, exit 2.

No test was added solely to mirror scaffolding; the retained tests
exercise real retained code.

## Interface concerns flagged for the next slice

1. **records → atproto edge** exists solely for `AtUri` parsing in URL
   derivation. If the projection layer prefers pre-parsed components,
   lift it then; today it is a real, acyclic, tested edge.
2. **`springleaf-records` has no consumer yet**, so it is absent from
   `[workspace.dependencies]` (unused entries warn). Re-add the pin
   when `workflow`/`web` first depends on it.
3. **`facts.rs` recovery must change shape** (retain raw payloads) —
   copying it verbatim would import the design's stated defect.
4. **Experiment fixture paths**: records' tests reference
   `../../../experiments/03-post-builder/built-record.json` (one more
   level than before the move) — covered by the passing suite.

## Next integration slice

The accepted design's first build join (§4) is the right next
commission: editor fixture → keyed MDX → prose/mark/move edit → same
remote objects → pull, plus frontmatter presence, scoped fingerprints,
and one nested-placement binding fixture. Concretely from this
scaffold: (a) give `springleaf-editor` the Yrs dependency and recover
`yjs.rs`/`fractional.rs`/a payload-retaining `facts.rs` from
`80436b61`; (b) give `springleaf-mdx` Satteri and the YAML profile;
(c) let the document model emerge from that fixture rather than
pre-designing it here. Workflow/config recovery and any web transport
follow after the loop is proven.
