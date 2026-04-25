use eframe::egui;

use crate::app::{SettingTab, ZhuQianEditor};
use crate::parser;
use crate::parser::{Language, LabelType};
use crate::theme_io::*;

pub(crate) fn render_settings(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    if !app.show_settings { return; }

    let bg_side = egui::Color32::from_rgb(app.prefs.theme.bg_side[0], app.prefs.theme.bg_side[1], app.prefs.theme.bg_side[2]);
    let text_side = egui::Color32::from_rgb(app.prefs.theme.text_side[0], app.prefs.theme.text_side[1], app.prefs.theme.text_side[2]);
    let accent_ui = egui::Color32::from_rgb(app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]);
    let accent_col = accent_ui;
    let s = parser::get_strings(app.prefs.language);

    egui::SidePanel::right("settings_panel").resizable(true).default_width(290.0)
        .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::same(8)))
        .show(ctx, |ui| {
            if let Some(ref tex) = app.panel_bg_texture.clone() {
                draw_bg_image(ui, tex, ui.max_rect());
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // ── Display Settings ──
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&s.display_settings).size(13.0).color(accent_col));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("×").size(11.0)).fill(accent_ui)).clicked() {
                            app.show_settings = false;
                        }
                    });
                });
                ui.add(egui::Separator::default().spacing(4.0));

                // ── Settings Tabs ──
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut app.settings_tab, SettingTab::General, if app.prefs.language == Language::Zh { "通用" } else { "General" });
                    ui.selectable_value(&mut app.settings_tab, SettingTab::Colors, if app.prefs.language == Language::Zh { "配色" } else { "Theme" });
                    ui.selectable_value(&mut app.settings_tab, SettingTab::Labels, &s.labels_tab);
                    ui.selectable_value(&mut app.settings_tab, SettingTab::Dictionary, &s.dict_tab);
                });
                ui.add(egui::Separator::default().spacing(8.0));

                match app.settings_tab {
                    SettingTab::General => render_general_tab(app, ui, text_side, &s),
                    SettingTab::Colors => render_colors_tab(app, ctx, ui, text_side, accent_ui, &s),
                    SettingTab::Labels => render_labels_tab(app, ui, text_side, accent_ui, &s),
                    SettingTab::Dictionary => render_dictionary_tab(app, ui, text_side, accent_ui, &s),
                }
            });
        });
}

fn render_general_tab(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, s: &parser::LangStrings) {
    // Font
    ui.label(egui::RichText::new(&s.font).size(11.0).color(text_side));
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut app.font_search)
            .hint_text(&s.search_font)
            .desired_width(160.0));
    });
    let search_lower = app.font_search.to_lowercase();
    let current_display = app.font_display_name().to_string();
    egui::ComboBox::from_id_salt("font_sel")
        .selected_text(&current_display)
        .width(220.0)
        .show_ui(ui, |ui| {
            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                for fe in &app.font_entries.clone() {
                    if !search_lower.is_empty() && !fe.display.to_lowercase().contains(&search_lower) { continue; }
                    if ui.selectable_label(app.prefs.font_name == fe.key, &fe.display).clicked() {
                        app.prefs.font_name = fe.key.clone();
                    }
                }
            });
        });

    ui.add_space(8.0);
    ui.label(egui::RichText::new(&s.font_size).size(11.0).color(text_side));
    ui.add(egui::Slider::new(&mut app.prefs.font_size, 10.0..=36.0).suffix("px"));

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.prefs.markdown_render, "");
        ui.label(egui::RichText::new(&s.markdown_render).size(11.0).color(text_side));
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.prefs.hide_labels, "");
        ui.label(egui::RichText::new(&s.hide_labels).size(11.0).color(text_side));
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.prefs.zen_mode, "");
        let zen_label = if app.prefs.language == Language::Zh { "Zen 模式 — 居中写作画布（最大 800px）" } else { "Zen mode — centered writing canvas (max 800px)" };
        ui.label(egui::RichText::new(zen_label).size(11.0).color(text_side));
    });

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&s.language).size(11.0).color(text_side));
        egui::ComboBox::from_id_salt("lang_sel")
            .selected_text(match app.prefs.language { Language::Zh => "中文", Language::En => "English" })
            .show_ui(ui, |ui| {
                if ui.selectable_label(app.prefs.language == Language::Zh, "中文").clicked() {
                    app.prefs.language = Language::Zh;
                }
                if ui.selectable_label(app.prefs.language == Language::En, "English").clicked() {
                    app.prefs.language = Language::En;
                }
            });
    });
}

