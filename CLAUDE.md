# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

(ZhuQian) is a Rust/eframe aesthetic writing tool with customizable label highlighting. It supports `.txt`, `.md`, `.log`, and `.zq` (proprietary format with embedded display preferences) files.

## Build Commands

```bash
cargo build          # Debug build
cargo build --release # Release build
cargo run            # Run the application
```

## Features

- **Multi-tab editing** with dirty state tracking
- **Custom syntax highlighting** via regex rules (saved as themes)
- **Markdown rendering mode** with heading sizes, bold, italic, code, strikethrough
- **Background images** for editor and panels
- **Display presets** for saving/loading font and color settings
- **Document statistics** (lines, characters, words, labels)
- **File tree and outline** sidebar modes

## Architecture

**Two main modules:**

- `src/main.rs` — GUI application using eframe/egui. Contains:
  - `ZhuQianEditor` struct: main application state (tabs, files, fonts, preferences, themes, background textures)
  - Tab management with dirty-state tracking and auto-save on tab switch
  - Custom flat theme applied via `apply_theme()`
  - System font loading from `C:\Windows\Fonts`
  - Background image loading and rendering

- `src/parser.rs` — Text processing and highlighting:
  - `HighlightRule`: regex-based highlighting with foreground/background color support
  - `HighlightThemeData`: collection of rules, persisted to `themes.json`
  - `highlight()` / `highlight_markdown()`: builds egui `LayoutJob` for styled text rendering
  - `compute_stats()`: document statistics (chars, words, lines, labels)

**Key data flow:**
- `.zq` files store metadata (display prefs as JSON) between `---zq-meta---` and `---end-meta---` markers
- User themes saved to `themes.json`, display presets to `display_presets.json`
- Custom rules apply first, then markdown styling (if enabled) overlays on top
- Background images loaded on-demand via `image` crate

## Keyboard Shortcuts

- `Ctrl+S` — Save current file

## Platform

Should support all platforms.
