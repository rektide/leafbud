---
type: Design
title: "The native gated-image cleanup question, explained for decision"
description: "Plain-language account of the one remaining policy blocker for native publication-container publishing: Leaflet's own post-publish storage cleanup, its user impact, and the operator's options."
resource: /design/native-cleanup-question0.glm53m.md
status: draft
generated:
  at: 2026-11-10
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e39ce3c43ffeVNFymyP82EAVDN"
    title: "Operator explainer for the native gated-image cleanup decision; all behavioral claims re-verified against the clean pinned archive and Leafbud preview sources"
  - resource: /design/native-publication0.gpt61s.md
    title: "Parent-accepted 0.22 preflight checkpoint recording the gated-cleanup blocker and its publication-container-only scope"
  - resource: /design/publication-verification0.gpt61s.md
    title: "Parent-accepted 0.23 verification checkpoint; parent integration review owns this policy question"
  - resource: /crates/workflow/src/publication.rs
    title: "Preview adds the gated-cleanup blocker only when a publication container is present"
  - resource: /doc/mdx/coverage.md
    title: "members-only-delimiter listed in the unsupported editor inventory"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/gatedDocumentImages.ts
    title: "Document-scoped gated-image copies and stale-copy deletion"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "Gated store constructed only with publication_uri; removeStale after every successful publication publish"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts
    title: "Public action argument surface; no cleanup-suppression flag"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts
    title: "Members-only images bypass the PDS and are copied into gated storage"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_leaflet_data.ts
    title: "The RPC available to Leafbud reads Postgres rows only, not Supabase storage"
---

# The native gated-image cleanup question

## TL;DR

When Leaflet publishes a post **into a publication** (a container like a blog
with its own identity, as opposed to a standalone personal post), it always —
as part of that action, with no way to switch it off — tidies a per-document
folder of "members-only" image copies, deleting copies the newest publish no
longer uses. This is Leaflet's own housekeeping; the identical thing happens
when the author republishes from Leaflet's own web UI. Leafbud flagged it as a
blocker because we refuse destructive side effects we cannot inspect or prove
harmless beforehand, and the folder's contents cannot be read through any
interface Leafbud has. For Leafbud users the practical exposure is narrow:
Leafbud cannot author members-only content at all, so the only documents
affected are ones whose author previously used that feature through the web
UI. The operator decision is whether to accept this disclosed native behavior
(option A), demand an upstream switch (B), defer by shipping standalone-only
(C), or something in between (D, analyzed below and found mostly illusory).
Doing nothing leaves publication-container publishing refused and nothing else
changes.

## Vocabulary used in this document

- **Standalone vs publication-container posts.** A Leaflet document can be
  published as a standalone post under the author's own identity, or into a
  *publication* — a container with its own identity where the record lives in
  the publication owner's repository. Publishing into a publication is the
  only context in question here.
- **Gated / members-only images.** Publications can mark part of a post as
  visible to members only. Images in that region are deliberately never
  uploaded to the PDS (the public record store) — that would make them public.
  Instead, at publish time Leaflet copies each such image into a private
  storage folder belonging to that one document
  ([publishHelpers.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts)
  lines 25–47).
- **Preflight / preview.** Leafbud's dry-run report before any real action. It
  lists the effects that would occur and *blockers* — stated reasons the real
  action is currently refused.
- **Witness.** Evidence we can actually read beforehand that proves a
  condition ("the folder is empty"). A witness turns "we believe this is
  harmless" into "we checked."

## The mechanism in plain words

Leafbud publishes natively by invoking the same server action the web UI's
publish button uses (`publishToPublication`; see
`app/(app)/(identity)/[leaflet_id]/publish/PublishPost.tsx` lines 2, 303 in
the archive). That action is a package deal: along with the actual publish it
performs several pieces of housekeeping that no argument can turn off.

The piece in question works like a shelf managed by the publisher. Each
document that publishes into a publication gets its own shelf — a folder named
`gated-documents/<owner-did>/<post-rkey>` in Leaflet's Supabase storage
([gatedDocumentImages.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/gatedDocumentImages.ts)
lines 4, 16–19). Every time this document publishes, Leaflet puts copies of
the current members-only images on the shelf (reusing the same filename each
time, so unchanged images are left in place rather than duplicated — lines
29–51, 83–90). Then, after the publish succeeds, it takes away everything on
the shelf this publish did not put there — `removeStale()` lists the folder
and deletes every copy not referenced by the newest publish, **including all
copies when the post no longer has any gated content** (lines 72–79). The
deletion is best-effort: failures are swallowed with `.catch(console.error)`
and the publish still reports success
([publishLeaflet.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts)
lines 578–581).

Two safety properties worth knowing, both verified in source:

