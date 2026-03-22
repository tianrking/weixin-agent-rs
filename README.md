# wechat-rs-sdk

A modern Rust SDK for running WeChat iLink bots with pluggable AI agents.

It implements the full practical pipeline used by the TypeScript reference projects:

- QR login (`get_bot_qrcode`, `get_qrcode_status`)
- Long polling (`getupdates`) with persistent `get_updates_buf`
- Text messaging (`sendmessage`)
- Media upload (`getuploadurl` + CDN upload)
- Media download/decrypt from CDN (AES-128-ECB)
- Typing status (`getconfig`, `sendtyping`)
- Session-expired cooldown handling (`errcode = -14`)
- Multi-account local credential persistence
- Agent abstraction to connect OpenAI, Anthropic, ACP-compatible agents, or your own backend

For Chinese docs, see [README.zh-CN.md](./README.zh-CN.md).

## Requirements

- Rust stable (1.78+ recommended)
- WeChat iLink API availability

## Quick Start

```bash
cd /Volumes/ok/Linux_dev_rewrite/wechat_dev/wechat-rs-sdk
cargo check --examples

# login first
cargo run --example echo -- login

# start bot
cargo run --example echo
```

## Examples

- `echo`: minimal echo bot
- `openai`: OpenAI Chat Completions integration
- `anthropic`: Anthropic Messages API integration
- `acp`: ACP subprocess adapter (Claude/Codex/Kimi style ACP agents)

### OpenAI Example

```bash
export OPENAI_API_KEY=sk-...
export OPENAI_MODEL=gpt-5.4
cargo run --example openai -- login
cargo run --example openai
```

### Anthropic Example

```bash
export ANTHROPIC_API_KEY=...
export ANTHROPIC_MODEL=claude-sonnet-4-20250514
cargo run --example anthropic -- login
cargo run --example anthropic
```

### ACP Example

```bash
# default launches: npx -y @zed-industries/codex-acp
cargo run --example acp -- login
cargo run --example acp
```

Optional ACP overrides:

```bash
export ACP_COMMAND=npx
export ACP_ARGS="-y @zed-industries/claude-agent-acp"
cargo run --example acp
```

## Public API

Core entry points:

- `Bot::login(LoginOptions)`
- `Bot::start(agent, StartOptions)`
- `Agent` trait (`chat(ChatRequest) -> ChatResponse`)

ACP adapter:

- `wechat_rs_sdk::agent::acp::AcpAgent`
- `wechat_rs_sdk::agent::acp::AcpAgentOptions`

## CI

GitHub Actions CI is included for:

- Ubuntu
- Windows
- macOS

Workflow file: `.github/workflows/ci.yml`
