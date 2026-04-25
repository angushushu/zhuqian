use eframe::egui;
use std::path::PathBuf;

use crate::app::{SidebarDragItem, SidebarMode, ZhuQianEditor};
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

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let w = 28.0;
                if ui.add_sized([w, 24.0], egui::SelectableLabel::new(app.sidebar_mode == SidebarMode::FileTree, "📁")).on_hover_text(&s.files).clicked() { app.sidebar_mode = SidebarMode::FileTree; }
                if ui.add_sized([w, 24.0], egui::SelectableLabel::new(app.sidebar_mode == SidebarMode::Outline, "☰")).on_hover_text(&s.outline).clicked() { app.sidebar_mode = SidebarMode::Outline; }
                if ui.add_sized([w, 24.0], egui::SelectableLabel::new(app.sidebar_mode == SidebarMode::Semantic, "🏷")).on_hover_text(&s.semantic).clicked() { app.sidebar_mode = SidebarMode::Semantic; }
                if ui.add_sized([w, 24.0], egui::SelectableLabel::new(app.sidebar_mode == SidebarMode::LogicGraph, "🔗")).on_hover_text(&s.logic_topology).clicked() { app.sidebar_mode = SidebarMode::LogicGraph; }

                ui.add_space(4.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let text_side_dim = text_side.gamma_multiply(0.7);
                    if ui.add(egui::Button::new(egui::RichText::new("?").size(12.0).color(text_side_dim)).fill(bg_side)).on_hover_text(&s.help_shortcuts).clicked() { app.show_help = !app.show_help; }
                    if ui.add(egui::Button::new(egui::RichText::new("×").size(14.0).color(text_side_dim)).fill(bg_side)).clicked() { app.show_left = false; }
                });
            });
            ui.add(egui::Separator::default().spacing(4.0));

            match app.sidebar_mode {
                SidebarMode::FileTree => render_file_tree(app, ui, accent_ui, text_side),
                SidebarMode::Outline => render_markdown_outline(app, ui, text_side, accent_ui, &s),
                SidebarMode::Semantic => render_semantic_outline(app, ui, text_side, accent_ui, &s),
                SidebarMode::LogicGraph => render_logic_graph(app, ui, text_side, accent_ui, &s),
            }
        });
}

fn render_module_header(ui: &mut egui::Ui, title: &str, accent_ui: egui::Color32, actions: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).strong().size(12.0).color(accent_ui));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), actions);
    });
    ui.add_space(2.0);
    ui.separator();
    ui.add_space(4.0);
}


fn render_file_tree(app: &mut ZhuQianEditor, ui: &mut egui::Ui, accent_ui: egui::Color32, text_side: egui::Color32) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let file_list: Vec<PathBuf> = app.files.clone();
        let mut drop_target: Option<PathBuf> = None;
        for path in &file_list {
            let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let is_active = app.tabs.get(app.active_tab).map(|t| t.path.as_ref() == Some(path)).unwrap_or(false);
            let is_open = app.tabs.iter().any(|t| t.path.as_ref() == Some(path));

            let color = if is_active { accent_ui }
                else if is_open { accent_ui.gamma_multiply(0.7) }
                else { text_side };

            let mut resp = ui.selectable_label(is_active, egui::RichText::new(&name).size(12.0).color(color));
            resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            
            if resp.drag_started() {
                app.dragged_item = Some(SidebarDragItem::File(path.clone()));
            }

            if let Some(SidebarDragItem::File(dragged_path)) = &app.dragged_item {
                if dragged_path != path && resp.hovered() {
                    drop_target = Some(path.clone());
                    ui.painter().rect_stroke(resp.rect, egui::CornerRadius::ZERO, egui::Stroke::new(1.0, accent_ui), egui::StrokeKind::Outside);
                }
            }

            if resp.clicked() {
                app.open_file(&path.clone());
            }
        }

        if let Some(SidebarDragItem::File(dragged_path)) = app.dragged_item.clone() {
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(target_path) = drop_target {
                    // In file mode, target_idx is not used, we pass a dummy or use a new method
                    // For now we keep the API consistent but files might need a dedicated handler
                }
                app.dragged_item = None;
            }
        }
    });
}

enum OutlineItem {
    Heading { ln: usize, lv: usize, title: String },
    Label(parser::SemanticLabel),
    GhostHeading { lv: usize, title: String, desc: String },
    GhostLabel { category: String, color: [u8; 3], desc: String },
}