fn render_colors_tab(app: &mut ZhuQianEditor, ctx: &egui::Context, ui: &mut egui::Ui, text_side: egui::Color32, accent_ui: egui::Color32, s: &parser::LangStrings) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        macro_rules! color_row {
            ($label:expr, $field:expr) => {
                ui.horizontal(|ui| {
                    let mut color = egui::Color32::from_rgb($field[0], $field[1], $field[2]);
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        $field = [color.r(), color.g(), color.b()];
                    }
                    ui.label(egui::RichText::new($label).size(11.0).color(text_side));
                });
            };
        }

        ui.add_space(8.0);

        color_row!(&s.editor_bg,          app.prefs.theme.bg_main);
        color_row!(&s.panel_bg,           app.prefs.theme.bg_side);
        color_row!(&s.editor_text_color,  app.prefs.theme.text_main);
        color_row!(&s.accent_ui_color,    app.prefs.theme.accent_ui);

        ui.add_space(12.0);
        ui.label(egui::RichText::new(&s.bg_image).size(11.0).color(text_side));
        ui.horizontal(|ui| {
            if ui.button(&s.editor_bg_img).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Some(p) = path.to_str() {
                        app.prefs.bg_image_path = Some(p.to_string());
                        app.bg_texture = ZhuQianEditor::load_bg_image(ctx, p);
                    }
                }
            }
            if app.prefs.bg_image_path.is_some() && ui.button(&s.clear).clicked() {
                app.prefs.bg_image_path = None;
                app.bg_texture = None;
            }
        });

        ui.add_space(16.0);
        ui.label(egui::RichText::new(&s.theme_section).size(11.0).color(accent_ui));
        if !app.zq_themes.is_empty() {
            egui::ComboBox::from_id_salt("load_zqtheme")
                .selected_text(&s.load_theme)
                .show_ui(ui, |ui| {
                    for t in &app.zq_themes {
                        if ui.selectable_label(false, &t.name).clicked() {
                            let name_backup = app.prefs.theme.name.clone();
                            app.prefs.theme = t.clone();
                            if app.prefs.theme.name.is_empty() { app.prefs.theme.name = name_backup; }
                        }
                    }
                });
        }
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut app.new_theme_name).hint_text(&s.theme_name).desired_width(120.0));
            if ui.button(&s.save_theme).clicked() && !app.new_theme_name.is_empty() {
                let mut t = app.prefs.theme.clone();
                t.name = app.new_theme_name.clone();
                save_zq_theme(&t);
                app.zq_themes.push(t);
                app.new_theme_name.clear();
            }
        });
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(if app.prefs.language == Language::Zh { "语义层级颜色" } else { "Semantic Level Colors" }).size(11.0).color(accent_ui));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("+").on_hover_text("Add Level").clicked() {
                    app.prefs.theme.level_colors.push([128, 128, 128]);
                }
            });
        });
        ui.add_space(8.0);
        
        let mut to_remove_level = None;
        for i in 0..app.prefs.theme.level_colors.len() {
            let label = format!("Level {}", i + 1);
            ui.horizontal(|ui| {
                let mut color = egui::Color32::from_rgb(app.prefs.theme.level_colors[i][0], app.prefs.theme.level_colors[i][1], app.prefs.theme.level_colors[i][2]);
                if ui.color_edit_button_srgba(&mut color).changed() {
                    app.prefs.theme.level_colors[i] = [color.r(), color.g(), color.b()];
                }
                ui.label(egui::RichText::new(&label).size(11.0).color(text_side));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("×").size(10.0)).fill(egui::Color32::TRANSPARENT)).clicked() {
                        to_remove_level = Some(i);
                    }
                });
            });
        }
        if let Some(idx) = to_remove_level {
            if app.prefs.theme.level_colors.len() > 1 {
                app.prefs.theme.level_colors.remove(idx);
            }
        }
    });
}

