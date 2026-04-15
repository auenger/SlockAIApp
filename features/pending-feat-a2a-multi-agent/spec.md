# Feature: feat-a2a-multi-agent 多 Agent A2A 协作（Push Notification + 任务委托 + Artifact 共享）

## Basic Information
- **ID**: feat-a2a-multi-agent
- **Name**: 多 Agent A2A 协作（Push Notification + 任务委托 + Artifact 共享）
- **Priority**: 70
- **Size**: L
- **Dependencies**: [feat-a2a-adapter, feat-a2a-remote-client]
- **Parent**: feat-a2a-runtime
- **Children**: []
- **Created**: 2026-04-14

## Description

实现基于 A2A 协议的多 Agent 高级协作能力：Agent 间任务委托、异步状态推送通知、跨 Agent Artifact 共享。这是 A2A Runtime 重构的最终阶段，将 AgentsZone 从"多 Agent 对话工具"升级为"真正的多 Agent 协作平台"。

**本 Feature 建立在 Connection-Centric 模型之上**：
- 委托目标可以是本地 Agent（connection_mode=Local，通过 Unix Socket A2A）
- 委托目标也可以是远程 Agent（connection_mode=Remote，通过 HTTPS A2A）
- Push Notification 支持远程 Agent 完成任务后回调本机
- Artifact 共享跨本地/远程边界

**核心目标**：
- Push Notification 支持：远程 Agent 完成任务后主动回调通知本地
- Agent-to-Agent Task 委托：Agent A 可以将子任务委派给 Agent B 执行（无论 B 在本地还是远端）
- 跨 Agent Artifact 共享：一个 Agent 的产出物可被其他 Agent 引用和消费
- Channel 级别的协作编排可视化

**不做的事**：
- 不做分布式任务调度框架
- 不做复杂的权限/RBAC 系统
- 不改变单 Agent 对话的核心体验

## User Value Points

### V1: Push Notification 异步回调
用户价值：发起远程任务后无需轮询，任务完成时自动收到通知。
- 本地启动 webhook listener（Tauri 内嵌 HTTP server）
- 远程 Agent 配置 push notification URL 指向本机
- 收到 push event 后通过 Tauri event 更新前端 UI
- 支持任务完成、失败、需要输入等多种事件类型
- **Connection-Centric**: Push URL 可配置为 localhost（本机回环）或内网地址

### V2: Agent 间任务委托
用户价值：在对话中自然地让 Agent A 把工作交给 Agent B 执行。
- @mention 触发升级为 A2A SendMessage 到目标 Agent
- 委托消息包含上下文摘要（自动从当前对话提取）
- 子任务状态可追踪（父 Task → 子 Task 关联）
- 委托结果自动回传到原始对话
- **Connection-Centric**: 目标 Agent B 可以是 local 或 remote，委托逻辑不感知差异

### V3: 跨 Agent Artifact 共享
用户价值：Agent 生成的代码/文件可以被其他 Agent 直接使用。
- Artifact 通过 A2A 协议传递引用
- 本地文件系统作为 Artifact 存储后端
- Agent 可查询和获取其他 Agent 的产出物
- 引用链可追踪（谁生成了什么，谁消费了什么）
- **Connection-Centric**: 远程 Agent 的 Artifact 通过 A2A API 获取

### V4: 协作可视化
用户价值：在 UI 中看到多 Agent 协作的完整过程。
- Channel 中展示任务委托关系（视觉连线或卡片）
- 各 Agent 的执行状态实时更新
- 时间线视图展示协作过程

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/a2a/types.rs` — A2A 类型 + 连接模型（P1）
- `src-tauri/src/runtime/a2a/server.rs` — A2A Server（需扩展 Push Notification handler）
- `src-tauri/src/runtime/a2a/client.rs` — A2A Client（用于发送委托请求）
- `src-tauri/src/runtime/a2a/remote.rs` — RemoteConnectionManager（P3，用于远程委托）
- `src-tauri/src/workspace/mention.rs` — @mention 解析逻辑
- `src-tauri/src/workspace/channel.rs` — Channel 数据管理
- `src-tauri/src/commands/channel.rs` — Channel IPC commands
- 前端 Channel 组件 — 需扩展协作可视化

### Related Features
- **feat-a2a-adapter** ⬅️ 依赖 — 需要 A2A Server 能力（Push handler 注册）
- **feat-a2a-remote-client** ⬅️ 依赖 — 需要远程连接能力（远程委托 + 远程 Push）
- **feat-agent-a2a-trigger** ✅ 已完成 — @{agent} 触发机制基础（将升级为 A2A 委托）
- **feat-task-data-model** ✅ 已完成 — Task 数据模型（父子关联扩展）
- **feat-task-execution** 🔄 pending — Task 执行引擎（可并行开发）

## Technical Solution

<!-- 待实现时填充 -->

### 架构概要

```
新增/扩展:
  src-tauri/src/runtime/a2a/
    push.rs                # Push Notification receiver (webhook handler)
    delegation.rs          # Task delegation logic (connection-mode-agnostic)
    artifact_store.rs      # Cross-agent artifact management

  src-tauri/src/commands/
    collaboration.rs       # Collaboration IPC commands

  src/components/channel/
    CollaborationView.tsx   # 协作关系可视化
    AgentTaskCard.tsx      # 单个 Agent 的任务状态卡片