fn render_markdown_outline(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, accent_ui: egui::Color32, s: &parser::LangStrings) {
    let text = app.active_text().to_string();
    let headings = parser::extract_headings(&text);
    let total_lines = text.lines().count().max(1);

    let mut items: Vec<OutlineItem> = headings.into_iter().map(|(ln, lv, title)| OutlineItem::Heading { ln, lv, title }).collect();

    for th in &app.active_template.expected_headings {
        let found = items.iter().any(|it| match it {
            OutlineItem::Heading { lv, title, .. } => *lv == th.level && (th.title.is_none() || title.contains(th.title.as_ref().unwrap())),
            _ => false,
        });
        if !found && th.required {
            items.push(OutlineItem::GhostHeading { lv: th.level, title: th.title.clone().unwrap_or_else(|| format!("H{}", th.level)), desc: th.description.clone() });
        }
    }

    items.sort_by_key(|i| match i {
        OutlineItem::Heading { ln, .. } => *ln,
        _ => 999999,
    });

    // Detect current line
    let current_line = get_current_line(app, ui);

    render_module_header(ui, &s.outline, accent_ui, |_| {});
    render_progress_bar(ui, current_line, total_lines, text_side, accent_ui);
    ui.add_space(4.0);

    egui::ScrollArea::vertical().id_salt("md_outline_scroll").show(ui, |ui| {
        if items.is_empty() {
            ui.label(egui::RichText::new(&s.empty).size(11.0).color(text_side));
            return;
        }

        let mut drop_target: Option<(usize, bool)> = None;

        for (idx, item) in items.iter().enumerate() {
            match item {
                OutlineItem::Heading { ln, lv, title } => {
                    let indent = (*lv as f32) * 12.0;
                    let is_active = current_line >= *ln && (idx + 1 == items.len() || match &items[idx+1] {
                        OutlineItem::Heading { ln: next_ln, .. } => current_line < *next_ln,
                        _ => true,
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(indent);
                        let marker = egui::RichText::new(format!("{} ", "#".repeat(*lv))).size(11.0).color(text_side.gamma_multiply(0.3));
                        ui.label(marker);
                        let text_color = if is_active { accent_ui } else { text_side };
                        
                        let mut job = egui::text::LayoutJob::default();
                        job.append(title, 0.0, egui::TextFormat {
                            font_id: egui::FontId::proportional(13.0),
                            color: text_color,
                            ..Default::default()
                        });
                        let galley = ui.painter().layout_job(job);
                        let (rect, resp) = ui.allocate_at_least(egui::vec2(ui.available_width(), 18.0), egui::Sense::click_and_drag());
                        ui.painter().with_clip_rect(rect).galley(rect.min, galley, egui::Color32::PLACEHOLDER);
                        
                        let mut resp = resp;
                        resp = resp.on_hover_cursor(egui::CursorIcon::Grab);
                        
                        let preview = get_chunk_preview(&text, *ln, true);
                        resp = resp.on_hover_ui(|ui| {
                            ui.set_max_width(300.0);
                            ui.label(egui::RichText::new(title).strong().color(accent_ui));
                            ui.add(egui::Separator::default().spacing(4.0));
                            ui.label(egui::RichText::new(preview).size(11.0).color(text_side.gamma_multiply(0.8)));
                        });

                        if resp.drag_started() {
                            app.dragged_item = Some(SidebarDragItem::Markdown(*ln));
                        }

                        // Static Hit Testing for Drop
                        if let Some(SidebarDragItem::Markdown(from_ln)) = app.dragged_item {
                            if from_ln != *ln && ui.rect_contains_pointer(resp.rect) {
                                let y_rel = (ui.input(|i| i.pointer.interact_pos()).unwrap_or_default().y - resp.rect.min.y) / resp.rect.height();
                                
                                if y_rel < 0.25 {
                                    // Insert BEFORE
                                    drop_target = Some((idx, false));
                                    let line_y = resp.rect.top();
                                    ui.painter().hline(egui::Rangef::new(resp.rect.left() - 4.0, resp.rect.right()), line_y, egui::Stroke::new(2.0, accent_ui));
                                    ui.painter().circle_filled(egui::pos2(resp.rect.left() - 4.0, line_y), 2.5, accent_ui);
                                } else if y_rel > 0.75 {
                                    // Insert AFTER
                                    drop_target = Some((idx + 1, false));
                                    let line_y = resp.rect.bottom();
                                    ui.painter().hline(egui::Rangef::new(resp.rect.left() - 4.0, resp.rect.right()), line_y, egui::Stroke::new(2.0, accent_ui));
                                    ui.painter().circle_filled(egui::pos2(resp.rect.left() - 4.0, line_y), 2.5, accent_ui);
                                } else {
                                    // Nest ON
                                    drop_target = Some((idx, true));
                                    ui.painter().rect_filled(resp.rect, egui::CornerRadius::ZERO, accent_ui.gamma_multiply(0.15));
                                    ui.painter().rect_stroke(resp.rect, egui::CornerRadius::ZERO, egui::Stroke::new(1.5, accent_ui), egui::StrokeKind::Outside);
                                }
                            }
                        }

                        if resp.clicked() {
                            app.scroll_to_line = Some(*ln);
                        }
                    });
                }
                OutlineItem::GhostHeading { lv, title, desc } => {
                    ui.horizontal(|ui| {
                        ui.add_space((*lv as f32) * 12.0);
                        let rt = egui::RichText::new(format!("+ {}", title)).size(13.0).color(text_side.gamma_multiply(0.4)).italics();
                        if ui.selectable_label(false, rt).on_hover_text(desc).clicked() {
                            let prefix = "#".repeat(*lv);
                            app.insert_text_at_end(&format!("\n\n{} {}\n", prefix, title));
                        }
                    });
                }
                _ => {}
            }
            ui.add_space(2.0);
        }

        if let Some(dragged) = app.dragged_item.clone() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            if ui.input(|i| i.pointer.any_released()) {
                if let Some((target_idx, is_child)) = drop_target {
                    app.handle_structural_move(dragged, target_idx, is_child);
                }
                app.dragged_item = None;
            }
        }
    });
}

