#!/usr/bin/env bash
# Fetch the SingleStepTests nes6502 corpus used by src/cpu/tests/harte.rs.
#
#   https://github.com/SingleStepTests/65x02  (nes6502/v1)
#
# 256 files, one per opcode, 10,000 cases each. ~1.1 GB, gitignored.
set -euo pipefail

BASE="https://raw.githubusercontent.com/SingleStepTests/65x02/main/nes6502/v1"
DEST="$(dirname "$0")/../jc-nes/.harte/v1"

mkdir -p "$DEST"
printf '%02x\n' $(seq 0 255) \
  | xargs -P 10 -I{} curl -sfL --retry 3 -o "$DEST/{}.json" "$BASE/{}.json"

echo "fetched $(ls -1 "$DEST"/*.json | wc -l) files into $DEST"
