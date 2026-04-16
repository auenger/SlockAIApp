# Feature: feat-lan-a2a-bridge 远程 Workspace 网关

## Basic Information
- **ID**: feat-lan-a2a-bridge
- **Name**: 远程 Workspace 网关（独立 A2A Bridge 二进制 + 远程 workspace 管理）
- **Priority**: 70
- **Size**: L
- **Dependencies**: feat-lan-a2a-access
- **Parent**: null
- **Children**:
- **Created**: 2026-04-16
- **Updated**: 2026-04-17

## Description

重新定义 A2A Bridge 为「远程 Workspace 网关」—— 不只是轻量二进制，而是让本地 AgentsZone 能感知和操作远程 Claude Code workspace 的桥梁。

通过 Feature Flags 在现有 crate 中编译出独立 `az-bridge` 二进制，复用 95% 纯 Rust A2A 模块（无 Tauri 依赖），扩展 A2A 协议支持远程 workspace 查询。

### 核心场景
1. 在远程服务器运行 `az-bridge`，暴露 Claude Code 为 A2A 端点
2. 本地 AgentsZone 连接远程 bridge，查看远程 workspace 的 agents、文件
3. 通过标准 A2A 协议与远程 agent 对话协作
4. 一键分发：`scp az-bridge user@server:~ && ./az-bridge`

### 设计原则
- **轻量化**: Feature Flags 排除 Tauri/WebView/SQLite，最小依赖
- **跨平台**: 纯 Rust，支持 Linux/macOS，可交叉编译
- **简单兼容**: 复用现有 A2A 模块，扩展不替换
- **渐进增强**: Phase 1 实现核心功能，Phase 2 提取为 Cargo workspace

## User Value Points

### V1: 独立轻量 Bridge 二进制
编译出不含 Tauri 的独立二进制，可在无 GUI 服务器运行，体积远小于完整 App。
- Feature Flags 隔离 Tauri 依赖
- 复用现有 A2A transport/adapter/bridge 模块
- 支持 CLI 参数和 TOML 配置文件

### V2: 远程 Workspace 协议
扩展 A2A JSON-RPC 协议，支持远程 workspace 查询：
- `bridge.getWorkspaceInfo` — workspace 元信息
- `bridge.getAgents` — 远程 agent 列表（身份、类型）
- `bridge.listFiles` — 远程 workspace 文件浏览
- `bridge.readFile` — 远程文件内容读取

### V3: 本地 AgentsZone 远程 Workspace 可视化
本地 AgentsZone 连接 bridge 后，在 UI 中展示远程 workspace 信息：
- 远程 agent 卡片（emoji + name + runtime type）
- 远程文件浏览器（只读）
- 连接状态感知（自动检测 bridge 支持能力）

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/a2a/adapter/handler.rs` — AdapterServer + handler 注册模式 + TCP accept loop
- `src-tauri/src/cli.rs` — Headless serve 入口，无 Tauri 依赖的纯 Rust 模式
- `src-tauri/src/workspace/manager.rs` — AgentManager，磁盘 agent 发现
- `src-tauri/src/workspace/agent.rs` — AgentWorkspace 目录管理
- `src-tauri/src/workspace/identity.rs` — AgentIdentity 身份数据
- `src-tauri/src/commands/mod.rs` — `list_workspace_dir`/`read_workspace_file` 路径安全模式
- `src-tauri/src/runtime/a2a/types.rs` — A2A 协议类型（纯数据，无 Tauri）
- `src-tauri/src/runtime/claude.rs` — Claude Code Runtime（纯进程管理）

### Related Features
- `feat-lan-a2a-access` (completed) — LAN A2A 服务端 + GUI 开关
- `feat-lan-headless-serve` (completed) — Headless CLI 模式
- `feat-a2a-adapter` (completed) — A2A Server Adapter
- `feat-a2a-transport` (completed) — A2A Transport 基础设施

### Architecture

```
┌─────────────────────┐         A2A JSON-RPC          ┌──────────────────────────┐
│  Local AgentsZone   │ ◄────────────────────────────► │  Remote az-bridge        │
│  (桌面 App)         │                                 │                          │
│  - 远程 Agent 卡片   │   sendMessage / streamMessage   │  - Claude Code Runtime   │
│  - 远程 Workspace   │   bridge.getWorkspaceInfo       │  - AgentManager          │
│  - 任务委托/协作     │   bridge.getAgents              │  - Workspace 目录管理     │
│                     │   bridge.listFiles               │  - TOML 配置              │
│                     │   bridge.readFile                │  - Graceful Shutdown      │
└─────────────────────┘                                 └──────────────────────────┘
```

### Cargo Feature Flags Strategy

```toml
[features]
default = ["tauri-app"]
tauri-app = ["dep:tauri", "dep:tauri-plugin-log", "dep:rusqlite", "dep:keyring"]

