# Feature: feat-sidebar-style Sidebar 标题更名与面板可调宽度

## Basic Information
- **ID**: feat-sidebar-style
- **Name**: Sidebar 标题更名与面板可调宽度
- **Priority**: 70
- **Size**: M
- **Dependencies**: none
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10T23:00:00+08:00

## Description

1. **Sidebar 标题更名（高优先级）**：将最左侧 Sidebar 顶部的 "Development" 文字改为 "AgentsZone"，同时优化该标题区域的样式，使其更符合产品定位。
2. **面板可调宽度（低优先级）**：让 Sidebar（左侧栏）和 ThreadPanel（右侧面板）支持用户拖拽调整宽度，提升布局灵活性。

## User Value Points

### VP1: Sidebar 标题更名 + 样式优化
- 用户一眼能看到产品品牌名 "AgentsZone"
- 标题区域样式与新粗野主义风格统一、更精致

### VP2: 面板可调宽度
- 用户可按需调整 Sidebar 和 ThreadPanel 宽度，适配不同使用场景
- 拖拽手柄视觉清晰，操作流畅

## Context Analysis

### Reference Code
- `src/components/Sidebar.tsx:96` — 当前标题 "Development" 所在位置
- `src/App.tsx:120-161` — 三栏布局（Sidebar | MainContent | ThreadPanel）
- `src/components/ThreadPanel.tsx` — 右侧面板组件
- Sidebar 当前固定 `w-64`（256px），ThreadPanel 固定 `w-80`（320px）

### Related Documents
- `project-context.md` — 项目产品定位、UI 设计风格

### Related Features
- feat-style-consistency (已完成) — 原型 MVP 移植

## Technical Solution
<!-- To be filled during implementation -->

## Acceptance Criteria (Gherkin)

### User Story
作为 AgentsZone 用户，我希望看到正确的品牌名称，并能按需调整面板宽度，以获得更好的使用体验。

### Scenarios (Given/When/Then)

#### Scenario 1: Sidebar 标题显示 AgentsZone
```gherkin
Given 应用已启动
When 用户查看左侧 Sidebar 顶部
Then 标题显示为 "AgentsZone"
And 标题样式符合新粗野主义设计风格
```

#### Scenario 2: 调整 Sidebar 宽度
```gherkin
Given 应用已启动且三栏布局可见
When 用户拖拽 Sidebar 右边缘
Then Sidebar 宽度随拖拽实时变化
And 宽度限制在最小 180px 和最大 400px 之间
And MainContent 自动填充剩余空间
```

#### Scenario 3: 调整 ThreadPanel 宽度
```gherkin
Given ThreadPanel 已打开
When 用户拖拽 ThreadPanel 左边缘
Then ThreadPanel 宽度随拖拽实时变化
And 宽度限制在最小 240px 和最大 560px 之间
And MainContent 自动调整宽度
```

#### Scenario 4: 拖拽手柄视觉提示
```gherkin
Given 三栏布局可见
When 用户将鼠标悬停在面板边缘的拖拽区域
Then 显示视觉反馈（如高亮线或光标变化）
And 鼠标光标变为 col-resize
```

### UI/Interaction Checkpoints
- [ ] Sidebar 顶部标题从 "Development" 变为 "AgentsZone"
- [ ] 标题区域样式优化（字体、间距、图标等）
- [ ] Sidebar 右侧出现拖拽手柄
- [ ] ThreadPanel 左侧出现拖拽手柄
- [ ] 拖拽时光标变为 col-resize
- [ ] 拖拽中面板宽度实时响应

### General Checklist
- [ ] 不影响现有功能逻辑
- [ ] 宽度调整后 MainContent 自适应
- [ ] 拖拽操作流畅，无卡顿
