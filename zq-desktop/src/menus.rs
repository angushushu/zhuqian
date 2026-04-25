use eframe::egui;

use crate::app::ZhuQianEditor;
use crate::parser;
use crate::parser::Language;

pub(crate) fn render_command_palette(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    if !app.show_cmd_palette { return; }

    let bg_side = egui::Color32::from_rgb(app.prefs.theme.bg_side[0], app.prefs.theme.bg_side[1], app.prefs.theme.bg_side[2]);
    let text_side = egui::Color32::from_rgb(app.prefs.theme.text_side[0], app.prefs.theme.text_side[1], app.prefs.theme.text_side[2]);
    let accent_ui = egui::Color32::from_rgb(app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]);
    let s = parser::get_strings(app.prefs.language);

    let mut execute_cmd: Option<crate::app::EditorCommand> = None;

    let window_width = 500.0;
    let screen_rect = ctx.screen_rect();
    let pos = egui::pos2(screen_rect.center().x - window_width / 2.0, 100.0);

    egui::Window::new("Command Palette")
        .fixed_pos(pos)
        .fixed_size(egui::vec2(window_width, 400.0))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(egui::Frame::window(&ctx.style())
            .fill(bg_side.gamma_multiply(0.98))
            .rounding(egui::CornerRadius::ZERO)
            .inner_margin(0.0) // Remove global margin
            .stroke(egui::Stroke::new(1.0, accent_ui.gamma_multiply(0.3)))
            .shadow(egui::epaint::Shadow {
                offset: [0, 10],
                blur: 20,
                spread: 0,
                color: egui::Color32::from_black_alpha(40),
            })
        )
        .show(ctx, |ui| {
            ui.spacing_mut().window_margin = egui::Margin::ZERO;
            ui.set_min_width(window_width);
            
            // Padding for top search area
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    let re = ui.add(egui::TextEdit::singleline(&mut app.cmd_query)
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::proportional(18.0))
                        .frame(false)
                        .hint_text(&s.cmd_hint));
                    re.request_focus();
                    if re.changed() { app.cmd_cursor = 0; }

                    ui.add_space(8.0);
                    ui.add(egui::Separator::default().spacing(8.0).grow(16.0));
                });
                ui.add_space(16.0);
            });

            ui.add_space(8.0);

            let zen_label = if app.prefs.zen_mode { &s.zen_mode_on } else { &s.zen_mode_off };
            let labels_label = if app.prefs.hide_labels { &s.labels_on } else { &s.labels_off };
            let commands: Vec<(&str, &str, crate::app::EditorCommand)> = vec![
                (&s.new_file, "Ctrl+N", crate::app::EditorCommand::NewFile),
                (&s.open_file_btn, "Ctrl+O", crate::app::EditorCommand::OpenFile),
                (&s.open_folder_btn, "Ctrl+Shift+O", crate::app::EditorCommand::OpenFolder),
                (&s.save, "Ctrl+S", crate::app::EditorCommand::SaveCurrent),
                (&s.save_as, "Ctrl+Shift+S", crate::app::EditorCommand::SaveAsCurrent),
                (&s.show_sidebar, "Ctrl+B", crate::app::EditorCommand::ToggleSidebar),
                (&s.show_settings, "Ctrl+,", crate::app::EditorCommand::ToggleSettings),
                (&s.outline, "Ctrl+P", crate::app::EditorCommand::ToggleQuickNav),
                (&s.help_shortcuts, "F1", crate::app::EditorCommand::ShowHelp),
                (zen_label, "Ctrl+Z", crate::app::EditorCommand::ToggleZenMode),
                (labels_label, "Ctrl+H", crate::app::EditorCommand::ToggleLabels),
                (&s.exit, "Alt+F4", crate::app::EditorCommand::Exit),
            ];

            let query = app.cmd_query.to_lowercase();
            let filtered: Vec<_> = commands.into_iter()
                .filter(|(name, _, _)| query.is_empty() || name.to_lowercase().contains(&query))
                .collect();

            let mut scroll_to_selected = false;
            if !filtered.is_empty() {
                app.cmd_cursor = app.cmd_cursor.min(filtered.len().saturating_sub(1));
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    app.cmd_cursor = (app.cmd_cursor + 1) % filtered.len();
                    scroll_to_selected = true;
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    app.cmd_cursor = (app.cmd_cursor + filtered.len() - 1) % filtered.len();
                    scroll_to_selected = true;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    execute_cmd = Some(filtered[app.cmd_cursor].2.clone());
                }
            }

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 2.0;

                    for (i, (name, shortcut, cmd)) in filtered.into_iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width() - 16.0, 28.0), egui::Sense::click());
                            
                            let is_selected = i == app.cmd_cursor;
                            if ui.rect_contains_pointer(rect) || is_selected {
                                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, accent_ui.gamma_multiply(0.15));
                                if is_selected && scroll_to_selected {
                                    resp.scroll_to_me(None);
                                }
                            }

                            if resp.clicked() {
                                execute_cmd = Some(cmd);
                            }

                            // Name on left
                            ui.painter().text(rect.left_center() + egui::vec2(8.0, 0.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(14.0), text_side);
                            
                            // Shortcut on right
                            ui.painter().text(rect.right_center() - egui::vec2(8.0, 0.0), egui::Align2::RIGHT_CENTER, shortcut, egui::FontId::monospace(11.0), text_side.gamma_multiply(0.5));
                        });
                    }
                });
            ui.add_space(8.0);

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
            ui.spacing_mut().item_spacing.x = 0.0;
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
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

                            let close_resp = ui.add(egui::Button::new(egui::RichText::new(" × ").size(11.0).color(fg_col)).fill(bg_col));
                            if close_resp.clicked() { close_idx = Some(i); }
                        });
                    }
                    if let Some(idx) = close_idx { app.close_tab(idx); }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("☰").size(14.0).color(text_side)).fill(bg_side)).on_hover_text("Command Palette (Ctrl+Shift+P)").clicked() {
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
    let _accent_ui = egui::Color32::from_rgb(app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]);
    let s = parser::get_strings(app.prefs.language);

    egui::TopBottomPanel::bottom("status_bar")
        .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::symmetric(8, 2)))
        .show(ctx, |ui| {
            let stats = parser::compute_stats(app.active_text());

            // Detect context: what is the cursor on?
            let mut context_info = String::new();
            if let Some(tab) = app.tabs.get(app.active_tab) {
                let editor_id = if app.active_pane == crate::app::SplitPane::Left { "editor_main" } else { "editor_split" };
                if let Some(state) = egui::text_edit::TextEditState::load(ctx, egui::Id::new(editor_id)) {
                    if let Some(crange) = state.cursor.char_range() {
                        let c_idx = crange.primary.index;
                        let byte_pos = tab.text.char_indices().nth(c_idx).map(|(i, _)| i).unwrap_or(tab.text.len());
                        let current_line = tab.text[..byte_pos].lines().count() + 1;

                        // Check if cursor is on a label
                        let labels = parser::parse_semantic_labels(&tab.text);
                        if let Some(label) = labels.iter().find(|l| c_idx >= l.start_byte && c_idx <= l.end_byte) {
                            let cat_color = parser::get_label_color(&label.category, &app.prefs.label_types, [app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]]);
                            let cat_color32 = egui::Color32::from_rgb(cat_color[0], cat_color[1], cat_color[2]);
                            let same_count = labels.iter().filter(|l| l.category == label.category).count();
                            context_info = format!("Ln {} | [{}] {} | {}x", current_line, label.category, label.text, same_count);

                            // Left side: context info with colored label
                            ui.horizontal(|ui| {
                                // Color dot
                                let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                                ui.painter().circle_filled(dot_rect.center(), 4.0, cat_color32);
                                ui.label(egui::RichText::new(&context_info).size(11.0).color(text_side));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(format!(
                                        "{} {}  {} {}  {} {}",
                                        s.lines, stats.lines, s.words, stats.words, s.labels, stats.labels
                                    )).size(11.0).color(text_side.gamma_multiply(0.6)));
                                });
                            });
                            return;
                        }

                        // Check if cursor is in a heading section
                        let headings = parser::extract_headings(&tab.text);
                        let mut current_heading: Option<&str> = None;
                        let mut section_start_line = 1;
                        for (ln, _lv, title) in &headings {
                            if *ln <= current_line {
                                current_heading = Some(title);
                                section_start_line = *ln;
                            } else {
                                break;
                            }
                        }
                        if let Some(heading) = current_heading {
                            let section_lines = current_line - section_start_line;
                            context_info = format!("Ln {} | {} | +{}", current_line, heading, section_lines);
                        } else {
                            context_info = format!("Ln {}", current_line);
                        }
                    }
                }
            }

            ui.horizontal(|ui| {
                if context_info.is_empty() {
                    ui.label(egui::RichText::new(format!(
                        "{} {}  {} {}  {} {}  {} {}",
                        s.lines, stats.lines, s.chars, stats.chars, s.words, stats.words, s.labels, stats.labels
                    )).size(11.0).color(text_side));
                } else {
                    ui.label(egui::RichText::new(&context_info).size(11.0).color(text_side));
                    ui.label(egui::RichText::new(format!(
                        "  {} {}  {} {}  {} {}",
                        s.chars, stats.chars, s.words, stats.words, s.labels, stats.labels
                    )).size(11.0).color(text_side.gamma_multiply(0.6)));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!(
                        "{}  {}px", app.font_display_name(), app.prefs.font_size as u32
                    )).size(11.0).color(text_side.gamma_multiply(0.5)));
                });
            });
        });
}

