---
type: Design
title: "One working document, two witnessed edges — Sol round 1"
description: "Snapshot-free Leaflet-managed synchronization, symmetric editor push and publication, complete block support, and optional name/path reconciliation."
resource: design/document-sync1.gpt61s.md
status: proposal
generated:
  at: 2026-11-07
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4900b8d7ffeDT9QuFwmKRAB7t"
    title: "User rejects saved bases, selects Leaflet-managed lifecycle, requires complete JSON-to-editor support, and commissions independent round 1"
  - resource: /design/document-sync0.gpt61s.md
    title: "Sol round 0 and comparison; saved-ancestor recommendation explicitly withdrawn"
  - resource: /design/document-sync0.gpt6a.md
    title: "Previous independent Astra option, available before round 1"
  - resource: /crates/drafts/src/author.rs
    title: "Current incomplete reverse converter; omissions are implementation debt, not the target contract"
  - resource: /crates/cli/src/publish.rs
    title: "Existing raw PDS publication and filename-derived identity to redesign"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts"
    title: "Ordinary native publish action rejects caller-chosen record key"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts"
    title: "Native publication reads editor facts, preserves identity on updates, normalizes path, links output, and performs lifecycle effects"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/admin/importGhost.ts"
    title: "Trusted admin import can preserve a source slug as rkey; not an ordinary user publish capability"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts"
    title: "Draft metadata fields and removal semantics; updatePublicationDraft has no slug/path field"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/[leaflet_id]/publish/PublishPost.tsx"
    title: "Inspected UI sequence pushes editor state before native publication; Bluesky sharing is separate"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/addImage.ts"
    title: "Editor images require URL/dimension facts and asset storage handling"
  - resource: /design/document-sync1.gpt6a.md
    title: "Read after committing Sol's independent round 1; comparison and revised recommendations below"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/resolveDocumentFilter.ts"
    title: "Checked after Astra's report: publication paths are resolved before rkey fallback"
  - resource: /experiments/02-draft-internals/REPORT.md
    title: "Existing read/source evidence for full pulls, acknowledged mutation failures, and replay protection"
---

# One working document, two witnessed edges — Sol round 1

## Standing and short intent

New independent draft, written without reading Astra's round-1 document. Both
round-0 options were available. No code implementation or live writes.

**Keep one editable document. Save it to Leaflet with `push`; release it with
`publish`. Both operations mean the same thing before and after first
publication. Optional `tidy` reconciles its filename and document path.**

The user rejected the saved-common-ancestor proposal entirely. This draft has
no embedded base document, hidden snapshot store, merge sidecar, or persisted
request-body copy. Hashes and identity/revision witnesses are metadata, not
another document to maintain. No compatibility layer or migration is needed.

## 1. User model: lifecycle is not revision state

A file is exactly one of:

- **draft**: never published through its current binding;
- **published**: bound to an actual published identity.

Editing a published file does not make it a draft. It is a published document
with unreleased edits. The Leaflet editor that backs it is part of the same
post's machinery, not a second active draft document.

There are two user-facing reconciliation edges:

```text
filesystem name <-> logical document path <-> Leaflet working document
```

Publication is an explicit release of that working document. Its output URI,
CID, and release fingerprint are receipts/evidence of what is live, not a
second locally maintained payload. This matches Leaflet's own inspected UI:
`PublishPost.tsx:272,303-318` pushes editor changes before invoking publication.

**Proposed strongest symmetry:**

| Operation | Draft file | Published file |
|---|---|---|
| `status` | Naming and Leaflet-editor differences | Same, plus whether working content differs from last release |
| `pull` | Read Leaflet working content | Read the same post's Leaflet working content, possibly not yet live |
| `push` | Create/link editor if needed; replace working content | Replace working content of the bound post; do not silently go live |
| `publish` | Save selected file version and release; become published | Save selected file version and release an update to the same identity |
| `tidy` | Deliberate filename/path repair | Same local operation; never infer a new published identity |

This is a proposal, not an assumed interpretation of previous commands. It
makes the distinction visible: `push` means save to web, `publish` means make
public. A user wanting both chooses publish, whose plan includes any required
push. Avoid `push` publishing immediately for posts but merely saving drafts.

