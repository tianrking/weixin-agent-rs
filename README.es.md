# wechat-rs-sdk

![CI](https://github.com/tianrking/weixin-agent-rs/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Version](https://img.shields.io/badge/Version-v0.0.1-brightgreen)

SDK moderno en Rust para WeChat iLink con interfaz de Agente extensible y lanzador unificado (`wechat-agent`).

Versiones de idioma:
- 中文（por defecto）: [README.md](./README.md)
- English: [README.en.md](./README.en.md)
- Español: `README.es.md`

## Funcionalidades

- Login por QR (`get_bot_qrcode`, `get_qrcode_status`)
- Long polling (`getupdates`) con persistencia de `get_updates_buf`
- Mensajes de texto (`sendmessage`)
- Subida de media (`getuploadurl` + CDN)
- Descarga y descifrado de media (AES-128-ECB)
- Estado de “escribiendo” (`getconfig`, `sendtyping`)
- Manejo de sesión expirada (`errcode = -14`)
- Persistencia local de credenciales multi-cuenta
- Capa de Agente para OpenAI / Anthropic / ACP / backend propio

## Inicio Rápido

```bash
# un solo comando (login + Codex ACP)
wechat-agent --login --agent codex
```

## Un Comando Para Agentes Locales

```bash
# Claude Code ACP
wechat-agent --login --agent claude

# Codex ACP
wechat-agent --login --agent codex

# OpenClaw ACP
wechat-agent --login --agent openclaw
```

Forzar una cuenta específica (recomendado con múltiples cuentas):

```bash
wechat-agent --agent claude --account <account_id>
```

Puedes obtener `account_id` desde la salida de login, por ejemplo: `login success: xxx-im-bot`.

## Otros Backends

```bash
# OpenAI
OPENAI_API_KEY=... wechat-agent --agent openai

# Anthropic
ANTHROPIC_API_KEY=... wechat-agent --agent anthropic
```

## Comportamiento en Ejecución

- Logs de entrada: remitente, tipo de mensaje y vista previa del texto.
- Logs de salida: tipo de respuesta (`text`, `media`, `fallback`).
- Si el agente no devuelve contenido, el SDK envía un texto de respaldo automáticamente.

## Solución de Problemas

- `session expired (errcode -14)`: token vencido. Haz login otra vez y/o fuerza la cuenta:
  - `wechat-agent --agent claude --account <account_id>`
- En escenarios multi-cuenta, usa `--account` siempre que sea posible.

## Ejemplos

- `echo`: bot mínimo
- `openai`: OpenAI Chat Completions
- `anthropic`: Anthropic Messages API
- `acp`: adaptador ACP (Claude / Codex / Kimi, etc.)

## CI

Matriz de GitHub Actions:
- Ubuntu
- Windows
- macOS

Workflow: `.github/workflows/ci.yml`
