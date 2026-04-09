# Tasks: feat-claude-runtime

## Task Breakdown

### 1. AgentRuntime Trait & Types 定义
- [x] 创建 `src-tauri/src/runtime/mod.rs`
- [x] 定义 `AgentRuntime` trait (id, name, runtime_type, capabilities, detect, health_check, execute)
- [x] 定义核心类型: `AgentRuntimeStatus`, `AgentCapability`, `AgentRuntimeInfo`, `ExecuteParams`, `StreamEvent`
- [x] 定义前端兼容的 Tauri Event 类型

### 2. Claude Code Runtime 实现
- [x] 创建 `src-tauri/src/runtime/claude.rs`
- [x] 实现 `ClaudeCodeRuntime` struct
- [x] `detect()` — 通过 `which claude` 检测 CLI 存在性 + `claude --version` 获取版本
- [x] `health_check()` — 验证 CLI 可用性
- [x] `execute()` — 核心：构建 CLI 参数、spawn 子进程、stdout/stderr 双线程读取、解析 stream-json、发送 StreamEvent
- [x] 处理 --verbose 模式下的嵌套 JSON 结构 (`message.content[]`)
- [x] 实现基于活跃度的 idle watchdog 超时机制

### 3. Runtime Registry 实现
- [x] 创建 `src-tauri/src/runtime/registry.rs`
- [x] `RuntimeRegistry` — 线程安全的 runtime 注册/检测/查询
- [x] `scan_all()` — 遍历所有注册 runtime，检测可用性
- [x] `list_all()` — 返回所有 runtime 信息（使用缓存）
- [x] `get_runtime_instance()` — 获取 trait object 引用

### 4. Tauri Commands 注册
- [x] 创建 `src-tauri/src/runtime/commands.rs`
- [x] `scan_agent_runtimes` command
- [x] `list_agent_runtimes` command
- [x] `runtime_execute` command — 调用 execute() 并通过 thread 转发事件到前端
- [x] `runtime_session_start` command — 生成 UUID session_id
- [x] `runtime_session_stop` command — 终止当前进程
- [x] 在 `lib.rs` 的 `invoke_handler` 中注册所有 commands
- [x] 扩展 `AppState` 加入 `agent_runtime_registry` 和 `agent_session`

### 5. 前端 Hook & 类型
- [x] 在 `src/types.ts` 中定义 `AgentRuntimeInfo`, `AgentRuntimeStatusType`, `StreamEvent` 类型
- [x] 创建 `src/lib/useAgentRuntimes.ts` hook
- [x] 实现 `scan()` — invoke('scan_agent_runtimes')
- [x] 实现 runtime 列表和状态管理
- [x] Dev fallback 模式（非 Tauri 环境下模拟数据）

### 6. Keyring 安全集成
- [x] 添加 `keyring` crate 依赖
- [x] 实现 `store_api_key` / `has_api_key` / `delete_api_key` commands
- [x] API Key 仅在 Rust 端使用，不暴露到前端

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-08 | Created | Feature created, referencing AINative implementation patterns |
| 2026-04-08 | Completed | All 6 tasks implemented in worktree |
