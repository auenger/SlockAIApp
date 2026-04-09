# Tasks: feat-agent-workspace-design

## Task Breakdown

### 1. Workspace 目录结构设计
- [x] 定义 SlockAI 的 workspaces/ 目录结构
- [x] 确定全局模板 vs Agent 级别模板的优先级规则
- [x] 设计 conversations/, context/, output/ 的文件命名规范

### 2. 模板文件创建
- [x] 编写 SOUL.md 模板（适配 SlockAI 的 Agent 人格定义）
- [x] 编写 IDENTITY.md 模板（Agent 元信息：名称、类型、风格、Emoji、Avatar）
- [x] 编写 USER.md 模板（用户偏好档案）
- [x] 编写 AGENTS.md 模板（Agent 行为指令）
- [x] 编写 TOOLS.md 模板（工具使用说明，适配 SlockAI 的上下文编排引擎）

### 3. Rust Workspace 管理模块
- [x] 实现 AgentWorkspace 结构体和基本操作
- [x] 实现 AgentIdentity 解析（从 IDENTITY.md 读取）
- [x] 实现 AgentManager（创建、切换、列举 Agent）
- [x] 实现模板初始化逻辑（首次创建时生成模板文件）
- [x] 实现模板同步逻辑（增量同步，不覆盖已有文件）

### 4. 上下文编排集成
- [x] 上下文编排引擎加载 Agent 的 SOUL.md 作为 system prompt 前缀
- [x] 加载 IDENTITY.md 信息构建 Agent 上下文
- [x] 实现 Agent 级别的 SOUL.md 优先于全局 SOUL.md 的覆盖机制

### 5. Tauri IPC 接口
- [x] 添加 workspace 相关的 Tauri Commands
- [x] 添加 agent 管理的 Tauri Commands（create/switch/list）
- [x] 前端 Agent 选择器展示 IDENTITY.md 中的 Emoji/Name

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-08 | 创建 feature | 完成 anyclaw 参考项目分析，确定设计方案 |
| 2026-04-08 | 完成实现 | 全部 5 个 task 完成，含 Rust 后端 + TypeScript 前端 |
