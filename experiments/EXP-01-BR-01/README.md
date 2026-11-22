# EXP-01-BR-01 evidence

Fixed-version Cargo cache restoration attempt. Final status: **BR_FAIL_FETCH**.

Baseline: upstream `Lagrange-Labs/deep-prove` @
`9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`, `Cargo.lock` blob
`7b1b9da7bae322ccebcaff11981c2c4019c613f5`, toolchain `nightly-2026-01-27`,
target `cargo build --release -p zkml --bin bench-llm`.

## Files

| File | Content |
| --- | --- |
| `README.md` | This index |
| `environment.txt` | Host, toolchain, network and path record |
| `cache-source-manifest.json` | Machine-readable per-phase status and invariants |
| `a0-clone.log` | Phase A0 clone / fetch / checkout of the pinned commit |
| `a1-version-check.log` | Phase A1 commit + Cargo.lock blob verification |
| `a2a3-fetch.log` | Combined A2/A3 raw log (both fetch commands) |
| `fetch-online.log` | A3 online `cargo +stable fetch --locked` segment (exit 101) |
| `fetch-offline.log` | A3 offline `--locked --offline` validation segment (exit 101) |
| `git-source-locks.txt` | 9 unique Git sources extracted from the fixed Cargo.lock |
| `SHA256SUMS.txt` | SHA-256 of all text evidence files |

## What passed

- A0: independent clone/fetch/checkout of the pinned commit, all exit 0.
- A1: HEAD and Cargo.lock blob match the fixed baseline exactly; worktree clean.
- A2: isolated `CARGO_HOME` with stable cargo 1.98.1 (>= 1.85).
- A4: 9 Git sources extracted from the fixed lock file; all pinned commits
  match the user-supplied list.

## What failed / was not run

- A3: online `cargo fetch --locked` exit 101 due to proxy `CONNECT 502` /
  schannel SSL while updating `a16z/arkworks-algebra#76bb3a45`; offline
  validation consequently exit 101. `git-fetch-with-cli` was enabled in the
  isolated CARGO_HOME (cargo official setting), but three subsequent git CLI
  probes still returned 502. Per the task-package stop condition, no further
  retry loop was run.
- A5: not executed — there is no dependency bundle to archive.
- Phase B (cache migration / offline fetch) and Phase C (bench-llm build):
  not executed.

## Non-goals honored

No upstream source change, no `cargo update`, no Cargo.lock regeneration, no
path-dependency substitution, no toolchain or feature change. No model run, no
EOS prompt search, no B1 and no EXP-01-C. This stage neither proves nor
disproves the EOS suspicion.
