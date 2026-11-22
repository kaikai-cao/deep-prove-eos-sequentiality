# EXP-01-B3 — Run 1 preflight record (run id 45024563431)

URL: https://github.com/kaikai-cao/deep-prove-eos-sequentiality/actions/runs/45024563431
Triggered: 2026-11-22T12:52:56Z via `workflow_dispatch` (ref=main, workflow id 426808542).
Conclusion: `failure`. Job `b3` steps 1–9 success, step 10 (Build B3 harness) failure,
steps 11–13 skipped/success as shown below.

## Step outcomes
| # | step | result |
| - | ---- | ------ |
| 1 | Set up job | success |
| 2 | Checkout experiment repo (B3 branch, 4cc1aa0) | success |
| 3 | Install fixed nightly toolchain | success |
| 4 | Prepare fresh CARGO_HOME (no cache) | success |
| 5 | Clone fixed upstream 9d1a53e2 + verify lock blob | success |
| 6 | Inject B3 harness bin into zkml/src/bin | success |
| 7 | Resolve pinned HF revision + download fixed files | success |
| 8 | Copy prompt seeds | success |
| 9 | **Official fresh build bench-llm** | **success** |
| 10 | Build B3 harness | **failure (4 compile errors)** |
| 11 | Run witness search + proof + dual verifier | skipped |
| 12 | Collect evidence | success |
| 13 | Upload run artifacts | success |

## Key independent facts established by run 1
- **Cloud fresh build of `bench-llm` from fixed upstream commit SUCCEEDED.**
  This removes the local `RECOVERY_FAIL` build blocker: the fixed revision's
  Git dependencies (including `burn#c0717304`) fetch and compile fine on the
  GitHub-hosted `ubuntu-24.04` runner with a fresh CARGO_HOME and the fixed
  Cargo.lock. This is an environment result, NOT an EOS finding.
- HF revision resolved to `607a30d783dfa663caf39e06633721c8d4cfcd7e`
  (`openai-community/gpt2` main tip at run time); fixed-revision
  model.safetensors / tokenizer.json / config.json downloaded successfully.
- No witness search or proof ran (harness did not compile), so run 1 yields
  NO EOS conclusion. It is a build-preflight success + harness-compile failure.

## Harness compile errors (all in b3_witness.rs) — FIXED in commit 2ba7501
1. E0433/E0432 `rand` crate not linked into the bin -> use `ark_std::rand`
   (v0.8.5) instead; matches fixed-source usage.
2. E0505 move out of `trace` while borrowed by `MappedRwLockReadGuard`
   -> scope the argmax extraction so the guard drops before returning trace.
3. E0382 borrow of moved `meta_json` -> clone at serialization.
4. warning unused import `ProverContext` -> removed.

Local machine cannot compile (local path = RECOVERY_FAIL on burn pack transfer),
so these were found via cloud run 1 and fixed statically, then re-dispatched as
run 2 (id 45026064200). Re-running entails a full fresh build again because the
workflow intentionally uses no cache.
