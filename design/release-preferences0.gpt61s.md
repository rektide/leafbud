---
type: ImplementationReceipt
title: "Source-owned publication visibility preferences"
description: "0.26 tri-state visibility frontmatter and edit flags wired through native working replacement, deterministic quiet publication, default-aware public meaning and linkage settlement; fixture-only evidence and explicit research corrections."
resource: /design/release-preferences0.gpt61s.md
status: implemented-awaiting-parent-review
generated:
  at: 2026-11-10
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e396fdd15ffedI3bJni7Y1uvTs"
    title: "Fresh Sol high implements the four-preference loop, records commission/native-source corrections, verifies source/wire/save/public settlement fixtures and preserves absent/default hash continuity; no live effects"
  - resource: /design/publish-options0.glm53m.md
    title: "Primary inventory, tri-state proposal, native resolution and implementation map"
  - resource: /design/native-publish0.gpt61s.md
    title: "Existing guarded save-first publication, explicit-null witnesses, dispatch uncertainty and complete settlement being extended"
  - resource: /design/native-publication0.gpt61s.md
    title: "Pinned action/effect surface and native hidden effects which preference flags do not suppress"
  - resource: /design/frontmatter0.gpt61s.md
    title: "Source ownership and missing/null/explicit precedent; delivery remains reserved"
  - resource: /doc/release/README.md
    title: "Maintained post-visibility and public-meaning contract"
  - resource: /doc/mdx/README.md
    title: "Typed source, metadata patch and operation-specific authority"
  - resource: /doc/mdx/coverage.md
    title: "Executable evidence and remaining refusals"
  - resource: /crates/document/model/src/preferences.rs
    title: "Strict four-boolean mapping and default/persisted predictions"
  - resource: /crates/workflow/src/publication.rs
    title: "Per-key disclosure, native selection and inherited-meaning guard"
  - resource: /crates/workflow/src/native/publication.rs
    title: "Paired action args, fingerprinted core and exact direct-linkage settlement"
  - resource: /crates/workflow/src/native/publication_tests.rs
    title: "Fixture-only first/update/clear/no-op and mismatched-linkage proof"
  - resource: /crates/leaflet/web/tests/fixtures/publish-preferences-args.json
    title: "Independent literal omitted/clear/value action argument expectations"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "Re-read clean pinned lines 309–331: whole-object preference selection, Discover-only record fallback and false-only persistence"
---

# Publication preferences — 0.26

## Standing and checkpoints

Implemented in `0af9c65e`, with review correction `f7176afc` preserving absent/default
hash continuity. All workspace packages and exact internal pins are **0.26.0**;
the Cargo lock is updated. Installed `cargo bump` 1.1.0 refuses workspace manifests,
so the coordinated root version/pins were patched and Cargo regenerated the lock.
No dependency, migration framework, new receipt field, body snapshot or transport
retry was introduced. `leafbud-release-preferences` awaits parent acceptance; this
session claims implementation/verification, not ticket acceptance.

No live request, mint, save, publish, delivery, delete or `~/l` change occurred.
Tests use fixtures and loopback servers. The archived publisher read in this session
is at `600c7b5f1f7b31b66fb626c293cc104312a12caf`; its inspected source file has no
working changes. Earlier native-publication live proof remains historical evidence,
not a live proof of these new arguments.

## Source and CLI surface

```yaml
preferences:
  showInDiscover: false
  showComments: false
  showMentions: true
  showRecommends: false
```

Missing/null means inherit without sending preferences. `{}` explicitly requests
no overrides. A populated mapping accepts exactly these four optional boolean
keys; arrays, unknown keys, strings/numbers and per-key null refuse. Manual mapping
deserialization requires an object: derived struct deserialization alone accepted
an empty sequence during the first scoped test, so that hole was closed explicitly.
The existing YAML parser also refuses duplicate keys, tags, anchors and aliases.

