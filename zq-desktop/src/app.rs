use eframe::egui;
use std::path::PathBuf;
use std::fs;

use crate::parser::{ZqTheme, DisplayPrefs};
use crate::fonts::FontEntry;
use crate::theme_io::*;

/// Check if a file path is a supported ZhuQian file format.
/// Handles double-extension .zq.md as well as single extensions.
fn is_supported_file(p: &PathBuf) -> bool {
    // Check .zq.md double extension first
    if p.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".zq.md"))
        .unwrap_or(false)
    {
        return true;
    }
    // Check single extension
    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
        return matches!(ext, "txt" | "md" | "log" | "zq");
    }
    false
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) enum SplitPane { Left, Right }

#[derive(Clone, Debug)]
pub(crate) enum EditorCommand {
    NewFile, SaveCurrent, SaveAsCurrent, CopyClean, ToggleSidebar, ToggleSettings, ToggleSplitRight, ToggleCommandPalette, OpenFile, OpenFolder, Exit, ToggleLabels, ToggleZenMode
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum SidebarMode { FileTree, Outline, Semantic, LogicGraph }

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum AutocompleteMode { None, Category, Shortcode }

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum SettingTab { General, Colors, Labels, Dictionary }

pub(crate) struct TabData {
    pub path: Option<PathBuf>,
    pub text: String,
    pub is_dirty: bool,
}

pub(crate) struct ZhuQianEditor {
    pub tabs: Vec<TabData>,
    pub active_tab: usize,
    pub files: Vec<PathBuf>,
    pub font_entries: Vec<FontEntry>,
    pub font_search: String,
    pub prefs: DisplayPrefs,
    pub zq_themes: Vec<ZqTheme>,
    pub new_theme_name: String,

    pub show_settings: bool,
    pub show_cmd_palette: bool,
    pub cmd_query: String,
    pub settings_tab: SettingTab,
    pub show_left: bool,
    pub sidebar_mode: SidebarMode,

    pub active_pane: SplitPane,
    pub split_right_tab: Option<usize>,

    pub bg_texture: Option<egui::TextureHandle>,
    pub panel_bg_texture: Option<egui::TextureHandle>,
    pub scroll_to_line: Option<usize>,
    pub just_jumped: bool,
    // Autocomplete state
    pub ac_mode: AutocompleteMode,
    pub ac_search: String,
    pub ac_cursor: usize,
    pub ac_pos: Option<egui::Pos2>,
    
    // Label type editing state
    pub new_label_type_name: String,
    pub new_label_type_color: [f32; 3],
}

impl ZhuQianEditor {
    pub fn new(font_entries: Vec<FontEntry>) -> Self {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && is_supported_file(&p) { files.push(p); }
            }
        }
        files.sort();
        let mut tabs = Vec::new();
        let mut prefs = DisplayPrefs::default();
        if let Some(first) = files.first() {
            let (text, file_prefs) = load_file(first);
            prefs = file_prefs;
            tabs.push(TabData { path: Some(first.clone()), text, is_dirty: false });
        }

