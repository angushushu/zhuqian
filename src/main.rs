#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::path::PathBuf;
use std::fs;

mod parser;
use parser::HighlightRule;

// ── Language Support ──
#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Default)]
enum Language { #[default] Zh, En }

struct LangStrings {
    app_title: &'static str,
    file: &'static str,
    save: &'static str,
    save_as: &'static str,
    close_tab: &'static str,
    exit: &'static str,
    view: &'static str,
    hide_sidebar: &'static str,
    show_sidebar: &'static str,
    hide_settings: &'static str,
    show_settings: &'static str,
    files: &'static str,
    outline: &'static str,
    display_settings: &'static str,
    font: &'static str,
    font_size: &'static str,
    editor_bg: &'static str,
    panel_bg: &'static str,
    editor_text_color: &'static str,
    panel_text_color: &'static str,
    accent_color: &'static str,
    border_color: &'static str,
    close_btn_color: &'static str,
    checkbox_color: &'static str,
    textbox_color: &'static str,
    tab_active_color: &'static str,
    tab_inactive_color: &'static str,
    menu_bg_color: &'static str,
    selected_file_color: &'static str,
    status_bar_color: &'static str,
    markdown_render: &'static str,
    theme_section: &'static str,
    load_theme: &'static str,
    save_theme: &'static str,
    delete_theme: &'static str,
    theme_name: &'static str,
    highlight_rules: &'static str,
    new_rule: &'static str,
    add: &'static str,
    bg_image: &'static str,
    editor_bg_img: &'static str,
    panel_bg_img: &'static str,
    clear: &'static str,
    language: &'static str,
    empty: &'static str,
    open_file: &'static str,
    lines: &'static str,
    chars: &'static str,
    words: &'static str,
    labels: &'static str,
}

impl LangStrings {
    fn zh() -> Self {
        Self {
            app_title: "竹签 ZhuQian",
            file: "文件",
            save: "  保存  Ctrl+S",
            save_as: "  另存为...",
            close_tab: "  关闭标签页",
            exit: "  退出",
            view: "查看",
            hide_sidebar: "  隐藏侧边栏",
            show_sidebar: "  显示侧边栏",
            hide_settings: "  隐藏设置",
            show_settings: "  显示设置",
            files: "文件",
            outline: "大纲",
            display_settings: "— 显示设置 —",
            font: "字体",
            font_size: "字号",
            editor_bg: "编辑器背景",
            panel_bg: "面板背景",
            editor_text_color: "编辑器文字",
            panel_text_color: "面板文字",
            accent_color: "强调色",
            border_color: "边框颜色",
            close_btn_color: "关闭按钮",
            checkbox_color: "复选框颜色",
            textbox_color: "输入框背景",
            tab_active_color: "活动标签",
            tab_inactive_color: "标签文字",
            menu_bg_color: "菜单背景",
            selected_file_color: "选中文件",
            status_bar_color: "状态栏",
            markdown_render: "Markdown 渲染",
            theme_section: "— 主题 —",
            load_theme: "载入主题...",
            save_theme: "保存主题",
            delete_theme: "删除主题",
            theme_name: "主题名",
            highlight_rules: "— 高亮规则 —",
            new_rule: "+ 新规则",
            add: "添加",
            bg_image: "背景图片",
            editor_bg_img: "编辑器背景...",
            panel_bg_img: "面板背景...",
            clear: "清除",
            language: "语言",
            empty: "(空)",
            open_file: "// 从侧边栏打开文件",
            lines: "行",
            chars: "字",
            words: "词",
            labels: "签",
        }
    }
    fn en() -> Self {
        Self {
            app_title: "ZhuQian Editor",
            file: "File",
            save: "  Save  Ctrl+S",
            save_as: "  Save As...",
            close_tab: "  Close Tab",
            exit: "  Exit",
            view: "View",
            hide_sidebar: "  Hide Sidebar",
            show_sidebar: "  Show Sidebar",
            hide_settings: "  Hide Settings",
            show_settings: "  Show Settings",
            files: "Files",
            outline: "Outline",
            display_settings: "— Display Settings —",
            font: "Font",
            font_size: "Font Size",
            editor_bg: "Editor BG",
            panel_bg: "Panel BG",
            editor_text_color: "Editor Text",
            panel_text_color: "Panel Text",
            accent_color: "Accent",
            border_color: "Border Color",
            close_btn_color: "Close Button",
            checkbox_color: "Checkbox Color",
            textbox_color: "Input BG",
            tab_active_color: "Active Tab",
            tab_inactive_color: "Tab Text",
            menu_bg_color: "Menu BG",
            selected_file_color: "Selected File",
            status_bar_color: "Status Bar",
            markdown_render: "Markdown Render",
            theme_section: "— Theme —",
            load_theme: "Load Theme...",
            save_theme: "Save Theme",
            delete_theme: "Delete Theme",
            theme_name: "Theme Name",
            highlight_rules: "— Highlight Rules —",
            new_rule: "+ New Rule",
            add: "Add",
            bg_image: "Background Image",
            editor_bg_img: "Editor BG...",
            panel_bg_img: "Panel BG...",
            clear: "Clear",
            language: "Language",
            empty: "(empty)",
            open_file: "// open a file from the sidebar",
            lines: "Ln",
            chars: "Ch",
            words: "Wd",
            labels: "Lb",
        }
    }
}

fn get_strings(lang: Language) -> LangStrings {
    match lang {
        Language::Zh => LangStrings::zh(),
        Language::En => LangStrings::en(),
    }
}

const ZQ_META_PREFIX: &str = "---zq-meta---\n";
const ZQ_META_SUFFIX: &str = "\n---end-meta---\n";

// ── Font entry: file key + display name ──
#[derive(Clone)]
struct FontEntry {
    key: String,
    display: String,
}

/// Scan Windows Fonts dir, validate, extract font names, register families.
fn load_system_fonts(fonts: &mut egui::FontDefinitions) -> Vec<FontEntry> {
    let font_dir = "C:\\Windows\\Fonts";
    let mut loaded: Vec<FontEntry> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(font_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext != "ttf" && ext != "ttc" && ext != "otf" { continue; }

            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let stem_lower = stem.to_lowercase();
            if stem_lower.ends_with("bd") || stem_lower.ends_with("bi")
                || (stem_lower.ends_with('b') && stem_lower.len() > 3)
                || (stem_lower.ends_with('i') && stem_lower.len() > 3)
                || stem_lower.ends_with("it")
                || (stem_lower.ends_with('l') && stem_lower.len() > 3)
                || stem_lower.ends_with("li")
                || (stem_lower.ends_with('z') && stem_lower.len() > 3) {
                continue;
            }

            if let Ok(data) = std::fs::read(&path) {
                let face = match ttf_parser::Face::parse(&data, 0) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let display_name = face.names()
                    .into_iter()
                    .filter(|n| n.name_id == ttf_parser::name_id::FAMILY)
                    .find_map(|n| n.to_string())
                    .unwrap_or_else(|| stem.clone());

                let key = stem.clone();
                fonts.font_data.insert(key.clone(), egui::FontData::from_owned(data).into());
                fonts.families.insert(
                    egui::FontFamily::Name(key.clone().into()),
                    vec![key.clone()],
                );
                loaded.push(FontEntry { key, display: display_name });
            }
        }
    }
    loaded.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));
    loaded.dedup_by(|a, b| a.display.to_lowercase() == b.display.to_lowercase());
    loaded
}

