---
type: Report
title: "Experiment 01 — draft list extraction from /home"
description: Characterizes the leaflet.pub /home draft-list surface (href ids, UUIDv7 timestamps, RSC flight payload), the get_leaflet_data RPC, and recommends the algorithm + output schema for `leafspring drafts list`.
resource: experiments/01-draft-list/REPORT.md
status: draft
generated:
  at: 2026-11-07
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e4caf9f73ffess4fRaY41iD9XH"
    title: "Session that probed /home and get_leaflet_data, derived the extraction algorithm and schema, and built the fixture + prototype extractor"
  - resource: "file:///home/rektide/src/leafspring/design/leafspring.glm53m.md"
    title: "Leafspring design doc — experiment brief and ground truth being verified"
  - resource: "https://leaflet.pub/home"
    title: "Production home page (live probe, 2026-11-07; raw HTML kept uncommitted in scratch)"
---

# Experiment 01 — draft list extraction from `/home`

Mission: prove a headless "list my drafts" path and specify extraction
algorithm + output schema for `leafspring drafts list`.

Probed 2026-11-07 with the `auth_token` cookie (read at runtime from
`~/.config/leaflet/auth-token`; value never printed, logged, or committed).
Raw evidence lives in scratch only (`/tmp/opencode/leafspring-01/`); everything
below and in this directory is synthetic or redacted.

## 1. The `/home` HTML surface

One `GET https://leaflet.pub/home` (cookie) → 200, ~263 KB, private/no-store.

### 1.1 Leaflet hrefs

Each homepage leaflet renders one absolutely-positioned card link:

```html
<a class="absolute w-full h-full top-0 left-0 …" href="/01a188c2-340b-7ddd-857c-03b900ac7d0d"></a>
```

Observed href inventory (real fetch): 30 uuid-shaped hrefs = exactly the 30
owned leaflets in the payload. Non-uuid hrefs seen: `/reader`, `/home`,
`/lish/createPub`, `/lish/{did}/{pub-rkey}/dashboard`, plus assets
(`/_next/static/...`, `/favicon.ico`, `/apple-icon.png`, `/manifest.json`,
`/home/icon-...`). Contributor leaflets do **not** get href cards.

**Match rule** (robust, no exclusion list needed): match
`href="/<uuid>"` where `<uuid>` is exactly 8-4-4-4-12 lowercase hex; then drop
reserved prefixes (`lish`, `reader`, `home`, `manual`, `api`, `_next`,
`manifest.json`, favicon/apple-icon). A bare `/uuid` with no further path
segment is a token id; anything with a `/` after the first segment is a route.
Do **not** require version-7: the account has one legacy UUIDv4 token
(pre-v7 adoption, created 2025-11-24) that is a perfectly ordinary leaflet.

```text
href="/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"
```

### 1.2 UUIDv7 timestamps and ordering

Token ids are UUIDv7: the first 48 bits are unix milliseconds. Decoding all 29
v7 ids and comparing with the payload's `created_at` rows: v7 ms ≈ row
`created_at` minus 0–50 ms (token generation vs row insert). The flight payload
row timestamp is the authority; v7 decode is a good fallback and cross-check.

Two orderings were measured:

- **Flight payload array**: sorted by id **ascending** (for v7s this is
  oldest-first; the legacy v4 sorts after all current v7s because `0927…` >
  `01a1…` lexically).
- **DOM card order**: `created_at` **descending** — v7s newest-first, and the
  legacy v4 card is **last** (its row is the oldest), *not* in lexical-id
  position.

So lexical UUID order equals chronological order **only among v7s**. Sort the
output by `created_at` desc (row value, v7-decode fallback, `null` last) to
replicate UI order and place pre-v7 tokens correctly.

Bonus observation: `root_entity` ids share the token's 48-bit timestamp and
version nibble (`<token-ms-hex>-7…` with a different tail), i.e. the root
entity is minted at token creation. Cheap structural sanity check, not
load-bearing.

## 2. The RSC flight payload

The page carries 13 `self.__next_f.push([1,"…"])` script chunks. Each chunk's
second element is a JSON string fragment; concatenated they form the flight
stream (plain JSON lines). The identity data (what `getIdentityData` returned
server-side) is a JSON object embedded in that stream. Locate it via the key
`"permission_token_on_homepage":` and brace-match backwards to the enclosing
object — no full RSC parsing needed.

