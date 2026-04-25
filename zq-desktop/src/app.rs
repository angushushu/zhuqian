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
    NewFile, SaveCurrent, SaveAsCurrent, ToggleSidebar, ToggleSettings, ToggleSplitRight, ToggleCommandPalette, OpenFile, OpenFolder, Exit, ToggleLabels, ToggleZenMode, ToggleQuickNav, ShowHelp
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
    pub display_text: String,
    pub last_cleaned_text: String,
    pub scroll_target_y: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SidebarDragItem {
    Semantic(usize),        // Index in labels list
    Markdown(usize),        // Line number of heading
    File(std::path::PathBuf), // File path
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
    pub show_quick_nav: bool,
    pub quick_nav_query: String,
    pub show_help: bool,
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
    
    pub show_logic_overlay: bool,
    pub active_template: crate::parser::ZqTemplate,
    pub current_folder: Option<PathBuf>,
    pub node_positions: std::collections::HashMap<String, egui::Pos2>,
    pub scroll_to_byte: Option<usize>,
    pub cmd_cursor: usize,
    pub dragged_item: Option<SidebarDragItem>,
}

impl ZhuQianEditor {
    pub fn new(font_entries: Vec<FontEntry>) -> Self {
        let session = load_session().unwrap_or_default();
        
        let mut files = Vec::new();
        let folder = session.last_folder.clone().unwrap_or_else(|| PathBuf::from("."));
        if let Ok(entries) = fs::read_dir(&folder) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && is_supported_file(&p) { files.push(p); }
            }
        }
        files.sort();

        let mut tabs = Vec::new();
        for p in &session.last_opened_files {
            if p.exists() {
                let (text, _) = load_file(p);
                tabs.push(TabData {
                    path: Some(p.clone()),
                    text,
                    is_dirty: false,
                    display_text: String::new(),
                    last_cleaned_text: String::new(),
                    scroll_target_y: None,
                });
            }
        }

        // If no files in session, fall back to current directory files
        if tabs.is_empty() {
            if let Some(first) = files.first() {
                let (text, _) = load_file(first);
                tabs.push(TabData { path: Some(first.clone()), text, is_dirty: false, display_text: String::new(), last_cleaned_text: String::new(), scroll_target_y: None });
            }
        }

        let active_tab = session.active_tab.min(tabs.len().saturating_sub(1));
        Self {
            tabs,
            active_tab,
            files,
            font_entries,
            font_search: String::new(),
            prefs: session.global_prefs,
            zq_themes: load_zq_themes(),
            new_theme_name: String::new(),
            show_settings: false,
            show_cmd_palette: false,
            cmd_query: String::new(),
            show_quick_nav: false,
            quick_nav_query: String::new(),
            show_help: false,
            settings_tab: SettingTab::General,
            show_left: true,
            sidebar_mode: SidebarMode::FileTree,
            
            active_pane: SplitPane::Left,
            split_right_tab: None,

            bg_texture: None,
            panel_bg_texture: None,
            scroll_to_line: None,
            just_jumped: false,
            
            ac_mode: AutocompleteMode::None,
            ac_search: String::new(),
            ac_cursor: 0,
            ac_pos: None,

            new_label_type_name: String::new(),
            new_label_type_color: [0.9, 0.3, 0.3],

            show_logic_overlay: false,
            active_template: crate::parser::ZqTemplate::freeform(),
            current_folder: Some(folder),
            node_positions: std::collections::HashMap::new(),
            scroll_to_byte: None,
            cmd_cursor: 0,
            dragged_item: None,
        }
    }

    pub fn save_current_session(&self) {
        let session = ZqSession {
            last_opened_files: self.tabs.iter().filter_map(|t| t.path.clone()).collect(),
            last_folder: self.current_folder.clone(),
            active_tab: self.active_tab,
            global_prefs: self.prefs.clone(),
        };
        save_session(&session);
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
            // Check for template directive
            if let Some(tname) = crate::parser::parse_template_directive(&self.tabs[idx].text) {
                self.set_active_template_by_name(&tname);
            }
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
        
        // Auto-switch template
        if let Some(tname) = crate::parser::parse_template_directive(&text) {
            self.set_active_template_by_name(&tname);
        } else {
            self.active_template = crate::parser::ZqTemplate::freeform();
        }

        self.tabs.push(TabData { path: Some(path.clone()), text, is_dirty: false, display_text: String::new(), last_cleaned_text: String::new(), scroll_target_y: None });
        self.active_tab = self.tabs.len() - 1;
        self.save_current_session();
    }

    fn set_active_template_by_name(&mut self, name: &str) {
        for t in crate::parser::ZqTemplate::all_builtins() {
            if t.name == name {
                self.active_template = t;
                return;
            }
        }
    }

    pub fn new_file(&mut self) {
        self.tabs.push(TabData {
            path: None,
            text: String::new(),
            is_dirty: false,
            display_text: String::new(),
            last_cleaned_text: String::new(),
            scroll_target_y: None,
        });
        self.active_tab = self.tabs.len() - 1;
        self.save_current_session();
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
                self.save_current_session();
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
        self.save_current_session();
    }

    pub fn handle_structural_move(&mut self, item: SidebarDragItem, target_idx: usize, is_child: bool) {
        match item {
            SidebarDragItem::Semantic(from_idx) => {
                self.move_semantic_chunk(from_idx, target_idx, is_child);
            }
            SidebarDragItem::Markdown(from_line) => {
                self.move_markdown_chunk(from_line, target_idx, is_child);
            }
            SidebarDragItem::File(path) => {
                // To be implemented: File moving logic
                println!("Move file: {:?} to target index: {}", path, target_idx);
            }
        }
    }

    fn move_markdown_chunk(&mut self, from_line: usize, target_idx: usize, is_child: bool) {
        let tab = if let Some(t) = self.tabs.get_mut(self.active_tab) { t } else { return; };
        let headings = crate::parser::extract_headings(&tab.text);
        
        let from_info = headings.iter().find(|(ln, _, _)| *ln == from_line);
        if from_info.is_none() { return; }
        let (ln, lv, _) = *from_info.unwrap();
        
        let lines: Vec<_> = tab.text.split_inclusive('\n').collect();
        if ln > lines.len() { return; }
        
        let start_byte: usize = lines[..ln-1].iter().map(|s| s.len()).sum();
        
        let mut end_line = lines.len();
        for (h_ln, h_lv, _) in &headings {
            if *h_ln > ln && *h_lv <= lv {
                end_line = *h_ln - 1;
                break;
            }
        }
        let end_byte: usize = lines[..end_line].iter().map(|s| s.len()).sum();
        let chunk_text = tab.text[start_byte..end_byte.min(tab.text.len())].to_string();

        tab.text.drain(start_byte..end_byte.min(tab.text.len()));
        tab.is_dirty = true;

        let headings_after = crate::parser::extract_headings(&tab.text);
        let lines_after: Vec<_> = tab.text.split_inclusive('\n').collect();

        // Adjust target index if we moved from before the target
        let actual_target_idx = if headings.iter().position(|(h_ln, _, _)| *h_ln == from_line).unwrap_or(0) < target_idx {
            target_idx.saturating_sub(1)
        } else {
            target_idx
        };

        let insert_pos = if actual_target_idx >= headings_after.len() {
            tab.text.len()
        } else {
            let (t_ln, t_lv, _) = headings_after[actual_target_idx];
            if is_child {
                let mut chunk_end_ln = lines_after.len();
                for (h_ln, h_lv, _) in &headings_after {
                    if *h_ln > t_ln && *h_lv <= t_lv {
                        chunk_end_ln = *h_ln - 1;
                        break;
                    }
                }
                lines_after[..chunk_end_ln].iter().map(|s| s.len()).sum()
            } else {
                lines_after[..t_ln-1].iter().map(|s| s.len()).sum()
            }
        };

        tab.text.insert_str(insert_pos, &chunk_text);
    }

    fn move_semantic_chunk(&mut self, from_idx: usize, target_idx: usize, is_child: bool) {
        let tab = if let Some(t) = self.tabs.get_mut(self.active_tab) { t } else { return; };
        let labels = crate::parser::parse_semantic_labels(&tab.text);
        if from_idx >= labels.len() { return; }
        
        let re_label = regex::Regex::new(r"\[([^\]]+)\]").unwrap();
        let from_label = &labels[from_idx];
        let from_depth = from_label.depth;
        
        // 1. Identify "From Chunk" range
        let start_byte = from_label.start_byte;
        let mut end_byte = tab.text.len();
        for i in (from_idx + 1)..labels.len() {
            if labels[i].depth <= from_depth {
                end_byte = labels[i].start_byte;
                break;
            }
        }
        
        let chunk_text = tab.text[start_byte..end_byte].to_string();
        let mut final_chunk = chunk_text;
        
        // 2. Count labels in from_chunk
        let mut labels_in_chunk = 0;
        for i in from_idx..labels.len() {
            if i > from_idx && labels[i].depth <= from_depth { break; }
            labels_in_chunk += 1;
        }

        // 3. Remove from original position
        tab.text.drain(start_byte..end_byte);
        tab.is_dirty = true;
        
        // 4. Recalculate target position after drain
        let labels_after = crate::parser::parse_semantic_labels(&tab.text);
        
        let actual_target_idx = if from_idx < target_idx {
            target_idx.saturating_sub(labels_in_chunk)
        } else {
            target_idx
        };

        let insert_pos = if actual_target_idx >= labels_after.len() {
            tab.text.len()
        } else {
            let target_label = &labels_after[actual_target_idx];
            if is_child {
                // Determine new path for the chunk (Parent.NextIndex)
                let parent_path = &target_label.category;
                let mut next_child_idx = 1;
                
                // Scan current children to find the next index
                for i in (target_idx + 1)..labels_after.len() {
                    if labels_after[i].depth <= target_label.depth { break; }
                    if labels_after[i].depth == target_label.depth + 1 {
                        next_child_idx += 1;
                    }
                }
                
                // Update the FIRST label in the chunk, preserving internal description if it exists
                let re_tag_full = regex::Regex::new(r"\[([^\]\-]+)(?:-([^\]]*))?\]").unwrap();
                if let Some(caps) = re_tag_full.captures(&final_chunk) {
                    let old_desc = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                    let new_tag = if old_desc.is_empty() {
                        format!("[{}.{}]", parent_path, next_child_idx)
                    } else {
                        format!("[{}.{}-{}]", parent_path, next_child_idx, old_desc)
                    };
                    // Use replace with limit 1 to only touch the root label
                    final_chunk = re_tag_full.replace(&final_chunk, new_tag.as_str()).to_string();
                }

                // Find end of target chunk (including its children)
                let target_depth = target_label.depth;
                let mut chunk_end = tab.text.len();
                for i in (target_idx + 1)..labels_after.len() {
                    if labels_after[i].depth <= target_depth {
                        chunk_end = labels_after[i].start_byte;
                        break;
                    }
                }
                chunk_end
            } else {
                target_label.start_byte
            }
        };
        
        tab.text.insert_str(insert_pos, &final_chunk);
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
            self.current_folder = Some(path.clone());
            self.files.clear();
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() && is_supported_file(&p) { 
                        self.files.push(p); 
                    }
                }
            }
            self.files.sort();
            self.save_current_session();
        }
    }

    pub fn font_display_name(&self) -> &str {
        self.font_entries.iter()
            .find(|e| e.key == self.prefs.font_name)
            .map(|e| e.display.as_str())
            .unwrap_or(&self.prefs.font_name)
    }

    pub fn insert_text_at_end(&mut self, text: &str) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.text.push_str(text);
            tab.is_dirty = true;
        }
    }

    pub fn delete_block(&mut self, start: usize, _end: usize) {
        let (b_start, b_end) = if let Some(tab) = self.tabs.get(self.active_tab) {
            calculate_block_range(&tab.text, start)
        } else { return; };

        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.text.replace_range(b_start..b_end, "");
            tab.is_dirty = true;
        }
    }

    pub fn move_block_up(&mut self, start: usize, _end: usize) {
        let (b_start, b_end, prev_start) = if let Some(tab) = self.tabs.get(self.active_tab) {
            let (bs, be) = calculate_block_range(&tab.text, start);
            if bs == 0 { return; }
            let prev_markers = get_structure_markers(&tab.text[..bs]);
            if let Some(ps) = prev_markers.last().map(|m| m.0) {
                (bs, be, Some(ps))
            } else { (bs, be, None) }
        } else { return; };

        if let Some(ps) = prev_start {
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                let block_content = tab.text[b_start..b_end].to_string();
                tab.text.replace_range(b_start..b_end, "");
                tab.text.insert_str(ps, &block_content);
                tab.is_dirty = true;
            }
        }
    }

    pub fn move_block_down(&mut self, start: usize, _end: usize) {
        let (b_start, b_end, next_end) = if let Some(tab) = self.tabs.get(self.active_tab) {
            let (bs, be) = calculate_block_range(&tab.text, start);
            if be >= tab.text.len() { return; }
            
            let next_markers = get_structure_markers(&tab.text);
            let mut ne = None;
            for (m_start, _) in next_markers {
                if m_start > be {
                    let (_, next_e) = calculate_block_range(&tab.text, m_start);
                    ne = Some(next_e);
                    break;
                }
            }
            (bs, be, ne)
        } else { return; };

        if let Some(ne) = next_end {
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                let block_content = tab.text[b_start..b_end].to_string();
                tab.text.insert_str(ne, &block_content);
                tab.text.replace_range(b_start..b_end, "");
                tab.is_dirty = true;
            }
        }
    }

    pub fn handle_command(&mut self, cmd: EditorCommand, ctx: &egui::Context) {
        match cmd {
            EditorCommand::NewFile => self.new_file(),
            EditorCommand::SaveCurrent => self.save_current(),
            EditorCommand::SaveAsCurrent => self.save_as_current(),
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
            EditorCommand::ToggleQuickNav => {
                self.show_quick_nav = !self.show_quick_nav;
                self.quick_nav_query.clear();
            }
            EditorCommand::ShowHelp => {
                self.show_help = !self.show_help;
            }
        }
        self.save_current_session();
    }
}

fn calculate_block_range(text: &str, start: usize) -> (usize, usize) {
    let markers = get_structure_markers(text);
    let mut end = text.len();
    for (m_start, _) in markers {
        if m_start > start {
            end = m_start;
            break;
        }
    }
    (start, end)
}

fn get_structure_markers(text: &str) -> Vec<(usize, String)> {
    let mut markers = Vec::new();
    // Labels
    for l in crate::parser::parse_semantic_labels(text) {
        markers.push((l.start_byte, format!("[{}]", l.category)));
    }
    // Headings (approximate byte index from line number)
    let lines: Vec<&str> = text.lines().collect();
    for (ln, _, title) in crate::parser::extract_headings(text) {
        let mut byte_idx = 0;
        for line in lines.iter().take(ln - 1) {
            byte_idx += line.len() + 1; // +1 for \n
        }
        markers.push((byte_idx, title));
    }
    markers.sort_by_key(|m| m.0);
    markers
}
