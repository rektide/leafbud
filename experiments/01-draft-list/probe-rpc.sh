#!/usr/bin/env sh
# Experiment 01 probe: read-only get_leaflet_data for the given token ids.
# Response envelope is documented in REPORT.md. Raw JSON lands in scratch.
# Usage: probe-rpc.sh <token-id> [more ids...]
set -eu
T=$(cat ~/.config/leafspring/auth-token)
mkdir -p /tmp/opencode/leafspring-01
for id in "$@"; do
  OUT="/tmp/opencode/leafspring-01/rpc-$id.json"
  curl -s -X POST -H 'content-type: application/json' -H "Cookie: auth_token=$T" \
    -d "{\"token_id\":\"$id\"}" -o "$OUT" -w "%{http_code} $id\n" \
    https://leaflet.pub/api/rpc/get_leaflet_data
  echo "  -> $OUT ($(wc -c < "$OUT") bytes)"
done
