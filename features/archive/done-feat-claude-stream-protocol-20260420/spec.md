# Feature: feat-claude-stream-protocol Claude Code Stream JSON 协议增强

## Basic Information
- **ID**: feat-claude-stream-protocol
- **Name**: Claude Code Stream JSON 协议增强（全自动执行 + MCP 注入）
- **Priority**: 85
- **Size**: M
- **Dependencies**: none
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-20

## Description

借鉴 Multica 的 Claude Code 通信方案，增强 CLI Runtime 与 Claude Code 子进程的通信协议，实现全自动执行能力。

核心改动：
1. 添加 `--input-format stream-json` 启用 stdin JSON 协议，支持 `control_response` 自动批准工具调用
2. 添加 `--permission-mode bypassPermissions` 实现全自动执行，避免 CLI 弹出权限确认
3. 添加 `--strict-mcp-config` 支持注入受控 MCP 配置，控制 Agent 的 MCP 服务访问范围

## User Value Points

### VP1: 全自动执行 (Autonomous Execution)
用户不再需要手动批准 Claude Code 的工具调用权限（文件读写、命令执行等），Agent 可以完全自主执行任务。

### VP2: MCP 服务访问控制
通过 MCP 配置注入，可以精确控制 Agent 能访问哪些 MCP 服务，提高安全性。

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/claude.rs` — Claude Code CLI Runtime 主实现
  - `ProcessHandle` — Thread 模式长连接进程管理
  - `execute_thread_mode()` — Thread 模式执行入口
  - `execute_channel_mode()` — Channel 模式执行入口
  - `build_command()` — CLI 参数构建（需修改）
  - `read_stdout_line()` — stdout 解析（需扩展解析 control_request）
- `src-tauri/src/runtime/mod.rs` — AgentRuntime trait 定义
- `src-tauri/src/runtime/a2a/adapter/claude_adapter.rs` — A2A Claude 适配器

### Related Documents
- Multica `server/pkg/agent/claude.go` — Claude Code 通信协议参考
- Multica `server/pkg/agent/agent.go` — Backend 接口定义

### Related Features
- `feat-claude-control-protocol` (已完成) — Control Protocol Runtime 基础
- `feat-claude-runtime` (已完成) — Claude Code Runtime 基础

## Technical Solution

### 1. `--input-format stream-json`

在 `build_command()` 中为 Thread 模式添加 `--input-format stream-json` 参数。这使得 stdin 接受 JSON 格式消息（我们已在使用 JSON 格式写 stdin，显式声明可启用 control_response 支持）。

```rust
// claude.rs - build_command() 修改
fn build_command(...) -> Command {
    let mut cmd = Command::new(exec_path);
    cmd.arg("-p")
        .arg("--output-format").arg("stream-json")
        .arg("--input-format").arg("stream-json")  // NEW
        .arg("--verbose");
    // ...
}
```

### 2. `--permission-mode bypassPermissions`

在 CLI 参数中添加，使 Claude Code 自动批准所有工具调用：

```rust
cmd.arg("--permission-mode").arg("bypassPermissions");
```

### 3. `control_response` 自动批准

在 stdout 解析循环中，新增对 `control_request` 类型消息的处理。当 Claude Code 请求权限批准时，自动回复 `control_response`：

```rust
// 解析 control_request
"control_request" => {
    // 从 stdin 写入 control_response 自动批准
    let response = json!({
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": msg.request_id,
            "response": {
                "behavior": "allow",
                "updatedInput": msg.input
            }
        }
    });
    stdin_writer.write_all(format!("{}\n", response).as_bytes());
}
```

注意：即使设置了 `bypassPermissions`，添加 `control_response` 处理作为双保险仍有价值——某些场景下 Claude Code 仍可能发送 control_request（如 MCP 工具首次调用）。

### 4. `--strict-mcp-config`

在 Agent 配置中添加可选的 `mcp_config` 字段，执行时写入临时文件并通过 `--mcp-config` 传入：

```rust
// Agent 配置扩展
pub struct AgentConfig {
    // ... existing fields
    pub mcp_config: Option<serde_json::Value>,  // NEW
}

// 执行时
if let Some(mcp_config) = &agent.mcp_config {
    let temp_path = write_temp_mcp_config(mcp_config)?;
    cmd.arg("--mcp-config").arg(&temp_path)
       .arg("--strict-mcp-config");
}
```

## Acceptance Criteria (Gherkin)

### VP1: 全自动执行

```gherkin
Feature: Claude Code 全自动执行

Scenario: Agent 执行文件读写操作无需人工批准
  Given 一个配置了 Claude Code Runtime 的 Agent
  And Agent 的 permission_mode 设置为 bypassPermissions
  When 用户在 Thread 中发送 "读取 src/main.rs 并修改第 10 行"
  Then Claude Code 自动执行文件读取和修改
  And 不弹出任何权限确认对话框
  And 操作结果通过流式返回给用户

Scenario: Agent 执行 shell 命令无需人工批准
  Given 一个配置了 Claude Code Runtime 的 Agent
  When 用户在 Thread 中发送 "运行 npm test"
  Then Claude Code 自动执行 shell 命令
  And 命令输出通过流式返回给用户

Scenario: control_request 自动批准作为 fallback
  Given Claude Code 运行中触发了 control_request
  When Runtime 收到 control_request 类型的消息
  Then 自动通过 stdin 发送 control_response 批准
  And 执行不中断
```

### VP2: MCP 服务访问控制

```gherkin
Feature: MCP 配置注入

Scenario: Agent 配置了 MCP config 后执行时注入
  Given 一个 Agent 配置了 mcp_config 包含 filesystem server
  When 用户触发该 Agent 执行任务
  Then Claude Code 使用 --mcp-config 加载指定的 MCP 服务
  And 使用 --strict-mcp-config 限制只访问配置的服务

Scenario: 未配置 MCP config 时不添加参数
  Given 一个 Agent 未配置 mcp_config
  When 用户触发该 Agent 执行任务
  Then Claude Code 启动时不包含 --mcp-config 参数
  And Agent 使用默认的 MCP 配置（如果有）
```

### General Checklist
- [x] Thread 模式和 Channel 模式均支持新参数
- [x] `--input-format stream-json` 正确启用 stdin JSON 协议
- [x] `--permission-mode bypassPermissions` 全自动执行
- [x] `control_response` 自动批准 fallback 正常工作
- [x] `--strict-mcp-config` 注入受控 MCP 配置
- [x] MCP config 临时文件在进程结束后清理
- [x] 不影响现有 A2A adapter 的 Claude Code 通信
- [x] 向后兼容：不破坏未配置 MCP 的 Agent

---

## Merge Record

- **Completed**: 2026-04-20T11:00:00+08:00
- **Merged Branch**: feature/feat-claude-stream-protocol
- **Merge Commit**: (merged to main via --no-ff)
- **Archive Tag**: feat-claude-stream-protocol-20260420
- **Conflicts**: none
- **Verification**: PASS (code analysis, all 5 Gherkin scenarios verified)
- **Stats**: 12 files changed, 536 insertions(+), 9 deletions(-), 8 source files modified
