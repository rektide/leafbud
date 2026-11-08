---
type: Proposal
title: "Document-centered leafspring — Sol's independent option"
description: "A file-first lifecycle with one active remote, two reconciliation edges, embedded witnesses, and alternative document encodings."
resource: design/document-sync0.gpt61s.md
status: draft
generated:
  by: model:openai/gpt-6.1-sol#high
  at: 2026-11-07
sources:
  - resource: "opencode://ses_e4900b8d7ffeDT9QuFwmKRAB7t"
    title: "User's redesign directions and Sol's independent architecture option; not implementation authorization"
  - resource: /design/records-canonical.glm53m.md
    title: "Previous pure-record and sidecar model, deliberately revised here"
  - resource: /design/records-canonical2.glm53m.md
    title: "Landed configuration layering"
  - resource: /crates/cli/src/publish.rs
    title: "Current filename-derived identity, put, move, and sidecar implementation"
  - resource: /crates/drafts/src/author.rs
    title: "Current append-only converter and explicit media omissions"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts"
    title: "Locally inspected native publishing, identity preservation, path normalization, dates, and side effects"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts"
    title: "Locally inspected publication action and server-chosen identity"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts"
    title: "Locally inspected removeBlock semantics, including image cleanup and nested content"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Earlier source and live-read evidence for pull/push, acknowledgments, and Yjs behavior"
  - resource: /design/document-sync0.gpt6a.md
    title: "Read only after the independent Sol option was committed; comparison and revised leans below"
---

# Document-centered leafspring — Sol's independent option

## Standing and intent

Independent proposal, not the final redesign or an implementation commission.
Astra is preparing a separate same-wave option; this document was written
without reading it. No live writes or new protocol probes were performed.

The user explicitly permits reconsidering the architecture and has no valuable
persisted data requiring migration. Do not preserve the previous sidecar format
or filename-derived identity through compatibility machinery.

**One local document, one active remote, two reconciliation edges:**

```text
filename <-> document path <-> Leaflet draft OR published post
```

The central operation is correspondence, not transport. A shared classifier
should explain the state of either edge; endpoint adapters supply the evidence
and execution. Similar user behavior does not imply identical server guarantees.

## Grounded constraints

Source inspected in the local Leaflet checkout at revision
`600c7b5f1f7b31b66fb626c293cc104312a12caf`:

- `publishLeaflet.ts:276-280,415-426` preserves an existing document's rkey,
  chooses a fresh TID on first publish unless a trusted caller supplied a key,
  and writes `path = "/" + rkey`.
- `actions/publishToPublication.ts:11-38` excludes caller-selected rkey and
  clamps a future publication date to now. A requested public path therefore
  cannot simply be promised to survive native publication.
- `publishLeaflet.ts:481-487,489-499` writes the PDS and then maintains local
  application linkage. `:624-659` handles the newsletter send slot on first
  publication, with `sendEmail` defaulting true at `:176`. A publish is not
  equivalent to a bare record put.
- The native publisher reads facts, converts pages, and can offload/truncate
  content (`:285-291,409-454`). Its returned application record and the actual
  PDS record need not be structurally identical.
- `mutations.ts:735-772` shows removal is more than dropping an ordered block
  reference: it may remove descendants, answers, and enqueue image cleanup.
  Wholesale replacement must account for that, not reuse the word "atomic"
  merely because mutations share one push.
- Our current JSON-to-draft converter explicitly skips image/gallery/drawing/
  embedded-canvas content and some styling. This is independent of Markdown.

These facts constrain adapters, not the document-centered user model. A native
publish transport, its action discovery, auth prerequisites, and verification
remain unbuilt and must be established before advertising that lifecycle.

## Local document and lifecycle

JSON contains the complete logical document plus a reserved `@leafspring`
object. Record-only serialization removes that object explicitly; test the
outbound object rather than relying on server rejection of unknown properties.
Metadata editing preserves unknown document fields rather than reconstructing
the payload from only the fields a command understands.

Directories remain useful lifecycle groupings:

```text
drafts/<friendly-name>.json
posted/<friendly-name>.json
```

The embedded tagged state is authoritative; the directory is its conventional
placement. A manual move does not silently publish or unpublish a document.
A directory/state mismatch is reported, and an explicit tidy operation can
repair placement. There is never a second active copy maintained in both dirs.

