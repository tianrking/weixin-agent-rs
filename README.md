# wechat-rs-sdk

![CI](https://github.com/tianrking/weixin-agent-rs/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Release](https://img.shields.io/github/v/release/tianrking/weixin-agent-rs?sort=semver)

一个偏 CLI、可管理多账号与多空间的 Rust WeChat iLink 工具链。核心入口是 `wechat-agent`，重点是安全、快、稳、小，而不是做成笨重的控制面板。

语言版本：
- 中文（默认）：`README.md`
- English: [README.en.md](./README.en.md)
- Español: [README.es.md](./README.es.md)

## 项目亮点

- `space` 抽象：把微信账号、默认 agent、用户绑定、日志和运行状态收进一个轻量空间
- 多账号管理：支持扫码登录、列出、删除本地账号
- 多 agent 路由：支持 `claude`、`codex`、`openclaw`、`openai`、`anthropic`、`echo`
- CLI 优先：`create / ls / start / stop / logs / inspect / bind / update`
- 跨平台交付：macOS、Windows、Ubuntu `.deb` 与 Linux 可移植包

## 效果预览

### 1. 扫码登录（终端二维码）
![扫码登录](./media/scan_code.png)

### 2. 聊天时候终端显示（入站/出站日志）
![终端日志](./media/in_chat.png)

### 3. 实用优雅的命令交互
![命令交互](./media/cool_cli_01.png)
![Space 管理](./media/cool_cli_02.png)

### 4. 手机聊天体验（可控制 Claude/Codex/OpenClaw，后续将支持更多 Agent）
![手机聊天体验](./media/on_my_phone.png)

### 5. 欢迎加入讨论群组
![群聊效果](./media/wechat_agent_group.JPG)

## 安装与环境

需要：
- Rust 1.78+
- Node.js / `npx`：用于本地 ACP agent
- 能访问 WeChat iLink API

云模型额外环境变量：
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`

## 核心概念

- `account`：本地保存的微信登录凭据
- `space`：一个独立运行空间，保存默认 agent、绑定账号、用户级 agent 绑定、日志和 pid
- `bind`：把某个微信用户固定路由到指定 agent
- `agent`：空间默认 agent 的查看和切换

## 快速开始

1. 登录一个微信账号

```bash
wechat-agent account login
```

2. 查看本地账号

```bash
wechat-agent account ls
```

3. 创建空间

```bash
wechat-agent space create dev --agent codex
```

4. 绑定账号

```bash
wechat-agent space bind-account dev <account_id>
```

5. 启动空间

```bash
wechat-agent space start dev
```

6. 查看日志

```bash
wechat-agent space logs dev --tail 100 -f
```

## 命令总览

```bash
wechat-agent account login|ls|rm
wechat-agent space create|ls|ps|inspect|start|stop|restart|logs|rm|bind-account|unbind-account
wechat-agent agent ls|switch
wechat-agent bind ls|set|rm
wechat-agent update

# 低层/实验性
wechat-agent run --space <name>
wechat-agent daemon start|status|stop
```

## 命令详解

### `account`

管理本地微信登录凭据。

```bash
wechat-agent account login
```

- 发起扫码登录
- 成功后输出 `account_id`

```bash
wechat-agent account ls
```

- 列出已保存账号
- 显示 token 是否存在、用户 ID、保存时间

```bash
wechat-agent account rm <account_id>
```

- 删除本地账号凭据

### `space`

管理运行空间，是当前 CLI 的核心。

```bash
wechat-agent space create <name> --agent <agent> [--account <account_id>]
```

- 创建空间
- `name` 会被规范化
- 默认 agent 不填时使用 `codex`

```bash
wechat-agent space ls
wechat-agent space ps
```

- 列出空间
- 显示运行状态、pid、默认 agent、绑定账号、用户绑定数量

```bash
wechat-agent space inspect <name>
```

- 输出完整 JSON
- 包含空间目录、日志文件、pid 文件、当前 pid、用户绑定等

```bash
wechat-agent space start <name>
wechat-agent space stop <name>
wechat-agent space restart <name>
```

- 后台启动、停止、重启空间
- `start` 需要该空间已经绑定账号

```bash
wechat-agent space logs <name> --tail 100 -f
```

- 查看空间日志
- `--tail` 控制末尾行数
- `-f`/`--follow` 持续跟随

```bash
wechat-agent space rm <name>
```

- 删除空间
- 运行中的空间不能直接删除，必须先 `stop`

```bash
wechat-agent space bind-account <name> <account_id>
wechat-agent space unbind-account <name>
```

- 绑定或解绑空间使用的微信账号

### `agent`

查看可用 agent，或切换某个空间的默认 agent。

```bash
wechat-agent agent ls
wechat-agent agent switch <space> <agent>
```

可用 agent：
- `claude`
- `codex`
- `openclaw`
- `openai`
- `anthropic`
- `echo`

### `bind`

做用户级 agent 路由。

```bash
wechat-agent bind ls <space>
wechat-agent bind set <space> <user_id> <agent>
wechat-agent bind rm <space> <user_id>
```

适合这种场景：
- 默认走 `codex`
- 某个用户固定走 `claude`

### `update`

用于源码 checkout 下的自更新。

```bash
wechat-agent update
```

行为：
- 执行 `git pull --ff-only`
- 执行 `cargo build --release --locked`
- 输出新的 release 二进制路径

说明：
- 这是面向源码仓库用户的更新方式
- 不是二进制自替换升级器

### `daemon` 与 `run`

这两个是低层能力。

```bash
wechat-agent daemon start
wechat-agent daemon status
wechat-agent daemon stop
```

- 当前是实验性壳层
- 正常使用不需要先理解 daemon

```bash
wechat-agent run --space <name>
```

- 直接以前台方式运行空间
- 一般由 `space start` 间接调用

## Agent 模式

### 本地 ACP

```bash
wechat-agent space create dev --agent claude
wechat-agent space create dev --agent codex
wechat-agent space create dev --agent openclaw
```

说明：
- `claude`、`codex` 会尝试通过本机命令启动 ACP
- Windows 下会额外处理 `.cmd/.bat/.ps1`
- `codex` 有 CLI fallback

### 云模型

```bash
OPENAI_API_KEY=... wechat-agent space create openai-space --agent openai
ANTHROPIC_API_KEY=... wechat-agent space create anthropic-space --agent anthropic
```

## 常见流程

### 单账号单空间

```bash
wechat-agent account login
wechat-agent account ls
wechat-agent space create dev --agent codex
wechat-agent space bind-account dev <account_id>
wechat-agent space start dev
wechat-agent space logs dev -f
```

### 一个空间里给某个用户切换 agent

```bash
wechat-agent bind set dev user@im.wechat claude
wechat-agent bind ls dev
```

### 查看空间状态

```bash
wechat-agent space ls
wechat-agent space inspect dev
```

### 更新当前源码版

```bash
wechat-agent update
```

## 排障

- `space has no bound account`
  说明空间还没绑定微信账号，先执行：
  `wechat-agent space bind-account <space> <account_id>`

- `failed to initialize local agent`
  检查本机 `npx`、`codex`、`openclaw` 或相关依赖是否存在

- `session expired (errcode -14)`
  token 过期，重新登录：
  `wechat-agent account login`

- Windows 下 `Access is denied` 删除不了 `wechat-agent.exe`
  说明旧进程还在运行，先停止旧进程再重新构建

## 预编译 CLI 下载

从 Releases 下载对应平台安装包：
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

## 贡献

欢迎通过 Issue / Pull Request 参与改进。

## 开源许可

本项目采用 MIT License，详见 [LICENSE](./LICENSE)。