pub(crate) fn render_quick_nav(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    if !app.show_quick_nav { return; }

    let bg_side = egui::Color32::from_rgb(app.prefs.theme.bg_side[0], app.prefs.theme.bg_side[1], app.prefs.theme.bg_side[2]);
    let text_side = egui::Color32::from_rgb(app.prefs.theme.text_side[0], app.prefs.theme.text_side[1], app.prefs.theme.text_side[2]);
    let accent_ui = egui::Color32::from_rgb(app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]);

    let mut jump_to: Option<usize> = None;

    egui::Window::new("Quick Navigate")
        .fixed_pos(ctx.screen_rect().center() - egui::vec2(200.0, 150.0))
        .fixed_size(egui::vec2(400.0, 300.0))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(egui::Frame::window(&ctx.style()).fill(bg_side).rounding(8.0).inner_margin(12.0).stroke(egui::Stroke::new(1.0, accent_ui.gamma_multiply(0.5))))
        .show(ctx, |ui| {
            let re = ui.add(egui::TextEdit::singleline(&mut app.quick_nav_query)
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(16.0))
                .hint_text("Search headings & labels... (@labels, #headings)"));
            re.request_focus();

            ui.add_space(6.0);

            let text = app.active_text().to_string();
            let headings = parser::extract_headings(&text);
            let labels = parser::parse_semantic_labels(&text);
            let query = app.quick_nav_query.to_lowercase();
            let show_labels = !query.starts_with('#');
            let show_headings = !query.starts_with('@');
            let filter = if query.starts_with('@') { query[1..].to_string() } else if query.starts_with('#') { query[1..].to_string() } else { query.clone() };

            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                if show_headings {
                    for (ln, lv, title) in &headings {
                        if filter.is_empty() || title.to_lowercase().contains(&filter) {
                            let marker = "#".repeat(*lv);
                            let label = format!("{} {} (L{})", marker, title, ln);
                            let is_match = !filter.is_empty() && title.to_lowercase().contains(&filter);
                            let color = if is_match { accent_ui } else { text_side };
                            let resp = ui.add(egui::SelectableLabel::new(false, egui::RichText::new(&label).size(13.0).color(color)));
                            if resp.clicked() { jump_to = Some(*ln); }
                        }
                    }
                }

                if show_labels {
                    for lbl in &labels {
                        let display = format!("[{}] {} (L{})", lbl.category, lbl.text, lbl.line);
                        let searchable = display.to_lowercase();
                        if filter.is_empty() || searchable.contains(&filter) {
                            let cat_color = parser::get_label_color(&lbl.category, &app.prefs.label_types, [app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]]);
                            let cat_c32 = egui::Color32::from_rgb(cat_color[0], cat_color[1], cat_color[2]);
                            let is_match = !filter.is_empty() && searchable.contains(&filter);
                            let color = if is_match { cat_c32 } else { text_side };

                            ui.horizontal(|ui| {
                                let (dot_r, _) = ui.allocate_exact_size(egui::vec2(14.0, 16.0), egui::Sense::hover());
                                ui.painter().circle_filled(dot_r.center(), 4.0, cat_c32);
                                let resp = ui.add(egui::SelectableLabel::new(false, egui::RichText::new(&format!("{} {} (L{})", lbl.category, lbl.text, lbl.line)).size(13.0).color(color)));
                                if resp.clicked() { jump_to = Some(lbl.line); }
                            });
                        }
                    }
                }
            });

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.show_quick_nav = false;
            }
        });

    if let Some(ln) = jump_to {
        app.scroll_to_line = Some(ln);
        app.show_quick_nav = false;
    }
}

