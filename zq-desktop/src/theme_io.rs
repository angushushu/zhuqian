use std::path::PathBuf;
use std::fs;
use eframe::egui;

use crate::parser::{
    ZqTheme, DisplayPrefs,
    deserialize_zq_file, serialize_zq_file,
};

pub(crate) fn load_file(path: &std::path::Path) -> (String, DisplayPrefs) {
    let raw = fs::read_to_string(path).unwrap_or_default();
    if path.extension().and_then(|s| s.to_str()) == Some("zq") {
        let (body, prefs) = deserialize_zq_file(&raw);
        return (body, prefs.unwrap_or_default());
    }
    // .zq.md and other formats: plain text, no embedded prefs
    (raw, DisplayPrefs::default())
}

pub(crate) fn save_file(path: &std::path::Path, text: &str, prefs: &DisplayPrefs) {
    if path.extension().and_then(|s| s.to_str()) == Some("zq") {
        let data = serialize_zq_file(text, prefs);
        let _ = fs::write(path, data);
    } else {
        // .zq.md, .md, .txt, .log — plain text, no meta header
        let _ = fs::write(path, text);
    }
}

pub(crate) fn zhuqian_dir() -> PathBuf {
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

pub(crate) fn load_zq_themes() -> Vec<ZqTheme> {
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
                if let Ok(t) = serde_json::from_str::<ZqTheme>(&data) {
                    let mut t = t;
                    if t.name.is_empty() {
                        t.name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unnamed").to_string();
                    }
                    themes.push(t);
                }
            }
        }
    }
    themes
}

pub(crate) fn save_zq_theme(theme: &ZqTheme) {
    let mut path = zhuqian_dir();
    let fname = theme.name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    path.push(format!("{}.zqtheme", fname));
    if let Ok(d) = serde_json::to_string_pretty(theme) {
        let _ = fs::write(path, d);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub(crate) struct ZqSession {
    pub last_opened_files: Vec<PathBuf>,
    pub last_folder: Option<PathBuf>,
    pub active_tab: usize,
    pub global_prefs: DisplayPrefs,
}

pub(crate) fn load_session() -> Option<ZqSession> {
    let mut path = zhuqian_dir();
    path.push("session.json");
    if let Ok(data) = fs::read_to_string(path) {
        return serde_json::from_str(&data).ok();
    }
    None
}

pub(crate) fn save_session(session: &ZqSession) {
    let mut path = zhuqian_dir();
    path.push("session.json");
    if let Ok(data) = serde_json::to_string_pretty(session) {
        let _ = fs::write(path, data);
    }
}


pub(crate) fn draw_bg_image(ui: &egui::Ui, tex: &egui::TextureHandle, rect: egui::Rect) {
    let tex_size = tex.size_vec2();
    let scale = (rect.width() / tex_size.x).max(rect.height() / tex_size.y);
    let uv_w = rect.width() / (tex_size.x * scale);
    let uv_h = rect.height() / (tex_size.y * scale);
    let uv_x = (1.0 - uv_w) * 0.5;
    let uv_y = (1.0 - uv_h) * 0.5;
    let uv = egui::Rect::from_min_max(egui::pos2(uv_x, uv_y), egui::pos2(uv_x + uv_w, uv_y + uv_h));
    ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
}
