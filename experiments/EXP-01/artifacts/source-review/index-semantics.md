# Source Review — EXP-01-B Index Semantics

Pinned upstream: `Lagrange-Labs/deep-prove` @
`9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`.

Classification of each statement: **[F]** = direct source fact; **[D]** =
mathematical derivation from source facts; **[E]** = experimental result
(filled only after execution); **[U]** = unconfirmed assumption.

## 1. CLI and control knobs

**[F]** `zkml/src/bin/bench/llm.rs` `LLMArgs`:

- `weights` group requires exactly one of `--gguf` / `--hf`.
- `length` group requires one of `--sequence` / `--max-context`.
- `--sequence n1,n2,...`: for each `n`, prompt length = `n-2`, generation
  length = 2, `max_context = max(n_i)` (lines 59–64, 78–81, 179–191).
- `--max-context M` together with `--min-user-len L` (default 1): prompt
  length = `L`, generation up to `M` (lines 74–90, 179–181).
- `--hf <model-id>` auto-downloads SafeTensors weights + tokenizer; CPU
  build command from `zkml/README.md:91`: `cargo build --release -p zkml
  --bin bench-llm`.
- Bench uses `driver.random_sequence(user_prompt)` (line 273), i.e. random
  token IDs in `[0, vocab_size)`, **not a real text prompt**. Randomness is
  from `rng_from_env_or_random()` (`zkml/src/model/llm.rs:296–304`).
- `zkml/src/lib.rs:278–288`: `rng_from_env_or_random()` reads `RNG_SEED`
  (u64) if set, else random. Setting `RNG_SEED` makes prompt token IDs
  deterministic — a legitimate control that changes neither model weights,
  EOS token ID, nor verifier. This is the mechanism for reproducible B1 search.

**[D]** CLI-level `--sequence 8` = prompt 6 tokens + 2 generated tokens;
it does not put EOS at a specified position. `--max-context M --min-user-len
L` gives a generation window of `M-L` positions, which is the control needed
for B1.

## 2. Trace / IO structure

**[F]** `zkml/src/model/trace.rs`: `Trace<D>` holds `steps: HashMap<NodeId,
Step<D>>`, `input: Vec<TensorHandle<D>>`, `output: Vec<TensorHandle<D>>`.

**[F]** `zkml/src/iop/verifier.rs:68–75`: `IO<F>` has public `input:
Vec<Tensor<SerializableField<F>>>` and `output: Vec<Tensor<
SerializableField<F>>>`, i.e. one input tensor and one output tensor in the
LLM case; `input[0]` is the full sentence (prompt + generated tokens),
`output[0]` is the model output at each position.

**[F]** `zkml/src/model/llm.rs:115–116` comment: IO contains the _full_
input, "user input + the generated tokens".

**[F]** `zkml/src/model/llm.rs:573–581, 597–640`: during `run_elements`, the
code asserts for every `i in num_input_tokens..full_sentence.len()`:
`trace.outputs()[0].tensor().data()[i-1] == trace.inputs()[0].tensor().data()[i]`.
Hence `output.data()[i-1]` corresponds to the token at absolute position
`i` in `full_sentence`, for `i >= num_input_tokens` (the generated region).

**[D]** Therefore `output.data()[j]` (0-based) corresponds to `full_sentence`
position `j+1`. For the generated region with prompt length `seq_len`:
`output[seq_len-1]` predicts the first generated token at `full_sentence[seq_len]`.

## 3. `user_input.len()` / `seq_len` / `prover_input`

**[F]** `LLMVerifierContext::verify(proof, user_input, inference_output)` at
`zkml/src/model/llm.rs:804–809`. In bench, `user_tokens` (the prompt token
IDs) are passed as `user_input` (line 342).

**[F]** verifier line 856: `let seq_len = user_input.len();` — the prompt
length in tokens.

**[F]** verifier lines 856–863 check `prover_input.data()[..seq_len]` equals
`user_input[..seq_len]` elementwise; any crafted IO must preserve the prompt
prefix.

**[F]** verifier lines 849–852 verify the proof itself via `zkml::verify`
before the sequentiality check.

**[D]** `prover_input` is `IO.input[0]` (full sentence including generated
tokens), `prover_output` is `IO.output[0]`, and `output` (line 826) is a
clone of `prover_output`. `seq_len` counts prompt tokens only.

## 4. EOS find_position and index mapping

**[F]** verifier lines 835–847:

```
take(max_window).skip(user_input.len()-1).find_position(eos)
unpadded_output_len = pos.0 if found else max_window
```

**[D]** `find_position` returns the index relative to the **skipped**
iterator, whose first element is `output.data()[seq_len-1]`. If EOS appears
at absolute output index `a >= seq_len-1`, then `pos.0 = a - (seq_len-1)`.
Example: EOS at absolute output index `seq_len-1` (first generated token)
gives `pos.0 = 0`; EOS at absolute `seq_len` gives `pos.0 = 1`.

**[F]** verifier line 865–875:

```
for i in seq_len-1 .. unpadded_output_len-1 {
    ensure!(prover_input.data()[i+1] == prover_output.data()[i], ...)
}
```

**[D]** The loop start `seq_len-1` is an **absolute** output index, but the
end `unpadded_output_len-1 = pos.0-1` uses the **relative** position. For
cases where `pos.0 < seq_len`, the range is empty or underflows, so the
sequentiality equations are not enforced. Worked example (prompt `seq_len=6`,
max_context=8; `output.data()[j]` predicts `full_sentence[j+1]`):

| EOS at absolute output index | `pos.0` (relative) | `unpadded_output_len` | loop `seq_len-1 .. unpadded-1` |
| --- | --- | --- | --- |
| none | n/a | `max_window` (8) | `5..7` — full check |
| 5 (= first generated prediction) | 0 | 0 | `5..(0-1)` — usize underflow |
| 6 (= second generated prediction) | 1 | 1 | `5..0` — empty |
| 7 | 2 | 2 | `5..1` — empty |

Specific boundary expectations:
- EOS at first generated position → `pos.0 = 0` → `unpadded_output_len-1`
  underflows (`0usize-1`); debug panics, release wraps to `usize::MAX`
  leading to out-of-bounds on later iterations. **Experimental observation
  required; do not treat as proven.**
- EOS at second generated position → `pos.0 = 1` → range `seq_len-1..0`
  empty → no sequentiality check.
- No EOS → `unpadded_output_len = max_window` → full range checked.

**[U]** Whether the honest `run_elements` ever places EOS at a *generated*
output position at all — see next section. This determines whether any of
the above cases is reachable via honest execution.

## 5. Run-side EOS handling

**[F]** `run_elements_with_runner` lines 471–489: a background thread
receives each generated token; if `token == eos_token`, it sets `stop=true`
and returns **without pushing the EOS token into `generated_tokens`**.

**[F]** lines 495–502: the inference loop checks `stop` at the top of each
iteration and breaks.

**[F]** lines 530–543: after each inference, the last token before padding
is selected and sent through the channel, then used as the next input.

**[F]** lines 546–556: after the loop, `full_sentence.extend(generated_tokens)`,
then a final heavy trace is run on `full_sentence` (lines 563–572).

**[D]** On honest EOS generation, the EOS token is **not** appended to
`full_sentence`; the trace is executed on the prompt plus the tokens
generated before EOS. Thus the honest trace's `output.data()` may not
contain the EOS token at all, which would make the verifier's
`find_position(eos)` return `None` even in a run that stopped on EOS. This
is a semantic question that EXP-01-B must answer empirically.

**[D]** If EOS appears at the first generated position, `generated_tokens`
is empty, `full_sentence == input_data`, and the generated region is empty;
the verifier sees `user_input.len() == full_sentence.len()` and EOS is not
present in `output.data()` unless the prompt itself contained an EOS token.

## 6. Verifier constraints beyond sequentiality

**[F]** verifier lines 820–834: requires `output.len() == 1`, and
`padded_max_len <= max_window.next_power_of_two()`.

**[F]** verifier lines 849–852: full proof verification (`zkml::verify`).

**[F]** verifier lines 856–863: user-input prefix equality.

**[F]** verifier lines 865–875: sequentiality loop (only when the range is
non-empty).

**[D]** A crafted non-autoregressive IO must pass (a) proof verification,
(b) prompt-prefix check, and (c) padded length bound. Removing (d)
sequentiality alone is not sufficient to claim an accepted forgery; EXP-01-C
must demonstrate all constraints simultaneously.

## 7. Source-review provenance

Facts above were read from the pinned commit via `api.github.com/contents`
raw responses (network fallback because direct `github.com` CONNECT was
intermittently 502). Blob SHAs are recorded in
`../../../upstream/UPSTREAM-REVISION.md`. Local checkout status is recorded
in the B0/B1 environment logs. No claim of a defect is made in this file.
