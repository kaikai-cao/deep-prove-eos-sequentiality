# EXP-01-B — EOS Boundary Validation Results

Date: 2026-11-22. Scope: B0 (end-to-end smoke) + B1 (EOS boundary cases).
Upstream: `Lagrange-Labs/deep-prove` @ `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`.

## Final status

**B0_FAIL** (build blocked by environment network) → B1 not executed.
This is a build/environment outcome, **not** a vulnerability finding. No
claim about EOS handling correctness or exploitability is made.

| Acceptance item | Status |
| --- | --- |
| B0 real model loaded | not reached |
| B0 real trace generated | not reached |
| B0 real proof generated | not reached |
| B0 original verifier verified | not reached |
| B1 EOS-first real sample or unreachability evidence | not reached |
| B1 EOS-second real sample or unreachability evidence | not reached |
| B1 no-EOS real sample or unreachability evidence | not reached |
| B1 full evidence saved | not reached (evidence directories reserved) |
| Upstream verifier logic unmodified | yes (no patch was ever applied) |

## 1. Fixed commit and source SHAs

Pinned commit `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` retrieved into
isolated worktree `upstream/deep-prove/` (Git-excluded). Verified blob SHAs:

| File | Blob SHA |
| --- | --- |
| `zkml/src/model/llm.rs` | `ab987ac13e05914cb7ce45e6838dd586730e8b83` |
| `zkml/src/bin/bench/llm.rs` | `7726eee9677d5f440b8b8f00427cf3930bb74841` |
| `deep-prove/src/middleware/llm.rs` | `d1c6da39e5a9729ce25954fa03e571fe6a4bff6d` |

Upstream was never modified; no PR; no push to upstream.

## 2. Environment

- rustc/cargo 1.98.1 (host), pinned `nightly-2026-01-27` installed via USTC
  mirror → `rustc 1.95.0-nightly (474276961 2026-01-26)`.
- Windows 11 Insider build 26320, Intel i7-14700 (20C/28T), 31.69 GB RAM
  (16.93 GB free at start), D: 164 GB available.
- Proxy `127.0.0.1:53706`; `github.com` git CONNECT repeatedly 502/EOF;
  `api.github.com` / `codeload.github.com` reachable; `static.rust-lang.org`
  ~50 KB/s; USTC mirrors 3.8 MB/s+.
- Full record: `experiments/EXP-01/artifacts/B0/environment.md`.

## 3. Actual call chain (source-established, not executed)

```
bench-llm main
  -> driver.random_sequence(prompt_len)        # random token prompt
  -> driver.tokens_to_tensor(user_tokens)
  -> driver.run_elements(input_tensor, store)  # generate + trace
  -> driver.prove(&prover_ctx, trace)          # (proof, io)
  -> verifier_ctx.with_max_context(ctx)
       .verify(proof, user_tokens, io)
  -> [path under test] LlmProvable::verify -> LLMVerifierContext::verify
```

v2 generic `Provable::verify` does not perform the autoregressive check and
was out of scope. Evidence: `experiments/EXP-01/artifacts/source-review/call-chain-evidence.md`.

## 4. EOS boundary construction plan (not executed)

- `--sequence 8` = prompt 6 + 2 generated tokens; it is only a smoke test,
  not an EOS test.
- B1 would have used a temporary integration test calling the same real
  driver/prover/verifier chain, with a legitimate text prompt tokenized by
  the real GPT-2 tokenizer (upstream's own unit-test pattern), because the
  bench CLI has no token/prompt input flag. Model, EOS ID and verifier
  untouched. Scripts: `experiments/EXP-01/scripts/eos_boundary_probe.rs`,
  plan: `experiments/EXP-01/artifacts/B1/search/search-plan.md`.
- No prompt search was run; no hit rate exists.

## 5. Three EOS cases — results

| Case | Construction | Proof / verifier result |
| --- | --- | --- |
| EOS at first generated position | not attempted | no data |
| EOS at second generated position | not attempted | no data |
| No EOS | not attempted | no data |

No exception, panic, or backtrace was observed because no code ran.

## 6. Key index derivation (source facts + math, NOT experiment)

Classification: [F] source fact, [D] derivation, [U] unconfirmed. See
`experiments/EXP-01/artifacts/source-review/index-semantics.md` for the
full table. Summary (prompt `seq_len`, verifier `zkml/src/model/llm.rs`):

- [F] `seq_len = user_input.len()` (prompt length); `prover_input` is
  `IO.input[0]` = full sentence (prompt + generated), `prover_output` is
  `IO.output[0]`.
- [F] verifier computes `eos_token_found` via `find_position` on
  `output.take(max_window).skip(seq_len-1)` and sets
  `unpadded_output_len = pos.0` (relative index).
- [F] sequentiality loop runs `for i in seq_len-1 .. unpadded_output_len-1`
  checking `prover_input[i+1] == prover_output[i]`.
- [D] The start bound is an absolute output index; the end bound is a
  relative index. With `seq_len=6`: EOS at absolute output index 5 →
  `pos.0=0` → `0usize-1` underflow; index 6 → `pos.0=1` → range `5..0`
  empty; no EOS → full range `5..7`. These are derivations, not observed
  behavior.
- [F] run side: on EOS generation the token is **not** pushed into
  `generated_tokens` (`llm.rs:471–489`); [D] therefore an honest trace may
  not contain EOS in `output.data()` at all — a semantic question this
  phase could not answer experimentally.
- [U] Whether any legitimate prompt can place EOS at the first/second
  generated position with GPT-2 (eos token 50256, vocab 50257) remains
  untested.

## 7. Agreement with earlier source observation

The pre-phase source observation (index base mismatch between
`find_position` and the sequentiality loop) is **unchanged and remains a
derivation only**. EXP-01-B did not confirm, refute, or exercise it; no
experimental evidence was added.

## 8. What could not be completed and why

B0 build failed at dependency resolution (cargo cannot fetch ~9 pinned
GitHub git dependencies; HTTP 502 / SSL EOF across all retries). Full log:
`experiments/EXP-01/artifacts/B0/build.log`. B1 is blocked by B0. Recovery
conditions are listed in `experiments/EXP-01/artifacts/B0/result.md`.

## 9. Conclusion and EXP-01-C start condition

**B0_FAIL; B1 not executed.** The experiment produced no verifier evidence.
It neither supports nor refutes the suspected EOS index issue.

EXP-01-C (crafted non-autoregressive sequence) must **not** start yet. Its
start condition remains: a successful B0 on the pinned commit (real model,
trace, proof, verifier all passing) and at least one B1 case with real
verifier output. Until then the suspicion stays exactly where the source
review left it: a well-defined, untested index-base question.
