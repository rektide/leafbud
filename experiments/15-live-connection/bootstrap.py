#!/usr/bin/env python3
"""Bounded public-bundle discovery + read-only existing-binding validation.

No values from deployment responses or credentials are printed. Output files
must remain under ignored .test-agent/live-connection. No document writes.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import re
import urllib.parse
import urllib.request


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="https://leaflet.pub")
    parser.add_argument("--workspace", type=Path, required=True)
    args = parser.parse_args()
    base = urllib.parse.urlsplit(args.base)
    assert base.scheme == "https" and not base.username and not base.password
    assert base.path in ("", "/") and not base.query and not base.fragment
    origin = args.base.rstrip("/")
    scratch = Path(__file__).resolve().parents[2] / ".test-agent/live-connection"
    scratch.mkdir(parents=True, exist_ok=True)
    scratch.chmod(0o700)
    opener = urllib.request.build_opener(NoRedirect)

    def get(url, headers=None, body=None):
        request = urllib.request.Request(url, headers=headers or {}, data=body)
        with opener.open(request, timeout=30) as response:
            data = response.read(8 * 1024 * 1024 + 1)
            assert len(data) <= 8 * 1024 * 1024, "response size cap"
            return data

    def save(name, value):
        path = scratch / name
        # Create privately even on a permissive umask.
        with os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w") as file:
            json.dump(value, file)
        path.chmod(0o600)

    page = get(origin).decode()
    scripts = list(dict.fromkeys(re.findall(r'<script[^>]+src="([^"]+)"', page)))
    assert len(scripts) <= 48, "script count cap"
    candidates = set()
    for script in scripts:
        path = urllib.parse.urlsplit(script)
        if path.scheme or path.netloc or not path.path.startswith("/_next/"):
            continue
        text = get(origin + script).decode()
        urls = set(re.findall(r'https://[a-zA-Z0-9.-]+\.supabase\.co', text))
        keys = set(re.findall(r'eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+', text))
        for url in urls:
            for key in keys:
                claims = json.loads(base64.urlsafe_b64decode(key.split(".")[1] + "==="))
                # Discovery only, not JWT verification. Exclude service-role keys.
                if claims.get("role") == "anon" and claims.get("ref") == urllib.parse.urlsplit(url).hostname.split(".")[0]:
                    candidates.add((url, key))
    assert len(candidates) == 1, "ambiguous/missing public project config; supply explicit config instead"
    url, key = candidates.pop()
    save("bootstrap.json", {"url": url, "key": key})
    print("public_bootstrap=found public_key_role=anon")

    token = os.environ.get("LEAFLET_AUTH_TOKEN")
    auth_path = Path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config"))) / "leafbud/auth-token"
    if token is None and auth_path.exists():
        token = auth_path.read_text().strip()
    headers = {"Content-Type": "application/json"}
    if token:
        assert token and all(ord(c) == 0x21 or 0x23 <= ord(c) <= 0x2b or 0x2d <= ord(c) <= 0x3a or 0x3c <= ord(c) <= 0x5b or 0x5d <= ord(c) <= 0x7e for c in token)
        headers["Cookie"] = "auth_token=" + token
    for file in args.workspace.resolve().rglob("*.mdx"):
        text = file.read_text()
        fields = {}
        for name in ("token", "root"):
            match = re.search(r'["\']?' + name + r'["\']?\s*:\s*["\']?([a-f0-9-]{36})', text)
            if match:
                fields[name] = match[1]
        if len(fields) != 2:
            continue
        response = json.loads(get(origin + "/api/rpc/get_leaflet_data", headers,
                                  json.dumps({"token_id": fields["token"]}).encode()))
        result = response.get("result", {})
        assert result.get("error") is None and result.get("data", {}).get("root_entity") == fields["root"], "binding validation failed"
        save("binding.json", fields)
        print("existing_binding=validated workspace_modified=false")
        return
    raise RuntimeError("no existing binding found")


if __name__ == "__main__":
    # A security boundary: URL-bearing urllib errors and response content must
    # not reach terminal logs. This script never suppresses failure status.
    try:
        main()
    except Exception as error:
        print("probe_bootstrap_failed category=" + type(error).__name__)
        raise SystemExit(1)
