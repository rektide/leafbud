#!/usr/bin/env node
// Execute pinned native serializer code, not a port of its output algorithm.
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { homedir } from "node:os";
import { join } from "node:path";
import ts from "typescript";
import * as Y from "yjs";
import * as base64 from "base64-js";
import { UnicodeString, BlobRef } from "@atproto/api";

const run = promisify(execFile);
const upstream = process.env.LEAFLET_SOURCE ?? join(homedir(), "archive/hyperlink-academy/leaflet");
const revision = "600c7b5f1f7b31b66fb626c293cc104312a12caf";
assert.equal((await run("git", ["rev-parse", "HEAD"], { cwd: upstream })).stdout.trim(), revision);
assert.equal((await run("git", ["status", "--porcelain"], { cwd: upstream })).stdout, "");
const hashes = {};
async function source(path) {
  const text = await readFile(join(upstream, path), "utf8");
  hashes[path] = createHash("sha256").update(text).digest("hex");
  return text;
}
function evaluate(text, imports = {}) {
  const output = ts.transpileModule(text, { compilerOptions: { target: ts.ScriptTarget.ES2023, module: ts.ModuleKind.CommonJS } }).outputText;
  const exports = {};
  new Function("require", "exports", output)(name => {
    assert.ok(Object.hasOwn(imports, name), `unexpected native import ${name}`);
    return imports[name];
  }, exports);
  return exports;
}
function declaration(text, name) {
  const ast = ts.createSourceFile("native.ts", text, ts.ScriptTarget.Latest, true);
  const found = ast.statements.find(node => ts.isVariableStatement(node) && node.declarationList.declarations.some(d => d.name.getText(ast) === name));
  assert.ok(found, name);
  return found.getText(ast);
}
const unavailable = () => { throw new Error("unsupported native branch called by image-free fixture"); };
const scan = evaluate(declaration(await source("src/replicache/utils.ts"), "scanIndexLocal"));
const lists = evaluate(await source("src/utils/parseBlocksToList.ts"));
const blocks = evaluate(await source("src/replicache/getBlocks.ts"), {
  "src/replicache/utils": scan,
  "src/replicache/blockMirror": { getBlockStructureMirror: unavailable },
  "src/utils/canvasBlockOrder": { canvasBlockOrder: unavailable },
});
const strings = evaluate(await source("src/utils/yjsFragmentToString.ts"), { yjs: Y });
const ids = evaluate(declaration(await source("lexicons/api/lexicons.ts"), "ids"));
const pageLinkDisplay = evaluate(await source("src/utils/pageLinkDisplay.ts"));
const native = evaluate(await source("src/utils/factsToPagesRecord.ts"), {
  yjs: Y, "base64-js": base64, "@atproto/api": { UnicodeString },
  "src/utils/pageLinkDisplay": pageLinkDisplay,
  "src/questionsAudience": { isQuestionsAudience: unavailable },
  "lexicons/api/lexicons": ids,
  "components/Blocks/DrawingBlock/ink": { drawingFills: unavailable },
  "src/replicache/utils": scan,
  "src/replicache/getBlocks": blocks,
  "src/utils/isTextBlock": { isTextBlock: {} },
  "src/utils/parseBlocksToList": lists,
  "src/utils/yjsFragmentToString": strings,
  "components/ThemeManager/colorToLexicons": { ColorToRGB: unavailable, ColorToRGBA: unavailable },
  "components/ThemeManager/themeUtils": { ThemeDefaults: {} },
  "@react-stately/color": { parseColor: unavailable },
});
const offload = evaluate(await source("src/utils/offloadPagesToBlob.ts"), { "@atproto/lexicon": { BlobRef } });
const hooks = { uploadImage: unavailable, uploadPoll: unavailable };
let counter = 0;
const uuid = () => `11111111-1111-4111-8111-${String(++counter).padStart(12, "0")}`;
function graph() {
  const root = uuid(), page = uuid(), facts = [];
  const fact = (entity, attribute, type, value, extra = {}) => facts.push({ id: uuid(), entity, attribute, data: { type, value, ...extra } });
  fact(root, "root/page", "ordered-reference", page, { position: "a" });
  fact(page, "page/type", "page-type-union", "doc");
  const add = (kind, attrs = [], target = page) => {
    const entity = uuid();
    fact(target, "card/block", "ordered-reference", entity, { position: `a${String(counter).padStart(12, "0")}` });
    fact(entity, "block/type", "block-type-union", kind);
    for (const [attribute, type, value] of attrs) fact(entity, attribute, type, value);
    return entity;
  };
  return { root, page, facts, fact, add };
}
function text(parts) {
  const doc = new Y.Doc();
  doc.clientID = ++counter;
  const p = new Y.XmlElement("paragraph");
  doc.getXmlFragment("prosemirror").insert(0, [p]);
  for (const part of parts) {
    if (part === null) { p.insert(p.length, [new Y.XmlElement("hard_break")]); continue; }
    const t = new Y.XmlText();
    p.insert(p.length, [t]);
    for (const [insert, attributes] of part) t.applyDelta([{ insert, ...(attributes && { attributes }) }]);
  }
  return Buffer.from(Y.encodeStateAsUpdate(doc)).toString("base64");
}
const cases = [];
{
  const g = graph();
  // Equal visible/mark meaning but 1600 separate XmlText facets, not one run.
  g.add("text", [["block/text", "text", text(Array.from({ length: 1600 }, () => [["x", { strong: {} }]]))]]);
  cases.push(["fragmented-marked", g]);
}
{
  const g = graph();
  const all = { strong: {}, em: {}, code: {}, strikethrough: {}, underline: {}, highlight: { color: "1" }, link: { href: "https://example.test/é?x=\"\\\u0000\n🦋" } };
  const value = text([[["A 🦋", { strong: {} }]], [[" café", { strong: {} }]], null, [["\"\\/\u0000\b\f\n\r\t雪\u2028\u2029", all], [" tail", {}]]]);
  g.add("heading", [["block/text", "text", value], ["block/heading-level", "number", 2]]);
  g.add("blockquote", [["block/text", "text", value]]);
  g.add("text", [["block/text", "text", value]]);
  for (const style of ["unordered", "unordered", "ordered", "ordered"]) {
    g.add("text", [["block/text", "text", value], ["block/is-list", "boolean", true], ["block/list-style", "list-style-union", style]]);
  }
  g.add("code", [["block/code", "string", "\"\\雪\u0000\n🦋"], ["block/code-language", "string", "rust"]]);
  g.add("horizontal-rule");
  const child = uuid();
  g.fact(child, "page/type", "page-type-union", "doc");
  g.add("card", [["block/card", "reference", child]]);
  g.add("text", [["block/text", "text", text([[["child page"]]])]], child);
  cases.push(["unicode-marks-links-lists-pages-breaks", g]);
}
{
  const g = graph();
  g.add("code", [["block/code", "string", "Native ignores this block's latent block/text"], ["block/text", "text", "not valid Yjs; inactive"]]);
  cases.push(["inactive-text-and-absent-language", g]);
}
for (const delta of [-1, 0, 1]) {
  const g = graph();
  g.add("code", [["block/code", "string", ""], ["block/code-language", "string", "rust"]]);
  const empty = (await native.processBlocksToPages({ facts: g.facts, root_entity: g.root, hooks })).pages;
  g.facts.find(f => f.attribute === "block/code").data.value = "x".repeat(102400 + delta - Buffer.byteLength(JSON.stringify(empty), "utf8"));
  cases.push([`boundary-${delta}`, g]);
}
const fixtures = [];
for (const [name, g] of cases) {
  const { pages } = await native.processBlocksToPages({ facts: g.facts, root_entity: g.root, hooks });
  const json_utf8_bytes = Buffer.byteLength(JSON.stringify(pages), "utf8");
  let uploads = 0;
  const agent = { com: { atproto: { repo: { uploadBlob: async blob => {
    uploads++;
    assert.equal(blob.size, json_utf8_bytes);
    return { data: { blob: { synthetic: true } } };
  } } } } };
  await offload.maybeOffloadPagesToBlob({ content: { $type: "pub.leaflet.content", pages } }, agent);
  assert.equal(uploads, json_utf8_bytes > 102400 ? 1 : 0);
  fixtures.push({ name, root: g.root, facts: g.facts, pages, json_utf8_bytes, would_offload: uploads === 1 });
}
assert.ok(fixtures[0].json_utf8_bytes > 102400);
assert.equal(fixtures[0].pages[0].blocks[0].block.facets.length, 1600);
for (let i = 0; i < 3; i++) assert.equal(fixtures[i + 3].json_utf8_bytes, 102399 + i);
// Compare the JSON representation: JSON.stringify deliberately omits native
// undefined properties such as code.syntaxHighlightingTheme.
const evidence = JSON.parse(JSON.stringify({ revision, source_sha256: hashes, fixtures }));
const path = new URL("../../crates/workflow/tests/fixtures/native-publication-size.json", import.meta.url);
if (process.argv.includes("--write")) await writeFile(path, JSON.stringify(evidence) + "\n");
else assert.equal(createHash("sha256").update(JSON.stringify(JSON.parse(await readFile(path, "utf8")))).digest("hex"), createHash("sha256").update(JSON.stringify(evidence)).digest("hex"), "native fixture JSON digest differs");
console.log(`ok - ${fixtures.length} actual pinned native serializer/offload fixtures; fragmentation and exact threshold boundaries`);
