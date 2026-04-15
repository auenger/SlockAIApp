# Feature: feat-claude-control-protocol Claude Code Control Protocol Runtime

## Basic Information
- **ID**: feat-claude-control-protocol
- **Name**: Claude Code Control Protocol Runtime（双向持久会话 + 权限交互）
- **Priority**: 80
- **Size**: M
- **Dependencies**: []
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-15

## Description

将 Claude Code Runtime 从 one-shot 模式 (`--print`) 升级为 Control Protocol 模式（双向 stdin/stdout JSON 流式通信）。

**当前问题**：
- 每次请求 spawn 新进程，无法保持上下文跨消息持久化
- `stdin(Stdio::null())` 单向通信，无法处理权限确认
- 依赖 `--dangerously-skip-permissions`，生产环境不安全
- 进程退出后状态全丢，无崩溃恢复能力

**Control Protocol 方案**：
- Claude CLI 以长驻进程模式运行（无 `--print`）
- `--input-format stream-json` 支持通过 stdin 写入 JSON 消息
- `--output-format stream-json` 通过 stdout 返回 JSONL（与现有解析逻辑兼容）
- `--permission-prompt-tool stdio` 权限请求通过 stdin/stdout 交互
- 每个 Agent 对应一个独立的长驻 claude 进程

**核心目标**：
- 持久会话：一次 spawn，多轮复用，上下文自然保持
- 双向通信：stdin 写入消息，stdout 读取 JSONL 响应
- 权限交互：结构化权限请求/响应，替代 `--dangerously-skip-permissions`
- 进程管理：健康检查、崩溃恢复、资源清理
- 前端无感：`Receiver<StreamEvent>` 接口完全不变

**不做的事**：
- 不做 A2A 协议层（那是 feat-a2a-* 的事）
- 不改前端代码（StreamEvent 格式不变）
- 不改 Codex Runtime（保持现有 one-shot 模式）
- 不做远程进程管理（仅本地）

**与 A2A 方案的关系**：
- 本 Feature 是 feat-a2a-adapter 的软前置：远端 A2A Server 内部用 Control Protocol 与本地 claude CLI 通信
- ACP（本 Feature）= 本地进程 I/O 层；A2A = 网络发现层；两者在不同抽象层级

## User Value Points

### V1: 持久会话 + 双向通信
用户价值：多轮对话不再每次重启 CLI，响应更快更自然。
- 首次消息启动 claude 长驻进程（Control Protocol flags）
- 后续消息通过 stdin JSON 写入已有进程
- 输出 JSONL 格式不变，现有解析逻辑完全复用
- 对话上下文在进程内存中自然保持，无需 `--resume`
- 每条消息的响应延迟降低（无进程启动开销）

### V2: 权限交互 + 进程生命周期
用户价值：权限操作可控可审计，进程管理健壮可靠。
- 权限请求通过 stdio 结构化传递，前端弹窗让用户确认
- 进程崩溃后自动恢复（--resume session_id）
- 多 Agent 进程完全隔离互不影响
- 空闲进程超时自动关闭节省资源
- 应用退出时优雅清理所有进程

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/claude.rs` (474 lines) — 当前 one-shot 实现，需全面重写 execute()
  - L131-138: 当前 args（`--print --output-format stream-json --dangerously-skip-permissions`）
  - L170-182: 进程 spawn（每次新建，stdin=null）
  - L209-400: stdout JSONL 解析逻辑（需复用，是最有价值的代码）
  - L439-470: idle watchdog（需改为进程级别而非请求级别）
- `src-tauri/src/runtime/mod.rs` (274 lines) — AgentRuntime trait + ExecuteParams + StreamEvent
  - L151-163: ExecuteParams 需新增 `agent_id` 字段（用于进程池 key）
  - L193-220: AgentRuntime trait（execute() 签名不变）
- `src-tauri/src/runtime/codex.rs` — Codex Runtime（需适配 ExecuteParams 变更）
- `src-tauri/src/runtime/registry.rs` — Runtime 注册（不变）
- `src-tauri/src/workspace/manager.rs` — AgentManager（需传入 agent_id 到 ExecuteParams）
- `src-tauri/src/commands/channel.rs` — 构建 ExecuteParams 的地方

### Related Features
- **feat-claude-runtime** ✅ 已完成 — 原始 one-shot Claude Code runtime（本 Feature 的基础）
- **feat-agent-runtime-model** ✅ 已完成 — Runtime trait 泛化
- **feat-a2a-adapter** ➡️ 软依赖 — Adapter 内部将使用 Control Protocol 与本地 claude 通信
- **feat-a2a-runtime** — A2A 协议重构父 feature

## Technical Solution

### 架构概要

```
改动范围（全在现有文件中，无新增文件）：

