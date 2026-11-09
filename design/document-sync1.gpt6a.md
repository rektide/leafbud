---
type: Proposal
title: Leaflet-managed documents with compact witnesses
description: Round-one architecture for one file, exclusive lifecycle, complete editor conversion, hash-only conflicts, and native publication updates.
resource: /design/document-sync1.gpt6a.md
status: draft
generated:
  at: 2026-11-07
  by: model:openai/gpt-6-astra#medium
sources:
  - resource: "opencode://ses_e4872728affe5URF2qJwbTDS3R"
    title: Independent second-round design responding to user corrections and local slug-routing research
  - resource: /design/document-sync0.gpt6a.md
    title: Previous Astra option; snapshot and PDS-default recommendations replaced here
  - resource: /design/document-sync0.gpt61s.md
    title: Previous Sol option and parent join, read before this independent round
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/actions/publishToPublication.ts"
    title: Ordinary native post publication at source revision 600c7b5f1f7b31b66fb626c293cc104312a12caf
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/resolveDocumentFilter.ts"
    title: Path-aware post routing at the same revision
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/src/replicache/mutations.ts"
    title: Draft metadata and mutation semantics at the same revision
---

# Leaflet-managed documents with compact witnesses

## Intent

**One editable file represents one Leaflet-managed document. Draft push saves it;
published push saves and republishes it. Pull obtains the current editable
Leaflet document in either lifecycle.** Native publication remains the operation
that first makes a draft public. The same commands, content format, and conflict
choices apply throughout.

The file is exclusively `draft` or `published`. A published post's retained
editor is its editing machinery, not a second active draft. Directories are
defaults. Synchronization remembers compact fingerprints, never an ancestor
document or persisted alternative copy. Conflicts compare today's local and
remote content and let the user choose or edit.

This independently written round incorporates the user's corrections and both
previous options, including Sol's join. It does not read the parent's new round.
It replaces my former recommendations of an embedded ancestor and PDS-default
published updates. No implementation, live calls, or tickets were made.

## What native Leaflet currently permits

Evidence is pinned to the locally inspected checkout at
`600c7b5f1f7b31b66fb626c293cc104312a12caf`, not the unknown deployed revision.

### Slug, rkey, path, and filename are distinct

- **Filename** is the user's local label, freely renameable.
- **Record path** is document data scoped to its site. It participates in public
  routing but does not establish record identity.
- **Rkey** is part of the actual AT-URI identity. Import preserves the actual URI;
  path changes never imply minting or rebinding.
