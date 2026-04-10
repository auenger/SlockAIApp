# Tasks: feat-svg-icon-system

## Task Breakdown

### 1. Icon 库集成
- [x] 调研并选择 SVG icon 方案（扩展现有 lucide-react 或引入新 icon 库）
- [x] 定义 icon 分类和命名规范
- [x] 创建 icon 常量/类型定义（icon name 映射表）

### 2. Agent 数据模型扩展
- [x] 在 Agent 类型中新增 `icon` 字段（SVG icon name）
- [x] 更新 CreateAgentRequest 支持 icon 参数
- [x] 确保向后兼容 emoji 字段

### 3. Icon Picker 组件
- [x] 实现 IconPicker 组件（Popover + Grid）
- [x] 实现分类导航
- [x] 实现搜索过滤功能
- [x] 实现选中预览
- [x] 集成颜色选择（可选）

### 4. 图标渲染统一
- [x] 创建 AgentIcon 渲染组件（兼容 emoji 和 SVG icon）
- [x] 更新 Sidebar 中的 Agent 图标渲染
- [x] 更新 Channel/Thread 中的 Agent 图标渲染
- [x] 更新 Agent Profile 页面的图标渲染

### 5. Create Agent 集成
- [x] 在 CreateAgentModal 中替换 emoji 选择为 IconPicker
- [x] 保存 icon name 到 agent identity

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开发 |
| 2026-04-10 | Implementation complete | All 5 tasks done: iconRegistry, AgentIcon, IconPicker, type extensions, component integration |
