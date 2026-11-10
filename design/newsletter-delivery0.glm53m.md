---
type: Design
title: "Native newsletter and share delivery: research and command-surface design"
description: "Read-only trace of Leaflet 600c7b5f email/share machinery, the observable-versus-assumed delivery evidence map, and a frontmatter-intent design for leafbud-release-delivery. Research only; no implementation commissioned."
resource: /design/newsletter-delivery0.glm53m.md
status: draft
generated:
  at: 2026-11-10
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e3917df05ffexDi7k5XmXAi7Lj"
    title: "Read-only archive research and design draft: traced send-slot claim, firehose broadcast interlock, per-subscriber event log, share flavors and RPC observability; wrote this document"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "First-publish send-slot claim with skipped/pending statuses and the inngest dispatch; sendEmail defaults true; email-only draft refusal"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/inngest/functions/sync_document_metadata.ts
    title: "Firehose-driven broadcast claim on create events; already-claimed slots skip"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/inngest/functions/send_post_broadcast.ts
    title: "Broadcast job lifecycle: sending/sent/failed row transitions and subscriber snapshot"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/emailPosts/broadcast.ts
    title: "loadUnsentSubscribers event-log dedupe, Postmark batch acceptance semantics, per-recipient send_failed invisibility in terminal status"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/migrations/20260421000000_add_newsletter_tables.sql
    title: "Newsletter schema: publication_post_sends composite PK, subscriber states, append-only event log with post_sent/bounce/complaint"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/migrations/20260630000001_post_sends_skipped_status.sql
    title: "The skipped status and its quiet-publish pre-claim purpose, in the migration's own words"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_leaflet_data.ts
    title: "Token-scoped RPC selects newsletter enabled but no send-slot or subscriber-event state"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_publication_data.ts
    title: "Owner/contributor-gated dashboard RPC: the only send-state read surface (publication_post_sends status, subscriber_count)"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/lish/[did]/[publication]/dashboard/PublishedPostsLists.tsx
    title: "PostSendStatus display: SENT/SENDING/FAILED with contact-support copy; no resend control; skipped renders nothing"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/[leaflet_id]/publish/PublishPost.tsx
    title: "Share-step defaults all-on, quiet forces channels off, client-side post-publish Bluesky share whose non-OAuth failure is reported as a publish failure"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/[leaflet_id]/publish/ShareOptions.tsx
    title: "ShareState toggles and the EmailPreview test-address action"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publications/sendPostPreview.tsx
    title: "Test email: single direct Postmark send from draft pages, [preview] subject, no slot claim, no event log"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishBskyPost.ts
    title: "Two share flavors: rkey re-puts the document record with bskyPostRef; documentUri share creates only an app.bsky.feed.post card"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/appview/index.ts
    title: "Managed publications fire sync-document-metadata on the firehose, enabling the racing broadcast claim"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/inngest/functions/publish_scheduled_post.ts
    title: "Scheduled publish persists send_email and shares on Bluesky server-side; separate Pro surface"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/emailPosts/eligibility.ts
    title: "Email-only posts are Pro-gated; separate delivery-only surface"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/postmark/webhook/route.ts
    title: "Async bounce/complaint events flip subscribers to unsubscribed; not exposed to any RPC"
  - resource: /crates/leaflet/web/src/publication.rs
    title: "Leafbud transport hard-codes sendEmail:false and documents the deliberate absence of a delivery escape hatch"
  - resource: /design/native-publish0.gpt61s.md
    title: "Live-proven quiet publish/update/delete and the forced-quiet low-level argument"
  - resource: /design/release-preferences0.gpt61s.md
    title: "Tri-state frontmatter precedent, whole-object selection lessons, and the retained delivery boundary"
---

# Newsletter and share delivery — research and design

## Plain-language summary

