# Feature: feat-sidebar-agent-panel Sidebar 对话按钮活跃 Agent 弹出面板

## Basic Information
- **ID**: feat-sidebar-agent-panel
- **Name**: Sidebar 对话按钮活跃 Agent 弹出面板
- **Priority**: 70
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: none
- **Created**: 2026-04-17

## Description

优化 Sidebar 顶部的 MessageSquare 对话按钮（当前无点击功能），点击后弹出面板展示所有活跃（available 状态）的 Agent 列表。用户点击某个 Agent 即可快速开始 Thread 对话。

## User Value Points

1. **快捷访问活跃 Agent**：一键查看所有在线 Agent，无需滚动到 Sidebar Agents 区域
2. **快速发起对话**：从弹出面板直接点击 Agent 开始 Thread 对话，减少操作步骤

## Context Analysis

### Reference Code
- `src/components/Sidebar.tsx:165-172` — 当前 MessageSquare 按钮，无点击逻辑
- `src/components/Sidebar.tsx:537-650` — 现有 Agent 列表实现
- `src/lib/useAgentStatus.ts` — Agent 运行时状态 hook，提供 `AgentWithRuntime[]`
- `src/lib/useAllAgents.ts` — 统一 Agent 列表（本地 + 远程）
- `src/components/AgentIcon.tsx` — Agent 图标组件

### Related Documents
- CLAUDE.md — 项目架构约定

### Related Features
- feat-remote-agent-ui — 远程 Agent UI 融入（已完成）

## Technical Solution

### 实现方案
1. 为 MessageSquare 按钮添加 `onClick` 状态管理，控制弹出面板的显隐
2. 创建 `ActiveAgentPanel` 组件：
   - 从 `useAgentStatus` 获取 agents 列表
   - 过滤 `runtime_status === 'available'` 的 Agent
   - 复用 `AgentIcon` 组件展示 Agent 头像
   - 点击 Agent 触发 `onAgentSelect` 回调（与现有 Agent 列表共用）
3. 弹出面板使用绝对定位，从按钮下方展开，点击外部关闭
4. 样式遵循 brutal design 风格（brutal-border、brutal-shadow）
5. 按钮增加 active 状态高亮（与当前视图匹配时）

### 关键设计
- 面板宽度与 Sidebar 一致或略宽
- 无活跃 Agent 时显示空状态提示
- 支持 ESC 键关闭面板

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我想通过点击 Sidebar 对话按钮快速查看所有活跃 Agent 并发起对话。

### Scenarios (Given/When/Then)

#### Scenario 1: 点击按钮展示活跃 Agent
```gherkin
Given Sidebar 顶部有 MessageSquare 对话按钮
And 系统中存在 3 个 Agent，其中 2 个状态为 available
When 用户点击 MessageSquare 按钮
Then 弹出面板展示 2 个活跃 Agent
And 每个 Agent 显示头像、名称、运行时状态
And 弹出面板使用 brutal design 风格
```

#### Scenario 2: 从面板快速发起对话
```gherkin
Given 弹出面板已打开，展示活跃 Agent 列表
When 用户点击某个 Agent
Then 面板关闭
And 系统选中该 Agent 并打开 Thread 对话
```

#### Scenario 3: 无活跃 Agent
```gherkin
Given 系统中没有任何 available 状态的 Agent
When 用户点击 MessageSquare 按钮
Then 弹出面板显示空状态提示（如 "No active agents"）
```

#### Scenario 4: 关闭面板
```gherkin
Given 弹出面板已打开
When 用户点击面板外部区域 或 按 ESC 键
Then 弹出面板关闭
```

### UI/Interaction Checkpoints
- MessageSquare 按钮 hover 有视觉反馈
- 按钮 active 时有高亮状态
- 弹出面板从按钮下方平滑展开
- Agent 列表项 hover 有交互反馈

### General Checklist
- [ ] 不影响现有 Agent 列表功能
- [ ] 弹出面板 z-index 正确，不被其他元素遮挡
- [ ] 响应式适配（面板不超出视口）
