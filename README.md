---
type: ProjectGuide
title: Leafbud
description: "Current capabilities, commands, source contracts and verification map for Leafbud."
resource: /README.md
status: active
updated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3d6d728effe6RFnyZcrUtifT9"
    title: "Retained native-default no-op correction and explicit terminal recovery/replanning with bounded old-attempt provenance, current-witness conflicts and recovery/follower verification"
  - resource: /design/terminal-recovery0.gpt61s.md
    title: "Version-0.17 terminal recovery API, atomic state policy, no-op correction and acceptance evidence"
  - resource: "opencode://ses_e3d96851dffeS2cPD2vPHjGklF"
    title: "Complete keyed native compiler, guarded public push, independent Yjs and HTTP/Phoenix verification, and create-free deployed P1 baseline/revision/pull/repeat proof"
  - resource: /design/keyed-save0.gpt61s.md
    title: "Version-0.16 complete authoring receipt, APIs, tests and remaining recovery boundaries"
  - resource: "opencode://ses_e3dcb1f8effebiT0KtCRZB8dWf"
    title: "First-binding default/publication preparation and selected-locator recovery; corrected shared description-clear semantics and settled original P1 intent without content effects"
  - resource: /design/native-default-adoption0.gpt61s.md
    title: "Version-0.15 correction receipt, API/verification map and remaining terminal-partial boundaries"
  - resource: "opencode://ses_e4121e55dffeF009EbMiG1Xinz"
    title: "Native-effect foundation, compact recovery interfaces, loopback verification and bounded deployed verification with explicit partial/uncertain outcomes"
  - resource: /design/native-effects0.gpt61s.md
    title: "Version-0.14 native mint/push/recovery checkpoint and compiler/deployment join limits"
  - resource: "opencode://ses_e416ba4e7ffeZ1ThfycXYgNh5S"
    title: "Connected watch implementation, shared guarded refresh and locking, current usage, tests and passive deployed CLI verification"
  - resource: /design/connected-watch0.gpt61s.md
    title: "Implementation receipt and exact proof limits for connected inbound follow"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Accepted complete-authoring and early connected-follow direction"
---

# leafbud

