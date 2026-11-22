# EXP-01-B2 — Fixed Source Identity Snapshot

All analysis in EXP-01-B2 is performed against this exact, immutable snapshot.
No Cargo invocation, no build, no model load, no upstream modification.

## Repository identity (recorded 2026-11-22, Asia/Shanghai)

| Item | Value |
| --- | --- |
| Experiment repo | `kaikai-cao/deep-prove-eos-sequentiality` (Private) |
| Experiment HEAD (task start) | `8d51d6850d70a75222a140a26e37e8bcea645891` |
| Experiment worktree | clean |
| Local upstream checkout | `D:/Research/deep-prove-cache-source` |
| Upstream HEAD | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` |
| `Cargo.lock` blob | `7b1b9da7bae322ccebcaff11981c2c4019c613f5` |
| Upstream worktree | clean |

Identity check: PASS. Upstream HEAD and `Cargo.lock` blob match the frozen baseline
from EXP-01-B / BR-01. No switch to upstream latest branch was made.

## Reviewed source blobs (git `HEAD:<path>` on the fixed commit)

| Path | blob SHA | lines |
| --- | --- | --- |
| `zkml/src/model/llm.rs` | `ab987ac13e05914cb7ce45e6838dd586730e8b83` | 1250 |
| `zkml/src/model/trace.rs` | `475dc30b0d67a156c40c523a8273e30844ffdc00` | 774 |
| `zkml/src/model/mod.rs` | `3e7f87cacdfc8b6ece550ef2faa04a789eb08bd1` | 2430 |
| `zkml/src/model/exec_graph.rs` | `501527d6f64d3724ce6c7d1a51fed1a119fa4e23` | — |
| `zkml/src/parser/llm/mod.rs` | `798bfb05bfc1d375ff40c352dd16ffd5047e839d` | 361 |
| `zkml/src/parser/llm/metadata.rs` | `b80671a5f6fdfb3b8007851be3e4eba6c827518a` | 105 |
| `zkml/src/parser/llm/config.rs` | (read for eos_token only) | — |
| `zkml/src/layers/transformer/logits.rs` | `c3e931ace046a491ffbb91f463801c4aa51f26f1` | 1375 |
| `zkml/src/iop/verifier.rs` | `863d2c277cab227f711eb4b15274577acb76c693` | 811 |
| `zkml/src/tensor/mod.rs` | (read for pad semantics) | — |
| `zkml/src/tensor/burn_wrapper.rs` | (read for argmax/pad) | — |
| `zkml/src/bin/bench/llm.rs` | `7726eee9677d5f440b8b8f00427cf3930bb74841` | 590 |
| `deep-prove/src/bin/worker/immediate.rs` | `4fd9d167da56b8b1752481049535193d9e427e4b` | 320 |
| `deep-prove/src/middleware/llm.rs` | `d1c6da39e5a9729ce25954fa03e571fe6a4bff6d` | 35 |

## Scope discipline

- Only the files above and their direct call relations were inspected.
- Source-only: every conclusion cites file + commit + line number.
- No proof / IO / trace was constructed; no witness (P, z) was searched.
- `LlmProvable::verify` (v1, `middleware/llm.rs`) is the verifier wrapper in scope;
  v2 `Provable::verify` is out of scope, unchanged from EXP-01.