src-tauri/src/runtime/
  claude.rs     — 重写: ClaudeCodeRuntime (stateful, process pool)
  mod.rs        — 微改: ExecuteParams +agent_id
  codex.rs      — 微改: 适配 ExecuteParams agent_id

src-tauri/src/workspace/
  manager.rs    — 微改: execute_agent() 传入 agent_id

src-tauri/src/commands/
  channel.rs    — 微改: ExecuteParams 构建处 +agent_id
```

### 核心数据结构

```rust
/// 长驻 claude 进程句柄
struct ProcessHandle {
    child: Child,                                    // 运行中的 claude 进程
    stdin_writer: BufWriter<ChildStdin>,            // stdin 写入器
    current_sender: Arc<Mutex<Option<Sender<StreamEvent>>>>, // 当前请求的输出 channel
    session_id: Option<String>,                      // 会话 ID
    last_active: Instant,                            // 最后活动时间
    workspace: Option<String>,                       // 工作目录
}

/// ClaudeCodeRuntime 改为有状态
pub struct ClaudeCodeRuntime {
    processes: Arc<Mutex<HashMap<String, ProcessHandle>>>,  // agent_id → ProcessHandle
}
```

### 交互流程

```
首次 execute(agent_id="a1", message="Hello"):
  1. processes.lock() → no entry for "a1"
  2. spawn_process("a1")
     → Command::new("claude")
       .args(["--output-format", "stream-json",
              "--input-format", "stream-json",
              "--permission-prompt-tool", "stdio",
              "--verbose"])
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
     → spawn stdout reader thread (persistent)
     → spawn stderr reader thread (persistent)
  3. create (tx, rx) channel
  4. store tx as current_sender
  5. write {"type":"user_message","content":"Hello"} to stdin
  6. return rx

后续 execute(agent_id="a1", message="继续"):
  1. processes.lock() → found entry for "a1"
  2. check child.alive() → true
  3. create new (tx, rx) channel
  4. replace current_sender with new tx
  5. write {"type":"user_message","content":"继续"} to stdin
  6. return rx

权限请求 (stdout reader thread):
  1. parse stdout JSON → {"type":"permission_request","tool":"Bash","input":{...}}
  2. emit Tauri event "runtime://permission-request"
  3. wait for user response (via IPC command)
  4. write {"type":"permission_response","granted":true/false} to stdin

进程崩溃恢复:
  1. execute() → check child.alive() → false
  2. remove dead ProcessHandle
  3. spawn new process with --resume {session_id}
  4. continue as normal
