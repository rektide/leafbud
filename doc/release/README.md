---
type: CapabilityContract
title: "Published editors and independent release evidence"
description: "Current verified native/public identity, explicit lifecycle adoption, editor-only saves and read-only public projection/status/diff boundaries."
resource: /doc/release/README.md
status: active
updated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e3983e092ffeuzOsaIZioZQNKF"
    title: "Corrected present-null one-to-one effect witnesses, retained publication-response diagnostics and actual Flight shapes; disposable first/update content settlement and record/editor deletion, with first-response evidence limitation"
  - resource: "opencode://ses_e39a14490ffevJd86pqM2pp36t"
    title: "0.25 native publication execution/recovery and distinct release-patch authority; fixture verification and honest blocked live smoke"
  - resource: /design/native-publish0.gpt61s.md
    title: "Execution/settlement proof and remaining deployed metadata/decoder boundaries"
  - resource: "opencode://ses_e3bb92bc4ffeN77oF8X2ayoH9a"
    title: "Added separate canonical record-content verification and actual loaded/compiled native inline-size witness; retained all execution/cleanup/delivery boundaries"
  - resource: /design/publication-verification0.gpt61s.md
    title: "0.23 content-CID and native size proof derivation, independent fixtures and fresh execution receiving brief"
  - resource: "opencode://ses_e3bcb6540ffegZC39tN1E6wuTc"
    title: "Added bounded native action/preflight foundation, exact standalone default-theme correction, and explicit hidden-effect/execution boundaries"
  - resource: /design/native-publication0.gpt61s.md
    title: "Pinned native action contract and fresh execution/recovery handoff"
  - resource: "opencode://ses_e3c10b84effeXUnnnL5oZ5M7TJ"
    title: "Implemented published native adoption/refresh/save and independent public evidence/Markdown diff; verified CLI, native/PDS HTTP and Phoenix fixtures"
  - resource: /design/published-lifecycle0.gpt61s.md
    title: "Initial native-source checkpoint and completed implementation receipt"
  - resource: /design/authoring-interface0.gpt6a.md
    title: "Accepted exclusive lifecycle, same source, save versus release and current-endpoint comparison"
  - resource: /crates/workflow/src/published.rs
    title: "Authoritative association and exact-URI native discovery"
  - resource: /crates/workflow/src/release.rs
    title: "Independent credential-free public acquisition/classification"
  - resource: /crates/leaflet/records/src/projection.rs
    title: "Strict supported public-meaning mapping and explicit richer/resource refusals"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Separate tracking/editor/lifecycle patch authority and preserved release witnesses"
---

# Published editors and independent release evidence

**0.25 implements preview-first native `publish FILE [--yes] [--json] [--take local]`.**
It saves the complete supported editor under one stable-ID lock, atomically records
editor-achieved/release-undispatched progress with the save receipt, persists
may-have-been-sent before invocation, then verifies exact native association,
PDS content CID and complete supported public meaning. A separate loss-checked
release authority settles lifecycle/URI, declared path/date/byline normalization
and `released`. No email/share, custom rkey, arbitrary actor or Leafbud cleanup.
Uncertain dispatch never retries; recovery only observes/settles correspondence.
See the [execution receipt](/design/native-publish0.gpt61s.md).

**0.22 supplied the action/preflight foundation.**
`leafbud_web::publication` supplies current-origin/session action discovery, literal
no-email arguments, explicit low-level transport and bounded results.
`workflow::publication::preview` validates complete source before remote reads,
reports missing keys/native page IDs, unsaved editor effects, actual owner/context
and path/date normalization, and rechecks source/duplicates/native metadata after
awaits. It writes nothing; execution reacquires fresh locked evidence and never
treats preview as persistent authorization.

**0.23 supplied verification prerequisites:** a bound/online preview replaces the blanket size blocker with
an actual loaded/compiled cumulative-native size witness. It retains XmlText/delta
facet fragmentation, counts UTF-8 JSON overhead, and proves no offload only when
the conservative bound is at most 102400 bytes. Saves/new UUID page IDs are explicitly
conditional/provisional; unbound/offline previews remain size-unwitnessed. Native
facts/metadata are rechecked after awaits, and future dispatch still requires fresh
saved-state proof—not a semantic estimate or server-CAS claim.

