#!/usr/bin/env bash
# Verify that a copied source workspace is still the fixed upstream revision.
# Read-only: no fetch, no checkout, no modification.
set -euo pipefail

SRC="${1:?usage: verify-source.sh <source-workspace>}"

EXPECTED_HEAD="9d1a53e2ef49ffa2c902b8689cd3c58057a4e662"
EXPECTED_LOCK_BLOB="7b1b9da7bae322ccebcaff11981c2c4019c613f5"

HEAD="$(git -C "$SRC" rev-parse HEAD)"
LOCK_BLOB="$(git -C "$SRC" rev-parse HEAD:Cargo.lock)"
STATUS="$(git -C "$SRC" status --porcelain)"

echo "HEAD=$HEAD"
echo "Cargo.lock_blob=$LOCK_BLOB"
echo "--- git status --short ---"
printf '%s\n' "$STATUS"
echo "--- end status ---"

if [ "$HEAD" != "$EXPECTED_HEAD" ]; then
  echo "FAIL: HEAD mismatch (expected $EXPECTED_HEAD)"
  exit 1
fi
if [ "$LOCK_BLOB" != "$EXPECTED_LOCK_BLOB" ]; then
  echo "FAIL: Cargo.lock blob mismatch (expected $EXPECTED_LOCK_BLOB)"
  exit 1
fi
if [ -n "$STATUS" ]; then
  echo "FAIL: worktree not clean"
  exit 1
fi

echo "SOURCE_IDENTITY=OK"
