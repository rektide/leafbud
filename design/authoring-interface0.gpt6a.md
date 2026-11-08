---
type: Design
title: "Authoring interface finalization — CLI, crates, and frontmatter"
description: "A focused decision proposal joining the keyed bidirectional handoff to a document-first CLI, domain-grouped Rust workspace and scoped YAML receipts."
resource: /design/authoring-interface0.gpt6a.md
status: proposal
generated:
  at: 2026-11-08
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Recovered the authoring handoff and proposed the remaining interface, crate and YAML decisions before build orchestration"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Controlling keyed bidirectional addendum and CLI capability inventory"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Concrete YAML candidate reviewed here"
  - resource: /design/document-sync1.gpt61s.md
    title: "Independent naming/editor/release evidence and current-endpoint resolution"
  - resource: /design/authoring-recommendation0.gpt61s.md
    title: "Satteri recommendation and declarative MDX language"
  - resource: /Cargo.toml
    title: "Current four-crate workspace"
  - resource: /crates/cli/src/main.rs
    title: "Current command tree and lifecycle orchestration in the binary"
  - resource: /crates/cli/src/config.rs
    title: "Existing Figment workspace/global layering"
  - resource: /crates/drafts/src/lib.rs
    title: "Web client already serves both draft and published Leaflets"
  - resource: /crates/drafts/src/facts.rs
    title: "Current fact decoder discards opaque payloads, requiring deliberate replacement for robust witnesses"
  - resource: /experiments/13-satteri-authoring/src/model.rs
    title: "Existing transient model includes nested lists and keyed content, but not a complete production binding schema"
---

# Authoring interface finalization

## Starting point

