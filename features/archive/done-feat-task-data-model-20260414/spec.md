# Feature: feat-agent-task-system — Agent Task 系统（完整设计）

## Basic Information

- **ID**: feat-agent-task-system
- **Name**: Agent Task 系统（混合型任务管理 + Agent 执行引擎）
- **Priority**: 85
- **Size**: L (拆分为 5 个子 Feature)
- **Dependencies**: fix-channel-state-isolation
- **Parent**: null
- **Children**:
  - feat-task-data-model
  - feat-task-ui-board
  - feat-task-execution
  - feat-task-conversation-bind
  - feat-task-advanced
- **Created**: 2026-04-14

## Description

为 AgentsZone 引入完整的 Agent Task 系统。Task 是 Agent 协作的核心工作单元，支持：

1. **双来源创建**：手动创建 + 对话中自动识别生成
2. **双上下文模式**：绑定 Channel/Thread 的上下文 Task + 独立 Task
3. **双执行模式**：实时对话式执行 + 异步队列式执行
4. **全局统一视图**：Kanban 看板 + 列表 + 过滤
5. **高级能力**：子任务拆分、依赖关系、A2A 任务传递

### 核心设计理念

Task 不是静态的 Todo List，而是 **Agent 执行的工作单元**。用户通过 Task 告诉 Agent 做什么，Agent 通过 Task 报告做了什么。Task 是人机协作的契约。

## User Value Points

### VP1: Task 数据模型与 CRUD（基础设施）
用户可以创建、编辑、删除、分配 Task，Task 有完整的状态流转。

### VP2: Task 看板与全局视图（可视化）
用户在任何位置看到所有 Task，通过 Kanban/列表视图管理，按状态/优先级/分配者过滤。

### VP3: Agent Task 执行引擎（核心能力）
Agent 接收 Task 并执行，支持实时和异步两种模式，执行过程可观测。

### VP4: 对话驱动 Task 生成（智能性）
Agent 在对话中识别用户意图，自动生成 Task 并关联到对话上下文。

### VP5: 高级 Task 协作（扩展能力）
支持子任务、依赖关系、Agent 间任务传递，构建复杂工作流。

---

## Context Analysis

### Reference Code

- `src-tauri/src/storage/db_helpers.rs:324-413` — 现有 TaskRow + CRUD（需重构）
- `src-tauri/src/storage/migrations/V001__initial.sql:47-56` — 现有 tasks 表（需 migration 升级）
- `src/types.ts:110-115` — 现有 Task interface（需扩展）
- `src/components/MainContent.tsx:671-932` — 现有 Task UI 壳（需重做）
- `src-tauri/src/commands/channel.rs` — Channel 对话处理（需集成 Task 生成）
- `src-tauri/src/runtime/mod.rs` — AgentRuntime trait（需扩展 Task 执行能力）

### Related Features

- `feat-channel-zone-protocol` — Channel Prompt 7 层架构，Task 需要集成到 Zone Protocol
- `feat-agent-a2a-trigger` — Agent-to-Agent 触发机制，Task A2A 传递依赖此能力

---

## Technical Solution

### 一、数据模型

#### 1.1 tasks 表（Migration 改造现有表）

> 注意：不创建 `tasks_v2`，而是通过 Migration 在现有 `tasks` 表上 `ALTER` 或 `DROP + CREATE`。
> 项目早期，现有 tasks 表无生产数据，直接重建即可。

```sql
-- Migration: V002__tasks_v2.sql
-- Drop existing minimal tasks table (no production data) and recreate
DROP TABLE IF EXISTS tasks;

CREATE TABLE tasks (
    id              TEXT PRIMARY KEY,          -- UUID
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    status          TEXT NOT NULL DEFAULT 'todo'
                    CHECK(status IN ('todo','in_progress','in_review','done','blocked','cancelled')),
    priority        INTEGER NOT NULL DEFAULT 3 CHECK(priority BETWEEN 1 AND 5),
    creator_type    TEXT NOT NULL DEFAULT 'user',   -- user | agent
    creator_id      TEXT NOT NULL DEFAULT '',        -- user_id 或 agent_id，始终填充便于审计
    assignee_id     TEXT,                           -- agent_id (磁盘目录名，应用层校验)
    channel_id      TEXT,                           -- 绑定的 Channel ID (JSON store，应用层校验)
    thread_id       TEXT,                           -- 绑定的 Thread（可选）
    parent_task_id  TEXT,                           -- 父 Task（可选）
    execution_mode  TEXT NOT NULL DEFAULT 'realtime'
                    CHECK(execution_mode IN ('realtime','async')),
    source          TEXT NOT NULL DEFAULT 'manual'
                    CHECK(source IN ('manual','conversation','agent_created','subtask')),
    source_message_id TEXT,                         -- 来源消息 ID（如果是对话生成）
    result          TEXT,                           -- 执行结果摘要
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT,
    FOREIGN KEY (parent_task_id) REFERENCES tasks(id) ON DELETE SET NULL
);

-- 索引：高频查询字段
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id);
CREATE INDEX idx_tasks_channel ON tasks(channel_id);
CREATE INDEX idx_tasks_parent ON tasks(parent_task_id);
CREATE INDEX idx_tasks_source ON tasks(source);
```

