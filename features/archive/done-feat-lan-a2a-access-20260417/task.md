# Tasks: feat-lan-a2a-access

## Task Breakdown

### 1. TCP 服务循环
- [x] 在 `handler.rs` 中新增 `run_adapter_server_loop()` 函数
- [x] 实现 accept 循环（后台线程）
- [x] 实现 graceful shutdown（AtomicBool + shutdown channel）
- [x] 支持并发连接（每个连接独立线程）
- [x] 错误处理与日志记录

### 2. Tauri Commands
- [x] 新建 `src-tauri/src/commands/a2a_server.rs`
- [x] 实现 `start_a2a_server(port)` command
- [x] 实现 `stop_a2a_server()` command
- [x] 实现 `get_a2a_server_status()` command
- [x] 实现 `get_local_ip_addresses()` command
- [x] 在 `lib.rs` 中注册 commands
- [x] Tauri State 管理 server lifecycle

### 3. 前端 UI
- [x] 新建 `src/components/settings/LanAccessPanel.tsx`
- [x] Toggle 开关组件（启用/禁用 LAN）
- [x] 端口输入框（默认 7878）
- [x] 状态指示灯（running / stopped / error）
- [x] 本机 IP 展示列表
- [x] 复制连接地址按钮
- [x] 新建 `src/lib/useLanServer.ts` hook
- [x] 集成到 Settings 面板

### 4. 默认配置调整
- [x] `A2AServerConfig::new()` 默认 host 改为 `0.0.0.0`
- [x] 确保现有测试不受影响

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-16 | Feature created | Waiting for development |
| 2026-04-17 | All tasks implemented | TCP loop + Tauri commands + UI + config change |
