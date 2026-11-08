# springleaf

Local-first MDX authoring for [leaflet.pub](https://leaflet.pub)
documents, in Rust: one MDX file per document, durable identities,
push saves the Leaflet editor, publish releases, pull adopts web edits.

Status: **first read-only remote slice, version 0.3.0.** The accepted authoring interface
([`design/authoring-interface0.gpt6a.md`](design/authoring-interface0.gpt6a.md))
settles root CLI verbs (`create`/`check`/`list`/`status`/`diff`/
`push`/`pull`/`publish`/`import`/`edit`/`tidy`/`export`, plus
`keys rename`, `remote`, `pubs`, `auth`, `config`), all-applicable
`tidy` with `--keys` narrowing, eight domain-grouped crates, and
operation-specific YAML preservation. The workspace now matches that
destination shape. Implemented versus pending:

| implemented | not implemented |
| --- | --- |
| `springleaf-atproto` — identity resolution, XRPC client, app-password sessions (public reads, putRecord, blob upload), offline-tested | document model, MDX parsing/keying, editor fact graph/Yrs |
| `springleaf-records` — generated pinned Leaflet lexicon record types, validation, wire-shape and URL-derivation helpers, fixture round-trip tests | native saves/publication, imports/pulls and local file inventory |
| `springleaf-web` — cookie-authenticated `/home` discovery and `get_leaflet_data` inspection, raw response retention | web writes and editor-content fetching |
| `springleaf-workflow` + CLI — credential resolution, `remote list [--state draft\|published]`, `remote show LOCATOR`, human/JSON reports | all other CLI operations honestly report not-implemented, exit 1 |

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
                read-only discovery/metadata implemented; writes pending
    records/   springleaf-records: generated pinned lexicon record types,
               validation, wire-shape helpers — see LEAFLET-PIN.md
  atproto/     springleaf-atproto: hand-rolled ATProto surface — handle→DID→PDS
               identity resolution, XRPC client, sessions, public reads
  workflow/    springleaf-workflow: local files, config, inventory,
                remote read/config implemented; local orchestration pending
  cli/         springleaf (binary): thin Clap/rendering layer;
                remote list/show implemented, other commands pending
design/        architecture, accepted interface, and format designs
experiments/   protocol probes and evidence (tracked, load-bearing)
```

Dependency direction (accepted design §2): pure document model →
MDX adapter / editor / record projections; `web` owns effects;
`workflow` composes the libraries; the CLI stays thin. The scaffold
adds no crate dependency edges that no code uses yet.

## Remote reads

```sh
cargo run -p springleaf -- remote list --state draft
cargo run -p springleaf -- remote list --json
cargo run -p springleaf -- remote show 01999000-0000-7aaa-8000-000000000001
cargo run -p springleaf -- remote show https://leaflet.pub/01999000-0000-7aaa-8000-000000000001 --json
```

The examples use a synthetic UUID; replace it with a real permission-token ID.
Credentials: `--token` > `LEAFLET_AUTH_TOKEN` >
`$XDG_CONFIG_HOME/springleaf/auth-token` (else
`~/.config/springleaf/auth-token`). Prefer env/file to keep tokens out of
shell history and process listings. Tokens are trimmed; explicit empty
credentials and unreadable/empty files are errors. A missing file permits
anonymous requests, which may fail authentication. There is no login or TOML
configuration implementation in this slice.

`--base-url` > `LEAFLET_BASE_URL` > `https://leaflet.pub`; overrides must be
HTTP(S) origins without credentials, path prefixes, query or fragment.
`show` accepts a UUID or a native `/<uuid>` editor URL on that same origin.
Document AT-URI resolution is explicitly unsupported for now.

`remote list` defaults to all **discovered** Leaflets. Publication association
without a published document is still `draft`; direct publication/standalone
document linkage is `published`. Coverage is always reported as **unknown**:
only the observed homepage/contributor payload is read; account-wide
completeness and pagination are unverified. Root `list` remains a separate,
unimplemented **local** inventory command.

`show` inspects metadata, reported rights and linkage—not editor facts, MDX
import or a proven ability to write. JSON reports have `version: 1` and
`outcome: observed`; list items retain `raw_home_rows`, and show retains the
complete RPC result under `data.raw`, including unknown current fields.
Treat these outputs as private. Errors go to stderr, exit 1; no success JSON
is emitted on failure. Requests never follow redirects with the auth cookie.

Implementation receipt and next seam:
[`design/remote-reads0.gpt61s.md`](design/remote-reads0.gpt61s.md).

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
- Implemented read-only slice: [`design/remote-reads0.gpt61s.md`](design/remote-reads0.gpt61s.md)
- Experiment reading map and evidence protocol: [`experiments/README.md`](experiments/README.md)

Never commit credentials or real response bodies; see
[`experiments/README.md`](experiments/README.md).
