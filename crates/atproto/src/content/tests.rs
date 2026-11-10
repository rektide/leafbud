use super::*;
use serde_json::json;

#[test]
fn independent_js_canonical_bytes_and_cids_not_a_self_roundtrip() {
    let vectors: Value = serde_json::from_str(include_str!("../../tests/fixtures/record-cids.json")).unwrap();
    assert_eq!(vectors.as_array().unwrap().len(), 4);
    for vector in vectors.as_array().unwrap() {
        let record: Value = serde_json::from_str(vector["record_json"].as_str().unwrap()).unwrap();
        let cid = vector["cid"].as_str().unwrap();
        let bytes = at_data(&record, 0).unwrap().to_dag_cbor().unwrap();
        let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(hex, vector["dag_cbor_hex"], "{}", vector["name"]);
        assert_eq!(record_content_cid(&record).unwrap(), cid);
        let verified = verify_record_content(&record, cid).unwrap();
        assert_eq!(verified.cid(), cid);
        assert_eq!(verified.dag_cbor_bytes(), bytes.len());
        let mut changed = record.clone();
        changed["another"] = json!(true);
        assert_eq!(verify_record_content(&changed, cid), Err(Error::DigestMismatch));
    }
}

#[test]
fn invalid_at_data_refuses_instead_of_permissive_bytes_or_plain_maps() {
    for (value, error) in [
        (json!([]), Error::RecordMap),
        (json!({"x":1.0}), Error::Integer),
        (json!({"x":-0.0}), Error::Integer),
        (json!({"x":0.5}), Error::Integer),
        (json!({"x":9223372036854775808u64}), Error::Integer),
        (json!({"x":{"$bytes":false}}), Error::SpecialValue),
        (json!({"x":{"$bytes":"AA==","extra":1}}), Error::SpecialValue),
        (json!({"x":{"$link":42}}), Error::SpecialValue),
        (json!({"x":{"$link":"invalid"}}), Error::CidProfile),
        (json!({"x":{"$bytes":"__8="}}), Error::Bytes),
        (json!({"x":{"$bytes":"A==="}}), Error::Bytes),
        (json!({"x":{"$bytes":"AB=="}}), Error::Bytes),
        (json!({"$future":true}), Error::ReservedField),
        (json!({"$type":false}), Error::SpecialValue),
    ] {
        assert_eq!(record_content_cid(&value), Err(error), "{value}");
    }
    assert_eq!(
        record_content_cid(&json!({"x":{"$bytes":"AA=="}})).unwrap(),
        record_content_cid(&json!({"x":{"$bytes":"AA"}})).unwrap()
    );
    // $link must not silently hash as a plain map or as a CID-looking string.
    let cid = "bafyreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku";
    assert_ne!(
        record_content_cid(&json!({"x":{"$link":cid}})).unwrap(),
        record_content_cid(&json!({"x":cid})).unwrap()
    );
    assert!(verify_record_content(&json!({}), "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku").is_err_and(|e| e == Error::RecordCodec));
    let base: Cid = cid.parse().unwrap();
    for bad in [
        base.to_string_of_base(cid::multibase::Base::Base58Btc).unwrap(),
        cid.to_uppercase(),
        Cid::new_v1(0x70, *base.hash()).to_string(),
        Cid::new_v1(0x71, Multihash::wrap(0x1e, &[0; 32]).unwrap()).to_string(),
        Cid::new_v1(0x71, Multihash::wrap(0x12, &[0; 16]).unwrap()).to_string(),
    ] {
        assert_eq!(verify_record_content(&json!({}), &bad), Err(Error::CidProfile));
    }
}
