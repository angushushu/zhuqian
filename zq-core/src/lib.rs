use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

pub mod label;
pub use label::*;

pub mod template;
pub use template::*;



pub const ZQ_META_PREFIX: &str = "---zq-meta---\n";
pub const ZQ_META_SUFFIX: &str = "\n---end-meta---\n";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StyledSpan {
    pub start: usize,
    pub end: usize,
    pub fg: Option<[u8; 3]>,
    pub bg: Option<[u8; 3]>,
    pub size_mult: f32, // multiplier of base font size
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub is_hidden: bool,
}

// Legacy HighlightRule removed, preserving simple StyledSpan


#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BaseTheme { Light, Dark }

#[derive(Clone, Serialize, Deserialize)]
pub struct ZqTheme {
    pub name: String,
    pub base: BaseTheme,
    pub bg_main: [u8; 3],
    pub bg_side: [u8; 3],
    pub text_main: [u8; 3],
    pub text_side: [u8; 3],
    pub accent_ui: [u8; 3],
    pub accent_hl: [u8; 3],
    #[serde(default = "default_level_colors")]
    pub level_colors: Vec<[u8; 3]>,
}

fn default_level_colors() -> Vec<[u8; 3]> {
    vec![
        [230, 80, 80],
        [80, 200, 80],
        [80,  80, 230],
        [230, 180, 50],
        [180, 80, 200],
        [80, 200, 200],
    ]
}

impl ZqTheme {
    pub fn new_light() -> Self {
        Self {
            name: "竹签默认 (浅)".into(),
            base: BaseTheme::Light,
            bg_main:             [248, 246, 235],
            bg_side:             [238, 236, 225],
            text_main:           [30,  30,  30 ],
            text_side:           [45,  45,  45 ],
            accent_ui:           [142, 172, 80 ], // Bamboo Green
            accent_hl:           [180, 160, 0  ],
            level_colors:        default_level_colors(),
        }
    }

    pub fn new_dark() -> Self {
        let mut dark_colors = default_level_colors();
        dark_colors[0] = [180, 60, 60];
        dark_colors[1] = [60, 150, 60];
        dark_colors[2] = [60, 60, 180];
        Self {
            name: "竹签默认 (深)".into(),
            base: BaseTheme::Dark,
            bg_main:             [30,  30,  30 ],
            bg_side:             [45,  45,  45 ],
            text_main:           [220, 220, 220],
            text_side:           [180, 180, 180],
            accent_ui:           [160, 200, 60 ], // Vibrant Bamboo Green
            accent_hl:           [255, 230, 0  ],
            level_colors:        dark_colors,
        }
    }
}

impl Default for ZqTheme {
    fn default() -> Self {
        Self::new_light()
    }
}

pub fn strip_labels(text: &str) -> String {
    let labels = label::parse_semantic_labels(text);
    label::strip_semantic_labels(text, &labels)
}

pub fn extract_headings(text: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with('#') {
            let lv = t.chars().take_while(|c| *c == '#').count().min(6);
            let title = t[lv..].trim().to_string();
            if !title.is_empty() { out.push((i + 1, lv, title)); }
        }
    }
    out
}

pub fn extract_semantic_labels(text: &str) -> Vec<label::SemanticLabel> {
    label::parse_semantic_labels(text)
}

pub fn deserialize_zq_file(raw: &str) -> (String, Option<DisplayPrefs>) {
    if raw.starts_with("---zq-meta---") {
        if let Some(suffix_idx) = raw.find("---end-meta---") {
            let prefix_end = raw.find('\n').unwrap_or(0) + 1;
            let meta = &raw[prefix_end..suffix_idx].trim();
            let body_start = suffix_idx + "---end-meta---".len();
            let body = &raw[body_start..].trim_start_matches(|c| c == '\r' || c == '\n');
            if let Ok(prefs) = serde_json::from_str::<DisplayPrefs>(meta) {
                return (body.to_string(), Some(prefs));
            }
        }
    }
    (raw.to_string(), None)
}

