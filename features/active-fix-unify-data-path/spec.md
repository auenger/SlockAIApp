# Feature: fix-unify-data-path 统一应用数据目录

## Basic Information
- **ID**: fix-unify-data-path
- **Name**: 统一应用数据目录至 ~/.agentszone
- **Priority**: 95
- **Size**: S
- **Dependencies**: 无
- **Parent**: null
- **Children**: 无
- **Created**: 2026-04-11T22:00:00+08:00

## Description

当前应用数据存储在 Tauri 管理的系统目录 `~/Library/Application Support/com.agentszone.app/workspaces/`，需要统一改为 `~/.agentszone/`。

这样做的好处：
- 路径简洁，用户可以直接在终端访问 `~/.agentszone`
- 跨平台一致（不依赖各平台 app data dir 的差异）
- 方便调试和手动操作

## User Value Points

1. **统一数据路径** — 所有应用数据（agents、channels、threads、db 等）统一存储在 `~/.agentszone/`，用户和开发者都可以方便地访问和管理

## Context Analysis

### Reference Code
- `src-tauri/src/lib.rs:14-24` — `DEFAULT_WORKSPACE_DIR` 常量和 `resolve_workspace_root()` 函数，这是唯一需要修改的核心路径
- 所有子模块（manager、agent、channel、thread、storage、context）都通过 workspace root 参数获取路径，无需修改
- `src-tauri/tauri.conf.json:5` — 应用标识 `com.agentszone.app`

### Related Documents
- CLAUDE.md 中存储模式章节

### Related Features
- 无前置依赖

## Technical Solution

修改 `src-tauri/src/lib.rs` 中的 `resolve_workspace_root()` 函数：
- 不再使用 `app.path().app_data_dir()`
- 改为使用 `$HOME/.agentszone/` 作为 workspace root
- 移除 `DEFAULT_WORKSPACE_DIR` 常量（不再需要 `workspaces` 子目录层级）
- 使用 `dirs::home_dir()` 获取用户 home 目录（Rust `dirs` crate 或 `std::env`）

修改后的路径结构：
```
~/.agentszone/
├── agentszone.db          (SQLite)
├── activity.jsonl         (Activity 日志)
├── USER.md / SOUL.md / AGENTS.md / TOOLS.md
├── memory/
│   ├── MEMORY.md
│   └── HISTORY.md
├── agents/{agent_id}/
│   ├── IDENTITY.md
│   ├── conversations/
│   ├── context/
│   ├── output/
│   ├── skills/
│   └── config/
└── channels/
    └── channel_{id}.json
```

### 数据迁移考虑
- 开发阶段，无需处理旧路径数据迁移
- 后续可在 setup 中检测旧路径存在时自动迁移

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望所有应用数据存储在 `~/.agentszone/` 目录下，这样我可以方便地在终端中查看和管理。

### Scenarios (Given/When/Then)

```gherkin
Scenario: 新安装应用数据目录创建
  Given 应用首次启动
  When 应用初始化 workspace
  Then 应在 $HOME/.agentszone/ 下创建完整目录结构
  And 目录结构包含 agents/、channels/ 子目录
  And 数据库文件位于 $HOME/.agentszone/agentszone.db

Scenario: Agent 创建使用新路径
  Given 应用已启动
  And workspace 已初始化在 ~/.agentszone/
  When 用户创建一个名为 "gaby" 的 Agent
  Then Agent 工作目录应为 ~/.agentszone/agents/gaby/
  And Agent 目录下包含 IDENTITY.md 等标准文件

Scenario: 已有数据路径正常运行
  Given ~/.agentszone/ 目录已存在且有历史数据
  When 应用启动
  Then 应正常加载已有的 agents 和 channels
  And 不应重新创建已存在的目录
```

### General Checklist
- [ ] `resolve_workspace_root` 使用 `~/.agentszone/`
- [ ] 日志输出正确显示新路径
- [ ] 所有子模块通过 workspace root 参数正常工作
- [ ] 前端 mock 路径与实际路径一致
