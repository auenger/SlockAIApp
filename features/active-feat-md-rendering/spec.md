# Feature: feat-md-rendering Markdown 渲染优化

## Basic Information
- **ID**: feat-md-rendering
- **Name**: Markdown 渲染优化 & Tool Call 结构化展示
- **Priority**: 60
- **Size**: L
- **Dependencies**: none
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10

## Description
优化 Chat、Thread、Channel 对话的 Markdown 渲染能力，兼容 MD 字符串的渲染。同时优化 Claude Code 的 tool call（Read/Edit/Bash/Write/Glob/Grep 等）的结构化展示。

当前状态：所有消息内容以纯文本 + whitespace-pre-wrap 渲染，仅支持 @mention 高亮。没有任何 Markdown 解析或代码高亮能力。

## User Value Points

### VP1: Markdown 消息渲染
在所有对话场景（Chat/Thread/Channel）中正确渲染 Markdown 格式的消息内容，支持标题、列表、粗体、斜体、链接、表格、引用、分割线等。兼容纯文本消息（无 markdown 语法的消息保持原有展示效果）。

### VP2: 代码块 Shiki 语法高亮
AI 返回的代码片段带语言检测、VSCode 级别语法高亮（基于 Shiki）、行号显示、一键复制功能。支持多语言代码高亮，暗色/亮色主题适配。

### VP3: Tool Call 结构化渲染
Claude Code 的 tool call（如 Read/Edit/Bash/Write/Glob/Grep 等）不再以原始 JSON 展示，而是以结构化卡片形式呈现：工具名称、参数摘要、执行状态、可折叠的结果区域。让用户清晰了解 Agent 执行了哪些操作。

## Context Analysis

### Reference Code
- `src/components/MainContent.tsx` (730-739 行) — 当前消息渲染入口，纯文本 + renderMentionText
- `src/types.ts` — ThreadMessageData、Message、StreamEvent 类型定义
- `src-tauri/src/runtime/claude.rs` — Claude Code runtime，extract_content_blocks 处理流式响应
- `reference/AINative/neuro-syntax-ide/src/components/common/MarkdownRenderer.tsx` — 参考：ReactMarkdown + remarkGfm 实现
- `reference/anyclaw/tauri-app/src/components/ai-elements/code-block.tsx` — 参考：Shiki 代码高亮实现
- `reference/anyclaw/tauri-app/src/components/ai-elements/message.tsx` — 参考：Tool call 结构化渲染

### Related Documents
- project-context.md — 技术栈 React 19 + Tailwind CSS 4 + Tauri V2

### Related Features
- feat-thread-chat (已完成) — Thread 对话基础
- feat-channel-multi-agent (已完成) — Channel 多 Agent 对话
- feat-agent-runtime-exec (已完成) — 多 Runtime 对话执行

## Technical Solution

### 实现方案

#### 新增文件
- `src/components/markdown/MarkdownRenderer.tsx` — 统一 Markdown 渲染，使用 react-markdown + remark-gfm + rehype-raw，自定义 Neo-Brutalism 样式
- `src/components/markdown/CodeBlock.tsx` — Shiki 语法高亮代码块，支持 20+ 语言，一行复制，行号显示，暗色/亮色主题
- `src/components/markdown/ToolCallBlock.tsx` — Tool Call 结构化卡片（Read/Edit/Bash/Write/Glob/Grep），可折叠/展开
- `src/components/markdown/MessageContentRenderer.tsx` — 消息内容统一渲染入口，支持纯文本/Markdown/ContentBlock[]
- `src/components/markdown/types.ts` — ContentBlock 类型定义和解析工具函数
- `src/components/markdown/index.ts` — Barrel export

#### 修改文件
- `src/components/MainContent.tsx` — Agent 消息渲染使用 MarkdownRenderer，流式文本也通过 MarkdownRenderer
- `src/components/ThreadPanel.tsx` — Thread Agent 消息渲染使用 MarkdownRenderer (compact mode)
- `src/types.ts` — StreamEvent 增加 content_blocks 字段
- `src/index.css` — 添加 Shiki 代码高亮样式和 Markdown body 样式
- `src-tauri/src/runtime/mod.rs` — StreamEvent 结构体增加 content_blocks 字段
- `src-tauri/src/runtime/claude.rs` — 从 Claude CLI verbose JSON 提取 tool_use/tool_result 结构化数据
- `src-tauri/src/runtime/codex.rs` — StreamEvent 兼容更新

### 技术选型
- **react-markdown** — Markdown 解析渲染
- **remark-gfm** — GitHub Flavored Markdown 支持（表格、删除线、任务列表）
- **rehype-raw** — 允许渲染原始 HTML
- **shiki** — 代码块语法高亮（VSCode 级别，支持主题切换）