fn get_current_byte(app: &ZhuQianEditor, ui: &egui::Ui) -> usize {
    if let Some(tab) = app.tabs.get(app.active_tab) {
        let editor_id = if app.active_pane == crate::app::SplitPane::Left { "editor_main" } else { "editor_split" };
        if let Some(state) = egui::text_edit::TextEditState::load(ui.ctx(), egui::Id::new(editor_id)) {
            if let Some(crange) = state.cursor.char_range() {
                let c_idx = crange.primary.index;
                return tab.text.char_indices().nth(c_idx).map(|(i, _)| i).unwrap_or(tab.text.len());
            }
        }
    }
    0
}

fn get_current_line(app: &ZhuQianEditor, ui: &egui::Ui) -> usize {
    if let Some(tab) = app.tabs.get(app.active_tab) {
        let editor_id = if app.active_pane == crate::app::SplitPane::Left { "editor_main" } else { "editor_split" };
        if let Some(state) = egui::text_edit::TextEditState::load(ui.ctx(), egui::Id::new(editor_id)) {
            if let Some(crange) = state.cursor.char_range() {
                let c_idx = crange.primary.index;
                let byte_pos = tab.text.char_indices().nth(c_idx).map(|(i, _)| i).unwrap_or(tab.text.len());
                return tab.text[..byte_pos].lines().count() + 1;
            }
        }
    }
    0
}

fn render_progress_bar(ui: &mut egui::Ui, current_line: usize, total_lines: usize, text_side: egui::Color32, accent_ui: egui::Color32) {
    let progress = (current_line as f32 / total_lines as f32).min(1.0);
    let progress_width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(progress_width, 3.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, text_side.gamma_multiply(0.1));
    ui.painter().rect_filled(
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + progress_width * progress, rect.max.y)),
        0.0,
        accent_ui.gamma_multiply(0.6)
    );
}

