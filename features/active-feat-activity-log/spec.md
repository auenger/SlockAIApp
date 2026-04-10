# Feature: feat-activity-log Activity 日志后端集成

## Basic Information
- **ID**: feat-activity-log
- **Name**: Activity 日志后端集成
- **Priority**: 35
- **Size**: S
- **Dependencies**: None
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10

## Description
实现 Activity 日志的后端存储和前端展示集成。记录 Agent 的关键活动（创建、删除、对话开始、Skill 变更等），以时间线形式展示，帮助用户了解 Agent 的活动历史。

## User Value Points

### VP1: 活动时间线
用户可以在界面中查看 Agent 或全局的活动时间线，了解系统发生了什么。

### VP2: 活动详情
用户可以点击活动条目查看详细信息，包括时间、类型、涉及的对象等。

## Context Analysis

### 前端相关文件
| 文件 | 说明 |
|------|------|
| `src/components/Sidebar.tsx` | 可能需要 Activity 入口 |
| `src/components/MainContent.tsx` | 承载 Activity 视图 |
| `src/lib/ipc.ts` | IPC 命令层 |
| `src/types.ts` | 类型定义 |

### 后端相关文件
| 文件 | 说明 |
|------|------|
| `src-tauri/src/commands/mod.rs` | Tauri 命令注册 |
| `src-tauri/src/storage/` | 存储层 |

### Related Documents
- project-context.md

### Related Features
- 无

## Technical Solution
待实现时细化。初步方向：
1. 定义 Activity Log 数据模型（id, timestamp, type, agent_id, details）
2. Rust 后端在关键操作点插入日志记录
3. 提供 `list_activities` command 支持分页和过滤
4. 前端新增 Activity 时间线组件

## Acceptance Criteria (Gherkin)

### User Story
作为一个用户，我希望查看 Agent 的活动历史记录，以便了解系统的运行情况和 Agent 的行为。

### Scenarios (Given/When/Then)

```gherkin
Scenario: 查看活动日志
  Given 系统中存在活动记录
  When 用户打开 Activity 页面
  Then 应按时间倒序显示活动列表

Scenario: 按 Agent 过滤活动
  Given 存在多个 Agent 的活动记录
  When 用户选择某个 Agent 进行过滤
  Then 应只显示该 Agent 的活动记录
```

### General Checklist
- [ ] Activity Log 数据模型定义完成
- [ ] 后端日志记录和查询实现完成
- [ ] 前端 Activity 时间线展示
- [ ] 关键操作点已插入日志记录
