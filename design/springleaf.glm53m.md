---
type: Design
title: Springleaf — a Leaflet client for agents and the CLI
description: Architecture, protocol map, and staged build plan for a Rust multi-crate CLI that lists drafts, reads draft internals, and publishes posts to leaflet.pub.
resource: design/springleaf.glm53m.md
status: draft
generated:
  at: 2026-11-07
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e4ceef2e9ffeMaBQB1Yc3bcyMD"
    title: "Session that mapped the Leaflet posting/draft surface, extracted auth, and probed the home listing"
  - resource: "file:///home/rektide/r/atproto/leaflet.md"
    title: "Lead research: social-cli, noteleaf, forest, pai posting patterns"
  - resource: "file:///home/rektide/r/atproto/leaflet-drafts.md"
    title: "Leaflet draft system: fact graph, Replicache sync, publish flow"
---

# Springleaf — a Leaflet client for agents and the CLI

## Purpose

Post content to [leaflet.pub](https://leaflet.pub) from an agent and from the
CLI. Three concrete capabilities, in order: **get the list** (of the user's
drafts), **get the draft** (read one draft's internals), **build a post**
(construct and publish Leaflet records). Draft *authoring* (creating/editing
drafts headlessly) is a later stage.

## Ground truth (verified 2026-11-06)

Sources: `~/archive/hyperlink-academy/leaflet` @ `600c7b5f`,
`~/archive/letta-ai/social-cli` @ `05b661b`, and the two research notes in
`file:///home/rektide/r/atproto/`.

### Auth

- **`auth_token` cookie** (httpOnly, UUID, ~365d): authenticates the web RPCs
  and server actions as the user. Extracted to `~/.config/leaflet/auth-token`
  (mode 600) via a logged-in browser session; verified working from curl.
- **App-password session** (`com.atproto.server.createSession`): the only
  programmatic write path for PDS records (`putRecord`/`uploadBlob`). Not yet
  provisioned; the user supplies creds when needed
  (`ATPROTO_HANDLE` / `ATPROTO_APP_PASSWORD` / `ATPROTO_PDS`, social-cli
  convention).
- `pub.leaflet.authFullPermissions` OAuth permission set exists as the
  non-personal-CLI alternative; out of scope for v0.

### Draft list (no dedicated RPC)

- The homepage leaflet list comes from the `getIdentityData` server action,
  rendered server-side into `/home`. Headless discovery = fetch `/home` with
  the cookie and parse: leaflet links are `href="/<uuid>"` (UUIDv7 permission
  token ids, newest-first); the RSC flight payload (`self.__next_f.push`)
  embeds `permission_token_on_homepage` structures.
- Per-draft enrichment exists as a real RPC: `POST /api/rpc/get_leaflet_data`
  `{token_id}` → entity set, rights, publication linkage, documents.
  `/api/rpc/*` are unauthenticated JSON POSTs; the cookie gates identity.
- Also available: `POST /api/rpc/getFactsForRoots`, `/api/rpc/pull`
  (Replicache PullRequestV1, scoped by token id).

### Post records (two live generations)

- Current: `site.standard.document` (rkey tid; `site`, `title`,
  `publishedAt`, `path`, optional `content` as `pub.leaflet.content` union
  member). Static pages: `site.standard.publication` + publicationPage.
- Legacy (still ingested): `pub.leaflet.document` (pages inline).
- **Slug rule** (social-cli, load-bearing): default rkey = slug; Leaflet's
  middleware rewrites `{domain}/{slug}` → `/lish/{did}/{pub-rkey}/{slug}` and
  looks up by rkey. URL derivation: publication record's `url` field, else
  `https://leaflet.pub/lish/{did}/{pubRkey}/{rkey}`.
- `putRecord` (upsert), not `createRecord`. Images via `uploadBlob`;
  top-level `blobs[]` mirroring for GC safety is unverified — gate images on
  verification.
- Reference implementation: `~/archive/letta-ai/social-cli/src/commands/publish.ts`.
  Markdown converter reference: `~/archive/desertthunder.dev/noteleaf/internal/public/convert.go`.

## Architecture

Rust, single workspace, domain-grouped crates. Package names carry the
registry prefix; directories do not.

```
springleaf/
  crates/
    atproto/   # springleaf-atproto: XRPC client, app-password sessions,
               #   DID/handle→PDS resolution, records, blobs. Generic ATProto.
    leaflet/   # springleaf-leaflet: GENERATED record types (jacquard-codegen
               #   from pinned leaflet lexicons; committed code = the pin) +
               #   markdown⇄blocks converter + slug/URL rules
    drafts/    # springleaf-drafts: web surface — cookie auth, /home discovery,
               #   get_leaflet_data, facts/pull readers; (later) push mutations
    cli/       # springleaf: the binary — clap commands, config, output formats
  design/      # this document + staged findings promoted from experiments/
  experiments/ # protocol probes and evidence; see experiments/README.md
```

Dependency direction: `cli → drafts, leaflet, atproto`; `leaflet → atproto`
(types only where needed); `drafts` is standalone HTTP + cookie.

### Library choices

| Concern | Crate | Note |
|---|---|---|
| HTTP | `reqwest` (rustls, json) | |
| CLI | `clap` v4 derive (+ `clap_complete`) | |
| Errors | `thiserror` (libs), `anyhow` (cli) | |
| Time | `jiff` | |
| Diagnostics | `tracing` + `tracing-subscriber` | |
| Home listing | RSC flight parse (no DOM crate) | experiment 01: one GET, concat `self.__next_f.push` chunks, brace-match `permission_token_on_homepage`; href set kept as drift alarm only. If DOM scraping ever returns: `scraper` / `dom_smoothie` / `lol_html` |
| Lexicon types | `jacquard-codegen` (dev-time) + `jacquard-common`/`jacquard-lexicon` (runtime) | see "Updates" — verdict (b) from experiments/04: generate typed site.standard.* + pub.leaflet.* modules from the pinned leaflet checkout; committed generated code IS the pin |
| IDs | `uuid` | token ids are UUIDv7; sortable by creation |
| Tests | `cargo-nextest`; `wiremock` for HTTP | synthetic fixtures only — never commit real home HTML or tokens |

Config: env + flag + `~/.config/leaflet/auth-token` resolution in `cli` for
v0; adopt `figment2` when settings grow. Rust edition 2024. Version 0.1.0
across all crates, shared.

### CLI surface (v0)

```
springleaf auth status                     # verify cookie: GET /home, report identity state
springleaf drafts list [--json] [--fetch]  # home → ids (+titles via get_leaflet_data with --fetch)
springleaf drafts show <token-id> [--json] # one draft: title, rights, publication, documents
springleaf pubs list [--json]              # listRecords publications (public, no auth)
springleaf publish build <file.md> --publication <at-uri> [--dry-run is default]
                                           # prints the exact record JSON + derived URL
springleaf publish run <file.md> ...       # putRecord; refuses without explicit --yes
```

Agent-contract conventions (from social-cli): `--json` output, meaningful
exit codes, dry-run prints the exact record, no ledger/state machinery.
Token values are never logged or printed.

## Experiments policy

Protocol probes live under `experiments/<nn-topic>/` (tracked, unlike the
`.test-agent/` convention, because protocol evidence is load-bearing here).
Hard rules: **no credentials in experiments** (read tokens from
`~/.config/leaflet/auth-token` at runtime), synthetic data in committed
fixtures, findings promoted into `design/` when load-bearing.

## Stages

0. **This document** (done).
1. **Experiments wave** (parallel, read-only): draft-list extraction,
   draft-internals mapping, post-record construction. See
   `experiments/*/REPORT.md`.
2. **Build: `drafts list`/`show` + `auth status`** — workspace scaffold
   (remove the empty TS placeholders `package.json`/`springleaf.ts` from the
   init commit), `drafts` + `cli` crates against the experiment findings.
3. **Build: `atproto` + `leaflet` crates, `pubs list`** — public reads,
   record types, live verification.
4. **Build: `publish build`/`run`** — markdown→blocks converter (start at
   social-cli's subset: paragraphs, ATX headings, images, link/bold/italic/
   code facets with UTF-8 byte offsets), slug/rkey + URL derivation,
   dry-run first. Live write only after operator go-ahead.
5. **Later: draft authoring** — Replicache push mutations + minimal Yjs
   text encoding; publish-button automation is explicitly out of scope for
   the CLI (Playwright stays an ops-side tool).

## Risks

- **Scrape fragility**: `/home` HTML/RSC shapes are unversioned; mitigation
  is minimal scrape surface (ids only) + RPC enrichment.
- **Undocumented mutations** (stage 5): the push vocabulary and Yjs encoding
  are the hard part; experiments/02 must scope it before any build.
- **Lexicon drift**: record shapes move (noteleaf's dead
  `pub.leaflet.document.draft` is the cautionary tale). Pinning strategy
  (revised per experiments/04): regenerate `crates/leaflet` types with
  `jacquard-codegen` from a SHA-pinned leaflet checkout; the committed
  generated code is the pin. Regeneration is a deliberate, reviewed stage.
- **Blob GC**: verify top-level `blobs[]` mirroring semantics before
  shipping images. The jacquard-generated `pub.leaflet.content` schema
  itself documents the mirroring rule (experiments/04), which partially
  answers this — verification still required.
- **Jacquard dependency risk**: high quality but 97% single-author (Orual),
  silent ~4 months as of 2026-11, and 0.11→0.12 was a wholesale breaking
  rewrite. Verdict (b) limits exposure: codegen at dev time, small runtime
  deps, exact version pins, thin hand-rolled `atproto` crate preserved.
- No time estimates anywhere in this plan, by policy.

## Updates

- **2026-11-07, after experiments/01 + 04**: home listing switched from
  HTML scraping to RSC-flight parsing (no DOM crate in v0; see
  `experiments/01-draft-list/REPORT.md`). Lexicon strategy switched from
  hand-written serde + JSON pinning to jacquard codegen with committed
  generated code as the pin — verdict (b), lexicon toolchain only, thin
  `atproto` crate preserved (see `experiments/04-jacquard/REPORT.md`).
  Auth note: jacquard's `FileAuthStore` lacks 0600 hardening — springleaf
  owns its own session-file permissions either way.
