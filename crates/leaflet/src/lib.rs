//! springleaf-leaflet — Leaflet record vocabulary for springleaf.
//!
//! Two layers:
//!
//! - **Generated lexicon bindings** (crate-root modules `com_atproto`,
//!   `pub_leaflet`, `site_standard`, `builder_types`): typed, validating
//!   record types produced by `jacquard-codegen` from the SHA-pinned
//!   leaflet lexicon checkout. The generated tree is committed — the
//!   committed code IS the pin. Regeneration procedure, source SHA, and
//!   runtime dependency pins: [`LEAFLET-PIN.md`] in the crate root.
//! - **Hand-written code** (`markdown`, `record`, `publication`): the
//!   markdown→blocks converter at social-cli's v0 subset (paragraphs, ATX
//!   headings, link/bold/italic/code facets with UTF-8 **byte** offsets),
//!   the frontmatter → `site.standard.document` record builder with the
//!   `path == "/" + rkey` invariant, the published-URL derivation rules
//!   (`{pubDomain}/{slug}`, else the `/lish/{did}/{pubRkey}/{rkey}`
//!   fallback), and publication-record lookup for those URL derivations.
//!   Proven by `experiments/03-post-builder/`.
//!
//! One wire-shape adaptation sits at the boundary (see `record`): jacquard
//! collapses single-variant lexicon unions to the bare struct, so the
//! `$type` discriminators on `content` (`pub.leaflet.content`) and on
//! block wrappers (`pub.leaflet.pages.linearDocument#block`) — which real
//! records and experiment 03 both carry — are re-injected after
//! serialization. Deserialization needs no help: unknown `$type` fields
//! land in the generated `extra_data` maps.

extern crate alloc;

// ── Generated lexicon bindings (DO NOT EDIT — see LEAFLET-PIN.md) ─────────
// Declarations mirror the codegen's lib.rs (feature-gated namespaces +
// builder_types). The allows scope generated-code lint noise (unused
// imports, camelCase NSID function names, builder idioms) to these
// modules only; hand-written code stays fully lint-clean.

#[cfg(feature = "com_atproto")]
#[allow(unused_imports, non_snake_case, clippy::all)]
pub mod com_atproto;

#[cfg(feature = "pub_leaflet")]
#[allow(unused_imports, non_snake_case, clippy::all)]
pub mod pub_leaflet;

#[cfg(feature = "site_standard")]
#[allow(unused_imports, non_snake_case, clippy::all)]
pub mod site_standard;

#[allow(unused_imports, non_snake_case, clippy::all)]
pub mod builder_types;

// ── Hand-written layers ────────────────────────────────────────────────────

pub mod markdown;
pub mod publication;
pub mod record;

/// The generated types' default string backing, re-exported so callers
/// can name `Type<DefaultStr>` without a direct jacquard dependency.
pub use jacquard_common::DefaultStr;

pub use markdown::{MarkdownError, convert_markdown};
pub use publication::{PublicationError, PublicationInfo, publication_for_record};
pub use record::{
    BuildError, BuildOptions, BuiltRecord, Frontmatter, ValidationError, build_record, derive_document_url, parse_frontmatter, reinject_type_discriminators,
    slugify, standalone_site_did, validate_document_record,
};
