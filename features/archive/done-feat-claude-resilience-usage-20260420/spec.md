# Feature: feat-claude-resilience-usage Claude Code 健壮性 + Token 用量统计

## Basic Information
- **ID**: feat-claude-resilience-usage
- **Name**: Claude Code 健壮性 + Token 用量统计（Session Resume 降级重试 + 按模型聚合统计）
- **Priority**: 75
- **Size**: S
- **Dependencies**: feat-claude-stream-protocol
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-20

## Description

增强 Claude Code Runtime 的健壮性和可观测性：

1. **Session Resume 降级重试** — 当 `--resume` 恢复会话失败时（session_id 不匹配），自动清空 resume 重试新 session，避免任务卡死
2. **Token Usage 统计** — 从 Claude Code stdout 的 `claudeSDKMessage.usage` 提取 token 消耗数据，按 model 聚合 input/output/cache tokens，用于成本追踪和展示

## User Value Points

### VP1: Session 恢复健壮性
当 Agent 尝试恢复上一次对话 session 失败时，自动降级为全新 session，确保任务不会因 session 失效而卡死。

### VP2: Token 用量可观测
用户可以看到每次对话的 token 消耗（按模型细分），了解 API 成本。

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/claude.rs` — Claude Code CLI Runtime
  - `ProcessHandle::session_id` — 当前 session ID
  - `execute_thread_mode()` — Thread 模式使用 `--resume`
  - stdout 解析循环 — 已解析 `system`, `assistant`, `result` 等类型
- `src-tauri/src/runtime/mod.rs` — `AgentRuntime` trait，`ExecuteParams` 结构
- `src-tauri/src/runtime/a2a/adapter/claude_adapter.rs` — A2A adapter

### Multica Reference
- `server/pkg/agent/claude.go` — `resolveSessionID()` session 降级逻辑
- `server/pkg/agent/claude.go` — `claudeUsage` struct token 统计
- `server/pkg/agent/agent.go` — `TokenUsage` struct, `Result.Usage` map

## Technical Solution

### 1. Session Resume 降级重试

**问题**: Thread 模式下 `ProcessHandle` 被驱逐后，下次对话使用 `--resume <session_id>`。但如果 session 已过期或不存在，Claude Code 会创建新 session 并立即退出，导致任务失败。

**方案**: 检测 resume 失败并自动重试。

```rust
// claude.rs - execute_thread_mode() 修改

// 首次尝试 resume
let result = self.execute_with_resume(session_id, params);

// 检测 resume 失败：result 失败 且 session_id 变了
if result.is_failed() && session_id.is_some() {
    let new_session_id = extract_session_id(&output);
    if new_session_id != session_id {
        log::warn!("Session resume failed, retrying with fresh session");
        // 清空 resume，重跑
        return self.execute_with_resume(None, params);
    }
}
```

关键判断条件（来自 Multica 的 `resolveSessionID`）：
- 运行状态 = failed
- 请求了 resume (PriorSessionID != "")
- Claude 返回了不同的 session_id
- 说明 resume 没有成功着陆

### 2. Token Usage 统计

**数据来源**: Claude Code `stream-json` 输出中 `assistant` 类型消息的 `message.usage` 字段：

```json
{
  "type": "assistant",
  "message": {
    "role": "assistant",
    "model": "claude-sonnet-4-6",
    "usage": {
      "input_tokens": 1234,
      "output_tokens": 567,
      "cache_read_input_tokens": 890,
      "cache_creation_input_tokens": 123
    }
  }
}
```

**方案**: 在 stdout 解析中提取 usage 数据，累加到 `ExecuteResult`。

```rust
// types 扩展
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

pub struct ExecuteResult {
    // ... existing fields
    pub token_usage: HashMap<String, TokenUsage>,  // NEW: model -> usage
}

// stdout 解析
"assistant" => {
    // 提取 usage
    if let Some(usage) = msg.message.get("usage") {
        let model = msg.message["model"].as_str().unwrap_or("unknown");
        // 累加到 result.token_usage[model]
    }
}
```

**前端展示**: 在 Thread/Channel 消息中展示 token 消耗，或在 Agent 状态面板中聚合展示（可后续 feature 处理，本 feature 先完成数据采集）。

## Acceptance Criteria (Gherkin)

### VP1: Session 恢复健壮性

```gherkin
Feature: Session Resume 降级重试

Scenario: Session 恢复成功时正常执行
  Given 一个 Agent 的 ProcessHandle 有 session_id "abc123"
  When 用户发送新消息触发 --resume abc123
  And Claude Code 成功恢复该 session
  Then 对话正常进行
  And session_id 保持 "abc123"

Scenario: Session 恢复失败时自动降级
  Given 一个 Agent 的 ProcessHandle 有 session_id "expired-session"
  When 用户发送新消息触发 --resume expired-session
  And Claude Code 返回新的 session_id "new-session" 且执行失败
  Then Runtime 检测到 resume 失败
  And 自动以全新 session 重试执行
  And 用户收到正常的响应

Scenario: 连续 resume 失败不无限循环
  Given Session 恢复失败已触发一次降级重试
  When 降级重试也失败
  Then 返回错误信息给用户
  And 不再尝试更多重试
```

### VP2: Token 用量统计

```gherkin
Feature: Token Usage 统计

Scenario: 解析 assistant 消息中的 usage 数据
  Given Claude Code 输出包含 usage 字段的 assistant 消息
  When Runtime 解析该消息
  Then token_usage 按 model 累加 input/output/cache tokens
  And 数据存储在 ExecuteResult 中

Scenario: 多轮对话 token 累加
  Given 一个 Thread 对话产生 3 轮 assistant 响应
  When 对话完成
  Then ExecuteResult.token_usage 包含所有轮次的 token 总和
  And 按 model 分组统计

Scenario: 无 usage 数据时不报错
  Given Claude Code 输出的 assistant 消息不包含 usage 字段
  When Runtime 解析该消息
  Then 跳过 usage 统计，不报错
  And token_usage 为空 HashMap
```

### General Checklist
- [x] Session resume 降级重试正确工作
- [x] 降级重试有日志记录（warn 级别）
- [x] 降级重试最多执行一次，不无限循环
- [x] Token usage 从 assistant 消息正确提取
- [x] Token usage 按 model 正确累加
- [x] 无 usage 数据时优雅处理
- [x] ExecuteResult 结构扩展向后兼容
- [x] 数据可供前端消费（Tauri event 或 IPC）

## Merge Record

- **Completed**: 2026-04-20T16:30:00+08:00
- **Merged Branch**: feature/feat-claude-resilience-usage
- **Merge Commit**: 992382c
- **Feature Commit**: 787b36d
- **Archive Tag**: feat-claude-resilience-usage-20260420
- **Conflicts**: None
- **Verification**: All 6 Gherkin scenarios passed (code analysis)
- **Stats**: 1 commit, 6 files changed, +319/-2 lines
