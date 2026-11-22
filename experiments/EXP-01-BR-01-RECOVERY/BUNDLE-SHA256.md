# BR-01-RECOVERY — 缓存包 SHA-256 记录（待异机填写）

本文件在异机完成 A′4 打包后填写；实验机 B′1 迁移时按此值校验。
未填写前本阶段状态为 `NOT STARTED`。

| 字段 | 值 |
| --- | --- |
| 生成机器 | （待填） |
| 生成时间 | （待填） |
| 源文件工作区 HEAD | （待填，应为 `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`） |
| `Cargo.lock` blob | （待填，应为 `7b1b9da7bae322ccebcaff11981c2c4019c613f5`） |
| stable cargo 版本 | （待填，应 >= 1.85） |
| `cargo +stable fetch --locked` exit | （待填，必须 0） |
| `cargo +stable fetch --locked --offline` exit | （待填，必须 0） |
| 缓存包文件名 | （待填，建议 `cargo-bundle-remote.tar.gz`） |
| SHA-256 | （待填，来自 `.sha256` 文件） |
| 打包内容 | `git/` + `registry/`（不含 `config.toml`） |
| 实验机 B′1 校验结果 | （待填，必须 OK） |
