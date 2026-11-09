# Yjs `block/text` samples — annotated

All samples are **synthetic** (fabricated client ids and strings), generated
with yjs 13.6.18 (`Y.encodeStateAsUpdate`) and re-verified with this repo's
`scripts/yjs_decode.py`. Real draft values observed live (2026-11-07) have the
same shapes; one real short block decoded to the identical 5-struct layout as
sample 2 below, with real prose in the string item.

`block/text` fact data is `{ "type": "text", "value": "<base64 of a Yjs v1
update>" }`. The update encodes a ProseMirror doc: one root type
`prosemirror` holding `paragraph` XmlElements holding `XmlText` holding
string content. Authors: `src/utils/createYjsText.ts` (server-side seed),
y-prosemirror in the editor (adds `meta.schemaVersion`, format items,
tombstones).

## Sample 1 — `createYjsText("Hello world")` (40 bytes, hex)

Produced by `scripts/yjs_encode_text.py "Hello world" --client 979896792 --hex`
and byte-identical to yjs 13.6.18 with `doc.clientID = 979896792`:

```
01                            numClients = 1
03                            numStructs  = 3
d8 93 a0 d3 03                clientID    = 979896792 (varuint, LSB-first)
00                            startClock  = 0
-- struct 0 (clock 0) --------------------------------------------------------
07                            info: contentRef 7 ContentType; no origin(0x80),
                              no rightOrigin(0x40), no parentSub(0x20)
01                            parentInfo: 1 = parent is a root-type name
0b 70726f73656d6972726f72      varstring "prosemirror" (11 bytes)
03                            typeRef 3 = YXmlElement
09 706172616772617068         varstring "paragraph" (nodeName)
-- struct 1 (clock 1) --------------------------------------------------------
07                            info: ContentType
00                            parentInfo: 0 = parent is an item ID
d8 93 a0 d3 03 00             parent = ID(client 979896792, clock 0) — the paragraph
06                            typeRef 6 = YXmlText
-- struct 2 (clock 2, len 11) ------------------------------------------------
04                            info: contentRef 4 ContentString
00                            parentInfo: 0 = parent ID
d8 93 a0 d3 03 01             parent = ID(client, clock 1) — the XmlText
0b 48656c6c6f20776f726c64     varstring "Hello world"
-- deleteSet -----------------------------------------------------------------
00                            0 clients with deletes (empty)
```

Empty text collapses to `numStructs=2` (yjs skips the empty insert) — still a
valid doc that renders an empty paragraph.

## Sample 2 — edited block with `meta.schemaVersion` and a tombstone (57 bytes)

The shape an *edited* block converges to (one real live draft block decoded
to exactly this layout; strings/ids here are synthetic). Built by yjs:
insert paragraph/XmlText/"Synthetic sample textXXXXXXXX", set
`meta.schemaVersion = 1`, delete 8 chars; `encodeStateAsUpdate` then emits 5
structs + a delete-set range.

hex: `0105cdf8acd1010007010b70726f73656d6972726f7203097061726167726170680700cdf8acd10100060400cdf8acd101011553796e7468657469632073616d706c65207465787481cdf8acd10116082801046d6574610d736368656d6156657273696f6e017d0101cdf8acd101011708`

Decoded (`scripts/yjs_decode.py <b64>`):

| clock | info   | content                                   | notes |
|------:|--------|-------------------------------------------|-------|
| 0     | `0x07` | ContentType YXmlElement `paragraph`        | parent = yKey `prosemirror` |
| 1     | `0x07` | ContentType YXmlText                      | parent = ID(c,0) |
| 2     | `0x04` | ContentString `Synthetic sample text`      | parent = ID(c,1), len 21 |
| 23    | `0x81` | ContentDeleted len 8                      | origin = ID(c,22) — tombstone of the deleted `"XXXXXXXX"` suffix (the string item was split at integration) |
| 31    | `0x28` | ContentAny `[1]`                          | parent = yKey `meta`, parentSub `schemaVersion` — `0x20` bit set; value: `01`(len) `7d`(writeAny int tag) `01`(varint 1) |

deleteSet: client `cdf8acd1 01` (=439041101), 1 range `clock 23 len 8`.

Byte-level bits worth knowing:

- lib0 varuint: 7 bits/byte, LSB first, `0x80` continuation.
- lib0 varint (`writeAny` ints): sign-magnitude — first byte `0x40` sign +
  **6** payload bits, then 7 bits/byte.
- `writeAny` tags (lib0): 116 bytes, 117 array, 118 object, 119 string,
  120/121 bools, 123 float64, 124 float32, 125 int, 126 null, 127 undefined.
- `parentInfo` is written only when **neither** origin nor rightOrigin is set
  (parent can be copied from a neighbour); yKey=1 / parent-ID=0.
- contentRefs: 1 Deleted, 2 JSON, 3 Binary, 4 String, 5 Embed, 6 Format,
  7 Type, 8 Any, 9 Doc. typeRefs: 3 YXmlElement(+nodeName), 6 YXmlText,
  4 YXmlFragment, 0/1/2/5 = YArray/YMap/YText/YXmlHook.
- Structs of one client are contiguous by clock; `clock += content.length`.

## Anomaly: 3-byte non-updates

Some empty text blocks on live drafts store the 3-byte value `b5 ec 6d`,
which is **not** a valid update — yjs `applyUpdate` throws
`Unexpected end of array` (first varuint decodes to 1,799,733 "clients").
Treat as empty/absent when reading; leafbud's reader must never assume
every `block/text` value parses.
