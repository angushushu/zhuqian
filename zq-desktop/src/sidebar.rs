use eframe::egui;
use std::path::PathBuf;

use crate::app::{SidebarMode, ZhuQianEditor};
use crate::parser;
use crate::parser::LangStrings;
use crate::theme_io::draw_bg_image;

pub(crate) fn render_sidebar(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    if !app.show_left { return; }

    let bg_side = egui::Color32::from_rgb(app.prefs.theme.bg_side[0], app.prefs.theme.bg_side[1], app.prefs.theme.bg_side[2]);
    let text_side = egui::Color32::from_rgb(app.prefs.theme.text_side[0], app.prefs.theme.text_side[1], app.prefs.theme.text_side[2]);
    let accent_ui = egui::Color32::from_rgb(app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]);
    let s = parser::get_strings(app.prefs.language);

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(200.0)
        .width_range(0.0..=f32::INFINITY)
        .frame(egui::Frame::NONE.fill(bg_side).inner_margin(egui::Margin::same(6)))
        .show(ctx, |ui| {
            if let Some(ref tex) = app.panel_bg_texture.clone() {
                draw_bg_image(ui, tex, ui.max_rect());
            }

            ui.horizontal(|ui| {
                let w = 28.0;
                if ui.add_sized([w, 24.0], egui::SelectableLabel::new(app.sidebar_mode == SidebarMode::FileTree, "🗂")).on_hover_text(&s.files).clicked() { app.sidebar_mode = SidebarMode::FileTree; }
                if ui.add_sized([w, 24.0], egui::SelectableLabel::new(app.sidebar_mode == SidebarMode::Outline, "📄")).on_hover_text(&s.outline).clicked() { app.sidebar_mode = SidebarMode::Outline; }
                if ui.add_sized([w, 24.0], egui::SelectableLabel::new(app.sidebar_mode == SidebarMode::Semantic, "🏷")).on_hover_text(&s.semantic).clicked() { app.sidebar_mode = SidebarMode::Semantic; }
                if ui.add_sized([w, 24.0], egui::SelectableLabel::new(app.sidebar_mode == SidebarMode::LogicGraph, "🕸")).on_hover_text("Logic").clicked() { app.sidebar_mode = SidebarMode::LogicGraph; }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut text_side_dim = text_side;
                    if ui.add(egui::Button::new(egui::RichText::new("×").size(14.0).color(text_side_dim)).fill(bg_side)).clicked() { app.show_left = false; }
                });
            });
            ui.add(egui::Separator::default().spacing(4.0));

            match app.sidebar_mode {
                SidebarMode::FileTree => render_file_tree(app, ui, accent_ui, text_side),
                SidebarMode::Outline => render_outline(app, ui, text_side, &s),
                SidebarMode::Semantic => render_semantic_typed(app, ui, text_side, accent_ui, &s),
                SidebarMode::LogicGraph => render_logic_graph(app, ui, text_side, accent_ui, &s),
            }
        });
}

fn render_file_tree(app: &mut ZhuQianEditor, ui: &mut egui::Ui, accent_ui: egui::Color32, text_side: egui::Color32) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let file_list: Vec<PathBuf> = app.files.clone();
        for path in &file_list {
            let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let is_active = app.tabs.get(app.active_tab).map(|t| t.path.as_ref() == Some(path)).unwrap_or(false);
            let is_open = app.tabs.iter().any(|t| t.path.as_ref() == Some(path));

            let color = if is_active { accent_ui }
                else if is_open { accent_ui.gamma_multiply(0.7) }
                else { text_side };

            if ui.selectable_label(is_active, egui::RichText::new(&name).size(12.0).color(color)).clicked() {
                app.open_file(&path.clone());
            }
        }
    });
}

fn render_outline(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, s: &parser::LangStrings) {
    let text = app.active_text().to_string();
    let headings = parser::extract_headings(&text);
    egui::ScrollArea::vertical().show(ui, |ui| {
        if headings.is_empty() {
            ui.label(egui::RichText::new(&s.empty).size(11.0).color(text_side));
        }
        for (ln, lv, title) in &headings {
            let indent = "  ".repeat(lv.saturating_sub(1));
            let sz = 13.0 - (*lv as f32 * 0.5);
            let rt = egui::RichText::new(format!("{}{}", indent, title)).size(sz).color(text_side);
            if ui.selectable_label(false, rt).clicked() {
                app.scroll_to_line = Some(*ln);
            }
        }
    });
}

