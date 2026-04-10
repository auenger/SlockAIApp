# Feature: feat-channel-zone-protocol Channel Zone Protocol

## Basic Information
- **ID**: feat-channel-zone-protocol
- **Name**: Channel Zone Protocol（Prompt 7层架构 + Zone Agent Protocol 实现）
- **Priority**: 60
- **Size**: M
- **Dependencies**: none
- **Parent**: feat-channel-agent-prompt-arch
- **Children**: (none)
- **Created**: 2026-04-10

## Description

设计并实现 Channel 对话的 Prompt 7 层架构，核心是新增 **Zone Agent Protocol** 层。当 Agent 在 Channel 中被触发时，该层会告知 Agent 当前 Channel 的成员列表、各成员的角色和能力、协作规则，以及如何通过 @{agent} 与其他 Agent 交互。

### Prompt 7 层架构

| Layer | 名称 | 来源 | 内容 |
|-------|------|------|------|
| L1 | Runtime System Prompt | Agent Runtime 自带 | Claude Code / Codex / Gemini 的系统级能力、行为规范、工具定义 |
| L2 | **Zone Agent Protocol** | **本 feature 实现** | Channel 成员列表、角色能力、协作规则、@{agent} 触发协议 |
| L3 | Role Definition | IDENTITY.md + SOUL.md | Agent 身份、性格、专长定义 |
| L4 | Environment Context | workspace 元数据 | 工作目录、项目信息、运行环境 |
| L5 | Persistent Memory | memory/MEMORY.md | Agent 自维护的持久化记忆 |
| L6 | Conversation History | JSONL 对话记录 | 当前 channel 的近期对话（滑动窗口 + 压缩摘要） |
| L7 | System Reminders | 动态注入 | 日期、channel 描述、特殊指令等动态上下文 |

### Zone Agent Protocol 层内容

当 Agent 在 Channel 中被触发时，L2 层注入如下信息：

```
## Channel: {channel_name}

### 当前 Channel 成员

| Agent | 角色 | 能力 | Runtime |
|-------|------|------|---------|
| @Claude | 代码专家 | 代码编写、Review、Debug | claude-code |
| @Codex | 研究助手 | 信息检索、分析总结 | codex |

### 协作规则

1. 你可以通过 @{AgentName} 在回复中提及其他 Agent，系统会自动触发对应 Agent 进行响应
2. 如果需要其他 Agent 的专业能力来完成任务，请主动 @提及
3. 保持协作性：如果问题更适合其他 Agent 处理，建议用户 @提及该 Agent
4. 每次回复开头标注你的名字，让对话上下文清晰

### @触发格式

- 简单提及：@AgentName
- 带空格名称：@{Agent Name}
- 提及时附带指令：@Claude 请 review 这段代码
```

## User Value Points

1. **Agent 感知同伴** - Agent 在 Channel 中能知道还有哪些 Agent 共存，各自擅长什么
2. **协作规则内化** - Agent 理解多 Agent 协作规则，知道如何与其他 Agent 配合
3. **结构化 Prompt** - 清晰的 7 层架构让每层职责分明，便于调试和迭代

## Context Analysis

### Reference Code
- `src-tauri/src/context/mod.rs` — 现有 ContextBuilder，需扩展支持 Channel Zone Protocol 层
- `src-tauri/src/commands/channel.rs` — Channel 消息发送，调用 ContextBuilder 组装上下文
- `src-tauri/src/workspace/channel.rs` — Channel 数据模型（members, messages）
- `src-tauri/src/workspace/identity.rs` — Agent 身份解析（IDENTITY.md）
- `src-tauri/src/workspace/mention.rs` — @mention 解析逻辑

### Related Documents
- `project-context.md` — 项目整体架构
- `reference/anyclaw/templates/SOUL.md` — SOUL.md 模板参考

### Related Features
- `feat-agent-a2a-trigger` — Agent-to-Agent @{agent} 触发（依赖本 feature）
- `fix-channel-ui-bugs` — Channel UI 修复（pending）

## Technical Solution

### 核心变更

#### 1. 新增 `ChannelZoneProtocol` 模块

在 `src-tauri/src/context/` 下新增 `zone_protocol.rs`：