#### 1.2 task_dependencies 表

```sql
CREATE TABLE IF NOT EXISTS task_dependencies (
    task_id         TEXT NOT NULL,
    depends_on_id   TEXT NOT NULL,  -- task_id 依赖 depends_on_id 完成
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (task_id, depends_on_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (depends_on_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_task_deps_depends ON task_dependencies(depends_on_id);
```

#### 1.3 task_history 表（状态变更历史）

```sql
CREATE TABLE IF NOT EXISTS task_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id     TEXT NOT NULL,
    field       TEXT NOT NULL,      -- status | assignee_id | priority | ...
    old_value   TEXT,
    new_value   TEXT,
    changed_by  TEXT NOT NULL,      -- user:{id} | agent:{agent_id}
    changed_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_task_history_task ON task_history(task_id);
```

#### 1.4 TypeScript 类型

```typescript
export type TaskStatus = 'todo' | 'in_progress' | 'in_review' | 'done' | 'blocked' | 'cancelled';
export type TaskPriority = 1 | 2 | 3 | 4 | 5;
export type TaskExecutionMode = 'realtime' | 'async';
export type TaskSource = 'manual' | 'conversation' | 'agent_created' | 'subtask';

export interface Task {
  id: string;                    // UUID (TEXT), 与 DB 一致
  title: string;
  description: string;
  status: TaskStatus;
  priority: TaskPriority;
  creatorType: 'user' | 'agent';
  creatorId: string;            // 始终填充：user_id 或 agent_id
  assigneeId?: string;          // agent_id
  channelId?: string;           // bound channel
  threadId?: string;            // bound thread
  parentTaskId?: string;        // parent task
  executionMode: TaskExecutionMode;
  source: TaskSource;
  sourceMessageId?: string;
  result?: string;
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
  // Joined fields (from queries)
  assigneeName?: string;
  assigneeEmoji?: string;
  assigneeIcon?: string;
  channelName?: string;
  childTaskCount?: number;
  dependencyCount?: number;
}

export interface TaskDependency {
  taskId: string;
  dependsOnId: string;
}

export interface TaskHistoryEntry {
  id: number;
  taskId: string;
  field: string;
  oldValue?: string;
  newValue?: string;
  changedBy: string;
  changedAt: string;
}

export interface CreateTaskInput {
  title: string;
  description?: string;
  priority?: TaskPriority;
  creatorId: string;             // 必填：创建者标识
  assigneeId?: string;
  channelId?: string;
  threadId?: string;
  parentTaskId?: string;
  executionMode?: TaskExecutionMode;
  source?: TaskSource;
  sourceMessageId?: string;
}
```

### 二、Rust 后端架构

#### 2.1 Task 管理 Commands

```rust
// src-tauri/src/commands/task.rs

#[tauri::command]
pub async fn create_task(app: AppHandle, input: CreateTaskInput) -> Result<Task, String>

#[tauri::command]
pub async fn list_tasks(
    app: AppHandle,
    status_filter: Option<String>,
    channel_id: Option<String>,
    assignee_id: Option<String>,
    parent_task_id: Option<String>,
) -> Result<Vec<Task>, String>

#[tauri::command]
pub async fn get_task(app: AppHandle, task_id: String) -> Result<TaskDetail, String>

#[tauri::command]
pub async fn update_task(app: AppHandle, task_id: String, updates: UpdateTaskInput) -> Result<Task, String>

#[tauri::command]
pub async fn delete_task(app: AppHandle, task_id: String) -> Result<(), String>

#[tauri::command]
pub async fn update_task_status(app: AppHandle, task_id: String, status: String) -> Result<Task, String>

#[tauri::command]
pub async fn assign_task(app: AppHandle, task_id: String, agent_id: Option<String>) -> Result<Task, String>

#[tauri::command]
pub async fn execute_task(app: AppHandle, task_id: String) -> Result<(), String>

#[tauri::command]
pub async fn cancel_task(app: AppHandle, task_id: String) -> Result<(), String>

#[tauri::command]
pub async fn add_task_dependency(app: AppHandle, task_id: String, depends_on_id: String) -> Result<(), String>

#[tauri::command]
pub async fn remove_task_dependency(app: AppHandle, task_id: String, depends_on_id: String) -> Result<(), String>

#[tauri::command]
pub async fn get_task_history(app: AppHandle, task_id: String) -> Result<Vec<TaskHistoryEntry>, String>
```

