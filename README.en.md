# wechat-rs-sdk

![CI](https://github.com/tianrking/weixin-agent-rs/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Release](https://img.shields.io/github/v/release/tianrking/weixin-agent-rs?sort=semver)

A CLI-first Rust toolkit for WeChat iLink with multi-account, multi-space, and pluggable agent support. The main entrypoint is `wechat-agent`.

Language versions:
- Chinese: [README.md](./README.md)
- English: `README.en.md`
- Español: [README.es.md](./README.es.md)

## Highlights

- `space` as the core unit: account, default agent, user bindings, logs, and pid in one lightweight runtime
- Multi-account support with local credential storage
- Multiple agents: `claude`, `codex`, `openclaw`, `openai`, `anthropic`, `echo`
- CLI-first workflow: `create / ls / start / stop / logs / inspect / bind / update`
- Cross-platform releases for macOS, Windows, Ubuntu, and portable Linux

## Preview

### 1. QR login in terminal
![QR login](./media/scan_code.png)

### 2. Terminal logs during chat
![Terminal logs](./media/in_chat.png)

### 3. Practical CLI interaction
![CLI interaction](./media/cool_cli_01.png)
![Space management](./media/cool_cli_02.png)

### 4. Mobile chat experience
![Phone chat](./media/on_my_phone.png)

### 5. Community group
![Group](./media/wechat_agent_group.JPG)

## Requirements

- Rust 1.78+
- Node.js / `npx` for local ACP agents
- Network access to WeChat iLink API

Extra env vars for cloud models:
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`

## Core Concepts

- `account`: a locally saved WeChat login
- `space`: an isolated runtime with default agent, bound account, user bindings, logs, and pid
- `bind`: route a specific WeChat user to a specific agent
- `agent`: list or switch the default agent of a space

## Quick Start

1. Login one account

```bash
wechat-agent account login
```

2. List saved accounts

```bash
wechat-agent account ls
```

3. Create a space

```bash
wechat-agent space create dev --agent codex
```

4. Bind the account

```bash
wechat-agent space bind-account dev <account_id>
```

5. Start the space

```bash
wechat-agent space start dev
```

6. Tail logs

```bash
wechat-agent space logs dev --tail 100 -f
```

## Command Summary

```bash
wechat-agent account login|ls|rm
wechat-agent space create|ls|ps|inspect|start|stop|restart|logs|rm|bind-account|unbind-account
wechat-agent agent ls|switch
wechat-agent bind ls|set|rm
wechat-agent update

# low-level / experimental
wechat-agent run --space <name>
wechat-agent daemon start|status|stop
```

## Command Guide

### `account`

Manage local WeChat credentials.

```bash
wechat-agent account login
wechat-agent account ls
wechat-agent account rm <account_id>
```

- `login`: starts QR login and prints the resulting `account_id`
- `ls`: lists saved accounts, token presence, user id, and saved time
- `rm`: removes one saved account

### `space`

The main runtime management command.

```bash
wechat-agent space create <name> --agent <agent> [--account <account_id>]
wechat-agent space ls
wechat-agent space ps
wechat-agent space inspect <name>
wechat-agent space start <name>
wechat-agent space stop <name>
wechat-agent space restart <name>
wechat-agent space logs <name> --tail 100 -f
wechat-agent space rm <name>
wechat-agent space bind-account <name> <account_id>
wechat-agent space unbind-account <name>
```

- `create`: creates a space, default agent is `codex`
- `ls` / `ps`: show running state, pid, agent, account, and binding count
- `inspect`: prints a full JSON view including runtime paths
- `start` / `stop` / `restart`: manage the background process
- `logs`: read and follow the space log file
- `rm`: removes a stopped space
- `bind-account` / `unbind-account`: attach or detach a WeChat account

### `agent`

List available agents or switch the default agent of a space.

```bash
wechat-agent agent ls
wechat-agent agent switch <space> <agent>
```

Available agents:
- `claude`
- `codex`
- `openclaw`
- `openai`
- `anthropic`
- `echo`

### `bind`

Per-user routing inside one space.

```bash
wechat-agent bind ls <space>
wechat-agent bind set <space> <user_id> <agent>
wechat-agent bind rm <space> <user_id>
```

Useful when one space defaults to `codex` but one specific user should always use `claude`.

### `update`

Update a source checkout.

```bash
wechat-agent update
```

Behavior:
- runs `git pull --ff-only`
- runs `cargo build --release --locked`
- prints the rebuilt release binary path

This is intended for source users, not binary self-replacement.

### `daemon` and `run`

Low-level commands.

```bash
wechat-agent daemon start
wechat-agent daemon status
wechat-agent daemon stop
wechat-agent run --space <name>
```

- `daemon` is currently experimental
- `run --space` is the direct foreground runtime and is usually called by `space start`

## Agent Modes

### Local ACP agents

```bash
wechat-agent space create dev --agent claude
wechat-agent space create dev --agent codex
wechat-agent space create dev --agent openclaw
```

Notes:
- `claude` and `codex` try to launch local ACP commands
- Windows wrapper scripts are handled specially
- `codex` has a CLI fallback path

### Cloud agents

```bash
OPENAI_API_KEY=... wechat-agent space create openai-space --agent openai
ANTHROPIC_API_KEY=... wechat-agent space create anthropic-space --agent anthropic
```

## Common Flows

### One account, one space

```bash
wechat-agent account login
wechat-agent account ls
wechat-agent space create dev --agent codex
wechat-agent space bind-account dev <account_id>
wechat-agent space start dev
wechat-agent space logs dev -f
```

### Route one user to another agent

```bash
wechat-agent bind set dev user@im.wechat claude
wechat-agent bind ls dev
```

### Inspect runtime state

```bash
wechat-agent space ls
wechat-agent space inspect dev
```

## Troubleshooting

- `space has no bound account`
  Bind an account first with:
  `wechat-agent space bind-account <space> <account_id>`

- `failed to initialize local agent`
  Check whether `npx`, `codex`, `openclaw`, or related local dependencies exist

- `session expired (errcode -14)`
  Re-login with:
  `wechat-agent account login`

- Windows `Access is denied` while replacing `wechat-agent.exe`
  A previous process is still running; stop it before rebuilding

## Prebuilt Downloads

Download platform packages from Releases:
<https://github.com/tianrking/weixin-agent-rs/releases>

- macOS Intel: `wechat-agent-<version>-macos-x86_64.dmg`
- macOS Apple Silicon: `wechat-agent-<version>-macos-arm64.dmg`
- Ubuntu 22.04: `wechat-agent_<version>_ubuntu22.04_amd64.deb`
- Ubuntu 24.04: `wechat-agent_<version>_ubuntu24.04_amd64.deb`
- Ubuntu 24.04 ARM64: `wechat-agent_<version>_ubuntu24.04_arm64.deb`
- Linux GNU x86_64: `wechat-agent-<version>-linux-gnu-x86_64.tar.gz`
- Linux GNU arm64: `wechat-agent-<version>-linux-gnu-arm64.tar.gz`
- Linux MUSL x86_64: `wechat-agent-<version>-linux-musl-x86_64.tar.gz`
- Linux MUSL arm64: `wechat-agent-<version>-linux-musl-arm64.tar.gz`
- Windows: `wechat-agent-<version>-windows-x86_64.exe`

## Contributing

Issues and pull requests are welcome.

## License

This project is licensed under the MIT License. See [LICENSE](./LICENSE).
