# Tasks: feat-remote-agent-ui
## Task Breakdown
### 1. AgentBadge 通用组件
- [x] 创建 AgentBadge 组件（支持远程标识、状态展示）
- [x] 统一本地/远程 agent 的视觉语言

### 2. Sidebar 改造
- [x] AGENTS 区域展示远程 agents（混合排列）
- [x] 远程 agent 离线状态视觉提示

### 3. Channel 成员选择器改造
- [x] 成员添加列表包含远程 agents
- [x] 添加远程 agent 时校验连接状态
- [x] Channel header 正确展示远程成员

### 4. Thread Agent 选择改造
- [x] Thread agent 选择器包含远程 agents
- [x] 选择远程 agent 时进入远程对话模式
- [x] offline 远程 agent 禁选或提示

### 5. Hook 整合
- [x] 统一 agent 列表 hook（合并本地+远程）
- [x] 远程 agent 状态实时更新

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-17 | Feature created | 等待 feat-remote-agent-model 完成 |
| 2026-04-17 | 全部完成 | 5 tasks implemented, build verified |
