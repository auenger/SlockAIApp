# Feature: feat-task-async-exec — Task Async 执行（后台 Runtime + 结果投递）

## Basic Information
- **ID**: feat-task-async-exec
- **Name**: Task Async 执行 — 后台 Runtime 执行 + 结果投递
- **Priority**: 65
- **Size**: M
- **Dependencies**: feat-task-channel-selector
- **Parent**: feat-task-exec-runtime
- **Children**: none
- **Created**: 2026-04-16

## Description

实现 Task Async 执行模式：Rust 端在后台线程中 spawn Agent Runtime 执行 Task，执行完成后调用 `on_task_completed`，并根据是否绑定 Channel 决定结果投递方式。

### 执行流程

```
用户点击 Execute (async mode)
  → Rust TaskEngine.enqueue()
  → Poll 线程 dispatch
    → emit "task://execute-async"
    → spawn 后台线程调用 Agent Runtime
  → 后台线程执行中
    → emit "task://progress" (进度更新)
  → 执行完成
    → task_engine.on_task_completed(task_id, result)
    → if channel_id: sendChannelMessage(channel_id, result)
    → if no channel: result 存入 task.result 字段
```

### 关键设计点

1. **后台 Runtime 执行**：在 Rust 端 spawn 线程直接调用 Agent Runtime（claude.rs / codex.rs）
2. **进度上报**：Runtime 的 stdout 输出通过 `task://progress` 事件实时推送前端
3. **结果投递**：
   - 有 Channel：结果作为消息发送到 Channel
   - 无 Channel：结果存入 Task，在 TaskDetail 中查看

## User Value Points

1. **后台异步执行**：Task 在后台运行，不阻塞 UI
2. **结果自动投递**：执行结果自动发送到 Channel 或在 TaskDetail 中查看

## Context Analysis

### Reference Code
- `src-tauri/src/task_engine/mod.rs` — `poll_and_dispatch_inner` 当前只 emit 事件不执行
- `src-tauri/src/runtime/claude.rs` — Claude Code CLI runtime
- `src-tauri/src/runtime/codex.rs` — Codex CLI runtime
- `src-tauri/src/runtime/registry.rs` — Runtime 注册中心
- `src-tauri/src/commands/channel.rs` — `send_channel_message` 用于结果投递
- `src/components/task/TaskDetail.tsx` — TaskDetail 面板，需显示执行过程

### Related Documents
- 已完成 `feat-task-execution` — TaskEngine 骨架
- 已完成 `feat-agent-runtime-exec` — 多 Runtime 对话执行
- 已完成 `feat-channel-contentblocks-persist` — Tool 调用过程保留

### Related Features
- `feat-task-channel-selector` (前置依赖)
- `feat-task-realtime-exec` (并行)

## Technical Solution

### 方案概述

1. **Rust：`poll_and_dispatch_inner` 改造**
   - Dispatch 后 spawn 一个新线程执行 Agent Runtime
   - 调用 `runtime.execute(task_prompt)` 获取流式输出
   - 实时 emit `task://progress` 事件
   - 执行完成后调用 `self.on_task_completed()` / `self.on_task_failed()`

2. **Runtime 调用**
   ```rust
   // 在 poll_and_dispatch_inner dispatch 后
   let runtime = get_runtime_for_agent(&agent_id)?;
   let task_prompt = build_task_context_prompt(&task);

   std::thread::spawn(move || {
       match runtime.execute(&task_prompt) {
           Ok(result) => task_engine.on_task_completed(&task_id, &result),
           Err(e) => task_engine.on_task_failed(&task_id, &e.to_string()),
       }
   });
   ```

3. **结果投递到 Channel**
   - 在 `on_task_completed` 中检查 `task.channel_id`
   - 如果有 channel：调用 channel 的消息发送逻辑，将结果作为系统消息投递
   - 如果无 channel：结果仅存入 task.result

4. **前端：TaskDetail 显示执行过程**
   - 监听 `task://progress` 事件
   - 在 TaskDetail 中添加 "Execution Log" 区域
   - 显示实时进度文本

### 实现细节

**TaskEngine 需要 Arc<Self>**
当前 `poll_and_dispatch_inner` 是独立函数，无法调用 `self.on_task_completed`。
方案：将 TaskEngine 包装为 `Arc<TaskEngineInner>`，或在 dispatch 时传递回调。

**推荐方案**：将 TaskEngine 改为 `Arc<Mutex<TaskEngineInner>>` 模式：
```rust
pub struct TaskEngine {
    inner: Arc<TaskEngineInner>,
}

// inner 包含所有状态 + app handle
// 这样可以 clone Arc 传到 spawn 的线程中
```

**Runtime 获取**
```rust
// 从 agent_id 获取对应的 runtime
fn get_runtime_for_agent(agent_id: &str) -> Result<Box<dyn AgentRuntime>, String> {
    // 查询 agent 的 runtime_type
    // 从 registry 获取对应 runtime
}
```

**结果投递到 Channel**
```rust
// 在 on_task_completed 中
if let Some(channel_id) = &task.channel_id {
    // 构建结果消息
    let result_msg = format!("[Task Complete] {}\n\n{}", task.title, result);
    // 通过 channel 命令发送
    send_task_result_to_channel(&app, channel_id, &agent_id, &result_msg)?;
}
```

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我想要 Async 模式的 Task 在后台执行，执行完成后结果自动投递到 Channel 或在 TaskDetail 中查看。

### Scenarios

#### Scenario 1: Async 后台执行
```gherkin
Given 存在 Task "重构 API 层" (async mode, agent: Claude, channel: "开发频道")
When 用户点击 Execute
Then Task 入队并开始后台执行
And Task 状态变为 "in_progress"
And 前端显示执行进度指示器
```

#### Scenario 2: 执行进度实时更新
```gherkin
Given Async Task 正在后台执行
When Agent Runtime 输出进度信息
Then TaskDetail 中实时显示进度文本
And 前端通过 task://progress 事件接收更新
```

#### Scenario 3: 有 Channel 的结果投递
```gherkin
Given Async Task "重构 API 层" (channel: "开发频道") 执行完成
When on_task_completed 被调用
Then "开发频道" 中出现结果消息
And Task 状态变为 "in_review"
And Task result 字段包含结果摘要
```

#### Scenario 4: 无 Channel 的结果查看
```gherkin
Given Async Task "独立分析" (无 Channel) 执行完成
When on_task_completed 被调用
Then Task result 字段包含完整执行结果
And TaskDetail 中可查看执行过程和结果
And 不发送任何 Channel 消息
```

#### Scenario 5: 执行失败重试
```gherkin
Given Async Task 执行失败 (retry_count < MAX_RETRY)
When on_task_failed 被调用
Then Task 重新入队等待重试
And emit task://retry 事件
And 重试次数耗尽后标记为 blocked
```

### UI/Interaction Checkpoints
- Task 卡片显示后台执行动画
- TaskDetail 新增 "Execution Log" 区域显示进度
- 有 Channel 时 Channel 列表中对应 Channel 显示新消息 badge

### General Checklist
- 不阻塞 UI 线程
- Agent busy 状态正确管理
- CancellationToken 在 async 模式下工作
- 不影响 Realtime 模式

## Merge Record

- **Completed**: 2026-04-17
- **Merged Branch**: feature/feat-task-async-exec
- **Archive Tag**: feat-task-async-exec-20260417
- **Conflicts**: None
- **Verification**: passed (9/9 tests, 5/5 Gherkin scenarios)
- **Files Changed**: 9 (Rust: 1, React: 5, Docs: 3)
- **Duration**: ~1 session
