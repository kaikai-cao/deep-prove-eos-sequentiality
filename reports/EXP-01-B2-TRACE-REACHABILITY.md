# EXP-01-B2 — EOS Trace Reachability Analysis

- Date: 2026-11-22 (Asia/Shanghai)
- Upstream: `Lagrange-Labs/deep-prove` @ `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`
- Method: **source-only** data-flow / reachability analysis. No Cargo, no build, no
  model inference, no GPT-2 load, no witness search, no proof/IO fabrication, no
  upstream modification, no EXP-01-C.
- This report is an **addition**; it does not modify any prior report
  (EXP-01-B, EXP-01-BR-01 series, EXP-01-B-FINALIZATION).

## Final verdict

**`B2_STRUCTURAL_PATH_CONFIRMED_WITNESS_UNKNOWN`**

A structurally reachable proof path exists in the fixed source that does **not**
go through `run_elements`, combined with a base-mismatch in
`LLMVerifierContext::verify` that skips the exact token transition the path can
violate. But the model-dependent witness `(P, z)`, a real verifier acceptance, and
the deployment exposure surface are all **unverified**. This is **not** a claim that
a vulnerability is reproduced. Per the task's hard rule, confirming B2-2 and B2-3
does not by itself establish a security flaw.

### Acceptance criteria (four questions)

| ID | Question | Verdict |
| --- | --- | --- |
| B2-1 | Can the final Trace output of the normal `run_elements` path contain EOS? | **Conditional YES** — structurally expected as the last output row; no explicit source assertion, not measured. |
| B2-2 | Does `Driver::run` allow a provable Trace directly over `P‖z`? | **YES (source-confirmed API capability)** — deployment exposure unknown. |
| B2-3 | With EOS at the second generated position, does the current verifier skip that transition check? | **YES (source-confirmed)** — empty `Range` under the index base mismatch. |
| B2-4 | Can current source show no other constraint fills that transition? | **YES, within the defined audit scope** — all seven listed constraint classes checked; none forces `X[n] = Y[n-1]`. |

---

## 1. Fixed source identity and blob SHA

Recorded at task start; full snapshot in `experiments/EXP-01-B2/SOURCE-IDENTITY.md`.

| Item | Value |
| --- | --- |
| Experiment HEAD | `8d51d6850d70a75222a140a26e37e8bcea645891` (clean) |
| Upstream HEAD | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` (clean) |
| `Cargo.lock` blob | `7b1b9da7bae322ccebcaff11981c2c4019c613f5` |
| `zkml/src/model/llm.rs` | `ab987ac13e05914cb7ce45e6838dd586730e8b83` |
| `zkml/src/model/trace.rs` | `475dc30b0d67a156c40c523a8273e30844ffdc00` |
| `zkml/src/model/mod.rs` | `3e7f87cacdfc8b6ece550ef2faa04a789eb08bd1` |
| `zkml/src/model/exec_graph.rs` | `501527d6f64d3724ce6c7d1a51fed1a119fa4e23` |
| `zkml/src/parser/llm/mod.rs` | `798bfb05bfc1d375ff40c352dd16ffd5047e839d` |
| `zkml/src/parser/llm/metadata.rs` | `b80671a5f6fdfb3b8007851be3e4eba6c827518a` |
| `zkml/src/layers/transformer/logits.rs` | `c3e931ace046a491ffbb91f463801c4aa51f26f1` |
| `zkml/src/iop/verifier.rs` | `863d2c277cab227f711eb4b15274577acb76c693` |
| `zkml/src/bin/bench/llm.rs` | `7726eee9677d5f440b8b8f00427cf3930bb74841` |
| `deep-prove/src/bin/worker/immediate.rs` | `4fd9d167da56b8b1752481049535193d9e427e4b` |
| `deep-prove/src/middleware/llm.rs` | `d1c6da39e5a9729ce25954fa03e571fe6a4bff6d` |

Identity check: PASS. All line numbers below refer to these blobs on the fixed commit.

---

## 2. Two execution paths — call graphs

### Path H — normal autoregressive generation (`run_elements`)

```
P  (Tensor)
 └─ Driver::run_elements                         llm.rs:380-386
     └─ run_elements_with_split_info             llm.rs:360-371
         └─ run_elements_with_runner             llm.rs:417-657
             ├─ model.reset()                    llm.rs:429
             ├─ full_sentence = input_data       llm.rs:459 ( = P )
             ├─ spawn results thread             llm.rs:472-491
             │    ├─ token = tensor[0]           llm.rs:477
             │    ├─ if token == EOS: stop; return (EOS NOT pushed)  llm.rs:478-481
             │    └─ else generated_tokens.push(token)              llm.rs:482
             ├─ for seq_len in n..=max_window    llm.rs:495-544
             │    ├─ run_with_runner             llm.rs:510-517
             │    ├─ output = model_outputs[0]   llm.rs:519-528 (ArgMax tensor)
             │    ├─ select last position        llm.rs:530-537
             │    ├─ tx.send(last_token)         llm.rs:539
             │    └─ input_handles = last_token  llm.rs:542-543
             ├─ full_sentence.extend(generated_tokens)   llm.rs:554
             ├─ reset_selective(SOFTMAX_LAYER)   llm.rs:560-561
             ├─ trace = model.run_with_split_nodes_info(full_sentence)  llm.rs:570-572
             │        └─ Model::run_with_split_nodes_info  mod.rs:1587-1625
             │             └─ run_with_runner + TraceRunner (mod.rs:1552-1585, 1613-1619)
             ├─ shift assertions (prover-side)   llm.rs:573-651
             └─ return trace                     llm.rs:656
 └─ Driver::prove                                llm.rs:774-796
     └─ chunked_prove_local → Prover::chunked_prove_local  llm.rs:721-747
         └─ (proof, io); io from trace.to_verifier_io()
