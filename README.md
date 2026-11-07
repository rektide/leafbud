# springleaf

A [leaflet.pub](https://leaflet.pub) client for agents and the CLI, in Rust.
Check auth, list your leaflets (drafts) and their internals, list
publications, build and publish posts as canonical ATProto records, and
sync what has been posted.

Status: **stage 4** — `auth status`, `drafts list`/`show`, `pubs list`
(read-only, live-verified), `publish build` (offline-safe), `publish run`
(writes; the live putRecord is still operator-gated — see
[Publishing for real](#publishing-for-real)), and `sync` (report-only by
default; `--pull` adopts remote records locally). The architecture,
protocol map, and staged plan live in
[`design/springleaf.glm53m.md`](design/springleaf.glm53m.md); the
on-disk record format is specified by
[`design/records-canonical.glm53m.md`](design/records-canonical.glm53m.md)
(the ATProto record *is* the document); the extraction algorithm was
proven by
[`experiments/01-draft-list/REPORT.md`](experiments/01-draft-list/REPORT.md)
and the record builder by
[`experiments/03-post-builder/REPORT.md`](experiments/03-post-builder/REPORT.md)
(the reports supersede the design doc where they conflict).

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
             get_leaflet_data RPC, v1 output schema
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
loopback HTTP responder; publish/sync flows inject a canned put
transport). No test executes a write against any real PDS.

## Commands

```
springleaf auth status                     # GET /home, report did/handle or anonymous
springleaf drafts list [--json] [--fetch]  # leaflet table (or v1 JSON schema)
springleaf drafts show <token-id> [--json] # one leaflet via get_leaflet_data
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
```

- `drafts list` parses the `/home` RSC flight payload in one request.
  `--json` emits schema v1 verbatim (see experiments/01 §5); `--fetch`
  additionally enriches each row via `get_leaflet_data`
  (`source: home-rsc+rpc`).
- `drafts show --json` emits the raw `get_leaflet_data` result data; the
  human rendering covers title, rights, entity set, publication linkage,
  published documents, and state.
- `pubs list` needs no leaflet auth: it resolves `--handle` (a handle or
  a full DID; default: `handle` from springleaf.toml in the XDG config
  dir, hand-parsed — no toml crate yet), then reads
  `site.standard.publication` + `pub.leaflet.publication` from the PDS
  via `listRecords`. The record decoder tries the jacquard-generated
  types first and falls back to raw field extraction (third-party
  publications are not guaranteed lexicon-clean).
- Exit codes: `0` ok, `1` operational error (anonymous `auth status`
  counts), `2` usage error or an intentional refusal (`publish run` /
  `sync --force` without `--yes`).

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
- Root resolution: `--root` > the `workspace` key in the XDG config
  `springleaf.toml` (the only place it can be found before the workspace
  is known) > cwd. `drafts/` and `posted/` are created on demand.
- `publish build` needs a publication target: `--publication <at-uri>`
  or `publication` in the workspace `springleaf.toml`. Passing the flag
  also fetches the publication record (one public read) to derive the
  publication-domain URL; otherwise the fallback
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

## Auth model

Two surfaces, never mixed:

- **leaflet.pub web cookie** (`auth status`, `drafts …`): the
  `auth_token` cookie value resolved from `--token` >
  `$LEAFLET_AUTH_TOKEN` > `~/.config/leaflet/auth-token` (one line, mode
  600 recommended).
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
