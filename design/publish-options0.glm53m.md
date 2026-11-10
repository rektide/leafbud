---
type: Design
title: "Publish-surface options inventory and frontmatter/CLI exposure design"
description: "Every Leaflet publish-page and post-settings option mapped to its server effect and current Leafbud support, with a tri-state frontmatter/CLI design for the four show* preferences and a stay-refused list for everything unsound."
resource: /design/publish-options0.glm53m.md
status: proposal-awaiting-concurrent-implementation
generated:
  at: 2026-11-10
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e3980e50bffe7VSTILelV4gWMV"
    title: "Operator question on publish options; pinned-archive UI inventory, Leafbud mapping, preferences exposure design and stay-refused analysis"
  - resource: /doc/release/README.md
    title: "Current 0.25 release boundaries including the preferences refusal and forced-off delivery"
  - resource: /design/native-publication0.gpt61s.md
    title: "Arguments surface, inherited-preferences semantics and native effects that flags do not suppress"
  - resource: /design/native-publish0.gpt61s.md
    title: "Current publish execution, dispatch fingerprint and settlement comparison"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Reserved delivery field and the tri-state missing/null/explicit precedent"
  - resource: /crates/leaflet/web/src/publication.rs
    title: "Arguments::wire today: title/description/tags/publishedAt, sendEmail forced false, preferences omitted"
  - resource: /crates/workflow/src/publication.rs
    title: "Preview effect-field guards including the nondefault-preferences blocker"
  - resource: /crates/leaflet/editor/src/write/metadata.rs
    title: "Working metadata routing through updatePublicationDraft"
  - resource: /crates/leaflet/records/src/projection.rs
    title: "supported_metadata refusal of cover/delivery/preferences and record-side refusal"
  - resource: /README.md
    title: "Current edit command flags"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/%28app%29/%28identity%29/%5Bleaflet_id%5D/publish/PublishPost.tsx
    title: "Publish page: details, backdate/schedule, tags, cover, share submission"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/%28app%29/%28identity%29/%5Bleaflet_id%5D/publish/ShareOptions.tsx
    title: "Share options: email, Bluesky, discover, quiet, test email"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "Server-side record construction, preferences resolution and linkage/newsletter effects"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts
    title: "updatePublicationDraft working-table and client-key writes"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/PostSettings.tsx
    title: "In-editor post settings toggles including post-publish discover"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/%28app%29/%28editor%29/%5Bleaflet_id%5D/actions/PublishButton.tsx
    title: "Editor Update path args including session-local publishedAt"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/scheduledPosts/save.ts
    title: "Scheduled-post row and separate action"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/%28app%29/%28identity%29/%5Bleaflet_id%5D/email/EmailOnlyForm.tsx
    title: "Email-only send surface with audience and scheduling"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/lexicons/pub/leaflet/publication.json
    title: "preferences lexicon defaults including publication-only fields"
---

# Publish options: inventory and exposure design

Archive citations below use `leaflet:` = the pinned clean archive at
`600c7b5f1f7b31b66fb626c293cc104312a12caf` (verified clean before this review),
and `leafbud:` = this repository's current code. Leaflet paths are relative to
the archive root.

## Plain-language answer

**What the Leaflet publish UI offers, and where Leafbud stands:**

- **Fully supported and wired today:** title, description, and tags. You set
  them in frontmatter (`title`, `description`, `tags`) or with
  `leafbud edit --title/--description/--tag/--clear-tags`. Saves write them to
  the same working tables the native editor uses, and publish sends them
  explicitly. An empty description clears; tags replace.
- **Supported as intent with a native clamp:** the publish date. Frontmatter
  `publishedAt` or `edit --published-at/--now`. A future date is never
  scheduled — Leaflet clamps it to "now" at publish, and preview says so.
- **Supported separately:** the editor watermark (frontmatter `theme`), which
  is a page setting, not a publish option.
- **Deliberately refused today:** the post **preferences** — Show in Discover,
  Show Comments, Show Mentions, Show Recommends. Leafbud currently sends
  nothing, so the native side inherits whatever state the draft/record already
  had, and preview blocks if that inherited state is nondefault because
  Leafbud cannot yet predict or verify the output. Email sending is forced
  off; Bluesky sharing is never invoked.
