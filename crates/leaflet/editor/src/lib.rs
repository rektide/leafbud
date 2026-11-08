//! springleaf-editor — the Leaflet editor surface.
//!
//! Destination ownership (accepted design,
//! [`design/authoring-interface0.gpt6a.md`](../../../design/authoring-interface0.gpt6a.md) §2):
//! fact-graph and text decoding, durable bindings, bidirectional mapping,
//! scoped witnesses, and pure edit planning over already-fetched state and
//! supplied resource results. It depends on the document model and, when
//! implemented, Yrs; it accepts fetched state rather than performing
//! network effects itself (those belong to `springleaf-web`).
//!
//! **Status: scaffold only — nothing here is implemented.** The first
//! integration slice (editor fixture → keyed MDX → edit → same remote
//! objects → pull, accepted design §4) **promotes proven Yrs-based
//! paths deliberately from the isolated experiments**: 05 (Rust ↔ real
//! Yjs/y-prosemirror conversion on a pinned upstream Yrs), 13 (keyed
//! MDX model, planned native mutation arguments, fresh-Yrs text decode,
//! fact scanning), and 14 (the explicit `{#key}`/brace adapter seam).
//! Promotion is a reviewed lift of experiment code into this crate,
//! distinct from production path dependencies: the `yrs` crate lands
//! only when promoted code needs it. The pre-reset hand-rolled Yjs
//! codec (revision `80436b61`, `crates/drafts/src/yjs.rs`) is
//! historical oracle/fixture evidence — the wire-shape and byte-parity
//! reference for tests — not this crate's text architecture; fractional
//! positioning and the push transport shape are selectively recoverable
//! there. Any recovered fact scanning must retain raw payloads (the old
//! `FactData::Other` drops them, per the accepted design).