#### 2.2 Task 执行引擎

```rust
// src-tauri/src/task_engine/mod.rs

/// Task 执行引擎 — 管理任务的生命周期和 Agent 分发
///
/// TaskEngine 不直接执行 Agent 调用，而是作为 channel.rs 之上的
/// **Task 状态管理层**。调用关系：
///
///   用户点击"执行 Task"
///       → Tauri command: execute_task()
///       → TaskEngine.submit(task_id, mode)
///       → 如果 realtime: TaskEngine 组装 task prompt 注入 channel.rs 的 send_message 流程
///       → 如果 async:   TaskEngine 入队，由后台 poll 线程分发
///
/// channel.rs 的 send_channel_message 保持不变，TaskEngine 只是额外注入
/// Task 上下文（标题、描述、来源消息）作为 system_prompt 的一部分。
pub struct TaskEngine {
    // 异步任务队列
    async_queue: Arc<Mutex<VecDeque<QueuedTask>>>,
    // 实时执行中的任务
    active_tasks: Arc<DashMap<String, ActiveTask>>,
    // Agent 忙闲状态 — 按 (agent_id, channel_id) 粒度跟踪
    // 同一 Agent 可在不同 Channel 并行执行
    agent_busy: Arc<DashMap<(String, String), bool>>,
    // Tauri AppHandle — 用于 emit 事件和访问 State
    app: AppHandle,
    // 后台轮询线程 handle
    poll_handle: Option<std::thread::JoinHandle<()>>,
}

/// 队列中的异步任务
pub struct QueuedTask {
    pub task_id: String,
    pub priority: u32,
    pub enqueued_at: String,
    pub retry_count: u32,       // 已重试次数
}

/// 活跃任务
pub struct ActiveTask {
    pub task_id: String,
    pub agent_id: String,
    pub channel_id: String,
    pub started_at: String,
    pub mode: TaskExecutionMode,
    pub cancel_token: CancellationToken,
}

/// 取消令牌 — 用于中断正在执行的任务
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self { Self { cancelled: Arc::new(AtomicBool::new(false)) } }
    pub fn cancel(&self) { self.cancelled.store(true, Ordering::SeqCst); }
    pub fn is_cancelled(&self) -> bool { self.cancelled.load(Ordering::SeqCst) }
}

// 最大重试次数
const MAX_RETRY: u32 = 2;

impl TaskEngine {
    /// 提交任务执行
    pub fn submit(&self, task_id: &str, mode: &TaskExecutionMode) -> Result<()>;

    /// 实时执行 — 注入 Task 上下文到 Channel 对话流
    ///
    /// 不绕过 channel.rs，而是在 channel.rs 的 send_message 流程中
    /// 注入额外的 system_prompt 段：
    /// ```
    /// [Task Execution Context]
    /// Task: {title}
    /// Description: {description}
    /// Source: {source_message_summary}
    /// Please execute this task and report results.
    /// ```
    pub fn execute_realtime(&self, task: &Task, channel_id: &str) -> Result<()> {
        // 1. 检查依赖
        if !self.check_dependencies(&task.id)? { return Err("blocked by dependencies".into()); }
        // 2. 标记 active + 设置 cancel token
        // 3. 更新 DB status → in_progress
        // 4. 调用 channel.rs 的 send_message，注入 task context 到 system_prompt
        //    → 复用现有上下文编排 + A2A 链
        // 5. 注册 on_complete 回调
    }

    /// 入队异步执行 — 放入队列，后台线程自动拾取
    pub fn enqueue(&self, task_id: &str) -> Result<()>;

    /// 取消任务执行
    ///
    /// 实时任务：设置 cancel token → channel.rs 检查 token 中断流式输出
    /// 异步任务：从队列移除，状态 → cancelled
    pub fn cancel_task(&self, task_id: &str) -> Result<()>;

    /// Agent 完成任务回调（由 channel.rs 执行完成时调用）
    pub fn on_task_completed(&self, task_id: &str, result: &str) -> Result<()>;

    /// 任务执行失败回调 — 根据 retry 策略决定重试或标记 failed
    pub fn on_task_failed(&self, task_id: &str, error: &str) -> Result<()> {
        // 检查 retry_count < MAX_RETRY → 重新入队
        // 否则 status → failed，emit task://failed
    }

    /// 检查依赖是否满足（所有 depends_on 任务均 done）
    pub fn check_dependencies(&self, task_id: &str) -> Result<bool>;

    /// 后台轮询循环（在独立线程中运行）
    ///
    /// 启动方式：TaskEngine::new() 中 spawn 一个 std::thread，
    /// 每 5 秒轮询一次 async_queue，寻找可分发的任务。
    fn poll_loop(&self) {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            self.poll_and_dispatch().ok();
        }
    }

    /// 单次轮询：找到空闲 Agent + 无依赖阻塞的 Task，分发执行
    pub fn poll_and_dispatch(&self) -> Result<Vec<String>>;
}
```

#### 2.2.1 异步执行 Thread 上下文管理

异步任务在后台创建 Thread 执行时，需要完整的上下文：

```rust
/// 为异步任务构建 Thread 执行上下文
pub struct AsyncTaskContext {
    pub agent_id: String,
    pub workspace: String,       // Agent 的 workspace 目录
    pub task_prompt: String,     // 组装好的任务 prompt
}

