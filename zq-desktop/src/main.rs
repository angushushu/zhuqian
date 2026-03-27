#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::path::PathBuf;
use std::fs;

mod parser;
use parser::{
    HighlightRule, ZqTheme, DisplayPrefs, Language, BaseTheme, LangStrings, get_strings,
    HighlightRuleSet, extract_headings, deserialize_zq_file, serialize_zq_file
};

fn c32(rgb: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

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



// ZqTheme and DisplayPrefs are now in zq_core

// ── Sidebar Mode ──
#[derive(PartialEq, Clone, Copy)]
enum SidebarMode { FileTree, Outline }

#[derive(PartialEq, Clone, Copy)]
enum SettingTab { General, Colors, Rules }

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
    settings_tab: SettingTab,
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
            show_settings: false, 
            settings_tab: SettingTab::General,
            show_left: true,
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

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text Files", &["txt", "md", "zq", "log"])
            .pick_file() {
            self.open_file(&path);
        }
    }

    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.files.clear();
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                            if matches!(ext, "txt" | "md" | "log" | "zq") { self.files.push(p); }
                        }
                    }
                }
            }
            self.files.sort();
        }
    }
    fn font_display_name(&self) -> &str {
        self.font_entries.iter()
            .find(|e| e.key == self.prefs.font_name)
            .map(|e| e.display.as_str())
            .unwrap_or(&self.prefs.font_name)
    }
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
        let t = &self.prefs.theme;
        let is_dark = t.base == BaseTheme::Dark;
        let c32 = |c: [u8; 3]| egui::Color32::from_rgb(c[0], c[1], c[2]);

        ctx.style_mut(|style| {
            let v = &mut style.visuals;
            v.panel_fill = c32(t.bg_side);
            v.window_fill = c32(t.bg_side);
            v.extreme_bg_color = c32(t.bg_main);
            v.selection.bg_fill = c32(t.accent_ui).linear_multiply(0.4);
            v.override_text_color = Some(c32(t.text_main));
            
            // ── Geek Aesthetic: Sharp Corners ──
            style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;
        });

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

        let s = get_strings(self.prefs.language);

        // ── Semantic Color Mapping ──
        let bg_main   = c32(self.prefs.theme.bg_main);
        let bg_side   = c32(self.prefs.theme.bg_side);
        let text_main = c32(self.prefs.theme.text_main);
        let text_side = c32(self.prefs.theme.text_side);
        let accent_ui = c32(self.prefs.theme.accent_ui);
        let accent_hl = c32(self.prefs.theme.accent_hl);
        
        // Derived colors
        let border_col = if is_dark { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(200) };
        let shadow_col = if is_dark { egui::Color32::from_black_alpha(100) } else { egui::Color32::from_black_alpha(30) };

        // ── UI Styles ──
        let mut visuals = if is_dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.noninteractive.bg_fill = bg_side;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text_side);
        visuals.widgets.inactive.bg_fill = bg_side;
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_side.gamma_multiply(0.6));
        visuals.widgets.hovered.bg_fill = accent_ui.gamma_multiply(0.15);
        visuals.widgets.active.bg_fill = accent_ui.gamma_multiply(0.25);
        visuals.selection.bg_fill = accent_ui.gamma_multiply(0.3);
        visuals.selection.stroke = egui::Stroke::new(1.0, accent_ui);
        ctx.set_visuals(visuals);

        let font_size = self.prefs.font_size;
        let font_family = egui::FontFamily::Name(self.prefs.font_name.clone().into());
        let editor_col = c32(self.prefs.theme.text_main);
        let panel_col  = c32(self.prefs.theme.text_side);
        let accent_col = c32(self.prefs.theme.accent_ui);

        // ── Menu Bar ──
        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::symmetric(8, 2)))
            .show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(s.file, |ui| {
                    if ui.button(s.open_file_btn).clicked() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    if ui.button(s.open_folder_btn).clicked() {
                        self.open_folder_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(s.save).clicked() {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            save_file(&tab.path, &tab.text, &self.prefs);
                            tab.is_dirty = false;
                        }
                        ui.close_menu();
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
                    ui.label(egui::RichText::new(name.as_ref()).color(accent_ui).size(12.0));
                    if tab.is_dirty {
                        ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(255, 100, 80)).size(10.0));
                    }
                }
            });
        });

        // ── Tab Bar ──
        egui::TopBottomPanel::top("tab_bar")
            .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::symmetric(4, 1)))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut close_idx: Option<usize> = None;
                for (i, tab) in self.tabs.iter().enumerate() {
                    let name = tab.path.file_name().unwrap_or_default().to_string_lossy();
                    let is_active = i == self.active_tab;
                    let label = if tab.is_dirty { format!("● {}", name) } else { name.into_owned() };
                    
                    // ── Geek Tab Style: Minimalist square blocks ──
                    let (bg_col, fg_col) = if is_active {
                        (accent_ui.gamma_multiply(0.15), accent_ui)
                    } else {
                        (egui::Color32::TRANSPARENT, text_side.gamma_multiply(0.7))
                    };

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        let resp = ui.add(egui::Button::new(egui::RichText::new(&label).size(12.0).color(fg_col)).fill(bg_col));
                        if resp.clicked() { self.active_tab = i; }
                        
                        let close_resp = ui.add(egui::Button::new(egui::RichText::new("×").size(11.0).color(fg_col)).fill(bg_col));
                        if close_resp.clicked() { close_idx = Some(i); }
                        
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("│").size(10.0).color(text_side.gamma_multiply(0.3)));
                        ui.add_space(8.0);
                    });
                }
                if let Some(idx) = close_idx { self.close_tab(idx); }
            });
        });

        // ── Status Bar ──
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::symmetric(8, 2)))
            .show(ctx, |ui| {
            let stats = parser::compute_stats(self.active_text());
            let status_color = text_side;
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
                .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::same(6)))
                .show(ctx, |ui| {
                if let Some(ref tex) = self.panel_bg_texture.clone() {
                    draw_bg_image(ui, tex, ui.max_rect());
                }

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sidebar_mode, SidebarMode::FileTree, egui::RichText::new(s.files).size(12.0).color(text_side));
                    ui.selectable_value(&mut self.sidebar_mode, SidebarMode::Outline, egui::RichText::new(s.outline).size(12.0).color(text_side));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let cc = accent_ui;
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

                                let color = if is_active { accent_ui }
                                    else if is_open { accent_ui.gamma_multiply(0.7) }
                                    else { text_side };

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
                                ui.label(egui::RichText::new(s.empty).size(11.0).color(text_side));
                            }
                            for (_ln, lv, title) in &headings {
                                let indent = "  ".repeat(lv.saturating_sub(1));
                                let sz = 13.0 - (*lv as f32 * 0.5);
                                let rt = egui::RichText::new(format!("{}{}", indent, title)).size(sz).color(text_side);
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
                .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::same(8)))
                .show(ctx, |ui| {
                if let Some(ref tex) = self.panel_bg_texture.clone() {
                    draw_bg_image(ui, tex, ui.max_rect());
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // ── Display Settings ──
                    ui.label(egui::RichText::new(s.display_settings).size(13.0).color(accent_col));
                    ui.add(egui::Separator::default().spacing(4.0));

                    // ── Settings Tabs ──
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.settings_tab, SettingTab::General, if self.prefs.language == Language::Zh { "通用" } else { "General" });
                        ui.selectable_value(&mut self.settings_tab, SettingTab::Colors, if self.prefs.language == Language::Zh { "配色" } else { "Theme" });
                        ui.selectable_value(&mut self.settings_tab, SettingTab::Rules, if self.prefs.language == Language::Zh { "规则" } else { "Rules" });
                    });
                    ui.add(egui::Separator::default().spacing(8.0));

                    match self.settings_tab {
                        SettingTab::General => {
                            // Font
                            ui.label(egui::RichText::new(s.font).size(11.0).color(text_side));
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

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(s.font_size).size(11.0).color(text_side));
                            ui.add(egui::Slider::new(&mut self.prefs.font_size, 10.0..=36.0).suffix("px"));

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.prefs.markdown_render, "");
                                ui.label(egui::RichText::new(s.markdown_render).size(11.0).color(text_side));
                            });

                            ui.add_space(16.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(s.language).size(11.0).color(text_side));
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
                        }
                        SettingTab::Colors => {
                             egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.add_space(8.0);
                                
                                macro_rules! color_row {
                                    ($label:expr, $field:expr) => {
                                        ui.horizontal(|ui| {
                                            let mut c = [$field[0] as f32/255.0, $field[1] as f32/255.0, $field[2] as f32/255.0];
                                            if ui.color_edit_button_rgb(&mut c).changed() {
                                                $field = [(c[0]*255.0) as u8, (c[1]*255.0) as u8, (c[2]*255.0) as u8];
                                            }
                                            ui.label(egui::RichText::new($label).size(11.0).color(text_side));
                                        });
                                    };
                                }

                                ui.add_space(8.0);

                                color_row!(s.editor_bg,          self.prefs.theme.bg_main);
                                color_row!(s.panel_bg,           self.prefs.theme.bg_side);
                                color_row!(s.editor_text_color,  self.prefs.theme.text_main);
                                color_row!(s.accent_ui_color,    self.prefs.theme.accent_ui);

                                ui.add_space(12.0);
                                ui.label(egui::RichText::new(s.bg_image).size(11.0).color(text_side));
                                ui.horizontal(|ui| {
                                    if ui.button(s.editor_bg_img).clicked() {
                                        if let Some(path) = rfd::FileDialog::new().pick_file() {
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

                                ui.add_space(16.0);
                                ui.label(egui::RichText::new(s.theme_section).size(11.0).color(accent_ui));
                                if !self.zq_themes.is_empty() {
                                    egui::ComboBox::from_id_salt("load_zqtheme")
                                        .selected_text(s.load_theme)
                                        .show_ui(ui, |ui| {
                                            for t in &self.zq_themes {
                                                if ui.selectable_label(false, &t.name).clicked() {
                                                    let name_backup = self.prefs.theme.name.clone();
                                                    self.prefs.theme = t.clone();
                                                    if self.prefs.theme.name.is_empty() { self.prefs.theme.name = name_backup; }
                                                    for r in &mut self.prefs.theme.highlight_rules { r.compile(); }
                                                }
                                            }
                                        });
                                }
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut self.new_theme_name).hint_text(s.theme_name).desired_width(120.0));
                                    if ui.button(s.save_theme).clicked() && !self.new_theme_name.is_empty() {
                                        let mut t = self.prefs.theme.clone();
                                        t.name = self.new_theme_name.clone();
                                        save_zq_theme(&t);
                                        self.zq_themes.push(t);
                                        self.new_theme_name.clear();
                                    }
                                });
                            });
                        }
                        SettingTab::Rules => {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label(egui::RichText::new(s.highlight_rules).size(11.0).color(accent_ui));
                                // Load rule set
                                if !self.hl_themes.is_empty() {
                                    let load_label = if self.prefs.language == Language::Zh { "载入规则集..." } else { "Load Rules..." };
                                    egui::ComboBox::from_id_salt("load_hl_theme").selected_text(load_label).show_ui(ui, |ui| {
                                        for hl in &self.hl_themes {
                                            if ui.selectable_label(false, &hl.name).clicked() {
                                                self.prefs.theme.highlight_rules = hl.rules.clone();
                                                for r in &mut self.prefs.theme.highlight_rules { r.compile(); }
                                            }
                                        }
                                    });
                                }
                                
                                ui.add_space(8.0);
                                let mut to_remove = None;
                                for i in 0..self.prefs.theme.highlight_rules.len() {
                                    ui.group(|ui| {
                                        let rule = &mut self.prefs.theme.highlight_rules[i];
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(&rule.name).size(11.0));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("×").clicked() { to_remove = Some(i); }
                                            });
                                        });
                                        ui.horizontal(|ui| {
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

                                ui.add_space(8.0);
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new(s.new_rule).size(11.0));
                                    ui.text_edit_singleline(&mut self.new_rule_name);
                                    ui.text_edit_singleline(&mut self.new_rule_pattern);
                                    ui.horizontal(|ui| {
                                        ui.color_edit_button_rgb(&mut self.new_rule_color);
                                        ui.checkbox(&mut self.new_rule_bold, "B");
                                        ui.checkbox(&mut self.new_rule_bg, "bg");
                                        if ui.button(s.add).clicked() && !self.new_rule_name.is_empty() {
                                            let c = [(self.new_rule_color[0]*255.0) as u8, (self.new_rule_color[1]*255.0) as u8, (self.new_rule_color[2]*255.0) as u8];
                                            let mut r = HighlightRule::new(&self.new_rule_name, &self.new_rule_pattern, c, self.new_rule_bold, self.new_rule_bg);
                                            r.compile();
                                            self.prefs.theme.highlight_rules.push(r);
                                        }
                                    });
                                });
                            });
                        }
                    }
                });
            });
        }

        // ── Central Panel: Editor ──
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(bg_main).inner_margin(egui::Margin::same(4)))
            .show(ctx, |ui| {
            if let Some(ref tex) = self.bg_texture.clone() {
                draw_bg_image(ui, tex, ui.max_rect());
            }

            if self.tabs.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(s.open_file).size(16.0).color(text_main));
                });
                return;
            }

            let rules = self.prefs.theme.highlight_rules.clone();
            let ff = font_family.clone();

            let mut layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                let spans = parser::parse_markdown_to_spans(
                    string.as_str(), 
                    &rules, 
                    [accent_hl.r(), accent_hl.g(), accent_hl.b()],
                    [text_main.r(), text_main.g(), text_main.b()]
                );
                let mut job = parser::render_to_job(string.as_str(), &spans, font_size, ff.clone(), text_main);
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
                            .text_color(text_main)
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

// ── Restore Helper Functions ──

fn load_file(path: &PathBuf) -> (String, DisplayPrefs) {
    let raw = fs::read_to_string(path).unwrap_or_default();
    if path.extension().and_then(|s| s.to_str()) == Some("zq") {
        let (body, prefs) = deserialize_zq_file(&raw);
        return (body, prefs.unwrap_or_default());
    }
    (raw, DisplayPrefs::default())
}

fn save_file(path: &PathBuf, text: &str, prefs: &DisplayPrefs) {
    if path.extension().and_then(|s| s.to_str()) == Some("zq") {
        let data = serialize_zq_file(text, prefs);
        let _ = fs::write(path, data);
    } else {
        let _ = fs::write(path, text);
    }
}

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
    let mut themes = vec![ZqTheme::new_light(), ZqTheme::new_dark()];
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
                        t.name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unnamed").to_string();
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
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("zqrules"))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(mut hl) = serde_json::from_str::<HighlightRuleSet>(&data) {
                    if hl.name.is_empty() {
                        hl.name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unnamed").to_string();
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

// ── Extract headings moved to core ──


// ── Drawing background image ──

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