Publication-container context discloses accepted native housekeeping: **old image
copies may be deleted; count unknown**. It cannot be ruled out from an image-free
current source. Leafbud adds no deletion/storage access/retry. Present explicit null
in both selected one-to-one reply/schedule joins is the authoritative empty witness;
missing keys, malformed shapes or nonempty relations refuse equally for adopted and
freshly minted editors. Nonempty reply/schedule,
unsupported inherited preferences/current output and incomplete authority evidence
refuse or block. Native email is forced off; Bluesky sharing is a separate unused
action, not an inferred guarantee that all native lifecycle effects are absent.
See the [original action contract](/design/native-publication0.gpt61s.md) and
[current execution standing](/design/native-publish0.gpt61s.md). Both contexts are
fixture-proven. The final disposable standalone proof settled first publication and
same-URI update, verified clean editor/release status twice, then observed native
record/editor deletion and unbound the local source. The retained update response
decodes offline against its settled PDS CID; first-response bytes were unavailable,
so decoded first-action evidence is not claimed. The prior smoke's “missing reply”
diagnosis was null-vs-array confusion, not established deployed field omission.

## One source, verified identities

`@leafbud.state` is draft or published. The existing complete native binding
(origin, token, root, entity set, entry and explicit publication association)
adds optional `uri`. A draft cannot carry one; the current published source
profile requires an exact DID-based `site.standard.document` or legacy
`pub.leaflet.document` URI plus that complete native editor group. URI-only
local sources are not implemented; no partial/fake editor binding is accepted.

Published import reads **the current editor**, including unreleased work, and
keeps the exact public identity separately. `get_leaflet_data` linkage and joined
document/publication identities must agree. Publication owner comes from verified
native publication data; standalone site/owner must match the actual URI. Sibling
linkage on a different root, ambiguous links and missing/mismatched joined records
refuse. Native write permission and public readability are not interchangeable.

`import AT_URI` selects a unique exact native `/home` candidate, then reverifies
the association. Missing discovery is editing-unavailable through that inventory,
**not** public deletion. It never falls back to a public body, guessed title/path,
new editor or a second local file. Reimport uses guarded pull at the known path.
Duplicate UUID/native bindings/public URIs—including other origins—block adoption.

`pull --adopt-published` explicitly accepts a native draft-to-published transition
on the same binding. Default pull/watch/status/save do not accept that transition.
All native identity fields and publication association must remain unchanged;
only public URI/lifecycle are newly established. It preserves local UUID/keys,
writer/resolution evidence, names and explicit local date/path. Omitted/null
date/path adopt the verified published values. Editor conflict policy still applies;
this flag is not `--take remote`. Pending attempts block it. No release receipt is
created, even if current public/editor meanings happen to match.

## Editor saves stay editor saves

Working title/description/tags are native linkage-table values, not
`documents.data`. Published standalone metadata therefore uses
`updatePublicationDraft` like publication editors, rather than the loose-draft
permission-token metadata operation. The pinned native mutation changes these
working tables only; facts/Yrs continue through the existing keyed compiler.

`push` in either lifecycle never invokes a publication action or PDS write.
It retains public URI and any older `released` witness exactly. Pull/watch adopt
editor content while retaining local-only date/path, comments, independent names
and release evidence. Joined public body/date changes do not enter the published
editor witness; current fact/extras coverage remains conservative. Public drift is
its own observation, not an editor conflict or an instruction to overwrite it.

Native authority, original-byte checks, stable-ID locks, late full-selection
duplicates and no-clobber writes still apply. New public adoption also uses the
existing OS-lock mechanism keyed by public URI, so alias tokens cannot race two
local checkouts into existence. This is not filesystem/server CAS.

## Public meaning profile

`leaflet-public/1` maps supported working meaning to the pinned native public
record shapes and decodes fetched output through generated lexicon validation
plus strict mapping coverage. It is **not a publication effect plan**.

Includes title, exact nonempty description, ordered tags/contributors, established
site/publication, author path/date intent, native public page identities, block
order, text/marks/breaks, headings/quotes/rules/code, flat lists and unique reachable
linear page links. Native UTF-8 byte facets decode to the existing document model;
run segmentation/feature order normalize by actual text/marks. Equivalent offset
dates normalize to one instant. Empty/missing description, tags and contributors
use their defined empty meanings. Source keys/layout/comments, arbitrary unknown
author metadata, names, writer/sync and resolution provenance are excluded.
Editor watermark is excluded because the inspected publisher does not project it;
this is not permission to ignore an arbitrary public theme.

Explicit current limits:

- Release projection/diff supports `site.standard.document`; legacy published
  sources can adopt/refresh/save, but legacy release comparison refuses.
