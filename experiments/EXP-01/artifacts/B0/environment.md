# EXP-01-B0/B1 Environment

Recorded 2026-11-22 (Asia/Shanghai) before any build/run.

## Experiment repository

| Field | Value |
| --- | --- |
| Path | `D:\Research\deep-prove-eos-sequentiality` |
| HEAD at phase start | `4ccd1c0544e2b502850da2d66169d17de2975e4a` |
| Worktree status at phase start | clean |
| Branch | `main` |

## Upstream

| Field | Value |
| --- | --- |
| Repository | `https://github.com/Lagrange-Labs/deep-prove` |
| Pinned commit | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` |
| Isolated workspace | `upstream/deep-prove/` (excluded by `.gitignore`) |
| Upstream modifications | none; no PR; no push |

## Toolchain / OS

| Field | Value |
| --- | --- |
| `rustc --version` | `rustc 1.98.1 (48a229cea 2026-09-01)` |
| `cargo --version` | `cargo 1.98.1 (797e8a9bc 2026-08-05)` |
| Shell | Git Bash (MINGW64_NT-10.0-26320 3.6.9-b4195d69.x86_64) |
| OS | Microsoft Windows 11 Insider Preview build 26320 |
| CPU | Intel Core i7-14700, 20 physical / 28 logical cores |
| RAM | 31.69 GB total, 16.93 GB free at phase start |
| Disk D: | 245 GB total, 164 GB available |
| Network | HTTP(S) proxy `127.0.0.1:53706`; `github.com` CONNECT intermittently 502; `api.github.com` usable |

## Model / config

Model selection for B0: `openai-community/gpt2` via `--hf`, model type
`gpt2`, CPU build (`ndarray`), no `--accuracy`.
Model/tokenizer/config SHA-256 and cache paths: **to be filled after
download** (see `B0/model-hashes.txt`).

## Notes

- `--sequence 8` means prompt length 6 + 2 generated tokens (pinned source
  `zkml/src/bin/bench/llm.rs:78–81,179–191`), not an EOS test.
- B1 EOS cases will use `--max-context M --min-user-len L` (or an explicit
  search driver) to control the generation window; exact configuration will
  be recorded per case.
- No failure, compile error, or panic is recorded as a security result.
