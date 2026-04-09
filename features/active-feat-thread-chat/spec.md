# Feature: feat-thread-chat Thread: 1对1 Agent 对话

## Basic Information
- **ID**: feat-thread-chat
- **Name**: Thread: 1对1 Agent 对话
- **Priority**: 85
- **Size**: M
- **Dependencies**: feat-agent-status
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-09

## Description
实现 Thread（1对1对话）的完整功能：用户可以与单个 Agent 进行实时对话。包括 Thread 数据模型（Rust 后端 + TypeScript 前端类型）、Thread CRUD 操作、Chat UI 接入 Claude Code Runtime 实现真正的流式对话、Session 管理（创建和恢复）。

这是 AgentsZone 最核心的用户价值 —— 用户第一次能与 Agent 进行真正的对话。

## User Value Points

### VP1: 创建和管理 Thread
用户可以为任意 Agent 创建独立的对话 Thread，查看 Thread 列表，删除不需要的 Thread。

### VP2: 实时流式对话
用户在 Thread 中发送消息后，Agent 通过 Claude Code Runtime 实时流式回复，支持逐字显示。

### VP3: Session 持续性
用户可以在同一个 Thread 中持续对话，利用 Claude Code 的 `--resume` 保持上下文连续。

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/` — ClaudeCodeRuntime 已实现 execute()，支持 stream-json 和 --resume
- `src-tauri/src/runtime/commands.rs` — `runtime_execute`, `runtime_session_start/stop` commands
- `src/components/MainContent.tsx` — CHAT tab 已有基础 UI，使用 mock 响应
- `src/types.ts` — 已有 Message, Thread 类型定义（需扩展）
- `reference/AINative/neuro-syntax-ide/src-tauri/src/lib.rs` — ReqAgent 完整实现参考

### Related Documents
- feat-claude-runtime spec — Runtime 层详细设计
- feat-agent-workspace-design spec — Workspace 目录结构（conversations/ 目录）

### Related Features
- feat-agent-status（前置）— Agent 选择
- feat-conversation-store（后续）— 对话持久化
- feat-channel-infra（后续）— Channel 也需要 chat 能力

## Technical Solution

### Backend (Rust/Tauri)
- **Thread data model** (`src-tauri/src/workspace/thread.rs`): `Thread` struct with id, agent_id, title, session_id, messages, timestamps. `ThreadStore` manages JSON file persistence in agent's `conversations/` directory.
- **Thread commands** (`src-tauri/src/commands/thread.rs`): CRUD commands (`create_thread`, `list_threads`, `get_thread`, `delete_thread`) + `send_message` that integrates with ClaudeCodeRuntime for streaming.
- **Runtime integration**: `send_message` saves user message, calls `ClaudeCodeRuntime::execute()` with session_id for `--resume`, spawns thread to forward `agent://chunk` events to frontend. Emits `agent://thread-response` with accumulated response for persistence via `save_agent_response`.

### Frontend (React/TypeScript)
- **Types** (`src/types.ts`): `Thread`, `ThreadMessageData`, `ThreadInfo` interfaces mirroring Rust structs.
- **IPC** (`src/lib/ipc.ts`): Type-safe wrappers for all Thread Tauri commands.
- **useThreadChat hook** (`src/lib/useThreadChat.ts`): Central state management for active thread, streaming text, isThinking/isStreaming states. Handles Tauri event listeners for `agent://chunk` and `agent://thread-response`.
- **Chat UI** (`src/components/MainContent.tsx`): Replaced mock handleSendMessage with real thread system. Streaming text rendered with animated cursor. Dynamic placeholder with agent name. Disabled send during streaming.
- **Thread list** (`src/components/Sidebar.tsx`): Real thread data from backend with title + preview. Click to switch threads. "+" button for new thread creation.

## Acceptance Criteria (Gherkin)

### User Story
作为 AgentsZone 用户，我希望与选中的 Agent 进行 1对1 实时对话，Agent 能流式回复我的消息。

### Scenarios (Given/When/Then)

#### Scenario 1: 创建 Thread
```gherkin
Given Sidebar 中选中了一个 Agent（如 "克劳德"）
When 用户点击 "New Thread" 或直接在 CHAT tab 开始输入
Then 创建一个新的 Thread，标题为 "Thread with {AgentName}"
And Session ID 被生成并关联到该 Thread
And CHAT tab 显示空白对话界面
```

#### Scenario 2: 发送消息并接收流式回复
```gherkin
Given 用户在一个活跃的 Thread 中
When 用户输入 "你好" 并点击 Send
Then 用户消息立即显示在聊天区域
And Agent 回复通过 Tauri Event 流式推送到前端
And 前端逐字显示 Agent 回复内容
And 回复完成后 Agent 头像旁的 "Thinking..." 消失
```

#### Scenario 3: Session 恢复
```gherkin
Given 用户之前与 Agent "克劳德" 有一个 Thread
When 用户点击该 Thread 继续对话
Then 加载该 Thread 的历史消息
And 新消息使用 --resume 恢复上下文
And Agent 能引用之前的对话内容
```

#### Scenario 4: Agent 处理中状态
```gherkin
Given Agent 正在处理用户消息（streaming 中）
When 用户尝试发送新消息
Then 发送按钮被禁用
And 显示 "Thinking..." 动画
And streaming 完成后发送按钮恢复可用
```

#### Scenario 5: Thread 列表显示
```gherkin
Given 用户有多个 Thread
When 查看 Sidebar Threads 区域
Then 显示所有 Thread 列表，每项显示标题和最后一条消息预览
And 点击 Thread 可切换到对应对话
```

### UI/Interaction Checkpoints
- Chat 输入框 placeholder 变为 "Message @{AgentName}..."
- 消息气泡区分用户（紫色头像）和 Agent（青色头像）
- Agent streaming 时显示逐字输出动画
- Thread 列表在 Sidebar 中可点击切换
- 支持 Enter 发送 / Shift+Enter 换行

### General Checklist
- [x] Thread 数据模型（Rust struct + TS interface）
- [x] Thread CRUD Tauri commands
- [x] Chat UI 接入真实 Runtime
- [x] Tauri Event 流式推送正常
- [x] Session create/resume 机制正常
- [x] 移除 MainContent 中的 mock handleSendMessage
- [x] Thread 列表渲染在 Sidebar
