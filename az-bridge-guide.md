# az-bridge 使用与测试指南

az-bridge 是 AgentsZone 的独立轻量二进制，可在无 GUI 的远程服务器上运行，将本地 Claude Code workspace 暴露为 A2A 端点，供远程 AgentsZone 桌面端连接和协作。

## 通信原理

### 整体架构

```text
┌─────────────────────────┐          HTTP/JSON-RPC          ┌──────────────────────────┐
│  本地 AgentsZone 桌面端   │ ◄────────────────────────────► │  远程 az-bridge          │
│  (Tauri + React)         │                                 │  (独立 Rust 二进制)       │
│                          │                                 │                          │
│  前端 fetch() ──────► Tauri Webview ──────► TCP ──────► TCP Listener              │
│       │                                          │         │       │                  │
│       │              跨域请求                      │         │  HTTP 解析 + CORS       │
│       │          (CORS 预检 OPTIONS)               │         │       │                  │
│       ▼                                          ▼         │       ▼                  │
│  BridgeWorkspacePanel                    119.119.118.146:7878                        │
│  useBridgeWorkspace()                     AdapterServer                              │
│  bridgeRpc()                              ├─ 标准 A2A handlers                       │
│                                           └─ bridge.* handlers                      │
│                                                     │                                  │
│                                                     ▼                                  │
│                                              AgentManager                             │
│                                              (workspace/agent 管理)                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### 通信流程

1. **前端发起请求**：`useBridgeWorkspace` hook 通过 `fetch()` 发送 HTTP POST

2. **Tauri Webview**：请求经 Tauri 内置 WebView 发出（受浏览器 CORS 策略约束）

3. **TCP 层**：az-bridge 监听 TCP 端口，接收 HTTP 请求

4. **HTTP 解析**：读取 request line + headers + body，解析出 JSON-RPC 消息

5. **CORS 处理**：

   * 浏览器先发 `OPTIONS` 预检请求

   * 服务端返回 `204 No Content` + `Access-Control-Allow-Origin: *`

   * 浏览器确认允许后再发实际 `POST` 请求

   * 服务端返回 `200 OK` + CORS 头 + JSON-RPC 响应

6. **JSON-RPC 路由**：根据 `method` 字段分发到对应 handler

### 协议层

```text
┌─────────────────────────────────────────┐
│ HTTP/1.1 (传输层)                         │
│  - POST / 请求                            │
│  - OPTIONS / 预检 (CORS)                  │
│  - Access-Control-Allow-Origin: *         │
├─────────────────────────────────────────┤
│ JSON-RPC 2.0 (协议层)                     │
│  - {"jsonrpc":"2.0","method":"...","id":1} │
│  - 标准 A2A 方法 + bridge.* 扩展方法        │
├─────────────────────────────────────────┤
│ 应用数据 (数据层)                          │
│  - AgentCard / BridgeWorkspaceInfo        │
│  - BridgeAgent[] / BridgeFileEntry[]      │
│  - BridgeFileContent                      │
└─────────────────────────────────────────┘
```

### CORS 方案

浏览器 `fetch()` 跨域请求必须经过 CORS 预检：

```text
浏览器                          az-bridge
  │                                │
  │──── OPTIONS / ────────────────►│  (预检请求)
  │                                │
  │◄─── 204 No Content ───────────│
  │     Access-Control-Allow-Origin: *
  │     Access-Control-Allow-Methods: POST, OPTIONS
  │     Access-Control-Allow-Headers: Content-Type
  │                                │
  │──── POST / (JSON-RPC) ────────►│  (实际请求)
  │                                │
  │◄─── 200 OK ───────────────────│
  │     Access-Control-Allow-Origin: *
  │     Content-Type: application/json
  │     {"jsonrpc":"2.0","result":{...},"id":1}
```

实现位置：`src-tauri/src/runtime/a2a/adapter/handler.rs` 的 `handle_tcp_connection` 函数。

### 数据流：Bridge Workspace 加载

```text
App 启动 → Settings → 点 Test → 连接成功 (status: online)
    │
    ▼
BridgeWorkspacePanel 渲染
    │
    ▼
