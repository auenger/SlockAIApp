# az-bridge 使用与测试指南

az-bridge 是 AgentsZone 的独立轻量二进制，可在无 GUI 的远程服务器上运行，将本地 Claude Code workspace 暴露为 A2A 端点，供远程 AgentsZone 桌面端连接和协作。

## 1. 编译

```bash
cd src-tauri
cargo build --bin az-bridge --no-default-features
```

编译成功后二进制位于 `src-tauri/target/debug/az-bridge`。

Release 构建：
```bash
cargo build --bin az-bridge --no-default-features --release
# 二进制位于 src-tauri/target/release/az-bridge
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

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--port` | 7878 | 监听端口 |
| `--bind` | 0.0.0.0 | 绑定地址 |
| `--workspace` | ~/.agentszone | workspace 根目录 |
| `--config` | ~/.agentszone/bridge.toml | 配置文件路径 |

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
- 监听地址和端口
- 本机 IP 地址列表（方便远程连接）
- 已加载的 agent 数量
- Ctrl+C 优雅关闭提示

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

# SSH 登录远程服务器启动
ssh user@server
./az-bridge --port 7878
```

### 4.2 本地 AgentsZone 连接

1. 打开 AgentsZone 桌面 App
2. 进入 Settings → Remote Connections
3. 点击 Add，填入远程服务器 IP 和端口（如 `192.168.1.100:7878`）
4. 连接成功后：
   - 自动检测 `bridge.*` 操作支持
   - 显示远程 agent 卡片（emoji + name + runtime type）
   - 可浏览远程 workspace 文件（只读）
   - 可点击文件查看内容

### 4.3 发送消息协作

通过标准 A2A 协议与远程 agent 对话：

```bash
curl -X POST http://<remote-ip>:7878 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":7,"method":"sendMessage","params":{"message":"hello from local"}}'
```

## 5. API 参考

### 标准 A2A 方法

| 方法 | 说明 |
|------|------|
| `getAgentCard` | 获取 agent 身份和能力 |
| `sendMessage` | 发送消息（同步等待） |
| `streamMessage` | 流式消息 |
| `getTask` | 查询任务状态 |
| `cancelTask` | 取消任务 |
| `listTasks` | 列出任务 |

### Bridge 扩展方法

| 方法 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `bridge.getWorkspaceInfo` | `{}` | `{ workspace_root, total_agents, enabled_agents, active_agent_id }` | workspace 元信息 |
| `bridge.getAgents` | `{}` | `{ agents: [{ agent_id, name, emoji, creature, vibe, runtime_type }] }` | agent 列表 |
| `bridge.listFiles` | `{ agent_id, path? }` | `{ entries: [{ name, is_dir, size, modified }] }` | 文件浏览 |
| `bridge.readFile` | `{ agent_id, file_path }` | `{ name, size, mime_type, content }` | 文件读取 |

### 安全

- 所有文件操作限制在 workspace 基目录内
- 路径遍历检查：拒绝 `..`、绝对路径、反斜杠
- 文件读取使用 canonicalize + starts_with 二次验证

## 6. 建议测试顺序

1. **编译** — 确认无 Tauri 链接错误
2. **单元测试** — `cargo test --bin az-bridge --no-default-features`
3. **本地启动 + curl** — 测试各 JSON-RPC 方法
4. **路径安全** — 验证 `../../etc/passwd` 被拒绝
5. **两台机器连接** — 实际远程连接测试
6. **GUI 验证** — 在 AgentsZone 桌面端确认远程 workspace 面板显示