impl AsyncTaskContext {
    /// 从 Task 记录构建异步执行上下文
    pub fn from_task(task: &Task) -> Self {
        let mut prompt = String::new();
        prompt.push_str(&format!("[Task Execution]\nTitle: {}\n", task.title));
        if !task.description.is_empty() {
            prompt.push_str(&format!("Description: {}\n", task.description));
        }
        if let Some(ref result_hint) = task.result {
            prompt.push_str(&format!("Expected output: {}\n", result_hint));
        }
        prompt.push_str("\nPlease complete this task. When done, summarize what you did.\n");

        Self {
            agent_id: task.assignee_id.clone().unwrap_or_default(),
            workspace: /* 从 AgentManager 获取 agent workspace */,
            task_prompt: prompt,
        }
    }
}
```

#### 2.2.2 TaskEngine 与 channel.rs 集成流程

```
用户点击 "执行 Task"
    ↓
Tauri command: execute_task(task_id)
    ↓
TaskEngine.execute_realtime(task, channel_id)
    ↓
检查依赖 → 标记 active → 更新 DB status
    ↓
调用 channel.rs: send_channel_message(
    channel_id,
    message: "[Executing Task: {title}]",
    agent_id: task.assignee_id,
    extra_system_prompt: Task context injection  ← 关键：注入 Task 上下文
)
    ↓
channel.rs 走正常的消息发送流程（上下文编排 → runtime.execute → 流式推送）
    ↓
channel.rs 完成回调 → TaskEngine.on_task_completed()
    ↓
更新 DB → emit task://completed
```

#### 2.3 Tauri Events（任务状态推送）

```
task://created        — 任务创建
task://suggested      — Agent 建议创建任务（等待用户确认）
task://status-changed — 状态变更
task://assigned       — 任务分配
task://progress       — 执行进度更新
task://completed      — 任务完成
task://failed         — 任务失败
task://cancelled      — 任务已取消
task://dependency-met — 依赖满足，可以执行
task://retry          — 任务重试
```

### 三、前端架构

#### 3.1 新增 Hooks

```typescript
// src/lib/useTasks.ts — 全局 Task 管理
export function useTasks(filters?: TaskFilters) {
  // 返回: tasks, createTask, updateTask, deleteTask, assignTask, executeTask, cancelTask
}

// src/lib/useTaskSuggestions.ts — 对话中 Task 建议
export function useTaskSuggestions(channelId: string) {
  // confirmSuggestions: 确认创建建议的 Tasks
  // dismissSuggestion: 忽略建议
}

// src/lib/useTaskBoard.ts — 看板视图
export function useTaskBoard() {
  // 按 status 分组: todoTasks, inProgressTasks, inReviewTasks, doneTasks
  // 拖拽排序支持
}

// src/lib/useTaskEngine.ts — 执行引擎状态
export function useTaskEngine() {
  // 实时执行状态、异步队列状态、Agent 忙闲
  // cancelTask: 取消正在执行的任务
}
```

#### 3.2 UI 组件

```
src/components/
  task/
    TaskBoard.tsx          — 全局看板视图 (Kanban columns)
    TaskList.tsx           — 列表视图 (表格)
    TaskCard.tsx           — 任务卡片 (用于看板和列表)
    TaskDetail.tsx         — 任务详情面板 (侧边抽屉)
    TaskCreateModal.tsx    — 创建/编辑 Task 对话框
    TaskAssignDropdown.tsx — Agent 分配下拉
    TaskStatusBadge.tsx    — 状态徽章
    TaskSuggestionCard.tsx — 对话中 Task 建议卡片 (交互式)
    TaskDependencyGraph.tsx — 依赖关系可视化 (Phase 2)
    TaskTimeline.tsx       — 任务历史时间线
```

#### 3.3 导航集成

Sidebar 新增 TASKS 入口，支持切换：
- 全局看板视图 (所有 Task)
- Channel Task 标签 (当前 Channel 的 Task)

MainContent 的 TASKS tab 改造为 Channel 内 Task 视图。

### 四、对话 → Task 生成流程

#### 4.1 输出协议定义 — Task Output Format

在 Zone Protocol (L2) 中注入 Task 输出指令，告诉 Agent 用特定格式返回建议任务：

```markdown
<!-- 注入到 Zone Protocol L2 层 -->

