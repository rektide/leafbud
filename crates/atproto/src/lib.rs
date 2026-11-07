//! springleaf-atproto — a thin, hand-rolled ATProto client surface.
//!
//! Deliberately minimal per the jacquard verdict
//! (`experiments/04-jacquard/REPORT.md` §6): four XRPC calls over reqwest,
//! identity resolution following the chain proven by
//! `experiments/03-post-builder/REPORT.md` §1, typed errors, and no client
//! machinery beyond that. Record types live in `springleaf-leaflet`
//! (jacquard-generated); this crate stays lexicon-agnostic JSON.
//!
//! # Identity resolution
//!
//! `Resolver::resolve` accepts a handle or a full DID:
//!
//! 1. handle → DID: DNS TXT `_atproto.<handle>` (system resolver, UDP;
//!    errors fall through), then HTTPS `https://<handle>/.well-known/atproto-did`
//!    (bsky.social-hosted handles use this path).
//! 2. DID → DID document: `did:plc:` via `https://plc.directory/<did>`
//!    (the modern PLC host; `plc.bsky.social` 404s), `did:web:` via
//!    `https://<domain>/.well-known/did.json`. `did:webvh` needs a log
//!    verifier and is rejected as unsupported for now.
//! 3. The document's `#atproto_pds` service (`AtprotoPersonalDataServer`)
//!    gives the PDS endpoint.
//!
//! When the input was a handle, `ResolvedIdentity.handle_binding_verified`
//! records whether the DID document's `alsoKnownAs` contains
//! `at://<handle>` (bidirectional verification, experiment 03 step 4).
//!
//! # Sessions and records
//!
//! [`XrpcClient`] is bound to one PDS endpoint and is **stateless with
//! respect to secrets**: [`XrpcClient::create_session`] returns a
//! [`Session`] whose JWTs are never logged, printed, or embedded in error
//! messages (see its `Debug` impl), and authenticated calls take the
//! session explicitly. Public reads ([`XrpcClient::list_records`],
//! [`XrpcClient::list_all_records`], [`XrpcClient::get_record`]) need no
//! session at all.
//!
//! # Stage 4 (not built here)
//!
//! Writes arrive in stage 4, on this client, with the same shape:
//!
//! - `put_record(session, repo, collection, rkey, record)` —
//!   `com.atproto.repo.putRecord` (upsert; `putRecord`, not `createRecord`,
//!   per the design doc), returning `{ uri, cid }`;
//! - `upload_blob(session, bytes, mime_type)` —
//!   `com.atproto.repo.uploadBlob` with a raw Content-Type body;
//! - session persistence with 0600 permissions under springleaf's own
//!   config (jacquard's `FileAuthStore` lacks 0600 hardening — we own it).
//!
//! They are intentionally absent: no stub bodies, no dead code.
//!
//! Token/secret hygiene: nothing in this crate logs or embeds the app
//! password, `accessJwt`, or `refreshJwt`. Transport errors surface only
//! URLs; XRPC error bodies surface their `error`/`message` fields.

use std::sync::LazyLock;

use serde::Deserialize;
use thiserror::Error;

pub mod dns;
pub mod identity;
pub mod xrpc;

pub use identity::{Did, Handle, ResolvedIdentity, Resolver};
pub use xrpc::{ListRecordsPage, Record, Session, XrpcClient};

/// Errors across the crate. Secrets never appear in any variant.
#[derive(Debug, Error)]
pub enum Error {
    #[error("transport error")]
    Transport(#[from] reqwest::Error),
    #[error("{method} returned HTTP {status}")]
    HttpStatus { method: String, status: u16 },
    #[error("{method} XRPC error {code}: {message}")]
    Xrpc { method: String, code: String, message: String },
    #[error("{0} response decode failed: {1}")]
    Decode(String, #[source] serde_json::Error),
    #[error("identity resolution for {input} failed: {reason}")]
    Identity { input: String, reason: String },
    #[error("{0}")]
    BadInput(String),
}

/// Required NSID shape check (`a.b.c` dotted identifiers), enough to reject
/// nonsense collections before they hit the wire.
static NSID_RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9-]*(\.[a-zA-Z0-9][a-zA-Z0-9-]*)+$").expect("nsid regex"));

/// Validate an NSID-shaped string (collection name).
pub fn validate_nsid(s: &str) -> Result<(), Error> {
    if s.len() <= 317 && NSID_RE.is_match(s) {
        Ok(())
    } else {
        Err(Error::BadInput(format!("invalid NSID {s:?}")))
    }
}

/// An at-uri `at://<authority>/<collection>/<rkey>`, split into parts.
///
/// Authority may be a DID or a handle; both shapes are accepted here
/// (callers that need a DID authority should resolve handles first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtUri {
    pub authority: String,
    pub collection: String,
    pub rkey: String,
}

static ATURI_RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^at://([^/\s]+)/([^/\s]+)/([^/\s]+)$").expect("at-uri regex"));

impl AtUri {
    pub fn parse(s: &str) -> Result<Self, Error> {
        let caps = ATURI_RE.captures(s).ok_or_else(|| Error::BadInput(format!("invalid at-uri {s:?}")))?;
        Ok(AtUri {
            authority: caps[1].to_string(),
            collection: caps[2].to_string(),
            rkey: caps[3].to_string(),
        })
    }
}

impl std::fmt::Display for AtUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at://{}/{}/{}", self.authority, self.collection, self.rkey)
    }
}

/// Raw `listRecords` item as returned by the PDS.
#[derive(Debug, Clone, Deserialize)]
pub struct RawRecord {
    pub uri: String,
    pub cid: String,
    pub value: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_uri_acceptance() {
        for ok in [
            "at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.publication/3molsbwkh522o",
            "at://example.com/site.standard.document/my-post",
            "at://did:web:example.com/pub.leaflet.document/3abcxyz",
        ] {
            let uri = AtUri::parse(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
            assert_eq!(uri.to_string(), ok);
        }
        for bad in [
            "at://did:plc:x/site.standard.publication", // missing rkey
            "https://example.com/x",
            "at://did with space/site.standard.document/r",
            "at://did:plc:x//r",
            "",
        ] {
            assert!(AtUri::parse(bad).is_err(), "{bad} should not parse");
        }
        let uri = AtUri::parse("at://did:plc:zjbq26wybii5ojoypkso2mso/site.standard.publication/3molsbwkh522o").unwrap();
        assert_eq!(uri.authority, "did:plc:zjbq26wybii5ojoypkso2mso");
        assert_eq!(uri.collection, "site.standard.publication");
        assert_eq!(uri.rkey, "3molsbwkh522o");
    }

    #[test]
    fn nsid_acceptance() {
        for ok in ["site.standard.publication", "pub.leaflet.document", "com.atproto.repo.getRecord", "a-b.c-d.e"] {
            validate_nsid(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
        for bad in ["", "nodots", ".a.b", "a..b", "a.b.", "-x.y", "a/b"] {
            assert!(validate_nsid(bad).is_err(), "{bad} should be rejected");
        }
    }
}
