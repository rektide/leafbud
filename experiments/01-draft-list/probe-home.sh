#!/usr/bin/env sh
# Experiment 01 probe: fetch https://leaflet.pub/home with the auth cookie and
# print quick surface stats. Read-only; token read at runtime, never printed.
# Output HTML goes to an uncommitted scratch path (never commit real data).
set -eu
T=$(cat ~/.config/leaflet/auth-token)
OUT="${1:-/tmp/opencode/springleaf-01/home.html}"
curl -s -H "Cookie: auth_token=$T" -o "$OUT" https://leaflet.pub/home
echo "saved: $OUT ($(wc -c < "$OUT") bytes)"
echo "uuid-shaped hrefs: $(grep -oP 'href=\"/([0-9a-f-]{36})\"' "$OUT" | wc -l)"
echo "flight pushes:     $(grep -c 'self.__next_f.push' "$OUT")"
echo "leaflets in payload: $(grep -o 'permission_token_on_homepage' "$OUT" | wc -l)"
