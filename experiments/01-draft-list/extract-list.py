#!/usr/bin/env python3
"""Prototype of `springleaf drafts list` extraction (experiment 01).

Reads /home HTML (from file, or fetched live with the cookie from
~/.config/leaflet/auth-token), parses the RSC flight payload, and emits the
draft list as JSON (schema v1, see REPORT.md) or a human table.

Read-only. The auth token is read at runtime and never printed or logged.

Usage:
  extract-list.py FIXTURE_OR_HTML_FILE [--human]
  extract-list.py --live [--human] [--fetch]   # fetches https://leaflet.pub/home
                                              # --fetch: enrich each id via
                                              # get_leaflet_data (N+1 requests)
"""
import argparse
import datetime
import json
import re
import sys
import urllib.request

HOME_URL = "https://leaflet.pub/home"
RPC_URL = "https://leaflet.pub/api/rpc/get_leaflet_data"
TOKEN_PATH = __import__("pathlib").Path.home() / ".config/leaflet/auth-token"
UUID_HREF = re.compile(r'href="/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"')
PUSH = re.compile(r"self\.__next_f\.push\((\[.*?\])\)</script>", re.S)
KNOWN_ROUTES = {
    "reader", "home", "manual", "manifest.json", "favicon.ico", "apple-icon.png",
    "lish", "api", "_next", "icon", "createPub",
}


def flight_stream(html: str) -> str:
    frags = []
    for m in PUSH.finditer(html):
        frags.append(json.loads(m.group(1))[1])
    return "".join(frags)


def enclosing_object(stream: str, key: str):
    i = stream.find(f'"{key}":')
    if i < 0:
        return None
    depth = 0
    for j in range(i - 1, -1, -1):
        c = stream[j]
        if c in "}]":
            depth += 1
        elif c in "{[":
            if depth == 0 and c == "{":
                obj, _ = json.JSONDecoder().raw_decode(stream, j)
                return obj
            depth -= 1
    return None


def uuid_v7_ms(tid: str):
    if tid.split("-")[2][0] != "7":
        return None  # pre-v7 (v4) tokens carry no timestamp
    return int(tid.replace("-", "")[:12], 16)


def iso(ms) -> str:
    return datetime.datetime.fromtimestamp(ms / 1000, datetime.timezone.utc).isoformat()


def derive_url(link: dict) -> str:
    doc = link.get("documents") or {}
    path = (doc.get("data") or {}).get("path") or ""
    pubs = link.get("publications") or {}
    url = ((pubs.get("record") or {}).get("url") or "").rstrip("/")
    if url and path:
        return url + path
    m = re.match(r"at://([^/]+)/site\.standard\.publication/([^/]+)", link.get("publication") or "")
    d = re.match(r"at://([^/]+)/site\.standard\.document/([^/]+)", link.get("doc") or "")
    if m and d:
        return f"https://leaflet.pub/lish/{m.group(1)}/{m.group(2)}/{d.group(2)}"
    return ""


def classify(entry: dict) -> dict:
    """draft | published | in_publication (linked, no doc; scheduled_* may be set)."""
    links = entry["permission_tokens"].get("leaflets_in_publications") or []
    published = [l for l in links if l.get("doc")]
    if published:
        l = max(published, key=lambda x: (x["documents"] or {}).get("data", {}).get("publishedAt") or "")
        doc = (l.get("documents") or {}).get("data") or {}
        pub = (l.get("publications") or {}).get("record") or {}
        return {"state": "published",
                "publication": {"uri": l.get("publication"), "name": pub.get("name"), "url": pub.get("url")},
                "published_doc_uri": l.get("doc"),
                "published_at": doc.get("publishedAt"),
                "published_url": derive_url(l)}
    if links:
        return {"state": "in_publication",
                "publication": {"uri": links[0].get("publication"), "name": None, "url": None},
                "published_doc_uri": None, "published_at": None, "published_url": None}
    return {"state": "draft", "publication": None, "published_doc_uri": None,
            "published_at": None, "published_url": None}


