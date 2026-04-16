# Feature: feat-lan-mdns-discovery mDNS 零配置自动发现

## Basic Information
- **ID**: feat-lan-mdns-discovery
- **Name**: mDNS 零配置局域网自动发现（方案B）
- **Priority**: 50
- **Size**: M
- **Dependencies**: feat-lan-a2a-access
- **Parent**: null
- **Children**: 
- **Created**: 2026-04-16

## Description

通过 mDNS/DNS-SD (Bonjour/Avahi) 实现局域网内 AgentsZone 实例的零配置自动发现。用户无需手动输入 IP 地址，打开应用即可看到局域网内的其他 Agent。

技术方案：
- Rust 端使用 `mdns-sd` crate 注册/发现服务
- 服务类型：`_a2a._tcp.local.`
- 发现后自动获取 AgentCard，在前端 "LAN Peers" 面板中展示
- 用户点击即可连接

## User Value Points

### V1: 零配置发现
打开应用自动看到局域网内的其他 AgentsZone 实例，无需手动输入任何地址。

### V2: 实时状态感知
LAN Peers 面板实时显示对端的在线/离线状态，Agent 能力信息。

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/a2a/types.rs` — AgentCard
- `src/components/settings/RemoteConnectionsPanel.tsx` — 可扩展为 LAN Peers 面板

### Technical Solution (Draft)
- `mdns-sd` crate 实现服务注册和发现
- 新增 `src-tauri/src/runtime/a2a/discovery.rs`
- 注册：`_a2a._tcp.local.` + port + AgentCard metadata
- 发现：持续监听新 peer，缓存结果，TTL 过期清理
- Tauri Event 通知前端新 peer 出现/消失
- 前端：Settings → LAN Peers 面板

## Acceptance Criteria (Gherkin)

```gherkin
Given 电脑 A 和电脑 B 都运行了 AgentsZone 且开启了 LAN Access
When 电脑 A 打开 LAN Peers 面板
Then 自动显示电脑 B（名称、Agent 类型、能力）
And 点击电脑 B 的条目即可直接连接
And 电脑 B 离线后自动从列表消失
```

## Status: DEFERRED — 等待 feat-lan-a2a-server 完成后启动
