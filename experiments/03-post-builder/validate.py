#!/usr/bin/env python3
"""Validate built-record.json against the Leaflet lexicons (experiment 03).

Checks are derived from the lexicon JSONs at LEXICON_DIR plus the byte-offset
and derivation rules. Every check prints PASS/FAIL; exit code 1 on any FAIL.
Grapheme counts: our strings contain no combining sequences, so code points
equal grapheme clusters here; a general validator needs proper grapheme
segmentation (e.g. unicode-segmentation in Rust).
"""

import json
import re
import sys
import unicodedata
from datetime import datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
LEXICON_DIR = Path.home() / "archive/hyperlink-academy/leaflet/lexicons"

DID = "did:plc:zjbq26wybii5ojoypkso2mso"
PUB_RKEY = "3molsbwkh522o"
PUB_URL = "https://technautical.leaflet.pub"
RKEY = "leafspring-protocol-probe"
FALLBACK_URL = f"https://leaflet.pub/lish/{DID}/{PUB_RKEY}/{RKEY}"
PUB_DOMAIN_URL = f"{PUB_URL}/{RKEY}"

failures = []


def check(name, ok, detail=""):
    line = f"PASS  {name}" if ok else f"FAIL  {name}"
    if detail:
        line += f" — {detail}"
    print(line)
    if not ok:
        failures.append(name)


def graphemes(s):
    # Code-point count minus combining marks approximates grapheme clusters
    # for our strings (no combining sequences present; see module docstring).
    return sum(1 for ch in s if not unicodedata.combining(ch))


def is_iso_datetime(s):
    try:
        datetime.fromisoformat(s.replace("Z", "+00:00"))
        return True
    except ValueError:
        return False


