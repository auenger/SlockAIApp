---
last_updated: '2026-04-10'
version: 5
features_completed: 27
---

# Project Context: AgentsZone

> AI 原生协作桌面应用 — 人类 @Agent 触发对话，上下文编排 + 自动摘要驱动 Claude Code / Codex / Gemini，SQLite + JSONL 持久化。

---

## 产品定位

**AgentsZone** 是一个 Tauri V2 桌面端 AI 协作应用。核心模型极简：

1. **Channel = 多 Agent 协作容器** — 支持 @mention 触发、自动上下文压缩、摘要持久化
2. **Thread = 1 对 1 Agent 深度对话** — 独立上下文的专注对话模式
3. **@Agent = 触发器** — 在 Channel 里 @某个 Agent，触发一次 LLM 请求
4. **上下文编排 = 核心引擎** — 滑动窗口 + 自动摘要，控制 token 预算
5. **Workspace = Agent 文件目录** — 每个 Agent 对应独立的文件存放目录
6. **SQLite + JSONL 混合存储** — SQLite 管理结构化元数据，JSONL 存储消息体，Keyring 存储 API 密钥

### 核心交互流

```
用户在 Channel 中输入消息 → @Claude 触发 →
  App 从对话记录中编排上下文 (summary + recent messages) →
    调用 Claude Code / Codex CLI Runtime →
      Agent 响应写回对话 →
        持久化到 SQLite + JSONL →
          消息数 > 30 时自动触发摘要压缩
```

### 竞品参考

| 维度 | Slock.ai (参考) | 本项目 (AgentsZone) |
|------|-----------------|--------|
| 载体 | Web SPA | **Tauri 桌面应用** |
| Agent Runtime | Claude Code via MCP | **Claude Code + Codex + Gemini(规划)** |
| 通信 | MCP over WebSocket | **CLI Runtime 直接调用** |
| 上下文管理 | 隐式（Agent 自行管理 MEMORY.md） | **显式上下文编排 + 自动摘要压缩** |
| 存储 | 云端 + 本地混合 | **SQLite + JSONL 混合存储，全部本地** |
| 触发机制 | Agent 自动轮询消息 | **@Agent 显式触发** |
| 密钥管理 | 服务端托管 | **系统 Keyring 安全存储** |

## 技术栈

| Category | Technology | Notes |
|----------|-----------|-------|
| Frontend | React 19 + TypeScript 5.8 | SPA, 无 React Router |
| Build | Vite 6 | HMR + 生产构建 |
| Styling | Tailwind CSS 4 | 新粗野主义 (Neo-Brutalism) 风格 |
| Desktop Shell | **Tauri V2** | Rust 后端 |
| IPC | `invoke()` / `listen()` | Tauri v2 语法 |
| Backend | Rust | Tauri Commands + Events |
| Agent Runtime | **Claude Code** / **Codex** / Gemini(规划) | 多 runtime，trait 抽象 |
| LLM 调用 | Claude CLI / Codex CLI | Rust 端管理子进程，流式接收 |
| 对话存储 | **SQLite** (元数据) + **JSONL** (消息体) | 混合存储，结构化查询 + 追加写入 |
| Channel 存储 | **JSON 文件** (ChannelStore) | 单独 JSON 文件管理 Channel 数据 |
| 密钥存储 | **Keyring** | 系统原生密钥管理 |
| State | `useState` + switch 视图切换 | 不引入 Redux |
| Testing | Vitest | 前端测试 |

## 实际目录结构

