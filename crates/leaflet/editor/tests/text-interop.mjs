#!/usr/bin/env node
// Production Yrs writer -> actual pinned Yjs / y-prosemirror consumer. Dependencies
// deliberately reuse experiment 05's exact lock, but not its experimental editor.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
const require = createRequire(new URL("../../../../experiments/05-yrs-conversion/package.json", import.meta.url));
const Y = require("yjs");
const { Schema } = require("prosemirror-model");
const { prosemirrorJSONToYXmlFragment, yXmlFragmentToProsemirrorJSON } = require("y-prosemirror");
const binary = fileURLToPath(new URL("../../../../target/debug/examples/text-write", import.meta.url));
const schema = new Schema({
  nodes:{doc:{content:"block"},paragraph:{content:"inline*",group:"block"},text:{group:"inline"},hard_break:{group:"inline",inline:true}},
  marks:{strong:{},em:{},code:{},strikethrough:{},underline:{},highlight:{attrs:{color:{}}},link:{attrs:{href:{}}}},
});
const b64 = bytes => Buffer.from(bytes).toString("base64");
const bytes = payload => Uint8Array.from(Buffer.from(payload,"base64"));
const full = doc => b64(Y.encodeStateAsUpdate(doc));
const p = doc => doc.getXmlFragment("prosemirror").get(0);
const run = (text, marks=[]) => ({kind:"run",text,marks});
const br = {kind:"hardBreak"};
function make(text) {
  const doc = new Y.Doc();
  prosemirrorJSONToYXmlFragment(schema,{type:"doc",content:[{type:"paragraph",content:text ? [{type:"text",text}] : []}]},doc.getXmlFragment("prosemirror"));
  doc.getMap("meta").set("schemaVersion",1);
  doc.getMap("unrelated").set("keep","unchanged");
  return doc;
}
function write(current,desired,failure=false) {
  const result = spawnSync(binary,[],{input:JSON.stringify({current,desired}),encoding:"utf8"});
  if(failure) {assert.notEqual(result.status,0);return;}
  assert.equal(result.status,0,result.stderr);
  return JSON.parse(result.stdout).update;
}
function project(doc) {
  schema.nodeFromJSON(yXmlFragmentToProsemirrorJSON(doc.getXmlFragment("prosemirror"))).check();
  const out=[];
  for(const child of p(doc).toArray()) {
    if(child instanceof Y.XmlElement) {assert.equal(child.nodeName,"hard_break");out.push(br);continue;}
    for(const delta of child.toDelta()) {
      assert.equal(typeof delta.insert,"string");
      const marks=Object.entries(delta.attributes??{}).map(([key,value])=> {
        const names={strong:"strong",em:"em",code:"code",strikethrough:"strikethrough",underline:"underline"};
        return key==="link"?{kind:"link",href:value.href}:key==="highlight"?{kind:"highlight",color:value.color}:{kind:names[key]};
      }).sort((a,b)=>JSON.stringify(a).localeCompare(JSON.stringify(b)));
      const last=out.at(-1);
      if(last?.kind==="run"&&JSON.stringify(last.marks)===JSON.stringify(marks))last.text+=delta.insert;
      else out.push(run(delta.insert,marks));
    }
  }
  return out;
}
const normalized=desired=>desired.map(i=>i.kind==="run"?run(i.text,[...i.marks].sort((a,b)=>JSON.stringify(a).localeCompare(JSON.stringify(b)))):i);
function apply(doc,desired) {
  const payload=write(full(doc),desired);assert.ok(payload);
  Y.applyUpdate(doc,bytes(payload));Y.applyUpdate(doc,bytes(payload));
  assert.deepEqual(project(doc),normalized(desired));
  assert.equal(doc.getMap("unrelated").get("keep"),"unchanged");
  assert.equal(write(full(doc),desired),null);
  return payload;
}
let groups=0;
const test=(name,fn)=>{fn();console.log(`ok ${++groups} - ${name}`);};
test("scalar-safe Unicode splice retains loaded paragraph/text and suffix anchor",()=>{
  const doc=make("Red é😀e\u0301 boats drift.");const para=p(doc),text=para.get(0);
  const anchor=Y.createRelativePositionFromTypeIndex(text,10);
  apply(doc,[run("Blue é😀e\u0301 boats drift.")]);
  assert.equal(p(doc),para);assert.equal(p(doc).get(0),text);
  assert.equal(Y.createAbsolutePositionFromRelativePosition(anchor,doc).index,11);
});
test("all marks, mark-only removal and changed link/highlight retain characters",()=>{
  const doc=make("marked");const text=p(doc).get(0);const anchor=Y.createRelativePositionFromTypeIndex(text,3);
  const marks=[{kind:"strong"},{kind:"em"},{kind:"code"},{kind:"strikethrough"},{kind:"underline"},{kind:"highlight",color:"1"},{kind:"link",href:"https://example.test/old"}];
  apply(doc,[run("marked",marks)]);
  apply(doc,[run("marked",[{kind:"link",href:"https://example.test/new"},{kind:"highlight",color:"2"}])]);
  apply(doc,[run("marked")]);assert.equal(p(doc).get(0),text);
  assert.equal(Y.createAbsolutePositionFromRelativePosition(anchor,doc).index,3);
});
test("deletion-only complete payload carries delete set without a changed state vector",()=>{
  const doc=make("abc");const vector=Y.encodeStateVector(doc);apply(doc,[run("ac")]);assert.deepEqual(Y.encodeStateVector(doc),vector);
});
test("break/topology coarse branch retains paragraph/unrelated roots; empty and repeated edits",()=>{
  const doc=make("é😀x");const para=p(doc);
  for(const desired of [[run("é😀"),br,run("x")],[br,run("é😀"),br,br,run("x"),br],[run("é😀x")],[],[run("e\u0301🦀")]]) {
    apply(doc,desired);assert.equal(p(doc),para);
  }
});
test("semantic no-op ignores XML segmentation and tombstones",()=>{
  const doc=make("");const a=new Y.XmlText(),b=new Y.XmlText();p(doc).insert(0,[a,b]);a.insert(0,"one ");b.insert(0,"two!");b.delete(3,1);
  assert.equal(write(full(doc),[run("one two")]),null);
});
test("fresh only for genuinely new text; cumulative last payload does not depend on missing deltas",()=>{
  const initial=write(null,[run("abc")]);const base=new Y.Doc();Y.applyUpdate(base,bytes(initial));
  const second=write(full(base),[run("ac")]);const edited=new Y.Doc();Y.applyUpdate(edited,bytes(second));
  const last=write(full(edited),[run("AXc")]);const server=new Y.Doc();Y.applyUpdate(server,bytes(Y.mergeUpdates([bytes(initial),bytes(last)])));
  assert.deepEqual(project(server),[run("AXc")]);
});
test("unknown active semantics and incomplete causal updates cannot trigger a rewrite",()=>{
  const doc=make("abc");const vector=Y.encodeStateVector(doc);p(doc).get(0).insert(1,"X");
  write(b64(Y.encodeStateAsUpdate(doc,vector)),[run("other")],true);
  p(doc).get(0).format(0,1,{futureMark:{}});write(full(doc),[run("other")],true);
});
console.log(`${groups} production Yrs/Yjs groups passed`);
