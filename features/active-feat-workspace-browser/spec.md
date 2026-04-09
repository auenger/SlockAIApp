# Feature: feat-workspace-browser Workspace 文件浏览器

## Basic Information
- **ID**: feat-workspace-browser
- **Name**: Workspace 文件浏览器
- **Priority**: 45
- **Size**: M
- **Dependencies**: feat-agent-workspace-design (completed)
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-09

## Description

当前 MainContent.tsx 的 WORKSPACE tab 是硬编码的静态内容（路径写死为 `~/.slock/agents/8ce6fe04-...`，文件夹列表固定为 kagent/kubectl-mcp-server/notes）。需要实现真实的 Workspace 文件浏览器：读取 Agent 的 workspace 目录结构，展示文件/文件夹树，支持文件内容查看。

## User Value Points

1. **文件目录浏览** — 用户可查看 Agent Workspace 的真实目录结构
2. **文件内容查看** — 用户可点击文件查看其内容（Markdown、配置文件等）

## Context Analysis

### Reference Code
- `src/components/MainContent.tsx:510-577` — WORKSPACE tab（当前硬编码）
- `src-tauri/src/workspace/agent.rs` — AgentWorkspace 提供目录路径方法
- `src/lib/ipc.ts` — 需新增 workspace 浏览 IPC

### Related Features
- feat-agent-workspace-design (completed) — Agent Workspace 结构

## Technical Solution

1. 后端新增 `list_workspace_dir(agent_id, subpath?)` 命令，返回目录条目列表
2. 后端新增 `read_workspace_file(agent_id, file_path)` 命令，返回文件内容
3. 前端 WORKSPACE tab 根据选中 Agent 加载真实目录
4. 左侧文件树展示目录结构，右侧展示选中文件内容
5. 顶部显示真实 workspace 路径

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我想浏览和查看 Agent Workspace 中的文件内容。

### Scenarios (Given/When/Then)

#### Scenario 1: 展示真实目录
```gherkin
Given 用户选中了一个 Agent
When 切换到 WORKSPACE tab
Then 显示该 Agent workspace 的真实目录结构
And 顶部显示真实路径
```

#### Scenario 2: 查看文件内容
```gherkin
Given 目录树已加载
When 用户点击一个文件（如 MEMORY.md）
Then 右侧显示该文件的真实内容
And 显示文件名和大小
```

#### Scenario 3: 无 Agent 选中
```gherkin
Given 用户未选中 Agent
When 切换到 WORKSPACE tab
Then 显示 "Select an agent to view workspace" 提示
```

### General Checklist
- [x] 移除硬编码路径和文件列表
- [x] 后端文件读取安全校验（防止路径穿越）
- [x] 前端真实数据渲染
