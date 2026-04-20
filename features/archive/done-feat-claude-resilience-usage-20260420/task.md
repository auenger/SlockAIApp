# Tasks: feat-claude-resilience-usage

## Task Breakdown

### 1. Session Resume 降级重试 (claude.rs)
- [x] 在 `execute()` 中检测 resume 失败条件
  - event.error.is_some()
  - prior_session_id.is_some()
  - 实际返回的 session_id != 请求的 session_id
- [x] 实现降级逻辑：`wrap_with_resume_retry()` 包装 receiver，自动重试
- [x] 添加 warn 日志：记录 resume 失败和降级重试
- [x] 确保最多重试一次，防止无限循环
- [x] 重试成功时合并两次的 token usage

### 2. Token Usage 数据提取 (claude.rs)
- [x] 定义 `TokenUsage` struct (input/output/cache_read/cache_write)
- [x] 在 `parse_stream_event()` 中提取 `message.usage` 字段
- [x] 按 model name 累加 token 数据
- [x] 将 token_usage 存入 StreamEvent
- [x] 处理 usage 字段缺失的情况（跳过不报错）

### 3. ExecuteResult 结构扩展 (mod.rs)
- [x] 在 `StreamEvent` 中添加 `token_usage: Option<HashMap<String, TokenUsage>>`
- [x] 确保序列化正确（Tauri event 前端可消费，skip_serializing_if）
- [x] 更新所有 StreamEvent 构造点（codex.rs, bridge.rs, streaming.rs, cli_adapter.rs）

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-20 | Created | 依赖 feat-claude-stream-protocol |
| 2026-04-20 | Implemented | All 3 tasks completed, compiles successfully (pre-existing errors unrelated) |