// ── HighlightRuleSet: named set of rules only (no colors) ──
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct HighlightRuleSet {
    name: String,
    rules: Vec<HighlightRule>,
}

// ── ZqTheme: unified color + highlight rules ──
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ZqTheme {
    #[serde(default)]
    name: String,
    // Colors
    bg_color: [u8; 3],
    panel_color: [u8; 3],
    editor_text: [u8; 3],
    panel_text: [u8; 3],
    accent: [u8; 3],
    #[serde(default = "default_button_color")]
    button_color: [u8; 3],
    #[serde(default = "default_close_btn_color")]
    close_btn_color: [u8; 3],
    #[serde(default = "default_checkbox_color")]
    checkbox_color: [u8; 3],
    #[serde(default = "default_textbox_color")]
    textbox_color: [u8; 3],
    tab_active_text: [u8; 3],
    tab_inactive_text: [u8; 3],
    menu_bg: [u8; 3],
    selected_file_text: [u8; 3],
    status_text: [u8; 3],
    // Highlight rules
    #[serde(default)]
    highlight_rules: Vec<HighlightRule>,
}

fn default_button_color()    -> [u8; 3] { [174, 201, 87] }
fn default_close_btn_color() -> [u8; 3] { [200, 100, 100] }
fn default_checkbox_color()  -> [u8; 3] { [174, 201, 87] }
fn default_textbox_color()  -> [u8; 3] { [212, 205, 155] }

impl Default for ZqTheme {
    fn default() -> Self {
        let mut rules = vec![
            HighlightRule::new("一层括号", r"\[[^\[\]]*\]", [230, 80, 80], false, true),
            HighlightRule::new("二层括号", r"\[\[[^\[\]]*\]\]", [80, 200, 80], false, true),
            HighlightRule::new("三层括号", r"\[\[\[[^\[\]]*\]\]\]", [80, 80, 230], false, true),
        ];
        for r in &mut rules { r.compile(); }
        Self {
            name: "竹签默认".into(),
            bg_color:            [226, 219, 172], // #E2DBAC
            panel_color:         [212, 205, 155],
            editor_text:         [60,  50,  30 ],
            panel_text:          [60,  50,  30 ],
            accent:              [174, 201, 87 ], // #AEC957
            button_color:        [174, 201, 87 ],
            close_btn_color:     [200, 100, 100],
            checkbox_color:      [174, 201, 87 ],
            textbox_color:       [212, 205, 155],
            tab_active_text:     [60,  50,  30 ],
            tab_inactive_text:   [120, 110, 80 ],
            menu_bg:             [212, 205, 155],
            selected_file_text:  [174, 201, 87 ],
            status_text:         [83,  120, 20 ],
            highlight_rules:     rules,
        }
    }
}

// ── Display Prefs (global + embedded in .zq files) ──
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct DisplayPrefs {
    font_size: f32,
    font_name: String,
    markdown_render: bool,
    #[serde(default)]
    bg_image_path: Option<String>,
    #[serde(default)]
    panel_bg_image_path: Option<String>,
    #[serde(default)]
    language: Language,
    #[serde(default)]
    theme: ZqTheme,
}

impl Default for DisplayPrefs {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            font_name: "times".into(),
            markdown_render: false,
            bg_image_path: None,
            panel_bg_image_path: None,
            language: Language::default(),
            theme: ZqTheme::default(),
        }
    }
}

// ── Sidebar Mode ──
#[derive(PartialEq, Clone, Copy)]
enum SidebarMode { FileTree, Outline }

// ── Tab ──
struct TabData {
    path: PathBuf,
    text: String,
    is_dirty: bool,
}

