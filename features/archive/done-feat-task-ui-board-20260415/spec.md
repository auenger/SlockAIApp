# Feature: feat-task-ui-board — Task 看板 + 列表 + 详情 UI

## Basic Information

- **ID**: feat-task-ui-board
- **Name**: Task 看板 + 列表 + 详情 UI（可与执行引擎并行）
- **Priority**: 85
- **Size**: S
- **Dependencies**: feat-task-data-model
- **Parent**: feat-agent-task-system
- **Created**: 2026-04-14

## Merge Record

- **Completed**: 2026-04-15T22:30:00+08:00
- **Merged Branch**: feature/feat-task-ui-board
- **Merge Commit**: 86253e4
- **Feature Commit**: b2e4c58
- **Archive Tag**: feat-task-ui-board-20260415
- **Conflicts**: none
- **Verification**: passed (6/6 Gherkin ACs, 12/12 tasks, build passes)
- **Stats**: 13 files changed, 1940 insertions, 67 deletions

## Description

为 AgentsZone 构建 Task 的可视化界面，包含 Kanban 看板、列表视图、详情面板、创建/编辑对话框，以及与 Sidebar/MainContent 的导航集成。

### 核心目标

1. **Kanban 看板**：按状态分列，支持拖拽改变状态
2. **列表视图**：表格形式，支持多选批量操作
3. **详情面板**：侧边抽屉展示完整 Task 信息
4. **创建/编辑**：对话框支持创建和编辑 Task
5. **导航集成**：Sidebar TASKS 入口 + MainContent TASKS tab

## Technical Solution

### 前端组件

```
src/components/
  task/
    TaskBoard.tsx          — 全局看板视图 (Kanban columns, @dnd-kit)
    TaskList.tsx           — 列表视图 (表格)
    TaskCard.tsx           — 任务卡片 (用于看板和列表)
    TaskDetail.tsx         — 详情侧边抽屉
    TaskCreateModal.tsx    — 创建/编辑 Task 对话框
    TaskAssignDropdown.tsx — Agent 分配下拉
    TaskStatusBadge.tsx    — 状态徽章
```

### Sidebar 集成

```
┌─────────────────┐
│ AGENTS           │
│   Claude   ●     │
│   Codex    ○     │
│ CHANNELS         │
│   #dev-team      │
│ TASKS            │  ← 新增一级导航
│   Board          │
│   List           │
└─────────────────┘
```

### 全局 Task Board (Kanban)

```
┌──────────────────────────────────────────────────────────────────┐
│  TASK BOARD          [Board | List]  [+ New Task]  [Filter]     │
├──────────┬──────────┬──────────┬──────────┬──────────┬──────────┤
│  TODO    │PROGRESS  │ REVIEW   │  DONE    │ BLOCKED  │CANCELLED │
│ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │ ┌──────┐ │          │
│ │Review │ │ │Refact│ │ │Write │ │ │Setup │ │ │DB    │ │          │
│ │PR #42 │ │ │Login │ │ │Tests │ │ │CI/CD │ │ │Migrat│ │          │
│ │@Claude│ │ │@Codex│ │ │@Claude│ │ │@Claude│ │ │@Codex│ │          │
│ │ P1    │ │ │ P2   │ │ │ P3   │ │ │ P4   │ │ │ P1   │ │          │
│ └──────┘ │ └──────┘ │ └──────┘ │ └──────┘ │ └──────┘ │          │
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘
```

### Task Detail 抽屉

```
┌─────────────────────────────┐
│ Task Detail                  │
│                              │
│ Review PR #42               │
│ Status: IN PROGRESS          │
│ Priority: Critical           │
│ Assignee: @Claude            │
│ Channel: #dev-team           │
│ Mode: Realtime               │
│ Created: 2026-04-14 10:30   │
│                              │
│ -- Description --            │
│ Review the pull request     │
│ and check for security...   │
│                              │
│ -- Activity --              │
│  10:30 Created by User      │
│  10:31 Assigned to Claude   │
│  10:32 Status -> In Progress│
│                              │
│ [Execute] [Edit] [Delete]   │
└─────────────────────────────┘
```

### TypeScript 类型 (已在 feat-task-data-model 定义)

