# Feature: feat-channel-infra Channel 基础设施

## Basic Information
- **ID**: feat-channel-infra
- **Name**: Channel 基础设施
- **Priority**: 70
- **Size**: M
- **Dependencies**: feat-agent-status, feat-thread-chat
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-09

## Description
实现 Channel 的数据模型、存储和 CRUD 操作。Channel 是多 Agent 协作的容器，可以包含多个 Agent 成员。本 feature 负责 Channel 的创建、管理、Agent 成员关系维护，以及替换 Sidebar 中硬编码的 Channel 列表。

注意：本 feature 只包含 Channel 基础设施和单 Agent 消息，不包含 @Agent mention 和多 Agent 并发响应（这是 feat-channel-multi-agent 的范围）。

## User Value Points

### VP1: 创建和管理 Channel
用户可以创建 Channel（指定名称和参与 Agent），查看 Channel 列表，管理 Channel 设置。

### VP2: Channel 成员管理
用户可以在 Channel 中添加/移除 Agent 成员，形成多 Agent 协作组。

### VP3: Channel 消息收发
用户在 Channel 中发送消息，Channel 中的 Agent 能接收并回复（单 Agent 回复模式）。

## Context Analysis

### Reference Code
- `src/components/Sidebar.tsx` — Channels 区域使用 hardcoded `['all', 'kagent-integrate-sap-ai-core']`
- `src/components/MainContent.tsx` — Chat placeholder 显示 `"Message #kagent-integrate-sap-ai-core"`
- `src/types.ts` — 已有 `Channel` interface（id, name, unreadCount），需扩展

### Related Documents
- feat-agent-workspace-design spec — conversations 目录 `{channel-name}/` 前缀设计

### Related Features
- feat-agent-status（前置）— Agent 信息
- feat-thread-chat（前置）— 复用 Chat UI 和 Runtime 调用模式
- feat-channel-multi-agent（后续）— @Agent mention 和多 Agent 协作

## Technical Solution
<!-- To be filled during implementation -->

## Acceptance Criteria (Gherkin)

### User Story
作为 AgentsZone 用户，我希望创建 Channel 来组织多 Agent 协作，并在 Channel 中与 Agent 成员对话。

### Scenarios (Given/When/Then)

#### Scenario 1: 创建 Channel
```gherkin
Given Sidebar 显示 Channels 区域
When 用户点击 "+" 按钮创建新 Channel
Then 弹出创建 Channel 表单（名称 + 选择 Agent 成员）
And 填写名称 "project-alpha" 并选择 Agent "克劳德" 和 "Alice"
And Channel 创建成功，出现在 Sidebar Channel 列表中
```

#### Scenario 2: Channel 列表展示
```gherkin
Given 用户有多个 Channel
When 查看 Sidebar Channels 区域
Then 显示所有 Channel 列表
And 每个 Channel 显示名称和未读消息数
And 点击 Channel 切换到对应聊天界面
```

#### Scenario 3: Channel 成员管理
```gherkin
Given 用户打开一个 Channel
When 查看 Channel 信息
Then 显示该 Channel 的所有 Agent 成员
And 可以添加新 Agent 成员或移除现有成员
```

#### Scenario 4: Channel 基础消息
```gherkin
Given 用户在一个 Channel 中（包含 Agent "克劳德"）
When 用户发送消息 "请分析一下架构"
Then 消息发送成功，默认 Agent（或第一个成员）进行回复
And 回复以流式方式显示在 Channel 聊天区域
```

### UI/Interaction Checkpoints
- Sidebar Channels 区域使用真实数据（替换 hardcoded）
- Channel 创建按钮 "+" → 弹窗/内联表单
- Channel 选中高亮 → MainContent 切换到 Channel 视图
- Channel header 显示名称和 Agent 成员头像

### General Checklist
- [ ] Channel 数据模型（Rust + TS）
- [ ] Channel CRUD Tauri commands
- [ ] Channel-Agent 成员关系管理
- [ ] Sidebar Channel 列表动态渲染
- [ ] Channel 聊天 UI（复用 Thread Chat 模式）
- [ ] 替换所有 hardcoded channel 引用
