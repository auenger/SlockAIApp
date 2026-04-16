# Tasks: feat-lan-a2a-bridge

## Task Breakdown

### Stage 1: 后端 Bridge 基础设施

#### 1. Cargo.toml Feature Flags + Binary Target
- [x] 添加 `[features]` default = ["tauri-app"], tauri-app = ["dep:tauri", "dep:tauri-plugin-log", "dep:rusqlite", "dep:keyring"]
- [x] 添加 `[[bin]] name = "az-bridge" path = "src/bin/az_bridge.rs"`
- [x] 添加 `toml = "0.8"` 依赖
- [x] 验证 `cargo check --bin az-bridge --no-default-features` 可编译

#### 2. lib.rs cfg Gate
- [x] 用 `#[cfg(feature = "tauri-app")]` 包裹 `commands`, `storage`, `task_engine` 模块声明
- [x] 用 `#[cfg(feature = "tauri-app")]` 包裹 `run()` 函数和 Tauri imports
- [x] 添加 `pub mod bridge;`（无条件）
- [x] 验证双模式编译通过

#### 3. Bridge 模块骨架 + 配置
- [x] 创建 `src-tauri/src/bridge/mod.rs`
- [x] 创建 `src-tauri/src/bridge/config.rs`
- [x] 实现 `BridgeConfig` struct（workspace_root, bind, port, name）
- [x] 实现 CLI 参数解析（clap）
- [x] 实现 TOML 配置文件解析（默认 ~/.agentszone/bridge.toml）
- [x] 配置优先级：CLI > TOML > 默认值

#### 4. BridgeServer 核心
- [x] 创建 `src-tauri/src/bridge/server.rs`
- [x] 实现 `BridgeServer` struct（AdapterServer + AgentManager）
- [x] 实现 workspace 初始化 + agent 加载
- [x] 注册标准 A2A handlers（复用 register_adapter_handlers）
- [x] 注册 bridge.* 扩展 handlers
- [x] AgentCard 增强（添加 bridge.* 到 supported_operations）

#### 5. Bridge 扩展协议 Handlers
- [x] 创建 `src-tauri/src/bridge/handlers.rs`
- [x] 实现 `bridge.getWorkspaceInfo` handler
- [x] 实现 `bridge.getAgents` handler
- [x] 实现 `bridge.listFiles` handler（含路径遍历防护）
- [x] 实现 `bridge.readFile` handler（含路径遍历防护）
- [x] 单元测试：每个 handler 的正常/错误场景

#### 6. 独立 Binary 入口
- [x] 创建 `src-tauri/src/bin/az_bridge.rs`
- [x] 实现 main()：配置加载 → BridgeServer 创建 → TCP accept loop
- [x] 打印启动信息（IP、端口、agent 数量）
- [x] Ctrl+C 优雅关闭（复用 cli.rs 的信号处理模式）

### Stage 2: 前端集成

#### 7. TypeScript 类型 + Bridge HTTP 调用
- [x] 添加 `BridgeWorkspaceInfo`, `BridgeAgent`, `BridgeFileEntry` 到 types.ts
- [x] 添加 bridge HTTP 调用函数到 useBridgeWorkspace hook

#### 8. useBridgeWorkspace Hook
- [x] 创建 `src/lib/useBridgeWorkspace.ts`
- [x] 检测远程连接是否支持 bridge.* 操作
- [x] 获取 bridge.getWorkspaceInfo + bridge.getAgents
- [x] 缓存 + 轮询更新机制

#### 9. BridgeWorkspacePanel UI
- [x] 创建 `src/components/settings/BridgeWorkspacePanel.tsx`
- [x] 在 RemoteConnectionsPanel 中条件渲染
- [x] 显示远程 agent 卡片列表（emoji + name + runtime type）
- [x] 文件浏览器（只读，调用 bridge.listFiles/readFile）

### Stage 3: 测试 + 文档

#### 10. 测试覆盖
- [x] BridgeConfig 解析单元测试
- [x] Bridge handlers 单元测试（正常 + 错误场景）
- [x] 路径遍历安全测试
- [x] Feature flag 编译测试（双模式通过）
- [x] 208 tests passed, 0 failed

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-16 | Feature created | 初始 spec (轻量二进制) |
| 2026-04-17 | Spec 重写 | 重新定义为远程 Workspace 网关，11 tasks |
| 2026-04-17 | 全部实现完成 | 10/10 tasks 完成, 208 tests passed |
