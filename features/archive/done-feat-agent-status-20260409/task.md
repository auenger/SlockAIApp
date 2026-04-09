# Tasks: feat-agent-status

## Task Breakdown

### 1. Rust Backend - Runtime Status Command
- [x] 添加 `get_agent_runtime_status` Tauri command，返回每个 agent 对应 runtime 的状态
- [x] 融合 AgentManager (workspace) + RuntimeRegistry (runtime) 信息

### 2. Frontend - Agent Status Hook
- [x] 扩展 `useAgentRuntimes` hook，融合 workspace agent 信息
- [x] 新增 `useAgentStatus` hook，提供 `{ agents, loading, scan }` 接口
- [x] 自动在 mount 时触发 runtime scan

### 3. Frontend - Sidebar 更新
- [x] 移除 Sidebar 中的 hardcoded fallback demo agents
- [x] 使用 `useAgentStatus` 渲染真实 Agent 列表
- [x] 添加 runtime 状态指示灯（available/not-installed/unhealthy）
- [x] Agent hover 显示 tooltip（runtime 状态、安装提示）

### 4. Frontend - Agent 选择联动
- [x] Sidebar Agent 点击 → 设置 selectedAgent state
- [x] 传递 selectedAgent 到 MainContent
- [x] MainContent header 更新为选中 Agent 的名称/描述/状态

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-09 | Task 1 done | Added `get_agent_runtime_status` command fusing AgentManager + RuntimeRegistry |
| 2026-04-09 | Task 2 done | Created `useAgentStatus` hook with auto-scan on mount, dev fallback |
| 2026-04-09 | Task 3 done | Sidebar uses real agent data with status indicators, removed mock fallback |
| 2026-04-09 | Task 4 done | App manages selectedAgent state, MainContent header updates dynamically |