```sh
leafbud edit post.mdx --preference showComments=false --preference showInDiscover=false
leafbud edit post.mdx --clear-preferences
leafbud publish post.mdx --json
leafbud publish post.mdx --yes --json
```

Repeated preference flags merge selected keys into the existing source mapping;
within one invocation the last value for a repeated key wins. Clear conflicts with
preference flags and writes `{}`. Removing the field or spelling null by hand
returns to inherit. Publish has **no** preference override flags: source remains
the inspectable input. Offline managed edit keeps the native-attempt guard, selected
field authority, exact body/unrelated YAML and no-op bytes/mtime behavior.

## Native resolution and design corrections

The primary [options design](publish-options0.glm53m.md) remains the inventory and
architecture basis. These discrepancies are resolved explicitly, not silently:

1. **Clear flag:** its §2.4 removes the field, but the present commission explicitly
   requires `{}`. The implementation follows the commission. Clear means explicit
   clear, not inherit.
2. **Whole-object selection:** the pinned publisher destructures
   `postPreferences ?? draft.preferences ?? {}`. Once the argument exists, missing
   individual keys do **not** merge from the old draft. Only Discover has the
   further existing-record fallback. The research's per-key inheritance language
   and unconditional authored→draft→record description are not literally correct
   in the explicit-object case. Preview accounts for the preceding full-replace
   editor save, as execution does.
3. **True clear needs two arguments:** `{}` alone does not clear an existing
   record's Discover false. Leafbud sends `postPreferences:{}` plus
   `showInDiscover:true`, preserving the promised explicit-clear meaning.
4. **Nondefault inheritance:** the commission retains a source-control requirement,
   stronger than a possible reading of §2.6 that any readable inherited shape may
   publish. Default true shapes are accepted; inherited nondefault output blocks
   until authored. In particular, a partial mapping without Discover must explicitly
   select it if an existing record would contribute false. This avoids an implicit
   opt-out reset and keeps dispatch/core prediction source-reconstructible without
   expanding the durable continuation schema.

For explicit mappings, action args contain the authored Comments/Mentions/Recommends
subset and a paired resolved Discover boolean. The three-key object may be empty;
neither argument appears for inherit. The dispatch fingerprint includes both via
the existing argument hash. Low-level transport refuses unknown keys and unpaired
arguments before POST. `sendEmail:false` remains unconditional.

`showInDiscover:true` is **dispatch-only**: the record and published draft-row output
retain only its false opt-out. The other three explicit true values do persist.
Preview reports authored input, per-key explicit-send/inherit/default sources and
observed values, wire spelling, replacement and predicted persisted output.
Unobserved previews label missing evidence unknown rather than inventing observed
defaults. An explicit object can replace unreadable draft preferences; unreadable
existing public output remains a hard refusal even for explicit clear.

## Save, observation and settlement

Where a publication/published working row exists, the existing native metadata
mutation sends the **whole authored four-key subset**, including authored Discover
true, only when it differs from the observed row. It does not infer differences
from the local file or released record. Clear sends `{}`. Loose standalone drafts
have no such row: their preference intent remains local until first publication,
then subsequent saves use the published standalone row normally.

The native preference client key is `post_preferences`, not `publication_preferences`.
The protection witness excludes it only for a declared preference replacement;
the allowed protected-metadata field list grows from three to four. Read-back
verifies both the direct working row and client key against the exact replacement.
Other native scope remains protected. Working-row preference evidence joins the
conservative published-editor witness; released-record preferences do not.

Import can represent known native preferences. Unreadable draft preference evidence
stays conservatively witnessed without pretending it decoded into a supported
author map; inheritance at publication refuses it, while explicit source replacement
can repair it. Existing missing intent stays inherit on pull/watch. For explicit
intent, semantically equal native normalization retains original source spelling.

The strict public decoder admits only the four-key boolean subset, with optional
exact `pub.leaflet.publication#preferences` discriminator. Persisted Discover true,
unknown keys, null/nonboolean values, wrong types and unknown public output refuse.
`working` predicts Discover's actual persisted shape before comparison.

