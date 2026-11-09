#!/usr/bin/env python3
"""Read-only Leaflet draft-internals probe (experiment 02).

Token is read at runtime from ~/.config/leafbud/auth-token (or $LEAFLET_TOKEN)
and is never printed or written anywhere. All requests are GETs or the
documented read RPCs; no push/mutations are issued.

Usage:
  probe_leaflet.py home                 # list draft token ids from /home
  probe_leaflet.py data <token-id>      # get_leaflet_data (raw JSON, stdout)
  probe_leaflet.py facts <root-entity>... # getFactsForRoots (raw JSON, stdout)
  probe_leaflet.py pull <token-id> [client-group-id]
                                        # one full-snapshot Replicache pull
  probe_leaflet.py roots <token-id>     # convenience: entity-set roots for a token
"""
import json
import os
import sys
import urllib.request
import uuid

BASE = "https://leaflet.pub"
TOKEN_PATH = os.path.expanduser("~/.config/leafbud/auth-token")


def token() -> str:
    v = os.environ.get("LEAFLET_TOKEN") or open(TOKEN_PATH).read().strip()
    if not v:
        raise SystemExit("no auth token available")
    return v


def get(path: str) -> bytes:
    req = urllib.request.Request(BASE + path,
                                 headers={"Cookie": f"auth_token={token()}"})
    with urllib.request.urlopen(req) as r:
        return r.read()


def rpc(command: str, body: dict) -> dict:
    req = urllib.request.Request(
        f"{BASE}/api/rpc/{command}",
        data=json.dumps(body).encode(),
        headers={"Content-type": "application/json",
                 "Cookie": f"auth_token={token()}"},
        method="POST")
    with urllib.request.urlopen(req) as r:
        return json.loads(r.read())


def cmd_home():
    import re
    html = get("/home").decode()
    ids = re.findall(r'href="/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-'
                     r'[0-9a-f]{4}-[0-9a-f]{12})"', html)
    for i in dict.fromkeys(ids):  # dedupe, keep order (newest first)
        print(i)


def cmd_data(token_id: str):
    uuid.UUID(token_id)  # reject junk before it hits the wire
    print(json.dumps(rpc("get_leaflet_data", {"token_id": token_id}), indent=1))


def cmd_roots(token_id: str):
    d = rpc("get_leaflet_data", {"token_id": token_id})
    data = d["result"]["data"] or {}
    seen = set()
    for r in data.get("permission_token_rights") or []:
        es = r.get("entity_sets")
        if not es:
            continue
        root = es.get("root_entity")
        if root and root not in seen:
            seen.add(root)
            print(root)
        for pt in es.get("permission_tokens") or []:
            if pt.get("root_entity") not in seen:
                seen.add(pt["root_entity"])
                print(pt["root_entity"])


def cmd_facts(*roots: str):
    for r in roots:
        uuid.UUID(r)
    print(json.dumps(rpc("getFactsForRoots", {"roots": list(roots)}), indent=1))


def cmd_pull(token_id: str, group: str = "leafbud-probe"):
    # PullRequestV1, cookie=null -> legacy full-snapshot response
    # (the CVR path needs a Redis-backed cookie we don't have; null always
    # yields the full patch -- src/replicache/cvr.ts buildPullResponse).
    uuid.UUID(token_id)
    body = {
        "pullRequest": {
            "pullVersion": 1,
            "schemaVersion": "",
            "profileID": "00000000-0000-0000-0000-000000000000",
            "cookie": None,
            "clientGroupID": group,
        },
        "token_id": token_id,
    }
    print(json.dumps(rpc("pull", body), indent=1))


def main():
    cmds = {"home": cmd_home, "data": cmd_data, "roots": cmd_roots,
            "facts": cmd_facts, "pull": cmd_pull}
    if len(sys.argv) < 2 or sys.argv[1] not in cmds:
        print(__doc__)
        sys.exit(2)
    cmds[sys.argv[1]](*sys.argv[2:])


if __name__ == "__main__":
    main()
