---
type: Design
title: "Proposed YAML frontmatter for a keyed MDX document"
description: "Concrete author/tool field separation, exclusive lifecycle, durable node bindings, compact synchronization witnesses and alternatives for the single-file authoring format."
resource: /design/frontmatter0.gpt61s.md
status: proposal
generated:
  at: 2026-11-08
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e4900b8d7ffeDT9QuFwmKRAB7t"
    title: "User's single-file/identity/sync requirements and request for a concrete YAML design with subsection alternatives"
  - resource: /design/authoring-handoff0.gpt61s.md
    title: "Original metadata sketch plus controlling keyed bidirectional addendum and CLI design arena"
  - resource: /design/authoring-recommendation0.gpt61s.md
    title: "Satteri recommendation, MDX storage and native publication boundaries"
  - resource: /experiments/13-satteri-authoring/src/model.rs
    title: "Actual limited camelCase metadata model and source identities, not the proposed complete schema"
  - resource: /experiments/13-satteri-authoring/src/author.rs
    title: "Actual YAML-node extraction and serde-saphyr parsing used by the experiment"
  - resource: /experiments/13-satteri-authoring/Cargo.toml
    title: "Executed serde-saphyr 2.1.1 and parser dependencies"
  - resource: /experiments/14-satteri-authoring-attributes/REPORT.md
    title: "Body author keys, source preservation and the explicit brace adapter"
  - resource: /design/yrs-conversion0.gpt6a.md
    title: "Entity/membership/text identity, transient CRDT state and resource representation obligations"
  - resource: /design/records-canonical2.glm53m.md
    title: "Existing global/workspace configuration layering, distinct from per-document frontmatter"
  - resource: "opencode://ses_e46711846ffeWAA4snDkH28tpL"
    title: "Linked accepted interface refinements for scoped hashes, presence semantics, placement identity and YAML normalization"
---

# Proposed YAML frontmatter

## Recommendation in brief

**Subsequent acceptance:** the [authoring interface decision](authoring-interface0.gpt6a.md)
accepts this overall envelope with refinements to hash scopes, missing/null
semantics, binding validation and operation-specific preservation. Explicit
normalization lives under `tidy`, all applicable passes by default; `tidy --keys`
assigns missing identities alone. Read that decision before freezing the schema;
relationship fixtures and rich-content mappings still supply exact field details.

**Use ordinary YAML frontmatter: author-facing document fields at the top level,
one quoted `"@leafspring"` mapping for tool-maintained state, and durable block
identifiers attached to the actual Markdown/MDX content.**

The file stores one current document. Metadata holds identities and compact
witnesses, not another version of its body or serialized Yjs history.

This is a concrete proposed destination, not implemented schema validation.
Field names and policies below are recommendations for review. Experiment 13
already parses YAML through Satteri and `serde-saphyr`, but its small metadata
struct does **not** implement the full schema proposed here. No live writes,
production changes, dependency additions or new parser executions were made.

## 1. A new file should remain small

```yaml
---
format: leafspring-mdx/1
title: An essay
description: Notes from the garden
tags: [gardening, agents]
---
```

No publication date, public path, token or invented record URI is required to
begin writing. A never-managed file can omit `@leafspring`. Local prepare/create
can add its identity and draft state; remote fields appear only after binding.

**Alternative:** put author fields under `document:`. That makes ownership
visually explicit but adds indentation to the fields people edit most often.
**Lean:** flat author fields, one reserved state envelope. Do not introduce both
spellings or duplicate title/tags under the envelope.

## 2. An example after binding and publication

The following is **illustrative YAML**, not usable remote credentials or a
fixture asserting real CIDs/hashes. Angle-bracket values are placeholders.
Optional state sections need not all be present.

