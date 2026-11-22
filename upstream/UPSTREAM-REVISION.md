# Pinned Upstream Revision

## Revision

| Field | Value |
| --- | --- |
| Repository | `https://github.com/Lagrange-Labs/deep-prove` |
| Commit (pinned) | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` |
| Commit metadata date | `2026-05-26T08:36:48Z` |
| Query time for HEAD | 2026-11-22 (Asia/Shanghai) |
| Visibility | public upstream; this experiment repository is private |

Pinning was done by reading `git ls-remote .../deep-prove.git HEAD
refs/heads/main`; both resolved to the commit above at query time. Re-check
before any new experiment round; if HEAD moved, keep EXP-01 on this commit.

## Retrieve the exact source

```bash
git clone https://github.com/Lagrange-Labs/deep-prove.git
cd deep-prove
git checkout 9d1a53e2ef49ffa2c902b8689cd3c58057a4e662
git rev-parse HEAD
```

Use an isolated worktree/copy. Never apply experiment patches to this
checkout's upstream remote and never commit experiment changes upstream.

## Relevant files and blob SHAs

| Path (at pinned commit) | Blob SHA | Lines | Role |
| --- | --- | --- | --- |
| `zkml/src/model/llm.rs` | `ab987ac13e05914cb7ce45e6838dd586730e8b83` | 1250 | `Driver`, `prove`, `LLMVerifierContext::verify` |
| `zkml/src/bin/bench/llm.rs` | `7726eee9677d5f440b8b8f00427cf3930bb74841` | 590 | End-to-end run/prove/verify entry point |
| `deep-prove/src/middleware/llm.rs` | `d1c6da39e5a9729ce25954fa03e571fe6a4bff6d` | 35 | `LlmProvable::verify` public wrapper |

To verify a blob: `git rev-parse HEAD:zkml/src/model/llm.rs` in the pinned
checkout.

## Source map (line numbers at pinned commit)

`zkml/src/model/llm.rs`:

| Symbol | Lines | Notes |
| --- | --- | --- |
| `LLMVerifierContext<F, PCS>` | 69–77 | fields: `verifier_ctx`, `md`, `max_context` |
| `LLMProof<F, PCS>` | 107–117 | proof plus full IO (prompt + generated tokens) |
| `Driver<N>` | 124–132 | model, metadata, `max_context`, padding mode |
| `Driver::run` / `run_elements*` | 340–~510 | `run_elements_with_runner` at 417 |
| Run-side EOS handling | 471–489 | stop flag set when a generated token equals `eos_token` |
| `Driver<Element>::prove` | 774–796 | delegates to `chunked_prove_local` (721–747) |
| `LLMVerifierContext::verify` | 804–877 | proof verification 849–852; input check 856–863; sequentiality loop 865–875 |
| Verifier EOS index computation | 835–847 | `find_position` after `take(max_window).skip(user_input.len()-1)`; `unpadded_output_len = pos.0` |

`zkml/src/bin/bench/llm.rs`: `run_elements` at 298, `prove` at 317,
`with_max_context(max_ctx).verify(proof, user_tokens, io)` at 339–343.

`deep-prove/src/middleware/llm.rs`: `LlmProvable::verify` at 32 calls
`self.ctx.verify(self.proof, self.user_tokens, self.io)`.

## Call chain under test

```
Driver::run_elements(input_tensor, store)            -> Trace
Driver::prove(&prover_ctx, trace)                    -> (Proof, IO)
LlmProvable::verify()  [deep-prove/src/middleware/llm.rs:32]
  -> LLMVerifierContext::verify(proof, user_tokens, io)
       -> zkml::verify(...)                          (proof itself)
       -> user-input prefix equality check           (lines 856–863)
       -> autoregressive sequentiality loop          (lines 865–875)
```

The v2 generic path `deep-prove/src/middleware/v2.rs::Provable::verify`
calls `verify(...)` directly and is not the path under test in EXP-01.

## Source observation (not a vulnerability claim)

The sequentiality loop's upper bound is derived from a `find_position` index
that is relative to the iterator after `skip(user_input.len()-1)`, while the
loop starts at absolute index `seq_len - 1`. The base conversion is not
performed in the pinned code. Whether this permits acceptance of a crafted
non-autoregressive sequence is an empirical question for EXP-01-C/D. See
`notes/source-inspection.md` for the exact excerpt and the planned minimal
fix candidate. No defect is asserted here.