`pull` must say **working/editor version**, particularly for a published file.
A record-only read/export of the live version can be provided separately and
does not establish another synchronized file. Importing a PDS-only document
without a recoverable Leaflet editor binding must report that limitation;
never silently switch management mode or mint a duplicate on publication.

## 2. Storage and identity

JSON is a full record-shaped document plus top-level `@leafspring`. Local
drafts need not satisfy final wire-required publication fields before they
exist. Unknown document fields survive ordinary edits. Strip the envelope
explicitly for remote record serialization and record-only export.

Default workspace:

```text
drafts/my-working-name.json
posted/my-working-name.json
```

These are defaults, never authority. Create/import choose suitable defaults.
If the user chooses an explicit path, honor it. This option does not silently
move or rename files during publication; offer lifecycle placement through
optional tidy. Thus first publication cannot unexpectedly relocate an open
editor file. A default-layout auto-move policy is possible, but must be
explicitly chosen rather than deduced from the old sidecar implementation.

Identity roles:

- **Local id**: minted UUID, stable across filename/encoding changes.
- **Document path**: canonical local path intent, not a selector for remote id.
- **Leaflet locator**: base URL, permission token, root entity; retained across
  draft-to-published transition as the editor locator for the same resource.
- **Published URI**: actual returned/verified identity, absent until published.
  Never reconstructed from filename, title, path, or publication owner guesses.

Use one tagged binding, not simultaneous `draft` and `post` management sections.
An illustrative outline (exact names remain open):

```text
@leafspring
  version, localId
  binding: unlinked-draft | leaflet-draft | leaflet-post
    baseUrl, token, rootEntity          where linked
    uri                                only where published
  names
    acceptedFile, acceptedPath, acceptedAt
  agreement
    localHash, editorHash, mappingVersion, acceptedAt
  release                              only where published
    submittedHash, uri, cid, confirmedAt
  pending                              only while an operation is unsettled
    id, kind, phase, target, intendedHash, expectedEditorHash, expectedCid
    deterministicIdSeed, clientId, firstMutationId
```

No field holds a historical body, facts snapshot, or serialized mutation batch.
Persist only identifiers needed for recovery, not redundant aliases derivable
from existing identities. A release fingerprint is over the submitted logical
publication input; its output CID is distinct because native normalization,
asset uploads, offloading, or truncation can alter the wire representation.

## 3. Compact evidence: what it establishes

### Naming witness

Remember an explicitly accepted pair of actual filename and document path.
Compare each current value with its accepted value independently. Conventional
mapping ignores the encoding suffix; include extension/location in diagnostics
so moving/converting a file can still be explained. Use reversible single-path
mapping where supported; ambiguous or unsafe paths require an explicit name.

`tidy --from path` renames the file; `tidy --from filename` edits the path;
`tidy --accept` accepts the current pair even when intentionally mismatched.
Preview and collision-check all writes. Naming witnesses advance only on
creation/import's initial establishment or explicit naming settlement, not on
ordinary status, push, pull, or publish.

An edited path affects the other edge only to the extent Leaflet can represent
it. Unsupported public path intent remains a visible limitation, not evidence
that a new remote identity should be created.

### Leaflet agreement witness

Two compact fingerprints answer two different questions:

- `localHash`: Has the selected logical document changed since the last
  confirmed agreement? Exclude envelope and serialization trivia.
- `editorHash`: Has Leaflet's relevant editor content/metadata changed since
  that agreement, including content the converter might not recognize yet?

This is what becomes of **draft drift evidence**: it is **Leaflet working-state
evidence in either lifecycle**, not evidence for a second draft beside a post.
It detects remote edits before we replace the working content.

The adapter needs both a robust upstream witness and semantic equivalence for
read-back. Raw fact ids, fractional ordering keys, Yjs encodings, upload URLs,
and editor entities need not equal canonical JSON fields byte for byte. Do not
compare a fact-graph digest with a record hash directly. Normalize equivalent
asset references by a verified identity/content rule, retain page/reference
relationships, and declare the mapping version. Unknown fact attributes must
not vanish from drift detection just because conversion does not understand
them. Exclude purely derived indexes, pull cookies, and mutation ack counters
from the content witness; otherwise polling creates false edits.

