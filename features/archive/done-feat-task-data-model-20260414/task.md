# Tasks: feat-agent-task-system

## 子 Feature 拆分

### 子1: feat-task-data-model (S) — 数据模型 + 后端 CRUD

- [x] DB Migration V002: DROP + CREATE tasks 表（含 CHECK 约束、索引）
- [x] DB Migration: 新建 task_dependencies 表（含索引）
- [x] DB Migration: 新建 task_history 表（含索引）
- [x] Rust: 重构 TaskRow struct (扩展字段，id 统一 TEXT/UUID)
- [x] Rust: 扩展 db_helpers.rs Task CRUD 函数
- [x] Rust: 新建 task dependency CRUD 函数
- [x] Rust: 新建 task history 记录函数
- [x] Rust: 新建 src-tauri/src/commands/task.rs (含 cancel_task command)
- [x] Rust: 注册 task commands 到 lib.rs
- [x] TS: 扩展 src/types.ts Task 相关类型 (id: string UUID, creatorId: string 必填)
- [x] TS: 新建 src/lib/ipc.ts task 相关 IPC 封装
- [x] TS: 新建 src/lib/useTasks.ts hook (基础版)

### 子2: feat-task-ui-board (S) — Task 看板 + 列表 UI (可与子3并行)

- [ ] 安装 @dnd-kit 拖拽库
- [ ] 新建 src/components/task/ 目录
- [ ] TaskStatusBadge.tsx — 状态徽章组件
- [ ] TaskCard.tsx — 任务卡片组件
- [ ] TaskCreateModal.tsx — 创建/编辑对话框
- [ ] TaskAssignDropdown.tsx — Agent 分配下拉
- [ ] TaskBoard.tsx — Kanban 看板 (@dnd-kit 拖拽)
- [ ] TaskList.tsx — 列表视图 (含多选批量操作)
- [ ] TaskDetail.tsx — 详情侧边抽屉
- [ ] Sidebar 集成 TASKS 导航入口 (含未完成数红点)
- [ ] MainContent TASKS tab 改造为 Channel Task 视图
- [ ] 全局视图切换 (Board / List)
- [ ] 搜索栏 + 过滤下拉 (status/assignee/priority/channel)

### 子3: feat-task-execution (M) — Task 执行引擎 (可与子2并行)

- [ ] Rust: 新建 src-tauri/src/task_engine/mod.rs
- [ ] Rust: TaskEngine 核心逻辑 (提交/轮询/分配)
- [ ] Rust: agent_busy 按 (agent_id, channel_id) 粒度跟踪
- [ ] Rust: 实时执行 — 注入 Task 上下文到 channel.rs send_message 流程
- [ ] Rust: 异步执行 — 队列 + 后台 poll 线程 (5秒轮询)
- [ ] Rust: AsyncTaskContext 构建 (workspace + task_prompt)
- [ ] Rust: CancellationToken 取消机制
- [ ] Rust: 错误重试逻辑 (MAX_RETRY=2)
- [ ] Rust: Tauri Event 推送 (task://* 事件)
- [ ] TS: useTaskEngine.ts — 执行状态 hook
- [ ] TS: 执行按钮、进度条、取消按钮、结果展示

### 子4: feat-task-conversation-bind (M) — 对话驱动 Task

- [ ] Zone Protocol 注入 Task Suggestion Protocol (<task-suggestions> 格式)
- [ ] Rust: parse_task_suggestions 解析器 (带容错：无 tag/JSON 错误/字段缺失)
- [ ] Rust: task_suggestion 类型消息写入 JSONL
- [ ] Rust: confirm_task_suggestions / dismiss_suggestion commands
- [ ] TS: useTaskSuggestions.ts hook
- [ ] TS: TaskSuggestionCard.tsx — 交互式消息卡片 (确认/编辑/忽略)
- [ ] TS: MainContent 消息渲染适配 task_suggestion 类型
- [ ] Task 详情展示来源消息
- [ ] 消息详情展示关联 Task

### 子5: feat-task-advanced (M) — 高级协作

- [ ] Rust: would_create_cycle DAG 循环依赖检测
- [ ] Rust: 父子任务级联状态规则 (子全 done→父 in_review, 父 cancelled→子级联)
- [ ] 父子任务关系 UI (子任务列表/树)
- [ ] 任务依赖管理 UI (添加/移除依赖, 循环依赖提示)
- [ ] 依赖满足自动解锁逻辑
- [ ] Agent A2A Task 创建 (复用 Sub4 的 <task-suggestions> 协议)
- [ ] TaskHistory 时间线展示
- [ ] 依赖图可视化 (Phase 2 可选)

## Progress Log

| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | 初始设计完成 | 5 个子 Feature 规划，完整技术方案 |
| 2026-04-14 | Review 后修订 | 修复数据模型、补全执行引擎、定义 Task 输出协议、加循环依赖检测、改依赖链为并行 |
