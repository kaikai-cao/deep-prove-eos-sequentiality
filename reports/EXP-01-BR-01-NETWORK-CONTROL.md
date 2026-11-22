# EXP-01-BR-01 — Network Control Experiment (proxy vs direct on burn#c0717304)

- Date: 2026-11-22 19:38–19:41 (Asia/Shanghai)
- Type: **network-layer diagnosis only**. Not A3, not cargo fetch --locked,
  not cache restoration, not a security experiment.
- Target: single commit `tracel-ai/burn@c071730459a135b4d06ad0e14c8a0e5771e5ba58`
- Stop rule: one pass per probe, no loop retry. Honored.

## 1. Environment

| Item | Value |
| --- | --- |
| Proxy env | `HTTP_PROXY/HTTPS_PROXY/http_proxy/https_proxy = http://127.0.0.1:55251` |
| `NO_PROXY` | `127.0.0.1,localhost,::1` |
| Git config | no proxy set at global / system / repo level |
| WinHTTP | direct (no proxy) |
| "Direct" probe definition | all `*_PROXY` env vars unset via `env -u` |
| git | `2.55.0.windows.3`; curl trace via `GIT_TRACE_CURL=1` |

## 2. Control matrix and results

| Probe | Proxy path | Direct path |
| --- | --- | --- |
| `git ls-remote ... HEAD` (small) | exit 128, 10 s, `CONNECT tunnel failed, 502` | 1st: exit 128, 21 s, `Failed to connect to github.com:443`; **2nd: SUCCESS**, HEAD `d05d7f49…` |
| shallow fetch of exact commit (large stream) | exit 128, 10 s, CONNECT 502, no TLS tunnel | GET `info/refs` **200**, POST `git-upload-pack` **200**, then `fatal: fetch-pack: invalid index-pack output`, timeout 180 s, local db only **5.7 MB**, commit object absent |
| `curl api.github.com` | HTTP 200 (~0.5 s) | — |
| direct `curl https://www.baidu.com` | — | HTTP 200 (direct internet works) |
| manual `CONNECT github.com:443` ×3 | HTTP 000 / 400 / 400 | — |
| raw.githubusercontent | HTTP 000 (curl 7) | — |
| codeload tarball HEAD | HTTP 200 (~1.4 s) | — |

Raw evidence: `experiments/EXP-01-BR-01-NETWORK-CONTROL/`
(`lsremote-{proxy,direct}.log`, `fetch-proxy.log`,
`fetch-direct-trace-summary.log`, `api-burn-commit.json`,
`api-burn-repo.json`, `env-snapshot.txt`). The full `GIT_TRACE_CURL` dump was
~11.5 MB / 101k lines (pack stream bytes); only the 89-line protocol skeleton
(TLS handshake, request/response headers, terminal error) was retained, and
the raw dump was deleted. No temporary bare repositories were committed.

## 3. Decisive findings

1. **The pinned commit exists — confirmed by GitHub API.**
   `GET /repos/tracel-ai/burn/git/commits/c0717304…` → HTTP 200 with exact SHA
   and message `ensure that tensor is owned on iter_dim call`. This finally
   closes the `BR_FAIL_REVISION` question: **the revision is real**. Earlier
   Cargo `revision … not found` was purely a downstream artifact of failed
   transfer (as the user judged throughout).

2. **The failure is in the pack-data stream, not in GitHub reachability, not
   in the proxy alone.** On the direct path, both HTTP requests of Git smart
   protocol succeeded (`info/refs` 200, `git-upload-pack` 200) — the tunnel,
   DNS, TLS and protocol negotiation all worked. The stream then died mid-
   transfer: `invalid index-pack output`, 180 s, only 5.7 MB landed. This is
   the **same failure signature as A3** (`early EOF`, `curl 28`, truncated
   pack) reproduced **without any proxy at all**. Therefore the hypothesis
   "it is only the proxy/node" is **rejected**: the proxy's 502 is one symptom
   of an unstable path to this large repository's pack stream, not the sole
   cause.

3. **Path availability is intermittent and path-dependent.**
   - Direct github.com:443: first attempt failed to connect (21 s), a later
     attempt succeeded for `ls-remote` — the link opens and closes.
   - Proxy CONNECT to github.com:443: repeated 502 / 400 / empty replies.
   - `api.github.com` via proxy: consistently 200. Raw host: failed.
   - Codeload tarball HEAD: 200 (an alternative byte path exists).
   So GitHub is not uniformly blocked; small/API paths work, the git
   smart-protocol pack stream is the fragile part.

4. **Direct internet is not blocked** (baidu 200), so "no network" is not the
   diagnosis. It is specifically the GitHub pack stream on this machine.

## 4. What this means for BR-01-RECOVERY

- Root cause is now localized: **large Git pack streams from GitHub are
  truncated on this machine regardless of proxy**, while API/small paths work.
- The cross-machine recovery kit (`6451a1f`) is still the recommended path:
  it sidesteps this machine's transfer problem entirely, and this experiment
  gives no reason to keep retrying A3 here.
- The codeload tarball path returns 200. Per the standing invariant this is
  **not** a substitute for exact-rev cache restoration (it cannot prove the
  pinned commit's cache identity), so it is recorded only as an observation,
  not adopted.

## 5. Status

| Item | Result |
| --- | --- |
| `BR_FAIL_FETCH` (A1–A3) | unchanged |
| `BR_FAIL_REVISION` | **eliminated** (API 200 at exact SHA) |
| Proxy/node as sole cause | **rejected** (direct path reproduces pack failure) |
| GitHub fully blocked | **rejected** (API 200, direct ls-remote succeeded once) |
| B0 / B1 / EXP-01-C | frozen |
| BR-01-RECOVERY | still recommended, NOT STARTED |
| EOS suspicion | neither proved nor disproved |

## 6. Invariants honored

No A3 re-run, no cargo cache change, no upstream / lock / toolchain change,
no tarball substitution adopted, no model run, no EOS work, no previous report
modified. Repository remains Private.
