#!/usr/bin/env bash
# On the experiment machine: verify bundle SHA-256, unpack into a NEW isolated
# CARGO_HOME, then run nightly offline fetch (mode=fetch) or the bench-llm build
# (mode=build). Build is gated on the caller having observed a successful fetch.
set -euo pipefail

BUNDLE="${1:?usage: verify-and-build.sh <bundle.tar.gz> <fresh-cargo-home> <source-workspace> [fetch|build]}"
NEW_HOME="${2:?usage: verify-and-build.sh <bundle.tar.gz> <fresh-cargo-home> <source-workspace> [fetch|build]}"
SRC="${3:?usage: verify-and-build.sh <bundle.tar.gz> <fresh-cargo-home> <source-workspace> [fetch|build]}"
MODE="${4:-fetch}"

SHA_FILE="$(dirname "$BUNDLE")/$(basename "$BUNDLE").sha256"
if [ ! -f "$SHA_FILE" ]; then
  echo "FAIL: missing checksum file $SHA_FILE"
  exit 1
fi
( cd "$(dirname "$BUNDLE")" && sha256sum -c "$(basename "$BUNDLE").sha256" )

mkdir -p "$NEW_HOME"
# Only unpack if git/ and registry/ are not already present (idempotence).
if [ ! -d "$NEW_HOME/git" ] && [ ! -d "$NEW_HOME/registry" ]; then
  tar -xzf "$BUNDLE" -C "$NEW_HOME"
fi

export CARGO_HOME="$NEW_HOME"
cd "$SRC"

case "$MODE" in
  fetch)
    echo "MODE=fetch"
    cargo +nightly-2026-01-27 fetch --locked --offline
    echo "NIGHTLY_FETCH_EXIT=0"
    ;;
  build)
    echo "MODE=build"
    cargo +nightly-2026-01-27 build --locked --offline --release \
      -p zkml --bin bench-llm
    echo "BENCH_LLM_BUILD_EXIT=0"
    ;;
  *)
    echo "FAIL: unknown mode '$MODE' (use fetch|build)"
    exit 1
    ;;
esac
