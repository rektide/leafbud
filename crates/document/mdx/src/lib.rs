//! springleaf-mdx — MDX reading and writing.
//!
//! Destination ownership (accepted design,
//! [`design/authoring-interface0.gpt6a.md`](../../../design/authoring-interface0.gpt6a.md) §2):
//! Satteri parsing, the restricted YAML frontmatter profile, source-key
//! insertion, and MDX serialization. It depends on the document model
//! ([`springleaf-document`](https://docs.rs/springleaf-document)), retains
//! source bytes, and produces validated models or diagnostics and
//! proposed byte edits. It performs no filesystem or network effects and
//! does not expose Satteri arenas as its domain API.
//!
//! **Status: scaffold only — nothing here is implemented.** The Satteri
//! dependency is deliberately deferred until the first integration slice
//! proves the parsing/keying contract (experiments 13/14 own the current
//! evidence; `design/authoring-recommendation0.gpt61s.md` records the
//! parser selection).
