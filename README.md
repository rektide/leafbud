# springleaf

A [leaflet.pub](https://leaflet.pub) client for agents and the CLI, in Rust.
Check auth, list your leaflets (drafts) and their internals, list
publications, build and publish posts as canonical ATProto records, and
sync what has been posted.

Status: **stage 5** — `auth status`, `drafts list`/`show`/`export`
(read-only, live-verified), `pubs list`, `publish build` (offline-safe),
`publish run` (writes; the live putRecord is still operator-gated — see
[Publishing for real](#publishing-for-real-operator-gated)), `sync` (report-only by
default), `config show/set` for the layered `springleaf.toml` files
(figment2 layering — see
[Configuration](#configuration-springleaftoml-layering)), and the
extraction ⇄ authoring pair complete:
`drafts export` pulls a draft into a canonical record, `drafts author`
builds (and, operator-gated, pushes) the reverse mutation batch, and
`drafts create` (operator-gated) mints the fresh draft via `GET /new`
— see
[Authoring into leaflet drafts](#authoring-into-leaflet-drafts-operator-gated).
The architecture, protocol map, and staged plan live in
[`design/springleaf.glm53m.md`](design/springleaf.glm53m.md); the
on-disk record format is specified by
[`design/records-canonical.glm53m.md`](design/records-canonical.glm53m.md)
(the ATProto record *is* the document); the `page.parts` federated
mention services that leaflet's editor consumes are researched in
[`design/page-parts.glm53m.md`](design/page-parts.glm53m.md); the extraction
algorithm was proven by
[`experiments/01-draft-list/REPORT.md`](experiments/01-draft-list/REPORT.md)
and the record builder by
[`experiments/03-post-builder/REPORT.md`](experiments/03-post-builder/REPORT.md)
(the reports supersede the design doc where they conflict).

For the proposed local-MDX authoring redesign, see the
[experiment reading map](experiments/README.md) and
[current recommendation with alternatives](design/authoring-recommendation0.gpt61s.md).
The [preliminary design and implementation handoff](design/authoring-handoff0.gpt61s.md)
connects those findings to the proposed production boundaries and landing plan.
The [interface finalization proposal](design/authoring-interface0.gpt6a.md)
narrows the remaining CLI, domain-grouped crate, and YAML decisions using the
handoff's keyed bidirectional direction.
Those are experimental/proposed directions, not changes to the implemented
command surface below.

## Layout

```
crates/
  atproto/   springleaf-atproto: thin hand-rolled ATProto surface —
             handle→DID→PDS identity resolution (DNS TXT + well-known +
             PLC/web DID docs), XRPC client, app-password sessions
             (putRecord/uploadBlob writes + XDG session store with 0600
             permissions), listRecords/getRecord public reads
  leaflet/   springleaf-leaflet: jacquard-generated record types
             (committed code = the lexicon pin; see LEAFLET-PIN.md) +
             markdown→blocks converter + record builder + slug/URL rules
             + publication-record lookup + record validation
  drafts/    springleaf-drafts: cookie-authed web surface — /home RSC parse,
             get_leaflet_data/pull/push RPCs, yjs text codec + fractional
             keygen, extraction ⇄ authoring converters, /new draft minting,
             v1 output schema
  cli/       springleaf: the binary (clap commands, token resolution, output)
design/      architecture + canonical record format
experiments/ protocol probes and evidence (tracked, load-bearing)
```

## Build and run

```
cargo build
cargo run -p springleaf -- auth status
cargo run -p springleaf -- drafts list
cargo run -p springleaf -- publish build post.md --publication <at-uri>
```

Checks: `cargo fmt`, `cargo clippy --workspace --all-targets`,
`cargo nextest run`. Unit tests are offline, against the committed
synthetic fixtures under `experiments/` (wire tests use a hand-rolled
loopback HTTP responder; publish/sync/authoring flows inject canned
write transports). No test executes a write against any real PDS or
draft.

## Commands

```
springleaf auth status                     # GET /home, report did/handle or anonymous
springleaf drafts list [--json] [--fetch]  # leaflet table (or v1 JSON schema)
springleaf drafts show <token-id> [--json] # one leaflet via get_leaflet_data
springleaf drafts create [--yes] [--json]  # mint a fresh draft via GET /new
                                           #   (harvests the token id from
                                           #   the 307 Location; exit 2
                                           #   without --yes)
springleaf drafts export <token-id> [--root DIR] [--slug S] [--overwrite]
                                           #   read-only pull → canonical record
                                           #   in drafts/<slug>.json
springleaf drafts author <token-id> --from <slug-or-file> [--root DIR]
                                           #   record → push mutation batch
                                           #   (prints it, exit 2); --yes pushes
                                           #   into the draft (operator-gated)
springleaf pubs list [--json] [--handle H] # publications via listRecords (public)
springleaf publish build <file.md> [--publication <at-uri>] [--root DIR]
                                           #   build record -> drafts/<slug>.json
                                           #   [--overwrite] [--json]; always safe
springleaf publish run <slug-or-file> [--yes] [--json]
                                           #   putRecord (upsert) -> move to
                                           #   posted/ + sidecar; exit 2 without --yes
springleaf sync [--pull] [--force] [--yes] [--json] [--handle H]
                                           #   report posted/ vs the PDS (default:
                                           #   report-only, no writes)
springleaf config show [--json]            # the effective layered config + which
                                           #   file each value came from
springleaf config set <key> <value>        # write one key (workspace|handle|pds|
                                           #   publication) to the global toml,
                                           #   comments preserved; --workspace-file
                                           #   targets the workspace toml instead
```

- `drafts list` parses the `/home` RSC flight payload in one request.
  `--json` emits schema v1 verbatim (see experiments/01 §5); `--fetch`
  additionally enriches each row via `get_leaflet_data`
  (`source: home-rsc+rpc`).
- `drafts show --json` emits the raw `get_leaflet_data` result data; the
  human rendering covers title, rights, entity set, publication linkage,
  published documents, and state.
- `pubs list` needs no leaflet auth: it resolves `--handle` (a handle or
  a full DID; default: `handle` layered from the workspace and global
  `springleaf.toml` — see
  [Configuration](#configuration-springleaftoml-layering)), then reads
  `site.standard.publication` + `pub.leaflet.publication` from the PDS
  via `listRecords`. The record decoder tries the jacquard-generated
  types first and falls back to raw field extraction (third-party
  publications are not guaranteed lexicon-clean).
- `drafts export` (read-only): one `pull` + `get_leaflet_data` per run,
  writes the pure canonical record to `drafts/<slug>.json`. Exported
  records with no publication linkage carry the standalone
  `site: https://leaflet.pub/p/{did}` form — `publish run` accepts both
  site forms (the standalone URL derives as
  `https://leaflet.pub/p/{did}/{rkey}`).
- `drafts author` is the reverse direction: it validates the record,
  resolves the target draft (`get_leaflet_data` for the root entity +
  write right + entity set, one read-only `pull` for the start page,
  existing block count, and append seeds), then builds the push batch —
  `addBlock` + `assertFact` per block with caller-minted uuid v7 ids,
  fractional append positions from the draft's last key, yjs-encoded
  text (byte-parity with the committed encoder reference), the
  `updateLeafletMetadata` title tail. Poll blocks resolve their option
  texts from the pollRef's PDS record (a public read); image/gallery/
  drawing/embedded-canvas blocks warn and skip (v0).
- Exit codes: `0` ok, `1` operational error (anonymous `auth status`
  counts), `2` usage error or an intentional refusal (`publish run`,
  `sync --force`, `drafts create`/`drafts author` without `--yes`, and
  `config set`'s invalid routing — empty value, or `workspace` +
  `--workspace-file`).

## Workspace: drafts/ and posted/

`publish` and `sync` operate on a workspace per
[`design/records-canonical.glm53m.md`](design/records-canonical.glm53m.md):

```
<root>/
  drafts/               unposted canonical records (<slug>.json, pure)
  posted/               records as put (<slug>.json + <slug>.meta.json)
  springleaf.toml       identity defaults: handle, pds, publication
```

- The record files are **pure** ATProto record JSON — no frontmatter, no
  local fields — exactly what `putRecord` writes. All local state lives
  in the sidecar (`rkey`, `uri`, `cid`, `publication`, `publishedAt`,
  `lastPutAt`, `contentHash` sha256 of the record file, `remoteCid`).
- Root resolution: `--root` > the global `workspace` key (see
  [Configuration](#configuration-springleaftoml-layering) — the only
  place it can be found before the workspace is known) > cwd. Relative
  `workspace` values resolve against the config file's directory.
  `drafts/` and `posted/` are created on demand.
- `publish build` needs a publication target: `--publication <at-uri>`
  or `publication` in the workspace or global `springleaf.toml`
  (workspace wins — see
  [Configuration](#configuration-springleaftoml-layering)). Passing the
  flag also fetches the publication record (one public read) to derive
  the publication-domain URL; otherwise the fallback
  `https://leaflet.pub/lish/{did}/{pubRkey}/{rkey}` derives offline.
- `publish run` moves the record to `posted/` — a slug lives in exactly
  one place. Re-running over a posted record is an idempotent re-put
  (upsert) with a hash check against the sidecar; changed bytes warn and
  re-put.
- `sync` (report-only by default) reconciles `posted/` against
  `site.standard.document` `listRecords` by rkey: **clean**,
  **local-newer** (hash changed, cid unchanged → `--force --yes`
  re-put), **remote-drift** (hash unchanged, cid changed → `--pull`
  adopts the remote bytes or `--force --yes` re-puts), **true-conflict**
  (both changed → blocked), **remote-orphan** (remote only → `--pull`
  adopts into posted/), **local-missing** (posted but absent remotely →
  re-put), and **no-sidecar** (indeterminate → re-put writes one).
  `--pull` is a local write only; nothing is ever deleted.

## Configuration (springleaf.toml layering)

Config is real toml parsed by [figment2](https://docs.rs/figment2) —
stacked providers, not per-key fallback chains. Every
`springleaf.toml` is a flat key set (`workspace`, `handle`, `pds`,
`publication`; unknown keys ignored); two files can supply them:

| layer | file |
| --- | --- |
| workspace | `<workspace-root>/springleaf.toml` |
| global | `$XDG_CONFIG_HOME/springleaf/springleaf.toml` (else `~/.config/springleaf/springleaf.toml`) |

Precedence per key (higher wins):

| key | resolution order |
| --- | --- |
| `publication` | `--publication` flag > workspace toml > global toml |
| `handle` | `--handle` flag > workspace toml > global toml > `$ATPROTO_HANDLE` (sync's final fallback only; `pubs list` skips the env) |
| `workspace` | `--root` resolves the root directly; else the global toml key; else cwd |

A workspace value overriding the global one is the point: a workspace
is its own publication context. `pubs list` resolves its default
handle through the same workspace + global layers (workspace found via
the global `workspace` key), so every command shares one identity
default.

`config` works on these files directly: `config show` renders the
effective layered values and which file each came from (the workspace
is resolved the usual way — global key, else cwd); `config set
<key> <value>` writes one key via toml_edit — comment- and
sibling-preserving, creating the file when absent, refusing malformed
files rather than rewriting them. Keys default to the global toml;
`--workspace-file` targets the resolved workspace's toml instead
(valid for `handle`/`pds`/`publication` — the `workspace` key
necessarily lives in the global file). Figment is read-only; the write
path is that separate, deliberate half.

**A malformed config is a loud error.** An absent file or key is
simply unset, but a present `springleaf.toml` that fails to parse — or
types a known key wrongly (`handle = 42`) — aborts with the file path
and the parser's error instead of silently falling back. (The old hand
parser never failed; a typo'd config silently shadowing defaults is a
footgun in a publishing tool, so this change is deliberate.) An
explicitly empty value (`handle = ""`) counts as unset. Relative
`workspace` values resolve against the config file's directory.

## Auth model

springleaf's config lives under one XDG dir —
`$XDG_CONFIG_HOME/springleaf/` (else `~/.config/springleaf/`):
`springleaf.toml` (identity defaults, the `workspace` root key —
layering per the [Configuration](#configuration-springleaftoml-layering)
section), `auth-token` (web cookie), and `session.json` (app-password
session).

Two surfaces, never mixed:

- **leaflet.pub web cookie** (`auth status`, `drafts …`): the
  `auth_token` cookie value resolved from `--token` >
  `$LEAFLET_AUTH_TOKEN` > `$XDG_CONFIG_HOME/springleaf/auth-token`
  (else `~/.config/springleaf/auth-token`; one line, mode 600
  recommended).
- **ATProto app-password session** (`publish run`, `sync --force`):
  `ATPROTO_HANDLE` + `ATPROTO_APP_PASSWORD` (an app password, not the
  account password), `ATPROTO_PDS` optional (default
  `https://bsky.social`). Sessions persist to
  `$XDG_CONFIG_HOME/springleaf/session.json` (file 0600, parent 0700)
  and are reused while the access JWT is live, refreshed at the issuing
  PDS when expired, else re-created.

Secrets of either kind are never printed, logged, or embedded in error
messages — including at `-v`/debug verbosity. `--base-url` (or
`$LEAFLET_BASE_URL`) selects a different leaflet instance; default
`https://leaflet.pub`.

## Authoring into leaflet drafts (operator-gated)

Two **writes against leaflet.pub** live behind the `drafts` commands,
both deliberately gated, exactly like `publish run` — neither has been
executed live from this codebase; both flows are mock-verified end to
end:

- `drafts create --yes` — one authenticated `GET /new`: the server runs
  its creation action (a server-action-only path no Replicache mutation
  covers, per `experiments/02-draft-internals/REPORT.md` §3) and
  answers 307 whose Location header carries the fresh token id.
- `drafts author --yes` — a Replicache push through
  `POST /api/rpc/push`.

1. **Mint a throwaway draft**: `springleaf drafts create` prints exactly
   what the probe will do and exits 2; re-run with `--yes` to mint and
   print the fresh token id. The `/new` Location shape is
   source-verified, not live-captured — this is its acceptance probe,
   and the mint happens even with a missing/invalid token (an orphan
   draft nobody lists), so check `springleaf auth status` first. Or
   create one in the web UI and grab its token id from
   `springleaf drafts list`.
2. **Dry-run** (default, always safe — read-only RPCs only):
   `springleaf drafts author <token-id> --from <slug-or-file>`
   prints the exact mutation batch — one line per mutation (name,
   entity ids, positions), the block summary, and the warn list — and
   exits 2 without touching the transport. A non-empty target draft is
   loudly warned: **authoring appends after existing blocks; nothing is
   replaced or removed** (overwriting is a deliberate v0 non-goal).
3. **Review** the batch: block types, the yjs text sizes, the appended
   positions, and any warns (skipped blocks name their reason).
4. **Execute**: re-run with `--yes`. The push runs under a fresh
   clientID with mutation ids from 1 (replay-safe: the server skips
   `id <= lastMutationID`), so a retry of the same run is idempotent —
   but a second `--yes` invocation of the same record *appends the
   content again* under its new ids.
5. **Verify**: `springleaf drafts export <token-id> --slug <check-slug>`
   re-pulls the draft and writes its canonical record; diff it against
   the source record (entity ids and positions differ — compare titles,
   block sequence, text, and facets).

Known v0 edges: `\n` inside authored text encodes as a literal newline
(decodes back identically; the editor may render it differently than an
authored hard break); bskyPost blocks carry a minimal
`{post:{uri,cid}}` thread blob (the editor's rich render may be
degraded); text alignment facts are not authored; the record
`questions`/`reply` `key` is draft-local and not re-minted.

## Publishing for real (operator-gated)

No write has been executed against any PDS from this codebase. The live
`putRecord` is deliberately gated:

1. **Create an app password** in the account's Bluesky settings
   (Settings → App passwords). Never use the account password.
2. **Prepare the record** (safe, offline):
   `springleaf publish build post.md --publication at://…/site.standard.publication/<rkey> --root <workspace>`
   and review the printed record, target uri, and derived URL.
   `publish run <slug>` without `--yes` prints the exact bytes that
   would be put and exits 2 — use it for the final check.
3. **Execute** with the env + flag:
   `ATPROTO_HANDLE=<handle> ATPROTO_APP_PASSWORD=<app-password> springleaf publish run <slug> --yes`
   (`ATPROTO_PDS` optional). The command validates the record, resumes
   or creates the session (verifying the session did matches the
   record's publication owner), puts the record as an upsert, moves it
   to `posted/<slug>.json`, and writes the sidecar. Afterwards
   `springleaf sync` should show the row as clean.
4. The pre-existing probe script
   [`experiments/03-post-builder/putRecord.sh`](experiments/03-post-builder/putRecord.sh)
   (bash equivalent of step 3, pinned to the experiment's frozen record)
   remains **unexecuted** and requires `GATED_OPERATOR_APPROVAL=yes` —
   kept as the independent, non-Rust cross-check for the same gated
   step.

Record-only publishing skips leaflet.pub's server-side side effects
(newsletter notifications, optional bsky announcement,
`leaflets_in_publications` linkage) — known and accepted per the
canonical-format design doc.

## Notes and known edges

- The homepage list may be paginated at the server (30 observed, cap
  unverified); `href_payload_mismatch` in the list output alarms on any
  drift between the DOM card hrefs and the payload rows.
- `drafts create`'s `/new` Location shape
  (`/<token-id>?focusFirstBlock&addToHome`, 307) is verified against the
  leaflet source (@`600c7b5f`), not a live capture — experiments/02 has
  no `/new` fixture. The parser accepts any 3xx whose first path segment
  is a uuid (relative or absolute Location); the first gated `--yes` run
  is the acceptance probe.
- `drafts author` never destroys: it appends blocks after the target
  draft's last position and subpages after its last `root/page` key.
  Replacing draft content would take `removeBlock`/`retractFact`
  sequences — a deliberate later step, not v0.
- Published-URL derivation knows `site.standard.*` at-uris; a linkage to
  a legacy `pub.leaflet.publication` yields an empty `published_url`
  under the v1 rule (seen in experiments/02 fixtures; `pubs list` now
  reads the legacy collection directly — the drafts-list v1 rule is
  unchanged).
- `sync`'s remote-orphan count against a live account (10 documents for
  jauntywunderkind at stage 4 verification) counts
  `site.standard.document` records only — publications themselves do
  not sync.
- Never commit credentials or real response bodies; see
  [`experiments/README.md`](experiments/README.md).
