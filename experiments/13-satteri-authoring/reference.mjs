#!/usr/bin/env node
// Independent consumer: only the new compiler's stdout and real Yjs/ProseMirror.
// Schema subset attributed to Leaflet schema.ts @600c7b5f and experiment 05.
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import * as Y from "yjs";
import { Schema } from "prosemirror-model";
import { yXmlFragmentToProsemirrorJSON } from "y-prosemirror";

const run = promisify(execFile);
const cwd = fileURLToPath(new URL(".", import.meta.url));
const schema = new Schema({
  nodes: {
    doc: { content: "block" }, paragraph: { content: "inline*", group: "block" },
    text: { group: "inline" }, hard_break: { group: "inline", inline: true, selectable: false },
  },
  marks: { strong: {}, em: {}, code: {}, strikethrough: {}, link: { attrs: { href: {} }, inclusive: false } },
});
const original = await readFile(new URL("fixtures/garden.mdx", import.meta.url));
const { stdout } = await run("./target/debug/satteri-authoring-experiment", ["check", "fixtures/garden.mdx"], { cwd, maxBuffer: 4 * 1024 * 1024 });
const checked = JSON.parse(stdout);

function mark(m) {
  if (m.kind === "link") return { type: "link", attrs: { href: m.href } };
  return { type: m.kind };
}
function expected(content) {
  const nodes = content.map(n => n.kind === "hardBreak" ? { type: "hard_break" }
    : { type: "text", text: n.text, ...(n.marks.length ? { marks: n.marks.map(mark) } : {}) });
  return { type: "doc", content: [{ type: "paragraph", ...(nodes.length ? { content: nodes } : {}) }] };
}
// XML node partition/mark ordering is not semantic identity. Normalize independently.
function normalized(pm) {
  const result = structuredClone(pm);
  const content = [];
  for (const node of result.content[0].content ?? []) {
    if (node.marks?.length) {
      for (const mark of node.marks) {
        if (mark.attrs && Object.keys(mark.attrs).length === 0) delete mark.attrs;
      }
      node.marks.sort((a, b) => a.type.localeCompare(b.type));
    }
    else delete node.marks;
    const previous = content.at(-1);
    if (node.type === "text" && previous?.type === "text" && JSON.stringify(previous.marks) === JSON.stringify(node.marks)) previous.text += node.text;
    else content.push(node);
  }
  if (content.length) result.content[0].content = content;
  else delete result.content[0].content;
  return result;
}

let textCount = 0;
for (const fact of checked.plan.facts.filter(f => f.attribute === "block/text")) {
  assert.equal(fact.data.type, "text");
  const peer = new Y.Doc();
  const bytes = Uint8Array.from(Buffer.from(fact.data.value, "base64"));
  Y.applyUpdate(peer, bytes);
  assert.equal(peer.getMap("meta").get("schemaVersion"), 1);
  const actual = yXmlFragmentToProsemirrorJSON(peer.getXmlFragment("prosemirror"));
  schema.nodeFromJSON(actual).check();
  assert.deepEqual(normalized(actual), normalized(expected(checked.plan.expected_text[fact.entity])));
  Y.applyUpdate(peer, bytes);
  assert.deepEqual(normalized(yXmlFragmentToProsemirrorJSON(peer.getXmlFragment("prosemirror"))), normalized(actual), "exact full-update replay must not duplicate a paragraph");
  textCount++;
}
assert.equal(textCount, 9);
console.log(`ok - ${textCount} fresh Satteri→IR text payloads consumed by Yjs 13.6.15 / y-prosemirror 1.2.5 and checked by ProseMirror 1.25.0`);

const argument = checked.plan.author_keys.argument.entity;
const prose = checked.plan.expected_text[argument];
assert.equal(prose.map(n => n.kind === "hardBreak" ? "\n" : n.text).join(""), "The beds are ready, and Café maps help us plan. An escaped *star* & Unicode é😀 stay text.\nBring seeds, not a runtime.");
assert.equal(prose.filter(n => n.kind === "hardBreak").length, 1);
const detailText = Object.values(checked.plan.expected_text).find(c => c.some(n => n.text?.includes("raw\ntext")));
assert.ok(detailText);
assert.equal(detailText.filter(n => n.kind === "hardBreak").length, 1);
console.log("ok - independently asserted decoded prose, Unicode/escapes, soft wrap, exact Run newline and actual hard-break topology");

const poll = Object.keys(checked.plan.polls)[0];
const options = checked.plan.facts.filter(f => f.entity === poll && f.attribute === "poll/options").sort((a, b) => a.data.position.localeCompare(b.data.position));
assert.deepEqual(options.map(o => checked.plan.facts.find(f => f.entity === o.data.value && f.attribute === "poll-option/name").data.value), ["Gardens & maps", "Café 🦀", "  exact spacing  "]);
assert.ok(checked.plan.facts.some(f => f.attribute === "block/card" && f.data.value === checked.plan.pages.details));
assert.ok(!JSON.stringify(checked.plan).includes("pollRef"), "no invented published reference");
console.log("ok - unpublished Poll option facts preserve exact ordered values; PageLink resolves the generated named page");

const { stdout: demo } = await run("./target/debug/satteri-authoring-experiment", ["demo", "fixtures/garden.mdx"], { cwd, maxBuffer: 4 * 1024 * 1024 });
const simulated = JSON.parse(demo);
for (const flag of ["offlineSimulation", "interruptedStageKeptOldEntry", "repeatDidNotAppend", "editedSourceProducedNewBody", "stablePollReused"]) assert.equal(simulated[flag], true);
assert.deepEqual(await readFile(new URL("fixtures/garden.mdx", import.meta.url)), original);
console.log("ok - actual CLI recreate scenario and byte-untouched canonical source (simulation, not a server test)");
