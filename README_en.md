# ZhuQian (竹签)

[简体中文](./README.md) | [English]

**Clarity in Mind, Structure in Tags (胸有“竹”，心有“签”)**

A **structural writing and logic modeling tool** built with **Rust** and **eframe / egui**. Based on the [Semout (Semantic Outline)](./semout/README.md) standard, it supports drag-and-drop reorganization, logic visualization, and template-guided writing.

The name **ZhuQian (竹签)** is a portmanteau of the Chinese idiom **"胸有成竹"** (literally "having a plan in mind") and **"标签"** (Label/Tag). It aims to help authors organize logical threads through semantic tagging.

![Screenshot](screenshot.png)

## Core Features

### 1. Structural Writing & Drag-and-Drop (DND)
- **Semantic Outline**: Build the document skeleton in real-time using `[1.2.1]` style semantic labels.
- **Block Reorganization**: Reorder sections or nesting relationships directly in the outline via Drag & Drop; the body content is synchronized automatically.
- **Auto-Indexing**: Supports both explicit paths (e.g., `[s1.1]`) and relative paths (e.g., `[.1]`), with automatic identifier recalculation during moves.

### 2. Logic Topology Visualization
- **Relationship Modeling**: Establish logical connections between tags using the `|` operator.
- **Dynamic Mapping**: Visualize implicit narrative threads in real-time in the sidebar logic graph.

### 3. Template Guidance & Ghost Outline
- **Structural Constraints**: Define the "expected skeleton" of an article via `.zqt` templates.
- **Placeholder Guidance**: Display template requirements as "ghost" nodes in the outline to guide the author through functional modules.

### 4. Focused Layout (Zen Mode)
- **Centered Editor**: Supports a centered layout to reduce visual distraction.
- **Rendering Efficiency**: Built with a native Rust graphics engine for smooth scrolling and real-time highlighting.
- **Layout Optimization**: Sidebar utilizes hard truncation to present more hierarchical information in limited space.

### 5. Multi-Platform Architecture
- **zq-core**: A shared parsing and logic engine in Rust, ensuring 100% synchronization between the desktop client and the VS Code extension.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) toolchain installed (`rustc`, `cargo`).

### Build & Run
```bash
# 1. Compile and run
cargo run --package zhuqian

# 2. Build release version (.exe)
cargo build --release
# Output located at: target/release/zhuqian.exe
```

## Shortcuts

| Shortcut | Action |
|:---|:---|
| `Ctrl + S` | Save current file |
| `Ctrl + Shift + C` | Clean Copy (strip semantic labels) |
| `Ctrl + T` | Toggle Logic Topology view |

## Project Structure

| Path | Description |
|:---|:---|
| `zq-core/` | Cross-platform shared parsing engine (Rust) |
| `zq-desktop/` | Native desktop client based on egui |
| `semout/` | ZhuQian Semantic Outline protocol specification |
| `docs/` | Project documentation and [File Standard Definition](./docs/en/FILE_STANDARD.md) |

---
*ZhuQian: Structural writing based on semantic tagging.*
