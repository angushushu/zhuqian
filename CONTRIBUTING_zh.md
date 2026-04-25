# 贡献指南

[English](./CONTRIBUTING.md) | [简体中文]

感谢你对贡献 [ZhuQian (竹签)](https://github.com/angushushu/zhuqian) 项目的兴趣！我们欢迎针对核心引擎、桌面端以及 VS Code 插件的各类贡献。

## 项目结构

*   **`zq-core/`**: 基于 Rust 的解析与逻辑引擎。
*   **`zq-desktop/`**: 基于 egui 的桌面应用程序。
*   **`zq-vscode/`**: VS Code 插件。
*   **`semout/`**: 语义大纲标准与各语言 SDK。

## 如何贡献

1.  **报告 Bug**：打开 Issue 并提供清晰的描述及复现步骤。
2.  **提议新功能**：使用 Issue 追踪器提出你的创意。
3.  **提交 Pull Requests**：
    *   针对 **Semout 标准**，请参阅 [Semout 贡献指南](./semout/CONTRIBUTING_zh.md)。
    *   针对 **ZhuQian 实现**，请确保代码符合 Rust/TypeScript 最佳实践，并在适用时包含测试。

## 开发原则

*   **逻辑优先**：优先保证写作体验的结构化与逻辑完整性。
*   **东方美学**：保持“竹风”极简的设计语言。
*   **多端一致性**：确保通过 `zq-core` 实现桌面端与 VS Code 插件的功能同步。

## 开源协议

通过贡献代码，你同意你的贡献将遵循项目的 **GNU GPL v3**（针对应用程序）和 **MIT**（针对 semout 标准）协议。
