# Feature: feat-lan-a2a-bridge 独立 A2A Bridge 轻量二进制

## Basic Information
- **ID**: feat-lan-a2a-bridge
- **Name**: 独立 A2A Bridge 轻量二进制（方式3）
- **Priority**: 40
- **Size**: M
- **Dependencies**: feat-lan-a2a-access
- **Parent**: null
- **Children**: 
- **Created**: 2026-04-16

## Description

编译一个独立的轻量 A2A Bridge 二进制，不依赖 Tauri WebView，体积小（几 MB）。适合：
- 给团队成员快速分发：`scp a2a-bridge user@192.168.1.20:~`
- 在无 GUI 的服务器/CI 环境中运行
- 嵌入到其他工具链中

只包含核心模块（server + adapter + claude runtime），约 2000 行代码。

## User Value Points

### V1: 零依赖 A2A 端点
一个独立二进制文件，不需要安装任何运行时，直接运行即可将本机 Claude Code 暴露为 A2A 端点。

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/a2a/` — 核心模块（复用）
- `src-tauri/src/runtime/claude.rs` — Claude Code Runtime
- 需要：独立的 `Cargo.toml`，只引入必要依赖

### Technical Solution (Draft)
- 新增 `src-tauri/bin/a2a-bridge.rs` 或独立 crate `crates/a2a-bridge/`
- 精简依赖：只包含 reqwest, serde, tokio, log
- 不包含 Tauri, WebView, rusqlite 等 GUI/存储依赖
- 编译为静态二进制（musl target）

## Acceptance Criteria (Gherkin)

```gherkin
Given a2a-bridge 二进制已编译
When 运行 "./a2a-bridge --port 7878"
Then 二进制启动并监听 0.0.0.0:7878
And 其他 AgentsZone 实例可通过 HTTP 连接
And 二进制体积 < 10MB
```

## Status: DEFERRED — 等待 feat-lan-a2a-server 完成后评估