```
SlockAI/
├── src/                          # React 前端
│   ├── components/
│   │   ├── MainContent.tsx       # 主聊天区域 (Channel + Thread 模式切换)
│   │   ├── Sidebar.tsx           # 侧边栏 (Agent/Channel/Thread 导航)
│   │   ├── ThreadPanel.tsx       # Thread 详情面板
│   │   ├── MentionAutocomplete.tsx # @mention 自动补全 + 渲染
│   │   ├── CreateAgentModal.tsx   # Agent 创建对话框
│   │   ├── EditAgentModal.tsx     # Agent 编辑对话框
│   │   ├── AgentIcon.tsx         # Agent SVG 图标组件
│   │   ├── IconPicker.tsx        # 图标选择器
│   │   ├── SkillsPanel.tsx       # 技能管理面板
│   │   ├── ApiKeyManager.tsx     # API 密钥管理
│   │   └── Modals.tsx            # 其他弹窗
│   ├── lib/
│   │   ├── ipc.ts                # Tauri IPC 封装
│   │   ├── useChannel.ts         # Channel 管理 hook
│   │   ├── useThreadChat.ts      # Thread 对话 hook
│   │   ├── useAgentStatus.ts     # Agent 状态 hook
│   │   ├── useAgentProfile.ts    # Agent Profile hook
│   │   ├── useAgentRuntimes.ts   # Agent Runtime hook
│   │   ├── useRuntimeStatus.ts   # Runtime 状态检测 hook
│   │   ├── useSkills.ts          # 技能管理 hook
│   │   ├── useWorkspace.ts       # Workspace 操作 hook
│   │   ├── useActivityLog.ts     # Activity 日志 hook
│   │   ├── useApiKeys.ts         # API Key hook
│   │   ├── useUserProfile.ts     # 用户 Profile hook
│   │   ├── utils.ts              # 工具函数
│   │   └── iconRegistry.ts       # SVG 图标注册表
│   ├── types.ts                  # TypeScript 类型定义
│   ├── App.tsx                   # 应用入口
│   └── main.tsx                  # 渲染入口
├── src-tauri/                    # Rust 后端
│   └── src/
│       ├── lib.rs                # Tauri 入口 + Command 注册
│       ├── commands/
│       │   ├── channel.rs        # Channel CRUD + 消息 + 上下文编排 + 自动摘要
│       │   ├── thread.rs         # Thread CRUD + 对话
│       │   └── activity.rs       # Activity 日志
│       ├── runtime/
│       │   ├── mod.rs            # AgentRuntime trait + RuntimeType enum
│       │   ├── claude.rs         # Claude Code CLI runtime (流式、session、tool use)
│       │   ├── codex.rs          # Codex CLI runtime (流式、session、tool use)
│       │   ├── registry.rs       # Runtime 注册中心 (线程安全)
│       │   └── commands.rs       # Runtime 相关 Tauri commands
│       ├── storage/
│       │   ├── mod.rs            # 存储模块入口
│       │   ├── db.rs             # SQLite 数据库 + migration 系统
│       │   ├── db_helpers.rs     # 数据库查询辅助
│       │   ├── jsonl.rs          # JSONL 消息追加写入
│       │   ├── activity.rs       # Activity 日志存储 (双写 SQLite + JSONL)
│       │   ├── keyring.rs        # API Key 安全存储 (系统 Keyring)
│       │   └── migrations/       # SQLite migration SQL 文件
│       ├── workspace/
│       │   ├── mod.rs            # Workspace 模块入口
│       │   ├── manager.rs        # AgentManager (多 Agent 管理 + 生命周期)
│       │   ├── agent.rs          # 单个 Agent 管理 (身份、配置、状态)
│       │   ├── channel.rs        # Channel 数据管理 (ChannelStore JSON)
│       │   ├── thread.rs         # Thread 管理
│       │   ├── mention.rs        # @mention 解析与路由
│       │   ├── identity.rs       # Agent 身份系统 (IDENTITY.md, SOUL.md)
│       │   ├── skill.rs          # Agent 技能管理
│       │   └── templates.rs      # Agent 模板系统
│       └── context/
│           └── mod.rs            # 上下文组装 (从 Markdown 文件读取)
├── feature-workflow/             # Feature 工作流
│   ├── config.yaml               # 工作流配置
│   └── queue.yaml                # Feature 队列 (active/pending/completed)
├── features/                     # Feature 目录
│   └── archive/                  # 已完成 Feature 归档
├── README.md                     # 项目说明
├── CLAUDE.md                     # Claude Code 项目指南
└── project-context.md            # 本文件
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
│  │  │ Channels │  │ @Agent 触发   │  │ 1对1 深度对话 │  ││
│  │  │ Agents   │  │ 彩色 mention  │  │ 独立上下文    │  ││
│  │  │ Workspace│  │ 流式响应      │  │              │  ││
│  │  └──────────┘  └───────────────┘  └──────────────┘  ││
│  └─────────────────────┬────────────────────────────────┘│
│                        │ IPC (invoke / listen)            │
│  ┌─────────────────────┴────────────────────────────────┐│
│  │              Rust Backend (Tauri)                     ││
│  │                                                      ││
│  │  ┌─────────────────────────────────────────────┐     ││
│  │  │         上下文编排引擎 (Orchestrator)         │     ││
│  │  │  ┌───────────┐  ┌──────────────────────────┐│     ││
│  │  │  │ Summary   │  │ Sliding Window           ││     ││
│  │  │  │ (压缩摘要) │  │ 最近 N 条全量 + 自动压缩  ││     ││
│  │  │  └───────────┘  └──────────────────────────┘│     ││
│  │  │  Auto-Compact: >30 msgs → Agent 生成摘要      │     ││
│  │  └────────────────────┬────────────────────────┘     ││
│  │                       │ 编排后的上下文                  ││
│  │  ┌────────────────────┴────────────────────────┐     ││
│  │  │         Agent Runtime Layer (trait)          │     ││
│  │  │  ┌──────────────┐  ┌──────────────────┐     │     ││
│  │  │  │ Claude Code  │  │     Codex        │     │     ││
│  │  │  │ (Anthropic)  │  │   (OpenAI)       │     │     ││
│  │  │  └──────────────┘  └──────────────────┘     │     ││
│  │  └─────────────────────────────────────────────┘     ││
│  │                                                      ││
│  │  ┌─────────────────────────────────────────────┐     ││
│  │  │         Storage Layer                        │     ││
│  │  │  SQLite: metadata (agents/threads/skills)   │     ││
│  │  │  JSONL: message bodies (append-only)        │     ││
│  │  │  ChannelStore: Channel JSON files            │     ││
│  │  │  Keyring: API keys (system native)          │     ││
│  │  └─────────────────────────────────────────────┘     ││
│  └──────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────┘
```

