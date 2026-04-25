# ZhuQian (竹签) Labels for VS Code

[English](./README.md) | [简体中文]

ZhuQian Labels 是 [ZhuQian (竹签)](https://github.com/angushushu/zhuqian) 编辑器的官方 VS Code 插件——旨在为这款极具东方美学的竹风文本处理器提供无缝的辅助体验。

## 核心特性
- **极客极简美学**：同步 ZhuQian 设计系统，采用直角边框与“竹绿”品牌色调。
- **自定义强调色**：可在 VS Code 设置中直接配置 `accentColorUi`（界面元素）和 `accentColorHl`（标题与高亮）。
- **实时语法高亮**：基于 Wasm 高性能核心，通过正则对特定的 ZQ 文本格式进行飞速渲染。
- **动态标签渲染**：输入时自动识别并高亮语义标签（`[label]`）。

## 配置项
- `zhuqian.accentColorUi`: 主界面元素的自定义十六进制颜色（默认：竹绿）。
- `zhuqian.accentColorHl`: 标题与高亮区域的自定义十六进制颜色。
- `zhuqian.highlightRules`: (Alpha) 定义桌面端与 VS Code 共用的高亮正则规则。

## 架构设计
本插件由 **ZhuQian Universal Core** 驱动，核心逻辑采用 Rust 编写，并编译为 WebAssembly，以确保在 VS Code 内部获得原生级别的性能。

## 开源协议
GNU GPL v3