### Task Suggestion Protocol

When you identify actionable tasks from the user's request, output them in the following format:

<task-suggestions>
[
  {
    "title": "Task title",
    "description": "Brief description",
    "priority": 1-5,
    "assignee": "AgentName or null",
    "dependencies": ["title of prerequisite task or null"]
  }
]
</task-suggestions>

Rules:
- Only suggest tasks that are concrete and actionable
- Use priority 1 (critical) to 5 (trivial)
- Each task should be independently completable
- If no actionable tasks are identified, do NOT output this block
- You may output regular text before/after the block
```

#### 4.2 解析容错策略

```rust
/// 从 Agent 响应中提取 <task-suggestions> 块
pub fn parse_task_suggestions(response: &str) -> Result<Vec<SuggestedTask>, ParseError> {
    // 1. 提取 <task-suggestions>...</task-suggestions> 之间的内容
    // 2. JSON parse → Vec<SuggestedTask>
    // 3. 容错：
    //    - 未找到 tag → 返回空 vec（正常情况，Agent 未建议任务）
    //    - JSON 格式错误 → log::warn + 返回空 vec（不阻塞对话）
    //    - 字段缺失 → 使用默认值（priority=3, assignee=null）
    //    - 数组为空 → 返回空 vec
}

#[derive(Debug, Deserialize)]
pub struct SuggestedTask {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_priority")]
    pub priority: u32,
    pub assignee: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}
```

#### 4.3 前端交互式消息组件

suggested_tasks 在对话中渲染为特殊的**交互式消息卡片**，而非普通 markdown。

```typescript
// src/components/task/TaskSuggestionCard.tsx
// 消息类型：task_suggestion（新增 Message.content type）

interface TaskSuggestionMessage {
  type: 'task_suggestion';
  suggestions: SuggestedTask[];
  status: 'pending' | 'confirmed' | 'dismissed';
  confirmedTaskIds?: string[];
}

// 渲染逻辑：
// 1. 后端解析到 suggested_tasks → 创建一条特殊消息写入 JSONL
// 2. 前端渲染时识别 task_suggestion 类型 → 渲染 TaskSuggestionCard
// 3. 用户操作：
//    - "确认创建" → invoke('confirm_task_suggestions', { message_id, selected })
//    - "编辑" → 打开编辑 modal，修改后确认
//    - "忽略" → 标记 dismissed，不再显示操作按钮
```

#### 4.4 完整流程

```
用户在 Channel 发消息 "帮我重构一下登录模块"
    ↓
消息发送到 Channel command
    ↓
上下文编排引擎组装 Prompt
    ↓
Zone Protocol (L2) 中已注入 Task Suggestion Protocol
    ↓
Agent 响应中包含 <task-suggestions>[...]</task-suggestions>
    ↓
后端在 channel.rs 的流式输出完成后：
  → parse_task_suggestions(full_response)
  → 如果解析到建议 → 写入 task_suggestion 类型消息到 JSONL
  → 推送 task://suggested 事件到前端
    ↓
前端渲染 TaskSuggestionCard（交互式卡片）
    ↓
用户点击 "确认创建" / "编辑" / "忽略"
    ↓
确认 → 后端创建 Task + 记录 task_history
    ↓
推送 task://created 事件
```

### 五、Agent Task 执行流程

#### 5.1 实时执行

```
用户点击 Task 的 "执行" 按钮
    ↓
任务状态 → in_progress，推送事件
    ↓
如果 Task 绑定 Channel → 在 Channel 中创建 Agent 消息
  "开始执行 Task: {title}"
    ↓
编排上下文 (summary + recent + task context)
    ↓
调用 AgentRuntime.execute() 执行
    ↓
流式输出到 Channel/Thread
    ↓
执行完成 → 任务状态 → done/in_review
    ↓
更新 task.result，推送 task://completed
```

#### 5.2 异步执行

```
用户创建 Task，execution_mode = async
    ↓
TaskEngine.enqueue(task_id)
    ↓
放入异步队列，状态保持 todo
    ↓
TaskEngine 后台 poll 线程每 5 秒轮询
    ↓
找到空闲 Agent + 无依赖阻塞的 Task
    ↓
分配 Agent → 状态 → in_progress
    ↓
构建 AsyncTaskContext（workspace + task_prompt）
    ↓
创建后台 Thread，注入 Task 上下文作为 prompt
    ↓
调用 AgentRuntime.execute() 在 Thread 中执行
    ↓
流式输出到 Thread → 推送 task://progress
    ↓
