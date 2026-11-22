# EXP-01-BR-01-RECOVERY — 异机完整缓存获取

- Date: 2026-11-22
- Phase type: **recovery / environment migration**, 不是新的安全实验
- Execution site: **另一台 GitHub 访问稳定的机器**（A′ 段）+ 本实验机（B′/C′ 段）
- Current status: **NOT STARTED**（本任务包仅完成准备，未执行任何 fetch/build）
- 本阶段**不**证明也**不**证伪 EOS 索引疑点；B0 / B1 / EXP-01-C 继续冻结。

## 1. 为什么另起阶段而不是继续 A3

A3 首次尝试停在 `a16z/arkworks-algebra#76bb3a45`；单次重跑越过该点、
`arkworks` 精确 commit 已成功入库（git db 约 28 MB），随后在
`tracel-ai/burn#c0717304` 遇到 `curl 18` / `early EOF` / 代理 `502`，
在线与离线 fetch 均为 exit 101。

判读口径（权威）：Cargo 末尾的 `revision c0717304… not found` 是传输中断的
**下游报错**，前面已明确记录下载失败；**不能**据此判断 Burn 的 commit 不存在。
状态是 `BR_FAIL_FETCH`（部分缓存已取得），**不是** `BR_FAIL_REVISION`，
不是锁定版本问题，也不是固定 commit 的代码构建失败。

结论：失败点在依赖传输过程，不在锁定版本本身。因此**停止在当前网络重复同一
A3 路径**，改为把获取环境迁移到 GitHub 访问稳定的机器。

## 2. 固定基线（沿用，不重新解析）

| 项 | 固定值 |
| --- | --- |
| 上游 | `https://github.com/Lagrange-Labs/deep-prove.git` |
| 上游 commit | `9d1a53e2ef49ffa2c902b8689cd3c58057a4e662` |
| `Cargo.lock` blob | `7b1b9da7bae322ccebcaff11981c2c4019c613f5` |
| Git 依赖源 | 9 个，见 `../EXP-01-BR-01/git-source-locks.txt` |
| 目标 toolchain | `nightly-2026-01-27` |
| 目标包 / 二进制 | `zkml` / `bench-llm` |
| 最终构建命令 | `cargo +nightly-2026-01-27 build --locked --offline --release -p zkml --bin bench-llm` |

## 3. 执行分段

| 段 | 在哪台机器 | 内容 | 通过判据 |
| --- | --- | --- | --- |
| A′1 | 异机 | 复制固定源码工作区并复核 HEAD / lock blob | `SOURCE_IDENTITY=OK` |
| A′2 | 异机 | 独立 `CARGO_HOME` + `cargo +stable fetch --locked` | exit 0 |
| A′3 | 异机 | `cargo +stable fetch --locked --offline` | exit 0 |
| A′4 | 异机 | 打包 `git` + `registry`，记录 SHA-256 | 校验文件可 `sha256sum -c` 通过 |
| B′1 | 实验机 | 迁移校验 + 新独立 `CARGO_HOME` 解包 | SHA-256 一致 |
| B′2 | 实验机 | `cargo +nightly-2026-01-27 fetch --locked --offline` | exit 0 |
| C′1 | 实验机 | `bench-llm` 离线编译 | exit 0 + 二进制存在 |

详细逐条命令见 `RUNBOOK.md`。脚本见 `scripts/`。

## 4. 明确不做（non-goals）

- 不重做 A0 / A1 / A4：已有记录有效，只做**身份复核**，不重新 clone / fetch 源码。
- 不删除当前部分缓存：`D:\Research\deep-prove-cargo-bundle\cargo-home` 原样保留，
  `arkworks` 部分缓存作为备用；异机缓存到位后使用**新的**独立 Cargo Home。
- 不在当前网络重复 A3，不做无终点重试。
- 不改上游源码、不改 `Cargo.toml` / `Cargo.lock`、不 `cargo update`、
  不用 path-dependency 替代、不改 toolchain / feature。
- 不跑 B0 / B1 / EXP-01-C，不跑模型，不做 EOS prompt 搜索，不改三份既有报告。

## 5. 停止条件

1. 异机 `cargo +stable fetch --locked` 失败 → 停在 A′2，不带失败缓存迁移。
2. `--offline` 自检失败 → 停在 A′3，不打 A′4 包。
3. 迁移后 SHA-256 不一致 → 停在 B′1，不使用该缓存。
4. B′2 nightly 离线 fetch 失败 → 停在 B′2，不进入 C′1 编译。
5. 任一时刻出现 `CURLE 18` / `early EOF` / `502`：不循环重试；记录窗口与新失败点，
   由用户决定是否再换环境。**本阶段不把"再试一次同一路径"作为恢复手段。**

## 6. 状态口径

| 阶段 | 状态 |
| --- | --- |
| EXP-01-B | 冻结 |
| BR-01 原始尝试 | `BR_FAIL_FETCH` |
| BR-01 A3 重跑 | 部分缓存成功，整体仍 `BR_FAIL_FETCH` |
| BR-01-RECOVERY | **NOT STARTED**（任务包已备） |
| B0 / B1 / C | 继续冻结 |

权威决策与判读记录见 `reports/EXP-01-BR-01-RECOVERY-PLAN.md`。