**What delivery actually is in Leaflet.** Publishing a post and emailing it are
two separate things joined by one small database row. The publish action writes
the public record first; then — only on a *first* publish into a
newsletter-enabled publication — it "claims the send slot" by inserting one row
into `publication_post_sends`. If you asked for email, the slot is claimed
`pending` and a background job emails every confirmed subscriber in Postmark
batches, ending at `sent` or `failed`. If you asked for quiet, the slot is
claimed `skipped`, and the reason that row still gets written is defensive: a
completely separate firehose-driven path would otherwise see the new public
record, find no slot row, claim it `pending` itself, and email your subscribers
anyway. Quiet works by getting there first. That also means quiet is only
durable once the publish flow reaches that final claim step: a crash between
the record write and the claim can let the firehose path auto-send a post the
author meant to be quiet — a native hazard that applies to Leafbud's forced
quiet today.

**What is verifiable, honestly.** Leafbud can read whether the publication has
newsletters on (existing token RPC). If the operator's cookie identity is the
publication owner or a confirmed contributor, a second existing RPC
(`get_publication_data`) exposes each document's slot status
(`pending/sending/sent/failed/skipped`) and a subscriber count — that is the
*only* delivery state any RPC returns. Per-subscriber receipts exist in an
append-only event log the server uses to avoid double-emailing on retries, but
no RPC exposes them; inngest job state is invisible; and even `sent` only means
Postmark *accepted* each batch message — individual recipients can have quietly
failed, and bounces surface days later through a webhook Leafbud cannot read.
So the honest ceiling for a delivery receipt is: slot status + subscriber
count + "accepted by Postmark" semantics, never per-subscriber proof.

**Resend does not exist.** Native retry is internal to the background job
(step retries with per-subscriber dedupe). Once a slot is `skipped`, `sent`, or
terminally `failed`, no surface Leafbud can invoke re-fires it — the web UI's
failed display literally says "contact us", and re-publishing is an update
that never touches the slot. Any Leafbud "resend" would be a lie; the design
refuses to offer one.

