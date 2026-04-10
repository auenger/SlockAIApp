# Feature: feat-skills-management Skills 管理 UI

## Basic Information
- **ID**: feat-skills-management
- **Name**: Skills 管理 UI
- **Priority**: 40
- **Size**: M
- **Dependencies**: None
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10

## Description
实现 Agent Skills 的管理界面，允许用户查看、添加、编辑和删除 Agent 的 Skills 配置。Skills 是 Agent 能力的扩展单元，包括工具调用、MCP 服务器连接、自定义命令等。本 feature 需要设计 Skills 的数据模型、存储方案，以及前后端交互的完整链路。

## User Value Points

### VP1: Skills 列表与浏览
用户可以在 Agent 详情或设置页面中查看该 Agent 已配置的 Skills 列表，了解每个 Skill 的名称、类型、状态。

### VP2: Skills 配置管理
用户可以添加新的 Skill（如 MCP Server URL、自定义工具配置），编辑已有 Skill 的参数，或移除不再需要的 Skill。

### VP3: Skill 状态反馈
用户可以直观看到每个 Skill 的运行状态（连接中/活跃/错误），便于排查配置问题。

## Context Analysis

### 前端相关文件
| 文件 | 说明 |
|------|------|
| `src/components/Sidebar.tsx` | Agent 列表，可能需要 Skills 入口 |
| `src/components/MainContent.tsx` | 主内容区，承载 Skills 管理视图 |
| `src/lib/ipc.ts` | IPC 命令层 |
| `src/types.ts` | 类型定义 |

### 后端相关文件
| 文件 | 说明 |
|------|------|
| `src-tauri/src/commands/mod.rs` | Tauri 命令注册 |
| `src-tauri/src/storage/` | 存储层 |
| `src-tauri/src/workspace/` | Workspace 相关，Agent 配置存储 |

### Related Documents
- project-context.md

### Related Features
- 无

## Technical Solution
待实现时细化。初步方向：
1. 定义 Skill 数据模型（id, name, type, config, status, agent_id）
2. Rust 后端实现 Skill CRUD commands
3. 前端新增 Skills 管理组件，集成到 Agent Profile 页面或独立 tab
4. Skill 状态通过运行时轮询或事件机制反馈

## Acceptance Criteria (Gherkin)

### User Story
作为一个用户，我希望在界面中管理 Agent 的 Skills 配置，以便灵活扩展 Agent 的能力。

### Scenarios (Given/When/Then)

```gherkin
Scenario: 查看 Agent 的 Skills 列表
  Given 用户选择了某个 Agent
  When 进入 Skills 管理页面
  Then 应显示该 Agent 所有已配置的 Skills

Scenario: 添加新 Skill
  Given 用户在 Skills 管理页面
  When 点击添加按钮并填写 Skill 配置
  Then 新 Skill 应被保存并出现在列表中

Scenario: 删除 Skill
  Given Skills 列表中有至少一个 Skill
  When 用户点击删除并确认
  Then 该 Skill 应从列表中移除
```

### General Checklist
- [ ] Skill 数据模型定义完成
- [ ] 后端 CRUD commands 实现完成
- [ ] 前端 Skills 列表展示
- [ ] 前端 Skill 添加/编辑/删除功能
- [ ] Skill 状态反馈