// ── App ──
struct ZhuQianEditor {
    tabs: Vec<TabData>,
    active_tab: usize,
    files: Vec<PathBuf>,
    font_entries: Vec<FontEntry>,
    font_search: String,
    prefs: DisplayPrefs,
    zq_themes: Vec<ZqTheme>,
    new_theme_name: String,
    hl_themes: Vec<HighlightRuleSet>,
    new_hl_theme_name: String,
    show_settings: bool,
    show_left: bool,
    sidebar_mode: SidebarMode,
    // Rule editor state
    new_rule_name: String,
    new_rule_pattern: String,
    new_rule_color: [f32; 3],
    new_rule_bold: bool,
    new_rule_bg: bool,
    // Background textures
    bg_texture: Option<egui::TextureHandle>,
    panel_bg_texture: Option<egui::TextureHandle>,
}

impl ZhuQianEditor {
    fn new(font_entries: Vec<FontEntry>) -> Self {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        if matches!(ext, "txt" | "md" | "log" | "zq") { files.push(p); }
                    }
                }
            }
        }
        files.sort();
        let mut tabs = Vec::new();
        let mut prefs = DisplayPrefs::default();
        if let Some(first) = files.first() {
            let (text, file_prefs) = load_file(first);
            prefs = file_prefs;
            tabs.push(TabData { path: first.clone(), text, is_dirty: false });
        }

        Self {
            tabs, active_tab: 0, files, font_entries,
            font_search: String::new(),
            prefs,
            zq_themes: load_zq_themes(),
            new_theme_name: String::new(),
            hl_themes: load_hl_themes(),
            new_hl_theme_name: String::new(),
            show_settings: false, show_left: true,
            sidebar_mode: SidebarMode::FileTree,
            new_rule_name: String::new(), new_rule_pattern: String::new(),
            new_rule_color: [1.0, 0.3, 0.3], new_rule_bold: false, new_rule_bg: true,
            bg_texture: None,
            panel_bg_texture: None,
        }
    }

    fn load_bg_image(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        let img = image::open(path).ok()?;
        let size = [img.width() as _, img.height() as _];
        let pixels: Vec<egui::Color32> = img.to_rgba8()
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        let color_image = egui::ColorImage { size, pixels, source_size: egui::Vec2::new(size[0] as f32, size[1] as f32) };
        Some(ctx.load_texture(path, color_image, egui::TextureOptions::default()))
    }

    fn open_file(&mut self, path: &PathBuf) {
        if let Some(idx) = self.tabs.iter().position(|t| &t.path == path) {
            self.active_tab = idx;
            return;
        }
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if tab.is_dirty { save_file(&tab.path, &tab.text, &self.prefs); }
        }
        let (text, file_prefs) = load_file(path);
        if path.extension().and_then(|e| e.to_str()) == Some("zq") {
            self.prefs = file_prefs;
            self.bg_texture = None;
            self.panel_bg_texture = None;
        }
        self.tabs.push(TabData { path: path.clone(), text, is_dirty: false });
        self.active_tab = self.tabs.len() - 1;
    }

    fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() { return; }
        if self.tabs[idx].is_dirty {
            save_file(&self.tabs[idx].path, &self.tabs[idx].text, &self.prefs);
        }
        self.tabs.remove(idx);
        if self.tabs.is_empty() { self.active_tab = 0; }
        else if self.active_tab >= self.tabs.len() { self.active_tab = self.tabs.len() - 1; }
    }

    fn active_text(&self) -> &str {
        self.tabs.get(self.active_tab).map(|t| t.text.as_str()).unwrap_or("")
    }

    fn font_display_name(&self) -> &str {
        self.font_entries.iter()
            .find(|e| e.key == self.prefs.font_name)
            .map(|e| e.display.as_str())
            .unwrap_or(&self.prefs.font_name)
    }
}

// ── File I/O ──

fn load_file(path: &PathBuf) -> (String, DisplayPrefs) {
    let raw = fs::read_to_string(path).unwrap_or_default();
    if path.extension().and_then(|s| s.to_str()) == Some("zq") {
        if let Some(rest) = raw.strip_prefix(ZQ_META_PREFIX) {
            if let Some(idx) = rest.find(ZQ_META_SUFFIX) {
                let meta = &rest[..idx];
                let body = &rest[idx + ZQ_META_SUFFIX.len()..];
                if let Ok(prefs) = serde_json::from_str::<DisplayPrefs>(meta) {
                    return (body.to_string(), prefs);
                }
            }
        }
    }
    (raw, DisplayPrefs::default())
}

fn save_file(path: &PathBuf, text: &str, prefs: &DisplayPrefs) {
    if path.extension().and_then(|s| s.to_str()) == Some("zq") {
        let meta = serde_json::to_string_pretty(prefs).unwrap_or_default();
        let _ = fs::write(path, format!("{}{}{}{}", ZQ_META_PREFIX, meta, ZQ_META_SUFFIX, text));
    } else {
        let _ = fs::write(path, text);
    }
}

// ── Theme file I/O (~/.zhuqian/) ──

fn zhuqian_dir() -> PathBuf {
    let mut d = dirs_or_home();
    d.push(".zhuqian");
    let _ = fs::create_dir_all(&d);
    d
}

fn dirs_or_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load_zq_themes() -> Vec<ZqTheme> {
    let dir = zhuqian_dir();
    let mut themes = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("zqtheme"))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(mut t) = serde_json::from_str::<ZqTheme>(&data) {
                    if t.name.is_empty() {
                        t.name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unnamed")
                            .to_string();
                    }
                    for r in &mut t.highlight_rules { r.compile(); }
                    themes.push(t);
                }
            }
        }
    }
    themes
}

fn save_zq_theme(theme: &ZqTheme) {
    let mut path = zhuqian_dir();
    let fname = theme.name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    path.push(format!("{}.zqtheme", fname));
    if let Ok(d) = serde_json::to_string_pretty(theme) {
        let _ = fs::write(path, d);
    }
}