- URL images lack resolved public blob byte identity. No image fetch/upload, fake
  blob equivalence or lossy omission. Offloaded/gated content, canvases, richer
  blocks/facets, cover/nondefault-theme/preferences/share references and unknown output fields
  refuse a complete comparison. Observed CID/change remains visible when possible.
  The exact pinned standalone default theme (`showPageBackground:true` plus its
  type discriminator only) now compares as default; any extra/nondefault property
  still refuses. The publisher retains that object because its key-count test
  includes the discriminator. Editor watermark remains separate.
- Colored highlight slots needing native theme resolution refuse; default slot 1
  is supported. Nested/task lists, numbered restarts and adjacent separate
  same-style list boundaries refuse rather than flattening public meaning away.
- Repeated/cyclic or unreachable page graphs refuse. Native repeated references
  can repeat subpages in output; this profile does not invent deduplication.
  A new page without a native public identity needs an editor save first.
- Multiple selected contributors refuse working projection: native pull lacks
  the created-at ordering the publisher uses. It does not silently sort bylines.
- Missing/unresolved path/date cannot become an invented path or current time.
  An empty public title also refuses: this source profile requires a nonempty title.
  Existing imported dates remain explicit; date changes require author edits.
  Unsupported local release fields refuse; supported path comparison does not
  imply custom native path mutation. Future publishing must disclose its `/rkey`
  normalization and settle accepted author values separately.

## Independent evidence and commands

```sh
leafbud status post.mdx --json                    # editor, names and release
leafbud status post.mdx --public-only --json      # no native credentials/read
leafbud status post.mdx --offline --json          # neither remote edge fetched
leafbud diff post.mdx --edge release --tool diff --tool-arg=-u --tool-exit diff
```

Public reads use the binding URI's actual repo DID and existing ATProto resolver/
`getRecord` client without session creation, cookies or bearer tokens. The bound
owner resolves to its PDS; global/workspace PDS defaults cannot select a different
account's server for this binding. An explicit `--pds` (or library endpoint override)
selects an endpoint, not a different repo/account. Evidence names the bound URI and
whether the endpoint was explicit or owner-resolved; missing means the selected
endpoint's response, not proof of global deletion. No configured handle/publication retargets it.
`status --public-only` and release diff do not even open the native credential store.
Only the actually fetched record is compared; a failed public read produces no
invented empty body. Human views reuse the existing Markdown/external-tool path,
including private temporary views, foreground lifetime, exit modes and cleanup.
Supported site/publication drift on the **same fetched URI** can be shown in diff
while status reports linkage-mismatch; viewing it does not accept a new binding.

Optional compact `released` contains profile, confirmed input hash, reported
output CID, actual path/date and confirmation time. It is last **confirmed
correspondence**, not a public-read cache. Import leaves it absent. Ordinary editor
tracking/adoption explicitly cannot advance it; only verified native publication
execution/recovery creates it through `leafbud_mdx::settle_release`.

Status reports input/output change independently. A known baseline can yield
clean, input-changed, output-changed, both-changed or converged-unaccepted.
An imported unknown baseline yields equal-untracked/different-untracked—never a
fabricated release acknowledgment. Missing record, unavailable fetch, identity/CID
mismatch, linkage mismatch, unsupported projection, offline and not-requested are
distinct. Source changes during reads mark stale evidence and preserve the file.
Known missing public output is not permission to resurrect it.

Status/diff CID evidence remains **PDS-reported and syntax/profile validated, not
independently rehashed or signature verified**. Separately,
`atproto::content::verify_record_content` and
`PublicReader::fetch_content_verified` verify actual canonical AT-data DAG-CBOR
content against its reported CID, with typed data/profile/digest refusals. This
stronger API is used for publication settlement, not silently applied to old
observations. Rehash establishes content addressing
only, not authorship, actor, time, repo signature or delivery.
Full supported current meaning is compared as
well; a claimed unchanged CID with inconsistent current meaning is not clean.
Unknown future owned receipt fields/profiles refuse managed mutation, not drop.

Undecoded publication responses use the same private bounded create-new receipts
as mint responses. Reports retain the diagnostic/path even when independent
observation settles correspondence; they distinguish a decoded agreeing action
result from observation-only settlement. No receipt body enters source or logs.

See [source ownership](../mdx/README.md), [executable coverage](../mdx/coverage.md),
[usage](/README.md) and the [live proof and decoder qualification](/design/native-publish0.gpt61s.md#final-disposable-proof-and-bounded-decoder-repair).
No `~/l` or pre-existing document was written. Publication-container effects remain
fixture/source-grounded, not a newly commissioned live publication-container test.
