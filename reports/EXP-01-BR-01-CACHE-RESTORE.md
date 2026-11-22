# EXP-01-BR-01 — Fixed-Version Cargo Cache Restoration

- Date: 2026-11-22
- Final status: **BR_FAIL_FETCH**
- Scope: restore reproducible build capability only. **No B1, no EXP-01-C, no
  model run, no EOS prompt search.** This stage neither proves nor disproves
  the EOS suspicion.

## 1. Fixed baseline (verified)

| Item | Fixed value | Verified locally |
| --- | --- | --- |
| Upstream | `https://github.com/Lagrange-Labs/deep-prove.git` | — |
| Upstream commit | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` | yes (`HEAD` of isolated checkout) |
| `Cargo.lock` blob | `7b1b9da7bae322ccebcaff11981c2c4019c613f5` | yes (`git rev-parse HEAD:Cargo.lock`) |
| Rust toolchain | `nightly-2026-01-27` | installed, rustc 1.95.0-nightly |
| Target package / binary | `zkml` / `bench-llm` | — |
| Target build command | `cargo build --release -p zkml --bin bench-llm` | not reached |

The existing experiment checkout at `upstream/deep-prove` was **not** re-cloned
and **not** modified. A separate workspace `D:\Research\deep-prove-cache-source`
was created for Phase A.

## 2. Phase A — cache generation

### A0 / A1: workspace and version verification — PASS

Command sequence (Git Bash equivalent of the supplied PowerShell steps):

```bash
export GIT_LFS_SKIP_SMUDGE=1
git clone --filter=blob:none --no-checkout \
  https://github.com/Lagrange-Labs/deep-prove.git D:/Research/deep-prove-cache-source
git -C D:/Research/deep-prove-cache-source fetch --depth=1 origin 9d1a53e2...
git -C D:/Research/deep-prove-cache-source checkout --detach 9d1a53e2...
```

All three commands exited 0. At 18:33:58 `git ls-remote` succeeded for
`deep-prove`, `a16z/arkworks-algebra` and `Lagrange-Labs/dp-crypto`; the
GitHub network path was open at that moment.

Version check (`a1-version-check.log`):

```
HEAD=9d1a53e2ef49ffa2c902b8689cd3c58057a4e662
Cargo.lock_blob=7b1b9da7bae322ccebcaff11981c2c4019c613f5
git status --short: (empty)
A1_RESULT=MATCH_OK
```

This satisfies the task-package requirement: exact commit, exact lock blob,
clean worktree.

### A2: isolated Cargo Home — PASS

`CARGO_HOME=D:\Research\deep-prove-cargo-bundle\cargo-home`, separate from the
user's existing Cargo state. `cargo +stable --version` = `1.98.1`, satisfying
the `>= 1.85` requirement. No personal cache, credentials or unrelated
project files were copied into this home.

### A3: locked dependency fetch — FAIL (stop condition 2)

First online attempt:

```
cargo +stable fetch --locked
FETCH_ONLINE_EXIT=101
```

Error chain (full log in `fetch-online.log`):

```
warning: spurious network error: [7] Could not connect to server
         (CONNECT tunnel failed, response 502)
error: failed to load source for dependency `ark-bn254`
Caused by: unable to update
  https://github.com/a16z/arkworks-algebra?branch=dev%2Ftwist-shout#76bb3a45
Caused by: revision 76bb3a4518928f1ff7f15875f940d614bb9845e6 not found
Caused by: network failure seems to have happened
Caused by: [35] SSL connect error (schannel: failed to receive handshake ...)
```

Offline validation consequently failed:

```
cargo +stable fetch --locked --offline
FETCH_OFFLINE_EXIT=101
... can't checkout from 'https://github.com/a16z/arkworks-algebra':
    you are in the offline mode (--offline)
```

Diagnosis: this is a **network backend failure**, not a bad revision, not a
missing object and not a lock-file inconsistency. The pinned commit object is
known to exist (the user independently verified all nine via GitHub API), and
A0 fetched from the same host minutes earlier.

Cargo's own error message recommends `net.git-fetch-with-cli`. That official
setting was added **only** to the isolated Cargo Home (`config.toml`), which
changes the network backend, not the source, lock file, dependency versions or
features. After applying it, three finite git-CLI probes were run:

```
18:41:50  git ls-remote https://github.com/a16z/arkworks-algebra.git HEAD
          -> CONNECT tunnel failed, response 502
