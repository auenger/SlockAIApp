# Tasks: feat-task-advanced

## Task List

### T1: Rust — DAG 循环依赖检测 (would_create_cycle)
- [ ] 实现 `would_create_cycle` 函数 (BFS 从 depends_on_id 出发)
- [ ] 在 `add_task_dependency` command 中调用循环检测
- [ ] 循环依赖时返回明确错误信息

### T2: Rust — 父子任务级联状态规则
- [ ] 子任务全部 done → 父任务自动变为 in_review
- [ ] 父任务 cancelled → 所有子任务级联取消
- [ ] 父任务重新打开 (todo) → 子任务保持当前状态（不级联）
- [ ] 级联操作记录到 task_history

### T3: Rust — 依赖满足自动解锁
- [ ] 任务状态变为 done 时检查依赖此任务的其他任务
- [ ] 依赖全部满足 → 解锁 blocked 任务为 todo
- [ ] 推送 task://dependency-met 事件

### T4: Rust — A2A Task 创建支持
- [ ] confirm_task_suggestions 支持 source=agent_created
- [ ] 创建时设置 creator_id 为发起 Agent 的 agent_id
- [ ] 推送 task://assigned 事件给目标 Agent

### T5: TS — 父子任务关系 UI
- [ ] TaskDetail 中显示子任务列表
- [ ] 创建 Task 时可选父任务
- [ ] 子任务状态汇总显示

### T6: TS — 任务依赖管理 UI
- [ ] TaskDetail 中显示依赖列表
- [ ] 添加/移除依赖的交互
- [ ] 循环依赖错误提示

### T7: TS — TaskHistory 时间线展示
- [ ] TaskDetail 底部展示历史时间线
- [ ] get_task_history IPC 调用 + hook
- [ ] 时间线条目渲染（字段变更对比）

## Progress Log

| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Feature started | From parent feat-agent-task-system |
