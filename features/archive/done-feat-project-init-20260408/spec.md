# Feature: feat-project-init Project Init (Tauri V2 + React 19)

## Basic Information
- **ID**: feat-project-init
- **Name**: Project Init (Tauri V2 + React 19)
- **Priority**: 90
- **Size**: M
- **Dependencies**: none
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-07

## Description

按照 project-context.md 的技术架构，完成项目的初始化搭建：

1. **Tauri V2 桌面应用初始化** — 使用 Tauri V2 创建桌面应用壳，配置 Rust 后端基础结构
2. **React 19 前端初始化** — 在 Tauri 项目内搭建 React 19 + TypeScript 5.8 + Vite 6 + Tailwind CSS 4 前端

技术栈：React 19, TypeScript 5.8, Vite 6, Tailwind CSS 4, Tauri V2, Rust

## User Value Points

### VP1: Tauri V2 桌面应用骨架
用户能够启动一个 Tauri V2 桌面应用，Rust 后端具备基础模块结构（commands, context, runtime, storage 目录）。

### VP2: React 19 前端开发环境
开发者能够在前端进行 React 开发，具备 HMR 热更新、TypeScript 类型检查、Tailwind CSS 样式系统、基础三栏布局可见。

## Context Analysis

### Reference Code
- project-context.md — 完整技术架构和目录结构规划
- 当前项目仅有 feature-workflow 配置，无业务代码

### Related Documents
- Tauri V2 官方文档: https://v2.tauri.app
- React 19 文档
- Vite 6 文档
- Tailwind CSS 4 文档

### Related Features
- 无前置依赖
- 后续 feature: 三栏布局、JSONL 存储层、Channel 对话容器等

## Technical Solution

### 初始化步骤

#### 1. Tauri V2 项目创建
```bash
# 使用 Tauri V2 + React + TypeScript 模板
npm create tauri-app@latest
# 或手动初始化: npm create vite@latest + tauri init
```

Tauri 配置要点:
- `tauri.conf.json` v2 格式
- 窗口大小、标题等基础配置
- Rust 后端目录结构按 project-context 规划

#### 2. React 前端配置
- React 19 + TypeScript 5.8
- Vite 6 构建
- Tailwind CSS 4 样式系统
- `cn()` 工具函数 (clsx + tailwind-merge)
- 基础类型定义 `types.ts`

#### 3. Rust 后端目录结构
```
src-tauri/src/
├── main.rs
├── lib.rs
├── commands/     (mod.rs 占位)
├── context/      (mod.rs 占位)
├── runtime/      (mod.rs 占位)
└── storage/      (mod.rs 占位)
```

#### 4. 前端目录结构
```
src/
├── components/
│   └── layout/   # 三栏布局占位
├── hooks/
├── lib/
│   ├── ipc.ts    # Tauri IPC 封装占位
│   └── utils.ts  # cn() 等工具函数
├── types.ts
├── App.tsx
├── main.tsx
└── index.css     # Tailwind 入口
```

## Acceptance Criteria (Gherkin)

### User Story
作为一个开发者，我希望项目初始化完成后能够直接运行桌面应用并进行前端开发，以便后续在此基础上开发业务功能。

### Scenarios (Given/When/Then)

#### Scenario 1: Tauri 桌面应用启动
```gherkin
Given 一个新克隆的 SlockAI 项目
When 运行 `cargo tauri dev`
Then Tauri 桌面窗口正常打开
And 窗口标题为 "SlockAI"
And Rust 后端模块结构按规划就位
```

#### Scenario 2: React 前端开发环境
```gherkin
Given 项目已初始化
When 在 src/ 目录下修改 React 组件
Then Vite HMR 热更新生效
And TypeScript 类型检查正常
And Tailwind CSS 样式生效
```

#### Scenario 3: IPC 通信验证
```gherkin
Given Tauri 应用正在运行
When 前端调用 Tauri invoke 发送测试 command
Then Rust 后端接收并返回响应
And 前端正确显示返回结果
```

#### Scenario 4: 构建产物验证
```gherkin
Given 项目已完成初始化
When 运行 `cargo tauri build`
Then 成功生成桌面应用安装包
And 无编译错误或 TypeScript 错误
```

### UI/Interaction Checkpoints
- 桌面窗口正常显示，无白屏
- 基础三栏布局占位可见 (Sidebar | Main | Detail)
- 新粗野主义样式基础色彩可辨别

### General Checklist
- [x] 无 console 错误
- [x] Rust 后端编译无 warning
- [x] TypeScript 无类型错误
- [x] `cargo tauri dev` 正常启动
- [x] `cargo tauri build` 正常构建

## Merge Record
- **Completed**: 2026-04-08
- **Merged Branch**: feature/feat-project-init
- **Merge Commit**: 2c127be
- **Archive Tag**: feat-project-init-20260408
- **Conflicts**: None
- **Verification**: All 4 Gherkin scenarios passed, 26/26 tasks completed
- **Files Changed**: 44
