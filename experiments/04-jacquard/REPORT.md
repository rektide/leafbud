---
type: Report
title: "Experiment 04 — Jacquard evaluation for springleaf reuse"
description: What Jacquard is, stack solidity, the lexicon codegen story, auth/session handling, posting capability, and a reuse verdict for springleaf.
resource: experiments/04-jacquard/REPORT.md
status: draft
generated:
  at: 2026-11-07
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e4ca91c62ffejYe7DfZ5gw5uMq"
    title: "Session that evaluated the Jacquard archive checkout, ran the codegen probe against pinned leaflet lexicons, and wrote this report"
  - resource: "file:///home/rektide/src/springleaf/design/springleaf.glm53m.md"
    title: "Springleaf design doc — architecture, stages, and risks this experiment probes"
---

# Experiment 04 — Jacquard evaluation for springleaf reuse

Source evaluated: `~/archive/nonbinary.computer/jacquard` @ `8bf1f4146`
(2026-06-28), upstream `tangled.org/nonbinary.computer/jacquard`, workspace
version **0.12.1**, license MPL-2.0. All citations below are relative to that
checkout root unless prefixed otherwise. No credentials were used; the only
probes were a read-only `cargo check` and an offline codegen run
(`probe-codegen.sh` in this directory, outputs under `/tmp/opencode`).

## 1. What is Jacquard?

A suite of Rust crates for AT Protocol development — "batteries included, but
easily replaceable batteries" (`README.md`). It covers client, auth, identity,
lexicon tooling, and repo primitives end-to-end:

- **`crates/jacquard`** — main crate: `Agent<A: AgentSession>` client,
  `CredentialSession` (app password), OAuth session integration, record-op
  helpers (`AgentSessionExt` in `crates/jacquard/src/client.rs`), re-exports
  the rest.
- **`crates/jacquard-common`** — foundation: validated types (`Did`, `Handle`,
  `Nsid`, `AtUri`, `Tid`, `Cid`, `BlobRef`, ISO-8601 `Datetime` in
  `crates/jacquard-common/src/types/datetime.rs`), the `Data`/`RawData` value
  types, XRPC core traits, and "bootstrap" types for
  `createSession`/`putRecord`/`uploadBlob` etc. so record ops work without the
  generated API crate (CHANGELOG 0.12.0, "Bootstrap XRPC types").
- **`crates/jacquard-api`** — generated bindings from **764 lexicon schemas**
  (`llms.txt`; `AGENTS.md` says 646 — the count drifts with regenerations),
  feature-gated per namespace.
- **`crates/jacquard-derive`** — proc macros: `#[lexicon]` (adds `extra_data`
  capture), `#[open_union]` (adds `Unknown(Data)` variant), derives
  `IntoStatic`, `XrpcRequest`, `LexiconSchema`.
- **`crates/jacquard-oauth`** — full atproto OAuth/DPoP with loopback server,
  scopes/permission sets, nonce retry, token refresh.
- **`crates/jacquard-identity`** — handle→DID and DID→doc resolution
  (DNS TXT, well-known, PLC, PDS XRPC, public API fallback) plus
  `pds_for_did`/`pds_for_handle`.
- **`crates/jacquard-repo`** — MST, commits, CAR I/O, block stores.
- **`crates/jacquard-lexicon` + `crates/jacquard-lexgen`** — the lexicon
  system (see §3). `binaries/` ships prebuilt `lex-fetch` and
  `jacquard-codegen` executables with SHA256SUMS.
- **`crates/jacquard-axum`** — server-side XRPC extractors, service auth.
- Helpers: `lazy-collections`, `mini-moka-wasm`, plus `examples/` (18
  examples incl. `app_password_create_post.rs`, `post_with_image.rs`).

**The headline for springleaf:** Jacquard's `lexicons.kdl` already lists a
**"leaflet" source** at priority 100 (`lexicons.kdl` lines 7–10, fetching
`https://github.com/hyperlink-academy/leaflet`), so `jacquard-api` **already
ships generated bindings for both live record generations**: `site.standard.*`
(`crates/jacquard-api/src/site_standard/`: `document.rs`, `publication.rs`,
themes, graph) and `pub.leaflet.*`
(`crates/jacquard-api/src/pub_leaflet/`: `content.rs`, all 19 `blocks/*`
types, `document`, `publication`, `publicationPage`, `richtext_facet`,
`authFullPermissions`, …), with lexicon JSON snapshots alongside in
`crates/jacquard-api/lexicons/`. Both are reachable via the default feature
chain `default = api_full → jacquard-api/other → site_standard, pub_leaflet`
(`crates/jacquard/Cargo.toml`, `crates/jacquard-api/Cargo.toml` features).

## 2. Stack assessment — "what stack is this, solid?"

