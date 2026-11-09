---
type: Review
title: "Authoring foundation review and connected-watch join"
description: "Parent architectural review of batch A: accepted source choices, concrete identity/preparation/settlement defects, a focused correction commission and independent realtime transport work."
resource: /design/authoring-review0.gpt6a.md
status: corrections-commissioned
generated:
  at: 2026-11-09
  by: model:openai/gpt-6-astra#high
sources:
  - resource: "opencode://ses_e4217c526ffeXeHByo5ZswnB3F"
    title: "Parent reviewed batch A's actual model/parser/patch/status code and commissioned focused fixes before the connected-watch integration join"
  - resource: /design/author-create-keys0.glm53m.md
    title: "Batch A implementation receipt and author's judgment calls"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Accepted architecture and sequencing; complete drafting remains the eventual save delivery"
  - resource: /crates/document/model/src/lib.rs
    title: "Reviewed editor_value new-object associations and semantic_value tracking inclusion at 6aee3dd9"
  - resource: /crates/document/mdx/src/parse.rs
    title: "Reviewed source traversal versus page-grouped output, provisional allocation and explicit key registration"
  - resource: /crates/document/mdx/src/keys.rs
    title: "Reviewed positional preparation, collision checks and caller-input panic paths"
  - resource: /crates/document/mdx/src/patch.rs
    title: "Reviewed tracking-only comparison and envelope insertion assumptions"
  - resource: /crates/document/mdx/src/view.rs
    title: "Reviewed new-page comparison rendering"
  - resource: /crates/workflow/src/create.rs
    title: "Reviewed offline identity/publication creation and no-clobber path"
  - resource: /crates/workflow/src/editor_state.rs
    title: "Reviewed local UUID independence, typed association reconciliation and adoption"
  - resource: /crates/workflow/src/status.rs
    title: "Reviewed unreachable no-receipt branch and early-return duplicate handling"
  - resource: /crates/workflow/src/tidy.rs
    title: "Reviewed preparation/normalization composition and source byte guard"
  - resource: /experiments/15-live-connection/live-connection0.gpt61s.md
    title: "Accepted transport evidence and interface handoff for independent web implementation"
  - resource: /design/live-transport0.gpt61s.md
    title: "Reviewed production transport checkpoint, loopback results and passive deployed adapter verification"
  - resource: /crates/leaflet/web/src/live.rs
    title: "Reviewed actual driver, latest-state root sequences, selection generations and socket/topic lifecycle"
  - resource: /crates/leaflet/web/src/live/bootstrap.rs
    title: "Reviewed origin-scoped cookie-free discovery and explicit public project configuration"
  - resource: /crates/leaflet/web/src/live/presence.rs
    title: "Reviewed transactional bounded presence reduction and unknown-on-gap semantics"
---

# Authoring foundation review and connected-watch join

## Disposition

Batch A has the right architecture and useful offline functionality. The reviewed
state is committed through `6aee3dd9`. The implementer reported a passing workspace
suite, formatting, clippy and a create/check/tidy smoke; those checks are trusted,
not automatically rerun by the parent. The final chat reported 173 tests while
the durable receipt says 175; the correction handoff should report its actual
final check result rather than propagate that discrepancy.

**Integration acceptance is conditional on the corrections below.** These are
contract gaps in the new boundaries, not a request to replace the architecture.
They were identified by reading code; the correction agent must turn them into
executable regression cases before closing its pass.

## Choices accepted

- Format 2 and rebuildable local data keep the implementation direct.
- Unkeyed authored lists become new per-item blocks, just as unkeyed paragraphs
  become new blocks. Requiring users to key the entire structure by hand would
  defeat the authoring goal.
- Main-page identity belongs in the tool envelope, independent of the heading.
- Typed block/page occurrences make the native identity boundary clearer.
- Tracking-only settlement and remote-content adoption deserve distinct APIs.
- Bound observations retain the locally assigned document UUID rather than
  demanding import's token-derived UUID.
- The comparison view may render new-keyed source content. Display equality
  continues to be separate from identity/receipt equality.
- Explicit list-number override support remains a separately identified content
  extension. This review does not commission a nested-list or numbering engine.

## Corrections required before adoption integration