```

Facts from source:
- EOS detection happens on the **model output tensor** (`model_outputs[0]`,
  `llm.rs:519-528`), which is the ArgMax output (see §3). The token compared to
  `eos_token` at `llm.rs:477-478` is a value read from that ArgMax tensor.
- When EOS is detected, the results thread sets `stop` and **returns immediately**
  (`llm.rs:478-481`); the EOS token is **not** pushed into `generated_tokens`.
- `full_sentence` is extended only with `generated_tokens` (`llm.rs:554`), i.e.
  **the final full forward input does not contain EOS**.
- The final Trace is produced by a single full forward over `full_sentence`
  (`llm.rs:570-572` → `mod.rs:1587-1625`).
- The source does **not** contain an explicit assertion that the final Trace's
  last output row equals the previously detected EOS. The shift assertions
  (`llm.rs:573-581`, `586-641`, `643-651`) compare `output[i-1] == input[i]` for
  `i in num_input_tokens..full_sentence.len()` only; they never assert the final
  output row against EOS.

### Path M — direct full-sequence single forward

```
P‖z  (caller-supplied Tensor)
 └─ Driver::run(inputs: Vec<Tensor<N>>, store)   llm.rs:339-344   [pub]
     └─ self.model.run(inputs, store)            mod.rs:1628-1633
         └─ run_with_split_nodes_info(..., None) mod.rs:1587-1625
              └─ run_with_runner + TraceRunner → Trace
     └─ self.model.reset()
 └─ Driver::prove(ctx, trace)                    llm.rs:774-796   [pub]
     └─ chunked_prove_local → (proof, io)
 └─ LLMVerifierContext::verify(proof, user_input, io)  llm.rs:804-877