**Language/tooling.** Rust, edition 2024 (`Cargo.toml` workspace). Toolchain
pinned to `nightly` (`rust-toolchain.toml`) but **zero `#![feature]` gates**
anywhere in `crates/` (verified by grep) — the pin is soft. It checked clean
on local nightly `1.101.0-nightly (2026-10-13)`, four months newer than the
last commit: `cargo check --release -p jacquard-common -p jacquard-lexicon
-p jacquard-lexgen -p jacquard` finished in 2m48s with one trivial warning
(unused dependency `jacquard-derive` in lexgen). Nix flake + justfile +
pre-commit; CI on Tangled, not GitHub (`.tangled/workflows/build.yml`: wasm
build check, `cargo nextest run --workspace`, doctests). WASM and embedded
targets are supported (`rust-toolchain.toml` targets list; `justfile
check-wasm`).

**Dependencies.** Modern and high-quality: reqwest 0.12, serde/smol_str/
fluent-uri, cid/ipld-core/serde_ipld_dagcbor, p256 + jose-jwk for DPoP,
miette/thiserror, bon builders, chrono (⚠ not jiff), trait-variant. Some
unusual pinned minors (`rand_core 0.6` alongside `rand 0.9`).

**Architecture.** Genuinely sophisticated: the borrow-or-share type system
(`S: BosStr = DefaultStr(=SmolStr)`, GATs on `XrpcResp`) gives zero-copy when
wanted and `DeserializeOwned` by default; validated newtypes everywhere;
layered client (`HttpClient` → `XrpcExt` stateless → `XrpcClient`/`Agent`
stateful). Documented in an excellent `llms.txt` (~1k lines, honest about
common mistakes) and a detailed CHANGELOG.

**Tests.** ~980 test functions across the workspace (rg count), including 50
in `crates/jacquard-lexicon/src/validation/tests.rs`, dedicated
`crates/jacquard-codegen-tests/` with fixtures and roundtrip tests, and
doctests in CI. This is a tested codebase, not a weekend dump.

**Maintenance cadence.** 434 commits, 2025-09-26 → 2026-06-28. Bursty:
237 commits in Oct 2025, near-silence Jan–Feb 2026, pickup Mar–Jun 2026,
**silent for ~4.3 months** as of this writing. Authorship: **Orual 421/434
commits (97%)** — Lyna, Zephyra, and two others account for the rest. Bus
factor is effectively one. Published on crates.io (README claims 117k
downloads; real downstream users listed: Tranquil PDS, Weaver, skywatch,
PDS MOOver, Porxie).

**Solidity verdict.** *High-quality, actively-designed, pre-1.0 with real
breaking-change appetite.* 0.11→0.12 was a wholesale rewrite ("many breaking
changes… borrow-or-share rewrite… far from the only API to have changed" —
README warning banner). Depending on it means: pin exact versions, expect
0.13 to break you, and accept that a single maintainer's availability gates
fixes. The leaflet-relevant surfaces (generated types, record ops, app-password
session) are the library's stable core, which mitigates but doesn't eliminate
this. Solid to *use*; risky to *marry*.

## 3. The lexicon crate (the important question)

**`jacquard-lexicon` is all three: a lexicon definition-format parser, a Rust
code generator, and a runtime validation layer** — plus a reverse direction
(Rust types → lexicon docs) and permission-set types.

- **Parsing** (`crates/jacquard-lexicon/src/lexicon.rs`): explicit fork of
  `atrium-lexicon` (header comment), full `LexiconDoc`/`LexUserType` model.
- **Fetching** (`crates/jacquard-lexgen/src/fetch/`): driven by a KDL config
  (`lexicons.kdl`). Source types: `git`, `http`, `atproto` (from a DID's
  repo), `local` dir, `jsonfile`, `slices`, merged by priority
  (`fetch/config.rs`). Git sources accept an optional `ref` — but implemented
  as `git clone --depth 1 --branch <ref>`
  (`fetch/sources/git.rs` lines 24–31), so **branches/tags only, not commit
  SHAs**. There is **no `--sha` flag** on `lex-fetch` (`src/cli.rs`).
- **Codegen** (`src/codegen/`, binary `src/bin/codegen.rs`): generates
  **Rust only** (no TS): record structs + builders, `Collection` impls +
  `GetRecordOutput` wrappers + `Type::uri()` helpers, XRPC request/output/
  error enums with server-endpoint markers, open unions with
  `Unknown(Data)` catch-all, `extra_data: BTreeMap<SmolStr, Data<S>>` for
  unknown fields, per-namespace Cargo feature gating (auto-updated in
  Cargo.toml when run via `lex-fetch`).
