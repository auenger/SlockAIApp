# SlockAI 项目设计 Review 报告

> **审查日期**: 2026-04-08
> **审查范围**: 全部已完成 feature 的代码和设计文档
> **已完成 Features**: feat-project-init, feat-claude-runtime, feat-style-consistency, feat-agent-workspace-design

---

## 一、总体评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 架构设计 | 7/10 | 模块划分清晰，但部分核心模块尚未实现 |
| 代码质量 | 5/10 | Rust 端结构良好但缺失关键实现，前端存在大量 mock 代码 |
| 设计一致性 | 4/10 | project-context.md 与实际代码有显著差距 |
| 功能完整度 | 3/10 | 核心功能（对话存储、上下文编排、流式响应）均未实现 |
| 安全性 | 5/10 | 密钥管理有基础实现，但缺乏输入验证和权限控制 |

**核心结论**: 项目架构设计合理、模块边界清晰，但**实现深度严重不足**。4 个已完成 feature 主要完成了脚手架和基础框架搭建，核心业务逻辑（上下文编排引擎、JSONL 对话存储、流式 Agent 交互）尚未落地。当前状态距离 MVP 仍有较大差距。

---

## 二、严重问题 (Critical)

### C1. 上下文编排引擎仅有骨架，无实际编排逻辑

**位置**: `src-tauri/src/context/mod.rs`

**现状**: `ContextBuilder` 仅实现了从文件读取 system prompt 的功能，但 project-context.md 中描述的核心编排逻辑**完全未实现**：
- 无对话历史读取（JSONL）
- 无滑动窗口策略
- 无 token 预算控制
- 无重要性评分
- 无对话压缩

**影响**: 这是系统最核心的模块，缺失意味着 Agent 调用无法携带有效的上下文。

**建议**: 作为下一个最高优先级 feature 实现，包含：
1. JSONL 对话历史读取
2. 滑动窗口 + 压缩策略
3. Token 预算管理
4. 上下文组装

---

### C2. JSONL 对话存储完全缺失

**现状**: `storage/mod.rs` 为空文件，`project-context.md` 中规划的 `jsonl.rs` 和 `markdown.rs` 均未创建。

**影响**: 对话无法持久化，刷新即丢失。这是 project-context.md 中明确标注为 "Must Follow" 的规则。

**建议**: 实现 JSONL 存储层，包含：
1. 对话记录的追加写入
2. 按日期/频道读取
3. 简单的索引和查询

---

### C3. 前端 AI 服务完全使用 Mock，无真实集成

**位置**: `src/components/MainContent.tsx`

**现状**: MainContent 中的 AI 响应全部是硬编码的 mock 数据和定时器模拟。虽然 `ipc.ts` 和 `useAgentRuntimes.ts` 已搭建 IPC 框架，但 MainContent 组件**完全没有调用**这些接口。

**影响**: 用户无法获得真实的 AI 响应，产品不可用。

**对比**: MVP 原型 (`ReactDemo/slockai-prototype/src/services/geminiService.ts`) 已实现真实的 Gemini API 调用，但主项目未复用此实现。

**建议**: 将 MainContent 的消息处理逻辑改为通过 Tauri IPC 调用 Rust 后端的 Agent Runtime。

---

### C4. Agent Runtime 仅支持检测，不支持实际执行

**位置**: `src-tauri/src/runtime/claude.rs`

**现状**: `ClaudeRuntime` 实现了 `detect()` 和 `health_check()`，但 `execute()` 方法虽然有实现框架，实际的 Claude Code CLI 集成仍不完整。流式事件的解析和转发未经过实际验证。

**影响**: 即使前端接入了 IPC，也无法真正调用 Claude Code。

**建议**: 完善 execute 实现，确保：
1. Claude Code CLI 进程管理
2. 流式输出解析（SSE/JSONL）
3. 错误处理和超时
4. 端到端测试验证

---

## 三、架构问题 (High)

### H1. 前端状态管理过于分散

**位置**: `src/App.tsx`, `src/components/MainContent.tsx`

