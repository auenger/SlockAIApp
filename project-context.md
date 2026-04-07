---
last_updated: '2026-04-06'
version: 3
features_completed: 0
---

# Project Context: SlockAI

> AI 原生协作桌面应用 — 人类 @Agent 触发对话，上下文编排驱动 Claude Code / Codex，文档/JSONL 持久化。

---

## 产品定位

**SlockAI** 是一个 Tauri 桌面端 AI 协作应用。核心模型极简：

1. **Channel / Thread = 对话容器** — 不同 channel 存储不同对话记录
2. **@Agent = 触发器** — 在 channel 里 @某个 Agent，触发一次 LLM 请求
3. **上下文编排 = 核心引擎** — 决定对话历史哪些压缩、哪些全量携带，喂给 Agent
4. **Workspace = Agent 文件目录** — 每个 Agent 对应独立的文件存放目录
5. **文档/JSONL 驱动** — 不用数据库，App 直接读写文件

### 核心交互流

```
用户在 Channel 中输入消息 → @Claude 触发 →
  App 从对话记录中编排上下文 (history + compression) →
    调用 Claude Code / Codex API →
      Agent 响应写回对话 →
        持久化到 JSONL / Markdown 文件
```

### 竞品参考

| 维度 | Slock.ai (参考) | 本项目 |
|------|-----------------|--------|
| 载体 | Web SPA | **Tauri 桌面应用** |
| Agent Runtime | Claude Code via MCP | **Claude Code + Codex 双 runtime** |
| 通信 | MCP over WebSocket | **直接调用 LLM API** |
| 上下文管理 | 隐式（Agent 自行管理 MEMORY.md） | **显式上下文编排引擎** |
| 存储 | 云端 + 本地混合 | **纯本地 JSONL / Markdown** |
| 触发机制 | Agent 自动轮询消息 | **@Agent 显式触发** |

## 技术栈

| Category | Technology | Notes |
|----------|-----------|-------|
| Frontend | React 19 + TypeScript 5.8 | SPA, 无 React Router |
| Build | Vite 6 | HMR + 生产构建 |
| Styling | Tailwind CSS 4 | 新粗野主义风格 |
| Desktop Shell | **Tauri V2** | Rust 后端 |
| IPC | `invoke()` / `listen()` | Tauri v2 语法 |
| Backend | Rust | Tauri Commands + Events |
| Agent Runtime | **Claude Code** / **Codex** | 双 runtime 支持 |
| LLM 调用 | Anthropic API / OpenAI API | Rust 端直接调用 |
| 对话存储 | **JSONL** (对话记录) + **Markdown** (文档) | 文件驱动，无数据库 |
| State | `useState` + switch 视图切换 | 不引入 Redux |
| Testing | Vitest | 前端测试 |

## 目录结构（规划）

```
SlockAI/
├── src/                          # React 前端
│   ├── components/
│   │   ├── layout/               # 三栏布局 (Sidebar / Main / Detail)
│   │   ├── channel/              # 频道 = 对话容器
│   │   ├── thread/               # 线程 = 对话分支
│   │   ├── message/              # 消息渲染 (Markdown / Code / Agent 响应)
│   │   └── agent/                # Agent 选择器 / 状态指示
│   ├── hooks/
│   ├── lib/
│   │   ├── ipc.ts                # Tauri IPC 封装
│   │   └── context.ts            # 上下文编排逻辑 (前端侧)
│   ├── types.ts
│   └── main.tsx
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── chat.rs           # 对话相关 commands
│   │   │   ├── agent.rs          # Agent runtime commands
│   │   │   └── workspace.rs      # Workspace 管理
│   │   ├── context/
│   │   │   ├── orchestrator.rs   # 上下文编排引擎 (核心)
│   │   │   ├── compressor.rs     # 对话历史压缩
│   │   │   └── history.rs        # 对话记录管理
│   │   ├── runtime/
│   │   │   ├── claude.rs         # Claude Code runtime
│   │   │   ├── codex.rs          # Codex runtime
│   │   │   └── mod.rs            # Runtime trait 抽象
│   │   ├── storage/
│   │   │   ├── jsonl.rs          # JSONL 对话记录读写
│   │   │   └── markdown.rs       # Markdown 文档读写
│   │   └── lib.rs
│   └── Cargo.toml
├── workspaces/                   # Agent 工作区 (运行时)
│   └── {agent-id}/
│       ├── conversations/        # 对话记录 (JSONL)
│       ├── context/              # 上下文快照
│       └── output/               # Agent 输出文件
├── feature-workflow/
├── features/
└── project-context.md
```