fn delete_zq_theme(name: &str) {
    let mut path = zhuqian_dir();
    let fname = name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    path.push(format!("{}.zqtheme", fname));
    let _ = fs::remove_file(path);
}

fn load_hl_themes() -> Vec<HighlightRuleSet> {
    let dir = zhuqian_dir();
    let mut sets = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("zqrules"))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(mut hl) = serde_json::from_str::<HighlightRuleSet>(&data) {
                    if hl.name.is_empty() {
                        hl.name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unnamed")
                            .to_string();
                    }
                    for r in &mut hl.rules { r.compile(); }
                    sets.push(hl);
                }
            }
        }
    }
    sets
}

fn save_hl_theme(hl: &HighlightRuleSet) {
    let mut path = zhuqian_dir();
    let fname = hl.name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    path.push(format!("{}.zqrules", fname));
    if let Ok(d) = serde_json::to_string_pretty(hl) {
        let _ = fs::write(path, d);
    }
}

fn delete_hl_theme(name: &str) {
    let mut path = zhuqian_dir();
    let fname = name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    path.push(format!("{}.zqrules", fname));
    let _ = fs::remove_file(path);
}

// ── Heading extraction ──

fn extract_headings(text: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with('#') {
            let lv = t.chars().take_while(|c| *c == '#').count().min(6);
            let title = t[lv..].trim().to_string();
            if !title.is_empty() { out.push((i + 1, lv, title)); }
        }
    }
    out
}

// ── Draw background image (cover: fill + crop, preserve ratio) ──

fn draw_bg_image(ui: &egui::Ui, tex: &egui::TextureHandle, rect: egui::Rect) {
    let tex_size = tex.size_vec2();
    let scale = (rect.width() / tex_size.x).max(rect.height() / tex_size.y);
    let uv_w = rect.width() / (tex_size.x * scale);
    let uv_h = rect.height() / (tex_size.y * scale);
    let uv_x = (1.0 - uv_w) * 0.5;
    let uv_y = (1.0 - uv_h) * 0.5;
    let uv = egui::Rect::from_min_max(egui::pos2(uv_x, uv_y), egui::pos2(uv_x + uv_w, uv_y + uv_h));
    ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
}

// ── Theme application ──

fn c32(rgb: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

fn apply_theme(ctx: &egui::Context, prefs: &DisplayPrefs) {
    let t = &prefs.theme;
    let _bg    = c32(t.bg_color);
    let panel  = c32(t.panel_color);
    let accent = c32(t.accent);
    let btn      = c32(t.button_color);
    let _checkbox = c32(t.checkbox_color);
    let textbox  = c32(t.textbox_color);
    let btn_dim = egui::Color32::from_rgb(
        (t.button_color[0] as f32 * 0.8) as u8,
        (t.button_color[1] as f32 * 0.8) as u8,
        (t.button_color[2] as f32 * 0.8) as u8,
    );
    let btn_hover = egui::Color32::from_rgb(
        (t.button_color[0] as f32 * 0.9) as u8,
        (t.button_color[1] as f32 * 0.9) as u8,
        (t.button_color[2] as f32 * 0.9) as u8,
    );
    let menu_bg = c32(t.menu_bg);
    let panel_text = c32(t.panel_text);
    let cr = egui::CornerRadius::ZERO;

    ctx.style_mut(|style| {
        // Start from a clean light base so no dark-mode remnants slip through
        // (e.g. checkbox borders, ComboBox popups, color_edit backgrounds).
        style.visuals = egui::Visuals::light();
        let v = &mut style.visuals;
        v.panel_fill = panel;
        v.window_fill = menu_bg;
        v.extreme_bg_color = textbox;
        v.faint_bg_color = egui::Color32::from_rgb(
            ((t.bg_color[0] as f32) * 0.95) as u8,
            ((t.bg_color[1] as f32) * 0.95) as u8,
            ((t.bg_color[2] as f32) * 0.95) as u8,
        );
        v.override_text_color = Some(panel_text);

        v.window_corner_radius = cr;
        v.menu_corner_radius = cr;
        v.window_shadow = egui::epaint::Shadow::NONE;
        v.popup_shadow = egui::epaint::Shadow::NONE;

        let stroke_dim    = egui::epaint::Stroke::new(1.0, btn_dim);
        let stroke_btn    = egui::epaint::Stroke::new(1.0, btn);

        v.widgets.noninteractive.corner_radius = cr;
        v.widgets.noninteractive.bg_fill   = panel;
        v.widgets.noninteractive.fg_stroke = egui::epaint::Stroke::new(1.0, panel_text);
        v.widgets.noninteractive.bg_stroke = stroke_dim;

        v.widgets.inactive.corner_radius = cr;
        v.widgets.inactive.bg_fill   = btn;
        v.widgets.inactive.fg_stroke = egui::epaint::Stroke::new(1.0, panel_text);
        v.widgets.inactive.bg_stroke = stroke_dim;

        v.widgets.hovered.corner_radius = cr;
        v.widgets.hovered.bg_fill   = btn_hover;
        v.widgets.hovered.fg_stroke = stroke_btn;
        v.widgets.hovered.bg_stroke = stroke_btn;

        v.widgets.active.corner_radius = cr;
        v.widgets.active.bg_fill   = btn;
        v.widgets.active.fg_stroke = stroke_btn;
        v.widgets.active.bg_stroke = stroke_btn;

        v.widgets.open.corner_radius = cr;
        v.widgets.open.bg_fill   = btn_hover;
        v.widgets.open.fg_stroke = stroke_btn;
        v.widgets.open.bg_stroke = stroke_dim;

        v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(
            t.checkbox_color[0], t.checkbox_color[1], t.checkbox_color[2], 180,
        );
        v.selection.stroke = egui::epaint::Stroke::new(1.0, accent);

        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.window_margin = egui::Margin::same(4);
    });
}

fn main() -> eframe::Result {
    let icon = image::open("zq_icon.png").ok().map(|img| {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        egui::IconData { rgba: rgba.into_raw(), width: w, height: h }
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("竹签 ZhuQian")
        .with_inner_size([1100.0, 700.0]);
    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(std::sync::Arc::new(icon_data));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "竹签 ZhuQian",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            let font_entries = load_system_fonts(&mut fonts);
            // Add a CJK-capable font as fallback for ALL font families
            // (Proportional, Monospace, and every named family) so that
            // Chinese/Japanese/Korean glyphs render correctly regardless of
            // which font the user has selected.
            let cjk_key: Option<String> = ["msyh", "simsun", "simhei", "msjh", "mingliu"]
                .iter()
                .find(|k| fonts.font_data.contains_key(**k))
                .map(|k| k.to_string());
            if let Some(ref cjk) = cjk_key {
                let all_families: Vec<egui::FontFamily> = fonts.families.keys().cloned().collect();
                for family in all_families {
                    if let Some(list) = fonts.families.get_mut(&family) {
                        if !list.contains(cjk) {
                            list.push(cjk.clone());
                        }
                    }
                }
            }
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(ZhuQianEditor::new(font_entries)))
        }),
    )
}