If the witness changes but editable semantics are equal, report structural or
editor-only change and check preservation before accepting it. An unrelated
comment should not cause the replacement planner to delete the comment. Full
block support does not mean treating the entire fact graph as disposable.

### Release evidence

For a published file, compare current publication-input semantics against the
last confirmed submission to explain **unreleased edits**. Inspect live output
CID/linkage before republishing. An out-of-band PDS change is an external
publication mismatch: show it explicitly, do not silently import it over editor
work or overwrite it through native republish.

Read-only status does not acknowledge these changes. When live evidence changes
without editor changes, publication needs an explicit decision about preserving
that external change; it cannot claim editor/live agreement from a CID alone.
This is a limitation of Leaflet's multi-artifact implementation, not a proposal
to keep another synchronized local copy.

## 4. Shared classification and current-endpoint resolution

Both edges reuse `unchanged`, `left-changed`, `right-changed`, `both-changed`,
`converged`, and `untracked`; missing target is distinguished from an unlinked
new draft. Observation times explain when evidence was collected, never which
side wins. If both sides changed but current supported semantics agree, settle
convergence explicitly without a destructive write.

There is no saved historical document. `diff` fetches current Leaflet state,
holds it in memory, and compares current endpoints. It can print differences
or stream a remote view to the user's tool; the application does not stash it
as a maintained artifact. It cannot show an unavailable historical merge base.

Resolution means:

- **take remote**: replace the selected local document with current working
  content, preserving local-only intent fields; acknowledge after local save;
- **take local**: inspect current differences and authorize replacement of that
  observed remote state, then perform a fresh preflight and verified push;
- **edit manually**: edit the one file using the displayed current differences,
  then choose push or pull deliberately. Editing alone does not mark a conflict
  resolved or change witnesses.

If resolve and execute are separate commands, store at most the reviewed
fingerprints as a short resolution intent; any later file/remote change
invalidates it. No copied bodies. Report unresolved remote absence rather than
automatically resurrecting or deleting content.

## 5. Complete JSON/editor support and write verification

All current Leaflet blocks, pages/canvases, nested structures, alignment,
themes, and necessary assets belong in the adapter contract. Current skips in
`crates/drafts/src/author.rs` are unfinished implementation, not acceptable
permanent sync semantics. Full forward/reverse fixture coverage and semantic
read-back are acceptance criteria.

Media references are not the media bytes: resolve published blobs, draft URLs,
and gated assets with the appropriate access and upload path. Preserve assets
across replacement; inspected removal can enqueue image cleanup. Drawings and
canvases need complete structural mappings. Missing assets or truly unknown
future block types are actionable failures before deletion, not success with
silently omitted content. Explicit lossy approval may exist but is exceptional,
not how normal JSON push becomes usable.

Normal push replaces document content while preserving unrelated settings,
comments, and relationships. `drafts append` is deliberately additive plumbing,
not the synchronization path. Do not overwrite existing Yjs text with unrelated
new-client updates and assume replacement; build the correct structural plan.

Preflight checks agreement, plans changes, then performs the write. Draft push
and native publication offer no verified caller-supplied revision precondition.
They are not CAS-safe merely because a transaction or publish lock exists.
Read-back must confirm achieved state, because acknowledged draft mutations may
have failed individually. Only confirmed agreement advances witnesses.

Persist compact `pending` metadata before remote effects and atomically save
the file. Deterministic seeds can regenerate a batch only if the source and
required planning input still match. Otherwise re-observe and replan; do not
persist a full batch or replay an acknowledged failure blindly. Before saving
a remote result, confirm the on-disk document was not edited during the await.

Lost creation response means uncertain outcome, not license to mint again.
Lost publish response is investigated through the token's actual publication
linkage and output identity before retry. A successful remote effect plus a
failed local save leaves compact recoverable intent when possible; do not
promise exactly-once behavior across an unreceived response and broken disk.

## 6. Slug/path: confirmed capability versus hope

At pinned Leaflet revision `600c7b5f1f7b31b66fb626c293cc104312a12caf`:

- `publishToPublication.ts:11-38` removes caller rkey even if supplied at runtime.
- `publishLeaflet.ts:276-280,425` chooses TID on first ordinary publication,
  preserves the bound record's rkey on updates, and constructs `path = /rkey`.
