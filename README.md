# springleaf

A [leaflet.pub](https://leaflet.pub) client for agents and the CLI, in Rust.
Read-only today: check auth, list your leaflets (drafts), inspect one draft's
internals. Publishing is a later stage.

Status: **stage 2** — `auth status`, `drafts list`, `drafts show` are live
against production. The architecture, protocol map, and staged plan live in
[`design/springleaf.glm53m.md`](design/springleaf.glm53m.md); the extraction
algorithm implemented here was proven by
[`experiments/01-draft-list/REPORT.md`](experiments/01-draft-list/REPORT.md)
(the report supersedes the design doc where they conflict — e.g. the RSC
flight-payload parse replaced HTML scraping, so no `scraper` dependency).

## Layout

```
crates/
  drafts/    springleaf-drafts: cookie-authed web surface — /home RSC parse,
             get_leaflet_data RPC, v1 output schema
  cli/       springleaf: the binary (clap commands, token resolution, output)
  atproto/   springleaf-atproto: stub until stage 3 (XRPC, sessions, records)
  leaflet/   springleaf-leaflet: stub until stage 3 (record types, markdown)
design/      architecture + canonical record format
experiments/ protocol probes and evidence (tracked, load-bearing)
```

## Build and run

```
cargo build
cargo run -p springleaf -- auth status
cargo run -p springleaf -- drafts list
cargo run -p springleaf -- drafts show <token-id>
```

Checks: `cargo fmt`, `cargo clippy --workspace --all-targets`,
`cargo nextest run`. Unit tests are offline, against the committed synthetic
fixtures under `experiments/`.

## Commands

```
springleaf auth status                     # GET /home, report did/handle or anonymous
springleaf drafts list [--json] [--fetch]  # leaflet table (or v1 JSON schema)
springleaf drafts show <token-id> [--json] # one leaflet via get_leaflet_data
```

- `drafts list` parses the `/home` RSC flight payload in one request. `--json`
  emits schema v1 verbatim (see experiments/01 §5); `--fetch` additionally
  enriches each row via `get_leaflet_data` (`source: home-rsc+rpc`).
- `drafts show --json` emits the raw `get_leaflet_data` result data; the human
  rendering covers title, rights, entity set, publication linkage, published
  documents, and state.
- Exit codes: `0` ok, `1` operational error (anonymous `auth status` counts),
  `2` usage error.

## Auth model

The `auth_token` cookie value (extracted from a logged-in browser session)
authenticates against leaflet.pub. Resolution order:

1. `--token <value>`
2. `$LEAFLET_AUTH_TOKEN`
3. `~/.config/leaflet/auth-token` (one line, mode 600 recommended)

The token is never printed, logged, or embedded in error messages — including
at `-v`/debug verbosity. `--base-url` (or `$LEAFLET_BASE_URL`) selects a
different instance; default `https://leaflet.pub`.

## Notes and known edges

- The homepage list may be paginated at the server (30 observed, cap
  unverified); `href_payload_mismatch` in the list output alarms on any drift
  between the DOM card hrefs and the payload rows.
- Published-URL derivation knows `site.standard.*` at-uris; a linkage to a
  legacy `pub.leaflet.publication` yields an empty `published_url` under the
  v1 rule (seen in experiments/02 fixtures; revisit at stage 3).
- Never commit credentials or real response bodies; see
  [`experiments/README.md`](experiments/README.md).