Suggested metadata responsibilities, not a frozen wire schema:

```text
@leafspring
  version
  id                         stable local identity, independent of all names
  state                      draft | published (tagged, mutually exclusive)
  naming
    base                     witnessed filename stem + document path
    observedAt               time of that accepted observation
  remote                     exactly one active adapter/link
    kind                     leaflet-draft | leaflet-post
    target                   draft token OR published AT URI
    editor                   optional locator for a post's editing machinery
    base
      localHash              document projection at last accepted sync
      remoteHash             observed remote projection at that point
      projectionVersion      specifies normalization and field coverage
      revision               CID or upstream revision if available
      observedAt
  operation                  optional pending write/transition receipt
```

Fresh local drafts have no remote link. A post may retain the editor locator
Leaflet requires for updating that same post; that locator is subordinate
transport machinery, not a second draft lifecycle or independently synced head.
If keeping editor and publication consistent requires extra checks, expose
them in the post adapter's plan rather than inventing a second active target.

A local UUID permits repeat imports and workspace scans to recognize documents
without filenames as keys. Remote identities also identify repeat imports.
Two files claiming the same local or remote identity are ambiguous: report them,
do not select a winner or prune automatically. An intentional fork creates a
new local identity and clears remote linkage explicitly.

## Two edges, one comparison vocabulary

Given witnessed endpoints `(L0, R0)` and current endpoints `(L, R)`, classify:

| Left changed | Right changed | Meaning |
|---|---|---|
| no | no | unchanged since accepted observation |
| yes | no | left changed |
| no | yes | right changed |
| yes | yes | both changed; test current equivalence before calling conflict |
| unknown | unknown | no usable baseline; untracked, not clean |

Missing endpoints are separate conditions: unlinked, absent remotely, or
missing locally. Remote absence never means automatic local deletion or
automatic resurrection. Times explain observations, not winner selection.

### Naming edge

Left is filename stem; right is the document's `path`, compared through a
declared name/path convention. Extension belongs to encoding, not slug.
Start with the common single-segment mapping; paths that cannot be represented
reversibly by that convention require an explicit destination, not sanitization
that silently changes the public path.

`tidy --from path` renames the file; `tidy --from filename` edits the document
path. Both preview the exact delta, collision-check, and acknowledge the new
naming pair only after success. Equal endpoints can establish an initial
baseline; unequal untracked endpoints need a direction. If both changed but
still correspond, acknowledge that convergence without calling it conflict.

Do not restamp naming baselines during an ordinary report, remote push, or
unrelated title edit. A deliberately unconventional filename is allowed; an
explicit accept-as-is action may witness the divergent pair without pretending
it follows the convention. Later endpoint changes remain detectable.

Changing document path marks the remote edge changed where that field is
covered. It does not change the bound remote identity. If the remote cannot
honor that path, show the unsupported change or native normalization rather
than minting a replacement post or silently treating a failed path change as
accepted.

### Remote edge

Left is the decoded document excluding local metadata; right is the active
remote's normalized view. Hashes are over a versioned deterministic logical
JSON projection: sorted object keys, preserved array order, no formatting or
envelope differences. This is a local fingerprint, not a claim to reproduce an
ATProto CID. Equivalent JSON and Markdown encodings should produce the same
fingerprint; unsupported conversion must not be mistaken for equivalence.

Each adapter declares field coverage and normalization. Editor facts do not
necessarily carry publication path/date; the native application record may
inflate pages absent from the PDS payload. Unsupported local fields stay local
and are listed, not quietly dropped by pull or counted as remotely confirmed.

Paired local/remote hashes acknowledge representation differences, but their
scope is the declared mapping. After a forced lossy write, report the accepted
loss and retained local-only content. "Unchanged" means no changes since that
accepted result, not proof of a lossless clone.

Hashes suffice for change classification, not three-way merging. Initial
implementation can inspect current local/remote differences and offer explicit
whole-side choices; do not store full base documents or promise automatic merge
without a demonstrated need. An external merge of the current versions is
available, with the result checked against a fresh remote observation on push.

## Symmetric operations and conflict resolution

Illustrative command grammar, not a decided CLI specification:

