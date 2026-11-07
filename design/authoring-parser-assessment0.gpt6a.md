---
type: Assessment
title: "Authoring parser selection after four completed experiments"
description: "Comparative source-and-test review recommending native MDX structure with comment identities, a bounded markdown-rs literal-callback proof, and explicit code-ownership and dependency gates."
resource: /design/authoring-parser-assessment0.gpt6a.md
status: proposal
generated:
  at: 2026-11-07
  by: model:openai/gpt-6-astra#medium
sources:
  - resource: "opencode://ses_e48053fe8ffe3NnxwMn7hPx6TK"
    title: "Independent comparative assessment after all four experiments completed; inspected reports, implementation, tests and selected pinned upstream source"
  - resource: /design/mdx-recreate0.gpt6a.md
    title: "Single-source recreation and native publication goal, read with the commissioning correction that ordinary prose needs no wrappers and keys are optional"
  - resource: /experiments/06-rushdown-identity/REPORT.md
    title: "Completed Rushdown evidence, report checkpoint 53866ffd; implementation/control checkpoints through 9bec2a7e"
  - resource: /experiments/07-satteri-identity/REPORT.md
    title: "Completed Satteri evidence at f4804f03, including silent multiline-code misbinding"
  - resource: /experiments/08-pulldown-cmark-identity/REPORT.md
    title: "Completed pulldown evidence through bc8334da, including strict-rejection and tight-list controls"
  - resource: /experiments/09-mdxjs-identity/REPORT.md
    title: "Completed mdxjs evidence through c9b11499, including stock interruption and guarded adapter"
  - resource: /experiments/06-rushdown-identity/src/extension.rs
    title: "Read actual parser registrations, marker recognition and binding"
  - resource: /experiments/07-satteri-identity/src/lib.rs
    title: "Read native parse path, unguarded sibling binding, component gate and duplicate rejection"
  - resource: /experiments/08-pulldown-cmark-identity/src/lib.rs
    title: "Read offset-event tree, derived tight paragraphs, identity recognition and rejection-only strict mode"
  - resource: /experiments/09-mdxjs-identity/src/lib.rs
    title: "Read public parsing, blank-boundary guard, typed projection and component lowering"
  - resource: /experiments/09-mdxjs-identity/src/literal.rs
    title: "Read strict JSON visitor, recursive duplicate-key rejection and end-of-input check"
  - resource: /Cargo.toml
    title: "Current production Serde requirements differ from mdxjs experiment's isolated old pins"
  - resource: "https://github.com/wooorm/mdxjs-rs/blob/282729e6a0750c19dfd9217bc511971bf3295a29/src/lib.rs"
    title: "Directly inspected public mdast_util_from_mdx and mandatory expression/ESM callback wiring"
  - resource: "https://github.com/wooorm/mdxjs-rs/blob/282729e6a0750c19dfd9217bc511971bf3295a29/Cargo.toml"
    title: "Directly inspected mandatory SWC parsing/codegen/visit dependencies"
  - resource: "https://github.com/wooorm/markdown-rs/blob/1506572f9b406431402928f3a8b3df0b4ae2d8f5/src/configuration.rs"
    title: "Directly inspected public expression callback field and brace-only configuration documentation"
  - resource: "https://github.com/wooorm/markdown-rs/blob/1506572f9b406431402928f3a8b3df0b4ae2d8f5/src/util/mdx.rs"
    title: "Directly inspected ExpressionKind and incremental Ok/Eof/Error callback contract"
  - resource: "https://github.com/wooorm/markdown-rs/blob/1506572f9b406431402928f3a8b3df0b4ae2d8f5/Cargo.toml"
    title: "Directly inspected small normal dependency set; SWC is a dev dependency here"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/src/firstpass.rs"
    title: "Directly inspected mdx_block_interrupts and paragraph-interruption integration"
  - resource: "https://github.com/bruits/satteri/blob/7d1297b7e4551a1c5124fa07bd2d1d05f66236e6/crates/satteri-pulldown-cmark/Cargo.toml"
    title: "Directly inspected MDX feature's OXC dependencies and parser/arena/AST coupling"
---

# Authoring parser selection after four completed experiments

## Executive recommendation

**Choose the native MDX tree architecture demonstrated by experiment 09, with
preceding comment keys and literal-data components. Make `markdown-rs`'s public
parse-only interface the next bounded substrate proof, rather than adopting
`mdxjs` 1.0.4's compiler dependency stack in production unchanged.**

This distinguishes a demonstrated choice from a proposed refinement:

- **Best demonstrated end-to-end parser/adapter baseline: mdxjs experiment 09.**
  It already combines ordinary headings/paragraphs, scoped optional keys, nested
  JSX, exact string data and rejection of nonliteral properties. Its adapter
  rejects the known code-span identity trap. It is the reference implementation
  for the next proof, not a production-ready compiler.
- **Preferred production candidate: underlying `markdown` 1.0.0 with an explicit
  literal/comment callback.** That interface exists and feeds the same mdast
  family, but equivalent data-boundary behavior without SWC is **not yet proved**.
  Do one small proof before selecting this dependency for production.
- **Strongest alternative: Satteri's direct parser/arena API**, after adding and
  proving a key-boundary rejection rule. It already parses semantic JSX with
  current Serde, but its present author adapter silently misbinds code-shaped
  input. Its smaller literal-policy gaps are ours to fix, not parser defects.

The deciding weight is **prose plus rich structured data in one source**, not
which library can most elegantly attach an ID. Rushdown wins the demonstrated
grammar-extension comparison; pulldown supplies useful offset events. Neither
has demonstrated the component grammar that this actual product needs. A light
preference for `{#id}` does not justify owning that missing grammar first.

I would change the choice if the markdown-rs callback needs substantial lexer
repair, if guarded Satteri proves materially cleaner integration, or if the user
requires unchanged CommonMark code/brace semantics rather than an explicit
MDX-shaped dialect. That last decision would favor a Rushdown component proof.
No performance ranking or general upstream maintenance-quality ranking is inferred.

## Goal and decision boundary

The [recreation proposal](mdx-recreate0.gpt6a.md) gives the relevant pipeline:
canonical local source → transient typed authoring document → generated Leaflet
content under the existing binding → native publication. Selected blocks may
have continuity keys; normal prose needs neither keys nor `<Paragraph>`/`<Header>`
wrappers. Keys are author identities, not HTML anchors, list indices or Yjs IDs.
Their eventual allocation/continuity semantics are downstream work.

Complete current Leaflet representation remains the destination: named pages,
Canvas/Place nesting, media and precise text/marks need structured declarations.
Literal JSON property values are useful syntax inside this source; they do not
make a saved JSON document authoritative. No sidecar, saved base, historical body
or multiplayer editing architecture is needed to use any of these parsers.

The CLI primarily reads the body and patches owned metadata. Retaining its input
bytes is sufficient for body no-op preservation. Missing JSX attribute spans
affect diagnostic precision; they are not grounds to reject an otherwise useful
compiler substrate for lack of an unrequested body rewriter.

## Comparable matrix

All entries refer to the pinned experiments, not every capability of a project's
latest release. “Native” means parser behavior; key and data policy remain ours.

| Concern | Rushdown 0.18.0 / 06 | Satteri parser 0.6.7 / 07 | pulldown-cmark 0.13.6 / 08 | mdxjs 1.0.4 + markdown 1.0.0 / 09 |
| --- | --- | --- | --- | --- |
| `{#id}` | Custom inline heading suffix and final paragraph-line recognizers; works in chosen dialect | Native Markdown attributes; heading works with MDX flags, paragraph tail becomes rejected MDX expression | Implemented on plain offset events; native heading attrs deliberately off | Exact examples parse as expressions, not keys; some otherwise legal key punctuation fails SWC |
| Comment keys | Exact MDX-shaped spelling via custom block parser; not native MDX | Native flow expression + sibling binder; currently unsafe interruption case | Not implemented; reserved spelling diagnoses | Native flow expression + sibling binder with blank-before guard |
| Prose/code ownership | Tested multiline/inline/fenced/indented literals preserved; custom block parser cannot interrupt prose | Ordinary semantic Markdown nodes, but MDX disables indented code and splits tested multiline span; adapter silently keys remainder | Tested CommonMark code ownership preserved; tight-list prose reconstructed from omitted paragraph tags | Same tested native flow interruption; adapter diagnoses and assigns no key; MDX indented code disabled |
| Components and data | JSON-prop Poll becomes prose; simpler JSX-shaped text becomes HTML | Native flow/text JSX, property kinds, multiline props and Markdown children; JSON gate demonstrated | Opaque HTML or prose; strict mode rejects all unescaped prose `<`, not a component parser | Native nested Canvas/Place/Image and Markdown children; exact Run strings; strict JSON with recursive duplicate-key rejection |
| Extension seam | Registered block/inline parsers, custom nodes, post-parse transformer | Public parser → arena/codecs → AST pass; no equivalent arbitrary grammar injection shown | Public offset iterator → derived event tree → transform; not parser injection | Public parse-only function → typed mdast → adapter; no general mdxjs plugin hook; underlying markdown has configurable callbacks |
| Source mapping | Exact key ranges, block starts, payload segments; no universal end range | Node byte ranges, exact comment range; UTF-16 columns; BOM base adjustment | Native event byte ranges; tight-paragraph hull explicitly derived; exact key ranges | Node/comment byte ranges; expression stops; no complete attribute spans |
| Error model | Permissive Markdown plus adapter diagnostics; malformed component intent can remain clean prose | Native errors plus partial arena; adapter diagnostics; known false-clean author result | Permissive Markdown plus adapter diagnostics; strict rejection is intentionally broad | Native `Result` error before projection, then adapter diagnostics with partial projection |
| Minimal remaining adapter burden | Semantic text decoding and complete lowering, **plus component grammar composition** | Boundary guard, fail-closed supported-node compiler, literal/schema policy and lowering | Tight-list normalization and complete lowering, **plus component grammar composition** | Complete typed schemas/lowering and dialect bounds; dependency integration or direct markdown callback proof |