Public comparison is **meaning-based**: all four defaults are true, so absent,
empty and explicit default-true shapes compare equally. False values remain part
of the public core. Canonical default public metadata omits the new preferences
member, retaining prior default receipt hashes. Absent source intent likewise
retains prior editor/authored/semantic hash shapes; explicit mappings are hashed.
This is a bounded canonicalization choice, not a compatibility shim or migration.

Settlement requires predicted meaning to agree across the content-verified PDS,
native association record, decoded action result when available, and the exact
direct linkage row's preferences. No new `released` field is needed. Authored
preferences—including Discover true—survive settlement exactly; they are not
rewritten to the normalized output. Before the no-op shortcut, explicit intent
also checks persisted preference shape, so a requested `{}` is not skipped merely
because an old explicit true has default meaning. A clean repeat after actual
native normalization remains zero-effect and byte/mtime-stable.

## Verification and retained boundaries

Final code gate: **421/421 regular workspace tests**, ten existing opt-in skips,
nextest run `9dfd1eb1-c1e6-43d9-97df-81b159e9f30c`. Strict workspace/all-target Clippy
passes. Final formatting/Markdown provenance checks are recorded below after the
documentation pass. The full [coverage account](../doc/mdx/coverage.md#026-preferences-loop)
names the fixtures; principal new evidence is:

- strict source tri-state/shape/types, merge/clear and exact block/flow/header/body
  preservation; offline real CLI validation, byte/mtime no-ops and flag-free publish;
- literal native metadata and actual Next argument fixtures for omitted/clear/value
  cases, paired argument validation and unconditional no-email;
- each public preference's true/false/default meaning and strict unsupported output;
- **1200 pure resolution cases** over the three ordinary keys × Discover tri-states,
  explicit/inherited/empty/null selection, draft/record fallback and both contexts;
- full loopback first/update/clear in both contexts, unknown draft replacement,
  default-true inheritance acceptance versus nondefault/unknown pre-save refusal,
  full authored save payload, replacement rather than merge, linkage mismatch
  despite matching PDS/action output, retained source intent and clean-repeat no-op;
- real CLI edit→publish in both contexts, with observation-only uncertainty recovery.

Cover upload remains refused: blob identity/bytes and upload need their own seam.
Scheduling is a separate action, and native publish cancels schedules; nonempty
schedule/reply relations remain blocked with the existing explicit-null evidence
requirements. Email-only posts and test email have separate delivery-only surfaces.
Newsletter email stays forced off; Bluesky sharing is a separate unused action;
there is no quiet/share/test-email control or publication-level preference support.
The accepted native image housekeeping disclosure is unchanged. No cleanup or
automatic retry was added.

## Parent review and road ahead

Review/accept `leafbud-release-preferences` first: the consequential judgments are
whole-object replacement, explicit-clear spelling, nondefault inheritance control
and default hash canonicalization. Ticket closure remains with the parent.

The next possible fresh Sol high batch is **leafbud-release-delivery**, if selected
and commissioned: establish observable newsletter/share effects and uncertainty
boundaries before enabling any sending flag. Alternatively **leafbud-mdx-media**
is an independent fresh Sol high design/build slice for acquisition/upload and
resource identity, prerequisite to lifting cover refusal. They should not be
bundled merely because both touch publishing. Neither adjacent feature is authorized
by this implementation receipt.

Reported context at the completed code/doc-drafting boundary was **221261** tokens;
the final handoff measurement follows below. This session ends at this major feature
boundary, without starting another ticket.

### Final gate and handoff

`cargo fmt --all --check` and strict `cargo clippy --workspace --all-targets -- -D warnings`
pass. The final code gate above is **421/421**, ten opt-in skips. Rekon's explicit
`check --okf` passes for all five owned Markdown documents: **165 links and 88
sources**, including native-session provenance. No live validation was substituted
for the fixture argument contract. Last reported context at this completed gate:
**227329** tokens; handoff/rotation follows this major feature boundary.
