# Feature: feat-task-execution — Task 执行引擎（实时 + 异步双模式）

## Basic Information

- **ID**: feat-task-execution
- **Name**: Task 执行引擎（实时 + 异步双模式，可与 UI 并行）
- **Priority**: 85
- **Size**: M
- **Dependencies**: feat-task-data-model
- **Parent**: feat-agent-task-system
- **Children**: none
- **Created**: 2026-04-14

## Description

Task 执行引擎是 Agent Task 系统的核心执行能力层。TaskEngine 作为 channel.rs 之上的 Task 状态管理层，支持两种执行模式：

1. **实时执行 (realtime)**：注入 Task 上下文到 Channel 对话流，复用 channel.rs 的 send_message 流程
2. **异步执行 (async)**：队列 + 后台 poll 线程自动分配 + 后台 Thread 执行

支持取消机制（CancellationToken）、错误重试（MAX_RETRY=2）、执行进度推送（Tauri Events）。

## User Value Points

### VP1: Agent 执行 Task（核心能力）
用户可以为 Task 点击"执行"按钮，Agent 接收 Task 并执行，执行过程可观测。

### VP2: 双模式执行（灵活性）
实时模式适合交互式场景（绑定 Channel），异步模式适合后台任务（自动队列分配）。

### VP3: 执行可观测（透明性）
执行进度、完成结果、失败重试全程通过 Tauri Events 推送到前端。

## Context Analysis

### Reference Code

- `src-tauri/src/commands/task.rs` — Task CRUD commands（feat-task-data-model 已创建）
- `src-tauri/src/storage/db_helpers.rs` — TaskRow + CRUD 函数
- `src-tauri/src/commands/channel.rs` — Channel 对话处理（集成点）
- `src-tauri/src/runtime/mod.rs` — AgentRuntime trait
- `src/types.ts` — TypeScript 类型定义（Task, TaskStatus 等）
- `src/lib/ipc.ts` — IPC 封装

### Dependencies Completed

- `feat-task-data-model` — 数据模型 + 后端 CRUD + IPC Commands 已完成

---

## Technical Solution

### 一、Rust 后端 — TaskEngine 模块

#### 1.1 新建 task_engine 模块

```rust
// src-tauri/src/task_engine/mod.rs

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use dashmap::DashMap;
use tauri::AppHandle;

/// Task 执行引擎 — 管理任务的生命周期和 Agent 分发
pub struct TaskEngine {
    async_queue: Arc<Mutex<VecDeque<QueuedTask>>>,
    active_tasks: Arc<DashMap<String, ActiveTask>>,
    agent_busy: Arc<DashMap<(String, String), bool>>,
    app: AppHandle,
    poll_handle: Option<std::thread::JoinHandle<()>>,
}

pub struct QueuedTask {
    pub task_id: String,
    pub priority: u32,
    pub enqueued_at: String,
    pub retry_count: u32,
}

pub struct ActiveTask {
    pub task_id: String,
    pub agent_id: String,
    pub channel_id: String,
    pub started_at: String,
    pub mode: TaskExecutionMode,
    pub cancel_token: CancellationToken,
}

pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

const MAX_RETRY: u32 = 2;

impl TaskEngine {
    pub fn new(app: AppHandle) -> Self;
    pub fn submit(&self, task_id: &str, mode: &str) -> Result<()>;
    pub fn execute_realtime(&self, task: &Task, channel_id: &str) -> Result<()>;
    pub fn enqueue(&self, task_id: &str) -> Result<()>;
    pub fn cancel_task(&self, task_id: &str) -> Result<()>;
    pub fn on_task_completed(&self, task_id: &str, result: &str) -> Result<()>;
    pub fn on_task_failed(&self, task_id: &str, error: &str) -> Result<()>;
    pub fn check_dependencies(&self, task_id: &str) -> Result<bool>;
    fn poll_loop(&self);
    pub fn poll_and_dispatch(&self) -> Result<Vec<String>>;
    pub fn start_poll_thread(&mut self);
}
```

#### 1.2 实时执行流程

