# CLAUDE.md — AgentsZone 项目指南

## 项目概述

AgentsZone 是一个 AI 原生协作桌面应用，基于 **Tauri V2 (Rust) + React 19** 构建。用户通过 @Agent 在 Channel 中触发多 Agent 协作对话，支持 Claude Code 和 Codex 双 Runtime。

## 技术栈

- **Frontend**: React 19 + TypeScript 5.8 + Tailwind CSS 4 + Vite 6
- **Backend**: Rust (Tauri V2)
- **Storage**: SQLite (元数据) + JSONL (消息体) + Keyring (API 密钥)
- **Runtime**: Claude Code CLI / Codex CLI / Gemini (规划中)

## 关键约定

### 代码风格

- 使用 `cn()` (clsx + tailwind-merge) 合并样式
- TypeScript 类型定义集中在 `src/types.ts`
- Rust 端使用 `log::info!` / `log::error!` 日志
- 前端 IPC 调用封装在 `src/lib/ipc.ts`，hooks 在 `src/lib/use*.ts`

### 架构规则

- Agent Runtime 通过 trait 抽象 (`AgentRuntime`)，所有 runtime 实现统一接口
- 上下文编排必须经过编排引擎，不能直接透传全部历史
- Channel 消息通过滑动窗口 + 自动摘要压缩管理 token 预算
- Thread 最多一层嵌套，不支持递归
- 对话持久化到 SQLite + JSONL，不能仅内存保存
- API Key 通过 Keyring 安全存储，不在前端硬编码
- 不使用 React Router，视图切换通过 state 管理
- 不使用云端存储，全部本地文件

### IPC 通信

- Tauri v2 语法：前端 `invoke()` 调用 Rust commands，`listen()` 接收 Rust events
- Command 注册在 `src-tauri/src/lib.rs`
- Event 命名格式：`channel://xxx`、`thread://xxx`

### 存储模式

```
写入消息: JSONL.append(message) + SQLite.update(metadata)
查询列表: SQLite.query("SELECT ...")
加载历史: JSONL.read_all(path)
Channel 数据: 单独 JSON 文件 (ChannelStore)
```

## 目录结构

```
src/                        # React 前端
  components/               # UI 组件 (MainContent, Sidebar, ThreadPanel, etc.)
  lib/                      # Hooks (useChannel, useThreadChat, etc.) + IPC
  types.ts                  # TypeScript 类型定义

src-tauri/src/              # Rust 后端
  commands/                 # Tauri Commands
    channel.rs              # Channel CRUD + 消息发送 + 上下文编排 + 自动摘要
    thread.rs               # Thread 对话
    activity.rs             # Activity 日志
  runtime/                  # Agent Runtime 实现
    claude.rs               # Claude Code CLI runtime
    codex.rs                # Codex CLI runtime
    registry.rs             # Runtime 注册中心
  storage/                  # 存储层
    db.rs                   # SQLite 数据库 + migrations
    jsonl.rs                # JSONL 消息存储
    keyring.rs              # API Key 安全存储
    activity.rs             # Activity 日志存储
  workspace/                # Workspace 管理
    manager.rs              # AgentManager (多 Agent 管理)
    agent.rs                # 单个 Agent 管理
    channel.rs              # Channel 数据管理
    thread.rs               # Thread 管理
    mention.rs              # @mention 解析
    identity.rs             # Agent 身份系统
    templates.rs            # Agent 模板
  context/                  # 上下文组装
```

## Feature 工作流

项目使用 `feature-workflow` 管理开发流程：

- **配置**: `feature-workflow/config.yaml`
- **队列**: `feature-workflow/queue.yaml`
- **Feature 目录**: `features/` (活跃) + `features/archive/` (归档)
- **分支命名**: `feature/feat-xxx` / `feature/fix-xxx`
- **完成流程**: commit → merge to main → create tag → cleanup worktree/branch

## 常用命令

```bash
npm run dev              # 前端开发服务器
npm run tauri dev        # Tauri 桌面应用开发模式
npm run tauri build      # 生产构建
npm run test             # 运行测试 (Vitest)
```

## 当前开发重点

详见 `feature-workflow/queue.yaml` 中的 pending features。
