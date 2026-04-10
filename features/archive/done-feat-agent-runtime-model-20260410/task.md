# Tasks: feat-agent-runtime-model

## Task Breakdown

### 1. 数据模型扩展
- [x] 在 `RuntimeType` enum 中定义支持的 runtime 类型 (claude_code, codex, gemini, custom)
- [x] AgentIdentity 增加 `runtime_type` 字段，默认 `ClaudeCode`
- [x] 向后兼容：已有 agent 自动填充 `runtime_type: ClaudeCode`
- [x] 更新 workspace manager 的 save/load 逻辑

### 2. 泛化 AgentRuntime Trait
- [x] 扩展 `AgentRuntime` trait：增加 `typed_runtime_type()`、`binary_name()` 方法
- [x] 重构 `AgentRuntimeInfo` 增加 `runtime_category`、`runtime_type` (enum)、`binary_name` 字段
- [x] 重构 `ClaudeCodeRuntime` 适配新 trait 签名
- [x] 保持现有 claude.rs 功能不变，只扩展接口

### 3. Codex Runtime 框架
- [x] 创建 `runtime/codex.rs` 框架代码
- [x] 实现 CLI 检测逻辑（检查 `codex` binary）
- [x] 实现 `AgentRuntime` trait 的基本方法
- [x] 实现 `execute()` — 包装 codex CLI 的 stdin/stdout 流式交互

### 4. Runtime Registry 扩展
- [x] 注册 CodexRuntime 到默认 registry
- [x] `get_runtime_by_type()` 根据类型返回 runtime 实例
- [x] `AgentRuntimeInfo` 增加 binary_name、runtime_type enum 元信息

### 5. IPC Commands 更新
- [x] `create_agent` 增加 `runtime_type` 参数（通过 CreateAgentRequest）
- [x] 新增 `get_runtime_info` command 获取单个 runtime 详情
- [x] `get_agent_runtime_status` 使用 agent 的 runtime_type 查找对应 runtime
- [x] 前端类型定义同步更新 (`src/types.ts`)
- [x] IPC wrapper 更新 (`src/lib/ipc.ts`)

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 拆分自 feat-agent-runtime-select |
| 2026-04-10 | All tasks completed | 63 tests passing, cargo check clean |