[[bin]]
name = "agentszone"
path = "src/main.rs"

[[bin]]
name = "az-bridge"
path = "src/bin/az_bridge.rs"
```

- `az-bridge` 用 `--no-default-features` 编译，排除 Tauri/SQLite/Keyring
- `lib.rs` 中 `commands`/`storage`/`task_engine` 模块用 `#[cfg(feature = "tauri-app")]` 门控
- `bridge` 模块无条件编译，只依赖 `runtime::a2a` + `workspace`

## Technical Solution

### 新增文件

```
src-tauri/src/
  bridge/
    mod.rs              # 模块根 — pub mod config/server/handlers
    config.rs           # BridgeConfig (CLI + TOML) + 验证
    server.rs           # BridgeServer = AdapterServer + AgentManager + bridge handlers
    handlers.rs         # 4 个 bridge.* JSON-RPC handlers
  bin/
    az_bridge.rs        # 独立 binary 入口 (parse args → run bridge)
```

### BridgeConfig
```rust
struct BridgeConfig {
    workspace_root: PathBuf,   // 默认 ~/.agentszone
    bind: String,              // 默认 "0.0.0.0"
    port: u16,                 // 默认 7878
    name: String,              // 默认 hostname
}
```

配置优先级：CLI args > TOML 文件 > 默认值
配置文件位置：`~/.agentszone/bridge.toml` 或 `--config` 指定

### BridgeServer
```rust
struct BridgeServer {
    adapter_server: Arc<AdapterServer>,   // 标准 A2A 服务
    agent_manager: Arc<Mutex<AgentManager>>,  // workspace 管理
    config: BridgeConfig,
}
```

初始化流程：
1. `AgentManager::new(workspace_root)` → `initialize_workspace()` → `load()`
2. `ClaudeCodeAdapter::new()` → `AdapterServer::new(adapter, agent_card)`
3. `register_adapter_handlers(config)` — 标准 A2A 方法
4. `register_bridge_handlers()` — 扩展 bridge.* 方法
5. `run_adapter_server_loop(server, tcp_config)` — TCP accept loop

### Bridge Handlers

| 方法 | 参数 | 返回 | 实现 |
|------|------|------|------|
| `bridge.getWorkspaceInfo` | `{}` | `{ workspace_root, total_agents, enabled_agents, active_agent_id }` | AgentManager 状态 |
| `bridge.getAgents` | `{}` | `{ agents: [{ agent_id, name, emoji, creature, vibe, runtime_type }] }` | AgentManager.list_agents() |
| `bridge.listFiles` | `{ agent_id, path? }` | `{ entries: [{ name, is_dir, size, modified }] }` | AgentWorkspace + fs::read_dir |
| `bridge.readFile` | `{ agent_id, file_path }` | `{ name, size, mime_type, content }` | AgentWorkspace + fs::read |

安全：路径遍历检查，确保请求路径在 workspace 基目录内。

### AgentCard 增强
Bridge 的 AgentCard 在 `supported_operations` 中添加：
```
"sendMessage", "streamMessage", "getTask", "cancelTask", "listTasks",
"bridge.getWorkspaceInfo", "bridge.getAgents", "bridge.listFiles", "bridge.readFile"
```
本地 AgentsZone 通过检测 `bridge.*` 操作判断远程是否为 bridge 端点。

### 前端变更

**新类型** (`src/types.ts`):
```typescript
interface BridgeWorkspaceInfo {
  workspace_root: string;
  total_agents: number;
  enabled_agents: number;
  active_agent_id: string | null;
}

interface BridgeAgent {
  agent_id: string;
  name: string;
  emoji: string;
  creature: string;
  vibe: string;
  runtime_type: string;
}

interface BridgeFileEntry {
  name: string;
  is_dir: boolean;
  size: number;
  modified: number;
}
```

