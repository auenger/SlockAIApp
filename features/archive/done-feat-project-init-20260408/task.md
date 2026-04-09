# Tasks: feat-project-init

## Task Breakdown

### 1. Tauri V2 + Vite + React 项目脚手架
- [x] 使用 `npm create tauri-app` 或手动方式初始化 Tauri V2 项目
- [x] 配置 Vite 6 作为前端构建工具
- [x] 配置 React 19 + TypeScript 5.8
- [x] 配置 `tauri.conf.json` v2 格式（窗口标题 "SlockAI"、窗口大小等）
- [x] 验证 `cargo tauri dev` 能正常启动桌面应用

### 2. Tailwind CSS 4 + 样式基础
- [x] 安装并配置 Tailwind CSS 4
- [x] 安装 clsx + tailwind-merge，创建 `cn()` 工具函数
- [x] 配置新粗野主义基础色板（Lemon Yellow 侧边栏、米白背景等）
- [x] 创建 `index.css` Tailwind 入口文件

### 3. Rust 后端模块结构
- [x] 创建 `src-tauri/src/commands/mod.rs` 占位模块
- [x] 创建 `src-tauri/src/context/mod.rs` 占位模块
- [x] 创建 `src-tauri/src/runtime/mod.rs` 占位模块
- [x] 创建 `src-tauri/src/storage/mod.rs` 占位模块
- [x] 创建一个测试用 Tauri command（验证 IPC 通信）
- [x] 更新 `lib.rs` 注册所有模块和 command

### 4. 前端目录结构 + 基础组件
- [x] 创建 `src/components/layout/` 目录及占位组件
- [x] 创建 `src/components/layout/Sidebar.tsx` 三栏布局左侧边栏占位
- [x] 创建 `src/components/layout/MainView.tsx` 中间主视图占位
- [x] 创建 `src/components/layout/DetailView.tsx` 右侧详情面板占位
- [x] 创建 `src/lib/ipc.ts` Tauri IPC 封装（invoke/listen 类型安全包装）
- [x] 创建 `src/types.ts` 基础类型定义
- [x] 更新 `App.tsx` 使用三栏布局

### 5. 验证与构建
- [x] 确认 `cargo tauri dev` 无错误启动
- [x] 确认 `cargo tauri build` 能正常构建
- [x] 确认 TypeScript 无类型错误
- [x] 确认 Rust 编译无 warning

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-07 | Feature created | 初始化 spec 和 task |
| 2026-04-08 | Implementation complete | All tasks done, TypeScript/Vite/Rust all compile clean |
