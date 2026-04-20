# Feature: feat-token-usage-ui Token 用量前端展示

## Basic Information
- **ID**: feat-token-usage-ui
- **Name**: Token 用量前端展示（消息底部消耗 + Agent 面板聚合统计）
- **Priority**: 75
- **Size**: M
- **Dependencies**: feat-claude-resilience-usage
- **Parent**: null
- **Children**: empty
- **Created**: 2026-04-20

## Description

在 UI 中展示 Claude Code Token 用量统计数据。后端已通过 `StreamEvent.token_usage` (HashMap<String, TokenUsage>) 采集并传递 token 数据，本 feature 负责前端消费和展示。

两个展示位置：
1. **消息底部**：每条 Agent 回复消息末尾显示本次 token 消耗（input/output/cache tokens）
2. **Agent 面板**：Agent Profile/状态面板中按模型聚合展示累计用量

## User Value Points

### VP1: 消息级 Token 透明
用户在每次 Agent 回复后可立即看到本次消耗，感知 API 成本。

### VP2: Agent 级用量聚合
用户在 Agent 详情面板中可查看该 Agent 历史累计的 token 消耗，按模型分组。

## Context Analysis

### Reference Code
- `src/types.ts` — StreamEvent 类型，需添加 `token_usage` 字段
- `src/types.ts` — ChannelMessage / ThreadMessageData，需添加 token_usage 持久化字段
- `src/lib/useAgentRuntimes.ts:150` — Channel 模式 `agent://chunk` 事件监听
- `src/lib/useThreadChat.ts:343` — Thread 模式 `agent://chunk` 事件监听
- `src/components/markdown/MessageContentRenderer.tsx` — 消息内容渲染
- `src/components/markdown/MarkdownRenderer.tsx` — Markdown 渲染器
- `src/components/ThreadPanel.tsx` — Thread 面板消息渲染
- `src/components/MainContent.tsx` — 主内容区，包含 PROFILE tab

### Backend Data Format
```rust
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}
// StreamEvent.token_usage: Option<HashMap<String, TokenUsage>>
// key = model name (e.g. "claude-sonnet-4-6")
```

### Related Features
- feat-claude-resilience-usage — 后端 Token 数据采集（已完成）
- feat-claude-stream-protocol — Stream JSON 协议增强（已完成）

## Technical Solution

### 1. TypeScript 类型更新
- `StreamEvent` 添加 `token_usage?: Record<string, TokenUsage>`
- 新增 `TokenUsage` interface
- `ChannelMessage` 添加 `token_usage?: Record<string, TokenUsage>`
- `ThreadMessageData` 添加 `token_usage?: Record<string, TokenUsage>`

### 2. 数据采集层
- `useAgentRuntimes.ts` — `is_done` 事件时提取 `token_usage`，附加到 ChannelMessage
- `useThreadChat.ts` — `is_done` 事件时提取 `token_usage`，附加到 ThreadMessageData
- 消息持久化时一同保存 token_usage（JSONL）

### 3. 消息底部 Token 展示
- 新建 `TokenUsageBadge.tsx` 组件 — 紧凑的内联 badge，显示总 token 数
  - 默认折叠：`1.2k tokens`
  - hover/点击展开：显示 input/output/cache 分布 + 模型名
  - 仅 agent 类型消息显示，user 消息不显示
- 集成到 `MessageContentRenderer.tsx` 和 `ThreadPanel.tsx` 消息末尾

### 4. Agent 面板聚合统计
- `useAgentProfile.ts` 新增 `token_usage_summary` 状态
- 聚合该 Agent 所有消息的 token_usage（从消息历史计算）
- 在 Agent Profile 页中展示统计卡片：按模型分组的累计用量

## Acceptance Criteria (Gherkin)

### Scenario 1: Channel 消息显示 Token 用量
```gherkin
Given 用户在 Channel 中发送消息触发 Agent
When Agent 完成回复（is_done=true 且 token_usage 非空）
Then 该 Agent 回复消息底部显示 Token 用量 badge
And badge 显示总 token 数（如 "1.2k tokens"）
And hover 时展开显示 input/output/cache 分布和模型名
```

### Scenario 2: Thread 消息显示 Token 用量
```gherkin
Given 用户在 Thread 中与 Agent 对话
When Agent 完成回复（is_done=true 且 token_usage 非空）
Then 该回复消息底部显示 Token 用量 badge
```

### Scenario 3: 无 Token 数据时不显示
```gherkin
Given Agent 回复完成
When is_done=true 但 token_usage 为空（旧数据或非 Claude Runtime）
Then 消息底部不显示 Token badge
```

### Scenario 4: Agent 面板聚合统计
```gherkin
Given 用户进入 Agent Profile 页
When 该 Agent 有历史消息包含 token_usage 数据
Then Profile 页显示 Token 统计卡片
And 卡片按模型分组显示累计 input/output/cache tokens
```

### Scenario 5: 多模型分别统计
```gherkin
Given Agent 在不同对话中使用了 claude-sonnet-4-6 和 claude-haiku-4-5
When 查看 Agent Profile Token 统计
Then 两个模型的用量分别显示
```

### UI/Interaction Checkpoints
- TokenUsageBadge 样式：semi-transparent 背景色，小字体，紧贴消息底部右侧
- Agent Profile Token 卡片：与现有 Profile 卡片风格一致（brutalist card 样式）
- 数字格式化：>1000 显示为 "1.2k"，>1000000 显示为 "1.2M"

### General Checklist
- [x] TypeScript 类型无错误
- [x] 不影响现有消息渲染性能
- [x] token_usage 可选字段，向后兼容旧消息

## Merge Record
- **Completed**: 2026-04-20
- **Merged Branch**: feature/feat-token-usage-ui
- **Merge Commit**: 73d84a1
- **Archive Tag**: feat-token-usage-ui-20260420
- **Conflicts**: None
- **Verification**: All 5/5 Gherkin scenarios passed (code analysis)
- **Stats**: 2 commits, 14 files changed, 488 insertions, 75 deletions
