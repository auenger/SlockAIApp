# Tasks: feat-sidebar-style

## Task Breakdown

### 1. Sidebar 标题更名与样式优化
- [x] 将 Sidebar.tsx 中 "Development" 改为 "AgentsZone"
- [x] 优化标题区域样式（字体、间距、图标、视觉层次）

### 2. 可调宽度拖拽组件
- [x] 实现通用 ResizablePanel / ResizeHandle 组件或逻辑
- [x] 支持最小/最大宽度约束
- [x] 鼠标悬停视觉反馈（col-resize 光标 + 高亮线）

### 3. Sidebar 可调宽度
- [x] 替换 Sidebar 固定 `w-64` 为动态宽度
- [x] 集成拖拽手柄到 Sidebar 右边缘
- [x] 宽度范围：180px ~ 400px

### 4. ThreadPanel 可调宽度
- [x] 替换 ThreadPanel 固定 `w-80` 为动态宽度
- [x] 集成拖拽手柄到 ThreadPanel 左边缘
- [x] 宽度范围：240px ~ 560px

### 5. App.tsx 布局集成
- [x] 在 App.tsx 三栏布局中接入动态宽度状态
- [x] 确保 MainContent 自适应剩余空间
- [x] 处理边界情况（面板收起/展开时的宽度）

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开发 |
| 2026-04-11 | Implementation complete | All 5 tasks done: useResizable hook, title rename, dynamic widths for Sidebar & ThreadPanel, App.tsx integration |
