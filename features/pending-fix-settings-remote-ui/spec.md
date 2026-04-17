# Feature: fix-settings-remote-ui 设置页面 Remote Connections UI 优化

## Basic Information
- **ID**: fix-settings-remote-ui
- **Name**: 设置页面 Remote Connections UI 优化（样式统一 + Emoji→SVG + 溢出修复）
- **Priority**: 60
- **Size**: M
- **Dependencies**: 无
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-17

## Description

设置页面中 REMOTE CONNECTIONS 部分存在三个问题：

1. **Card 样式不一致** — RemoteConnectionsPanel 和 BridgeWorkspacePanel 使用普通暗色风格（gray-800/700 bg-gray-800/50），与项目整体 brutalist 设计语言（brutal-border, brutal-shadow, brutal-card, yellow/cyan/pink 强调色）不统一。Card 背景色暗淡导致文字信息难以辨认。
2. **Emoji 需替换为 SVG** — BridgeWorkspacePanel 中使用 📁📄 emoji 表示文件/文件夹，agent 卡片使用 `{agent.emoji}` 显示。需要替换为 lucide-react SVG 图标（Folder, FileText 等）。
3. **Workspace 展开溢出** — BridgeWorkspacePanel 嵌入在 ConnectionCard 内部，点击 workspace 文件夹展开后内容超出弹出框边界，且无法折叠收缩。需要添加折叠/展开控制和最大高度限制。

## User Value Points

### VP1: 统一 Card 样式 — 视觉一致性
将 RemoteConnectionsPanel 和 BridgeWorkspacePanel 的样式对齐到 brutalist 设计系统。使用 brutal-border, brutal-shadow, brutal-btn 等已有 CSS 类，替换灰暗配色。改善 Card 中文字可读性。

### VP2: Emoji → SVG 图标 — 专业感
将 BridgeWorkspacePanel 中的 emoji 图标替换为 lucide-react SVG 图标，与项目其他组件保持一致（项目已有 iconRegistry.ts 管理图标）。

### VP3: Workspace 溢出修复 — 交互可用性
修复 BridgeWorkspacePanel 展开时超出弹出框的问题，添加折叠/展开控制，确保内容在合理高度内滚动。

## Context Analysis

### Reference Code
- `src/components/settings/RemoteConnectionsPanel.tsx` — 主面板，ConnectionCard 组件
- `src/components/settings/BridgeWorkspacePanel.tsx` — Workspace 浏览面板
- `src/lib/iconRegistry.ts` — 已有的 SVG 图标注册中心（lucide-react）
- `src/components/AgentIcon.tsx` — Agent 图标组件（brutal-border 样式）
- `src/components/SkillsPanel.tsx` — 参考示例：使用 brutal-border/brutal-shadow 的弹出面板
- `src/components/EditAgentModal.tsx` — 参考示例：brutal 风格的模态框

### Related Documents
- `src/index.css` — brutal-border, brutal-shadow, brutal-btn 等 CSS 类定义

### Related Features
- `feat-a2a-remote-client` — Remote Connections UI 原始实现
- `feat-svg-icon-system` — SVG 图标系统

## Technical Solution
<!-- To be filled during implementation -->

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望设置页面的 Remote Connections 部分与应用整体风格一致，图标清晰专业，Workspace 展开时不会溢出，以便我可以舒适地管理远程连接。

### Scenarios (Given/When/Then)

#### Scenario 1: Card 样式与整体风格一致
```gherkin
Given 用户打开了设置页面的 PROFILE 标签
When 页面显示 REMOTE CONNECTIONS 区域
Then Connection Card 使用 brutalist 设计风格（brutal-border, 黑色粗边框）
And Card 背景颜色确保文字清晰可读
And 按钮（Test/Edit/Delete）使用 brutal-btn 风格
And StatusBadge 与整体设计协调
```

#### Scenario 2: Emoji 替换为 SVG 图标
```gherkin
Given 用户已添加一个 Bridge 类型的远程连接且状态为 online
When BridgeWorkspacePanel 显示 workspace 内容
Then 文件夹使用 lucide-react Folder 图标（而非 📁 emoji）
And 文件使用 lucide-react FileText 图标（而非 📄 emoji）
And Agent 列表中的 emoji 改用 SVG 图标或 AgentIcon 组件
```

#### Scenario 3: Workspace 展开不溢出弹出框
```gherkin
Given 用户已添加一个 Bridge 类型的远程连接且状态为 online
When 用户点击 Agent 展开 workspace 文件列表
Then 文件列表在合理高度内滚动显示
And BridgeWorkspacePanel 有折叠/展开切换按钮
And 内容不会超出父容器边界
And 用户可以点击按钮收起 workspace 面板
```

#### Scenario 4: 空状态和加载状态样式一致
```gherkin
Given 用户打开 REMOTE CONNECTIONS 但无连接
When 显示空状态提示
Then 空状态提示使用 brutalist 风格
And 加载状态也使用一致样式
```

### UI/Interaction Checkpoints
- [ ] ConnectionCard 边框: brutal-border (2px solid black)
- [ ] Card 背景: bg-brutal-bg 或白色，确保文字可读
- [ ] 按钮: brutal-btn 风格
- [ ] StatusBadge: 使用 brutalist 色彩（brutal-green, brutal-cyan 等）
- [ ] 文件图标: lucide-react Folder / FileText
- [ ] Agent 图标: 使用 AgentIcon 组件或 SVG
- [ ] BridgeWorkspacePanel: 有展开/折叠切换
- [ ] 整体最大高度约束: BridgeWorkspacePanel 在 max-height 内滚动

### General Checklist
- [ ] 所有修改仅涉及 UI 层，不改变后端逻辑
- [ ] 保持所有现有功能（CRUD、测试、健康检查）正常工作
- [ ] 不引入新的依赖