/// Semantic panel: outline tree visualization based on depth.
fn render_semantic_typed(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, accent_ui: egui::Color32, s: &parser::LangStrings) {
    let text = app.active_text().to_string();
    let labels = parser::parse_semantic_labels(&text);
    let label_types = &app.prefs.label_types;

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_min_width(0.0);
        ui.visuals_mut().indent_has_left_vline = false;
        ui.spacing_mut().indent = 12.0;

        if labels.is_empty() {
            ui.label(egui::RichText::new(&s.empty).size(11.0).color(text_side));
            return;
        }

        // Stats line
        ui.label(egui::RichText::new(format!("{} {}", s.labels, labels.len())).size(11.0).color(accent_ui));
        ui.add_space(4.0);

        for label in &labels {
            ui.horizontal(|ui| {
                ui.add_space(8.0 + (label.depth.saturating_sub(1) as f32 * 16.0));
                
                let cat_color = parser::get_label_color(&label.category, label_types, [app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]]);
                let cat_color32 = egui::Color32::from_rgb(cat_color[0], cat_color[1], cat_color[2]);
                
                let resp = ui.add(egui::Label::new(
                    egui::RichText::new(format!("[{}]", label.category)).size(11.0).color(cat_color32).strong()
                ).sense(egui::Sense::click()));

                if resp.clicked() {
                    app.scroll_to_line = Some(label.line);
                }

                if !label.text.is_empty() {
                    ui.add(egui::Label::new(
                        egui::RichText::new(&label.text).size(11.0).color(text_side)
                    ).truncate());
                }

                if !label.properties.is_empty() {
                    for prop in &label.properties {
                        let mut resolved = None;
                        for tag in &app.prefs.tag_codes {
                            if tag.symbol == *prop {
                                resolved = Some((tag.color, tag.symbol.clone(), tag.label.clone()));
                                break;
                            }
                        }
                        if resolved.is_none() {
                            for rel in &app.prefs.relation_codes {
                                if prop.starts_with(&rel.prefix) {
                                    resolved = Some((rel.color, prop.clone(), rel.label.clone()));
                                    break;
                                }
                            }
                        }

                        if let Some((col, sym, desc)) = resolved {
                            let rc = egui::Color32::from_rgb((col[0]*255.0) as u8, (col[1]*255.0) as u8, (col[2]*255.0) as u8);
                            let lbl = ui.add(egui::Label::new(
                                egui::RichText::new(format!(" {} ", sym)).size(10.0).color(egui::Color32::WHITE).background_color(rc.gamma_multiply(0.8))
                            ).sense(egui::Sense::hover()));
                            lbl.on_hover_text(desc);
                        } else {
                            ui.label(egui::RichText::new(prop).size(10.0).color(accent_ui).background_color(cat_color32.gamma_multiply(0.2)));
                        }
                    }
                }
            });
            ui.add_space(2.0);
        }
    });
}


fn render_logic_graph(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, accent_ui: egui::Color32, s: &parser::LangStrings) {
    let text = app.active_text().to_string();
    let labels = parser::parse_semantic_labels(&text);
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        if labels.is_empty() {
            ui.label(egui::RichText::new(&s.empty).size(11.0).color(text_side));
            return;
        }
        
        ui.label(egui::RichText::new("Logic Relation Topology").size(12.0).color(accent_ui).strong());
        ui.add_space(10.0);
        
        let mut node_rects = std::collections::HashMap::new();
        let mut edges = Vec::new();
        
        for (i, label) in labels.iter().enumerate() {
            for prop in &label.properties {
                for rel in &app.prefs.relation_codes {
                    if prop.starts_with(&rel.prefix) {
                        let target = prop.trim_start_matches(&rel.prefix).trim().to_string();
                        edges.push((i, target, rel.color));
                    }
                }
            }
            
            let key = label.category.clone();
            
            ui.horizontal(|ui| {
                ui.add_space(40.0 + (label.depth as f32 * 10.0));
                
                let cat_color = parser::get_label_color(&label.category, &app.prefs.label_types, [app.prefs.theme.accent_ui[0], app.prefs.theme.accent_ui[1], app.prefs.theme.accent_ui[2]]);
                let cat_color32 = egui::Color32::from_rgb(cat_color[0], cat_color[1], cat_color[2]);
                
                let resp = ui.add(egui::Label::new(
                    egui::RichText::new(format!("[{}]", key)).size(11.0).color(text_side).background_color(cat_color32.gamma_multiply(0.2))
                ).sense(egui::Sense::click()));
                
                if resp.clicked() {
                    app.scroll_to_line = Some(label.line);
                }
                
                node_rects.insert(key, resp.rect);
                node_rects.insert((i + 1).to_string(), resp.rect); 
            });
            ui.add_space(15.0);
        }
        
        let painter = ui.painter();
        for (src_idx, target_key, color) in edges {
            let src_key_str1 = (src_idx + 1).to_string();
            let src_rect_opt = node_rects.get(&labels[src_idx].category).or_else(|| node_rects.get(&src_key_str1));
            
            if let Some(src_rect) = src_rect_opt {
                if let Some(target_rect) = node_rects.get(&target_key) {
                    let c = egui::Color32::from_rgb((color[0]*255.0) as u8, (color[1]*255.0) as u8, (color[2]*255.0) as u8);
                    
                    let start_point = src_rect.left_center();
                    let end_point = target_rect.left_center();
                    
                    let control_offset = egui::vec2(-30.0 - ((src_idx as f32 % 3.0) * 15.0), 0.0);
                    let shape = egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                        points: [
                            start_point,
                            start_point + control_offset,
                            end_point + control_offset,
                            end_point
                        ],
                        closed: false,
                        fill: egui::Color32::TRANSPARENT,
                        stroke: egui::Stroke::new(1.5, c).into(),
                    });
                    painter.add(shape);
                    
                    let dir = (end_point - (end_point + control_offset)).normalized();
                    let perp = egui::vec2(-dir.y, dir.x) * 4.0;
                    let tip = end_point;
                    let p1 = end_point - dir * 8.0 + perp;
                    let p2 = end_point - dir * 8.0 - perp;
                    painter.add(egui::Shape::convex_polygon(vec![tip, p1, p2], c, egui::Stroke::NONE));
                }
            }
        }
    });
}