/// Semantic panel: outline tree visualization based on depth.
fn render_semantic_outline(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, accent_ui: egui::Color32, s: &parser::LangStrings) {
    let text = app.active_text().to_string();
    let labels = parser::parse_semantic_labels(&text);

    // Detect current byte context
    let current_byte = get_current_byte(app, ui);
    
    // Find active label (closest preceding)
    let mut active_idx = None;
    for (i, label) in labels.iter().enumerate() {
        if current_byte >= label.start_byte {
            active_idx = Some(i);
        } else {
            break;
        }
    }

    render_module_header(ui, &s.semantic, accent_ui, |ui| {
        if ui.selectable_label(app.show_logic_overlay, "🔗").on_hover_text(&s.logic_links).clicked() { 
            app.show_logic_overlay = !app.show_logic_overlay; 
        }
    });

    egui::ScrollArea::vertical().id_salt("semantic_outline_scroll").show(ui, |ui| {
        ui.set_min_width(0.0);

        if labels.is_empty() {
            ui.label(egui::RichText::new(&s.empty).size(11.0).color(text_side));
            return;
        }

        // ── Health Check ──
        render_health_check(app, ui, &labels, text_side);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        // ── Semantic Tree (Document Order) ──
        let mut node_rects = std::collections::HashMap::new();
        
        let mut drop_target: Option<(usize, bool)> = None; // (target_idx, is_child)

        for (idx, label) in labels.iter().enumerate() {
            let indent = (label.depth.saturating_sub(1) as f32) * 16.0;
            let is_active = Some(idx) == active_idx;

            ui.horizontal(|ui| {
                ui.add_space(indent);

                let depth_idx = label.depth.saturating_sub(1) % app.prefs.theme.level_colors.len();
                let depth_color = app.prefs.theme.level_colors[depth_idx];
                let depth_color32 = egui::Color32::from_rgb(depth_color[0], depth_color[1], depth_color[2]);

                let cat_color = if label.explicit_leaf.is_some() {
                    parser::get_label_color(&label.category, &app.prefs.label_types, depth_color)
                } else {
                    [150, 150, 150]
                };
                let cat_color32 = egui::Color32::from_rgb(cat_color[0], cat_color[1], cat_color[2]);

                let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 14.0), egui::Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 3.5, depth_color32);

                let text_color = if is_active { accent_ui } else { text_side };
                let bg = if is_active { depth_color32.gamma_multiply(0.2) } else { egui::Color32::TRANSPARENT };

                let mut job = egui::text::LayoutJob::default();
                let cat_display = label.category.clone();
                
                job.append(&format!("[{}] ", cat_display), 0.0, egui::TextFormat {
                    font_id: egui::FontId::proportional(11.0),
                    color: cat_color32,
                    background: bg,
                    ..Default::default()
                });
                job.append(&label.text, 0.0, egui::TextFormat {
                    font_id: egui::FontId::proportional(11.0),
                    color: text_color,
                    background: bg,
                    ..Default::default()
                });

                let galley = ui.painter().layout_job(job);
                let (rect, resp) = ui.allocate_at_least(egui::vec2(ui.available_width(), 16.0), egui::Sense::click_and_drag());
                ui.painter().with_clip_rect(rect).galley(rect.min, galley, egui::Color32::PLACEHOLDER);
                
                let mut resp = resp;
                
                let preview = get_chunk_preview(&text, label.start_byte, false);
                resp = resp.on_hover_ui(|ui| {
                    ui.set_max_width(300.0);
                    ui.label(egui::RichText::new(format!("[{}] {}", label.category, label.text)).strong().color(accent_ui));
                    ui.add(egui::Separator::default().spacing(4.0));
                    ui.label(egui::RichText::new(preview).size(11.0).color(text_side.gamma_multiply(0.8)));
                }).on_hover_cursor(egui::CursorIcon::Grab);
                
                if resp.drag_started() {
                    app.dragged_item = Some(SidebarDragItem::Semantic(idx));
                }

                // Static Hit Testing for Drop
                if let Some(SidebarDragItem::Semantic(dragged_idx)) = app.dragged_item {
                    if dragged_idx != idx && ui.rect_contains_pointer(resp.rect) {
                        let y_rel = (ui.input(|i| i.pointer.interact_pos()).unwrap_or_default().y - resp.rect.min.y) / resp.rect.height();
                        
                        if y_rel < 0.25 {
                            drop_target = Some((idx, false));
                            let line_y = resp.rect.top();
                            ui.painter().hline(egui::Rangef::new(resp.rect.left() - 4.0, resp.rect.right()), line_y, egui::Stroke::new(2.0, accent_ui));
                            ui.painter().circle_filled(egui::pos2(resp.rect.left() - 4.0, line_y), 2.5, accent_ui);
                        } else if y_rel > 0.75 {
                            drop_target = Some((idx + 1, false));
                            let line_y = resp.rect.bottom();
                            ui.painter().hline(egui::Rangef::new(resp.rect.left() - 4.0, resp.rect.right()), line_y, egui::Stroke::new(2.0, accent_ui));
                            ui.painter().circle_filled(egui::pos2(resp.rect.left() - 4.0, line_y), 2.5, accent_ui);
                        } else {
                            drop_target = Some((idx, true));
                            ui.painter().rect_filled(resp.rect, egui::CornerRadius::ZERO, accent_ui.gamma_multiply(0.15));
                            ui.painter().rect_stroke(resp.rect, egui::CornerRadius::ZERO, egui::Stroke::new(1.5, accent_ui), egui::StrokeKind::Outside);
                        }
                    }
                }

                if resp.clicked() {
                    app.scroll_to_line = Some(label.line);
                    app.scroll_to_byte = Some(label.start_byte);
                }
                node_rects.insert(label.category.clone(), resp.rect);
                node_rects.insert((idx + 1).to_string(), resp.rect);
            });
            ui.add_space(2.0);
        }

        // Handle the actual move on release
        if let Some(dragged) = app.dragged_item.clone() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            if ui.input(|i| i.pointer.any_released()) {
                if let Some((target_idx, is_child)) = drop_target {
                    app.handle_structural_move(dragged, target_idx, is_child);
                }
                app.dragged_item = None;
            }
        }

        // ── Logic Overlay (Topology) ──
        if app.show_logic_overlay {
            render_logic_overlay(app, ui, &labels, &node_rects);
        }

        // ── Ghost Labels ──
        ui.add_space(10.0);
        let expected_labels = app.active_template.label_types.clone();
        for tl in &expected_labels {
            let found = labels.iter().any(|l| l.category == tl.name);
            if !found && tl.required {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let c = egui::Color32::from_rgb(tl.color[0], tl.color[1], tl.color[2]).gamma_multiply(0.4);
                    if ui.selectable_label(false, egui::RichText::new(format!("+ [{}]", tl.name)).size(11.0).color(c).italics()).on_hover_text(&tl.description).clicked() {
                        app.insert_text_at_end(&format!("\n[{}- ]\n", tl.name));
                    }
                });
            }
        }
    });
}

