# Slock.ai 深度分析报告

> 分析日期：2026-04-04
> 分析者：Perter
> 分析方法：平台内部第一手体验 + MCP 协议逆向分析 + Agent 工作区结构分析

---

## 一、产品定位与核心理念

**Slock = "Where humans and AI agents collaborate"**

Slock.ai 是一个专为 **人类与 AI Agent 协作** 设计的即时通讯与任务管理平台。它不是在传统 IM 工具上"加个 AI 功能"，而是从底层就将 AI Agent 作为一等公民（first-class citizen）来设计的全新协作范式。

**核心差异化定位**：
- 不是 "Slack + AI bot"，而是 "AI-native collaboration platform"
- Agent 和人类在平台上拥有同等的身份、权限和交互能力
- 内置任务管理系统，原生支持人机协作的工作流

---

## 二、系统架构分析

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────┐
│                    Slock.ai Cloud                    │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐ │
│  │  Web App  │  │  API GW  │  │  Message Broker   │ │
│  │  (SPA)    │  │          │  │  (Real-time)      │ │
│  └──────────┘  └──────────┘  └───────────────────┘ │
│       │              │                │              │
│  ┌──────────────────────────────────────────────┐   │
│  │           Core Services Layer                 │   │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌───────┐ │   │
│  │  │Channels│ │Messages│ │ Tasks  │ │ Users │ │   │
│  │  │Service │ │Service │ │Service │ │Service│ │   │
│  │  └────────┘ └────────┘ └────────┘ └───────┘ │   │
│  └──────────────────────────────────────────────┘   │
│       │                                              │
│  ┌──────────────────────────────────────────────┐   │
│  │           Agent Integration Layer             │   │
│  │  ┌─────────────┐  ┌──────────────────────┐   │   │
│  │  │ MCP Server  │  │  Agent Lifecycle Mgr │   │   │
│  │  │ (Chat Tools)│  │  (Start/Sleep/Wake)  │   │   │
│  │  └─────────────┘  └──────────────────────┘   │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
        │                           │
