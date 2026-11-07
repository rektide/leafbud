---
type: Design
title: Draft extraction — leaflet drafts ⇄ canonical records
description: Build-ready design for converting a leaflet editor draft's fact graph into springleaf's canonical site.standard.document record and back via Replicache push; block coverage matrix, Yjs text recovery, the new-draft gap, Rust port notes, and verification gates.
resource: design/draft-extraction.glm53m.md
status: draft
generated:
  at: 2026-11-07
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e4c9d0a5dffe6GpmPVETxrbW6S"
    title: "Session that read the Leaflet conversion sources, designed the coverage matrix and both directions, and wrote this document"
  - resource: "opencode://ses_e48e1411effeFlQHJe2NwO9UjN"
    title: "Session that built `drafts create` (5c): the /new redirect-harvest client and CLI gating, plus the 5c outcomes addendum"
  - resource: "file:///home/rektide/src/springleaf/design/records-canonical.glm53m.md"
    title: "The canonical format this pipeline targets (pure site.standard.document records)"
  - resource: "file:///home/rektide/src/springleaf/experiments/02-draft-internals/REPORT.md"
    title: "Fact model, pull/push contracts, minimal-mutation plan, Yjs encoder/decoder proofs"
  - resource: "file:///home/rektide/src/springleaf/experiments/03-post-builder/REPORT.md"
    title: "Canonical record shape, byte-offset facet math, lexicon validation"
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/factsToPagesRecord.ts"
    title: "Leaflet's own draft→record conversion — the primary extraction reference (@600c7b5f)"
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/src/utils/publishLeaflet.ts"
    title: "Record envelope assembly: site/path/rkey/publishedAt/preferences, hooks, side effects"
---

# Draft extraction — leaflet drafts ⇄ canonical records

Two converters, one file format. **Extract** turns a leaflet editor draft
(private fact graph) into the canonical record of
`design/records-canonical.glm53m.md`. **Author** turns a canonical record
into mutations pushed into an *existing* draft. The reference for extraction
is Leaflet's own `src/utils/factsToPagesRecord.ts` (@`600c7b5f`); the
reference for authoring is `experiments/02-draft-internals/` (pull/push
contracts, byte-verified Yjs encoder). This document is the build spec — no
implementation here.

## Fact-graph input

| Source | When | Caveat |
|---|---|---|
| `POST /api/rpc/pull` (cookie `null` → full snapshot) | default | full client view: deep chains, extras (`publication_{title,description,tags}`, `draft_contributors`) |
| `POST /api/rpc/getFactsForRoots` | already-have-roots fast path | hardcodes `max_depth: 3` — reaches root→page→block but not comment/reply chains or deeply nested group children (experiment 02 §1) |
| `POST /api/rpc/get_leaflet_data` | always (envelope) | token row, `root_entity`, rights (→ `entity_set` for push), publication linkage |

Fact rows carry `indexes` only on pull; both forms feed the same scan index.

## Extraction algorithm (draft → canonical record)

Mirror `factsToPagesRecord.ts` — where Leaflet's publish drops a block, we drop
it too, but we *warn* instead of staying silent. Steps:

1. **Scan index.** Build `entity → attribute → [facts]` (the pull shape of
   `scanIndexLocal`, `src/replicache/utils.ts`). Cardinality-`one` attributes
   read `[0]`; `many` reads all.
2. **Root and start page.** `root_entity` from `get_leaflet_data`. Start page
   = first `root/page` fact by `position` (fractional key), matching
   `factsToPagesRecord.ts:646-651`. Multiple `root/page` facts (publication
   nav machinery, `addPublicationNavPage`) are out of scope — warn if >1.
3. **Spine walk.** `root --root/page--> page --page/type--> "doc"|"canvas"
   --card/block--> block --block/type--> …`. Page recursion: `card` and
   `embedded-canvas` blocks point (`block/card`) at another page; serialize it
   recursively and append to `pages[]`; the start page is unshifted first
   (`factsToPagesRecord.ts:646-651`, `221-245`).
4. **Ordering.** Sort `card/block` facts by `(position, fact id)` —
   lexicographic fractional key (`a0`, `a1V`, `a2G`…), id tiebreak
   (`src/replicache/getBlocks.ts:77-81`). Positions come from
   `generateKeyBetween` (npm `fractional-indexing`).
