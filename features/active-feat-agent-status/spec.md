# Feature: feat-agent-status Agent 状态与选择器

## Basic Information
- **ID**: feat-agent-status
- **Name**: Agent 状态与选择器
- **Priority**: 90
- **Size**: S
- **Dependencies**: 无
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-09

## Description
将 Rust 后端的 Agent Runtime Registry 状态接入前端 UI，实现 Agent 真实在线状态展示和选择功能。用户可以在 Sidebar 中看到每个 Agent 的 runtime 可用性（available/not-installed/unhealthy），并通过选择 Agent 来准备创建 Thread 对话。

核心目标：消除 mock 数据，让 Agent 列表和状态来自真实的 Runtime 扫描结果。

## User Value Points

### VP1: Agent 真实状态展示
用户能看到每个 Agent 对应的 runtime 是否可用（online/offline/error），而不是静态的 mock 状态。

### VP2: Agent 选择交互
用户通过点击 Agent 来选中它，作为后续创建 Thread 对话的前置条件。

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/` — RuntimeRegistry 已实现，支持 scan/detect/health_check
- `src-tauri/src/workspace/manager.rs` — AgentManager 已实现 list/switch
- `src/lib/useAgentRuntimes.ts` — 前端 runtime hook 已存在
- `src/lib/ipc.ts` — IPC bridge 已封装 workspace 和 runtime commands
- `src/components/Sidebar.tsx` — 已有 Agent 列表 UI，使用 workspaceAgents + fallback demo 数据

### Related Features
- feat-claude-runtime（已完成）— Runtime 层基础
- feat-agent-workspace-design（已完成）— Workspace 管理基础
- feat-thread-chat（后续）— 依赖本 feature 的 Agent 选择

## Technical Solution

### Architecture

1. **Backend (Rust)**: Added `get_agent_runtime_status` Tauri command that fuses data from `AgentManager.list_agents()` with `RuntimeRegistry.list_all()`. Returns `Vec<AgentWithRuntime>` combining workspace agent summary with runtime status/version/install_hint.

2. **Frontend Hook**: Created `useAgentStatus` hook that:
   - Calls `scan_agent_runtimes` first to refresh detection data
   - Then calls `get_agent_runtime_status` to get fused agent+runtime data
   - Auto-triggers on mount
   - Provides dev fallback mock data when not in Tauri

3. **Sidebar**: Replaced hardcoded mock agents with real data from `useAgentStatus`. Each agent shows a status indicator dot (green/gray/yellow) based on runtime availability. Hover shows tooltip with status and install hint.

4. **Agent Selection**: App component manages `selectedAgent` state. Clicking an agent in Sidebar passes the full `AgentWithRuntime` object up to App, which forwards it to MainContent. MainContent header dynamically shows the selected agent's name, emoji, status, and version.

### Files Changed
- `src-tauri/src/commands/mod.rs` — Added `AgentWithRuntime` struct and `get_agent_runtime_status` command
- `src-tauri/src/lib.rs` — Registered new command
- `src/types.ts` — Added `AgentWithRuntime` interface
- `src/lib/ipc.ts` — Added `scanAgentRuntimes` and `getAgentRuntimeStatus` functions
- `src/lib/useAgentStatus.ts` — New hook (agent status with auto-scan)
- `src/components/Sidebar.tsx` — Rewritten to use real agent data
- `src/components/MainContent.tsx` — Header updates with selected agent info
- `src/App.tsx` — Added selectedAgent state management

## Acceptance Criteria (Gherkin)

### User Story
作为 AgentsZone 用户，我希望在 Sidebar 中看到每个 Agent 的真实 runtime 状态，并能选择一个 Agent 开始对话。

### Scenarios (Given/When/Then)

#### Scenario 1: Agent 列表展示真实状态
```gherkin
Given Claude Code CLI 已安装且可用
When 用户打开应用 Sidebar
Then Agents 区域显示所有已注册的 Agent
And 每个 Agent 旁显示绿色在线指示灯
And Agent 名称和 emoji 来自 IDENTITY.md
```

#### Scenario 2: Runtime 不可用时状态显示
```gherkin
Given Claude Code CLI 未安装
When 用户打开应用 Sidebar
Then 对应 Agent 旁显示灰色离线指示灯
And Hover 时显示安装提示 "npm install -g @anthropic-ai/claude-code"
```

#### Scenario 3: Agent 选择交互
```gherkin
Given Sidebar 中有多个 Agent
When 用户点击某个 Agent
Then 该 Agent 被选中（高亮显示）
And 其他 Agent 取消选中
And MainContent 区域头部更新为选中 Agent 的信息
```

### UI/Interaction Checkpoints
- Sidebar Agent 列表使用真实数据（不再有 fallback demo agents）
- Agent 状态指示灯：绿色(available)、灰色(not-installed)、黄色(unhealthy)
- 点击 Agent → 高亮选中 → 更新 MainContent header

### General Checklist
- [ ] 移除 Sidebar 中的 mock fallback agents
- [ ] 融合 Workspace Agent 信息与 Runtime 状态
- [ ] Agent 点击事件传递到 MainContent
- [ ] Runtime scan 在应用启动时自动执行
