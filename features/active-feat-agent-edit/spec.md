# Feature: feat-agent-edit Agent 编辑能力

## Basic Information
- **ID**: feat-agent-edit
- **Name**: Agent 编辑能力
- **Priority**: 65
- **Size**: M
- **Dependencies**: [feat-svg-icon-system]
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10

## Description

为 Agent 新增编辑能力，让用户可以修改已创建的 Agent 的属性，包括名称、图标、creature、vibe、描述等。当前 Agent 一旦创建就无法修改，只能删除重建，体验不友好。

需要实现：
- 后端 `updateAgent` IPC 命令
- EditAgentModal 组件
- Agent Profile 页面的编辑入口
- Sidebar 中的编辑快捷入口

## User Value Points

### VP1: Agent 属性编辑
用户可以修改已创建 Agent 的所有可编辑属性：
- 名称 (name)
- 图标 (icon / emoji) — 依赖 feat-svg-icon-system 的 Icon Picker
- Creature（Agent 类型描述）
- Vibe（Agent 性格描述）
- 描述 (description)

### VP2: 编辑入口和交互
提供便捷的编辑入口：
- Agent Profile 页面的编辑按钮
- Sidebar Agent 右键菜单或 hover 操作
- 编辑流程复用 Create Agent 的表单结构，但预填现有数据

## Context Analysis

### Reference Code
- `src/components/agent/CreateAgentModal.tsx` — 创建 Agent 的表单，可参考复用
- `src/lib/ipc.ts` — 当前只有 createAgent/deleteAgent，缺少 updateAgent
- `src-tauri/src/commands/agent.rs` — Rust 端 Agent commands
- `src/types.ts` — Agent 类型定义

### Related Documents
- project-context.md — Agent 数据模型

### Related Features
- feat-svg-icon-system — 提供图标选择组件，本 feature 依赖

## Technical Solution

### 实现方案
- **后端**: Rust 端新增 `update_agent` command，支持部分更新（只更新传入的字段）
- **icon 字段**: 在 AgentIdentity 和 AgentSummary 中新增 `icon: Option<String>` 字段，持久化到 IDENTITY.md
- **前端 IPC**: 新增 `updateAgent(agentId, request)` 函数和 `UpdateAgentRequest` 类型
- **EditAgentModal**: 独立组件（未提取共用 AgentForm），打开时从 `getAgentIdentity` 加载预填数据
- **编辑入口**: Profile 页右上角 Pencil 按钮 + Sidebar agent 项 hover 显示编辑按钮
- **状态同步**: 编辑后调用 `scan()` 刷新全局 agent 列表，App.tsx 通过 useEffect 自动同步 selectedAgent

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望可以编辑已创建的 Agent 的属性，以便在不删除重建的情况下调整 Agent 的名称、图标和配置。

### Scenarios (Given/When/Then)

#### Scenario 1: 从 Agent Profile 页编辑
```gherkin
Given 用户打开某个 Agent 的 Profile 页
When 用户点击编辑按钮
Then 应弹出编辑 Modal，预填当前 Agent 的所有属性
And 用户可以修改任意字段
And 点击保存后 Agent 属性更新
And UI 即时反映修改结果
```

#### Scenario 2: 从 Sidebar 编辑 Agent
```gherkin
Given 用户在 Sidebar 中 hover 或右键某个 Agent
When 用户选择编辑操作
Then 应弹出编辑 Modal，预填该 Agent 的属性
And 编辑保存后 Sidebar 即时更新
```

#### Scenario 3: 修改 Agent 图标
```gherkin
Given 用户在编辑 Modal 中
When 用户点击图标区域打开 Icon Picker
Then 应显示 Icon Picker 组件（来自 feat-svg-icon-system）
And 选择新图标后即时预览
And 保存后 Agent 图标全局更新
```

#### Scenario 4: 取消编辑
```gherkin
Given 用户在编辑 Modal 中修改了一些字段
When 用户点击取消或关闭 Modal
Then 所有修改应被丢弃
And Agent 属性保持不变
```

#### Scenario 5: 编辑表单验证
```gherkin
Given 用户在编辑 Modal 中清空了必填字段（如名称）
When 用户点击保存
Then 应显示验证错误提示
And 不允许保存不合法的数据
```

### UI/Interaction Checkpoints
- 编辑 Modal 打开时预填当前值
- 表单验证实时反馈
- 保存后所有展示 Agent 的位置即时更新（Sidebar、Channel、Thread、Profile）
- Modal 交互流畅（打开、编辑、保存/取消）

### General Checklist
- [ ] 后端 update_agent command 实现
- [ ] 前端 IPC 层新增 updateAgent
- [ ] 编辑 Modal 组件完成
- [ ] 编辑入口覆盖 Profile 和 Sidebar
- [ ] 保存后 UI 状态全局同步更新
