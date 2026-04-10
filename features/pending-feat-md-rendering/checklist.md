# Checklist: feat-md-rendering

## Completion Checklist

### Development
- [ ] All tasks completed
- [ ] Code self-tested
- [ ] react-markdown + remark-gfm 集成完成
- [ ] Shiki 语法高亮工作正常
- [ ] Tool Call 结构化渲染实现
- [ ] Chat/Thread/Channel 三处统一使用 MarkdownRenderer

### Code Quality
- [ ] Code style follows conventions (Tailwind + Neo-Brutalism)
- [ ] 组件类型定义完整
- [ ] 无安全漏洞（XSS — react-markdown rehype-raw 配置安全）

### Testing
- [ ] Markdown 基础元素渲染验证
- [ ] 代码块高亮 + 复制功能验证
- [ ] GFM 表格和任务列表渲染验证
- [ ] Tool Call 卡片展示和折叠功能验证
- [ ] 纯文本消息回归测试通过
- [ ] 不同语言代码块高亮验证

### Documentation
- [ ] spec.md technical solution filled
- [ ] 新增组件使用说明
