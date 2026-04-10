# Tasks: feat-activity-log

## Task Breakdown

### 1. 数据模型与存储
- [x] 定义 ActivityLog 数据模型（id, timestamp, type, agent_id, details, workspace_id）
- [x] 设计日志存储方案（JSONL 追加写入）
- [x] 实现日志存储的读写操作

### 2. Rust 后端 Commands
- [x] `log_activity` — 记录一条活动日志
- [x] `list_activities` — 分页查询活动日志（支持按 agent_id 过滤）
- [x] `clear_activities` — 清除活动日志

### 3. 日志埋点
- [x] Agent 创建/删除时记录
- [x] 对话开始/结束时记录
- [x] Skill 变更时记录
- [x] Channel 创建/变更时记录

### 4. Frontend Types & IPC
- [x] 扩展 `src/types.ts` — ActivityLog 类型定义
- [x] 扩展 `src/lib/ipc.ts` — Activity IPC commands
- [x] 新增 `useActivityLog` hook

### 5. Frontend Activity UI
- [x] Activity 时间线列表组件
- [x] 活动类型图标/颜色区分
- [x] 按 Agent 过滤功能
- [x] 集成到 Sidebar 或独立 tab

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开始实现 |
| 2026-04-10 | All tasks completed | Rust backend + React frontend fully implemented |