Enclosing identity object keys (all observed in one fetch):

```text
atp_did, bsky_profiles, connectedAccount, contributor_leaflets,
contributor_publications, created_at, custom_domains, email, entitlements,
fetched_at, home_page, id, interface_state, metadata, notifications,
permission_token_on_homepage, publication_email_subscribers,
publication_memberships, publication_subscriptions, publications,
subscription, tutorial
```

`bsky_profiles` gives `{did, handle, record}` for the header; `publications`
holds the user's publications (with `record.url` custom domain and a
`draft_leaflet` token id); `home_page` is itself a token-id string.

### 2.1 `permission_token_on_homepage` entry shape (synthetic)

```json
{
  "archived": null,
  "created_at": "2026-06-25T03:36:35.199666+00:00",
  "permission_tokens": {
    "id": "019efcd9-d5b8-7ccc-8c00-03cccc000000",
    "title": "Older untitled-era draft",
    "description": null,
    "root_entity": "019efcd9-d5b8-7ccc-8c00-03aaaa000000",
    "leaflets_to_documents": [],
    "leaflets_in_publications": [],
    "permission_token_rights": [
      {
        "read": true, "write": true,
        "token": "019efcd9-d5b8-7ccc-8c00-03cccc000000",
        "created_at": "2026-06-25T03:36:35.199666+00:00",
        "entity_set": "019efcd9-d5b8-7ccc-8c00-03eeee000000",
        "create_token": true, "change_entity_set": true
      }
    ]
  }
}
```

Real fetch stats: 30 entries; 29 real titles + 1 `"Untitled"`; all `archived`
null; `leaflets_to_documents` empty on every entry (legacy linkage? never
observed populated — do not rely on it).

### 2.2 Publication linkage — where "published vs draft" lives

`permission_tokens.leaflets_in_publications[]` items carry the answer
(synthetic example):

```json
{
  "doc": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3syndoctid002",
  "tags": ["synthetic", "fixture"],
  "title": "first synthetic post",
  "leaflet": "01a0d0b8-89f0-7aaa-8a00-01aaaa000000",
  "archived": null, "reply_to": null, "reply_to_block": null,
  "documents": {
    "uri": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3syndoctid002",
    "data": {
      "$type": "site.standard.document",
      "path": "/3syndoctid002",
      "site": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/3synpubtid001",
      "title": "first synthetic post",
      "description": "a synthetic fixture document",
      "publishedAt": "2026-09-24T00:02:30.040Z",
      "tags": ["synthetic", "fixture"],
      "coverImage": null, "contributors": [], "bskyPostRef": null,
      "content": { "$type": "pub.leaflet.content", "pages": [ … ] }
    }
  },
  "email_only": false, "cover_image": null, "description": "…", "preferences": null,
  "publication": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/3synpubtid001",
  "publications": { "uri": "at://…", "record": { "$type": "site.standard.publication", "name": "…", "url": "https://example-pub.leaflet.pub", … } },
  "scheduled_publish_at": null, "scheduled_publish_data": null
}
```

**State derivation rule** (verified against 30 real leaflets):

| Condition | State |
|---|---|
| linkage item with `doc != null` (equivalently `documents != null`) | `published` — `doc` is the record at-uri, `documents.data` the full record incl. `publishedAt` |
| linkage items exist but all `doc == null` | `in_publication` — added to a publication, not published; `scheduled_publish_at`/`scheduled_publish_data` may carry scheduling info (one real leaflet sits here) |
| no linkage items | `draft` |

The DOM's "Published"/"Draft N ago" card labels are client-rendered and not
href-adjacent; do not scrape them — derive from the data as above. Real fetch:
3 published, 1 in_publication, 26 drafts (of 30 owned).

**Published URL derivation**: prefer the publication record's `url` field
(custom domain, e.g. `https://example-pub.leaflet.pub`) + `documents.data.path`
(= `/` + doc rkey); else `https://leaflet.pub/lish/{did}/{pub-rkey}/{doc-rkey}`
from the at-uris. This matches the design doc's slug rule.

### 2.3 `contributor_leaflets`

