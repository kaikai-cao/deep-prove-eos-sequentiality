# Experiment Plan — EXP-01: EOS Sequentiality Bypass

## Goal

Determine whether a malicious prover can make DeepProve's LLM verifier accept
a generated token sequence that is not the output of genuine autoregressive
inference, by exploiting the EOS-boundary index handling.

**No vulnerability claim is permitted before successful reproduction.**

## Fixed baseline

- Upstream: `Lagrange-Labs/deep-prove`
- Pinned commit: `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`
- Relevant source: `zkml/src/model/llm.rs`, `zkml/src/bin/bench/llm.rs`,
  `deep-prove/src/middleware/llm.rs`
- Exact source locations and blob SHAs: `upstream/UPSTREAM-REVISION.md`

The pinned commit is checked out into an isolated worktree/copy. Upstream
history is never rewritten and no patch is applied to the upstream remote.

## Attack model

- Malicious prover: may choose any proof-shaped object, IO tensor contents,
  and token sequence that the honest verifier accepts as input.
- Honest verifier: uses the unmodified pinned verification code path.
- Verification path under test: `LlmProvable::verify()` in
  `deep-prove/src/middleware/llm.rs`, which calls
  `LLMVerifierContext::verify()` in `zkml/src/model/llm.rs`.
  The v2 generic `Provable::verify()` path does not perform the same
  autoregressive check and is **out of scope** for EXP-01.
- The prover/verifier must agree on model, tokenizer, `max_context`, and
  serialization. Any deviation is a confounder, not evidence.

## Evidence required for every phase

1. Upstream commit and source blob SHAs.
2. Model identifier, config (context length, eos token, vocab size),
   tokenizer, `max_context`, and prompt token IDs.
3. The proof and IO used (stored outside Git; hashes/manifest in Git).
4. Exact verifier stdout/stderr and exit status.
5. Reproduction command lines, toolchain version, and machine notes.
6. For the control phase: the minimal fix diff and the unchanged proof/IO
   hashes.

## Phase boundaries

### EXP-01-A — Pin upstream and establish the verification path

**In:** pinned upstream revision; source files listed above.

**Do:**
- Record commit, blob SHAs, and line numbers of `Driver::run_elements*`,
  `Driver::prove`, `LLMVerifierContext::verify`, `LlmProvable::verify`.
- Confirm the call chain `run_elements → prove → verify` is reachable from
  a public API/bench entry point (`zkml/src/bin/bench/llm.rs` lines
  ~298/317/339–343 and `LlmProvable::verify`).
- Record the exact EOS-related code in the verifier and the run side.

**Out:** no candidate tokens, no proofs, no fix, no conclusion about a bug.

**Exit criterion:** `notes/source-inspection.md` and
`upstream/UPSTREAM-REVISION.md` are complete enough for a third party to
reproduce the same source state; API reachability is explicitly recorded.

**Artifacts:** `notes/source-inspection.md`, `upstream/UPSTREAM-REVISION.md`.

### EXP-01-B — EOS boundary behavior

**In:** an available model/config and tokenizer at the pinned revision;
identical `max_context` for prover and verifier.

**Do:** run three separate honest cases and record verifier output:
- B1: EOS at the first generated position.
- B2: EOS at the second generated position.
- B3: no EOS within `max_context`.

Record the actual token IDs, output tensor shape, `eos_token_found` position,
`unpadded_output_len`, and whether verification succeeds. Keep proofs/IO even
for successful honest runs as inputs for phase C.

**Out:** no crafted sequences; no fix; no claim of bypass. A panic or an
index anomaly is recorded as an observation, not as a successful attack.

**Exit criterion:** all three cases have reproducible logs and the honest
behavior at the boundary is documented. If a model/config cannot be obtained
or a case cannot be executed, record the blocker; do not substitute a
simulated verifier for evidence.

**Artifacts:** `experiments/EXP-01/artifacts/EXP01-B1-*`, `B2-*`, `B3-*`.

### EXP-01-C — Crafted non-autoregressive sequence

**In:** phase B evidence; candidate token IDs satisfying the attack condition
identified from the pinned verifier code.

**Do:** construct a concrete token sequence whose post-EOS tokens do not
satisfy `prover_input[i+1] == prover_output[i]`; generate a real proof and IO
at the pinned revision; submit to the **original** verifier with no patch
applied.

Record: candidate selection derivation, prompt tokens, full generated
sequence, proof/IO hashes, verifier output, and exit status.

**Out:** no fix; no disclosure; no claim until phase D control is done.

**Exit criterion:** the original verifier **accepts** the crafted input with
a real proof. Rejection means the candidate condition is disproved for this
configuration; it does not disprove all configurations.

**Artifacts:** `experiments/EXP-01/artifacts/EXP01-C-*`.

### EXP-01-D — Minimal-fix control

**In:** the exact accepted proof and IO from phase C (hashes recorded);
a patch that changes only the EOS-index handling in
`LLMVerifierContext::verify()`.

**Do:** apply the minimal patch to an isolated copy/worktree of the pinned
revision; verify the stored proof and IO with the corrected verifier without
regenerating them. Record the diff, the hashes of proof/IO before and after,
and both verifier outputs.

**Out:** no other code changes; no re-proofing; no new model/config.

**Exit criterion:** the corrected verifier **rejects** the same proof and IO,
and the diff touches only the EOS-index handling. If the fix requires wider
changes, record INCONCLUSIVE rather than claiming a controlled reproduction.

**Artifacts:** `experiments/EXP-01/artifacts/EXP01-D-*`,
`experiments/EXP-01/patches/*`.

## Final verdict rule

- **REPRODUCED SECURITY DEFECT**: C accepts and D rejects with the same
  proof/IO and a minimal isolated fix.
- **NOT REPRODUCED**: any phase fails to produce the required acceptance or
  rejection evidence.
- **INCONCLUSIVE**: evidence exists but a confounder (model, context, fix
  scope, toolchain) prevents isolation.

No other verdict is allowed. "Found an index anomaly" and "observed a panic"
are not verdicts.

## Explicit out of scope

- Modifying or pushing to the upstream DeepProve repository.
- Mixing this experiment with `agent-exec-binding-mvp` or PA-VFANN.
- v2 generic verification path, other components, or model-weight issues.
- Public disclosure or paper framing before the verdict rule is satisfied.
