use serde::{Serialize, Deserialize};
use regex::Regex;
use wasm_bindgen::prelude::*;

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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    pub name: String,
    pub pattern: String,
    pub color: [u8; 3],
    pub bold: bool,
    pub is_background: bool,
    #[serde(skip)]
    compiled: Option<Regex>,
}

impl HighlightRule {
    pub fn new(name: &str, pattern: &str, color: [u8; 3], bold: bool, is_bg: bool) -> Self {
        let compiled = Regex::new(pattern).ok();
        Self { name: name.into(), pattern: pattern.into(), color, bold, is_background: is_bg, compiled }
    }
    pub fn compile(&mut self) { self.compiled = Regex::new(&self.pattern).ok(); }
    pub fn regex(&self) -> Option<&Regex> { self.compiled.as_ref() }
}

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
    pub highlight_rules: Vec<HighlightRule>,
}

impl ZqTheme {
    pub fn new_light() -> Self {
        let mut rules = vec![
            HighlightRule::new("一层括号", r"\[[^\[\]]*\]", [230, 80, 80], false, true),
            HighlightRule::new("二层括号", r"\[\[[^\[\]]*\]\]", [80, 200, 80], false, true),
            HighlightRule::new("三层括号", r"\[\[\[[^\[\]]*\]\]\]", [80, 80, 230], false, true),
        ];
        for r in &mut rules { r.compile(); }
        Self {
            name: "竹签默认 (浅)".into(),
            base: BaseTheme::Light,
            bg_main:             [248, 246, 235],
            bg_side:             [238, 236, 225],
            text_main:           [30,  30,  30 ],
            text_side:           [45,  45,  45 ],
            accent_ui:           [142, 172, 80 ], // Bamboo Green
            accent_hl:           [180, 160, 0  ],
            highlight_rules:     rules,
        }
    }

    pub fn new_dark() -> Self {
        let mut rules = vec![
            HighlightRule::new("一层括号", r"\[[^\[\]]*\]", [180, 60, 60], false, true),
            HighlightRule::new("二层括号", r"\[\[[^\[\]]*\]\]", [60, 150, 60], false, true),
            HighlightRule::new("三层括号", r"\[\[\[[^\[\]]*\]\]\]", [60, 60, 180], false, true),
        ];
        for r in &mut rules { r.compile(); }
        Self {
            name: "竹签默认 (深)".into(),
            base: BaseTheme::Dark,
            bg_main:             [30,  30,  30 ],
            bg_side:             [45,  45,  45 ],
            text_main:           [220, 220, 220],
            text_side:           [180, 180, 180],
            accent_ui:           [160, 200, 60 ], // Vibrant Bamboo Green
            accent_hl:           [255, 230, 0  ],
            highlight_rules:     rules,
        }
    }
}

impl Default for ZqTheme {
    fn default() -> Self {
        Self::new_light()
    }
}#[derive(Clone, Serialize, Deserialize)]
pub struct HighlightRuleSet {
    pub name: String,
    pub rules: Vec<HighlightRule>,
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

pub fn deserialize_zq_file(raw: &str) -> (String, Option<DisplayPrefs>) {
    if let Some(rest) = raw.strip_prefix(ZQ_META_PREFIX) {
        if let Some(idx) = rest.find(ZQ_META_SUFFIX) {
            let meta = &rest[..idx];
            let body = &rest[idx + ZQ_META_SUFFIX.len()..];
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

#[derive(Clone, Serialize, Deserialize)]
pub struct DisplayPrefs {
    pub font_size: f32,
    pub font_name: String,
    pub markdown_render: bool,
    pub bg_image_path: Option<String>,
    pub panel_bg_image_path: Option<String>,
    pub language: Language,
    pub theme: ZqTheme,
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
pub fn parse_to_spans_wasm(text: &str, rules_json: &str, accent_json: &str) -> Result<JsValue, JsValue> {
    let mut rules: Vec<HighlightRule> = serde_json::from_str(rules_json)
        .map_err(|e| JsValue::from_str(&format!("Regex parse error: {}", e)))?;
    let ctx: serde_json::Value = serde_json::from_str(accent_json).unwrap_or_default();
    let acc_r = ctx["accent_hl"][0].as_u64().unwrap_or(180) as u8;
    let acc_g = ctx["accent_hl"][1].as_u64().unwrap_or(160) as u8;
    let acc_b = ctx["accent_hl"][2].as_u64().unwrap_or(0) as u8;
    
    let txt_r = ctx["text_main"][0].as_u64().unwrap_or(30) as u8;
    let txt_g = ctx["text_main"][1].as_u64().unwrap_or(30) as u8;
    let txt_b = ctx["text_main"][2].as_u64().unwrap_or(30) as u8;

    // Compile rules
    for r in &mut rules { r.compile(); }

    let mut spans = parse_markdown_to_spans(text, &rules, [acc_r, acc_g, acc_b], [txt_r, txt_g, txt_b]);
    
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
pub fn get_default_theme_json() -> String {
    serde_json::to_string(&ZqTheme::default()).unwrap_or_default()
}

// ── Portable Highlighting ──

pub fn parse_markdown_to_spans(text: &str, rules: &[HighlightRule], theme_accent: [u8; 3], theme_text: [u8; 3]) -> Vec<StyledSpan> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let len = text.len();
    if len == 0 { return vec![]; }

    let mut fg:      Vec<Option<[u8; 3]>> = vec![None;  len];
    let mut bg:      Vec<Option<[u8; 3]>> = vec![None;  len];
    let mut sizes:   Vec<f32>             = vec![1.0; len];
    let mut strikes: Vec<bool>            = vec![false; len];
    let mut italics: Vec<bool>            = vec![false; len];
    let mut bolds:   Vec<bool>            = vec![false; len];

    // ── Custom rules ──
    for rule in rules {
        if let Some(re) = rule.regex() {
            let c = rule.color;
            for mat in re.find_iter(text) {
                for i in mat.start()..mat.end() {
                    if rule.is_background { bg[i] = Some(c); } else { fg[i] = Some(c); }
                    if rule.bold { bolds[i] = true; }
                }
            }
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
        let changed = fg[i] != fg[start] || bg[i] != bg[start] || sizes[i] != sizes[start] || bolds[i] != bolds[start] || italics[i] != italics[start] || strikes[i] != strikes[start];
        if changed {
            spans.push(StyledSpan {
                start, 
                end: i,
                fg: fg[start], bg: bg[start],
                size_mult: sizes[start],
                bold: bolds[start], italic: italics[start], strikethrough: strikes[start],
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
    });

    spans
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LangStrings {
    pub app_title: String,
    pub file: String,
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
}

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
