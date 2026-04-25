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

                let scroll_area = egui::ScrollArea::vertical().id_salt(format!("scroll_{}", id_suffix));
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
                        *scroll_to_line = None;
                        *just_jumped = true;
                    } else if let Some(target_ln) = *scroll_to_line {
                        let mut current_pos = 0;
                        for (i, line) in tab.text.lines().enumerate() {
                            if i + 1 == target_ln {
                                let mut state = egui::text_edit::TextEditState::load(ctx, editor_id).unwrap_or_default();
                                let ccursor = egui::text::CCursor::new(current_pos);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                                egui::TextEdit::store_state(ctx, editor_id, state);
                                break;
                            }
                            current_pos += line.len() + 1; // +1 for \n
                        }
                        *scroll_to_line = None;
                        *just_jumped = true;
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
                let min_margin = 48.0;
                let canvas_width = if prefs.zen_mode {
                    800.0f32.min(avail - min_margin * 2.0)
                } else {
                    avail - min_margin * 2.0
                };

                // 1. Outer ScrollArea for global scrollbar
                scroll_area.show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal_top(|ui| {
                        // 2. Centered Editor Column
                        let res = ui.vertical_centered(|ui| {
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

                            if is_focused_pane && *just_jumped {
                                resp.request_focus();
                                resp.scroll_to_me(None);
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
                        });

                        let (out, resp) = res.inner;

                        // 3. Minimap (to the right of editor, inside ScrollArea)
                        let minimap_w = 36.0;
                        let mm_bg = egui::Color32::from_rgb(prefs.theme.bg_main[0], prefs.theme.bg_main[1], prefs.theme.bg_main[2]);
                        let mm_text = egui::Color32::from_rgb(prefs.theme.text_main[0], prefs.theme.text_main[1], prefs.theme.text_main[2]);
                        let mm_accent = egui::Color32::from_rgb(prefs.theme.accent_hl[0], prefs.theme.accent_hl[1], prefs.theme.accent_hl[2]);
                        let minimap_h = ui.available_height().min(400.0);

                        if !prefs.zen_mode && minimap_h > 50.0 {
                            let (minimap_rect, minimap_resp) = ui.allocate_exact_size(
                                egui::vec2(minimap_w, minimap_h),
                                egui::Sense::click(),
                            );
                            let painter = ui.painter().with_clip_rect(minimap_rect);
                            painter.rect_filled(minimap_rect, 2.0, mm_bg.gamma_multiply(0.5));

                            for (ln, lv, _title) in &mm_headings {
                                let y = minimap_rect.top() + (*ln as f32 / total_lines_mm as f32) * minimap_h;
                                let bar_h = 2.0;
                                let bar_w = minimap_w * (0.3 + 0.15 * (*lv as f32).min(4.0));
                                let bar_x = minimap_rect.left() + (minimap_w - bar_w) / 2.0;
                                painter.rect_filled(egui::Rect::from_min_max(egui::pos2(bar_x, y), egui::pos2(bar_x + bar_w, y + bar_h)), 0.0, mm_text.gamma_multiply(0.3));
                            }

                            for label in &mm_labels {
                                let y = minimap_rect.top() + (label.line as f32 / total_lines_mm as f32) * minimap_h;
                                let cat_color = parser::get_label_color(&label.category, &mm_label_types, mm_accent_ui);
                                let dot_color = egui::Color32::from_rgb(cat_color[0], cat_color[1], cat_color[2]);
                                painter.circle_filled(egui::pos2(minimap_rect.center().x, y), 3.0, dot_color);
                            }

                            if minimap_resp.clicked() {
                                if let Some(pos) = minimap_resp.interact_pointer_pos() {
                                    let click_y = pos.y - minimap_rect.top();
                                    let target_line = ((click_y / minimap_h) * total_lines_mm as f32).max(1.0) as usize;
                                    *scroll_to_line = Some(target_line);
                                }
                            }

                            // Viewport indicator
                            let current_line_mm = if let Some(state) = egui::text_edit::TextEditState::load(ctx, editor_id) {
                                if let Some(crange) = state.cursor.char_range() {
                                    let c_idx = crange.primary.index;
                                    let byte_pos = tab.text.char_indices().nth(c_idx).map(|(i, _)| i).unwrap_or(tab.text.len());
                                    tab.text[..byte_pos].lines().count() + 1
                                } else { 0 }
                            } else { 0 };

                            if current_line_mm > 0 {
                                let vy = minimap_rect.top() + (current_line_mm as f32 / total_lines_mm as f32) * minimap_h;
                                let viewport_h = (minimap_h / total_lines_mm as f32 * 20.0).max(6.0).min(minimap_h);
                                painter.rect_stroke(egui::Rect::from_min_max(egui::pos2(minimap_rect.left() + 1.0, vy - viewport_h / 2.0), egui::pos2(minimap_rect.right() - 1.0, vy + viewport_h / 2.0)), 1.0, egui::Stroke::new(1.0, mm_accent.gamma_multiply(0.6)), egui::epaint::StrokeKind::Outside);
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
                    });
                });

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
