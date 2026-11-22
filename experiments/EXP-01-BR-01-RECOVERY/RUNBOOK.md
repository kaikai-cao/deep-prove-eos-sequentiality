# BR-01-RECOVERY — 逐条执行 runbook

按段顺序执行；每段有明确通过判据，未通过则停在当段，不进入下一段。
命令均用 bash / Git Bash 语法；路径含空格时加双引号。
脚本位于本目录 `scripts/`，请先复制到目标机器对应位置。

---

## A′ 段 — 在 GitHub 访问稳定的异机执行

### A′0 前置

异机需具备：bash、git、tar、sha256sum、stable Rust（`cargo +stable --version >= 1.85`）。
不需要 nightly（nightly 只在实验机 B′/C′ 段使用）。

### A′1 复制源码工作区并复核身份

把实验机上的固定源码工作区 `D:\Research\deep-prove-cache-source` 整个复制到异机
（例如 `SRC`）。**只复制，不重新 clone、不 fetch。**

```bash
SRC="/path/to/deep-prove-cache-source"
bash scripts/verify-source.sh "$SRC" | tee a1-source-identity.log
```

通过判据：输出末行为 `SOURCE_IDENTITY=OK`，且
`HEAD=9d1a53e2ef49ffa2c902b8689cd3c58057a4e662`、
`Cargo.lock_blob=7b1b9da7bae322ccebcaff11981c2c4019c613f5`、
`git status --short` 为空。

> 若复制工具丢失 `.git`（wor tree 变成普通目录），此步必然失败。此时应重新按
> `../EXP-01-BR-01/README.md` 的固定步骤获取精确 commit，但**不重做 A1/A4 的
> 新解析**——只恢复同一 commit 与同一 lock blob。

### A′2 / A′3 独立 Cargo Home 的在线 + 离线自检

```bash
export SRC="/path/to/deep-prove-cache-source"
export REMOTE_HOME="/path/to/remote-cargo-home"   # 新目录，勿复用实验机缓存
mkdir -p "$REMOTE_HOME"
printf '[net]\ngit-fetch-with-cli = true\n' > "$REMOTE_HOME/config.toml"

cd "$SRC"
export CARGO_HOME="$REMOTE_HOME"
cargo +stable fetch --locked          2>&1 | tee a2-online-fetch.log
echo "A2_ONLINE_EXIT=${PIPESTATUS[0]}" | tee -a a2-online-fetch.log
```

**判据 A′2**：`A2_ONLINE_EXIT=0`。若为 101，停在 A′2，不迁移、不打包。

```bash
cargo +stable fetch --locked --offline 2>&1 | tee a3-offline-fetch.log
echo "A3_OFFLINE_EXIT=${PIPESTATUS[0]}" | tee -a a3-offline-fetch.log
```

**判据 A′3**：`A3_OFFLINE_EXIT=0`。若为 101，停在 A′3，不打包。
两条命令必须**都**成功，才允许进入 A′4。

### A′4 打包 git + registry 并记录 SHA-256

```bash
export REMOTE_HOME="/path/to/remote-cargo-home"
export BUNDLE="/path/to/cargo-bundle-remote.tar.gz"   # 建议绝对路径
bash scripts/pack-bundle.sh "$REMOTE_HOME" "$BUNDLE" | tee a4-pack.log
```

脚本只打包 `$REMOTE_HOME/git` 与 `$REMOTE_HOME/registry`；`config.toml` 是机器相关
网络设置，**刻意不打进包**，迁移后在实验机重新建立自己的独立 home。
产物：`cargo-bundle-remote.tar.gz` 与 `cargo-bundle-remote.tar.gz.sha256`。

自检：

```bash
cd "$(dirname "$BUNDLE")" && sha256sum -c cargo-bundle-remote.tar.gz.sha256
```

**判据 A′4**：校验输出 `OK`，并把 SHA-256 值抄入 `BUNDLE-SHA256.md`。

---

## B′ / C′ 段 — 回到实验机执行

### B′0 前置

实验机已有 `nightly-2026-01-27`（已安装，rustc 1.95.0-nightly）。
**不要删除** `D:\Research\deep-prove-cargo-bundle\cargo-home`；新建独立 home。

### B′1 迁移校验 + 解包到新 Cargo Home

```bash
export BUNDLE="/path/to/cargo-bundle-remote.tar.gz"      # 从异机传回
export LOCAL_HOME="D:/Research/deep-prove-cargo-bundle/cargo-home-recovery"
export SRC="D:/Research/deep-prove-cache-source"
mkdir -p "$LOCAL_HOME"
cd "$(dirname "$BUNDLE")" && sha256sum -c cargo-bundle-remote.tar.gz.sha256
```

**判据 B′1**：SHA-256 校验 `OK`。不一致则停止，不解包、不使用该缓存。

### B′2 nightly 离线 fetch

```bash
bash scripts/verify-and-build.sh "$BUNDLE" "$LOCAL_HOME" "$SRC" \
  fetch 2>&1 | tee b2-nightly-offline-fetch.log
```

等价直接命令：

```bash
export CARGO_HOME="$LOCAL_HOME"
cd "$SRC"
cargo +nightly-2026-01-27 fetch --locked --offline
```

**判据 B′2**：exit 0。若失败，停在 B′2，不进入 C′1。

### C′1 离线编译 bench-llm

仅在 B′2 通过后执行：

```bash
export CARGO_HOME="$LOCAL_HOME"
cd "$SRC"
cargo +nightly-2026-01-27 build --locked --offline --release \
  -p zkml --bin bench-llm 2>&1 | tee c1-build-bench-llm.log
```

**判据 C′1**：exit 0，且目标二进制存在（记录实际路径与大小）。
编译通过后**仍不**自动进入 B0 / B1 / EXP-01-C；需用户另行下达任务包。

---

## 证据归档约定

各段落日志、SHA-256 文件、二进制路径与退出码统一放在
`experiments/EXP-01-BR-01-RECOVERY/` 下；最终结论写入
`reports/EXP-01-BR-01-RECOVERY-RESULT.md`（成功或失败都要写）。
本 runbook 与三份既有 BR-01 / EXP-01-B 报告保持独立，不修改旧文件。