No candidate's “source retained” test proves a lossless AST serializer. None
proves every Leaflet kind, all Markdown extensions, fuzz resilience, depth limits,
memory consumption or a production build on all deployment targets.

### Rust dependencies and maintenance fit

The four isolated manifests were read. Rushdown and pulldown use their Markdown
crate plus serialization; there is no demonstrated JS parser requirement in
those paths. That is attractive only if we account for the rich grammar we would
then maintain ourselves. Their pre-1.0 APIs warrant pins and retained controls,
not an invented claim that one upstream is unreliable.

Satteri's executed API is **not the `satteri` 0.2.17 facade**. It is
`satteri-pulldown-cmark` 0.6.7 + `satteri-ast` 0.6.3 + `satteri-arena` 0.4.3,
at source `7d1297b7e4551a1c5124fa07bd2d1d05f66236e6`. Its MDX feature brings
OXC parsing dependencies (reported lock resolution 0.121.0). This is native Rust,
not a Node/NAPI runtime requirement. The cost includes tracking coordinated
parser/AST/arena interfaces; compile-time or runtime costs were not measured.

mdxjs source `282729e6a0750c19dfd9217bc511971bf3295a29` makes `swc_core` mandatory,
including codegen/visit features, even when only `mdast_util_from_mdx` is called.
The experiment reports a real `serde::__private` build failure in resolved
`swc_common` 12.0.1 and works around it with Serde 1.0.219/serde_json 1.0.140.
Production currently requests **1.0.229/1.0.154** in [Cargo.toml](../Cargo.toml).
The isolated successful build therefore does not prove compatible workspace
adoption. Do not silently transplant the old pins or call this mere hypothetical
dependency risk. A coherent SWC update/patch might solve it, but was not tested.

Direct markdown-rs is a concrete architectural alternative, not a claim that a
missing crate can be wished away. At inspected revision
`1506572f9b406431402928f3a8b3df0b4ae2d8f5`, its manifest's normal dependencies
are `unicode-id` and optional log/serde; SWC is a **dev dependency**. Removing
mdxjs from a consumer graph can remove its compiler stack. Whether our required
dialect remains correct after that removal is the next proof, not an inference
from the manifest. All reported Rust checks used the stated nightly toolchain;
deployment/MSRV/cross-target integration remains unverified.

## The decisive negative control: literal code becoming identity

The Satteri fixture is:

````text
Code `first
{/* springleaf:key literal */}
last`.
````

In Markdown-only mode this is one paragraph with a multiline code span. In
Satteri MDX mode it is Paragraph / flow comment / Paragraph, without a native
error. `bind_comments` in experiment 07, lines 232–286, accepts the flow node and
keys its next sibling. The result has key `literal` on `last\`.` and exits 0.
The comments test asserts the undesired capture both at root and inside a quote;
this is a passing **negative-observation test**, not a passing identity invariant.

There are two distinct responsibilities:

1. **Upstream grammar:** Satteri's `mdx_block_interrupts` at firstpass.rs:4304–4306
   recognizes leading JSX/expressions as paragraph interrupts; the integration
   at :4391–4394 permits the split before the inline code parser can own the span.
   Indented-code disabling is also deliberate MDX behavior. The code-span result
   is a mismatch with the desired CommonMark expectation, not established here as
   a Satteri-specific conformance bug.
