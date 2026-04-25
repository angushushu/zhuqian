# ZhuQian Labels for VS Code

[English] | [简体中文](./README_zh.md)

ZhuQian Labels is the official VS Code companion for the ZhuQian Editor—the world's most elegant, bamboo-inspired text processor. 

## Features
- **Geek-Minimalist Aesthetics**: Synchronized with the ZhuQian design system, featuring Sharp Corners and Bamboo Green branding.
- **Custom Accent Colors**: Configure `accentColorUi` (for UI elements) and `accentColorHl` (for headings and highlights) directly in VS Code settings.
- **Real-Time Highlighting**: Fast, regex-based highlighting for specialized ZQ text formats, powered by a high-performance Wasm core.
- **Dynamic Label Rendering**: Automatically tracks and highlights labels (`[label]`) as you type.

## Settings
- `zhuqian.accentColorUi`: Custom hex color for primary UI accents (default: Bamboo Green).
- `zhuqian.accentColorHl`: Custom hex color for highlights and headings.
- `zhuqian.highlightRules`: (Alpha) Define shared highlight regex patterns across desktop and VS Code.

## Architecture
This extension is powered by the **ZhuQian Universal Core**, written in Rust and compiled to WebAssembly for native performance within VS Code.

## License
GNU GPL v3