fn render_health_check(app: &mut ZhuQianEditor, ui: &mut egui::Ui, labels: &[parser::SemanticLabel], text_side: egui::Color32) {
    let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut questions = Vec::new();
    for label in labels {
        for prop in &label.properties {
            for tag in &app.prefs.tag_codes {
                if prop == &tag.symbol {
                    *tag_counts.entry(tag.symbol.clone()).or_insert(0) += 1;
                }
            }
            if prop.contains('?') { questions.push(label); }
        }
    }

    egui::collapsing_header::CollapsingHeader::new(egui::RichText::new("📊 Stats").size(11.0).color(text_side.gamma_multiply(0.6)))
        .default_open(true)
        .show(ui, |ui| {
            let bar_area_w = ui.available_width();
            for tag in &app.prefs.tag_codes {
                let count = *tag_counts.get(&tag.symbol).unwrap_or(&0);
                let color = egui::Color32::from_rgb((tag.color[0]*255.0) as u8, (tag.color[1]*255.0) as u8, (tag.color[2]*255.0) as u8);
                ui.horizontal(|ui| {
                    ui.add_sized([12.0, 12.0], egui::Label::new(egui::RichText::new(&tag.symbol).color(color).strong()));
                    let fraction = (count as f32 / labels.len().max(1) as f32).min(1.0);
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_area_w - 24.0, 4.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, text_side.gamma_multiply(0.1));
                    ui.painter().rect_filled(egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + (rect.width() * fraction), rect.max.y)), 2.0, color.gamma_multiply(0.7));
                });
            }

            let pos_count = *tag_counts.get("+").unwrap_or(&0);
            let neg_count = *tag_counts.get("-").unwrap_or(&0);
            let sub_count = *tag_counts.get("~").unwrap_or(&0);
            if pos_count > 0 && neg_count == 0 && sub_count == 0 {
                ui.label(egui::RichText::new("⚠ Argument might be unbalanced.").size(10.0).color(egui::Color32::from_rgb(200, 150, 50)).italics());
            }

            if !questions.is_empty() {
                ui.add_space(4.0);
                egui::collapsing_header::CollapsingHeader::new(egui::RichText::new(format!("❓ Pending ({})", questions.len())).size(10.0).color(egui::Color32::from_rgb(200, 180, 80))).show(ui, |ui| {
                    for q in questions {
                        if ui.selectable_label(false, egui::RichText::new(format!("L{}: {}", q.line, q.text)).size(10.0).color(text_side)).clicked() {
                            app.scroll_to_line = Some(q.line);
                        }
                    }
                });
            }
        });
}