```

Facts from source:
1. `Driver::run` is **`pub`** (`llm.rs:340`) and directly accepts a caller-provided
   `Vec<Tensor<N>>` (`llm.rs:340`).
2. It calls `self.model.run(inputs, store)` only (`llm.rs:341`) — a single forward;
   it performs **no autoregressive token generation**, no EOS detection, no
   `generated_tokens` accumulation, and none of the `run_elements` shift
   assertions.
3. It returns a normal `Trace<N>` (`llm.rs:341-343`), built by the same
   `TraceRunner` (`mod.rs:1613-1619`) used by Path H's final forward.
4. `Driver::prove` is **`pub`** (`llm.rs:774`) and accepts any `Trace<Element>`
   (`llm.rs:777`); it does not check how the trace was created.
5. There is **no runtime check** in `run` or `prove` forcing input token `X[i+1]`
   to equal output token `Y[i]`.

Consequence: an API caller can supply `X = P‖z` with arbitrary `z`, obtain a
valid Trace over `X`, and prove it. The proof attests only that the model was
evaluated once on `X`.

---

## 3. Trace input / output — real data semantics

Sources: `trace.rs` (`Trace` struct `29-40`; `inputs()` `304-307`; `outputs()`
`299-302`; `to_verifier_io()` `311-339`), `mod.rs` (`model_outputs()` `957-979`;
`automatic_output_labelling()` `1520-1544`; `run_with_split_nodes_info()` `1587-1625`),
`parser/llm/mod.rs:160-178`, `parser/llm/metadata.rs:8-18`,
`layers/transformer/logits.rs:187-263`.

| Object | Confirmed semantics | Production site |
| --- | --- | --- |
| `Trace.input` | The Tensor actually fed to the model; for LLM a single rank-1 tensor of token IDs | `trace.rs:34`; set by `TraceRunner::new(..., input_handles)` `mod.rs:1613` |
| `Trace.output` | The model graph's final output tensor = ArgMax token IDs (not logits) | `trace.rs:35`; collected via `model_outputs()` `mod.rs:957-979` from sink node |
| `Trace.steps` | Per-node intermediate data (`HashMap<NodeId, Step<D>>`) | `trace.rs:33` |
| `IO.input` | `Trace.input` → `pad_next_power_of_two()` → `to_field()` | `trace.rs:316-324`, `336` |
| `IO.output` | `Trace.output` → `pad_next_power_of_two()` → `to_field()` | `trace.rs:326-336` |

ArgMax confirmation chain:
- `parser/llm/mod.rs:166-168`: the last layer is
  `Layer::Logits(Logits::new_argmax())`, then `automatic_output_labelling()`.
- `mod.rs:1520-1544`: `automatic_output_labelling` marks the single sink node as
  the model output. The sink is the ArgMax layer.
- `parser/llm/metadata.rs:14-17`: `logits` = layer containing logits; `argmax` =
  layer containing the argmax of the logits. `parser/llm/mod.rs:175-176` sets
  `logits: final_proj_id`, `argmax: logits_id` (the ArgMax node).
- `logits.rs:229-250` (Element path): `binput.max_dim_with_indices(1)` returns
  `(indices_bt, max_bt)` and `LayerOut::from_vec(indices)` is the layer output;
  `logits.rs:255-263` sets output shape `[rows, 1]`.
  ⇒ Each output row is a token ID index `y_i = argmax(logits_i)`, **not** a logit.
- Therefore verifier's `output.data()[i]` is legitimately a predicted token ID.

`to_verifier_io()` does only padding + field conversion; it does **not** reorder
or reinterpret data (`trace.rs:311-339`). Padding is `pad_next_power_of_two`,
which fills with the type default (`tensor/mod.rs:812-813`, `817-835`), i.e. 0
for Element-derived tensors (`burn_wrapper.rs:1382-1389`).

Input/output length relation: for a forward over a length-`L` token tensor,
`Trace.input` has `L` rows and the ArgMax `Trace.output` has `L` rows
(`logits.rs:255-263`, shape `[rows,1]`). The verifier's shift check is
`input[i+1] == output[i]`, encoding the causal LM convention "row `i` predicts
the token at row `i+1`".

---

## 4. Does EOS enter the output Tensor? (conditional judgment)

**B2-1 = Conditional YES.**

Reasoning (all on the fixed commit):
- EOS is detected on the ArgMax output tensor during incremental generation
  (`llm.rs:477-478`, tensor from `llm.rs:519-528`). The detected EOS value is the
  prediction emitted for the position just processed.
- The EOS token is not pushed into `generated_tokens` (`llm.rs:478-481`) and thus
  not appended to `full_sentence` (`llm.rs:554`). So **EOS is absent from the final
  Trace input**.
- The final Trace is a single full forward over `full_sentence` (`llm.rs:570-572`).
  Its last output row is `argmax` at the last position — structurally the same
  "next token" prediction that, in the incremental loop, was the EOS that stopped
  generation. So EOS is **expected in the last output row**.
- But the source contains **no explicit assertion** `output[last] == EOS`; the
  shift assertions (`llm.rs:573-651`) only cover `output[i-1] == input[i]` for
  `i in num_input_tokens..full_sentence.len()`. The final output row is never
  compared to EOS.
- No model run was performed, so numerical equivalence between the incremental
  prediction and the final full-forward last row was **not measured**.

Therefore the honest-path statement "EOS is in the verifier output Tensor" is a
**structural expectation**, not a measured or source-asserted fact. This matters
because the verifier searches EOS in `output.data()` (§5), i.e. in the ArgMax
output — so if the expectation holds, the honest path's EOS is exactly where the
verifier looks. If it does not hold, honest proofs with EOS would be treated as
"no EOS found" and get full-range checking instead (fails safe). Either way it
does not create the bypass; the bypass comes from Path M (§6).

---

## 5. Verifier I/O and index — item-by-item coordinate mapping

`LLMVerifierContext::verify`, `llm.rs:804-877`. Let `n = user_input.len()`.

| Step | Code | Line | Meaning |
| --- | --- | --- | --- |
| 1 | `output.len() == 1` | 820-824 | exactly one output tensor |
| 2 | `output = io.output[0]`; `padded_max_len = output.shape().numel()` | 826-827 | ArgMax output, padded length |
| 3 | `max_window = max_context.unwrap_or(context_length)` | 828 | context bound |
| 4 | `padded_max_len <= max_window.next_power_of_two()` | 831-834 | size bound |
| 5 | `eos_token = md.config.eos_token` | 836 | GPT-2 = `50256` (`config.rs:195`, `models/gpt2/mod.rs:331,361`) |
| 6 | `output.data().iter().take(max_window).skip(n-1).find_position(==eos)` | 837-842 | EOS search over **output** (ArgMax) tensor; skips first `n-1` rows |
| 7 | `unpadded_output_len = pos.0` if found, else `max_window` | 843-847 | **relative** index stored |
| 8 | `prover_input = io.input[0]`; `prover_output = io.output[0]` | 850-851 | the two tensors |
| 9 | `verify(verifier_ctx, proof, io)` | 852 | cryptographic proof check (see §7) |
| 10 | `seq_len = n` | 856 | prefix length |
| 11 | prefix check `prover_input[..n] == user_input[..n]` | 857-863 | prompt prefix equality |
| 12 | `for i in seq_len-1 .. unpadded_output_len-1` | 865-875 | sequentiality loop |
| 13 | `ensure!(prover_input[i+1] == prover_output[i])` | 868-874 | the transition check |

Two coordinate systems must not be conflated:
- **EOS search** runs over the **output** tensor `output.data()`. `skip(n-1)` skips
  rows `0..n-1`; `find_position` returns `pos.0`, which is the index **relative to
  the skipped start**. If the first EOS is at absolute output row `k` (`k >= n-1`),
  then `pos.0 = k - (n-1)`.
- **Sequentiality loop** indexes both tensors with **absolute** indices: it reads
  `prover_input.data()[i+1]` and `prover_output.data()[i]`.

The mismatch: `unpadded_output_len` is set to the **relative** `pos.0` (step 7),
but step 12 uses it as if it were an **absolute** end index in the loop range.
The semantic end should be an absolute output row (the last row that has a
following input token to compare), not a relative offset.

Padding note: EOS search uses `.take(max_window)`, so padding rows beyond
`max_window` are excluded; padding value is 0, which is not the GPT-2 EOS
(`50256`), so zero-fill cannot masquerade as EOS within the searched window. For
Gemma3, EOS values are `1`/`106` (`models/gemma3/mod.rs:465,1134`), also non-zero.
A model whose EOS id were `0` is not covered here and would need separate review.

---

## 6. The `P‖z` scenario — concrete derivation

Symbols (mapped to source variables):
- Prompt length `n = user_input.len()` (`llm.rs:856`).
- User input `P = (p_0,…,p_{n-1})` = `user_input` (`llm.rs:807`).
- Prover submitted model input `X = P‖z` = `prover_input = io.input[0]`
  (`llm.rs:850`), length `n+1`, with `z = X[n]` arbitrary.
- Model ArgMax output `Y = (y_0,…,y_n)` = `prover_output = io.output[0]`
  (`llm.rs:851`), length `n+1`.
- `y_{n-1} = a` = prediction at prompt's last row (first generated token).
- `y_n = EOS` = prediction after appending `z`.
- Condition under study: `z ≠ a` and `y_n = EOS`.

Verifier trace through the scenario:

1. **Prefix check** (`llm.rs:857-863`): `prover_input[0..n] == user_input[0..n]`.
   `X[0..n] = P`, so this **passes**. `z = X[n]` is outside `[0..n]` and is not
   checked here. (`llm.rs:858-861`)

2. **EOS search** (`llm.rs:837-842`): over `Y`, `.skip(n-1)` starts at row `n-1`.
   `Y[n-1] = a` (assume `a ≠ EOS`), `Y[n] = EOS`. First EOS at absolute row `k=n`,
   so relative `pos.0 = n - (n-1) = 1`.

3. **`unpadded_output_len`** (`llm.rs:843-847`) = `pos.0 = 1`.

4. **Sequentiality loop** (`llm.rs:866`):
   ```
   for i in seq_len-1 .. unpadded_output_len-1
          = n-1 .. 1-1
          = n-1 .. 0
   ```
   For `n >= 1`, `n-1 >= 0`, so `Range(n-1, 0)` is **empty** (start ≥ end). The
   loop body at `llm.rs:868-874` never executes.

5. The specific check that would catch the violation is the iteration `i = n-1`:
   `prover_input[n] == prover_output[n-1]`, i.e. `z == a`. This iteration is
   skipped. So **`z ≠ a` is not detected**.

6. **Proof verification** (`llm.rs:852` → `iop/verifier.rs:737-754`) passes because
   the proof correctly attests that the model was evaluated once on `X` to produce
   `Y`; nothing in the proof binds `X[n]` to `Y[n-1]`.

Conclusion: under the witness condition, `verify` returns `Ok` with an unchecked
token transition `X[n-1] → X[n]` (output `a`, input `z`, `z ≠ a`).

Root cause in one line: `pos.0` is a **skip-relative** index but is stored into
`unpadded_output_len` and then used as an **absolute** loop bound; at the second
generated position (`pos.0 = 1`) the bound collapses to `0`, yielding an empty
range. This is the same base inconsistency first flagged in EXP-01-B for the
`pos.0 = 0` underflow case (`llm.rs:866`, `unpadded_output_len - 1`); here it is
the `pos.0 = 1` empty-range case. (EXP-01-B recorded: seq_len=6, EOS absolute 5 →
`pos.0=0` underflow; absolute 6 → `pos.0=1` range `5..0` empty; no EOS → `5..7`.)

Why Path M is required for this to be attacker-relevant: in Path H the prover-side
shift assertions (`llm.rs:573-651`) force `input[i+1] == output[i]` for all
generated positions, so an honest `run_elements` trace cannot contain `z ≠ a`.
Path M (`Driver::run`) produces a provable trace **without** those assertions,
supplying the structural capability to present `P‖z` directly. The verifier cannot
tell which path produced the trace, because it sees only `(proof, io, user_input)`.

---

## 7. Other constraint checks (task §6)

Only constraints directly tied to this question were checked.

| Constraint | Source | Does it force `X[n] = Y[n-1]`? |
| --- | --- | --- |
| Prompt prefix exact match | `llm.rs:857-863` | **No** — covers `[0..n]` only; `X[n]` outside range |
| Input/output legal length | `llm.rs:820-824, 831-834` | No — bounds only |
| Max context allows `P‖z` | `llm.rs:828, 443-448` (`run_elements`); `tokens_to_tensor` `llm.rs:308-318` | No — `run` bypasses `run_elements` length/vocab checks; `tokens_to_tensor` is optional and caller-side |
| ArgMax output has model proof constraint | `logits.rs:375-391, 700-760`; `iop/verifier.rs:737-754` | Partly — proof binds `Y = Model(X)` (single forward), but not `X[n] = Y[n-1]` |
| Independent cross-position transition constraint | only `llm.rs:865-875` | **No** — empty range in the scenario (§6) |
| Padding may affect EOS search | `trace.rs:316-336`; `tensor/mod.rs:812-835` | No — `take(max_window)` + non-zero EOS id; padding 0 cannot match |
| EOS effective position within `take(max_window)` | `llm.rs:837-842` | Assumption — requires first EOS absolute row `< max_window`; for `n+1 <= max_window` this holds |

No additional constraint in the reviewed scope forces `X[n] = Y[n-1]`. B2-4 is
therefore YES **within the defined scope**; this does not rule out constraints in
code outside the permitted review set (e.g. a future v2 verifier, or a deployment
layer that re-derives the trace).

---

## 8. Official path vs. direct API (task §5)

| Path | Call site | Uses `run_elements`? |
| --- | --- | --- |
| Benchmark (sequential) | `zkml/src/bin/bench/llm.rs:298` | **Yes** — `driver.run_elements(...)` |
| Benchmark (distributed) | `bench/llm.rs:283` → `model/exec_graph.rs:57-64` | **Yes** — `InferenceEngine::LLM(driver)` calls `driver.run_elements_with_split_info(...)` |
| Worker immediate | `deep-prove/src/bin/worker/immediate.rs:165` | **Yes** — `driver.run_elements(...)` then `driver.prove(...)` at `166` |
| Verifier wrapper | `deep-prove/src/middleware/llm.rs:31-34` | N/A — `LlmProvable::verify` passes `proof, user_tokens, io` straight to `ctx.verify`; it does not know which Driver method created the trace |
| Direct API | `llm.rs:340` `Driver::run` + `llm.rs:774` `Driver::prove` | **No** — full caller-supplied sequence, single forward, provable |

Key distinctions, kept separate as required:
- **API capability**: `Driver::run` + `Driver::prove` are public and can construct a
  proof over a caller-chosen full sequence. **Source-confirmed.**
- **Official calling habit**: every official LLM generation/prove call site uses
  `run_elements` (or its split-info variant). **Source-confirmed.**
- **Deployment exposure**: whether a real proving service exposes `run` + `prove`
  to an external party, or otherwise accepts externally supplied `io`, is **not
  determinable from this source scope** and remains UNKNOWN. `LlmProvable::verify`
  accepting a proof/IO does not imply the trace's origin is checked — it simply
  never inspects it.
- Official use of `run_elements` ≠ absence of other legal proof paths. `Driver::run`
  existing ≠ proven external reachability in the target deployment.

---

## 9. Evidence ledger (four classes)

### Confirmed facts (source, file + line + commit)
- F1. EOS is detected on the ArgMax output tensor in `run_elements`
  (`llm.rs:477-478`, tensor from `519-528`). Commit `9d1a53e2`.
- F2. Detected EOS is not pushed to `generated_tokens` (`llm.rs:478-481`) and thus
  not in `full_sentence` (`llm.rs:554`).
- F3. Final Trace is one full forward over `full_sentence` (`llm.rs:570-572` →
  `mod.rs:1587-1625`).
- F4. `Driver::run` is `pub`, accepts caller `Vec<Tensor>`, single forward
  (`llm.rs:339-344`); `Driver::prove` is `pub`, accepts any `Trace<Element>`
  (`llm.rs:774-796`).
- F5. verifier EOS search returns skip-relative `pos.0` (`llm.rs:837-847`).
- F6. sequentiality loop uses absolute indices and ends at
  `unpadded_output_len - 1` (`llm.rs:865-875`).
- F7. `to_verifier_io` only pads and converts to field (`trace.rs:311-339`).
- F8. Model output is ArgMax token ID, not logits (`parser/llm/mod.rs:166-168`;
  `mod.rs:1520-1544`; `logits.rs:229-263`; `metadata.rs:14-17`).
- F9. All official call sites use `run_elements`
  (`bench/llm.rs:298,283`; `exec_graph.rs:57-64`; `immediate.rs:165`).
- F10. GPT-2 EOS id = 50256 (`config.rs:195`, `models/gpt2/mod.rs:331,361`).

### Derived results
- D1. With EOS at absolute output row `k`, `pos.0 = k - (n-1)` (from F5).
- D2. At `k = n` (second generated position), `pos.0 = 1`, so
  `unpadded_output_len - 1 = 0` and `Range(n-1, 0)` is empty for `n >= 1` (from F5,F6).
- D3. The skipped iteration is `i = n-1`, i.e. `prover_input[n] == prover_output[n-1]`
  ⇔ `z == a` (from D2 + `llm.rs:868-874`).
- D4. Prefix check passes for `X = P‖z` since it covers only `[0..n]` (from `llm.rs:857-863`).
- D5. Path M has none of the `run_elements` shift assertions, so it can present a
  trace with `z ≠ a` (from F3,F4).

### Conditional judgments
- C1. In the honest Path H, EOS appears in the **last output row** — structurally
  expected but **not** source-asserted and **not** measured (B2-1).
- C2. The bypass requires witness `z ≠ a` and `y_n = EOS` for a concrete model;
  existence is **unknown** (model-dependent).
- C3. Deployment exposes `run`+`prove` or accepts external `io` — **unknown**;
  not shown by source scope.
- C4. Verifier path here is v1 `LlmProvable::verify` / `LLMVerifierContext::verify`;
  v2 `Provable::verify` is out of scope, unchanged from EXP-01.

### Unknowns
- U1. Whether any concrete `(P, z)` on GPT-2 satisfies `z ≠ a ∧ y_n = EOS`.
- U2. Whether a real verifier accepts such a proof/IO (no build, no run this phase).
- U3. Whether a minimal verifier fix (use absolute EOS row as loop end) rejects the
  same proof/IO — the EXP-01判决 rule still requires this to call it reproduced.
- U4. Whether padding or quantization alters the incremental-vs-full-forward last
  row equality on real hardware.
- U5. Deployment attack surface (external access to `run`/`prove`/`io`).

---

## 10. Recommendation for the next (real-model) phase

The structural question is essentially resolved at source level; the remaining
questions are all empirical or deployment-scoped. Recommend, **only after a另行
task decision** and only once the build environment is restored (currently
`RECOVERY_FAIL`, `bench-llm` not compiled):

1. Restore the fixed-commit build via a reliable environment (cloud runner or a
   second host); re-run `B0` smoke before any EOS test. Do not substitute a
   codeload tarball path-dependency for the original build.
2. Minimal witness search: for a fixed GPT-2 checkpoint and a chosen prompt `P`
   with `a = argmax(P)[n-1]`, search a single token `z ≠ a` such that
   `argmax(P‖z)[n] = EOS`. Record `P`, `z`, `a`, the logits rows, and the shift.
3. Build the proof via **`Driver::run(P‖z)` + `Driver::prove`** (Path M) — NOT
   `run_elements`, which would assert away `z ≠ a`. Capture `(proof, io)`.
4. Run `LLMVerifierContext::verify(proof, P, io)` and record accept/reject.
5. Apply the **minimal verifier repair**: set the sequentiality end to the
   absolute last predictive row (e.g. use `(n-1) + pos.0` as the end, or search
   EOS with absolute coordinates) and re-run the **same** `(proof, io)`; record
   reject. Only the pair (original accepts ∧ repaired rejects) with a real
   verifier satisfies the EXP-01 reproduction rule.
6. Separately audit deployment exposure: whether any service accepts externally
   supplied `io`/trace, or exposes `run`+`prove`. Without this, the result stays a
   structural path, not a demonstrated remote exploit.
7. Keep the project-level discipline: this is a **concrete-contribution** finding;
   do not generalize to "EOS sequentiality check is broken in all paths" and do
   not claim a vulnerability until steps 4–5 pass with a real verifier.

## Phase end

Report complete; evidence ledger checked; repository commit + push pending in the
delivery step. This phase stops here. No witness search, no build, no model run,
no EXP-01-C, no new experiment direction was started.