impl eframe::App for ZhuQianEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, &self.prefs);
        let s = get_strings(self.prefs.language);

        // Load background images if paths are set but textures aren't loaded
        if self.bg_texture.is_none() {
            if let Some(ref path) = self.prefs.bg_image_path.clone() {
                self.bg_texture = Self::load_bg_image(ctx, path);
            }
        }
        if self.panel_bg_texture.is_none() {
            if let Some(ref path) = self.prefs.panel_bg_image_path.clone() {
                self.panel_bg_texture = Self::load_bg_image(ctx, path);
            }
        }

        // Keyboard shortcuts
        let mut save_requested = false;
        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                save_requested = true;
            }
        });
        if save_requested {
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                save_file(&tab.path, &tab.text, &self.prefs);
                tab.is_dirty = false;
            }
        }

        let bg         = c32(self.prefs.theme.bg_color);
        let panel_c    = c32(self.prefs.theme.panel_color);
        let font_size  = self.prefs.font_size;
        let editor_col = c32(self.prefs.theme.editor_text);
        let panel_col  = c32(self.prefs.theme.panel_text);
        let accent_col = c32(self.prefs.theme.accent);
        let font_family = egui::FontFamily::Name(self.prefs.font_name.clone().into());

        // ── Menu Bar ──
        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::NONE.fill(panel_c).inner_margin(egui::Margin::symmetric(8, 2)))
            .show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(s.file, |ui| {
                    if ui.button(s.save).clicked() {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            save_file(&tab.path, &tab.text, &self.prefs);
                            tab.is_dirty = false;
                        }
                        ui.close();
                    }
                    if ui.button(s.save_as).clicked() {
                        let save_info = self.tabs.get(self.active_tab).map(|t| {
                            (t.path.clone(), t.text.clone())
                        });
                        if let Some((old_path, text)) = save_info {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Text", &["txt", "md", "log", "zq"])
                                .set_file_name(old_path.file_name().unwrap_or_default().to_string_lossy().as_ref())
                                .save_file()
                            {
                                save_file(&path, &text, &self.prefs);
                                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                    tab.path = path.clone();
                                    tab.is_dirty = false;
                                }
                                if !self.files.contains(&path) {
                                    self.files.push(path);
                                    self.files.sort();
                                }
                            }
                        }
                        ui.close();
                    }
                    if ui.button(s.close_tab).clicked() {
                        let idx = self.active_tab;
                        self.close_tab(idx);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(s.exit).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button(s.view, |ui| {
                    if self.show_left {
                        if ui.button(s.hide_sidebar).clicked() { self.show_left = false; ui.close(); }
                    } else {
                        if ui.button(s.show_sidebar).clicked() { self.show_left = true; ui.close(); }
                    }
                    if self.show_settings {
                        if ui.button(s.hide_settings).clicked() { self.show_settings = false; ui.close(); }
                    } else {
                        if ui.button(s.show_settings).clicked() { self.show_settings = true; ui.close(); }
                    }
                });

                // File name in menu bar
                ui.separator();
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    let name = tab.path.file_name().unwrap_or_default().to_string_lossy();
                    ui.label(egui::RichText::new(name.as_ref()).color(accent_col).size(12.0));
                    if tab.is_dirty {
                        ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(255, 100, 80)).size(10.0));
                    }
                }
            });
        });

        // ── Tab Bar ──
        egui::TopBottomPanel::top("tab_bar")
            .frame(egui::Frame::NONE.fill(panel_c).inner_margin(egui::Margin::symmetric(4, 1)))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut close_idx: Option<usize> = None;
                for (i, tab) in self.tabs.iter().enumerate() {
                    let name = tab.path.file_name().unwrap_or_default().to_string_lossy();
                    let is_active = i == self.active_tab;
                    let label = if tab.is_dirty { format!("● {}", name) } else { name.into_owned() };
                    let color = if is_active { c32(self.prefs.theme.tab_active_text) }
                                else { c32(self.prefs.theme.tab_inactive_text) };
                    let rt = egui::RichText::new(&label).size(12.0).color(color);
                    ui.horizontal(|ui| {
                        if ui.selectable_label(is_active, rt).clicked() { self.active_tab = i; }
                        let close_color = c32(self.prefs.theme.close_btn_color);
                        if ui.add(egui::Button::new(egui::RichText::new("×").size(11.0)).fill(close_color)).clicked() { close_idx = Some(i); }
                    });
                    if i + 1 < self.tabs.len() {
                        ui.label(egui::RichText::new("│").size(10.0).color(c32(self.prefs.theme.tab_inactive_text)));
                    }
                }
                if let Some(idx) = close_idx { self.close_tab(idx); }
            });
        });

        // ── Status Bar ──
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::NONE.fill(panel_c).inner_margin(egui::Margin::symmetric(8, 2)))
            .show(ctx, |ui| {
            let stats = parser::compute_stats(self.active_text());
            let status_color = c32(self.prefs.theme.status_text);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!(
                    "{} {}  {} {}  {} {}  {} {}",
                    s.lines, stats.lines, s.chars, stats.chars, s.words, stats.words, s.labels, stats.labels
                )).size(11.0).color(status_color));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!(
                        "{}  {}px", self.font_display_name(), self.prefs.font_size as u32
                    )).size(11.0).color(status_color));
                });
            });
        });

        // ── Left Sidebar ──
        if self.show_left {
            egui::SidePanel::left("left_panel").resizable(true).default_width(200.0)
                .frame(egui::Frame::NONE.fill(panel_c).inner_margin(egui::Margin::same(6)))
                .show(ctx, |ui| {
                if let Some(ref tex) = self.panel_bg_texture.clone() {
                    draw_bg_image(ui, tex, ui.max_rect());
                }

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sidebar_mode, SidebarMode::FileTree, egui::RichText::new(s.files).size(12.0).color(panel_col));
                    ui.selectable_value(&mut self.sidebar_mode, SidebarMode::Outline, egui::RichText::new(s.outline).size(12.0).color(panel_col));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let cc = c32(self.prefs.theme.close_btn_color);
                        if ui.add(egui::Button::new(egui::RichText::new("×").size(11.0)).fill(cc)).clicked() { self.show_left = false; }
                    });
                });
                ui.add(egui::Separator::default().spacing(4.0));

                match self.sidebar_mode {
                    SidebarMode::FileTree => {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let file_list: Vec<PathBuf> = self.files.clone();
                            for path in &file_list {
                                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                                let is_active = self.tabs.get(self.active_tab).map(|t| &t.path == path).unwrap_or(false);
                                let is_open = self.tabs.iter().any(|t| &t.path == path);

                                let color = if is_active { c32(self.prefs.theme.selected_file_text) }
                                    else if is_open { egui::Color32::from_rgb(
                                        (self.prefs.theme.selected_file_text[0] as f32 * 0.75) as u8,
                                        (self.prefs.theme.selected_file_text[1] as f32 * 0.75) as u8,
                                        (self.prefs.theme.selected_file_text[2] as f32 * 0.75) as u8,
                                    )}
                                    else { panel_col };

                                if ui.selectable_label(is_active, egui::RichText::new(&name).size(12.0).color(color)).clicked() {
                                    self.open_file(&path.clone());
                                }
                            }
                        });
                    }
                    SidebarMode::Outline => {
                        let text = self.active_text().to_string();
                        let headings = extract_headings(&text);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if headings.is_empty() {
                                ui.label(egui::RichText::new(s.empty).size(11.0).color(panel_col));
                            }
                            for (_ln, lv, title) in &headings {
                                let indent = "  ".repeat(lv.saturating_sub(1));
                                let sz = 13.0 - (*lv as f32 * 0.5);
                                let rt = egui::RichText::new(format!("{}{}", indent, title)).size(sz).color(panel_col);
                                let _ = ui.selectable_label(false, rt);
                            }
                        });
                    }
                }
            });
        }

        // ── Right Settings Panel ──
        if self.show_settings {
            egui::SidePanel::right("settings_panel").resizable(true).default_width(290.0)
                .frame(egui::Frame::NONE.fill(panel_c).inner_margin(egui::Margin::same(8)))
                .show(ctx, |ui| {
                if let Some(ref tex) = self.panel_bg_texture.clone() {
                    draw_bg_image(ui, tex, ui.max_rect());
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // ── Display Settings ──
                    ui.label(egui::RichText::new(s.display_settings).size(13.0).color(accent_col));
                    ui.add(egui::Separator::default().spacing(4.0));

                    // Font
                    ui.label(egui::RichText::new(s.font).size(11.0).color(panel_col));
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.font_search)
                            .hint_text(if self.prefs.language == Language::Zh { "搜索字体..." } else { "Search font..." })
                            .desired_width(160.0));
                    });
                    let search_lower = self.font_search.to_lowercase();
                    let current_display = self.font_display_name().to_string();
                    egui::ComboBox::from_id_salt("font_sel")
                        .selected_text(&current_display)
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                                for fe in &self.font_entries.clone() {
                                    if !search_lower.is_empty() && !fe.display.to_lowercase().contains(&search_lower) { continue; }
                                    if ui.selectable_label(self.prefs.font_name == fe.key, &fe.display).clicked() {
                                        self.prefs.font_name = fe.key.clone();
                                    }
                                }
                            });
                        });

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(s.font_size).size(11.0).color(panel_col));
                    ui.add(egui::Slider::new(&mut self.prefs.font_size, 10.0..=36.0).suffix("px"));

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.prefs.markdown_render, "");
                        ui.label(egui::RichText::new(s.markdown_render).size(11.0).color(panel_col));
                    });

                    // ── Color Pickers ──
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("— 颜色 Colors —").size(13.0).color(accent_col));
                    ui.add(egui::Separator::default().spacing(4.0));

                    macro_rules! color_row {
                        ($label:expr, $field:expr) => {
                            ui.horizontal(|ui| {
                                let mut c = [$field[0] as f32/255.0, $field[1] as f32/255.0, $field[2] as f32/255.0];
                                if ui.color_edit_button_rgb(&mut c).changed() {
                                    $field = [(c[0]*255.0) as u8, (c[1]*255.0) as u8, (c[2]*255.0) as u8];
                                }
                                ui.label(egui::RichText::new($label).size(11.0).color(panel_col));
                            });
                        };
                    }

                    color_row!(s.editor_bg,          self.prefs.theme.bg_color);
                    color_row!(s.panel_bg,            self.prefs.theme.panel_color);
                    color_row!(s.editor_text_color,   self.prefs.theme.editor_text);
                    color_row!(s.panel_text_color,    self.prefs.theme.panel_text);
                    color_row!(s.accent_color,        self.prefs.theme.accent);
                    color_row!(s.border_color,        self.prefs.theme.button_color);
                    color_row!(s.close_btn_color,     self.prefs.theme.close_btn_color);
                    color_row!(s.checkbox_color,      self.prefs.theme.checkbox_color);
                    color_row!(s.textbox_color,       self.prefs.theme.textbox_color);
                    color_row!(s.tab_active_color,    self.prefs.theme.tab_active_text);
                    color_row!(s.tab_inactive_color,  self.prefs.theme.tab_inactive_text);
                    color_row!(s.menu_bg_color,       self.prefs.theme.menu_bg);
                    color_row!(s.selected_file_color, self.prefs.theme.selected_file_text);
                    color_row!(s.status_bar_color,    self.prefs.theme.status_text);

                    // ── Background Images ──
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(s.bg_image).size(11.0).color(panel_col));
                    ui.horizontal(|ui| {
                        if ui.button(s.editor_bg_img).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "gif"])
                                .pick_file()
                            {
                                if let Some(p) = path.to_str() {
                                    self.prefs.bg_image_path = Some(p.to_string());
                                    self.bg_texture = Self::load_bg_image(ctx, p);
                                }
                            }
                        }
                        if self.prefs.bg_image_path.is_some() && ui.button(s.clear).clicked() {
                            self.prefs.bg_image_path = None;
                            self.bg_texture = None;
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button(s.panel_bg_img).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "gif"])
                                .pick_file()
                            {
                                if let Some(p) = path.to_str() {
                                    self.prefs.panel_bg_image_path = Some(p.to_string());
                                    self.panel_bg_texture = Self::load_bg_image(ctx, p);
                                }
                            }
                        }
                        if self.prefs.panel_bg_image_path.is_some() && ui.button(s.clear).clicked() {
                            self.prefs.panel_bg_image_path = None;
                            self.panel_bg_texture = None;
                        }
                    });

                    // Language
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(s.language).size(11.0).color(panel_col));
                        egui::ComboBox::from_id_salt("lang_sel")
                            .selected_text(match self.prefs.language { Language::Zh => "中文", Language::En => "English" })
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(self.prefs.language == Language::Zh, "中文").clicked() {
                                    self.prefs.language = Language::Zh;
                                }
                                if ui.selectable_label(self.prefs.language == Language::En, "English").clicked() {
                                    self.prefs.language = Language::En;
                                }
                            });
                    });

                    // ── Theme Save/Load ──
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(s.theme_section).size(13.0).color(accent_col));
                    ui.add(egui::Separator::default().spacing(4.0));

                    // Load existing theme
                    if !self.zq_themes.is_empty() {
                        let themes_snap: Vec<ZqTheme> = self.zq_themes.clone();
                        egui::ComboBox::from_id_salt("load_zqtheme")
                            .selected_text(s.load_theme)
                            .width(ui.available_width() - 4.0)
                            .show_ui(ui, |ui| {
                                for t in &themes_snap {
                                    if ui.selectable_label(false, &t.name).clicked() {
                                        let name_backup = self.prefs.theme.name.clone();
                                        self.prefs.theme = t.clone();
                                        if self.prefs.theme.name.is_empty() {
                                            self.prefs.theme.name = name_backup;
                                        }
                                        // recompile rules
                                        for r in &mut self.prefs.theme.highlight_rules { r.compile(); }
                                    }
                                }
                            });
                    }

                    // Save current as theme
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.new_theme_name)
                            .hint_text(s.theme_name)
                            .desired_width(140.0));
                        if ui.button(s.save_theme).clicked() && !self.new_theme_name.is_empty() {
                            let mut t = self.prefs.theme.clone();
                            t.name = self.new_theme_name.clone();
                            save_zq_theme(&t);
                            if let Some(existing) = self.zq_themes.iter_mut().find(|x| x.name == t.name) {
                                *existing = t;
                            } else {
                                self.zq_themes.push(t);
                            }
                            self.new_theme_name.clear();
                        }
                    });

                    // Delete current theme from disk
                    let cur_name = self.prefs.theme.name.clone();
                    if !cur_name.is_empty() && self.zq_themes.iter().any(|t| t.name == cur_name) {
                        if ui.small_button(s.delete_theme).clicked() {
                            delete_zq_theme(&cur_name);
                            self.zq_themes.retain(|t| t.name != cur_name);
                        }
                    }

                    // ── Highlight Rules ──
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(s.highlight_rules).size(13.0).color(accent_col));
                    ui.add(egui::Separator::default().spacing(4.0));

                    // Load rule set
                    if !self.hl_themes.is_empty() {
                        let hl_snap = self.hl_themes.clone();
                        let load_label = if self.prefs.language == Language::Zh { "载入规则集..." } else { "Load Rules..." };
                        egui::ComboBox::from_id_salt("load_hl_theme")
                            .selected_text(load_label)
                            .width(ui.available_width() - 4.0)
                            .show_ui(ui, |ui| {
                                for hl in &hl_snap {
                                    if ui.selectable_label(false, &hl.name).clicked() {
                                        self.prefs.theme.highlight_rules = hl.rules.clone();
                                        for r in &mut self.prefs.theme.highlight_rules { r.compile(); }
                                    }
                                }
                            });
                    }

                    // Save / delete rule set
                    ui.horizontal(|ui| {
                        let hint = if self.prefs.language == Language::Zh { "规则集名" } else { "Rule set name" };
                        let save_lbl = if self.prefs.language == Language::Zh { "保存" } else { "Save" };
                        ui.add(egui::TextEdit::singleline(&mut self.new_hl_theme_name)
                            .hint_text(hint)
                            .desired_width(130.0));
                        if ui.button(save_lbl).clicked() && !self.new_hl_theme_name.is_empty() {
                            let hl = HighlightRuleSet {
                                name: self.new_hl_theme_name.clone(),
                                rules: self.prefs.theme.highlight_rules.clone(),
                            };
                            save_hl_theme(&hl);
                            if let Some(ex) = self.hl_themes.iter_mut().find(|x| x.name == hl.name) {
                                *ex = hl;
                            } else {
                                self.hl_themes.push(hl);
                            }
                            self.new_hl_theme_name.clear();
                        }
                        let del_lbl = if self.prefs.language == Language::Zh { "删除" } else { "Del" };
                        if !self.new_hl_theme_name.is_empty()
                            && self.hl_themes.iter().any(|x| x.name == self.new_hl_theme_name)
                            && ui.button(del_lbl).clicked()
                        {
                            delete_hl_theme(&self.new_hl_theme_name.clone());
                            self.hl_themes.retain(|x| x.name != self.new_hl_theme_name);
                            self.new_hl_theme_name.clear();
                        }
                    });

                    ui.add_space(4.0);
                    let mut to_remove: Option<usize> = None;
                    let rules_snap_len = self.prefs.theme.highlight_rules.len();
                    for i in 0..rules_snap_len {
                        ui.group(|ui| {
                            let rule = &mut self.prefs.theme.highlight_rules[i];
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&rule.name).size(11.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let cc = c32(self.prefs.theme.close_btn_color);
                                    if ui.add(egui::Button::new(egui::RichText::new("×").size(11.0)).fill(cc)).clicked() { to_remove = Some(i); }
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("re:").size(10.0).color(panel_col));
                                if ui.text_edit_singleline(&mut rule.pattern).changed() { rule.compile(); }
                            });
                            ui.horizontal(|ui| {
                                let mut c = [rule.color[0] as f32/255.0, rule.color[1] as f32/255.0, rule.color[2] as f32/255.0];
                                ui.color_edit_button_rgb(&mut c);
                                rule.color = [(c[0]*255.0) as u8, (c[1]*255.0) as u8, (c[2]*255.0) as u8];
                                ui.checkbox(&mut rule.bold, "B");
                                ui.checkbox(&mut rule.is_background, "bg");
                            });
                        });
                    }
                    if let Some(idx) = to_remove { self.prefs.theme.highlight_rules.remove(idx); }

                    ui.add_space(4.0);
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(s.new_rule).size(11.0));
                        ui.text_edit_singleline(&mut self.new_rule_name);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("re:").size(10.0).color(panel_col));
                            ui.text_edit_singleline(&mut self.new_rule_pattern);
                        });
                        ui.horizontal(|ui| {
                            ui.color_edit_button_rgb(&mut self.new_rule_color);
                            ui.checkbox(&mut self.new_rule_bold, "B");
                            ui.checkbox(&mut self.new_rule_bg, "bg");
                            if ui.button(s.add).clicked() && !self.new_rule_name.is_empty() && !self.new_rule_pattern.is_empty() {
                                let c = [(self.new_rule_color[0]*255.0) as u8, (self.new_rule_color[1]*255.0) as u8, (self.new_rule_color[2]*255.0) as u8];
                                let mut r = HighlightRule::new(&self.new_rule_name, &self.new_rule_pattern, c, self.new_rule_bold, self.new_rule_bg);
                                r.compile();
                                self.prefs.theme.highlight_rules.push(r);
                                self.new_rule_name.clear();
                                self.new_rule_pattern.clear();
                            }
                        });
                    });
                });
            });
        }

        // ── Central Panel: Editor ──
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(bg).inner_margin(egui::Margin::same(4)))
            .show(ctx, |ui| {
            if let Some(ref tex) = self.bg_texture.clone() {
                draw_bg_image(ui, tex, ui.max_rect());
            }

            if self.tabs.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(s.open_file).size(16.0).color(editor_col));
                });
                return;
            }

            let rules = self.prefs.theme.highlight_rules.clone();
            let ff = font_family.clone();
            let use_md = self.prefs.markdown_render;

            let mut layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut job = if use_md {
                    parser::highlight_markdown(string.as_str(), &rules, font_size, ff.clone(), editor_col, accent_col)
                } else {
                    parser::highlight(string.as_str(), &rules, font_size, ff.clone(), editor_col)
                };
                job.wrap.max_width = wrap_width;
                ui.painter().layout_job(job)
            };

            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let resp = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut tab.text)
                            .frame(false)
                            .font(egui::FontId::new(font_size, font_family.clone()))
                            .text_color(editor_col)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    );
                    if resp.changed() { tab.is_dirty = true; }
                });
            }
        });
    }
}