## 核心架构

### 架构全景

```
┌──────────────────────────────────────────────────────────┐
│                 Tauri Desktop Application                  │
│                                                           │
│  ┌──────────────────────────────────────────────────────┐│
│  │              React Frontend (WebView)                 ││
│  │                                                      ││
│  │  ┌──────────┐  ┌───────────────┐  ┌──────────────┐  ││
│  │  │ Sidebar  │  │ Channel View  │  │ Thread/Detail │  ││
│  │  │ Channels │  │ (对话容器)     │  │ (对话分支)    │  ││
│  │  │ Agents   │  │ @Agent 触发   │  │ 消息流渲染    │  ││
│  │  │ Workspace│  │ 消息输入/输出  │  │              │  ││
│  │  └──────────┘  └───────────────┘  └──────────────┘  ││
│  └─────────────────────┬────────────────────────────────┘│
│                        │ IPC (invoke / listen)            │
│  ┌─────────────────────┴────────────────────────────────┐│
│  │              Rust Backend (Tauri)                     ││
│  │                                                      ││
│  │  ┌─────────────────────────────────────────────┐     ││
│  │  │         上下文编排引擎 (Orchestrator)         │     ││
│  │  │  ┌───────────┐  ┌──────────┐  ┌──────────┐  │     ││
│  │  │  │ History   │  │Compressor│  │ Context   │  │     ││
│  │  │  │ (JSONL)   │  │(压缩策略)│  │ Assembler │  │     ││
│  │  │  └───────────┘  └──────────┘  └──────────┘  │     ││
│  │  └────────────────────┬────────────────────────┘     ││
│  │                       │ 编排后的上下文                  ││
│  │  ┌────────────────────┴────────────────────────┐     ││
│  │  │         Agent Runtime Layer                  │     ││
│  │  │  ┌──────────────┐  ┌──────────────────┐     │     ││
│  │  │  │ Claude Code  │  │     Codex        │     │     ││
│  │  │  │ (Anthropic)  │  │   (OpenAI)       │     │     ││
│  │  │  └──────────────┘  └──────────────────┘     │     ││
│  │  └─────────────────────────────────────────────┘     ││
│  │                                                      ││
│  │  ┌─────────────────────────────────────────────┐     ││
│  │  │    Storage Layer (文档 / JSONL 驱动)          │     ││
│  │  │  workspaces/{agent-id}/conversations/        │     ││
│  │  └─────────────────────────────────────────────┘     ││
│  └──────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────┘
```

### 核心概念

#### 1. Channel = 对话容器

Channel 不是聊天室，是**对话记录的容器**。每个 Channel 对应一组独立的对话记录（JSONL 文件）。

```
Channel "code-review"
  → conversations/code-review/2026-04-06.jsonl
  → conversations/code-review/2026-04-05.jsonl
  ...

Channel "general"
  → conversations/general/2026-04-06.jsonl
  ...
```

#### 2. @Agent = 触发器

在 Channel 中 `@Claude` 或 `@Codex`，本质是**触发一次 LLM 请求**：
- 用户消息 + 编排后的上下文 → 发给对应 Agent Runtime
- Agent 响应写回对话记录
- 前端通过 Tauri Event 实时渲染

```
用户: "帮我 review 一下这段代码 @Claude"
  → 触发 Claude Code runtime
  → 上下文编排: 携带最近 N 轮对话 + 压缩的更早历史
  → Claude 响应: "这段代码有 3 个问题..."
  → 写入 JSONL
```

#### 3. 上下文编排引擎 (核心)

这是系统最核心的模块，决定给 Agent 喂什么上下文：

```
编排策略:
┌─────────────────────────────────────────────────┐
│  当前消息                                        │
│  + 最近 N 轮对话 (全量携带，保留完整上下文)        │
│  + 更早对话的压缩摘要 (token 预算控制)            │
│  + Channel 级别的 persistent context (固定前缀)   │
│  + @mentioned 的文件内容 (如果用户附带了文件)      │
├─────────────────────────────────────────────────┤
│  压缩策略:                                       │
│  - 滑动窗口: 最近 K 条全量，更早的压缩             │
│  - 重要性评分: 含 @mention / 代码块 的消息优先    │
│  - Token 预算: 不超过 Agent 的 context window     │
└─────────────────────────────────────────────────┘
```

