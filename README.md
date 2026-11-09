# leafbud

Local-first MDX authoring for [leaflet.pub](https://leaflet.pub)
documents, in Rust: one MDX file per document, durable identities,
push saves the Leaflet editor, publish releases, pull adopts web edits.

Status: **Markdown-first rich text with checked fallbacks, deliberate
`tidy --markdown`, scoped read-only status, external-tool human diff, guarded pull, layered config, version 0.9.0.** The accepted authoring interface
([`design/authoring-interface0.gpt6a.md`](design/authoring-interface0.gpt6a.md))
settles root CLI verbs (`create`/`check`/`list`/`status`/`diff`/
`push`/`pull`/`publish`/`import`/`edit`/`tidy`/`export`, plus
`keys rename`, `remote`, `pubs`, `auth`, `config`), all-applicable
`tidy` with `--keys` narrowing, eight domain-grouped crates, and
operation-specific YAML preservation. The workspace now matches that
destination shape. Implemented versus pending:

| implemented | not implemented |
| --- | --- |
| `leafbud-atproto` — retained identity resolution, XRPC, app-password sessions; `leafbud-records` — pinned record types/validation/helpers | native saves/publication and generalized managed push |
| `leafbud-web` — cookie-authenticated discovery, metadata and full editor pull; `leafbud-editor` — raw-fact scanning and strict Yrs text decoding | canvas, lists, interactive/other unsupported editor content |
| `leafbud-document` + `leafbud-mdx` — bounded current-document model, durable bindings, real Satteri/YAML parse, loss-checked keyed MDX writer with Markdown-first rich blocks and block-local exact fallback, source-preserving updates and parser-derived Markdown tidy planning | generalized authoring model; key assignment and naming tidy passes |
| `leafbud-workflow` + CLI — `remote list/show`, draft `import`, imported-profile `check`, read-only `status [FILE...] --offline/--json`, `diff FILE --edge editor` via configured external tool, guarded `pull`, `tidy FILE.../--all` with `--markdown/--dry-run/--json`, Figment2 config show/set and shared workspace root | local `list`, `create`, `push`, `publish`, `edit`, `export`, `keys`, `pubs`, `auth` commands; `diff --edge names/release`; `tidy --keys/--from/--accept` selectors (rejected honestly until built) |

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
                bounded pure observation; write planning still pending
    web/       leafbud-web: cookie auth, discovery, RPC/actions,
                read-only discovery/metadata/content pulls; writes pending
    records/   leafbud-records: generated pinned lexicon record types,
               validation, wire-shape helpers — see LEAFLET-PIN.md
  atproto/     leafbud-atproto: hand-rolled ATProto surface — handle→DID→PDS
               identity resolution, XRPC client, sessions, public reads
  workflow/    leafbud-workflow: local files, config, inventory,
                layered config, native reads, guarded draft import and pull
  cli/         leafbud (binary): thin Clap/rendering layer;
                 reads/import/check/status/pull/config implemented; native writes pending
design/        architecture, accepted interface, and format designs
experiments/   protocol probes and evidence (tracked, load-bearing)
```

Dependency direction (accepted design §2): pure document model →
MDX adapter / editor / record projections; `web` owns effects;
`workflow` composes the libraries; the CLI stays thin. The scaffold
adds no crate dependency edges that no code uses yet.

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
unverified. No remote save or publication was performed.

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
local/remote change evidence. Missing/inaccessible native objects, unbound or
unmanaged operands, unsupported profiles/lifecycles/content, identity changes,
invalid files, duplicate document IDs/bindings and local read races are explicit
per-file outcomes. Missing does not prove deletion. Other origins are not
retargeted: select the bound `--base-url` explicitly.

Offline status requires no credentials or network and **never says clean**:
remote change/equality remain unknown, even when the stored local input matches.
Online `current` means fetched during this invocation, not a transaction/CAS;
metadata-only, checked-missing, unavailable and not-requested evidence are
distinguished. Naming witnesses are `not-tracked`; release equality is
`unknown-not-implemented`. Local-only author metadata is not compared to the editor.

JSON schema `version: 1` has `workspace`, `offline`, `selected`, `failed`,
`ignored_unmanaged`, `names`, `release` and `items`. Each item carries its
`path`, `outcome`, safe `detail`, `local` (`unchanged`/`changed`/`unknown`),
`baseline`, `receipt_profile`, nullable `endpoints_equal`/`document_id`, and
`remote` (`freshness`, `change`, nullable `observed_profile`/`observed_hash`).
Current observation hashes name their scope; change evidence uses the stored
receipt's interpretation. Neither human nor JSON reports include document
bodies, raw remote responses or credentials. Exit 0 includes successfully
described conflicts, uncertainty, missing remotes and unsupported profiles;
unreadable/invalid/ambiguous/racing files or failed remote reads give 1 when all
items fail, 4 when mixed. Usage errors give 2.

**Editor receipts:** new imports and verified pull settlements use
`leaflet-editor/2`. Local hashes cover supported editor content/metadata and
object/occurrence associations—not custom author fields, publication date,
delivery/naming intent, source-key spelling or non-entry page-definition layout.
Title, description, tags, publication association, watermark and contributors
remain editor-owned. Remote hashes conservatively cover current meaningful
fact payloads/unknown fields, extras and mapped metadata; unsupported conversion
does not erase observed evidence. Unknown content still blocks adoption.

Existing `leaflet-import/1` hashes keep their old whole-document interpretation.
They cannot supply a new scoped historical baseline: differing endpoints are
`unverified`; verified equal supported endpoints are `converged`. An explicit
`pull` can settle that current correspondence into the new profile without
rewriting the body; status cannot. Unknown profiles block managed mutation.
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
| Legacy receipt, differing current endpoints | `needs-decision` — no scoped historical baseline inferred |
| Local-only change, or diverged local and remote edits | `needs-decision` — file preserved verbatim |
| Missing remote, published lifecycle, unsupported content, broken bindings | explicit per-file outcome; nothing is pruned or created |

`--take remote` deliberately replaces differing **editor-owned** local content
(keeping local-only metadata fields, comments and manual key renames); it never
changes bindings or ignores unsupported content. Source identities are stable:
imported deterministic keys are first-association choices, key renames survive
refresh wherever the binding identifies the same remote object, and document
IDs never re-mint. Writes re-verify remote identity after content acquisition
and are atomic against the original local bytes (a concurrent user edit aborts
the write). Exit codes follow the accepted contract: 0 success/no-op,
1 operational failure, 3 needs-decision, 4 partial/diverged batches.

Before the project rename, the live 27-file workspace refreshed as an exact no-op: full repeat run after
landing reported **27 unchanged, 0 writes**, with byte and nanosecond-mtime
equality across all files.
The current live namespace/store gap and new receipt interpretation are recorded
in [status](#read-only-localeditor-status); that historical no-op is not a claim
that the renamed binary has migrated live data.

Current implementation/verification handoff:
[`design/guarded-pull0.glm53m.md`](design/guarded-pull0.glm53m.md).

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
ignored. `--keys`, `--from` and `--accept` are not implemented yet and are
rejected with exit 1 instead of half-running. Exit codes: 0 success/no-op,
1 all failed, 4 mixed batches (2 usage).

A read-only `tidy --all --markdown --dry-run` over the live 27-file
workspace predicted 26 files normalizable (281 blocks) and 0 failures; one
file was already clean and one block was kept exact as a candidate-mismatch.
The user-indicated motivating link file converts its single href-only `Run`
to bracket Markdown with frontmatter bytes, Key markers and semantics
unchanged (verified on a scratch copy; no live file was rewritten).

Current implementation/verification handoff:
[`design/markdown-roundtrip1.glm53m.md`](design/markdown-roundtrip1.glm53m.md).

## Build and check

```
cargo build
cargo run -p leafbud -- --version
cargo run -p leafbud -- push essay.mdx   # -> "not implemented", exit 1
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
- Guarded refresh of bound drafts: [`design/guarded-pull0.glm53m.md`](design/guarded-pull0.glm53m.md)
- Scoped editor witnesses and read-only status: [`design/status-witnesses0.gpt61s.md`](design/status-witnesses0.gpt61s.md)
- Transient human Markdown/external-tool diff: [`design/human-diff0.gpt61s.md`](design/human-diff0.gpt61s.md)
- Markdown-first rich text (approved design): [`design/markdown-roundtrip0.gpt6a.md`](design/markdown-roundtrip0.gpt6a.md)
- Markdown round-trip implementation receipt: [`design/markdown-roundtrip1.glm53m.md`](design/markdown-roundtrip1.glm53m.md)
- Experiment reading map and evidence protocol: [`experiments/README.md`](experiments/README.md)

Never commit credentials or real response bodies; see
[`experiments/README.md`](experiments/README.md).
