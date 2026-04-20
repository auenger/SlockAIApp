# az-bridge Windows 交叉编译打包指南

## 前置条件

### 1. Rust target（只需一次）

```bash
rustup target add x86_64-pc-windows-gnu
```

### 2. MinGW 交叉编译工具链（只需一次）

```bash
brew install mingw-w64
```

验证安装：

```bash
which x86_64-w64-mingw32-gcc
# 应输出 /usr/local/bin/x86_64-w64-mingw32-gcc
```

## 编译命令

```bash
# 进入 Tauri 后端目录
cd src-tauri

# 编译 az-bridge（必须加 --no-default-features，排除 Tauri GUI 依赖）
cargo build --bin az-bridge --target x86_64-pc-windows-gnu --release --no-default-features
```

## 产物路径

```
src-tauri/target/x86_64-pc-windows-gnu/release/az-bridge.exe
```

约 9.7MB。

## 部署到 Windows

将 `az-bridge.exe` 上传到 Windows 服务器，启动命令：

```powershell
.\az-bridge.exe --port 7878
# 或指定工作区
.\az-bridge.exe --port 7878 --workspace C:\Users\your-user\agentszone
```

## 注意事项

- **必须加 `--no-default-features`**：不加会链接 Tauri 框架（~29MB），Windows 上因缺少 WebView2 运行时而无法启动
- **代码中 Windows 兼容**：`claude.rs` / `codex.rs` 已通过 `cfg!(windows)` 自动使用 `claude.cmd` / `codex.cmd`
- **桥代码在 `#[cfg(feature = "tauri-app")]` 守卫下**：`registry.rs` 中依赖 `rusqlite` 的部分仅 Tauri 应用编译，不影响桥

## 快速更新流程

```bash
# 1. 改代码后，一键编译
cd src-tauri && cargo build --bin az-bridge --target x86_64-pc-windows-gnu --release --no-default-features

# 2. 产物位置
ls -lh target/x86_64-pc-windows-gnu/release/az-bridge.exe

# 3. 上传到 Windows 服务器替换旧文件
scp target/x86_64-pc-windows-gnu/release/az-bridge.exe user@server:/path/to/az-bridge.exe
```

## 常见问题

### brew install mingw-w64 报锁文件冲突

```bash
# 清理锁文件后重试
rm -f ~/Library/Caches/Homebrew/downloads/*mingw*.incomplete
brew install mingw-w64
```

### 编译报 dlltool not found

说明 MinGW 未安装或不在 PATH，确认：

```bash
ls /usr/local/Cellar/mingw-w64/
```
