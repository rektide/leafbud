---
type: Design
title: "Leafbud's authentication architecture, explained: is Leaflet-for-everything the plan?"
description: "Plain-language account of every remote interaction Leafbud makes, what authentication each uses, how PDS writes actually happen through Leaflet's server, and what it would take for Leafbud to ever need its own ATProto credentials."
resource: /design/auth-architecture0.glm53m.md
status: draft
generated:
  at: 2026-11-10
  by: model:zai-coding-plan/glm-5.3#max
sources:
  - resource: "opencode://ses_e399c8b55ffeXTet28ACQMbei6"
    title: "Research session tracing Leafbud's remote interactions and the Leaflet-brokered PDS write path; operator explainer for the Leaflet-only architecture question"
  - resource: /crates/workflow/src/lib.rs
    title: "Leaflet session-cookie credential resolution: --token, LEAFLET_AUTH_TOKEN, XDG auth-token file"
  - resource: /crates/leaflet/web/src/lib.rs
    title: "LeafletClient transport: cookie header, disabled redirects, /home and get_leaflet_data reads"
  - resource: /crates/leaflet/web/src/actions.rs
    title: "Same-origin action discovery guards: zero/ambiguous references refuse"
  - resource: /crates/leaflet/web/src/publication.rs
    title: "publishToPublication transport; no caller rkey; oauth_session_expired failure kind"
  - resource: /crates/leaflet/web/src/delete.rs
    title: "deleteLeaflet/deletePost transports and their typed failure contract"
  - resource: /crates/atproto/src/lib.rs
    title: "Public identity-resolution chain and the crate's unused session machinery"
  - resource: /crates/atproto/src/session.rs
    title: "Dormant app-password session store; no workflow caller"
  - resource: /crates/workflow/src/release.rs
    title: "PublicReader: unauthenticated resolver + getRecord reads"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts
    title: "pdsDid selection (actor vs publication owner), restoreOAuthSession, putRecord, compensating deleteRecord"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/atproto-oauth.ts
    title: "restoreOAuthSession and the Supabase-backed OAuth session store"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/lish/[did]/[publication]/dashboard/deletePost.ts
    title: "deletePost restores the record owner's OAuth session and deletes from both collections"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts
    title: "Public action strips caller rkey and clamps future dates"
  - resource: https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/inngest/functions/delete_records_from_pds.ts
    title: "Upstream acknowledgment that the appview can miss firehose delete events"
---

# Leafbud's authentication architecture, explained

## TL;DR

