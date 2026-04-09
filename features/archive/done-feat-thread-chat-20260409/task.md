# Tasks: feat-thread-chat

## Task Breakdown

### 1. Rust Backend - Thread 数据模型
- [x] 定义 `Thread` struct（id, agent_id, title, session_id, created_at, updated_at）
- [x] 定义 `ThreadMessage` struct（id, thread_id, role, content, timestamp）
- [x] Thread 存储：在 agent workspace 的 `conversations/` 目录下管理

### 2. Rust Backend - Thread CRUD Commands
- [x] `create_thread` — 创建新 Thread，生成 session_id，关联 agent
- [x] `list_threads` — 列出所有 Thread（含最后消息预览）
- [x] `get_thread` — 获取单个 Thread 详情
- [x] `delete_thread` — 删除 Thread 及其消息
- [x] `send_message` — 发送消息并触发 Runtime execute

### 3. Rust Backend - Runtime 集成
- [x] `send_message` command 内部调用 `ClaudeCodeRuntime::execute()`
- [x] 通过 `app_handle.emit("agent://chunk")` 推送流式事件
- [x] 支持 `--resume` 参数恢复 session

### 4. Frontend - Thread Types & IPC
- [x] 扩展 `src/types.ts` — Thread, ThreadMessageData, ThreadInfo 类型
- [x] 扩展 `src/lib/ipc.ts` — Thread CRUD commands
- [x] 新增 `useThreadChat` hook — 管理 Thread 状态和消息收发

### 5. Frontend - Chat UI 改造
- [x] 改造 MainContent CHAT tab — 接入真实 Runtime
- [x] 流式消息渲染（逐字显示）
- [x] 输入框 placeholder 动态显示 Agent 名称
- [x] 发送/接收状态管理（isThinking, isStreaming）

### 6. Frontend - Thread 列表
- [x] Sidebar Threads 区域接入真实 Thread 数据
- [x] Thread 点击切换 → MainContent 加载对应对话
- [x] 新建 Thread 入口按钮

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-09 | All tasks implemented | Rust backend + Frontend complete, all tests pass |
