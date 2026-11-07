---
type: Design
title: One document, one remote target, two witnessed edges
description: Independent redesign option separating optional filename reconciliation from document synchronization, with embedded state and a Leaflet-native publication transition.
resource: /design/document-sync0.gpt6a.md
status: proposal
generated:
  at: 2026-11-07
  by: model:openai/gpt-6-astra#medium
sources:
  - resource: "opencode://ses_e4872728affe5URF2qJwbTDS3R"
    title: Independent architecture option commissioned by the parent; local source research and pressure testing
  - resource: /README.md
    title: Current implemented CLI and its former record-only publishing contract
  - resource: /design/records-canonical.glm53m.md
    title: Earlier design, superseded on sidecars, naming authority, and publication intent
  - resource: /experiments/02-draft-internals/REPORT.md
    title: Draft protocol evidence and archive location
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/publishLeaflet.ts"
    title: Leaflet native publication at 600c7b5f1f7b31b66fb626c293cc104312a12caf
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/app/api/rpc/[command]/push.ts"
    title: Draft mutation acknowledgement and concurrency semantics at the same revision
---

# One document, one remote target, two witnessed edges

## Intent option

**Treat each file as an independently addressable document with one active remote
target. Give both remote kinds the same inspect/pull/push/resolve vocabulary;
keep filename tidying a separate, optional operation. Publish is an explicit
Leaflet-native lifecycle transition.**

This option favors a simple invariant over mirroring every Leaflet artifact:
after publication, ordinary synchronization follows the published record, not
the surviving web editor. Its consequential tradeoff is explicit: published
`push` uses guarded PDS record updates; `publish` uses Leaflet's native pipeline.
If *every update* must also run the native pipeline, this option needs a different
published adapter and must expose the editor's independent unpublished changes.
That choice is a decision for comparison, not an implementation assumption.

This is independent research, not accepted architecture. No implementation,
network probes, live writes, or tickets were performed. The requested
`design/README.md` did not exist; the root [README](../README.md) provided the
current reading map. No other same-wave option was read.

## Source-backed constraints

All upstream facts below refer to local Leaflet checkout revision
`600c7b5f1f7b31b66fb626c293cc104312a12caf`; they are source verification, not proof
of the deployed server's exact build or an exercised CLI transport.