Yes — using Leaflet for all remote work is the accepted, deliberately chosen
design, and no, Leafbud has never minted ATProto credentials of its own and
does not need any for anything it does today. Leafbud holds exactly one
secret: the Leaflet website session cookie. Every read of public records goes
to the PDS unauthenticated (public reads need no login), and every *write* to
the PDS happens **inside Leaflet's server**, which uses the OAuth connection
the record's owner already established by logging into Leaflet with their
ATProto identity. Leafbud never talks to the PDS with credentials; it hands
sealed instructions to Leaflet and Leaflet's server does the signed writing.
The roadmap records this as settled: the native-lifecycle ticket's acceptance
criterion says failures "do not fall back to raw PDS writes," and direct-PDS
tooling exists only as two parked P3 tickets
([front F](#is-leaflet-for-everything-the-accepted-design) below). The real
costs of this path are coupling to Leaflet's server-action surface, dependence
on the owner's OAuth link staying healthy, and a capability ceiling (custom
record keys, scheduling, gated content, arbitrary uploads) — each examined
with evidence below.

## Vocabulary used in this document

- **PDS (Personal Data Server).** The AT Protocol repository host where a
  person's public records actually live — the "data home" that other services
  (like Leaflet) read from and write to on the person's behalf. Every ATProto
  identity (DID) points at one PDS endpoint.
- **OAuth.** The standard authorization flow that lets one service hold a
  revocable permission to act as a user on another service, without the user
  handing over a password. When you log into Leaflet with your ATProto
  identity, Leaflet's server obtains and stores an OAuth session with your
  PDS, and thereafter writes to your PDS as you.
- **Session cookie.** The `auth_token` cookie a browser holds after logging
  into the Leaflet *website*. It is the one secret Leafbud possesses. Sending
  it with a request to `leaflet.pub` is "being logged in."
- **Server action.** A Next.js mechanism where the website's JavaScript calls
  a named function that executes on Leaflet's server. The publish button, the
  delete menu item — each invokes a server action. Leafbud invokes the *same*
  actions, identified by per-deployment references it discovers fresh each
  time.
- **rkey (record key).** The last path segment of an AT URI
  (`at://<did>/<collection>/<rkey>`) — the record's name inside its
  collection. Whoever chooses the rkey chooses the record's URL.
- **App password.** A weaker, older ATProto authentication style: a
  server-generated password exchanged directly with the PDS for API tokens.
  Leafbud has *built but never used* machinery for this (see
  [below](#the-dormant-direct-pds-machinery)); Leaflet itself uses OAuth
  instead.

## The map: every remote interaction and what authenticates it

Leafbud talks to exactly two kinds of remote systems. Verified against the
current code:

**A. Everything to `leaflet.pub` (default origin,
[lib.rs](/crates/leaflet/web/src/lib.rs) line 29; overridable via
`LEAFLET_BASE_URL` or `--base-url`, workflow
[lib.rs](/crates/workflow/src/lib.rs) lines 66–69) — authenticated by the
session cookie:**

| Interaction | What it is | Auth |
|---|---|---|
| `GET /home` | Discovery of your leaflets + your identity payload | Cookie ([lib.rs](/crates/leaflet/web/src/lib.rs) lines 280–290) |
| `POST /api/rpc/get_leaflet_data` | Read one leaflet's metadata/rows | Cookie (lines 292–313) |
| `POST /api/rpc/pull`, `/api/rpc/push` | Yrs document snapshots and edits | Cookie ([pull.rs](/crates/leaflet/web/src/pull.rs) lines 21–36; [push.rs](/crates/leaflet/web/src/push.rs) lines 11–25) |
| Facts RPC | Explicit entity-fact reads | Cookie ([pull.rs](/crates/leaflet/web/src/pull.rs) lines 39–45) |
| `GET /new`, `POST createPublicationDraft` | Mint a new draft | Cookie required ([mint.rs](/crates/leaflet/web/src/mint.rs) lines 33–59) |
| `POST publishToPublication` | Publish (the PDS write — see next section) | Cookie + discovered action reference ([publication.rs](/crates/leaflet/web/src/publication.rs) lines 62–123) |
| `POST deleteLeaflet` / `deletePost` | Delete editor / delete published record | Cookie + discovered action reference ([delete.rs](/crates/leaflet/web/src/delete.rs) lines 49–148) |

**B. Everything to the wider ATProto network — deliberately unauthenticated
public reads:**

| Interaction | What it is | Auth |
|---|---|---|
| DNS TXT `_atproto.<handle>`, `https://<handle>/.well-known/atproto-did`, `plc.directory`, `did:web` well-known | Handle → DID → PDS endpoint resolution | None — public infrastructure ([atproto lib.rs](/crates/atproto/src/lib.rs) lines 10–26) |
| `GET com.atproto.repo.getRecord` on the PDS | Read a published record for status/diff/verification | None — the query helper sends no credentials ([xrpc.rs](/crates/atproto/src/xrpc.rs) lines 132–137, 269–285; [release.rs](/crates/workflow/src/release.rs) lines 32–58) |
| Live realtime socket bootstrap | Public realtime configuration for watch/sync | None — "public anon JWT" by design ([live.rs](/crates/leaflet/web/src/live.rs) lines 44–46; resolved by a separate cookie-free client, [lib.rs](/crates/leaflet/web/src/lib.rs) lines 248–252) |

**Where the one secret lives.** The cookie is resolved in priority order:
`--token` flag, then `LEAFLET_AUTH_TOKEN` environment variable, then a file
at `$XDG_CONFIG_HOME/leafbud/auth-token` (workflow
[lib.rs](/crates/workflow/src/lib.rs) lines 65–80). On this machine that file
exists (mode 0600), alongside a `cookies.json` capture record noting the
cookie was "extracted from logged-in playwright session on leaflet.pub" —
i.e., the operator logged into the website in a browser and the cookie was
harvested; no Leafbud flow ever logs in by itself. The cookie is sent as a
marked-sensitive header, and redirects are disabled so it can never follow a
server-directed change of origin ([lib.rs](/crates/leaflet/web/src/lib.rs)
lines 219–235). A `pds` config key exists but only selects an explicit read
endpoint for public reads (workflow
[config.rs](/crates/workflow/src/config.rs) line 66;
[release.rs](/crates/workflow/src/release.rs) lines 23–30) — it is not a
credential.

**Confirmed: no ATProto credentials were ever minted by or for Leafbud.**
Three independent checks: (1) the session store file
`$XDG_CONFIG_HOME/leafbud/session.json` that the direct-PDS machinery would
create ([session.rs](/crates/atproto/src/session.rs) lines 97–108) does not
exist on this machine; (2) no code outside the `leafbud-atproto` crate calls
`create_session`/`resume_or_create_session` — the write-side session
machinery has zero workflow callers; (3) no configuration key or environment
variable carries an app password (the full key set is workspace, handle, pds,
publication, diff-program/args/exit — config.rs lines 63–71).

## How PDS writes actually happen today

Leafbud never sends a credentialed request to any PDS. When Leafbud publishes
or deletes, the sequence inside Leaflet's server (all verified in the pinned
clean archive at `600c7b5f`) is:

1. **Whose repository gets the record is decided first.** For a *standalone*
   post, it is the acting user: `pdsDid = actorDid`
   ([publishLeaflet.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishLeaflet.ts)
   lines 180–186). For a *publication-container* post, it is the publication
   owner: `pdsDid = data.identity_did!` (lines 188–204) — contributors
   publish through the owner's repository, with contributor status checked at
   lines 199–201. (In the operator's actual setup the same person is both —
   the configured publication
   `at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.publication/3molsbwkh522o`
   is owned by the configured handle's DID, per `~/.config/leafbud/leafbud.toml`.)
2. **Leaflet's server restores that owner's OAuth session.**
   `restoreOAuthSession(pdsDid)` (line 244) pulls the stored OAuth session
   from Leaflet's Supabase `oauth_session_store` table
   ([atproto-oauth.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/atproto-oauth.ts)
   lines 171–190). That session exists because the owner logged into
   Leaflet's website through the ATProto OAuth flow
   (`app/api/oauth/[route]/route.ts` lines 125, 145–150 — the authorize and
   callback steps). If the stored session is missing or cannot be refreshed,
   the caller gets a typed `oauth_session_expired` error naming the DID
   (atproto-oauth.ts lines 238–260).
3. **The write rides a generated lexicon client bound to that session.**
   `new AtpBaseClient(loggedFetchHandler(credentialSession, …))`
   (publishLeaflet.ts lines 249–255) then calls
   `agent.com.atproto.repo.putRecord(…)` (line 481). Blob uploads flow through
   the same agent via publish upload hooks (line 377).
4. **Deletion mirrors this.** `deletePost` resolves the record's repository
   owner from the AT URI itself (`ownerDid = new AtUri(document_uri).host`,
   [deletePost.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/(app)/(identity)/lish/[did]/[publication]/dashboard/deletePost.ts)
   lines 26–53), restores *that owner's* OAuth session (line 78), and deletes
   the record from both document collections with per-collection errors
   swallowed, alongside the database rows (lines 90–105).

So the operator's mental model is exactly right: **ATProto writes ride on the
operator's existing Leaflet↔PDS OAuth link, brokered by Leaflet's server.**
Leafbud's cookie gets the action accepted; Leaflet's stored OAuth session
performs the actual cryptographic write. Leafbud's own transport encodes this
honestly — the publish failure type includes `OauthSessionExpired`
([publication.rs](/crates/leaflet/web/src/publication.rs) lines 145–151), and
the delete transport documents `oauth_session_expired`/`not_authorized` as
clean refusals that precede record effects
([delete.rs](/crates/leaflet/web/src/delete.rs) lines 113–118).

## Is Leaflet-for-everything the accepted design?

Yes, on several converging records — this is not an accident of
implementation:

- **The active ticket's acceptance criterion forbids the alternative.**
  `leafbud-release-native` (in progress) accepts native publication with:
  "failures do not fall back to raw PDS writes" (`bd show
  leafbud-release-native`, acceptance criteria). The direct-PDS path is
  explicitly *not* a fallback.
- **The accepted authoring contract parks PDS tools in a separate group.**
  [authoring-interface0](/design/authoring-interface0.gpt6a.md) lines 164–167:
  the old `publish build/run` and PDS `sync` "must not compete with the new
  native meanings," to be reused only under an explicit `pds` group if
  retained.
- **The sync design rules out silent PDS substitution.**
  [document-sync1](/design/document-sync1.gpt61s.md) lines 313–317: "There is
  no silent PDS fallback. PDS-managed operations with chosen rkeys are
  separate future tickets," and lines 400–401 name the architecture: "One
  Leaflet-managed adapter; editor/application/PDS are upstream components."
- **The fronts review keeps direct-PDS work parked.**
  [next-fronts0](/design/next-fronts0.gpt6a.md) front F (lines 428–432):
  `leafbud-pds-publish` and `leafbud-pds-sync` "remain P3… Reassess these
  only for an explicit direct-PDS use case."
- **The parked tickets say what they would add.** `leafbud-pds-publish` (P3,
  open): an explicit PDS-managed first-publication path with
  caller-selected/validated rkeys, envelope stripping, identity preservation,
  and documented lifecycle tradeoffs — "a document has one chosen management
  mode, never simultaneous independent Leaflet/PDS writers," and "no live
  writes without separate operator authorization." `leafbud-pds-sync` (P3,
  open): PDS-managed pull/push with CID-preconditioned updates and explicit
  conflict resolution. Both originate from an operator request during
  redesign round 1 and were retained deliberately at P3 in the approved
  roadmap — parked because the native Leaflet lifecycle was chosen as the
  primary path and a second writer identity would complicate every conflict
  decision, not because they were judged worthless.

## Consequences and risks of the Leaflet-only path

Stated honestly, each with its evidence:

**(a) Coupling to undocumented server actions — but refusal, not corruption,
on change.** Leafbud does not hardcode action IDs; each effect re-discovers
the action reference from the current deployment's own pages by scanning
same-origin `/_next/static/*.js` chunks and pinning the function/export
pairing rather than a deployment hash
([actions.rs](/crates/leaflet/web/src/actions.rs) lines 37–44, 95–99). If a
Leaflet deploy renames, moves, or duplicates the action: zero references, two
references, changed shape, or a redirected page all *refuse* — "current
native action reference unavailable; no fallback" (lines 40–44), "ambiguous
native action reference" (lines 82–84), "mint/publication action
redirected/not found; outcome uncertain" ([mint.rs](/crates/leaflet/web/src/mint.rs)
lines 83–85; [publication.rs](/crates/leaflet/web/src/publication.rs) lines
104–106). The risk is unavailability until Leafbud is updated to the new
shape — not silent misbehavior. The residual risk is a *shape-compatible*
action whose semantics changed upstream, which discovery cannot detect; the
pinned-archive discipline (re-verification against `600c7b5f`, per the
roadmap receipts) is the standing mitigation.

**(b) Dependence on the owner's Leaflet↔PDS OAuth link.** Publish and delete
both fail with `oauth_session_expired` when the record owner's stored session
is gone or unrefreshable (publishLeaflet.ts lines 240–247; deletePost.ts
lines 78–81; surfaced in our transport at publication.rs lines 145–151). The
fix is *not* anything Leafbud can do: the record owner must log into
Leaflet's website again to re-establish the OAuth link (the flow at
`app/api/oauth/[route]/route.ts`). For the operator's single-person setup
this is a mild, self-service failure — Leafbud reports which DID needs
re-login. Leaflet also runs an `cleanup_expired_oauth_sessions` inngest job
(seen in `app/api/inngest/functions/`), so lapsed sessions are actively
swept, not merely ignored.