**Recommended surface.** Delivery intent stays authorable frontmatter
(`delivery.email: true|false`, missing = today's quiet default), carried as an
*always-explicit* `sendEmail` boolean on the wire (native default is `true`,
so omission is dangerous). Preview refuses intent that cannot fire (updates,
disabled newsletters). Delivery receipts are observed after settlement through
the owner-gated RPC and recorded separately from publication receipts. Test
email and post-hoc Bluesky sharing are separate bounded commands for later
slices; publish-time Bluesky sharing mutates the released record and stays out
of scope. No automatic delivery ever enters push/save/ordinary publish.

## The native delivery machinery (traced)

### Publish-path send-slot claim

`publishLeaflet` accepts `sendEmail?: boolean`
(src/utils/publishLeaflet.ts:91) and destructures it with a default of
**true** (publishLeaflet.ts:176) — an omitted argument means *send*. The web
UI always passes it explicitly: the share step defaults all channels on
(`{ bluesky: true, postToReaders: true, email: true, quiet: false }`,
app/(app)/(identity)/[leaflet_id]/publish/PublishPost.tsx:153) and sends
`sendEmail: shareState.email && !shareState.quiet` (PublishPost.tsx:316);
checking "Post Quietly" unchecks every channel
(app/(app)/(identity)/[leaflet_id]/publish/ShareOptions.tsx:85-96).

After the record is written and linked (putRecord publishLeaflet.ts:481-487,
document/join upserts :493-530), schedule cancellation (:554-558), version cut
(:571-576), image GC (:581), reply submission (:589-613) and mention
notifications (:616-622), a first publish into a publication claims the slot
(`if (publication_uri && !existingDocUri)`, publishLeaflet.ts:630): it reads
`publication_newsletter_settings.enabled` (:631-635), then upserts
`publication_post_sends` with `status: sendEmail ? "pending" : "skipped"` under
`onConflict: "publication,document", ignoreDuplicates: true` (:636-647). The
composite PK is the real idempotency guard; the code's own comment states the
skipped claim is "also what stops the firehose-driven broadcast in
sync_document_metadata from emailing a post the author opted out of: it finds
the row already present and skips" (publishLeaflet.ts:624-629). Only when the
insert actually happened does it dispatch `newsletter/post.send.requested` to
inngest (:648-657). Email-only drafts are refused outright before any of this
("This draft is an email, it can't be published as a post",
publishLeaflet.ts:205-212). Updates never reach the claim at all.

### The racing firehose broadcast

The appview indexes the firehose and, for documents in Leaflet-managed
publications (`managed = inPublication && isLeafletPublication(site)`,
appview/index.ts:871-873), emits `appview/sync-document-metadata` on create or
update (appview/index.ts:893-901) — this fires for records Leafbud publishes,
because the appview cannot tell who published. The `sync_document_metadata`
job debounces 60 s per document
(app/api/inngest/functions/sync_document_metadata.ts:17-21) and **only on
`event_type === "create"`** (:154) checks the same newsletter settings
(:166-173) and attempts the same `ignoreDuplicates` upsert with status
`pending` (:175-185); a row already present yields
`skipped: "already_sent_or_pending"` (:186-188). Only a fresh claim sends the
broadcast event (:193-201). Two consequences:

- Exactly one of the two paths ever broadcasts per (publication, document):
  whichever claims first. Normally the publish path wins by minutes (the
  firehose path also waits out its debounce).
- **A publish that crashes between putRecord and its slot claim leaves the
  slot free**, so the firehose path claims `pending` and broadcasts — even
  when the author chose quiet. Quiet intent is not durable server-side until
  the claim lands; the claim is among the publish flow's last steps. This
  hazard exists for Leafbud's forced `sendEmail:false` today.

### The broadcast job

`send_post_broadcast` (app/api/inngest/functions/send_post_broadcast.ts)
loads publication and document (:42-59), resolves a sender — failing with
`newsletter_not_enabled` or `no_from_address` marks the row `failed`
(:61-75) — then snapshots recipients via `loadUnsentSubscribers`
(:110-126), flipping the row to `sending` with a `subscriber_count`. Zero
recipients finalize directly to `sent` with count 0 (:128-140). Sending runs
in Postmark `/email/batch` calls capped at 500 messages / 40 MB
(src/emailPosts/broadcast.ts:34-37); each batch response maps per-recipient
`ErrorCode === 0` to success (broadcast.ts:494-531). For every recipient a
`post_sent` or `send_failed` event row is inserted through the
`log_subscriber_events` RPC — insert failure throws so inngest retries the
atomic insert, because "a lost insert would re-email the whole batch on a
resend" (broadcast.ts:534-559). Transport failures throw and exhaust into
`onFailure`, which marks the row `failed` with the error
(send_post_broadcast.ts:21-34); successful completion finalizes `sent` with
`completed_at` (:158-167).

### Subscriber selection

`loadUnsentSubscribers` selects the publication's `confirmed` email
subscribers (broadcast.ts:194-201), then removes anyone with a `post_sent`
event whose `metadata->>document` equals this document (:205-218) — "a
retried or resumed send must not email them twice" (broadcast.ts:186-189).
Member-gating splits recipients into full-body and truncated-preview groups
(broadcast.ts:258-407); entitlement comes from active memberships plus owner
and confirmed contributors (:319-371). Delivery to **atproto** subscribers is
not email at all — the share step's Bluesky checkbox notes "Subscribers will
be updated via a custom Bluesky feed" (ShareOptions.tsx:138-140).

### Resend and retry semantics

- **Retry** is inngest-internal: step retries re-run the snapshot, and the
  event log excludes already-sent subscribers (broadcast.ts:205-218). It is
  not user-triggerable.
- **Resend does not exist.** Production code touches `publication_post_sends`
  in exactly four places (publishLeaflet, sync_document_metadata,
  send_post_broadcast, get_publication_data); nothing ever resets a slot or
  re-fires an event for an existing row. Updates never re-claim
  (publishLeaflet.ts:630). The dashboard's `PostSendStatus` renders
  SENT/SENDING/FAILED — failed says "Contact us at contact@leaflet.pub" —
  and `skipped` renders nothing
  (app/(app)/(identity)/lish/[did]/[publication]/dashboard/PublishedPostsLists.tsx:235-277).
  A re-fired event would only reach never-sent subscribers anyway
  (broadcast.ts:205-218); there is no "send again to everyone" primitive.

### Test email

The share step's "Send preview" (ShareOptions.tsx:195-290) calls the
`sendPostPreview` server action: owner/confirmed-contributor only
(actions/publications/sendPostPreview.tsx:48-61), renders from the **draft**
pages (:67), and sends one direct Postmark `/email` message with
`MessageStream: "outbound"` and a `[preview]` subject prefix (:123-137,
:115). It claims no slot, logs no subscriber events, and reports only
Postmark HTTP acceptance (:138-142).

### Scheduled and email-only surfaces (adjacent, not ours)

Scheduled posts persist `send_email` and pass it to `publishLeaflet` at fire
time (src/scheduledPosts/publish.ts:47), with the Bluesky share executed
server-side in the scheduled job
(app/api/inngest/functions/publish_scheduled_post.ts:105-110). Email-only
posts are a separate Pro-gated surface with its own
`publication_email_posts` lifecycle and revision-guarded sends
(src/emailPosts/eligibility.ts:19-33;
app/api/inngest/functions/send_email_post.ts:12-92). Leafbud neither
schedules nor authors email-only posts; both stay out of scope.

### The Bluesky share surface

`postDocumentToBsky` (src/utils/publishBskyPost.ts:104-204) has two flavors:

- **Publish-time (`rkey` given):** after the post exists, it creates the
  `app.bsky.feed.post` card (:160-174), then **re-puts the whole document
  record** with `bskyPostRef` embedded and updates `documents.data`
  (:176-201). In the web UI this runs client-side after publish returns; only
  OAuth failures are surfaced, and a thrown non-OAuth error produces the
  generic "Something went wrong while publishing" message even though the
  record is already published (PublishPost.tsx:343-363, 319-326).
- **Post-hoc (`documentUri` given, no rkey):** creates only the
  external-card post; the document record is untouched
  (publishBskyPost.ts:120-153). The dashboard share button uses this flavor
  (components/Interactions/InteractionShareButton.tsx:88-160). No linkage
  table records it; attribution is analytics-only via UTM parameters
  (app/api/rpc/[command]/get_publication_bsky_posts.ts:21-25, gated behind
  Pro entitlements :29-39).

Both flavors restore the sharer's OAuth session (publishBskyPost.ts:38-64);
the only typed failure is `OAuthSessionError` (:30-32); everything else
throws.

## Observable versus assumed outcomes

| Question | Evidence | Verifiable by Leafbud? |
| --- | --- | --- |
| Is the publication newsletter-enabled? | `publication_newsletter_settings(enabled)` selected by the token RPC in both publication joins and the draft-publication join (app/api/rpc/[command]/get_leaflet_data.ts:10, :14) | **Yes**, read-only, today |
| Was the send slot claimed, and what state is it in? | `publication_post_sends(status, subscriber_count)` per document — returned only by `get_publication_data`, gated to the owner or a confirmed contributor web identity (get_publication_data.ts:57, :95-113, :136) | **Yes**, via a new owner-cookie RPC call; not via the token RPC |
| How many subscribers would receive it? | Full `publication_email_subscribers` rows in `get_publication_data` (get_publication_data.ts:60) | Yes, but the RPC returns **emails too**; Leafbud must count and not persist rows |
| Did each subscriber receive it? | `post_sent`/`send_failed` rows in the append-only event log (migrations/20260421000000_add_newsletter_tables.sql:123-128, :171-173) | **No — no RPC selects `publication_email_subscriber_events`** |
| Is the batch job still running / retried? | inngest state; only row `status` reflects it | Only through the slot-status proxy |
| Did Postmark actually deliver? | `sent` means batch acceptance (`ErrorCode === 0`, broadcast.ts:521-530); bounces/complaints arrive asynchronously via webhook and flip subscribers to unsubscribed (app/api/postmark/webhook/route.ts:17-22) | **No**; acceptance ≠ delivery, partial per-recipient failure does not change terminal status (broadcast.ts:561-565) |
| Which path claimed a `pending` slot? | Publish path and firehose path write indistinguishable rows | **No**; practically irrelevant, but honest receipts should not attribute |
| Was a Bluesky card post created? | Returned post URI; record `bskyPostRef` only for the rkey flavor | rkey flavor: yes, via record/documents read; post-hoc flavor: only by reading the returned URI's PDS record |

Slot statuses are exactly `pending/sending/sent/failed/skipped`
(migrations/20260421000000_add_newsletter_tables.sql:212-214;
migrations/20260630000001_post_sends_skipped_status.sql adds `skipped`).
The publish action's return value (`PublishResult`,
publishLeaflet.ts:62-74) carries **no** delivery information — a decoded
success proves only the record. However, ordering gives one derivable fact:
the slot claim and inngest dispatch precede the return
(publishLeaflet.ts:630-659 before :666-674), so a **decoded success** on a
first publish to a newsletter-enabled publication implies the slot was
claimed with exactly the intent sent (`pending` + event for email,
`skipped` for quiet). A lost/undecoded response implies nothing about the
slot — that is the observation case below.

