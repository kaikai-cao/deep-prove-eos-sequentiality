# EXP-01-BR-01 — A3 Third Retry (proxy node switched)

- Date: 2026-11-22 19:34–19:35 (Asia/Shanghai)
- Trigger: user switched proxy node and asked for one more A3 run.
- Result: **still BR_FAIL_FETCH**; online and offline fetch both exit 101.
- This file supplements (does not modify) all prior BR-01 reports.

## 1. Preconditions re-verified (unchanged)

| Item | Value |
| --- | --- |
| Source HEAD | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` |
| `Cargo.lock` blob | `7b1b9da7bae322ccebcaff11981c2c4019c613f5` |
| Worktree | clean |
| stable cargo | `1.98.1`; `[net] git-fetch-with-cli = true` |
| git db | 31 MB before and after (arkworks complete, burn partial) |

## 2. Commands and results

```bash
cargo +stable fetch --locked            # FETCH_ONLINE_EXIT=101  (56 s)
cargo +stable fetch --locked --offline  # FETCH_OFFLINE_EXIT=101 (0.35 s)
```

Raw logs: `experiments/EXP-01-BR-01/a3-retry3-{online,offline}-fetch.log`;
SHA-256 in `SHA256SUMS-retry3.txt`.

## 3. Failure detail — mixed symptoms on the new node

```
fatal: unable to access '.../burn.git': OpenSSL SSL_read: unexpected eof      (attempt 1)
fatal: unable to access '.../burn.git': CONNECT tunnel failed, response 502  (attempts 2,3)
fatal: unable to access '.../burn.git': Empty reply from server              (attempt 4)
error: revision c071730459a135b4d06ad0e14c8a0e5771e5ba58 not found
```

The failure point is unchanged (`burn#c0717304`). Unlike retry 2 (pure
low-throughput / TLS EOF), the new node shows a **mix** of TLS EOF, proxy 502
and empty replies — the proxy-layer `502` reappeared. Switching the node did
not produce a stable transfer window for the large `burn` fetch-pack.

## 4. Object-level check (read-only)

```
burn db: 19 MB; cat-file -t c0717304… -> could not get object info; for-each-ref -> 0
git db total: 31 MB
```

Same as retry 2: partial data, no imported target commit, no refs — consistent
with a truncated fetch-pack. **Not evidence that the pinned commit is absent.**

## 5. What this run does and does not establish

| Question | Answer from this run |
| --- | --- |
| Did switching nodes fix `burn` transfer? | **No** — still exit 101 at the same commit |
| Is the proxy the sole cause? | **Not established** — symptoms changed but the failure persisted; cannot isolate a single node-level cause |
| Is the pinned revision bad / missing? | **No evidence** — object check shows truncated transfer, not absence |
| Fixed-commit code build failure? | **No** — Phase C never reached |
| EOS suspicion proved or disproved? | **Neither** |

This is an empirical check of the "proxy/node" hypothesis raised in discussion:
changing the node did **not** restore the cache fetch. It does not prove the
proxy is irrelevant, but it removes "just switch to a better node" as a
demonstrated fix on this machine.

## 6. Verdict and stop condition

Status remains **BR_FAIL_FETCH**. Per the stop condition, no further retry was
run — this was a single user-authorized run after the node change. Phase A5 /
B / C, B0, B1 and EXP-01-C stay frozen.

`experiments/EXP-01-BR-01-RECOVERY/` (cross-machine recovery kit, commit
`6451a1f`) remains the recommended path and stays **NOT STARTED**. Three
attempts on this machine have now failed at the same `burn` transfer under
different network conditions; repeated same-path retries are not a recovery
strategy.

## 7. Invariants honored

No upstream / `Cargo.toml` / `Cargo.lock` / toolchain / feature change, no
`cargo update`, no path-dependency substitution, no PR, no model run, no EOS
prompt search, no modification of previous reports. Repository remains Private.
