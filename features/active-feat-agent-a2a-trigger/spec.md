# Feature: feat-agent-a2a-trigger Agent-to-Agent Trigger

## Basic Information
- **ID**: feat-agent-a2a-trigger
- **Name**: Agent-to-Agent @{agent} 触发机制
- **Priority**: 60
- **Size**: S
- **Dependencies**: feat-channel-zone-protocol
- **Parent**: feat-channel-agent-prompt-arch
- **Children**: (none)
- **Created**: 2026-04-10

## Description

实现 Agent 在响应中通过 @{agent} 触发其他 Agent 的能力。当一个 Agent 的回复中包含 @other_agent 时，系统自动解析并触发对应 Agent 进行响应，形成多 Agent 协作链。

### 核心机制

```
用户: "@Claude 分析这段代码的安全性"
Claude 回复: "这段代码有 SQL 注入风险。@Codex 请查一下类似漏洞的修复方案。"
  → 系统解析 Claude 回复中的 @Codex
  → 自动触发 Codex，附带 Claude 的分析作为上下文
Codex 回复: "针对 SQL 注入，推荐使用参数化查询..."
  → 写回 Channel 对话
```

### 安全机制

1. **最大深度限制**: Agent 触发链最多 3 层（A→B→C→STOP）
2. **去重**: 同一个 Agent 在一条触发链中只触发一次
3. **超时**: 每个 Agent 响应有独立的超时控制
4. **用户可见**: 每条 Agent 间的触发都在 Channel 中可见，用户可以观察协作过程

## User Value Points

1. **Agent 自主协作** - Agent 可以主动请求其他 Agent 的帮助，无需用户手动触发
2. **协作链可视化** - 用户可以看到 Agent 之间如何协作，保持透明度

## Context Analysis

### Reference Code
- `src-tauri/src/commands/channel.rs` — Channel 消息发送，需扩展支持 A2A 触发
- `src-tauri/src/workspace/mention.rs` — @mention 解析，复用解析逻辑
- `src-tauri/src/runtime/registry.rs` — Runtime 执行，复用执行逻辑

### Related Documents
- `feat-channel-zone-protocol/spec.md` — Zone Protocol 定义了 @{agent} 触发格式

### Related Features
- `feat-channel-zone-protocol` — 前置依赖，提供 Zone Agent Protocol 层

## Technical Solution

### 核心变更

#### 1. Agent 响应 @mention 解析

复用 `mention.rs` 中的解析逻辑，对 Agent 的响应内容进行 @mention 解析：

```rust
/// 解析 Agent 响应中的 @mention，提取触发的 Agent ID
fn extract_agent_triggers(
    response: &str,
    channel_members: &[ChannelMember],
    agents: &[Agent],
) -> Vec<String> {
    // 复用 mention 解析逻辑
    let mentions = parse_mentions(response, &build_member_lookup(channel_members, agents));
    mentions.into_iter().map(|m| m.agent_id).collect()
}
```

#### 2. 触发链执行

修改 `send_channel_message` 支持递归触发：

```rust
struct TriggerContext {
    depth: u32,
    max_depth: u32,
    triggered_agents: HashSet<String>,
}

async fn execute_with_a2a(
    params: ExecuteParams,
    channel: &mut Channel,
    trigger_ctx: TriggerContext,
) -> Result<()> {
    // 1. 执行 Agent
    let response = execute_agent(params).await?;

    // 2. 解析响应中的 @mention
    let triggers = extract_agent_triggers(&response, &channel.members, &agents);

    // 3. 过滤：深度限制 + 去重
    let valid_triggers: Vec<String> = triggers.into_iter()
        .filter(|id| !trigger_ctx.triggered_agents.contains(id))
        .filter(|_| trigger_ctx.depth < trigger_ctx.max_depth)
        .collect();

    // 4. 递归触发
    for agent_id in valid_triggers {
        let mut new_ctx = trigger_ctx.clone();
        new_ctx.depth += 1;
        new_ctx.triggered_agents.insert(agent_id.clone());

        execute_with_a2a(new_params, channel, new_ctx).await?;
    }

    Ok(())
}
```

#### 3. 前端事件通知

A2A 触发时发送特定事件，让前端区分用户触发和 Agent 触发：

```rust
// Tauri Events
app.emit("agent://channel-a2a-start", Payload {
    agent_id: triggered_agent_id,
    triggered_by: source_agent_id,
    depth: current_depth,
})?;
```

## Acceptance Criteria (Gherkin)

### User Story
作为用户，当 Agent 在 Channel 中 @提及另一个 Agent 时，我希望系统自动触发被提及的 Agent 进行响应，实现多 Agent 协作。

### Scenarios

#### Scenario 1: Agent 成功触发另一个 Agent
```gherkin
Given Channel 包含 Claude 和 Codex
And Claude 已收到 Zone Agent Protocol 上下文
When Claude 的回复中包含 "@Codex 请查一下相关资料"
Then 系统解析出 @Codex mention
And 自动触发 Codex 执行
And Codex 收到的消息包含 Claude 的回复作为上下文
And Channel 中可见两条 Agent 消息（Claude + Codex）
```

#### Scenario 2: 触发链深度限制
```gherkin
Given 最大触发深度为 3
When Claude → Codex → Gemini 的触发链已执行
And Gemini 的回复中包含 "@Claude"
Then 系统不再触发 Claude（已达到最大深度）
And 在 Channel 中记录一条系统消息 "触发链已达到最大深度"
```

#### Scenario 3: 防止循环触发
```gherkin
Given Claude 触发了 Codex
When Codex 的回复中包含 "@Claude"
Then 系统不再触发 Claude（已在本轮触发链中触发过）
And Codex 的回复正常显示
```

#### Scenario 4: 无效 @mention 静默忽略
```gherkin
Given Channel 包含 Claude 和 Codex
When Claude 的回复中包含 "@GPT-4 这个我不确定"
Then 系统忽略 @GPT-4（不是 Channel 成员）
And Claude 的回复正常显示，不触发任何额外 Agent
```

### General Checklist
- [ ] A2A 触发正确解析 Agent 响应中的 @mention
- [ ] 深度限制生效（默认 3 层）
- [ ] 去重机制防止循环触发
- [ ] 前端正确显示 A2A 触发事件
- [ ] A2A 触发的 Agent 回复写入 Channel 对话
