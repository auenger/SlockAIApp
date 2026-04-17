# Tasks: feat-remote-agent-model
## Task Breakdown
### 1. Rust 后端 — Agent Sync Commands
- [x] 新增 `sync_remote_agents` command（从 bridge 拉取 + 创建本地代理）
- [x] 新增 `get_remote_agents` command（获取所有远程代理 agents）
- [x] 新增 `refresh_remote_agents` command（刷新指定连接的 agents）
- [x] 实现去重逻辑（相同 agent_id + connection_id 不重复创建）
- [x] 实现 agent 状态同步（随健康检查更新）

### 2. Rust 后端 — 数据存储
- [x] agents 表支持 remote 类型存储（connection_mode, remote_connection_id 字段）
- [x] 远程连接删除时级联清理关联 agents

### 3. 前端 IPC
- [x] `src/lib/ipc.ts` 新增 syncRemoteAgents / getRemoteAgents / refreshRemoteAgents
- [x] 新增 `useRemoteAgents.ts` hook

### 4. 集成
- [x] 远程连接健康检查时触发 agent 同步
- [x] 连接断开时标记远程 agents 为 offline

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-17 | Feature created | 等待开发 |
| 2026-04-17 | 全部实现完成 | 270 测试通过，Rust + TS 代码完成 |