```
用户点击 "执行 Task"
    -> Tauri command: execute_task(task_id)
    -> TaskEngine.execute_realtime(task, channel_id)
    -> 检查依赖 -> 标记 active -> 更新 DB status -> in_progress
    -> 调用 channel.rs: send_channel_message(
        channel_id,
        message: "[Executing Task: {title}]",
        agent_id: task.assignee_id,
        extra_system_prompt: Task context injection
    )
    -> channel.rs 走正常消息发送流程
    -> 完成回调 -> TaskEngine.on_task_completed()
    -> 更新 DB -> emit task://completed
```

#### 1.3 异步执行流程

```
TaskEngine.enqueue(task_id)
    -> 放入异步队列，状态保持 todo
    -> 后台 poll 线程每 5 秒轮询
    -> 找到空闲 Agent + 无依赖阻塞的 Task
    -> 分配 Agent -> 状态 -> in_progress
    -> 构建 AsyncTaskContext
    -> 创建后台 Thread 执行
    -> 完成回调 -> 更新结果 -> emit task://completed
    -> 失败 -> retry_count < MAX_RETRY -> 重新入队
              retry_count >= MAX_RETRY -> status -> failed
```

#### 1.4 AsyncTaskContext

```rust
pub struct AsyncTaskContext {
    pub agent_id: String,
    pub workspace: String,
    pub task_prompt: String,
}

impl AsyncTaskContext {
    pub fn from_task(task: &Task) -> Self {
        let mut prompt = String::new();
        prompt.push_str(&format!("[Task Execution]\nTitle: {}\n", task.title));
        if !task.description.is_empty() {
            prompt.push_str(&format!("Description: {}\n", task.description));
        }
        prompt.push_str("\nPlease complete this task. When done, summarize what you did.\n");
        Self {
            agent_id: task.assignee_id.clone().unwrap_or_default(),
            workspace: /* 从 AgentManager 获取 */,
            task_prompt: prompt,
        }
    }
}
```

#### 1.5 Tauri Events

```
task://status-changed — 状态变更
task://progress       — 执行进度更新
task://completed      — 任务完成
task://failed         — 任务失败
task://cancelled      — 任务已取消
task://retry          — 任务重试
task://dependency-met — 依赖满足
```

### 二、前端 — 执行状态展示

#### 2.1 useTaskEngine.ts Hook

```typescript
export function useTaskEngine() {
  // 实时执行状态、异步队列状态、Agent 忙闲
  // executeTask: 提交任务执行
  // cancelTask: 取消正在执行的任务
  // listen to task://* events
}
```

#### 2.2 执行 UI 组件

- 执行按钮（Task 列表/卡片上的 play 按钮）
- 进度条（执行中状态展示）
- 取消按钮（执行中可取消）
- 结果展示（完成后显示 result）

---

## Acceptance Criteria (Gherkin)

### AC1: 实时执行 Task
```gherkin
Given Task 已分配给 Agent，状态 Todo，绑定 Channel
When 用户点击 "Execute" 按钮
Then Task 状态变为 in_progress
And 在绑定的 Channel 中创建执行消息
And Agent 开始执行，流式输出过程
When Agent 完成执行
Then Task 状态变为 in_review
And Task.result 记录执行结果
And 推送 task://completed 事件
```

### AC2: 取消执行中的 Task
```gherkin
Given Task 正在实时执行 (in_progress)
When 用户点击 "Cancel" 按钮
Then Task 状态变为 cancelled
And Agent 流式输出被中断
And 推送 task://cancelled 事件
```

### AC3: 异步任务执行
```gherkin
Given Task 执行模式为 async
When 系统将 Task 分配给空闲 Agent
Then Task 状态变为 in_progress
And 后台 Thread 执行 Task
When 执行完成
Then Task 状态变为 in_review
And Task.result 记录执行结果
```

### AC4: 异步任务失败重试
```gherkin
Given 异步 Task 执行失败
And retry_count < MAX_RETRY (2)
Then 系统自动重新入队
And 推送 task://retry 事件
When retry_count >= MAX_RETRY
Then Task 状态变为 failed
And 推送 task://failed 事件
```

### AC5: Agent 忙闲状态跟踪
```gherkin
Given Agent Claude 正在 Channel "dev-team" 执行 Task
When 另一个 Task 试图在同一 Channel 分配给 Claude
Then 分配被拒绝（Agent busy）
When Claude 在 Channel "other" 无任务
Then 可以在 Channel "other" 分配给 Claude
```
