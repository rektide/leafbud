---
type: OperatorDecisionExplainer
title: "May Leafbud trigger Leaflet's per-post image cleanup?"
description: "An implementation-side, plain-language account of the publication-container cleanup choice, its actual user impact, and the work each option requires."
resource: /design/native-cleanup-question0.gpt61s.md
status: decided-option-a
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3bb92bc4ffeN77oF8X2ayoH9a"
    title: "Implementer independently rechecked pinned native cleanup, action/UI scope and Leafbud guards, then explained operator options; no parallel explainer consulted, no policy accepted or code changed"
  - resource: /design/native-publication0.gpt61s.md
    title: "0.22 action/effect findings and later publication-container-only scope clarification"
  - resource: /design/publication-verification0.gpt61s.md
    title: "Parent-accepted 0.23 verification prerequisites and remaining execution/recovery join"
  - resource: /crates/workflow/src/publication.rs
    title: "Current publication-container-only cleanup refusal, distinct execution and size checks"
  - resource: /crates/workflow/src/publication/tests.rs
    title: "Existing both-context preview test checks cleanup refusal only for publication containers"
  - resource: /crates/leaflet/web/src/publication.rs
    title: "Current action arguments force email off but have no cleanup control"
  - resource: /crates/leaflet/editor/src/lib.rs
    title: "Supported native block kinds exclude the members-only delimiter"
  - resource: /crates/leaflet/records/src/projection.rs
    title: "Release profile refuses images rather than silently omitting them"
  - resource: "file:///home/rektide/src/rekon/code/GLOBAL.md"
    title: "Operating principle: no automatic prune/reconciliation without an explicit request, lines 66–67"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/gatedDocumentImages.ts
    title: "Document folder, copy naming, cached listing limited to 1000, ensured set and removal"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "Publication owner, stable post key, conditional store creation, post-write cleanup and separate linkage-failure compensation"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts
    title: "Authenticated ordinary action forwards arguments; no cleanup suppression capability"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts
    title: "Members-only image copies use the gated store; ordinary images take a different path"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/factsToPagesRecord.ts
    title: "Members-only delimiter determines which following blocks use the gated image path"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/membership.ts
    title: "Public PDS record deliberately excludes the members-only portion of publication posts"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/addImage.ts
    title: "Draft upload filenames derive from UUIDs, with an extension for animated files"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_leaflet_data.ts
    title: "Available editor/document metadata response does not list Supabase storage objects"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/%5Bleaflet_id%5D/publish/PublishPost.tsx
    title: "The native UI's ordinary Publish button calls the same action"
---

# May Leafbud trigger Leaflet's per-post image cleanup?

## 1. TL;DR — the decision you can make

**This is not a proposal for Leafbud to tidy your files.** Leaflet's own publishing
operation can delete old copies of images that used to appear in a post's
members-only section. Calling that operation from Leafbud can trigger the same
deletion, even when the post now contains no images.

The choice is: **allow that existing Leaflet behavior with a clear warning, or keep
publication-container publishing blocked until we can suppress it or genuinely
prove there is nothing to delete.** Standalone posts are outside this cleanup
branch. You can also choose to build standalone publishing first and revisit the
container choice later.

Here, a **publication container** means a named Leaflet publication, such as the
publication containing your newsletter's posts. A **standalone post** is published
on its own, without that association. “Gated” means placed in a members-only
section; it is not a claim that every underlying image URL is secret.

