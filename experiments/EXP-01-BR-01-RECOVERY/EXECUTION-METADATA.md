# BR-01-RECOVERY — execution metadata (this attempt)

- Date: 2026-11-22 19:52–20:08 (Asia/Shanghai)
- Final status: **RECOVERY_FAIL** (A′2 online fetch failed on the local machine)

## Scope interpretation

The task package's A′0 says: if no second host is available, record
`BLOCKED_NO_SECOND_HOST` and do not treat the current machine as the remote
host. The user's final instruction for this round — "try once more on our
machine; if it fails we give up" — overrides that for this single attempt.
So this round is an explicit **local final attempt**, executed as if the local
fresh Cargo Home were the remote home. No second host was available and none
was simulated as being one; this is disclosed rather than claimed.

## Fixed baseline (verified before start)

| Item | Value |
| --- | --- |
| Source HEAD | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` |
| `Cargo.lock` blob | `7b1b9da7bae322ccebcaff11981c2c4019c613f5` |
| `verify-source.sh` | `SOURCE_IDENTITY=OK` |
| nightly toolchain | `rustc 1.95.0-nightly (474276961 2026-01-26)`; `cargo 1.95.0-nightly` |
| stable cargo | `1.98.1` |
| Old partial cache | `cargo-home` 31 MB — preserved, untouched |

## Fresh-path requirement (user-added detail)

All three target paths were confirmed **absent** before execution:

| Path | Pre-check |
| --- | --- |
| `D:\Research\deep-prove-cargo-bundle\cargo-home-recovery` | ABSENT |
| `D:\Research\deep-prove-cargo-bundle\recovery-remote-home` | ABSENT |
| `D:\Research\deep-prove-cargo-bundle\recovery-bundle.tar.gz` | ABSENT |

This was to prevent the restore script from skipping unpack on an existing
partial directory. They remain absent after the failed run (no bundle created).

## Executed steps

| Step | Command | Result |
| --- | --- | --- |
| A′1 | `bash scripts/verify-source.sh "$SRC"` | `SOURCE_IDENTITY=OK` |
| A′2 | fresh `CARGO_HOME=.../recovery-remote-home`, `cargo +stable fetch --locked` | **exit 124** (900 s timeout wrapper) |
| A′3 | `cargo +stable fetch --locked --offline` | NOT RUN (stop condition) |
| A′4 | `scripts/pack-bundle.sh` | NOT RUN |
| B′1/B′2 | migrate + nightly offline fetch | NOT RUN |
| C′1 | `bench-llm` build | NOT RUN |

## A′2 detail

- Fresh home created with `[net] git-fetch-with-cli = true`.
- `arkworks-algebra#76bb3a45`: retry after one 300 s timeout, then succeeded
  (`new ref` present, commit object verified).
- `burn#c0717304`: `fatal: fetch-pack: invalid index-pack output`, retried,
  never completed. Command reached the 900 s cap and was killed (124).
- Final git db: 24 MB. `burn` bare db 12 MB, target commit object absent,
  refs = 0 — consistent with truncated transfer, not a missing revision
  (see `EXP-01-BR-01-NETWORK-CONTROL.md`, API 200 at exact SHA).

## Files

| File | Content |
| --- | --- |
| `recovery-a2-online-fetch.log` | Full A′2 output with terminal exit marker |
| `SHA256SUMS-recovery.txt` | SHA-256 of the above |
| `EXECUTION-METADATA.md` | This file |
| `BUNDLE-SHA256.md` | Still a blank template — no bundle was produced |

## Invariants honored

No `Cargo.lock` / `Cargo.toml` / upstream / toolchain / feature change, no
`cargo update`, no path-dependency substitution, no tarball adoption, no model
run, no B0/B1/C, no modification of previous reports. The 31 MB partial cache
and all prior evidence remain untouched. Repository stays Private.
