# wechat-rs-sdk

一个现代化的 Rust WeChat iLink Bot SDK，支持可插拔 Agent。

已覆盖主要完整链路：

- 扫码登录（`get_bot_qrcode`、`get_qrcode_status`）
- 长轮询收消息（`getupdates`）+ `get_updates_buf` 断点续拉
- 文本发送（`sendmessage`）
- 媒体上传（`getuploadurl` + CDN）
- 媒体下载解密（AES-128-ECB）
- 打字状态（`getconfig`、`sendtyping`）
- 会话过期保护（`errcode = -14`）
- 多账号本地凭据持久化
- Agent 抽象层，可接 OpenAI/Anthropic/ACP/自研后端

英文文档见 [README.md](./README.md)。

## 快速开始

```bash
cd /Volumes/ok/Linux_dev_rewrite/wechat_dev/wechat-rs-sdk
cargo check --examples

# 先登录
cargo run --example echo -- login

# 启动
cargo run --example echo
```

## 示例

- `echo`：最小回声机器人
- `openai`：OpenAI Chat Completions 接入
- `anthropic`：Anthropic Messages API 接入
- `acp`：ACP 子进程适配（Claude/Codex/Kimi 这类 ACP Agent）

### OpenAI

```bash
export OPENAI_API_KEY=sk-...
export OPENAI_MODEL=gpt-5.4
cargo run --example openai -- login
cargo run --example openai
```

### Anthropic

```bash
export ANTHROPIC_API_KEY=...
export ANTHROPIC_MODEL=claude-sonnet-4-20250514
cargo run --example anthropic -- login
cargo run --example anthropic
```

### ACP

```bash
# 默认: npx -y @zed-industries/codex-acp
cargo run --example acp -- login
cargo run --example acp
```

切换 ACP 命令：

```bash
export ACP_COMMAND=npx
export ACP_ARGS="-y @zed-industries/claude-agent-acp"
cargo run --example acp
```

## CI

已包含 GitHub Actions 三平台 CI：

- Ubuntu
- Windows
- macOS

配置文件：`.github/workflows/ci.yml`
