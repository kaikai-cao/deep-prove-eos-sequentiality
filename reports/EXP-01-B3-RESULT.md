# EXP-01-B3 — EOS Witness Feasibility and Verifier Differential Test: Result

**Final status: `B3_NO_WITNESS_WITHIN_BUDGET`**

This is NOT a reproduction. No proof was generated, no dual-verifier comparison
ran, and no EOS conclusion was established. Per the task package, a bounded
search returning no witness is not a mathematical impossibility result.

Date: 2026-11-22. Branch: `exp-01-b3-eos-witness`.
Cloud run: https://github.com/kaikai-cao/deep-prove-eos-sequentiality/actions/runs/45026064200 (`conclusion=success`, all 13 steps green).

---

## 1. Fixed identity (verified, unchanged)

| item | value |
| ---- | ----- |
| B2 baseline | `e7a7df9560a4e3485d622a0a3971f713cdf87544` |
| B3 branch head at run | `2ba750123651f239f8de185ce3304e40cc9ee373` (harness compile fixes) |
| upstream | `Lagrange-Labs/deep-prove` @ `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` |
| Cargo.lock blob | `7b1b9da7bae322ccebcaff11981c2c4019c613f5` |
| toolchain | `nightly-2026-01-27` (rustc 1.95.0-nightly 474276961) |
| target | `zkml` / `bench-llm`, official CPU build, fresh CARGO_HOME, no cache, `CARGO_BUILD_JOBS=2` |

`main` handling: workflow file added to default branch in an isolated commit
`6d5579c` (parent = B2 freeze `e7a7df9`); no B2 report/evidence file was touched,
history not rewritten. The run itself checked out branch `exp-01-b3-eos-witness`.

## 2. What succeeded (independent environment result)

- **Cloud fresh build of `bench-llm` from the fixed commit PASSED** (9m18s),
  including `burn#c0717304` and all git dependencies. This removes the local
  `RECOVERY_FAIL` build blocker. This is an environment fact, NOT an EOS finding.
- **B3 harness compiled and ran** in the fixed `zkml` crate (injected as an extra
  bin into the ephemeral CI checkout; no upstream source file modified).
- **Fixed-revision model pinned and loaded.** HF `openai-community/gpt2` main tip
  resolved to `607a30d783dfa663caf39e06633721c8d4cfcd7e`; files downloaded from
  `resolve/<sha>/` and hashed:
  - `model.safetensors` `248dfc3911869ec493c76e65bf2fcf7f615828b0254c12b473182f0f81d3a707` (523M)
  - `tokenizer.json` `8414cab924d8b9b33013f0d221c5862f365ee9be39c5c2bfae8a5a9e970478a6`
  - `config.json` `0daed7749b4f02b8f76240d5444551d7b08712dab4d0adb8239c56ba823bb7b4`
- Quantized model built from these fixed files: 261 layers, `vocab_size=50257`,
  output node `Node(162)`. Search used DeepProve quantized path only
  (`load_from_model → into_provable_llm → Driver::run`), never HF float outputs.
- **Quantized-logits top-k extraction succeeded for every prompt** (0 failures in
  `b3-run.log`); each prompt produced a full 64-candidate set (top-16 logits +
  fixed-seed token ids + EOS, deduped, z≠a filtered at evaluation).

## 3. Search execution and budget accounting

| metric | value |
| ------ | ----- |
| wall clock (search) | 2702 s (~45 min) — hit `SEARCH_BUDGET_SECS=2700` |
| forward evaluations | 701 (cap was 3072; time cap hit first) |
| prompts processed | 11 of 48 (seeds 0–10) |
| avg time per candidate forward | ~3.85 s on 2 vCPU runner |
| witnesses found | 0 |

Per prompt: `a = Y[n-1]` from a single quantized forward over P; then each
candidate `z` evaluated by a single quantized forward over `X=P||z`, accepting
only if `z != a ∧ Y[n-1] == a ∧ Y[n] == EOS`. Observed `a` values:
866, 11, 11, 326, 286, 13, 1444, 11, 13, 284, 286.