```

### Control Protocol flags 对比

| Flag | 当前 (one-shot) | Control Protocol |
|------|-----------------|------------------|
| `--print` | ✅ (单次请求) | ❌ 移除 |
| `--output-format stream-json` | ✅ | ✅ 保留 |
| `--input-format stream-json` | ❌ 无 | ✅ 新增 |
| `--permission-prompt-tool stdio` | ❌ 无 | ✅ 新增 |
| `--dangerously-skip-permissions` | ✅ (不安全) | ❌ 移除 |
| `--verbose` | ✅ | ✅ 保留 |
| `--resume {session_id}` | ✅ (每次) | 仅崩溃恢复时 |
| `stdin` | `Stdio::null()` | `Stdio::piped()` |
| 进程生命周期 | 请求结束即退出 | 长驻直到显式关闭 |

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望 Claude Code Agent 使用持久会话和结构化双向通信，
以便获得更流畅的多轮对话体验、安全的权限处理、和更可靠的进程管理。
前端体验应与当前完全一致，无需任何代码改动。

### Scenarios

#### Scenario 1: 首次消息 — 启动持久进程
```gherkin
Given 用户向 Agent A 发送第一条消息
When 系统启动 claude 进程
Then 使用 flags: --output-format stream-json --input-format stream-json --permission-prompt-tool stdio --verbose
And stdin 为 piped（可写入），stdout 为 piped（可读取）
And 流式响应通过 stdout JSONL → StreamEvent 返回
And 进程在响应完成后继续存活（不退出）
And session_id 从首次响应中提取并缓存
```

#### Scenario 2: 后续消息 — 复用已有进程
```gherkin
Given Agent A 已有一个活跃的 claude 进程（session 已建立）
When 用户发送第二条消息
Then 系统通过 stdin 向已有进程写入 JSON 消息
And 不创建新进程（进程数不增加）
And 响应通过同一进程的 stdout 流式返回
And 对话上下文自然保持（无需 --resume）
```

#### Scenario 3: 权限确认交互
```gherkin
Given Agent A 的 claude 进程正在执行需要权限的操作（如 Bash: rm -rf）
When claude 通过 stdout 发送 permission_request 事件
Then 系统解析请求（tool name, input description）
And 发出 Tauri event "runtime://permission-request"
And 前端显示权限确认对话框
When 用户点击"允许"
Then 系统通过 stdin 发送 {"type":"permission_response","granted":true}
And claude 继续执行
When 用户点击"拒绝"
Then 系统通过 stdin 发送 {"type":"permission_response","granted":false,"reason":"User rejected"}
And claude 收到拒绝并调整行为
```

#### Scenario 4: 进程崩溃自动恢复
```gherkin
Given Agent A 的 claude 进程意外崩溃（OOM、segfault、被 kill）
When 用户发送新消息
Then 系统检测到进程已死亡（child.try_wait() → Some(exit_status)）
And 自动使用 --resume {session_id} 启动新进程
And 新进程恢复之前的会话上下文
And 前端无感知（StreamEvent 格式不变）
And 日志记录崩溃事件和恢复操作
```

#### Scenario 5: 前端无感升级
```gherkin
Given Control Protocol runtime 已启用
When 前端发送消息并接收响应
Then StreamEvent 格式与 one-shot 模式完全一致
And Receiver<StreamEvent> 接口签名不变
And 前端不需要任何代码修改
And msg_type, text, content_blocks, is_done 字段行为不变
```

#### Scenario 6: 多 Agent 进程隔离
```gherkin
Given Agent A 和 Agent B 各有独立的 claude 进程
When Agent A 和 Agent B 同时执行任务
Then 各自进程独立运行互不影响
And Agent A 的 stdin/stdout 不与 Agent B 交叉
And 各进程有独立的 session_id 和工作目录
And 关闭 Agent A 的进程不影响 Agent B
```

#### Scenario 7: 空闲超时清理
```gherkin
Given Agent A 的 claude 进程空闲超过 N 秒（可配置）
When 后台 watchdog 检测到超时
Then 优雅终止进程（stdin EOF 或 SIGTERM）
And 从进程池中移除
And 下次请求时重新启动（--resume）
```

### General Checklist
- [ ] Control Protocol 消息格式验证通过（stdin JSON 输入 / stdout JSONL 输出）
- [ ] 持久进程 spawn + stdin 写入 + stdout 读取完整流程
- [ ] 进程池管理（get_or_spawn / kill / cleanup_all）
- [ ] Channel swapping 机制（多个 execute() 调用路由到正确 channel）
- [ ] 权限请求/响应处理（stdout → Tauri event → 用户确认 → stdin response）
- [ ] 进程崩溃检测 + 自动恢复（--resume）
- [ ] 空闲超时清理
- [ ] ExecuteParams 新增 agent_id（所有调用点适配）
- [ ] CodexRuntime 适配（忽略 agent_id）
- [ ] 前端零改动（StreamEvent 格式不变）
- [ ] cargo build 通过
- [ ] 手动测试：单 Agent 多轮对话
- [ ] 手动测试：多 Agent 并发

## Merge Record

- **Completed**: 2026-04-15
- **Merged to**: main (code was already merged in prior commits: a68d80f, f8d68ba, 2909962)
- **Archive tag**: feat-claude-control-protocol-20260415
- **Conflicts**: None
- **Verification**: PASS (6/7 Gherkin scenarios PASS, 1 PARTIAL - permission handling deferred to v1)
- **Evidence**: features/archive/done-feat-claude-control-protocol-20260415/evidence/verification-report.md
- **Key changes**:
  - ClaudeCodeRuntime: stateful with process pool (`Arc<Mutex<HashMap<String, ProcessHandle>>>`)
  - Dual execution mode: persistent (Thread) via stdin/stdout JSON, one-shot (Channel) via --print
  - LRU eviction (max 5 concurrent processes), idle timeout (300s)
  - Crash recovery via --resume session_id
  - ExecuteParams gained `persistent: bool` and `agent_id: String` fields
  - Zero frontend change (StreamEvent interface unchanged)
