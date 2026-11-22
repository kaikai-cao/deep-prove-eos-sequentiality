# EXP-01 — EOS Sequentiality Bypass

Status: **NOT STARTED**. No reproduction claim.

Authoritative scope and verdict rule: `../../EXPERIMENT-PLAN.md`.
Source observation: `../../notes/source-inspection.md`.
Pinned revision: `../../upstream/UPSTREAM-REVISION.md`.

## Directory use

| Path | Contents |
| --- | --- |
| `scripts/` | Reproduction/analysis scripts; each script records pinned commit in its header |
| `patches/` | Minimal fix diffs only; one patch per hypothesis, never applied upstream |
| `artifacts/` | Logs, token sequences, proof/IO hashes, verifier output (large files stored outside Git) |

## Phase checklist

- [x] EXP-01-A: pin revision and verify API reachability — **done at source level**; see `../../notes/source-inspection.md`
- [ ] EXP-01-B: **B0_FAIL** (build blocked by git-dependency network); B1 not executed. See `../../reports/EXP-01-B-BOUNDARY-RESULTS.md`
- [ ] EXP-01-C: crafted non-autoregressive sequence accepted by original verifier (not started; blocked by B0)
- [ ] EXP-01-D: minimal EOS-index fix rejects the same proof and IO (not started)

## Naming convention

```
EXP01-B1-<timestamp>-<model>-<case>.log
EXP01-B2-<timestamp>-<model>-<case>.log
EXP01-B3-<timestamp>-<model>-<case>.log
EXP01-C-<timestamp>-<model>-tokens.json
EXP01-C-<timestamp>-<model>-verifier.log
EXP01-D-<timestamp>-<patch-id>-verifier.log
```

Every hash manifest must include: upstream commit, model/config, tokenizer,
prompt token IDs, `max_context`, proof hash, IO hash, verifier command,
verifier exit status.

## Isolation rules

- Apply candidate fixes only to an isolated worktree/copy of the pinned
  upstream revision; never to the upstream remote.
- Do not regenerate proof or IO between EXP-01-C and EXP-01-D.
- Do not claim reproduction from panic, index anomaly, or source algebra.