### 核心概念

#### 1. Channel = 多 Agent 协作容器

Channel 是多 Agent 协作的对话空间。支持 @mention 触发特定 Agent，消息自动路由。

```
Channel "code-review"
  成员: [Claude, Codex]
  用户: "帮我 review 这段代码 @Claude"
  → Claude 响应 → 写入 Channel 消息
  用户: "按 Claude 的建议修改 @Codex"
  → Codex 响应 → 写入 Channel 消息
```

Channel 数据结构包含：
- `summary` — 旧消息的压缩摘要
- `summary_up_to` — 已摘要化的最后一条消息 ID
- `summary_updated_at` — 摘要最后更新时间

#### 2. Thread = 1 对 1 深度对话

Thread 提供与单个 Agent 的专注对话模式，拥有独立的上下文，不受 Channel 干扰。

#### 3. @Agent = 触发器

在 Channel 中 `@Claude` 或 `@Codex`，触发一次 LLM 请求：
- 用户消息 + 编排后的上下文 → 发给对应 Agent Runtime
- Agent 响应写回对话记录
- 前端通过 Tauri Event 实时流式渲染

#### 4. 上下文编排引擎 (核心)

滑动窗口 + 自动摘要策略：

```
编排策略:
┌─────────────────────────────────────────────────┐
│  Channel Summary (自动生成，旧消息压缩摘要)       │
│  + 最近 N 条消息 (全量携带，保留完整上下文)        │
│  + Channel persistent context (固定前缀)          │
│  + Agent Workspace context (SOUL.md, IDENTITY.md) │
├─────────────────────────────────────────────────┤
│  自动压缩:                                       │
│  - 消息数 > 30 时自动触发                        │
│  - 保留最近 10 条消息全量                         │
│  - 更早消息由 Agent runtime 生成摘要              │
│  - 已有摘要时增量更新，非全量重建                  │
│  - 前端收到 needs-compact 事件后调用 compact 命令  │
└─────────────────────────────────────────────────┘
```

#### 5. Workspace = Agent 文件目录