2. **Adapter policy:** accepting that expression as identity without a boundary
   check is an immediate correctness defect for this product's key contract.
   A trustworthy native AST does not by itself establish author intent.

Experiment 09 already supplies the analogous baseline: its
`native_mdx_flow_comments_interrupt_apparent_multiline_code` stock test asserts
the same three-node split. Its author adapter, `children` at lib.rs:214–281,
requires a blank line before a key comment when there is a preceding sibling.
The identity test then asserts `misplaced-key` and **no keys**. This is a proven
guard against the tested capture; it is not restoration of multiline code.
There is no need to execute either green suite again to discover this distinction.

**Satteri is blocked unchanged, not disqualified forever.** Porting the boundary
policy is a plausible small engineering change, but must be proved using its
own ranges/container behavior, including the quoted case. Documenting “use fences”
alone while continuing to accept the bad input silently is insufficient.

For a bounded MDX-shaped dialect I recommend fenced blocks for multiline code
examples, single-line inline code for directive examples, and explicit blank
boundaries before identity comments. Reject known ambiguous key placements.
Four-space indentation is not code protection in MDX: that is an explicit dialect
choice, and the blank-line guard does not change it. If all ordinary CommonMark
code interpretations must survive, neither native MDX run qualifies unchanged.
That requires a parser-level ownership change/proof, not an AST key transform.
Even with restrictions, the current cases are not a universal no-misbinding theorem;
container adjacency and code-shaped component keys belong in the next small corpus.

## Syntax choice: comments now, compact attributes only with a reason

Recommended selected-block syntax:

```mdx
{/* springleaf:key introduction */}
## Introduction

{/* springleaf:key central-argument */}
Ordinary **formatted** prose.

<Poll key="topic" options={["Gardens", "Maps"]} />
```

Specify one case-sensitive document-wide namespace; bind to the immediate
supported sibling in the same container, never skip definitions/comments or
escape a closed item/component. A marker before a list names the list; one inside
an item names its next paragraph. Leave direct list-item naming unspecified until
needed. Duplicate keys and comment/property collisions invalidate the document.
Moving a keyed block preserves its name, not its source offset or tree index.

The user's compact syntax preference remains technically possible:

| Option | Consequence |
| --- | --- |
| Rushdown custom `{#id}` layer | Proven heading suffix/final paragraph-line behavior, with code protection; components still need their own grammar |
| pulldown event-layer `{#id}` | Proven without a parser fork; cannot infer component syntax ownership from those events |
| Native heading attributes plus another paragraph form | Satteri demonstrates heading attributes with MDX, but not paragraph tails; native last-ID-wins semantics need additional evidence-preserving validation |
| Reinterpret mdxjs hash expressions | Not general: `#introduction` and `#central-argument` are accepted, `#trailing.` and `#two..dots` fail before our transform; would couple author names to incidental SWC grammar |
| Custom markdown-rs callback / lexical layer | Could recognize a reserved body-expression language; still needs heading-versus-flow attachment and code/container policy. Context-aware lexical preprocessing would own source mapping and grammar boundaries too |
| Different marker syntax | Potentially avoids the expression collision, but supplies no existing proof and adds a new author convention |
| Native comment expressions | Already demonstrated with rich components, complete key alphabet independent of JS private-name parsing, and exact comment ranges |

Select comments for the next proof. Do not implement both spellings merely
because experiments contain both. The preferred compact form can be revisited
if authorship experience makes its benefit worth another grammar contract.
Unescaped braces/angle syntax in prose now has MDX meaning; literal escapes/code
and exact Run strings must be documented. This is ordinary prose without wrappers,
not a promise of accepting every unmodified CommonMark document.

## What the experiments actually established

- **06 / Rushdown:** real parser registration, not regex source substitution;
  inline ownership protects code and AST siblings provide deterministic key
  attachment. Its eager-interruption control explains why the chosen architecture
  works. The JSON-prop Poll counterexample proves the extension cannot distinguish
  component intent from prose. Raw text escapes/entities in its projection still
  need semantic decoding. Starts/payload segments are not complete CST ranges.
- **07 / Satteri:** a real native component tree and comment expressions, including
  multiline props and Markdown children; no facade compilation or JS execution
  needed. The adapter's Poll/Image registry is policy, not the parser's component
  limit. It does not demonstrate the exact nested Canvas fixture from 09, recursive
  duplicate JSON-key rejection, or a safe code-interruption boundary. Its `Other`
  projection also needs a fail-closed supported-node policy in a real compiler.
