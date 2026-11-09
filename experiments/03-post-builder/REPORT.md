---
type: Experiment
title: "Experiment 03 — post record construction, validation, and derivation"
description: Dry-run construction of a publishable site.standard.document record for leaflet.pub; identity chain, publication inventory, lexicon validation, rkey/URL derivation, and an unexecuted putRecord script.
resource: experiments/03-post-builder/REPORT.md
status: draft
generated:
  at: 2026-11-07
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e4caf9f70ffeNzpcr75XIKbdG6"
    title: "Session that resolved identity, inventoried publications, built and validated the record"
  - resource: "file:///home/rektide/src/leafbud/design/leafbud.glm53m.md"
    title: "Leafbud design: auth, post records, slug rule, staged build plan"
  - resource: "file:///home/rektide/archive/letta-ai/social-cli/src/commands/publish.ts"
    title: "Exemplar implementation: slug=rkey rule, byte-offset facets, URL derivation"
  - resource: "file:///home/rektide/archive/hyperlink-academy/leaflet/lexicons/"
    title: "Leaflet lexicon JSONs at repo checkout (site.standard.*, pub.leaflet.*)"
---

# Experiment 03 — post record construction, validation, and derivation

**No write was executed.** Identity resolution and `listRecords` are public;
`putRecord.sh` exists as a gated, never-run artifact. All requests made:
one `resolveHandle`, one DID-document fetch, four `listRecords`, one DNS TXT
query, one well-known fetch.

## 1. Identity resolution chain

Handle: `jauntywunderkind.bsky.social`

| Step | Surface | Result |
|---|---|---|
| 1 | DNS TXT `_atproto.jauntywunderkind.bsky.social` | **no record** (bsky.social-hosted handles use the HTTPS well-known method) |
| 2 | `GET https://jauntywunderkind.bsky.social/.well-known/atproto-did` | `did:plc:zjbq26wybii5ojoypkso2mso` |
| 3 | `GET https://bsky.social/xrpc/com.atproto.identity.resolveHandle?handle=…` | same DID — cross-check ✓ |
| 4 | `GET https://plc.directory/did:plc:zjbq26wybii5ojoypkso2mso` (DID document) | `alsoKnownAs: ["at://jauntywunderkind.bsky.social"]` → **bidirectional verification holds**; service `#atproto_pds` (`AtprotoPersonalDataServer`) = `https://puffball.us-east.host.bsky.network` |

- **DID:** `did:plc:zjbq26wybii5ojoypkso2mso`
- **PDS:** `https://puffball.us-east.host.bsky.network`
- Note: the modern PLC directory host is `plc.directory` (plc.bsky.social
  404s on `/did:plc:…`); the DID doc keys on `id` (standard DID-core), not
  `did`.

## 2. Publication inventory

`listRecords` against the PDS, both collections, cursor followed to exhaustion:

- **`pub.leaflet.publication`: 0 records.** The legacy collection has nothing
  from this account — leafbud only needs to read it for compat, not write.

- **`site.standard.publication`: 3 records** (all carry `$type`):