- `updatePublicationDraft` accepts title/description/tags/date intent/preferences
  but no slug/path (`mutations.ts:1201-1257`). Date intent there is client-local;
  the publish action must still receive the selected date.
- The inspected publish UI passes no slug/path and uses returned rkey for the
  public URL (`PublishPost.tsx:303-341`).
- **Custom first keys are possible internally**: admin Ghost import can retain
  source slug and pass it as rkey (`importGhost.ts:57-87,156-166`), but it runs
  behind `asAdmin`. Publication navigation pages have separate route semantics;
  they are not evidence of custom user post slugs.

Thus no supported ordinary custom-first-slug path has been verified in this
source. That is narrower than claiming Leaflet can never support one; deployed
behavior and upstream changes remain unprobed. Native custom-slug support is
an upstream/API opportunity, not something to fake by sending an ignored field.

Local create may derive an initial document path from filename/title, clearly
labeled local intent, not reserved public URL. Before first native publish,
the plan states that Leaflet chooses the path/key. On success adopt the actual
path and URI. Preserve the old naming witness so optional tidy can align the
filename; do not save a second "desired path document" or force a rename.

A deliberate published path edit does not change the actual binding; warn that
the native publisher resets it. Offer explicit acknowledgment of normalization
or refuse a request that requires exact path preservation. There is no silent
PDS fallback. PDS-managed operations with chosen rkeys are separate future
tickets `leafspring-pds-publish` and `leafspring-pds-sync`.

## 7. Dates, alternate encodings, and commands

Local create does not invent publication time. Preserve selected/observed
`publishedAt`; when absent at first publication let a single resolved instant
be chosen by the publication flow. Updating retains it unless explicitly
changed. `edit --published-at TIME` and `edit --now` affect the document only;
push must not silently reset publication dates. Native future-date clamping
is reported; scheduling remains separate.

Markdown is an alternative serialization of the same document with metadata
and `@leafspring` in frontmatter. No second maintained JSON output, conversion
baseline, or Markdown sync edge. Conversion loses nothing it can preserve;
report actual unsupported content and require `--force-downconvert` for lossy
conversion, including pull into a limited Markdown representation. A lossy
result must not be called a fully faithful sync. Rich/unknown-block markup
is future work in `leafspring-mdx`.

Illustrative surface:

```sh
leafspring create drafts/essay.json --title "An essay"
leafspring import LEAFLET-LOCATOR          # picks default layout unless --to
leafspring status [FILE]                   # both edges and release evidence
leafspring diff FILE --edge names|leaflet
leafspring pull FILE                       # current working editor state
leafspring push FILE --yes                 # save to Leaflet, never publish
leafspring publish FILE --yes              # save + publish / republish
leafspring resolve FILE --take local|remote
leafspring tidy FILE --from filename|path
leafspring tidy FILE --accept
leafspring edit FILE --now
leafspring drafts append TOKEN --from FILE # additive low-level operation
leafspring convert FILE --to json|markdown
leafspring export FILE --record-only
```

Kind-specific groups may scope the same implementation. Bulk discovery/import
feeds the same per-file classifier and reports failures individually; verify
home-list completeness before advertising "all drafts". Config layering remains
global defaults overridden by workspace and explicit command inputs.

## 8. Open implementation gates, not new architectural forks

1. Verify native action invocation/discovery and cookie/OAuth prerequisites;
   do not substitute a raw put if the managed transport fails.
2. Finish complete block/media forward and reverse mappings with fixture
   coverage, reference preservation, and verified replacement.
3. Define versioned semantic fingerprints versus full editor witnesses,
   including ids/assets/editor-only content and publication metadata.
4. Establish dependable editor locator discovery for importing published posts;
   explicit limitation for imported PDS-only records without native linkage.
5. Specify external-live-change handling before republish without introducing
   a second maintained local document or claiming unsupported atomicity.

The main deliberate difference to test against Astra is **save/release symmetry**:
push always updates the Leaflet editor, publish always releases it, regardless
of exclusive lifecycle. This keeps normal operations unsurprising and puts
publication effects behind an explicitly named operation.

## Related work