```typescript
export type TaskStatus = 'todo' | 'in_progress' | 'in_review' | 'done' | 'blocked' | 'cancelled';
export type TaskPriority = 1 | 2 | 3 | 4 | 5;
export type TaskExecutionMode = 'realtime' | 'async';
export type TaskSource = 'manual' | 'conversation' | 'agent_created' | 'subtask';

export interface Task {
  id: string;
  title: string;
  description: string;
  status: TaskStatus;
  priority: TaskPriority;
  creatorType: 'user' | 'agent';
  creatorId: string;
  assigneeId?: string;
  channelId?: string;
  threadId?: string;
  parentTaskId?: string;
  executionMode: TaskExecutionMode;
  source: TaskSource;
  sourceMessageId?: string;
  result?: string;
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
  assigneeName?: string;
  assigneeEmoji?: string;
  assigneeIcon?: string;
  channelName?: string;
  childTaskCount?: number;
  dependencyCount?: number;
}
```

### IPC 调用 (已在 feat-task-data-model 定义)

```typescript
// src/lib/ipc.ts
export const taskIpc = {
  create: (input: CreateTaskInput) => invoke<Task>('create_task', { input }),
  list: (filters?: TaskFilters) => invoke<Task[]>('list_tasks', filters),
  get: (id: string) => invoke<TaskDetail>('get_task', { taskId: id }),
  update: (id: string, updates: Partial<Task>) => invoke<Task>('update_task', { taskId: id, updates }),
  delete: (id: string) => invoke<void>('delete_task', { taskId: id }),
  updateStatus: (id: string, status: string) => invoke<Task>('update_task_status', { taskId: id, status }),
  assign: (id: string, agentId?: string) => invoke<Task>('assign_task', { taskId: id, agentId }),
};
```

## Acceptance Criteria (Gherkin)

### AC1: 手动创建 Task
```gherkin
Given 用户在全局 Task Board
When 用户点击 "New Task" 按钮
And 填写标题 "Review PR #42" 和描述
And 选择分配给 Agent "Claude"
And 选择执行模式 "realtime"
Then 系统创建 Task 并在看板 Todo 列显示
And 发送 task://created 事件
```

### AC2: 看板拖拽更新状态
```gherkin
Given Task "Review PR #42" 状态为 Todo
When 用户将卡片从 Todo 列拖到 In Progress 列
Then 系统更新 Task 状态为 in_progress
And 记录状态变更到 task_history
And 推送 task://status-changed 事件
```

### AC3: 列表视图与搜索过滤
```gherkin
Given 存在多个 Task
When 用户切换到 List 视图
Then Task 以列表/表格形式展示
When 用户在搜索栏输入 "PR"
Then 过滤显示标题包含 "PR" 的 Task
When 用户选择状态过滤 "in_progress"
Then 只显示进行中的 Task
```

### AC4: Task 详情查看
```gherkin
Given Task "Review PR #42" 存在
When 用户点击 Task 卡片
Then 右侧打开详情抽屉
And 显示完整 Task 信息（标题、描述、状态、优先级、分配者、执行模式等）
```

### AC5: 编辑 Task
```gherkin
Given Task "Review PR #42" 存在
When 用户在详情面板点击 "Edit"
Then 打开编辑对话框，预填充当前值
When 用户修改标题并保存
Then Task 标题更新
And 推送 task://status-changed 事件
```

### AC6: Sidebar 导航集成
```gherkin
Given 用户在应用中
Then Sidebar 显示 TASKS 导航入口
When 存在未完成的 Task
Then TASKS 入口显示未完成数红点
When 用户点击 TASKS
Then 主内容区显示 Task Board
```

## Reference Code

- `src/types.ts:110-115` — 现有 Task interface（已在 feat-task-data-model 扩展）
- `src/components/MainContent.tsx:671-932` — 现有 Task UI 壳（需重做）
- `src/lib/ipc.ts` — 已有 task IPC 封装
- `src/lib/useTasks.ts` — 已有基础 hook

## Merge Record

- **Completed**: 2026-04-15
- **Merged Branch**: feature/feat-task-ui-board
- **Merge Commit**: 1b01a12
- **Archive Tag**: feat-task-ui-board-20260415b
- **Conflicts**: None
- **Verification**: PASS (13/13 tasks, 7/7 scenarios, 0 TS errors)
- **Development Stats**: 2 commits, 1 file changed (useTaskEngine.ts TS fix)
- **Components**: TaskView, TaskBoard, TaskList, TaskDetail, TaskCard, TaskCreateModal, TaskStatusBadge, TaskAssignDropdown
