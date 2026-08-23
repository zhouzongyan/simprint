# Simprint 本地开发配置

本文按 Windows + PowerShell 编写，目标是从一台没有开发环境的机器开始，完成 Simprint 桌面端的依赖安装和开发启动。

## 1. 先了解项目结构

这个仓库包含三套相互独立的工程：

| 目录 | 技术栈 | 是否是桌面端启动的必需项 |
| --- | --- | --- |
| 根目录 | React、Vite、TypeScript、pnpm | 是 |
| `src-tauri` | Tauri 2、Rust、嵌入式 SQLite | 是 |
| `server` | Axum、Rust、PostgreSQL | 仅服务端联调时需要 |

桌面端当前是 local-first 架构：业务请求会进入 `src-tauri` 内嵌的 SQLite 业务层，首次启动会自动创建本地数据库和迁移表结构。因此，普通桌面端开发不需要先启动 PostgreSQL、Redis 或远程 API。

## 2. 安装 Windows 前置环境

### 2.1 Node.js

仓库的 CI 使用 Node.js 20。建议安装 Node.js 20 LTS，不建议使用与 CI 差异较大的版本作为开发基线。

安装后在 PowerShell 检查：

```powershell
node --version
npm --version
```

### 2.2 pnpm 9

仓库 CI 固定使用 pnpm 9，且 [`pnpm-lock.yaml`](../pnpm-lock.yaml) 的锁文件版本为 `9.0`。安装 pnpm 9：

```powershell
npm install --global pnpm@9
pnpm --version
```

### 2.3 Rust stable MSVC

安装 Rustup 后，确认使用 Windows MSVC 工具链：

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustc --version
cargo --version
rustup show
```

如果 `rustup show` 中没有 `stable-x86_64-pc-windows-msvc`，补装目标：

```powershell
rustup target add x86_64-pc-windows-msvc
```

### 2.4 Visual Studio C++ 构建工具

通过 Visual Studio Installer 安装 Build Tools，并勾选：

- Desktop development with C++
- MSVC 编译工具
- Windows 10/11 SDK

缺少这些组件时，Rust 编译通常会出现 `link.exe`、Windows SDK 或 C++ 工具链错误。

### 2.5 WebView2

Tauri 2 使用 Microsoft Edge WebView2。Windows 11 通常已经安装；Windows 10 如果启动时提示缺少 WebView2，请安装 Microsoft WebView2 Runtime。

## 3. 获取代码并进入仓库

```powershell
git clone REPOSITORY_URL simprint
Set-Location D:\code\rust\simprint
```

后续命令默认都在 `D:\code\rust\simprint` 执行。路径可以替换为实际目录，但不要在仓库外执行根目录的 `pnpm install`。

## 4. 安装根目录前端依赖

使用锁文件安装，避免依赖解析结果漂移：

```powershell
pnpm install --frozen-lockfile
```

成功后应存在 `node_modules` 目录。若提示锁文件需要更新，优先确认 pnpm 是 9.x，不要直接删除锁文件或使用 `--no-frozen-lockfile`。

## 5. 安装 Tauri CLI

仓库的 [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml) 使用 Tauri 2，但根目录 `package.json` 没有声明 npm 版 Tauri CLI。项目 README 使用 Cargo 版命令，因此安装 Cargo CLI：

```powershell
cargo install tauri-cli --version "^2.0.0" --locked
cargo tauri --version
```

安装过程可能较慢，因为 Cargo 需要编译 CLI 本身。`cargo tauri --version` 能输出版本，才继续下一步。

## 6. 创建桌面端开发配置

复制示例配置：

```powershell
Copy-Item `
  .\src-tauri\config.example.toml `
  .\src-tauri\config.development.toml