执行完成 → 更新结果 → 推送 task://completed
执行失败 → retry_count < 3 → 重新入队（推送 task://retry）
           retry_count >= 3 → 状态 → failed（推送 task://failed）
```

---

## Sub-Feature 拆分

### 子 Feature 1: feat-task-data-model (S)

**范围**：数据模型 + 后端 CRUD + IPC Commands

- DB migration：重建 tasks 表（DROP + CREATE，项目早期无生产数据）+ 新建 task_dependencies + task_history
- 所有表加索引，tasks 表加 CHECK 约束，去掉不可用的 FK (agents/channels)
- Rust：重构 TaskRow、CRUD 函数、Tauri commands (含 cancel_task)
- TS：扩展 Task 类型定义（id 统一为 string/UUID）
- IPC：注册所有 task commands 到 lib.rs
- Hook：useTasks() 基础版

**验收**：通过 IPC 调用可以创建、查询、更新、删除、取消 Task

### 子 Feature 2: feat-task-ui-board (S)

**范围**：Task 看板 + 列表 + 详情 UI

- TaskBoard.tsx — Kanban 看板（按状态分列，拖拽库：@dnd-kit）
- TaskList.tsx — 列表视图
- TaskCard.tsx — 任务卡片
- TaskDetail.tsx — 详情侧边抽屉
- TaskCreateModal.tsx — 创建/编辑对话框
- TaskStatusBadge.tsx — 状态徽章
- Sidebar 集成 TASKS 入口（含未完成数红点）
- MainContent TASKS tab 改造
- 搜索栏 + 过滤下拉 + 多选批量操作

**验收**：可以手动创建 Task、看板展示、拖拽改状态、查看详情、搜索过滤

### 子 Feature 3: feat-task-execution (M)

**范围**：Task 执行引擎 + 实时/异步执行

- TaskEngine 核心逻辑（作为 channel.rs 之上的状态管理层）
- agent_busy 按 (agent_id, channel_id) 粒度跟踪
- 实时执行：注入 Task 上下文到 Channel 对话流（复用 channel.rs send_message）
- 异步执行：队列 + 自动分配 + 后台 Thread + AsyncTaskContext 上下文管理
- 后台 poll 线程（每 5 秒轮询）
- 取消机制（CancellationToken）
- 错误重试（MAX_RETRY=2）
- 执行进度推送 (Tauri Events)
- 前端执行状态展示 + 取消按钮

**验收**：Agent 可以接收 Task 并执行（实时+异步），可取消，失败自动重试

### 子 Feature 4: feat-task-conversation-bind (M)

**范围**：对话驱动 Task 生成 + 上下文绑定

- Zone Protocol (L2) 注入 Task Suggestion Protocol（<task-suggestions> 格式）
- Rust：parse_task_suggestions 解析器（带容错）
- 自动创建 task_suggestion 类型消息到 JSONL
- TaskSuggestionCard.tsx — 交互式消息卡片（确认/编辑/忽略）
- useTaskSuggestions hook
- Task 详情中展示来源消息
- 消息详情展示关联 Task

**验收**：对话中可以自动生成 Task 建议，用户确认后创建，Task 和对话消息双向关联

### 子 Feature 5: feat-task-advanced (M)

**范围**：子任务 + 依赖 + A2A 传递

- 父子任务关系 (parent_task_id)
- 任务依赖 (task_dependencies) + **DAG 循环依赖检测**
- 父子任务级联状态规则
- Agent 创建任务给其他 Agent (A2A) — 复用 Sub 4 的 `parse_task_suggestions` 协议
- 依赖满足后自动解锁
- TaskHistory 时间线展示

**验收**：支持复杂任务拆分和多 Agent 任务传递

#### 5.1 循环依赖检测 (DAG 验证)

添加依赖前必须验证不会形成环：

```rust
/// 检查添加 task_id → depends_on_id 依赖后是否形成环
/// 使用 BFS/DFS 从 depends_on_id 出发，看能否回到 task_id
pub fn would_create_cycle(conn: &Connection, task_id: &str, depends_on_id: &str) -> Result<bool> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(depends_on_id.to_string());

    while let Some(current) = queue.pop_front() {
        if current == task_id { return Ok(true); } // 形成环
        if visited.contains(&current) { continue; }
        visited.insert(current.clone());

        // 查找 current 依赖的所有任务
        let deps = get_dependencies(conn, &current)?;
        for dep in deps {
            queue.push_back(dep.depends_on_id);
        }
    }
    Ok(false) // 无环
}
```

#### 5.2 父子任务级联规则

```
子任务全部完成 (done)     → 父任务自动变为 in_review（等待用户确认）
父任务取消 (cancelled)    → 所有子任务级联取消
父任务重新打开 (todo)     → 子任务保持当前状态（不级联重开）
子任务 blocked           → 父任务不受影响（由用户决定）
```

---

## Dependency Chain

```
fix-channel-state-isolation
    ↓