def build_list(html: str, source: str) -> dict:
    stream = flight_stream(html)
    ident = enclosing_object(stream, "permission_token_on_homepage")
    if ident is None:
        sys.exit("no permission_token_on_homepage in flight payload")
    hrefs = UUID_HREF.findall(html)
    hrefs = [h for h in hrefs if h.split("-")[0] not in KNOWN_ROUTES and not h.startswith("lish")]

    leaflets = []
    for rows, contributor in ((ident.get("permission_token_on_homepage") or [], False),
                              (ident.get("contributor_leaflets") or [], True)):
        for row in rows:
            pt = row["permission_tokens"]
            ms = uuid_v7_ms(pt["id"])
            rights = next((r for r in pt.get("permission_token_rights") or []
                           if r.get("token") == pt["id"]), None)
            leaflets.append({
                "token_id": pt["id"],
                "created_at": row.get("created_at") or (iso(ms) if ms else None),
                "created_at_source": "row" if row.get("created_at") else ("uuidv7" if ms else None),
                "uuid_version": 7 if ms else int(pt["id"].split("-")[2][0]),
                "title": pt.get("title"),
                "description": pt.get("description"),
                "root_entity": pt.get("root_entity"),
                "archived": row.get("archived") is not None,
                "on_homepage": not contributor,
                "contributor": contributor,
                "rights": ({"read": rights.get("read"), "write": rights.get("write")}
                           if rights else None),
                **classify({"permission_tokens": pt}),
            })
    # UI order: created_at desc (row value; v7 lexical == chronological only for
    # v7 tokens — legacy v4 tokens sort correctly only by row created_at).
    # Tiebreak token_id desc. Contributor-only leaflets keep their row dates.
    by_id = {}
    for l in leaflets:
        if l["token_id"] in by_id:  # owned + contributor: merge flags
            by_id[l["token_id"]]["contributor"] = True
        else:
            by_id[l["token_id"]] = l
    leaflets = sorted(by_id.values(),
                      key=lambda x: (x["created_at"] or "0000", x["token_id"]), reverse=True)

    prof = ident.get("bsky_profiles") or {}
    mismatch = sorted(set(hrefs) ^ {l["token_id"] for l in leaflets if l["on_homepage"]})
    return {"version": 1, "source": source,
            "fetched_at": ident.get("fetched_at"),
            "identity": {"did": prof.get("did") or ident.get("atp_did"), "handle": prof.get("handle")},
            "count": len(leaflets),
            "href_payload_mismatch": mismatch,
            "leaflets": leaflets}


def enrich(listing: dict) -> dict:
    """--fetch: merge get_leaflet_data per id (adds blocked_by_admin etc.)."""
    req_cookie = "Cookie: auth_token=" + TOKEN_PATH.read_text().strip()
    for l in listing["leaflets"]:
        req = urllib.request.Request(RPC_URL, method="POST",
                                     data=json.dumps({"token_id": l["token_id"]}).encode(),
                                     headers={"content-type": "application/json", "Cookie": req_cookie})
        with urllib.request.urlopen(req) as r:
            data = json.load(r)["result"]["data"]
        if data:
            l["blocked_by_admin"] = data.get("blocked_by_admin")
            l["reply_drafts"] = data.get("reply_drafts")
    listing["source"] = "home-rsc+rpc"
    return listing


def human(listing: dict) -> str:
    out = []
    id_w = max(36, *(len(l["token_id"]) for l in listing["leaflets"])) if listing["leaflets"] else 36
    out.append(f"{'STATE':<15} {'CREATED':<12} {'TITLE':<40} {'TOKEN-ID':<{id_w}} PUBLISHED-URL")
    for l in listing["leaflets"]:
        date = (l["created_at"] or "")[:10]
        title = (l["title"] or "Untitled")
        if len(title) > 38:
            title = title[:35] + "…"
        flag = "*" if l["contributor"] else ""
        out.append(f"{l['state']:<15} {date:<12} {title:<40} {l['token_id']:<{id_w}} {l['published_url'] or ''}{flag}")
    n_pub = sum(1 for l in listing["leaflets"] if l["state"] == "published")
    out.append(f"\n{listing['count']} leaflets ({n_pub} published, "
               f"{listing['count'] - n_pub} draft/unpublished)"
               + (f" as {listing['identity']['handle']}" if listing["identity"]["handle"] else ""))
    if listing["href_payload_mismatch"]:
        out.append(f"warning: href/payload id mismatch: {listing['href_payload_mismatch']}")
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("file", nargs="?", help="saved /home HTML (e.g. fixture-home.html)")
    ap.add_argument("--live", action="store_true", help="fetch https://leaflet.pub/home")
    ap.add_argument("--fetch", action="store_true", help="enrich per-id via get_leaflet_data (live only)")
    ap.add_argument("--human", action="store_true")
    a = ap.parse_args()
    if a.live:
        req = urllib.request.Request(HOME_URL, headers={
            "Cookie": "auth_token=" + TOKEN_PATH.read_text().strip(),
            "user-agent": "springleaf-experiment/0"})
        with urllib.request.urlopen(req) as r:
            html = r.read().decode()
        source = "home-rsc"
    elif a.file:
        html = open(a.file).read()
        source = "fixture" if "fixture" in a.file else "home-rsc"
    else:
        ap.error("need FILE or --live")
    listing = build_list(html, source)
    if a.live and a.fetch:
        listing = enrich(listing)
    print(human(listing) if a.human else json.dumps(listing, indent=1, ensure_ascii=False))


if __name__ == "__main__":
    main()