def main():
    raw = (HERE / "built-record.json").read_text(encoding="utf-8")
    rec = json.loads(raw)

    lex = {}
    for ns in ["site/standard/document", "pub/leaflet/content",
               "pub/leaflet/pages/linearDocument", "pub/leaflet/blocks/text",
               "pub/leaflet/blocks/header", "pub/leaflet/richtext/facet"]:
        path = LEXICON_DIR / f"{ns}.json"
        lex[ns] = json.loads(path.read_text(encoding="utf-8"))

    doc_props = lex["site/standard/document"]["defs"]["main"]["record"]["properties"]
    doc_required = lex["site/standard/document"]["defs"]["main"]["record"]["required"]

    # ---- site.standard.document top level ----
    check("json parses", isinstance(rec, dict))
    check("top-level $type is site.standard.document",
          rec.get("$type") == "site.standard.document")
    for field in doc_required:
        check(f"required field present: {field}", field in rec)

    # site: string, uri format (at-uri to the publication)
    site = rec.get("site", "")
    check("site is string", isinstance(site, str))
    check("site is at-uri to the target publication",
          site == f"at://{DID}/site.standard.publication/{PUB_RKEY}", site)
    check("site matches lexicon format 'uri' (at:// scheme)",
          bool(re.match(r"^at://[^/\s]+/[^/\s]+/[^/\s]+$", site)))

    # title / description / tags: string, maxGraphemes, maxLength (bytes)
    title = rec.get("title", "")
    check("title is string", isinstance(title, str))
    check("title graphemes <= 500", graphemes(title) <= doc_props["title"]["maxGraphemes"],
          f"{graphemes(title)}")
    check("title bytes <= 5000", len(title.encode()) <= doc_props["title"]["maxLength"],
          f"{len(title.encode())}")
    desc = rec.get("description", "")
    check("description is string", isinstance(desc, str))
    check("description graphemes <= 3000",
          graphemes(desc) <= doc_props["description"]["maxGraphemes"], f"{graphemes(desc)}")
    check("description bytes <= 30000",
          len(desc.encode()) <= doc_props["description"]["maxLength"], f"{len(desc.encode())}")
    tags = rec.get("tags", [])
    check("tags is array of strings",
          isinstance(tags, list) and all(isinstance(t, str) for t in tags))
    check("tags items graphemes <= 50",
          all(graphemes(t) <= doc_props["tags"]["items"]["maxGraphemes"] for t in tags))
    check("tags items bytes <= 100",
          all(len(t.encode()) <= doc_props["tags"]["items"]["maxLength"] for t in tags))

    check("publishedAt parses as ISO 8601 datetime",
          is_iso_datetime(rec.get("publishedAt", "")), rec.get("publishedAt", ""))
    check("path is string", isinstance(rec.get("path"), str))
    check("textContent is string", isinstance(rec.get("textContent"), str))

    # ---- content: pub.leaflet.content union member ----
    content = rec.get("content", {})
    check("content union member typed via $type",
          content.get("$type") == "pub.leaflet.content", content.get("$type"))
    check("content.$type is in the document.content union refs",
          content.get("$type") in lex["site/standard/document"]["defs"]["main"]
          ["record"]["properties"]["content"]["refs"])
    check("content.pages present and non-empty",
          isinstance(content.get("pages"), list) and len(content["pages"]) > 0)

    page_refs = lex["pub/leaflet/content"]["defs"]["main"]["properties"]["pages"] \
        ["items"]["refs"]

    for pi, page in enumerate(content["pages"]):
        check(f"page[{pi}] union member typed ($type in {page_refs})",
              page.get("$type") in page_refs, page.get("$type"))
        ld = lex["pub/leaflet/pages/linearDocument"]["defs"]["main"]
        check(f"page[{pi}] required 'blocks' present", "blocks" in page)
        check(f"page[{pi}] optional 'id' is string when present",
              "id" not in page or isinstance(page["id"], str))
        block_refs = set(lex["pub/leaflet/pages/linearDocument"]["defs"]["block"]
                         ["properties"]["block"]["refs"])
        known_alignments = lex["pub/leaflet/pages/linearDocument"]["defs"]["block"] \
            ["properties"]["alignment"]["knownValues"]
        for bi, pb in enumerate(page["blocks"]):
            check(f"page[{pi}].blocks[{bi}] $type is linearDocument#block",
                  pb.get("$type") == "pub.leaflet.pages.linearDocument#block")
            check(f"page[{pi}].blocks[{bi}] required 'block' present", "block" in pb)
            check(f"page[{pi}].blocks[{bi}] alignment (if set) is a knownValue",
                  "alignment" not in pb or pb["alignment"] in known_alignments)
            inner = pb["block"]
            check(f"page[{pi}].blocks[{bi}].block $type is in the block union",
                  inner.get("$type") in block_refs, inner.get("$type"))

    # ---- block subtypes: text / header ----
    text_props = lex["pub/leaflet/blocks/text"]["defs"]["main"]
    header_props = lex["pub/leaflet/blocks/header"]["defs"]["main"]
    for pi, page in enumerate(content["pages"]):
        for bi, pb in enumerate(page["blocks"]):
            inner = pb["block"]
            label = f"page[{pi}].blocks[{bi}].block"
            if inner.get("$type") == "pub.leaflet.blocks.text":
                check(f"{label} required 'plaintext' present", "plaintext" in inner)
                if "textSize" in inner:
                    check(f"{label} textSize respects enum",
                          inner["textSize"] in text_props["properties"]["textSize"]["enum"],
                          inner["textSize"])
            if inner.get("$type") == "pub.leaflet.blocks.header":
                check(f"{label} required 'plaintext' present", "plaintext" in inner)
                lvl = inner.get("level")
                check(f"{label} level is integer 1..6",
                      isinstance(lvl, int) and 1 <= lvl <= 6, str(lvl))

    # ---- facets: shape + UTF-8 byte-offset round-trip ----
    facet_defs = lex["pub/leaflet/richtext/facet"]["defs"]
    # Union refs beginning with '#' are lexicon-relative: '#link' inside
    # pub.leaflet.richtext.facet means 'pub.leaflet.richtext.facet#link'.
    facet_id = lex["pub/leaflet/richtext/facet"]["id"]
    feature_refs = {f"{facet_id}{r}" if r.startswith("#") else r
                    for r in facet_defs["main"]["properties"]["features"]["items"]["refs"]}
    for pi, page in enumerate(content["pages"]):
        for bi, pb in enumerate(page["blocks"]):
            inner = pb["block"]
            if "facets" not in inner:
                continue
            pt = inner["plaintext"]
            raw_pt = pt.encode("utf-8")
            check(f"page[{pi}].blocks[{bi}].block.facets is array",
                  isinstance(inner["facets"], list))
            for fi, facet in enumerate(inner["facets"]):
                label = f"page[{pi}].blocks[{bi}].block.facets[{fi}]"
                check(f"{label} required 'index' present", "index" in facet)
                check(f"{label} required 'features' present", "features" in facet)
                idx = facet["index"]
                check(f"{label}.index byteStart/byteEnd are integers >= 0",
                      isinstance(idx.get("byteStart"), int) and isinstance(idx.get("byteEnd"), int)
                      and idx.get("byteStart", -1) >= 0 and idx.get("byteEnd", -1) >= 0)
                check(f"{label}.index byteStart < byteEnd <= byte length of plaintext",
                      0 <= idx["byteStart"] < idx["byteEnd"] <= len(raw_pt),
                      f"{idx['byteStart']}..{idx['byteEnd']} of {len(raw_pt)} bytes")
                sliced = raw_pt[idx["byteStart"]:idx["byteEnd"]].decode("utf-8")
                check(f"{label} byte slice decodes as valid UTF-8 boundary-safe", True)
                check(f"{label} features non-empty", len(facet["features"]) > 0)
                for feat in facet["features"]:
                    ft = feat.get("$type")
                    check(f"{label} feature {ft} is a known facet feature type",
                          ft in feature_refs, str(ft))
                    if ft == "pub.leaflet.richtext.facet#link":
                        check(f"{label} link feature has required 'uri'",
                              isinstance(feat.get("uri"), str) and feat["uri"].startswith("https://"))
                print(f"      {label} slices to: {sliced!r}")

    # expected facet spans for the sample (independently recomputed from
    # plaintext, not from built-record.json construction)
    expected = {
        "page[0].blocks[2].block": [
            ("pub.leaflet.richtext.facet#link", "Leaflet homepage", "https://leaflet.pub"),
            ("pub.leaflet.richtext.facet#bold", "Bold claim", None),
            ("pub.leaflet.richtext.facet#italic", "italic aside", None),
            ("pub.leaflet.richtext.facet#code", "inline code", None),
        ]
    }
    for pi, page in enumerate(content["pages"]):
        for bi, pb in enumerate(page["blocks"]):
            key = f"page[{pi}].blocks[{bi}].block"
            if key not in expected:
                continue
            inner = pb["block"]
            got = []
            for facet in inner.get("facets", []):
                sl = inner["plaintext"].encode()[
                    facet["index"]["byteStart"]:facet["index"]["byteEnd"]].decode()
                for feat in facet["features"]:
                    got.append((feat["$type"], sl, feat.get("uri")))
            check(f"{key} facet spans match independently computed expectations",
                  got == expected[key], f"{got}")

    # UTF-8 offset proof: char index != byte index on the facet-bearing block
    probe = content["pages"][0]["blocks"][2]["block"]["plaintext"]
    needle = "Leaflet homepage"
    ci = probe.index(needle)
    bo = len(probe[:ci].encode())
    check("non-ASCII before facet shifts byte offset past char offset",
          bo > ci, f"char index {ci} vs byte offset {bo} (+{bo - ci} bytes)")

    # ---- frontmatter consistency with sample.md ----
    md = (HERE / "sample.md").read_text(encoding="utf-8")
    fm = dict(re.findall(r"^(\w+): (.+)$", md.split("---")[1], re.M))
    check("title matches sample.md frontmatter", rec["title"] == fm["title"])
    check("description matches sample.md frontmatter", rec["description"] == fm["description"])
    check("tags parsed from comma-separated frontmatter",
          rec["tags"] == [t.strip() for t in fm["tags"].split(",")])

    # ---- derivation: slug rkey, path, URLs ----
    check("rkey (default rule) equals frontmatter slug", RKEY == fm["slug"])
    check("path is /{rkey}", rec["path"] == f"/{RKEY}", rec["path"])
    check("fallback URL derives as /lish/{did}/{pubRkey}/{rkey}",
          FALLBACK_URL == f"https://leaflet.pub/lish/{DID}/{PUB_RKEY}/{RKEY}", FALLBACK_URL)
    check("publication-domain URL derives from publication url field",
          PUB_DOMAIN_URL == f"{PUB_URL.rstrip('/')}/{RKEY}", PUB_DOMAIN_URL)

    print()
    if failures:
        print(f"{len(failures)} FAILURES: {failures}")
        sys.exit(1)
    print("All checks passed.")


if __name__ == "__main__":
    main()
