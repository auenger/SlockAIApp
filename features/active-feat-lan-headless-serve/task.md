# Tasks: feat-lan-headless-serve

## Task Breakdown

### 1. CLI 参数解析
- [ ] 添加 `clap` 依赖到 Cargo.toml
- [ ] 定义 CLI 结构体（serve 子命令 + port/bind 参数）
- [ ] 实现 --help 输出

### 2. 双模式入口
- [ ] 修改 `main.rs` / `lib.rs` 检测 serve 子命令
- [ ] serve 模式：跳过 Tauri Builder，直接运行 TCP 服务
- [ ] GUI 模式：保持现有启动流程不变

### 3. 服务启动与信息输出
- [ ] 打印启动信息（listening address、agent card、local IPs）
- [ ] 初始化 ClaudeCodeAdapter + AdapterServer
- [ ] 调用 run_adapter_server_loop（复用 feat-lan-a2a-access）

### 4. Graceful Shutdown
- [ ] 监听 Ctrl+C（SIGINT）
- [ ] 触发 server shutdown
- [ ] 等待进行中请求完成（超时）
- [ ] 打印 "Server stopped" 信息

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-16 | Feature created | Waiting for feat-lan-a2a-access completion |
