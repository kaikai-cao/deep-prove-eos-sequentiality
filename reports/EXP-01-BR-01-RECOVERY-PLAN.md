# EXP-01-BR-01-RECOVERY — 阶段决策与现状（权威记录）

- Date: 2026-11-22
- Type: 环境迁移恢复阶段，**不是**新的安全实验
- Status: **NOT STARTED**（本轮只完成任务包准备；未执行任何 fetch / build）
- 本记录与 `EXP-01-BR-01-CACHE-RESTORE.md`、`EXP-01-BR-01-A3-RETRY.md`
  并列；**不修改**二者，也不修改三份 EXP-01-B 报告。

## 1. A3 重跑的权威判读

用户在核对远端提交 `b3222fedadbe8a472cb0b793801bd1356b7dbea3` 后确认：

- 相对 `c3dc5c6` 只新增 `a3-retry-fetch.log` 与 `EXP-01-BR-01-A3-RETRY.md`，
  原报告未改动；远端可读，差异一致。
- 日志中 `arkworks-algebra#76bb3a45` 精确 commit 已成功拉取；随后 `burn` 下载
  中断，出现 `curl 18`、`early EOF`、代理 `502`；在线与离线 fetch 仍为 exit 101。
- **关键细节**：Cargo 最后打印 `revision c0717304… not found`，但前面已明确记录
  下载中断。因此**不能**据此判断 Burn 的 commit 不存在；更合理的解释是 Git 获取
  过程未完成。

据此固定口径：

| 表述 | 判定 |
| --- | --- |
| 整体状态 | `BR_FAIL_FETCH`（部分缓存已取得） |
| 是否 `BR_FAIL_REVISION` | **否**，锁定版本未被证伪 |
| 是否固定 commit 代码构建失败 | **否**，Phase C′ 从未执行 |
| 是否 EOS 疑点被证实 / 证伪 | **均否** |

## 2. 决策：停止本机重试，迁移获取环境

不再在当前网络环境重复同一 A3 路径。已有证据（A0 曾成功 clone、A3 重跑成功
入库 arkworks、失败点前移到 burn）表明问题在**依赖传输**，而非锁定版本本身。
因此当前最短、最干净的路径是把缓存获取放到 GitHub 访问稳定的另一台机器，
完整获取并离线自检后再迁移回实验机。

## 3. 阶段定义

| 项 | 内容 |
| --- | --- |
| 名称 | BR-01-RECOVERY：异机完整缓存获取 |
| 范围 | 只在异机完成完整 Cargo 缓存获取与离线自检；回实验机做迁移校验、nightly 离线 fetch、`bench-llm` 编译 |
| 不做 | 不重做 A0/A1/A4；不删除当前部分缓存；不重复 A3；不改源码/锁文件/toolchain/feature；不跑 B0/B1/EXP-01-C |
| 固定基线 | commit `9d1a53e2…`、`Cargo.lock` blob `7b1b9da7…`、9 个 Git 源、`nightly-2026-01-27` |

执行步骤（逐条命令与判据见 `experiments/EXP-01-BR-01-RECOVERY/RUNBOOK.md`）：

1. 把固定源码工作区复制到异机，复核 HEAD 与 lock blob 不变（`SOURCE_IDENTITY=OK`）。
2. 异机独立 CARGO_HOME 下 `cargo +stable fetch --locked` 与
   `cargo +stable fetch --locked --offline` **两条都必须 exit 0**。
3. 打包该 CARGO_HOME 的 `git` 与 `registry` 目录，记录 SHA-256。
4. 缓存包迁移回实验机，在新的独立 CARGO_HOME 下
   `cargo +nightly-2026-01-27 fetch --locked --offline`（须先校验 SHA-256）。
5. 离线 fetch 通过后，才执行
   `cargo +nightly-2026-01-27 build --locked --offline --release -p zkml --bin bench-llm`。

当前部分缓存 `D:\Research\deep-prove-cargo-bundle\cargo-home` 原样保留，
`arkworks` 部分缓存作为备用；异机缓存经完整性校验后才使用。

## 4. 状态表

| 阶段 | 状态 |
| --- | --- |
| EXP-01-B | 冻结 |
| BR-01 原始尝试 | `BR_FAIL_FETCH` |
| BR-01 A3 重跑 | 部分缓存成功，整体仍 `BR_FAIL_FETCH` |
| BR-01-RECOVERY | **NOT STARTED**（任务包已备，异机执行待用户安排） |
| B0 / B1 / C | 继续冻结 |

## 5. 停止条件与不变量

停止条件：异机在线 fetch 失败→停 A′2；离线自检失败→停 A′3；
SHA-256 不一致→停 B′1；nightly 离线 fetch 失败→停 B′2。
出现 `CURLE 18` / `early EOF` / `502` 时不循环重试，记录新失败点由用户决定。

不变量：不改上游源码、`Cargo.toml`、`Cargo.lock`；不 `cargo update`；不用
path-dependency 替代；不改 toolchain / feature；不提交 PR；不跑模型与 EOS
prompt 搜索；不修改既有报告；仓库保持 Private、不做披露或论文包装。

## 6. 结论边界

在异机缓存恢复与 `bench-llm` 编译通过之前，固定版本的可重复构建能力仍未恢复。
本阶段即使全部通过，也只说明构建环境恢复，**不**构成对 EOS 索引疑点的任何
安全结论；回到 EOS 需用户另行下达任务包（B0 / B1 / EXP-01-C 仍冻结）。