5. **Linear blocks.** A block with `block/is-list=true` heads a list: its
   children are `card/block` facts *on the block entity*, recursed by depth
   (`getBlocks.ts:90-134`); the flat block list is folded into nested lists by
   `parseBlocksToList.ts`, then split into runs of `ordered`/`unordered`
   (`splitListByStyle`, `factsToPagesRecord.ts:825-838`) emitting
   `pub.leaflet.blocks.orderedList`/`unorderedList` with `startIndex` from
   `block/list-number`, `checked` from `block/check-list`.
6. **Per-type conversion.** The coverage matrix below. Every block lands as
   `{$type: "pub.leaflet.pages.linearDocument#block", block: …}`; alignment
   from `block/text-alignment` maps to the `lex:…textAlign*` knownValues
   (`factsToPagesRecord.ts:743-756`).
7. **Members-only.** Blocks after a `members-only-delimiter` are flagged; the
   delimiter itself stays (`factsToPagesRecord.ts:762-774`). Our canonical
   record keeps the *full* content — the same fully-inflated record Leaflet
   persists locally (`publishLeaflet.ts:410-413`); the publish-time PDS copy
   is truncated separately (`truncateDocumentRecordForPDS`). Warn when a
   delimiter is present so a future publish step knows.
8. **Yjs text.** `block/text` → plaintext + facets (next section).
9. **Envelope** (`site.standard.document`), following `publishLeaflet.ts`:
   - `title`: first heading/text block's text (the `extractTitleFromFacts`
     rule the homepage already uses; experiment 02 §1), else
     `permission_tokens.title`, else "Untitled".
   - `description`: `permission_tokens.description`; `tags`: publication
     linkage's `tags` when the draft sits in a publication.
   - `site`: publication at-uri when linked, else `https://leaflet.pub/p/{did}`
     (`publishLeaflet.ts:418-419`; did from the `/home` identity payload).
   - `publishedAt`: linked published document's `publishedAt`, else now
     (`publishLeaflet.ts:403-407` precedence).
   - `path`: `"/" + slug` with the slug rule from
     `experiments/03-post-builder/REPORT.md` §6; `textContent`: block
     plaintexts joined with `\n` (experiment 03 precedent).
   - `content`: `{$type: "pub.leaflet.content", pages}`.
10. **Output.** Pure record JSON to `drafts/<slug>.json`
    (`design/records-canonical.glm53m.md` layout) plus a structured warn list
    (block type, entity, reason). Extraction never fails on unmappable
    blocks — it fails only on structural errors (no root page, malformed
    positions).

### Block-type coverage matrix

The 30 `block/type` values (`src/replicache/attributes.ts:648-685`) against
`pub.leaflet.blocks.*`. Classes: **direct** (facts copy into fields),
**convert** (restructuring, recursion, or a publish-time side effect),
**canvas-only**, **draft-only** (Leaflet's own publish drops it — skip +
warn). Citations are `factsToPagesRecord.ts` unless noted.

