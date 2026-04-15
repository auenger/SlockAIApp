# Tasks: feat-claude-control-protocol

## Task Breakdown

### 0. 调研验证 — Control Protocol 消息格式 ⚠️ 前置
- [x] 启动 claude 交互式进程：`claude --output-format stream-json --input-format stream-json --permission-prompt-tool stdio`
- [x] 验证 stdin JSON 输入格式（尝试 `{"type":"user_message","content":"hello"}` 等格式）
- [x] 验证 stdout JSONL 输出格式（与当前 `--print --output-format stream-json` 对比差异）
- [x] 验证 `--permission-prompt-tool stdio` 的权限请求/响应 JSON 格式
- [x] 验证不带 `--print` 时进程是否保持长驻
- [x] 验证 session 恢复机制（`--resume` 在 Control Protocol 模式下的行为）
- [x] 记录完整的消息格式文档到 spec.md Technical Solution 中

### 1. 数据模型变更
- [x] `ExecuteParams` 新增 `pub agent_id: String` 字段（`src-tauri/src/runtime/mod.rs`）
- [x] 更新所有 `ExecuteParams` 构建处传入 agent_id：
  - [x] `src-tauri/src/commands/channel.rs` — channel_send_message 构建 ExecuteParams
  - [x] `src-tauri/src/commands/thread.rs` — thread_send_message 构建 ExecuteParams（如存在）
- [x] `CodexRuntime::execute()` 适配（忽略 agent_id，保持现有逻辑）
- [x] 定义 `ProcessHandle` 结构体（child, stdin_writer, current_sender, session_id, last_active, workspace）
- [x] cargo build 确认所有改动编译通过

### 2. 核心进程管理 — ClaudeCodeRuntime 重写
- [x] `ClaudeCodeRuntime` 从 `#[derive(Default)]` 改为有状态：
  - [x] 新增 `processes: Arc<Mutex<HashMap<String, ProcessHandle>>>` 字段
  - [x] 实现 `new()` 和 `Default`
- [x] 实现 `spawn_process()` 方法：
  - [x] 构建 CLI args（无 `--print`，加 `--input-format stream-json --permission-prompt-tool stdio`）
  - [x] `stdin(Stdio::piped())`（非 null）
  - [x] spawn 进程并创建 ProcessHandle
  - [x] 启动持久 stdout reader thread
  - [x] 启动持久 stderr reader thread
- [x] 实现 `get_or_spawn()` 方法：
  - [x] 查 processes map → 有且 alive → 返回
  - [x] 有但 dead → 清理 + 重新 spawn（--resume session_id）
  - [x] 无 → 新 spawn
- [x] 实现 `send_message()` 方法：
  - [x] 获取 ProcessHandle
  - [x] 创建新 (tx, rx) channel
  - [x] 替换 current_sender 为新 tx
  - [x] 写入 JSON 到 stdin
  - [x] 返回 rx
- [x] 实现 `kill_process(agent_id)` — 优雅终止指定进程
- [x] 实现 `cleanup_all()` — 终止所有进程

### 3. stdout/stderr 解析（复用现有逻辑）
- [x] 持久 stdout reader thread：
  - [x] 从 BufReader<ChildStdout> 逐行读取
  - [x] JSON 解析逻辑直接复用当前 claude.rs L230-380 的代码
  - [x] 发送到 current_sender（需 lock Mutex）
  - [x] 检测 permission_request 类型事件 → 路由到权限处理
  - [x] 检测 session_id → 更新 ProcessHandle.session_id
- [x] 持久 stderr reader thread：
  - [x] 复用当前 stderr 逻辑，日志记录
- [x] 注意：stdout reader 不随 execute() 结束而终止，是进程级别的长驻线程

### 4. execute() 重写
- [x] 移除当前的 `Command::new("claude").args(...).spawn()` 逻辑
- [x] 改为调用 `get_or_spawn(agent_id)` + `send_message(agent_id, message)`
- [x] 返回 `Receiver<StreamEvent>`（接口不变）
- [x] 移除 `--dangerously-skip-permissions` 参数
- [x] 移除 `--print` 参数
- [x] system_prompt 通过 stdin JSON 的 system字段 或 --append-system-prompt 传入（根据调研结果确定）

### 5. 权限处理
- [x] stdout reader 中检测 permission_request 事件
- [x] 发出 Tauri event `runtime://permission-request`（含 agent_id, tool, input）
- [x] 新增 IPC command `permission_respond(agent_id, granted, reason)` — 写入 stdin
- [x] v1 权限策略：可配置为 auto-allow / auto-deny / interactive（默认 auto-allow 保持向后兼容）
- [x] 在 `src-tauri/src/lib.rs` 注册 permission_respond command

### 6. 进程生命周期
- [x] watchdog thread 改为进程级别（非请求级别）：
  - [x] 定期检查所有 ProcessHandle 的 last_active
  - [x] 超时进程优雅终止（SIGTERM → 等待 → SIGKILL）
  - [x] 从 processes map 移除
- [x] 应用退出时（Drop trait 或显式调用）cleanup_all()
- [x] Channel/Agent 删除时关联清理进程
- [x] session_id 缓存：首次 execute 获取后存入 ProcessHandle，用于崩溃恢复

### 7. 集成 & 手动测试
- [x] cargo build 通过
- [x] 手动测试：单 Agent 多轮对话（验证进程复用）
- [x] 手动测试：多 Agent 并发对话（验证进程隔离）
- [x] 手动测试：权限弹窗（如触发 tool_use）
- [x] 手动测试：手动 kill 进程后继续对话（验证崩溃恢复）
- [x] 手动测试：长时间空闲后重新对话（验证超时清理 + 恢复）
- [x] 确认前端零改动下功能正常

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Feature created | Initial task breakdown — 7 modules + 1 validation step |
| 2026-04-15 | Implementation complete | All code merged to main (commits a68d80f, f8d68ba, 2909962). Dual execution mode: persistent (Thread) + one-shot (Channel). LRU eviction, idle timeout, crash recovery. cargo build passes. |
