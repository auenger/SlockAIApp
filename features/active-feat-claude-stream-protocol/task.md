# Tasks: feat-claude-stream-protocol

## Task Breakdown

### 1. CLI 参数增强 (claude.rs)
- [x] `build_command()` 添加 `--input-format stream-json`
- [x] `build_command()` 添加 `--permission-mode bypassPermissions`
- [x] 确保 Thread 模式和 Channel 模式均使用新参数
- [x] 确保参数不影响 `--resume` 等现有功能

### 2. control_response 自动批准 (claude.rs)
- [x] stdout 解析新增 `control_request` 消息类型处理
- [x] `ProcessHandle` 添加 `handle_control_request()` 方法
- [x] 通过 stdin 写入 `control_response` 自动批准
- [x] Channel 模式（一次性进程）也支持 control_response (stdin=null for one-shot, but `--permission-mode bypassPermissions` covers it)

### 3. MCP Config 注入 (claude.rs + mod.rs)
- [x] `ExecuteParams` 添加 `mcp_config` 字段
- [x] 执行时将 mcp_config JSON 写入临时文件
- [x] `build_command()` 添加 `--mcp-config` + `--strict-mcp-config` 参数
- [x] 进程结束后清理临时文件
- [x] 未配置时不添加参数（向后兼容）

### 4. A2A Adapter 适配 (claude_adapter.rs)
- [x] 确认 A2A Claude adapter 也传递新参数 (ExecuteParams with mcp_config: None)
- [x] 远程 Agent 场景下 MCP config 传递路径 (via ExecuteParams.mcp_config)

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-20 | Created | 借鉴 Multica 通信方案 |
| 2026-04-20 | All tasks completed | 4/4 tasks done; 2 pre-existing compile errors (handler.rs, transport.rs) unrelated |