| `block/type` | Canonical block | Class | Rule |
|---|---|---|---|
| `text` | `blocks.text` | direct | `block/text` yjs → plaintext/facets; `block/text-size` → `textSize` (:352-362) |
| `heading` | `blocks.header` | direct | `block/heading-level` → `level` (floor, default 1); text+facets (:331-342) |
| `blockquote` | `blocks.blockquote` | direct | text+facets (:343-351) |
| `math` | `blocks.math` | direct | `block/math` → `tex` (:494-501) |
| `code` | `blocks.code` | direct | `block/code` → `plaintext`, `block/code-language` → `language`, **root's** `theme/code-theme` → `syntaxHighlightingTheme` (:450-461) |
| `horizontal-rule` | `blocks.horizontalRule` | direct | no facts (:296-301) |
| `signup` | `blocks.signup` | direct | no facts (:638-643) |
| `post-header` | `blocks.postHeader` | direct | `post-header/compact` (:630-637) |
| `button` | `blocks.button` | direct | `button/text` + `button/url`; dropped if either missing (:534-544) |
| `bluesky-post` | `blocks.bskyPost` | direct | `block/bluesky-post` value → `postRef{uri,cid}`; `bluesky-post/{host,view}` (:246-261) |
| `standard-site-post` | `blocks.standardSitePost` | direct | uri + optional `size`, `show-publication-theme` (:262-279) |
| `standard-site-publication` | `blocks.standardSitePublication` | direct | uri + optional flag (:280-295) |
| `recommended-pubs` | `blocks.recommendedPubs` | direct | `recommended-pubs/compact` (:593-600) |
| `reply` | `blocks.reply` | direct | button/prompt texts; `key` = entity id (:601-613) |
| `questions` | `blocks.questions` | direct | button text, `audience` (≠"anyone" only), `showPending`; `key` = entity id (:614-629) |
| `members-only-delimiter` | `blocks.membersOnlyDelimiter` | convert | audience fallback: explicit `block/members-only-audience` else tiers-implied else `"paid"`; sorted `tierIds` (:302-330) |
| `posts-list` | `blocks.postsList` | convert | ~10 optional facts; `readerControls` on ⇒ write `readerSearch/TagFilter/Sort` all defaulting true (:545-592) |
| `link` | `blocks.website` | convert | `link/url/src/title/description`; `link/preview` image needs an upload side effect → v0 warn + omit `previewImage` (:432-449) |
| `embed` | `blocks.iframe` \| `blocks.html` | convert | legacy split: `embed/html` wins over `embed/url`; `height` default 600; `embed/aspect-ratio` `"w/h"` parsed (:179-208) |
| `html` | `blocks.html` | convert | same handler as `embed` |
| `image` | `blocks.image` | convert | **requires `uploadImage` hook** (PDS blob write). v0 extraction: warn + skip — the lexicon demands a BlobRef we cannot mint without writing (:365-387) |
| `image-gallery` | `blocks.imageGallery` | convert | child entities via `gallery/image` ordered by `position`; per-image upload ⇒ v0 warn + skip; `format/gap/max-width` (:388-431) |
| `card` | `blocks.page` | convert | `block/card` → subpage serialized recursively and appended to `pages[]`; `page-link/display` unless default (:221-234) |
| `embedded-canvas` | `blocks.embeddedCanvas` | convert | same recursion; `image/alt` → `alt` (:235-245) |
| `drawing` | `blocks.drawing` | convert | `drawing/view-box` + `drawing/stroke` sorted by fact id; fills derived (`drawingFills`); rounding (:462-493) |
| `poll` | `blocks.poll` | convert | **requires `uploadPoll` hook** — a separate `pub.leaflet.poll.definition` PDS record ⇒ v0 warn + skip; options from `poll/options` → `poll-option/name` (:502-533) |
| `group` | — (inline `linearDocument`) | canvas-only | serialized only inside canvas pages by `canvasBlocksToRecord` (:943-949); invisible in linear flow |
| `datetime` | — | draft-only | publish returns `undefined` (:216) |
| `rsvp` | — | draft-only | :217 |
| `mailbox` | — | draft-only | :220 |

Derived outputs with no `block/type` of their own: `blocks.orderedList` /
`blocks.unorderedList` (§step 5), and the page-level theme object
(`pub.leaflet.theme.page` from `theme/{card-background,primary,accent-background,accent-text}`
— `factsToPagesRecord.ts:691-718`; v0 warn + skip, colors need hsba parsing).

**Headline: 30 types → 15 direct + 11 convert (26 linear-mappable) + 1
canvas-only + 3 draft-only (skip+warn), plus 2 derived list types.** With
side-effect hooks unavailable read-only, v0 extraction additionally skips
image, image-gallery, and poll blocks (warn): a lexicon-valid record cannot
carry them without blob writes.

Canvas pages (`page/type=canvas` → `pub.leaflet.pages.canvas`, spatial
`canvas/block` + `x/y/width/rotation/stack-order`, background image, pattern,
mobile-view: `factsToPagesRecord.ts:669-689`, `931-987`) are specified by the
same source but deferred: v0 warns and omits the page. Same for the record
envelope's `coverImage` (`root/cover-image`, `publishLeaflet.ts:366-388`) and
`theme` (`extractThemeFromFacts`) — upload-dependent, warn + skip.

## Yjs text recovery (`block/text` → plaintext + facets)

What the value holds (experiment 02 §4, `fixtures/yjs-samples.md`): base64 of
a Yjs v1 update encoding a ProseMirror doc — named root type `prosemirror`
→ `paragraph` (YXmlElement) → YXmlText → ContentString items. A second named
type `meta` (YMap) may carry `schemaVersion` (ContentAny). Marks are
ContentFormat items (ref 6, key + lib0 `writeAny` value) interleaved with the
strings inside the XmlText; the delta view exposes them per run as
`strong`, `em`, `code`, `strikethrough`, `underline`,
`highlight{color:"1"|"2"|"3"}`, `link{href}` — exactly what Leaflet's
`YJSFragmentToFacets` maps to `pub.leaflet.richtext.facet#` features
(`factsToPagesRecord.ts:1087-1133`).

