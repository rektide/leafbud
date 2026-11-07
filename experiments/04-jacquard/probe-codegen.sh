#!/usr/bin/env bash
# Probe: can jacquard's prebuilt codegen generate typed leaflet record types
# from the SHA-pinned leaflet checkout? (It can — verified 2026-11-07.)
#
# Inputs (read-only):
#   - ~/archive/nonbinary.computer/jacquard — Jacquard checkout @ 8bf1f4146 (2026-06-28)
#     prebuilt binary: target/release/jacquard-codegen (built 2026-06-25)
#   - ~/archive/hyperlink-academy/leaflet — leaflet checkout @ 600c7b5f (2026-11-06),
#     the same pin springleaf's design doc cites as ground truth.
#
# Output: standalone generated module tree in $OUT (throwaway, /tmp/opencode).
# No credentials involved; no network; nothing written into either archive.
set -euo pipefail

JQ=~/archive/nonbinary.computer/jacquard
LEAFLET=~/archive/hyperlink-academy/leaflet
OUT=/tmp/opencode/04-jacquard

rm -rf "$OUT/lex-in" "$OUT/lex-out"
mkdir -p "$OUT/lex-in"

# Flatten leaflet's lexicon tree (filename is irrelevant; NSID comes from JSON "id").
cd "$LEAFLET/lexicons"
fd -e json . | while read -r f; do
  cp "$f" "$OUT/lex-in/$(echo "$f" | tr '/.' '__').json"
done
echo "staged $(ls "$OUT/lex-in" | wc -l) lexicon JSONs (incl. vendored atproto/bsky + parts)"

# Generate. To narrow to leaflet-only namespaces, stage only
# pub/leaflet/** and site/standard/** plus their com.atproto.repo.strongRef /
# com.atproto.label.defs dependencies.
"$JQ/target/release/jacquard-codegen" -i "$OUT/lex-in" -o "$OUT/lex-out"

echo '--- generated namespaces ---'
ls "$OUT/lex-out"
echo '--- site.standard.document LOC ---'
wc -l "$OUT/lex-out"/site_standard/document.rs
