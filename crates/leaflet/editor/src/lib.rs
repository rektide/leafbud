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
//! **Status: scaffold only — nothing here is implemented.** The Yrs
//! dependency is deliberately deferred until the first integration slice
//! (editor fixture → keyed MDX → edit → same remote objects → pull,
//! accepted design §4). Proven recovery candidates from the pre-reset
//! `springleaf-drafts` crate (revision `80436b61`) for that slice: the
//! hand-rolled Yjs text codec (`yjs.rs`), fractional positioning
//! (`fractional.rs`), and the fact-row scan index (`facts.rs` — its
//! `FactData::Other` variant must be replaced with raw-payload-retaining
//! observations, per the accepted design).