pub(crate) fn render_help(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    if !app.show_help { return; }

    let bg_side = egui::Color32::from_rgb(app.prefs.theme.bg_side[0], app.prefs.theme.bg_side[1], app.prefs.theme.bg_side[2]);
    let text_side = egui::Color32::from_rgb(app.prefs.theme.text_side[0], app.prefs.theme.text_side[1], app.prefs.theme.text_side[2]);
    let accent_ui = egui::Color32::from_rgb(app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]);
    let s = parser::get_strings(app.prefs.language);

    egui::Window::new("Help")
        .fixed_pos(ctx.screen_rect().center() - egui::vec2(220.0, 200.0))
        .fixed_size(egui::vec2(440.0, 400.0))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(egui::Frame::window(&ctx.style()).fill(bg_side).rounding(egui::CornerRadius::ZERO).inner_margin(16.0).stroke(egui::Stroke::new(1.0, accent_ui.gamma_multiply(0.5))))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(&s.help_shortcuts).size(18.0).color(accent_ui).strong());
            ui.add_space(12.0);

            let shortcuts = [
                ("Ctrl+S", &s.save),
                ("Ctrl+N", &s.new_file),
                ("Ctrl+B", &s.show_sidebar),
                ("Ctrl+P", &s.outline),
                ("Ctrl+Shift+P", &s.cmd_hint),
                ("Ctrl+,", &s.show_settings),
                ("Ctrl+\\", &s.logic_topology), // Using topology for split view as placeholder
                ("Ctrl+H", &s.hide_labels),
                ("Ctrl+C", &s.copy_clean),
            ];

            for (key, desc) in &shortcuts {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(*key).size(12.0).color(accent_ui).strong().monospace());
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new(*desc).size(12.0).color(text_side));
                });
                ui.add_space(2.0);
            }

            ui.add_space(12.0);
            ui.label(egui::RichText::new(&s.help_syntax).size(16.0).color(accent_ui).strong());
            ui.add_space(8.0);

            let syntax = [
                ("[category-text]", "Label with category and text"),
                ("[category-text|prop1]", "Label with properties"),
                ("[.category-text]", "Nested label (depth 2)"),
                ("[注-note text]", "Annotation / author note"),
            ];

            for (syn, desc) in &syntax {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(*syn).size(12.0).color(text_side).monospace().background_color(text_side.gamma_multiply(0.08)));
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(*desc).size(11.0).color(text_side.gamma_multiply(0.7)));
                });
                ui.add_space(2.0);
            }

            ui.add_space(12.0);
            if ui.button(egui::RichText::new("Close").size(13.0).color(text_side)).clicked() {
                app.show_help = false;
            }

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.show_help = false;
            }
        });
}
