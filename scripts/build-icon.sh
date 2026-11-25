#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v vectorcraft-cli >/dev/null 2>&1; then
  printf '%s\n' 'vectorcraft-cli is required to generate the MOTAA Desktop icon.' >&2
  exit 1
fi

mkdir -p assets/icon
steps_json=$(mktemp "${TMPDIR:-/tmp}/motaa-icon-steps.XXXXXX.json")
iconset_dir=$(mktemp -d "${TMPDIR:-/tmp}/motaa-iconset.XXXXXX")
trap 'rm -rf "$steps_json" "$iconset_dir"' EXIT

python3 scripts/generate-icon-steps.py "$steps_json"

vectorcraft-cli --safe-mode run \
  --commands "$steps_json" \
  --export assets/icon/motaa-icon.vectorcraft \
  --export assets/icon/motaa-icon.svg \
  --export assets/icon/motaa-icon-1024.png \
  --scale 1 >/dev/null

# Verify native document opens cleanly
vectorcraft-cli info assets/icon/motaa-icon.vectorcraft >/dev/null

# In-app 256x256 PNG asset for GPUI sidebar header
sips -z 256 256 assets/icon/motaa-icon-1024.png --out assets/app-icon-256.png >/dev/null

# Build native macOS AppIcon.icns when running on macOS
if [[ $(uname -s) == Darwin ]]; then
  appiconset="$iconset_dir/AppIcon.iconset"
  mkdir -p "$appiconset"
  for size in 16 32 128 256 512; do
    double=$((size * 2))
    sips -z "$size" "$size" assets/icon/motaa-icon-1024.png --out "$appiconset/icon_${size}x${size}.png" >/dev/null
    sips -z "$double" "$double" assets/icon/motaa-icon-1024.png --out "$appiconset/icon_${size}x${size}@2x.png" >/dev/null
  done
  iconutil -c icns "$appiconset" -o assets/AppIcon.icns
fi

printf '%s\n' \
  'assets/icon/motaa-icon.vectorcraft' \
  'assets/icon/motaa-icon.svg' \
  'assets/icon/motaa-icon-1024.png' \
  'assets/app-icon-256.png' \
  'assets/AppIcon.icns'