## First delivery, retry, resend — the design

### Intent surface (frontmatter, tri-state)

Following the preferences precedent
(design/release-preferences0.gpt61s.md:61-88) and the operator's frontmatter
preference, delivery intent is authorable source, never a publish flag:

```yaml
delivery:
  email: true   # or false; key omitted / block missing / null => quiet default
```

- **Missing/null** (block or key): today's behavior — `sendEmail:false` on
  the wire, no delivery promise, not hashed as intent. This preserves the
  quiet default boundary verbatim.
- **`delivery.email: false`**: same wire effect, but explicit and hashed
  into the dispatch fingerprint; preview reports "explicit quiet: first
  publish will pre-claim the send slot as skipped" so the active side effect
  of quiet is disclosed, not hidden.
- **`delivery.email: true`**: publish sends `sendEmail:true`. Preview gates
  it against the three native preconditions and **refuses when the intent
  cannot fire**: an update publish (updates never claim,
  publishLeaflet.ts:630) or a newsletter-disabled publication (the claim is
  skipped entirely, publishLeaflet.ts:636). Refusal follows the
  preferences ticket's source-controlled-outcome rule
  (release-preferences0.gpt61s.md:108-114): a delivery promise the native
  surface will silently drop is not allowed to look successful. A standalone
  (non-publication) target with email:true also refuses — standalone
  publishes have no slot path at all (publishLeaflet.ts:630;
  sync_document_metadata.ts:164).

