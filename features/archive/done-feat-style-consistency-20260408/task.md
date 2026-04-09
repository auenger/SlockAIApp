# Tasks: feat-style-consistency

## Task Breakdown

### 1. 样式系统移植
- [x] 移植 `index.css` 中的 `@theme` 色彩/字体变量到项目
- [x] 移植 `@utility` 自定义工具类（brutal-border 系列、brutal-shadow 系列、brutal-btn、brutal-card）
- [x] 移植 `@layer base` 基础样式
- [x] 安装字体依赖（JetBrains Mono, Inter）或配置 CDN 加载
- [x] 安装样式工具依赖（clsx, tailwind-merge）

### 2. 类型和工具函数移植
- [x] 移植 `types.ts`（TabType, Agent, Channel, Thread, Task, Message 类型定义）
- [x] 移植 `lib/utils.ts`（cn() 工具函数）
- [x] 适配 import 路径以匹配项目目录结构

### 3. 布局与页面框架
- [x] 创建主布局组件（三栏布局：Sidebar | Main | ThreadPanel）
- [x] 实现主布局的状态管理（activeChannel, activeTab, isThreadOpen）
- [x] 适配 Tauri V2 窗口尺寸（确保 h-screen 正确工作）

### 4. Sidebar 组件移植
- [x] 移植 Sidebar 组件（Header + Nav + Channels + Threads + Agents + Humans + Footer）
- [x] 确保频道选择交互（brutal-pink 高亮 + 阴影偏移）
- [x] 确保 Agent 状态指示器显示正确

### 5. MainContent 组件移植
- [x] 移植主内容区顶部 Header（Agent 信息 + 操作按钮）
- [x] 移植标签页导航栏（6个标签页切换）
- [x] 移植 CHAT 标签页（消息列表 + 输入框 + 发送）
- [x] 移植 TASKS 标签页（筛选器 + 任务列表 + 新建按钮）
- [x] 移植 WORKSPACE 标签页（文件树 + 文件查看器）
- [x] 移植 SKILLS 标签页（技能卡片网格）
- [x] 移植 ACTIVITY 标签页（日志列表）
- [x] 移植 PROFILE 标签页（Agent 详情）

### 6. ThreadPanel 组件移植
- [x] 移植 ThreadPanel 组件（Header + Content + Input）
- [x] 实现展开/收起动画或状态切换

### 7. 模态框组件移植
- [x] 移植 Modal 基础组件
- [x] 移植 CreateTaskModal
- [x] 移植 InviteHumanModal

### 8. AI 服务适配
- [x] 评估 Gemini 服务是否保留或替换为 mock
- [x] 创建 mock 服务返回预设回复（内联在 MainContent 中）
- [x] 确保 CHAT 标签页的消息收发流程正常工作

### 9. 集成验证
- [x] 确保应用在 Tauri V2 中正确启动
- [x] 验证三栏布局在不同窗口尺寸下的表现
- [x] 验证所有标签页内容正确渲染
- [x] 验证模态框打开/关闭正常
- [x] 修复移植过程中的任何 TypeScript 编译错误

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-07 | Feature created | 等待 feat-project-init 完成后开始实施 |
| 2026-04-07 | Feature modified | 策略调整：从"提取设计系统"改为"直接移植原型为MVP" |
| 2026-04-08 | Implementation complete | 所有9个任务完成。TypeScript编译通过，Vite构建成功。使用mock AI服务替代Gemini。 |
