# Feature: feat-remote-agent-chat 远程 Agent 消息通信

## Basic Information
- **ID**: feat-remote-agent-chat
- **Name**: 远程 Agent 消息通信（A2A 协议发送 + 流式响应 + @mention 触发）
- **Priority**: 70
- **Size**: S
- **Dependencies**: feat-remote-agent-model, feat-remote-agent-ui
- **Parent**: feat-remote-agent-integration
- **Children**: []
- **Created**: 2026-04-17

## Description
实现远程 Agent 的消息通信能力：通过 A2A 协议向远程 agent 发送消息，接收并流式渲染响应。支持 Channel 中 @mention 远程 agent 触发协作，以及 Thread 1:1 对话模式。这是远程 Agent 融入的执行层。

## User Value Points
1. **Channel @mention 远程 Agent** — 在 Channel 中 @远程agent 触发远程执行，响应流式显示在 Channel 中
2. **Thread 1:1 远程对话** — 在 Thread 面板与远程 agent 进行 1:1 对话

## Context Analysis
### Reference Code
- `src-tauri/src/runtime/a2a/remote_runtime.rs` — RemoteRuntime 已有基础框架
- `src-tauri/src/runtime/a2a/types.rs` — A2A 消息类型定义
- `src-tauri/src/commands/channel.rs` — Channel 消息发送 + @mention 解析
- `src-tauri/src/commands/thread.rs` — Thread 对话命令
- `src-tauri/src/workspace/mention.rs` — @mention 解析器
- `src-tauri/src/runtime/registry.rs` — Runtime 注册中心

### Key Architecture
现有 Channel 消息流程：
1. 用户发消息 → mention 解析 → 确定 target agents
2. 为每个 target agent 构建 prompt（7层架构）
3. 调用对应 runtime 执行 → 流式响应回前端

远程 Agent 需要：
1. mention 解析识别远程 agent → 已有 connection_mode 可判断
2. 构建远程 prompt → 通过 A2A 协议发送
3. 接收远程响应 → 流式回前端（需要 A2A streaming 或 polling）

## Technical Solution

### 1. RemoteRuntime 执行实现
```rust
// src-tauri/src/runtime/a2a/remote_runtime.rs

impl AgentRuntime for RemoteRuntime {
    async fn execute(&self, request: RuntimeRequest) -> Result<RuntimeResponse> {
        // 1. 通过 A2A 协议发送消息到远程 bridge
        // 2. 等待远程 agent 执行结果
        // 3. 流式转发响应
    }
}
```

关键实现点：
- **A2A 消息格式**：构建符合 A2A 协议的 JSON-RPC 请求
- **流式响应**：通过 A2A 的 streaming 或 SSE 接收远程 agent 输出
- **超时处理**：远程执行可能有网络延迟，设置合理超时
- **错误恢复**：网络中断时提供友好错误信息

### 2. Channel @mention 远程 Agent
- 现有 mention 解析器 (`workspace/mention.rs`) 已支持任意 agent
- 执行引擎根据 `connection_mode` 判断使用本地 runtime 还是 remote runtime
- 远程 agent 的响应通过 Tauri event 流式发送到前端
- 响应格式与本地 agent 保持一致（共享 ContentBlocks 渲染）

### 3. Thread 远程对话
- Thread 对话选择远程 agent 时，使用 RemoteRuntime 执行
- 对话持久化到本地 JSONL（与本地 agent 一致）
- 远程 agent 的 context 通过 A2A 协议传递

### 4. Runtime 注册
```rust
// RuntimeRegistry 根据 connection_mode 返回对应 runtime
fn get_runtime(agent: &AgentSummary) -> Box<dyn AgentRuntime> {
    match &agent.connection_mode {
        ConnectionMode::Local => self.local_runtime(),
        ConnectionMode::Remote { connection_id } => {
            self.remote_runtime(connection_id)
        }
    }
}
```

## Acceptance Criteria (Gherkin)
### User Story
作为用户，我希望在 Channel 中 @远程agent 或在 Thread 中与远程 agent 对话时，能像和本地 agent 对话一样获得流式响应。

### Scenarios
```gherkin
Scenario: Channel 中 @mention 远程 agent
  Given Channel "project-x" 包含本地 agent "Alice" 和远程 agent "Bob"
  When 用户发送 "@Bob 帮我分析一下这个远程仓库的代码结构"
  Then 消息通过 A2A 协议发送到远程 bridge
  And 远程 agent "Bob" 的响应流式显示在 Channel 中
  And 响应格式与本地 agent 一致（支持 Markdown、Tool Call 展示）

Scenario: Thread 与远程 agent 1:1 对话
  Given 用户在 Thread 面板选择了远程 agent "Bob"
  When 用户发送消息
  Then 消息通过 A2A 协议发送到远程 bridge
  And 远程响应流式显示在 Thread 面板
  And 对话持久化到本地 JSONL

Scenario: 远程 agent 执行超时处理
  Given 用户 @mention 远程 agent "Bob"
  When 远程 bridge 30 秒未响应
  Then 系统显示超时错误提示
  And 用户可以选择重试或取消

Scenario: 远程 agent 连接断开时的 @mention
  Given Channel 中有远程 agent "Bob"
  When "Bob" 的远程连接已断开
  And 用户 @mention "Bob"
  Then 系统提示 "Bob 当前不可用（远程连接断开）"
  And 不发送消息到远程

Scenario: 多 agent 协作中的远程 agent
  Given Channel "project-x" 包含本地 "Alice" 和远程 "Bob"
  When 用户发送 "@Alice @Bob 一起讨论这个方案"
  Then Alice 和 Bob 并行执行
  And Alice 使用本地 runtime，Bob 使用 remote runtime
  And 两者的响应都流式显示在 Channel 中
```

### General Checklist
- [ ] 远程 agent 响应格式与本地 agent 一致
- [ ] 超时和错误处理完善
- [ ] 流式响应性能可接受
- [ ] 不影响本地 agent 执行性能