| rkey | name | url | notes |
|---|---|---|---|
| `skyreader-links` | JauntyWunderKind's links | `https://linkblogs.skyreader.app/did:plc:zjbq26wybii5ojoypkso2mso/` | third-party skyreader linkblog; extra field `skyreaderLinkblog` (open lexicon, tolerated) |
| `blento.self` | JauntyWunderKind | `https://blento.app/jauntywk.bsky.social` | third-party blento profile-page publication; `preferences: {baseColor, accentColor}` (not the lexicon's preferences shape — open object tolerated) |
| `3molsbwkh522o` | JauntyWunderKind's Technautical Expeditions | `https://technautical.leaflet.pub` | **Leaflet-native**: icon blob, `site.standard.theme.basic` colors, full lexicon `preferences` |

Supporting evidence — `site.standard.document` listRecords (10 records): 7
posts under skyreader-links, 3 under technautical (`first post`,
`A chance to peer in`, `bring back callee, the other this`). Every document
uses an inline `pub.leaflet.content`, and every rkey is a **TID with
`path` = `/{rkey}`** (the web app's own convention — see derivation below).

### Target pick: `3molsbwkh522o` (technautical)

Evidence:

1. It is the operator's personal Leaflet-native publication — its existing
   posts include literally `first post`; a protocol probe there is low-stakes
   and in keeping with the publication's experimental character.
2. It carries a real `url` (custom leaflet.pub subdomain), exercising the
   publication-domain URL derivation end-to-end.
3. skyreader-links is an automated linkblog feed (7 machine-shaped posts);
   a test record would pollute it and possibly its syndication. blento.self
   fronts the operator's profile on a third-party app — also not a sandbox.

`built-record.json` therefore sets
`site: at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.publication/3molsbwkh522o`.

## 3. sample.md — v0 converter subset

Four constructs on purpose: baseline ASCII paragraph (no facets), an ATX
heading, the facet-bearing paragraph, and a closing paragraph. The
facet-bearing paragraph puts **five non-ASCII code points before the first
facet** — `É`+`é` (Émigré), `—` (em-dash, 3 bytes), `é` (cafés), `é`
(Montréal) — forcing byte-offset arithmetic.

## 4. built-record.json

Structure ( `$type` discriminators at every union hop):

```
site.standard.document
├─ site, title, description, tags[], publishedAt, path, textContent
└─ content ($type: pub.leaflet.content)          ← union member typed
   └─ pages[ ($type: pub.leaflet.pages.linearDocument)
        ├─ id: 01a1b353-1c2f-7d4d-9284-3fa5d8782a4a   (UUIDv7-style, frozen for reproducibility)
        └─ blocks[ ($type: pub.leaflet.pages.linearDocument#block)
             ├─ header level 1  "A dry-run probe"
             ├─ text   baseline paragraph (no facets)
             ├─ text   "Émigré — …" + 4 facets
             └─ text   closing paragraph ] ]
```

### Byte-offset math (the point of the exercise)

Plaintext of the facet-bearing block (167 bytes / 161 chars — the 6-byte
delta is exactly 4 two-byte accents + 1 three-byte em-dash):

```
Émigré — the cafés of Montréal sit before the facet below: see the Leaflet homepage …
```

| facet | feature | bytes | slice round-trip |
|---|---|---|---|
| link | `#link` uri=https://leaflet.pub | 73–89 | `'Leaflet homepage'` ✓ |
| bold | `#bold` | 108–118 | `'Bold claim'` ✓ |
| italic | `#italic` | 120–132 | `'italic aside'` ✓ |
| code | `#code` | 138–149 | `'inline code'` ✓ |

Proof of byte-vs-char: "Leaflet homepage" sits at **character index 67 but
byte offset 73** (+6). UTF-16 or codepoint slicing would misplace every facet
after the accents; `validate.py` asserts this explicitly.

## 5. Validation checklist

`validate.py` (stdlib Python, exits 1 on any failure) — **all 78 checks
pass**. Checks performed, each read from the lexicon JSONs rather than
hand-copied:

**site.standard.document (top level)**
- parses as JSON object; `$type` exact
- required fields present: `site`, `title`, `publishedAt`
- `site`: string, AT-URI shape, points at the target publication
- `title`: string, ≤500 graphemes / ≤5000 bytes (25/25)
- `description`: string, ≤3000 graphemes / ≤30000 bytes (91/93)
- `tags`: array of strings; each item ≤50 graphemes / ≤100 bytes
- `publishedAt`: ISO 8601 datetime parseable
- `path`, `textContent`: strings

**content union**
- `content.$type` = `pub.leaflet.content`, present in the document lexicon's
  `content.refs` union
- `pages` non-empty; each page's `$type` in `{linearDocument, canvas}`
- each page has required `blocks`; optional `id` is a string

**blocks**
- each page-block: `$type` = `pub.leaflet.pages.linearDocument#block`,
  required `block` present, `alignment` (if set) a knownValue
- each inner block `$type` in the 28-member block union
- header: `plaintext` present, `level` integer 1–6
- text: `plaintext` present, `textSize` (if set) in enum

**facets**
- `index.byteStart/byteEnd` integers ≥ 0, `byteStart < byteEnd ≤` byte length
- every feature `$type` in the facet lexicon's feature union; `#link` has
  required `uri`
- **round-trip**: slice each block's plaintext **bytes** at the facet index →
  decodes to exactly the marked span (`'Leaflet homepage'` etc.)
- facet spans re-derived independently from plaintext match the record
- non-ASCII shift assertion: byte offset > char index before first facet

**frontmatter + derivation**
- `title`/`description`/`tags`/`slug` in the record match `sample.md`
- rkey (default rule) = frontmatter slug; `path` = `/{rkey}`
- fallback URL and publication-domain URL match the derivation below

Finding worth carrying into the crate: **lexicon union refs beginning with
`#` are lexicon-relative** — `#link` inside `pub.leaflet.richtext.facet`
resolves to `pub.leaflet.richtext.facet#link`. A validator that compares
record `$type`s against raw `#…` refs rejects valid records (mine did, first
run). Also: grapheme counts here use code points minus combining marks;
general code needs real grapheme segmentation.

## 6. Derivation

- **rkey (default rule):** `slugify(title)` → from frontmatter slug →
  `leafbud-protocol-probe`. social-cli's rule: default rkey = slug, TID
  only on explicit opt-out. Observed counter-evidence: the leaflet.pub web
  app itself uses TID rkeys with `path=/\{rkey\}` — both work because
  Leaflet's middleware rewrites `{domain}/{path}` →
  `/lish/{did}/{pub-rkey}/{rkey}` and looks up **by rkey**, so the invariant
  that matters is `path == "/" + rkey`. Two of the three publications
  (`skyreader-links`, `blento.self`) also use slug-style rkeys despite
  `"key": "tid"` in the lexicon — PDSs don't enforce keyType and Leaflet
  ingests them fine.
- **Fallback URL:**
  `https://leaflet.pub/lish/did:plc:zjbq26wybii5ojoypkso2mso/3molsbwkh522o/leafbud-protocol-probe`
- **Publication-domain URL:** publication `url` =
  `https://technautical.leaflet.pub` →
  `https://technautical.leaflet.pub/leafbud-protocol-probe`
  (trailing-slash stripped per social-cli `fetchPublicationUrl` flow).

## 7. putRecord.sh — NOT EXECUTED

`putRecord.sh` holds the exact write for the later gated stage:
`createSession` (`identifier=$ATPROTO_HANDLE`, `password=$ATPROTO_APP_PASSWORD`)
→ Bearer `accessJwt` → `putRecord {repo: did, collection:
"site.standard.document", rkey, record: built-record.json}` against
`https://puffball.us-east.host.bsky.network`. It refuses to run without
`GATED_OPERATOR_APPROVAL=yes` set by the operator, and never echoes the
token. Syntax-checked (`bash -n`) only.

## Files

- `sample.md` — frontmatter + v0-subset markdown
- `built-record.json` — the exact record a dry run would print
- `validate.py` — lexicon + offset validator (run: `./validate.py`)
- `putRecord.sh` — gated, unexecuted write script
