#!/usr/bin/env bash
# ============================================================================
# putRecord.sh — NOT EXECUTED. Dry-run artifact for experiment 03.
#
# This script has never been run. It exists so a later, operator-gated stage
# has the exact write call ready. Per experiments/README.md rule 3, no write
# is executed against any PDS without explicit operator go-ahead.
#
# Auth assumed (social-cli convention, see publish.ts):
#   1. An app password created in Bluesky settings (Settings → App passwords).
#   2. com.atproto.server.createSession with identifier=$ATPROTO_HANDLE,
#      password=$ATPROTO_APP_PASSWORD against the PDS → returns accessJwt
#      (short-lived) + did. No OAuth, no refresh token handling in this v0
#      sketch — one session per publish.
#   3. Bearer $accessJwt on com.atproto.repo.putRecord (upsert semantics).
#
# Usage (LATER, gated):  ATPROTO_HANDLE=... ATPROTO_APP_PASSWORD=... \
#                        GATED_OPERATOR_APPROVAL=yes ./putRecord.sh
# ============================================================================
set -euo pipefail

: "${ATPROTO_HANDLE:?set ATPROTO_HANDLE (e.g. jauntywunderkind.bsky.social)}"
: "${ATPROTO_APP_PASSWORD:?set ATPROTO_APP_PASSWORD (an app password, NOT the account password)}"
: "${GATED_OPERATOR_APPROVAL:?this script writes to production; only the operator sets GATED_OPERATOR_APPROVAL=yes at the gated stage}"

PDS="https://puffball.us-east.host.bsky.network"
DID="did:plc:zjbq26wybii5ojoypkso2mso"
COLLECTION="site.standard.document"
RKEY="springleaf-protocol-probe"   # default rule: rkey = slug (experiment 03 derivation)
RECORD_FILE="$(cd "$(dirname "$0")" && pwd)/built-record.json"

# 1. Session (app-password flow). accessJwt is captured, never printed.
SESSION_JSON="$(curl -sS -X POST "$PDS/xrpc/com.atproto.server.createSession" \
  -H 'Content-Type: application/json' \
  -d "{\"identifier\":\"$ATPROTO_HANDLE\",\"password\":\"$ATPROTO_APP_PASSWORD\"}")"
ACCESS_JWT="$(printf '%s' "$SESSION_JSON" | jq -r '.accessJwt')"
SESSION_DID="$(printf '%s' "$SESSION_JSON" | jq -r '.did')"
[ "$SESSION_DID" = "$DID" ] || { echo "session did $SESSION_DID != expected $DID" >&2; exit 1; }

# 2. putRecord (upsert). Writes at://$DID/site.standard.document/$RKEY
#    Expected URL after success: https://technautical.leaflet.pub/springleaf-protocol-probe
#    (fallback: https://leaflet.pub/lish/$DID/3molsbwkh522o/$RKEY)
jq -n --arg repo "$DID" --arg collection "$COLLECTION" --arg rkey "$RKEY" \
     --slurpfile record "$RECORD_FILE" \
     '{repo: $repo, collection: $collection, rkey: $rkey, record: $record[0]}' \
| curl -sS -X POST "$PDS/xrpc/com.atproto.repo.putRecord" \
    -H "Authorization: Bearer $ACCESS_JWT" \
    -H 'Content-Type: application/json' \
    -d @- | jq '{uri, cid}'
