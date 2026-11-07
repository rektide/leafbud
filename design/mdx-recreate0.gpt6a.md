---
type: Recommendation
title: "Local MDX → replace the Leaflet body → publish"
description: "A source-authoritative authoring pipeline: fresh generated content under a stable Leaflet/public identity, without CRDT edit compilation or bidirectional synchronization."
resource: /design/mdx-recreate0.gpt6a.md
status: proposal
generated:
  at: 2026-11-07
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e48556aa8ffe9Fzbp0FmthuoDH"
    title: "Astra recommendation following the user's request to simplify to local MDX authorship and recreation; bounded source verification"
  - resource: /experiments/05-yrs-conversion/REPORT.md
    title: "Executed text interoperability and negative proof that unrelated fresh content does not replace an existing Yjs root"
  - resource: /design/yrs-conversion0.gpt6a.md
    title: "Earlier identity-preserving conversion proposal; substantial machinery is unnecessary for this new direction"
  - resource: /design/document-sync1.gpt61s.md
    title: "Prior lifecycle/native publication research and existing leafspring-mdx work reference"
  - resource: /design/document-sync1.gpt6a.md
    title: "Prior native identity, slug and compact-metadata findings"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/mutations.ts"
    title: "Read createEntity/assertFact/retractFact, addBlock, movement and removal semantics"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/replicache/cachedServerMutationContext.ts"
    title: "Read explicit many-fact ID updates, existing-text merge, permission checks and removal side effects"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts"
    title: "Rechecked editor linkage lookup, existing document rkey, root fact traversal and native publication arguments"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts"
    title: "Ordinary authenticated publication strips requested rkey and clamps future dates"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts"
    title: "Rechecked poll entity ID as public poll rkey and native image uploads"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/rpc/%5Bcommand%5D/getFactsForRoots.ts"
    title: "Explicit roots permit inspecting a staged subtree; depth is capped at three"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/supabase/migrations/20240913204647_update_all_facts.sql"
    title: "get_facts follows reference/ordered-reference/spatial-reference reachability from the root"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/components/Pages/index.tsx"
    title: "Editor first page comes from root/page unless an explicit page query is open"
  - resource: "https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/createNewLeaflet.ts"
    title: "Draft creation and redirect token receipt"
---

# Local MDX → replace the Leaflet body → publish

## Recommendation

**Treat the `.mdx` file as the source, and Leaflet's editor body as generated
output. Keep the Leaflet itself; recreate its authored page/block subtree when
the source changes. Then use native publication to release that body.**

This is a build-and-replace workflow, not bidirectional synchronization. We do
not need a character-level CRDT edit compiler, persistent identity for every
paragraph, or an editor-to-MDX importer to deliver it. Yrs becomes a reliable
encoder for fresh text documents and a small read-back verifier.

This proposal responds to the user's new objective: “author locally on a computer
and make it so.” **Source-authoritative replacement of web edits is the proposed
policy, not an already accepted revocation of earlier drift-aware proposals.**
Use explicit replacement commands so that policy is visible without a recurring
conflict-resolution workflow. Existing work reference: **`leafspring-mdx`**.

## 1. One authoritative source

Start with Markdown prose plus literal components for richer content. The dialect
is MDX-shaped data: no imports, runtime JavaScript, component execution or spread
attributes. A small compiler lowers it to a typed AST and then to editor facts.
Neither that AST nor a JSON document is saved alongside the source.

Minimal new file, `essay.mdx`:

````mdx
---
format: springleaf-mdx/1
title: A small essay
description: An example authored locally
tags: [notes]
---