1. **Native publish constructs the record from server facts.**
   [`publishLeaflet.ts`](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/publishLeaflet.ts)
   lines 266–291 fetch facts and convert pages; 276–280 retain the linked URI's
   rkey on updates or mint a TID; 415–447 assign `path: "/" + rkey`.
   [`publishToPublication.ts`](file:///home/rektide/archive/hyperlink-academy/leaflet/actions/publishToPublication.ts)
   lines 11–38 explicitly strip caller-supplied rkey and clamp future dates.
   Arbitrary friendly paths cannot currently survive that native operation.
2. **Native publish is substantially more than putRecord.** The same function
   lines 481–675 writes the PDS, checks DB linkage writes, cuts a publish version,
   creates first-publication mention notifications, claims newsletter send state,
   and revalidates routes. A failed first-publication link attempts compensating
   record deletion. It also performs upstream-owned gated-image cleanup. These
   are existing server behaviors, not proposed CLI reconciliation or cleanup.
   Bluesky sharing is a separate subsequent action in
   [`PublishPost.tsx`](file:///home/rektide/archive/hyperlink-academy/leaflet/app/(app)/(identity)/[leaflet_id]/publish/PublishPost.tsx)
   lines 303–357, not an automatic effect of calling publishToPublication alone.
3. **Record identity is the actual AT-URI.**
   [`uriHelpers.ts`](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/uriHelpers.ts)
   lines 9–12 query document URIs by DID/rkey, supporting both collection names.
   The standalone [route](file:///home/rektide/archive/hyperlink-academy/leaflet/app/(app)/(published)/p/[didOrHandle]/[rkey]/page.tsx)
   lines 29–34 uses that lookup; lines 58–69 advertise the actual AT-URI in HTML
   metadata. Publication [routing](file:///home/rektide/archive/hyperlink-academy/leaflet/app/(app)/(published)/lish/[did]/[publication]/[rkey]/page.tsx)
   also has publication-page matching before post rendering. A URL segment alone
   is therefore insufficient import evidence. A changed record path is neither a
   new rkey nor a verified working Leaflet route.
4. **Draft replacement is an operation to build, not an existing atomic API.**
   [`push.ts`](file:///home/rektide/archive/hyperlink-academy/leaflet/app/api/rpc/[command]/push.ts)
   lines 55–61 accepts no expected revision; 99–104 serialize pushes with a DB
   advisory lock; 151–188 skip replayed mutations and acknowledge even throwing
   mutations. A transaction and acknowledged mutation IDs do not prove all
   requested changes occurred. [`removeBlock`](file:///home/rektide/archive/hyperlink-academy/leaflet/src/replicache/mutations.ts)
   lines 735–772 deletes entities, handles groups/answers, and may enqueue image
   cleanup. The old [report](../experiments/02-draft-internals/REPORT.md) explains
   Yjs text merge behavior; replacing by asserting unrelated text on an existing
   entity risks merging rather than replacement.
5. **Concurrency protections differ.** The checked
   [putRecord lexicon](file:///home/rektide/archive/hyperlink-academy/leaflet/lexicons/com/atproto/repo/putRecord.json)
   lines 39–47 exposes `swapRecord` and `swapCommit`. Native publish's put at
   lines 481–487 supplies neither. Its
   [publish lock](file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/publishLock.ts)
   serializes overlapping publications of one leaflet, not draft edits and not
   arbitrary PDS writers. Draft preflight reads can detect drift but cannot make
   a subsequent replacement compare-and-swap.

The old Rust [sync](../crates/cli/src/sync.rs) hashes file bytes and keys on file
stems; [workspace state](../crates/cli/src/workspace.rs) lives in sidecars.
Those mechanisms are replaced, not migrated. Current
[push response handling](../crates/drafts/src/push.rs) checks transport/envelope
errors, not achieved content. Current [extraction](../crates/drafts/src/extract.rs)
also skips some media; symmetry cannot be claimed merely by reusing it.

## Logical document and embedded metadata

Keep the record-shaped document at the top level and reserve `@springleaf` for
all local state. Strip it completely for record-only export and remote record
writes. JSON is the lossless default; Markdown is an alternative serialization
of this *same* logical object. Do not require state-specific directories.

Illustrative metadata, with placeholders rather than actual credentials:

```json
{
  "@springleaf": {
    "version": 1,
    "localId": "uuid",
    "target": {"kind": "draft", "binding": null},
    "names": {
      "base": {"file": "notes/post.json", "path": "/post"},
      "acceptedAt": "2026-11-07T12:00:00Z"
    },
    "remote": {
      "base": null,
      "observed": null
    },
    "pending": null
  }
}
```

- `target` is a tagged union: draft with no binding yet or
  `{baseUrl, token, rootEntity}`; published with `{uri}`. Published requires an
  actual identity. Never have `draft` and `published` active bindings together.
  `localId` correlates copies/renames locally; it is not remote identity.
- The document's own `path` is the canonical desired path. Its scope includes
  `site`; this does not make path a remote lookup key. Missing draft path may be
  set at local creation, while URI/rkey remains genuinely absent.
- `names.base` stores a deliberately accepted *pair*, not a requirement that
  filename and path match. Files can be named `essay-final.json` while their path
  is `/summer`. The workspace-relative filename includes the encoding extension.
- `remote.base` stores `{localHash, remoteRevision, agreedDocument, acceptedAt}`.
  Retaining the last agreed document inside the envelope makes real three-way
  inspection possible without a sidecar or reliance on VCS. This duplicates one
  document snapshot in the file; that size/readability cost buys self-contained
  conflict resolution. It is not a second editable source.
- Published `remoteRevision` is a CID. Draft revision is a digest of the relevant
  observed fact graph and metadata, including opaque/unrepresentable content;
  Replicache cookies/ack IDs are diagnostic evidence, not a comparable version.
  Separate the full draft witness from the extracted editable document so an
  unsupported-block edit cannot disappear from drift detection.
- `remote.observed` records the latest remote revision and observation time;
  a status read may report it without saving. Observation never advances `base`.
  `pending` records a bounded operation's desired hash, expected revision, and
  replay identifiers before an attempted write. It is recovery state, not a
  third sync edge. Completed operations clear it only after local settlement.

Hash a deterministic logical record representation excluding the envelope,
object key order, and serialization trivia. Preserve array order and content;
do not normalize away unknown fields. An adapter declares which document fields
it actually represents. Draft-local path and date intent remain in the file
when the fact graph has no equivalent. They are validated at publication, not
falsely reported as stored in the editor.

## Two edges, one classification vocabulary

The chain is **filename ⇄ document path ⇄ active remote document**. The second
edge transports the document including its path where supported, not just a
string. Reuse `unchanged`, `left-changed`, `right-changed`, `both-changed`,
`converged`, `unbased`, and `missing`; use edge-specific baselines and policies.
Avoid “newer”: timestamps establish observations, not winners.

### Naming edge

Compare current filesystem location and document path to `names.base`. This
reports what changed even when the user intentionally tolerates a mismatch.
Do nothing to names during ordinary push, pull, import refresh, or publication.

Optional `tidy FILE --from path` chooses a filename from the document path;
`tidy FILE --from filename` changes document path from the actual filename;
`tidy FILE --accept` witnesses the current pair without changing either.
Check collisions and display the concrete before/after pair. For nested paths,
use a declared reversible mapping where possible; require an explicit
`--path`/destination for ambiguous or filesystem-unsafe forms, rather than
pretending a slugification is reversible. These are deliberate local writes.

After tidying, advance only the naming baseline. A path edit leaves the remote
baseline intact and appears as a document change on the other edge. Conversely,
a pulled remote path change leaves the naming baseline intact. No cascade.

### Remote edge

Compare logical local hash to `base.localHash`, remote revision to
`base.remoteRevision`, and current endpoint contents when both changed.
If both now agree, acknowledge convergence without another remote write.

- Local-only changes: push can apply them after fresh remote preflight.
- Remote-only changes: pull adopts them, preserving local-only draft intent.
- Both changed: inspect base/local/remote; select local, remote, or edited merge.
- Missing binding: a draft push mints first; a published target cannot guess an
  identity. Missing remote: report; recreation/rebinding is an explicit choice.
- Unbased: show a two-way diff; establish equality or request an explicit choice.
  Importing an identity must not silently authorize replacing its contents.

`resolve FILE --take remote` installs the observed remote content and establishes
a fresh base. `--take local` or `--merged FILE` saves the chosen result and a
resolution intent pinned to the remote revision just reviewed; the old base
remains until push verifies success. A later remote change invalidates that
intent. This avoids making a conflict disappear merely because someone edited
a hash or clicked “resolved.” Merge inspection can emit base/local/remote to
stdout or a user-selected temporary location; no maintained merge sidecars.

After push: fetch again, compare intended supported content with actual content,
and only then store the achieved document, local hash, and remote revision as
the next base. Draft acknowledgement alone cannot settle it. A partial batch
retains `pending` and reports achieved differences. PDS updates use
`swapRecord = observed CID`; a failed comparison returns to inspection.

## Lifecycle and symmetric command sketch

```text
springleaf create FILE [--path /post]       # local draft, no remote identity
springleaf import SOURCE --to FILE         # draft token or published AT-URI/URL
springleaf status FILE                     # both edges; remote read by default
springleaf diff FILE [--edge names|remote]  # baseline/local/remote
springleaf pull FILE                       # route by tagged target
springleaf push FILE                       # draft mint-or-replace; published CAS
springleaf resolve FILE --take local|remote
springleaf resolve FILE --merged MERGEFILE
springleaf tidy FILE --from path|filename
springleaf tidy FILE --accept
springleaf publish FILE [--now]             # draft -> published through Leaflet
springleaf drafts push FILE                # same push, require draft kind
springleaf drafts append TOKEN --from FILE # explicit append, no sync claim
springleaf convert FILE --to json|markdown # developer-facing encoding tool
springleaf export FILE --record-only       # no @springleaf
```

The kind-specific command is a convenient entry point, not separate semantics.
Append replaces the old `author` name without a compatibility alias. Append to
some other draft does not retarget the source file or claim synchronized state.
The same pull/push/resolve machinery serves drafts and published documents.

**Publication sequence:** settle draft content, retain its token in the pending
operation, invoke the source-verified native publish action, inspect its result
and the actual PDS record, then switch the target union to published in one local
file replacement. Store any former token as inert publication provenance if
useful, never as another active target. Publication does not automatically move
or rename the file. The path returned by the server must be adopted explicitly
as the achieved canonical document path; leave the naming witness untouched.

There is a genuine upstream mismatch: native publication mints a server rkey
and fixes path to it. Before execution, tell the user that their friendly draft
path will be replaced; do not imply `--path` can override native publishing.
Supporting native arbitrary paths needs upstream support. The server-action
transport/build identifier discovery and publication linking/setup remain
unverified integration work; there is no demonstrated stable publish RPC here.
Never silently substitute raw putRecord for a failed native publication.

**After publication:** this option follows the PDS record and updates it with
CAS. The surviving web editor may contain stale or new unpublished work; a later
web publish can replace the PDS record and is then remote drift. It is not
silently followed. PDS-only updates do not run all native side effects. If that
tradeoff is unacceptable, choose a Leaflet-managed published adapter instead:
it must inspect *both* published output and editor input before staging changes,
and native publication still lacks a CID precondition. Calling that one simple
remote edge must not hide the extra reconciliation problem.

## Draft replace, dates, and encodings

Draft push preflights complete representability before mutations. Build fresh
replacement block entities instead of asserting fresh Yjs text onto old ones;
retire only the explicitly replaced page/block structure. Preserve unrelated
comments/settings rather than treating the whole fact graph as garbage. Exact
references between pages/questions and retained editor identities need an
adapter design; this is not solved by existing append code. A preview must name
unsupported blocks and destructive replacement effects. Default refusal plus an
explicit lossy-draft override is sufficient; do not silently skip media and
declare successful replacement. This override is separate from Markdown loss.

Local create should not stamp a publication date merely to satisfy a final
record schema. Use draft-valid documents with absent `publishedAt`; require or
resolve it at publication. Preserve an existing supplied date. `--now` explicitly
sets `publishedAt` from one captured instant; it does not rename, resolve a
conflict, or refresh all metadata. Native publication chooses now if absent and
clamps future dates; future scheduling is a separate capability, not a promise
of this option. Import/pull preserve record dates; observation/accepted/write
times live only in the envelope and never determine precedence.

Markdown stores record metadata and `@springleaf` in frontmatter and encodes
representable content as its body. Reading either encoding yields the same
logical document. Converting encodings in place does not reset remote baselines;
its filename change is visible on the naming edge. Plain Markdown cannot retain
every Leaflet block: report concrete losses and require `--force-downconvert`
before a lossy conversion, including a pull into a Markdown file. Such a pull
must retain a local-change/loss indication relative to the full remote baseline,
not mark the reduced document perfectly synchronized. The default alternative
is to keep/use lossless JSON. No separately synced Markdown source and JSON
output pair. MDX-like rich/unknown-block preservation remains a future topic
for the parent's requested ticket.

## Pressure tests

| Situation | Outcome |
| --- | --- |
| Manual `mv` | Filename edge changes; embedded binding survives; remote sync still addresses the same identity. |
| Manual path edit | Naming right side and remote local side change independently; no inferred rkey change. |
| Both filename and path edited | Show both old/new values; accept pair or choose tidy direction explicitly. No timestamp winner. |
| No baseline | Unbased two-way inspection; equality can establish a base, otherwise explicit resolution. |
| Local create before remote ID | Draft with null binding; first push mints, records returned token, then replaces. |
| Web and file edits | Both-changed with saved common document; reviewed resolution pinned to observed remote version. |
| Published path changes | Update same URI with CAS if deliberately chosen; warn Leaflet route/native-republish semantics are unsupported, never create another record. |
| Successful write, failed local save | Retained pending intent plus read-back recovers achieved state; do not blindly repeat publication or append. |
| Mint succeeds, token response lost | Outcome uncertain; list/recover remote draft explicitly. No idempotent mint guarantee and no automatic orphan cleanup. |
| Publish response lost | Inspect linkage via retained draft token and PDS identity before another call; native lock alone does not make retries side-effect-free. |
| Unsupported block during replace | Fail conversion preflight before sending destructive mutations; explicit loss override must describe actual dropped content. |
| Mutation acknowledged but skipped | Read-back mismatch leaves pending/failed result; old agreement remains. Reusing replay IDs cannot repair an acknowledged failed mutation. |
| Re-import same remote repeatedly | Match exact URI or base URL/token to existing files, then reuse ordinary pull/conflict classification; no duplicate file or baseline reset. |
| Two files claim same identity | Report duplicate ownership and require an explicit destination; do not pick filename order or prune. |

Pending operations are persisted in the document before remote writes. Replayed
draft batches reuse their exact client/mutation IDs; repairing partial success
requires a newly planned batch. A minted token is persisted as soon as received.
If local storage is broken at that instant, print the recovery identity and stop;
the one-file model cannot guarantee recovery from an unreceived server response.
This bounded recovery account is sufficient without building a general journal.

## Tradeoffs and decisions to carry forward

1. **Simple active-target invariant versus all-native published updates.** My lean
   is the explicit record-following published adapter described here, provided
   the user accepts its update-side-effect difference. If publication means
   *every update through Leaflet*, select the other adapter consciously and expose
   its editor-input drift. Neither adapter can conjure native CAS support.
2. **Friendly canonical paths versus Leaflet's current native contract.** The file
   can express a desired path independently of its name, but native publish
   currently overwrites it. Preserve the distinction and negotiate the result;
   an architecture claiming both arbitrary native paths and this exact upstream
   behavior is unsupported.
3. **One embedded ancestor versus smaller files.** An embedded common document
   supports useful merges anywhere the file travels; hashes alone only classify.
   Large media stays referenced, but large content still doubles in the file.
4. **Symmetric operations versus unequal guarantees.** Draft/published commands
   can behave alike while printing different protection levels: draft
   preflight/read-back, published CAS, native publication serialized but not CAS.
5. **Representability is the principal implementation boundary.** Before promising
   replacement parity, decide how media, opaque blocks, offloaded pages, gated
   content, and editor-only references round-trip. Native publish may return an
   inflated record while PDS holds truncated/offloaded content (source lines
   411–454); ordinary PDS import must preserve that actual representation rather
   than mislabeling it as the full private editor. These are adapter capabilities,
   not grounds for a new Markdown sync relation.

Related established work: the [canonical-record proposal](records-canonical.glm53m.md)
supplies the record-shaped center but has superseded sidecar/naming assumptions;
[draft extraction](draft-extraction.glm53m.md) and the
[draft internals report](../experiments/02-draft-internals/REPORT.md) supply
conversion and mutation evidence. This option intentionally leaves those older
documents intact for the parent's comparison and eventual synthesis.
