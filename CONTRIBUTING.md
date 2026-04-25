# Contributing to ZhuQian (竹签)

[English] | [简体中文](./CONTRIBUTING_zh.md)

Thank you for your interest in contributing to ZhuQian! We welcome contributions to any part of the project, from the core engine to the desktop client and VS Code extension.

## Project Structure

*   **`zq-core/`**: The Rust-based parsing and logic engine.
*   **`zq-desktop/`**: The egui-based desktop application.
*   **`zq-vscode/`**: The VS Code extension.
*   **`semout/`**: The Semantic Outline standard and SDKs.

## How to Contribute

1.  **Report Bugs**: Open an issue with a clear description and steps to reproduce.
2.  **Suggest Features**: Use the issue tracker to propose new ideas.
3.  **Pull Requests**: 
    *   For the **Semout standard**, please see the [Semout Contributing Guide](./semout/CONTRIBUTING.md).
    *   For the **ZhuQian implementation**, ensure your code follows Rust/TypeScript best practices and includes tests where applicable.

## Development Principles

*   **Logic First**: Prioritize the structural and logical integrity of the writing experience.
*   **Oriental Aesthetics**: Maintain the "Bamboo-inspired" minimalist design language.
*   **Cross-platform Consistency**: Ensure changes are synchronized between the desktop and VS Code via `zq-core`.

## License

By contributing, you agree that your contributions will be licensed under the project's **GNU GPL v3** (for the applications) and **MIT** (for the semout standard).
