//! Record **content addressing** only. No signature, repo, actor, timestamp,
//! linkage, lexicon-validity or delivery verification is implied.
use base64::{
    Engine, alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
};
use cid::{Cid, Version, multihash::Multihash};
use jacquard_common::types::{cid::Cid as DataCid, value::RawData};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("record content must be an AT data map")]
    RecordMap,
    #[error("AT data numbers must be signed 64-bit integers, not floats")]
    Integer,
    #[error("invalid AT JSON special value (expected singleton $link or $bytes)")]
    SpecialValue,
    #[error("unsupported reserved AT data field")]
    ReservedField,
    #[error("CID is not canonical base32lower CIDv1/SHA-256/32-byte AT profile")]
    CidProfile,
    #[error("record CID codec must be dag-cbor")]
    RecordCodec,
    #[error("invalid RFC4648 standard base64 bytes")]
    Bytes,
    #[error("AT data nesting exceeds verification profile")]
    Nesting,
    #[error("canonical DAG-CBOR encoding failed")]
    Encoding,
    #[error("reported record CID does not match canonical record content")]
    DigestMismatch,
}

/// Evidence for exactly the supplied value at the time of verification. Keep it
/// with that immutable value; this is not a receipt or a claim about future reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedContent {
    cid: String,
    dag_cbor_bytes: usize,
}
impl VerifiedContent {
    pub fn cid(&self) -> &str {
        &self.cid
    }
    pub fn dag_cbor_bytes(&self) -> usize {
        self.dag_cbor_bytes
    }
}

/// Strict AT JSON -> library-owned canonical DAG-CBOR -> SHA-256 -> CIDv1.
/// Plain CID-looking strings remain strings (e.g. strongRef.cid). Only actual
/// singleton $link/$bytes wrappers become tag-42 links / byte strings.
/// Unknown reserved fields refuse this verification profile, never disappear.
pub fn record_content_cid(record: &Value) -> Result<String, Error> {
    Ok(encoded(record)?.0.to_string())
}

pub fn verify_record_content(record: &Value, reported_cid: &str) -> Result<VerifiedContent, Error> {
    let reported = parse_cid(reported_cid)?;
    if reported.codec() != 0x71 {
        return Err(Error::RecordCodec);
    }
    let (actual, bytes) = encoded(record)?;
    if actual != reported {
        return Err(Error::DigestMismatch);
    }
    Ok(VerifiedContent {
        cid: actual.to_string(),
        dag_cbor_bytes: bytes,
    })
}

fn parse_cid(value: &str) -> Result<Cid, Error> {
    let cid: Cid = value.parse().map_err(|_| Error::CidProfile)?;
    if cid.version() != Version::V1
        || !matches!(cid.codec(), 0x55 | 0x71)
        || cid.hash().code() != 0x12
        || cid.hash().size() != 32
        || !value.starts_with('b')
        || cid.to_string() != value
    {
        return Err(Error::CidProfile);
    }
    Ok(cid)
}

fn encoded(record: &Value) -> Result<(Cid, usize), Error> {
    if !record.is_object() {
        return Err(Error::RecordMap);
    }
    let data = at_data(record, 0)?;
    if !matches!(data, RawData::Object(_)) {
        return Err(Error::RecordMap);
    }
    // Jacquard's permissive JSON deserializer can retain InvalidNumber/InvalidData
    // (serialized as bytes!) and string-only CIDs (serialized as text!). Build only
    // validated variants, with a parsed Ipld CID, instead of that permissive path.
    let bytes = data.to_dag_cbor().map_err(|_| Error::Encoding)?;
    let hash = Multihash::<64>::wrap(0x12, &Sha256::digest(&bytes)).map_err(|_| Error::Encoding)?;
    Ok((Cid::new_v1(0x71, hash), bytes.len()))
}

fn at_data(value: &Value, depth: usize) -> Result<RawData<'_>, Error> {
    if depth > 128 {
        return Err(Error::Nesting);
    }
    Ok(match value {
        Value::Null => RawData::Null,
        Value::Bool(value) => RawData::Boolean(*value),
        Value::Number(value) => RawData::SignedInt(value.as_i64().ok_or(Error::Integer)?),
        Value::String(value) => RawData::String(value.as_str().into()),
        Value::Array(values) => RawData::Array(values.iter().map(|v| at_data(v, depth + 1)).collect::<Result<_, _>>()?),
        Value::Object(values) => {
            if values.contains_key("$link") || values.contains_key("$bytes") {
                if values.len() != 1 {
                    return Err(Error::SpecialValue);
                }
                if let Some(value) = values.get("$link") {
                    RawData::CidLink(DataCid::ipld(parse_cid(value.as_str().ok_or(Error::SpecialValue)?)?))
                } else {
                    let value = values["$bytes"].as_str().ok_or(Error::SpecialValue)?;
                    let engine = GeneralPurpose::new(
                        &alphabet::STANDARD,
                        GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
                    );
                    RawData::Bytes(engine.decode(value).map_err(|_| Error::Bytes)?.into())
                }
            } else {
                if values.keys().any(|key| key.starts_with('$') && key != "$type") {
                    return Err(Error::ReservedField);
                }
                if values.get("$type").is_some_and(|value| !value.is_string()) {
                    return Err(Error::SpecialValue);
                }
                RawData::Object(
                    values
                        .iter()
                        .map(|(k, v)| Ok((k.as_str().into(), at_data(v, depth + 1)?)))
                        .collect::<Result<_, Error>>()?,
                )
            }
        }
    })
}

#[cfg(test)]
mod tests;