每个 Agent 对应一个 Workspace 目录：
- `workspaces/{agent-id}/conversations/` — 对话记录 (JSONL)
- `workspaces/{agent-id}/IDENTITY.md` — Agent 身份描述
- `workspaces/{agent-id}/SOUL.md` — Agent 灵魂/人格
- `workspaces/{agent-id}/output/` — Agent 输出文件

#### 6. 混合存储架构

**SQLite (结构化元数据)**:
- `agents` — Agent 配置、状态、runtime_type
- `threads` — Thread 元数据 + JSONL 路径指针
- `skills` — 技能配置
- `activity_log` — Activity 时间线索引
- Migration 系统支持从 JSONL 自动迁移

**JSONL / JSON (追加写入)**:
- Thread 消息体: `agents/{agent_id}/conversations/threads/{thread_id}.jsonl`
- Channel 数据: 单独 JSON 文件 (ChannelStore)
- Agent 身份: Markdown 文件

**Keyring (安全存储)**:
- API Keys 通过系统原生 Keyring 存储
- 前端通过 `useApiKeys` hook 管理

### Agent Runtime 抽象

```rust
// 核心 trait — 所有 runtime 必须实现
pub trait AgentRuntime: Send + Sync {
    fn execute(&self, params: ExecuteParams) -> Result<Receiver<RuntimeEvent>>;
    fn is_ready(&self) -> bool;
    fn name(&self) -> &str;
    fn health_check(&self) -> RuntimeHealth;
    fn stop(&self) -> Result<()>;
}

// 已实现: Claude Code (claude-code), Codex (codex)
// 规划中: Gemini (gemini)
```

每个 Runtime 支持：
- 流式输出 (通过 `Receiver<RuntimeEvent>`)
- Session 持续 (通过 `session_id`)
- Tool use / 结构化输出
- CLI 健康检测
- 超时控制

### @mention 系统

1. **前端**: `MentionAutocomplete` 组件提供自动补全，`renderMentionText` 将消息中的 @mention 渲染为彩色 agent 药丸
2. **后端**: `mention.rs` 解析消息中的 @mention，确定目标 Agent 并路由
3. **Runtime 标签**: 自动补全中显示 runtime 类型 (Claude Code / Codex / Gemini)

## 已完成功能 (27 个 Feature)

| Feature | 完成日期 | 说明 |
|---------|---------|------|
| feat-project-init | 04-08 | Tauri V2 + React 19 脚手架 |
| feat-claude-runtime | 04-08 | Claude Code Runtime 对接 |
| feat-style-consistency | 04-08 | 原型 MVP 移植 |
| feat-agent-workspace-design | 04-08 | Workspace 与身份系统 |
| feat-project-review | 04-08 | 项目设计评审 |
| feat-rename-agentszone | 04-08 | 项目重命名 |
| feat-agent-status | 04-09 | Agent 状态与选择器 |
| feat-thread-chat | 04-09 | Thread 1对1 对话 |
| feat-conversation-store | 04-09 | 对话持久化 |
| feat-channel-infra | 04-09 | Channel 基础设施 |
| feat-channel-multi-agent | 04-09 | Channel 多 Agent 协作 |
| feat-thread-context-inject | 04-09 | Thread Context 注入 |
| feat-agent-profile-page | 04-09 | Agent Profile 页 |
| feat-agent-create-ui | 04-09 | Agent 创建 UI |
| feat-thread-panel-live | 04-09 | ThreadPanel 真实数据 |
| feat-workspace-browser | 04-09 | Workspace 文件浏览器 |
| feat-apikey-management-ui | 04-10 | API Key 管理 |
| feat-skills-management | 04-10 | Skills 管理 |
| feat-activity-log | 04-10 | Activity 日志 |
| feat-data-storage | 04-10 | SQLite + JSONL 混合存储 |
| feat-svg-icon-system | 04-10 | SVG Icon 系统 |
| feat-agent-runtime-model | 04-10 | Runtime 数据模型 & trait 泛化 |
| feat-agent-runtime-ui | 04-10 | Agent 创建 UI Runtime 选择 |
| feat-agent-runtime-exec | 04-10 | 多 Runtime 对话执行 |
| feat-agent-edit | 04-10 | Agent 编辑能力 |
| channel-compaction | 04-10 | Channel 对话摘要压缩 |
| mention-rendering | 04-10 | @mention 彩色渲染优化 |

