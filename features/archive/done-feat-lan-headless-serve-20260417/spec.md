# Feature: feat-lan-headless-serve Headless A2A Server CLI 模式

## Basic Information
- **ID**: feat-lan-headless-serve
- **Name**: Headless A2A Server CLI 模式（方式2）
- **Priority**: 80
- **Size**: S
- **Dependencies**: feat-lan-a2a-access
- **Parent**: feat-lan-a2a-server
- **Children**: 
- **Created**: 2026-04-16

## Description

为 AgentsZone 添加 headless CLI 模式，用户无需打开桌面 GUI 即可运行 A2A Server。通过 `agentszone serve --port 7878 --bind 0.0.0.0` 命令启动后台服务，让其他设备通过 A2A 协议访问本机的 Claude Code。

复用 feat-lan-a2a-access 中实现的 `AdapterServer` + TCP 服务循环代码，Tauri v2 支持通过参数控制是否显示窗口。

## User Value Points

### V1: 无 GUI 运行 A2A Server
在终端中一条命令启动 A2A 服务，不需要启动完整的桌面应用。适合：
- 服务器/无头机器
- 后台常驻服务
- 快速测试

## Context Analysis

### Reference Code
- `src-tauri/src/main.rs` 或 `src-tauri/src/lib.rs` — Tauri 入口，需添加 CLI 参数解析
- `src-tauri/src/runtime/a2a/adapter/handler.rs` — run_adapter_server_loop（feat-lan-a2a-access 实现）
- `src-tauri/src/runtime/a2a/adapter/claude_adapter.rs` — ClaudeCodeAdapter
- `src-tauri/Cargo.toml` — 可能需要添加 `clap` 依赖

### Related Features
- feat-lan-a2a-access — TCP 服务循环 + Tauri commands（前置依赖）

## Technical Solution

### 1. CLI 参数解析

使用 `clap` crate 添加子命令：

```bash
# 正常启动 GUI
agentszone

# Headless 模式
agentszone serve --port 7878 --bind 0.0.0.0
agentszone serve -p 7878 -b 0.0.0.0
```

### 2. 双模式入口

在 `main.rs` / `lib.rs` 中：
- 检测 `serve` 子命令 → 跳过 Tauri Builder，直接运行 TCP 服务循环
- 无子命令 → 正常启动 Tauri GUI 应用

### 3. Graceful Shutdown

- 监听 Ctrl+C (SIGINT/SIGTERM)
- 优雅关闭 TCP 服务
- 打印服务状态信息

### 4. 启动信息

```
$ agentszone serve --port 7878
[AgentsZone A2A Server] Starting...
[AgentsZone A2A Server] Listening on 0.0.0.0:7878
[AgentsZone A2A Server] Agent: Claude Code (streaming, tool_use, sessions)
[AgentsZone A2A Server] Local IPs: 192.168.1.10, 10.0.0.5
[AgentsZone A2A Server] Press Ctrl+C to stop
```

## Acceptance Criteria (Gherkin)

### Scenario 1: Headless 启动与连接

```gherkin
Given 电脑 B 已安装 AgentsZone CLI
When 用户在终端运行 "agentszone serve --port 7878"
Then 终端显示 "Listening on 0.0.0.0:7878" 和本机 IP
And 电脑 A 可以通过 "http://192.168.1.20:7878/a2a" 连接
And 电脑 A 获取到电脑 B 的 AgentCard
And 电脑 A 可以发送消息并获得响应
```

### Scenario 2: 优雅关闭

```gherkin
Given agentszone serve 正在运行
And 有一个活跃的远程连接
When 用户按下 Ctrl+C
Then 服务停止接受新连接
And 等待进行中的请求完成（超时 5 秒）
Then 终端显示 "Server stopped"
```

### Scenario 3: 端口冲突提示

```gherkin
Given 端口 7878 已被占用
When 用户运行 "agentszone serve --port 7878"
Then 终端显示错误 "Failed to bind to 0.0.0.0:7878: Address already in use"
And 进程以非零退出码退出
```

### General Checklist
- [x] 不影响 GUI 模式的正常启动
- [x] CLI 参数有 --help 输出
- [x] 日志输出到 stdout/stderr
- [x] 复用 feat-lan-a2a-access 的核心代码

## Merge Record

- **Completed**: 2026-04-17T04:30:00+08:00
- **Merged Branch**: feature/feat-lan-headless-serve
- **Merge Commit**: 80dc1a2
- **Archive Tag**: feat-lan-headless-serve-20260417
- **Conflicts**: None
- **Verification**: PASS (258/258 tests, 3/3 Gherkin scenarios)
- **Duration**: ~30 minutes
- **Files Changed**: 10 (6 new, 4 modified)