The [previous Sol option](/design/document-sync0.gpt61s.md) and
[previous Astra option](/design/document-sync0.gpt6a.md) preserve independent
round-0 reasoning. Their ancestor-storage recommendation is rejected, not
carried forward. The [original record design](/design/records-canonical.glm53m.md)
provides the document-first precedent, but its sidecar, purity, filename-key,
and raw-publication assumptions are not this redesign's destination.

## Round-1 join: comparison, not acceptance

Sol's independent draft above was committed as `e3e7345e` before reading
[Astra's new draft](/design/document-sync1.gpt6a.md), committed as `bf4e9161`.
Both replace earlier ancestor-storage and PDS-primary recommendations. These
documents remain proposals, not implementation authorization.

### Agreement

- One mutable local document, embedded compact metadata, no saved base bodies,
  historical fact graphs, persisted request bodies, or maintained merge copies.
- Exclusive draft/post lifecycle. Published content with unreleased edits is
  still a published document, not a new draft.
- One Leaflet-managed adapter; editor/application/PDS are upstream components
  whose differences must be exposed without turning them into local masters.
- Independent filename/path and document/Leaflet witnesses; report-only status,
  optional two-way tidy, no automatic name changes after native publication.
- Current-endpoint conflict inspection, explicit resolution pinned to reviewed
  hashes, preflight/read-back rather than invented transactional guarantees.
- Complete current-block and asset support; no permanent warn-and-skip adapter.
- Directories are defaults, not authority; no implicit publication-time move.
- Markdown is an alternate encoding; lossy conversion needs explicit approval,
  not another maintained input/output synchronization relationship.

### Differences and Sol's revised leans

| Topic | Sol | Astra | Recommendation for eventual synthesis |
|---|---|---|---|
| Meaning of push | Save editor in either lifecycle | Save draft; save and republish post | Keep save/release symmetry: push saves, publish releases in both states; needs user selection |
| Meaning of publish on a post | Release update to same identity | Already published; use push | Same choice as above, not an independent fork |
| Initial document path | May derive local intent from filename | Absent/unassigned by default | Adopt Astra's unassigned default; explicit path remains possible but its unsupported native intent is shown |
| Accepted metadata shape | Editor agreement separate from release receipt | E/A/P component tuple with pending publication status | Prefer separate compact save/release witnesses so saving can never imply public release; keep Astra's explicit component diagnostics |
| Asset materialization | Require resolvable assets and complete mappings | Specify read-only construction with compact asset-source locators | Adopt explicit read-only pull contract and compact asset locators; no upload just to represent a pulled image or invented published poll reference |

### Verified refinement: custom paths route

After reading Astra, inspected `resolveDocumentFilter.ts:20-38` and
`getPostPageData.ts:41-43` directly. Publication post lookup first matches
`documents.data.path` within publication membership, then falls back to rkey.
`resolveStandardSitePostUrl.ts:87-112` also resolves actual record paths.
Thus arbitrary matching single-segment publication paths are not inherently
forbidden or required to equal rkey. This does not establish every nested path
route or standalone-post custom-path behavior.

The ordinary native setter remains the constraint: publish chooses a first
key and writes `/rkey`, whereas trusted import can pass a chosen key. A PDS
adapter can exercise independent rkey/path control, subject to routing and
collision checks, but does not acquire native editor linkage or native
publication effects merely by choosing a better URL. No hybrid native-then-PDS
path rewrite is proposed: a subsequent native update would reset it again.

### Concrete snapshot-free drift account

Keep accepted local fingerprint, accepted editor fingerprint, and publication
receipt fingerprints/CID only. Fetch current remote payload transiently for
diff or read-back; it is not another stored document. Full editor fingerprint
detects opaque changes and topology; semantic mapping identifies actual editable
differences. These are two views of one upstream observation, not two copied
payloads. Editor save advances editor agreement; publish advances release
evidence only after outcome verification. Naming settlement advances neither.

If live output changes externally without editor change, report publication
divergence before republish. Hashes cannot reconstruct the prior body or prove
which external change should win, and this design does not claim otherwise.

### Remaining judgment

The significant user-facing choice is whether `push` may change public content
solely because a file is already published. Sol recommends no: preserve
predictable save versus release semantics and make `publish` valid for initial
and subsequent releases. Other source/adapter gates are implementation research,
not reasons to revive a saved base or dual local lifecycle.