Unknown keys, non-boolean values, per-key null, and an explicit empty
`delivery: {}` all refuse — there is no meaningful "explicitly set nothing"
state distinct from missing, unlike preferences' clear.

### Transport rule: the boolean is always explicit

The native default is `sendEmail = true` (publishLeaflet.ts:176), so
omission is delivery. The adapter's `Arguments` gains a required
`send_email: bool` (replacing the hard-coded `false` in
crates/leaflet/web/src/publication.rs:26-42) and the wire body carries
`sendEmail` on **every** invocation — `true` only when gated intent exists,
`false` otherwise. Never omitted, never optional.

### Command surface

- `leafbud publish FILE` — unchanged shape; honors `delivery.email` as
  above. Publish stays the *only* first-delivery trigger, because natively
  it is the only place a slot can be claimed.
- **No `deliver`, no `--resend`, no retry command.** Native broadcast retry
  is job-internal; terminal `failed` has no re-fire surface; `skipped` can
  never send later (the composite-PK row exists forever,
  migrations/20260421000000_add_newsletter_tables.sql:191-192). A Leafbud
  command pretending otherwise would be the design's biggest honesty risk;
  a failed send's honest report is "failed natively; no supported resend;
  the slot is terminal", with human recourse (a follow-up post, or the
  operator contacting Leaflet) named as such.
- `leafbud send-test FILE --to <address>` (later slice, optional): the
  bounded test-email action — exactly one message to one address, rendered
  from the draft, `[preview]` subject, no slot claim, no subscriber events
  (sendPostPreview.tsx:30-41, :123-142). Preview discloses all of that;
  `--yes` executes. Its result is Postmark acceptance only. Gated on the
  action-reference discovery proof named below.
