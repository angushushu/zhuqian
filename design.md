# 竹签 (ZhuQian) - 设计文档

“竹签”是一款基于 Rust 开发的、具备实时标签高亮功能的文本编辑工具。它旨在提供极致的写作与阅读专注体验。

## 核心架构：全能核心 (Universal Core)

为了支持跨平台的一致性体验，项目采用了“全能核心”架构，将核心逻辑与 UI 渲染分离。

### 1. `zq-core` (脱耦逻辑层)
- **职责**：作为唯一的“策略来源”，负责所有非 UI 逻辑。
- **解析引擎**：基于正则的标签识别、基于 `pulldown-cmark` 的 Markdown 解析。
- **中间表示 (Agnostic IR)**：将解析结果输出为平台无关的 `StyledSpan` 序列，不再直接依赖 GUI 框架的类型。
- **数据模型**：定义全平台通用的 `HighlightRule` (高亮规则)、`ZqTheme` (主题) 和 `DisplayPrefs` (显示偏好)。
- **平台中立性**：使用抽象的颜色表示（如 RGB 数组），不依赖任何 GUI 框架（如 `egui`）。
- **多端分发**：直接作为 Rust 库使用，或通过 `wasm-pack` 编译为 WebAssembly 供 VS Code 插件调用。

### 2. 多端呈现层 (Rendering Layers / Adapters)

#### **独立桌面应用 (ZhuQian App)**
- **技术栈**：Rust + `eframe` / `egui`。
- **适配逻辑**：实现 `EguiAdapter`，将 `zq-core` 输出的 `StyledSpan` 转换为 `egui::text::LayoutJob`。
- **特色功能**：支持沉浸式背景图、多标签页管理。

#### **VS Code 插件 (ZhuQian Extension)**
- **技术栈**：TypeScript + WebAssembly。
- **适配逻辑**：调用 Wasm 核心，将 `StyledSpan` 转换为 `vscode.TextEditorDecorationType`，实现非破坏性的实时高亮。

---

## 核心功能清单

- [x] **多标签编辑**：支持多文件同时开启与切换。
- [x] **非破坏性高亮**：通过正则动态识别 `[]` 等标签并着色，不改动源文件。
- [x] **Markdown 渲染**：在实时编辑中叠加 Markdown 样式。
- [x] **专有格式 `.zq`**：支持在纯文本中嵌入显示偏好元数据。
- [x] **通用主题系统**：支持保存、加载和删除高亮主题。
- [x] **后台背景**：编辑器支持纯色或图片背景。
- [x] **统计信息**：实时统计字数、行数、标签数。
- [ ] **LaTeX 公式**：支持 `$` 和 `$$` 语法的高亮识别与轻量预览。
- [ ] **BibTeX 引用**：支持 `@` 引用标记的高亮，并能识别基础 `.bib` 条目。

## 未来展望

- **Workspace 模式**：完善 Rust 项目结构，将 `zq-core` 独立为子 crate。
- **学术生态**：集成基础学术辅助工具，实现 LaTeX 公式实时渲染。
- **VS Code 基座**：发布官方 VS Code 插件，实现双端同步体验。
- **多语言支持**：核心库内置中英文多语言包。
