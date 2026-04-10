# Tasks: feat-md-rendering

## Task Breakdown

### 1. 依赖安装与基础配置
- [x] 安装 react-markdown、remark-gfm、rehype-raw
- [x] 安装 shiki 及相关类型定义
- [x] 配置 Shiki 主题（匹配 Neo-Brutalism 风格）

### 2. MarkdownRenderer 组件
- [x] 创建 `src/components/markdown/MarkdownRenderer.tsx` 统一渲染组件
- [x] 配置 react-markdown + remark-gfm 插件
- [x] 实现自定义 Markdown 元素样式（标题、列表、链接、表格、引用、分割线）
- [x] 集成 CodeBlock 组件处理代码块
- [x] 实现纯文本自动兼容（无 MD 语法的消息不出现异常渲染）
- [x] 编写 Tailwind 样式，匹配 Neo-Brutalism 设计风格

### 3. CodeBlock 组件（Shiki 高亮）
- [x] 创建 `src/components/markdown/CodeBlock.tsx` 代码块组件
- [x] 集成 Shiki 语法高亮引擎
- [x] 实现语言检测和标签显示
- [x] 实现一键复制按钮（复制到剪贴板）
- [x] 实现行号显示（可切换）
- [x] 处理代码块最大高度 + 滚动条
- [x] 暗色/亮色主题适配

### 4. Tool Call 结构化渲染
- [x] 扩展消息类型定义（ContentBlock、ToolUseBlock、ToolResultBlock）
- [x] 创建 `src/components/markdown/ToolCallBlock.tsx` 工具调用卡片组件
- [x] 创建 `src/components/markdown/ToolResultBlock.tsx` 结果展示组件
- [x] 实现折叠/展开交互
- [x] 实现执行状态指示器（运行中/完成/失败）
- [x] 为不同工具类型定制显示模板（Read/Edit/Bash/Write/Glob/Grep）

### 5. 集成到现有对话界面
- [x] 更新 `MainContent.tsx` — Channel 消息渲染使用 MarkdownRenderer
- [x] 更新 `ThreadPanel.tsx` — Thread 消息渲染使用 MarkdownRenderer
- [x] 增强 `src-tauri/src/runtime/claude.rs` 流式响应解析 — 区分 text/tool_use/tool_result
- [x] 更新 `useThreadChat.ts` 流式处理逻辑支持 ContentBlock[]
- [x] 保留 @mention 渲染兼容性

### 6. 样式打磨与测试
- [x] Neo-Brutalism 风格适配（粗描边、配色、圆角）
- [ ] 响应式测试 — 不同窗口宽度下渲染效果
- [ ] 性能测试 — 大量消息 + 长代码块渲染性能
- [ ] 纯文本消息回归测试 — 确保原有消息展示不受影响

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 初始任务拆解 |
| 2026-04-11 | Implementation | Tasks 1-4 完成，Task 5 UI集成完成，Rust流式解析待做 |
| 2026-04-11 | Build verified | TypeScript 编译通过，Vite build 成功 |
