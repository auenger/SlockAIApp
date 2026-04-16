# Feature: feat-lan-a2a-access LAN A2A 服务端 + GUI 开关

## Basic Information
- **ID**: feat-lan-a2a-access
- **Name**: LAN A2A 服务端 + GUI 开关（方式1）
- **Priority**: 80
- **Size**: M
- **Dependencies**: feat-a2a-adapter (已完成), feat-a2a-remote-client (已完成)
- **Parent**: feat-lan-a2a-server
- **Children**: 
- **Created**: 2026-04-16

## Description

补全 A2A TCP Server 的实际运行能力，让 AgentsZone 桌面应用可以接收局域网内其他设备的 A2A 协议请求。在 Settings 面板中提供 "Enable LAN Access" 开关，用户可一键启用/禁用局域网访问。

当前 `start_tcp_listener()` 仅 bind 并返回 TcpListener，没有 accept 循环。本 feature 补全服务循环、Tauri commands 和前端 UI。

## User Value Points

### V1: A2A TCP Server 运行能力
补全 TCP accept 循环，让 AdapterServer 能真正接收和响应 HTTP JSON-RPC 请求。支持并发连接、优雅关闭。

### V2: LAN Access 开关 UI
在 Settings 面板中添加局域网访问控制：
- Enable/Disable 开关
- 端口配置（默认 7878）
- 当前状态显示（listening / stopped / error）
- 本机 IP 展示（方便分享给其他设备）

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/a2a/adapter/handler.rs` — AdapterServer + start_tcp_listener（需补全服务循环）
- `src-tauri/src/runtime/a2a/server.rs` — A2AServer + A2AServerConfig（需改默认 host）
- `src-tauri/src/runtime/a2a/adapter/claude_adapter.rs` — ClaudeCodeAdapter（已完整）
- `src-tauri/src/commands/remote_connection.rs` — 远程连接 CRUD（已有）
- `src/components/settings/RemoteConnectionsPanel.tsx` — 远程连接 UI（已有）

### Related Features
- feat-a2a-adapter — 本地 Runtime → A2A Server Adapter（已完成）
- feat-a2a-remote-client — 远程 A2A Client（已完成）
- feat-a2a-multi-agent — 多 Agent 协作（已完成）

## Technical Solution

### 1. TCP 服务循环 (`handler.rs`)

新增 `run_adapter_server_loop()` 函数：

```
AdapterServer + ListenerConfig
  → spawn 后台线程
  → loop { listener.accept() → handle_tcp_connection() }
  → 支持 graceful shutdown via AtomicBool + channel
```

关键设计：
- 后台线程运行 accept 循环
- 每个 connection 在独立线程处理（或 thread pool）
- `Arc<AtomicBool>` 控制 shutdown
- `mpsc::Sender<()>` 通知 shutdown 完成

### 2. Tauri Commands (`commands/a2a_server.rs`)

```rust
#[tauri::command]
fn start_a2a_server(port: u16) -> Result<ServerInfo, String>

#[tauri::command]
fn stop_a2a_server() -> Result<(), String>

#[tauri::command]
fn get_a2a_server_status() -> Result<ServerStatus, String>

#[tauri::command]
fn get_local_ip_addresses() -> Result<Vec<String>, String>
```

ServerInfo 包含：port、local_ip、agent_card URL。
使用 Tauri State 管理 server lifecycle。

### 3. 前端 UI

在 Settings 中新增 "LAN Access" 面板：
- Toggle 开关（启用/禁用）
- 端口输入框（默认 7878）
- 状态指示灯（绿色=运行中，灰色=已停止）
- 本机 IP 显示（多网卡列出所有 IP）
- 复制按钮（复制 `http://{ip}:{port}/a2a` 给其他设备）

### 4. 默认绑定地址

`A2AServerConfig::new()` 默认 host 改为 `0.0.0.0`，让局域网可达。

## Acceptance Criteria (Gherkin)

### Scenario 1: 启用 LAN 访问后其他设备可连接

```gherkin
Given 用户 A 在 Settings 中打开了 "Enable LAN Access" 开关
And 端口设置为 7878
When 用户 B 在另一台电脑的 AgentsZone 中添加远程连接 "http://192.168.1.10:7878/a2a"
Then 连接测试成功
And 用户 B 可以看到用户 A 的 AgentCard
And 用户 B 可以通过该连接发送消息给 A 的 Claude Code Agent
And A 的 Claude Code 响应通过 SSE 流式返回给 B
```

### Scenario 2: 禁用 LAN 访问后连接被拒绝

```gherkin
Given 用户 A 的 LAN Access 处于开启状态
When 用户 A 关闭 "Enable LAN Access" 开关
Then TCP 服务停止监听
And 外部连接被拒绝（connection refused）
And Settings 面板显示状态为 "stopped"
```

### Scenario 3: 端口冲突处理

```gherkin
Given 端口 7878 已被其他程序占用
When 用户尝试启用 LAN Access
Then 显示错误提示 "端口 7878 已被占用"
And 用户可以更换端口后重试
```

### Scenario 4: 获取本机 IP 地址

```gherkin
Given 用户打开 LAN Access 设置面板
When 服务器处于运行状态
Then 面板显示本机所有局域网 IP（如 192.168.1.10, 10.0.0.5）
And 提供 "复制连接地址" 按钮，点击后复制 "http://192.168.1.10:7878/a2a"
```

### General Checklist
- [ ] TCP 服务不阻塞 UI 线程
- [ ] 优雅关闭：停止时不丢失进行中的请求
- [ ] 多个远程客户端可同时连接
- [ ] 日志记录：连接/断开/错误均有 log 输出
- [ ] 不影响现有本地 Runtime 功能
