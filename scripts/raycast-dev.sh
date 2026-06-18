#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXT="$ROOT/packages/better-screenshoot"

if [[ ! -d "$EXT" ]]; then
  echo "Extension folder not found: $EXT" >&2
  exit 1
fi

echo "Raycast extension: $EXT"
echo ""
echo "Before first run, import in Raycast:"
echo "  1. Open Raycast"
echo "  2. Run: Import Extension"
echo "  3. Select: $EXT"
echo ""

cd "$EXT"
npm install
exec npm run dev
