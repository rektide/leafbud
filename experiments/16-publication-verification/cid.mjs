#!/usr/bin/env node
// Independent fixed vectors: no Leafbud/Rust encoder is called here.
import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import * as cbor from "@ipld/dag-cbor";
import { CID } from "multiformats/cid";
import { sha256 } from "multiformats/hashes/sha2";

const link = "bafyreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku";
const cases = [
  ["unicode-length-first-key-order", '{"zz":1,"a":"🦋 café","é":"雪","$type":"example.test.record","aa":false}',
    { zz: 1, a: "🦋 café", é: "雪", $type: "example.test.record", aa: false }],
  ["cid-tag42-bytes-plain-strongref", JSON.stringify({ $type: "example.test.record", link: { $link: link }, bytes: { $bytes: "AP/+" }, ref: { cid: link, uri: "at://did:plc:abc/example.test.record/r" }, empty: { $bytes: "" } }),
    { $type: "example.test.record", link: CID.parse(link), bytes: Uint8Array.from([0, 255, 254]), ref: { cid: link, uri: "at://did:plc:abc/example.test.record/r" }, empty: new Uint8Array() }],
  ["signed-integer-boundaries", '{"$type":"example.test.record","values":[-9223372036854775808,9223372036854775807,-9007199254740991,9007199254740991,-25,-24,-1,0,23,24,255,256,65535,65536,4294967295,4294967296]}',
    { $type: "example.test.record", values: [-9223372036854775808n, 9223372036854775807n, -9007199254740991, 9007199254740991, -25, -24, -1, 0, 23, 24, 255, 256, 65535, 65536, 4294967295, 4294967296] }],
  ["nested-null-false-empty", '{"$type":"example.test.record","n":null,"f":false,"s":"","a":[{},[],true,{"longer":0,"z":null}]}',
    { $type: "example.test.record", n: null, f: false, s: "", a: [{}, [], true, { longer: 0, z: null }] }],
  ["modern-blob-raw-cid", '{"$type":"example.test.record","blob":{"$type":"blob","ref":{"$link":"bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"},"mimeType":"application/octet-stream","size":0}}',
    { $type: "example.test.record", blob: { $type: "blob", ref: CID.parse("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"), mimeType: "application/octet-stream", size: 0 } }],
];
const vectors = [];
for (const [name, record_json, data] of cases) {
  const bytes = cbor.encode(data);
  const digest = await sha256.digest(bytes);
  vectors.push({ name, record_json, dag_cbor_hex: Buffer.from(bytes).toString("hex"), cid: CID.createV1(cbor.code, digest).toString() });
}
// Lexical-only sorting would start $type, whereas canonical DAG-CBOR starts a.
assert.ok(vectors[0].dag_cbor_hex.startsWith("a56161"));
assert.ok(vectors[1].dag_cbor_hex.includes("d82a58250001711220"));
const path = new URL("../../crates/atproto/tests/fixtures/record-cids.json", import.meta.url);
if (process.argv.includes("--write")) await writeFile(path, JSON.stringify(vectors, null, 2) + "\n");
else assert.deepEqual(JSON.parse(await readFile(path, "utf8")), vectors);
console.log(`ok - ${vectors.length} independent canonical DAG-CBOR/CID vectors (BigInt, Unicode, tag42, bytes)`);