Local-first MDX authoring for [leaflet.pub](https://leaflet.pub)
documents, in Rust: one MDX file per document, durable identities,
push saves the Leaflet editor, publish releases, pull adopts web edits.

Status: **offline draft creation, one-time source-key preparation
(`tidy --keys`), format-2 implicit-main-page MDX with flat lists,
Markdown-first rich text with checked fallbacks, scoped read-only status,
external-tool human diff, guarded pull, connected inbound `watch`, passive
advisory presence, cooperative writer locks, layered config, complete keyed native editor saves through public `push`, and explicit terminal `recover`, version 0.17.0.** The accepted authoring interface
([`design/authoring-interface0.gpt6a.md`](design/authoring-interface0.gpt6a.md))
settles root CLI verbs (`create`/`check`/`list`/`status`/`diff`/
`push`/`pull`/`publish`/`import`/`edit`/`tidy`/`export`, plus connected `watch`,
`keys rename`, `remote`, `pubs`, `auth`, `config`), all-applicable
`tidy` with `--keys` narrowing, domain-grouped crates, and
operation-specific YAML preservation. The workspace now matches that
destination shape. Implemented versus pending:

| implemented | not implemented |
| --- | --- |
| `leafbud-atproto` — retained identity resolution, XRPC, app-password sessions; `leafbud-records` — pinned record types/validation/helpers | public release/publishing workflow |
| `leafbud-protocol` + native web/workflow — actual native mutation vocabulary, bounded immutable requests, both mint contexts, writer-group acknowledgments, explicit known-entity reads, compact attempts, guarded first-binding default preparation, selected-locator recovery and deliberate terminal accept-current/replan | original lost publication-mint attribution; general repair of incomplete/unsupported partial graphs |
| `leafbud-web` — cookie-authenticated discovery, metadata and full editor pull; origin-bound cookie-free realtime bootstrap, managed Phoenix joins/heartbeat/reconnect and passive presence; `leafbud-editor` — raw-fact scanning and complete keyed create/update/place/detach compiler, loaded Yrs text/marks/break editing and selected supported-block recreation | canvas, nested/task lists, interactive/other unsupported editor content; whole-body recreation |
| `leafbud-document` + `leafbud-mdx` — format-2 (`leafbud-mdx/2`) current-document model with implicit main pages, optional binding/receipts, tri-state publication intent and typed block/page occurrence locators; real Satteri/YAML parse accepting new/retired keys, flat lists, one-time key preparation with proof, tracking-only settlement separate from remote-content adoption, loss-checked Markdown-first writer with block-local exact fallback, parser-derived Markdown tidy planning | naming tidy passes; nested/task lists |
| `leafbud-workflow` + CLI — `remote list/show`, draft `import`, offline `create FILE --title [--standalone|--publication URI]`, preparation-aware `check`, read-only `status [FILE...] --offline/--json`, `diff FILE --edge editor` via configured external tool, preview-first complete `push FILE.../--all [--yes] [--take local] [--recreate KEY] [--json]`, `recover FILE [--take remote\|local] [--yes] [--json]`, guarded `pull`, connected `watch FILE.../--all` with rescan/backstop/JSONL and optional presence, cooperative stable-ID locks, `tidy FILE.../--all` with `--keys/--markdown/--dry-run/--json`, Figment2 config show/set and shared workspace root | local `list`, `publish`, `edit`, `export`, `keys`, `pubs`, `auth` commands; `diff --edge names/release`; `tidy --from/--accept` naming selectors (rejected honestly until built) |

The pre-reset stage-5 client (cookie web surface, JSON/sidecar
publish+sync lifecycle, append authoring, hand-rolled Yjs codec) was
removed by design and is recoverable at revision `80436b61`; the
[cleanup handoff](design/scaffold-reset0.glm53m.md) records exactly
what was kept, what was reset, and where each piece re-homes.

## Layout

```
crates/
  document/
    model/     leafbud-document: pure document model — current content,
                stable source IDs and compact bindings (bounded import profile)
    mdx/       leafbud-mdx: Satteri parsing, YAML frontmatter,
                loss-checked initial keyed MDX writing and validation
  leaflet/
    editor/    leafbud-editor: fact graph, Yrs text state, bindings,
                  pure observation and complete keyed native compilation
    protocol/  leafbud-protocol: native messages, bounded request encoding,
                 raw full views/acknowledgments; no HTTP, Yrs or files
    web/       leafbud-web: cookie auth, discovery, RPC/actions,
                 mint/push/known-ID reads and receive-only live subscriptions
    records/   leafbud-records: generated pinned lexicon record types,
               validation, wire-shape helpers — see LEAFLET-PIN.md
  atproto/     leafbud-atproto: hand-rolled ATProto surface — handle→DID→PDS
               identity resolution, XRPC client, sessions, public reads
  workflow/    leafbud-workflow: local files, config, inventory,
                 guarded import/pull, shared refresh, cooperative locks, live follow
  cli/         leafbud (binary): thin Clap/rendering layer;
                    reads/import/create/check/status/diff/push/recover/pull/watch/tidy/config
design/        architecture, accepted interface, and format designs
experiments/   protocol probes and evidence (tracked, load-bearing)
```

Dependency direction (accepted design §2): pure document model →
MDX adapter / editor / record projections; `web` owns effects;
`workflow` composes the libraries; the CLI stays thin. The scaffold
adds no crate dependency edges that no code uses yet.

## Keyed editor saves

```sh
leafbud create drafts/essay.mdx --title "Garden notes"
# Write ordinary Markdown; keys are assigned once on execution.
leafbud push drafts/essay.mdx                  # preview: no source/native writes
leafbud push drafts/essay.mdx --yes            # bind if needed, save; never publish
leafbud push --all --state draft --yes --json
leafbud push drafts/essay.mdx --take local --yes # deliberate current-remote decision
leafbud push drafts/essay.mdx --recreate chosen-key --yes
```

The complete supported source is preflighted before native effects, including
before first mint wherever representability/storage intent is knowable. Both
standalone and publication first binding reuse the native main page/entry and
detach the verified empty seed. Local UUID/title remain independent of the first
heading. Normal saves retain existing block/entity/text-fact/membership identities;
new keys create new objects. Move a block's marker with it, keep one original key
when splitting or joining, and omit a keyed occurrence to detach its placement.
Omission never deletes entities or assets. Changed pages use simple native
move-to-end normalization, not an optimal-edit engine.

Paragraphs, headings, quotes, all supported inline marks/links/hard breaks, flat
bullet/ordered items, rules, code, HTTP(S)-URL images and reachable linear named
pages/links save together. Compatible text uses scalar-safe UTF-16 prefix/suffix
splicing in the loaded Yrs document; awkward supported topology rebuilds only its
paragraph interior. Unchanged rich meaning emits no text mutation. Type/scalar
edits retain known inactive text. Shared occurrences must request consistent
entity fields, or explicitly fork one with `--recreate KEY`; recreation retains
the selected placement and leaves old entities/other occurrences intact. Repeated
same-entity placements within one parent are diagnosed, not guessed.

Unchanged retained blocks also preserve native default spellings: absent blank
text, implicit bullet style, explicit non-list false, empty code-field absence,
optional image-field absence and inactive text are not normalized just by saving.
Persisted title metadata is still checked separately from the `Untitled` display fallback.

Preview never persists keys/pending, mints or pushes. Effect counts come from
actual selected native arguments; an unbound preview instead reports source
preflight because no verified native seed yet exists. `--yes` alone never resolves
remote drift. `--take local` binds the decision to this invocation's current
witness and rechecks it before effects; it cannot bypass pending work, unknown
native semantics or unsupported content. The server is not a remote CAS.

Read-back verifies complete meaning/order/metadata/associations plus protected
facts, including explicit reads of retained detached objects. Only then does
tracking-only settlement advance receipts. Repeating an unchanged save sends
**nothing** and preserves file bytes/mtime. Source edits across waits are preserved;
remote success can remain locally unsettled. Exact encoded requests may retry
in-process; after restart, `push` only observes/settles already-achieved terminal
work or reports pending/partial uncertainty, never independently re-encodes it.
Terminal partial work uses the explicit recovery command below, not a push force flag.
When only a prior attempt settles, push reports **`settled-prior`**: no separately
requested recreation or new save ran. Repeat push to request that separate action.

Reports are version-1 JSON with per-file outcome and native effect counters.
Exit codes: **0** successful preview/save/no-op, **1** operational/preflight failure,
**2** usage, **3** needs-decision, **4** pending/partial/source-changed or mixed results.
`--recreate` requires one explicit file; `--recreate-body` is honestly unimplemented.
No release, email, automatic cleanup or background auto-push is performed.

Both first-mint contexts pass complete CLI/HTTP fixtures. Actual deployed
**human-selected existing P1** passed CLI baseline → combined revision → read-back
→ unchanged pull/repeat. This made zero draft-creation calls; original lost-mint
attribution is still unestablished, and the old standalone partial is untouched.
Implementation/evidence: [complete keyed save receipt](design/keyed-save0.gpt61s.md).

## Explicit save recovery

```sh
leafbud recover drafts/essay.mdx --json                  # read-only finality inspection
leafbud recover drafts/essay.mdx --yes                   # settle original achieved intent only
leafbud recover drafts/essay.mdx --take remote           # preview accepting current native state
leafbud recover drafts/essay.mdx --take remote --yes     # adopt current supported native state
leafbud recover drafts/essay.mdx --take local --yes      # new save of CURRENT source against CURRENT native state
```

`--yes` executes; `--take` chooses a new direction. Neither overrides unknown or
lower writer acknowledgment. Changed source does not prevent read-only finality
inspection, but cannot masquerade as the original attempt's success. Original
achieved intent still settles automatically from observation without a new save.

For an acknowledged terminal partial, remote adoption preserves local UUID,
binding, source labels via existing **and pending** associations, local-only fields
and unchanged source bytes. Local replanning reuses observed partial allocations
(including supported known detached blocks), compiles current authored intent and
reserves a **new attempt/range** under the same writer. Old IDs are never reused;
no old encoded update is replayed independently. Missing/ambiguous allocations,
unsupported partial graphs and detached named-page repair block precisely instead
of creating replacements or resurrecting unknown objects.

One compact `@leafbud.lastResolution` retains the old attempt ID/hashes,
writer/range/sent-through/acknowledgment, new direction, observed fingerprint and
optional successor attempt. Its disposition is **`originalNotVerified`**, not
original success. The old pending state survives until guarded atomic adoption,
verified no-effect correspondence, or replacement pending plus provenance. No
second body, executable journal, unbounded history or automatic cleanup is stored.

Recovery rechecks source bytes, all selected operands, native identity/rights and
the current native witness across waits. Preview changes no source/keys/attempts or
native state. JSON version 1 reports source/attempt IDs, whether a binding is known,
observed finality/source match, direction and outcome without capability URLs or
bodies. Exit **4** means pending/partial/blocked/source-changed; other codes match
push. Pull/watch keep pending state intact, then resume after resolution. Mint
uncertainty still requires the explicit supplied-locator library API; recover
never guesses a draft or creates another one. Evidence and limits:
[terminal recovery receipt](design/terminal-recovery0.gpt61s.md).

## Native-effect foundation (internal checkpoint)

The ninth, wire-only crate is now consumed by web, editor and workflow.
`RemoteReader::stage_binding` authenticates and resolves first-binding intent,
prepares source keys once, persists an attempt before mint, stores a returned
locator before verification, and leaves **pending materialization**, not a saved
document. Both standalone `/new` and current publication-dashboard action
discovery have real loopback coverage. `recover_native` observes/settles finite
attempts; it never remints or re-encodes a request after restart. Version 0.16 joins
this runner to the complete pure editor compiler and public `push` described above.

First binding records the verified publication association explicitly and copies
only **omitted** theme/contributor defaults into source. Explicit watermark true/
false and contributor lists (including `[]`) remain authoritative. Explicit
`theme: null`/`contributors: null` have no supported meaning and fail before mint;
description/tags are not adopted as defaults. Source prose, keys, local UUID,
local-only fields and unrelated comments remain unchanged. This is one guarded,
atomic preparation checkpoint with corresponding mint hashes, not relaxed read-back
verification. Unsupported contributor changes stop before push. A located mint
resumes through the same preparation; a dispatched push never has its old intent
retroactively rewritten. Actually observed empty contributor lists are now retained.

Description absence, null and exactly `""` all mean **clear**. Nonempty strings,
including whitespace-only strings, remain exact. Editor/semantic projections,
incoming mapping and human views share this rule, while source spelling and raw
native witnesses remain intact. Omission still requires a real clear operation
when the native description is nonempty; it never means preserve remote content.

`select_mint_locator` accepts an explicit existing recovery target for the expected
dispatched attempt, stores it before verification, and reuses ordinary mint
recovery. It does not discover candidates, retry creation, replace a known locator
or establish original-response attribution.

Attempts contain IDs, fingerprints, reserved writer ranges and protected-scope
witnesses, never source bodies, binary text or a request journal. HTTP success and
mutation acknowledgment are not proof of application. Status distinguishes
prepared, uncertain, materialization-pending and resolved-partial attempts without
fetching or advancing receipts; pull/watch/tidy preserve pending work. Persisted
title metadata is independent of the first heading; initial filename convenience
is a separate import fallback.

The authorized deployed check verified **one new standalone draft**, controlled
text/metadata writes, writer acknowledgments and explicit known-fact read-back.
It remains resolved-partial under its original dispatched intent; the new default
policy was not applied retroactively. The publication mint's original locator was
lost in Flight decoding. The operator subsequently selected **P1 as the existing
recovery target**, not as a proven original result. Create-free recovery verified
its seed/rights/context, recorded defaults, and acknowledged/observed controlled
title/text writes. Initial verification stopped on absent source description versus
native empty string. The shared clear-semantics correction then let normal recovery
verify **the original intent with unchanged pending hashes** and settle tracking
only. P1 now has clean status, equal endpoints, unchanged pull and an exact
byte/mtime repeat no-op. No description field was inserted or native content effect
sent during settlement; no push was replayed or independently re-encoded.
No release/email or unrelated user-document mutation occurred. Both creation slots
remain consumed; no new draft was minted during this correction. Initial evidence:
[native-effect receipt](design/native-effects0.gpt61s.md). Correction, exact API
contracts and remaining boundaries:
[default-adoption receipt](design/native-default-adoption0.gpt61s.md).

## Remote reads

```sh
cargo run -p leafbud -- remote list --state draft
cargo run -p leafbud -- remote list --json
cargo run -p leafbud -- remote show 01999000-0000-7aaa-8000-000000000001
cargo run -p leafbud -- remote show https://leaflet.pub/01999000-0000-7aaa-8000-000000000001 --json
```

The examples use a synthetic UUID; replace it with a real permission-token ID.
Credentials: `--token` > `LEAFLET_AUTH_TOKEN` >
`$XDG_CONFIG_HOME/leafbud/auth-token` (else
`~/.config/leafbud/auth-token`). Prefer env/file to keep tokens out of
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
cargo run -p leafbud -- config set workspace /home/rektide/l
cargo run -p leafbud -- config show --json
cargo run -p leafbud -- config set publication AT_URI --workspace-file
```

Workspace root: `--root DIR` > global TOML `workspace` > cwd. Global config is
`$XDG_CONFIG_HOME/leafbud/leafbud.toml` (else
`~/.config/leafbud/leafbud.toml`); relative global workspace paths resolve
from that file's directory. `<root>/leafbud.toml` supplies workspace defaults.
Figment2 resolves explicit flag > unbound document intent > workspace > global
for `handle`, `pds`, `publication`; `config show` reports per-key provenance.
`config set` edits only the chosen key, preserves comments/unknown settings,
and refuses malformed files. `workspace` belongs in the global file.

ATProto remains available in its library; native draft reads/imports use the
cookie web client, **not** the configured PDS. Imports retain the observed
remote publication association (including explicit standalone null), never
retargeting a binding from ambient defaults or inventing publication dates.

## Local authoring: create, check and key preparation

```sh
cargo run -p leafbud -- create drafts/essay.mdx --title "An essay"
cargo run -p leafbud -- create drafts/notes.mdx --title Notes --standalone
cargo run -p leafbud -- check drafts/essay.mdx
cargo run -p leafbud -- tidy drafts/essay.mdx --keys --dry-run --json
```

`create` writes one unbound local draft offline: format `leafbud-mdx/2`, a
stable random document UUID, an empty implicit main page, no binding or
receipts, and no remote object of any kind. Existing files are never
overwritten. Publication intent is tri-state and needs no native identity:
omitted inherits configuration at first binding, `--standalone` records an
explicit null, and the global `--publication URI` records a target.

The format-2 grammar is unambiguous: top-level Markdown blocks and
`<Key id="…" />` markers accumulate in order into the **implicit main
page**; `<Page name="…" kind="linear">` components are named page
definitions that may appear anywhere without interrupting main order;
`PageLink` targets may be forward references. Blocks without a Key marker
are new content, not parse errors — identity is assigned once through
preparation, never inferred from position or text. Flat `-`/`1.` lists are
one keyed block per item (task and nested lists are loud diagnostics);
ordered numbering normalizes to `1.` because the native model sequences
display numbering itself.

Preparation (`tidy --keys`, or executed push later) inserts one
`b-<uuid>` Key marker per new block — splitting authored multi-item lists
into per-item markers — persists the implicit main page's `p-<uuid>`
identity in the reserved envelope, keeps every unrelated byte (comments,
line endings, BOM, witnesses), proves the result through the production
parser, and repeats as an exact no-op. The confirmed node table may lag
desired content: keys without rows are new (explicit `new` identities in
the editor projection), rows without content are explicit pending
omissions rather than dropped or nulled. Retained-key renames never change
editor identity, and locally created UUIDs survive later binding. Check
reports how many objects still need keys; status labels unbound drafts
honestly and never hashes random previews as durable identity.

The `~/l` live checkout files predate format 2; they are rebuildable local
data and a deliberate fresh `import` replaces them (no migration path is
intended). Batch implementation handoff:
[`design/author-create-keys0.glm53m.md`](design/author-create-keys0.glm53m.md).

## Checkout drafts as editable MDX

```sh
cargo run -p leafbud -- import --all --state draft --root /home/rektide/l
cargo run -p leafbud -- import EDITOR_URL --to drafts/chosen.mdx --json
cargo run -p leafbud -- --root /home/rektide/l check drafts/chosen.mdx
```

This fetches native editor content, unlike metadata-only `remote show`.
Each document is one current `.mdx` file with YAML frontmatter, durable source
keys, native bindings and scoped hashes—no JSON siblings, CRDT history or base
snapshots. Default filenames combine a friendly title with the token UUID;
binding-based discovery recognizes repeated imports and renamed files.

The implemented profile covers linear paragraphs/headings/quotes, strict Yrs
rich-text marks/breaks, exact code text, images with native source/fallback/
dimensions/alt, page links/definitions and watermark metadata. Ordinary prose,
links, strong/emphasis/strikethrough and code spans render as **Markdown
first**; each block's candidate is proven through the production parser
before use, and anything not faithfully representable (underline/highlight
marks, awkward edge whitespace, bare URLs that would autolink, conflicting
link marks) keeps its exact `Run` component inside the same block, with
explicit typed block wrappers only where a block's candidate fails
validation. Code blocks use an exact `Code` component. `check` validates
this bounded profile, not every destination MDX feature. Images retain
remote URLs; no offline binary asset cache is claimed.

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
unverified. No remote save or publication was performed. Those files predate
format `leafbud-mdx/2` and are no longer parseable; local checkout data is
disposable, so a deliberate fresh `import` replaces them when wanted.

Current implementation/verification handoff:
[`design/draft-checkout0.gpt61s.md`](design/draft-checkout0.gpt61s.md).

## Read-only local/editor status

```sh
cargo run -p leafbud -- status                         # configured workspace
cargo run -p leafbud -- --root /home/rektide/l status --offline --json
cargo run -p leafbud -- status drafts/chosen.mdx --json
```

No operands selects local managed MDX files, not remote discovery or a new
`list` command. Operands resolve against the workspace; absolute operands are
also supported. Status never writes bodies, keys, receipts or remotes—even
when current endpoints have converged. It does not compare text or explain
an operation plan; [human diff](#human-localeditor-diff) is a separate view.

For supported bound drafts, status reports `clean`, `local-changed`,
`remote-changed`, `both-changed`, `converged` or `unverified`, with independent
local/remote change evidence. Missing/inaccessible native objects, unmanaged
operands, **unbound** local drafts, bound-without-receipt documents,
unsupported profiles/lifecycles/content, identity changes,
invalid files, duplicate document IDs/bindings and local read races are explicit
per-file outcomes. Unbound drafts report `needs-preparation` counts without
claiming durable identity. Missing does not prove deletion. Other origins are
not retargeted: select the bound `--base-url` explicitly.

Offline status requires no credentials or network and **never says clean**:
remote change/equality remain unknown, even when the stored local input matches.
Online `current` means fetched during this invocation, not a transaction/CAS;
metadata-only, checked-missing, unavailable and not-requested evidence are
distinguished. Naming witnesses are `not-tracked`; release equality is
`unknown-not-implemented`. Local-only author metadata is not compared to the editor.

JSON schema `version: 1` has `workspace`, `offline`, `selected`, `failed`,
`ignored_unmanaged`, `names`, `release` and `items`. Each item carries its
`path`, `outcome`, safe `detail`, `local` (`unchanged`/`changed`/`unknown`),
`baseline`, `receipt_profile`, nullable `endpoints_equal`/`document_id`,
`needs_preparation`, and
`remote` (`freshness`, `change`, nullable `observed_profile`/`observed_hash`).
Current observation hashes name their scope; change evidence uses the stored
receipt's interpretation. Neither human nor JSON reports include document
bodies, raw remote responses or credentials. Exit 0 includes successfully
described conflicts, uncertainty, missing remotes and unsupported profiles;
unreadable/invalid/ambiguous/racing files or failed remote reads give 1 when all
items fail, 4 when mixed. Usage errors give 2.

**Editor receipts:** new imports and verified pull settlements use
`leaflet-editor/3`. Local hashes cover supported editor content/metadata and
object/occurrence associations—not custom author fields, publication date,
delivery/naming intent, source-key spelling or non-entry page-definition layout.
Title, description, tags, publication association, watermark and contributors
remain editor-owned. Remote hashes conservatively cover current meaningful
fact payloads/unknown fields, extras and mapped metadata; unsupported conversion
does not erase observed evidence. Unknown content still blocks adoption.

Confirmed native associations are spelling-independent; new keyed objects carry
their explicit source identity, and locally retired associations remain visible
as omissions. Only current format/profile is supported: historical
`leaflet-import/1` and `leaflet-editor/2` receipts are not migrated or reinterpreted.
A current bound document with **no receipt** has no historical direction:
differing endpoints stay preserved; equal verified endpoints can settle through
pull/watch without rewriting the body. Status never settles receipts.
No saved body, ancestor, sidecar or CRDT snapshot was added.

**Last verified live rename gap** (status implementation session): the 27 files in `~/l` use
`springleaf-mdx/1`/`@springleaf`, and config/auth stores remain under the old
`springleaf` name. Online/offline status with explicit `--root ~/l` accounts
for all 27 as unsupported legacy managed files, with remote evidence
not-requested/offline—not a false clean report. Import refuses to create
duplicates alongside that namespace. No live file/store conversion or receipt
settlement was performed; all 27 byte hashes and nanosecond mtimes stayed equal.
Resolve namespace and store conversion explicitly before expecting fresh
editor status over those files. Configured workspace defaults are independently
tested with the current namespace.

Implementation/verification handoff:
[`design/status-witnesses0.gpt61s.md`](design/status-witnesses0.gpt61s.md).

## Human local/editor diff

```sh
leafbud diff drafts/chosen.mdx --tool diff --tool-arg=-u --tool-exit diff

# Or persist trusted viewer configuration (global, or add --workspace-file):
leafbud config set diff-program diff
leafbud config set diff-args '["-u"]'
leafbud config set diff-exit diff
leafbud diff drafts/chosen.mdx                 # --edge editor is the default
```

This compares **current local authored content → current bound native editor**,
not a saved base, public/released document, proposed operation or merge. It shares
status/pull's read-only observation and strict supported draft profile. Both sides
are rendered by the same normalized view writer; original MDX bytes are never
compared asymmetrically against generated remote output. No keys, receipts, local
source or remotes are written, including when content has converged. Missing or
inaccessible editors, unsupported content/lifecycle/namespace, incoherent or
duplicate bindings and a local save during acquisition stop comparison with an
error—never a partial or invented empty endpoint. Fresh means acquired in this
invocation, not transactional freshness through the lifetime of an open viewer.

**View contract.** The two transient UTF-8 files contain normalized YAML for title,
description, ordered tags, confirmed publication association (including null),
watermark theme and contributors; then distinct linear pages and ordered blocks.
Prose, headings, quotes, links, strong/emphasis/strike and inline code reuse the
existing checked Markdown-first renderer. Underline/highlight, difficult whitespace
or mark combinations, ineligible code text/language and image source/fallback/dimensions/
alt retain exact typed `Run`/block components where ordinary Markdown cannot carry
the supported meaning. Code uses a collision-safe fence only when the production
parser proves exact language/text preservation; otherwise it stays a `Code` component.
Hard breaks and empty blocks remain explicit when needed.
This is readable Markdown with exact MDX fallbacks, not a plain-Markdown export or
an executable document. No MDX is evaluated and linked resources are not fetched.

The view omits the whole tool envelope: receipts, hashes, UUID bindings, source
`Key` markers and source-key spelling. Page definitions receive presentation-only
`page-1`, `page-2`, … labels; the entry is first, other pages sort by native page
association (not printed), and page links refer to those labels. Reordering source
definitions or renaming keys therefore creates no writing differences. Ordinals
are not durable identities; adding/removing pages can renumber later ones.
Local-only dates, delivery/naming intent and custom/unknown author metadata are
excluded **on both sides**, not presented as if the editor lacked them and never
removed from the source. Authored links, image URLs, contributor DIDs and publication
URIs remain content, even if they contain identifiers. View equality does **not**
prove editor/receipt equality: status can still report identity or hidden-native-
state changes when visible writing is equal. Nothing here settles that evidence.

**Tool contract.** `diff-program` is one trusted executable name (PATH lookup) or
path; `diff-args` is a TOML string array, not a shell command. `config set diff-args`
accepts a JSON string array and writes the corresponding TOML array. Invocation is
exactly `PROGRAM ARG... /private/temp/local.md /private/temp/editor.md`, left local,
right editor. Arguments, spaces and punctuation remain literal: no shell expansion,
placeholders or quoting parser. The child runs in the selected workspace; relative
executable paths and argument paths resolve there. Config layers remain flag >
workspace > global. `--tool PROGRAM` overrides only the program; repeatable
`--tool-arg ARG` replaces the configured argument array when supplied;
`--tool-exit success|diff` overrides the exit convention. To clear saved arguments,
set `diff-args` to `[]`. `config show` includes each setting's provenance.

`success` (default) accepts only tool exit **0**; `diff` explicitly supports the
POSIX diff convention **0 equal, 1 different**, with both successful Leafbud
reports (exit **0**). Neither the executable's name nor its exit code determines
Leafbud's own normalized-view equality report. Other codes, a signal or a
spawn/wait/cleanup failure give Leafbud exit **1**; usage errors give **2**.
`--edge names/release` is explicitly unimplemented (exit 1). There is deliberately
**no `--json`**: stdin/stdout/stderr/TTY are inherited for CLI or interactive tools;
the tool owns stdout and Leafbud emits only a short execution summary on stderr.

Use a **foreground/waiting viewer**. For example, Vim's diff mode can use
`--tool vim --tool-arg=-d --tool-exit success`; a GUI launcher needs its documented
wait option. Leafbud owns a private temporary directory until that process
exits, then removes both files on success or tool failure. Drop cleanup also covers
ordinary spawn/wait errors; hard process termination can leave temporary files
(no automatic pruning or secure-erasure promise). Detached viewers are unsupported:
their paths disappear when the launcher exits. Editing a view never edits the source
or editor. Treat configured programs/workspace config as trusted code: the child
receives private writing and the normal inherited environment, and its own backups,
caches, rendering, resource access or side effects are outside Leafbud's control.

Implementation and verification: [human diff handoff](design/human-diff0.gpt61s.md).
The [last verified live rename gap](#read-only-localeditor-status) remains a separate
integration prerequisite; this capability was exercised on synthetic current-format
fixtures/loopback, not by converting `~/l` or its stores. A separate user session now
owns the further Leafbud rename and existing-data/config conversion; this diff slice
does not perform or promise that migration.

## Refresh bound drafts from their editors (guarded pull)

```sh
cargo run -p leafbud -- pull --all --root /home/rektide/l
cargo run -p leafbud -- pull drafts/chosen.mdx --dry-run --json
cargo run -p leafbud -- pull drafts/chosen.mdx --take remote
```

`pull` refreshes **existing bound local files** from their current native
editors; `--all` selects the workspace's managed files for this origin
(optionally filtered by local `--state`), never remote discovery — adopting
new remote documents remains `import`. Each file is classified against its
stored receipts using current endpoints only (no base snapshots, no CRDT
history, no server CAS):

| situation | outcome |
| --- | --- |
| Scoped local and remote unchanged | `unchanged` — exact no-op (bytes and mtime untouched) |
| Remote-only change (editor content or metadata) | `updated` — adopted; body rewritten only when page content actually changed |
| Remote change outside the supported projection (e.g. latent inactive text) | `receipts-only` — witness settled, author text and body untouched |
| Both changed to the same thing | `converged` — receipts settle without a body rewrite |
| No receipt, differing current endpoints | `needs-decision` — no historical baseline inferred (unknown profiles fail) |
| Local-only change, or diverged local and remote edits | `needs-decision` — file preserved verbatim |
| Missing remote, published lifecycle, unsupported content, broken bindings | explicit per-file outcome; nothing is pruned or created |

`--take remote` deliberately replaces differing **editor-owned** local content
(keeping local-only metadata fields, comments and manual key renames); it never
changes bindings or ignores unsupported content. Source identities are stable:
imported deterministic keys are first-association choices, key renames survive
refresh wherever the binding identifies the same remote object, and document
IDs never re-mint. Writes re-verify remote identity after content acquisition
and use the [cooperative lock and original-byte guard](#connected-inbound-watch)
against local races. Atomic replacement is **not** filesystem CAS against an
uncooperative editor. Exit codes follow the accepted contract: 0 success/no-op,
1 operational failure, 3 needs-decision, 4 partial/diverged batches.

Before the project rename, the live 27-file workspace refreshed as an exact no-op: full repeat run after
landing reported **27 unchanged, 0 writes**, with byte and nanosecond-mtime
equality across all files.
The current live namespace/store gap and new receipt interpretation are recorded
in [status](#read-only-localeditor-status); that historical no-op is not a claim
that the renamed binary has migrated live data.

Current implementation/verification handoff:
[`design/guarded-pull0.glm53m.md`](design/guarded-pull0.glm53m.md).

## Connected inbound watch

```sh
leafbud --root /path/to/current-checkout watch drafts/chosen.mdx
leafbud watch --all --json                     # versioned JSON Lines, not one JSON array
leafbud watch --all --presence                 # passive advisory observation only
leafbud watch --all --backstop 300 --rescan 5   # seconds; these are the defaults
```

`watch` maintains an actual origin-scoped Leaflet connection and adopts durable
saved editor edits into existing **current-format bound drafts**. It never pushes,
prepares keys, creates remote objects, imports undiscovered documents or prunes
missing files. Local-only edits are normal `local-dirty`; divergent edits become
`needs-decision`. Invalid, unknown owned state, pending recovery evidence,
unsupported content, unbound sources, duplicates and unavailable remotes stay
explicit and preserved. No persistent `--take remote` exists: use a separate
one-shot `pull FILE --take remote` for a deliberate current-content decision.

Initial live catch-up waits for acknowledged topic join, then uses the same
typed guarded refresh as pull: complete metadata plus null-cookie facts, identity
and lifecycle checks, native-association key reconciliation, verified receipts and
atomic local replacement. Pokes coalesce; a poke during a read queues another
pass. Rejoin catches up after gaps. Reads are serial and round-robin, with bounded
per-file retry backoff so busy/erroring documents do not stall other files or the
socket. The slow backstop reconciles missed events and metadata; if a subscription
or bootstrap is unavailable, its HTTP reads are labeled **`degraded-poll`**, never
a healthy live connection. Discovery/read/leave waits and retries are bounded.

Rescans follow explicit selections by stable local UUID through workspace renames;
`--all` follows the local inventory for this origin, including loud broken managed
files and new local bindings. A removed source is not recreated, a replacement
identity at an explicit old path is not silently selected, and duplicate UUIDs or
bindings block adoption. External explicit operands work, but rename discovery
outside the workspace is not recursive. New remote discovery remains `import`.

**Locks and races.** Push, pull, watch, tidy and import cooperate through OS-released
advisory locks at `$XDG_CONFIG_HOME/leafbud/locks` (else `~/.config/leafbud/locks`),
keyed by stable local UUID, not filename. Import also serializes one origin/token
binding before checking whether it already exists. A busy one-shot operation
reports a preserved per-file failure; watch backs off and retries. New local
drafts use random identity plus no-clobber creation. Lock files deliberately remain;
their presence says nothing about ownership, and no stale-owner cleanup runs.
Use the same user config directory for cooperating processes. Each transaction
rereads/revalidates selection after acquisition and holds its lock through remote
observation/adoption. External editors need not cooperate: a source save across
awaits aborts replacement through the byte guard, but the final check-to-rename
window is not an atomic filesystem compare-and-swap.

**Reports and interruption.** `--json` emits `version: 1` JSONL transitions of
types `connection`, `subscription`, `inventory`, `document`, optional `presence`,
then `summary`, each with local `observed_at` time (SystemTime epoch seconds/nanos).
Document events carry path, nullable local UUID, outcome, safe message and
`acquisition` (`live`, `degraded-poll`, or `inventory`), not remote root IDs or bodies.
Repeated unchanged dispositions are quiet; writes and actual state transitions
are reported. The summary contains refresh/write/cancel counts and latest selected
file outcomes. Ctrl-C/SIGTERM abandons pending HTTP safely, releases locks, performs
bounded best-effort topic leaves and exits 0 with a summary. Per-file problems are
reported state, not a session-wide operational exit; startup/reporting errors exit 1,
usage errors 2. A stopped summary does not mean every file was clean or observed.

**Bootstrap and presence.** Default discovery reads bounded same-origin public
Next assets using a separate cookie-free client. For deployments that need an
explicit override, set both `LEAFBUD_REALTIME_PROJECT_URL` and
`LEAFBUD_REALTIME_ANON_KEY`, or use `--project-url URL --anon-key PUBLIC_ANON_JWT`.
No project URL/key is hardcoded. Public topic access is not HTTP authorization;
normal Leaflet cookie credentials remain independently scoped to that origin.
`--presence` never tracks Leafbud as a peer or sends awareness/hello/content.
It reports `unknown`, `no_peers_observed`, or `{"observed_peers": n}` in JSON, with observation
time and subscription status. These are observed connections, not unique people,
write authority or proof that a document is idle; quiet roots keep poke subscriptions.

Production HTTP/WebSocket loopback tests cover actual poke-to-file adoption,
in-flight pokes, reconnect, local saves, preserved conflicts/state, fair retries,
selection changes and locking. A separate passive deployed CLI smoke freshly
imported one current-format scratch draft, maintained live readiness through a
23-second window, observed fresh empty presence, and preserved bytes/mtime on
unchanged catch-up. It made no remote edits and observed no deployed poke; deployed
poke-to-file adoption is **not claimed**. `~/l` remains untouched and format-stale.
Receipt and reusable APIs: [connected watch](design/connected-watch0.gpt61s.md).

## Markdown-first rich text and deliberate tidy

```sh
cargo run -p leafbud -- tidy --all --markdown --dry-run --json --root /home/rektide/l
cargo run -p leafbud -- tidy drafts/chosen.mdx            # default: all implemented passes
cargo run -p leafbud -- tidy drafts/chosen.mdx --markdown # this pass alone
```

Import and genuinely changed pull bodies now emit ordinary Markdown
(`[label](destination)`, `**strong**`, `*em*`, `~~strike~~`, backtick code
spans, backslash hard breaks) wherever the typed meaning survives the
production parser unchanged. Each block's candidate is validated in its real
Page/Key container through the same reader; ineligible runs stay exact `Run`
atoms (underline/highlight, control or awkward-edge whitespace, bare URLs the
GFM autolink pass would silently link, conflicting link marks, unescapable
destinations), and a failing block keeps its literal typed wrapper without
downgrading its neighbors. The composed document must still round-trip
exactly or nothing is written.

`tidy` normalizes **existing** files deliberately: parser-derived source
spans only (frontmatter bytes, comments outside replaced spans, Key markers
and Page boundaries are preserved; body MDX comments or unknown components
make the file fail loudly rather than partially rewrite), inserted syntax
keeps the file's line-ending convention, BOM/CRLF files work, witnesses and
stale receipts are never touched (formatting never accepts local edits), a
second tidy writes nothing, and writes are atomic under the original-byte
guard. `--all` selects the workspace's managed files; unmanaged files are
ignored. `--keys` assigns missing durable source identities (see
[authoring](#local-authoring-create-check-and-key-preparation)); `--from`
and `--accept` (naming reconciliation) are not implemented yet and are
rejected with exit 1 instead of half-running. Exit codes: 0 success/no-op,
1 all failed, 4 mixed batches (2 usage).

A read-only `tidy --all --markdown --dry-run` over the (then-format-1) live
27-file workspace predicted 26 files normalizable (281 blocks) and 0
failures; one file was already clean and one block was kept exact as a
candidate-mismatch. That prediction predates format 2; the pass mechanics
are unchanged and re-verified by the current test corpus.

Current implementation/verification handoff:
[`design/markdown-roundtrip1.glm53m.md`](design/markdown-roundtrip1.glm53m.md).

## Build and check

```
cargo build
cargo run -p leafbud -- --version
cargo run -p leafbud -- push essay.mdx   # -> "not implemented", exit 1
```

Checks: `cargo fmt`, `cargo clippy --workspace --all-targets`,
`cargo nextest run`. Regular tests are offline (loopback HTTP responders,
committed fixtures under `experiments/`) and never contact real drafts/PDSes.
Ignored deployment harnesses require separate explicit authorization/private
records; both original creation slots are consumed. The latest authorized harness
is create-free selected-P1 clear-intent settlement, with its passing result documented above.

## Reading map

- Accepted interface (CLI, crates, YAML policy):
  [`design/authoring-interface0.gpt6a.md`](design/authoring-interface0.gpt6a.md)
- Current write-side design (complete drafting with stable keyed blocks):
  [`design/keyed-authoring0.gpt6a.md`](design/keyed-authoring0.gpt6a.md)
- New/changed/dropped block flows and reordering options:
  [`design/keyed-edit-flows0.gpt6a.md`](design/keyed-edit-flows0.gpt6a.md)
- Proposed code boundaries, bounded bulk fallbacks and build path including
  persistent live watch and exploratory peer presence:
  [`design/keyed-build-path0.gpt6a.md`](design/keyed-build-path0.gpt6a.md)
- Governing handoff incl. keyed bidirectional addendum and CLI arena:
  [`design/authoring-handoff0.gpt61s.md`](design/authoring-handoff0.gpt61s.md)
- YAML frontmatter envelope: [`design/frontmatter0.gpt61s.md`](design/frontmatter0.gpt61s.md)
- Reset inventory and next slice: [`design/scaffold-reset0.glm53m.md`](design/scaffold-reset0.glm53m.md)
- Implemented read-only slice: [`design/remote-reads0.gpt61s.md`](design/remote-reads0.gpt61s.md)
- Config restoration and actual MDX checkout: [`design/draft-checkout0.gpt61s.md`](design/draft-checkout0.gpt61s.md)
- Guarded refresh of bound drafts: [`design/guarded-pull0.glm53m.md`](design/guarded-pull0.glm53m.md)
- Scoped editor witnesses and read-only status: [`design/status-witnesses0.gpt61s.md`](design/status-witnesses0.gpt61s.md)
- Transient human Markdown/external-tool diff: [`design/human-diff0.gpt61s.md`](design/human-diff0.gpt61s.md)
- Markdown-first rich text (approved design): [`design/markdown-roundtrip0.gpt6a.md`](design/markdown-roundtrip0.gpt6a.md)
- Markdown round-trip implementation receipt: [`design/markdown-roundtrip1.glm53m.md`](design/markdown-roundtrip1.glm53m.md)
- Offline create/key-preparation batch (format 2) receipt: [`design/author-create-keys0.glm53m.md`](design/author-create-keys0.glm53m.md)
- Corrected authoring foundation: [`design/authoring-corrections0.glm53m.md`](design/authoring-corrections0.glm53m.md)
- Production live adapter and receiving contract: [`design/live-transport0.gpt61s.md`](design/live-transport0.gpt61s.md)
- Integrated guarded live watch, locks and actual proof limits: [`design/connected-watch0.gpt61s.md`](design/connected-watch0.gpt61s.md)
- First-binding defaults, explicit selected-locator recovery and deployed partial
  boundaries: [`design/native-default-adoption0.gpt61s.md`](design/native-default-adoption0.gpt61s.md)
- Experiment reading map and evidence protocol: [`experiments/README.md`](experiments/README.md)

Never commit credentials or real response bodies; see
[`experiments/README.md`](experiments/README.md).