- **Not applicable / stay refused:** cover image (needs image upload),
  scheduling (a separate Pro-gated server action), share-to-Bluesky text, the
  quiet-publish toggle (for Leafbud it reduces to Show-in-Discover:false plus
  email-forced-off, which the design below covers), email-only sending, test
  emails, and reply submission.

**The design:** expose the four `show*` preferences as a tri-state frontmatter
`preferences:` mapping (absent = inherit native state, `{}` = explicit
"no overrides", keys present = explicit values) plus `edit
--preference key=bool` / `--clear-preferences` flags. Publish itself stays
flag-free and reads the prepared source, matching the established
preview-first-from-source precedent. This works because preferences are
ordinary fields of the public record — verifiable at settlement like title and
tags. Everything that is a **non-record side effect** (email, sharing) stays
refused until delivery verification is separately commissioned.

One pleasant surprise: Leaflet never records `showInDiscover: true` — only the
`false` opt-out is persisted — and "Post Quietly" is just "every channel off",
which for Leafbud is exactly our forced-off email/share plus
`showInDiscover: false`. No separate quiet capability is needed.

## Part 1 — Inventory of the native publish surface

### 1.1 The publish page

`PublishPost.tsx` is a two-step form: **Post Details** then **Share Options**,
with all flow-local state (toggles, tags, backdate, Bluesky draft) scoped per
document+destination in localStorage (`leaflet:…/publish/PublishPost.tsx:129-153`).

**Post Details step:**

1. **Title / description** — displayed (not edited) here; they come live from
   Replicache (`publication_title`/`publication_description`,
   `PublishPost.tsx:169-184`) and are submitted as action args
   (`:307-308`). Editing happens in the editor header
   (`components/Pages/PublicationMetadata.tsx:126-152`) or the pre-publish
   post-details form (`app/(app)/(editor)/[leaflet_id]/actions/PublishButton.tsx:379-399`).
2. **Publish Date** (`BackdateOptions`, `PublishPost.tsx:637-759`) — a
   date/time picker; unset means "Now". Picking a future date switches the
   submit into **scheduling** when the owner has Pro
   (`:208-212`, `:652-660`, upsell `:743-748`); the chosen instant is
   submitted as `publishedAt` (`:311-312`). The server action clamps future
   dates to now (`actions/publishToPublication.ts:29-34`).
3. **Tags** — a `TagSelector`; with a draft, changes go through
   `updatePublicationDraft({tags})`, otherwise they stay local
   (`PublishPost.tsx:243-251`); submitted as `tags` (`:309`).
4. **Social Preview + cover image** — preview card using title/description/
   cover (`:439-473`); `CoverImageControls` uploads/crops/removes a
   `root/cover-image` fact on the document root (`:563-632`, entity read
   `:215-217`).

**Share Options step** (`ShareOptions.tsx`, state `:15-20`, defaults
bluesky/postToReaders/email all true, quiet false — `PublishPost.tsx:143-153`):

5. **Email** (newsletter-enabled publications only,
   `ShareOptions.tsx:100-129`) — submitted as `sendEmail`
   (`PublishPost.tsx:316`); server-side this claims the post's
   `publication_post_sends` slot (status pending/skipped) and kicks the
   newsletter job (`publishLeaflet.ts:624-659`).
6. **Send preview email** — a test send of the rendered email to one address
   (`sendPostPreview`, `ShareOptions.tsx:195-290`); publish-independent.
7. **Share on Bluesky** — a ≤300-char composer post (with site embed)
   submitted as a **separate action after publish**
   (`publishPostToBsky`, `PublishPost.tsx:343-363`); not an argument of the
   publish action.
8. **Show in Discover** — submitted as the boolean `showInDiscover` arg
   (`PublishPost.tsx:317`); this is baked into the record as the
   `showInDiscover:false` opt-out so it survives appview re-indexing
   (`publishLeaflet.ts:314-322`).
9. **Post Quietly** — checking it unchecks/forces off every other channel
   (`ShareOptions.tsx:85-96`); at submit it ANDs `sendEmail`/`showInDiscover`
   false (`PublishPost.tsx:314-317`). It is a UI composite, not its own arg.

**Submitted in the same call but configured elsewhere:**

