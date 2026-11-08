//! springleaf-document — the pure document model.
//!
//! Destination ownership (accepted design,
//! [`design/authoring-interface0.gpt6a.md`](../../../design/authoring-interface0.gpt6a.md) §2):
//! current authoring semantics, stable source IDs, diagnostics, and
//! compact tracking data — including explicitly Leaflet-specific locators
//! where needed. It is network- and parser-independent and is not a
//! promise of a universal document SDK.
//!
//! **Status: scaffold only — nothing here is implemented.** The model's
//! concrete types arrive with the first integration slice (editor fixture
//! → keyed MDX → edit → same remote objects → pull, accepted design §4),
//! shaped by the frontmatter envelope ([`design/frontmatter0.gpt61s.md`](../../../design/frontmatter0.gpt61s.md))
//! and the keyed bidirectional addendum
//! ([`design/authoring-handoff0.gpt61s.md`](../../../design/authoring-handoff0.gpt61s.md) §12).
//! No speculative type hierarchy is pre-built.
