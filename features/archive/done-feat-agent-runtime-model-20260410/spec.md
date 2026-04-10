# Feature: feat-agent-runtime-model Agent Runtime 数据模型 & 泛化 Runtime Trait

## Basic Information
- **ID**: feat-agent-runtime-model
- **Name**: Agent Runtime 数据模型 & 泛化 Runtime Trait
- **Priority**: 75
- **Size**: S
- **Dependencies**: feat-data-storage
- **Parent**: feat-agent-runtime-select
- **Children**: feat-agent-runtime-ui, feat-agent-runtime-exec
- **Created**: 2026-04-10

## Description

为 Agent 系统建立 runtime 类型数据模型，并泛化 Rust 侧的 `AgentRuntime` trait，使其能支持 Claude Code、Codex、Gemini 等多种 agent client。这是整个 runtime 选择功能的后端基础，后续 UI 和执行层都依赖此模块。

参考 `reference/AINative/` 中的 runtime 架构方案：
- 统一的 `AgentRuntime` trait（已完成基础版）
- Session-based 执行模式：`runtime_session_start` → `runtime_execute` → `agent://chunk`
- Smart Routing：根据任务类型和 runtime 可用性智能选择
- Pipeline Engine：多阶段流水线执行

## User Value Points

1. **泛化 Runtime 抽象层**: 一个统一的 trait 接口，任何 CLI/API agent client 都能接入
2. **Agent-Runtime 绑定数据模型**: 每个 Agent 明确关联到某个 runtime 类型，对话时自动使用

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/mod.rs` — 现有 `AgentRuntime` trait 定义
- `src-tauri/src/runtime/claude.rs` — Claude Code runtime 实现
- `src-tauri/src/runtime/registry.rs` — Runtime 注册与检测
- `src-tauri/src/workspace/manager.rs` — Agent workspace 管理
- `src-tauri/src/commands/agent.rs` — Agent IPC commands
- `reference/AINative/neuro-syntax-ide/src-tauri/src/lib.rs` — 参考 trait 设计

### Related Documents
- `reference/AINative/module_5_ai_orchestration.md` — AI Agent Kernel 架构
- `project-context.md` — 项目架构：Agent Runtime 抽象层

### Related Features
- `feat-data-storage` (依赖) — 需要数据存储层来持久化 agent 的 runtime_type
- `feat-agent-runtime-ui` (子功能) — UI 层 runtime 选择器
- `feat-agent-runtime-exec` (子功能) — 多 runtime 对话执行

## Technical Solution

### 1. Agent 数据模型扩展

在 Agent 配置中增加 `runtime_type` 字段：

```rust
// workspace/manager.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeType {
    ClaudeCode,   // Claude Code CLI
    Codex,        // OpenAI Codex CLI
    Gemini,       // Google Gemini CLI/API
    Custom(String), // 可扩展的自定义 runtime
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub name: String,
    pub emoji: String,
    pub avatar: Option<String>,
    pub enabled: bool,
    pub runtime_type: RuntimeType,  // 新增
    pub system_prompt: Option<String>,  // 新增
    pub created_at: String,
    pub updated_at: String,
}
```

### 2. 泛化 Runtime Trait

扩展现有 `AgentRuntime` trait，支持多 runtime：

```rust
// runtime/mod.rs
#[async_trait]
pub trait AgentRuntime: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn runtime_type(&self) -> RuntimeType;
    fn capabilities(&self) -> Vec<AgentCapability>;

    // 生命周期
    async fn health_check(&self) -> Result<RuntimeHealth>;
    async fn detect() -> Result<Option<Box<dyn AgentRuntime>>> where Self: Sized;

    // Session 管理
    async fn create_session(&self, config: SessionConfig) -> Result<Session>;
    async fn resume_session(&self, session_id: &str) -> Result<Session>;

    // 执行
    async fn execute(&self, params: ExecuteParams) -> Result<ExecuteResult>;

    // CLI 检测
    fn binary_name(&self) -> &str;       // e.g. "claude", "codex"
    fn install_hint(&self) -> &str;      // 安装提示
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub workspace_dir: Option<String>,
    pub system_prompt: Option<String>,
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteParams {
    pub session_id: Option<String>,
    pub message: String,
    pub context: Option<Vec<ContextMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub session_id: String,
    pub response: String,
    pub tool_calls: Vec<ToolCall>,
    pub tokens_used: Option<TokenUsage>,
}
```

### 3. Codex Runtime 框架

```rust
// runtime/codex.rs
pub struct CodexRuntime {
    binary_path: Option<PathBuf>,
    version: Option<String>,
}