10. **Post preferences** — `postPreferences` arg (`PublishPost.tsx:226-233`,
    `:313`) reads the Replicache `post_preferences` key, which the in-editor
    Settings popover writes: Show in Discover (post-publish only), Show
    Recommends, Show Comments, Show Mentions
    (`components/PostSettings.tsx:39-56`, toggles `:69-106`) via
    `updatePublicationDraft({preferences})`.
11. **Scheduling** — a future date plus Pro eligibility submits
    `saveScheduledPost` instead of publishing, storing
    `publication_scheduled_posts` (send_email, show_in_discover, full bsky
    post payload, revision; `PublishPost.tsx:273-302`;
    `src/scheduledPosts/save.ts:90-128`; Pro gate
    `src/scheduledPosts/eligibility.ts:5-20`).
12. **Replying-to** — a draft started from a reply block submits a reply
    record at publish (`PublishPost.tsx:78`, `:35`;
    `publishLeaflet.ts:583-613`).

The email-only flow (`app/(app)/(identity)/[leaflet_id]/email/`) is a separate
surface entirely: audience (all/free/paid), send now/scheduled/on-subscribe,
test email, Pro-gated (`EmailOnlyForm.tsx:34-56`, `:81-146`, `:103-114`).

### 1.2 Server-side effects and wire shape

`publishToPublication(args)` strips any `rkey`, clamps future `publishedAt` to
now, and calls `publishLeaflet` (`actions/publishToPublication.ts:11-39`). The
full argument surface (`PublishLeafletArgs`, `publishLeaflet.ts:76-99`):

```json
{
  "root_entity": "…", "leaflet_id": "…", "publication_uri": "…?",
  "title": "…?", "description": "…?", "tags": ["…"]?,
  "entitiesToDelete": ["…"]?, "publishedAt": "ISO?",
  "postPreferences": {"showInDiscover": false, "showComments": false,
                      "showMentions": false, "showRecommends": false}?,
  "sendEmail": true?, "showInDiscover": true?, "rkey": "(stripped)"
}
```

Preferences resolution — the part any exposure design must mirror exactly
(`publishLeaflet.ts:309-331`):

- `basePreferences` = (`postPreferences` arg, else the draft row's
  `preferences`, else `{}`) **minus** its `showInDiscover` key.