- The deletion can only target objects the server successfully *listed*. If
  the folder listing fails, the code treats the folder as empty and deletes
  nothing — the failure direction is safe (`data ?? []`,
  gatedDocumentImages.ts lines 24–25).
- The listing fetches at most 1000 names per run (line 24), so a single
  publish's deletion sweep is bounded.

## When it fires and when it never fires

- **Never for standalone posts.** The gated-image store is constructed only
  when a `publication_uri` argument is present; standalone publishes never
  create it and the cleanup line is a no-op for them (publishLeaflet.ts lines
  281–283, 581). The 0.22 receipt's scope clarification says the same
  ([native-publication0](/design/native-publication0.gpt61s.md), "Verification
  follow-up — 0.23").
- **Always for publication-container publishes** — even when the post has no
  images at all. If `gatedImages` exists, `removeStale()` runs, period. There
  is no argument to suppress it: the action's full argument surface is the
  `PublishLeafletArgs` type minus `rkey` (publishLeaflet.ts lines 76–99;
  [publishToPublication.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts)
  lines 14–16, 28–39), and it contains no such flag.
- **Every publish path behaves the same way.** The web UI's publish button,
  scheduled posts (`src/scheduledPosts/publish.ts` lines 38–41) and the Ghost
  importer (`src/import/importPost.ts` lines 83–86) all call the same
  `publishLeaflet` with `publication_uri` set, so all of them run the same
  cleanup.

## Concrete user-impact scenarios

| # | Situation | What a Leafbud publish does | Real-world effect |
|---|---|---|---|
| 1 | Standalone document (no publication) | Nothing — no store, no cleanup (publishLeaflet.ts 281–283, 581) | None. |
| 2 | Publication post that never used gated images | Lists the document's folder and deletes nothing (absent/empty folder) | Effectively harmless — but we cannot *prove* emptiness beforehand (see below). |
| 3 | Publication post whose author once used gated images via the web UI and later removed them | Deletes the leftover old copies | Old image URLs (in cached pages, old links) stop loading. **Identical to republishing from Leaflet's own web UI**; bounded to this one document's folder. |
| 4 | Publication post that still uses gated images | Current copies are found in place and kept; only unreferenced ones go (gatedDocumentImages.ts 34, 72–79) | Live images unaffected; URLs stay stable across republishes. |

**What does not happen:** no deletion of the author's draft uploads in
general (only copies inside this document's own folder are candidates); no
cross-document effects (each document's folder is keyed by owner DID and post
rkey, gatedDocumentImages.ts 16–19); and this path never deletes PDS records —
it only removes storage objects. (A separate, already-disclosed native
behavior: if first-time record linking fails, the server tries a compensating
PDS delete, publishLeaflet.ts 532–542. That is a different mechanism, recorded
in the 0.22 receipt, not part of this question.)

## Why Leafbud flagged it

Leafbud's standing principles: no automatic prune or cleanup unless explicitly
requested, and no destructive side effect we cannot scope or pre-verify. This
cleanup fails the second test in a specific way: the folder contents live
behind a Supabase *storage* API, and the only native interface Leafbud has —
the `get_leaflet_data` RPC — is a Postgres query over tokens, drafts,
documents, schedules and reply drafts
([get_leaflet_data.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_leaflet_data.ts)
lines 21–36). It has no storage surface at all. So we cannot witness "the
folder is empty" before dispatching a publish.

The contrast with schedules shows this is not arbitrary. Publishing also
deletes any schedules for the post — but schedules *are* readable through the
RPC (line 32), so the preview refuses whenever observed schedules are
nonempty and accepts only when the deletion is provably a no-op. Gated
storage is the one disclosed native effect for which no such witness exists;
hence it is the one carrying a blocker
([publication.rs](/crates/workflow/src/publication.rs) lines 131–143, tested
in `crates/workflow/src/publication/tests.rs` line 92).

One more fact that shrinks the exposure: Leafbud's supported editor cannot
author members-only content at all — `members-only-delimiter` is in the
explicitly unsupported inventory
([coverage.md](/doc/mdx/coverage.md) lines 193–197). So a Leafbud-authored
document has no gated images, and the only way a folder can have contents is
history created earlier through Leaflet's own web UI.

## The options

| Option | Outcome | User impact | Work required | Dependency | Reversible? |
|---|---|---|---|---|---|
| **A. Permit as disclosed behavior** | Publication-container publishing unblocked; execution proceeds for both contexts | Publish from Leafbud behaves exactly like the web UI's publish, housekeeping included; scenario-3 deletions possible for web-UI history | Policy change + small code edit (drop the publication-scoped blocker, keep/expand the effect disclosure in preview) | None upstream | Yes — blocker can be restored; past deletions are not, but they equal native behavior |
| **B. Require an upstream opt-out** | Publication-container publishing stays refused; standalone proceeds | Publication users blocked indefinitely (must keep using the web UI — which itself runs the cleanup) | Upstream contribution (a suppression argument on the action, or a storage-listing RPC), coordination, re-pin, tests | Upstream accepts and ships; timeline not ours | Yes, but blocking persists meanwhile |
| **C. Standalone-first sequencing** | Standalone execution lands now; decision deferred with usage data | Standalone users — currently the entire local corpus — unblocked; publication users still refused | Same scope as B short-term, plus an explicit revisit trigger | None external | Yes — flip to A or B at revisit |
| **D. Witness-by-record middle ground** | Publish only when the current public record contains no gated-image references | Narrower than A: Leafbud would never delete copies a *live* record still shows | Record scanning for gated references (synthetic `$link` refs into storage) added to preflight | None upstream | As reversible as A |
| **Do nothing** | Status quo persists | Publication-container publishing refused; standalone works once execution lands | None | None | N/A |

**A — permit as disclosed behavior.** The operator accepts that invoking the
native publish action means accepting the native action's whole contract,
including housekeeping we did not design, and Leafbud discloses it in every
publish preview. The deletion remains bounded to the document's own
publish-owned folder and mirrors what the author's own web UI would do on the
same action. Work is a policy decision plus a small edit in
[publication.rs](/crates/workflow/src/publication.rs) (remove lines 140–143's
push while keeping the effect listed at line 135, with wording that names the
deletion explicitly); the already-designed execution slice then proceeds for
both contexts unchanged.

**B — require an upstream opt-out.** Keep the blocker; ask Leaflet for either
a publish argument that suppresses stale-copy cleanup or an RPC exposing the
gated folder so Leafbud can witness emptiness; re-pin a new archive revision
when it exists and add fixtures. Two honest observations: an RPC witness only
proves the no-gated-content case (a folder with contents still cannot be
safely published without a suppression flag), and meanwhile the users we would
be protecting must keep publishing through the web UI — which performs the
very cleanup we are refusing to perform. Publication-container users are
blocked for an indefinite, externally controlled time.

**C — standalone-first sequencing.** Ship the execution slice scoped to
standalone now and revisit the publication-container question with real usage
data. Interplay checked: the local corpus under `~/l` is entirely standalone
(every draft carries `"publication": null`), so Leafbud's own use is unblocked;
import/save/track of publication-*bound* documents is separate from the
publish action and unaffected, since the blocker gates only the
publication-container branch of the preview (publication.rs lines 131–143);
tests exercise both contexts and would keep asserting the blocker for the
publication fixture. Nothing else in the execution design depends on
publication-container publishing being available first. This is B's scope with
an explicit revisit; the judgment it defers does not shrink.

**D — analyzed, not advocated: the record-based middle ground.** The idea:
permit publication-container publishing only when the document's verified
current public record contains no gated-image references, reasoning that this
publish ensures nothing and the folder "should" be empty. The reasoning does
not hold:

- A clean current record does **not** prove an empty folder. Copies can
  survive from older publishes — cleanup failures are swallowed
  (`.catch(console.error)`, publishLeaflet.ts line 581), and a crash after the
  PDS write but before cleanup leaves orphans. Folder state tracks the last
  *successful* cleanup, not the current record.
- Since every publish path runs the same cleanup, the residual set D leaves
  unprotected is precisely the scenario-3 documents — web-UI gated history —
  which is the population the question is about. D narrows but does not
  remove it.
- What D genuinely offers: when the record is clean, no *live* published post
  references the folder, so deletions could only break cached or hand-copied
  old image URLs, never the current rendering. That is a real but modest risk
  reduction, purchased with new preflight machinery (scanning records for
  synthetic storage `$link` references) — and the user surprise ("it deleted
  orphans anyway") survives.

Verdict: D is sound only as "never delete what a live record still shows,"
and illusory as "no deletion happens." If the operator's real concern is the
former, A plus honest preview disclosure achieves nearly the same protection
without the machinery; if the concern is the latter, only B delivers it.

**Do nothing.** Publication-container publishing stays refused (the blocker
remains), standalone publishing works once the execution slice lands, and
nothing else is affected — imports, saves and tracking of publication-bound
documents do not pass through this branch. The question stays open until
revisited.

## Lean (operator decides)

Lean **A**. The effect is bounded to the document's own housekeeping folder,
is exactly what the native product does on the same action from its own UI,
cannot arise from Leafbud-authored content (no gated authoring support), and
the blocker's real root — no available witness — is an accident of which
interfaces exist, not evidence of danger. Leafbud already accepts and
discloses several unwitnessable-by-flag native effects of this same action;
this one differs only in lacking a witness, and D shows the witness gap cannot
be closed from our side. C is the reasonable fallback if the operator prefers
to decide with usage data, at the cost of keeping publication users on the web
UI, which performs the same cleanup anyway.
