# EXP-01-B1 — NOT EXECUTED

B1 was not executed because its prerequisite (B0: the pinned binary must
build and prove a real sequence) failed. See `../B0/result.md`.

Per the task package stop condition "fixed commit cannot be built", the
phase stopped here. No EOS sample was constructed; no prompt search was
run; no verifier output was collected. No panic or index anomaly was
observed and none is claimed.

## Prepared but not executed

- `../../scripts/eos_boundary_probe.rs` — temporary integration test that
  would call the real `Driver::run_elements → prove → verify` chain with a
  prompt tokenized by the real GPT-2 tokenizer (upstream unit-test pattern,
  `zkml/src/model/llm.rs:1006–1040`). Not compiled, not run.
- `search/search-plan.md` — prompt search procedure and honesty rules for
  the three EOS cases. Not executed.
- `case-eos-first/`, `case-eos-second/`, `case-no-eos/` — empty, reserved.

## Why no mock fallback

The task package forbids replacing the real proof chain. A simulated
verifier or hand-built IO would not answer the question "does the real
pinned verifier accept a non-autoregressive sequence", so producing one
would be misleading rather than useful.