useBridgeWorkspace.refresh()
    │
    ├── isBridgeEndpoint() — 检查 agent_card.supported_operations 是否含 bridge.*
    │
    ├── bridgeRpc('bridge.getWorkspaceInfo') ──► 返回 workspace 元信息
    │
    └── bridgeRpc('bridge.getAgents') ─────────► 返回 agent 列表
    │
    ▼
渲染 agent 卡片 → 点击 agent → listFiles(agent_id)
    │
    ▼
bridgeRpc('bridge.listFiles', {agent_id, path}) ──► 返回文件列表
    │
    ▼
点击文件 → readFile(agent_id, file_path)
    │
    ▼
bridgeRpc('bridge.readFile', {agent_id, file_path}) ──► 返回文件内容
```

## 1. 编译

### macOS (本地开发)

```bash
cd src-tauri
cargo build --bin az-bridge --no-default-features --release
# 二进制: target/release/az-bridge
```

### Windows (交叉编译)

```bash
# 安装工具链 (首次)
brew install zig
pip3 install cargo-zigbuild
rustup target add x86_64-pc-windows-gnu

# 编译
cd src-tauri
cargo zigbuild --bin az-bridge --no-default-features --release --target x86_64-pc-windows-gnu
# 二进制: target/x86_64-pc-windows-gnu/release/az-bridge.exe
```

## 2. 启动

### CLI 参数方式

```bash
./az-bridge --port 7878
```

完整参数：

```bash
./az-bridge --port 7878 --bind 0.0.0.0 --workspace ~/.agentszone
```

| 参数            | 默认值                       | 说明            |
| ------------- | ------------------------- | ------------- |
| `--port`      | 7878                      | 监听端口          |
| `--bind`      | 0.0.0.0                   | 绑定地址          |
| `--workspace` | ~/.agentszone             | workspace 根目录 |
| `--config`    | ~/.agentszone/bridge.toml | 配置文件路径        |

### TOML 配置文件方式

创建 `~/.agentszone/bridge.toml`：

```toml
port = 9090
bind = "0.0.0.0"
name = "my-remote-server"
workspace = "/home/user/.agentszone"
```

直接启动即可加载配置：

```bash
./az-bridge
```

配置优先级：CLI 参数 > TOML 文件 > 默认值。

启动后会打印：

* 监听地址和端口

* 本机 IP 地址列表（方便远程连接）

* 已加载的 agent 数量

* Ctrl+C 优雅关闭提示

## 3. 本地测试（单机）

### 3.1 启动服务

```bash
./az-bridge --port 7878
```

### 3.2 curl 测试 JSON-RPC 接口

**获取 AgentCard：**

```bash
curl -X POST http://localhost:7878 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAgentCard","params":{}}'
```

应返回包含 `bridge.*` 操作的 AgentCard。

**获取 Workspace 信息：**

```bash
curl -X POST http://localhost:7878 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"bridge.getWorkspaceInfo","params":{}}'
```

返回：workspace 路径、agent 总数、启用数、活跃 agent ID。

**获取 Agent 列表：**

```bash
curl -X POST http://localhost:7878 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"bridge.getAgents","params":{}}'
```

返回每个 agent 的身份信息（name, emoji, creature, vibe, runtime_type）。

**浏览文件：**

```bash
curl -X POST http://localhost:7878 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":4,"method":"bridge.listFiles","params":{"agent_id":"default"}}'
```

返回指定 agent workspace 目录下的文件和子目录列表。

**读取文件：**

```bash
curl -X POST http://localhost:7878 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":5,"method":"bridge.readFile","params":{"agent_id":"default","file_path":"IDENTITY.md"}}'
```

返回文件内容和 MIME 类型。

**路径遍历防护测试（应返回错误）：**

```bash
curl -X POST http://localhost:7878 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":6,"method":"bridge.readFile","params":{"agent_id":"default","file_path":"../../etc/passwd"}}'
```

应返回错误，拒绝访问 workspace 外的文件。

### 3.3 运行单元测试

```bash
cd src-tauri
cargo test --bin az-bridge --no-default-features
```

覆盖 9 个单元测试：config 解析 4 个 + handlers 路径安全 5 个。

## 4. 两台机器测试（真实场景）

### 4.1 远程服务器部署

```bash
# 编译 release 二进制（或交叉编译）
cd src-tauri
cargo build --bin az-bridge --no-default-features --release

# 传输到远程服务器
scp target/release/az-bridge user@server:~