- `leafbud share FILE --text "…"` (later slice, optional): the post-hoc
  `documentUri` flavor only — creates one `app.bsky.feed.post` card,
  mutates nothing released (publishBskyPost.ts:120-153). Verification is
  the returned URI's PDS record. The publish-time `rkey` flavor is
  explicitly out of scope: it re-puts the released record with `bskyPostRef`
  (publishBskyPost.ts:176-201), changing the verified public core after
  settlement — supporting it would require its own record-verification and
  settlement extension and must not ride along with delivery.

### Receipts: what a delivery receipt can honestly record

A **delivery receipt** is stored separately from the released publication
receipt and never gates record settlement:

- observed slot `status` and `subscriber_count` for the document, with
  observation time and the RPC used;
- semantics named as **Postmark batch acceptance at finalize time**, with the
  recorded caveat that partial per-recipient failure is possible and
  invisible (broadcast.ts:561-565) and delivery/bounce confirmation does not
  exist in any readable surface (postmark webhook route.ts:17-22);
- no per-subscriber claims, ever.

Receipt observation is best-effort and owner-gated: it requires the
publication-owner (or confirmed-contributor) cookie identity the native
publication flow already uses (crates/leaflet/web/src/publication.rs:69-89)
and a new bounded `get_publication_data` client that selects only the
newsletter flag, per-document slot fields and a subscriber count — it must
**not persist subscriber rows or emails** the RPC returns
(get_publication_data.ts:60).

### Uncertainty and recovery

The publish continuation already records dispatched uncertainty
(design/native-publish0.gpt61s.md:76-82). Delivery adds two observation
rules to the existing observation-only recovery path:

1. **Uncertain first publish, any email intent:** observe the slot via
   `get_publication_data`. `pending`/`sending` → delivery in flight (do
   nothing; report). `sent`/`failed` → record the delivery receipt. `skipped`
   → quiet claim landed. **Row absent** → the claim may not have run yet;
   re-observe before concluding, because the firehose path may claim `pending`
   up to minutes later (its 60 s debounce,
   sync_document_metadata.ts:17-21). Never re-invoke publish to "complete"
   delivery — a repeat is an update and cannot touch the slot
   (publishLeaflet.ts:630).
2. **Quiet-intent unintended send:** if a quiet (or default-quiet) publish
   was uncertain and the slot later reads `pending`/`sending`/`sent`, the
   crash-window hazard fired (record written, claim lost, firehose claimed).
   Report it explicitly as an unintended delivery with its cause; there is
   no supported un-send. This is disclosed in preview for first publishes to
   newsletter-enabled publications: "quiet protection depends on the native
   flow completing its slot claim; a crash in that window can let the
   firehose path email subscribers."

Uncertain acknowledgments never cause blind resend — structurally impossible
here anyway, and the design keeps it that way by having no resend primitive
at all.

## Boundaries that must survive

1. **No automatic delivery, ever, from push/save/ordinary publish.** Push
   remains editor-only; save never reaches the publish action; publish
   without explicit `delivery.email:true` sends `sendEmail:false`.
2. **Quiet default unchanged.** Missing intent = `sendEmail:false`, exactly
   today's forced value (crates/leaflet/web/src/publication.rs:28); the only
   change is that explicit false becomes hashed, disclosed intent.
3. **No blind resend on uncertain.** Recovery observes; it never re-fires.
4. **Delivery receipts are separate from released publication receipts** and
   from record settlement; a delivery cannot block or unblock release.
5. **Test email, if built, is its own bounded action** — single recipient,
   draft-rendered, no slot, no events — and never a mode of publish.
6. **Publish-time Bluesky share stays out of scope** until record mutation
   after settlement has its own verification design.
7. **The email-only and scheduled surfaces are not adopted.** Leafbud
   authors documents; their distinct lifecycles
   (send_email_post.ts:12-92; publish_scheduled_post.ts:24-75) are recorded
   here as context only.

## Implementation map and slicing

All changes are in existing crates; no migration framework, no upstream
change assumed.

