---
type: Report
title: "Layered config restored; real drafts checked out as keyed MDX"
description: "Restored Figment2/workspace/publication defaults and delivered the authorized native read-only checkout into ~/l, with bounded production Yrs/Satteri integration, live counts, safety and verification receipts."
resource: /design/draft-checkout0.gpt61s.md
status: implemented
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e46368af7ffeGOrXJkxUfkr5a7"
    title: "Same Sol session: restored config, configured ~/l, implemented actual content import, imported all 27 observed drafts and verified loss/no-op/safety without committing private payloads"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted crates, import CLI, one-file lifecycle, YAML and scoped witness refinements"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Author/tool field separation, bindings, source identities and no-history constraints"
  - resource: /design/remote-reads0.gpt61s.md
    title: "Prior same-session native metadata read implementation, now extended to content checkout"
  - resource: /design/scaffold-reset0.glm53m.md
    title: "Selective recovery and Yrs promotion requirements"
  - resource: /experiments/01-draft-list/REPORT.md
    title: "Inventory discovery and its completeness uncertainty"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Proven null-cookie full pull, fact model and synthetic fixtures reused in regression tests"
  - resource: /experiments/05-yrs-conversion/src/lib.rs
    title: "Promoted strict Yrs XML/mark and legacy text schema conventions"
  - resource: /experiments/13-satteri-authoring/src/text.rs
    title: "Promoted production Yrs text-reading path, not the hand-rolled codec"
  - resource: /experiments/13-satteri-authoring/src/author.rs
    title: "Satteri arena lowering, literal JSX/Run/Key/Page precedents"
  - resource: /experiments/14-satteri-authoring-attributes/src/adapter.rs
    title: "Brace adapter inspected; imports deliberately use explicit Key markers rather than adding brace syntax here"
  - resource: https://github.com/rektide/leafspring/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/cli/src/config.rs
    title: "Recovered Figment2 precedence and strict config read posture"
  - resource: https://github.com/rektide/leafspring/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/cli/src/config_cmd.rs
    title: "Recovered toml_edit comment-preserving setter and provenance precedent"
  - resource: https://github.com/rektide/leafspring/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/cli/src/workspace.rs
    title: "Recovered root selection only, not old record directories/sidecars"
  - resource: https://github.com/rektide/leafspring/blob/80436b61cf69a2e41d157684b24c6cf2fd4674da/crates/drafts/src/pull.rs
    title: "Recovered proven native pull transport, strengthened to reject partial/malformed snapshots"
  - resource: file:///home/rektide/archive/hyperlink-academy/leaflet/actions/getIdentityData.ts
    title: "Pinned upstream query checked for pagination/limit and contributor embeds"
  - resource: file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/getPublicationMetadataFromLeafletData.ts
    title: "Sibling permission-token association lookup verified before draft selection"
  - resource: file:///home/rektide/archive/hyperlink-academy/leaflet/components/Blocks/ImageBlock.tsx
    title: "Native current image payload semantics, distinct from latent text after type conversion"
  - resource: file:///home/rektide/archive/hyperlink-academy/leaflet/components/Blocks/CodeBlock.tsx
    title: "Native current code payload semantics, distinct from latent block/text"
---

# Config restoration and actual draft checkout

## Outcome and checkpoints

The user commissioned actual work with their data, not another inventory.
Code remains in `~/src/leafspring`; **`~/l` is the document workspace**.

- `ba87d27c` — **Restore layered authoring config and shared document workspace resolution**.
- `3ba5a977` — **Import native drafts as loss-checked keyed MDX without remote writes**.
- Coordinated version **0.4.0**, exact internal pins and updated root lockfile.

Authorized live result: **30 observed Leaflets → 27 verified drafts imported,
3 published objects excluded, 0 failures remaining**. Local output is
`/home/rektide/l/drafts/*.mdx`: 27 current documents, 335 blocks and 29 page
definitions. All files remain `state: draft`, without a public binding URI or
invented publication date. No raw-response siblings, base snapshots or CRDT
history files were created; the workspace contains no non-MDX files.

