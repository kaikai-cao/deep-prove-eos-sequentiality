#!/usr/bin/env bash
# EXP-01-B0: end-to-end smoke test of pinned upstream v1 path.
# Usage: bash run_b0_smoke.sh <upstream_dir> <artifact_dir>
# All raw output is tee'd to <artifact_dir>/smoke-console.log.
set -o pipefail
UP="${1:?upstream dir required}"
ART="${2:?artifact dir required}"
cd "$UP" || exit 1
mkdir -p "$ART"

echo "=== upstream HEAD ==="
git rev-parse HEAD | tee "$ART/upstream-head.txt"
git rev-parse HEAD:zkml/src/model/llm.rs HEAD:zkml/src/bin/bench/llm.rs deep-prove/src/middleware/llm.rs \
  | tee "$ART/source-blob-shas.txt"

echo "=== rust versions ==="
rustc --version | tee "$ART/rustc-version.txt"
cargo --version | tee "$ART/cargo-version.txt"
rustup show active-toolchain | tee "$ART/active-toolchain.txt"

echo "=== B0 command: sequence 8 (prompt 6 + 2 generated) ==="
START=$(date +%s)
RUST_BACKTRACE=1 timeout 1800 cargo run --release --bin bench-llm -- \
  --model gpt2 --hf openai-community/gpt2 --sequence 8 \
  2>&1 | tee "$ART/smoke-console.log"
EC=${PIPESTATUS[0]}
END=$(date +%s)
echo "B0_EXIT=$EC" | tee "$ART/smoke-exit.txt"
echo "B0_SECONDS=$((END-START))" | tee -a "$ART/smoke-exit.txt"

echo "=== produced files ==="
find . -maxdepth 2 -name "bench*.csv" -o -name "*.pk" -o -name "*.vk" | tee "$ART/produced-files.txt"
sha256sum bench*.csv 2>/dev/null | tee "$ART/csv-hashes.txt"
