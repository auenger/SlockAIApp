# Feature: feat-channel-multi-agent Channel 多Agent协作

## Basic Information
- **ID**: feat-channel-multi-agent
- **Name**: Channel 多Agent协作
- **Priority**: 65
- **Size**: M
- **Dependencies**: feat-conversation-store, feat-channel-infra
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-09

## Description
在 Channel 基础上实现 @Agent mention 触发机制和多 Agent 协作对话。用户可以在 Channel 消息中 @某个 Agent 来定向触发对话，多个 Agent 可以依次或并行响应。包含上下文编排引擎的集成——为多 Agent 场景组装正确的上下文（SOUL.md + 历史对话 + Channel 上下文）。

这是 Channel 的核心差异化价值 —— 真正的多 Agent 协作对话。

## User Value Points

### VP1: @Agent Mention 触发
用户在 Channel 消息中通过 `@AgentName` 精确指定哪个 Agent 应该回复，支持同时 @多个 Agent。

### VP2: 多 Agent 依次响应
在 Channel 中 @了多个 Agent 时，Agent 可以依次回复，形成协作讨论。

### VP3: 上下文编排
每个 Agent 在 Channel 中回复时，会获得正确的上下文（Channel 对话历史 + Agent 个人 SOUL.md + 相关记忆）。

## Context Analysis

### Reference Code
- `src-tauri/src/context/` — Context 编排引擎（部分实现）
- `src-tauri/src/workspace/identity.rs` — SOUL.md / IDENTITY.md 加载
- `reference/AINative/neuro-syntax-ide/src-tauri/src/lib.rs` — ReqAgent 的 @Agent 路由实现参考

### Related Documents
- feat-agent-workspace-design spec — 上下文编排引擎设计
- feat-claude-runtime spec — Runtime execute 参数（append_system_prompt, add_dir）

### Related Features
- feat-channel-infra（前置）— Channel 基础设施
- feat-conversation-store（前置）— 对话持久化

## Technical Solution

### Backend Architecture

**@Mention Parser** (`src-tauri/src/workspace/mention.rs`):
- Parses `@AgentName` (word format) and `@{Agent Name}` (braced format) from messages
- Case-insensitive matching against channel member agent_ids
- Supports CJK characters in agent names
- Falls back to first channel member when no valid @mention found
- Uses regex for braced mentions, manual parsing for word mentions

**Multi-Agent Execution** (`src-tauri/src/commands/channel.rs`):
- Serial execution: agents respond one at a time in @mention order
- Each agent gets its own context assembled via `ContextBuilder`
- Streaming events include `agent_id`, `agent_index`, `total_agents` for frontend routing
- Errors in one agent don't block subsequent agents
- `agent://channel-agent-start` event signals frontend which agent is responding
- `agent://channel-chunk` event carries per-agent streaming data
- `agent://channel-response` event signals completion per agent

**Context Orchestration** (integrated into `send_channel_message`):
- Uses existing `ContextBuilder::build_context_prefix()` for SOUL.md + IDENTITY.md + MEMORY.md
- Appends Channel conversation history (last 20 messages) as formatted text
- Channel context includes sender names (resolved from agent_id) for clarity
- Combined context passed via `--append-system-prompt` to the Claude runtime

### Frontend Architecture

**@Mention Autocomplete** (`src/components/MentionAutocomplete.tsx`):
- Replaces standard textarea in channel mode
- Detects `@` trigger and shows dropdown of channel agent members
- Keyboard navigation (Up/Down/Tab/Enter/Escape)
- Inserts `@AgentName` text on selection
- `renderMentionText()` utility highlights @mentions in blue/bold

**Multi-Agent Reply Display** (updated `MainContent.tsx`):
- Per-agent color palette for visual distinction (8 colors rotating)
- `AgentStreamBubble` component shows per-agent thinking/streaming/done states
- Agent emoji/avatar shown in colored bubbles
- `(1/3)` progress indicator for multi-agent responses
- Context info badges (SOUL.md, Channel History, MEMORY.md) shown below agent messages

**Channel Hook** (updated `src/lib/useChannel.ts`):
- `AgentStreamState` tracks per-agent streaming state
- Listens for `agent://channel-agent-start`, `agent://channel-chunk`, `agent://channel-response`
- Backward compatible: single-agent `streamingText` still works
- Multi-agent `agentStreams` array for new UI

## Acceptance Criteria (Gherkin)

### User Story
作为 AgentsZone 用户，我希望在 Channel 中通过 @Agent 来让多个 Agent 协作讨论，每个 Agent 都能理解上下文并做出有意义的回复。

### Scenarios (Given/When/Then)

#### Scenario 1: @Agent Mention 触发回复
```gherkin
Given Channel "project-alpha" 中有 Agent "克劳德" 和 "Alice"
When 用户发送 "@克劳德 请分析这个架构设计"
Then 只有 "克劳德" 被触发执行
And 克劳德的回复包含其 SOUL.md 定义的个性化风格
And 回复显示在 Channel 聊天区域，标注为 "克劳德" 发送
```

#### Scenario 2: 多 Agent 协作
```gherkin
Given Channel "project-alpha" 中有 Agent "克劳德" 和 "Alice"
When 用户发送 "@克劳德 @Alice 请分别review这个方案"
Then "克劳德" 先回复其分析
And "Alice" 随后回复其分析
And 两条回复都显示在 Channel 聊天区域
And 每条回复标注了对应的 Agent 名称和头像
```

#### Scenario 3: Mention 自动补全
```gherkin
Given 用户在 Channel 消息输入框中输入 "@"
Then 弹出 Agent 成员列表下拉菜单
And 继续输入 "@克" 时列表过滤为匹配的 Agent
And 选择 Agent 后自动插入 "@克劳德"
```

#### Scenario 4: Channel 上下文传递
```gherkin
Given Channel 中已有之前的对话历史
When Agent 被触发回复
Then Agent 收到的 system prompt 包含：
  | Agent 的 SOUL.md 内容
  | Channel 最近 N 条对话历史
  | Agent 的 MEMORY.md 内容
And Agent 能引用之前的对话内容
```

### UI/Interaction Checkpoints
- 消息输入框输入 `@` 时弹出 Agent 成员下拉列表
- Agent 回复在聊天区域用不同的头像/颜色区分
- 多个 Agent streaming 时分别显示各自的进度
- @mention 文本高亮显示（如 `@克劳德` 显示为可点击的蓝色标签）

### General Checklist
- [ ] @Agent mention 解析器
- [ ] 消息路由到指定 Agent Runtime
- [ ] 多 Agent 响应协调（串行执行）
- [ ] Context 编排集成（SOUL.md + 历史 + 记忆）
- [ ] Mention 自动补全 UI
- [ ] 多 Agent 回复的视觉区分