```sh
leafspring create my-post                 # local file, no remote required
leafspring status [FILE]                  # both edges, no baseline mutation
leafspring diff FILE --edge remote
leafspring pull FILE                      # active remote -> document
leafspring push FILE                      # document -> active remote
leafspring tidy FILE --from path          # path -> filename
leafspring tidy FILE --from filename      # filename -> path
leafspring resolve FILE --edge remote --take local
leafspring resolve FILE --edge remote --take remote
leafspring publish FILE                   # Leaflet transition, not bare put
leafspring edit FILE --published-at TIME
leafspring edit FILE --now
```

Draft/post groups can expose the same verbs as scoped entrypoints. Ordinary
push replaces, append is explicitly additive plumbing. Remote writes are
planned and explicitly confirmed; local operations need not masquerade as
network writes. `sync` may aggregate the plan but should not hide direction.
Bulk draft/post discovery feeds the same per-document machinery with a summary
of successes, unsupported items, and conflicts; "all" must qualify discovery
completeness (the current home draft listing may be capped).

Default pull refuses to overwrite changed local content; default push refuses
remote changes. Explicit take-local/take-remote authorizes that overwrite,
does not merely bless an unresolved discrepancy. Restamp only after the chosen
write and read-back succeed. A report never resolves conflicts. Do not conflate
conflict override, lossy-conversion approval, and `--yes` execution consent.

For published raw PDS writes, investigate conditional updates as an adapter
capability; existing transport has none. For native Leaflet publication,
source currently shows no caller-provided expected revision. For draft pushes,
pull then push does not exclude concurrent edits. Expose this limitation and
verify postconditions; do not manufacture atomicity with timestamps.

## Publication and interrupted operations

Local create produces a useful document without a server id. First draft push
creates the editor container, records linkage, and replaces supported content.
Repetition should reuse linkage rather than append or mint again.

Publish plans required editor synchronization and Leaflet publication as one
visible transition. After success, adopt the server-returned path/date/identity,
switch to published state, reset the remote baseline for the new endpoint, and
move into `posted/`. The naming witness remains old so a server-normalized path
appears as optional naming drift, not a covert local rename.

Propose one small pending-operation receipt in the envelope before a remote
write: operation id, intended target, planned local digest, and enough transport
identity to retry or inspect safely. Atomically replace local files. If the
server succeeded but local finalization failed, reconcile the same target on
retry rather than minting again. Native first publication can recover through
the linked editor's document relationship; a lost `/new` response has no proven
idempotency key, so report uncertain creation instead of retrying blindly.
This is bounded crash recovery, not a new scheduler or permanent journal.

Replacing unsupported media produces a specific conversion loss report and
requires explicit approval before deletion starts. A push acknowledgment is
not proof every mutation took effect: the earlier experiment describes
skipped-but-acknowledged mutations. Read-back validates the actual result before
marking the operation synchronized. Verify image-reference lifetime before
reusing media across delete/recreate replacement.

## Dates, encodings, and scope

`publishedAt` is publication metadata, not an edit clock. New local drafts can
leave it unset until publication preparation. An explicitly selected date is
preserved when supported; updates retain the date unless the user edits it.
`edit --now` is an explicit change. Local draft validation must therefore be
distinct from a fully publishable wire record's required-field validation.
Scheduling is separate; the native action currently clamps future dates.

JSON is the full-fidelity encoding. Markdown is an alternative encoding of the
same logical document, with document fields and `@leafspring` in frontmatter.
One chosen file is maintained, never a Markdown source plus synchronized JSON
output. Converters can create outputs for inspection, but do not establish a
third sync edge. Before conversion, report unrepresentable fields/blocks;
`--force-downconvert` authorizes the named loss. JSON formatting is irrelevant
to change detection; Markdown text/facet semantics are not.

Future ticket `leafspring-mdx` explores declarative rich/unknown-block escapes
and lossless markup. It is not required to ship this architecture. Linkage in
local files is accepted by the user; account credentials remain in config.
Record-only export strips the envelope. No sidecar migration, editor merge
engine, automatic deletion, or executable MDX runtime is proposed.

## Pressure tests and remaining choices

- Manual `mv`: remote identity unchanged; naming edge detects it.
- Manual path edit: naming edge and covered remote projection detect it;
  bound published URI is unchanged.
