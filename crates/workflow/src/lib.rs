//! springleaf-workflow — the application workflow layer.
//!
//! Destination ownership (accepted design,
//! [`design/authoring-interface0.gpt6a.md`](../../../design/authoring-interface0.gpt6a.md) §2):
//! composes the libraries — source acquisition, configuration, workspace
//! inventory, planning, resource acquisition, pending receipts, execution,
//! and verification — and offers the same operations to the CLI and other
//! Rust callers. Business policy and file lifecycle live here, not in the
//! `springleaf` binary.
//!
//! **Status: scaffold only — nothing here is implemented.** The first
//! slice recovers the proven figment2/toml_edit configuration layering
//! from the pre-reset CLI (revision `80436b61`: `config.rs`,
//! `config_cmd.rs`) into this crate. The old drafts/posted sidecar
//! lifecycle is intentionally not recovered: the accepted YAML frontmatter
//! envelope ([`design/frontmatter0.gpt61s.md`](../../design/frontmatter0.gpt61s.md))
//! replaces it, and no business workflow is pre-built here.
