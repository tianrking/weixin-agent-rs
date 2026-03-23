# wechat-rs-sdk

![CI](https://github.com/tianrking/weixin-agent-rs/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Release](https://img.shields.io/github/v/release/tianrking/weixin-agent-rs?sort=semver)

Herramienta Rust orientada a CLI para WeChat iLink con soporte para múltiples cuentas, múltiples espacios y agentes enchufables. La entrada principal es `wechat-agent`.

Versiones de idioma:
- 中文: [README.md](./README.md)
- English: [README.en.md](./README.en.md)
- Español: `README.es.md`

## Puntos clave

- `space` como unidad principal: cuenta, agente por defecto, bindings por usuario, logs y pid
- Soporte multi-cuenta con credenciales locales
- Múltiples agentes: `claude`, `codex`, `openclaw`, `openai`, `anthropic`, `echo`
- Flujo CLI primero: `create / ls / start / stop / logs / inspect / bind / update`
- Releases multiplataforma para macOS, Windows, Ubuntu y Linux portátil

## Vista previa

### 1. Login QR en terminal
![QR login](./media/scan_code.png)

### 2. Logs de terminal durante el chat
![Logs de terminal](./media/in_chat.png)

### 3. Interacción CLI práctica
![CLI](./media/cool_cli_01.png)
![Gestión de space](./media/cool_cli_02.png)

### 4. Experiencia en móvil
![Móvil](./media/on_my_phone.png)

### 5. Grupo de comunidad
![Grupo](./media/wechat_agent_group.JPG)

## Requisitos

- Rust 1.78+
- Node.js / `npx` para agentes ACP locales
- Acceso de red a WeChat iLink API

Variables extra para modelos en la nube:
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`

## Conceptos principales

- `account`: login de WeChat guardado localmente
- `space`: runtime aislado con agente por defecto, cuenta asociada, bindings, logs y pid
- `bind`: enrutar un usuario de WeChat a un agente específico
- `agent`: listar o cambiar el agente por defecto de un space

## Inicio rápido

1. Iniciar sesión con una cuenta

```bash
wechat-agent account login
```

2. Listar cuentas guardadas

```bash
wechat-agent account ls
```

3. Crear un space

```bash
wechat-agent space create dev --agent codex
```

4. Asociar la cuenta

```bash
wechat-agent space bind-account dev <account_id>
```

5. Iniciar el space

```bash
wechat-agent space start dev
```

6. Seguir logs

```bash
wechat-agent space logs dev --tail 100 -f
```

## Resumen de comandos

```bash
wechat-agent account login|ls|rm
wechat-agent space create|ls|ps|inspect|start|stop|restart|logs|rm|bind-account|unbind-account
wechat-agent agent ls|switch
wechat-agent bind ls|set|rm
wechat-agent update

# bajo nivel / experimental
wechat-agent run --space <name>
wechat-agent daemon start|status|stop
```

## Guía de comandos

### `account`

Gestiona credenciales locales de WeChat.

```bash
wechat-agent account login
wechat-agent account ls
wechat-agent account rm <account_id>
```

- `login`: inicia login por QR y devuelve `account_id`
- `ls`: lista cuentas, token, usuario y fecha guardada
- `rm`: elimina una cuenta local

### `space`

El comando central para gestión del runtime.

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

- `create`: crea un space; el agente por defecto es `codex`
- `ls` / `ps`: muestran estado, pid, agente, cuenta y número de bindings
- `inspect`: imprime JSON completo con rutas runtime
- `start` / `stop` / `restart`: controlan el proceso en segundo plano
- `logs`: lee o sigue el archivo de logs
- `rm`: borra un space detenido
- `bind-account` / `unbind-account`: asocia o desasocia una cuenta

### `agent`

Lista agentes disponibles o cambia el agente por defecto de un space.

```bash
wechat-agent agent ls
wechat-agent agent switch <space> <agent>
```

Agentes disponibles:
- `claude`
- `codex`
- `openclaw`
- `openai`
- `anthropic`
- `echo`

### `bind`

Enrutamiento por usuario dentro de un mismo space.

```bash
wechat-agent bind ls <space>
wechat-agent bind set <space> <user_id> <agent>
wechat-agent bind rm <space> <user_id>
```

Útil cuando un space usa `codex` por defecto pero un usuario concreto debe usar `claude`.

### `update`

Actualiza un checkout del código fuente.

```bash
wechat-agent update
```

Comportamiento:
- ejecuta `git pull --ff-only`
- ejecuta `cargo build --release --locked`
- imprime la ruta del binario rebuilt

Está pensado para usuarios del código fuente, no para auto-reemplazo binario.

### `daemon` y `run`

Comandos de bajo nivel.

```bash
wechat-agent daemon start
wechat-agent daemon status
wechat-agent daemon stop
wechat-agent run --space <name>
```

- `daemon` es experimental por ahora
- `run --space` ejecuta el runtime en primer plano y normalmente lo usa `space start`

## Modos de agente

### Agentes ACP locales

```bash
wechat-agent space create dev --agent claude
wechat-agent space create dev --agent codex
wechat-agent space create dev --agent openclaw
```

Notas:
- `claude` y `codex` intentan iniciar comandos ACP locales
- en Windows se manejan scripts `.cmd/.bat/.ps1`
- `codex` tiene fallback por CLI

### Agentes en la nube

```bash
OPENAI_API_KEY=... wechat-agent space create openai-space --agent openai
ANTHROPIC_API_KEY=... wechat-agent space create anthropic-space --agent anthropic
```

## Flujos comunes

### Una cuenta, un space

```bash
wechat-agent account login
wechat-agent account ls
wechat-agent space create dev --agent codex
wechat-agent space bind-account dev <account_id>
wechat-agent space start dev
wechat-agent space logs dev -f
```

### Enviar un usuario a otro agente

```bash
wechat-agent bind set dev user@im.wechat claude
wechat-agent bind ls dev
```

### Inspeccionar el estado

```bash
wechat-agent space ls
wechat-agent space inspect dev
```

## Resolución de problemas

- `space has no bound account`
  Primero asocia una cuenta:
  `wechat-agent space bind-account <space> <account_id>`

- `failed to initialize local agent`
  Verifica si existen `npx`, `codex`, `openclaw` u otras dependencias locales

- `session expired (errcode -14)`
  Vuelve a hacer login:
  `wechat-agent account login`

- En Windows aparece `Access is denied` al reemplazar `wechat-agent.exe`
  Hay un proceso anterior ejecutándose; ciérralo antes de recompilar

## Descargas precompiladas

Descarga los paquetes desde Releases:
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

## Contribuir

Issues y pull requests son bienvenidos.

## Licencia

Este proyecto usa licencia MIT. Consulta [LICENSE](./LICENSE).
