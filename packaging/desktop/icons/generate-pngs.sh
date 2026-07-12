#!/usr/bin/env bash
set -euo pipefail

# Render the Popush SVG icon into PNGs at all required hicolor sizes.
# Uses rsvg-convert if available, otherwise falls back to inkscape.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SVG="${SCRIPT_DIR}/dev.popush.Popush.svg"
SIZES=(16 24 32 48 64 128 256 512)

if [[ ! -f "${SVG}" ]]; then
  echo "error: source SVG not found at ${SVG}" >&2
  exit 1
fi

render() {
  local size="$1"
  local out="${SCRIPT_DIR}/dev.popush.Popush-${size}.png"

  if command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w "${size}" -h "${size}" "${SVG}" -o "${out}"
  elif command -v inkscape >/dev/null 2>&1; then
    inkscape "${SVG}" \
      --export-type=png \
      --export-width="${size}" \
      --export-height="${size}" \
      --export-filename="${out}" >/dev/null 2>&1
  else
    echo "error: neither rsvg-convert nor inkscape is installed" >&2
    exit 1
  fi

  echo "rendered ${out}"
}

for size in "${SIZES[@]}"; do
  render "${size}"
done

echo "done: ${#SIZES[@]} icons generated"
