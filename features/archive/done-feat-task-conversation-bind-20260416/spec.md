# Feature: feat-task-conversation-bind — 对话驱动 Task 生成 + 上下文绑定

## Basic Information

- **ID**: feat-task-conversation-bind
- **Name**: 对话驱动 Task 生成 + 上下文绑定
- **Priority**: 85
- **Size**: M
- **Dependencies**: feat-task-execution
- **Parent**: feat-agent-task-system
- **Children**: none
- **Created**: 2026-04-14

## Description

Agent 在对话中自动识别用户意图，生成 Task 建议并展示为交互式消息卡片。用户确认后创建 Task 并自动绑定对话上下文（Channel/Thread + 来源消息）。Task 与消息双向关联，支持从对话跳转到 Task 详情。

### 核心流程

1. **Zone Protocol 注入**：在 Channel Prompt L2 层注入 Task Suggestion Protocol，指示 Agent 用 `<task-suggestions>` 格式输出建议任务
2. **后端解析**：Agent 响应完成后，后端解析 `<task-suggestions>` 块，创建 task_suggestion 类型消息写入 JSONL
3. **前端交互**：渲染 TaskSuggestionCard 组件，用户可确认/编辑/忽略建议
4. **Task 创建**：确认后创建 Task，绑定来源 Channel/Thread 和消息 ID
5. **双向关联**：Task 详情展示来源消息，消息详情展示关联 Task

## User Value Points

### VP1: 对话中自动生成 Task 建议
Agent 智能识别用户意图中的可执行任务，自动生成结构化建议，无需手动创建。

### VP2: 交互式确认与编辑
用户可以一键确认创建、编辑后再确认、或忽略建议，保持对 Task 创建的控制权。

### VP3: Task 与对话上下文绑定
Task 自动绑定来源 Channel/Thread 和触发消息，实现对话与任务的双向追溯。

---

## Context Analysis

### Reference Code

- `src-tauri/src/commands/channel.rs` — Channel 对话处理，流式输出后需要解析 task suggestions
- `src-tauri/src/context/` — 上下文组装，Zone Protocol L2 层需要注入 Task Suggestion Protocol
- `src-tauri/src/commands/task.rs` — 现有 Task commands，需要新增 confirm/dismiss suggestions
- `src-tauri/src/storage/jsonl.rs` — JSONL 消息存储，需要支持 task_suggestion 消息类型
- `src-tauri/src/storage/db_helpers.rs` — 数据库 helpers，Task 查询需要支持按 source_message_id 查找
- `src/types.ts` — TypeScript 类型，需要新增 TaskSuggestion 相关类型
- `src/lib/ipc.ts` — IPC 封装，需要新增 suggestion 相关调用
- `src/components/MainContent.tsx` — 消息渲染，需要适配 task_suggestion 类型
- `src/components/task/` — Task UI 组件目录

### Related Features

- `feat-task-data-model` (completed) — Task 数据模型，tasks 表含 channel_id, thread_id 字段
- `feat-task-execution` (completed) — Task 执行引擎，依赖其 Task CRUD 和执行能力
- `feat-task-ui-board` (completed) — Task 看板 UI，Task 详情面板需要展示来源消息
- `feat-channel-zone-protocol` (completed) — Zone Protocol 7 层架构，L2 层注入点

---

## Technical Solution

### 一、Task Suggestion Protocol 定义

在 Zone Protocol (L2) 中注入 Task 输出指令：