fn render_logic_overlay(app: &ZhuQianEditor, ui: &mut egui::Ui, labels: &[parser::SemanticLabel], node_rects: &std::collections::HashMap<String, egui::Rect>) {
    let painter = ui.painter();
    for (i, label) in labels.iter().enumerate() {
        for prop in &label.properties {
            for rel in &app.prefs.relation_codes {
                if prop.starts_with(&rel.prefix) {
                    let target_key = prop.trim_start_matches(&rel.prefix).trim().to_string();
                    if let (Some(src_rect), Some(target_rect)) = (node_rects.get(&label.category), node_rects.get(&target_key)) {
                        let c = egui::Color32::from_rgb((rel.color[0]*255.0) as u8, (rel.color[1]*255.0) as u8, (rel.color[2]*255.0) as u8);
                        let start_point = src_rect.left_center();
                        let end_point = target_rect.left_center();
                        let control_offset = egui::vec2(-20.0 - ((i as f32 % 3.0) * 10.0), 0.0);
                        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                            points: [start_point, start_point + control_offset, end_point + control_offset, end_point],
                            closed: false, fill: egui::Color32::TRANSPARENT, stroke: egui::Stroke::new(1.0, c.gamma_multiply(0.6)).into(),
                        }));
                        let dir = (end_point - (end_point + control_offset)).normalized();
                        let tip = end_point;
                        let p1 = end_point - dir * 6.0 + egui::vec2(-dir.y, dir.x) * 3.0;
                        let p2 = end_point - dir * 6.0 - egui::vec2(-dir.y, dir.x) * 3.0;
                        painter.add(egui::Shape::convex_polygon(vec![tip, p1, p2], c.gamma_multiply(0.6), egui::Stroke::NONE));
                    }
                }
            }
        }
    }
}