```rust
pub struct ChannelZoneProtocol {
    channel_name: String,
    channel_description: Option<String>,
    members: Vec<AgentMemberInfo>,
}

pub struct AgentMemberInfo {
    agent_id: String,
    display_name: String,
    creature: String,
    vibe: String,
    role_description: String,
    runtime_type: String,
}

impl ChannelZoneProtocol {
    /// 从 Channel 数据构建 Zone Protocol 上下文
    pub fn from_channel(channel: &Channel, agents: &[Agent]) -> Result<Self>;

    /// 渲染为 prompt 文本
    pub fn render(&self) -> String;
}
```

#### 2. 扩展 `ContextBuilder`

修改 `ContextBuilder::build_context_prefix()` 方法，增加 Channel Zone Protocol 层注入：

```rust
// 在 system_prompt (L3) 之前注入 Zone Protocol (L2)
if let Some(ref zone_protocol) = self.zone_protocol {
    prefix.push_str(&zone_protocol.render());
    prefix.push_str("\n\n---\n\n");
}
```

#### 3. 修改 `send_channel_message` 调用链

在 `commands/channel.rs` 中构建 Zone Protocol 并传入 ContextBuilder：

```rust
let zone_protocol = ChannelZoneProtocol::from_channel(&channel, &agents)?;
let context_prefix = context_builder
    .with_zone_protocol(zone_protocol)
    .build_context_prefix(&target_agent_id)?;
```

### Prompt 组装顺序（完整 7 层）

```
[L3] Role Definition (IDENTITY.md + SOUL.md)
  +
[L2] Zone Agent Protocol (Channel 成员 + 协作规则)
  +
[L4] Environment Context (workspace 信息)
  +
[L5] Persistent Memory (MEMORY.md)
  +
[L7] System Reminders (日期、channel 描述)
  → 组装为 system_prompt
  +
[L6] Conversation History (滑动窗口 + 摘要)
  → 作为 messages 数组
  +
[L1] Runtime System Prompt (Runtime 自带)
  → Runtime 自动注入
```

## Acceptance Criteria (Gherkin)

### User Story
作为用户，当我在 Channel 中 @触发一个 Agent 时，我希望该 Agent 能了解 Channel 中的其他 Agent 成员，并在需要时建议或主动 @其他 Agent 协作。

### Scenarios

#### Scenario 1: Agent 收到 Channel 成员上下文
```gherkin
Given 一个 Channel 包含 3 个 Agent 成员（Claude, Codex, Gemini）
When 用户发送消息 "@Claude 帮我分析这段代码"
Then Claude 的 prompt 中应包含 Zone Agent Protocol 层
And 该层列出所有 3 个 Agent 的名称、角色和 Runtime 类型
And 该层包含协作规则和 @{agent} 触发格式说明
```

#### Scenario 2: 单 Agent Channel 仍注入协议
```gherkin
Given 一个 Channel 只包含 1 个 Agent（Claude）
When 用户发送消息 "@Claude 你好"
Then Claude 的 prompt 中仍包含 Zone Agent Protocol 层
And 该层只列出 Claude 自身
And 协作规则中说明"当前 Channel 只有你一个 Agent"
```

#### Scenario 3: Channel 成员变更后协议更新
```gherkin
Given 一个 Channel 包含 Claude 和 Codex
When 用户将 Gemini 添加到 Channel
And 用户发送新消息 "@Claude 分析一下"
Then Claude 的 Zone Agent Protocol 层应包含 Gemini 作为新成员
```

#### Scenario 4: Agent 主动 @提及建议
```gherkin
Given Channel 包含 Claude（代码专家）和 Codex（研究助手）
When 用户发送 "@Claude 这个 API 的市场竞品有哪些？"
Then Claude 可能在回复中建议 "@Codex 可能更适合回答市场研究类问题"
```

### General Checklist
- [x] Zone Agent Protocol 层正确渲染 Channel 成员信息
- [x] 7 层 Prompt 按正确顺序组装
- [x] 不影响现有 Thread 对话（Thread 不注入 Zone Protocol）
- [x] 性能：Zone Protocol 渲染不超过 1ms
- [x] 兼容所有 Runtime 类型（Claude Code, Codex, Gemini）

## Merge Record

- **Completed**: 2026-04-11
- **Merged Branch**: feature/feat-channel-zone-protocol
- **Merge Commit**: 38783a5
- **Archive Tag**: feat-channel-zone-protocol-20260411
- **Conflicts**: None
- **Verification**: PASS (72/72 tests, 4/4 Gherkin scenarios validated)
- **Files Changed**: 3 (1 new, 2 modified)
- **New Tests**: 9 (7 zone_protocol + 2 ContextBuilder integration)