Privacy boundary: credentials were read only through existing stores; private
text, token values and raw responses were neither printed nor committed. This
report intentionally omits document titles, capability locators and filenames.
The live file set is private user data outside the source checkout.

## Config: retained, recovered, reused, new

**ATProto was retained across the scaffold reset** and its implementation is
unchanged here. Native draft discovery/content reads continue to use the cookie
web client, not an incidental configured PDS. Its separate library remains
available for appropriate identity/public-record/session operations.

**TOML config had been removed by the reset**, not used by the first remote
read slice. It is now selectively recovered in
[workflow/config.rs](../crates/workflow/src/config.rs), rather than restored
inside a CLI monolith:

- Figment2: explicit flags > unbound document intent > workspace TOML > global
  defaults; known `handle`, `pds`, `publication` values and per-key provenance.
- Shared root: `--root` > global `workspace` > cwd; relative global roots resolve
  from the config file's directory. Workspace TOML cannot bootstrap its own root.
- Thin `config show [--json]` and `config set KEY VALUE [--workspace-file]`.
- Reused toml_edit comment/decor preservation. Unknown settings remain untouched
  and are not dumped by `show`; malformed known-key types/TOML are loud errors.
- New atomic local writer checks original bytes before replacement, preserves
  existing permissions and uses no-clobber installation for new imports. This is
  a source-byte guard plus atomic replacement, not an adversarial filesystem CAS.

The actual global config's `workspace` was changed to `/home/rektide/l` through
`config set`. Existing **handle and publication choice were preserved**; PDS
remained unset. No workspace TOML or new auth system was needed, and credentials
were not changed. No old workspace files were moved, pruned or deleted.

Imported documents use **observed native association**, including explicit null
for standalone, never an inherited default. `--publication` is not an import
retargeting operation. The generic unbound-default stack does not itself bind or
publish anything; ordinary create/save/publication remains unimplemented.

## Production boundaries and current format

| Owner | Delivered seam |
| --- | --- |
| [web pull](../crates/leaflet/web/src/pull.rs) | Null-cookie full pull, strict initial-clear/initialized/patch checks, raw current response and fact/extras retention in memory |
| [web metadata](../crates/leaflet/web/src/lib.rs) | Verified one-level sibling permission-token association inspection; no guessed AT-URI resolution |
| [editor](../crates/leaflet/editor/src/lib.rs) | Pure fact scan, placement order, page graph and compact entity/fact associations; unsupported/unrepresented current facts fail |
| [Yrs text](../crates/leaflet/editor/src/text.rs) | Production dependency pinned to experiment 05/13's `e0b476b0…`; causal integration, legacy schema, strict XML/marks; no hand-rolled codec |
| [document model](../crates/document/model/src/lib.rs) | Bounded current content, draft state, source keys, occurrence-keyed memberships and compact witnesses |
| [MDX adapter](../crates/document/mdx/src/lib.rs) | Production Satteri/YAML parser, initial writer, semantic round-trip check and managed-header inspection for binding discovery |
| [workflow import](../crates/workflow/src/import.rs) | Root/config composition, all/single selection, binding-based local discovery, collision-safe creation, guarded no-op/preservation, per-item reports |
| [CLI import](../crates/cli/src/import.rs) | Thin parsing/rendering; `import --all --state draft --root DIR`, single locator `--to`, JSON and partial-failure exit 4 |

No production crate depends on an experiment path. Existing proof code was
reviewed and promoted with real dependencies; experiments remain intact.

The profile `leaflet-import/1` represents linear paragraphs/headings/quotes,
rich runs/marks/hard breaks, exact code, image URL/fallback/dimensions/alt,
page links/definitions and the observed watermark setting. It preserves remote
author title/description/tags/contributors when supplied. Unsupported meaningful
extras or editor attributes are per-document blockers, not omission warnings.

Ordinary prose uses keyed Markdown. Exact rich runs use the proven literal
`Run` JSX convention; when Markdown would split a flow run or reinterpret a
block, typed `Paragraph`/`Heading`/`Quote` boundaries preserve it. `Code` stores
exact code text/language; `Image` stores current resource descriptors, not a
snapshot. These are editable MDX components, not an imported JSON document
disguised as a checkout. Image binaries remain at their native source URLs;
no offline binary cache/materialization is claimed.

