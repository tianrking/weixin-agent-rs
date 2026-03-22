# wechat-rs-sdk

![CI](https://github.com/tianrking/weixin-agent-rs/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Version](https://img.shields.io/badge/Version-v0.0.1-brightgreen)

A modern Rust WeChat iLink SDK with a pluggable Agent interface and a unified launcher (`wechat-agent`).

Language versions:
- 中文（默认）: [README.md](./README.md)
- English: `README.en.md`
- Español: [README.es.md](./README.es.md)

## Features

- QR login (`get_bot_qrcode`, `get_qrcode_status`)
- Long polling (`getupdates`) with persistent `get_updates_buf`
- Text messaging (`sendmessage`)
- Media upload (`getuploadurl` + CDN)
- Media download/decryption (AES-128-ECB)
- Typing status (`getconfig`, `sendtyping`)
- Session-expired handling (`errcode = -14`)
- Multi-account local credential storage
- Agent abstraction for OpenAI / Anthropic / ACP / custom backends

## Quick Start

```bash
# one command (login + start Codex ACP)
wechat-agent --login --agent codex
```

## Prebuilt CLI Downloads

Download platform packages from Releases:  
<https://github.com/tianrking/weixin-agent-rs/releases>

- macOS Intel: `wechat-agent-<version>-macos-x86_64.dmg`
- macOS Apple Silicon: `wechat-agent-<version>-macos-arm64.dmg`
- Ubuntu 22.04: `wechat-agent_<version>_ubuntu22.04_amd64.deb`
- Ubuntu 24.04: `wechat-agent_<version>_ubuntu24.04_amd64.deb`
- Windows: `wechat-agent-<version>-windows-x86_64.exe`

## One Command For Local Agents

```bash
# Claude Code ACP
wechat-agent --login --agent claude

# Codex ACP
wechat-agent --login --agent codex

# OpenClaw ACP
wechat-agent --login --agent openclaw
```

Force a specific account (recommended for multi-account setups):

```bash
wechat-agent --agent claude --account <account_id>
```

You can copy `account_id` from login output, e.g. `login success: xxx-im-bot`.

## Other Agent Backends

```bash
# OpenAI
OPENAI_API_KEY=... wechat-agent --agent openai

# Anthropic
ANTHROPIC_API_KEY=... wechat-agent --agent anthropic
```

## Runtime Behavior

- Inbound logs include sender, message item types, and text preview.
- Outbound logs include reply kind (`text`, `media`, `fallback`).
- If an agent returns an empty response, SDK sends an automatic fallback text.

## Troubleshooting

- `session expired (errcode -14)`: token expired. Re-login and/or force account:
  - `wechat-agent --agent claude --account <account_id>`
- In multi-account environments, prefer always using `--account`.

## Examples

- `echo`: minimal bot
- `openai`: OpenAI Chat Completions
- `anthropic`: Anthropic Messages API
- `acp`: ACP subprocess adapter (Claude / Codex / Kimi, etc.)

## CI

GitHub Actions matrix:
- Ubuntu
- Windows
- macOS

Workflow: `.github/workflows/ci.yml`
