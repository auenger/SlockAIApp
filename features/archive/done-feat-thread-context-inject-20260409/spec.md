# Feature: feat-thread-context-inject Thread 模式 Context 注入

## Basic Information

* **ID**: feat-thread-context-inject

* **Name**: Thread 模式 Context 注入

* **Priority**: 75

* **Size**: S

* **Dependencies**: feat-thread-chat (completed), feat-agent-workspace-design (completed)

* **Parent**: null

* **Children**: []

* **Created**: 2026-04-09

## Description

当前 Thread 1:1 对话的 `send_message` 命令中 `system_prompt` 传的是 `None`，即 **Thread 模式完全没有注入 Agent 上下文**（SOUL.md、IDENTITY.md、MEMORY.md 等）。而 Channel 模式已通过 `ContextBuilder` 正确构建了完整上下文。需要让 Thread 模式也使用 `ContextBuilder` 注入 Agent 的角色和上下文信息。

## User Value Points

1. **Thread 上下文感知** — Agent 在 Thread 对话中知道自己是谁、有什么角色设定，回复更具个性化

## Context Analysis

### Reference Code

* `src-tauri/src/commands/thread.rs:238-253` — `send_message` 中 `system_prompt: None`

* `src-tauri/src/commands/channel.rs:317-371` — Channel 模式的 Context 构建逻辑（参考）

* `src-tauri/src/context/mod.rs` — ContextBuilder 已实现 `build_context_prefix()`

* `src-tauri/src/workspace/manager.rs` — AgentManager 可获取 workspace 路径

### Related Features

* feat-thread-chat (completed) — Thread 1:1 对话

* feat-agent-workspace-design (completed) — Agent Workspace & Identity

## Technical Solution

1. 在 `send_message` 的 `ExecuteParams` 构建之前，使用 `ContextBuilder::build_context_prefix(&agent_id)` 生成系统提示

2. 将生成的 context_prefix 作为 `system_prompt` 传入

3. 与 Channel 模式保持一致的上下文组装逻辑

## Acceptance Criteria (Gherkin)

### User Story

作为用户，我想在 Thread 对话中 Agent 能感知自己的角色设定和上下文。

### Scenarios (Given/When/Then)

#### Scenario 1: Thread 注入 Context

```gherkin
Given Agent 有 IDENTITY.md 和 SOUL.md 文件
When 用户在 Thread 中发送消息
Then send_message 使用 ContextBuilder 构建系统提示
And Claude Code CLI 收到 --append-system-prompt 参数
And Agent 回复反映其角色设定
```

#### Scenario 2: 无 Identity 文件

```gherkin
Given Agent 没有自定义 IDENTITY.md
When 用户在 Thread 中发送消息
Then 使用默认 context_prefix（全局 SOUL.md 等）
And 不报错
```

### General Checklist

* [ ] Thread 模式和 Channel 模式的上下文注入一致
* [ ] 不影响现有 Thread 对话功能

⠀