#!/usr/bin/env python3
"""Manual Yjs v1 update decoder for block/text fact values.

Implements the lib0 varint primitives and the Yjs v13 UpdateEncoderV1
write-side format directly from the yjs sources (v13.6.20):

  - writeClientsStructs  src/utils/encoding.js
  - Item.write           src/structs/Item.js
  - content writers      src/structs/Content{String,Any,Deleted,Type,...}.js
  - writeDeleteSet       src/utils/DeleteSet.js

Update layout (all varints are lib0 unsigned: 7 bits/byte, LSB first,
continuation bit 0x80; writeVarInt is sign-magnitude: first byte 0x40 sign +
6 payload bits, then 7 bits/byte):

    varuint  numClients
    per client (written in DESCENDING client order):
      varuint  numStructs
      varuint  clientID
      varuint  startClock
      numStructs x struct:
        uint8  info  (bits: 0x80 origin, 0x40 rightOrigin, 0x20 parentSub,
                      low 5 bits = content ref)
        [varuint client, varuint clock]        origin, if 0x80
        [varuint client, varuint clock]        rightOrigin, if 0x40
        if neither origin nor rightOrigin:
          varuint  parentIsYKey (1 = root-type name, 0 = parent ID)
          varstring parentYKey | [varuint client, varuint clock]
          [varstring parentSub]                if 0x20
        content (per ref below)
    deleteSet:
      varuint  numClients (0 = empty)
      per client: varuint client, varuint numRanges,
                  numRanges x (varuint clock, varuint len)

Content refs (low 5 bits of info):
  0 GC(+len)   1 Deleted(+varuint len)   2 JSON   3 Binary   4 String(varstring)
  5 Embed      6 Format(key,value)       7 Type(varuint typeRef [+varstring name])
  8 Any(varuint len + writeAny each)     9 Doc

Type refs (for content ref 7): 0 YArray 1 YMap 2 YText 3 YXmlElement(+name)
  4 YXmlFragment 5 YXmlHook 6 YXmlText

Usage: yjs_decode.py <base64-update> [--hex HEX]
"""
import base64
import json
import sys

TYPE_REFS = {0: "YArray", 1: "YMap", 2: "YText", 3: "YXmlElement",
             4: "YXmlFragment", 5: "YXmlHook", 6: "YXmlText"}


class Reader:
    def __init__(self, data: bytes):
        self.d = data
        self.p = 0

    def u8(self) -> int:
        v = self.d[self.p]
        self.p += 1
        return v

    def varuint(self) -> int:
        n, mult = 0, 1
        while True:
            b = self.d[self.p]
            self.p += 1
            n += (b & 0x7F) * mult
            if b < 0x80:
                return n
            mult *= 128

    def varint(self) -> int:
        """lib0 writeVarInt: sign-magnitude. First byte: 0x80 continuation,
        0x40 sign, low 6 bits payload; further bytes 7 bits each, LSB first."""
        b0 = self.d[self.p]
        self.p += 1
        n = b0 & 0x3F
        sign = -1 if (b0 & 0x40) else 1
        mult = 64
        while b0 & 0x80:
            b0 = self.d[self.p]
            self.p += 1
            n += (b0 & 0x7F) * mult
            mult *= 128
        return sign * n

    def varstring(self) -> str:
        ln = self.varuint()
        s = self.d[self.p:self.p + ln]
        self.p += ln
        return s.decode("utf-8", "replace")

    def varbuf(self) -> bytes:
        ln = self.varuint()
        b = self.d[self.p:self.p + ln]
        self.p += ln
        return b

    def any(self):
        """lib0 writeAny tags (lib0/src/encoding.js):
        116 Uint8Array, 117 array, 118 object, 119 string, 120 true, 121 false,
        122 bigint, 123 float64, 124 float32, 125 int (varint), 126 null,
        127 undefined."""
        tag = self.u8()
        if tag == 119:
            return self.varstring()
        if tag == 120:
            return True
        if tag == 121:
            return False
        if tag == 125:
            return self.varint()
        if tag == 124:
            import struct
            return struct.unpack("<f", self.d[self.p:self.p + 4])[0]
        if tag == 123:
            import struct
            return struct.unpack("<d", self.d[self.p:self.p + 8])[0]
        if tag == 117:
            return [self.any() for _ in range(self.varuint())]
        if tag == 118:
            return {self.varstring(): self.any()
                    for _ in range(self.varuint())}
        if tag == 116:
            return f"<{len(self.varbuf())} bytes>"
        if tag in (122,):
            import struct
            return struct.unpack("<q", self.d[self.p:self.p + 8])[0]
        if tag in (126, 127, 116):
            return None
        raise ValueError(f"unknown writeAny tag {tag} at {self.p-1}")

    def eof(self) -> bool:
        return self.p >= len(self.d)

    def rest(self) -> int:
        return len(self.d) - self.p


