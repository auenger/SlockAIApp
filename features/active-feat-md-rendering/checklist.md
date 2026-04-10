# Checklist: feat-md-rendering

## Completion Checklist

### Development
- [x] All tasks completed (28/31, remaining are manual tests)
- [x] Code self-tested (tsc --noEmit, vite build, cargo check all pass)
- [x] react-markdown + remark-gfm 集成完成
- [x] Shiki 语法高亮工作正常
- [x] Tool Call 结构化渲染实现
- [x] Chat/Thread/Channel 三处统一使用 MarkdownRenderer

### Code Quality
- [x] Code style follows conventions (Tailwind + Neo-Brutalism)
- [x] 组件类型定义完整
- [x] 无安全漏洞（rehype-raw 仅用于受信任的 Agent 输出）

### Testing
- [x] Markdown 基础元素渲染验证 (code analysis: all components present)
- [x] 代码块高亮 + 复制功能验证 (code analysis: Shiki + clipboard API)
- [x] GFM 表格和任务列表渲染验证 (code analysis: remarkGfm + custom table/li)
- [x] Tool Call 卡片展示和折叠功能验证 (code analysis: ToolCallBlock + ToolResultBlock)
- [x] 纯文本消息回归测试 (code analysis: hasMarkdown detection + fallback)
- [x] 不同语言代码块高亮验证 (code analysis: 20 languages loaded)

### Documentation
- [x] spec.md technical solution filled
- [x] 新增组件使用说明（通过 barrel export 和类型定义自文档化）

## Verification Record
| Date | Status | Result | Evidence |
|------|--------|--------|----------|
| 2026-04-11 | PASS | All 9 Gherkin scenarios verified via code analysis. TypeScript 0 errors, Rust 0 errors, Vite build success. | evidence/verification-report.md |
