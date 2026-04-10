# Feature: feat-agent-profile-page Agent Profile 真实数据

## Basic Information
- **ID**: feat-agent-profile-page
- **Name**: Agent Profile 页真实数据
- **Priority**: 65
- **Size**: S
- **Dependencies**: feat-agent-workspace-design (completed), feat-agent-status (completed)
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-09

## Description

当前 MainContent.tsx 的 PROFILE tab 全部是硬编码内容（Agent 名称写死为"克劳德"、角色描述固定、Runtime/Model/Machine 信息写死）。需要从后端读取真实的 Agent Identity（IDENTITY.md）、Runtime 状态、Workspace 信息来展示。

## User Value Points

1. **Agent 档案查看** — 用户可在 Profile 页查看选中 Agent 的真实身份、角色设定、Runtime 状态和配置信息
2. **角色设定编辑** — 用户可编辑 Agent 的角色描述（Role），保存到 IDENTITY.md

## Context Analysis

### Reference Code
- `src/components/MainContent.tsx:850-892` — PROFILE tab（当前全部硬编码）
- `src/lib/ipc.ts:98-100` — `getAgentIdentity(agentId)` 已实现
- `src/lib/ipc.ts:106-109` — `getAgentContext(agentId)` 已实现
- `src/lib/useAgentStatus.ts` — runtime status 已可用
- `src-tauri/src/commands/mod.rs:200-215` — `get_agent_identity` 后端命令
- `src-tauri/src/commands/mod.rs:234-263` — `get_agent_context` 后端命令
- `src-tauri/src/workspace/identity.rs` — AgentIdentity 结构
- `src-tauri/src/runtime/registry.rs` — Runtime 信息

### Related Features
- feat-agent-workspace-design (completed) — Agent 身份系统
- feat-agent-status (completed) — Agent 状态

## Technical Solution

1. PROFILE tab 根据 `selectedAgent` 加载真实数据
2. 调用 `getAgentIdentity(agentId)` 获取 Identity 信息
3. 调用 `getAgentRuntimeStatus()` 获取 Runtime 版本和状态
4. 显示：Agent 名称、emoji、creature、vibe、Role 描述
5. 显示：Runtime 类型、版本、安装路径、连接状态
6. 显示：Workspace 路径、创建时间
7. （可选）Role 编辑 → 写回 IDENTITY.md

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我想在 Profile 页查看 Agent 的真实身份信息和运行状态。

### Scenarios (Given/When/Then)

#### Scenario 1: 展示真实 Profile
```gherkin
Given 用户选中了一个 Agent
When 切换到 PROFILE tab
Then 显示该 Agent 的真实名称和 emoji
And 显示真实的 Identity 信息（creature, vibe, role）
And 显示 Runtime 状态（available/not-installed）和版本号
And 显示 Workspace 路径
```

#### Scenario 2: 无 Agent 选中
```gherkin
Given 用户未选中任何 Agent
When 切换到 PROFILE tab
Then 显示 "Select an agent to view profile" 提示
```

#### Scenario 3: 角色描述展示
```gherkin
Given Agent 有 IDENTITY.md 文件
When Profile 页加载
Then 从 Context 中读取并显示角色描述
```

### UI/Interaction Checkpoints
- 保持 brutal-border 风格
- Runtime 状态颜色指示器与 Sidebar 一致
- 加载状态（loading spinner）
- 保持现有两栏配置布局

### General Checklist
- [ ] 移除所有硬编码
- [ ] 数据从后端加载
- [ ] TypeScript 类型正确