#### 4. Workspace = Agent 文件目录

每个 Agent 对应一个 Workspace 目录：
- `workspaces/{agent-id}/conversations/` — 对话记录
- `workspaces/{agent-id}/context/` — 上下文快照
- `workspaces/{agent-id}/output/` — Agent 输出文件

#### 5. 文档/JSONL 驱动

不使用数据库。对话记录以 JSONL 格式存储，每行一条消息：

```jsonl
{"id":"msg_001","role":"user","content":"帮我 review 代码 @Claude","timestamp":"2026-04-06T22:00:00+08:00","channel":"code-review","trigger_agent":"claude"}
{"id":"msg_002","role":"agent","agent":"claude","content":"这段代码有 3 个问题...","timestamp":"2026-04-06T22:00:05+08:00","channel":"code-review"}
```

### Agent Runtime 抽象

```rust
// 伪代码 - Runtime trait
trait AgentRuntime {
    async fn send(&self, context: Context) -> Result<Response>;
    fn name(&self) -> &str;
    fn context_window(&self) -> usize;
}

// Claude Code Runtime → Anthropic API
// Codex Runtime → OpenAI API
```

## UI 设计风格

参考 Slock.ai 界面截图的**新粗野主义 (Neo-Brutalism)** 风格：

- **主色调**：高饱和亮黄色 (Lemon Yellow) 侧边栏
- **背景**：米白色/浅杏色 (Off-white/Beige)
- **强调色**：青色 (活跃)、粉色/红色 (警告/高亮)
- **线条**：粗重黑色描边 (2-3px)
- **头像**：像素风 (Pixel Art)
- **字体**：等宽/终端字体
- **布局**：经典三栏 — Sidebar | Channel View | Thread/Detail

## 关键设计决策

| Decision | Choice | Rationale |
|----------|--------|-----------|
| 应用形态 | Tauri 桌面应用 | 原生性能 + 本地文件系统直接访问 |
| Agent Runtime | Claude Code + Codex | 双 runtime，用户按需选择 |
| 对话模型 | Channel = 对话容器 | 不同频道隔离对话上下文 |
| 触发机制 | @Agent 显式触发 | 用户明确控制何时调用 LLM |
| 上下文管理 | 编排引擎 (Orchestrator) | 显式控制 token 预算和压缩策略 |
| 存储 | JSONL + Markdown (文件驱动) | 零数据库依赖，透明可查 |
| 状态管理 | useState + switch | 轻量，无 React Router |
| IPC | Tauri v2 invoke/listen | 前端 ↔ Rust 双向通信 |

## Critical Rules

### Must Follow

- 使用 `@tauri-apps/api` **v2** 语法（非 v1）
- `cn()` 合并样式（clsx + tailwind-merge）
- 类型定义集中在 `types.ts`
- @Agent 触发必须经过上下文编排引擎，不能直接透传全部历史
- 对话记录必须持久化到 JSONL，不能仅内存保存
- Agent Runtime 必须通过 trait 抽象，支持 Claude Code 和 Codex 切换
- 上下文编排必须考虑 token 预算，不能超出 Agent context window
- Thread 最多一层，不支持嵌套

### Must Avoid

- 不在前端硬编码 API Keys
- 不引入 React Router
- 不使用 SQLite 或任何数据库
- 不在前端 mock 终端 I/O
- 不在 Thread 内触发新的 @Agent（只有 Channel 顶层可触发）
- 不跳过上下文编排直接调用 LLM API
- 不使用云端存储，全部本地文件

## 近期规划

- [ ] Tauri V2 项目脚手架搭建
- [ ] 三栏布局基础框架
- [ ] JSONL 存储层 (Rust)
- [ ] Channel 对话容器 UI + 逻辑
- [ ] @Agent 触发器机制
- [ ] 上下文编排引擎 (核心)
- [ ] Claude Code Runtime 对接
- [ ] Codex Runtime 对接
- [ ] Thread (对话分支) 支持
- [ ] Workspace 文件管理

## Update Log

- 2026-04-06: 初始 project-context 创建
- 2026-04-06: v2 — 确定技术栈 (React + Tauri Rust)
- 2026-04-06: v3 — 核心架构重构：Channel=对话容器, @Agent=触发器, 上下文编排引擎, 双Runtime(Claude Code+Codex), JSONL驱动