**(c) A real capability ceiling.** Things Leaflet's public actions do not
expose, Leafbud cannot do at all:

- *Caller-selected rkeys / URLs.* The public `publishToPublication` strips a
  caller-supplied rkey before delegating — "The rkey is server-chosen here: a
  caller-supplied one could target an existing record in the publication
  owner's repo" ([publishToPublication.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/actions/publishToPublication.ts)
  lines 13–16, 28–39; our transport mirrors this: "intentionally no caller
  rkey" at [publication.rs](/crates/leaflet/web/src/publication.rs) lines
  10–11). First publish mints a TID server-side (publishLeaflet.ts lines
  278–280).
- *Scheduling as ordinary publish.* A future `publishedAt` is clamped to now
  (publishToPublication.ts lines 30–35); scheduled posts are a separate
  server mechanism (`saveScheduledPost`, per that comment and the
  `publish_scheduled_post` inngest function) with no action Leafbud invokes.
- *Members-only/gated content.* Leaflet deliberately never sends gated images
  to the PDS (they go to private storage —
  [publishHelpers.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/src/utils/publishHelpers.ts)
  lines 25–47), and Leafbud's editor cannot author the delimiter at all
  ([coverage.md](/doc/mdx/coverage.md) lines 193–197).
- *Arbitrary uploads.* Blob upload happens only inside the publish action via
  server-side hooks (publishLeaflet.ts line 377 and the hooks at lines
  285–291); there is no standalone upload action.