```yaml
---
format: leafspring-mdx/1
title: An essay
description: Notes from the garden
tags: [gardening, agents]
publication: "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/example-pub"
path: /example-post
publishedAt: "2026-11-08T10:00:00Z"

"@leafspring":
  id: "11111111-1111-4111-8111-111111111111"
  state: published

  binding:
    baseUrl: "https://leaflet.pub"
    token: "22222222-2222-4222-8222-222222222222"
    root: "33333333-3333-4333-8333-333333333333"
    entitySet: "44444444-4444-4444-8444-444444444444"
    entryFact: "55555555-5555-4555-8555-555555555555"
    publication: "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.publication/example-pub"
    uri: "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/site.standard.document/example-post"

  nodes:
    introduction:
      entity: "66666666-6666-4666-8666-666666666666"
      memberships: ["77777777-7777-4777-8777-777777777777"]
      textFact: "88888888-8888-4888-8888-888888888888"

  sync:
    profile: leaflet-working/1
    localHash: "sha256:<accepted-logical-document-digest>"
    remoteHash: "sha256:<accepted-editor-witness-digest>"
    acceptedAt: "2026-11-08T10:00:01Z"

  released:
    inputHash: "sha256:<last-confirmed-publication-input-digest>"
    cid: "<actual-published-record-cid>"
    path: /example-post
    publishedAt: "2026-11-08T10:00:00Z"
    confirmedAt: "2026-11-08T10:00:02Z"

  names:
    file: drafts/essay.mdx
    path: /example-post
    acceptedAt: "2026-11-08T10:00:02Z"
---
```

The corresponding body still carries its identity, for example
`## Introduction {#introduction}`. Frontmatter does not replace that association.
`drafts/essay.mdx` can hold a published document: directory and filename are not
the lifecycle authority. `names.file` is a previous accepted observation, not a
command to move the file back.

**Why quote `"@leafspring"`?** A leading `@` is not valid at the start of an
unquoted YAML plain scalar. The chosen field name is fine as a quoted mapping key.

## 3. Author fields: current document, not protocol-envelope copies

Use camelCase for multiword fields, matching existing `publishedAt` and the
experimental serde model. Runtime configuration remains its existing TOML format;
there is no requirement to rename config keys to match document YAML.

| Field | Proposed meaning | Missing/clearing behavior |
| --- | --- | --- |
| `format` | Whole local format/schema identifier, initially `leafspring-mdx/1` | Required for a managed file; unknown major blocks managed writes |
| `title` | Document title, separate from body headings | Required nonempty text |
| `description` | Current document description | Missing or null means no description; empty string can normalize to the same clear intent |
| `tags` | Current ordered tag list | Missing means `[]`; use `[]` to clear explicitly; null is a type error |
| `publication` | Intended publication AT URI, not the post's URI | Missing inherits a default only while unbound; null explicitly selects standalone publication |
| `path` | Canonical local document path/intent, not identity or filename | Missing/null leaves it unassigned or retains the established remote route until deliberately reconciled |
| `publishedAt` | Author-selected publication date | Missing/null preserves the confirmed existing date, or chooses now for a first release; `--now` is an explicit edit |
| `cover`, `theme` | Current document-level resource/style intent | Reserve typed subsections; exact schemas require content mapping, not arbitrary copied protocol objects |
| `delivery` | Publication preferences such as explicit email intent | Resolve at publication planning; no effect on an editor-only save |

These rules distinguish **declarative content** from **binding/default policy**.
For implemented source-owned content fields, removing a description or tags
means clearing it, not retaining stale remote content forever. Removing a
publication/date/path field is not permission to move identity or reset dates.
Only implemented/declared ownership scopes may be cleared.

For an unbound file, recommend explicit command input > document intent >
workspace > global defaults. After binding, actual target identity wins over
ambient defaults. Changing `publication` to a different target, including null,
is a retargeting request; ordinary push/publish must report that distinction,
not silently create another post. Preserve the established workspace-over-global
rule from [configuration](records-canonical2.glm53m.md).

Native publication currently chooses the first post key and normalizes the path
to `/rkey`. Custom publication paths can route, but an ordinary native setter
has not been verified. The publication plan must expose that constraint. When
the user accepts normalization, record the achieved path/date deliberately;
never infer a different post URI from an edited `path`.