- **Validation** (two layers):
  1. per-type `LexiconSchema::validate()` — generated into every type,
     checks maxLength/**maxGraphemes** (real unicode-segmentation grapheme
     counting), knownValues, formats — see the generated
     `Document::validate()` in
     `crates/jacquard-api/src/site_standard/document.rs`;
  2. `SchemaRegistry`/validator for untyped `Data` against any
     `LexiconDoc` with `ValidationPath` error reporting
     (`crates/jacquard-lexicon/src/validation.rs`).
- **Reverse codegen** (`src/schema.rs`, `#[derive(LexiconSchema)]`): Rust
  types can produce their lexicon doc — leaflet is even the doc example
  ("`pub.leaflet.poll.definition`", `schema.rs` line 94).

**Could springleaf use it to define/pin leaflet's lexicons and get typed
records + validation instead of hand-written serde? Yes — verified live.**
`probe-codegen.sh` in this directory ran the archive's prebuilt
`target/release/jacquard-codegen` against the **SHA-pinned** leaflet checkout
(`~/archive/hyperlink-academy/leaflet` @ `600c7b5f`, the same pin as the
design doc) and generated a complete standalone module tree in seconds:
`site_standard/` (document.rs = 1,026 lines incl. constraint validation),
`pub_leaflet/` (content, all blocks, …), plus `com_atproto` deps. The
generated `pub.leaflet.content` type even encodes the **`blobs[]` top-level
mirroring rule** with its schema description ("the PDS only scans the top
level of a record for blob references when deciding what to garbage-collect…")
— direct input on springleaf's Blob-GC risk item.

**Comparison with the alternatives:**

| | Hand-rolled serde (design default) | noteful `fetch lexicons --sha` | jacquard codegen |
|---|---|---|---|
| Pinning | types are the pin (hand-maintained) | JSON pinned at SHA, types still hand-written | **local source at pinned checkout** (git source can't take SHAs) — generated code is committed, so the pin is baked in |
| Validation | whatever you write | whatever you write | maxLength/maxGraphemes/format checks **generated** |
| Forward-compat (unknown fields, union members) | manual `#[serde(flatten)]` extra map | manual | `extra_data` + `Unknown(Data)` generated everywhere |
| Record ops | separate manual plumbing | manual | `Collection` trait → typed `put_record`/`get_record` |
| Cost | full control, no new deps; you own every constraint | small tool, types still manual | runtime deps on `jacquard-common` + `jacquard-derive` + `jacquard-lexicon`; BosStr generics pervade; chrono-based `Datetime` (not jiff); generated code is `DO NOT EDIT` |

## 4. Login / auth

Two first-class paths, both with resume helpers (0.12's "Sessions and OAuth
scopes" rework, `README.md`):

- **App password** (`CredentialSession`,
  `crates/jacquard/src/client/credential_session.rs`):
  `com.atproto.server.createSession` → Bearer token, auto-refresh on
  expiry, `resume(&SessionHint)` → resume-or-login challenge flow (see
  `examples/app_password_create_post.rs`). An explicit **PDS override**
  exists (`CredentialLoginOptions.pds`, `client.rs` line 72/386–394), else
  handle→DID→PDS via the resolver — i.e. writes go to the user's actual PDS,
  arbitrary PDS supported.
- **OAuth DPoP** (`crates/jacquard-oauth`): loopback localhost flow for CLIs
  (`OAuthClient::with_default_config` uses
  `AtprotoClientMetadata::default_localhost()`, `client.rs` line 110–116),
  confidential clients for servers, private_key_jwt, nonce retry, scope
  builder with **permission-set `include_aud` support** — the exact machinery
  `pub.leaflet.authFullPermissions` would need later.

**On-disk sessions:** `FileAuthStore` (wrapper over `FileTokenStore`,
`crates/jacquard/src/client/token.rs`, `crates/jacquard-common/src/session.rs`)
writes **one JSON file at a caller-chosen path** (no fixed/XDG location;
examples use `/tmp/jacquard-*.json`) holding a map keyed
`atp:{did}/{session_id}`, `oauth:{did}/{session_id}`, `oauth-state:{state}`.
**No 0600 permission hardening** (plain `std::fs::write`; verified by grep).

**What springleaf should borrow:** the resume-or-login shape (`SessionHint` →
resume → challenge → login) mapped onto
`ATPROTO_HANDLE`/`ATPROTO_APP_PASSWORD`/`ATPROTO_PDS`; a single-JSON session
store *but with mode 600 and a real config path* (`~/.config/springleaf/`
per the design's config resolution); keep the leaflet `auth_token` cookie
entirely separate in the `drafts` crate (Jacquard has nothing for web-app
cookies — correct separation). The loopback OAuth path is the ready-made
answer for `authFullPermissions` when v0's app-password scope ends.

## 5. Posting capability

**Yes, today, against an arbitrary PDS.**
`AgentSessionExt` (`crates/jacquard/src/client.rs`) provides:
`create_record` (line 706), **`put_record`** — the upsert springleaf needs —
(line 1081), `update_record` (fetch-modify-put, line 976), `delete_record`
(1036), `get_record` (783, typed via `Collection`), **`upload_blob`**
(bytes + `MimeType`, line 1150, sets real Content-Type). All use bootstrap
types so they work without the `api` feature; `listRecords` is available as a
generated request via `agent.send(...)`. `examples/post_with_image.rs` is
almost literally springleaf's `publish run` (upload blob → build record →
create), and `examples/app_password_create_post.rs` is the auth path.

There is no "session blobs/candidates/lexicon-record plumbing" for leaflet's
*draft* system — that's Replicache/Yjs territory the design correctly keeps
out — but everything on the PDS-record side (typed records, blobs, strongRefs,
`blobs[]` mirroring knowledge baked into the generated `Content` type) is
reusable **as a library dependency**: `jacquard` re-exports client + api +
identity + oauth, and the generated leaflet types are in the default feature
set.

## 6. Reuse verdict

**Recommend (b): use Jacquard's lexicon toolchain for springleaf's record
types, keep the hand-rolled thin `atproto` crate, borrow its auth patterns.**
With a documented upgrade path toward partial (a).

- **The crown jewel is the generated, validating leaflet types.**
  Re-deriving `site.standard.*` + `pub.leaflet.*` by hand-rolled serde
  re-does work Jacquard has already done well — including grapheme-limit
  validation, open unions, and unknown-field forward-compat that hand serde
  always skimps on. Run the codegen step (prebuilt binary, or `cargo run -p
  jacquard-lexgen`) against the pinned local leaflet checkout and **commit
  the generated code**, exactly like `probe-codegen.sh` demonstrates. Pinning
  then lives in the committed artifact + the checkout SHA, satisfying the
  design's "Lexicon drift" risk without noteful's `--sha` fetcher (which
  jacquard lacks).
- **The client is excellent but heavier than springleaf's needs.** The design
  doc's `atproto` crate is four calls (createSession, putRecord, uploadBlob,
  resolveHandle/listRecords) with social-cli as reference; owning that is
  cheap and keeps the design's jiff/thiserror/anyhow stack coherent.
  Adopting `jacquard` wholesale imports the BosStr generic model, miette,
  chrono, bon, and a large transitive tree, plus 0.13 breakage risk and a
  bus-factor-one upstream. That trade only pays off when springleaf wants
  OAuth loopback (`authFullPermissions`), jetstream/firehose, or repo
  primitives — at which point `Agent` + `put_record` + generated types can
  replace the thin `atproto` crate wholesale (the typed surface is the same
  `Collection` model either way, so the migration is contained).
- **(c)-style borrowing regardless of the above:** the `extra_data` flatten
  pattern, open-union `Unknown` handling, resume-or-login auth shape, and the
  schema-documented `blobs[]` top-level mirroring rule (which partially
  answers the design's Blob-GC risk for `pub.leaflet.content`; still verify
  for `site.standard.document.coverImage`, which is already top-level).

**Design-doc changes if (b) is accepted:**

1. `crates/leaflet` — replace "hand-written serde types" with a **generated
   module subtree** (own `lexicons.kdl` with `source "leaflet" type="local"`
   pointing at the pinned archive checkout, or vendored JSON at a recorded
   SHA); hand-written code shrinks to markdown⇄blocks conversion, slug/rkey +
   URL derivation, and glue converting generated `Datetime` (chrono string)
   ↔ jiff at the CLI boundary.
2. New workspace dep set: `jacquard-common`, `jacquard-derive`,
   `jacquard-lexicon` (default-features off to drop codegen deps in runtime
   builds); `jacquard-lexgen` as a build/tooling-only dep. Note the
   `Datetime`/jiff and smol_str entries in the library-choices table.
3. Risks section: replace "pin lexicon JSON at a SHA (noteleaf pattern)"
   with "lexicons pinned via committed generated code from the pinned
   checkout; regeneration is a deliberate `just` step"; downgrade Blob-GC
   from unknown to partially-answered (schema documents mirroring; verify
   PDS behavior before shipping images — unchanged recommendation).
4. Stages 3–4 unchanged in shape: `pubs list` still uses the thin `atproto`
   crate; `publish build/run` constructs **generated** `Document`/
   `Content`/block types instead of hand serde.

## Probes

- `probe-codegen.sh` — offline codegen from pinned leaflet checkout (this
  directory). Result: standalone `site_standard` + `pub_leaflet` (+ deps)
  module tree generated in seconds.
- `cargo check --release -p jacquard-common -p jacquard-lexicon -p
  jacquard-lexgen -p jacquard` in the archive — pass, 2m48s, one unused-dep
  warning, on nightly 1.101.0 (2026-10-13). No files committed in the
  archive; scratch output stayed in its pre-existing `target/` and
  `/tmp/opencode`.