def rid(r: Reader) -> dict:
    return {"client": r.varuint(), "clock": r.varuint()}


def read_content(r: Reader, ref: int):
    if ref == 0:
        return {"gc": r.varuint()}
    if ref == 1:
        return {"deleted": r.varuint()}
    if ref == 2:
        return {"json": r.varstring()}
    if ref == 3:
        return {"binary": len(r.varbuf()), "bytes": r.varbuf()}
    if ref == 4:
        return {"string": r.varstring()}
    if ref == 5:
        return {"embed": json.loads(r.varstring())}
    if ref == 6:
        k = r.varstring()
        return {"format": k, "value": r.any()}
    if ref == 7:
        t = r.varuint()
        name = r.varstring() if TYPE_REFS.get(t) == "YXmlElement" else None
        return {"type": TYPE_REFS.get(t, t), "nodeName": name}
    if ref == 8:
        n = r.varuint()
        return {"any": [r.any() for _ in range(n)]}
    if ref == 9:
        return {"subdoc": r.varstring()}
    raise ValueError(f"unknown content ref {ref}")


def decode_update(data: bytes) -> dict:
    r = Reader(data)
    out = {"numClients": r.varuint(), "clients": [], "deleteSet": []}
    for _ in range(out["numClients"]):
        client = {
            "numStructs": r.varuint(),
            "clientID": r.varuint(),
            "startClock": r.varuint(),
            "structs": [],
        }
        clock = client["startClock"]
        for _ in range(client["numStructs"]):
            info = r.u8()
            ref = info & 0x1F
            item = {"clock": clock, "info": f"0x{info:02x}",
                    "ref": ref, "origin": None, "rightOrigin": None}
            if info & 0x80:
                item["origin"] = rid(r)
            if info & 0x40:
                item["rightOrigin"] = rid(r)
            if not (info & 0xC0):
                if r.varuint() == 1:
                    item["parent"] = {"yKey": r.varstring()}
                else:
                    item["parent"] = {"id": rid(r)}
            if info & 0x20:
                item["parentSub"] = r.varstring()
            item["content"] = read_content(r, ref)
            ln = 1
            c = item["content"]
            for key in ("string", "gc", "deleted"):
                if isinstance(c, dict) and isinstance(c.get(key), int) and key != "string":
                    ln = c[key]
            if isinstance(c, dict) and "string" in c:
                ln = len(c["string"])
            elif isinstance(c, dict) and "any" in c:
                ln = len(c["any"])
            item["len"] = ln
            clock += ln
            client["structs"].append(item)
        out["clients"].append(client)
    ds_n = r.varuint()
    for _ in range(ds_n):
        c = r.varuint()
        ranges = [(r.varuint(), r.varuint()) for _ in range(r.varuint())]
        out["deleteSet"].append({"client": c, "ranges": ranges})
    out["bytesConsumed"], out["bytesTotal"] = r.p, len(data)
    out["fullyParsed"] = r.eof()
    return out


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    arg = sys.argv[1]
    data = (bytes.fromhex(sys.argv[2]) if arg == "--hex"
            else base64.b64decode(arg))
    print(json.dumps(decode_update(data), indent=1))


if __name__ == "__main__":
    main()
