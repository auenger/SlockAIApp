# Tasks: feat-activity-log

## Task Breakdown

### 1. 数据模型与存储
- [ ] 定义 ActivityLog 数据模型（id, timestamp, type, agent_id, details, workspace_id）
- [ ] 设计日志存储方案（JSONL 追加写入）
- [ ] 实现日志存储的读写操作

### 2. Rust 后端 Commands
- [ ] `log_activity` — 记录一条活动日志
- [ ] `list_activities` — 分页查询活动日志（支持按 agent_id 过滤）
- [ ] `clear_activities` — 清除活动日志

### 3. 日志埋点
- [ ] Agent 创建/删除时记录
- [ ] 对话开始/结束时记录
- [ ] Skill 变更时记录
- [ ] Channel 创建/变更时记录

### 4. Frontend Types & IPC
- [ ] 扩展 `src/types.ts` — ActivityLog 类型定义
- [ ] 扩展 `src/lib/ipc.ts` — Activity IPC commands
- [ ] 新增 `useActivityLog` hook

### 5. Frontend Activity UI
- [ ] Activity 时间线列表组件
- [ ] 活动类型图标/颜色区分
- [ ] 按 Agent 过滤功能
- [ ] 集成到 Sidebar 或独立 tab

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开始实现 |
