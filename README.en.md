# wechat-rs-sdk

![CI](https://github.com/tianrking/weixin-agent-rs/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Release](https://img.shields.io/github/v/release/tianrking/weixin-agent-rs?sort=semver)

A modern Rust WeChat iLink Bot SDK with pluggable agents and a unified launcher: `wechat-agent`.

Language versions:
- Chinese: [README.md](./README.md)
- English: `README.en.md`
- Español: [README.es.md](./README.es.md)

## Highlights

- One command entry for agents: `claude` / `codex` / `openclaw` / `openai` / `anthropic`
- Observable from terminal and phone: QR login, inbound logs, outbound logs, fallback replies
- Reliable multi-account operation with explicit account binding
- Release-first delivery across platforms
- Cross-platform packages for macOS, Windows, Ubuntu, and portable Linux

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

## Simple Practical Flow

Prepare:
- Rust 1.78+
- Node.js / `npx`
- Network access to WeChat iLink API

For cloud models, also set:
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`

If you just want to get it running, follow this path:

1. Login one account

```bash
wechat-agent account login
```

2. List local accounts

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

6. Follow logs

```bash
wechat-agent space logs dev --tail 100 -f
```

Core concepts:
- `account`: a locally saved WeChat login
- `space`: a lightweight runtime unit with account, default agent, bindings, logs, and pid
- `agent`: list or switch the default agent of a space
- `bind`: route a specific WeChat user to a specific agent

## Detailed Command Guide

### `account`

Manage local WeChat credentials.

```bash
wechat-agent account login
wechat-agent account ls
wechat-agent account rm <account_id>
```

- `login`: starts QR login and prints `account_id`
- `ls`: lists saved accounts, token presence, user id, and saved time
- `rm`: removes one saved account

### `space`

Manage runtime spaces, the core of the CLI.

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
- `inspect`: prints full JSON including runtime paths
- `start` / `stop` / `restart`: manage the background process
- `logs`: read or follow the space log file
- `rm`: removes a stopped space
- `bind-account` / `unbind-account`: attach or detach a WeChat account

### `agent`

List available agents or switch the default agent of a space.

```bash
wechat-agent agent ls
wechat-agent agent switch <space> <agent>
```

Supported agents:
`claude` / `codex` / `openclaw` / `openai` / `anthropic` / `echo`

### `bind`

Per-user agent routing inside one space.

```bash
wechat-agent bind ls <space>
wechat-agent bind set <space> <user_id> <agent>
wechat-agent bind rm <space> <user_id>
```

- `ls`: list bindings in one space
- `set`: pin one user to one agent
- `rm`: remove one user binding

Typical use:
default to `codex`, route one specific user to `claude`

### `update`

Update a source checkout.

```bash
wechat-agent update
```

Behavior:
- runs `git pull --ff-only`
- runs `cargo build --release --locked`
- prints the rebuilt release binary path

This is for source users, not binary self-replacement.

### `daemon` and `run`

These are low-level commands.

```bash
wechat-agent daemon start
wechat-agent daemon status
wechat-agent daemon stop
wechat-agent run --space <name>
```

- `daemon`: still experimental
- `run --space`: runs one space in the foreground, usually called by `space start`

## Agent Modes

### Local ACP

```bash
wechat-agent space create dev --agent claude
wechat-agent space create dev --agent codex
wechat-agent space create dev --agent openclaw
```

Notes:
- `claude` and `codex` try to launch local ACP commands
- Windows wrapper scripts are handled specially
- `codex` has a CLI fallback

### Cloud models

```bash
OPENAI_API_KEY=... wechat-agent space create openai-space --agent openai
ANTHROPIC_API_KEY=... wechat-agent space create anthropic-space --agent anthropic
```

## Troubleshooting

- `space has no bound account`
  Bind an account first:
  `wechat-agent space bind-account <space> <account_id>`

- `failed to initialize local agent`
  Check whether `npx`, `codex`, `openclaw`, or related dependencies exist

- `session expired (errcode -14)`
  Re-login with:
  `wechat-agent account login`

- Windows `Access is denied` while replacing `wechat-agent.exe`
  A previous process is still running; stop it before rebuilding

## Prebuilt CLI Downloads

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
