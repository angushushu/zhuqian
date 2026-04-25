use eframe::egui;

use crate::app::{ZhuQianEditor, AutocompleteMode, SplitPane};
use crate::parser;
use crate::theme_io::draw_bg_image;

pub(crate) fn render_editor(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    let bg_main = egui::Color32::from_rgb(app.prefs.theme.bg_main[0], app.prefs.theme.bg_main[1], app.prefs.theme.bg_main[2]);
    let text_main = egui::Color32::from_rgb(app.prefs.theme.text_main[0], app.prefs.theme.text_main[1], app.prefs.theme.text_main[2]);
    let accent_hl = egui::Color32::from_rgb(app.prefs.theme.accent_hl[0], app.prefs.theme.accent_hl[1], app.prefs.theme.accent_hl[2]);
    let s = parser::get_strings(app.prefs.language);

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(bg_main).inner_margin(0.0))
        .show(ctx, |ui| {
            if let Some(ref tex) = app.bg_texture.clone() {
                draw_bg_image(ui, tex, ui.max_rect());
            }

            if app.tabs.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(&s.open_file).size(16.0).color(text_main));
                });
                return;
            }

            let font_size = app.prefs.font_size;
            let font_family = egui::FontFamily::Name(app.prefs.font_name.clone().into());
            let ff = font_family.clone();

            let left_idx = app.active_tab;
            let right_idx_opt = app.split_right_tab;

            let ZhuQianEditor {
                tabs, prefs, scroll_to_line, scroll_to_byte, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, ..
            } = app;

            let render_pane = |
                ui: &mut egui::Ui,
                tab: &mut crate::app::TabData,
                id_suffix: &str,
                is_focused_pane: bool,
                prefs: &mut crate::parser::DisplayPrefs,
                scroll_to_line: &mut Option<usize>,
                scroll_to_byte: &mut Option<usize>,
                just_jumped: &mut bool,
                ac_mode: &mut AutocompleteMode,
                ac_search: &mut String,
                ac_cursor: &mut usize,
                ac_pos: &mut Option<egui::Pos2>,
                active_pane: &mut SplitPane,
                pane_type: SplitPane
            | {
                // Auto-register label types from current text
                let text_for_labels = tab.text.clone();
                let labels = parser::parse_semantic_labels(&text_for_labels);
                let updated_types = parser::auto_register_labels(&labels, &prefs.label_types);
                if updated_types.len() != prefs.label_types.len() {
                    prefs.label_types = updated_types;
                }

                let mut layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                    let current_text = string.as_str();
                    let current_labels = parser::parse_semantic_labels(current_text);

                    let mut spans = parser::parse_markdown_to_spans(
                        current_text,
                        &prefs.theme.level_colors,
                        [accent_hl.r(), accent_hl.g(), accent_hl.b()],
                        [text_main.r(), text_main.g(), text_main.b()],
                        prefs.hide_labels
                    );

                    let len = current_text.len();
                    for label in &current_labels {
                        if label.category == "注" && !prefs.hide_labels {
                            for span in &mut spans {
                                let overlap = span.end.min(label.end_byte.min(len)) > span.start.max(label.start_byte);
                                if overlap { span.italic = true; }
                            }
                        }
                    }

                    let mut job = parser::render_to_job(current_text, &spans, font_size, ff.clone(), text_main);
                    job.wrap.max_width = wrap_width;
                    ui.painter().layout_job(job)
                };

                let scroll_id = egui::Id::new(format!("scroll_{}", id_suffix));

                // Calculate minimap rect before ScrollArea so we can position the scrollbar there
                let pane_rect = ui.max_rect();
                let mm_w = 34.0;
                let mm_margin = 8.0;
                let mm_scrollbar_rect = if !prefs.zen_mode {
                    let r = egui::Rect::from_min_max(
                        egui::pos2(pane_rect.right() - mm_w - mm_margin, pane_rect.top() + 4.0),
                        egui::pos2(pane_rect.right() - mm_margin, pane_rect.bottom() - 4.0)
                    );
                    Some(r)
                } else {
                    None
                };

                // Customize scrollbar style: floating, wide, semi-transparent to overlay minimap
                let prev_scroll_style = ui.spacing_mut().scroll;
                if mm_scrollbar_rect.is_some() {
                    let mut mm_scroll_style = egui::style::ScrollStyle::floating();
                    mm_scroll_style.bar_width = mm_w;
                    mm_scroll_style.floating_width = mm_w;
                    mm_scroll_style.floating_allocated_width = 0.0;
                    mm_scroll_style.bar_inner_margin = 0.0;
                    mm_scroll_style.foreground_color = false;
                    mm_scroll_style.dormant_background_opacity = 0.0;
                    mm_scroll_style.active_background_opacity = 0.0;
                    mm_scroll_style.interact_background_opacity = 0.1;
                    mm_scroll_style.dormant_handle_opacity = 0.25;
                    mm_scroll_style.active_handle_opacity = 0.35;
                    mm_scroll_style.interact_handle_opacity = 0.5;
                    ui.spacing_mut().scroll = mm_scroll_style;
                }

                let mut scroll_area = egui::ScrollArea::vertical()
                    .id_salt(scroll_id)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);
                if let Some(r) = mm_scrollbar_rect {
                    scroll_area = scroll_area.scroll_bar_rect(r);
                }
                if let Some(target_y) = tab.scroll_target_y.take() {
                    scroll_area = scroll_area.vertical_scroll_offset(target_y);
                }
                let editor_id = egui::Id::new(id_suffix);

                if is_focused_pane {
                    if let Some(target_byte) = *scroll_to_byte {
                        let char_idx = tab.text.char_indices()
                            .take_while(|(b, _)| *b < target_byte)
                            .count();
                            
                        let mut state = egui::text_edit::TextEditState::load(ctx, editor_id).unwrap_or_default();
                        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(char_idx))));
                        egui::TextEdit::store_state(ctx, editor_id, state);
                        
                        *scroll_to_byte = None;
                        *just_jumped = true;
                    }
                    if let Some(_target_ln) = *scroll_to_line {
                        // We handle this after the text edit to use the galley for scrolling
                    }
                }

                // Parse labels and headings once for minimap + relations
                let mm_labels = parser::parse_semantic_labels(&tab.text);
                let mm_headings = parser::extract_headings(&tab.text);
                let total_lines_mm = tab.text.lines().count().max(1);
                let mm_label_types = prefs.label_types.clone();
                let mm_accent_ui = [prefs.theme.accent_ui[0], prefs.theme.accent_ui[1], prefs.theme.accent_ui[2]];
                let mm_relation_codes = prefs.relation_codes.clone();

                let avail = ui.available_width();
                let min_margin = if prefs.zen_mode { 48.0 } else { 64.0 };
                let canvas_width = if prefs.zen_mode {
                    800.0f32.min(avail - min_margin * 2.0)
                } else {
                    avail - min_margin * 2.0
                };

                // 1. Outer ScrollArea for global scrollbar
                let sa_out = scroll_area.show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        // 2. Centered Editor Column
                        ui.vertical_centered(|ui| {
                            ui.set_max_width(canvas_width);

                            let text_to_edit = &mut tab.text;
                            let text_edit = egui::TextEdit::multiline(text_to_edit)
                                .id(editor_id)
                                .frame(false)
                                .font(egui::FontId::new(font_size, ff.clone()))
                                .text_color(text_main)
                                .lock_focus(true)
                                .desired_width(canvas_width)
                                .layouter(&mut layouter);

                            // Pre-consume only the "Raw Copy" shortcut
                            let is_raw_copy_triggered = ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::C)));

                            let out = text_edit.show(ui);
                            let resp = out.response.clone();

                            // --- Restoring Focus and Highlight Logic ---
                            if resp.has_focus() {
                                if let Some(state) = egui::text_edit::TextEditState::load(ctx, editor_id) {
                                    if let Some(crange) = state.cursor.char_range() {
                                        let c_idx = crange.primary.index;
                                        let labels = parser::parse_semantic_labels(&tab.text);
                                        if let Some(active_label) = labels.iter().find(|l| c_idx >= l.start_byte && c_idx <= l.end_byte) {
                                            let galley = out.galley.clone();
                                            let start_pos = galley.pos_from_cursor(egui::text::CCursor::new(active_label.start_byte));
                                            let end_pos = galley.pos_from_cursor(egui::text::CCursor::new(active_label.end_byte));

                                            let rect = egui::Rect::from_min_max(
                                                resp.rect.min + egui::vec2(0.0, start_pos.min.y),
                                                resp.rect.min + egui::vec2(canvas_width, end_pos.max.y)
                                            );
                                            let painter = ui.painter().clone().with_layer_id(egui::LayerId::background());
                                            painter.rect_filled(rect, egui::CornerRadius::ZERO, accent_hl.gamma_multiply(0.05));
                                        }
                                    }
                                }
                            }

                            if resp.gained_focus() {
                                *active_pane = pane_type;
                            }
                            // Handle Jump-to-Line Request (from Sidebar or Minimap)
                            if pane_type == *active_pane {
                                if let Some(target_ln) = *scroll_to_line {
                                    let galley = out.galley.clone();
                                    let mut target_ccursor;

                                    if *just_jumped {
                                        // scroll_to_byte already positioned the cursor precisely
                                        // Use current cursor position for scroll calculation
                                        let state = egui::text_edit::TextEditState::load(ctx, editor_id).unwrap_or_default();
                                        target_ccursor = if let Some(crange) = state.cursor.char_range() {
                                            crange.primary
                                        } else {
                                            egui::text::CCursor::new(0)
                                        };
                                    } else {
                                        // Only scroll_to_line was set — position cursor at line start
                                        let mut current_pos = 0;
                                        target_ccursor = egui::text::CCursor::new(0);
                                        for (i, line) in tab.text.lines().enumerate() {
                                            if i + 1 == target_ln {
                                                let ccursor = egui::text::CCursor::new(current_pos);
                                                let mut state = egui::text_edit::TextEditState::load(ctx, editor_id).unwrap_or_default();
                                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                                                egui::TextEdit::store_state(ctx, editor_id, state);
                                                target_ccursor = ccursor;
                                                break;
                                            }
                                            current_pos += line.len() + 1;
                                        }
                                    }

                                    let pos = galley.pos_from_cursor(target_ccursor);
                                    let target_y = (pos.min.y - ui.available_height() / 2.0).max(0.0);
                                    tab.scroll_target_y = Some(target_y);
                                    
                                    *scroll_to_line = None;
                                    *just_jumped = true;
                                    resp.request_focus();
                                    ctx.request_repaint();
                                }
                            }

                            if is_focused_pane && *just_jumped {
                                resp.request_focus();
                                *just_jumped = false;
                            }

                            // --- Handle Copy/Cut Logic ---
                            if resp.has_focus() {
                                if is_raw_copy_triggered {
                                    if let Some(state) = egui::text_edit::TextEditState::load(ctx, editor_id) {
                                        if let Some(crange) = state.cursor.char_range() {
                                            if !crange.is_empty() {
                                                let start = crange.primary.index.min(crange.secondary.index);
                                                let end = crange.primary.index.max(crange.secondary.index);
                                                let raw_text: String = tab.text.chars().skip(start).take(end - start).collect();
                                                ui.ctx().copy_text(raw_text);
                                            }
                                        }
                                    }
                                } else if prefs.hide_labels {
                                    ui.ctx().output_mut(|o| {
                                        for cmd in &mut o.commands {
                                            if let egui::output::OutputCommand::CopyText(text) = cmd {
                                                if !text.is_empty() {
                                                    *text = parser::strip_all_labels_regex(text);
                                                }
                                            }
                                        }
                                    });
                                }
                            }

                            if resp.changed() { tab.is_dirty = true; }

                            // --- Autocomplete Logic ---
                            if resp.has_focus() {
                                let state_opt = egui::text_edit::TextEditState::load(ctx, editor_id);
                                if let Some(state) = state_opt {
                                    if let Some(crange) = state.cursor.char_range() {
                                        let c_idx = crange.primary.index;
                                        let galley = out.galley.clone();
                                        let cursor_pos = galley.pos_from_cursor(crange.primary);
                                        *ac_pos = Some(resp.rect.min + cursor_pos.min.to_vec2());

                                        let txt = tab.text.as_str();
                                        let mut start_idx = c_idx;
                                        let mut found_mode = AutocompleteMode::None;
                                        let search_limit = 20;

                                        let chars: Vec<(usize, char)> = txt.char_indices().collect();
                                        if c_idx <= chars.len() {
                                            let mut i = c_idx as i32 - 1;
                                            let mut count = 0;
                                            while i >= 0 && count < search_limit {
                                                let c = chars[i as usize].1;
                                                if c == '\n' || c == ']' { break; }
                                                if c == '[' { found_mode = AutocompleteMode::Category; start_idx = i as usize; break; }
                                                if c == '|' { found_mode = AutocompleteMode::Shortcode; start_idx = i as usize; break; }
                                                i -= 1;
                                                count += 1;
                                            }
                                        }

                                        if found_mode != AutocompleteMode::None {
                                            let search_str = chars.iter().skip(start_idx + 1).take(c_idx.saturating_sub(start_idx + 1)).map(|(_, c)| c).collect::<String>();
                                            if found_mode == AutocompleteMode::Category && search_str.contains('|') {
                                                *ac_mode = AutocompleteMode::None;
                                            } else {
                                                *ac_mode = found_mode;
                                                *ac_search = search_str.to_lowercase();
                                            }
                                        } else {
                                            *ac_mode = AutocompleteMode::None;
                                        }
                                    }
                                }
                            } else if is_focused_pane {
                                *ac_mode = AutocompleteMode::None;
                            }

                                (out, resp)
                            }).inner
                        }).inner
                });

                let (out, _resp) = sa_out.inner;

                // Restore previous scrollbar style
                ui.spacing_mut().scroll = prev_scroll_style;

                // Build line-to-pixel-Y mapping from galley (used for click-to-line targeting)
                let galley = &out.galley;
                let content_h = sa_out.content_size.y;

                // Map each text line (1-based) to its pixel Y position via galley
                let mut line_pixel_ys: Vec<f32> = Vec::new();
                {
                    let mut char_offset = 0;
                    for (_i, line_text) in tab.text.lines().enumerate() {
                        let ccursor = egui::text::CCursor::new(char_offset);
                        let pos = galley.pos_from_cursor(ccursor);
                        line_pixel_ys.push(pos.min.y);
                        char_offset += line_text.chars().count() + 1; // +1 for '\n'
                    }
                    if line_pixel_ys.is_empty() {
                        line_pixel_ys.push(0.0);
                    }
                }

                // Helper: convert a logical line number (1-based) to minimap Y ratio (0.0..1.0)
                let line_to_mm_ratio = |ln: usize| -> f32 {
                    if content_h > 0.0 && ln > 0 && ln <= line_pixel_ys.len() {
                        line_pixel_ys[ln - 1] / content_h
                    } else {
                        0.0
                    }
                };

                // 3. Fixed Minimap Overlay (Outside ScrollArea)
                if let Some(mm_rect) = mm_scrollbar_rect {
                    if total_lines_mm > 0 {
                        let mm_h = mm_rect.height();

                        let mm_bg = egui::Color32::from_rgb(prefs.theme.bg_main[0], prefs.theme.bg_main[1], prefs.theme.bg_main[2]);
                        let mm_text = egui::Color32::from_rgb(prefs.theme.text_main[0], prefs.theme.text_main[1], prefs.theme.text_main[2]);

                        ui.allocate_ui_at_rect(mm_rect, |ui| {
                            let rect = ui.max_rect();
                            let mm_resp = ui.interact(rect, ui.id(), egui::Sense::click());
                            let painter = ui.painter().with_clip_rect(rect);
                            painter.rect_filled(rect, 4.0, mm_bg.gamma_multiply(0.2));

                            let mm_padding = 8.0;
                            let mm_draw_h = mm_h - mm_padding * 2.0;

                            // Render headings using pixel-based positioning
                            for (ln, lv, _title) in &mm_headings {
                                let ratio = line_to_mm_ratio(*ln);
                                let y = rect.top() + mm_padding + ratio * mm_draw_h;
                                let bar_h = 2.0;
                                let bar_w = mm_w * (0.3 + 0.15 * (*lv as f32).min(4.0));
                                let bar_x = rect.left() + (mm_w - bar_w) / 2.0;
                                painter.rect_filled(egui::Rect::from_min_max(egui::pos2(bar_x, y), egui::pos2(bar_x + bar_w, y + bar_h)), 0.0, mm_text.gamma_multiply(0.25));
                            }

                            // Render labels using pixel-based positioning
                            for label in &mm_labels {
                                let ratio = line_to_mm_ratio(label.line);
                                let y = rect.top() + mm_padding + ratio * mm_draw_h;
                                
                                let depth_idx = label.depth.saturating_sub(1) % prefs.theme.level_colors.len();
                                let d_c = prefs.theme.level_colors[depth_idx];
                                let depth_color = egui::Color32::from_rgb(d_c[0], d_c[1], d_c[2]);
                                
                                painter.circle_filled(egui::pos2(rect.center().x, y), 2.5, depth_color);
                            }

                            // Click-to-scroll: use pixel-based ratio for accurate targeting
                            if mm_resp.clicked() {
                                if let Some(pos) = mm_resp.interact_pointer_pos() {
                                    let click_y = pos.y - rect.top() - mm_padding;
                                    let click_ratio = (click_y / mm_draw_h).clamp(0.0, 1.0);
                                    let target_pixel_y = click_ratio * content_h;
                                    let mut best_line = 1;
                                    let mut best_dist = f32::MAX;
                                    for (i, &py) in line_pixel_ys.iter().enumerate() {
                                        let dist = (py - target_pixel_y).abs();
                                        if dist < best_dist {
                                            best_dist = dist;
                                            best_line = i + 1;
                                        }
                                    }
                                    *scroll_to_line = Some(best_line);
                                }
                            }
                        });
                    }
                }

                        // Render autocomplete popup in a floating area
                        if is_focused_pane && *ac_mode != AutocompleteMode::None {
                             if let Some(pos) = *ac_pos {
                                let mut options = Vec::new();
                                let mut option_colors = Vec::new();
                                if *ac_mode == AutocompleteMode::Category {
                                    for lt in &prefs.label_types {
                                        if lt.name.to_lowercase().contains(&*ac_search) {
                                            options.push(lt.name.clone());
                                            option_colors.push(egui::Color32::from_rgb(lt.color[0], lt.color[1], lt.color[2]));
                                        }
                                    }
                                } else if *ac_mode == AutocompleteMode::Shortcode {
                                    for t in &prefs.tag_codes {
                                        if t.symbol.to_lowercase().contains(&*ac_search) || t.label.to_lowercase().contains(&*ac_search) {
                                            options.push(t.symbol.clone());
                                            option_colors.push(egui::Color32::TRANSPARENT);
                                        }
                                    }
                                    for r in &prefs.relation_codes {
                                        if r.prefix.to_lowercase().contains(&*ac_search) || r.label.to_lowercase().contains(&*ac_search) {
                                            options.push(r.prefix.clone());
                                            option_colors.push(egui::Color32::TRANSPARENT);
                                        }
                                    }
                                }

                                if !options.is_empty() {
                                    *ac_cursor = (*ac_cursor).min(options.len().saturating_sub(1));
                                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) { *ac_cursor = (*ac_cursor + 1) % options.len(); }
                                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) { *ac_cursor = if *ac_cursor == 0 { options.len() - 1 } else { *ac_cursor - 1 }; }

                                    let mut accept = false;
                                    if ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Tab)) { accept = true; }

                                    if accept {
                                        let selected = &options[*ac_cursor];
                                        if let Some(mut state) = egui::text_edit::TextEditState::load(ctx, editor_id) {
                                            if let Some(mut crange) = state.cursor.char_range() {
                                                let current_idx = crange.primary.index;
                                                let replace_start = current_idx - ac_search.chars().count();
                                                let mut chars: Vec<char> = tab.text.chars().collect();
                                                let insert_text = if *ac_mode == AutocompleteMode::Category { format!("{}|", selected) } else { selected.clone() };
                                                chars.splice(replace_start..current_idx, insert_text.chars());
                                                tab.text = chars.into_iter().collect();
                                                tab.is_dirty = true;
                                                let new_idx = replace_start + insert_text.chars().count();
                                                crange.primary.index = new_idx;
                                                crange.secondary = crange.primary;
                                                state.cursor.set_char_range(Some(crange));
                                                egui::TextEdit::store_state(ctx, editor_id, state);
                                                *ac_mode = AutocompleteMode::None;
                                            }
                                        }
                                    } else {
                                        egui::Area::new(egui::Id::new("autocomplete_popup").with(id_suffix))
                                            .fixed_pos(pos + egui::vec2(0.0, font_size * 1.5))
                                            .show(ctx, |popup_ui| {
                                                egui::Frame::menu(popup_ui.style()).show(popup_ui, |inner| {
                                                    inner.set_max_width(240.0);
                                                    for (i, opt) in options.iter().enumerate() {
                                                        let is_selected = i == *ac_cursor;
                                                        let bg = if is_selected { accent_hl } else { egui::Color32::TRANSPARENT };
                                                        inner.horizontal(|inner_hz| {
                                                            if i < option_colors.len() && option_colors[i] != egui::Color32::TRANSPARENT {
                                                                let dot_color = option_colors[i];
                                                                let (rect, _) = inner_hz.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                                                                inner_hz.painter().circle_filled(rect.center(), 4.0, dot_color);
                                                            }
                                                            let rt = egui::RichText::new(opt).color(if is_selected { bg_main } else { text_main }).background_color(bg);
                                                            if inner_hz.add(egui::Label::new(rt).selectable(false).sense(egui::Sense::click())).clicked() {
                                                                *ac_cursor = i;
                                                            }
                                                        }).response.highlight();
                                                    }
                                                });
                                            });
                                    }
                                }
                            }
                        }

                // === Label Relations Panel (below editor) ===
                if is_focused_pane {
                    if let Some(state) = egui::text_edit::TextEditState::load(ctx, editor_id) {
                        if let Some(crange) = state.cursor.char_range() {
                            let c_idx = crange.primary.index.min(tab.text.len());
                            if let Some(active_label) = mm_labels.iter().find(|l| c_idx >= l.start_byte && c_idx <= l.end_byte) {
                                let mut relations = Vec::new();
                                for prop in &active_label.properties {
                                    for rel in &mm_relation_codes {
                                        if prop.starts_with(&rel.prefix) {
                                            let target_name = prop.trim_start_matches(&rel.prefix).trim().to_string();
                                            if let Some(target) = mm_labels.iter().find(|l| l.category == target_name || l.text == target_name) {
                                                relations.push((target.line, target.category.clone(), target.text.clone()));
                                            }
                                        }
                                    }
                                }

                                // Also find labels that reference this one
                                let my_name = active_label.category.clone();
                                let my_text = active_label.text.clone();
                                for label in &mm_labels {
                                    if label.start_byte == active_label.start_byte { continue; }
                                    for prop in &label.properties {
                                        if prop.contains(&my_name) || prop.contains(&my_text) {
                                            relations.push((label.line, label.category.clone(), label.text.clone()));
                                        }
                                    }
                                }

                                if !relations.is_empty() {
                                    let mm_text2 = egui::Color32::from_rgb(prefs.theme.text_main[0], prefs.theme.text_main[1], prefs.theme.text_main[2]);
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(&s.links).size(11.0).color(mm_text2.gamma_multiply(0.5)));
                                        for (line, cat, txt) in &relations {
                                            let cat_color = parser::get_label_color(cat, &mm_label_types, mm_accent_ui);
                                            let cat_c32 = egui::Color32::from_rgb(cat_color[0], cat_color[1], cat_color[2]);
                                            let pill_bg = cat_c32.gamma_multiply(0.2);
                                            let pill = egui::RichText::new(format!("[{}] {}", cat, txt))
                                                .size(11.0)
                                                .color(cat_c32)
                                                .background_color(pill_bg);
                                            let resp = ui.add(egui::Label::new(pill).sense(egui::Sense::click()));
                                            if resp.clicked() {
                                                *scroll_to_line = Some(*line);
                                            }
                                            ui.add_space(4.0);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }

            };

            if let Some(right_idx) = right_idx_opt {
                if left_idx != right_idx && left_idx < tabs.len() && right_idx < tabs.len() {
                    ui.columns(2, |cols| {
                        let max_idx = std::cmp::max(left_idx, right_idx);
                        let min_idx = std::cmp::min(left_idx, right_idx);
                        let (left_slice, right_slice) = tabs.split_at_mut(max_idx);
                        let mut_min = &mut left_slice[min_idx];
                        let mut_max = &mut right_slice[0];
                        
                        let (left_tab, right_tab) = if left_idx < right_idx {
                            (mut_min, mut_max)
                        } else {
                            (mut_max, mut_min)
                        };

                        let left_is_focused = *active_pane == SplitPane::Left;
                        let right_is_focused = *active_pane == SplitPane::Right;

                        render_pane(&mut cols[0], left_tab, "editor_main", left_is_focused, prefs, scroll_to_line, scroll_to_byte, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, SplitPane::Left);
                        render_pane(&mut cols[1], right_tab, "editor_split", right_is_focused, prefs, scroll_to_line, scroll_to_byte, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, SplitPane::Right);
                    });
                } else {
                    if let Some(tab) = tabs.get_mut(left_idx) {
                        let is_focused = *active_pane == SplitPane::Left;
                        render_pane(ui, tab, "editor_main", is_focused, prefs, scroll_to_line, scroll_to_byte, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, SplitPane::Left);
                    }
                }
            } else {
                if left_idx < tabs.len() {
                    if let Some(tab) = tabs.get_mut(left_idx) {
                        let is_focused = *active_pane == SplitPane::Left;
                        render_pane(ui, tab, "editor_main", is_focused, prefs, scroll_to_line, scroll_to_byte, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, SplitPane::Left);
                    }
                }
            }
        });
}