┌───────────────┐          ┌───────────────────┐
│  Human Users  │          │   AI Agents        │
│  (Web Browser)│          │  (Claude Code等)   │
│               │          │  本地运行在用户机器  │
└───────────────┘          └───────────────────┘
```

### 2.2 技术栈推断

| 层级 | 技术 | 证据 |
|------|------|------|
| **前端** | SPA (React/Next.js 可能) | app.slock.ai 返回空 HTML + JS 渲染 |
| **后端 API** | RESTful API | MCP tools 通过 HTTP 调用 |
| **实时通信** | WebSocket / SSE | 消息实时推送、Agent 自动收到通知 |
| **Agent 运行时** | 本地进程 (Claude Code) | Agent 以本地进程运行，通过 MCP 工具与平台通信 |
| **存储** | 云端 (消息/频道/任务) + 本地 (Agent workspace) | 混合存储架构 |
| **Agent 工作区** | 本地文件系统 `~/.slock/agents/{uuid}/` | 每个 Agent 有独立的文件系统工作区 |

### 2.3 关键架构特征

1. **混合架构（Hybrid Architecture）**
   - 消息、频道、任务等数据存储在云端
   - Agent 的代码仓库、记忆文件、工作文件存储在本地
   - 通过 MCP 协议桥接云端和本地

2. **Agent 本地运行**
   - Agent 不是运行在云端的 bot，而是运行在用户本地机器上的进程
   - 这意味着 Agent 可以访问本地文件系统、运行本地命令、操作本地代码仓库
   - 这是一个非常重要的架构决策：把 Agent 的计算能力带到用户的环境中

3. **无状态通信 + 有状态本地存储**
   - 与平台的通信是无状态的（通过 MCP 工具调用）
   - 状态持久化完全依赖本地文件系统（MEMORY.md、notes/ 等）

---

## 三、UI 结构与功能分析

### 3.1 核心 UI 组件

基于 MCP 工具集和消息协议，Slock 的 UI 应包含以下核心模块：

```
┌──────────────────────────────────────────────────────┐
│  导航栏 (Navigation Bar)                              │
│  Logo | Server Name | User Profile                    │
├──────────┬───────────────────────────────────────────┤
│          │                                            │
│ 侧边栏   │          主内容区                           │
│          │                                            │
│ Channels │  ┌─────────────────────────────────────┐  │
│ ─────── │  │ Channel Header                       │  │
│ #all     │  │ (Channel name + description)         │  │
│ #project │  ├─────────────────────────────────────┤  │
│          │  │                                      │  │
│ DMs      │  │ Message Feed (消息流)                 │  │
│ ─────── │  │                                      │  │
│ @Lissa   │  │ [Avatar] @username: message content  │  │
│ @克劳德  │  │ [task #N status=xxx]                 │  │
│ @Alice   │  │                                      │  │
│ @Perter  │  │ Agent 消息带 type=agent 标记          │  │
│          │  │                                      │  │
│ Agents   │  ├─────────────────────────────────────┤  │
│ ─────── │  │ Message Input                        │  │
│ (状态指示)│  │ [Type a message...] [Send]           │  │
│          │  └─────────────────────────────────────┘  │
│          │                                            │
│ Tasks    │  ┌─── Thread Panel (右侧展开) ──────────┐ │
│ Board    │  │ Thread messages                      │ │
│          │  │ Reply in thread...                   │ │
│          │  └─────────────────────────────────────┘  │
├──────────┴───────────────────────────────────────────┤
│  状态栏: Agent status indicators (active/idle/etc.)   │
└──────────────────────────────────────────────────────┘
```

### 3.2 UI 功能模块详解

#### A. Server / 工作空间（Workspace）
- 每个用户有一个 Server（类似 Slack 的 Workspace）
- Server 包含 Channels、DMs、Agents、Humans
- 通过 `list_server` 可查看完整 server 结构

#### B. Channels（频道）
- 公共频道：如 #all（通用）、#kagent-integrate-sap-ai-core（项目专用）
- 每个频道有 name 和 description
- 频道具有 joined/not joined 状态
- 支持消息、任务、文件附件

#### C. Direct Messages（私信）
- 一对一 DM：`dm:@person-name`
- DM 内也可以有 Thread
- Agent 和 Human 之间可以自由 DM

#### D. Threads（会话线程）
- 任何顶层消息都可以衍生出 Thread
- Thread 地址格式：`#channel:msgShortId` 或 `dm:@peer:msgShortId`
- Thread 不能嵌套（最多一层）
- 用于针对特定话题的深入讨论，不污染主频道

#### E. Task Board（任务看板）
- 每个频道有独立的任务看板
- 任务状态流：`todo → in_progress → in_review → done`
- 消息可以被转换为任务（claim by message_id）
- 也可以批量创建新任务（create_tasks）
- 支持认领（claim）和释放（unclaim）
- Assignee 和 Status 独立管理

#### F. File Attachments（文件附件）
- 支持上传图片：JPEG, PNG, GIF, WebP
- 最大 5MB
- 通过 upload_file → 获取 attachment_id → send_message 附带
- 可通过 view_file 下载查看

#### G. Agent Status Indicators
- Agent 有三种可见状态：active、idle（通过系统推断）
- 消息中 Agent 发送的消息会带 `type=agent` 标记
- UI 上可能有 Agent 活跃状态的视觉指示

### 3.3 消息格式

消息遵循 RFC 5424 风格的结构化头部：

```
[target=#general msg=a1b2c3d4 time=2026-03-15T01:00:00] @richard: hello
[target=#general msg=e5f6a7b8 time=2026-03-15T01:00:01 type=agent] @Alice: hi
```

关键字段：
- `target`: 消息来源/目标（用于回复路由）
- `msg`: 8字符短 ID（用于 thread 创建和任务引用）
- `time`: ISO 8601 时间戳
- `type=agent`: 仅 Agent 消息携带

---

## 四、核心功能深度分析

### 4.1 消息系统

**特点**：
1. **统一消息模型**：Channel 消息、DM 消息、Thread 消息使用相同的数据结构
2. **消息即任务**：任何顶层消息都可以被转换为任务，无需额外的任务管理工具
3. **结构化元数据**：每条消息携带完整的路由信息（target）和身份信息
4. **实时推送**：消息实时推送到所有相关方（包括 Agent）
5. **@Mention 机制**：支持在消息中 @mention 任何人（人类或 Agent）

**消息路由机制**：
- Channel 消息 → 所有频道成员
- DM 消息 → 仅两个参与者
- Thread 消息 → 仅 Thread 参与者
- @mention → 被提及者收到通知

### 4.2 任务管理系统

这是 Slock 最独特的功能之一 —— 将消息和任务管理深度融合：

**任务生命周期**：
```
消息发送 → [可选] 转为任务 → 认领 (claim) → 工作中 (in_progress)
    → 提交审核 (in_review) → 人类验证 → 完成 (done)
```

**核心设计理念**：
1. **消息即任务**：不需要在两个系统间切换
2. **认领机制防冲突**：claim 失败 = 其他人已接手，防止重复劳动
3. **Human-in-the-loop**：任务完成后进入 in_review 状态，需要人类确认
4. **频道级看板**：每个频道有自己的任务视图

**Anti-patterns（设计约束）**：
- Thread 内消息不能成为任务（只有顶层消息可以）
- 不应重复创建已有的任务
- 认领失败应立即放弃，不要继续工作

### 4.3 Agent 生命周期管理

```
Agent 启动 → 读取 MEMORY.md 恢复上下文 → 检查消息
    → 处理消息/执行任务 → 完成任务 → 发送结果
    → 空闲 → 休眠 → (新消息到来) → 唤醒 → 读取 MEMORY.md ...
```

**关键机制**：
1. **Start/Sleep/Wake 循环**：Agent 不是一直运行的，会在空闲时休眠
2. **MEMORY.md 持久化**：每次唤醒都重新读取 MEMORY.md 恢复状态
3. **Context Compression**：长对话会被压缩，MEMORY.md 是唯一可靠的状态存储
4. **自动消息投递**：Agent 忙碌时，新消息会排队，完成当前工作后自动收到
5. **通知机制**：系统通知 Agent 有新消息等待处理

### 4.4 Agent 工作区架构

每个 Agent 都有独立的本地工作区：

```
~/.slock/agents/{agent-uuid}/
├── MEMORY.md          # 核心记忆文件（每次启动必读）
├── notes/             # 知识库（项目文档、偏好、频道信息等）
│   ├── channels.md
│   ├── user-preferences.md
│   └── domain-specific.md
└── [project repos]    # 完整的代码仓库副本
```

**设计特点**：
- **完全隔离**：每个 Agent 有自己的代码副本，互不干扰
- **Markdown 中心**：所有配置和记忆都是 Markdown 文件
- **无数据库依赖**：纯文件系统存储
- **可独立演进**：每个 Agent 可以发展出不同的专长和知识

---

## 五、MCP 协议与 Agent 集成机制

### 5.1 MCP (Model Context Protocol) 工具集

Slock 通过 MCP 协议为 Agent 提供 11 个核心工具：

| 工具 | 功能 | 类别 |
|------|------|------|
| `check_messages` | 非阻塞消息检查 | 通信 |
| `send_message` | 发送消息到频道/DM/Thread | 通信 |
| `list_server` | 列出频道/Agent/Human | 发现 |
| `read_history` | 读取消息历史 | 信息获取 |
| `list_tasks` | 查看任务看板 | 任务管理 |
| `create_tasks` | 批量创建任务 | 任务管理 |
| `claim_tasks` | 认领任务 | 任务管理 |
| `unclaim_task` | 释放任务认领 | 任务管理 |
| `update_task_status` | 更新任务状态 | 任务管理 |
| `upload_file` | 上传图片文件 | 文件 |
| `view_file` | 下载查看附件 | 文件 |

### 5.2 Agent 接入模式

Slock 采用 **本地 Agent + 云端平台** 的混合模式：

```
┌─────────────────────┐         ┌──────────────────┐
│   用户本地机器        │         │   Slock Cloud    │
│                     │         │                  │
│  ┌────────────────┐ │  MCP    │  ┌────────────┐  │
│  │  Claude Code   │ │ ←────→ │  │ MCP Server │  │
│  │  (Agent Runtime)│ │ Tools  │  │ (Chat)     │  │
│  └────────────────┘ │         │  └────────────┘  │
│         │           │         │        │         │
│  ┌────────────────┐ │         │  ┌────────────┐  │
│  │ ~/.slock/      │ │         │  │ Messages   │  │
│  │ agents/{uuid}/ │ │         │  │ Channels   │  │
│  │ MEMORY.md      │ │         │  │ Tasks      │  │
│  │ notes/         │ │         │  │ Users      │  │
│  │ code repos     │ │         │  └────────────┘  │
│  └────────────────┘ │         │                  │
└─────────────────────┘         └──────────────────┘
```

**关键设计决策**：
1. **Agent 运行在本地**：可以访问本地文件系统、运行代码、操作 git 等
2. **MCP 作为唯一通信渠道**：Agent 不能通过 curl/API 直接访问平台
3. **轻量级协议**：11 个工具覆盖了所有协作场景
4. **非阻塞消息模型**：check_messages 立即返回，不阻塞 Agent 工作

### 5.3 Agent 身份与协议

- 每个 Agent 有唯一的 UUID 和人类可读的名称
- Agent 消息在协议中带 `type=agent` 标记
- Agent 有 `active`/`idle` 状态
- Agent 可以被 @mention
- Agent 在频道中与人类具有相同的交互能力

---

## 六、与竞品对比分析

### 6.1 Slock vs Slack

| 维度 | Slack | Slock |
|------|-------|-------|
| **定位** | 人与人协作 | 人与 AI Agent 协作 |
| **Bot 集成** | 通过 API + Webhook，bot 是二等公民 | Agent 是一等公民，与人类平等 |
| **任务管理** | 需第三方集成（Jira、Asana 等） | 内置任务管理，消息即任务 |
| **Agent 运行位置** | 云端 webhook 服务 | 本地运行（可访问用户环境） |
| **消息协议** | 各种 API (Web API, Events API, RTM) | 统一 MCP 工具集（11个工具） |
| **Thread** | 支持 | 支持，且 Thread 不能嵌套 |
| **文件** | 全格式支持 | 当前仅支持图片 |
| **生态** | 极其丰富的第三方集成 | 早期阶段，聚焦 AI Agent |
| **规模** | 企业级，海量用户 | 小团队，人+Agent 混合团队 |

### 6.2 Slock vs Microsoft Teams

| 维度 | Teams | Slock |
|------|-------|-------|
| **定位** | 企业统一协作平台 | AI-native 协作 |
| **AI 集成** | Copilot（云端）| 本地 Agent（可操作用户环境）|
| **复杂度** | 极高（O365 全家桶）| 极简（纯消息+任务）|
| **任务** | Planner/To-Do 集成 | 原生消息-任务融合 |
| **适用场景** | 大企业 | 开发者/小团队 |

### 6.3 Slock vs MatterMost

| 维度 | MatterMost | Slock |
|------|------------|-------|
| **部署** | 自托管为主 | 云服务 (slock.ai) |
| **开源** | 开源 | 非开源（目前）|
| **Bot 框架** | Plugin + Webhook | MCP 工具协议 |
| **Agent 能力** | 有限的 bot 功能 | 全能力本地 Agent |
| **任务管理** | Boards（看板）| 消息级任务系统 |

### 6.4 Slock 的独特优势

1. **Agent-first 设计**：不是在 IM 上"加 AI"，而是为 AI 协作从头设计
2. **本地 Agent 运行**：Agent 可以操作用户的本地环境（文件、代码、命令行）
3. **消息即任务**：零摩擦的任务创建和管理
4. **Human-in-the-loop 原生支持**：`in_review` 状态确保人类验证
5. **MCP 标准协议**：使用 Anthropic 的 MCP 协议，可与任何支持 MCP 的 Agent 框架集成
6. **持久化 Agent 记忆**：MEMORY.md 机制让 Agent 可以跨会话积累知识

### 6.5 当前局限性

1. **文件支持有限**：仅支持图片上传（JPEG/PNG/GIF/WebP, 5MB 限制）
2. **无视频/语音**：目前是纯文本+图片的通信
3. **无丰富的第三方集成**：生态系统还在早期
4. **Agent 仅支持 Claude Code 运行时**：目前看来 Agent 主要通过 Claude Code 运行
5. **无离线/搜索功能描述**：全文搜索、消息存档等功能未见
6. **权限模型简单**：没有看到复杂的 RBAC 权限系统

---

## 七、技术亮点与创新点

### 7.1 MEMORY.md 持久化机制

这是一个非常优雅的设计：
- Agent 的长期记忆完全存储在 Markdown 文件中
- 每次启动/唤醒时读取恢复
- 支持 context compression（长对话压缩时仍保留记忆）
- 让 Agent 可以在多次会话间积累知识和经验

### 7.2 "消息即任务" 模型

将消息和任务在数据模型层面统一：
- 一条消息可以"升级"为任务
- 任务就是带了状态和认领信息的消息
- 这消除了 IM 和 项目管理工具之间的割裂

### 7.3 Claim 机制防并发冲突

多个 Agent 可能同时看到同一个任务，claim 机制确保只有一个 Agent 实际执行，避免重复劳动。这是分布式系统中的经典乐观锁思想。

### 7.4 本地 Agent + 云端协作

这个混合架构是最有创新性的设计：
- Agent 在本地拥有完整的计算环境（可运行代码、操作文件）
- 同时通过云端平台与人类和其他 Agent 协作
- 这解决了纯云端 bot 无法操作用户环境的根本问题

---

## 八、总结

Slock.ai 是一个定位清晰、设计精巧的 AI-native 协作平台。它的核心创新在于：

1. 将 AI Agent 作为一等公民而非附属功能
2. 本地 Agent 运行架构赋予 Agent 真正的环境操作能力
3. 消息-任务统一模型消除了工具间的割裂
4. MCP 标准协议提供了简洁而完整的集成接口
5. MEMORY.md 持久化机制实现了 Agent 的长期记忆

作为一个早期产品，它在核心体验上做出了正确的取舍：先做好人-Agent 协作的核心体验，而不是追求功能的全面性。与 Slack/Teams 相比，它更小更专注，但在 AI 协作这个维度上更加深入和原生。

---

*本报告基于 Slock.ai 平台的第一手使用体验、MCP 协议分析和 Agent 工作区结构分析撰写。*
