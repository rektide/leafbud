---
type: Experiment
title: "Experiment 02 — draft internals: fact model, Replicache sync, Yjs feasibility"
description: "Read-only recon of one draft's fact graph and pull/push protocol against leaflet.pub, with the minimal push-mutation plan for headless authoring and a byte-verified Yjs text encoder verdict. Gates stage 5 of design/springleaf.glm53m.md."
resource: experiments/02-draft-internals/REPORT.md
status: draft
generated:
  at: 2026-11-07
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e4caf9f71ffe8CFOaaaUHpI50r"
    title: "Session that mapped the draft fact model, pulled/pushed shapes, and proved the Yjs encoder byte-identical against yjs 13.6.18"
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet"
    title: "Leaflet checkout @600c7b5f (app/api/rpc/[command]/*, src/replicache/*, src/utils/*, drizzle/schema.ts, supabase/migrations)"
  - resource: "file:///home/rektide/src/springleaf/design/springleaf.glm53m.md"
    title: "Springleaf design doc (stage 5 gating, experiments policy)"
---

# Experiment 02 — draft internals

Read-only recon against `https://leaflet.pub` (2026-11-07) plus heavy source
reading of `~/archive/hyperlink-academy/leaflet` @ `600c7b5f`. All examples
below are synthetic (fabricated uuids/strings); raw probe outputs stayed in
scratch. No credentials anywhere in this directory; scripts read the token
from `~/.config/leaflet/auth-token` at runtime.

Probed drafts: 30 token ids scraped from `/home`; 26 had no publication
linkage (`leaflets_in_publications: []`). Deep-dived two pure drafts: a
48-fact doc draft (14 blocks) and a 5-fact minimal draft (h1 seed only,
matching `createLeaflet(pageType:"doc", firstBlocks:["h1"])` exactly).

## 1. The fact model

A leaflet is an **EAV graph** over UUID entities:

- `entities(id, set)` — uuids grouped into an `entity_sets` row
  (`drizzle/schema.ts:301-315`).
- `facts(id, entity, attribute, data jsonb, version, author_did, created_at,
  updated_at)` (`drizzle/schema.ts:317-332`). `data` is a tagged union:
  `{type, ...payload}` where `type` selects the payload shape
  (`src/replicache/attributes.ts:592-735`). `version` is unused in practice
  (always 0 live); `author_did` is set only on comment-author facts.
- `permission_tokens(id, root_entity, title, description)` — the "draft" the
  homepage lists; `permission_token_rights(token, entity_set, read, write,
  create_token, change_entity_set)` binds tokens to entity sets
  (`drizzle/schema.ts:465-488`).

Attribute vocabulary (`src/replicache/attributes.ts`): ~120 attributes across
Root/Page/Block/Comment/Mailbox/Link/Embed/Theme/… groups, each with
cardinality `one` (reusing the fact id on update) or `many` (fact id is the
identity; caller-generated). Structural attributes:

- `root/page` (ordered-reference, many): root → page, `position` =
  fractional-index key (`generateKeyBetween`, keys like `a0`, `a1V`, `a2G`).
- `page/type` (page-type-union, one): `"doc" | "canvas"`.
- `card/block` (ordered-reference, many): doc page → block (linear order via
  fractional positions). `canvas/block` (spatial-reference) is the canvas
  analogue, plus `canvas/block/{width,rotation,stack-order}`.
- `block/type` (block-type-union, one): 30 values — `text`, `heading`,
  `image`, `code`, `math`, `blockquote`, `horizontal-rule`, `datetime`,
  `rsvp`, `card`, `link`, `mailbox`, `embed`, `html`, `button`, `poll`,
  `bluesky-post`, `standard-site-post`, `standard-site-publication`,
  `members-only-delimiter`, `posts-list`, `recommended-pubs`, `reply`,
  `questions`, `signup`, `image-gallery`, `post-header`, `embedded-canvas`,
  `group`, `drawing` (`attributes.ts:648-685`).
- `block/text` (**text**, one): the payload — base64 of a Yjs v1 update
  holding a ProseMirror doc (`<paragraph>…</paragraph>` under a root type
  `prosemirror`). Headings add `block/heading-level` (number).

Chain: `root --root/page--> page --page/type--> "doc" --card/block--> block
--block/type--> "text" --block/text--> base64 yjs`. Everything else is
per-type ornament (e.g. `theme/*` on root, `block/code-language` on code
blocks).

**RPC surface** (`app/api/rpc/[command]/route.ts` + `lib.ts`): `POST
/api/rpc/<command>` with the zod-validated input as the JSON body; the cookie
gates identity (service-role supabase client does the reads; no per-user RLS
on this path). The response body is the handler's return value verbatim —
`get_leaflet_data` wraps itself in `{result: {data, error}}`, `pull` returns
the bare `PullResponseOKV1`, `push` returns `{"result":null}`.