**现状**: 所有状态通过 `useState` 管理，App.tsx 中有大量顶层状态（activeTab, channels, threads, agents, messages 等），通过 props 层层传递。MainContent 内部自行管理消息列表、任务列表、文件选择等独立状态。

**问题**:
- 状态分散在多个组件中，难以维护和调试
- 组件间状态共享通过 props 透传，层级过深
- 无状态持久化机制，刷新丢失所有数据

**建议**: 引入轻量状态管理方案（如 Zustand 或 React Context），至少解决：
1. 全局 Agent/Runtime 状态
2. Channel 和 Message 状态
3. 连接 Rust 后端的状态同步

---

### H2. 前端组件职责过重

**位置**: `src/components/MainContent.tsx` (约 700+ 行)

**现状**: MainContent 组件承担了 6 个标签页（CHAT/TASKS/WORKSPACE/SKILLS/ACTIVITY/PROFILE）的全部渲染和逻辑，是典型的"上帝组件"。

**问题**:
- 单文件代码量过大
- 所有标签页的逻辑耦合在一起
- 难以单独测试和优化

**建议**: 拆分为独立标签页组件，MainContent 仅负责标签切换和路由。

---

### H3. ReactDemo 原型与主项目代码重复

**现状**: `ReactDemo/slockai-prototype/` 和 `src/` 存在大量重复代码：
- `components/Sidebar.tsx`, `MainContent.tsx`, `ThreadPanel.tsx`, `Modals.tsx` 几乎完全重复
- `types.ts` 类型定义重复
- `lib/utils.ts` 工具函数重复

**问题**:
- 维护成本翻倍
- 两边修改容易不同步
- ReactDemo 中的 Gemini 服务集成未被复用

**建议**: 
1. 明确 ReactDemo 的定位（设计参考 or 功能参考）
2. 如果仅作参考，应从 git 跟踪中移除或归档
3. 如果需要保留功能，提取共享逻辑到独立模块

---

### H4. Rust 后端缺少统一的错误处理体系

**现状**: 不同模块使用独立的错误类型（`ManagerError`, `ContextError`, `TemplateError`），没有统一错误 trait 或错误转换机制。

**问题**:
- Tauri command 层需要处理多种错误类型
- 错误上下文在跨模块传递时丢失
- 前端难以统一处理错误

**建议**: 定义统一的 `AppError` 枚举，各模块错误通过 `From/Into` 转换。

---

## 四、设计问题 (Medium)

### M1. project-context.md 与实际实现不一致

| 设计描述 | 实际状态 |
|----------|----------|
| 三栏布局 (Sidebar/Main/Detail) | 已实现，基本一致 |
| Channel = 对话容器 | 仅前端 UI 占位，后端无 Channel 数据结构 |
| @Agent = 触发器 | UI 有基础，但后端无实际触发逻辑 |
| 上下文编排引擎 (核心) | 仅骨架，核心逻辑未实现 |
| JSONL 对话存储 | 完全缺失 |
| Codex Runtime | 未开始实现 |
| Markdown 文档存储 | 未实现 |
| Thread (对话分支) | 前端 UI 有占位，后端无支持 |

---

### M2. Codex Runtime 完全未开始

**现状**: project-context.md 规划了 Claude Code + Codex 双 runtime，但 `runtime/` 模块仅有 `claude.rs`，没有 `codex.rs`。

**影响**: 用户无法使用 OpenAI 系模型。

**建议**: 优先完成 Claude Runtime 端到端验证后再启动 Codex 支持。不要同时开发两个 runtime。

---

### M3. 会话管理机制薄弱

**位置**: `src-tauri/src/commands/mod.rs` — `AgentSessionState`

**现状**: 仅有简单的 `active_agent_id` 存储，缺少：
- 会话创建/销毁生命周期
- 会话超时和清理
- 多会话并行支持

**建议**: 实现 `SessionManager`，管理 Agent 会话的完整生命周期。

---

### M4. 工作空间模板系统未完善

**位置**: `src-tauri/src/workspace/templates.rs`

**现状**: 模板系统有基础框架（默认模板、SOUL.md 层级覆盖），但缺乏：
- 用户自定义模板支持
- 模板验证
- 模板版本管理