Same row shape, separate array (3 real entries). Caveat: they can **overlap**
owned homepage rows by token id (all 3 real ones did). Merge by token id —
`contributor: true` flag — rather than emitting duplicates.

## 3. `get_leaflet_data` RPC

`POST https://leaflet.pub/api/rpc/get_leaflet_data` with
`{"token_id": "<uuid>"}` (cookie sent; appears identity-scoped). Probed on a
fresh draft, a published leaflet, and the legacy v4.

Envelope:

```json
{ "result": { "error": null, "data": { … }, "count": null, "status": 200, "statusText": "" } }
```

`data` keys: `id, root_entity, blocked_by_admin, title, description,
permission_token_rights[], custom_domain_routes[], leaflets_in_publications[],
leaflets_to_documents[], publications[], publication_scheduled_posts,
reply_drafts`.

Differences vs the home payload:

- `permission_token_rights[]` additionally embeds an `entity_sets` object — a
  recursive echo of the token (with `publications`, `leaflets_to_documents`,
  nested rights). Useful for `drafts show`, not for the list.
- Adds `blocked_by_admin`, `custom_domain_routes`, `publication_scheduled_posts`.
- `leaflets_in_publications` items have the identical shape as §2.2.
- `reply_drafts` was `null` on all three probes (populated shape unknown —
  experiment 02 territory).
- `leaflets_to_documents` again always `[]`.

Error path (fabricated-but-unregistered id): HTTP 200 with
`result.error.code = "PGRST116"`, `result.status = 406` (PostgREST "no rows").
So check `result.error`, not just the HTTP status.

## 4. Recommended algorithm for `drafts list`

**Default: single-request RSC parse of `/home`.** Fetch once with the cookie,
parse the flight stream, read `permission_token_on_homepage` (+ merge
`contributor_leaflets`), derive state per §2.2.

| Option | Requests | Yields | Fragility |
|---|---|---|---|
| href scrape only | 1 | ids only (+v7 timestamps) | breaks only if card markup dies; weakest data |
| **RSC parse (chosen)** | **1** | ids, titles, descriptions, created_at, rights, publication linkage, doc uris, publishedAt, publication records, did/handle | breaks if payload key renames — but keys track the DB schema (`permission_token_on_homepage`), the most stable surface available |
| RSC + per-id `--fetch` | 1 + N | adds blocked_by_admin, entity_sets detail, custom_domain_routes, freshness | N+1 requests; per-id value is `drafts show`-shaped, little for the list |

Justification: titles and full publication linkage are **already** in the one
`/home` response, so enrichment buys almost nothing for listing; keep `--fetch`
as an optional verification/freshness mode (and future `drafts show` shares the
RPC client). The href set stays useful as a **cross-check**: emit
`href_payload_mismatch` when the href id set ≠ payload id set — an early alarm
that the page shape changed (the prototype does this).

Fallback ladder if the payload key vanishes: href scrape (ids + v7 timestamps)
→ per-id RPC impossible (ids unknown) → hard error with a clear message.

**Pagination cap risk**: 30 leaflets observed in both DOM and payload; unknown
whether the homepage truncates at some N. Nothing headless enumerates beyond
`/home` today (`pull`/`getFactsForRoots` need known ids). Flag in output if
`count == 30` (suspected page size)? Leave as a documented unknown.

## 5. Output schema

`leafspring drafts list --json` (schema `version: 1`):