```

仓库的开发启动约定要求从示例复制该文件；当前内容主要是固定版本 WebView 下载地址。它不包含远程 API 地址，也不需要填写 PostgreSQL 连接信息。

`config.development.toml` 已被 `.gitignore` 忽略，不要把真实环境地址、密钥或个人路径提交到仓库。

## 7. 启动桌面端

```powershell
cargo tauri dev --features development
```

启动过程会自动完成以下工作：

1. 执行 `pnpm dev`，启动 Vite/Slotkit 前端开发服务。
2. 编译 `src-tauri` 及两个本地 crate：`business`、`runtime`。
3. 创建本地 SQLite 数据库 `simprint.db`。
4. 执行内嵌数据库迁移，并初始化本地用户、工作区和浏览器内核目录。
5. 打开 Tauri 桌面窗口。

第一次编译 Rust 依赖可能需要较长时间，后续启动会使用 Cargo 缓存。

### 只调试前端

不需要 Rust 或 Tauri 窗口时，可以只启动前端：

```powershell
pnpm dev
```

但依赖 Tauri `invoke`、本地 SQLite、浏览器内核或系统窗口的功能，在纯 Vite 页面中不能完整工作。

## 8. 可选：启动服务端进行联调

只有需要验证独立 HTTP 服务、PostgreSQL 数据库或服务端路由时，才执行这一节。

### 8.1 准备 PostgreSQL

Windows 上推荐使用 Docker Desktop，只启动 Compose 中的 PostgreSQL：

```powershell
docker compose -f .\server\docker-compose.yml up -d postgres
```

Compose 会创建：

- 数据库：`simprintdb`
- 用户：`simprint`
- 密码：`change-me`
- 地址：`127.0.0.1:5432`

检查容器状态：

```powershell
docker compose -f .\server\docker-compose.yml ps
```

### 8.2 配置并启动服务端

```powershell
Set-Location .\server
Copy-Item .\configs\config.local.example.toml .\configs\config.local.toml
cargo fetch
cargo run -- -f .\configs\config.local.toml
```

服务端默认监听：

```text
http://127.0.0.1:40041
```

启动时会自动执行 `server/migrations` 中的数据库迁移。

当前服务端本地配置说明：

- PostgreSQL 是当前本地启动最关键的外部依赖。
- `storage.public_base_url` 只是资源 URL 配置，使用扩展、头像或版本下载功能时需要替换为真实地址。
- SMTP 是邮件功能的可选依赖。
- README 中提到 Redis，但当前服务上下文使用内存缓存，源码没有对应的 Redis 连接配置；不要因为桌面端开发而额外安装 Redis。

停止服务端：在运行服务端的 PowerShell 窗口按 `Ctrl+C`。停止 PostgreSQL 容器：

```powershell
docker compose -f .\server\docker-compose.yml stop postgres
```

## 9. 启动成功后的检查

完成桌面端启动后，至少确认：

- 桌面窗口能够打开。
- 没有出现 `cargo tauri`、`link.exe` 或 WebView2 初始化错误。
- 应用数据目录中生成了 `simprint.db`。
- 能进入主界面，不会持续显示数据库初始化失败。

代码检查命令从仓库根目录执行：

```powershell
pnpm lint
pnpm format:check
pnpm rust:fmt:check
pnpm rust:check
```

服务端单独检查：

```powershell
Push-Location .\server
cargo check
Pop-Location
```

## 10. 常见问题

### `cargo tauri` 不是有效命令

说明 Tauri CLI 没安装，执行：

```powershell
cargo install tauri-cli --version "^2.0.0" --locked
```

然后重新打开 PowerShell，再检查：

```powershell
cargo tauri --version
```

### `link.exe` 或 Windows SDK 找不到

重新打开 Visual Studio Installer，确认安装了 Desktop development with C++、MSVC 和 Windows SDK，并确认 Rust 使用 `stable-x86_64-pc-windows-msvc`。

### `pnpm install --frozen-lockfile` 失败

先检查版本：

```powershell
node --version
pnpm --version
```

优先使用 Node.js 20 和 pnpm 9。不要先删除 `pnpm-lock.yaml`。

### 端口 `40041` 被占用

查看占用进程：

```powershell
Get-NetTCPConnection -LocalPort 40041 -ErrorAction SilentlyContinue
```

关闭占用端口的程序，或修改 `server/configs/config.local.toml` 中的 `app.port`。

### 服务端提示 PostgreSQL 连接失败

确认容器正在运行：

```powershell
docker compose -f .\server\docker-compose.yml ps postgres
```

并确认 `server/configs/config.local.toml` 中的连接串仍是：

```text
postgres://simprint:change-me@127.0.0.1:5432/simprintdb
```

### 桌面端需要远程服务地址吗？

当前桌面端的主要业务路由已经由 `src-tauri` 的本地 SQLite 业务层处理，最小开发启动不需要配置 `base_url`。只有明确开发独立服务端或尚未迁移到本地业务层的功能时，才需要按具体代码路径配置远程服务。

## 11. 推荐的日常启动顺序

普通桌面端开发：

```powershell
Set-Location D:\code\rust\simprint
cargo tauri dev --features development
```

前端页面开发：

```powershell
Set-Location D:\code\rust\simprint
pnpm dev
```

桌面端 + 独立服务端联调：先启动 PostgreSQL 和 `server`，再在另一个 PowerShell 窗口启动：

```powershell
Set-Location D:\code\rust\simprint
cargo tauri dev --features development
```
