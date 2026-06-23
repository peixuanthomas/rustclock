# Rust Clock

一个基于 **Tauri** 的桌面时钟应用：Rust 作为原生外壳，界面由 HTML / CSS / SVG 渲染。
采用 Web 渲染栈以获得更好的图形效果——真正的 CSS `backdrop-filter` 毛玻璃质感，
以及 SVG `<feGaussianBlur>` 高斯模糊光晕。支持多种表盘风格、数字样式与倒计时功能。

## 功能特性

- 6 种时钟显示风格（`Classic hands`、`Frosted glass` 毛玻璃指针、`Luminous ticks`、`Triangle sweep`、`Orbit dots`、`Arc bands`）
- 3 种刻度数字样式（阿拉伯数字、罗马数字、无数字）
- 可切换秒针显示、平滑走针
- 内置倒计时（支持创建、选择、删除多个倒计时）
- 全屏显示与快捷键操作（`F11` 切换全屏，`Esc` 退出全屏）

## 技术栈

- **Rust 2021** + [Tauri 2](https://tauri.app/)（原生窗口 / WebView 外壳）
- 前端：纯静态 `ui/`（`index.html` + `styles.css` + `app.js`），无需 Node 构建步骤
- Windows 上使用 WebView2 运行时

## 项目结构

```
ui/                 前端（HTML/CSS/SVG/JS，时钟全部渲染逻辑）
src/main.rs         Tauri 入口
tauri.conf.json     Tauri 配置（窗口、前端目录、打包）
capabilities/       Tauri 权限（窗口全屏等）
icons/              应用图标（由 scripts/gen_icon.js 生成）
scripts/gen_icon.js 无依赖的图标生成脚本
```

## 快速开始

开发运行：

```bash
cargo run
```

发布构建 / 打包安装包：

```bash
cargo build --release
# 或使用 Tauri CLI 打包：
# cargo tauri build
```

> 提示：前端是纯静态文件，修改 `ui/` 下的内容后重新运行即可生效。

## macOS 安装说明（重要）

从 GitHub Release 下载 `.dmg` 安装后，若双击提示 **「"Clock" 已损坏，无法打开，您应该将它移到废纸篓」**，
这**不是文件真的损坏**，而是 macOS Gatekeeper 对「未经 Apple 公证的应用」的拦截
（本项目默认未购买付费的 Apple 开发者证书做公证）。

解决办法：把 Clock 拖入「应用程序」后，在「终端」执行一次以下命令解除隔离属性即可正常打开：

```bash
xattr -cr /Applications/Clock.app
```

> Apple Silicon（M 系列）芯片请下载文件名带 `aarch64` 的 DMG；Intel 芯片请下载带 `x64` 的 DMG。
>
> 维护者若拥有付费 Apple Developer 账号，可在仓库的 GitHub Secrets 中配置
> `APPLE_SIGNING_IDENTITY`、`APPLE_CERTIFICATE`、`APPLE_CERTIFICATE_PASSWORD`、
> `APPLE_ID`、`APPLE_PASSWORD`、`APPLE_TEAM_ID`，发布流程会自动进行 Developer ID
> 签名与公证，用户即可直接双击打开、无需上面的命令。

## 使用说明

- 在右侧面板中可调整 `Face`、`Dial`、`Smooth hands`、`Show second hand` 等显示选项。
- 在 `COUNTDOWN` 区域输入 `HH:MM:SS` 后点击 `Start countdown` 创建倒计时。
- 当存在多个倒计时时，可点击条目切换在表盘上高亮显示的倒计时，也可删除指定倒计时。

## 许可证

本项目使用 [MIT License](./LICENSE)。