18:42:06  -> CONNECT tunnel failed, response 502
18:42:21  -> CONNECT tunnel failed, response 502
```

The connectivity window observed at 18:33 had closed. Per the task-package
stop condition "online `cargo fetch --locked` failed → stop, no loop retry",
no further retry was attempted.

### A4: Git source lock list — PASS

Extracted from the fixed `Cargo.lock` (not from a re-resolution) using
`grep '^source = "git+' | strip prefix/suffix | sort -u`. Result: **9 unique
sources**, present in `git-source-locks.txt`. All nine locked commits from the
task package are present and match:

| Repository | Locked commit |
| --- | --- |
| `a16z/arkworks-algebra` | `76bb3a4518928f1ff7f15875f940d614bb9845e6` |
| `tracel-ai/burn` | `c071730459a135b4d06ad0e14c8a0e5771e5ba58` |
| `tracel-ai/cubecl` | `7cd8e9c869d363edcbf989fd905eb4b28df938ab` |
| `tracel-ai/cubek` | `8097a621dfd3a6a89f8d0433994e8c1adba377c2` |
| `Lagrange-Labs/dp-crypto` | `22e8e93cd0f94a7638616a1ae22190feb0a6b275` |
| `Lagrange-Labs/ec-gpu` | `2606f2c5eb9c3068b36778e5fdab2b1f374a6cdc` |
| `hackaugusto/mem-track` | `7586dd0d6cb8b6c7b3aee58a2acf4060933fb819` |
| `delehef/metro` | `77370daf7fbd5ff92af98b3835c273c2122b8577` |
| `nikkolasg/timed-rs` | `6a2524f8532e5a9c31740df20c4875754a5dc2cf` |

### A5: archive — NOT EXECUTED

No dependency bundle exists to archive, because A3 failed. No `cargo-cache.zip`
and no SHA-256 for such an archive were produced.

## 3. Phase B / C — not executed

- Phase B (migrate cache, verify SHA-256, offline fetch with fixed nightly):
  not executed; blocked by A3. No cache bundle was generated.
- Phase C (`cargo +nightly-2026-01-27 build --locked --offline --release -p
  zkml --bin bench-llm`): not executed; blocked by A3. No `bench-llm`
  compiled in this stage, no build log and no exit code exist for it.

Consequently there is no evidence here about whether the fixed commit compiles
or whether `bench-llm` is runnable. **This is not a code-build failure.**

## 4. Verdict and accounting

Final status: **BR_FAIL_FETCH** — online cache generation failed at the
`cargo fetch --locked` step.

| Item | Status |
| --- | --- |
| Fixed commit / lock blob verification | PASS |
| Isolated Cargo Home | PASS |
| 9 pinned Git sources from fixed lock file | PASS |
| Online `cargo fetch --locked` | FAIL (proxy 502 / SSL) |
| Offline fetch validation | FAIL (consequence of above) |
| Cache archive / migration | NOT EXECUTED |
| `bench-llm` build | NOT EXECUTED |
| B0 / B1 / EXP-01-C | NOT STARTED |

The failure is an external network condition. It is **not** a dependency
version problem, **not** a fixed-commit code defect, and **not** evidence for
or against the EOS index suspicion. The distinction between environment
blockage and security results remains in force (see
`EXP-01-B-FINALIZATION.md`).

## 5. Invariants honored

No upstream source modification, no `cargo update`, no `Cargo.lock` change, no
path-dependency substitution, no toolchain or default-feature change, no PR,
no model execution, no EOS prompt search, no B1, no EXP-01-C. The experiment
repository remains Private with no disclosure or paper packaging.

## 6. Recovery condition

This stage can be resumed when one of the following holds: a stable
GitHub-accessible window on this machine (the A0 clone succeeded at 18:34–18:39,
so the path is proven available intermittently), or a separate machine with
reliable GitHub access performing Phase A and transferring the verified cache
bundle. On resumption, A3 must succeed with `--locked` and `--offline`
validation before Phase B and C are attempted. The stop condition is unchanged:
no retry loop, no dependency rewriting.

## 7. Evidence

All per-phase logs and machine-readable status are in
`experiments/EXP-01-BR-01/`. The three previously committed reports
(`EOS-SEQUENTIALITY-BYPASS-EXP01.md`, `EXP-01-B-BOUNDARY-RESULTS.md`,
`EXP-01-B-FINALIZATION.md`) are unchanged. No cache archive was produced and
nothing was uploaded to a public repository.