```markdown
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

### 二、后端实现

#### 2.1 解析器 (parse_task_suggestions)

```rust
/// 从 Agent 响应中提取 <task-suggestions> 块
pub fn parse_task_suggestions(response: &str) -> Result<Vec<SuggestedTask>> {
    // 1. 正则提取 <task-suggestions>...</task-suggestions> 之间的内容
    // 2. JSON parse -> Vec<SuggestedTask>
    // 3. 容错：
    //    - 未找到 tag -> 返回空 vec（正常情况）
    //    - JSON 格式错误 -> log::warn + 返回空 vec（不阻塞对话）
    //    - 字段缺失 -> 使用默认值（priority=3, assignee=null）
    //    - 数组为空 -> 返回空 vec
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#### 2.2 task_suggestion 消息类型

```rust
// 在 JSONL 中写入 task_suggestion 类型消息
// message.content 格式:
// {
//   type: "task_suggestion",
//   suggestions: [...SuggestedTask],
//   status: "pending" | "confirmed" | "dismissed",
//   confirmed_task_ids: Option<Vec<String>>
// }
```

#### 2.3 Tauri Commands

```rust
// 确认 Task 建议：创建 Task + 更新消息状态
#[tauri::command]
pub async fn confirm_task_suggestions(
    app: AppHandle,
    message_id: String,
    channel_id: String,
    selected: Vec<SuggestedTask>,
) -> Result<Vec<Task>, String>

// 忽略 Task 建议：更新消息状态为 dismissed
#[tauri::command]
pub async fn dismiss_task_suggestions(
    app: AppHandle,
    message_id: String,
    channel_id: String,
) -> Result<(), String>
```

### 三、前端实现

#### 3.1 TypeScript 类型

```typescript
interface SuggestedTask {
  title: string;
  description: string;
  priority: number;
  assignee: string | null;
  dependencies: string[];
}

interface TaskSuggestionContent {
  type: 'task_suggestion';
  suggestions: SuggestedTask[];
  status: 'pending' | 'confirmed' | 'dismissed';
  confirmed_task_ids?: string[];
}
```

#### 3.2 IPC 封装

```typescript
// src/lib/ipc.ts
export async function confirmTaskSuggestions(
  messageId: string,
  channelId: string,
  selected: SuggestedTask[]
): Promise<Task[]> { ... }

export async function dismissTaskSuggestions(
  messageId: string,
  channelId: string
): Promise<void> { ... }
```

#### 3.3 useTaskSuggestions Hook

```typescript
// src/lib/useTaskSuggestions.ts
// 监听 task://suggested 事件
// 提供 confirm/dismiss 方法
```

#### 3.4 TaskSuggestionCard 组件

```typescript
// src/components/task/TaskSuggestionCard.tsx
// 交互式消息卡片：
// - 显示建议的 Task 列表（标题、描述、优先级、分配者）
// - 操作按钮：确认创建 / 编辑 / 忽略
// - 确认后状态变化，显示已创建的 Task ID
// - 支持编辑：打开编辑 modal 修改属性后确认
```

#### 3.5 消息渲染适配

- MainContent 消息渲染：识别 task_suggestion 类型，渲染 TaskSuggestionCard
- Task 详情：展示来源消息链接（channel_id + message_id）
- 消息详情：展示关联 Task 列表

### 四、数据库扩展

#### 4.1 tasks 表字段利用

已存在的 tasks 表已包含：
- `channel_id` — 绑定来源 Channel
- `thread_id` — 绑定来源 Thread
- `source` — 设为 'conversation'
- `source_message_id` — 新增字段，记录来源消息 ID

#### 4.2 Migration: 新增 source_message_id

```sql
-- V00x__task_conversation_bind.sql
ALTER TABLE tasks ADD COLUMN source_message_id TEXT DEFAULT '';
```

### 五、完整流程

```
用户在 Channel 发消息 "帮我重构一下登录模块"
    |
消息发送到 Channel command
    |
上下文编排引擎组装 Prompt (含 Task Suggestion Protocol)
    |
Agent 响应中包含 <task-suggestions>[...]</task-suggestions>
    |
后端在 channel.rs 流式输出完成后：
  -> parse_task_suggestions(full_response)
  -> 如果解析到建议 -> 创建 task_suggestion 类型消息写入 JSONL
  -> 推送 task://suggested 事件到前端
    |
前端渲染 TaskSuggestionCard（交互式卡片）
    |
用户点击 "确认创建" / "编辑" / "忽略"
    |
确认 -> 后端创建 Task (source=conversation, source_message_id=xxx)
      + 记录 task_history
    |
推送 task://created 事件
```

---

## Acceptance Criteria (Gherkin)

```gherkin
Feature: 对话驱动 Task 生成

Scenario: Agent 在对话中建议 Task
  Given 用户在 Channel 中发送消息
  And Agent 识别到可执行任务
  When Agent 响应包含 <task-suggestions> 块
  Then 后端解析出建议列表
  And 创建 task_suggestion 类型消息写入 JSONL
  And 推送 task://suggested 事件

Scenario: 用户确认 Task 建议
  Given 对话中显示 TaskSuggestionCard
  And 建议状态为 pending
  When 用户点击 "确认创建"
  Then 创建 Task (source=conversation)
  And Task 绑定 channel_id 和 source_message_id
  And 更新消息状态为 confirmed
  And 推送 task://created 事件

Scenario: 用户编辑后确认 Task 建议
  Given 对话中显示 TaskSuggestionCard
  When 用户修改建议的标题或优先级
  And 点击确认
  Then 使用修改后的属性创建 Task

Scenario: 用户忽略 Task 建议
  Given 对话中显示 TaskSuggestionCard
  When 用户点击 "忽略"
  Then 更新消息状态为 dismissed
  And 不创建 Task

Scenario: Agent 未建议 Task（正常情况）
  Given 用户在 Channel 中发送消息
  And Agent 未识别到可执行任务
  When Agent 响应不包含 <task-suggestions> 块
  Then 不创建 task_suggestion 消息
  And 不影响正常对话

Scenario: Task 建议解析容错
  Given Agent 响应包含格式错误的 <task-suggestions>
  When 后端解析
  Then log::warn 记录错误
  And 不阻塞对话流程
```

---

## Dependency Chain

```
feat-task-execution (completed)
    |
feat-task-conversation-bind (本 feature)
    |
feat-task-advanced (依赖本 feature)
```

---

## Merge Record

- **Completed**: 2026-04-16T10:00:00+08:00
- **Branch**: feature/feat-task-conversation-bind
- **Merge Commit**: c7772a077eeb644937ccc09c7531a374f704197f
- **Feature Commit**: 487d013
- **Archive Tag**: feat-task-conversation-bind-20260416
- **Conflicts**: none
- **Verification**: PASS (98/98 tests, 6/6 Gherkin scenarios)
- **Stats**: 12 files changed, 1057 insertions, started 2026-04-16T09:00:00+08:00
