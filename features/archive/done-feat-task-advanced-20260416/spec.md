# Feature: feat-task-advanced — 高级 Task 协作（子任务 + 依赖 + A2A 传递）

## Basic Information

- **ID**: feat-task-advanced
- **Name**: 高级 Task 协作（子任务 + 依赖 + A2A 传递）
- **Priority**: 85
- **Size**: M
- **Dependencies**: feat-task-conversation-bind (completed)
- **Parent**: feat-agent-task-system
- **Created**: 2026-04-14

## Description

实现高级 Task 协作能力：父子任务拆分、任务依赖关系（DAG 循环检测）、父子级联状态规则、Agent 间 A2A 任务传递、依赖满足自动解锁、TaskHistory 时间线展示。

## Scope

### In Scope
- Rust: would_create_cycle DAG 循环依赖检测
- Rust: 父子任务级联状态规则 (子全 done→父 in_review, 父 cancelled→子级联)
- 父子任务关系 UI (子任务列表/树)
- 任务依赖管理 UI (添加/移除依赖, 循环依赖提示)
- 依赖满足自动解锁逻辑
- Agent A2A Task 创建 (复用 Sub4 的 <task-suggestions> 协议)
- TaskHistory 时间线展示

### Out of Scope
- 依赖图可视化 (Phase 2 可选)

## Acceptance Criteria (Gherkin)

### US6: 子任务和依赖
```gherkin
Given Task "重构登录模块" 存在
When 用户创建子 Task "编写单元测试"
And 设置依赖 "重构登录模块" depends_on "编写单元测试"
Then "重构登录模块" 状态变为 blocked
When "编写单元测试" 完成
Then 系统检查依赖已满足
And "重构登录模块" 状态恢复为 todo
And 推送 task://dependency-met 事件
```

### US6b: 循环依赖拒绝
```gherkin
Given Task A depends_on Task B
When 用户尝试设置 Task B depends_on Task A
Then 系统拒绝此操作并提示 "会产生循环依赖"
And 依赖关系不变
```

### US6c: 父子任务级联
```gherkin
Given Task "重构" 有 2 个子任务
When 两个子任务均变为 done
Then "重构" 自动变为 in_review
When 用户取消 "重构"
Then 两个子任务级联变为 cancelled
```

### US7: A2A 任务传递
```gherkin
Given Agent Claude 在 Channel 中执行 Task
When Claude 在响应中输出 <task-suggestions> 包含分配给 Codex 的任务
And 用户确认创建
Then 新 Task 创建，source=agent_created，creator_id=claude
And Codex 收到 task://assigned 通知
And 新 Task 绑定到当前 Channel
When Codex 完成 Task
Then Claude 收到 task://completed 通知
And Claude 可以继续其原始 Task
```

---

## Merge Record

- **Completed**: 2026-04-16
- **Merged Branch**: feature/feat-task-advanced
- **Merge Commit**: (see git log)
- **Archive Tag**: feat-task-advanced-20260416
- **Conflicts**: None
- **Verification**: All 4 Gherkin scenarios passed (US6, US6b, US6c, US7)
- **Stats**: 9 files changed, 640 insertions, 30 deletions
