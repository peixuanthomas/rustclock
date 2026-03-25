# Rust Clock

一个基于 `eframe/egui` 与 `chrono` 的桌面时钟应用，支持多种表盘风格、数字样式与倒计时功能。

## 功能特性

- 5 种时钟显示风格（`Classic hands`、`Luminous ticks`、`Triangle sweep`、`Orbit dots`、`Arc bands`）
- 3 种刻度数字样式（阿拉伯数字、罗马数字、无数字）
- 可切换秒针显示、平滑走针
- 内置倒计时（支持创建、选择、删除多个倒计时）
- 全屏显示与快捷键操作（`F11` 切换全屏，`Esc` 退出全屏）

## 运行环境

- Rust 2021 Edition
- 依赖：
  - `eframe = 0.33.3`（`glow` 特性）
  - `chrono = 0.4`

## 快速开始

```bash
cargo run
```

发布模式运行：

```bash
cargo run --release
```

## 使用说明

- 在右侧面板中可调整 `Face`、`Dial`、`Smooth hands`、`Show second hand` 等显示选项。
- 在 `COUNTDOWN` 区域输入 `HH:MM:SS` 后点击 `Start countdown` 创建倒计时。
- 当存在多个倒计时时，可点击条目切换在表盘上高亮显示的倒计时，也可删除指定倒计时。

## 许可证

本项目使用 [MIT License](./LICENSE)。
