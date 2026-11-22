# EXP-01-B Finalization — Refined Research Statement and Frozen Status

- Date: 2026-11-22
- Scope: deep-prove-eos-sequentiality · EXP-01-B (B0_FAIL / B1 not executed)
- Baseline: local + remote `main` = `3741a61a78686768661de66354a7881e17a6e9c0`

## 0. Status of this document

This file is the **authoritative statement of the current research conclusion** for
EXP-01-B. It records the user's final review and tightens the wording used in
`reports/EXP-01-B-BOUNDARY-RESULTS.md` and `reports/EOS-SEQUENTIALITY-BYPASS-EXP01.md`.

- The two previously committed reports are **not modified**. They remain the
  traceable record of how the conclusion was reached.
- Where wording differs, **this document governs**.
- Nothing in this document changes the experiment verdict: **B0_FAIL / B1 not
  executed**. No security conclusion is drawn.

---

## 1. Refined research statement

The most accurate current formulation is:

> 已发现一个值得验证的 EOS 索引基数疑点，但尚未证明它能导致真实 verifier
> 错误接受。运行实验因外部 Git 依赖无法获取而受阻。

### 1.1 What can be asserted

| Item | Status |
| --- | --- |
| Fixed upstream commit and three key source files cross-checked | Confirmed (blob SHA matched local checkout) |
| `.skip(...).find_position(...)` returns the **relative** position in the skipped iterator, not the absolute index in the original output array | Source-level fact |
| Under the assumed index correspondence, `pos.0 = 1` can make the sequentiality `Range` empty | Conditional on the assumption (mathematical derivation) |
| `run_elements` does **not** append EOS to `full_sentence` when generation stops on EOS | Source-level fact |
| Consequence: whether an honest trace can ever let the verifier see EOS is now the more basic open question | Open, not experimentally tested |

### 1.2 What cannot yet be asserted

1. Whether `seq_len`, `user_input.len()` and `output.data()` indices really live
   in the same coordinate system assumed in the derivation.
2. Whether a real `run_elements → prove → verify` path can produce a sample with
   EOS at the second generated position.
3. Whether other constraints in the verifier still reject such a trace even if
   the sequentiality loop is empty.

These three unknowns are why the result is a **疑点 (suspicion)**, not a finding.

### 1.3 Careful reading of `pos.0 = 0`

`usize` underflow must not be reported as a security bypass by itself:

- In a build with overflow checks enabled, it may panic directly.
- In a typical release build with checks disabled, it may wrap to a very large
  value and subsequently trigger an out-of-bounds access.

Either outcome is an index-arithmetic observation. **Neither constitutes evidence
of a security bypass.**

---

## 2. Two findings worth preserving

### 2.1 Whether EOS enters the trace precedes random prompt search

Since `run_elements` stops on EOS without appending it to `full_sentence`,
blind enumeration of random prompts has low expected value. Three questions must
be settled first:

1. Is EOS only a stop signal, or does it enter the subsequent `Trace`?
2. How are `prover_input` and `prover_output` used by `Driver::prove` constructed
   from the trace?
3. Which tensor does the `output` searched by the verifier correspond to — the
   generated sequence, the full sequence, or another transformed tensor?

These three questions determine whether B1 has any executable path at all. They
are recorded as the first thing to resolve when the build environment is restored;
they are **not** answered in this frozen stage.

### 2.2 Environment blockage and security results must be accounted separately

The `cargo build` in this stage never reached compilation of upstream code.

- **B0_FAIL is a task-execution status**, caused by inaccessible external Git
  dependencies.
- It does **not** mean the fixed commit's code fails to build in general.
- It does **not** mean the suspicion has been falsified.
- It does **not** constitute any security result, positive or negative.

This distinction is deliberately preserved in the reports and must be preserved
in any later citation.

---

## 3. Next step (reserve status — not started this round)

Do **not** start EXP-01-C. Do **not** continue open-ended network retries.
The single objective for the next stage is:

**Restore reproducible builds of the fixed commit.**

Priority order:

1. In an environment with working GitHub access, obtain the exact pinned Git
   dependency revisions required by the fixed commit and prepare a portable
   Cargo Git cache.
2. Migrate that cache into the experiment environment and verify Cargo can
   complete dependency resolution using those exact revisions.
3. Re-run B0. Only after a real `run_elements → prove → verify` run passes
   end-to-end should B1 resume.

Explicitly rejected as a substitute: extracting codeload tarballs, converting
them to path dependencies, and labeling the result an as-is build of the fixed
upstream commit. That may be done as a **separate auxiliary experiment**, but it
cannot satisfy this experiment's version-consistency requirement.

---

## 4. Freeze terms

- EXP-01-B is **frozen** with status **B0_FAIL / B1 not executed**.
- EXP-01-C is **not started**. Its entry condition is unchanged: a successful B0
  on the fixed commit plus at least one real verifier result from B1.
- No further network retry loops, no scope expansion, no topic switch in this
  stage.
- The suspicion is kept in **reserve**. Once a fixed-version build environment
  is available, decide then whether it is a security defect, an implementation
  robustness issue, or an artifact of index presentation.

## 5. Value of this stage

This stage did not fail as a research direction. It narrowed a broad suspicion
down to a concrete, checkable program behavior and identified the real
infrastructure blocker. It produced no verifier evidence and makes no
vulnerability claim; the repository remains Private with no disclosure and no
paper packaging.
