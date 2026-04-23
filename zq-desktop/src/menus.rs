use eframe::egui;

use crate::app::ZhuQianEditor;
use crate::parser;
use crate::parser::Language;

pub(crate) fn render_command_palette(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    if !app.show_cmd_palette { return; }

    let bg_side = egui::Color32::from_rgb(app.prefs.theme.bg_side[0], app.prefs.theme.bg_side[1], app.prefs.theme.bg_side[2]);
    let text_side = egui::Color32::from_rgb(app.prefs.theme.text_side[0], app.prefs.theme.text_side[1], app.prefs.theme.text_side[2]);
    let accent_ui = egui::Color32::from_rgb(app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]);

    let mut execute_cmd: Option<crate::app::EditorCommand> = None;

    egui::Window::new("Command Palette")
        .fixed_pos(ctx.screen_rect().center() - egui::vec2(200.0, 150.0))
        .fixed_size(egui::vec2(400.0, 300.0))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(egui::Frame::window(&ctx.style()).fill(bg_side).rounding(8.0).inner_margin(12.0).stroke(egui::Stroke::new(1.0, accent_ui.gamma_multiply(0.5))))
        .show(ctx, |ui| {
            let re = ui.add(egui::TextEdit::singleline(&mut app.cmd_query)
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(16.0))
                .hint_text("Type a command..."));
            re.request_focus();

            ui.add_space(10.0);

            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                let zen_label = if app.prefs.zen_mode { "✓ Zen Mode  (centered canvas)" } else { "  Zen Mode  (centered canvas)" };
                let labels_label = if app.prefs.hide_labels { "✓ Hide Labels (reading mode)" } else { "  Show Labels (writing mode)" };
                let commands: Vec<(&str, crate::app::EditorCommand)> = vec![
                    ("New File (Ctrl+N)", crate::app::EditorCommand::NewFile),
                    ("Open File...", crate::app::EditorCommand::OpenFile),
                    ("Open Folder...", crate::app::EditorCommand::OpenFolder),
                    ("Save (Ctrl+S)", crate::app::EditorCommand::SaveCurrent),
                    ("Save As...", crate::app::EditorCommand::SaveAsCurrent),
                    ("Copy Clean Text (Ctrl+Shift+C)", crate::app::EditorCommand::CopyClean),
                    ("Toggle Sidebar (Ctrl+B)", crate::app::EditorCommand::ToggleSidebar),
                    ("Toggle Settings (Ctrl+Shift+P)", crate::app::EditorCommand::ToggleSettings),
                    ("Toggle Split View (Ctrl+\\)", crate::app::EditorCommand::ToggleSplitRight),
                    (zen_label, crate::app::EditorCommand::ToggleZenMode),
                    (labels_label, crate::app::EditorCommand::ToggleLabels),
                    ("Exit Application", crate::app::EditorCommand::Exit),
                ];

                let query = app.cmd_query.to_lowercase();
                for (name, cmd) in commands {
                    if query.is_empty() || name.to_lowercase().contains(&query) {
                        let btn = ui.add(egui::SelectableLabel::new(false, egui::RichText::new(name).size(14.0).color(text_side)));
                        if btn.clicked() {
                            execute_cmd = Some(cmd);
                        }
                    }
                }
            });

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.show_cmd_palette = false;
            }
        });

    if let Some(cmd) = execute_cmd {
        app.show_cmd_palette = false;
        app.handle_command(cmd, ctx);
    }
}

pub(crate) fn render_tab_bar(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    let bg_side = egui::Color32::from_rgb(app.prefs.theme.bg_side[0], app.prefs.theme.bg_side[1], app.prefs.theme.bg_side[2]);
    let text_side = egui::Color32::from_rgb(app.prefs.theme.text_side[0], app.prefs.theme.text_side[1], app.prefs.theme.text_side[2]);
    let accent_ui = egui::Color32::from_rgb(app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]);

    egui::TopBottomPanel::top("tab_bar")
        .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::symmetric(4, 1)))
        .show(ctx, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut close_idx: Option<usize> = None;
                    for (i, tab) in app.tabs.iter().enumerate() {
                        let name = tab.path.as_ref()
                            .and_then(|p| p.file_name())
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_else(|| (if app.prefs.language == Language::Zh { "未命名" } else { "Untitled" }).to_string());
                        
                        let is_active = i == app.active_tab || app.split_right_tab == Some(i);
                        let is_primary = i == app.active_tab;
                        let label = if tab.is_dirty { format!("● {}", name) } else { name };

                        let (bg_col, fg_col) = if is_active {
                            if is_primary {
                                (accent_ui.gamma_multiply(0.2), accent_ui)
                            } else {
                                (accent_ui.gamma_multiply(0.1), accent_ui.gamma_multiply(0.8))
                            }
                        } else {
                            (egui::Color32::TRANSPARENT, text_side.gamma_multiply(0.7))
                        };

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let resp = ui.add(egui::Button::new(egui::RichText::new(&label).size(12.0).color(fg_col)).fill(bg_col));
                            if resp.clicked() {
                                if app.active_pane == crate::app::SplitPane::Right && app.split_right_tab.is_some() {
                                    app.split_right_tab = Some(i);
                                } else {
                                    app.active_tab = i;
                                }
                            }

                            let close_resp = ui.add(egui::Button::new(egui::RichText::new("×").size(11.0).color(fg_col)).fill(bg_col));
                            if close_resp.clicked() { close_idx = Some(i); }

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("│").size(10.0).color(text_side.gamma_multiply(0.3)));
                            ui.add_space(8.0);
                        });
                    }
                    if let Some(idx) = close_idx { app.close_tab(idx); }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("☰").size(14.0).color(text_side)).fill(bg_side)).on_hover_text("Command Palette (Ctrl+P)").clicked() {
                            app.handle_command(crate::app::EditorCommand::ToggleCommandPalette, ctx);
                        }
                        ui.add_space(8.0);
                        let btn_text = if app.split_right_tab.is_some() { "◫ 合并" } else { "◫ 分屏" };
                        let btn = ui.add(egui::Button::new(egui::RichText::new(btn_text).size(12.0).color(text_side)).fill(bg_side));
                        if btn.on_hover_text("并排比对不同的文件 (Ctrl+\\)").clicked() {
                            app.handle_command(crate::app::EditorCommand::ToggleSplitRight, ctx);
                        }
                    });
                });
            });
        });
}

pub(crate) fn render_status_bar(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    let bg_side = egui::Color32::from_rgb(app.prefs.theme.bg_side[0], app.prefs.theme.bg_side[1], app.prefs.theme.bg_side[2]);
    let text_side = egui::Color32::from_rgb(app.prefs.theme.text_side[0], app.prefs.theme.text_side[1], app.prefs.theme.text_side[2]);
    let s = parser::get_strings(app.prefs.language);

    egui::TopBottomPanel::bottom("status_bar")
        .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::symmetric(8, 2)))
        .show(ctx, |ui| {
            let stats = parser::compute_stats(app.active_text());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!(
                    "{} {}  {} {}  {} {}  {} {}",
                    s.lines, stats.lines, s.chars, stats.chars, s.words, stats.words, s.labels, stats.labels
                )).size(11.0).color(text_side));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!(
                        "{}  {}px", app.font_display_name(), app.prefs.font_size as u32
                    )).size(11.0).color(text_side));
                });
            });
        });
}