pub fn serialize_zq_file(text: &str, prefs: &DisplayPrefs) -> String {
    let meta = serde_json::to_string_pretty(prefs).unwrap_or_default();
    format!("{}{}{}{}", ZQ_META_PREFIX, meta, ZQ_META_SUFFIX, text)
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Language { #[default] Zh, En }

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct TagCode {
    pub symbol: String,
    pub label: String,
    pub color: [f32; 3],
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationCode {
    pub prefix: String,
    pub label: String,
    pub color: [f32; 3],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DisplayPrefs {
    pub font_size: f32,
    pub font_name: String,
    pub markdown_render: bool,
    pub bg_image_path: Option<String>,
    pub panel_bg_image_path: Option<String>,
    pub language: Language,
    pub theme: ZqTheme,
    pub hide_labels: bool,
    #[serde(default = "default_zen_mode")]
    pub zen_mode: bool,
    #[serde(default = "default_delimiter")]
    pub label_delimiter: [String; 2],
    #[serde(default = "default_label_types")]
    pub label_types: Vec<LabelType>,
    #[serde(default = "default_tag_codes")]
    pub tag_codes: Vec<TagCode>,
    #[serde(default = "default_relation_codes")]
    pub relation_codes: Vec<RelationCode>,
}

fn default_tag_codes() -> Vec<TagCode> {
    vec![
        TagCode { symbol: "+".into(), label: "正面/支持".into(), color: [0.3, 0.8, 0.3] },
        TagCode { symbol: "-".into(), label: "负面/反对".into(), color: [0.8, 0.3, 0.3] },
        TagCode { symbol: "?".into(), label: "存疑/检查".into(), color: [0.8, 0.8, 0.3] },
        TagCode { symbol: "~".into(), label: "让步".into(), color: [0.4, 0.6, 0.8] },
    ]
}

fn default_relation_codes() -> Vec<RelationCode> {
    vec![
        RelationCode { prefix: "rf".into(), label: "驳斥".into(), color: [0.8, 0.2, 0.2] },
        RelationCode { prefix: "sp".into(), label: "支持".into(), color: [0.2, 0.8, 0.2] },
        RelationCode { prefix: "@".into(), label: "引用".into(), color: [0.4, 0.4, 0.8] },
    ]
}

fn default_zen_mode() -> bool { true }

fn default_delimiter() -> [String; 2] {
    [DEFAULT_OPEN.into(), DEFAULT_CLOSE.into()]
}

fn default_label_types() -> Vec<LabelType> {
    vec![LabelType::note_type()]
}

impl Default for DisplayPrefs {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            font_name: "times".into(),
            markdown_render: false,
            bg_image_path: None,
            panel_bg_image_path: None,
            language: Language::default(),
            theme: ZqTheme::default(),
            hide_labels: false,
            zen_mode: true,
            label_delimiter: default_delimiter(),
            label_types: default_label_types(),
            tag_codes: default_tag_codes(),
            relation_codes: default_relation_codes(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DocStats {
    pub chars: usize,
    pub chars_no_space: usize,
    pub words: usize,
    pub lines: usize,
    pub labels: usize,
}

pub fn compute_stats(text: &str) -> DocStats {
    let chars = text.chars().count();
    let chars_no_space = text.chars().filter(|c| !c.is_whitespace()).count();
    let lines = text.lines().count().max(1);
    let words = text.split_whitespace().count();
    let mut depth = 0i32;
    let mut labels = 0usize;
    for c in text.chars() {
        if c == '[' { depth += 1; }
        else if c == ']' && depth > 0 { labels += 1; depth -= 1; }
    }
    DocStats { chars, chars_no_space, words, lines, labels }
}

// ── Wasm Exports ──

#[wasm_bindgen]
pub fn parse_to_spans_wasm(text: &str, accent_json: &str) -> Result<JsValue, JsValue> {
    let ctx: serde_json::Value = serde_json::from_str(accent_json).unwrap_or_default();
    let acc_r = ctx["accent_hl"][0].as_u64().unwrap_or(180) as u8;
    let acc_g = ctx["accent_hl"][1].as_u64().unwrap_or(160) as u8;
    let acc_b = ctx["accent_hl"][2].as_u64().unwrap_or(0) as u8;
    
    let txt_r = ctx["text_main"][0].as_u64().unwrap_or(30) as u8;
    let txt_g = ctx["text_main"][1].as_u64().unwrap_or(30) as u8;
    let txt_b = ctx["text_main"][2].as_u64().unwrap_or(30) as u8;

    let mut level_colors = default_level_colors();
    if let Some(arr) = ctx["level_colors"].as_array() {
        for (i, v) in arr.iter().enumerate().take(6) {
            level_colors[i] = [
                v[0].as_u64().unwrap_or(0) as u8,
                v[1].as_u64().unwrap_or(0) as u8,
                v[2].as_u64().unwrap_or(0) as u8
            ];
        }
    }

    let hide_labels = ctx["hide_labels"].as_bool().unwrap_or(false);

    let mut spans = parse_markdown_to_spans(text, &level_colors, [acc_r, acc_g, acc_b], [txt_r, txt_g, txt_b], hide_labels);
    
    // Map byte offsets to UTF-16 code unit offsets for VS Code
    let len = text.len();
    let mut byte_to_u16 = vec![0; len + 1];
    let mut current_u16 = 0;
    for (i, c) in text.char_indices() {
        byte_to_u16[i] = current_u16;
        current_u16 += c.len_utf16();
    }
    byte_to_u16[len] = current_u16;

    for span in &mut spans {
        span.start = byte_to_u16[span.start.min(len)] as usize;
        span.end = byte_to_u16[span.end.min(len)] as usize;
    }

    Ok(serde_wasm_bindgen::to_value(&spans)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?)
}

#[wasm_bindgen]
pub fn extract_semantic_labels_wasm(text: &str) -> Result<JsValue, JsValue> {
    let labels = extract_semantic_labels(text);
    Ok(serde_wasm_bindgen::to_value(&labels)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?)
}

#[wasm_bindgen]
pub fn strip_labels_wasm(text: &str) -> String {
    strip_labels(text)
}

#[wasm_bindgen]
pub fn get_default_theme_json() -> String {
    serde_json::to_string(&ZqTheme::default()).unwrap_or_default()
}

#[wasm_bindgen]
pub fn parse_semantic_labels_wasm(text: &str) -> Result<JsValue, JsValue> {
    let labels = label::parse_semantic_labels(text);
    Ok(serde_wasm_bindgen::to_value(&labels)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?)
}

#[wasm_bindgen]
pub fn strip_semantic_labels_wasm(text: &str) -> String {
    let labels = label::parse_semantic_labels(text);
    label::strip_semantic_labels(text, &labels)
}

// ── Portable Highlighting ──

pub fn parse_markdown_to_spans(text: &str, level_colors: &[[u8; 3]], theme_accent: [u8; 3], _theme_text: [u8; 3], hide_labels: bool) -> Vec<StyledSpan> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let len = text.len();
    if len == 0 { return vec![]; }

    let mut fg:      Vec<Option<[u8; 3]>> = vec![None;  len];
    let mut bg:      Vec<Option<[u8; 3]>> = vec![None;  len];
    let mut sizes:   Vec<f32>             = vec![1.0; len];
    let mut strikes: Vec<bool>            = vec![false; len];
    let mut italics: Vec<bool>            = vec![false; len];
    let mut bolds:   Vec<bool>            = vec![false; len];
    let mut hiddens: Vec<bool>            = vec![false; len];

    // ── Semantic AST Highlighting ──
    let labels = label::parse_semantic_labels(text);
    for label in &labels {
        if hide_labels {
            for i in label.start_byte.min(len)..label.end_byte.min(len) {
                hiddens[i] = true;
            }
            continue;
        }
        
        let c = if let Some(col) = level_colors.get(label.depth.saturating_sub(1)) {
            *col
        } else {
            level_colors.last().copied().unwrap_or([150, 150, 150])
        };

        for i in label.start_byte.min(len)..label.end_byte.min(len) {
            bg[i] = Some(c);
        }
    }

    // ── Markdown ──
    let dim = [150, 150, 150]; // Hardcoded placeholder for dim, ideally based on theme
    let accent = theme_accent;
    let h_sizes = [1.90, 1.60, 1.35, 1.17, 1.07, 1.00];

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let mut heading_size: Option<f32> = None;
    let mut strong: u32 = 0;
    let mut emph: u32 = 0;
    let mut strike: u32 = 0;
    let mut code_block = false;
    let mut link_depth: u32 = 0;
    let mut quote_depth: u32 = 0;

    for (event, range) in Parser::new_ext(text, opts).into_offset_iter() {
        let s = range.start.min(len);
        let e = range.end.min(len);
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = (level as usize).saturating_sub(1).min(5);
                heading_size = Some(h_sizes[lvl]);
                let hashes = level as usize;
                for i in s..(s + hashes + 1).min(e) {
                    fg[i] = Some(dim);
                    sizes[i] = h_sizes[lvl];
                }
            }
            Event::End(TagEnd::Heading(_)) => { heading_size = None; }
            Event::Start(Tag::BlockQuote(_)) => { quote_depth += 1; }
            Event::End(TagEnd::BlockQuote(_)) => { quote_depth = quote_depth.saturating_sub(1); }
            Event::Start(Tag::CodeBlock(_)) => { code_block = true; }
            Event::End(TagEnd::CodeBlock) => { code_block = false; }
            Event::Start(Tag::Strong) => { strong += 1; }
            Event::End(TagEnd::Strong) => { strong = strong.saturating_sub(1); }
            Event::Start(Tag::Emphasis) => { emph += 1; }
            Event::End(TagEnd::Emphasis) => { emph = emph.saturating_sub(1); }
            Event::Start(Tag::Strikethrough) => { strike += 1; }
            Event::End(TagEnd::Strikethrough) => { strike = strike.saturating_sub(1); }
            Event::Start(Tag::Link { .. }) => { link_depth += 1; }
            Event::End(TagEnd::Link) => { link_depth = link_depth.saturating_sub(1); }
            Event::Text(_) => {
                for i in s..e {
                    if let Some(hs) = heading_size {
                        sizes[i] = hs;
                        fg[i] = Some(accent);
                        continue;
                    }
                    if code_block {
                        fg[i] = Some([190, 85, 15]);
                        bg[i] = Some([240, 240, 230]);
                        continue;
                    }
                    if strong > 0 { bolds[i] = true; }
                    if emph > 0 { italics[i] = true; }
                    if strike > 0 { strikes[i] = true; }
                    if link_depth > 0 { fg[i] = Some([50, 110, 220]); }
                }
            }
            Event::Code(_) => {
                for i in s..e {
                    fg[i] = Some([190, 85, 15]);
                    bg[i] = Some([240, 240, 230]);
                }
            }
            _ => {}
        }
    }

    // Build spans
    let mut spans = Vec::new();
    if len == 0 { return spans; }

    let mut start = 0;
    for i in 1..len {
        let changed = fg[i] != fg[start] || bg[i] != bg[start] || sizes[i] != sizes[start] 
            || bolds[i] != bolds[start] || italics[i] != italics[start] || strikes[i] != strikes[start]
            || hiddens[i] != hiddens[start];
        if changed {
            spans.push(StyledSpan {
                start, 
                end: i,
                fg: fg[start], bg: bg[start],
                size_mult: sizes[start],
                bold: bolds[start], italic: italics[start], strikethrough: strikes[start],
                is_hidden: hiddens[start],
            });
            start = i;
        }
    }
    spans.push(StyledSpan {
        start, 
        end: len,
        fg: fg[start], bg: bg[start],
        size_mult: sizes[start],
        bold: bolds[start], italic: italics[start], strikethrough: strikes[start],
        is_hidden: hiddens[start],
    });

    spans
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LangStrings {
    pub app_title: String,
    pub file: String,
    pub new_file: String,
    pub save: String,
    pub save_as: String,
    pub close_tab: String,
    pub exit: String,
    pub view: String,
    pub hide_sidebar: String,
    pub show_sidebar: String,
    pub hide_settings: String,
    pub show_settings: String,
    pub files: String,
    pub outline: String,
    pub semantic: String,
    pub logic_links: String,
    pub display_settings: String,
    pub font: String,
    pub font_size: String,
    pub editor_bg: String,
    pub panel_bg: String,
    pub editor_text_color: String,
    pub panel_text_color: String,
    pub accent_ui_color: String,
    pub accent_hl_color: String,
    pub markdown_render: String,
    pub theme_section: String,
    pub load_theme: String,
    pub save_theme: String,
    pub delete_theme: String,
    pub theme_name: String,
    pub highlight_rules: String,
    pub new_rule: String,
    pub add: String,
    pub bg_image: String,
    pub editor_bg_img: String,
    pub panel_bg_img: String,
    pub clear: String,
    pub language: String,
    pub empty: String,
    pub open_file: String,
    pub open_file_btn: String,
    pub open_folder_btn: String,
    pub lines: String,
    pub chars: String,
    pub words: String,
    pub labels: String,
    pub save_rules: String,
    pub rules_name: String,
    pub copy_clean: String,
    pub hide_labels: String,
    pub logic_topology: String,
    // Label management
    #[serde(default = "default_str")]
    pub labels_tab: String,
    #[serde(default = "default_str")]
    pub label_types: String,
    #[serde(default = "default_str")]
    pub delimiter: String,
    #[serde(default = "default_str")]
    pub add_type: String,
    #[serde(default = "default_str")]
    pub type_name: String,
    #[serde(default = "default_str")]
    pub description: String,
    
    // Command Palette & Help
    #[serde(default = "default_str")]
    pub cmd_hint: String,
    #[serde(default = "default_str")]
    pub zen_mode_on: String,
    #[serde(default = "default_str")]
    pub zen_mode_off: String,
    #[serde(default = "default_str")]
    pub labels_on: String,
    #[serde(default = "default_str")]
    pub labels_off: String,
    #[serde(default = "default_str")]
    pub help_shortcuts: String,
    #[serde(default = "default_str")]
    pub help_syntax: String,
    #[serde(default = "default_str")]
    pub links: String,
    #[serde(default = "default_str")]
    pub templates: String,
    #[serde(default = "default_str")]
    pub apply_template: String,
    
    // Settings & Sidebar tooltips
    #[serde(default = "default_str")]
    pub search_font: String,
    #[serde(default = "default_str")]
    pub dict_tab: String,
    #[serde(default = "default_str")]
    pub tag_codes: String,
    #[serde(default = "default_str")]
    pub tag_codes_desc: String,
    #[serde(default = "default_str")]
    pub rel_codes: String,
    #[serde(default = "default_str")]
    pub rel_codes_desc: String,
    #[serde(default = "default_str")]
    pub shortcode: String,
    #[serde(default = "default_str")]
    pub add_tag: String,
    #[serde(default = "default_str")]
    pub add_rel: String,
    #[serde(default = "default_str")]
    pub semantic_header: String,
    #[serde(default = "default_str")]
    pub topology_header: String,
}

fn default_str() -> String { String::new() }

impl LangStrings {
    pub fn load(lang: Language) -> Self {
        let json = match lang {
            Language::Zh => include_str!("../lang/zh.json"),
            Language::En => include_str!("../lang/en.json"),
        };
        serde_json::from_str(json).expect("Failed to parse embedded language file")
    }
}

pub fn get_strings(lang: Language) -> LangStrings {
    LangStrings::load(lang)
}