The **binding constraint was wall-clock time, not the eval cap**: each full
GPT-2 quantized forward over a 32-token graph costs ~3.85 s on the 2-vCPU
runner, so 3072 evals would need ~3.3 h, beyond the 45-min search budget set in
the task package. Coverage was therefore partial (11/48 prompts, 701/3072 evals).

## 4. What did NOT happen (and why it is not a conclusion)

- **No candidate witness** ⇒ proof generation (Path M `run(P||z)→prove`) was
  **not entered**, and the **dual verifier comparison never ran**.
- Therefore none of the reproduction conditions were tested:
  `OriginalVerify=Accept` / `PatchedVerify=Reject` on the same proof/IO.
- This result does **not** show the B2 structural path is closed. It shows only
  that within 45 min / 701 quantized forwards / 11 short prompts / this fixed
  model revision, no `(P,z)` satisfied `z≠a ∧ Y[n-1]=a ∧ Y[n]=EOS`.

## 5. Exit-criteria mapping

| possible outcome | status this run |
| ---------------- | --------------- |
| cloud dependency/build failure | `B3_BLOCKED_CLOUD_BUILD` — NO (build passed) |
| bounded search, no witness | **`B3_NO_WITNESS_WITHIN_BUDGET` — YES (this run)** |
| witness found, proof failed | `B3_PROOF_GENERATION_FAILED` — not reached |
| original verifier rejects | `B3_ORIGINAL_REJECTS` — not reached |
| patched verifier also accepts | `B3_PATCHED_ALSO_ACCEPTS` — not reached |
| original accept + patched reject | `B3_REPRODUCED` — not reached |

Only `B3_REPRODUCED` counts as reproduction. This run is one of the five
non-reproduction outcomes.

## 6. Decision input (per task package §5)

Following the pre-agreed decision rule for "bounded search with no result":
combine search scope, model behavior and real resource use — decide freeze or
abandon; do not present a bounded search as a proof of impossibility.

Relevant facts for that decision:
- The blocker was **search throughput**, not a negative model result: 3072 evals
  were specified but only 701 were affordable; 37 of 48 seeds were never tried.
- The witness condition `Y[n]==EOS` at the second generated position is a
  comparatively rare event; a 64-candidate-per-prompt random+top-k sweep over 11
  short prompts is a small sample of the search space.
- The build environment is now proven working in the cloud, so any further
  experiment is resource-bound (more runner time / wider candidate set), not
  environment-bound.

This report does not itself choose freeze vs. abandon; it records the evidence
for that decision. No B4/B5 was started, and B0/B1/C remain frozen.

## 7. Evidence index

- `experiments/EXP-01-B3/SOURCE-IDENTITY.md` — fixed identity, verifier patch, budgets.
- `experiments/EXP-01-B3/prompt-seeds.txt` — 48 fixed seeds.
- `experiments/EXP-01-B3/harness/b3_witness.rs` — search/proof/differential harness (commit `2ba7501`).
- `experiments/EXP-01-B3/run1-preflight.md` — run 1 (build passed, harness compile failed then fixed).
- `experiments/EXP-01-B3/run-output/candidates.json` = `[]`; `candidates.csv` = header only.
- `experiments/EXP-01-B3/run-output/logs/b3-status.txt` = `B3_RUN_EXIT=0`, `B3_STATUS=B3_NO_WITNESS_WITHIN_BUDGET`.
- `experiments/EXP-01-B3/run-output/logs/b3-run.log` — full quantized search log (top-k extraction success, 11 prompts, budget stop).
- `experiments/EXP-01-B3/run-output/logs/SHA256SUMS.txt`, `REVISION.txt` — pinned model hashes/revision.
- `experiments/EXP-01-B3/run-output/logs/build-bench-llm.log`, `build-harness.log` — build evidence.
- Workflow: `.github/workflows/exp-01-b3-eos-witness.yml` (id `426808542`).
