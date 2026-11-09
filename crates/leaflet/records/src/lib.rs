//! leafbud-records — the Leaflet public-record vocabulary for leafbud.
//!
//! Destination ownership (accepted design,
//! [`design/authoring-interface0.gpt6a.md`](../../../design/authoring-interface0.gpt6a.md) §2):
//! generated public types, validation, and the document-to-public record
//! projection. Editor facts and Yjs text state are **not** public records;
//! they belong to `leafbud-editor`. This crate parses no MDX, touches no
//! filesystem, and performs no network effects; the one public-read helper
//! that used to live beside these types (publication-record lookup) resets
//! with the old lifecycle and re-homes under `leafbud-web` when
//! publication discovery is implemented.
//!
//! Two layers:
//!
//! - **Generated lexicon bindings** (crate-root modules `com_atproto`,
//!   `pub_leaflet`, `site_standard`, `builder_types`): typed, validating
//!   record types produced by `jacquard-codegen` from the SHA-pinned
//!   leaflet lexicon checkout. The generated tree is committed — the
//!   committed code IS the pin. Regeneration procedure, source SHA, and
//!   runtime dependency pins: [`LEAFLET-PIN.md`] in the crate root.
//! - **Hand-written wire helpers** ([`record`]): validation of record JSON
//!   through the generated type, the `$type` discriminator re-injection
//!   that keeps serialized records faithful to what real servers write,
//!   and the proven public-URL derivation rules for document rkeys.
//!
//! One wire-shape adaptation sits at the boundary: jacquard collapses
//! single-variant lexicon unions to the bare struct, so the `$type`
//! discriminators on `content` (`pub.leaflet.content`) and on block
//! wrappers (`pub.leaflet.pages.linearDocument#block`) — which real
//! records and the committed fixtures both carry — are re-injected after
//! serialization ([`record::reinject_type_discriminators`]).
//! Deserialization needs no help: unknown `$type` fields land in the
//! generated `extra_data` maps.

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

// ── Hand-written layer ─────────────────────────────────────────────────────

pub mod record;

/// The generated types' default string backing, re-exported so callers
/// can name `Type<DefaultStr>` without a direct jacquard dependency.
pub use jacquard_common::DefaultStr;

pub use record::{UrlError, ValidationError, derive_document_url, reinject_type_discriminators, standalone_site_did, validate_document_record};
