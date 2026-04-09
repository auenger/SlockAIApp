# Tasks: feat-agent-status

## Task Breakdown

### 1. Rust Backend - Runtime Status Command
- [ ] 添加 `get_agent_runtime_status` Tauri command，返回每个 agent 对应 runtime 的状态
- [ ] 融合 AgentManager (workspace) + RuntimeRegistry (runtime) 信息

### 2. Frontend - Agent Status Hook
- [ ] 扩展 `useAgentRuntimes` hook，融合 workspace agent 信息
- [ ] 新增 `useAgentStatus` hook，提供 `{ agents, loading, scan }` 接口
- [ ] 自动在 mount 时触发 runtime scan

### 3. Frontend - Sidebar 更新
- [ ] 移除 Sidebar 中的 hardcoded fallback demo agents
- [ ] 使用 `useAgentStatus` 渲染真实 Agent 列表
- [ ] 添加 runtime 状态指示灯（available/not-installed/unhealthy）
- [ ] Agent hover 显示 tooltip（runtime 状态、安装提示）

### 4. Frontend - Agent 选择联动
- [ ] Sidebar Agent 点击 → 设置 selectedAgent state
- [ ] 传递 selectedAgent 到 MainContent
- [ ] MainContent header 更新为选中 Agent 的名称/描述/状态

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
