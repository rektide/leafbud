# springleaf

Local-first MDX authoring for [leaflet.pub](https://leaflet.pub)
documents, in Rust: one MDX file per document, durable identities,
push saves the Leaflet editor, publish releases, pull adopts web edits.

Status: **native draft checkout and layered config, version 0.4.0.** The accepted authoring interface
([`design/authoring-interface0.gpt6a.md`](design/authoring-interface0.gpt6a.md))
settles root CLI verbs (`create`/`check`/`list`/`status`/`diff`/
`push`/`pull`/`publish`/`import`/`edit`/`tidy`/`export`, plus
`keys rename`, `remote`, `pubs`, `auth`, `config`), all-applicable
`tidy` with `--keys` narrowing, eight domain-grouped crates, and
operation-specific YAML preservation. The workspace now matches that
destination shape. Implemented versus pending:

| implemented | not implemented |
| --- | --- |
| `springleaf-atproto` — retained identity resolution, XRPC, app-password sessions; `springleaf-records` — pinned record types/validation/helpers | native saves/publication and generalized managed push/pull |
| `springleaf-web` — cookie-authenticated discovery, metadata and full editor pull; `springleaf-editor` — raw-fact scanning and strict Yrs text decoding | canvas, lists, interactive/other unsupported editor content |
| `springleaf-document` + `springleaf-mdx` — bounded current-document model, durable bindings, real Satteri/YAML parse and loss-checked keyed MDX writer | generalized authoring model, source-preserving updates and full formatting |
| `springleaf-workflow` + CLI — `remote list/show`, draft `import`, imported-profile `check`, Figment2 config show/set and shared workspace root | local `list`, `create`, `status`, `diff`, `push`, `pull`, `publish`, `edit`, `tidy`, `export`, `keys`, `pubs`, `auth` commands |

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
                stable source IDs and compact bindings (bounded import profile)
    mdx/       springleaf-mdx: Satteri parsing, YAML frontmatter,
                loss-checked initial keyed MDX writing and validation
  leaflet/
    editor/    springleaf-editor: fact graph, Yrs text state, bindings,
                bounded pure observation; write planning still pending
    web/       springleaf-web: cookie auth, discovery, RPC/actions,
                read-only discovery/metadata/content pulls; writes pending
    records/   springleaf-records: generated pinned lexicon record types,
               validation, wire-shape helpers — see LEAFLET-PIN.md
  atproto/     springleaf-atproto: hand-rolled ATProto surface — handle→DID→PDS
               identity resolution, XRPC client, sessions, public reads
  workflow/    springleaf-workflow: local files, config, inventory,
                layered config, native reads and guarded draft import
  cli/         springleaf (binary): thin Clap/rendering layer;
                reads/import/check/config implemented; authoring writes pending
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
anonymous requests, which may fail authentication. Login remains unimplemented;
TOML authoring defaults are separate from these native credentials.

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

## Document workspace and publication defaults

```sh
cargo run -p springleaf -- config set workspace /home/rektide/l
cargo run -p springleaf -- config show --json
cargo run -p springleaf -- config set publication AT_URI --workspace-file
```

Workspace root: `--root DIR` > global TOML `workspace` > cwd. Global config is
`$XDG_CONFIG_HOME/springleaf/springleaf.toml` (else
`~/.config/springleaf/springleaf.toml`); relative global workspace paths resolve
from that file's directory. `<root>/springleaf.toml` supplies workspace defaults.
Figment2 resolves explicit flag > unbound document intent > workspace > global
for `handle`, `pds`, `publication`; `config show` reports per-key provenance.
`config set` edits only the chosen key, preserves comments/unknown settings,
and refuses malformed files. `workspace` belongs in the global file.

ATProto remains available in its library; native draft reads/imports use the
cookie web client, **not** the configured PDS. Imports retain the observed
remote publication association (including explicit standalone null), never
retargeting a binding from ambient defaults or inventing publication dates.

## Checkout drafts as editable MDX

```sh
cargo run -p springleaf -- import --all --state draft --root /home/rektide/l
cargo run -p springleaf -- import EDITOR_URL --to drafts/chosen.mdx --json
cargo run -p springleaf -- --root /home/rektide/l check drafts/chosen.mdx
```

This fetches native editor content, unlike metadata-only `remote show`.
Each document is one current `.mdx` file with YAML frontmatter, durable source
keys, native bindings and scoped hashes—no JSON siblings, CRDT history or base
snapshots. Default filenames combine a friendly title with the token UUID;
binding-based discovery recognizes repeated imports and renamed files.

The implemented profile covers linear paragraphs/headings/quotes, strict Yrs
rich-text marks/breaks, exact code text, images with native source/fallback/
dimensions/alt, page links/definitions and watermark metadata. Ordinary prose
uses Markdown; exact rich runs use `Run` components, with explicit typed block
boundaries where Markdown would split or reinterpret them. Code uses an exact
`Code` component. `check` validates this bounded profile, not every destination
MDX feature. Images retain remote URLs; no offline binary asset cache is claimed.

Import verifies sibling-token publication linkage before selecting drafts.
Unsupported content produces a per-document failure, not a truncated success.
Repeated equal imports do not write; local edits are preserved. Remote drift
currently reports a required explicit pull/refresh rather than overwriting an
existing file. Destination collisions, duplicate bindings and unsafe paths
are errors. No unseen files are pruned. Bulk partial failure exits 4.

The commissioned live checkout produced **27 draft MDX files** in `~/l/drafts/`,
with **335 blocks**, from 30 observed Leaflets; 3 published objects were excluded.
No draft failed. A full rerun preserved all 27 files' bytes and modification
times. This is all **observed** drafts: the pinned upstream identity query has
no explicit pagination/limit, but backend/account-wide completeness remains
unverified. No remote save or publication was performed.

Current implementation/verification handoff:
[`design/draft-checkout0.gpt61s.md`](design/draft-checkout0.gpt61s.md).

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
- Config restoration and actual MDX checkout: [`design/draft-checkout0.gpt61s.md`](design/draft-checkout0.gpt61s.md)
- Experiment reading map and evidence protocol: [`experiments/README.md`](experiments/README.md)

Never commit credentials or real response bodies; see
[`experiments/README.md`](experiments/README.md).
