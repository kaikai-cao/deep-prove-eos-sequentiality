# EXP-01-B0 Result — B0_FAIL (environmental build blockage)

Date: 2026-11-22. Pinned upstream commit:
`9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`.

## Verdict

**B0_FAIL** — the pinned upstream `bench-llm` binary could not be built in
this environment because cargo cannot fetch the project's git dependencies
through the available HTTP(S) proxy. This is a **build/environment failure**,
not a security result. No proof, verifier output, or EOS behavior was
observed. Nothing in this file is evidence for or against a vulnerability.

## What succeeded

1. Experiment repository clean at phase start (`4ccd1c0`).
2. Upstream source at the pinned commit retrieved into an isolated worktree
   (`upstream/deep-prove/`, excluded from Git). Verified blob SHAs:
   - `zkml/src/model/llm.rs` = `ab987ac13e05914cb7ce45e6838dd586730e8b83`
   - `zkml/src/bin/bench/llm.rs` = `7726eee9677d5f440b8b8f00427cf3930bb74841`
   - `deep-prove/src/middleware/llm.rs` = `d1c6da39e5a9729ce25954fa03e571fe6a4bff6d`
3. Toolchain: pinned `nightly-2026-01-27` (`rustc 1.95.0-nightly
   (474276961 2026-01-26)`) installed via USTC mirror after the official
   download path proved too slow (~50 KB/s).
4. Exact CLI parameters and call chain established from source (see
   `../source-review/call-chain-evidence.md`) — no CLI guesswork.
5. Full EOS index semantics derivation completed
   (`../source-review/index-semantics.md`).

## What failed, and why

`cargo build --release -p zkml --bin bench-llm` repeatedly failed at the
dependency-resolution stage, before compiling any upstream source:

```
error: failed to load source for dependency `ark-bn254`
Unable to update https://github.com/a16z/arkworks-algebra?branch=dev/twist-shout#76bb3a45
fatal: unable to access 'https://github.com/a16z/arkworks-algebra/':
  CONNECT tunnel failed, response 502
```

The pinned workspace pulls ~9 git dependencies from `github.com`
(arkworks-algebra, dp-crypto, ec-gpu, timed-rs, tracel-ai burn/cubecl/cubek,
mem-track, metro). In this environment:
- `github.com` git access returned HTTP 502 / SSL EOF on every attempt
  (3 cargo-internal retries + 3 shell retries with backoff, ~30 minutes).
- `api.github.com` and `codeload.github.com` were reachable, but cargo
  git dependencies cannot be fetched through either; a tarball cannot
  reproduce the pinned git object SHAs, and patching every crate would
  change the "build the pinned commit unmodified" semantics.
- `git config url.insteadOf` to local bare repos is not viable for `rev=`
  dependencies (burn/cubecl/cubek), whose exact object SHAs cannot be
  recreated from tarballs.

This is the task package's stop condition "fixed commit cannot be built".
Per the task package, no model substitution, no mock, and no verifier
relaxation was used to work around it.

## Commands and exit codes (all recorded)

| Command | Result | Log |
| --- | --- | --- |
| `git clone --filter=blob:none ... deep-prove` + checkout | partial (LFS slow); source SHA-verified | shell output |
| `rustup toolchain install nightly-2026-01-27` (official) | aborted (too slow) | tool download progress |
| `rustup toolchain install nightly-2026-01-27` (USTC mirror) | success, 37 s | shell output |
| `cargo build --release -p zkml --bin bench-llm` | exit 101, 3 rounds | `build.log` |
| `cargo build ...` retries 1–3 (20 s backoff) | exit 101 each | `build.log` |
| `git ls-remote https://github.com/{a16z/arkworks-algebra,Lagrange-Labs/dp-crypto,tracel-ai/burn}.git HEAD` | all 502/EOF | shell output |

## Files

- `build.log` — complete build stdout/stderr, all retry markers.
- `environment.md` — toolchain/OS/RAM/disk/network at phase start.
- `../source-review/` — source facts and index derivation (independent of build).

## Not performed

- Model/tokenizer download and SHA-256 (blocked before HF access).
- `Driver::run_elements`, `Driver::prove`, verifier execution.
- Proof/IO generation and hashes.
- Any B1 EOS case.

## Recovery condition for a future round

Any of: (a) a proxy that allows `git clone/fetch` from `github.com` for the
nine pinned git dependencies; (b) a pre-populated cargo git database/cache
containing the exact locked SHAs; or (c) a prebuilt `bench-llm` binary from
this exact commit. Then rerun `scripts/run_b0_smoke.sh`.