- **08 / pulldown:** event postprocessing suffices for selected prose IDs and
  semantic text. Tight-list Paragraphs are honestly derived; native heading IDs
  are unsuitable unchanged because their extraction can lose duplicate evidence
  and differs on escapes/closing hashes. `strict` rejecting `<Poll ...>` and
  `a < b` establishes a rejection boundary, not JSX parsing or data fidelity.
- **09 / mdxjs:** the strongest demonstrated composition: nested component tree,
  ordered attributes/children, Markdown children, exact JSON strings, source stops
  and guarded identity attachment. Strict JSON validation is adapter code; SWC
  would accept many expressions it rejects. No full property schemas, named-page
  reference table, Leaflet compiler, or workspace dependency integration is proved.

The reported 39/47/37/44 tests are receipts with different scopes, not scores.
All four permit inspectable partial results on adapter errors. A production API
should expose a validated document only after all syntax, identity and schema
checks pass; a key still visible in invalid inspection output is not permission
to lower or publish it. Error accumulation versus early native failure is an
interface difference, not evidence that permissive parsing is safer.

### Why not combine parsers immediately?

Rushdown could claim component syntax before HTML/paragraph fallback through its
public hooks. A bounded literal-component grammar would still need balanced
quoted/braced values, nesting, multiline attributes, Markdown children, malformed
input handling and source coordinates. pulldown would need an outer grammar or
equally explicit region-ownership mechanism; output rejection cannot recover a
native component tree. Neither composition has been built. Shipping an HTML
reparse or line-regex splitter would conceal rather than close the gap.

In contrast, Satteri already calls its underlying parser directly. The alternative
under mdxjs is unusually concrete: `mdast_util_from_mdx` itself calls
`markdown::to_mdast` with public `ParseOptions` and SWC callbacks. The public
`ExpressionKind` separates body expressions, spread attributes and property-value
expressions. A literal dialect can try accepting comments in the first, rejecting
the second, and parsing strict JSON in the third.

**The callback is also a boundary recognizer.** `Signal::Eof` asks the parser to
collect more input; `Error` ends parsing; `Ok` declares an expression complete.
Strings containing braces and nested objects must not terminate early. Simply
omitting callbacks is not equivalent: experiment 09's stock control shows
`markdown::to_mdast` then accepts `{a + }`, unlike mdxjs. A late JSON check cannot
repair an expression that was segmented incorrectly. This is why the direct
markdown route merits a proof rather than an immediate “easy replacement” claim.

## Smallest next proof and adoption gates

**Recommend one isolated markdown-rs literal-callback experiment, using 09 as the
behavioral reference.** It is new work to commission, not performed here. Avoid
another four-way parser implementation or a new broad dialect investigation.

1. **Boundary and dependency gate.** Use direct `markdown` with current production
   Serde requirements in an isolated workspace; demonstrate no mdxjs/SWC normal
   dependency. Implement the narrow callback and explicit construct options.
   Retain the tested blank-before rule. Accept only the selected comment grammar
   for body expressions, strict JSON values for properties, and no ESM/spreads.
   Explicitly reject unsupported source rather than letting disabled constructs
   silently become accepted prose. Prove incremental complete/incomplete/error
   handling with braces/quotes inside strings, nested JSON and truncated input.
2. **One mini compiler document.** Lower one source with an ordinary heading,
   selected keyed prose, an unkeyed paragraph, a list-item paragraph, a fenced
   literal marker, one nested Canvas/Place/Image, an exact Run string and a keyed
   Poll into an in-memory typed document. Add one named Page/reference if needed
   to exercise name resolution; no transport or persistence is necessary.
   Validate concrete property types/child models for only these components.
   Reorder the keyed prose and verify key/content continuity and unchanged
   unedited source bytes; offsets are expected to move.
3. **Rejection gate.** Use the shared decisive controls: the root and quoted
   multiline-code trap; empty item/component boundary; adjacent directives;
   duplicate prose/component names and JSON properties; malformed key/JSX;
   arbitrary expression; marker-shaped text inside JSON strings; Unicode/CRLF.
   Assert semantic outcomes and no accepted document on error, not just exit
   counts. Include indented directive/code-shaped JSX examples so the chosen
   MDX restriction is visible rather than mistaken for a repair.