- *Other deliveries.* Email send defaults to true inside the action (arg at
  publishLeaflet.ts line 176; status at line 643) — Leafbud pins
  `sendEmail:false` on its wire ([publication.rs](/crates/leaflet/web/src/publication.rs)
  line 25), but the newsletter/broadcast surface beyond that flag belongs to
  the open `leafbud-release-delivery` ticket, not to any action Leafbud
  currently drives.

**(d) Smaller frictions found along the way.** *Appview propagation lag* is
real and acknowledged upstream: an inngest job exists partly so "a post is
gone even if the appview misses the firehose delete event"
([delete_records_from_pds.ts](https://github.com/hyperlink-academy/leaflet/blob/600c7b5f1f7b31b66fb626c293cc104312a12caf/app/api/inngest/functions/delete_records_from_pds.ts)
lines 8–13) — Leafbud's own answer is content-verified settlement through
the PDS itself, not the appview. *Per-deployment action availability*: a
Leaflet deployment without the `deletePost` chunk leaves published-record
deletion unsupported — our transport refuses explicitly ([delete.rs](/crates/leaflet/web/src/delete.rs)
lines 119–123). *Server-side housekeeping* comes as a package deal with the
action (the accepted gated-image cleanup
([native-cleanup-question0](/design/native-cleanup-question0.glm53m.md)),
schedule deletion at publishLeaflet.ts lines 552–558, and a compensating
PDS delete on first-link failure at lines 532–542) — all disclosed in
preview rather than suppressible.

## The dormant direct-PDS machinery

One nuance the operator may not have known: Leafbud's codebase already
contains a complete, hardened direct-PDS credential stack — an app-password
`createSession`/`refreshSession` client and a 0600-mode session store
([session.rs](/crates/atproto/src/session.rs); store layout at
[atproto lib.rs](/crates/atproto/src/lib.rs) lines 41–46), plus
`putRecord`/`uploadBlob` shapes ([xrpc.rs](/crates/atproto/src/xrpc.rs)
lines 3–7). It is leftover foundation from the pre-reset direct-publish
design (historical evidence at revision 80436b61, per the `leafbud-pds-publish`
ticket notes), it has no callers, and it has never been given a password:
the store file does not exist. Nothing about its presence changes the
current architecture; it merely means reviving the P3 tickets would start
from existing, tested code rather than a blank page — with the caveat that
app-password auth is the older style and an OAuth-client path (what Leaflet
itself uses) might be preferred if that work is ever commissioned.

## When would Leafbud actually need its own ATProto credentials?

Concretely: **only to write.** Everything Leafbud does today reads public
data through endpoints that require no authentication — record reads
(`getRecord`), identity resolution (DNS/plc.directory/well-known), and the
realtime socket are all public by design (see the map above). Private blobs
are not a gap: gated content never reaches the PDS at all, so no credential
would retrieve it. Enumerating the triggers:

1. **Direct PDS publication with caller-selected rkeys** (`leafbud-pds-publish`):
   needs Leafbud's own credentialed session and `putRecord`. Public reads do
   not cover writes. This is the ticket's entire point.
2. **PDS-managed sync with CID preconditions** (`leafbud-pds-sync`): reads
   are already possible unauthenticated; the *conditional writes* and
   conflict resolution are what need credentials.
3. **Nothing else identified.** No current read, verification, watch, or
   authoring capability is blocked on authentication. (Uncertainty label:
   this survey covers Leafbud's present command surface; a future capability
   could of course change the list.)

## Path forward (analysis, not a decision)

| Option | What it gets | What it costs | Standing |
|---|---|---|---|
| **Stay Leaflet-only** (current accepted design) | One auth secret; native lifecycle parity with the web UI; every native effect disclosed in preview; no second writer identity to complicate conflicts | The risks above: upstream action coupling, owner-OAuth dependence, the rkey/scheduling/gated/upload ceiling | Matches the active ticket, the authoring contract, and front F |
| **Leaflet-only + monitor upstream** | Same as above, plus early warning: watch the pinned archive for action changes, refresh the pin deliberately, keep discovery guards as the tripwire | A little ongoing attention; no code path changes | A disciplined reading of the current design rather than a new one |
| **Revive the P3 direct-PDS tickets** | Chosen rkeys/URLs, direct writes without Leaflet in the loop, a second lifecycle for PDS-only documents | A second credential to mint and protect; a second writer identity requiring explicit management-mode switching; loss of native side effects (editor linkage, versioning, revalidation — enumerated in the ticket); real implementation work on dormant foundations; explicitly gated on "separate operator authorization" for live writes | Parked deliberately at P3 with a written reassess trigger: "only for an explicit direct-PDS use case" |

The evidence says the operator's expectation is correct: Leaflet-for-
everything is the intended and expected path forward, the credential
situation (one website cookie, no ATProto secrets) is exactly what that
design calls for, and the direct-PDS alternative is parked, documented, and
one deliberate decision away if a concrete need for chosen rkeys or
PDS-only lifecycle ever surfaces.

## Verification note

Every behavioral claim above was checked against the pinned clean Leaflet
archive at `600c7b5f1f7b31b66fb626c293cc104312a12caf` (`git status` clean,
HEAD verified before reading) or the current Leafbud working tree, with
file:line citations inline. Claims about operator-local state (files present
or absent under `~/.config/leafbud/`) were checked directly; secret values
were inspected only for shape, never reproduced. Deployed-behavior claims
beyond the pinned revision (e.g., future deploys' action IDs) are explicitly
framed as discovery-guarded uncertainties, not verified facts. Nothing
contradicted the parent session's summary; the one addition is the dormant
app-password machinery documented above.