**新 Hook** (`src/lib/useBridgeWorkspace.ts`):
- 检测远程连接 AgentCard.supported_operations 是否包含 `bridge.*`
- 获取 workspace info + agent 列表
- 缓存 + 轮询更新

**新组件** (`src/components/settings/BridgeWorkspacePanel.tsx`):
- 在 RemoteConnectionsPanel 中条件渲染
- 显示远程 agent 卡片列表
- 文件浏览器（只读列表）

## Acceptance Criteria (Gherkin)

### V1: 独立 Bridge 二进制

```gherkin
Feature: Bridge Binary

  Scenario: 编译独立 bridge 二进制
    Given 项目源码已 clone
    When 运行 "cargo build --bin az-bridge --no-default-features"
    Then 编译成功，无 Tauri 相关链接错误
    And 生成可执行文件 az-bridge

  Scenario: 启动 bridge 服务
    Given az-bridge 二进制已编译
    When 运行 "./az-bridge --port 7878"
    Then bridge 启动并监听 0.0.0.0:7878
    And 打印启动信息（IP、端口、agent 数量）
    And Ctrl+C 可优雅关闭

  Scenario: TOML 配置文件加载
    Given ~/.agentszone/bridge.toml 存在且包含 port = 9090
    When 运行 "./az-bridge" （无 CLI 参数）
    Then bridge 使用端口 9090
    And CLI --port 参数可覆盖 TOML 配置

  Scenario: 标准 A2A 协议兼容
    Given az-bridge 正在运行
    When 发送 JSON-RPC getAgentCard 请求
    Then 返回包含 bridge.* 操作的 AgentCard
    When 发送 JSON-RPC sendMessage 请求
    Then 正常创建任务并执行
```

### V2: 远程 Workspace 协议

```gherkin
Feature: Bridge Workspace Protocol

  Scenario: 获取 workspace 信息
    Given az-bridge 正在运行且 workspace 有 3 个 agents
    When 发送 JSON-RPC bridge.getWorkspaceInfo 请求
    Then 返回 { total_agents: 3, enabled_agents: 3, active_agent_id: "..." }

  Scenario: 获取 agent 列表
    Given az-bridge 正在运行
    When 发送 JSON-RPC bridge.getAgents 请求
    Then 返回每个 agent 的身份信息（name, emoji, creature, vibe, runtime_type）

  Scenario: 浏览 workspace 文件
    Given az-bridge 正在运行
    When 发送 JSON-RPC bridge.listFiles { agent_id: "default" }
    Then 返回该 agent workspace 目录下的文件和子目录列表
    And 每个条目包含 name, is_dir, size, modified

  Scenario: 读取 workspace 文件
    Given az-bridge 正在运行
    When 发送 JSON-RPC bridge.readFile { agent_id: "default", file_path: "IDENTITY.md" }
    Then 返回文件内容和 MIME 类型

  Scenario: 路径遍历防护
    Given az-bridge 正在运行
    When 发送 JSON-RPC bridge.readFile { agent_id: "default", file_path: "../../etc/passwd" }
    Then 返回错误，拒绝访问 workspace 外的文件
```

### V3: 本地 AgentsZone 远程可视化

```gherkin
Feature: Remote Workspace Visualization

  Scenario: 自动检测 bridge 端点
    Given 本地 AgentsZone 已配置远程连接指向 bridge 端点
    When 获取远程 AgentCard 成功
    And AgentCard.supported_operations 包含 "bridge.getAgents"
    Then 自动获取远程 workspace 信息并显示

  Scenario: 显示远程 agent 列表
    Given 远程 bridge 连接已建立
    When 进入远程连接详情
    Then 显示远程 workspace 的 agent 卡片
    And 每个 agent 显示 emoji、名称、runtime 类型

  Scenario: 浏览远程文件
    Given 远程 bridge 连接已建立
    When 点击某个远程 agent 的文件浏览
    Then 显示该 agent workspace 的文件列表
    And 可点击文件查看内容（只读）
```

## Status: COMPLETED

### Merge Record
- **Completed**: 2026-04-17
- **Merged Branch**: feature/feat-lan-a2a-bridge
- **Merge Commit**: 67bd2e1
- **Archive Tag**: feat-lan-a2a-bridge-20260417
- **Conflicts**: none
- **Verification**: 12/12 Gherkin scenarios passed, 208 unit tests passed
- **Files Changed**: 18 (7 new, 11 modified)
- **Lines Added**: 1256
