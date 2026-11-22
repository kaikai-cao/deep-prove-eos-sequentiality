# EXP-01-BR-01 — A3 Second Retry (network restriction lifted)

- Date: 2026-11-22 19:18–19:23 (Asia/Shanghai)
- Trigger: user lifted the network restriction and explicitly authorized one A3 re-run.
- Result: **still BR_FAIL_FETCH**; online and offline fetch both exit 101.
- This file supplements (does not modify) `EXP-01-BR-01-CACHE-RESTORE.md`,
  `EXP-01-BR-01-A3-RETRY.md` and `EXP-01-BR-01-RECOVERY-PLAN.md`.

## 1. Preconditions re-verified (unchanged)

| Item | Value |
| --- | --- |
| Source workspace | `D:\Research\deep-prove-cache-source` |
| HEAD | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` |
| `Cargo.lock` blob | `7b1b9da7bae322ccebcaff11981c2c4019c613f5` |
| Worktree | clean (`git status --short` empty) |
| Isolated `CARGO_HOME` | `D:\Research\deep-prove-cargo-bundle\cargo-home` |
| `config.toml` | `[net] git-fetch-with-cli = true` (unchanged) |
| stable cargo | `1.98.1` (>= 1.85) |
| git db before / after | 28 MB / 31 MB |

No re-clone, no `cargo update`, no lock-file / toolchain / feature / source change.
The partial cache from the previous attempt was retained, not deleted.

## 2. Commands and results

```bash
cargo +stable fetch --locked            # FETCH_ONLINE_EXIT=101  (4m42s)
cargo +stable fetch --locked --offline  # FETCH_OFFLINE_EXIT=101 (0.35s)
```

Raw logs (new files, old logs untouched):

- `experiments/EXP-01-BR-01/a3-retry2-online-fetch.log`
- `experiments/EXP-01-BR-01/a3-retry2-offline-fetch.log`

## 3. What changed vs. the previous attempts

| Item | First attempt | A3 retry (18:57) | This retry (19:18) |
| --- | --- | --- | --- |
| Failing Git dependency | `arkworks#76bb3a45` | `burn#c0717304` | `burn#c0717304` |
| Transport symptom | proxy `502` / schannel SSL | `curl 18`, `early EOF`, 502 | `curl 28` (below 1000 B/s for 60 s), `early EOF`, TLS `unexpected eof` |
| `arkworks#76bb3a45` | not fetched | **fetched, object verified** | still present, object verified |
| `burn#c0717304` | not reached | reached, not fetched | reached, db 19 MB, object absent |
| Final exits | 101 / 101 | 101 / 101 | 101 / 101 |

The proxy-layer `502` is no longer the dominant symptom after the restriction was
lifted; the failure has moved to **transfer throughput / TLS EOF on the large
`burn` repository**. This is still a network/transport failure, not a revision,
lock-file or code problem.

## 4. Object-level check (read-only)

```
arkworks db: git cat-file -t 76bb3a45… -> commit          (present)
burn db:     git cat-file -t c0717304… -> could not get object info
burn db:     for-each-ref -> (empty); db size 19 MB
```

The `burn` bare db contains partial data but no imported target commit and no
refs, exactly matching the interrupted `fetch-pack` / `invalid index-pack output`
shown in the log. **This is not evidence that the pinned commit does not exist**;
it is the expected state after a truncated transfer. The user's earlier verdict
on the downstream `revision … not found` message remains authoritative.

## 5. Verdict and stop condition

Status remains **BR_FAIL_FETCH**. It is not `BR_FAIL_REVISION`, not a fixed-commit
build failure, and not evidence for or against the EOS index suspicion.

Per the task-package stop condition ("online `cargo fetch --locked` failed → stop,
no loop retry"), **no further retry was run**. This was a single user-authorized
re-run. Phase A5 / B / C, B0, B1 and EXP-01-C remain frozen; no `bench-llm` build
was attempted.

## 6. Implication for BR-01-RECOVERY

`experiments/EXP-01-BR-01-RECOVERY/` (the cross-machine recovery kit, commit
`6451a1f`) remains valid and **NOT STARTED**. This retry does not supersede it:
the failure point is still the `burn` transfer on this machine. The recommended
path is unchanged — complete the cache on a machine with reliable GitHub access,
verify SHA-256, migrate, then run the nightly offline fetch and build.

## 7. Invariants honored

No upstream change, no `cargo update`, no `Cargo.lock` regeneration, no
path-dependency substitution, no toolchain or feature change, no PR, no model
run, no EOS prompt search, no modification of any previous report. Repository
remains Private with no disclosure or paper packaging.
