#!/usr/bin/env node
// Offline interoperability runner. No network API, cookie, or production code imports.
import assert from "node:assert/strict";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import * as Y from "yjs";
import { Schema } from "prosemirror-model";
import { prosemirrorJSONToYXmlFragment, yXmlFragmentToProsemirrorJSON } from "y-prosemirror";

const run = promisify(execFile);
const dir = fileURLToPath(new URL(".", import.meta.url));
const bytes = (s) => Uint8Array.from(Buffer.from(s, "base64"));
const b64 = (b) => Buffer.from(b).toString("base64");
const full = (d) => b64(Y.encodeStateAsUpdate(d));
const prefix = "pub.leaflet.richtext.facet#";
const feature = (type, rest = {}) => ({ $type: prefix + type, ...rest });
const facet = (a,b,...features) => ({index:{byteStart:a,byteEnd:b},features});
const canonical = (plaintext, facets = []) => ({plaintext,facets});

// Structural/mark subset of Leaflet schema.ts @600c7b5f; intentionally no DOM/UI imports.
const schema = new Schema({
  nodes:{doc:{content:"block"},paragraph:{content:"inline*",group:"block"},text:{group:"inline"},
    hard_break:{group:"inline",inline:true,selectable:false}},
  marks:{strong:{},em:{},code:{},underline:{},strikethrough:{},
    highlight:{attrs:{color:{default:"1"}}},link:{attrs:{href:{}},inclusive:false},
    comment:{attrs:{commentID:{}},inclusive:false}},
});
function make(content, client=1001) {
  const d=new Y.Doc();d.clientID=client;
  prosemirrorJSONToYXmlFragment(schema,{type:"doc",content:[{type:"paragraph",content}]},d.getXmlFragment("prosemirror"));
  d.getMap("meta").set("schemaVersion",1);
  return d;
}
const text=(text,marks=[])=>({type:"text",text,marks});
function from(update, client=8001) {const d=new Y.Doc();d.clientID=client;Y.applyUpdate(d,bytes(update));return d;}
const paragraph=(d)=>d.getXmlFragment("prosemirror").get(0);

// Independent projection using JS strings (UTF-16) and Buffer UTF-8 byte lengths.
// Produces a normalized facet list, preserving overlapping mark sets.
function project(d) {
  const root=d.getXmlFragment("prosemirror");assert.equal(root.length,1);
  const p=root.get(0);assert.equal(p.nodeName,"paragraph");
  const out=canonical("");
  const add=(s,attributes={})=>{
    const fs=[];
    for(const [key,name] of [["strikethrough","strikethrough"],["code","code"],["highlight","highlight"],["underline","underline"],["strong","bold"],["em","italic"],["link","link"]]) {
      if(attributes[key]) fs.push(feature(name,key==="link"?{uri:attributes.link.href}:{}));
    }
    const start=Buffer.byteLength(out.plaintext);out.plaintext+=s;
    if(!fs.length)return;
    const end=Buffer.byteLength(out.plaintext);const last=out.facets.at(-1);
    if(last && last.index.byteEnd===start && JSON.stringify(last.features)===JSON.stringify(fs))last.index.byteEnd=end;
    else out.facets.push(facet(start,end,...fs));
  };
  for(const n of p.toArray()) {
    if(n instanceof Y.XmlText)for(const delta of n.toDelta()){assert.equal(typeof delta.insert,"string");add(delta.insert,delta.attributes);}
    else {assert.equal(n.nodeName,"hard_break");add("\n");}
  }
  return out;
}
async function rust(input, {failure}={}) {
  // execFile doesn't accept stdin data; use the returned child's stdin, retaining promise errors.
  const promise=run("./target/debug/yrs-conversion-experiment",[],{cwd:dir,maxBuffer:4*1024*1024});
  promise.child.stdin.end(typeof input==="string"?input:JSON.stringify(input));
  try {const {stdout}=await promise;assert.equal(failure,undefined,"expected request failure");return JSON.parse(stdout);}
  catch(e){if(failure){assert.match(e.stderr??"",failure);return;}throw e;}
}

