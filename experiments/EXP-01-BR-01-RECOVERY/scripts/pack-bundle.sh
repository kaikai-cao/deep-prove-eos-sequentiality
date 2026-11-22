#!/usr/bin/env bash
# Pack only the git/ and registry/ directories of an isolated CARGO_HOME.
# Run on the remote (stable-network) machine, after BOTH
#   cargo +stable fetch --locked
#   cargo +stable fetch --locked --offline
# have exited 0. config.toml is intentionally excluded (machine-specific).
set -euo pipefail

CARGO_HOME_DIR="${1:?usage: pack-bundle.sh <isolated-cargo-home> <output.tar.gz>}"
OUT="${2:?usage: pack-bundle.sh <isolated-cargo-home> <output.tar.gz>}"

for sub in git registry; do
  if [ ! -d "$CARGO_HOME_DIR/$sub" ]; then
    echo "FAIL: missing $CARGO_HOME_DIR/$sub"
    exit 1
  fi
done

mkdir -p "$(dirname "$OUT")"
tar -czf "$OUT" -C "$CARGO_HOME_DIR" git registry
( cd "$(dirname "$OUT")" && sha256sum "$(basename "$OUT")" > "$(basename "$OUT").sha256" )

echo "BUNDLE=$OUT"
echo "--- SHA-256 ---"
cat "$OUT.sha256"