Stop if this needs a parallel lexer, broad source rewriting or a replacement JSX
parser. Then evaluate **guarded Satteri** against this same narrow document/corpus:
the exact first fix boundary is `bind_comments`, before any key assignment, with
position-based blank-boundary/container checks. Broader CommonMark preservation
would instead require changing upstream paragraph interruption; that is a separate
choice with its own proof, not a hidden requirement on a sibling binder.

Once these gates hold, a separately commissioned implementation can select the
substrate and grow typed component/block coverage. Every recognized-but-unlowered
node must diagnose. Full current Leaflet representation remains a coverage goal,
not something the first mini compiler claims by accepting arbitrary prop maps.
The next independent pipeline gate is the recreation proposal's offline staging
and entry-switch proof, followed by explicitly commissioned deployed transport,
media and native-publication verification. Parser success establishes none of
those external mechanisms, and selected prose keys alone establish no remote
continuity policy.

## Evidence, inspection and verification

This assessment began with a clean working copy at `57069890`, parent `f4804f03`.
The completed experiment reports and owned code were read directly. Checkpoint
sets supplied by the commissioning session are:

| Experiment | Preserved checkpoint set | Source identity |
| --- | --- | --- |
| 06 | `03907ae2`, `da737eed`, `27a8f483`, `9bec2a7e`, `53866ffd` | rushdown `e5eb4e4446541ea0ed53111c1b37e779283ff57c` |
| 07 | `ab28cda1`, `1d2338ad`, `818448ed`, `f4804f03` | satteri `7d1297b7e4551a1c5124fa07bd2d1d05f66236e6` |
| 08 | `527c09fa`, `bc8334da` | pulldown `014b61d38d235f382935bb676bab872b7680afa2` |
| 09 | `dc71aa22`, `f3fb4377`, `c9b11499` | mdxjs `282729e6a0750c19dfd9217bc511971bf3295a29`; markdown `1506572f9b406431402928f3a8b3df0b4ae2d8f5` |

Review anchors beyond the source entries above:

- [06 substrate controls](../experiments/06-rushdown-identity/tests/substrate.rs)
  and [attribute tests](../experiments/06-rushdown-identity/tests/attributes.rs),
  comment tests :75–209, and all of `src/projection.rs`: code interruption,
  component-as-prose, container identities and raw text/position limits.
- [07 substrate controls](../experiments/07-satteri-identity/tests/substrate.rs),
  [comments](../experiments/07-satteri-identity/tests/comments.rs) and
  [components](../experiments/07-satteri-identity/tests/components.rs): native
  attribute conflict, root/quote misbinding, literal gates and position behavior.
- [08 substrate controls](../experiments/08-pulldown-cmark-identity/tests/substrate.rs)
  and [identity tests](../experiments/08-pulldown-cmark-identity/tests/identity.rs):
  tight paragraphs, last-ID-wins, code ownership and rejection-only strict mode.
- [09 stock controls](../experiments/09-mdxjs-identity/tests/stock.rs),
  [identity tests](../experiments/09-mdxjs-identity/tests/identity.rs) and
  [component tests](../experiments/09-mdxjs-identity/tests/components.rs): matched
  native interruption, guard, nested JSX, exact literals and absent attribute spans.

Selected upstream source was read from the local archives; `git rev-parse HEAD`
confirmed the exact Satteri, mdxjs and markdown revisions cited above. Rushdown
and pulldown upstream internals are attributed to their reports and executable
controls, rather than claimed as independently reread in full. Historical build
failures and green nextest/clippy/fmt receipts remain reported experiment evidence.
No parser suite or new parser execution was run: the material analogous code-span
question already has direct stock and adapter controls. This review adds source
comparison, not independent replication of every assertion.

Verification command for this owned document:

```sh
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- check --okf design/authoring-parser-assessment0.gpt6a.md
```

### Explained cross-references

The [recreation design](mdx-recreate0.gpt6a.md) owns the downstream source-to-native
publication architecture; this assessment refines its front end and preserves
optional ordinary-prose identity. The four reports retain independent evidence:
[Rushdown](../experiments/06-rushdown-identity/REPORT.md) establishes the extension
alternative; [Satteri](../experiments/07-satteri-identity/REPORT.md) establishes the
native-AST alternative and unguarded failure;
[pulldown](../experiments/08-pulldown-cmark-identity/REPORT.md) separates iterator
transformation from grammar ownership; [mdxjs](../experiments/09-mdxjs-identity/REPORT.md)
supplies the strongest demonstrated composition and the dependency motivation for
the proposed next proof. The [experiment guide](../experiments/README.md) explains
why those tracked offline receipts remain separate from production implementation.