## 待开发功能 (Pending)

| Feature | 优先级 | 说明 |
|---------|--------|------|
| fix-channel-ui-bugs | 85 | Channel @Agent UI 修复 (thinking 状态 + icon 渲染) |
| fix-delete-and-render | 80 | 修复删除功能与渲染状态逻辑 |
| feat-agent-workspace-persist | 75 | Agent Workspace 对话持久化 |
| feat-sidebar-style | 70 | Sidebar 标题更名与面板可调宽度 |
| feat-md-rendering | 60 | Markdown 渲染优化 & Tool Call 结构化展示 |

## UI 设计风格

**新粗野主义 (Neo-Brutalism)** 风格：

- **主色调**：高饱和亮黄色 (Lemon Yellow) 侧边栏
- **背景**：米白色/浅杏色 (Off-white/Beige)
- **强调色**：青色 (活跃)、粉色/红色 (警告/高亮)
- **线条**：粗重黑色描边 (2-3px)
- **头像**：SVG 图标 + 像素风
- **字体**：等宽/终端字体
- **布局**：经典三栏 — Sidebar | Channel View | Thread/Detail
- **@mention**：彩色 agent 药丸 (brutal-cyan/pink/yellow/...) + emoji + 粗边框

## 关键设计决策

| Decision | Choice | Rationale |
|----------|--------|-----------|
| 应用形态 | Tauri 桌面应用 | 原生性能 + 本地文件系统直接访问 |
| Agent Runtime | 多 runtime trait 抽象 | 支持 Claude Code / Codex / Gemini，按需扩展 |
| 对话模型 | Channel (多Agent) + Thread (1对1) | 场景化对话隔离 |
| 触发机制 | @Agent 显式触发 | 用户明确控制何时调用 LLM |
| 上下文管理 | 滑动窗口 + 自动摘要 | 显式控制 token 预算，超阈值自动压缩 |
| 存储 | SQLite + JSONL + Keyring | SQLite 查询快，JSONL 追加写，Keyring 安全 |
| 状态管理 | useState + switch | 轻量，无 React Router |
| IPC | Tauri v2 invoke/listen | 前端 ↔ Rust 双向通信，Event 流式推送 |
| Feature 管理 | feature-workflow 系统 | 规范化开发流程，自动归档 |

## Critical Rules

### Must Follow

- 使用 `@tauri-apps/api` **v2** 语法（非 v1）
- `cn()` 合并样式（clsx + tailwind-merge）
- 类型定义集中在 `types.ts`
- @Agent 触发必须经过上下文编排引擎，不能直接透传全部历史
- 对话记录必须持久化到 JSONL，不能仅内存保存
- Agent Runtime 必须通过 trait 抽象，新增 runtime 需实现 `AgentRuntime` trait
- 上下文编排必须考虑 token 预算，不能超出 Agent context window
- Thread 最多一层，不支持嵌套
- Channel 消息数超阈值时必须触发自动摘要
- UTF-8 多字节字符处理必须使用 char-based 操作

### Must Avoid

- 不在前端硬编码 API Keys
- 不引入 React Router
- 不在前端 mock 终端 I/O
- 不在 Thread 内触发新的 @Agent（只有 Channel 顶层可触发）
- 不跳过上下文编排直接调用 LLM API
- 不使用云端存储，全部本地文件
- 不使用 byte-based 索引截断 UTF-8 字符串（会导致 panic）

## Update Log

- 2026-04-06: v1 — 初始 project-context 创建
- 2026-04-06: v2 — 确定技术栈 (React + Tauri Rust)
- 2026-04-06: v3 — 核心架构重构：Channel=对话容器, @Agent=触发器, 上下文编排引擎, 双Runtime, JSONL驱动
- 2026-04-10: v4 — 存储架构升级为 SQLite + JSONL 混合方案
- 2026-04-10: v5 — 全面更新：27 个 feature 完成，新增 Channel 自动摘要压缩、多 Runtime 支持 (Claude Code + Codex)、@mention 彩色渲染、Agent 编辑、SVG Icon 系统。更新实际目录结构与架构图。