Generated YAML quotes strings/keys and rejects duplicate keys, merges, aliases,
anchors and custom tags on read. Unknown author fields can be inspected and
fingerprinted; repeated imports never rewrite them or their comments. No
general source-preserving frontmatter/body writer is claimed.

IDs are deterministic at first association and remain with the file thereafter;
filenames are friendly-title plus full token UUID. Binding discovery recognizes
renames/repeated imports, rather than duplicating on title. Page references have
one definition with all observed relation locators; repeated block placements
have occurrence-specific source keys.

`localHash` covers represented content, source IDs/order and node associations,
not receipt hashes. `remoteHash` conservatively covers every current fact's
meaningful payload (including opaque state), extras and observed metadata/
association, excluding CVR cookies, indexes and row timestamps. Both are scoped
by the import profile; neither is a public-release receipt or a saved body.
Before handoff, the 27 session-generated hash receipts were strengthened to
include node associations, only after verifying their original semantic hashes;
no body/binding edits were made by that correction.

Native image/code blocks can retain inactive `block/text` from type conversion.
The pinned native components do not render it. The import therefore keeps its
locator separately as `inactiveTextFact` and includes the opaque payload in the
remote witness, without falsely displaying it as current body content or
claiming to have archived it locally. No native fact was cleared or changed.

## Live evidence and coverage

The existing cookie worked. `/home` returned 30 unique objects, with 3
homepage/contributor overlaps merged and no DOM/payload mismatch. Full native
metadata/sibling checks found 27 drafts and 3 published objects, with no sibling
reclassification needed for this account at observation time.

The pinned upstream checkout `600c7b5f1f7b31b66fb626c293cc104312a12caf` was read:
`getIdentityData` embeds homepage/contributor rows and has **no explicit limit,
range, cursor or pagination step**. This supports “all observed drafts,” not a
claim that server/backend limits cannot exist. Reports keep account-wide
completeness explicitly unverified. No pagination endpoint was invented.

The first live import created 15 files and reported 12 concrete blockers.
Those exposed exact JSX boundary handling and inactive text semantics; fixes
were verified, then the remaining 12 imported successfully. The final full
repeat reported **0 imported, 27 unchanged, 3 filtered, 0 failed**, exit 0;
SHA-256 file-byte hashes and nanosecond modification times were identical
before/after. Local validation passed all **27 files / 335 blocks**, with
**0 changed logical projections**, 27 unique document IDs and 27 unique native
bindings. Native publication or editorial mutation was never invoked.

## Verification and remaining seam

- `cargo fmt --all` / `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo nextest run --workspace` — **67/67 passed**. Tests use synthetic fixture
  and loopback data, not the user's documents or credentials.
- Config tests exercise layer precedence, root routing, provenance,
  preserving/no-op writes, malformed input and source-byte race refusal;
  CLI subprocesses cover show/set and unknown-setting privacy.
- Import tests exercise real native fixture → Yrs → keyed MDX → semantic parse,
  exact rich/empty/code/image/page representations, restricted YAML, unknown
  author fields, inactive opaque witnesses, repeat/rename/local edits, remote
  drift preservation, single destinations, collisions and path traversal.
- Full-pull tests reject partial/uninitialized/malformed responses; sibling
  linkage classification is exercised separately from direct metadata.
- Live checkout/repeat/check receipts above are separate from offline tests.
- Owned README/report link/provenance checks use `rkt check` and `--okf`.

Remaining boundaries: published-object import and document AT-URI resolution;
canvas/lists/interactive/other unsupported editor types; generalized push/pull,
source-preserving updates and formatter; offline asset materialization; proven
account-wide discovery completeness. There are **no remaining failed observed
drafts** in this checkout. Native writes remain out of scope.

Useful next small slice: guarded refresh/pull for these bound files, preserving
author text/comments and refusing local/remote conflicts. Reimport currently
preserves local edits and reports remote drift rather than silently overwriting;
implement refresh before promising an ongoing sync loop. See the
[accepted interface](authoring-interface0.gpt6a.md) for that contract and
[README usage](../README.md#checkout-drafts-as-editable-mdx) for current commands.
