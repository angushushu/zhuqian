use eframe::egui;

use crate::app::{ZhuQianEditor, AutocompleteMode, SplitPane};
use crate::parser;
use crate::theme_io::draw_bg_image;

pub(crate) fn render_editor(app: &mut ZhuQianEditor, ctx: &egui::Context) {
    let bg_main = egui::Color32::from_rgb(app.prefs.theme.bg_main[0], app.prefs.theme.bg_main[1], app.prefs.theme.bg_main[2]);
    let text_main = egui::Color32::from_rgb(app.prefs.theme.text_main[0], app.prefs.theme.text_main[1], app.prefs.theme.text_main[2]);
    let accent_hl = egui::Color32::from_rgb(app.prefs.theme.accent_hl[0], app.prefs.theme.accent_hl[1], app.prefs.theme.accent_hl[2]);
    let _s = parser::get_strings(app.prefs.language);

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(bg_main).inner_margin(egui::Margin::symmetric(30, 20)))
        .show(ctx, |ui| {
            if let Some(ref tex) = app.bg_texture.clone() {
                draw_bg_image(ui, tex, ui.max_rect());
            }

            if app.tabs.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(&_s.open_file).size(16.0).color(text_main));
                });
                return;
            }

            let font_size = app.prefs.font_size;
            let font_family = egui::FontFamily::Name(app.prefs.font_name.clone().into());
            let ff = font_family.clone();

            let left_idx = app.active_tab;
            let right_idx_opt = app.split_right_tab;

            let ZhuQianEditor {
                tabs, prefs, scroll_to_line, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, ..
            } = app;

            let render_pane = |
                ui: &mut egui::Ui,
                tab: &mut crate::app::TabData,
                id_suffix: &str,
                is_focused_pane: bool,
                prefs: &mut crate::parser::DisplayPrefs,
                scroll_to_line: &mut Option<usize>,
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
                let editor_id = ui.id().with(id_suffix);

                if is_focused_pane {
                    if let Some(target_ln) = *scroll_to_line {
                        let mut current_pos = 0;
                        for (i, line) in tab.text.lines().enumerate() {
                            if i + 1 == target_ln {
                                let mut state = egui::text_edit::TextEditState::load(ctx, editor_id).unwrap_or_default();
                                let ccursor = egui::text::CCursor::new(current_pos);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                                egui::TextEdit::store_state(ctx, editor_id, state);
                                break;
                            }
                            current_pos += line.len() + 1;
                        }
                        *scroll_to_line = None;
                        *just_jumped = true;
                    }
                }

                scroll_area.show(ui, |ui| {
                    let avail = ui.available_width();
                    let (canvas_width, side_pad) = if prefs.zen_mode {
                        let w = avail.min(800.0);
                        (w, ((avail - w) / 2.0).max(0.0))
                    } else {
                        (avail, 0.0)
                    };

                    // Reserve full width so scroll_area content is correct
                    ui.set_min_width(avail);

                    let text_edit = egui::TextEdit::multiline(&mut tab.text)
                        .id(editor_id)
                        .frame(false)
                        .font(egui::FontId::new(font_size, ff.clone()))
                        .text_color(text_main)
                        .lock_focus(true)
                        .desired_width(canvas_width)
                        .margin(egui::vec2(side_pad, 0.0))
                        .layouter(&mut layouter);

                    let out = text_edit.show(ui);
                    let resp = out.response;

                    if resp.gained_focus() {
                        *active_pane = pane_type;
                    }

                    if is_focused_pane && *just_jumped {
                        resp.request_focus();
                        resp.scroll_to_me(None);
                        *just_jumped = false;
                    }

                    if resp.changed() { tab.is_dirty = true; }

                    // Autocomplete logic
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

                                if found_mode != AutocompleteMode::None && c_idx > start_idx {
                                    let search_str = chars.iter().skip(start_idx + 1).take(c_idx - start_idx - 1).map(|(_, c)| c).collect::<String>();
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

                    // Popup render
                    if is_focused_pane && *ac_mode != AutocompleteMode::None {
                        if let Some(pos) = *ac_pos {
                            let mut options = Vec::new();
                            if *ac_mode == AutocompleteMode::Category {
                                for lt in &prefs.label_types {
                                    if lt.name.to_lowercase().contains(&*ac_search) { options.push(lt.name.clone()); }
                                }
                            } else if *ac_mode == AutocompleteMode::Shortcode {
                                for t in &prefs.tag_codes {
                                    if t.symbol.to_lowercase().contains(&*ac_search) || t.label.to_lowercase().contains(&*ac_search) {
                                        options.push(t.symbol.clone());
                                    }
                                }
                                for r in &prefs.relation_codes {
                                    if r.prefix.to_lowercase().contains(&*ac_search) || r.label.to_lowercase().contains(&*ac_search) {
                                        options.push(r.prefix.clone());
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
                                            chars.splice(replace_start..current_idx, selected.chars());
                                            tab.text = chars.into_iter().collect();
                                            tab.is_dirty = true;

                                            let new_idx = replace_start + selected.chars().count();
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
                                                inner.set_max_width(200.0);
                                                for (i, opt) in options.iter().enumerate() {
                                                    let is_selected = i == *ac_cursor;
                                                    let bg = if is_selected { accent_hl } else { egui::Color32::TRANSPARENT };
                                                    inner.horizontal(|inner_hz| {
                                                        let rt = egui::RichText::new(opt).color(if is_selected { bg_main } else { text_main }).background_color(bg);
                                                        inner_hz.add(egui::Label::new(rt).selectable(false).sense(egui::Sense::click()))
                                                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                                                    }).response.highlight();
                                                }
                                            });
                                        });
                                }
                            }
                        }
                    }
                }); // scroll_area
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

                        render_pane(&mut cols[0], left_tab, "editor_main", left_is_focused, prefs, scroll_to_line, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, SplitPane::Left);
                        render_pane(&mut cols[1], right_tab, "editor_split", right_is_focused, prefs, scroll_to_line, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, SplitPane::Right);
                    });
                } else {
                    if let Some(tab) = tabs.get_mut(left_idx) {
                        let is_focused = *active_pane == SplitPane::Left;
                        render_pane(ui, tab, "editor_main", is_focused, prefs, scroll_to_line, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, SplitPane::Left);
                    }
                }
            } else {
                if left_idx < tabs.len() {
                    if let Some(tab) = tabs.get_mut(left_idx) {
                        let is_focused = *active_pane == SplitPane::Left;
                        render_pane(ui, tab, "editor_main", is_focused, prefs, scroll_to_line, just_jumped, ac_mode, ac_search, ac_cursor, ac_pos, active_pane, SplitPane::Left);
                    }
                }
            }
        });
}