### 组件规划
1. `MarkdownRenderer` — 统一的 Markdown 渲染组件（替代当前纯文本渲染）
2. `CodeBlock` — 代码块组件（Shiki 高亮 + 复制 + 行号）
3. `ToolCallBlock` — Tool Call 结构化展示组件（可折叠卡片）
4. `ToolResultBlock` — Tool Result 结果展示组件

### 数据模型增强
- ThreadMessageData.content 需支持 string | ContentBlock[] 类型
- 新增 ToolUseBlock / ToolResultBlock 类型
- 流式响应解析需区分 text / tool_use / tool_result 内容块

## Acceptance Criteria (Gherkin)

### User Story
作为一个 AgentsZone 用户，我希望在 Chat、Thread、Channel 对话中看到格式良好的 Markdown 内容、语法高亮的代码块和结构化的 Tool Call 展示，以便我能高效阅读 Agent 的回复并理解 Agent 执行的操作。

### Scenarios (Given/When/Then)

#### VP1: Markdown 消息渲染

**Scenario 1: 基础 Markdown 元素渲染**
```gherkin
Given 用户在 Thread 对话中与 Agent 交互
When Agent 返回包含标题(#)、粗体(**)、斜体(*)、链接([]())、列表(-)的 Markdown 内容
Then 对话消息区域应正确渲染这些 Markdown 元素
And 纯文本消息（无 Markdown 语法）应保持原有展示效果不变
```

**Scenario 2: GFM 表格和任务列表渲染**
```gherkin
Given 用户在 Channel 对话中查看 Agent 响应
When Agent 返回包含 Markdown 表格和任务列表(- [ ]) 的内容
Then 表格应以格式化的 HTML 表格展示
And 任务列表应显示为可读的复选框列表
```

**Scenario 3: Chat/Thread/Channel 统一渲染**
```gherkin
Given 系统有 Thread 对话、Channel 对话、Chat 消息三种场景
When 任何场景中的消息包含 Markdown 内容
Then 所有场景应使用相同的 MarkdownRenderer 组件进行渲染
And 渲染效果在三种场景中保持一致
```

#### VP2: 代码块 Shiki 语法高亮

**Scenario 4: 代码块语法高亮与复制**
```gherkin
Given 用户查看 Agent 返回的代码块（```language ... ```）
When 代码块被渲染
Then 代码应有基于 Shiki 的语法高亮，根据编程语言着色
And 代码块右上角应有"复制"按钮
And 点击复制按钮应将代码内容复制到剪贴板
```

**Scenario 5: 多语言代码高亮**
```gherkin
Given Agent 返回包含 TypeScript、Rust、Python、Bash 等不同语言的代码块
When 渲染这些代码块
Then 每种语言应使用对应的语法高亮规则
And 代码块左上角应显示语言标签
```

**Scenario 6: 内联代码渲染**
```gherkin
Given Agent 消息中包含内联代码（`code`）
When 消息被渲染
Then 内联代码应以区分于正文背景色的样式展示
And 不应被渲染为独立的代码块
```

#### VP3: Tool Call 结构化渲染

**Scenario 7: Tool Call 卡片展示**
```gherkin
Given Agent 执行了 tool call（如 Read/Edit/Bash/Write）
When tool call 信息被包含在消息中
Then 应以结构化卡片展示工具名称（如 "Read file"）
And 卡片应显示关键参数摘要（如文件路径）
And 卡片应显示执行状态（运行中/完成/失败）
```

**Scenario 8: Tool Result 折叠展示**
```gherkin
Given tool call 执行完成并返回结果
When tool result 被渲染
Then 结果默认折叠，用户可点击展开查看详情
And 折叠状态应显示结果摘要（如行数、大小）
And 长结果内容应有滚动条而非撑开整个页面
```

**Scenario 9: Tool Call 流式状态更新**
```gherkin
Given Agent 正在流式执行 tool call
When tool call 开始执行
Then 应立即显示"运行中"状态指示器
When tool call 执行完成
Then 状态指示器应更新为"完成"并显示结果
```

### UI/Interaction Checkpoints
- MarkdownRenderer 应继承父容器宽度，不超出消息区域
- 代码块应有固定最大高度 + 滚动条
- Tool Call 卡片应有折叠/展开动画
- 整体风格需匹配项目的新粗野主义（Neo-Brutalism）设计
- 暗色/亮色代码高亮主题应跟随应用主题

### General Checklist
- 安装 react-markdown、remark-gfm、shiki 等依赖
- 创建 MarkdownRenderer 组件
- 创建 CodeBlock 组件（Shiki 集成）
- 创建 ToolCallBlock / ToolResultBlock 组件
- 更新 MainContent.tsx 消息渲染逻辑
- 更新 ThreadPanel.tsx 消息渲染逻辑
- 增强消息类型定义支持 ContentBlock[]
- 增强流式响应解析逻辑
- 样式适配 Neo-Brutalism 风格
