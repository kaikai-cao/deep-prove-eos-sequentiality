# DeepProve EOS Sequentiality

An isolated reproducibility study of the EOS boundary and autoregressive
sequentiality checks in DeepProve.

## Status

EXP-01: Not yet reproduced. No vulnerability claim is made at this stage.

## Scope

1. Verify EOS boundary behavior.
2. Test whether a non-autoregressive token sequence can be accepted.
3. Apply a minimal index correction as a control.
4. Re-run the exact same proof and IO against the corrected verifier.

No upstream source modifications are made. Candidate corrections are kept as
diffs under `experiments/EXP-01/patches/` and are applied only to an isolated
worktree/copy of the pinned upstream revision.

## Repository policy

- This is a vulnerability reproduction and verification repository, not a
  paper project. It is private until reproduction is confirmed and a
  responsible-disclosure decision has been made.
- A defect is considered reproduced only if the original verifier accepts a
  non-autoregressive sequence and the minimally corrected verifier rejects
  the exact same proof and IO. An index anomaly or a panic alone is not a
  completed attack reproduction.
- All evidence is versioned here: pinned upstream revision, model and
  configuration, proof, IO, verifier output, reproduction logs, and the
  minimal fix diff. Large generated artifacts are kept outside Git with
  hashes/manifests recorded in the repository.
- Do not publish PoC material before confirmation and disclosure review.

## Layout

| Path | Purpose |
| --- | --- |
| `EXPERIMENT-PLAN.md` | Authoritative EXP-01-A to EXP-01-D boundaries and acceptance criteria |
| `upstream/UPSTREAM-REVISION.md` | Pinned upstream commit, source paths, call chain |
| `experiments/EXP-01/` | Phase scripts, patches, artifacts |
| `reports/EOS-SEQUENTIALITY-BYPASS-EXP01.md` | Main report (skeleton until evidence exists) |
| `notes/` | Source inspection and experiment notes |

## Upstream

Pinned revision: `Lagrange-Labs/deep-prove` @ `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`.
See `upstream/UPSTREAM-REVISION.md` for source locations and retrieval commands.
