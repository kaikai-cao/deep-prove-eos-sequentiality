# Source Review — EXP-01-B Call Chain Evidence

Pinned upstream: `Lagrange-Labs/deep-prove` @ `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`.
All facts below were read from `api.github.com/contents` raw responses at
that commit; blob SHAs verified against the isolated checkout.

## 1. Bench CLI exact flags (from source, not guessed)

`zkml/src/bin/bench/llm.rs` `LLMArgs` (lines 59–142):

```
required group "weights":   --gguf <path|url> | --hf <model-id>
required group "length":    --sequence <n1,n2,...> | --max-context <M>
--sequence n:               prompt length = n-2, generate 2 tokens
--max-context M:            with --min-user-len L (default 1): prompt L, generate to M
--model <gpt2|gemma3|llama2>
--num-threads N             (0 = physical cores)
--save-paramS / --load-params <basename>   cache pk/vk
--accuracy, --distributed, --num-chunks, --memory, --measure-layerwise
```

README (`zkml/README.md:91,132-133`) confirms:
`cargo build --release -p zkml --bin bench-llm` and
`cargo run --release --bin bench-llm -- --model gpt2 --hf openai-community/gpt2 --sequence 64`.

## 2. Bench call chain (pinned source lines)

```
main (153)
  -> LLMArgs::parse() (161)
  -> (max_context, sequence) computed (179-191)
  -> parse_model(...) -> Driver<f32> + tokenizer (203)
  -> driver.context / compute_all_contexts (227-252)
  -> for each (user_prompt, max_ctx) in sequence (265):
       driver.with_max_context(max_ctx)            (272)
       user_tokens = driver.random_sequence(len)   (273)
       input_tensor = driver.tokens_to_tensor(...) (274)
       trace = driver.run_elements(input_tensor, store)  (298)
       (proof, io) = driver.prove(&prover_ctx, trace)    (317)
       verifier_ctx = verifier_ctx.with_max_context(max_ctx) (339)
       verifier_ctx.verify(proof, user_tokens, io)       (341-343)
```

## 3. v1 verifier wrapper

`deep-prove/src/middleware/llm.rs:25-34`: `LlmProvable::verify()` calls
`self.ctx.verify(self.proof, self.user_tokens, self.io)` — the same
`LLMVerifierContext::verify` exercised by the bench path.

v2 `Provable::verify()` (`deep-prove/src/middleware/v2.rs:21`) calls generic
`zkml::verify(...)` only, with no EOS/sequentiality check — explicitly out
of scope for EXP-01.

## 4. Random prompt and reproducibility

`Driver::random_sequence(len)` (`zkml/src/model/llm.rs:296-304`) samples each
token uniformly from `[0, vocab_size)`. `zkml/src/lib.rs:278-288`:
`rng_from_env_or_random()` reads `RNG_SEED` (u64) if set. Therefore
`RNG_SEED=<u64>` makes the bench prompt reproducible. This does not alter
the model, EOS ID, prover, or verifier.

## 5. Consequence for B1 construction

The bench CLI has **no flag** to supply a chosen token sequence or text
prompt; it always uses `random_sequence`. To test EOS at a specified
generated position with a legitimate prompt, EXP-01-B1 uses a temporary
Rust integration test that calls the same real `Driver::run_elements` →
`Driver::prove` → `LLMVerifierContext::verify` chain, but constructs the
prompt with the real GPT-2 tokenizer (`GPT2::new().load_tokenizer(&raw)`
and `tokenizer.tokenize("...")`), following the upstream unit-test pattern
in `zkml/src/model/llm.rs:1006-1040`. This is a control of the **input**,
not of the model or verifier. Every action is logged; the temporary test
file is copied into the isolated upstream worktree and removed afterward.
