# Tasks: feat-thread-context-inject

## Task Breakdown

### 1. 修改 send_message 命令
- [x] 在 `thread.rs` 的 `send_message` 中引入 `ContextBuilder`
- [x] 调用 `build_context_prefix(&agent_id)` 生成 system_prompt
- [x] 将 context_prefix 传入 `ExecuteParams.system_prompt`
- [x] 确保 workspace_root 可在 lock scope 内获取

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-09 | Feature created | 等待开发 |
| 2026-04-09 | Implemented | 使用 ContextBuilder 构建 system_prompt，与 Channel 模式保持一致 |