/// Labels tab: manage label types and delimiter config.
fn render_labels_tab(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, accent_ui: egui::Color32, s: &parser::LangStrings) {
    egui::ScrollArea::vertical().show(ui, |ui| {


        // ── Label Types List ──
        ui.label(egui::RichText::new(&s.label_types).size(12.0).color(accent_ui).strong());
        ui.add_space(4.0);

        let mut to_remove: Option<usize> = None;
        let types_len = app.prefs.label_types.len();

        for i in 0..types_len {
            let lt = &mut app.prefs.label_types[i];
            let is_system = lt.name == "注";

            ui.horizontal(|ui| {
                let mut color = egui::Color32::from_rgb(lt.color[0], lt.color[1], lt.color[2]);
                if ui.color_edit_button_srgba(&mut color).changed() {
                    lt.color = [color.r(), color.g(), color.b()];
                }
                ui.label(egui::RichText::new(&lt.name).size(11.0).color(text_side).strong());
                if is_system {
                    ui.label(egui::RichText::new(if app.prefs.language == Language::Zh { "(系统)" } else { "(System)" }).size(10.0).color(text_side.gamma_multiply(0.5)));
                } else {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("×").size(10.0)).fill(egui::Color32::TRANSPARENT)).clicked() {
                            to_remove = Some(i);
                        }
                    });
                }
            });
        }

        if let Some(idx) = to_remove {
            app.prefs.label_types.remove(idx);
        }

        ui.add_space(8.0);

        // ── Add New Type ──
        ui.group(|ui| {
            ui.label(egui::RichText::new(&s.add_type).size(11.0).color(accent_ui));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut app.new_label_type_name)
                    .hint_text(&s.type_name)
                    .desired_width(100.0));
                
                let mut new_col = egui::Color32::from_rgb(
                    (app.new_label_type_color[0] * 255.0) as u8,
                    (app.new_label_type_color[1] * 255.0) as u8,
                    (app.new_label_type_color[2] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut new_col).changed() {
                    app.new_label_type_color = [new_col.r() as f32 / 255.0, new_col.g() as f32 / 255.0, new_col.b() as f32 / 255.0];
                }
            });
            if ui.button(&s.add).clicked() && !app.new_label_type_name.is_empty() {
                let c = [
                    (app.new_label_type_color[0] * 255.0) as u8,
                    (app.new_label_type_color[1] * 255.0) as u8,
                    (app.new_label_type_color[2] * 255.0) as u8,
                ];
                let name = app.new_label_type_name.trim().to_string();
                // Don't add duplicate
                if !app.prefs.label_types.iter().any(|t| t.name == name) {
                    app.prefs.label_types.push(LabelType {
                        name,
                        color: c,
                        description: String::new(),
                    });
                }
                app.new_label_type_name.clear();
            }
        });
    });
}

fn render_dictionary_tab(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, accent_ui: egui::Color32, s: &parser::LangStrings) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Tag Codes
        ui.label(egui::RichText::new(&s.tag_codes).size(12.0).color(accent_ui).strong());
        ui.label(egui::RichText::new(&s.tag_codes_desc).size(10.0).color(text_side));
        ui.add_space(8.0);
        
        let mut to_remove_tag = None;
        for (i, t) in app.prefs.tag_codes.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let mut c = [t.color[0] as f32 / 255.0, t.color[1] as f32 / 255.0, t.color[2] as f32 / 255.0];
                if ui.color_edit_button_rgb(&mut c).changed() {
                    t.color = [(c[0]*255.0) as f32, (c[1]*255.0) as f32, (c[2]*255.0) as f32];
                }
                ui.add(egui::TextEdit::singleline(&mut t.symbol).desired_width(30.0).hint_text(&s.shortcode));
                ui.add(egui::TextEdit::singleline(&mut t.label).desired_width(120.0).hint_text(&s.description));
                if ui.add(egui::Button::new(egui::RichText::new("×").size(10.0)).fill(egui::Color32::TRANSPARENT)).clicked() {
                    to_remove_tag = Some(i);
                }
            });
        }
        if let Some(i) = to_remove_tag { app.prefs.tag_codes.remove(i); }
        if ui.button(&s.add_tag).clicked() {
            app.prefs.tag_codes.push(crate::parser::TagCode { symbol: "?".into(), label: "New Tag".into(), color: [0.5, 0.5, 0.5] });
        }
        
        ui.add_space(16.0);
        
        // Relation Codes
        ui.label(egui::RichText::new(&s.rel_codes).size(12.0).color(accent_ui).strong());
        ui.label(egui::RichText::new(&s.rel_codes_desc).size(10.0).color(text_side));
        ui.add_space(8.0);
        
        let mut to_remove_rel = None;
        for (i, r) in app.prefs.relation_codes.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let mut c = [r.color[0] as f32 / 255.0, r.color[1] as f32 / 255.0, r.color[2] as f32 / 255.0];
                if ui.color_edit_button_rgb(&mut c).changed() {
                    r.color = [(c[0]*255.0) as f32, (c[1]*255.0) as f32, (c[2]*255.0) as f32];
                }
                ui.add(egui::TextEdit::singleline(&mut r.prefix).desired_width(30.0).hint_text(&s.shortcode));
                ui.add(egui::TextEdit::singleline(&mut r.label).desired_width(120.0).hint_text(&s.description));
                if ui.add(egui::Button::new(egui::RichText::new("×").size(10.0)).fill(egui::Color32::TRANSPARENT)).clicked() {
                    to_remove_rel = Some(i);
                }
            });
        }
        if let Some(i) = to_remove_rel { app.prefs.relation_codes.remove(i); }
        if ui.button(&s.add_rel).clicked() {
            app.prefs.relation_codes.push(crate::parser::RelationCode { prefix: "new".into(), label: "New Relation".into(), color: [0.5, 0.5, 0.5] });
        }
    });
}