feat-task-data-model (子1)
    ↓               ↓
feat-task-ui-board  feat-task-execution
   (子2)              (子3)
    ↑               ↑
    └─── 并行 ───────┘   ← 子2 和子3 可并行开发
            ↓
feat-task-conversation-bind (子4) ← 依赖子3的执行能力
    ↓
feat-task-advanced (子5) ← 依赖子3+子4
```

> **变更说明**：子3 (执行引擎) 不再依赖子2 (UI)。执行引擎是纯后端逻辑，
> 应与前端 UI 并行开发。两者完成后联调集成。

---

## Cross-cutting Concerns

### C1: Token 预算管理

Task 上下文注入 prompt 时需控制 token 用量：

```
Task 上下文注入 token 预算：≤ 500 tokens
- title:        直接注入
- description:  截断至 200 tokens
- source_msg:   仅摘要（用现有 summary 机制）
- result:       仅在 review 状态下注入，截断至 100 tokens
```

注入位置：system_prompt 末尾的 `[Task Execution Context]` 段。Task 上下文计入
Channel 滑动窗口 + 自动摘要的 token 预算管理中，不额外占用。

### C2: 通知机制

异步 Task 完成时，用户可能不在 Task Board 页面：

- Task 完成时 emit `task://completed` → 前端 Sidebar TASKS 入口显示小红点
- Task 失败时 emit `task://failed` → 同上 + 可选系统通知 (Tauri notification)
- 依赖满足时 emit `task://dependency-met` → 仅在 Task Detail 中高亮

### C3: 批量操作

Task 列表和看板支持多选：
- 批量修改状态（选中多个 → 拖到目标列 / 右键菜单）
- 批量删除
- 批量分配 Agent

### C4: 搜索与过滤

Task 列表和看板支持：
- 全文搜索：标题 + 描述（SQLite FTS5 或 LIKE）
- 过滤：status / assignee / priority / channel / source
- 排序：created_at / priority / updated_at

---

## Acceptance Criteria (Gherkin)

### US1: 手动创建独立 Task
```gherkin
Given 用户在全局 Task Board
When 用户点击 "New Task" 按钮
And 填写标题 "Review PR #42" 和描述
And 选择分配给 Agent "Claude"
And 选择执行模式 "realtime"
Then 系统创建 Task 并在看板 Todo 列显示
And 发送 task://created 事件
```

### US2: 看板拖拽更新状态
```gherkin
Given Task "Review PR #42" 状态为 Todo
When 用户将卡片从 Todo 列拖到 In Progress 列
Then 系统更新 Task 状态为 in_progress
And 记录状态变更到 task_history
And 推送 task://status-changed 事件
```

### US3: 实时执行 Task
```gherkin
Given Task "Review PR #42" 已分配给 Claude，状态 Todo
When 用户点击 "Execute" 按钮
Then Task 状态变为 in_progress
And 在绑定的 Channel 中创建执行消息
And Agent 开始执行，流式输出过程
When Agent 完成执行
Then Task 状态变为 in_review
And Task.result 记录执行结果
```

### US3b: 取消执行中的 Task
```gherkin
Given Task "Review PR #42" 正在实时执行 (in_progress)
When 用户点击 "Cancel" 按钮
Then Task 状态变为 cancelled
And Agent 流式输出被中断
And 推送 task://cancelled 事件
```

### US3c: 异步任务执行失败重试
```gherkin
Given 异步 Task "Run tests" 执行失败
And retry_count < MAX_RETRY (2)
Then 系统自动重新入队
And 推送 task://retry 事件
When retry_count >= MAX_RETRY
Then Task 状态变为 failed
And 推送 task://failed 事件
```

### US4: 对话中自动生成 Task
```gherkin
Given 用户在 Channel "dev-team" 中
When 用户发送 "帮我重构登录模块，先写测试再改代码"
And Agent Claude 识别出 2 个可执行步骤
Then 系统在对话中展示 "Claude 建议创建 2 个 Task"
And 用户点击 "确认创建"
Then 系统创建 2 个 Task，绑定到当前 Channel
And Task 按 Agent 建议的顺序建立依赖关系
```

### US5: 异步队列执行
```gherkin
Given Task "Run all tests" 执行模式为 async
And Agent Claude 当前正在执行另一个 Task
When 系统创建此 Task
Then Task 进入异步队列，状态保持 todo
When Claude 完成当前 Task
Then 系统自动将队列中的 Task 分配给 Claude
And Task 状态变为 in_progress
```

