# Tasks: feat-lan-headless-serve

## Task Breakdown

### 1. CLI 参数解析
- [x] 添加 `clap` 依赖到 Cargo.toml
- [x] 定义 CLI 结构体（serve 子命令 + port/bind 参数）
- [x] 实现 --help 输出

### 2. 双模式入口
- [x] 修改 `main.rs` / `lib.rs` 检测 serve 子命令
- [x] serve 模式：跳过 Tauri Builder，直接运行 TCP 服务
- [x] GUI 模式：保持现有启动流程不变

### 3. 服务启动与信息输出
- [x] 打印启动信息（listening address、agent card、local IPs）
- [x] 初始化 ClaudeCodeAdapter + AdapterServer
- [x] 调用 run_adapter_server_loop（复用 feat-lan-a2a-access）

### 4. Graceful Shutdown
- [x] 监听 Ctrl+C（SIGINT）
- [x] 触发 server shutdown
- [x] 等待进行中请求完成（超时）
- [x] 打印 "Server stopped" 信息

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-16 | Feature created | Waiting for feat-lan-a2a-access completion |
| 2026-04-17 | Implementation complete | All 4 tasks done, 9 tests pass, 258 total tests pass |

## Implementation Notes
- Created `src-tauri/src/cli.rs` with CLI parsing (clap derive), headless server entry, IP detection, and signal handling
- Modified `src-tauri/src/main.rs` to detect `serve` subcommand and dispatch to headless or GUI mode
- Modified `src-tauri/src/lib.rs` to expose the `cli` module as `pub`
- Added dependencies: `clap` v4 (derive), `libc` v0.2 (Unix signal handling)
- Reuses `ClaudeCodeAdapter` + `AdapterServer` + `run_adapter_server_loop` from feat-lan-a2a-access
- Signal handling uses `libc::signal()` on Unix for SIGINT, with async-signal-safe flag setting
