# feat-claude-control-protocol 改动说明

## 修改文件清单

| 文件 | 改动类型 | 说明 |
|------|---------|------|
| `src-tauri/src/runtime/mod.rs` | 微改 | `ExecuteParams` 新增 `agent_id` 字段 |
| `src-tauri/src/runtime/claude.rs` | 重写 | 核心改造：one-shot → 持久进程池 |
| `src-tauri/src/commands/channel.rs` | 微改 | 2 处 ExecuteParams 构建 +agent_id |
| `src-tauri/src/commands/thread.rs` | 微改 | 1 处 ExecuteParams 构建 +agent_id |
| `src-tauri/src/runtime/commands.rs` | 微改 | 1 处 ExecuteParams 构建 +agent_id |

前端零改动。

---

## Before vs After 对比

### 进程模型

```
Before (One-shot):
  用户发消息 → spawn claude 进程 → 处理 → 输出 → 进程退出
  用户发消息 → spawn claude 进程 → 处理 → 输出 → 进程退出
  用户发消息 → spawn claude 进程 → 处理 → 输出 → 进程退出

After (Persistent):
  用户发消息 → 复用已有 claude 进程 → 输出
  用户发消息 → 复用已有 claude 进程 → 输出
  用户发消息 → 复用已有 claude 进程 → 输出
                 ↑ 进程常驻，直到空闲超时或应用退出
```

### CLI 参数对比

| 参数 | Before | After | 说明 |
|------|--------|-------|------|
| `--print` | ✅ | ✅ | 仍需（input-format 依赖它） |
| `--output-format stream-json` | ✅ | ✅ | 不变 |
| `--input-format stream-json` | ❌ | ✅ **新增** | 支持 stdin JSON 输入 |
| `--verbose` | ✅ | ✅ | 不变 |
| `--dangerously-skip-permissions` | ✅ | ✅ | 暂保留，后续权限交互后移除 |
| `stdin` | `Stdio::null()` | `Stdio::piped()` | 核心变化 |

### 通信方式对比

| 维度 | Before | After |
|------|--------|-------|
| 消息传递 | CLI args (`-- "message"`) | stdin JSON |
| 输入格式 | 无 stdin | `{"type":"user","message":{"role":"user","content":[{"type":"text","text":"..."}]}}` |
| 输出格式 | stdout JSONL | stdout JSONL（不变） |
| 进程生命周期 | 每次请求新建/退出 | 持久化，agent_id 维度的进程池 |
| 上下文保持 | `--resume session_id` 每次恢复 | 进程内存中自然保持 |
| 权限处理 | 跳过（不安全） | 预留 `control_request/response` 通道 |

---

## 核心优化点

### 1. 进程复用 — 消除冷启动开销

**Before**: 每条消息 spawn 新进程，每次经历：
- Node.js 启动（~200-500ms）
- Claude Code 初始化（加载 plugins、MCP servers、CLAUDE.md）
- API key 验证
- 建立新 session

**After**: 首次 spawn 后进程常驻，后续消息零启动开销：
- 直接写 stdin JSON
- 响应延迟降低
- 上下文在进程内存中连续，无 `--resume` 恢复成本

### 2. 双向通信 — 解锁权限交互能力

**Before**: `stdin(Stdio::null())`，无法与 claude 进程交互
- 只能用 `--dangerously-skip-permissions` 跳过所有权限
- 无法处理任何需要用户确认的操作

**After**: `stdin(Stdio::piped())`，支持全双工 JSON 通信
- claude 可通过 stdout 发送 `control_request`（权限确认、工具调用审批）
- 我们通过 stdin 回复 `control_response`
- 为后续移除 `--dangerously-skip-permissions` 铺路

### 3. 进程池隔离 — 多 Agent 并发安全

**Before**: Runtime 无状态，每次调用独立
- 多 Agent 并发 = 多个独立 claude 进程（OK 但无管理）
- 进程崩溃无感知，只能等到下次 execute 失败

**After**: `HashMap<agent_id, ProcessHandle>` 进程池
- 每个 Agent 有独立的 claude 进程，互不干扰
- `is_alive()` 主动检测进程状态
- 进程崩溃后自动 `--resume` 恢复（利用已缓存的 session_id）
- 应用退出时 `cleanup_all()` 优雅清理所有进程

### 4. 架构解耦 — 为 A2A 铺路

**Before**: claude.rs 和进程管理紧耦合在 `execute()` 方法内
- 无法被外部复用（A2A Adapter 无法调用同样的逻辑）

**After**: 进程管理独立为 `ProcessHandle` + 进程池
- `send_user_message()` 可独立调用
- `stdin/stdout` JSON 协议可被 A2A Adapter 直接使用
- A2A Server 内部用同样的 Control Protocol 与本地 claude 通信

---

## 代码结构变化

### Before: claude.rs（~474 行，无状态）

```rust
#[derive(Default)]
pub struct ClaudeCodeRuntime;

impl AgentRuntime for ClaudeCodeRuntime {
    fn execute(&self, params) {
        // 每次构建 args
        // 每次 spawn 新进程
        // stdin = null
        // 启动 3 个线程（stdout/stderr/watchdog）
        // 进程处理完自动退出
    }
}
```

### After: claude.rs（~480 行，有状态进程池）

```rust
pub struct ClaudeCodeRuntime {
    processes: Arc<Mutex<HashMap<String, ProcessHandle>>>,
}

struct ProcessHandle {
    child: Child,
    stdin_writer: Option<BufWriter<ChildStdin>>,
    current_sender: Arc<Mutex<Option<Sender<StreamEvent>>>>,
    session_id: Option<String>,
    last_active: Arc<AtomicU64>,
    reader_alive: Arc<AtomicBool>,
    workspace: Option<String>,
}

impl AgentRuntime for ClaudeCodeRuntime {
    fn execute(&self, params) {
        self.get_or_spawn(agent_id, ...)?;  // 复用或新建
        // 创建 (tx, rx) channel
        // 替换 current_sender = tx
        // stdin 写入 JSON 消息
        // 返回 rx
    }
}
```

关键设计：
- `current_sender` 的 Mutex swap 机制：每个 execute() 创建新 channel，stdout reader thread 读取 current_sender 发送事件
- stdout reader thread 是进程级别的长驻线程，不随 execute() 结束
- ProcessHandle 的 Drop trait 自动 shutdown 进程

---

## 验证方式

```bash
# 启动应用
npm run tauri dev

# 测试步骤：
# 1. 向 Agent A 发送第一条消息 → 日志应出现 "Spawning persistent process for agent xxx"
# 2. 向 Agent A 发送第二条消息 → 日志应出现 "Reusing existing process for agent xxx"
# 3. 向 Agent B 发送消息 → 日志应出现 "Spawning persistent process for agent yyy"（新进程）
# 4. 对话功能应与之前完全一致（StreamEvent 格式不变）
```

---

## 后续 TODO

- [ ] 权限交互：处理 stdout 中的 `control_request` → Tauri event → 用户确认 → stdin `control_response`
- [ ] 移除 `--dangerously-skip-permissions`，改为权限交互模式
- [ ] 进程级 watchdog：空闲超时自动关闭进程释放资源
- [ ] Channel/Agent 删除时关联清理进程
- [ ] 进程崩溃恢复的更完善处理（重试次数、错误提示）