```

### 协作流程（Connection-Centric）

```
User: "@agent-b 帮我写单元测试"
        │
        ▼
Agent A context engine extracts summary
        │
        ▼
delegation::create(request)
        │
        ├── 查询 Agent B 的 connection_mode ──┐
        │                                      │
        ├── Local → A2A Client (unix sock)     │
        └── Remote → A2A Client (HTTPS) ───────┤
               │                               │
               ▼                               │
    SendMessage to Agent B's endpoint           │
               │                               │
               ▼                               │
    [Sync] StreamMessage SSE ← real-time       │
    [Async] Push Notification ← callback ──────┘
               │
               ▼
    Result artifact shared back to Agent A's context
```

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望多个 Agent 能通过标准化协议互相委托任务并共享成果，
以便在复杂项目中实现真正的 AI 团队协作——无论这些 Agent 运行在本机还是远端。

### Scenarios

#### Scenario 1: Push Notification 接收（本地 Webhook Listener）
```gherkin
Given 本地运行着 Push Notification webhook listener (localhost:9470/push)
When 远程 Agent (或本地 Agent) 完成任务并发送 POST /push {taskId, status, result}
Then 系统解析 push event (验证 HMAC 签名)
And 发出 Tauri event "a2a://task-updated"
And 前端监听该 event 并更新对应 Task 的 UI 状态
And 用户看到通知提示 "Agent X 完成了任务 Y"
```

#### Scenario 2: Agent A 委派任务给 Agent B（本地→本地）
```gherkin
Given 用户在 Agent A (local) 的对话中输入 "@agent-b 帮我写单元测试"
When 系统识别 @mention 并触发委托
Then 自动生成包含当前上下文摘要的委托消息
And Agent B 的 connection_mode = Local
And 通过 A2A Unix Socket 发送 SendMessage 到 Agent B
And Agent B 的流式响应通过 SSE 回传到原 Channel 展示
```

#### Scenario 2.5: Agent A 委派任务给远程 Agent C（本地→远程）⭐ Connection-Centric
```gherkin
Given 用户在 Agent A (local) 的对话中输入 "@remote-reviewer 帮我 code review"
When 系统识别 @mention 并触发委托
Then 自动生成包含当前上下文摘要的委托消息
And Agent C (remote-reviewer) 的 connection_mode = Remote { connection_id: "conn-1" }
And 通过 A2A HTTPS (RemoteA2ARuntime) 发送 SendMessage 到远端
And 远端 Agent 的流式响应通过 SSE → bridge → StreamEvent → 前端渲染
And 用户体验与 Scenario 2 (本地委托) 完全一致
```

#### Scenario 3: Artifact 跨 Agent 引用
```gherkin
Given Agent A 生成了一个 Artifact (file: src/utils/helper.rs)
When Agent B 在其上下文中引用该 Artifact
Then Agent B 可以获取 Artifact 内容：
  - 若 A 是 local: 直接读本地文件系统
  - 若 A 是 remote: 通过 A2A GET /artifacts/{id} API 获取
And 引用记录被保存（producer: AgentA, consumer: AgentB）
And 在 UI 中可查看 Artifact 的来源和消费者列表
```

#### Scenario 4: 协作时间线可视化
```gherkin
Given 一个涉及多 Agent 的协作任务（含 local 和 remote Agent）
When 用户查看 Channel 的协作视图
Then 显示各 Agent 的参与时间线
And 任务之间的依赖关系清晰可见（箭头/连线）
And 每个 Agent 的连接模式有标识（local=电脑图标, remote=云图标）
And 各 Agent 的状态实时更新（进行中/已完成/失败）
```

#### Scenario 5: 委托失败处理
```gherkin
Given Agent A 委派任务给 Agent B
When Agent B 不可达（网络断开 or 未启动 or 认证失败）
Then 委托标记为 FAILED 并返回错误原因
And 提供"重试"和"自行处理"两个选项
If 用户选择"自行处理"则由 Agent A 自己完成任务
```

### General Checklist
- [ ] Push Notification webhook listener 实现
- [ ] PushNotificationConfig CRUD（注册/注销/列表）
- [ ] Task 委托完整流程（@mention → 提取上下文 → 发送 → 接收结果）
- [ ] 委托支持 local 和 remote 目标 Agent（Connection-Centric）
- [ ] Artifact 跨 Agent 存储和检索（local fs + remote A2A API 双路径）
- [ ] 协作可视化 UI 组件
- [ ] 委托异常处理（超时、失败、取消）
- [ ] 与现有 Task 系统（feat-task-execution）的集成
- [ ] cargo build + npm run build 全部通过
