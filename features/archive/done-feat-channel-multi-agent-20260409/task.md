# Tasks: feat-channel-multi-agent

## Task Breakdown

### 1. Rust Backend - @Mention 解析
- [x] 解析消息中的 `@{agent_name}` 模式
- [x] 匹配 Channel 成员中的 Agent
- [x] 无有效 @mention 时路由到默认 Agent

### 2. Rust Backend - 多 Agent 执行协调
- [x] 串行执行：按 @mention 顺序依次调用 Agent Runtime
- [x] 每个 Agent execute 时注入正确的上下文：
  - Agent 的 SOUL.md
  - Channel 最近 N 条对话历史
  - Agent 的 MEMORY.md
- [x] 每个 Agent 的流式事件独立推送，携带 agent_id 标识

### 3. Rust Backend - Context 编排集成
- [x] 组装 Channel 上下文：最近 N 条消息作为对话历史
- [x] 组装 Agent 上下文：SOUL.md + IDENTITY.md
- [x] 通过 `--append-system-prompt` 传递 Channel 上下文
- [x] 通过 `--resume` 传递 Agent 在该 Channel 的 session

### 4. Frontend - @Mention 自动补全
- [x] 输入 `@` 时弹出 Agent 成员下拉列表
- [x] 输入过滤匹配 Agent 名称
- [x] 选择后插入 `@AgentName` 文本
- [x] @mention 文本高亮渲染（蓝色/加粗）

### 5. Frontend - 多 Agent 回复展示
- [x] 消息列表区分不同 Agent 的回复（头像/颜色/名称）
- [x] 多个 Agent 同时 streaming 时分别显示进度
- [x] StreamEvent 增加 agent_id 字段，前端据此区分

### 6. Frontend - Context 显示（可选）
- [x] Agent 回复时显示使用的上下文摘要（debug/info 面板）
- [x] 显示 "基于 N 条历史消息 + SOUL.md + MEMORY.md"

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-09 | All tasks completed | Implemented @Mention parser, multi-agent serial execution, context orchestration, mention autocomplete UI, multi-agent reply display |