**Alternatives:** mirror raw record `site` and every public field verbatim, or
nest all document intent under a typed `document:` object. Raw mirroring makes
interchange simple but poorly represents unpublished/editor-only content and
confuses publication target with owner/identity. **Lean:** authoring fields and
typed current content, with an explicit public-record projection at export.

For `delivery`, a small explicit mapping is preferable to many boolean CLI-only
switches, but actual names/defaults remain open. Do not enable subscriber email
or repeat delivery merely because an unset boolean inherited an upstream default.
Bluesky sharing is a separate native action, not proof of publication success.

## 4. Identity and exclusive lifecycle

`@leafspring.id` is the local document UUID, independent of names and remote
keys. The tool maintains it; an ordinary filename change keeps it.

`state` is exactly `draft` or `published`:

- A fresh draft has no binding, or a binding with an editor locator and no URI.
- A published document requires its actual `binding.uri`.
- The same published binding may retain its Leaflet token/root so we can edit
  through the native lifecycle. That is not a second draft state.
- An imported post may have a URI but no discoverable writable editor locator.
  It is still published, with editing unavailable until a legitimate binding is
  established. Do not mint another Leaflet or switch to PDS implicitly.

Validate the YAML into an exclusive typed lifecycle; `state` and binding fields
are not independently meaningful unchecked flags. For example, draft plus a
published URI is contradictory, not a clever way to track both.

**Alternative:** a tagged `binding.kind` of `unlinked-draft`, `leaflet-draft` or
`leaflet-post`, as in the handoff sketch. It encodes invariants conveniently but
mixes lifecycle with linkage. **Lean:** readable `state` plus one validated
binding mapping; no simultaneous `draft:` and `published:` management objects.

Keep only useful target facts: base URL, token, root, entity set, selected entry,
confirmed publication association and actual document URI. `binding.publication`
records the verified target, whereas top-level `publication` is current author
intent; a difference is an explicit retargeting request. Confirmed standalone
association uses null. Missing association means unknown/unestablished, not
standalone; inspect it before a write rather than inheriting a new global target.
Do not store separately editable copies of DID/collection/rkey when they are
already recoverable from the URI. The token is an accepted local-file locator;
account credentials remain in the configured auth store.

## 5. Node associations: IDs with the content, locators in metadata

`nodes` maps a durable source identity to its remote association. Its key comes
from a body annotation/component, generated when the author supplied none:

```mdx
<Key id="b-90a52472-f6aa-4ee1-ac39-e833f419ca74" />
An ordinary paragraph.

## Introduction {#introduction}
```

Both keys have the same role. Human-readable spelling is optional. Assign IDs
once before establishing the remote association, not again on each parse/save.
An explicit rekey operation preserves the remote binding; copying a keyed block
requires a new source identity. Do not silently interpret duplicate IDs as moves.

Each row stores the entity and relevant fact identities, not its old content.
`memberships` is a sequence because references and placements can be independent
objects; avoid schema variants for one versus many. The planner still needs a
defined identity for each independently editable reference occurrence. Pages,
interactive objects and generated list structure require the combined proof;
not every parser node gets a remote entity.

The optional `textFact` identifies the rich-text fact. It does not supply Yjs
client clocks, tombstones or XML content. Pull current Yjs state into Yrs when
editing that same object; retain surviving internal identities through real
edits. Do not regenerate text under a reused numeric client ID.

**Alternatives:** use remote UUIDs directly in body keys; keep all identities in
frontmatter positions/pointers; or split machine ID from human alias on every
node. Direct remote IDs couple local authoring to an account and do not exist
before first binding. Position-only maps fail on move-plus-rewrite. Separate
aliases could improve renaming but add a second body identity field/schema.
**Lean:** one stable source key with a mapping, machine-generated or friendly;
keep alias syntax as an explicit future choice if it earns its complexity.

Avoid a per-character ledger. If a meaningful inline object requires its own
association, add a compact locator for that object, not a serialized CRDT tree.

## 6. Sync witnesses: agreement, not another document

Start with one document-level `sync` pair:

- `localHash`: accepted logical current-document fingerprint.
- `remoteHash`: corresponding accepted Leaflet editor/fact witness.
- `profile`: defines projection/normalization semantics.
- `acceptedAt`: when that correspondence was confirmed, not who wins later.

The hashes have different inputs; they are not expected to equal one another.
They let us detect local-only, remote-only and both-changed state. Read-only
status does not advance them. Successful push/pull or explicitly confirmed
current semantic convergence can do so after verification.

Logical hashes exclude the entire tool envelope, parser spans and formatting
trivia. Include meaningful author fields, durable content IDs, order, references
and resource identity/content. A conservative remote witness includes meaningful
opaque state too, without treating transport cookies as content. Exact digest
serialization is versioned by the profile and needs executable fixtures.

**Alternative:** per-node accepted hash pairs, as suggested in the addendum.
They support scoped conflict choices but repeat data and acceptance bookkeeping.
**Lean:** keep per-node identity bindings from the outset; begin with document
agreement unless the combined experiment needs independently accepted blocks.
Then add a node's small `sync` pair deliberately. Do not store both a full base
document and per-node copies; neither is proposed here.

With hashes only, diff reads today's local and remote contents. We cannot
reconstruct historical edits or automatically merge an unavailable ancestor.
This is sufficient for guarded push/pull and explicit current-endpoint choices.

## 7. Publication receipt is not the editor agreement

`released` records the last confirmed public result: publication input hash,
actual output CID/path/date and observation time. URI remains in the binding,
not repeated as another identity. Native projection may upload/offload or
normalize content; output CID is not an editor-content hash.

The author's `publishedAt`/`path` and receipt values can differ after a local
edit. That is useful scalar witness data, not two document copies: one is current
intent, the other the last confirmed public result. A confirmed first publication
can adopt the achieved path/date into author fields after disclosed normalization.

Editor save advances `sync`, not `released`. Publication advances `released`
only after its output is verified. A published file with unreleased edits remains
published. If email/share delivery cannot be verified, report that uncertainty;
do not infer it from record presence or blindly repeat a delivery operation.

**Alternative:** one big E/A/P agreement tuple for editor, application and PDS.
That is useful remote observation evidence but makes editor save easy to confuse
with public release. **Lean:** separate compact editor agreement and publication
receipt; show component disagreements in status/planning without storing bodies.

## 8. Filename/path witnesses are optional and independent

`names` stores an accepted `(file, path)` pair and time. Store `file` relative
to the workspace root where possible, with its extension; an external file needs
a clearly defined alternative coordinate, not an accidental absolute home path.
The live filesystem location is always discovered from the actual file.

`tidy --from path` renames the file; `--from filename` edits path intent;
`--accept` witnesses a deliberate mismatch. Publication or pulling a new path
must not automatically rename the file or clear name drift. A filename change
does not retarget its remote binding or node IDs.

**Alternative:** always regenerate filenames from path, or omit naming evidence
and report only present inequality. The first violates the user's breakable
convention; the second cannot tell which side changed. **Lean:** optional pair
witness, never a second authority or automatic repair mechanism.

## 9. Assets and pending operations

An optional `assets` map contains resource IDs, source locators, digests and
confirmed remote locations where needed. Prefer source-relative local paths;
the body/component still owns actual current content. No base64 media or old
document payload is required in metadata. Computing a blob identity does not
prove it is uploaded. Unpublished poll definitions belong in current content,
not as fabricated published references under `@leafspring`.

An optional `pending` mapping describes **one unsettled attempt**, not a durable
queue: operation ID/kind/phase, selected input hash, expected witnesses and known
returned IDs. Include a generation seed only for creation/recreation that needs
one. Ordinary identity-preserving text edits do not create a new generation.

On retry, read current state. If source changed, a hash cannot reconstruct the
old payload; do not pretend it can. If an outcome already happened, settle it
locally. A lost draft-creation response remains uncertain. Never persist full
mutation batches, editor snapshots or a historical request-body log in YAML.