A paragraph with **bold**, *italic*, and [a link](https://example.com).

## A section

- A list item
- Another item

<Image src="./assets/diagram.png" alt="A diagram" width={640} />

<Poll key="next-topic" options={["Gardens", "Maps"]} />
````

Ordinary paragraphs, headings, quotes, lists, links, images, rules and fenced
code lower to their corresponding Leaflet blocks. Do not silently reinterpret
the first heading as metadata: frontmatter owns document title/description/tags;
body headings remain body headings. Document theme, cover, publication target,
preferences and optional `publishedAt` also belong in frontmatter.

Components cover actual additional structure rather than simulating it with text:

````mdx
<Paragraph align="center">
  Exact inline content<HardBreak />and a
  <DidMention did="did:plc:example" text="named person" />.
</Paragraph>

<PageLink page="details" />
<Page name="details" kind="linear">
  More information, defined once and referenced by name.
</Page>

<Canvas name="map" width={800} height={400}>
  <Place x={12.25} y={40.5} width={300} rotation={2.75}>
    <Image src="./assets/map.png" alt="Map" />
  </Place>
</Canvas>
````

Top-level ordinary content defines the main linear page. Named page/canvas
definitions are separate objects; explicit links/embedded-canvas components
place them in the document. The compiler builds a reference table first, then
allocates all generated entities and resolves links. Reject missing/duplicate
names. No author-maintained UUID is needed for ordinary prose.

Specify a deterministic Markdown whitespace policy: a soft line wrap becomes a
space; an explicit Markdown hard break or `<HardBreak />` becomes a break node.
Provide exact escapes such as `<Run text={"  literal\ntext <>{}"} />`, literal
mark sets, and typed component properties for cases that Markdown normalizes.
Braced properties accept JSON literals only. Thus precise geometry, stroke arrays,
HTML/code text, inline mentions/footnotes and theme slots can be represented
without executing a program. Inline names/references are resolved within each
build; flattened publication facets are generated output, not the stored model.

Use frontmatter as authoring data plus a CLI-owned `@springleaf` section. After
binding, the latter contains only:

```text
localId                     stable local document UUID
state                       draft OR published
leaflet                     base URL, token locator, root entity, entity set
entryFact                   the existing root/page membership fact we replace
documentUri                 actual published URI, only after first publication
applied                     semantic input hash + current generation seed
released                    optional last verified publication-input hash/CID
pending                     optional generation/hash/phase for interrupted work
```

Permission-token locator is existing target identity; credentials stay in the
normal authentication store. Asset locators/digests may also be retained compactly.
No old body, facts, Yjs full state or mutation batch is persisted. The CLI reads
the body and only patches its owned metadata region; automatic body rewriting
and general source-preserving MDX editing are unnecessary for this workflow.
Publishing changes the state in that same file; editing a published file does
not turn it into a second draft. Filenames/directories are convenient placement.

## 2. The exact authoring flow

Proposed surface, not existing implemented commands:

```sh
springleaf create essay.mdx --title "A small essay"  # local file only
springleaf check essay.mdx                         # compile/validate; no writes
springleaf save essay.mdx --replace                # create once, then replace body
springleaf publish essay.mdx --replace             # ensure body, then publish/update
```

`publish --replace` includes save when needed; the normal short path is therefore
**write file → publish → edit file → publish again**. `save` provides an optional
web preview without making the content public. Both meanings are the same for
draft and published files. No default creation of an intermediate JSON file.

1. Compile and validate the entire source; resolve local asset paths relative to
   the file and acquire required media before changing the remote entry point.
2. If unbound, create one Leaflet through the existing `/new` mechanism and record
   its returned token/root/rights immediately. This is the only normal whole-
   Leaflet creation. Check authentication before creating, since anonymous `/new`
   can mint an orphan. A lost creation response needs lookup, not blind reminting.
3. Build fresh output as described below; verify it, switch the entry point,
   apply source-owned root metadata/theme fields, and verify the achieved body.
4. For `save`, finish. For `publish`, invoke native publication on **that same
   leaflet ID/root**, with source metadata and explicit delivery preferences.
5. Read back actual linkage and output. On first success record the URI and change
   local state to `published`; on later successes require the same URI. Update
   compact receipts only if the source has not changed during the operation.

Omit `publishedAt` initially to publish now. Later preserve the actual publication
date unless edited; `--now` can explicitly choose a new one. No scheduling
subsystem is needed. Ordinary native publication chooses the first rkey and writes
`path = /rkey`; a friendly `essay.mdx` filename does not promise a custom public
slug. Native email/share choices stay explicit, especially on retries.

The authenticated server-action invocation and ordinary media-upload transport
still need an implementation proof. Native source verifies the semantics, not a
stable externally callable CLI endpoint. No raw-PDS fallback is implied.

## 3. Recreate the body, retain the Leaflet

**Keep:** permission token, root entity, entity set, publication linkage and the
primary `root/page` fact's ID. **Regenerate:** ordinary page/block entities,
their membership facts and their Yjs text documents.

This preserves the identity that matters for native publication. At source
revision `600c7b5f1f7b31b66fb626c293cc104312a12caf`,
`publishLeaflet.ts:188–222` finds the existing document through the leaflet's
publication/standalone linkage; `:276–280` reuses its rkey. A new leaflet each run
would lose that lookup and may create another post. Supplying the old rkey is not
an ordinary-user workaround: `publishToPublication.ts:11–38` strips that argument.

### Proposed minimal switch-over

1. **Observe the current entry fact.** Bind to a dedicated post leaflet's primary
   `root/page` edge, preserving its ID and position. Do not guess among unrelated
   publication navigation pages. Explicit adoption can select the intended edge;
   subsequently it is the recorded replacement boundary.
2. **Stage a fresh, initially disconnected subtree in the same entity set.** Use
   `createEntity` then `assertFact` for page types, block types, text, ordered/spatial
   memberships and internal references. IDs derive from a new generation seed
   plus deterministic compiler paths. Assets must be usable before their facts
   are installed. Avoid helpers whose inferred defaults would change the compiled
   graph; the low-level primitives are sufficient.
3. **Verify staging before exposing it.** `getFactsForRoots` accepts explicit
   roots; its depth limit is three. Supply the planned entity IDs as roots in
   bounded batches, rather than assuming one depth-three page read proves a deeply
   nested tree. Check expected typed facts, links and decoded text. The plan's
   in-memory node inventory provides those IDs; no saved graph is required.
4. **Overwrite the existing entry fact, keeping its identity:**

   ```json
   {
     "name": "assertFact",
     "args": {
       "id": "existing-root-page-fact",
       "entity": "existing-root",
       "attribute": "root/page",
       "data": {
         "type": "ordered-reference",
         "value": "new-main-page",
         "position": "existing-position"
       }
     }
   }
   ```

   This is an ordinary fact update, **not a new atomic-swap API**.
   `cachedServerMutationContext.ts:195–232,311–332` honors explicit IDs for
   cardinality-many facts and upserts the value. The editor's `Pages/index.tsx:24–28`
   reads that reference; native publication walks reachable facts from the root.
   Nested generated pages are reached through their `block/card` references;
   the compiler need not register every one as another primary `root/page` edge.
5. **Set source-owned root fields and metadata, then read back.** Clear removed
   source-owned theme/cover fields by their explicit fact IDs; leave unrelated
   root attributes alone. These fields and external metadata actions are not
   guaranteed to switch atomically with the page reference. Publish only after
   the combined state is verified. Existing open browser tabs pinned to an old
   `?page=` can continue displaying that old page; this is not live-editor sync.

The one-reference switch minimizes partial visibility. Staging can fail without
replacing the old body. The source primitives support this recipe, but **the
whole staged-subtree replacement has not been executed or tested yet**. That is
the next decisive proof, rather than more character-level synchronization work.

Old descendants become detached when their authored membership is replaced.
Do not call `removeBlock` or `deleteEntity` as routine cleanup: the former may
enqueue image cleanup and delete interaction descendants; the latter removes
incoming references. This workflow explicitly retires the selected membership,
not everything formerly reachable. Other root links are preserved; an explicitly
owned obsolete auxiliary membership can be retracted deliberately. No automatic
orphan/entity/blob cleanup. Detached generated data will accumulate until a
separate, deliberate cleanup policy is chosen; it is not a managed local backup.

## 4. Yrs becomes encoding, not synchronization architecture

For each generated text entity, build a fresh Yrs `Doc`, `prosemirror` fragment,
paragraph and inline content; encode a **full v1 initial update**. One text fact
per fresh entity. Ordinary text changes trigger a fresh body generation.

That is correct because no old CRDT exists on the new entity. The experiment
shows why doing the same thing on an existing entity is wrong: Leaflet merges
text updates, so independently generated replacement text adds another paragraph
instead of overwriting the previous one. Fresh entity identity avoids that
problem rather than implementing a CRDT deletion/insertion diff.

Keep the verified Unicode/mark/XML rules and the actual wire encoder. Drop the
LCS edit compiler, retained-character matching, snapshot handling and multiplayer
anchor guarantees from this authoring path. The experiment proves the ordinary
rich-text interoperability subset, not all inline nodes or structural replacement.

## 5. Identity exceptions are semantic, not multiplayer features

The command's concise consequence statement should be:

> Replace the generated body from this file. Old block comments/anchors and
> page-specific links are not carried onto regenerated blocks.

Do not transfer old comment threads by text similarity. Detached entities need
not be deleted, but their annotations are no longer annotations on the active
body. Existing links to old page/block IDs can refer to retired content. The
document URI and native document-level linkage remain stable.

Most IDs can be regenerated. A few represent persistent public objects:

- **Polls:** `publishHelpers.ts:61–79` uses the poll block's entity ID as its
  public record key. Require a stable author key, e.g. `<Poll key="next-topic">`,
  and derive its entity ID from `(localId, kind, key)`, independent of generation.
  Reuse the unchanged poll entity from the staged graph. Changing option meanings
  must deliberately create a new poll/key rather than silently reinterpret votes;
  ordinary publication updates keep the old poll identity when appropriate.
- **Questions/reply blocks:** their published key is their entity ID. Use the same
  small named-key mechanism where interaction continuity is wanted. Repoint body
  membership to that stable entity; do not recreate/delete its answer records.
  Changes to its simple configuration are scoped writes, potentially visible
  before the page switch—not an atomic replacement promise.
- **Internal references:** resolve named page/footnote targets and all incoming
  references together within the new generation. No stable historical ID is
  necessary just to make the new document's own links correct.
- **Media:** reuse already available source URLs/content-addressed resources;
  do not reupload unchanged bytes merely because a block is new. New local assets
  need the editor's real upload path. Avoid deletion APIs that enqueue cleanup
  of resources still used by the replacement.

This is not a per-paragraph identity ledger. Named keys are needed only when the
author is referring to a durable semantic object, not to preserve live editing.

## 6. Repeatability and ordinary overwrite policy

A semantic input hash covers normalized AST, source-owned metadata, compiler
profile and referenced asset content digests. Formatting-only changes do not
force regeneration; meaningful whitespace and literal data do. A new generation
seed allocates fresh internal IDs on a real replacement.

Repeating `save --replace` must not append or mint another Leaflet. A source hash
alone cannot prove the remote body still matches: someone could have edited it
on the web. On the unchanged-input fast path, use the recorded generation to
reconstruct expected IDs and verify the active output before skipping. This is
a comparison against today's compiled source, not a reverse importer or a
historical merge. Alternatively the first implementation may simply recreate
on every explicit save; internal IDs change, but visible content and public
document identity still do not duplicate. Prefer verified no-op skipping once
the small read-back verifier is available.

Under the **proposed** policy, web edits inside the generated body are overwritten
by the next explicit replacement. A preview can report that fact; no automatic
three-way merge or conflict-resolution command family is necessary. The source
does not own other publication/navigation content outside its binding. Concurrent
writers can still race the reference or root fields; report failed read-back
rather than claiming atomic overwrite or retrying indefinitely against a writer.

For interrupted attempts, compact `pending` generation/hash/phase is sufficient
to locate staged entities and the current entry point. Reuse a pending generation
only when semantic source/compiler inputs still match. **Read existing staged
text before resuming:** if it already equals the intended content, leave its fact
untouched; if absent, initialize it; if corrupt or different, start a new generation.
Even an independently encoded copy of the same words must not be asserted over
an existing text fact. Exact in-memory batch retries remain safe; cross-process
recovery does not assume byte-identical Yrs re-encoding. Re-read and repair/replan
with fresh transport mutation IDs where necessary: the server can
acknowledge failed mutations, so replaying their old IDs is not a repair. If the
source changed, start a new generation; cleanup of abandoned staging remains
deliberate. A lost publish response is resolved by inspecting existing linkage
before repeating publication or delivery effects.

## 7. What still needs building—and what does not

The implementation is a **one-way compiler, staging writer, entry switch and
read-back check**, followed by the native publish adapter. Existing transport,
fractional ordering, record mapping and Yrs evidence can be reused.

The first useful implementation can cover prose, headings, lists, ordinary marks,
links, breaks, code and already-supported components, while reporting an exact
error for anything not yet compiled **before staging or replacing anything**.
Current-type completeness remains the target: all block/page kinds, canvases and
groups, drawings, galleries/media, rich inline atoms, themes, and unpublished poll
definitions need their forward mappings. It is unnecessary to first solve how
to reconstruct arbitrary web editor content as beautiful MDX. Unknown authoring
components are compilation errors; they are not silently omitted.

Native publication itself omits some editor-only blocks (`datetime`, `rsvp`,
`mailbox`) and rounds some geometry. Saving them into the editor can be complete
without claiming native publication preserves them. The publish plan should name
these actual native transformations. Source remains complete and authoritative.

Specifically unnecessary for this route:

- Saved bases, shadow JSON, CRDT history or serialized pending batches.
- Stable IDs and pointer-remapping machinery for every prose block.
- Character-level CRDT patches, concurrent-intent reconciliation or automatic
  local/remote merging.
- Full editor→MDX import or bidirectional round-trip conversion as a prerequisite.
- A general MDX source editor/formatter, JSX runtime or render-to-HTML pipeline.

### One next proof, then implementation

Extend offline evidence with a small compiler/staging scenario: compile an MDX
file with prose, a hard break, a nested page and one named interactive object;
construct fresh text updates and detached facts; simulate explicit-ID fact writes;
verify staging; change the existing `root/page` value; verify only the new body is
reachable for publication. Repeat, inject a failed stage and a lost switch
response, and prove the token/root/entry fact/public linkage remain unchanged.
This tests the actual proposed replacement boundary. It is recommended next work,
not commissioned or executed in this document.

The decisive external mechanisms are **staged-subtree reads and entry-point
replacement on the deployed Leaflet, ordinary asset upload, and authenticated
native publish invocation**. Source supports the design; none is promoted here
to a successful live-write claim. These are bounded transport/recreation checks,
not a reason to rebuild the discarded synchronization architecture.

## Relation to prior work

[The Yrs experiment](../experiments/05-yrs-conversion/REPORT.md) supplies the wire
foundation. [The previous conversion design](yrs-conversion0.gpt6a.md) remains
useful for complete block/inline mappings, but its fine-grained reconciliation
is optional future work for a different product goal. The two round-one designs
([Sol](document-sync1.gpt61s.md), [Astra](document-sync1.gpt6a.md)) preserve native
lifecycle research; this recommendation selects a simpler proposed save/release
flow for the new local-authoring objective. It belongs with `leafspring-mdx`.
