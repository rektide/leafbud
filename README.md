# springleaf

Local-first MDX authoring for [leaflet.pub](https://leaflet.pub)
documents, in Rust: one MDX file per document, durable identities,
push saves the Leaflet editor, publish releases, pull adopts web edits.

Status: **post-reset scaffold.** The accepted authoring interface
([`design/authoring-interface0.gpt6a.md`](design/authoring-interface0.gpt6a.md))
settles root CLI verbs (`create`/`check`/`list`/`status`/`diff`/
`push`/`pull`/`publish`/`import`/`edit`/`tidy`/`export`, plus
`keys rename`, `remote`, `pubs`, `auth`, `config`), all-applicable
`tidy` with `--keys` narrowing, eight domain-grouped crates, and
operation-specific YAML preservation. The workspace now matches that
destination shape, but only two crates contain real implementation:

| implemented | not implemented |
| --- | --- |
| `springleaf-atproto` — identity resolution, XRPC client, app-password sessions (public reads, putRecord, blob upload), offline-tested | document model, MDX parsing/keying, editor fact graph/Yrs, web effects, workflow orchestration |
| `springleaf-records` — generated pinned Leaflet lexicon record types, validation, wire-shape and URL-derivation helpers, fixture round-trip tests | every CLI operation (the binary parses the accepted surface and reports not-implemented, exit 1) |

The pre-reset stage-5 client (cookie web surface, JSON/sidecar
publish+sync lifecycle, append authoring, hand-rolled Yjs codec) was
removed by design and is recoverable at revision `80436b61`; the
[cleanup handoff](design/scaffold-reset0.glm53m.md) records exactly
what was kept, what was reset, and where each piece re-homes.

## Layout

```
crates/
  document/
    model/     springleaf-document: pure document model — current content,
               stable source IDs, diagnostics, tracking types (stub)
    mdx/       springleaf-mdx: Satteri parsing, YAML frontmatter, key
               insertion, MDX reading/writing (stub; Satteri deferred)
  leaflet/
    editor/    springleaf-editor: fact graph, Yrs text state, bindings,
               bidirectional mapping, pure edit planning (stub; Yrs deferred)
    web/       springleaf-web: cookie auth, discovery, RPC/actions,
               uploads, native publication (stub)
    records/   springleaf-records: generated pinned lexicon record types,
               validation, wire-shape helpers — see LEAFLET-PIN.md
  atproto/     springleaf-atproto: hand-rolled ATProto surface — handle→DID→PDS
               identity resolution, XRPC client, sessions, public reads
  workflow/    springleaf-workflow: local files, config, inventory,
               plan/execute/verify orchestration (stub)
  cli/         springleaf (binary): thin Clap/rendering layer; every
               command honestly reports not-implemented
design/        architecture, accepted interface, and format designs
experiments/   protocol probes and evidence (tracked, load-bearing)
```

Dependency direction (accepted design §2): pure document model →
MDX adapter / editor / record projections; `web` owns effects;
`workflow` composes the libraries; the CLI stays thin. The scaffold
adds no crate dependency edges that no code uses yet.

## Build and check

```
cargo build
cargo run -p springleaf -- --version
cargo run -p springleaf -- push essay.mdx   # -> "not implemented", exit 1
```

Checks: `cargo fmt`, `cargo clippy --workspace --all-targets`,
`cargo nextest run`. Tests are offline (loopback HTTP responders,
committed fixtures under `experiments/`); no test writes to any real
PDS or leaflet draft.

## Reading map

- Accepted interface (CLI, crates, YAML policy):
  [`design/authoring-interface0.gpt6a.md`](design/authoring-interface0.gpt6a.md)
- Governing handoff incl. keyed bidirectional addendum and CLI arena:
  [`design/authoring-handoff0.gpt61s.md`](design/authoring-handoff0.gpt61s.md)
- YAML frontmatter envelope: [`design/frontmatter0.gpt61s.md`](design/frontmatter0.gpt61s.md)
- Reset inventory and next slice: [`design/scaffold-reset0.glm53m.md`](design/scaffold-reset0.glm53m.md)
- Experiment reading map and evidence protocol: [`experiments/README.md`](experiments/README.md)

Never commit credentials or real response bodies; see
[`experiments/README.md`](experiments/README.md).
