# Tasks: feat-agent-workspace-persist

## Task Breakdown

### 1. 验证 workspace 创建流程
- [x] 确认 `initialize_workspace()` 在 app 启动时正确执行
- [x] 确认 `create_agent_internal()` 在用户创建 Agent 时正确执行
- [x] 检查错误处理——是否被 `if let Err(e)` 静默吞掉

**Implementation notes:**
- `initialize_workspace()` is correctly called in `lib.rs` setup block
- `create_agent_internal()` is correctly called via `create_agent()` command
- Error handling upgraded from `log::warn!` to `log::error!` with self-heal retry logic
- If initialization fails, app now attempts a self-heal before continuing

### 2. 检查 workspace 状态返回
- [x] 验证 `get_workspace_status` 返回完整信息（路径、agent 数量等）
- [x] 检查前端是否能正确显示 workspace 路径

**Implementation notes:**
- `ManagerStatus` enriched with `agents_health: Vec<AgentHealthInfo>` for per-agent health monitoring
- Added `AgentHealthInfo` struct tracking: workspace_exists, identity_file_exists, soul_file_exists, conversations_dir_exists, context_dir_exists, is_healthy, missing_items
- Added `health_check_workspace` command for on-demand health check with auto-repair
- Frontend types updated with matching `AgentHealthInfo` interface
- IPC wrapper added for `healthCheckWorkspace()`

### 3. 增强 workspace 健壮性（如需要）
- [x] `load()` 时如果 agent 目录存在但文件缺失，自动修复
- [x] 考虑添加 workspace 健康检查方法

**Implementation notes:**
- `load()` now performs lightweight health check on each agent, auto-creating missing subdirectories
- `check_agent_health()` static method added to `AgentManager` for thorough diagnostics
- `health_check_workspace` command provides on-demand repair capability
- Added startup log messages for better observability of workspace initialization

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created，范围从 Channel JSONL 缩小到 workspace 验证 | Channel per-agent JSONL 不必要，Channel 是共享上下文 |
| 2026-04-11 | All tasks implemented | Rust: 63 tests pass, TypeScript: builds clean |
