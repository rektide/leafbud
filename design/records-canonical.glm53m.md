---
type: Design
title: Canonical on-disk format — the ATProto record is the document
description: Springleaf stores posts as pure site.standard.document record JSON whether posted or not; drafts/ and posted/ directories with sidecar state, and sync semantics against the PDS.
resource: design/records-canonical.glm53m.md
status: draft
generated:
  at: 2026-11-07
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e4ceef2e9ffeMaBQB1Yc3bcyMD"
    title: "Session holding the protocol map; operator's canonical-format direction recorded here"
  - resource: "file:///home/rektide/src/springleaf/experiments/03-post-builder/REPORT.md"
    title: "Proof that a lexicon-valid record + putRecord posting path works"
---

# Canonical on-disk format — the ATProto record is the document

Operator direction (2026-11-07): the on-disk format is the canonical format,
"in whatever form it is." Posting prefers writing the ATProto record
directly; the same form is canonical whether posted or not. Two directories
— drafts and posted — that we sync.

## Decision

**The canonical file format is the pure ATProto record**: a
`site.standard.document` JSON wrapping a `pub.leaflet.content` union member,
exactly as it would be (or was) written to the PDS via
`com.atproto.repo.putRecord`. Experiment 03 (`experiments/03-post-builder/`)
already builds and validates this form offline (78/78 checks).

Answers to "will writing the ATProto record work for posting?": **yes**.

Evidence: social-cli posts this exact shape today; Leaflet's appview ingests
`site.standard.*` from any repo off the firehose; the derived URL rule
(`path == "/" + rkey`, publication `url` field) is proven. Caveats, named:

- **No leaflet-publish side effects**: server-side publishing adds
  newsletter notifications, an optional bsky announcement, and
  `leaflets_in_publications` DB linkage. Record-only publishing skips these.
  The bsky announcement is replicable record-side (`bskyPostRef` re-put,
  mirroring Leaflet's own `publishBskyPost.ts`); the rest are acceptable
  losses for a CLI workflow.
- **Not the collaborative draft**: leaflet.pub's editor drafts are private
  fact graphs, not records (see `file:///home/rektide/r/atproto/leaflet-drafts.md`).
  Our canonical drafts and leaflet's editor drafts are different artifacts;
  a facts→record extraction path may bridge them later (pending
  `experiments/02-draft-internals/` and the draft-extraction plan).

## Layout

```
<workspace>/
  drafts/                     # unposted canonical records
    <slug>.json               # PURE record JSON — nothing added
  posted/                     # records as actually written
    <slug>.json               # pure record, byte-faithful to the last put
    <slug>.meta.json          # sidecar state (below)
  springleaf.toml             # identity defaults: handle, pds, publication at-uri
```

Rules:

1. **Record files stay pure.** No frontmatter, no `_local` fields — PDS
   lexicon validation rejects unknown fields in closed records, and the file
   must be puttable as-is. Filenames carry the slug (default rkey).
2. **Sidecar state** (`posted/<slug>.meta.json`): `rkey`, `uri`, `cid`,
   `publication` at-uri, `publishedAt`, `lastPutAt`, `contentHash` (of the
   record file), `remoteCid` after sync. Drafts need no sidecar until
   published; optional `drafts/<slug>.meta.json` may note an intended
   publication target.
3. **posted/ is the source of truth for what exists remotely**; drafts/ is
   the staging area. Publishing = validate record → `putRecord` (rkey from
   filename) → move file to posted/ → write sidecar. This is a move, not a
   copy: a slug lives in exactly one place.

## Sync semantics (`springleaf sync`)

Reconcile posted/ against `com.atproto.repo.listRecords` (public read, no
auth) on the resolved PDS, by rkey:

| local | remote | action |
|---|---|---|
| present, hash unchanged | cid == sidecar `remoteCid` | clean |
| present, hash changed | cid unchanged | re-put (upsert) — local is newer |
| present, hash unchanged | cid changed | remote drift: report, require `--pull` (adopt remote bytes + update sidecar) or `--force` (re-put) |
| present, hash changed | cid changed | true conflict: block, demand explicit resolution |
| absent | present | `--pull` adopts remote record into posted/ (with sidecar); default reports orphan |
| present (drafts/) | — | untouched by sync; `publish` promotes |

No deletion ever happens automatically — cleanup is deliberate, per repo
policy. CIDs come from `listRecords` item metadata.

## Relationship to leaflet-native drafts

Getting a leaflet editor draft OUT (facts → canonical record) is a separate
planned capability: the fact graph → `pub.leaflet.content` conversion
mirrors Leaflet's own `src/utils/factsToPagesRecord.ts` (draft → publish
direction). The reverse (canonical record → new leaflet editor draft) would
need the Replicache push surface scoped by `experiments/02-draft-internals/`.
Both are later stages; neither changes the canonical format.

**2026-11-07 update:** both directions are now designed in
[`design/draft-extraction.glm53m.md`](draft-extraction.glm53m.md) —
extraction (26/30 block types linear-mappable, facet recovery incl.
byte-offset math), reverse authoring into existing drafts (6-mutation
template + Rust Yjs port ~1.7–2.2k LOC), and the new-draft gap resolution:
v0 authors into user-created drafts; v1 harvests `GET /new`'s 307
Location header for a fresh token id (one gated write probe).

## Open items

- `springleaf.toml` shape and whether publication default belongs there
  (yes, per experiment 03's pick-first-publication lesson).
- Whether `drafts/` may hold records targeted at multiple publications
  (sidecar `publication` defaulting to the toml value).
- Ordering of `sync` vs `publish` flags in the CLI surface; see
  `design/springleaf.glm53m.md` §CLI surface for the current command set —
  `publish run` and `sync` will extend it.
