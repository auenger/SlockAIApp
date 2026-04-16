# Tasks: feat-lan-a2a-access

## Task Breakdown

### 1. TCP 服务循环
- [ ] 在 `handler.rs` 中新增 `run_adapter_server_loop()` 函数
- [ ] 实现 accept 循环（后台线程）
- [ ] 实现 graceful shutdown（AtomicBool + shutdown channel）
- [ ] 支持并发连接（每个连接独立线程或 thread pool）
- [ ] 错误处理与日志记录

### 2. Tauri Commands
- [ ] 新建 `src-tauri/src/commands/a2a_server.rs`
- [ ] 实现 `start_a2a_server(port)` command
- [ ] 实现 `stop_a2a_server()` command
- [ ] 实现 `get_a2a_server_status()` command
- [ ] 实现 `get_local_ip_addresses()` command
- [ ] 在 `lib.rs` 中注册 commands
- [ ] Tauri State 管理 server lifecycle

### 3. 前端 UI
- [ ] 新建 `src/components/settings/LanAccessPanel.tsx`
- [ ] Toggle 开关组件（启用/禁用 LAN）
- [ ] 端口输入框（默认 7878）
- [ ] 状态指示灯（running / stopped / error）
- [ ] 本机 IP 展示列表
- [ ] 复制连接地址按钮
- [ ] 新建 `src/lib/useLanServer.ts` hook
- [ ] 集成到 Settings 面板

### 4. 默认配置调整
- [ ] `A2AServerConfig::new()` 默认 host 改为 `0.0.0.0`
- [ ] 确保现有测试不受影响

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-16 | Feature created | Waiting for development |
