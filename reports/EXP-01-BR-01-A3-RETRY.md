# EXP-01-BR-01 — A3 Retry Record

- Date: 2026-11-22 18:57–19:11 (Asia/Shanghai)
- Trigger: user asked to re-run A3 once ("把 A3 重新跑一下，试试看能不能成功")
- Result: **still BR_FAIL_FETCH**, with partial progress vs. the first attempt
- This file supplements (does not modify) `EXP-01-BR-01-CACHE-RESTORE.md`.

## 1. Command and conditions

Same as the original A3: independent `CARGO_HOME` with
`[net] git-fetch-with-cli = true`, fixed source workspace
`D:\Research\deep-prove-cache-source` at pinned commit `9d1a53e2…`, stable cargo
1.98.1. Pre-flight `git ls-remote` succeeded for both `deep-prove` and
`a16z/arkworks-algebra`, so the network window was open at start.

```bash
cargo +stable fetch --locked          # FETCH_ONLINE_EXIT=101
cargo +stable fetch --locked --offline # FETCH_OFFLINE_EXIT=101
```

Full raw log: `experiments/EXP-01-BR-01/a3-retry-fetch.log`.

## 2. What changed vs. the first A3 attempt

| Item | First attempt (18:41) | Retry (18:57–19:11) |
| --- | --- | --- |
| `a16z/arkworks-algebra#76bb3a45` | 502 / SSL, not fetched | **fetched successfully**, commit object verified present in cargo git db |
| `tracel-ai/burn#c0717304` | not reached | reached, but 3× `CONNECT tunnel failed, 502`; revision absent |
| git cache size | ~0 (git db empty) | 28 MB (arkworks db + checkout present) |
| Final exit | 101 | 101 |

So the retry advanced past the previous blocking repository. The failure simply
moved to the next Git dependency (`burn`) when the proxy window closed again
mid-run. This confirms the failure is **network intermittency**, not a missing
revision, a bad lock file, or a dependency-version problem.

## 3. Failure detail

```
error: failed to get `burn` as a dependency of package `zkml v2.0.1`
Caused by: unable to update https://github.com/tracel-ai/burn.git?rev=c0717304…
Caused by: failed to clone into ...\git\db\burn-6c277d792b0d5d7a
Caused by: revision c071730459a135b4d06ad0e14c8a0e5771e5ba58 not found
Caused by: git fetch ... exit code 128
           fatal: unable to access 'https://github.com/tracel-ai/burn.git/':
                  CONNECT tunnel failed, response 502
```

Offline validation failed as a direct consequence (burn not in cache, and
`--offline` forbids fetching it).

## 4. Decision

Per the task-package stop condition ("online `cargo fetch --locked` failed →
stop, no loop retry"), **no further retry was run**. The user's instruction was
a single re-run, which has been executed and recorded.

Status remains **BR_FAIL_FETCH**. Phase B/C, B0, B1 and EXP-01-C stay frozen.
The EOS suspicion is neither proven nor disproven. No upstream source,
Cargo.toml, Cargo.lock, toolchain or feature was changed; no path-dependency
substitution or `cargo update` was used.

## 5. Recovery note

The cache is now partially populated (arkworks complete). If the network window
stays open longer on a future run, `cargo fetch --locked` resumes from burn and
the remaining dependencies. The recovery condition is unchanged: a stable
GitHub-accessible window (or a machine with reliable GitHub access performing
Phase A and migrating the verified bundle), with A3's `--locked` + `--offline`
double pass required before Phase B/C.
