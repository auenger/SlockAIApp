# SlockAI

> AI 原生协作桌面应用 — 人类 @Agent 触发对话，上下文编排驱动 Claude Code / Codex，文档/JSONL 持久化。

## 核心模型

SlockAI 的设计极简，围绕五个核心概念构建：

| 概念 | 说明 |
|------|------|
| **Channel** | 对话容器，不同 Channel 存储独立对话记录 |
| **@Agent** | 触发器，在 Channel 中 @某个 Agent 触发 LLM 请求 |
| **上下文编排** | 核心引擎，决定对话历史的压缩与全量携带策略 |
| **Workspace** | Agent 文件目录，存放对话记录与输出 |
| **JSONL / Markdown** | 文件驱动持久化，不使用数据库 |

### 交互流

```
用户在 Channel 输入消息 → @Claude 触发 →
  上下文编排 (history + compression) →
    调用 Claude Code / Codex API →
      Agent 响应写回对话 → 持久化到 JSONL
```

## 技术栈

| Category | Technology |
|----------|-----------|
| Frontend | React 19 + TypeScript 5.8 + Tailwind CSS 4 |
| Desktop Shell | **Tauri V2** (Rust 后端) |
| Build | Vite 6 |
| Agent Runtime | Claude Code (Anthropic) / Codex (OpenAI) |
| Storage | JSONL (对话记录) + Markdown (文档) |
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
│  │  │  History → Compressor → Assembler  │      │ │
│  │  └──────────────┬─────────────────────┘      │ │
│  │                 │                              │ │
│  │  ┌──────────────┴─────────────────────┐      │ │
│  │  │  Agent Runtime Layer               │      │ │
│  │  │  Claude Code  │  Codex             │      │ │
│  │  └────────────────────────────────────┘      │ │
│  │                                              │ │
│  │  ┌────────────────────────────────────┐      │ │
│  │  │  Storage (JSONL / Markdown)        │      │ │
│  │  └────────────────────────────────────┘      │ │
│  └──────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

## UI 设计

采用**新粗野主义 (Neo-Brutalism)** 风格：

- 高饱和亮黄色侧边栏
- 米白色/浅杏色背景
- 粗重黑色描边 (2-3px)
- 像素风头像，等宽字体
- 经典三栏布局

## 项目结构

```
SlockAI/
├── ReactDemo/                    # React 原型 (Vite)
│   └── slockai-prototype/
├── PMFile/                       # 产品管理文档
├── PMDM/                         # 产品分析文档
├── feature-workflow/             # Feature 工作流配置
├── features/                     # Feature 队列与归档
└── project-context.md            # AI Agent 共享知识库
```

## 开发

```bash
# React 原型
cd ReactDemo/slockai-prototype
npm install
npm run dev
```

## License

Private
