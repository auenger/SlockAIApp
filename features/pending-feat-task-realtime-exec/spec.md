# Feature: feat-task-realtime-exec — Task Realtime 执行（Channel 消息流接入）

## Basic Information
- **ID**: feat-task-realtime-exec
- **Name**: Task Realtime 执行 — Channel 消息流接入
- **Priority**: 70
- **Size**: M
- **Dependencies**: feat-task-channel-selector
- **Parent**: feat-task-exec-runtime
- **Children**: none
- **Created**: 2026-04-16

## Description

实现 Task Realtime 执行模式：将 Task Prompt 注入 Channel 消息流，复用现有的 `send_channel_message` 机制执行，tool use 等中间过程在 Channel 中正常渲染。

### 执行流程

```
用户点击 Execute (realtime mode)
  → Rust TaskEngine.execute_realtime()
    → 校验依赖 + Agent 忙碌状态
    → 标记 Agent busy + Task in_progress
    → emit "task://execute-realtime" (task_id, agent_id, channel_id, task_prompt)
  → 前端收到 "task://execute-realtime"
    → 调用 sendChannelMessage(channel_id, task_prompt)
    → Agent 在 Channel 中处理，tool use 正常渲染
  → Agent 执行完成
    → 调用 on_task_completed(task_id, result)
    → Task 状态 → in_review
```

### 关键设计点

1. **复用 Channel 消息流**：不新建执行通道，直接在 Task 绑定的 Channel 中执行
2. **Tool Use 渲染**：与正常对话逻辑一致，ContentBlocks 渲染保留
3. **结果关联**：Agent 回复完成后，通过 `on_task_completed` 将结果写入 Task

## User Value Points

1. **实时可见执行**：用户在 Channel 中实时看到 Agent 执行 Task 的完整过程
2. **Tool Use 透明**：中间步骤（tool use、文件操作等）完整渲染，与普通对话一致

## Context Analysis

### Reference Code
- `src/lib/useTaskEngine.ts` — 当前只追踪 activeTasks，不触发执行
- `src-tauri/src/task_engine/mod.rs` — `execute_realtime()` emit 事件但无前端处理
- `src-tauri/src/commands/channel.rs` — `send_channel_message()` 已实现完整的消息流
- `src/lib/ipc.ts` — `sendChannelMessage()`, `completeTaskExecution()` IPC 函数
- `src/components/task/TaskView.tsx` — `handleExecute` 调用链

### Related Documents
- 已完成 `feat-task-execution` — TaskEngine 骨架（enqueue/submit/cancel）
- 已完成 `feat-channel-zone-protocol` — Channel Prompt 7层架构
- 已完成 `feat-channel-contentblocks-persist` — Tool 调用过程保留

### Related Features
- `feat-task-channel-selector` (前置依赖)
- `feat-task-async-exec` (并行)

## Technical Solution

### 方案概述

1. **前端：处理 `task://execute-realtime` 事件**
   - 在 `useTaskEngine` 或 TaskView 层添加事件处理
   - 收到事件后调用 `sendChannelMessage(channel_id, task_prompt, userName)`
   - 格式化 task_prompt 包含 Task 标题 + 描述 + 执行指令

2. **Agent 响应完成回调**
   - Channel 消息流中 Agent 完成回复后，需要识别这是 Task 执行
   - 方案：在 task_prompt 中嵌入 task_id 标记，Agent 回复结束后前端触发 `completeTaskExecution(task_id, result)`
   - 或者：Rust 端在 send_channel_message 完成后自动回调 on_task_completed

3. **执行过程渲染**
   - Channel 现有的 message rendering 已支持 tool use、thinking 等
   - 不需要额外 UI，直接复用

### 实现细节

**前端 `useTaskEngine` 扩展**：
```typescript
// 新增回调参数
export function useTaskEngine(
  onStatusChanged?: ...,
  onCompleted?: ...,
  onFailed?: ...,
  onRealtimeExecute?: (data: TaskExecuteRealtimeEvent) => void, // 新增
)

// 在 task://execute-realtime 监听中
await listen<TaskExecuteRealtimeEvent>('task://execute-realtime', (event) => {
  // 更新 activeTasks (已有)
  ...
  // 触发实际执行 (新增)
  onRealtimeExecute?.(event.payload);
});
```

**TaskView 层处理**：
```typescript
const handleRealtimeExecute = useCallback(async (data) => {
  await sendChannelMessage(data.channel_id, data.task_prompt);
  // Agent 开始在 Channel 中执行，tool use 正常渲染
}, []);
```

**结果收集**：
- 方案 A：在 channel.rs 的 agent 回调中检测 task 上下文，自动调用 on_task_completed
- 方案 B：前端监听 channel 的 agent 回复完成事件，手动调用 complete_task_execution
- 推荐方案 A（Rust 端自动处理，减少前端复杂度）

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我想要 Realtime 模式的 Task 在 Channel 中实时执行，以便看到 Agent 的完整执行过程。

### Scenarios

#### Scenario 1: Realtime 执行触发 Channel 消息
```gherkin
Given 存在 Task "实现登录功能" (realtime mode, channel: "开发频道", agent: Claude)
When 用户点击 Execute
Then Task 状态变为 "in_progress"
And "开发频道" Channel 中出现 Task Prompt 消息
And Claude Agent 开始在 Channel 中执行
```

#### Scenario 2: Tool Use 正常渲染
```gherkin
Given Realtime Task 正在执行
When Agent 执行 tool use (如读取文件、运行命令)
Then Channel 消息流中正常渲染 tool call 和 tool result
And 与普通对话的 tool use 渲染一致
```

#### Scenario 3: 执行完成更新 Task
```gherkin
Given Realtime Task 执行中
When Agent 完成执行并回复结果
Then Task 状态更新为 "in_review"
And Task result 字段包含执行结果摘要
And Channel 中可见完整的执行过程和最终回复
```

#### Scenario 4: 执行失败处理
```gherkin
Given Realtime Task 执行中
When Agent 执行出错
Then Task 状态更新为 "blocked"
And Task result 字段包含 "FAILED: {error}" 前缀的错误信息
```

### UI/Interaction Checkpoints
- Task 卡片/看板实时显示执行状态
- Channel 面板自动切换到对应 Channel（或提示用户查看）
- Execute 按钮在执行中变为禁用/Cancel 状态

### General Checklist
- 不影响 Channel 正常对话流程
- 不影响 Task Engine 的 cancel 功能
- 与 Async 模式共享 Agent busy 状态管理
