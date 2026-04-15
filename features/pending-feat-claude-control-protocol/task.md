# Tasks: feat-claude-control-protocol

## Task Breakdown

### 0. 调研验证 — Control Protocol 消息格式 ⚠️ 前置
- [ ] 启动 claude 交互式进程：`claude --output-format stream-json --input-format stream-json --permission-prompt-tool stdio`
- [ ] 验证 stdin JSON 输入格式（尝试 `{"type":"user_message","content":"hello"}` 等格式）
- [ ] 验证 stdout JSONL 输出格式（与当前 `--print --output-format stream-json` 对比差异）
- [ ] 验证 `--permission-prompt-tool stdio` 的权限请求/响应 JSON 格式
- [ ] 验证不带 `--print` 时进程是否保持长驻
- [ ] 验证 session 恢复机制（`--resume` 在 Control Protocol 模式下的行为）
- [ ] 记录完整的消息格式文档到 spec.md Technical Solution 中

### 1. 数据模型变更
- [ ] `ExecuteParams` 新增 `pub agent_id: String` 字段（`src-tauri/src/runtime/mod.rs`）
- [ ] 更新所有 `ExecuteParams` 构建处传入 agent_id：
  - [ ] `src-tauri/src/commands/channel.rs` — channel_send_message 构建 ExecuteParams
  - [ ] `src-tauri/src/commands/thread.rs` — thread_send_message 构建 ExecuteParams（如存在）
- [ ] `CodexRuntime::execute()` 适配（忽略 agent_id，保持现有逻辑）
- [ ] 定义 `ProcessHandle` 结构体（child, stdin_writer, current_sender, session_id, last_active, workspace）
- [ ] cargo build 确认所有改动编译通过

### 2. 核心进程管理 — ClaudeCodeRuntime 重写
- [ ] `ClaudeCodeRuntime` 从 `#[derive(Default)]` 改为有状态：
  - [ ] 新增 `processes: Arc<Mutex<HashMap<String, ProcessHandle>>>` 字段
  - [ ] 实现 `new()` 和 `Default`
- [ ] 实现 `spawn_process()` 方法：
  - [ ] 构建 CLI args（无 `--print`，加 `--input-format stream-json --permission-prompt-tool stdio`）
  - [ ] `stdin(Stdio::piped())`（非 null）
  - [ ] spawn 进程并创建 ProcessHandle
  - [ ] 启动持久 stdout reader thread
  - [ ] 启动持久 stderr reader thread
- [ ] 实现 `get_or_spawn()` 方法：
  - [ ] 查 processes map → 有且 alive → 返回
  - [ ] 有但 dead → 清理 + 重新 spawn（--resume session_id）
  - [ ] 无 → 新 spawn
- [ ] 实现 `send_message()` 方法：
  - [ ] 获取 ProcessHandle
  - [ ] 创建新 (tx, rx) channel
  - [ ] 替换 current_sender 为新 tx
  - [ ] 写入 JSON 到 stdin
  - [ ] 返回 rx
- [ ] 实现 `kill_process(agent_id)` — 优雅终止指定进程
- [ ] 实现 `cleanup_all()` — 终止所有进程

### 3. stdout/stderr 解析（复用现有逻辑）
- [ ] 持久 stdout reader thread：
  - [ ] 从 BufReader<ChildStdout> 逐行读取
  - [ ] JSON 解析逻辑直接复用当前 claude.rs L230-380 的代码
  - [ ] 发送到 current_sender（需 lock Mutex）
  - [ ] 检测 permission_request 类型事件 → 路由到权限处理
  - [ ] 检测 session_id → 更新 ProcessHandle.session_id
- [ ] 持久 stderr reader thread：
  - [ ] 复用当前 stderr 逻辑，日志记录
- [ ] 注意：stdout reader 不随 execute() 结束而终止，是进程级别的长驻线程

### 4. execute() 重写
- [ ] 移除当前的 `Command::new("claude").args(...).spawn()` 逻辑
- [ ] 改为调用 `get_or_spawn(agent_id)` + `send_message(agent_id, message)`
- [ ] 返回 `Receiver<StreamEvent>`（接口不变）
- [ ] 移除 `--dangerously-skip-permissions` 参数
- [ ] 移除 `--print` 参数
- [ ] system_prompt 通过 stdin JSON 的 system字段 或 --append-system-prompt 传入（根据调研结果确定）

### 5. 权限处理
- [ ] stdout reader 中检测 permission_request 事件
- [ ] 发出 Tauri event `runtime://permission-request`（含 agent_id, tool, input）
- [ ] 新增 IPC command `permission_respond(agent_id, granted, reason)` — 写入 stdin
- [ ] v1 权限策略：可配置为 auto-allow / auto-deny / interactive（默认 auto-allow 保持向后兼容）
- [ ] 在 `src-tauri/src/lib.rs` 注册 permission_respond command

### 6. 进程生命周期
- [ ] watchdog thread 改为进程级别（非请求级别）：
  - [ ] 定期检查所有 ProcessHandle 的 last_active
  - [ ] 超时进程优雅终止（SIGTERM → 等待 → SIGKILL）
  - [ ] 从 processes map 移除
- [ ] 应用退出时（Drop trait 或显式调用）cleanup_all()
- [ ] Channel/Agent 删除时关联清理进程
- [ ] session_id 缓存：首次 execute 获取后存入 ProcessHandle，用于崩溃恢复

### 7. 集成 & 手动测试
- [ ] cargo build 通过
- [ ] 手动测试：单 Agent 多轮对话（验证进程复用）
- [ ] 手动测试：多 Agent 并发对话（验证进程隔离）
- [ ] 手动测试：权限弹窗（如触发 tool_use）
- [ ] 手动测试：手动 kill 进程后继续对话（验证崩溃恢复）
- [ ] 手动测试：长时间空闲后重新对话（验证超时清理 + 恢复）
- [ ] 确认前端零改动下功能正常

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Feature created | Initial task breakdown — 7 modules + 1 validation step |