### get_leaflet_data

Input `{"token_id": "<uuid>"}`. Output (`get_leaflet_data.ts:21-36`): the
permission token row plus nested joins — `permission_token_rights` →
`entity_sets` (with `root_entity`, nested tokens, publication linkage),
`leaflets_in_publications` (draft↔publication rows incl. published
`site.standard.document` record), `leaflets_to_documents` (standalone
published docs), `publications` (the publication's own draft leaflet),
`custom_domain_routes`, `publication_scheduled_posts`, `reply_drafts`.
Synthetic shapes: `fixtures/get_leaflet_data-pure-draft.json`,
`fixtures/get_leaflet_data-pub-linked.json`. Homepage title fallback chain:
`leaflets_in_publications[0].title || leaflets_to_documents[0].title ||
permission_tokens.title || "Untitled"` (`src/utils/getLeafletTitle.ts`); for
pure drafts the home list actually renders the **first text/heading block's
Yjs text** via `extractTitleFromFacts` (`src/utils/extractTitleFromFacts.ts`)
— that's why untitled drafts still show titles.

### getFactsForRoots

Input `{"roots": ["<uuid>", ...]}`; handler hardcodes `max_depth: 3` and
groups rows by `root_id` (`getFactsForRoots.ts`). The SQL
(`supabase/migrations/20261028000000_get_facts_for_roots_set_based.sql`) is a
recursive CTE that joins `facts.entity` to `data->>'value'` whenever
`data->>'type' IN ('reference','ordered-reference','spatial-reference')`,
so the walk covers root → page → block (depth 2) with a level to spare.
Row shape in `fixtures/getFactsForRoots-doc.json`. Caveat: depth-3 from the
root does **not** reach `comment/reply` chains or deeply nested group
children; pull (below) returns the full client view instead.

## 2. The pull contract

`pull.ts` accepts `PullRequestV0` (rejected: `VersionNotSupported`) or
`PullRequestV1` wrapped with the token id:

```json
{
  "pullRequest": {
    "pullVersion": 1,
    "schemaVersion": "",
    "profileID": "<any string>",
    "cookie": null,
    "clientGroupID": "<any string>"
  },
  "token_id": "<token uuid>"
}
```

All four V1 fields are required by zod; `schemaVersion` is not actually set
by Leaflet's client (`src/replicache/index.tsx` — Replicache default `""`),
`profileID`/`clientGroupID` are caller-chosen. `cookie` accepts null/string/
number/`{order, ...}`; **null or a legacy number yields the full-snapshot
response**, an object with `{order, cvrID}` takes the CVR diff path
(`src/replicache/cvr.ts:54-77`, Redis-backed; an unknown cvrID falls back to
full snapshot too). Verified live: one POST with `cookie: null` returned
`PullResponseOKV1`:

```json
{
  "cookie": { "order": 1794009930586, "cvrID": "<uuid>" },
  "lastMutationIDChanges": {},
  "patch": [
    { "op": "clear" },
    { "op": "put", "key": "initialized", "value": true },
    { "op": "put", "key": "<fact-id>", "value": { "<fact row + indexes>" } },
    { "op": "put", "key": "draft_contributors", "value": [] }
  ]
}
```

- **Version map format**: none — the CVR strategy versions facts by Postgres
  `xmin` (`row_version`) server-side and only diffs them; clients never see
  versions, just puts/dels keyed by fact id (`src/replicache/cvr.ts`).
- **Fact serialization** (`src/replicache/utils.ts:9-23`): the fact row
  (`id, entity, attribute, data, version, author_did, created_at,
  updated_at`) plus `indexes: {eav: "<entity>-<attribute>-<factid>", vae:
  "<value>-<attribute>"}` for reference-typed facts (client-side scan
  indexes). Full example in `fixtures/pull-full-snapshot.json`.
- **Client rows**: `replicache_clients(client_group, client_id, last_mutation)`
  (`drizzle/schema.ts:820-830`) surface as `lastMutationIDChanges` — a fresh
  `clientGroupID` gets `{}`.
- Extras: `initialized`, `publication_{title,description,tags}`,
  `post_preferences`, `draft_contributors` (`cvr.ts:219-240`).
- A no-change CVR pull echoes the cookie back with an empty patch — the poll
  fast path. Reproducible via `scripts/probe_leaflet.py pull <token-id>`.

## 3. The push contract

`push.ts` input: `{pushRequest: PushRequestV1, rootEntity: "<root entity
uuid>", token: {id: "<token uuid>"}}` (client sends the whole token object;
only `id` is validated). `pushVersion` must be 1. Mutations are
`{id, name, args, timestamp, clientID}` — ids must increase monotonically per
`clientID` within a `clientGroupID`. Full example:
`fixtures/push-minimal-authoring.json`.

Per-push processing (`push.ts:99-213`):

1. Advisory xact lock on a hash of the token id (serialization per draft).
2. `getClientGroup` loads `{clientID → lastMutationID}` from
   `replicache_clients` for the group (`src/replicache/utils.ts:25-41`).
3. Per mutation in order: **replay protection** — skip if
   `mutation.id <= clientGroup[clientID]`; otherwise record the id, dispatch
   by name against the `mutations` map (unknown names silently skipped).
   A throwing mutation is **skipped-but-acknowledged**: its buffered writes
   roll back to the checkpoint, and the lastMutationID still advances — a
   poison mutation can't wedge a client.
4. `replicache_clients` upsert (`onConflictDoUpdate` on `client_id`).
5. `flush()` executes buffered writes; a Supabase broadcast poke on channel
   `rootEntity:<id>` tells live clients to re-pull.

Write checks (`src/replicache/cachedServerMutationContext.ts`):

- `createEntity` succeeds only if the token holds `write=true` on the target
  `permission_set` (entity set).
- `assertFact` checks write permission on `fact.entity`; cardinality-`one`
  attributes reuse the existing fact's id (update-in-place semantics); facts
  carrying an `author_did` (comments/answers) are writable only when the
  session cookie resolves to that DID.
- The session cookie (`auth_token` → `email_auth_tokens` → `identities`)
  resolves `sessionDid` once per push; **anonymous pushes still work** for
  non-authored facts where the token has write rights.
- `block/text` writes get special treatment: concurrent/stale values are
  merged with `Y.mergeUpdates` at flush (`cachedServerMutationContext.ts:33-47,
  292-310`) — a fresh standalone update is a first-class merge input
  (verified, §5).

### Mutation vocabulary (44, `src/replicache/mutations.ts:1569-1614`)

`retractAttribute, addBlock, addCanvasBlock, groupCanvasBlock,
ungroupCanvasBlock, moveCanvasBlockLayer, addLastBlock, outdentBlock,
moveBlockUp, moveBlockDown, addPageLinkBlock, addEmbeddedCanvasBlock,
addPublicationNavPage, addPublicationNavLink, removePublicationNavEntry,
moveBlock, moveBlocks, assertFact, retractFact, removeBlock, deleteEntity,
moveChildren, increaseHeadingLevel, archiveDraft, toggleTodoState,
createDraft, createQuestionAnswer, discardQuestionAnswer, createEntity,
addPollOption, removePollOption, addGalleryImage, removeGalleryImage,
updatePublicationDraft, updateLeafletMetadata, toggleDraftContributor,
toggleCollapsedBlocks, createFootnote, deleteFootnote, createEditorComment,
createEditorCommentReply, deleteEditorComment, deleteEditorCommentReply,
editEditorComment`

The workhorses for authoring are `addBlock`, `assertFact`, `createEntity`,
`updateLeafletMetadata`.

### Minimal authoring plan

**Crucial finding: a brand-new draft leaflet cannot be created via push.**
No mutation mints `entity_sets`/`permission_tokens`; new drafts come from the
Next.js server action `createNewLeaflet` (`actions/createNewLeaflet.ts`) →
`createLeaflet` (`src/utils/createLeaflet.ts`) → one SQL statement
(`insertLeaflet`, `src/utils/insertLeaflet.ts:47-70`) that inserts entity
set + entities + permission token + full rights + seed facts and links the
token to the operator's homepage. Calling it headless means a POST with the
build-time `Next-Action` id — a write op needing its own probe and operator
go-ahead (open item for stage 5). Everything *inside* an existing draft is
push-authorable today:

One push call, six mutations in order (fixture:
`fixtures/push-minimal-authoring.json`; template facts cross-checked against
`createLeaflet.ts` seed facts and live getFactsForRoots):

1. `addBlock` — h1 title block. Args: `parent` = page entity, `permission_set`
   = entity_set from token rights, **caller-generated** `factID` (uuid) and
   `newEntityID` (uuid), `type: "heading"`, `position` = fractional key
   (first block `"a0"`, next `"a1"`, …).
2. `assertFact` — `{entity: newEntityID, attribute: "block/heading-level",
   data: {type:"number", value: 1}}` (cardinality one; id optional).
3. `assertFact` — `{entity: newEntityID, attribute: "block/text",
   data:{type:"text", value: <base64 yjs from §4>}}`.
4. `addBlock` — text block (`type: "text"`, new ids, `"a1"`).
5. `assertFact` — its `block/text`.
6. `updateLeafletMetadata` — `{title}` (optional; runs server-side, writes
   `permission_tokens.title`; homepage title already derives from the h1
   text via `extractTitleFromFacts`).

Caller-generated ids are mandatory where noted: cardinality-`many` facts
(`card/block`) take their identity from the client-chosen `factID`, and
entity ids come from the caller because client and server run mutations
independently — a server-generated id would diverge
(`mutations.ts:791-799`). Ordering: `createEntity` (inside `addBlock`)
precedes facts on that entity; mutations execute in array order. Replay is
idempotent through `lastMutationID` — a retried push with the same ids is
harmless.

## 4. Yjs feasibility verdict: **YES — confidently feasible**

What `block/text` holds: a base64 **Yjs v1 update** of a ProseMirror doc:
root type `prosemirror` → `paragraph` (YXmlElement) → YXmlText → string.
`createYjsText` (`src/utils/createYjsText.ts`) builds exactly that and
stamps **no** `meta.schemaVersion` (plaintext helpers don't;
`components/Blocks/TextBlock/schemaVersion.tsx:44-48`) — version 0 is
editable by every schema-checking client, so unstamped is the *safe* choice.

Evidence (all reproducible from this directory + scratch node):

1. **Full binary format recovered and validated.** Manual lib0-varint + Yjs
   item decoder (`scripts/yjs_decode.py`, format from yjs v13.6.20 sources:
   `Item.write`, `writeClientsStructs`, `Content*.write`, `writeDeleteSet`).
   Parses real live draft blocks end-to-end (9/14 values fully; the other 5
   are 3-byte `b5ec6d` non-updates — `Y.applyUpdate` throws on them; they're
   empty-block garbage readers must tolerate). Annotated binary walkthrough:
   `fixtures/yjs-samples.md`.
2. **Encoder reproduced byte-for-byte.** `scripts/yjs_encode_text.py` emits
   40 bytes for `"Hello world"` — verified **identical** to yjs 13.6.18
   `encodeStateAsUpdate` output for the same `doc.clientID`, including the
   empty-string (2-struct) case. The doc is 3 structs + empty delete set;
   fully deterministic, zero prior state.
3. **Any clientID works.** A structurally identical update with a
   substituted client id applies cleanly (`<paragraph>Hello world</paragraph>`).
   springleaf can mint a random uint32 per draft — collision with the
   browser's client only matters for clock interleaving, which merges handle.
4. **Server merge path verified.** `Y.mergeUpdates([ourUpdate,
   otherClientEdit])` — exactly what `cachedServerMutationContext.flush`
   does to text facts — yields a correct merged doc with our update as base.
5. **Consumer verified.** `applyUpdate` + `getXmlFragment("prosemirror")`
   renders our update; title extraction and `storedBlockTextIsEmpty` read
   the same representation.

Caveats: multi-paragraph docs and rich marks (bold/link facets) would need
more `paragraph` items or `ContentFormat` items (ref 6) — out of scope for a
v0 "single text insert" encoder; the reader (`drafts show`) should still
decode generally (our decoder handles refs 1-9). Encoding size is trivial
(~27 bytes + varstring(text)).

## 5. What this unblocks / open items

- **Stage 5 authoring is buildable** as: pick/create draft → pull (or
  getFactsForRoots) for parent/positions → build 6-mutation push → send via
  `/api/rpc/push` with cookie auth. `rootEntity` and entity_set come from
  `get_leaflet_data`.
- **Open item 1 — new-draft creation** is server-action-only (Next-Action
  id discovery + a write probe + operator go-ahead) or upstream could expose
  an RPC; until then `drafts new` can't be pure-RPC.
- **Open item 2 — push is a write**: today's probes were read-only; the
  first live push test needs operator go-ahead (use a throwaway draft).
- Minor: `publication_scheduled_posts`/`reply_drafts` were null/absent on
  pure drafts (present in the type per `get_leaflet_data.ts:32-33`).

## Reproducing

```sh
scripts/probe_leaflet.py home            # token ids from /home
scripts/probe_leaflet.py roots <id>      # root entities for a token
scripts/probe_leaflet.py facts <root>    # fact graph
scripts/probe_leaflet.py pull <id>       # full-snapshot pull
scripts/yjs_encode_text.py "text" [--client N] [--hex]
scripts/yjs_decode.py <b64 | --hex HEX>
```

Fixtures: `fixtures/*.json` (synthetic RPC shapes),
`fixtures/yjs-samples.md` (annotated binary structure).
