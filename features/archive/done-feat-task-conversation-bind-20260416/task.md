# Tasks: feat-task-conversation-bind — 对话驱动 Task 生成 + 上下文绑定

## Task List

### T1: Zone Protocol 注入 Task Suggestion Protocol
- [x] 在 context 编排模块的 L2 Zone Protocol 中注入 Task Suggestion Protocol 指令
- [x] 指示 Agent 用 `<task-suggestions>` 格式输出建议任务

### T2: Rust 后端 — 解析器 + 消息写入
- [x] 新建 `src-tauri/src/commands/task_suggestion.rs` 模块
- [x] 实现 `SuggestedTask` struct (title, description, priority, assignee, dependencies)
- [x] 实现 `parse_task_suggestions(response: &str)` 解析器（带容错）
- [x] 在 `channel.rs` 流式输出完成后调用解析器
- [x] 解析到建议时创建 `task_suggestion` 类型消息写入 JSONL
- [x] 推送 `task://suggested` Tauri Event

### T3: Rust 后端 — 确认/忽略 Commands
- [x] 实现 `confirm_task_suggestions` command（创建 Task + 更新消息状态）
- [x] 实现 `dismiss_task_suggestions` command（更新消息状态为 dismissed）
- [x] 注册新 commands 到 `lib.rs`
- [x] Task 创建时设置 source=conversation, source_message_id

### T4: DB Migration — source_message_id
- [x] ~~新增 Migration 添加 `source_message_id` 字段到 tasks 表~~ (已存在于 V004)
- [x] TaskRow struct 和相关 CRUD 已支持 source_message_id

### T5: TypeScript 类型 + IPC
- [x] `src/types.ts` 新增 SuggestedTask, TaskSuggestionContent, TaskSuggestionStatus 类型
- [x] `src/lib/ipc.ts` 新增 confirmTaskSuggestions / dismissTaskSuggestions IPC 封装
- [x] 新建 `src/lib/useTaskSuggestions.ts` hook（监听 task://suggested 事件）

### T6: 前端 — TaskSuggestionCard 组件
- [x] 新建 `src/components/task/TaskSuggestionCard.tsx`
- [x] 显示建议列表（标题、描述、优先级、分配者）
- [x] 确认按钮：调用 confirmTaskSuggestions IPC
- [x] 编辑按钮：打开编辑 modal，修改后确认
- [x] 忽略按钮：调用 dismissTaskSuggestions IPC
- [x] 状态显示：pending/confirmed/dismissed 不同样式

### T7: 前端 — 消息渲染适配
- [x] MainContent 消息渲染中识别 `task_suggestion` 类型
- [x] 渲染 TaskSuggestionCard 而非普通 markdown
- [x] Task 详情面板展示来源消息信息（channel_id + message_id）

## Progress Log

| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Feature started | 从 feat-agent-task-system 拆分 |
| 2026-04-16 | Implementation complete | All 7 tasks done, Rust+TS compile clean |