- **Native ordinary post publishing currently chooses the first rkey.**
  [The action](file:///home/rektide/archive/hyperlink-academy/leaflet/actions/publishToPublication.ts)
  lines 11–38 excludes and strips caller rkey. Its
  [publisher](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/publishLeaflet.ts)
  lines 276–280 uses the existing URI's rkey or a TID; lines 421–426 write
  `path: "/" + rkey`. This also means a native update resets a nonmatching path.

**Additional findings refine the previous round:**

1. Custom post paths genuinely route. [resolveDocumentFilter](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/resolveDocumentFilter.ts)
   lines 20–38 resolves a publication's matching record path first, then falls
   back to rkey; duplicate paths choose the lexicographically greatest URI.
   [getPostPageData](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/getPostPageData.ts)
   uses that resolver. [URL resolution](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/resolveStandardSitePostUrl.ts)
   also looks up actual document paths within publications. Routing support is
   not evidence of an ordinary authenticated *setter* for those paths.
2. Custom first-publish keys exist in **admin import workflows**.
   [Ghost import](file:///home/rektide/archive/hyperlink-academy/leaflet/actions/admin/importGhost.ts)
   lines 79–87 and 156–167 accepts a source slug and passes rkey into the trusted
   importer; Substack import has the equivalent path mode.
   [asAdmin](file:///home/rektide/archive/hyperlink-academy/leaflet/src/admin/asAdmin.ts)
   checks the site's admin allowlist. This is native publication with a chosen
   key, but not a generally available user CLI capability or an acceptable
   implicit workaround.
3. **Publication pages** support editable routes independently of their rkeys.
   [publishPublicationPages](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/publishPublicationPages.ts)
   lines 38–40 validate `/` or a lowercase single-segment slug; lines 125–159
   publish `pub.leaflet.publicationPage`, with stable rkey and separate page path.
   [Nav extraction](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/publicationNavEntries.ts)
   reads `page/route`. These are navigation pages, not ordinary
   `site.standard.document` posts; page routing can shadow a post at that path.
4. [updatePublicationDraft](file:///home/rektide/archive/hyperlink-academy/leaflet/src/replicache/mutations.ts)
   lines 1201–1257 exposes title, description, tags, preferences and a local date,
   not slug/path/rkey. Its `localPublishedAt` is client-only; server updates omit
   it. The inspected [publish UI](file:///home/rektide/archive/hyperlink-academy/leaflet/app/(app)/(identity)/[leaflet_id]/publish/PublishPost.tsx)
   calls the ordinary action and derives its outgoing post URL from the returned
   rkey. No ordinary custom-post-slug control was found in this bounded search.

**Conclusion:** I cannot promise user-selected native post slugs with this source
surface. The source has routing and trusted import support, so “Leaflet cannot
do custom slugs” would be too broad. Ordinary-user native slug support needs a
verified additional endpoint or an upstream change. Do not silently use PDS puts.

Default local create therefore leaves `path` absent, visibly **unassigned**,
rather than inventing a public route from the friendly filename. Explicit
`--path` or filename-to-path tidy sets desired document data, but status marks it
unapplied; the native publication plan says it cannot honor that intent and
requires accepting the server-assigned path. On updates an incompatible path
edit likewise requires an explicit choice, not a silent normalization.

## File and compact metadata

The JSON body is the complete record-shaped logical document plus reserved
`@leafbud`. Draft validation permits fields such as path/date to be absent
before publication. Record-only export explicitly removes the envelope; every
outbound conversion starts from that stripped object. Credentials stay in config.

Illustrative published metadata; hashes and identifiers are placeholders:

```json
{
  "@leafbud": {
    "version": 1,
    "id": "local-uuid",
    "target": {
      "kind": "published",
      "uri": "at://did:plc:example/site.standard.document/key",
      "leaflet": {"baseUrl": "https://leaflet.pub", "token": "uuid"}
    },
    "names": {"file": "posted/essay.json", "path": "/key", "at": "2026-11-07T12:00:00Z"},
    "sync": {
      "profile": "leaflet-1",
      "local": "hash",
      "editor": "facts-and-metadata-hash",
      "application": "full-record-hash",
      "pds": "cid",
      "at": "2026-11-07T12:00:00Z"
    }
  }
}
```

For draft state, `target` is `{kind: "draft", leaflet: null}` before remote
creation, then contains the Leaflet locator. No `uri` until actual publication.
The published variant owns one URI and may carry its editor locator. Missing
editor access is `editing-unavailable`, not permission to manufacture a new draft
or switch to PDS writes. Import can still obtain published content read-only,
clearly reporting that full Leaflet-managed synchronization is not yet available.

Optional `pending` holds only operation ID, phase, desired local hash, reviewed
remote tuple, and returned IDs/hashes. A resolution choice similarly holds a
local hash and reviewed remote tuple. There are **no full base records, stored
mutation batches, persisted remote snapshots, merge files, or shadow documents**.
No history cache is required. Inspection reads current endpoints into memory.

`sync` is an accepted correspondence witness, not an equality assertion between
three serialization forms. Its single tuple records the Leaflet-managed post's
actual components. Fields absent for a draft stay absent. Read failures are
unknown, not absent; unavailable required components prevent a fully verified
published baseline. A status read does not update witnesses.

## One remote adapter with honest component checks

Call the remote observations **E** (editor facts plus draft metadata), **A**
(application's full published document), and **P** (actual PDS output). They are
existing upstream artifacts, never extra local documents the user maintains.

The editable remote view is assembled from E: content, layout, theme, title,
description, tags and preferences. A supplies established publication identity,
path/date and output-owned fields not available in E. For an unpublished draft,
unsent publication intent remains in the local record. Explicit local edits of
publication fields are passed as native action arguments where supported;
output-owned fields must not be reported as editable merely because JSON permits
editing them. Changing publication/site is a distinct retargeting request, not
an ordinary content push that could accidentally create a different post.

| Command | Draft | Published |
| --- | --- | --- |
| `status` | Compare local and E witnesses | Compare local and E/A/P tuple; show editable drift and publication state |
| `pull` | Install current editable E | Install current editable E plus publication fields; do not publish |
| `push` | Mint if unlinked, replace/save E, verify | Replace/save E, verify, native republish same URI, verify E/A/P |
| `publish` | Save and perform first native publication | Report already published; use `push` for updates |

If E contains newer web edits than A, report **unpublished web changes**. Pull
brings those edits into the same published file, whose lifecycle remains
published. After that pull it can be **editor-synchronized; publication pending**.
Push publishes them even if the local hash itself is unchanged. This is useful
symmetry with web editing, without pretending a web save already changed public
output. Pull always stays read-only remotely.

If A and P differ only by verified native transforms—page offloading, membership
truncation, and documented output fields—that is normal. Expand blobs and apply
the known publication projection in memory when comparing. An unexpected PDS
edit or application disagreement is **publication divergence**, not permission
to replace it silently. Ordinary pull still has an identified editorial source,
but reports this condition and cannot bless public consistency. Explicit
take-local/take-remote followed by push authorizes republishing the chosen
editorial document over that observed output. Inspection can show the particular
remote component difference; it does not ask the user to maintain another copy.

Import uses actual URI/token identity, discovers linkage, and selects this same
editable view. Repeated import is ordinary pull into the existing file; it never
resets local drift or picks identity from filename/path. If more than one file
claims the identity, report ambiguity. Never prune automatically.

## Fingerprints, classification, and conflict choices

The two edges remain **filename ⇄ canonical document path ⇄ Leaflet**. Reuse the
classification pattern, not a shared baseline:

- Neither witnessed endpoint changed: unchanged.
- Only left/right changed: local/remote change on the selected edge.
- Both changed: compare current semantic values; equivalent means converged,
  otherwise require a choice. No timestamp winner.
- No usable witness: unbased; inspect current endpoints and explicitly establish
  correspondence. Missing/unlinked/unknown are separate states.

The local hash is versioned deterministic logical JSON, excluding `@leafbud`,
preserving array order and unknown fields. Semantic projection compares editable
meaning: regenerated entity IDs and CRDT byte encodings are not content changes.
Do not erase meaningful reference topology when normalizing IDs.

E's **facts fingerprint** is separately conservative: sort the received graph
deterministically and include fact identities, values, relationships, unknown
attributes and relevant token/publication metadata. Exclude transport envelopes,
pull cookies and delivery timestamps only because they describe retrieval rather
than document state. Do not blanket-filter `updatedAt`, unknown fields or opaque
content as “volatile.” Initially unexplained graph changes remain visible; the
adapter may classify a verified ID/CRDT-only change as semantic convergence.
Comments or other non-authored facts can change the graph witness without
changing the document projection; show that provenance and preserve them.

`diff FILE` displays the **current** file and current remote editable view, plus
component drift. It cannot reconstruct the ancestor. The user may edit the file
directly, then choose `resolve --take local`; `--take remote` is a guarded pull.
Take-local pins the chosen current local hash and remote tuple but does not
declare success. Push rereads both before executing; any intervening change
invalidates the choice. No automatic three-way merge is promised.

Advance the remote witness only after a verified pull, verified push, or explicit
acknowledgment of current semantic convergence. A pull may witness an editor
correspondence while retaining publication-pending/divergence status; those
conditions are recomputed from E/A/P, never cleared merely by resetting hashes.
Partial writes do not advance a successful-sync witness. Observe components
again if the read sequence changes underfoot; a composite read is not an atomic
snapshot and instability must be reported.

Naming witnesses store actual filename and canonical path, including an absent
path. `tidy --from path` chooses a filename; `tidy --from filename` edits path;
`tidy --accept` accepts a deliberately mismatched pair. Collision-check and show
the exact transformation. Ambiguous nested/unsafe paths require an explicit
destination rather than irreversible slugification. Only successful tidy/accept
advances this witness. A pulled/server-assigned path does not rename the file;
a tidy path change does not reset the remote baseline. Manual moves affect only
the naming edge. Neither folder placement nor a filename change publishes.

## Native execution and bounded recovery

1. Validate complete conversion and resolve all needed assets before destructive
   editor changes. Read current E and, for posts, A/P. Classify drift.
2. Persist a compact pending receipt, then save the chosen document into the
   existing editor (mint and immediately record its token if needed).
3. Read back E and compare actual semantic content, metadata and asset references.
   Only an achieved save permits the publish step.
4. For published push/first publish, recheck the publication tuple, call the
   native action with explicit title/date/preferences and delivery choices, and
   read back linkage, A and P. Require the expected URI for updates. On first
   success switch the exclusive lifecycle and adopt server-assigned path/date.
5. Verify the original local hash is still current before replacing the file
   metadata/result. Set the accepted tuple and clear pending only after verified
   settlement; do not overwrite a concurrent local edit.

There is no proven conditional draft replace or native expected-CID publish.
The [push handler](file:///home/rektide/archive/hyperlink-academy/leaflet/app/api/rpc/[command]/push.ts)
serializes pushes but skips/acknowledges throwing mutations. The
[publish lock](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/publishLock.ts)
serializes publication attempts, not all editor/PDS writers. Preflight and
read-back detect important races; neither supplies atomic compare-and-update.

Recovery rereads the same target and compares against the still-present local
document and pending hashes. If the desired state is already achieved, settle
locally. If editor save succeeded but publication failed, retain that phase and
resume publication after new checks. If partial mutations remain, replan from
current facts; do not replay a reconstructed batch with stale mutation IDs.
Within one in-memory attempt the exact batch can be retried with its original
IDs. Across crashes no stored batch or historical document is required.

If the local document changed after the attempt, pending identifies uncertainty
but cannot reconstruct the former payload: inspect current endpoints explicitly.
A lost mint response still has no proven recovery/idempotency token; list/inspect
to recover rather than blindly minting again. A lost publish response calls for
linkage and output inspection, not blindly repeating email/share effects. Native
publish is multi-step; record presence alone cannot prove every side effect
completed. Report `publication observed; delivery outcome unknown` where needed.

## Complete content and media adapter

Implement **all current Leaflet block/page types**, including images, galleries,
drawings, canvases, nested content, richtext/facets, alignment and themes. The
existing converter's skips are missing implementation, not accepted format loss.
Use a bidirectional registry tied to the checked Leaflet schema, with structural
and semantic round-trip fixtures for every current kind and their compositions.

The [facts-to-pages converter](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/factsToPagesRecord.ts)
provides the publication model and asset hooks; the
[native offloader](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/offloadPagesToBlob.ts)
explains why the PDS record can differ from full content. Mirror those mappings,
including reference remapping, rather than treating published pages as flat text.
Prefer identity-preserving structural edits where references/comments require
stable entities. Fresh Yjs payloads on existing text entities merge, so true text
replacement needs correct CRDT edits or a deliberate entity replacement plan.

Published blob references require retrieving actual bytes from their owning repo
or authorized source and uploading them into the editor's supported asset
storage. Validate access and content before replacement. The upstream
[import uploader](file:///home/rektide/archive/hyperlink-academy/leaflet/src/import/importPost.ts)
lines 33–57 is evidence for resolving images before graph insertion, not proof
that its admin transport is a public CLI API. Verify the ordinary upload surface
as part of adapter work. Do not delete/recreate media entities in a way that
queues cleanup of still-used image objects.

Read-only draft pull must not require remote uploads to express media. Retain
content-addressed blob descriptors in the logical record and compact source
locators under `@leafbud.assets` when bytes are still in Leaflet/local storage;
resolve bytes in memory to construct/verify those descriptors. Poll references
and other external resources likewise need resolvable targets; never claim a
made-up AT-URI is a published object. Newly authored resources require their
creation path in the adapter. Assets are referenced resources, not saved copies
of the document. Inaccessible required inputs fail clearly; a future unknown
block is explicit unsupported, preserved locally. Full current-type support is
an acceptance condition, not a permanent `--force` bypass for unfinished work.

## Commands, dates, and encodings

```sh
leafbud create essay                    # defaults to drafts/essay.json
leafbud create --file notes/essay.json   # equally valid placement
leafbud import SOURCE --to FILE
leafbud status FILE
leafbud diff FILE
leafbud pull FILE
leafbud push FILE                       # draft save; published save + update
leafbud publish FILE --now              # first native publication
leafbud resolve FILE --take local       # pin reviewed overwrite choice
leafbud resolve FILE --take remote      # guarded current-remote pull
leafbud tidy FILE --from filename
leafbud tidy FILE --from path
leafbud tidy FILE --accept
leafbud edit FILE --published-at TIME
leafbud drafts push FILE                # scoped entry to the same operation
leafbud drafts append TOKEN --from FILE # low-level additive primitive
leafbud convert FILE --to markdown --force-downconvert
leafbud export FILE --record-only
```

`posted/` is the default destination for new published imports; `drafts/` for
local creates/draft imports. Publishing an existing file need not move it. A
separate explicit placement choice can do that. Append replaces `author` without
a compatibility alias and does not claim to synchronize the source file.

`publishedAt` is not an edit clock. Creation may omit it; first native publication
chooses now if unspecified. `--published-at TIME` selects a value and `--now`
captures one instant intentionally. Updates preserve the established value
unless edited. The inspected action clamps future dates; report that limitation
before publishing rather than pretending this is scheduling. Witness timestamps
record observation/acceptance, never precedence. Draft local date intent is not
falsely reported as saved by `updatePublicationDraft`'s client-only field.

Markdown is another serialization of this document, with substantial record and
local metadata in frontmatter. There is no JSON output partner to synchronize.
Lossless conversion preserves logical hashes. Genuine downconversion requires
`--force-downconvert` and a specific loss report, including on pull into a
Markdown file. Keep compact loss status and divergent local/remote hashes so a
lossy pull cannot masquerade as full equality or authorize later silent deletion.
Switching to lossless JSON is available without a second maintained document.
Rich declarative markup is future work under `leafbud-mdx`.

## Boundaries and next evidence

This is a future-forward rewrite, not migration work. PDS-managed operations are
a worthwhile separately tracked future adapter, with their own conditional-write
and lifecycle semantics; they are neither normal sync nor fallback here.

Before claiming implementation readiness, verify native action discovery/auth,
post-to-editor linkage retrieval, ordinary media upload, all current-type
round-trips, native output normalization, and interrupted-operation outcomes.
The highest user-facing unresolved capability is a supported ordinary-user
custom slug setter. The model remains useful without inventing one: friendly
filenames, unassigned initial paths, explicit native path adoption, and optional
two-way tidy are all coherent today.

Previous-round evidence and alternatives remain in
[Astra round zero](document-sync0.gpt6a.md) and
[Sol round zero with join](document-sync0.gpt61s.md). This proposal implements the
new constraints at the design level and leaves comparison with the parent's
independent current round to the requested later chat synthesis.