# Windows 交叉编译
scp target/x86_64-pc-windows-gnu/release/az-bridge.exe user@windows-pc:D:\az-bridge\

# SSH 登录远程服务器启动
ssh user@server
./az-bridge --port 7878
```

### 4.2 本地 AgentsZone 连接

1. 打开 AgentsZone 桌面 App

2. 点击左下角 **设置按钮**（用户名右边）

3. 在弹窗中找到 **Remote Connections** 区域

4. 点 **+ Add**

5. 填写：

   * **Name**: 远程机器名称

   * **Endpoint URL**: `http://<远程IP>:7878`

   * **Authentication**: 选 **None**

6. 点 **Save** → 点 **Test** → 状态变 **Online**

7. 自动出现 **Bridge Workspace** 面板：

   * 显示远程 agent 卡片（emoji + name + runtime type）

   * 点击 agent 可浏览远程文件（只读）

   * 点击文件可查看内容

### 4.3 发送消息协作

通过标准 A2A 协议与远程 agent 对话：

```bash
curl -X POST http://<remote-ip>:7878 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":7,"method":"sendMessage","params":{"message":"hello from local"}}'
```

## 5. API 参考

### 标准 A2A 方法

| 方法              | 说明             |
| --------------- | -------------- |
| `getAgentCard`  | 获取 agent 身份和能力 |
| `sendMessage`   | 发送消息（同步等待）     |
| `streamMessage` | 流式消息           |
| `getTask`       | 查询任务状态         |
| `cancelTask`    | 取消任务           |
| `listTasks`     | 列出任务           |

### Bridge 扩展方法

| 方法                        | 参数                        | 返回                                                                      | 说明            |
| ------------------------- | ------------------------- | ----------------------------------------------------------------------- | ------------- |
| `bridge.getWorkspaceInfo` | `{}`                      | `{ workspace_root, total_agents, enabled_agents, active_agent_id }`     | workspace 元信息 |
| `bridge.getAgents`        | `{}`                      | `{ agents: [{ agent_id, name, emoji, creature, vibe, runtime_type }] }` | agent 列表      |
| `bridge.listFiles`        | `{ agent_id, path? }`     | `{ entries: [{ name, is_dir, size, modified }] }`                       | 文件浏览          |
| `bridge.readFile`         | `{ agent_id, file_path }` | `{ name, size, mime_type, content }`                                    | 文件读取          |

### 安全

* 所有文件操作限制在 workspace 基目录内

* 路径遍历检查：拒绝 `..`、绝对路径、反斜杠

* 文件读取使用 canonicalize + starts_with 二次验证

## 6. 常见问题

### Q: Windows 上启动后立即 stopped

**原因**：旧版本的 `recv_timeout(10s)` 导致 10 秒后自动退出。\
**修复**：v1.1+ 改为 `done_rx.recv()` 无限阻塞，只在 Ctrl+C 时退出。\
**文件**：`src-tauri/src/bridge/server.rs`

### Q: App 显示 "Load failed"

**原因**：浏览器 CORS 预检（OPTIONS）请求被服务端拒绝。\
**修复**：v1.1+ 在 TCP handler 中添加 OPTIONS 处理和 `Access-Control-Allow-Origin: *` 头。\
**文件**：`src-tauri/src/runtime/a2a/adapter/handler.rs`

### Q: `npm run tauri dev` 报 "could not determine which binary to run"

**原因**：Cargo.toml 有多个 [[bin]] target，未指定默认。\
**修复**：在 `[package]` 中添加 `default-run = "agentszone"`。\
**文件**：`src-tauri/Cargo.toml`

### Q: 找不到 Remote Connections 入口

**位置**：左下角用户名右边的设置按钮 → 弹窗中 LAN Access 下方 → Remote Connections。\
**文件**：`src/components/ApiKeyManager.tsx`

## 7. 建议测试顺序

1. **编译** — 确认无 Tauri 链接错误

2. **单元测试** — `cargo test --bin az-bridge --no-default-features`

3. **本地启动 + curl** — 测试各 JSON-RPC 方法

4. **路径安全** — 验证 `../../etc/passwd` 被拒绝

5. **两台机器连接** — 实际远程连接测试

6. **GUI 验证** — 在 AgentsZone 桌面端确认远程 workspace 面板显示

⠀