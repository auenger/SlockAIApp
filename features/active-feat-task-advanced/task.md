# Tasks: feat-task-advanced

## Task List

### T1: Rust — DAG 循环依赖检测 (would_create_cycle)
- [x] 实现 `would_create_cycle` 函数 (BFS 从 depends_on_id 出发) — Already implemented in prior feature
- [x] 在 `add_task_dependency` command 中调用循环检测 — Already wired in prior feature
- [x] 循环依赖时返回明确错误信息 — Returns "adding this dependency would create a cycle"

### T2: Rust — 父子任务级联状态规则
- [x] 子任务全部 done → 父任务自动变为 in_review — `check_parent_cascade` in task.rs
- [x] 父任务 cancelled → 所有子任务级联取消 — `cascade_cancel_children` in task.rs
- [x] 父任务重新打开 (todo) → 子任务保持当前状态（不级联） — By design, no cascade on re-open
- [x] 级联操作记录到 task_history — Uses `system:parent-cascade` and `system:parent-cancelled`

### T3: Rust — 依赖满足自动解锁
- [x] 任务状态变为 done 时检查依赖此任务的其他任务 — `check_dependency_unlock` in task.rs
- [x] 依赖全部满足 → 解锁 blocked 任务为 todo — BFS through dependents, checks all deps
- [x] 推送 task://dependency-met 事件 — Emitted when task is unblocked

### T4: Rust — A2A Task 创建支持
- [x] confirm_task_suggestions 支持 source=agent_created — Added `source` and `agent_id` params
- [x] 创建时设置 creator_id 为发起 Agent 的 agent_id — Dynamic `creator_id` from agent_id param
- [x] 推送 task://assigned 事件给目标 Agent — Emitted when source=agent_created

### T5: TS — 父子任务关系 UI
- [x] TaskDetail 中显示子任务列表 — Loads child tasks via `getChildTasks`, renders with status badges
- [x] 创建 Task 时可选父任务 — Added parent task picker in `TaskCreateModal`
- [x] 子任务状态汇总显示 — Shows child task count with individual status badges

### T6: TS — 任务依赖管理 UI
- [x] TaskDetail 中显示依赖列表 — Shows dependencies with task names
- [x] 添加/移除依赖的交互 — Add dependency picker + remove button per dependency
- [x] 循环依赖错误提示 — Displays error from backend when cycle detected

### T7: TS — TaskHistory 时间线展示
- [x] TaskDetail 底部展示历史时间线 — Already existed, enhanced to show 20 entries
- [x] get_task_history IPC 调用 + hook — Already existed
- [x] 时间线条目渲染（字段变更对比） — Shows field, old -> new values, changed_by

## Progress Log

| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Feature started | From parent feat-agent-task-system |
| 2026-04-16 | T1 verified | Already implemented in feat-task-data-model |
| 2026-04-16 | T2 implemented | Parent cascade + child cascade cancel |
| 2026-04-16 | T3 implemented | Dependency auto-unlock on done |
| 2026-04-16 | T4 implemented | A2A task creation via confirm_task_suggestions |
| 2026-04-16 | T5-T7 implemented | Full UI: sub-tasks, dependencies, timeline |
| 2026-04-16 | Added serde rename_all = "camelCase" | For TaskInfo, TaskHistoryInfo, TaskDependencyInfo |
| 2026-04-16 | New IPC commands | get_task_dependencies, get_dependent_tasks, get_child_tasks |