**Slice 1 — email intent through publish** (model + workflow + transport):

- `crates/document/model`: parse/validate the `delivery:` block (tri-state
  email key; strict shapes mirroring the preferences parser).
- `crates/workflow/src/publication.rs`: preview gating (update/disabled/
  standalone refusals with observed evidence), intent disclosure, fingerprint
  inclusion.
- `crates/leaflet/web/src/publication.rs`: required `send_email: bool`;
  always-explicit wire boolean.
- Fixture tests: literal wire forms for both intents and default; refusal
  matrices; fingerprint sensitivity. Loopback HTTP tests alongside the
  existing publication matrix.

**Slice 2 — delivery observation and receipts**:

- `crates/leaflet/web`: bounded `get_publication_data` client (newsletter
  flag, slot status/count per document, subscriber count; no persisted
  subscriber rows).
- `crates/workflow`: post-settlement slot observation; separate delivery
  receipt store; the two recovery rules above wired into the existing
  continuation recovery; status reporting of slot state per released
  document.
- Tests: RPC shape fixtures (including the owner-gating failure shape),
  absent-row re-observation, unintended-send detection, receipt/recording
  boundaries.

**Slice 3 — test email** (optional, separately commissioned): action
reference discovery for `sendPostPreview` on the publish page (unproven —
see gaps), bounded invocation, `[preview]` disclosures.

**Slice 4 — post-hoc Bluesky share** (optional, separately commissioned):
`publishPostToBsky` documentUri flavor, share-text authoring, PDS
verification of the created post.

### Evidence gaps that shape the design

- **No RPC exposes per-subscriber events or inngest state** — verified by
  reading every RPC in `app/api/rpc/[command]/` (route.ts:44-70); receipts
  stop at slot status by necessity, not by choice.
- **`get_publication_data` is identity-gated** (get_publication_data.ts:95-113):
  delivery observation only works when the configured cookie identity owns
  the publication or is a confirmed contributor; for any other identity the
  slot is unobservable and receipts must say "unknown (not owner-gated)".
- **`sendPostPreview` action-reference discovery is unproven** — the action
  is imported by a client component (ShareOptions.tsx:10), so its Next
  action ID may not appear in the publish page HTML the way
  `publishToPublication` does (crates/leaflet/web/src/publication.rs:80).
  Slice 3 starts with a read-only discovery proof.
- **Delivery acceptance is not delivery** — no readable bounce surface; the
  receipt vocabulary must say "accepted", never "delivered".

## Surprises worth naming

1. **Quiet is an active write, not an omission** — the `skipped` row is the
   only thing standing between a quiet publish and the firehose path
   emailing everyone (publishLeaflet.ts:624-629;
   20260630000001_post_sends_skipped_status.sql:1-4). Leafbud's current
   forced quiet has been silently performing this protective write all
   along.
2. **The crash-window auto-send hazard** — a quiet publish that dies between
   record write and slot claim can trigger a full subscriber broadcast from
   the firehose path. Native, pre-existing, and applicable to Leafbud today.
3. **The publish UI can misreport publish failure when only the Bluesky
   share threw** (PublishPost.tsx:343-363) — a native honesty gap worth
   remembering whenever we consider the rkey share flavor.
4. **Native `sendEmail` defaults to true** — the single most dangerous
   omission hazard in the whole surface; our design's always-explicit wire
   rule exists because of it.
5. **`sent` with zero subscribers is a real terminal state**
   (send_post_broadcast.ts:128-140) — a delivery receipt for a
   newsletter-enabled publication with no subscribers is still a receipt.

## Standing

Research and design only, commissioned by the parent session. No code was
written, no live request made, no `~/l`, P1, standalone verification draft or
any existing object touched; the pinned archive was verified clean at
`600c7b5f1f7b31b66fb626c293cc104312a12caf` before reading. Implementation of
any slice requires this design's acceptance plus a separate explicit
commission; until then the adapter keeps forcing `sendEmail:false`
(crates/leaflet/web/src/publication.rs:26-42). Ticket disposition stays with
the parent.
