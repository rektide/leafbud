#!/usr/bin/env python3
"""Minimal `createYjsText` encoder — the reference for a Rust port.

Reproduces byte-for-byte what Leaflet's src/utils/createYjsText.ts stores in
a "block/text" fact (verified identical to yjs 13.6.18 `encodeStateAsUpdate`
with the same clientID; see REPORT.md "Yjs feasibility").

A fresh single-paragraph doc is exactly:

    varuint 1                    numClients
    varuint 3                    numStructs
    varuint clientID             client
    varuint 0                    startClock
    # struct 0: XmlElement("paragraph") in root type "prosemirror"
    u8 0x07                      ContentType, no origin/rightOrigin/parentSub
    varuint 1                    parentInfo: parent is a root-type name
    varstring "prosemirror"
    varuint 3                    typeRef YXmlElement
    varstring "paragraph"
    # struct 1: XmlText child of the paragraph (clock 0)
    u8 0x07
    varuint 0                    parentInfo: parent is an ID
    varuint clientID, varuint 0  parent = (client, 0)
    varuint 6                    typeRef YXmlText
    # struct 2: the text itself (clock 1), child of the XmlText
    u8 0x04                      ContentString
    varuint 0
    varuint clientID, varuint 1  parent = (client, 1)
    varstring text
    varuint 0                    deleteSet: empty

Empty text collapses to numStructs=2 (yjs skips the empty insert).

Usage: yjs_encode_text.py "Some text" [--client UINT32] [--hex]
"""
import base64
import secrets
import sys


def varuint(n: int) -> bytes:
    out = bytearray()
    while n > 0x7F:
        out.append(0x80 | (n & 0x7F))
        n >>= 7
    out.append(n)
    return bytes(out)


def varstring(s: str) -> bytes:
    b = s.encode()
    return varuint(len(b)) + b


def create_yjs_text(text: str, client_id: int | None = None) -> bytes:
    if client_id is None:
        client_id = secrets.randbelow(2 ** 31)
    assert 0 <= client_id < 2 ** 32
    out = bytearray()
    out += varuint(1)                       # numClients
    out += varuint(3 if text else 2)        # numStructs
    out += varuint(client_id)               # client
    out += varuint(0)                       # startClock
    # struct 0: paragraph XmlElement in root "prosemirror"
    out.append(0x07)                        # info: ContentType (ref 7)
    out += varuint(1)                       # parentInfo: y-key
    out += varstring("prosemirror")
    out += varuint(3)                       # typeRef: YXmlElement
    out += varstring("paragraph")
    # struct 1: XmlText in paragraph (clock 0)
    out.append(0x07)
    out += varuint(0)                       # parentInfo: parent id
    out += varuint(client_id) + varuint(0)
    out += varuint(6)                       # typeRef: YXmlText
    if text:
        # struct 2: string in the XmlText (clock 1)
        out.append(0x04)                    # info: ContentString (ref 4)
        out += varuint(0)
        out += varuint(client_id) + varuint(1)
        out += varstring(text)
    out += varuint(0)                       # deleteSet: empty
    return bytes(out)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    client = None
    if "--client" in sys.argv:
        client = int(sys.argv[sys.argv.index("--client") + 1])
    b = create_yjs_text(args[0], client)
    print(b.hex() if "--hex" in sys.argv else base64.b64encode(b).decode())


if __name__ == "__main__":
    main()