### US6: 子任务和依赖
```gherkin
Given Task "重构登录模块" 存在
When 用户创建子 Task "编写单元测试"
And 设置依赖 "重构登录模块" depends_on "编写单元测试"
Then "重构登录模块" 状态变为 blocked
When "编写单元测试" 完成
Then 系统检查依赖已满足
And "重构登录模块" 状态恢复为 todo
And 推送 task://dependency-met 事件
```

### US6b: 循环依赖拒绝
```gherkin
Given Task A depends_on Task B
When 用户尝试设置 Task B depends_on Task A
Then 系统拒绝此操作并提示 "会产生循环依赖"
And 依赖关系不变
```

### US6c: 父子任务级联
```gherkin
Given Task "重构" 有 2 个子任务
When 两个子任务均变为 done
Then "重构" 自动变为 in_review
When 用户取消 "重构"
Then 两个子任务级联变为 cancelled
```

### US7: A2A 任务传递
```gherkin
Given Agent Claude 在 Channel 中执行 Task
When Claude 在响应中输出 <task-suggestions> 包含分配给 Codex 的任务
And 用户确认创建
Then 新 Task 创建，source=agent_created，creator_id=claude
And Codex 收到 task://assigned 通知
And 新 Task 绑定到当前 Channel
When Codex 完成 Task
Then Claude 收到 task://completed 通知
And Claude 可以继续其原始 Task
```

---

## UI/Interaction Design

### Sidebar 变更

```
┌─────────────────┐
│ AGENTS           │
│   Claude   ●     │
│   Codex    ○     │
│ CHANNELS         │
│   #dev-team      │
│ TASKS            │  ← 新增一级导航
│   📋 Board       │
│   📝 List        │
│   ⏳ Queue       │
└─────────────────┘
```

### 全局 Task Board (Kanban)

```
┌──────────────────────────────────────────────────────────────────┐
│  TASK BOARD          [Board | List]  [+ New Task]  [Filter ▼]   │
├──────────┬──────────┬──────────┬──────────┬──────────┬──────────┤
│  TODO    │PROGRESS  │ REVIEW   │  DONE    │ BLOCKED  │CANCELLED │
│ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │          │
│ │Review │ │ │Refact│ │ │Write │ │ │Setup │ │ │DB    │ │          │
│ │PR #42 │ │ │Login │ │ │Tests │ │ │CI/CD │ │ │Migrat│ │          │
│ │@Claude│ │ │@Codex│ │ │@Claude│ │ │@Claude│ │ │@Codex│ │          │
│ │🔴 P1  │ │ │🟡 P2 │ │ │🟢 P3  │ │ │⚪ P4 │ │ │🔴 P1  │ │          │
│ └──────┘ │ └──────┘ │ └──────┘ │ └──────┘ │ └──────┘ │          │
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘
```

### Task Detail 抽屉

```
┌─────────────────────────────┐
│ ← Task Detail               │
│                              │
│ Review PR #42               │
│ Status: IN PROGRESS  [▶]    │
│ Priority: 🔴 Critical       │
│ Assignee: @Claude ●         │
│ Channel: #dev-team          │
│ Mode: Realtime              │
│ Created: 2026-04-14 10:30   │
│                              │
│ ── Description ──            │
│ Review the pull request     │
│ and check for security...   │
│                              │
│ ── Sub Tasks (2) ──         │
│  ✅ Read diff               │
│  🔄 Write review comments   │
│                              │
│ ── Dependencies ──          │
│  Blocks: Refactor login     │
│  Blocked by: (none)         │
│                              │
│ ── Activity ──              │
│  10:30 Created by User      │
│  10:31 Assigned to Claude   │
│  10:32 Status → In Progress │
│                              │
│ [Execute] [Edit] [Delete]   │
└─────────────────────────────┘
```

### 对话中 Task 生成提示

```
┌──────────────────────────────────────────────┐
│ 💬 User: 帮我重构登录模块                      │
│                                               │
│ 🤖 Claude: 好的，我建议将这个任务拆分为：       │
│                                               │
│  ┌──────────────────────────────────────┐     │
│  │ ✨ Claude 建议创建 2 个 Task           │     │
│  │                                       │     │
│  │ 1. 编写单元测试 (P1) → Claude          │     │
│  │ 2. 重构登录模块 (P2) → Claude          │     │
│  │    依赖: Task 1                       │     │
│  │                                       │     │
│  │ [✅ 确认创建] [编辑] [忽略]             │     │
│  └──────────────────────────────────────┘     │
└──────────────────────────────────────────────┘
```

---

## Merge Record

- **Completed**: 2026-04-14
- **Merged Branch**: feature/feat-task-data-model
- **Merge Commit**: 08895e6
- **Archive Tag**: feat-task-data-model-20260414
- **Conflicts**: None (clean rebase + merge)
- **Verification**: PASS (cargo check, tsc --noEmit, 11 IPC commands verified)
- **Stats**: 10 files changed, 1567 insertions, 36 deletions