**Alternatives:** no pending metadata, or a separate journal. No receipt makes
interrupted remote/local coordination harder to explain; a journal violates the
current one-file preference and expands scope. **Lean:** one bounded pending
receipt, added only where the real transport/recovery implementation uses it.

## 10. YAML profile and writing policy

Use YAML because Satteri already identifies its frontmatter and the experiment
has a working Rust parse path. The proposed profile is deliberately ordinary:
string-keyed mappings, sequences, strings, booleans, null and supported finite
numbers. Reject duplicate keys rather than selecting a winner. Avoid custom
tags, anchors, aliases and merge keys in this initial format; they complicate
field ownership/rewriting without a current authoring need.

Emit dates, IDs, URIs, hashes and the reserved envelope name quoted. A date must
be interpreted through the declared field type and explicit datetime parser,
not accidentally as a host YAML timestamp object. Stable scalar typing needs
tests against the actual production parser. Do not infer completed validation
merely because experiment 13 deserialized its smaller struct.

Recommend **one `format` major** for the body/frontmatter contract; `sync.profile`
versions witness interpretation separately. A second envelope `version` repeats
schema authority unless metadata truly needs an independent release lifecycle.
Unknown author top-level fields are retained and surfaced as non-applied metadata,
not silently sent upstream. Recognized fields with wrong types are errors.
Unknown state fields must survive a metadata update; an unsupported state shape
or future format may be inspected, but must not be rewritten speculatively.

Writing rules:

1. Preserve body bytes except deliberate ID insertion or an explicit pull/edit.
2. Preserve unknown frontmatter values. No-op operations write nothing.
3. Tool updates normally touch only `@leafspring`; explicit metadata commands and
   accepted native normalization may edit the corresponding author fields.
4. An initial serializer may normalize **frontmatter formatting** on a real write,
   but must disclose/test that behavior. Exact preservation of YAML comments and
   styles is desirable, not a reason to require a general-purpose source editor.
5. Validate the new document, recheck source has not changed during remote work,
   then atomically replace the file. Never overwrite a newer local edit merely
   to store a remote success receipt.

**Alternatives:** TOML frontmatter gives stricter scalar rules; JSON frontmatter
matches machine structures but is noisier for everyday fields; a YAML CST editor
could preserve every comment/style. **Lean:** YAML with the existing experiment's
parser candidate, an explicit profile and a bounded writer. Prove preservation
and duplicate/type rules before adopting a production implementation. No new
frontmatter library is required solely by this proposal.

## 11. What is intentionally not settled yet

- Exact `cover`, `theme`, `delivery` and asset descriptor schemas.
- Initial key-insertion command versus confirmed first-save preparation.
- Which nodes/relationships need source IDs in lists, shared references and
  split/join edits; no character-ID encoding is promised.
- Whether the first useful sync implementation needs per-node agreement pairs.
- How much comment/style preservation the YAML writer should provide initially.
- Exact canonical hashing bytes and resource identity rules for each profile.

These should be tested in the combined keyed MDX/Yrs experiment or a small
frontmatter integration slice, not resolved by expanding YAML into a database.

## Connections and handoff

This proposal consolidates the [handoff's storage sketch and bidirectional
addendum](authoring-handoff0.gpt61s.md), with some explicit recommended changes:
`state` plus one binding, one whole-file format version, and initially compact
document agreement rather than mandatory per-node hash pairs. Those are reviewable
choices, not silent claims of prior acceptance.

The [parser recommendation](authoring-recommendation0.gpt61s.md) remains intact.
The [earlier configuration design](records-canonical2.glm53m.md) still owns global
and workspace defaults; frontmatter owns per-document intent and binding. The
[Yrs conversion design](yrs-conversion0.gpt6a.md) supplies text/fact mapping
constraints, not permission to add CRDT history to metadata.

Before implementation, exercise fresh/unbound, linked-draft, published,
editing-unavailable and interrupted-operation examples; duplicate/type errors;
unknown-field preservation; a rename; and no-op/body-byte preservation. The
illustrative expanded YAML above is not a claim those tests already exist.