```json
{
  "version": 1,
  "source": "home-rsc",
  "fetched_at": 1794009762780,
  "identity": { "did": "did:plc:aaaaaaaaaaaaaaaaaaaaaaaa", "handle": "example.bsky.social" },
  "count": 5,
  "href_payload_mismatch": [],
  "leaflets": [
    {
      "token_id": "01a19bc1-50a0-7bbb-8b00-02bbbb000000",
      "created_at": "2026-11-02T10:15:00.999999+00:00",
      "created_at_source": "row",
      "uuid_version": 7,
      "title": "A synthetic draft title long enough to demonstrate truncation",
      "description": "description field demo",
      "root_entity": "01a19bc1-50a0-7bbb-8c00-03cccc000000",
      "archived": false,
      "on_homepage": true,
      "contributor": false,
      "rights": { "read": true, "write": true },
      "state": "draft",
      "publication": null,
      "published_doc_uri": null,
      "published_at": null,
      "published_url": null
    },
    {
      "token_id": "01a0d0b8-89f0-7aaa-8a00-01aaaa000000",
      "created_at": "2026-09-24T00:02:30.168453+00:00",
      "created_at_source": "row",
      "uuid_version": 7,
      "title": "first synthetic post",
      "description": null,
      "root_entity": "01a0d0b8-89f0-7aaa-8b00-02aaaa000000",
      "archived": false,
      "on_homepage": true,
      "contributor": false,
      "rights": { "read": true, "write": true },
      "state": "published",
      "publication": {
        "uri": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/3synpubtid001",
        "name": "Example Synthetic Publication",
        "url": "https://example-pub.leaflet.pub"
      },
      "published_doc_uri": "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/3syndoctid002",
      "published_at": "2026-09-24T00:02:30.040Z",
      "published_url": "https://example-pub.leaflet.pub/3syndoctid002"
    }
  ]
}
```

Field notes:

- `created_at_source`: `"row"` (payload `created_at`) | `"uuidv7"` (decoded) |
  `null` (pre-v7 id with no row timestamp — only possible on the fallback
  path).
- `uuid_version`: 7 or 4; explains why `created_at` may be absent for legacy
  ids on the fallback path.
- `state`: `"draft" | "in_publication" | "published"` per §2.2; `archived` is
  separate (row `archived` non-null) so archived-but-published stays
  representable.
- `--fetch` mode sets `source: "home-rsc+rpc"` and merges
  `blocked_by_admin` / `reply_drafts` per leaflet.
- Multiple publications per leaflet: emit the latest by `publishedAt`; full
  linkage remains available for `drafts show`.

Human default (what the prototype prints):

```text
STATE           CREATED      TITLE                              TOKEN-ID                             PUBLISHED-URL
published       2026-10-28   in javascript we used to have a…   01a1848b-d6b0-7998-a4ff-512b314461ef  https://…/3myxrsnvmzs2s
draft           2026-10-23   Missiles: the O.G. disposable…     01a1661c-b7b9-777a-91e5-9bd9b27bd64c
…
30 leaflets (3 published, 27 draft/unpublished) as jauntywunderkind.bsky.social
```

Columns `STATE | CREATED | TITLE (38 chars, ellipsized) | TOKEN-ID |
PUBLISHED-URL`; `*` suffix marks contributor entries; footer with identity and
counts; `href_payload_mismatch` surfaces as a warning line.

## 6. Surprises

1. **A legacy UUIDv4 token id exists** — the v7 assumption would silently drop
   it; ordering by v7-lexical would also misplace it (it sorts "newest"
   lexically but is the oldest leaflet).
2. **Titles are in the HTML** — no per-id fetching needed for a titled list;
   `--fetch` is optional freshness/verification, not a requirement.
3. **`permission_token_on_homepage` is id-ascending** while the DOM is
   created_at-descending — sort explicitly, don't trust either order.
4. **Contributor rows can duplicate owned rows** by token id — merge, don't
   concatenate.
5. **RPC errors ride a 200**: envelope `result.status` 406 + PostgREST code
   `PGRST116` is the "unknown token id" signal.
6. `leaflets_to_documents` never populated across home + 3 RPC probes — dead
   or rare linkage; don't build on it.

## 7. Files

- `REPORT.md` — this report.
- `fixture-home.html` — synthetic home page: 5 leaflets (newest draft with long
  title, published v7, mid draft, legacy v4 "Untitled", contributor-only
  published), publication with custom domain, known routes, 3 flight push
  chunks; v7 ids are computed from their displayed dates, so timestamp decoding
  round-trips.
- `extract-list.py` — algorithm prototype: `./extract-list.py fixture-home.html
  [--human]`, or `--live [--fetch]` against production (token read at runtime).
- `probe-home.sh` — fetch `/home` to scratch + surface stats.
- `probe-rpc.sh` — read-only `get_leaflet_data` for given ids, dumps to scratch.

Open for experiment 02: `reply_drafts` populated shape, entity_sets depth,
`pull`/`getFactsForRoots` enumeration (pagination cap question).
