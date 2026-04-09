# Feature: feat-style-consistency 原型 MVP 移植

## Basic Information
- **ID**: feat-style-consistency
- **Name**: 原型 MVP 移植
- **Priority**: 80
- **Size**: M
- **Dependencies**: feat-project-init
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-07
- **Modified**: 2026-04-07

## Description

将 ReactDemo 目录下的原型项目直接移植为 SlockAI 的 MVP（最小可行产品），先获得一个可运行、可交互的应用原型，后续再基于运行中的 MVP 迭代补充功能和优化架构。

**策略转变**：不再采用"先提取设计系统 → 再构建组件库 → 最后组装页面"的自底向上方式，而是采用"直接移植原型 → 获得可运行 MVP → 基于实际体验迭代优化"的快速验证方式。

ReactDemo 原型已实现的完整功能：
- **三栏布局**: Sidebar (导航/频道/Agent) | Main Content (6个功能标签页) | Thread Panel (线程详情)
- **Neo-Brutalism 视觉风格**: 粗黑边框 + 偏移阴影 + 鲜明色彩
- **CHAT 标签页**: 消息列表 + 输入框 + Gemini AI 集成
- **TASKS 标签页**: 任务列表 + 状态筛选 + 任务创建模态框
- **WORKSPACE 标签页**: 文件浏览器 + 文件查看
- **SKILLS 标签页**: 技能卡片展示
- **ACTIVITY 标签页**: 活动日志
- **PROFILE 标签页**: Agent 详情展示
- **Sidebar**: 频道切换、线程列表、Agent 列表、用户信息
- **Thread Panel**: 代码块、表格、消息展示
- **模态框**: 创建任务、邀请用户

## User Value Points

### VP1: 可运行的 MVP 应用
直接将原型移植到 Tauri V2 + React 19 项目中，获得一个完整可运行、可交互的桌面应用原型。用户可以浏览所有界面、切换标签、发送消息、查看任务等。

### VP2: 视觉与交互还原
确保 MVP 的视觉效果和交互体验与 ReactDemo 原型高度一致，包括 Neo-Brutalism 风格的三栏布局、动画效果、状态切换等。

## Context Analysis

### Reference Code (直接移植源)
- `ReactDemo/slockai-prototype/src/App.tsx` - 主布局与组件编排
- `ReactDemo/slockai-prototype/src/components/Sidebar.tsx` - 侧边栏（频道/线程/Agent/用户）
- `ReactDemo/slockai-prototype/src/components/MainContent.tsx` - 主内容区（6个标签页）
- `ReactDemo/slockai-prototype/src/components/ThreadPanel.tsx` - 线程详情面板
- `ReactDemo/slockai-prototype/src/components/Modals.tsx` - 模态框组件
- `ReactDemo/slockai-prototype/src/index.css` - Tailwind CSS 配置与 Neo-Brutalism 样式
- `ReactDemo/slockai-prototype/src/types.ts` - 类型定义
- `ReactDemo/slockai-prototype/src/lib/utils.ts` - cn() 工具函数

### Related Documents
- `project-context.md` - 项目技术栈与架构文档

### Related Features
- `feat-project-init` - 项目初始化（本功能的前置依赖，提供 Tauri V2 + React 19 基础框架）

## Technical Solution

### 移植策略
1. **直接移植**: 将原型组件、样式、类型、工具函数直接复制到项目中，适配 Tauri V2 环境
2. **样式移植**: 将 `index.css` 中的 `@theme`、`@utility`、`@layer base` 配置直接移植
3. **组件适配**: 组件保持原型结构不变，仅做必要的 import 路径和 API 适配
4. **AI 服务适配**: 将 Gemini 服务替换为项目实际使用的 AI 后端接口（或暂用 mock）
5. **布局适配**: 确保三栏布局在 Tauri 窗口中正确显示

### 后续迭代方向（不在本 feature 范围内）
- 提取设计令牌系统 → 建立统一 Design Tokens
- 封装基础组件库 → Button, Card, Input 等
- 接入真实后端 → 替换硬编码数据
- 路由系统 → 多页面导航
- 状态管理 → 全局状态方案

## Acceptance Criteria (Gherkin)

