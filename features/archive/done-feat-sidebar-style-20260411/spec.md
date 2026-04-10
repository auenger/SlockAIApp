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

### Approach
1. **useResizable hook** — Reusable React hook (`src/lib/useResizable.ts`) that manages panel width state, mousedown/mousemove/mouseup listeners, min/max clamping, and cursor feedback. Supports `edge: 'right'` (Sidebar) and `edge: 'left'` (ThreadPanel) orientations.
2. **Sidebar title rename** — Changed "Development" to "AgentsZone" with `tracking-tight` for better kerning.
3. **Dynamic width** — Removed hardcoded `w-64` / `w-80` Tailwind classes; instead each panel receives `style` and `resizeHandleRef` props from App.tsx.
4. **Layout integration** — App.tsx creates two `useResizable` instances (256px init for Sidebar, 320px init for ThreadPanel). The flex container with MainContent in the middle auto-fills remaining space.
5. **Resize handle** — A 4px-wide absolute-positioned strip on the panel edge with `cursor-col-resize` and hover highlight via `hover:bg-black/20`.

### Files Changed
- `src/lib/useResizable.ts` (new) — Reusable resizable panel hook
- `src/components/Sidebar.tsx` (modified) — Title rename, dynamic width props, resize handle
- `src/components/ThreadPanel.tsx` (modified) — Dynamic width props, resize handle
- `src/App.tsx` (modified) — useResizable hook instances, pass props to child panels

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
- [x] Sidebar 顶部标题从 "Development" 变为 "AgentsZone"
- [x] 标题区域样式优化（字体、间距、图标等）
- [x] Sidebar 右侧出现拖拽手柄
- [x] ThreadPanel 左侧出现拖拽手柄
- [x] 拖拽时光标变为 col-resize
- [x] 拖拽中面板宽度实时响应

### General Checklist
- [x] 不影响现有功能逻辑
- [x] 宽度调整后 MainContent 自适应
- [x] 拖拽操作流畅，无卡顿

## Merge Record

- **Completed**: 2026-04-11T15:00:00+08:00
- **Merged Branch**: feature/feat-sidebar-style
- **Merge Commit**: dc63dd6
- **Archive Tag**: feat-sidebar-style-20260411
- **Conflicts**: none
- **Verification**: passed (4/4 Gherkin scenarios)
- **Stats**:
  - Started: 2026-04-11T14:00:00+08:00
  - Duration: ~1 hour
  - Commits: 1 (implementation)
  - Files Changed: 4