- `hideFromDiscover` = (`showInDiscover` arg, else the destructured
  `showInDiscover`, else the existing record's) `=== false`.
- Record/row `preferences` = `{...base, showInDiscover:false}` when hiding;
  else `base` when nonempty; else **omitted**. So `showInDiscover: true` is
  never persisted — records carry only the opt-out — and an explicit empty
  `postPreferences: {}` means "publish with no overrides" (clears).

Effects: PDS `putRecord` with title/site/path=`/rkey`/resolved date/
description/tags/coverImage blob/theme (standalone only)/contributors/
preferences (`publishLeaflet.ts:396-479`); `documents` +
`leaflets_in_publications`/`leaflets_to_documents` upserts that **replace**
title/description/tags/preferences on the draft row (`:493-530`; preferences
column is `Json | null` on both tables — `supabase/database.types.ts:990-998`,
`:1063-1070`); schedule deletion (`:552-558`); actor contributor upsert on
every publication-container publish (`:336-352`); reply record (`:583-613`);
first-publish mention notifications (`:615-622`); first-release newsletter
slot claim (`:624-659`).

`updatePublicationDraft` (the save side) updates the same working tables'
title/description/tags/**preferences** (full replace, not merge) and sets
client keys; its `localPublishedAt` argument only writes a Replicache key and
no server field — nothing in the pinned tree reads that key back
(`src/replicache/mutations.ts:1201-1258`, only match:
`:1254`). The server pull seeds the client keys from the draft row
(`src/replicache/cvr.ts:230-239`).

Defaults come from the lexicon: all four `show*` default `true`
(`lexicons/pub/leaflet/publication.json:44-84`, which also defines
publication-level `showPrevNext`/`showFirstLast`/`prevNextDirection`/
`showOtherPublicationsInTags` that merge from the **publication** record, not
the post — `src/utils/mergePreferences.ts:19-39`).

### 1.3 Mapping table: UI option → server effect → Leafbud today → set today

| UI option | Server effect | Leafbud today | How set today |
| --- | --- | --- | --- |
| Title | record `title`; draft-row `title` | **Supported-and-wired** (`leafbud:crates/leaflet/web/src/publication.rs:24-25`, `leafbud:crates/leaflet/editor/src/write/metadata.rs:19-23`) | frontmatter `title`; `edit --title` |
| Description | record `description` (omitted when unset); draft-row | **Supported-and-wired**; empty explicitly clears (`leafbud:README.md:501-502`, `leafbud:crates/leaflet/editor/src/write/metadata.rs:21`) | frontmatter `description`; `edit --description ''` |
| Tags | record `tags`; draft-row `tags` | **Supported-and-wired**; replace semantics | frontmatter `tags`; `edit --tag/--clear-tags` |
| Publish Date | record `publishedAt` (arg > existing > now; future clamped by action) | **Supported as intent**; future-clamp disclosed (`leafbud:crates/workflow/src/publication.rs:269-283`; wire `leafbud:crates/workflow/src/native/publication.rs:480-486`) | frontmatter `publishedAt`; `edit --published-at/--now` |
| Show in Discover | `showInDiscover` arg → `preferences.showInDiscover:false` opt-out in record + draft row | **Deliberately refused** (inherited; nondefault blocks — `leafbud:crates/workflow/src/publication.rs:322-329`) | not settable |
| Show Comments / Mentions / Recommends | `postPreferences` → record + draft-row `preferences` | **Deliberately refused** (same guard) | not settable |
| Post Quietly | composite: forces `sendEmail`/`showInDiscover` false | **Not applicable as separate option**: email is already forced false (`leafbud:crates/leaflet/web/src/publication.rs:25`); the discover half is the refused `showInDiscover` | — |
| Email to subscribers | `publication_post_sends` slot claim + newsletter job | **Deliberately refused** — forced `sendEmail:false` (`leafbud:design/native-publication0.gpt61s.md:190-192`) | — |
| Share on Bluesky | separate `publishPostToBsky` action + `bskyPostRef` | **Refused** — never invoked (`leafbud:design/native-publication0.gpt61s.md:191`) | — |
| Send test email | `sendPostPreview` action | **Not applicable** (separate surface, no record effect) | — |
| Cover image | `root/cover-image` fact → blob upload → record `coverImage` | **Not supported** (no upload; release projection refuses cover — `leafbud:doc/release/README.md:150-154`) | — |
| Scheduling (future date) | separate `saveScheduledPost` action, Pro-gated | **Refused**: preview blocks nonempty schedules (`leafbud:crates/workflow/src/publication.rs:290-306`); publish deletes schedules (`leaflet:publishLeaflet.ts:552-558`); authored future dates disclose clamping | — |
| Backdate (past date) | record `publishedAt` | **Supported** (same as Publish Date) | `publishedAt` |
| Replying-to | `reply_drafts` → reply record | **Refused** — nonempty reply drafts block (`leafbud:crates/workflow/src/publication.rs:290-306`) | — |
| Email-only post | separate `saveEmailPost` surface; `email_only` drafts cannot publish | **Refused** (`leafbud:crates/workflow/src/publication.rs:319-321`) | — |
| Publication-level prefs (prev/next etc.) | publication record `preferences` | **Not applicable** — Leafbud does not manage publication records | — |
| Editor watermark | page fact `theme/page-leaflet-watermark` | **Supported** — frontmatter `theme` (`leafbud:crates/leaflet/editor/src/write/metadata.rs:34-44`) | frontmatter `theme.leafletWatermark` |

## Part 2 — Design: exposing the sound ones

### 2.1 Selection principle

**Expose a publish option only if its entire effect lands in fields Leafbud
already verifies** — the public record and the direct draft/linkage rows.
Preferences qualify: they are record fields, resolved deterministically from
our own inputs. Everything else on the page is either a non-record side effect
(email, notifications, share), a separate server action (schedule, email-only,
test send, reply), or a resource upload (cover) — see §2.8.

Split naming follows the same principle: **`preferences:`** (new, this design)
owns record-verifiable display/feed fields; the reserved **`delivery:`** field
(`leafbud:design/frontmatter0.gpt61s.md:157`, `:184-187`) stays refused for
future email/share intent, whose verification is a separate uncommissioned
capability.

### 2.2 Frontmatter field and tri-state semantics

A typed top-level author field, mirroring the native/lexicon name:

```yaml
preferences:
  showComments: false
  showMentions: true     # explicit true is meaningful: it is persisted
  showRecommends: false
  showInDiscover: false  # only the false opt-out is ever recorded (§2.3)
```

Tri-state, mirroring `publication`/`path`/`publishedAt`
(`leafbud:design/frontmatter0.gpt61s.md:147-158`):

| Spelling | Meaning | Wire behavior |
| --- | --- | --- |
| absent | **Inherit**: no author opinion; native draft/record state stands | `postPreferences` and `showInDiscover` omitted — exactly today's wire (`leafbud:crates/leaflet/web/src/publication.rs:32-34`) |
| `null` | Same as absent (no intent), spelled deliberately | same |
| `{}` | **Explicit "no overrides"**: publish with defaults, clearing any inherited draft/record overrides | `postPreferences: {}` — the native clear spelling |
| keys present | **Explicit values** for exactly those keys | resolved per §2.3 |

Per-key tri-state comes free: a key you did not write is not sent, and native
resolution supplies it from observed state. This is the same
absent-explicit/preserve distinction `publishedAt` uses
(`leafbud:crates/workflow/src/publication.rs:269-283`).

### 2.3 The resolution model Leafbud must implement

Preview and settlement must reproduce `leaflet:publishLeaflet.ts:309-331`
exactly, with observed values standing in for the native client's live state:

1. `base` = explicit three-key subset of the author mapping (inherit case:
   observed draft-row `preferences`, minus `showInDiscover`).
2. `showInDiscover` resolves authored → observed draft row → observed record →
   default `true`; `hideFromDiscover` = resolved `=== false`.
3. Predicted record/row `preferences` = `{...base, showInDiscover:false}` if
   hiding; else `base` if nonempty; else absent/`null`.

Two native quirks to disclose rather than fight:

- **`showInDiscover: true` is dispatch-only.** It is never written to the
  record or draft row; preview must say so, and settlement must predict the
  record *without* it. (Sending `true` still matters: it overrides a `false`
  sitting in the draft/record, flipping output to "no preferences object".)
- **Publish replaces, never merges, the draft row** (`leaflet:publishLeaflet.ts:508-516`
  upserts resolved-or-null), and `updatePublicationDraft` also replaces
  (`leaflet:src/replicache/mutations.ts:1228`). So a publish with explicit
  preferences **overwrites inherited state — that is the point** — and the
  pre-publish draft row is irrelevant to output whenever the author mapping is
  present. A draft-row value that does not decode within the supported subset
  stays a blocker only in the inherit case (it would flow into the record
  unreadable); with explicit intent it is simply replaced.

To keep dispatch deterministic when the author sets *any* preference key,
Leafbud resolves the full chain pre-dispatch and sends **both** the
three-key `postPreferences` and the boolean `showInDiscover`, so no
draft/record value can change the outcome between preview and dispatch
re-check. This mirrors how the native UI always sends `showInDiscover`
(`leaflet:PublishPost.tsx:316-317`).

### 2.4 CLI: `edit` flags; publish stays flag-free

```sh
leafbud edit post.mdx --preference showComments=false --preference showInDiscover=false
leafbud edit post.mdx --clear-preferences   # remove the mapping → inherit
```

- `--preference KEY=BOOL` is repeatable; KEY limited to the four, value
  strictly `true|false`. Flags **merge into** the existing mapping, matching
  the native Settings popover's one-key merge
  (`leaflet:components/PostSettings.tsx:51-56`); hand-editing the YAML owns
  the file's exact shape.
- `--clear-preferences` removes the whole mapping (absent = inherit). It does
  not write `{}`; authors wanting explicit "no overrides" write
  `preferences: {}` by hand (rare, and worth reading deliberately).
- Both join the existing `edit_fields` ArgGroup
  (`leafbud:crates/cli/src/main.rs:314`) and the offline
  pending-attempt guard (`leafbud:crates/document/mdx/src/author.rs:21-24`).
- **`publish` accepts no preference override flags.** Strong precedent:
  preview-first from source; flags edit source
  (`leafbud:design/native-publish0.gpt61s.md:46-50`). A flag on publish would
  create a second, less inspectable source of truth and an unpreviewable
  dispatch input.

### 2.5 Validation

- Typed struct on `Document` (like `theme`,
  `leafbud:crates/document/model/src/lib.rs:188-200`): four `Option<bool>`
  keys, `deny_unknown_fields`. Wrong types and unknown keys are errors, per
  the recognized-fields rule (`leafbud:design/frontmatter0.gpt61s.md:375-377`).
- The existing `supported_metadata` refusal of an extra `preferences` key
  (`leafbud:crates/leaflet/records/src/projection.rs:76-81`) moves from
  "refuse the concept" to "refuse only what the typed field cannot express";
  `cover`/`coverImage`/`delivery` stay refused.
- Release decode gains the supported subset: `$type` discriminator optional
  (native includes it in records), only the four known keys, boolean values,
  `showInDiscover` accepted only as `false` (true never appears in output;
  if it ever does, it is unknown native evidence — refuse). The current
  record-side refusal (test at
  `leafbud:crates/leaflet/records/src/projection.rs:602`) narrows to this
  subset. An **update whose existing record** carries preferences outside the
  subset still refuses — we will not blind-overwrite output we cannot read.

### 2.6 Preview disclosure

Preview gains a `preferences` object alongside `path`/`date`
(`leafbud:crates/workflow/src/publication.rs:27`), e.g.:

```json
"preferences": {
  "authored": {"showComments": false},
  "wire": {"postPreferences": {"showComments": false}, "showInDiscover": true},
  "record": {"showComments": false},
  "resolution": {"showInDiscover": "authored-true (dispatch-only; not recorded)",
                  "showComments": "authored", "showMentions": "inherited: draft {…}"},
  "replaces": {"draftRow": {"showRecommends": true}, "record": null}
}
```

- Inherit case: report the observed draft row and record values and the
  predicted inherited output; block only if that predicted output (or the
  observed sources it would flow from) falls outside the supported subset.
  The current shape-based blocker — which today rejects even an explicit
  `showComments: true` default toggle, since any nonempty object blocks
  (`leafbud:crates/workflow/src/publication.rs:325-328`) — becomes a
  meaning-based check.
- Explicit case: disclose that publish replaces the draft row and record
  values, and the dispatch-only nature of `showInDiscover: true`.

### 2.7 Saves vs publish; settlement

- **Save (`push`)**: when the author mapping differs from the observed draft
  row, include it in `UpdatePublicationDraft` — extend
  `PublicationMetadata` (`leafbud:crates/leaflet/protocol/src/push.rs:75-82`)
  with a full-replace `preferences` value, mirroring native replace semantics.
  The change-diff needs the **observed** row, not the local file: pass it into
  the builder the way `persisted_title` is passed today
  (`leafbud:crates/leaflet/editor/src/write/metadata.rs:5`). Write the full
  four-key authored mapping (including an authored `showInDiscover`) so the
  native UI shows the same state Leafbud will send.
- **Publish**: `arguments()` (`leafbud:crates/workflow/src/native/publication.rs:471-487`)
  adds `postPreferences`/`showInDiscover` when authored; the dispatch
  fingerprint hashes them with the rest of the args. Publication-container
  note: the draft-row upsert also rewrites `title/description/tags` —
  unchanged behavior.
- **Settlement**: predicted `preferences` joins the compared public meaning;
  the fetched record decodes through the extended subset decoder; the
  linkage-row `preferences` is compared against prediction (the same witness
  the preview guard already reads,
  `leafbud:crates/workflow/src/publication.rs:324`). `released` records only
  what it records today; no new receipt field is needed — preferences are part
  of the existing public-core hash.

### 2.8 Stay-refused list (with reasons)

1. **Email to subscribers** — `sendEmail` forced false; enabling requires
   observable newsletter-delivery verification, which is the uncommissioned
   `leafbud-release-delivery` dependency
   (`leafbud:design/native-publish0.gpt61s.md:176-178`). Also a first-release
   send-slot claim is a real native effect flags cannot suppress
   (`leafbud:design/native-publication0.gpt61s.md:203-207`).
2. **Share on Bluesky** — separate action, own transport/composer/char limit;
   `bskyPostRef` would additionally need record decode. Not part of the
   publish action, so not a publish option for Leafbud.
3. **Post Quietly** — no native arg of its own; for Leafbud it decomposes into
   already-forced-off email/share plus `preferences.showInDiscover: false`,
   which this design covers. Do not build a second knob.
4. **Scheduling** — different server action (`saveScheduledPost`), Pro-gated,
   stores a full bsky payload, needs a future-dispatch daemon Leaflet-side;
   also every publish deletes schedules, so scheduling and our publish path
   interact destructively. Authored future dates keep today's disclosed clamp.
5. **Cover image** — requires image upload (blob identity), crop metadata and
   release-side byte identity we deliberately do not fake
   (`leafbud:doc/release/README.md:151-154`).
6. **Email-only posts / test emails** — separate Pro-gated surfaces with no
   record output; out of the publish action entirely.
7. **Reply submission** — nonempty `reply_drafts` stay blocked; a reply
   capability is its own design.
8. **Publication-level preferences** (`showPrevNext`, `showFirstLast`,
   `prevNextDirection`, `showOtherPublicationsInTags`) — live on the
   publication record, which Leafbud does not manage.
9. **`localPublishedAt`** — pinned source shows it is ephemeral client state
   only (a Replicache key nothing reads back; §1.2). Leafbud's durable
   `publishedAt` intent is strictly stronger. Nothing to expose.

## Part 3 — Implementation map (not implemented; waits for the concurrent Sol)

The in-flight work owns `crates/workflow/src/native/`, publication paths, CLI
publish and version manifests. Everything below touches those; **do not start
until that lands and the parent reviews it.**

| Piece | Files | Notes |
| --- | --- | --- |
| Typed field + validation | `crates/document/model/src/lib.rs`, parser/serializer in the mdx crate | four `Option<bool>`, deny-unknown |
| `edit` flags | `crates/document/mdx/src/author.rs` (`MetadataEdit`), `crates/cli/src/main.rs:313-331` | merge semantics, `--clear-preferences` |
| Record encode/decode subset | `crates/leaflet/records/src/projection.rs` | narrow §2.5 refusals; resolution fn shared with preview |
| Save wiring | `crates/leaflet/protocol/src/push.rs:75-82`, `crates/leaflet/editor/src/write/metadata.rs`, compiler observation input | full-replace preferences, observed-row diff |
| Wire args | `crates/leaflet/web/src/publication.rs:12-36` | add `postPreferences`/`showInDiscover`; keep no-escape-hatch doc comment honest |
| Preview disclosure + guard change | `crates/workflow/src/publication.rs:287-332` | meaning-based blocker, `preferences` preview object |
| Dispatch + settlement | `crates/workflow/src/native/publication.rs` (`arguments()` :471-487, fingerprint, verify) | compare predicted record/row |
| Docs | `README.md` edit/publish sections, `doc/release/README.md` public-meaning profile | |

**Recommended slicing:** one coherent batch — the full preferences loop
(field → save → publish args → verify/settle → CLI). Half of it alone leaves
unverifiable state: sending args without the decoder extension makes
settlement refuse our own output, and the decoder without the field changes
nothing user-visible. If the batch must shrink, the first sliver is decoder +
preview disclosure of inherited supported-shape preferences (honest blocker
relaxation), with explicit send as the immediate second sliver.

Fixtures to cover: the full resolution matrix (each key absent/true/false ×
`showInDiscover` absent/true/false × inherit sources draft/record/none),
dispatch-only `true`, replace-not-merge of the draft row, update-with-
unreadable-record refusal, and CLI merge/clear round-trips.

## Connections

- [native publication contract](native-publication0.gpt61s.md) owns the
  action/effect surface this extends; [native publish](native-publish0.gpt61s.md)
  owns dispatch/settlement this plugs into; [frontmatter](frontmatter0.gpt61s.md)
  reserves `delivery` and sets the tri-state precedent; the maintained
  boundary summary is [release README](../doc/release/README.md).
- Tension to keep visible: this design **narrows** the 0.22–0.25 refusal
  posture ("nondefault preferences have no supported release mapping"). That
  posture was correct when preferences were unsendable and unreadable; this
  design makes them both, and the refusal retreats to the genuinely
  unsupported (unknown keys, non-boolean values, `showInDiscover:true` in
  observed output). Parent review should confirm that reading.
