# Feature: feat-remote-agent-integration 远程 Agent 完整融入

## Basic Information
- **ID**: feat-remote-agent-integration
- **Name**: 远程 Agent 完整融入（Bridge Agent → 本地可用）
- **Priority**: 70
- **Size**: L
- **Dependencies**: feat-lan-a2a-bridge, feat-a2a-remote-client
- **Parent**: null
- **Children**: feat-remote-agent-model, feat-remote-agent-ui, feat-remote-agent-chat
- **Created**: 2026-04-17

## Description
用户通过设置页面的 REMOTE CONNECTIONS 建立了到远程 workspace 的连接后，需要将远程 workspace 上的 agents 无缝融入本地 app 的使用体验中——在 Sidebar 可见、可加入 Channel、可 @mention、可 Thread 1:1 对话。

## User Value Points
1. **远程 Agent 发现与代理** — 从已连接的远程 bridge 拉取 agent 列表，在本地创建代理 Agent 实体
2. **远程 Agent UI 融入** — 远程 agents 出现在 Sidebar、Channel 成员选择器、Thread 列表中
3. **远程 Agent 消息通信** — 通过 A2A 协议向远程 agent 发送消息并接收流式响应

## Context Analysis
### Reference Code
- `src-tauri/src/runtime/a2a/remote.rs` — 远程连接管理器
- `src-tauri/src/runtime/a2a/remote_runtime.rs` — 远程 runtime 实现
- `src-tauri/src/workspace/manager.rs` — Agent 管理器
- `src-tauri/src/workspace/identity.rs` — Agent 身份系统
- `src-tauri/src/commands/channel.rs` — Channel 消息处理
- `src/types.ts` — 前端类型定义 (AgentSummary, ConnectionMode)
- `src/components/Sidebar.tsx` — 侧边栏
- `src/components/MainContent.tsx` — 主内容区

### Related Documents
- az-bridge-guide.md — Bridge 使用指南

### Related Features
- feat-lan-a2a-bridge (已完成) — 远程 Workspace 网关
- feat-a2a-remote-client (已完成) — 远程 A2A Client + 连接管理
- fix-settings-remote-ui (pending) — Remote Connections UI 优化

## Technical Solution
<!-- 分子 feature 实现，见各子 feature 的 spec.md -->

## Acceptance Criteria (Gherkin)
### User Story
作为一个 AgentsZone 用户，我想在连接远程 workspace 后，能像使用本地 agents 一样使用远程 agents——在 Sidebar 看到它们、把它们加入 Channel、@mention 它们、和它们 1:1 对话。

### Scenarios (Given/When/Then)
<!-- 见各子 feature 的详细场景 -->

### General Checklist
- [ ] 远程 agents 与本地 agents 在 UI 上有清晰视觉区分
- [ ] 远程 agent 离线时 UI 有明确状态提示
- [ ] 远程 agent 对话错误有友好错误处理
- [ ] 不影响现有本地 agent 功能
