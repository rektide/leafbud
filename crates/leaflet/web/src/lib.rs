//! springleaf-web — the Leaflet web effects surface.
//!
//! Destination ownership (accepted design,
//! [`design/authoring-interface0.gpt6a.md`](../../../design/authoring-interface0.gpt6a.md) §2):
//! cookie auth, native discovery, RPC/action execution, uploads, and
//! native publication. It executes native requests and returns
//! observations/outcomes; it parses no MDX and loads no workspace files.
//! Public reads go through the ATProto client ([`springleaf-atproto`](https://docs.rs/springleaf-atproto))
//! where appropriate.
//!
//! **Status: scaffold only — nothing here is implemented.** Proven
//! recovery candidates from the pre-reset `springleaf-drafts` crate
//! (revision `80436b61`): the cookie-authed `LeafletClient` (`/home` RSC
//! parse, `get_leaflet_data`/`pull`/`push` RPCs, `/new` minting), plus the
//! publication-record lookup helper that previously lived in the record
//! crate. The Replicache push transport returns here when the append
//! plumbing (`remote append`) or native save is implemented.
