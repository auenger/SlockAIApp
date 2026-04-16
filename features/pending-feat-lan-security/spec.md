# Feature: feat-lan-security LAN 安全加固

## Basic Information
- **ID**: feat-lan-security
- **Name**: LAN 安全加固 — TLS + 认证（方案C）
- **Priority**: 50
- **Size**: M
- **Dependencies**: feat-lan-mdns-discovery
- **Parent**: null
- **Children**: 
- **Created**: 2026-04-16

## Description

对局域网 A2A 通信进行安全加固，包括：
- TLS 加密传输（防止局域网窃听）
- Token 认证（防止未授权访问）
- 首次配对确认（类似蓝牙配对）

## User Value Points

### V1: 加密通信
局域网内的 A2A 通信通过 TLS 加密，防止中间人窃听。

### V2: 访问控制
通过 Token 认证限制只有授权设备可以连接。首次连接需要配对确认。

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/a2a/push.rs` — 已有 HMAC-SHA256 签名验证
- `src-tauri/src/runtime/a2a/transport.rs` — 已有 Bearer token 支持
- `src-tauri/src/storage/keyring.rs` — 安全凭据存储

### Technical Solution (Draft)
- TLS: `rustls` + 自签名证书（自动生成，pin 指纹验证）
- 认证: 首次连接显示 6 位配对码，对端输入确认
- Token: 配对成功后生成长期 API Key，存储在 Keyring
- 证书: 每个实例生成唯一 CA，信任链通过配对建立

## Acceptance Criteria (Gherkin)

```gherkin
Given 电脑 A 首次连接电脑 B
When A 发起连接请求
Then B 弹出配对确认，显示 6 位配对码
And A 输入配对码后连接建立
And 后续连接自动使用已存储的 Token 认证
And 所有通信通过 TLS 加密
```

## Status: DEFERRED — 等待 feat-lan-mdns-discovery 完成后启动
