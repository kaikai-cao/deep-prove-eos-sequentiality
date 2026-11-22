# B1 Prompt Search Plan

## Constraint

The pinned bench CLI cannot accept a chosen prompt; B1 therefore uses a
temporary integration test that calls the real `Driver::run_elements → prove
→ verify` chain with a prompt tokenized by the real GPT-2 tokenizer
(`zkml/src/model/llm.rs:1006–1040` upstream test pattern). Model, weights,
EOS token ID, prover and verifier are untouched.

## Target cases

- **case-eos-first**: first generated token is EOS (`eos_token`).
- **case-eos-second**: second generated token is EOS.
- **case-no-eos**: no EOS anywhere in the generation window.

## Why search is needed

GPT-2 `eos_token = 50256`, vocab size 50257. With a legitimate text prompt,
the probability that greedy/argmax generation emits token 50256 at the first
or second generated position is very low. This is a property of the real
model, not a defect. Per the task package: if a target position cannot be
reached by legitimate inputs, record the unreachability evidence; do **not**
mock the proof chain, change the EOS token ID, or forge IO.

## Search procedure

1. Deterministic candidate list: short natural sentences whose final token
   is a strong end-of-sentence signal (e.g. `"The end."`, `"Hello."`,
   `"Yes."`, `"done"`, `"The sky is blue."`, `"Q: hi A:"`, `"</s>"`
   escaped variants). All tokenized by the real GPT-2 tokenizer.
2. For each candidate, run `eos_boundary_probe` with `EOS_PROBE_CTX=8`
   (window of `8 - prompt_len` generated positions) and record
   `OUTPUT_EOS_POSITIONS`, `VERIFIER_SEQ_LEN`, `REL_POS`, and verifier result.
3. Record search space size (number of candidates tried), hit count per
   case, and hit rate. Save every candidate's full console log, not just hits.
4. If no candidate reaches first/second EOS, additionally try prompts whose
   last token is literally the EOS token ID by detokenization search over a
   small token set — only via the real tokenizer, never by writing token IDs
   directly into the verifier input.
5. `case-no-eos` is expected to be reachable with any ordinary prompt;
   still run the full prove+verify chain and record evidence.

## Honesty rules

- `OUTPUT_EOS_POSITIONS` is read from the honest trace's output tensor;
  if EOS is absent there, the case is **not reached** — never relabel it.
- A panic or underflow observed in the verifier is recorded as an
  observation, not as a security bypass.
- No fix code, no verifier modification, no upstream PR.
