# Feature: feat-remote-sidebar-panel Sidebar 远程机器 & Agent 概览面板

## Basic Information
- **ID**: feat-remote-sidebar-panel
- **Name**: Sidebar 远程机器 & Agent 概览面板
- **Priority**: 80
- **Size**: S
- **Dependencies**: feat-remote-agent-ui, feat-remote-agent-model
- **Parent**: null
- **Children**: empty
- **Created**: 2026-04-17

## Description
在 Sidebar 顶部 Monitor（电脑）按钮点击时，展开/折叠一个远程概览面板，展示所有远程连接及其 Agent 列表。用户无需切换到设置页面，即可快速查看远程机器状态、Agent 数量和连接健康度。

## User Value Points
1. **快速远程概览** — 一键查看所有远程机器和 Agent，无需导航到设置页

## Context Analysis
### Reference Code
- `src/components/Sidebar.tsx` — Monitor 按钮 (L170), Sidebar 整体结构
- `src/lib/useRemoteConnections.ts` — 远程连接管理 hook
- `src/lib/useRemoteAgents.ts` — 远程 Agent 管理 hook
- `src/lib/useAllAgents.ts` — 统一 Agent 列表 hook
- `src/types.ts` — RemoteConnectionInfo, RemoteAgentCard 类型
- `src/lib/ipc.ts` — remoteConnectionHealthAll, getRemoteAgents 等 IPC

### Related Documents
- project-context.md — 远程 Agent 架构说明

### Related Features
- feat-remote-agent-ui (已完成) — 远程 Agent UI 融入
- feat-remote-agent-model (已完成) — 远程 Agent 代理模型

## Technical Solution

### Implementation Approach
1. **Monitor Toggle**: Added `showRemotePanel` state to Sidebar.tsx. Monitor button uses `onClick` to toggle state, with `bg-brutal-pink text-white` highlight when active.
2. **RemoteOverviewPanel Component**: New component with three sub-components:
   - `ConnectionRow`: Renders a single connection with expand/collapse for agents
   - `EmptyState`: Shown when no remote connections exist
   - Main panel: Groups remote agents by connection ID and renders ConnectionRow list
3. **Data Integration**: Uses `useRemoteConnections` hook (directly in Sidebar) for connection data, plus `isRemoteAgent`/`getConnectionId` utilities to filter and group remote agents from the existing `agents` prop.
4. **Styling**: Neo-Brutalism with brutal-border, status dots (green=#22c55e, red=#ef4444, gray=#9ca3af), Cloud/CloudOff icons for remote agents.

### Files Changed
- `src/components/RemoteOverviewPanel.tsx` (NEW)
- `src/components/Sidebar.tsx` (MODIFIED)

## Acceptance Criteria (Gherkin)
### User Story
作为用户，我希望点击 Sidebar 的 Monitor 按钮就能看到所有远程机器和 Agent，以便快速了解远程基础设施状态。

### Scenarios (Given/When/Then)

#### Scenario 1: 展开远程概览面板
```gherkin
Given Sidebar 显示在界面左侧
And 存在多个远程连接（至少 2 个）
When 用户点击 Monitor（电脑图标）按钮
Then Sidebar 中展开远程概览面板
And 面板显示所有远程连接列表
And 每个连接显示名称、状态指示灯、Agent 数量
And 再次点击 Monitor 按钮面板折叠
```

#### Scenario 2: 查看远程连接下的 Agent
```gherkin
Given 远程概览面板已展开
And 某远程连接有 3 个 Agent
When 用户展开该连接
Then 显示该连接下所有 Agent 的名称和状态
And 远程 Agent 带有视觉区分（云图标或连接名标记）
```

#### Scenario 3: 无远程连接的空状态
```gherkin
Given 当前未配置任何远程连接
When 用户点击 Monitor 按钮展开面板
Then 面板显示空状态提示
And 提示引导用户前往设置页面添加远程连接
```

#### Scenario 4: 连接健康状态展示
```gherkin
Given 远程概览面板已展开
And 存在一个健康连接和一个不健康连接
Then 健康连接显示绿色状态指示灯
And 不健康连接显示红色状态指示灯
And 面板实时反映连接健康状态
```

### UI/Interaction Checkpoints
- Monitor 按钮点击切换面板展开/折叠（toggle 行为）
- 面板位于 Sidebar 内部，Channels/Agents 等区域上方
- 每个连接可折叠展开查看 Agent 列表
- 状态指示灯颜色：绿（healthy）、红（unhealthy）、灰（unknown）
- 空状态有友好提示和引导操作

### General Checklist
- [ ] 面板展开/折叠动画流畅
- [ ] 远程连接状态实时更新
- [ ] 与现有 Sidebar 布局不冲突

## Merge Record

- **Completed:** 2026-04-20T14:30:00+08:00
- **Merged Branch:** feature/feat-remote-sidebar-panel
- **Merge Commit:** 20b30a1
- **Feature Commit:** 7859780
- **Archive Tag:** feat-remote-sidebar-panel-20260420
- **Conflicts:** None
- **Verification:** PASS (4/4 Gherkin scenarios validated)
- **Files Changed:** 2 (1 new, 1 modified)
- **Commits:** 1
- **Duration:** ~30 minutes
