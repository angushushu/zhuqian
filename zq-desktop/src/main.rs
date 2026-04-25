#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod app;
mod editor;
mod export;
mod fonts;
mod menus;
mod parser;
mod settings;
mod sidebar;
mod theme_io;

use app::ZhuQianEditor;
use parser::BaseTheme;

fn main() -> eframe::Result {
    let icon = image::load_from_memory(include_bytes!("../zq_icon.png")).ok().map(|img| {
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
            let font_entries = fonts::load_system_fonts(&mut fonts);
            let cjk_key: Option<String> = ["msyh", "simsun", "simhei", "msjh", "mingliu"]
                .iter()
                .find(|k| fonts.font_data.contains_key(**k))
                .map(|k| k.to_string());
            if let Some(ref cjk) = cjk_key {
                let fallbacks = [
                    cjk.as_str(),
                    "seguiemj", 
                    "segoeuiemoji", 
                    "seguisym",
                    "msyh", 
                    "simsun", 
                    "arial unicode ms",
                    "noto sans cjk sc",
                    "noto color emoji"
                ];
                let all_families: Vec<egui::FontFamily> = fonts.families.keys().cloned().collect();
                for family in all_families {
                    if let Some(list) = fonts.families.get_mut(&family) {
                        for f in fallbacks {
                            if fonts.font_data.contains_key(f) && !list.contains(&f.to_string()) {
                                list.push(f.to_string());
                            }
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

            style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;
        });

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
        let mut cmd_to_run = None;
        ctx.input_mut(|i| {
            use egui::{KeyboardShortcut, Modifiers, Key};
            let ctrl = Modifiers::CTRL;
            let shift_ctrl = Modifiers::CTRL | Modifiers::SHIFT;

            if i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::S)) {
                cmd_to_run = Some(app::EditorCommand::SaveCurrent);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::N)) {
                cmd_to_run = Some(app::EditorCommand::NewFile);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(shift_ctrl, Key::P)) {
                cmd_to_run = Some(app::EditorCommand::ToggleCommandPalette);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::P)) {
                cmd_to_run = Some(app::EditorCommand::ToggleQuickNav);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::Comma)) {
                cmd_to_run = Some(app::EditorCommand::ToggleSettings);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::B)) {
                cmd_to_run = Some(app::EditorCommand::ToggleSidebar);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::Backslash)) {
                cmd_to_run = Some(app::EditorCommand::ToggleSplitRight);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::G)) {
                cmd_to_run = Some(app::EditorCommand::ToggleQuickNav);
            }
        });
        if let Some(cmd) = cmd_to_run {
            self.handle_command(cmd, ctx);
        }

        // ── UI Styles ──
        let bg_side   = egui::Color32::from_rgb(self.prefs.theme.bg_side[0], self.prefs.theme.bg_side[1], self.prefs.theme.bg_side[2]);
        let text_side  = egui::Color32::from_rgb(self.prefs.theme.text_side[0], self.prefs.theme.text_side[1], self.prefs.theme.text_side[2]);
        let accent_ui  = egui::Color32::from_rgb(self.prefs.theme.accent_ui[0], self.prefs.theme.accent_ui[1], self.prefs.theme.accent_ui[2]);

        let mut visuals = if is_dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        visuals.window_corner_radius = egui::CornerRadius::ZERO;
        visuals.menu_corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;
        
        visuals.widgets.noninteractive.bg_fill = bg_side;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text_side);
        visuals.widgets.inactive.bg_fill = bg_side;
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_side.gamma_multiply(0.6));
        visuals.widgets.hovered.bg_fill = accent_ui.gamma_multiply(0.15);
        visuals.widgets.active.bg_fill = accent_ui.gamma_multiply(0.25);
        visuals.selection.bg_fill = accent_ui.gamma_multiply(0.3);
        visuals.selection.stroke = egui::Stroke::new(1.0, accent_ui);
        ctx.set_visuals(visuals);

        // ── Render panels ──
        menus::render_tab_bar(self, ctx);
        menus::render_status_bar(self, ctx);
        sidebar::render_sidebar(self, ctx);
        settings::render_settings(self, ctx);
        editor::render_editor(self, ctx);
        menus::render_command_palette(self, ctx);
        menus::render_quick_nav(self, ctx);
        menus::render_help(self, ctx);
    }
}