impl AgentRuntime for CodexRuntime {
    fn id(&self) -> &str { "codex" }
    fn name(&self) -> &str { "OpenAI Codex" }
    fn runtime_type(&self) -> RuntimeType { RuntimeType::Codex }
    fn binary_name(&self) -> &str { "codex" }
    fn install_hint(&self) -> &str { "npm install -g @openai/codex" }
    // ... 其他实现
}
```

### 4. Runtime Registry 扩展

```rust
// runtime/registry.rs
impl RuntimeRegistry {
    pub async fn detect_all() -> Vec<RuntimeInfo> {
        let runtimes: Vec<Box<dyn AgentRuntime>> = vec![
            Box::new(ClaudeCodeRuntime::detect().await?),
            Box::new(CodexRuntime::detect().await?),
            // 未来可插拔
        ];
        // ...
    }

    pub fn get_runtime(runtime_type: &RuntimeType) -> Option<Box<dyn AgentRuntime>> {
        // 根据 runtime_type 返回对应实现
    }
}
```

### 5. IPC Command 扩展

```rust
// commands/agent.rs
#[tauri::command]
pub async fn create_agent(
    name: String,
    emoji: String,
    runtime_type: RuntimeType,  // 新增参数
    // ...
) -> Result<AgentSummary, String> { ... }

#[tauri::command]
pub async fn list_runtimes() -> Result<Vec<RuntimeInfo>, String> { ... }

#[tauri::command]
pub async fn get_runtime_info(
    runtime_type: RuntimeType
) -> Result<RuntimeInfo, String> { ... }
```

## Acceptance Criteria (Gherkin)

### User Story
As a developer, I want the backend to support multiple agent runtime types so that agents can be created with different AI backends.

### Scenarios

```gherkin
Scenario: Agent config stores runtime type
  Given an agent is created with runtime_type "claude_code"
  When the agent config is saved to workspace
  Then the config file contains runtime_type: "claude_code"

Scenario: Runtime detection scans all supported CLIs
  Given Claude Code CLI is installed at /usr/local/bin/claude
  And Codex CLI is not installed
  When runtime detection runs
  Then ClaudeCodeRuntime status is "available"
  And CodexRuntime status is "not-installed"

Scenario: Agent config defaults to Claude Code runtime
  Given a new agent is created without specifying runtime_type
  Then the agent's runtime_type defaults to "claude_code"
```

### General Checklist
- [x] AgentConfig 包含 runtime_type 字段
- [x] RuntimeType enum 支持至少 Claude Code、Codex、Gemini
- [x] AgentRuntime trait 支持 session 管理、执行、健康检查
- [x] RuntimeRegistry 支持 detect_all() 和 get_runtime()
- [x] IPC commands 支持按 runtime_type 创建 agent
- [x] 向后兼容：已有 agent 默认绑定 Claude Code runtime

## Merge Record

- **Completed**: 2026-04-10T18:45:00+08:00
- **Merged Branch**: feature/feat-agent-runtime-model
- **Merge Commit**: a0d0ef4
- **Archive Tag**: feat-agent-runtime-model-20260410
- **Conflicts**: None
- **Verification**: All 63 Rust tests passed, 3/3 Gherkin scenarios verified
- **Files Changed**: 12 (1 new, 11 modified)
- **Duration**: ~45min
