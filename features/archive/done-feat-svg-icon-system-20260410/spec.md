# Feature: feat-svg-icon-system SVG Icon 系统

## Basic Information
- **ID**: feat-svg-icon-system
- **Name**: SVG Icon 系统
- **Priority**: 70
- **Size**: M
- **Dependencies**: none
- **Parent**: null
- **Children**: [feat-agent-edit]
- **Created**: 2026-04-10

## Description

引入一套 SVG Icon 系统，替代当前纯 emoji 的 Agent/User 图标方案。提供可视化的 Icon Picker 组件，支持分类浏览和搜索，让用户和 Agent 都有更丰富、更专业的图标选择。

当前问题：
- Agent 图标只能用 emoji，视觉上不够专业
- avatar 字段存在但未启用
- 没有统一的 icon 管理和选择机制

## User Value Points

### VP1: SVG Icon 库集成
引入一套高质量的 SVG icon 集（如 Lucide Icons 扩展、Phosphor Icons 或自定义 icon），作为 Agent/User 图标的基础素材库。项目已使用 lucide-react，可以在此基础上扩展或引入额外 icon 包。

### VP2: Icon Picker 组件
提供可视化的 Icon Picker 组件：
- 按分类浏览（人物、动物、工具、自然、科技等）
- 支持搜索过滤
- 选中后预览效果
- 可以配合颜色选择

### VP3: Agent/User 图标应用
将选中的 SVG icon 应用到 Agent 和 User 的头像显示：
- Sidebar 中的 Agent 列表
- Channel 中的 Agent 标识
- Thread 中的 Agent 头像
- Agent Profile 页面
- 统一使用 SVG icon 替代 emoji

## Context Analysis

### Reference Code
- `src/components/agent/CreateAgentModal.tsx` — 当前 emoji 选择方式
- `src/components/layout/Sidebar.tsx` — Agent 图标展示
- `src/types.ts` — Agent 类型定义（emoji, avatar 字段）
- `src/lib/ipc.ts` — Agent IPC 调用

### Related Documents
- project-context.md — UI 风格为新粗野主义

### Related Features
- feat-agent-edit — 依赖本 feature 的 Icon Picker 组件

## Technical Solution

### 方案
- 扩展现有 lucide-react icon 集，100+ 图标，7 个分类（Characters, Nature, Tech, Objects, Chat, Arrows, Status）
- Agent 数据模型中 `icon` 字段存储 icon name（如 "Bot", "Cat", "Rocket"）
- Icon Picker 组件基于 Popover + Grid 布局，支持分类导航和搜索
- 颜色系统复用现有 AGENT_COLORS
- AgentIcon 组件统一渲染 SVG icon + emoji 向后兼容

## Merge Record
- **Completed**: 2026-04-10T17:35:00+08:00
- **Merged branch**: feature/feat-svg-icon-system
- **Merge commit**: 038b22c
- **Archive tag**: feat-svg-icon-system-20260410
- **Conflicts**: none
- **Verification**: passed (4/4 Gherkin scenarios, 17/17 tasks)
- **Files changed**: 8 (3 new, 5 modified)
- **Duration**: ~35 minutes

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望为 Agent 和自己选择专业的 SVG 图标，以便在界面上更好地区分不同的 Agent 和个人身份。

### Scenarios (Given/When/Then)

#### Scenario 1: Icon Picker 浏览和搜索
```gherkin
Given 用户打开 Icon Picker
When 用户浏览分类或输入搜索关键词
Then 应显示匹配的 SVG icon 列表
And 用户可以点击选择一个 icon
And 选中后显示预览效果
```

#### Scenario 2: Agent 图标显示 SVG
```gherkin
Given 一个 Agent 已配置了 SVG icon
When 该 Agent 出现在 Sidebar / Channel / Thread 中
Then 应使用对应的 SVG icon 渲染头像
And 图标应配合 Agent 的主题色显示
```

#### Scenario 3: Icon 数据持久化
```gherkin
Given 用户为 Agent 选择了 icon "rocket"
When Agent 配置保存后
Then agent.identity 中应包含 icon: "rocket"
And 下次加载时能正确还原显示
```

#### Scenario 4: 向后兼容 emoji
```gherkin
Given 已有 Agent 使用 emoji 作为图标
When 系统加载这些 Agent
Then 应保持 emoji 显示正常
And 用户可以选择升级为 SVG icon
```

### UI/Interaction Checkpoints
- Icon Picker 弹出层交互流畅
- 分类切换和搜索响应及时
- 选中 icon 后即时预览
- 大小适配 Sidebar 小尺寸和 Profile 大尺寸

### General Checklist
- [x] icon 选择结果持久化到 agent identity
- [x] 向后兼容已有 emoji 数据
- [x] Icon Picker 组件可复用于 Agent 和 User 场景
- [x] SVG icon 在小尺寸下清晰可辨