**Current-status caveat:** Leafbud's actual `publish` workflow is still unbuilt.
“Standalone can proceed” below means this cleanup question need not block building
and testing it—not that publishing already works. The accepted
[verification checkpoint](/design/publication-verification0.gpt61s.md#boundary)
supplies prerequisites, not a completed publisher. No decision is made by this note.

## 2. What Leaflet does — think of a separate photo album for each post

Leaflet keeps **publishing-owned copies** of members-only images. These copies
are separate from the original image uploads used while editing. The reason is
practical: removing an image from a draft should not immediately break an older,
still-published version of that post.

Think of a post's published images as its own photo album:

1. Publishing makes sure the album contains copies of the members-only images
   needed by the new version.
2. Once the post has been written and linked successfully, Leaflet checks the
   album for old copies the new version did not ask to keep.
3. It asks its storage service to delete those old copies. If the new version
   has no members-only images, nothing is marked to keep.

The actual storage service is **Supabase**. The album is a folder inside its
`minilink-user-assets` bucket:

```text
gated-documents/<post-owner DID, with colons changed to dots>/<post record key>/
```

A **DID** is an account identifier; the **record key** identifies this published
post within that account. For a publication-container post, the account is the
publication owner's, even when a contributor presses Publish. Updates reuse the
existing post key. Ordinary copies reuse the original upload's UUID-based
filename; an image from another URL instead gets a stable hash-based name.
This is not a sweep of the whole bucket or account.

**Implementation detail that changes the promise:** the helper lists **at most
1000 objects**, with no pagination in this function. It targets the *listed*
objects absent from its keep set. “Delete all unused copies” describes its intent,
not a guarantee that an arbitrarily large folder is completely cleared. Cleanup
is also best-effort: rejected cleanup errors are caught and logged, and storage
result errors are not turned into a cleanup-success report. Publishing success
does not prove cleanup succeeded, and cleanup failure does not roll back the post.

**Source:** [gatedDocumentImages.ts:4–25, 29–79, 83–99][store];
[addImage.ts:75–79][upload-name]; [publishLeaflet.ts:180–203, 276–291][publish-scope];
[publishLeaflet.ts:481–550, 578–581][publish-cleanup].

## 3. When it runs — and when it does not

| Situation | This gated-image cleanup |
| --- | --- |
| Standalone publishing or updating | **Never enters this branch.** Leaflet does not construct the gated-image store. |
| Publication-container first publish or update, reaching the normal post-write cleanup step | **Attempts cleanup**, whether today's post contains gated images, ordinary images, or no images at all. |
| Reading, comparing, or previewing in Leafbud | **Does not run.** Our current preview is read-only. |

“Always for a container” does **not** mean a rejected login or an early failed
publish deletes files. It means there is no “skip cleanup because this post is
image-free” condition when a container publish reaches that step. It happens
after the public post and its native linkage have been written; a later lost or
failed response must not be interpreted as proof that nothing happened.

Turning email off does not turn cleanup off. Neither the ordinary server
action's argument type nor Leafbud's current request has a cleanup switch.
Inventing and sending a new flag would not make the old server obey it.

**Source:** [publishLeaflet.ts:76–99, 276–291, 481–581][publish-all];
[publishToPublication.ts:14–39][action];
[Leafbud publication.rs](/crates/leaflet/web/src/publication.rs):10–34;
[read-only preview](/crates/workflow/src/publication.rs):1–2, 13–14.

## 4. What would happen to my posts?

### A standalone post: zero impact from this cleanup

No gated-image store is created, so this helper deletes no copies. This is true
even though the native action has the confusing name `publishToPublication`:
that action handles standalone posts too. Other publishing effects are separate;
“zero” here refers specifically to this cleanup path.

### A publication post that has never had members-only image copies

If its folder really is empty, there is nothing for this helper to delete.
The problem is **not that we expect hidden images in every post**. It is that
Leafbud cannot currently ask the server to prove the folder is empty.

`get_leaflet_data`, the server request we use to inspect the editor and its
publication association, returns database metadata and document records. It does
not perform the Supabase storage listing used by cleanup. “No images in the post”
and “no old files in the post's storage folder” are different facts.

### A post formerly containing members-only images, later removed in the web UI

Removing the image or members-only section leaves no reason for the *next
publish* to keep that image's publishing-owned copy. The native publisher can then
delete it from this post's folder. **The behavior belongs to the native operation,
not to the button that called it:** the web UI calls that operation, and the
planned Leafbud publisher uses it too.

If someone saved or shared the old copy's direct URL, deletion can break that URL
at the storage origin. Previously downloaded or cached bytes are not erased by
this operation. Leafbud is not offering an archive of old image URLs, nor a
restore button for deleted copies.

There is an important present-day limit: Leafbud refuses currently gated/richer
content rather than silently converting it. It cannot already publish any such
post. This scenario explains what the native operation does **if reached**; it
does not override those refusals. An otherwise supported, now image-free post can
still have leftovers from older versions—for example, if an earlier best-effort
cleanup failed or did not list every copy.

### A post still using members-only images

For each such image, the publisher requests its copy through `ensureImage` and
records its name in the keep set. Cleanup excludes those names; it targets old
unused copies, not the images this publish asked to keep. Copying itself has
failure/fallback behavior, so this is not a universal image-delivery guarantee.
Currently, authoring this gated content remains a **web-UI capability**, not a
supported Leafbud publishing capability.

### What this path does not delete

- **Not draft uploads generally.** The removal targets are names inside the
  calculated `gated-documents/<owner>/<post-key>/` folder, not the original upload
  paths used as copy sources.
- **Not other documents' folders.** There is no bucket-wide or cross-folder sweep;
  the deletion scope is this post's owner/key folder.
- **Not PDS records.** A **PDS** is the account's AT Protocol record server. This
  helper calls Supabase file removal, not a PDS record-delete operation. The
  publisher has a *different* first-publish compensation path that may delete a
  newly written record if linking it fails; that is not this image cleanup.

**Source:** [get_leaflet_data.ts:10–36][metadata];
[gatedDocumentImages.ts:29–79][store]; [publishHelpers.ts:21–59][image-hooks];
[PublishPost.tsx:303–318][ui-publish];
[publishLeaflet.ts:532–550, 578–581][publish-cleanup].
The cached-byte and missing-restore cautions are limits on what these removal
calls establish, not claims that we tested a storage deployment.

## 5. Why Leafbud stopped and asked

Our operating rule says not to add automatic cleanup without an explicit request:
[Rekon code/GLOBAL.md](file:///home/rektide/src/rekon/code/GLOBAL.md):66–67.
Publishing is a requested operation; silently treating it as permission to delete
old files as well would skip an actual operator choice.

The code therefore adds a **blocker**—a reason to refuse the operation—only when
the post belongs to a publication container. A **preflight** is the set of checks
before sending a request that can change remote state. A **witness** is evidence
from an actual read, such as a verified listing, rather than an assumption.

We can describe the folder scope. We **cannot enumerate or verify the files it
will delete before dispatch** with the current API, or suppress deletion. That is
the unknown—not “Leaflet might delete anything anywhere.” The existing
[preview code](/crates/workflow/src/publication.rs):131–143 and its
[both-context test](/crates/workflow/src/publication/tests.rs):80–101 already
distinguish containers from standalone correctly.

The exposure is narrower than the old wording might suggest. Leafbud's supported
release profile cannot author gated content: `members-only-delimiter` is absent
from its accepted native block kinds, and release projection refuses images.
The concern is **pre-existing history created outside this supported profile**,
not Leafbud quietly creating an image collection and deciding to prune it.
It still refuses an actively unsupported editor/public body; historical leftovers
are not a reason to weaken those content checks.

**Source:** [Leafbud editor reader](/crates/leaflet/editor/src/lib.rs):207–265;
[release projection](/crates/leaflet/records/src/projection.rs):179–180;
[native members-only classification — factsToPagesRecord.ts:758–774][delimiter].

## 6. The options — consequences, work, dependencies, reversibility

These are choices about the **future** publisher. None requires changing today's
editing, pull/watch synchronization, status, or comparison behavior.

### A. Permit this native cleanup, disclosed as part of publishing

**User impact:** a container publish behaves like Leaflet's own ordinary Publish
button. Unused historical copies in this post's folder may be removed, including
all listed copies when no gated images remain. Old direct-copy links may break.
There is no promise to preserve them or prove exactly how many were removed.

**Leafbud work:** record the operator's scoped acceptance; replace this particular
container blocker with a clear preview/confirmation disclosure. Keep the other
guards. Carry that accepted behavior into the execution path; do not add a
separate Leafbud file-deletion routine, storage credentials, or cleanup retry.
The notice should say “old copies may be deleted; count unknown,” not “no images,
therefore no deletions.” A general approval of automatic cleanup is unnecessary.

**Dependencies:** no upstream change for this cleanup question. The normal
save/execute/verify/recovery implementation still has to be built and tested.

**Reversibility:** we can later reinstate the refusal or adopt an upstream opt-out.
That cannot undo copies already deleted. Restoring a copy would require retained
bytes and an appropriate restore operation; neither is supplied by this choice.

### B. Require an upstream way to avoid deletion, or inspect real storage

**User impact:** container publishing stays refused while the missing capability
is obtained. Standalone publishing work can proceed independently.

There are two materially different upstream requests:

- **A real opt-out, the clearest no-deletion contract.** Ask Leaflet to add a
  publish argument such as a hypothetical `pruneGatedImages: false`, forward it
  through the ordinary authenticated action, and actually skip `removeStale` when
  false. Existing UI behavior could remain the default. This flag does **not exist
  today**. It would avoid this deletion even when old copies exist, at the cost
  of retaining obsolete storage.
- **A storage-inspection API.** Ask for an authenticated, read-only server request
  exposing the exact post folder or a complete empty/nonempty result, with correct
  owner/post association and explicit error handling. This could support “allow
  only an empty folder,” but **listing is not itself an opt-out**. A separate read
  can become stale before publishing. A strict promise never to delete existing
  copies additionally needs server-side enforcement—such as refusing a publish
  that would prune, or skipping cleanup—not just a hopeful client-side empty check.

**Exact coordination/work:** agree the behavior with Leaflet's maintainers; change
the server implementation and its tests; get that version deployed; provide a
verifiable way for Leafbud to recognize support. Then extend our action adapter
or reader, preview checks, execution guards and tests. Finding the same action
name is not proof that an older server understands a new flag. For inspection,
complete/error-aware listing matters: the current helper's 1000-item cap and
empty-on-missing-data treatment are not an adequate evidence contract to copy.
For a first publication, the post key is chosen inside the action, so a read API
also needs to explain how it covers that not-yet-established folder.

This is a service change and deployment dependency, not a flag we can turn on
locally. It need not involve rewriting the publisher or adding gated-image
authoring to Leafbud.

**Reversibility:** postponement can be ended by later choosing A. A genuine
opt-out can later be switched back to native cleanup only by another explicit
choice; retained old files could then become deletion candidates. Suppressing
cleanup in Leafbud does not change what a later web-UI publish does.

### C. Build standalone publishing first; explicitly revisit containers

**User impact:** get the unaffected publishing workflow first. Container posts
remain unavailable through Leafbud until a later decision; use the web UI if its
existing behavior is acceptable. This does not move or convert container posts
into standalone posts.

**Exact work:** implement the shared save/execute/verify/recovery pieces and the
standalone path; keep the container-specific refusal and test it. Leave a visible
follow-up decision rather than silently treating postponement as acceptance.

**Dependencies:** no upstream cleanup change. The substantive standalone execution
work remains. This is sequencing, compatible with eventually choosing A or B.

**Reversibility:** later add container execution after its policy is settled.
Nothing about C itself deletes these copies or needs a content migration.

### D. Permit publishing if the current public record has no gated-image links

**User impact as advertised:** apparently permits ordinary clean posts while
avoiding the danger. **As a no-deletion guarantee, this is partly illusory.**

A clean current record does not inventory old storage. Earlier copies can remain
after a failed or incomplete best-effort cleanup, even though the new record was
successfully written without them. More fundamentally, the public PDS record
deliberately excludes a publication post's members-only section. Absence of image
references there is not even a reliable inventory of currently used gated images.
A full native database record avoids that truncation problem, but still says
nothing complete about abandoned copies in the folder.

**Exact work:** adding a record-only check is easy, but it would not establish the
promised property. We should not remove the refusal on that basis. To make D
sound, replace its supposed witness with actual folder evidence and address the
read/publish race—then it becomes B's inspection/enforcement variant. Alternatively
call it a risk-tolerance policy, not proof of no cleanup, and seek explicit
acceptance just as in A.

**Dependencies/reversibility:** no existing content hash verifier fixes this
information gap; hashing proves which record was read, not what old files exist.
A weak check can be removed later, but cannot restore deletions allowed under it.

**Source:** [membership.ts:108–153][public-truncation];
[publishLeaflet.ts:481–550, 578–581][publish-cleanup];
[gatedDocumentImages.ts:21–25, 67–79][store].

### Do nothing for now

**User impact:** keep the existing publication-container refusal. Standalone is
unaffected by *this* question; other work can continue. No cleanup acceptance,
upstream request, or new storage subsystem is implied.

**Work/dependencies:** no cleanup-specific implementation. Keep the current guard
and truthful explanation. Unlike C, this does not positively choose standalone as
the next delivery slice. It is reversible by a later decision, with no deletion
caused by leaving the guard in place.

## 7. What this changes in the already-designed implementation

This choice removes or retains **one** reason not to send a publication request.
It is not the missing publisher by itself. The
[native action receipt](/design/native-publication0.gpt61s.md#proposed-executionrecovery-join--not-implemented)
still requires saving edits first, checking the exact owner/post, recording that a
request might have been sent before sending it, verifying the returned public
content, and handling an interrupted response without blindly sending it again.
**Minting** means creating a new remote editor/draft; agreeing to cleanup does not
authorize an unrelated mint or conversion.

Concretely, the next implementation should differ as follows. **These are proposed
tests/work, not claims that this document added them.**

| Choice | Preview/execution change | Tests needed for the cleanup decision |
| --- | --- | --- |
| A | Replace only the cleanup refusal with accepted native-effect disclosure; never claim an empty folder or cleanup success. | Both contexts; container notice even with image-free source; stale-history case does not get labeled zero-deletion; lost/successful action responses do not invent a verified deletion outcome or cause a cleanup retry. |
| B, opt-out | Refuse containers until actual server capability is verified, then explicitly request no pruning. | New flag on the wire; an old/unrecognized server still refuses; upstream stale-folder fixture remains untouched with opt-out and retains ordinary cleanup behavior by default. |
| B, inspection | Refuse absent, failed, incomplete or wrong-folder evidence; enforce the agreed handling of changes before publishing. | Empty/nonempty/error/large-folder/wrong-owner cases and a change between checking and publishing; server-side refusal or suppression if a strict no-deletion contract is promised. |
| C / do nothing | Preserve container refusal; C commissions standalone execution first. | Standalone never gets this refusal; container request is not sent; a bound container is never silently retargeted as standalone. |
| D, record-only | Do not substitute it for storage proof. | Counterexample: clean current record plus historical folder entries still has deletion exposure. |

All options leave the 0.23 content-address verification and native page-size proof
in place. Checking the saved editor again is necessary for correct publishing;
it does not make hidden storage contents visible. Existing reply, schedule,
contributor, unsupported-content and uncertain-request guards are separate too.
Choosing A must not remove them as collateral damage.

The concrete upstream coordination for B is chiefly **agreement on a server
guarantee, a reviewed patch and tests, deployment, and capability verification**.
The opt-out's control flow is small; obtaining and verifying the service contract
is the important dependency. A raw listing route is more conditional and needs
more careful ownership/completeness/race design than its short name suggests.

## 8. My lean — recommendation, not your decision

**I lean toward A: permit this native, per-post cleanup with an explicit disclosure
and a narrowly recorded operator exception.** It keeps Leafbud's publishing
behavior aligned with the web UI, does not add a second image-management system,
and avoids making every ordinary image-free container post wait on an upstream
change because of uninspectable history it probably does not have. The cost is
real: old copy URLs are not being preserved, and a successful publish cannot tell
us exactly which copies were deleted. If those historical URLs are important
archives, or “never delete a remote file during publication” is the desired hard
rule, I would choose B's genuine opt-out instead. If you are not ready to decide,
C is an honest way to keep standalone work moving—not implied acceptance of A.

An actionable response could simply be: **“Allow that existing per-post cleanup,”
“Require no deletion first,” or “Standalone first; revisit containers.”** None is
selected by this document.

## Source confidence and refinements to the earlier framing

Before this reread, the local upstream archive was confirmed clean at
**600c7b5f1f7b31b66fb626c293cc104312a12caf**. Upstream links below bind that exact
revision and line ranges; local code citations describe the parent-accepted 0.23
checkpoint through `cf523b8b`. This was source inspection, not a test against the
running service. No publish, file cleanup, credential access, ticket update, or
code change was performed to write this explainer. No parallel explainer was read.

Three refinements matter more than the original technical shorthand:

1. **“Every publication” means every publication-container publish reaching this
   step, not standalone.** Our existing blocker already has the correct scope.
2. **“Cleanup” is a bounded, best-effort deletion attempt, not a verified complete
   sweep.** Its 1000-object list cap and unreported storage errors make “all old
   copies deleted” too strong as an observed result.
3. **The public-record witness alternative is weaker than it sounds.** Even a
   fully verified clean record does not establish an empty historical folder;
   the PDS version can additionally omit the gated content on purpose.

[store]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/gatedDocumentImages.ts#L4-L99
[upload-name]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/addImage.ts#L75-L79
[publish-scope]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts#L180-L291
[publish-cleanup]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts#L481-L581
[publish-all]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts#L76-L581
[action]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts#L14-L39
[metadata]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/get_leaflet_data.ts#L10-L36
[image-hooks]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts#L21-L59
[ui-publish]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/%5Bleaflet_id%5D/publish/PublishPost.tsx#L303-L318
[delimiter]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/factsToPagesRecord.ts#L758-L774
[public-truncation]: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/membership.ts#L108-L153


## Decision (2026-11-10)

The operator chose **option A — "let Leaflet be Leaflet."** Publishing from
Leafbud accepts Leaflet's own publish-time behavior, this cleanup included,
exactly as the web UI's publish button does. Leafbud discloses it in the
preview in plain words ("old image copies may be deleted; count unknown") and
adds no deletion machinery of its own. This section records the outcome; the
analysis above is unchanged.