const frozen1="0103d893a0d3030007010b70726f73656d6972726f7203097061726167726170680700d893a0d30300060400d893a0d303010b48656c6c6f20776f726c6400";
// Sample 2 is read directly from the older committed fixture, avoiding a second transcription.
const annotated=await readFile(new URL("../02-draft-internals/fixtures/yjs-samples.md",import.meta.url),"utf8");
const sample2=annotated.match(/hex: `([0-9a-f]+)`/)[1];
const rich=make([
  text("Aé"),text("😀",[{type:"strong"}]),text("e\u0301",[{type:"strong"},{type:"em"}]),
  text(" link",[{type:"link",attrs:{href:"https://example.test/old"}}]),{type:"hard_break"},
  text("tail",[{type:"underline"},{type:"comment",attrs:{commentID:"synthetic-comment"}}]),
]);
const generated={
  provenance:{yjs:"13.6.15",yProsemirror:"1.2.5",leaflet:"600c7b5f1f7b31b66fb626c293cc104312a12caf",kind:"synthetic; frozen samples from experiment 02 (Yjs 13.6.18), editor-rich generated with versions above; no live capture"},
  samples:[
    {name:"frozen-plain",update:b64(Buffer.from(frozen1,"hex")),expected:canonical("Hello world")},
    {name:"frozen-tombstone",update:b64(Buffer.from(sample2,"hex")),expected:canonical("Synthetic sample text")},
    {name:"editor-rich",update:full(rich),expected:project(rich)},
  ],
};
const ids={root:"01999000-0000-7000-8000-000000000001",page:"01999000-0000-7000-8000-000000000002",block:"01999000-0000-7000-8000-000000000003"};
const row=(n,entity,attribute,data)=>({id:`01999000-0000-7000-8000-${String(n).padStart(12,"0")}`,entity,attribute,data});
const graph={root:ids.root,patch:[{op:"clear"},...[ 
  row(11,ids.root,"root/page",{type:"ordered-reference",value:ids.page,position:"a0"}),
  row(12,ids.page,"page/type",{type:"page-type-union",value:"doc"}),
  row(13,ids.page,"card/block",{type:"ordered-reference",value:ids.block,position:"a0"}),
  row(14,ids.block,"block/type",{type:"block-type-union",value:"text"}),
  row(15,ids.block,"block/text",{type:"text",value:full(rich)}),
  row(16,ids.block,"block/comment",{type:"ordered-reference",value:"synthetic-comment",position:"a0"}),
].map(value=>({op:"put",key:value.id,value}))]};
if(process.argv.includes("--write-fixtures")) {
  await mkdir(new URL("fixtures/",import.meta.url),{recursive:true});
  await writeFile(new URL("fixtures/wire.json",import.meta.url),JSON.stringify(generated,null,2)+"\n");
  await writeFile(new URL("fixtures/pull.json",import.meta.url),JSON.stringify(graph,null,2)+"\n");
  console.log("Wrote deterministic synthetic fixtures with real Yjs/y-prosemirror.");process.exit(0);
}
const wire=JSON.parse(await readFile(new URL("fixtures/wire.json",import.meta.url),"utf8"));
assert.deepEqual(wire,generated,"committed wire fixture differs from deterministic generation");
let passed=0;
async function test(name, fn){await fn();passed++;console.log(`ok ${passed} - ${name}`);}