| Boundary | Reviewed failure | Required property / regression case |
| --- | --- | --- |
| Spans and object order | Parser records spans/keyed flags in source traversal order, then groups main blocks before named pages in the returned document | Interleave main A, named ABOUT, main B; key preparation must attach each proposed ID to the correct block with mixed keyed/unkeyed inputs |
| Tracking settlement | `patch_tracking` compares `semantic_value`, which includes the node table it must update | First bind with actual node promotion and later retirement/receipt updates succeed while author metadata and body bytes remain exact |
| Binding continuity | Tracking patch checks only equal token when both bindings exist | Preserve the established origin/token/root/entity-set/entry/publication tuple; reject dropping it; allow deliberate first binding |
| New graph identity | Every absent node becomes the same `{"kind":"new"}`; page-link targets therefore collapse | Retargeting a link between two distinct new pages must change its editor projection; confirmed-key renames remain spelling-independent |
| Provisional/assigned names | Provisional names avoid only identities already traversed; preparation checks only some block keys | Reserve later explicit keys, page keys and confirmed/retired table keys; an unkeyed object must never borrow a confirmed association |
| Preparation input | Applying caller-visible plans uses unchecked range replacement and `expect` on the result | A stale/mismatched input or invalid proposal returns a useful error, never panic or a silently valid edit to a different source |
| Envelope shape | Main-page insertion infers block style from the next line and assumes two-space indentation | Valid flow mapping followed by a blank line and four-space block mapping both prepare correctly; normalize only the owned envelope when needed |
| Status states | Receipt-profile validation precedes missing-receipt handling; unbound returns precede duplicate checks | Absent/null receipts reach `bound-no-receipt`; duplicate unbound UUIDs remain explicit; unknown profiles still fail honestly |

The simplest new-object identity rule is explicit source identity for prepared-new
objects and native locators for confirmed ones. Deterministic provisional labels
can support read-only diagnostics, but they cannot alias confirmed rows or erase
graph connectivity. There is no need for a graph-canonicalization algorithm.

For source preparation, source coordinates remain transient addressing data, not
durable identities. Preserve their correspondence to the parsed object even when
the model groups page definitions differently from their textual positions.

For settlement, compare author-owned fields and body independently of allowed
tracking changes. A semantic check that includes bookkeeping can be stricter yet
wrong: it rejects the exact verified update that the API exists to perform.

## Commissions and join

The first g session finished around 399k reported tokens. Rotate the focused
correction into fresh **g**, `opencode://ses_e419a7411ffe4oaxpvanZ9R4hO`, retaining
the committed predecessor and landing coherent correction commits. It owns
model/MDX/status and regression tests, with updates to the existing author tickets.

In parallel, fresh **Sol high**, `opencode://ses_e41995c2fffesHpONkYRgWfiZq`, owns
the independent production `web::live` transport checkpoint for
`leafbud-editor-follow`: validated bootstrap, managed socket, joins/reconnect,
root invalidation sequences and passive advisory presence. It owns transport
dependencies/version coordination. Its current scope does not overlap the
model/MDX/status corrections.

The parent joins both results before continuing Sol into shared refresh/locking,
workspace selection, watcher scheduling and CLI integration. The transport-only
checkpoint does not close connected watch. No new offline list/metadata/rekey
batch is inserted ahead of the user's commissioned live-sync path.

### Transport checkpoint reviewed

Sol returned the production adapter through `224f08f8`, with implementation
checkpoints recorded in [its receipt](live-transport0.gpt61s.md). Parent read the
actual live driver, bootstrap and presence reducer and accepts that transport
boundary for integration. Reported scoped evidence is 27 passing web tests,
strict clippy/formatting, and a successful passive production-adapter session
including deselect/reselect and the heartbeat window. The point-in-time broad
run encountered two tests still being changed by the parallel foundation pass;
the final integrated check belongs after that pass returns.

In particular, the incoming state uses session-wide monotonic stamps across root
deselect/reselect, presence diffs are transactional, and the local driver remains
polled during joins/recovery. The controller must discard serviced stamps when
starting a new transport run, gate readiness on acknowledged joins, and service
only the sequence captured at refresh start. These are the receiving obligations,
not reasons to replace the transport with another implementation.

The transport session finished around 201k reported tokens. Connected-watch
integration is a separate coherent checkpoint; use a fresh Sol session with these
receipts if the remaining controller/locking scope would exceed the operator's
session target. The foundation correction still owns its files until return.

The [build path](keyed-build-path0.gpt6a.md) remains the current direction; the
[batch A receipt](author-create-keys0.glm53m.md) preserves the implementation
account; [experiment 15](../experiments/15-live-connection/live-connection0.gpt61s.md)
grounds the transport work. This review records the required join conditions.