### User Story
作为 SlockAI 的用户/开发者，我希望启动应用后能看到一个完整可交互的 MVP 界面，与 ReactDemo 原型视觉一致，这样我可以基于真实的运行体验来规划和迭代后续功能。

### Scenarios (Given/When/Then)

#### Scenario 1: MVP 启动与布局
```gherkin
Given feat-project-init 已完成项目初始化
When 用户启动 Tauri 应用
Then 应显示三栏布局界面（Sidebar | Main Content | Thread Panel）
And Sidebar 宽度约 256px，背景为 brutal-yellow
And Thread Panel 可展开/收起
And 整体视觉与 ReactDemo 原型一致
```

#### Scenario 2: 侧边栏功能
```gherkin
Given MVP 应用已启动
When 用户查看 Sidebar
Then 应包含以下区域：
  | 区域     | 内容                                     |
  | Header  | 工作空间名称 "Development" + 下拉箭头      |
  | Nav     | 消息/监控 图标切换按钮                     |
  | Channels| 频道列表（all, kagent-integrate-sap-ai-core）|
  | Threads | 线程预览列表                              |
  | Agents  | Agent 列表（克劳德/Alice/Perter）带状态指示 |
  | Humans  | 用户列表                                  |
  | Footer  | 当前用户信息 + 设置按钮                    |
And 点击频道可切换选中状态（brutal-pink 高亮 + 偏移阴影）
```

#### Scenario 3: 主内容区标签页
```gherkin
Given MVP 应用已启动
When 用户切换标签页
Then 每个标签页应正确渲染对应内容：
  | 标签页    | 内容                                     |
  | CHAT     | 消息列表 + 输入框 + 发送按钮               |
  | TASKS    | 任务筛选器 + 任务列表 + 新建任务按钮        |
  | WORKSPACE| 文件浏览器 + 文件查看器                     |
  | SKILLS   | 技能卡片网格                               |
  | ACTIVITY | 活动日志列表                               |
  | PROFILE  | Agent 详细信息展示                          |
And 选中标签页应显示 brutal-yellow 背景高亮
```

#### Scenario 4: 聊天功能
```gherkin
Given 用户在 CHAT 标签页
When 用户在输入框输入消息并发送
Then 消息应显示在消息列表中
And Agent 应显示 "Thinking..." 状态
And Agent 回复后消息应追加到列表中
And 消息滚动到底部
```

#### Scenario 5: 模态框交互
```gherkin
Given MVP 应用已启动
When 触发创建任务或邀请用户操作
Then 应显示对应的模态框
And 模态框包含标题栏、内容区、底部操作按钮
And 点击 Cancel 或 X 按钮可关闭模态框
And 模态框使用 brutal-card + brutal-shadow 样式
```

### UI/Interaction Checkpoints
- [ ] 三栏布局正确渲染，比例与原型一致
- [ ] Sidebar 频道选中态：brutal-pink 背景 + 偏移阴影
- [ ] 标签页切换：brutal-yellow 高亮 + 平移效果
- [ ] 按钮 brutal-btn 样式：2px 黑边框 + 2px 偏移阴影 + 点击收缩效果
- [ ] 卡片 brutal-card 样式：4px 偏移阴影
- [ ] 输入框聚焦：背景色变化为 brutal-bg
- [ ] Agent 状态指示器：online=#39FF14, busy=#FFDE00
- [ ] 模态框遮罩层 + backdrop-blur 效果

### General Checklist
- [ ] 所有原型组件已移植到项目中
- [ ] Tailwind CSS Neo-Brutalism 样式配置完整
- [ ] cn() 工具函数正常工作
- [ ] TypeScript 类型定义完整
- [ ] 在 Tauri V2 窗口中正确显示
- [ ] 无控制台错误或警告

## Merge Record
- **Completed**: 2026-04-08T14:30:00+08:00
- **Merged Branch**: feature/feat-style-consistency
- **Merge Commit**: 414a869
- **Archive Tag**: feat-style-consistency-20260408
- **Conflicts**: None during merge
- **Verification**: All 5 Gherkin scenarios passed. TypeScript 0 errors. Vite build successful.
- **Stats**: 1 commit, 12 files changed (+1026/-142 lines)
