# Tasks: feat-thread-ux-polish

## Task Breakdown

### 1. Thinking/Streaming 动画
- [x] 在 ThreadPanel 中添加 Thinking 状态指示器（弹跳动画）
- [x] 在 ThreadPanel 中添加 Streaming 文本实时显示 + 三点弹跳动画
- [x] 确保动画风格与 MainContent 中 brutalist 主题一致
- [x] 处理状态转换：idle → thinking → streaming → complete

### 2. 宽度调整优化
- [x] 检查现有 useResizable 在 ThreadPanel 的集成状态
- [x] 优化拖拽手柄的视觉可发现性（hover 效果、拖拽提示）
- [x] 调整宽度范围至更合理的值

### 3. 集成与测试
- [x] 测试 thinking → streaming 状态转换流畅性
- [x] 测试宽度调整不影响消息渲染
- [x] 测试动画在长时间对话中的性能

## Implementation Notes

### Files Modified
- `src/components/ThreadPanel.tsx` — Added isThinking/isStreaming/streamingText props, thinking indicator (pulse animation), streaming indicator (MarkdownRenderer + bouncing dots), improved resize handle with grip dots
- `src/App.tsx` — Pass streaming state props to ThreadPanel, widened resize range to 280-600px

### Design Decisions
- Thinking indicator: animate-pulse with gray progress bar, matching MainContent's thinking pattern
- Streaming indicator: MarkdownRenderer for real-time text + 3-dot bounce animation with staggered delays (0ms/150ms/300ms), using bg-brutal-cyan color
- Resize handle: w-1.5 with group hover showing 3 grip dots, opacity transition for discoverability
- Auto-scroll triggers on both thread.messages and streamingText changes

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Feature created | 待开发 |
| 2026-04-16 | Implementation complete | All tasks done, TypeScript compiles clean |
