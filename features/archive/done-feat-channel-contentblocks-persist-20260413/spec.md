# Feature: feat-channel-contentblocks-persist

## Basic Information
- **ID**: feat-channel-contentblocks-persist
- **Name**: Channel ContentBlocks 持久化可观测
- **Priority**: 50
- **Size**: M
- **Dependencies**: feat-channel-agent-thinking (completed)
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-13T15:00:00+08:00

## Description

Channel 对话历史中持久化保留 tool_use / tool_result 结构化内容块，使用户能在回看历史消息时观察 Agent 的完整执行过程（读文件、执行命令、编辑代码等）。

核心约束：
- **UI 展示**：历史消息中渲染 ContentBlockCard，用户可展开查看 tool 调用详情
- **上下文重建**：LLM 上下文只用 `msg.content` 文本，不包含 tool 调用结构
- **向后兼容**：旧 channel JSON 无 `content_blocks` 字段时反序列化为 None

同时包含两个已完成的优化（在本次 feature 中一并 commit）：
1. 移除 Zone Protocol 规则 4（Agent 不再需要自报姓名，UI 已渲染名称）
2. Streaming 指示器改为三个跳动小点（与 thinking 一致）

## User Value Points

### VP1: Tool 调用过程持久化
用户在 Channel 对话完成后，仍能回看 Agent 执行了哪些 tool 调用（Read、Bash、Edit 等），包括输入参数和执行结果。这对调试 Agent 行为和理解决策过程至关重要。

### VP2: 上下文隔离
LLM 上下文重建时只用纯文本内容，不携带 tool 调用详情，确保 token 预算不被 tool 输出占满，同时对话历史 UI 保持完整的可观测性。

## Context Analysis
### Reference Code
- `src-tauri/src/workspace/channel.rs` — ChannelMessage 结构体 + ChannelStore 读写
- `src-tauri/src/commands/channel.rs` — execute_single_agent_inner 事件处理 + 消息保存 + channel-response 事件
- `src-tauri/src/runtime/mod.rs` — StreamEvent.content_blocks 字段
- `src/types.ts` — ChannelMessage / ChannelResponseEvent / ContentBlock 接口
- `src/lib/useChannel.ts` — channel-response 事件处理 + UI state 更新
- `src/components/MainContent.tsx` — ContentBlockCard 组件 + 历史消息渲染
- `src-tauri/src/context/zone_protocol.rs` — Zone Protocol 规则（规则 4 已移除）

### Related Documents
- Plan: `~/.claude/plans/eventual-rolling-porcupine.md`

### Related Features
- feat-channel-agent-thinking (completed) — Channel Agent 思考过程渲染，本 feature 在此基础上持久化

## Technical Solution

### 1. Rust: ChannelMessage 加 content_blocks 字段

文件: `src-tauri/src/workspace/channel.rs`

```rust
pub struct ChannelMessage {
    // ...existing fields...
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub content_blocks: Option<Vec<serde_json::Value>>,
    pub timestamp: String,
}
```

`#[serde(skip_serializing_if, default)]` 保证向后兼容。

### 2. Rust: execute_single_agent_inner 收集并保存 content_blocks

文件: `src-tauri/src/commands/channel.rs`

- 新增 `collected_blocks: Vec<serde_json::Value>` 累积器
- assistant/user 事件带 content_blocks 时追加到累积器
- 保存 ChannelMessage 时写入 `content_blocks` 字段

### 3. Rust: channel-response 事件携带 content_blocks

文件: `src-tauri/src/commands/channel.rs`

emit payload 增加 `content_blocks` 字段。

### 4. TypeScript: 类型定义更新

文件: `src/types.ts`

- `ChannelMessage` 增加 `content_blocks?: ContentBlock[]`
- `ChannelResponseEvent` 增加 `content_blocks?: ContentBlock[]`

### 5. TypeScript: useChannel.ts channel-response handler

文件: `src/lib/useChannel.ts`

消息对象中携带 `content_blocks`。

### 6. UI: MainContent.tsx 历史消息渲染 ContentBlockCard

文件: `src/components/MainContent.tsx`

- `channelDisplayMessages` 映射保留 `content_blocks`
- Agent 历史消息中渲染 ContentBlockCard

### 附带改动（已做）

- `src-tauri/src/context/zone_protocol.rs` — 移除规则 4
- `src/components/MainContent.tsx` — Streaming 指示器改为三跳动小点

## Acceptance Criteria (Gherkin)

### Scenarios

#### Scenario 1: Tool 调用持久化并可在历史中查看
```gherkin
Given 一个包含 Agent 成员的 Channel
And 用户发送消息触发 Agent 执行（包含 Read/Bash 等 tool 调用）
When Agent 执行完成并返回最终文本结果
Then channel JSON 文件中该 agent 消息应包含 content_blocks 字段
And content_blocks 中应包含 tool_use 和 tool_result 类型的块
And 刷新页面重新加载 Channel 后，历史消息中仍能看到 tool 调用卡片
```

#### Scenario 2: 上下文重建只用文本
```gherkin
Given 一个 Channel 包含多条 agent 消息（含 content_blocks）
When 系统为 Agent 重建对话上下文
Then context_prefix 中应使用 [AgentName]: text 格式
And 不应包含 content_blocks 中的 tool 调用详情
```

#### Scenario 3: 向后兼容旧数据
```gherkin
Given 一个旧版 Channel JSON 文件（消息无 content_blocks 字段）
When 应用加载该 Channel
Then 消息应正常反序列化，content_blocks 为 undefined/None
And 不应导致渲染错误或崩溃
```

#### Scenario 4: Agent 不再自报姓名
```gherkin
Given 一个 Channel Zone Protocol 已加载
When Agent 收到回复指令
Then Agent 回复文本不应以 "AgentName: " 开头
And UI 消息头已正确渲染 Agent 名称
```

#### Scenario 5: Streaming 指示器使用跳动小点
```gherkin
Given Agent 正在 streaming 回复
When streaming 文本持续更新
Then 应显示三个 cyan 色跳动小点（与 thinking 一致）
And 不应使用闪烁光标块
```

### General Checklist
- [x] Rust 编译通过 (`cargo check`)
- [x] 前端编译通过 (`npm run build`)
- [x] 旧 Channel JSON 文件能正常加载

## Merge Record
- **Completed**: 2026-04-13T16:00:00+08:00
- **Merged branch**: feature/feat-channel-contentblocks-persist
- **Merge commit**: c0ae72c
- **Archive tag**: feat-channel-contentblocks-persist-20260413
- **Conflicts**: none
- **Verification**: passed (93 Rust tests, 5/5 Gherkin scenarios validated)
- **Stats**: 2 commits, 9 files changed, 498 insertions, 163 deletions