- Both naming values change consistently: converged, acknowledge explicitly.
- File and web both edited: inspect current differences; resolve direction or
  merge manually, then execute against a fresh observation.
- Local create before linkage: draft/unlinked, not remote-missing conflict.
- Post edited: remains published; its retained editor locator is implementation
  detail. Detect divergent editor/application/PDS views before overwriting them.
- Repeat import: match actual remote identity, preserve chosen local filename;
  protect local changes, never recompute identity from title or URL path.
- Duplicate files: ambiguity, no silent precedence or cleanup.
- Publish changes path: adopt actual path; optional tidy renames the file.
- Future date: report native normalization; do not imply scheduling occurred.

Key choices for comparison with Astra: tagged active endpoint instead of two
independent draft/post baselines; pair-hash witnesses rather than full base
snapshots; local id rather than token/file identity; public path edits retained
as intent subject to adapter support; native publication as the normal flow;
shared verbs rather than a growing collection of lifecycle-specific commands.

## Cross-references

- [Original canonical-record design](/design/records-canonical.glm53m.md):
  precedent for a document-first format, revised here to embed local metadata
  and reject filename authority. Its purity and sidecar rules are historical
  until a redesign is accepted and implemented.
- [Extraction and authoring design](/design/draft-extraction.glm53m.md):
  evidence for the editor adapter, not authority for append-only normal flow.
- [Configuration revision](/design/records-canonical2.glm53m.md): already-landed
  global/workspace layering remains useful across the redesigned commands.

## Join: comparison after both independent options were committed

Read [Astra's option](/design/document-sync0.gpt6a.md) only after preserving the
independent proposal above in commit `c51ee169`. The options agree on the main
architecture without requiring a compromise: one logical file, one active
target union, two independently witnessed edges, semantic hashing, optional
bidirectional tidy, local create, explicit native publication, no Markdown
source/output synchronization, and no sidecar compatibility work.

Meaningful differences and my revised recommendations:

| Topic | Sol independent option | Astra independent option | Revised lean, not acceptance |
|---|---|---|---|
| Conflict memory | Pair of local/remote hashes; inspect current endpoints; no promised three-way merge | Embedded agreed document as common ancestor | Adopt Astra's self-contained base snapshot for useful base/local/remote inspection; derive hashes consistently and avoid duplicating snapshots per edge |
| Published updates | Lean toward a Leaflet-managed adapter, retaining editor machinery | Native first publication, then CAS-protected PDS updates | Prefer Leaflet-managed updates under the user's "publish being Leaflet" intent, but expose the editor-input/PDS-output checks and weaker concurrency; ask before choosing PDS-only update semantics |
| Placement | Conventional draft/post dirs, explicit transition moves | No required state dirs, publication never moves a file | Lifecycle must be embedded, not inferred from dir; keep familiar default dirs but distinguish default layout from an optional tidy placement policy |
| Remote witness | Normalized paired projections with field coverage | Full draft fact/metadata revision including opaque material, plus agreed document | Adopt full draft drift witness separately from editable projection; otherwise unsupported-content changes can be missed |
| Forced Markdown loss | Accepted projection plus explicit retained-local/loss report | Keep loss/local-change indication relative to full remote baseline | Preserve explicit persistent loss status; don't present lossy encoding as fully synchronized |

An agreed document is an ancestor only for the supported mapping; a full draft
fact digest detects opaque changes but cannot reconstruct opaque content or
make it mergeable. Store enough to explain that distinction, rather than
calling an extracted document a complete draft backup.

The published-update choice is the largest fork. A Leaflet-managed post can
remain one active lifecycle/adapter, but that adapter has multiple upstream
artifacts to inspect: the editor input, application document, and actual PDS
output. Keep one remote edge at the user level without hiding these inputs in
its plan. Do not reset this composite witness when only one component was
observed or updated. If we instead choose record-following updates, explicitly
document that web editor content can diverge and later overwrite them through
native republish. Similar command names cannot erase that behavior difference.

Neither independent option is the accepted redesign. Final design should
settle published-update semantics and placement policy, specify one baseline
schema and adapter coverage contract, and verify the native transport before
implementation. The limited loss guard should remain proportional to actual
conversion loss rather than becoming a new safety-policy subsystem.