fn render_logic_graph(app: &mut ZhuQianEditor, ui: &mut egui::Ui, text_side: egui::Color32, accent_ui: egui::Color32, s: &parser::LangStrings) {
    let text = app.active_text().to_string();
    let labels = parser::parse_semantic_labels(&text);
    
    if labels.is_empty() {
        ui.label(egui::RichText::new(&s.empty).size(11.0).color(text_side));
        return;
    }

    render_module_header(ui, &s.logic_topology, accent_ui, |ui| {
        if ui.button("⟲").on_hover_text("Reset Layout").clicked() {
            app.node_positions.clear();
        }
    });

    // Both-way scroll for the large graph area
    egui::ScrollArea::both().id_salt("topology_scroll").show(ui, |ui| {
        // Calculate canvas bounds
        let mut max_x = ui.available_width().max(400.0);
        let mut max_y = 600.0f32;
        for pos in app.node_positions.values() {
            max_x = max_x.max(pos.x + 150.0);
            max_y = max_y.max(pos.y + 100.0);
        }
        
        let (total_rect, _) = ui.allocate_exact_size(egui::vec2(max_x, max_y), egui::Sense::hover());
        let painter = ui.painter_at(total_rect);
        let base_pos = total_rect.min;

        let mut node_rects = std::collections::HashMap::new();
        let mut edges = Vec::new();

        // ── Phase 1: Update Positions & Render Nodes ──
        for (i, label) in labels.iter().enumerate() {
            let key = label.category.clone();
            
            // Get or init position
            let pos_val = app.node_positions.entry(key.clone()).or_insert_with(|| {
                egui::pos2(40.0 + (label.depth as f32 * 40.0), i as f32 * 60.0 + 40.0)
            });
            
            let node_center = base_pos + egui::vec2(pos_val.x, pos_val.y);
            let node_rect = egui::Rect::from_center_size(node_center, egui::vec2(80.0, 28.0));
            
            // Interaction
            let node_id = ui.make_persistent_id(format!("node_{}", key));
            let resp = ui.interact(node_rect, node_id, egui::Sense::drag().union(egui::Sense::click()));
            
            if resp.dragged() {
                pos_val.x += resp.drag_delta().x;
                pos_val.y += resp.drag_delta().y;
            }
            if resp.clicked() {
                app.scroll_to_line = Some(label.line);
                app.scroll_to_byte = Some(label.start_byte);
            }
            
            // Dual coloring for topology nodes
            let depth_color = if let Some(c) = app.prefs.theme.level_colors.get(label.depth.saturating_sub(1)) {
                *c
            } else {
                app.prefs.theme.accent_ui
            };
            let depth_color32 = egui::Color32::from_rgb(depth_color[0], depth_color[1], depth_color[2]);

            let cat_color = if label.explicit_leaf.is_some() {
                parser::get_label_color(&key, &app.prefs.label_types, depth_color)
            } else {
                [150, 150, 150]
            };
            let cat_color32 = egui::Color32::from_rgb(cat_color[0], cat_color[1], cat_color[2]);
            
            // Background uses level color
            painter.rect_filled(node_rect, 2.0, depth_color32.gamma_multiply(0.15));
            // Stroke uses category color
            let stroke_color = if resp.dragged() { accent_ui } else { cat_color32.gamma_multiply(0.8) };
            painter.rect_stroke(node_rect, 2.0, egui::Stroke::new(1.2, stroke_color), egui::StrokeKind::Inside);
            
            painter.text(node_rect.center(), egui::Align2::CENTER_CENTER, &key, egui::FontId::proportional(11.0), text_side);
            
            node_rects.insert(key.clone(), node_rect);
            
            // Collect edges
            for prop in &label.properties {
                for rel in &app.prefs.relation_codes {
                    if prop.starts_with(&rel.prefix) {
                        let target = prop.trim_start_matches(&rel.prefix).trim().to_string();
                        edges.push((key.clone(), target, rel.color));
                    }
                }
            }
        }

        // ── Phase 2: Render Edges (Arrows) ──
        for (src_key, target_key, color) in edges {
            if let (Some(src_rect), Some(target_rect)) = (node_rects.get(&src_key), node_rects.get(&target_key)) {
                let c = egui::Color32::from_rgb((color[0]*255.0) as u8, (color[1]*255.0) as u8, (color[2]*255.0) as u8);
                
                let start_pt = src_rect.center();
                let end_pt = target_rect.center();
                
                // Offset start/end to rect edges
                let dir = (end_pt - start_pt).normalized();
                let start_edge = start_pt + dir * 15.0;
                let end_edge = end_pt - dir * 15.0;

                let dist = start_edge.distance(end_edge);
                let control_scale = (dist * 0.3).min(50.0);
                let cp1 = start_edge + dir * control_scale + egui::vec2(-control_scale, 0.0);
                let cp2 = end_edge - dir * control_scale + egui::vec2(-control_scale, 0.0);

                painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                    points: [start_edge, cp1, cp2, end_edge],
                    closed: false,
                    fill: egui::Color32::TRANSPARENT,
                    stroke: egui::Stroke::new(1.2, c.gamma_multiply(0.6)).into(),
                }));
                
                // Arrow head
                let arrow_dir = (end_edge - cp2).normalized();
                let perp = egui::vec2(-arrow_dir.y, arrow_dir.x) * 3.5;
                painter.add(egui::Shape::convex_polygon(
                    vec![end_edge, end_edge - arrow_dir * 7.0 + perp, end_edge - arrow_dir * 7.0 - perp],
                    c.gamma_multiply(0.6),
                    egui::Stroke::NONE
                ));
            }
        }
    });
}

fn get_chunk_preview(text: &str, start_pos: usize, is_line: bool) -> String {
    let content = if is_line {
        // start_pos is 1-indexed line number
        text.lines().skip(start_pos).take(5).collect::<Vec<_>>().join("\n")
    } else {
        // start_pos is byte offset
        // Skip the label itself (approximate)
        let slice = &text[start_pos..];
        let end = slice.find(']').unwrap_or(0);
        let body = &slice[end+1..];
        body.chars().take(200).collect::<String>()
    };
    
    let cleaned = content.trim().replace('\r', "");
    if cleaned.is_empty() {
        "(Empty content)".to_string()
    } else if cleaned.chars().count() > 150 {
        format!("{}...", cleaned.chars().take(150).collect::<String>())
    } else {
        cleaned
    }
}