        Self {
            tabs, active_tab: 0, files, font_entries,
            font_search: String::new(),
            prefs,
            zq_themes: load_zq_themes(),
            new_theme_name: String::new(),
            show_settings: false,
            show_cmd_palette: false,
            cmd_query: String::new(),
            settings_tab: SettingTab::General,
            show_left: true,
            sidebar_mode: SidebarMode::FileTree,
            
            active_pane: SplitPane::Left,
            split_right_tab: None,

            bg_texture: None,
            // Background textures
            panel_bg_texture: None,
            scroll_to_line: None,
            just_jumped: false,
            
            ac_mode: AutocompleteMode::None,
            ac_search: String::new(),
            ac_cursor: 0,
            ac_pos: None,

            new_label_type_name: String::new(),
            new_label_type_color: [0.9, 0.3, 0.3],
        }
    }

    pub fn load_bg_image(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        let img = image::open(path).ok()?;
        let size = [img.width() as _, img.height() as _];
        let pixels: Vec<egui::Color32> = img.to_rgba8()
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        let color_image = egui::ColorImage { size, pixels, source_size: egui::Vec2::new(size[0] as f32, size[1] as f32) };
        Some(ctx.load_texture(path, color_image, egui::TextureOptions::default()))
    }

    pub fn open_file(&mut self, path: &PathBuf) {
        if let Some(idx) = self.tabs.iter().position(|t| t.path.as_ref() == Some(path)) {
            self.active_tab = idx;
            return;
        }
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if tab.is_dirty {
                if let Some(ref p) = tab.path {
                    save_file(p, &tab.text, &self.prefs);
                }
            }
        }
        let (text, file_prefs) = load_file(path);
        if path.extension().and_then(|e| e.to_str()) == Some("zq") {
            self.prefs = file_prefs;
            self.bg_texture = None;
            self.panel_bg_texture = None;
        }
        self.tabs.push(TabData { path: Some(path.clone()), text, is_dirty: false });
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn new_file(&mut self) {
        self.tabs.push(TabData {
            path: None,
            text: String::new(),
            is_dirty: false,
        });
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn save_current(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(ref path) = tab.path {
                save_file(path, &tab.text, &self.prefs);
                tab.is_dirty = false;
            } else {
                self.save_as_current();
            }
        }
    }

    pub fn save_as_current(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let old_name = tab.path.as_ref()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "untitled.zq.md".to_string());

            if let Some(path) = rfd::FileDialog::new()
                .add_filter("ZhuQian Markdown", &["zq.md", "md", "txt", "log", "zq"])
                .set_file_name(&old_name)
                .save_file()
            {
                save_file(&path, &tab.text, &self.prefs);
                tab.path = Some(path.clone());
                tab.is_dirty = false;
                if !self.files.contains(&path) {
                    self.files.push(path);
                    self.files.sort();
                }
            }
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() { return; }
        if self.tabs[idx].is_dirty {
            if let Some(ref p) = self.tabs[idx].path {
                save_file(p, &self.tabs[idx].text, &self.prefs);
            }
        }
        self.tabs.remove(idx);
        if self.tabs.is_empty() { self.active_tab = 0; }
        else if self.active_tab >= self.tabs.len() { self.active_tab = self.tabs.len() - 1; }
    }

    pub fn active_text(&self) -> &str {
        self.tabs.get(self.active_tab).map(|t| t.text.as_str()).unwrap_or("")
    }

    pub fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ZhuQian Markdown", &["zq.md", "md", "txt", "log", "zq"])
            .pick_file() {
            self.open_file(&path);
        }
    }

    pub fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.files.clear();
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() && is_supported_file(&p) { self.files.push(p); }
                }
            }
            self.files.sort();
        }
    }

    pub fn font_display_name(&self) -> &str {
        self.font_entries.iter()
            .find(|e| e.key == self.prefs.font_name)
            .map(|e| e.display.as_str())
            .unwrap_or(&self.prefs.font_name)
    }

    pub fn handle_command(&mut self, cmd: EditorCommand, ctx: &egui::Context) {
        match cmd {
            EditorCommand::NewFile => self.new_file(),
            EditorCommand::SaveCurrent => self.save_current(),
            EditorCommand::SaveAsCurrent => self.save_as_current(),
            EditorCommand::CopyClean => {
                let labels = crate::parser::parse_semantic_labels(self.active_text());
                let clean = crate::parser::strip_semantic_labels(self.active_text(), &labels);
                ctx.copy_text(clean);
            },
            EditorCommand::ToggleSidebar => self.show_left = !self.show_left,
            EditorCommand::ToggleSettings => self.show_settings = !self.show_settings,
            EditorCommand::ToggleCommandPalette => {
                self.show_cmd_palette = !self.show_cmd_palette;
                self.cmd_query.clear();
            },
            EditorCommand::OpenFile => self.open_file_dialog(),
            EditorCommand::OpenFolder => self.open_folder_dialog(),
            EditorCommand::Exit => ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close),
            EditorCommand::ToggleLabels => self.prefs.hide_labels = !self.prefs.hide_labels,
            EditorCommand::ToggleZenMode => self.prefs.zen_mode = !self.prefs.zen_mode,
            EditorCommand::ToggleSplitRight => {
                if self.split_right_tab.is_some() {
                    self.split_right_tab = None;
                    self.active_pane = SplitPane::Left;
                } else if self.tabs.len() > 1 {
                    let other = (0..self.tabs.len()).find(|&i| i != self.active_tab).unwrap_or(0);
                    if other != self.active_tab {
                        self.split_right_tab = Some(other);
                    }
                }
            }
        }
    }
}