await test("frozen wire, tombstones, root names and editor-rich UTF-8 facets",async()=>{
  for(const s of wire.samples){const r=await rust({update:s.update});assert.deepEqual(r.canonical,s.expected);assert.deepEqual(project(from(s.update)),s.expected);}
  assert.equal(Buffer.from(wire.samples[0].update,"base64").length,63);
  const r=await rust({update:wire.samples[2].update});
  assert.deepEqual(r.roots,["meta","prosemirror"]);
  assert.deepEqual(r.children,[{type:"XmlText",utf16Length:11},{type:"XmlElement",tag:"hard_break"},{type:"XmlText",utf16Length:4}]);
  const old=JSON.parse(await readFile(new URL("../02-draft-internals/fixtures/push-minimal-authoring.json",import.meta.url),"utf8"));
  for(const m of old.request.pushRequest.mutations.filter(m=>m.args.attribute==="block/text")) {
    assert.deepEqual((await rust({update:m.args.data.value})).canonical,project(from(m.args.data.value)));
  }
});
const desired=canonical("Aé😀e\u0301 LINK\ntail!",[
  facet(3,10,feature("italic")),facet(11,15,feature("link",{uri:"https://example.test/new"})),
  facet(16,20,feature("underline")),
]);
let edited;
await test("JSON edit -> incremental yrs update -> Yjs + ProseMirror; retained identities/comments",async()=>{
  const base=wire.samples[2].update; const peer=from(base);
  const p=paragraph(peer), first=p.get(0), br=p.get(1), tail=p.get(2);
  const anchor=Y.createRelativePositionFromTypeIndex(first,2);
  edited=await rust({update:base,desired});
  assert.ok(edited.incrementalUpdate);assert.deepEqual(edited.canonical,desired);
  Y.applyUpdate(peer,bytes(edited.incrementalUpdate));
  assert.deepEqual(project(peer),desired);
  assert.equal(paragraph(peer),p);assert.equal(p.get(0),first);assert.equal(p.get(1),br);assert.equal(p.get(2),tail);
  assert.equal(Y.createAbsolutePositionFromRelativePosition(anchor,peer).index,2);
  assert.ok(tail.toDelta().some(d=>d.attributes?.comment?.commentID==="synthetic-comment"));
  const pm=yXmlFragmentToProsemirrorJSON(peer.getXmlFragment("prosemirror"));
  schema.nodeFromJSON(pm).check();
  assert.equal(pm.content[0].content.filter(n=>n.type==="hard_break").length,1);
  console.log(`  rich delta ${Buffer.from(edited.incrementalUpdate,"base64").length} bytes; full ${Buffer.from(edited.fullUpdate,"base64").length} bytes`);
});
await test("no-op with tombstones; repeated desired JSON; duplicate delta delivery",async()=>{
  for(const s of wire.samples)assert.equal((await rust({update:s.update,desired:s.expected})).incrementalUpdate,null);
  const peer=from(wire.samples[2].update);
  Y.applyUpdate(peer,bytes(edited.incrementalUpdate));Y.applyUpdate(peer,bytes(edited.incrementalUpdate));
  assert.deepEqual(project(peer),desired);
  assert.equal((await rust({update:full(peer),desired,client:9002})).incrementalUpdate,null);
});
await test("overlapping facets normalize; mark removal and changed link values",async()=>{
  const base=full(make([text("abcd",[{type:"strong"}])],1010));
  const target=canonical("abcd",[facet(0,3,feature("bold")),facet(1,4,feature("italic"))]);
  const r=await rust({update:base,desired:target}); const peer=from(base);Y.applyUpdate(peer,bytes(r.incrementalUpdate));
  assert.deepEqual(project(peer),canonical("abcd",[facet(0,1,feature("bold")),facet(1,3,feature("bold"),feature("italic")),facet(3,4,feature("italic"))]));
});
await test("all seven supported marks use real y-prosemirror attribute values",async()=>{
  const marks=[{type:"strong"},{type:"em"},{type:"code"},{type:"strikethrough"},{type:"underline"},
    {type:"highlight",attrs:{color:"1"}},{type:"link",attrs:{href:"https://example.test/marks"}}];
  const d=make([text("marked",marks)],1011);
  const r=await rust({update:full(d)});assert.deepEqual(r.canonical,project(d));
  assert.equal(r.canonical.facets[0].features.length,7);
  const clear=await rust({update:full(d),desired:canonical("marked")});
  Y.applyUpdate(d,bytes(clear.incrementalUpdate));assert.deepEqual(project(d),canonical("marked"));
});
await test("deletion-only update has effects despite unchanged state vector",async()=>{
  const base=full(make([text("abc")],1020));const peer=from(base);const sv=Y.encodeStateVector(peer);
  const r=await rust({update:base,desired:canonical("ac")});Y.applyUpdate(peer,bytes(r.incrementalUpdate));
  assert.deepEqual(Y.encodeStateVector(peer),sv);assert.equal(project(peer).plaintext,"ac");
});
await test("insert/delete XML hard breaks, empty text and Unicode edits",async()=>{
  let update=full(make([text("é😀x")],1030));
  for(const [i,s] of ["é😀\nx","\né😀\n\nx\n","é😀x","","e\u0301🦀"].entries()) {
    const target=canonical(s);const r=await rust({update,desired:target,client:9100+i});
    const peer=from(update);Y.applyUpdate(peer,bytes(r.incrementalUpdate));assert.deepEqual(project(peer),target);
    schema.nodeFromJSON(yXmlFragmentToProsemirrorJSON(peer.getXmlFragment("prosemirror"))).check();update=full(peer);
  }
});
await test("concurrent insertion survives known-character deletion; delivery orders converge",async()=>{
  const base=full(make([text("abc")],1040));const remote=from(base,1041);const sv=Y.encodeStateVector(remote);
  paragraph(remote).get(0).insert(1,"R");const remoteDelta=Y.encodeStateAsUpdate(remote,sv);
  const r=await rust({update:base,desired:canonical("ac")});
  const a=from(base),b=from(base);
  Y.applyUpdate(a,remoteDelta);Y.applyUpdate(a,bytes(r.incrementalUpdate));
  Y.applyUpdate(b,bytes(r.incrementalUpdate));Y.applyUpdate(b,remoteDelta);
  assert.deepEqual(project(a),project(b));assert.equal(project(a).plaintext,"aRc");
  assert.notDeepEqual(project(a),canonical("ac"));
  assert.deepEqual((await rust({update:full(a),client:9200})).canonical,project(a));
});
await test("unrelated fresh content is NOT replacement",async()=>{
  const old=make([text("old")],1050),fresh=make([text("new")],1051);
  const merged=from(b64(Y.mergeUpdates([Y.encodeStateAsUpdate(old),Y.encodeStateAsUpdate(fresh)])));
  assert.equal(merged.getXmlFragment("prosemirror").length,2);
  await rust({update:full(merged)},{failure:/exactly one paragraph/});
});
await test("JS surrogate splitting and unpaired input: wire replacement vs JSON rejection",async()=>{
  const d=make([text("a😀b")],1060);paragraph(d).get(0).insert(2,"Z");
  assert.equal(project(d).plaintext,"a�Z�b");assert.equal((await rust({update:full(d)})).canonical.plaintext,"a�Z�b");
  const lone=make([text("a\ud800b")],1061);
  assert.equal((await rust({update:full(lone)})).canonical.plaintext,"a�b");
  await rust(`{"update":${JSON.stringify(full(d))},"desired":{"plaintext":"\\ud800"}}`,{failure:/surrogate|unicode|hex escape/i});
});
await test("malformed input, bad boundaries, schema/inline/mark limits and v2 are refused",async()=>{
  await rust({update:b64(Buffer.from("b5ec6d","hex"))},{failure:/Yjs v1 decode/});
  await rust({update:wire.samples[2].update,desired:canonical("😀",[facet(1,4,feature("bold"))])},{failure:/UTF-8 facet/});
  const d=make([text("valid")],1070);d.getMap("meta").set("schemaVersion",2);
  await rust({update:full(d)},{failure:/schema version/});
  const mention=make([],1071);paragraph(mention).insert(0,[new Y.XmlElement("didMention")]);
  await rust({update:full(mention)},{failure:/unsupported inline/});
  const unknown=make([text("valid")],1072);paragraph(unknown).get(0).format(0,1,{futureMark:{}});
  await rust({update:full(unknown)},{failure:/unsupported mark/});
  await rust({update:b64(Y.encodeStateAsUpdateV2(make([text("v2")],1073)))},{failure:/Yjs v1 decode|missing prosemirror|incomplete update/});
  // An incremental payload alone is not a complete pulled document.
  await rust({update:edited.incrementalUpdate,client:9990},{failure:/incomplete update/});
});
await test("fact graph -> JSON -> one mutation -> simulated server merge -> read-back",async()=>{
  const pulled=JSON.parse(await readFile(new URL("fixtures/pull.json",import.meta.url),"utf8"));
  assert.deepEqual(pulled,graph);
  const rows=new Map();for(const p of pulled.patch){if(p.op==="clear")rows.clear();else if(p.op==="put")rows.set(p.key,p.value);else rows.delete(p.key);}
  const one=(entity,attr)=>[...rows.values()].find(f=>f.entity===entity&&f.attribute===attr);
  const page=one(pulled.root,"root/page").data.value;
  const edge=one(page,"card/block"),entity=edge.data.value,tf=one(entity,"block/text");
  const pulledText=(await rust({update:tf.data.value})).canonical;
  const local={"@leafspring":{version:1,target:{kind:"draft",root:ids.root},entities:{"/content/pages/0/blocks/0":{entity,edge:edge.id}}},
    $type:"site.standard.document",title:"Synthetic",content:{$type:"pub.leaflet.content",pages:[{$type:"pub.leaflet.pages.linearDocument",id:page,blocks:[{block:{$type:"pub.leaflet.blocks.text",...pulledText}}]}]}};
  Object.assign(local.content.pages[0].blocks[0].block,desired);
  const {"@leafspring":metadata,...body}=local;
  const {plaintext,facets}=body.content.pages[0].blocks[0].block;
  const planned=await rust({update:tf.data.value,desired:{plaintext,facets}});
  const mutation={id:1,name:"assertFact",clientID:"experiment",timestamp:0,args:{entity:metadata.entities["/content/pages/0/blocks/0"].entity,attribute:"block/text",data:{type:"text",value:planned.incrementalUpdate}}};
  assert.ok(!JSON.stringify(mutation).includes("@leafspring"));
  const untouched=[...rows.values()].filter(f=>f!==tf).map(f=>structuredClone(f));
  // Mirrors only the text merge and cardinality-one identity behavior, not the DB/auth handler.
  tf.data.value=b64(Y.mergeUpdates([bytes(tf.data.value),bytes(mutation.args.data.value)]));
  assert.deepEqual((await rust({update:tf.data.value,client:9300})).canonical,desired);
  assert.deepEqual([...rows.values()].filter(f=>f!==tf),untouched);
  assert.equal((await rust({update:tf.data.value,desired,client:9301})).incrementalUpdate,null);
});
await test("same-client last-write cache loses separate deltas, cumulative one works",async()=>{
  const base=full(make([text("abc")],1080));const d=from(base,1081);
  const sv=Y.encodeStateVector(d);paragraph(d).get(0).insert(3,"X");const first=Y.encodeStateAsUpdate(d,sv);
  const next=Y.encodeStateVector(d);paragraph(d).get(0).insert(4,"Y");const second=Y.encodeStateAsUpdate(d,next);
  const lost=from(b64(Y.mergeUpdates([bytes(base),second])));
  assert.equal(project(lost).plaintext,"abc");
  const cumulative=Y.mergeUpdates([first,second]);
  const ok=from(b64(Y.mergeUpdates([bytes(base),cumulative])));assert.equal(project(ok).plaintext,"abcXY");
});
console.log(`PASS: ${passed} offline interoperability groups; no remote writes.`);
