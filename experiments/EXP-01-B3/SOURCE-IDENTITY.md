# EXP-01-B3 — Fixed identity snapshot

Recorded before any CI trigger. All values verified read-only from the local
fixed checkout and the experiment repo on 2026-11-22.

## Experiment repo
- remote: `kaikai-cao/deep-prove-eos-sequentiality` (private)
- B2 baseline / branch base commit: `e7a7df9560a4e3485d622a0a3971f713cdf87544`
- B3 branch: `exp-01-b3-eos-witness` (created from the B2 commit)
- remote `main` ref verified via GitHub API = B2 commit (before B3 branch creation)

## Fixed upstream
- repo: `Lagrange-Labs/deep-prove`
- commit: `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`
- Cargo.lock blob: `7b1b9da7bae322ccebcaff11981c2c4019c613f5`
- local checkout: `D:/Research/deep-prove-cache-source` (unchanged, clean)
- toolchain: `nightly-2026-01-27` (rustc 1.95.0-nightly 474276961 2026-01-26)
- official CPU build: `cargo build --locked --release -p zkml --bin bench-llm`

## Key source blobs (from fixed commit)
- `zkml/src/model/llm.rs` = `ab987ac1…` (1250 lines) — `Driver::run` L340, `run_elements` L380, `prove` L774, `LLMVerifierContext::verify` L804–877
- `zkml/src/bin/bench/llm.rs` = `7726eee9…`
- `deep-prove/src/middleware/llm.rs` = `d1c6da39…`

## Verifier index fact (B2 → B3 patch target)
Fixed `llm.rs:837-847`:
```
eos_token_found = output.data().iter().take(max_window)
    .skip(user_input.len()-1)
    .find_position(|t| t.to_element() as usize == eos_token);
unpadded_output_len = pos.0            // skip-relative  <-- defect
```
`llm.rs:866` sequentiality loop: `for i in seq_len-1 .. unpadded_output_len-1`.

B3 patched verifier (`patched_verify` in harness) changes exactly one line:
```
unpadded_output_len = (user_input.len()-1) + pos.0 + 1   // absolute EOS index + 1
```
so that with `pos.0 == 1` (EOS at absolute position n) the loop covers `i = n-1`
and checks `X[n] == Y[n-1]` (i.e. `z == a`).

## Model (pinned at CI runtime, not by `resolve/main/`)
- model: `openai-community/gpt2`
- workflow resolves the HF `main` revision SHA via `huggingface.co/api/models` and
  downloads `model.safetensors`, `tokenizer.json`, `config.json` from
  `resolve/<sha>/…`; records each file's SHA-256 in `run-output/logs/SHA256SUMS.txt`.
- `RawSafeTensors::new(model, tokenizer, config)` loads these fixed files.
- Search uses DeepProve quantized path only: `load_from_model` → `into_provable_llm` → `Driver::run`.
- EOS token id = 50256 (GPT-2).

## Search budget (hard caps in harness)
- ≤ 48 prompt seeds; prompt token length 2–20; max context 32
- ≤ 64 candidates per prompt; ≤ 3072 candidate forward evals; ≤ 45 min wall clock
- candidates: quantized-logits top-k (k=16) + fixed-seed token ids + EOS, deduped
- witness predicate: `z != a` ∧ `Y[n-1] == a` ∧ `Y[n] == EOS`
- ≤ 3 proof attempts

## Exit criteria (B3_*)
`B3_BLOCKED_CLOUD_BUILD` / `B3_NO_WITNESS_WITHIN_BUDGET` / `B3_PROOF_GENERATION_FAILED`
/ `B3_ORIGINAL_REJECTS` / `B3_PATCHED_ALSO_ACCEPTS` / `B3_REPRODUCED`.
Only `B3_REPRODUCED` (original ACCEPT + patched REJECT via sequentiality, same
proof/IO) counts as reproduction. Environment/network failure is never a safety
conclusion.