The controlling target is the [handoff's keyed bidirectional addendum](authoring-handoff0.gpt61s.md#keyed-bidirectional-addendum):
one MDX file, automatically assigned durable identities, retained Leaflet/Yrs
objects on ordinary edits, useful pull, and explicit recreation when needed.
The parser recommendation is Satteri. The destination includes all current
Leaflet content; the first integrated profile can be smaller and must report
unsupported content explicitly.

This document proposes the remaining interface decisions for human review.
It is not an implementation report or evidence that native publication and the
combined bidirectional loop have been exercised end-to-end.

## 1. Recommended CLI

### Main document loop

```sh
springleaf create drafts/essay.mdx --title "An essay"
springleaf check drafts/essay.mdx
springleaf push drafts/essay.mdx               # plan
springleaf push drafts/essay.mdx --yes         # assign missing keys + save editor
springleaf publish drafts/essay.mdx --yes      # save as needed + release
springleaf pull drafts/essay.mdx              # adopt current editor changes
```

**Push saves; publish releases; pull reads the working editor.** These meanings
stay the same for draft and published files. Publication updates the same bound
document and leaves its filename alone. Paths are explicit file operands;
filenames never select remote identities.

Recommend automatic missing-key assignment during executed push/publish, plus
`keys assign FILE` for local preparation. A normal author need not run a separate
preparation command. `check`, status/diff and an unexecuted push do not write keys.
Execution persists the assigned identities before remote creation or mutation;
subsequent retries use those identities. Source changes during planning invalidate
the plan. No-op keyed input stays byte-identical.

### Command map

| Command | Job |
| --- | --- |
| `create FILE --title TEXT` | Create one local source, with format and document ID; no remote creation |
| `check FILE...` | Offline syntax, component, reference and identity validation; missing assignable keys are a preparation requirement |
| `list [--state draft\|published]` | Inventory local managed files in the selected workspace |
| `status [FILE...] [--offline]` | Naming, editor and release summaries; with no files, inspect the local workspace inventory |
| `diff FILE --edge editor\|names\|release` | Compare current endpoints; editor is the default; release compares the current publication projection with fetched live output |
| `push FILE... [--yes]` | Plan/execute identity-preserving editor updates and first binding |
| `pull FILE... [--take remote]` | Guarded refresh of bound files from their current editors; local write |
| `publish FILE... [--yes]` | Plan/execute editor synchronization and native release |
| `import LOCATOR [--to FILE]` | Adopt a remote document into one bound MDX file |
| `import --all [--state draft\|published]` | Discover and adopt remote documents into the workspace; report per-item outcomes and discovery completeness |
| `keys assign FILE` | Add missing source identities locally |
| `keys rename FILE OLD NEW` | Rename a source key and its references/binding together, retaining remote identity |
| `edit FILE --title TEXT / --description TEXT / --published-at TIME / --now` | Explicit local metadata edits; date alternatives are mutually exclusive |
| `tidy FILE --from filename\|path` | Deliberately reconcile file name and path intent |
| `tidy FILE --accept` | Witness an intentional name/path mismatch |
| `export FILE --format mdx\|markdown\|record-json` | Emit a representation, with explicit loss reporting; stdout unless `--to FILE` |
| `remote list [--state draft\|published]` | Discover remote Leaflets/posts, distinct from local `list` |
| `remote show LOCATOR` | Inspect one remote object and its native editing availability |
| `remote append LOCATOR --from FILE [--yes]` | Retained additive plumbing, if carried forward; never the ordinary update path |
| `pubs list`, `auth status`, `config show/set` | Retain publication-container discovery, auth diagnostics and configuration |

This is a destination map, not a requirement to implement every command in the
first slice. `keys` groups occasional identity operations; there is no separate
mandatory `prepare` verb. There is one ordinary save verb, `push`.

Bulk bound-file operations use `--all` instead of file operands, with the same
optional state filter. For import, `--all` selects discovered remotes; for
push/pull/publish it selects local managed files. Invalid or ambiguous bindings
remain per-item failures. Repeated imports find existing bindings and use pull
logic, rather than deriving fresh destinations from changed titles. Duplicate
local document IDs or remote bindings require a destination choice.

LOCATOR accepts a native editor URL, permission-token UUID, or document AT URI.
An AT URI requires verified native linkage for editor operations. An unbound
local document is different from a bound document whose remote has disappeared.
Neither a missing editor nor discovery omission automatically creates or deletes
anything. Published-record-only exports remain useful when editing is unavailable.

The old `publish build/run` and PDS `sync` must not compete with the new native
meanings. Reuse their record/transport capabilities under an explicit `pds` group
if retained; their old sidecar-based lifecycle is not a second authoring system.
Their exact low-level command surface can be specified with that separate work.

### Conflict and effect contract

| Observed editor edge | Ordinary push | Ordinary pull |
| --- | --- | --- |
| Unchanged | No content write | No body rewrite |
| Local-only change | Apply local edit | Report local edits; do not discard them |
| Remote-only change | Report remote edits; suggest pull | Adopt remote edit |
| Both changed | Report conflict | Report conflict |
| Current endpoints equivalent | Verify correspondence; settle witnesses without content writes | Same |
| Untracked/missing/unsupported | Explain required association, recovery or representation decision | Same |

Use `push --take local` and `pull --take remote` to deliberately replace differing
current content. Publish can carry `--take local` for its editor-save phase.
There is no persistent resolution document and no automatic historical merge.
Execution reads current endpoints, displays its scope, rechecks observed witnesses
before applying and verifies afterward. This is not server-enforced CAS.

Recommend `push FILE --recreate KEY` (repeatable), or `--recreate-body`, for
explicit recreation. Neither flag resolves remote drift automatically. The plan
names the selected scope and lost associations. Recreation of interactive objects
still follows their separate identity/meaning policy.

Retain the handoff's preview-by-default remote writes and `--yes` execution.
`--yes` approves execution, not conflict resolution, retargeting or lossy conversion.
Local-writing commands execute their stated local operation; `--dry-run` previews
those operations. Existing destination collisions need `--overwrite`; that flag
does not bypass a tracked document's pull conflict. `--force-downconvert` applies
only to reported export loss, never to an ordinary managed push or pull.

Recommend exit codes: **0** successful report/plan/no-op/completed operation;
**1** operational failure; **2** CLI usage error; **3** requested operation needs
a decision or supported representation; **4** partial/diverged/uncertain execution.
Status/diff succeed when they successfully describe a conflict; action commands
blocked by that conflict use 3. Machine output carries a version and per-item
outcomes, with diagnostics on stderr. Offline status labels remote state unknown
and can report only local changes relative to stored receipts.

## 2. Recommended domain-grouped workspace

Prefer a **document domain, Leaflet integration domain, protocol client, and
application workflow**, with the CLI as a thin caller:

```text
crates/
  document/
    model/        # current content, stable source IDs, diagnostics, tracking types
    mdx/          # Satteri, YAML, key insertion, MDX reading/writing
  leaflet/
    editor/       # fact graph, Yrs, bidirectional mapping, pure edit planning
    web/          # cookie auth, discovery, RPC/actions, uploads, native publication
    records/      # generated public types, validation, public-record projection
  atproto/        # existing public reads, identity, explicitly selected PDS writes
  workflow/       # local files, config, inventory, plan/execute/verify/recovery
  cli/            # Clap, rendering, exit mapping
```

Eight crates, not one crate per module. Package names can remain descriptive
(`springleaf-document`, `springleaf-mdx`, `springleaf-editor`,
`springleaf-web`, `springleaf-records`, `springleaf-atproto`,
`springleaf-workflow`, binary package `springleaf`). Directory names do not repeat
the project prefix.

Dependencies point from consumers to their lower-level dependencies:

- `document/model` is network/parser independent. It owns current authoring
  semantics and compact tracking data, including explicitly Leaflet-specific
  locators where needed; it is not a promise of a universal document SDK.
- `document/mdx` depends on the document model. It retains source bytes and
  produces validated models or diagnostics and proposed byte edits. It does not
  perform filesystem/network effects or expose Satteri arenas as the domain API.
- `leaflet/editor` depends on the document model and Yrs. It owns graph and text
  decoding, bindings, mapping capability, scoped witnesses and edit plans. It
  accepts already-fetched state and supplied resource results.
- `leaflet/records` owns public types and document-to-public projection, depending
  on the document model where needed. Editor facts are not public records.
- `leaflet/web` depends on editor wire/operation types as needed. It executes
  native requests and returns observations/outcomes. It does not parse MDX or
  load workspace files. Public reads use the ATProto client where appropriate.
- `workflow` composes these libraries: source acquisition, configuration,
  planning, resource acquisition, pending receipts, execution and verification.
  It offers the same operations to the CLI and other Rust callers.
- `cli` selects operations and renders results. Business policy and file lifecycle
  leave today's large command implementation modules.

Keep raw unknown remote fact payloads available in transient observations. The
current `FactData::Other { kind }` drops the payload and cannot establish the
conservative opaque-content witness described in the designs. A typed semantic
view can coexist with those raw current observations without saving a snapshot.

**Smaller plausible alternative:** six crates at
`crates/{authoring,editor,web,records,atproto,cli}`, with model/MDX modules inside
authoring and workflow modules inside a library target of the CLI package.
It reduces crate administration but gives weaker parser-dependency isolation and
an awkward home for reusable application operations. Prefer the eight-crate
domain-grouped form for the intended full bidirectional product. Assets, hashing,
keys, publication and configuration begin as modules in their owning domains.

Existing `drafts` splits into editor/web responsibilities; existing `leaflet`
becomes records rather than keeping a misleading umbrella name. These are
destination boundaries, not a mandate to move every existing module before the
first end-to-end slice. Preserve isolated experiments as evidence.

## 3. YAML: retain the envelope, tighten its contracts

The [frontmatter proposal](frontmatter0.gpt61s.md) has the right overall shape:

```yaml
---
format: springleaf-mdx/1
title: An essay
tags: [notes]

"@springleaf":
  id: "11111111-1111-4111-8111-111111111111"
  state: draft
---
```

Keep flat author fields, camelCase, one whole-file format version and quoted
`"@springleaf"`. The envelope grows only with actual binding/receipt needs.
Keep `binding`, `nodes`, `sync`, `released`, optional `names`, `assets` and `pending`
as distinct concerns. A published binding with no native editor locator is valid
but editing-unavailable. A published URI does not require a known local release
receipt; an imported public document may have an unknown release baseline.

### Corrections before freezing schema v1

1. **Scope each fingerprint.** `sync.localHash` covers the supported *editor
   projection*, source associations and inputs relevant to that projection.
   `sync.remoteHash` covers the corresponding owned editor scope, conservatively
   including opaque meaningful state. A change only to publication date,
   delivery preference or filename must not become an editor conflict. Fields
   belong in both fingerprints only when both operations actually consume them.
   `released.inputHash` has its own `profile` and covers resolved publication
   content/metadata and resource digests, excluding source-key spelling, layout
   trivia and delivery-attempt bookkeeping. Named profile fixtures define exact
   normalization and field ownership.
2. **Treat missing/null publication as three-way input.** Missing means inherit
   while unbound; null explicitly means standalone; a URI selects a publication.
   Missing bound association means unknown, not standalone. Ordinary Rust
   `Option<T>` decoding commonly collapses absent and null: use an explicit
   presence representation. Bound identity takes precedence over defaults.
3. **Validate an editor locator as a coherent group.** A partial collection of
   token/root/entity-set/entry values is not a usable writable binding. Native
   discovery may establish the group; a URI-only published binding remains
   representable. The account auth cookie never goes in document YAML.
4. **Give placements an identity policy.** The proposed `memberships: [UUID]`
   stores locators but does not explain which source occurrence owns each one.
   A single-placement node is straightforward. Repeated references, nested lists
   and generated containers need a worked binding fixture before their field
   shape is frozen. Prefer occurrence-keyed relationships or typed role mappings
   over interpreting array index as stable identity. A page reference's occurrence
   key and its target page key do different jobs.
5. **Distinguish current binding rows from pending creation.** New source IDs can
   exist without a confirmed remote row. Keep planned/known creation IDs in
   `pending` until verified; promotion to `nodes` follows read-back. A vanished
   source key does not prove a manual rename: report remove/add effects, and use
   `keys rename` to preserve an intentional association. A copied file retaining
   its document ID/binding is the same managed document until explicitly forked.
6. **Keep native normalization out of perpetual drift.** Publication planning
   resolves path/date and describes normalization. On confirmed success, adopt
   accepted normalized author values and fingerprint that resolved input, rather
   than leaving a file that immediately reports unreleased edits. Naming witnesses
   remain independently settled. An unknown imported release baseline is reported
   as unknown, not fabricated from today's editor.
7. **Separate delivery outcome from release equality.** A record CID proves
   neither email nor Bluesky delivery. Persist delivery evidence only where the
   implemented operation can establish it; unknown outcomes remain unknown.
   A stored preference is not an instruction to repeat delivery on every update.
   The exact `delivery`, `cover`, `theme` and asset descriptor vocabularies need
   their real mapping contracts before those fields are accepted for application.

### YAML writing recommendation

Preserve the body and author frontmatter text during routine tool-state updates;
replace or insert only the reserved envelope. Explicit metadata edits patch their
selected author fields. The generated envelope itself can use stable normalized
formatting. This gives useful comment/style preservation without promising a
general MDX formatter or byte-preserving editor for every YAML construct.

Use the proposed restricted YAML profile: string mapping keys, ordinary scalar
types and sequences, no duplicate keys, aliases/anchors, custom tags or merge
keys. Whole-file validation precedes patching. Locate ranges through a parser or
token/source-position facility, not regex boundary guesses that can misread a
multiline scalar containing `"@springleaf":`. The current semantic YAML parser
proof does not yet prove this writer. If bounded source-preserving patching
cannot be implemented cleanly, bring back the whole-frontmatter normalization
tradeoff explicitly.

Unknown author fields survive and are reported as non-applied. Known reserved
fields such as `theme` must not silently pass as applied when unsupported.
Unknown owned-state fields are preserved, and unrecognized state semantics block
managed mutation rather than being rewritten speculatively. No-op writes touch
nothing. File replacement checks the original bytes/revision after network waits;
it must not overwrite an intervening user edit to record a remote success.

## 4. Decisions and the first build join

Recommended human selections:

1. Root-level push/pull/publish, local `list` versus `remote list`, and `keys`
   helpers; retain preview-by-default remote execution.
2. Automatic key insertion on executed push/publish, with optional local assignment.
3. Eight domain-grouped crates with reusable workflow outside the CLI.
4. Source-preserving author YAML patches with a normalized tool envelope.

Once selected, the first integration joins the existing MDX/keying and Yrs proofs:
editor fixture -> keyed MDX -> prose/mark/move edit -> same remote objects -> pull.
Add frontmatter presence, scoped fingerprints, no-op and local-edit-race cases,
and one nested/repeated-placement binding fixture. This fixes the remaining
data-shape details from executable evidence before broad implementation fans out.
Native publication and full content/media expansion then have explicit owners
and seams. Their current evidence gaps remain visible in the original handoff.

### Connections

- [Main handoff](authoring-handoff0.gpt61s.md): governing intent and experimental
  reuse; this proposal supplies one receiving interface rather than reviving its
  superseded recreation default.
- [YAML candidate](frontmatter0.gpt61s.md): retained envelope and lifecycle,
  refined here for projection-specific hashes, placement identity and writing.
- [Earlier sync design](document-sync1.gpt61s.md): precedent for independent
  naming/editor/release witnesses and explicit current-endpoint resolution.
- [Current README](../README.md): implemented CLI, distinct from this proposal.
