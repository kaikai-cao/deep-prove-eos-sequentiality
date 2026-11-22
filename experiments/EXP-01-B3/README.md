# EXP-01-B3 — EOS Witness Feasibility and Verifier Differential

## Scope (single decisive question)
Can a real GPT-2 input witness be constructed in DeepProve's fixed-version
quantized inference path such that the fixed-version **original** verifier
accepts a proof generated via Path M (`Driver::run(P||z)` → `Driver::prove`)
while a **minimally patched** verifier rejects the *same* proof/IO?

Success requires BOTH, on byte-identical proof + IO:
`OriginalVerify(proof,IO)=Accept` AND `FixedVerify(proof,IO)=Reject`.

## Layout
- `SOURCE-IDENTITY.md` — fixed identity, verifier patch, model pin, budgets.
- `prompt-seeds.txt` — 48 fixed prompt seeds.
- `harness/b3_witness.rs` — copied into upstream `zkml/src/bin/` in ephemeral CI
  checkout only; does search + proof + dual verifier differential.
- `run-output/` — produced by CI: candidates.{json,csv}, proof-metadata.json,
  proof-N.bin, io-N.bin, logs/.
- `../../.github/workflows/exp-01-b3-eos-witness.yml` — manual-trigger workflow.

## Non-goals / hard limits
- No change to upstream source or Cargo.lock; no fake IO; no run_elements.
- No local Cargo build (local path is `RECOVERY_FAIL` on burn pack transfer).
- No HF float output as witness; search uses quantized path only.
- No automatic progression to B4; B0/B1/C remain frozen.
- Environment/network failure is never a safety conclusion.

## Exit states
`B3_BLOCKED_CLOUD_BUILD`, `B3_NO_WITNESS_WITHIN_BUDGET`,
`B3_PROOF_GENERATION_FAILED`, `B3_ORIGINAL_REJECTS`,
`B3_PATCHED_ALSO_ACCEPTS`, `B3_REPRODUCED`. Only `B3_REPRODUCED` counts.