**Decoder port** (from `scripts/yjs_decode.py`, extended):

- lib0 primitives: unsigned varuint (7-bit, LSB-first, `0x80` continuation),
  sign-magnitude varint, varstring, `writeAny` tags 116–127.
- Struct walk: per client (descending id order), `info` bits (`0x80` origin,
  `0x40` rightOrigin, `0x20` parentSub, low 5 = content ref 0–9);
  `parentInfo` present only when neither origin is set; delete set last.
- **Item integration** (new, required): true order inside a YXmlText is the
  linked-list order defined by origin/rightOrigin pointers, not clock order.
  Assemble items per parent type by linking origins (~100 LOC; the standard
  Yjs integration step). Without it, edited/tombstoned text orders wrong.
- **Delta reconstruction**: walk the integrated XmlText; ContentString
  accumulates text, ContentFormat opens/closes mark runs (null value =
  close), producing plaintext + per-run attributes → facets with **UTF-8
  byte offsets** (same arithmetic proven in experiment 03 §4).

**v0 facet scope** — recovered: `bold` (`strong`), `italic` (`em`), `code`,
`strikethrough`, `underline`, `link` (uri from `href`); `highlight` emitted
without color (slot-1 behavior; slots 2/3 need theme facts). `hard_break`
XmlElement → `\n`. Deferred with warn: `footnote` (needs recursive resolution
of the footnote entity's own `block/text`, `factsToPagesRecord.ts:1004-1025`),
`didMention`/`atMention` XmlElements (:1028-1065), ContentEmbed items.
**Parity rule:** Leaflet's publish reads only `nodes[0]` of the fragment
(`factsToPagesRecord.ts:151-152, 167-169`) — extraction decodes the first
paragraph and warns if more nodes exist.

**Tolerance rules**: the 3-byte value `b5ec6d` seen on empty blocks is *not*
a valid update (`Y.applyUpdate` throws) — treat as empty string, never error
(`fixtures/yjs-samples.md` "Anomaly"). Any undecodable value → warn + empty
plaintext, block still emitted. Missing `block/text` fact → empty
(`factsToPagesRecord.ts:163-164` does the same). `meta.schemaVersion` is read
but never asserted by our encoder: unstamped is version 0, editable by every
client — the safe choice (experiment 02 §4).

## Reverse: canonical record → existing draft

**Constraint: push authors *into* an existing draft. No mutation mints
`entity_sets`/`permission_tokens` — draft creation is server-action-only**
(experiment 02 §3, the crucial finding). The new-draft gap is next section.

Preconditions: token id with `write` right; `get_leaflet_data` for
`root_entity` + `entity_set` (→ `permission_set` arg); a pull (or
getFactsForRoots) for the target page entity and its last `card/block`
position. Then one `POST /api/rpc/push`
(`{pushRequest: PushRequestV1, rootEntity, token:{id}}`) carrying mutations
in record order — the proven 6-mutation template scaled up
(`fixtures/push-minimal-authoring.json`):

1. Per block, in order: `addBlock {parent: page-entity, permission_set,
   factID: <uuid-v7>, type, newEntityID: <uuid-v7>, position,
   list?: {listStyle, checklist}}` — `addBlock` internally does
   `createEntity` + `card/block` ordered-reference + `block/type` (+ auto
   `block/is-list` when the parent is a block: `src/replicache/mutations.ts:250-310`).
2. Follow-on `assertFact`s per coverage matrix, e.g. heading →
   `block/heading-level` (number) + `block/text` (yjs base64); code →
   `block/code` + `block/code-language`; button → `button/text` + `button/url`;
   lists nest via `addBlock` with `parent` = the parent *block* entity.
3. Subpages (`blocks.page`): `createEntity` for the page, `assertFact
   root/page` (ordered-reference, appended position), `page/type` `"doc"`,
   then `block/card` on the card block, then its blocks.
4. Tail: `updateLeafletMetadata {title, description?}` — server-side write
   to `permission_tokens` (`mutations.ts:1260-1284`).

**Fractional positions**: `generateKeyBetween(prev, null)` per block starting
from the draft's existing last key — appends never collide with concurrent
editor inserts between keys. **Ids**: uuid v7 for every caller-generated
`factID`/`newEntityID` (cardinality-`many` facts take identity from the
client-chosen id; Leaflet's own client mints v7 —
`src/replicache/clientMutationContext.ts:56`). Mutation ids `1..n`,
monotonic per `clientID`; replay is idempotent via `lastMutationID`
(experiment 02 §3).

Reverse coverage: header, text, blockquote, code, math, horizontalRule,
button, website (link minus preview), lists, subpages, bskyPost,
standardSite{Post,Publication}, membersOnlyDelimiter, postHeader, signup,
recommendedPubs, postsList, reply/questions (mint a fresh entity id for
`key` — the record's key is a stale reference in a new draft; warn),
poll (draft-side only: option entities + `poll/options` facts — the PDS poll
record is publish-time). **v0 warn + skip**: image, imageGallery (need the
`minilink-user-assets` Supabase storage upload surface,
`src/utils/addImage.ts:74-80` — unscoped), drawing (v1; data is all
present), facets (**the v0 encoder is plaintext-only** — a fresh
single-paragraph doc; facet authoring needs ContentFormat items, v1),
alignment/theme facts. Reverse appends; it never deletes — replacing draft
content would use `removeBlock`/`retractFact` sequences (they exist in the
44-mutation vocabulary) and stays a deliberate v1 step, per the no-automatic-
prune policy.

## The new-draft gap

`drafts new` cannot be pure-RPC today. Options:

- **(a) Next.js server-action POST.** The `Next-Action` id is build-time,
  embedded in the client bundle, changes per deploy; capture means scraping a
  JS chunk or observing a browser create. It is a write → gated on operator
  approval anyway. Highest fragility, no advantage over (d).
- **(b) Playwright browser automation.** Ops-side tool, not CLI — already
  policy (`design/springleaf.glm53m.md` stage 5).
- **(c) User creates drafts in the web UI; springleaf lists and picks.**
  Zero new write surface beyond push itself; `drafts list` already exists.
- **(d) `GET /new` harvest — verified in source.**
  `app/(app)/(identity)/new/route.ts:7-15` is a plain GET route
  (`force-dynamic`, `force-no-store`) that calls `createNewLeaflet` with
  `redirectUser: true`; `actions/createNewLeaflet.ts:56-59` then
  `redirect("/${permTokenId}?focusFirstBlock…")` — a 307 whose **Location
  header carries the new token id**. One authenticated GET (no Next-Action
  id, no body) creates a doc draft (h1 seed, watermark fact, homepage link —
  `addToHomepage` defaults true) and hands us the id. It is still a write:
  needs one operator-approved live probe before shipping.

**Recommendation: v0 = (c)** (list → pick → author into), **v1 = (d)** after
its approval probe, with (a) as fallback only if the `/new` route changes.
(b) stays out of the CLI permanently.

## Rust port notes

| Piece | Approach | LOC (est.) |
|---|---|---|
| lib0 varint reader/writer | hand-rolled (`varuint`, sign-magnitude `varint`, `varstring`, `writeAny`) — subset is tiny; fixture-tested against the Python scripts' outputs | ~150 |
| Minimal Yjs text encoder | port of `scripts/yjs_encode_text.py`: 3 structs (2 for empty), deterministic, byte-identical (proven vs yjs 13.6.18); clientID = random u32 per draft | ~80 |
| Yjs update decoder | port of `scripts/yjs_decode.py` + item integration (origin/rightOrigin linking) + delta/mark reconstruction | ~400–450 |
| Fractional indexing | hand-rolled `generateKeyBetween`/`generateNKeysBetween` (npm README is the spec; same integer-pair key space ⇒ keys sort against existing `a0`/`a1V` keys), known-vector tests; crates (`fractional_index`, …) exist but are thin — adopt one only if outputs match npm vectors | ~80 |
| uuid v7 | `uuid` crate, feature `v7`, `Uuid::now_v7()` — matches Leaflet's client convention | ~5 |
| Extraction core | scan index, spine walk, coverage matrix, envelope, warn list | ~500–700 |
| Reverse builder | record → mutation batch, position/id minting, dry-run printing | ~250–350 |
| drafts-crate push client | `POST /api/rpc/push` envelope on the existing cookie HTTP layer | ~150 |
| CLI (`drafts extract` / `drafts author`) | clap subcommands, `--json`, dry-run default | ~100–150 |

Total ≈ **1.7–2.2k LOC** across `crates/drafts` (fact graph, yjs, fractional)
and `crates/leaflet` (record types via jacquard codegen, already staged).
No calendar estimates anywhere, by policy.

CLI surface addition (extends `design/springleaf.glm53m.md` §CLI surface):

```
springleaf drafts extract <token-id> [--out FILE] [--json]   # pull → canonical record + warns
springleaf drafts author <token-id> <record.json>            # prints mutation batch; --yes sends
```

`author` refuses without `--yes` (push is a write; `--dry-run` is the
default posture, matching `publish run`).

## Verification gates

1. **Unit vectors** (no network): encoder golden bytes — "Hello world" =
   the 40-byte hex of `fixtures/yjs-samples.md` sample 1; empty text =
   2-struct form. Decoder on sample 2 (tombstone, `meta.schemaVersion`,
   delete set) asserting plaintext `"Synthetic sample text"` and schemaVersion
   1. `b5ec6d` tolerance test.
2. **Synthetic extraction goldens**: `fixtures/getFactsForRoots-doc.json` →
   expected canonical record; a new synthetic fixture with one block of each
   of the 30 types asserting the warn set is exactly
   `{datetime, rsvp, mailbox}` (+ canvas-page skip).
3. **Round-trip, in-memory**: `experiments/03-post-builder/built-record.json`
   → mutation batch → replay against an in-memory fact map (createEntity +
   assertFact with cardinality-`one` update-in-place) → re-extract →
   normalized equality (id remap + position normalization). Catches
   encoder/decoder asymmetry and matrix drift in both directions.
4. **Gated live probe — the acceptance gate**: read-only `pull` of one real
   draft (operator-chosen) → canonical record → operator compares against
   the web editor render. Token read at runtime, never logged; findings
   promote to fixtures only as synthetic shapes.
5. **Gated live push** (separate, later): one throwaway draft, explicit
   operator go-ahead (experiment 02 open item 2), then the full
   author→pull→extract round-trip against production.

## Open items

- Item-integration fixtures: real edited blocks with multi-client
  interleaving and marks — need one more read-only pull to harvest shapes
  (synthetic re-encodings via yjs can cover most).
- Mention facets (`footnote` recursion, `didMention`, `atMention`) and
  highlight slot colors — v0.5.
- Canvas page serialization and reverse drawing/image (storage upload
  surface) — v1, rules already cited.
- Publish-side parity for canonical-from-draft records:
  `truncateDocumentRecordForPDS` (members-only) and
  `maybeOffloadPagesToBlob` (large docs) if such records are later published
  through springleaf.
- `b5ec6d` provenance (why empty blocks carry it) — irrelevant beyond the
  tolerance rule, noted for completeness.

## Outcomes (2026-11-07, after the 5a build)

Shipped as `springleaf drafts export` (the doc's "extract" name did not
survive the build). Divergences and findings, accepted:

- **embedded-canvas → skip+warn**: its target is a canvas page, which v0
  defers; the convert-with-recursion listing overstated it.
- **orderedList `startIndex` always present** (Leaflet parity: `listStart || 1`).
- **Heading levels** outside 1..=6 clamp with a warn rather than emitting a
  lexicon-invalid record.
- **Mention/footnote plaintext retained** (facets still deferred) — the doc
  was silent on their text contribution; `YJSFragmentToString` parity kept.
- **`experiments/02` fixture prose** says the yjs encoder emits 40 bytes;
  the committed fixture is 63 — stale text, left unedited (other authors'
  evidence files).
- **Known gap for 5b**: `publish run` rejects the standalone HTTPS `site`
  form (its publication fetch assumes an at-uri) — exported standalone
  records are valid but cannot flow through `publish run` until 5b adds the
  standalone path.
- Decoder parity: field-for-field vs the python reference on shared
  fixtures; integration-order semantics proven (middle insert, concurrent
  tail appends, delete splits); one real 163-byte corrupt value fails
  identically in Rust, python, and yjs itself — tolerance path exercised
  live. The gated live-pull acceptance gate (§gates item 4) was exercised
  read-only: draft `019f0b10…` exported clean.

## Outcomes (2026-11-07, after the 5b build)

Shipped as `springleaf drafts author <token-id> --from <slug-or-file>`
(dry-run default, exit 2; `--yes` executes the push — never run live,
mock-verified end to end). Divergences from the reverse section, all
flagged:

- **Facets ARE encoded** (the reverse section deferred them: "the v0
  encoder is plaintext-only"). The stage-5b mission upgraded this: the
  encoder emits origin-chained ContentFormat items mirroring the
  decoder's recovery, and `decode(encode(text, facets)) == (text,
  facets)` holds exactly for run-normalized facet lists — the shape both
  the markdown converter and extraction emit. `highlight` (slot 1)
  rides along with the mark set.
- **Standalone publish-run URL rule**: the 5a gap fix derives
  `{site}/{rkey}` (i.e. `https://leaflet.pub/p/{did}/{rkey}`), Leaflet's
  own `getDocumentURL` rule (no publication ⇒ base = doc.site + path)
  and a real route (`/p/[didOrHandle]/[rkey]`) — not the `/lish` form,
  which needs a publication rkey standalone records do not have.
- **Encoder parity evidence**: byte-identical to the committed python
  reference on shared fixtures (including the two `block/text` values
  inside `push-minimal-authoring.json`, reproduced verbatim with client
  979896792); the parity goldens were regenerated by running the script.
- **Fractional keygen**: ported from fractional-indexing 3.2.0 exactly
  (the dgreensp implementation Leaflet pins — not the older rocicorp
  README vectors, which describe a different midpoint algorithm);
  verified against vectors generated by running the package itself.
- **bskyPost** reverse writes a minimal `{post:{uri,cid}}` thread blob —
  our own extraction reads it back, but Leaflet's editor stores the full
  getPostThread response, so the block's rich render may be degraded
  (warned at build time).
- **Poll** options resolve from the pollRef's PDS record (a public
  getRecord the CLI performs before building); unresolved refs warn and
  skip the block.
- **Alignment facts** are warn + skip exactly as the reverse section
  says (not upgraded with facets).
- Minor shape choices: `block/list-number` is asserted only when
  `startIndex != 1` (mirroring extraction's `listStart || 1` read);
  nested list runs share one position chain per enclosing item;
  `questions`/`reply` record `key`s warn only (draft-local reference,
  per the reverse section).
- The gated live push (§gates item 5) remains the open acceptance step;
  the README documents the operator procedure (throwaway draft → dry-run
  review → `--yes` → re-export and diff).

## Outcomes (2026-11-07, after the 5c build)

Shipped as `springleaf drafts create` — the (d) `/new` harvest promoted
from v1 plan to implementation with the operator gate intact. Decisions
and divergences, all flagged:

- **The Location shape is source-verified, not live-captured.** No `/new`
  redirect exists among experiments/02's fixtures (its probes were
  read-only). The parser therefore accepts any 3xx (the source's
  `redirect()` emits 307), relative or absolute Location, and requires
  only a uuid first path segment; the assumed exact form
  (`/<token-id>?focusFirstBlock&addToHome`, with `&welcomeModal`
  inserted when the route's query carries it) is asserted against
  `actions/createNewLeaflet.ts` @`600c7b5f` in unit tests. The first
  operator-gated `--yes` run is the acceptance probe; a non-uuid target
  (e.g. an anonymous redirect to `/login`) alarms with the segment in
  the message.
- **The mint is a write even when anonymous.** `insertLeaflet` never
  consults the cookie — only the homepage-link CTE does — so a
  missing/invalid token still mints an orphan draft nobody lists. The
  CLI's dry-run prints that hazard and suggests `auth status` first;
  the module docs record it.
- **Redirects are disabled for the probe request only.** `LeafletClient`
  carries a second inner reqwest client with `Policy::none()`; every
  other request keeps reqwest's default follow behavior. A loopback
  test proves exactly one request leaves the client — following the 307
  would GET the new draft's page and lose the id.
- CLI posture mirrors `drafts author`: dry-run default (prints exactly
  what will happen, exit 2, the transport is never entered), `--yes`
  executes through an injected transport (`CreateFn`, the `PushFn`
  pattern), output is the new token id plus `drafts author`/`drafts
  list` follow-on suggestions, with `--json` v1 schemas for both the
  plan and the outcome.
- The gated live probes remain open: both the `/new` mint (this stage's
  acceptance step) and the stage-5b push are still unexecuted live from
  this codebase.
