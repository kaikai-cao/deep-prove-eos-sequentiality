# EXP-01-BR-01-RECOVERY — Final Result

- Date: 2026-11-22 19:52–20:08 (Asia/Shanghai)
- Final status: **`RECOVERY_FAIL`**
- This report is new; it does not modify any prior report.

## 1. Verdict

`RECOVERY_FAIL` — the online dependency fetch (A′2) did not complete on the
local machine, so the recovery chain stopped before packing, migration,
offline nightly fetch and `bench-llm` compilation.

No other status applies: this is not `RECOVERY_PASS`, not `RECOVERY_PARTIAL`
(no bundle or offline result was produced), and not `BLOCKED_NO_SECOND_HOST`
(the user explicitly authorized a final local attempt, which was executed).

## 2. Why the local attempt was run

The prepared task package (A′0) required a second host with stable GitHub Git
data transfer; absent one, it called for `BLOCKED_NO_SECOND_HOST`. No second
host was available. The user's closing instruction — "try once more on our
machine; if it fails we give up" — overrode A′0 for this one attempt. It was
executed with a **brand-new** Cargo Home (`recovery-remote-home`), not the
31 MB partial cache, so the result is equivalent to a fresh-home fetch. This
interpretation is recorded in `experiments/EXP-01-BR-01-RECOVERY/EXECUTION-METADATA.md`.

## 3. What happened

| Step | Result |
| --- | --- |
| Pre-check: source identity | `SOURCE_IDENTITY=OK` (HEAD `9d1a53e2…`, lock blob `7b1b9da7…`, clean) |
| Pre-check: nightly toolchain | `nightly-2026-01-27` present |
| Pre-check: fresh paths | `cargo-home-recovery`, `recovery-remote-home`, bundle path all ABSENT |
| A′2 online `cargo +stable fetch --locked` | **FAIL** — 900 s cap, exit 124 |
| A′3 / A′4 / B′ / C′ | not run (stop condition) |

A′2 log: `experiments/EXP-01-BR-01-RECOVERY/recovery-a2-online-fetch.log`
(SHA-256 in `SHA256SUMS-recovery.txt`).

Failure detail: `arkworks-algebra#76bb3a45` eventually succeeded after one
300 s timeout; `burn#c0717304` hit `fatal: fetch-pack: invalid index-pack output`
and never completed. Fresh git db ended at 24 MB; `burn` bare db 12 MB with no
target commit object and no refs — truncated transfer, not a bad revision.

This is the **fourth** failure at the same `burn` pack stream under different
network conditions (original A3, two retries, and this fresh-home attempt),
and matches the network control finding that pack-sized GitHub transfers are
truncated on this machine regardless of proxy (see
`reports/EXP-01-BR-01-NETWORK-CONTROL.md`; GitHub API confirms the commit exists
at HTTP 200).

## 4. Scope closure

Per the user's instruction ("if it fails we give up"), the local path is now
abandoned. No further A3 / fetch retries, no proxy tuning, no tarball
substitution, no A5/B/C, no B0/B1/C. The cross-machine recovery kit remains
in the repository but is not being executed in this round.

## 5. Status ledger

| Stage | Status |
| --- | --- |
| EXP-01-B | frozen |
| BR-01 original / A3 retry 1 / retry 2 / retry 3 | `BR_FAIL_FETCH` |
| BR-01 network control | complete — root cause localized to pack-stream truncation |
| BR-01-RECOVERY (this attempt) | **`RECOVERY_FAIL`** (local final attempt) |
| `bench-llm` compiled | **No** — Phase C′ never reached |
| B0 / B1 / EXP-01-C | frozen |
| EOS index suspicion | neither proved nor disproved |

## 6. Invariants honored

No upstream / `Cargo.toml` / `Cargo.lock` / toolchain / feature change; no
`cargo update`; no path-dependency or tarball substitution; no model run; no
EOS work; no prior report modified; the 31 MB partial cache and all prior
evidence untouched. Repository remains Private with no disclosure or paper
packaging. The user-added requirement for a brand-new, previously nonexistent
`CARGO_HOME` was satisfied (pre- and post-checks recorded).