**优先级较低**，可在核心功能完善后再迭代。

---

### M5. 前端 IPC 层与后端命令不完全对齐

**现状**:
- `src/lib/ipc.ts` 定义了 IPC 调用接口
- `src/lib/useAgentRuntimes.ts` 实现了 Agent 管理 Hook
- 但 MainContent 等组件未使用这些接口
- 部分后端 Tauri command 缺少前端对应调用

**建议**: 建立 IPC 接口清单，确保前后端命令一一对应。

---

## 五、功能缺失清单 (按优先级排序)

| # | 功能 | 优先级 | 复杂度 | 说明 |
|---|------|--------|--------|------|
| 1 | JSONL 对话存储 | P0 | M | 核心基础设施，无此无法持久化 |
| 2 | 上下文编排引擎 | P0 | L | 系统核心，需要历史读取 + 压缩 + 预算控制 |
| 3 | 前端接入真实 AI 调用 | P0 | M | 替换 mock 为 IPC 调用 |
| 4 | Claude Code 端到端验证 | P0 | M | 确保完整的发送-接收-渲染链路 |
| 5 | Channel 数据模型 (Rust) | P1 | S | 后端需要 Channel 结构体和 CRUD |
| 6 | 统一错误处理 | P1 | S | 定义 AppError 枚举 |
| 7 | 前端状态管理重构 | P1 | M | 引入 Zustand 或 Context |
| 8 | MainContent 组件拆分 | P1 | M | 按标签页拆分 |
| 9 | 会话生命周期管理 | P2 | S | SessionManager |
| 10 | Codex Runtime | P2 | M | 在 Claude Runtime 验证后启动 |
| 11 | Thread (对话分支) | P2 | M | 需要存储层和上下文编排支持 |
| 12 | 清理 ReactDemo 重复 | P2 | S | 归档或提取共享逻辑 |

---

## 六、安全风险评估

### S1. API 密钥存储 (已部分实现)
- `storage/keyring.rs` 使用操作系统密钥环，方案合理
- 但缺少密钥验证、过期检查
- 前端 IPC 缺少权限校验

### S2. 输入验证缺失
- Tauri command 层缺乏输入参数验证
- 用户消息内容未做 sanitize
- Agent ID 等标识符未做格式校验

### S3. 文件系统访问控制
- Workspace 模块直接操作文件系统
- 缺乏路径遍历攻击防护
- 未限制文件操作范围到 Workspace 目录内

---

## 七、推荐的开发路线图

### Phase 1: 核心可用 (优先)
1. **JSONL 存储层** — 实现对话记录的读写
2. **上下文编排引擎** — 实现基础编排（全量历史 + 简单截断）
3. **前端 AI 集成** — 将 mock 替换为真实 IPC 调用
4. **端到端验证** — 确保 用户输入 → 上下文编排 → Claude → 流式渲染 完整链路

### Phase 2: 体验优化
5. **前端状态管理** — 引入 Zustand
6. **组件拆分** — MainContent 按功能拆分
7. **统一错误处理** — AppError 体系
8. **Channel 后端模型** — Rust 端 Channel CRUD

### Phase 3: 功能扩展
9. **Codex Runtime** — OpenAI 系支持
10. **Thread (对话分支)** — 对话分支功能
11. **会话管理** — 完整生命周期
12. **上下文压缩优化** — 滑动窗口 + 重要性评分

---

## 八、总结

SlockAI 项目的**架构设计方向正确**：Tauri + Rust 后端、Agent Runtime 抽象、上下文编排引擎的设计思路都是合理的。问题在于**实现深度不够**，4 个已完成 feature 主要交付了项目脚手架和 UI 框架，核心业务逻辑尚未落地。

**最关键的行动项**:
1. 实现 JSONL 存储层（没有它，其他一切都是空中楼阁）
2. 实现上下文编排引擎（系统的核心价值所在）
3. 将前端 mock 替换为真实 AI 调用（让产品可用）

完成这三项后，项目将达到真正的 MVP 状态，可以在此基础上迭代优化。
