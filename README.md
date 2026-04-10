# AgentsZone

> AI 原生协作桌面应用 — 人类 @Agent 触发对话，上下文编排 + 自动摘要驱动 Claude Code / Codex / Gemini，SQLite + JSONL 持久化。

## 核心模型

| 概念 | 说明 |
|------|------|
| **Channel** | 多 Agent 协作对话容器，支持 @mention 触发与自动上下文压缩 |
| **Thread** | 1 对 1 Agent 深度对话，独立上下文 |
| **@Agent** | 触发器，在 Channel 中 @某个 Agent 触发 LLM 请求 |
| **上下文编排** | 滑动窗口 + 自动摘要，控制 token 预算 |
| **Workspace** | Agent 文件目录，存放身份、对话记录与输出 |
| **SQLite + JSONL** | 混合存储：SQLite 管理元数据，JSONL 追加写入消息体 |

### 交互流

```
用户在 Channel 输入消息 → @Claude 触发 →
  上下文编排 (summary + recent messages) →
    调用 Claude Code / Codex Runtime →
      Agent 响应写回对话 → 持久化到 SQLite + JSONL
        消息数超阈值时自动触发摘要压缩
```

## 技术栈

| Category | Technology |
|----------|-----------|
| Frontend | React 19 + TypeScript 5.8 + Tailwind CSS 4 |
| Desktop Shell | **Tauri V2** (Rust 后端) |
| Build | Vite 6 |
| Agent Runtime | Claude Code (Anthropic) / Codex (OpenAI) / Gemini (规划中) |
| Storage | SQLite (元数据) + JSONL (消息体) + Keyring (密钥) |
| IPC | Tauri v2 `invoke()` / `listen()` |

## 架构概览

```
┌──────────────────────────────────────────────────┐
│            Tauri Desktop Application              │
│                                                   │
│  ┌─────────────────────────────────────────────┐ │
│  │        React Frontend (WebView)              │ │
│  │   Sidebar │ Channel View │ Thread/Detail     │ │
│  └──────────────────┬──────────────────────────┘ │
│                     │ IPC                          │
│  ┌──────────────────┴──────────────────────────┐ │
│  │           Rust Backend (Tauri)               │ │
│  │                                              │ │
│  │  ┌────────────────────────────────────┐      │ │
│  │  │    上下文编排引擎 (Orchestrator)    │      │ │
│  │  │  Summary + Recent → Assembler      │      │ │
│  │  │  Auto-Compact (>30 msgs)           │      │ │
│  │  └──────────────┬─────────────────────┘      │ │
│  │                 │                              │ │
│  │  ┌──────────────┴─────────────────────┐      │ │
│  │  │  Agent Runtime Layer (trait)       │      │ │
│  │  │  Claude Code │ Codex │ Gemini(TBD) │      │ │
│  │  └────────────────────────────────────┘      │ │
│  │                                              │ │
│  │  ┌────────────────────────────────────┐      │ │
│  │  │  Storage Layer                     │      │ │
│  │  │  SQLite (metadata) + JSONL (msgs)  │      │ │
│  │  │  Keyring (API keys)               │      │ │
│  │  └────────────────────────────────────┘      │ │
│  └──────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

## 已实现功能

- **多 Agent 管理** — 创建、编辑、删除 Agent，自定义身份 (emoji, 名称, 描述, SOUL.md)
- **多 Runtime 支持** — Claude Code / Codex 双 runtime，Agent 可绑定不同 runtime
- **Channel 多 Agent 协作** — @mention 触发、自动路由消息、彩色 mention 渲染
- **Thread 1 对 1 对话** — 独立上下文的深度对话模式
- **上下文编排** — 滑动窗口 + 自动摘要压缩 (>30 条消息自动触发)
- **Workspace 文件浏览器** — 浏览 Agent 工作目录
- **Skills 管理** — Agent 技能配置
- **Activity 日志** — 操作历史记录
- **API Key 管理** — Keyring 安全存储
- **SVG Icon 系统** — 自定义 Agent 图标

## UI 设计

采用**新粗野主义 (Neo-Brutalism)** 风格：

- 高饱和亮黄色侧边栏
- 米白色/浅杏色背景
- 粗重黑色描边 (2-3px)
- 像素风头像，等宽字体
- 经典三栏布局

## 项目结构

```
AgentsZone/
├── src/                          # React 前端
│   ├── components/               # UI 组件
│   └── lib/                      # Hooks & IPC 封装
├── src-tauri/                    # Rust 后端 (Tauri V2)
│   └── src/
│       ├── commands/             # Tauri Commands (channel, thread, activity)
│       ├── runtime/              # Agent Runtime (Claude, Codex, Registry)
│       ├── storage/              # SQLite + JSONL + Keyring + Migrations
│       ├── workspace/            # Agent/Channel/Thread/Mention 管理
│       └── context/              # 上下文组装
├── feature-workflow/             # Feature 工作流配置
├── features/                     # Feature 队列与归档
└── project-context.md            # AI Agent 共享知识库
```

## 开发

```bash
# 前端开发
npm install
npm run dev

# Tauri 桌面应用
npm run tauri dev

# 构建
npm run tauri build
```

## License

Private
