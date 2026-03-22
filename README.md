# wechat-rs-sdk

Rust 版本的 WeChat iLink Bot SDK，支持扫码登录、长轮询收发、媒体上传下载（AES-128-ECB）、多账号状态持久化，以及可插拔 Agent 接口。

## 功能覆盖

- QR 扫码登录（`get_bot_qrcode` + `get_qrcode_status`）
- 长轮询接收消息（`getupdates`）
- 文本消息收发（`sendmessage`）
- 图片/视频/文件媒体发送（`getuploadurl` + CDN 上传）
- 图片/视频/文件/语音媒体接收（CDN 下载 + 解密）
- typing 指示（`getconfig` + `sendtyping`）
- `get_updates_buf` 断点续拉落盘
- 会话过期（`errcode = -14`）冷却保护
- Agent 抽象接口，方便接 OpenAI / Claude / 自研模型

## 快速开始

```bash
cd /Volumes/ok/Linux_dev_rewrite/wechat_dev/wechat-rs-sdk
cargo run --example echo -- login
cargo run --example echo
```

## Agent 接口

```rust
#[async_trait::async_trait]
trait Agent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
}
```

`ChatRequest` 包含 `conversation_id`、`text`、可选 `media`（本地已解密路径）。
