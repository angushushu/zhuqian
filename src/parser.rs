use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{Color32, FontId, FontFamily};
use regex::Regex;
use serde::{Serialize, Deserialize};

// ── Highlight Rule ──

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

// ── Color helpers ──

fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
        (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
        (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
    )
}

// ──────────────────────────────────────────────────
// Highlight engine (custom rules only, no markdown)
// ──────────────────────────────────────────────────

pub fn highlight(
    text: &str, rules: &[HighlightRule],
    font_size: f32, font_family: FontFamily, default_color: Color32,
) -> LayoutJob {
    let len = text.len();
    if len == 0 { return LayoutJob::default(); }

    let mut fg: Vec<Option<Color32>> = vec![None; len];
    let mut bg: Vec<Option<Color32>> = vec![None; len];

    for rule in rules {
        if let Some(re) = rule.regex() {
            let c = Color32::from_rgb(rule.color[0], rule.color[1], rule.color[2]);
            for mat in re.find_iter(text) {
                for i in mat.start()..mat.end() {
                    if rule.is_background { bg[i] = Some(c); } else { fg[i] = Some(c); }
                }
            }
        }
    }

    let sizes   = vec![font_size; len];
    let strikes = vec![false; len];
    let italics = vec![false; len];
    build_job_with_styles(text, &fg, &bg, &sizes, &strikes, &italics, &font_family, default_color)
}

// ──────────────────────────────────────────────────
// Markdown-aware highlight (uses pulldown-cmark)
// IMPORTANT: preserves every character of input text
// (TextEdit requires LayoutJob to match input exactly)
// ──────────────────────────────────────────────────

pub fn highlight_markdown(
    text: &str,
    rules: &[HighlightRule],
    font_size: f32,
    font_family: FontFamily,
    default_color: Color32,
    accent: Color32,
) -> LayoutJob {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let len = text.len();
    if len == 0 { return LayoutJob::default(); }

    let mut fg:      Vec<Option<Color32>> = vec![None;  len];
    let mut bg:      Vec<Option<Color32>> = vec![None;  len];
    let mut sizes:   Vec<f32>             = vec![font_size; len];
    let mut strikes: Vec<bool>            = vec![false; len];
    let mut italics: Vec<bool>            = vec![false; len];

    // ── Custom rules (lowest priority, overwritten by markdown) ──
    for rule in rules {
        if let Some(re) = rule.regex() {
            let c = Color32::from_rgb(rule.color[0], rule.color[1], rule.color[2]);
            for mat in re.find_iter(text) {
                for i in mat.start()..mat.end() {
                    if rule.is_background { bg[i] = Some(c); } else { fg[i] = Some(c); }
                }
            }
        }
    }

    // ── Derived theme-aware colors ──
    // Delimiters: very faint (40% opacity of text color)
    let dim = Color32::from_rgba_unmultiplied(
        default_color.r(), default_color.g(), default_color.b(), 100,
    );
    // Heading text: accent
    let accent_col = accent;
    // Bold: accent blended 80% toward accent (vivid)
    let bold_col = accent;
    // Italic: slight accent tint
    let ital_col = blend(default_color, accent, 0.3);
    // Bold+italic: stronger tint
    let bold_ital_col = blend(default_color, accent, 0.55);
    // Inline code fg: warm brownish (theme-relative)
    let code_fg_col = blend(default_color, Color32::from_rgb(190, 85, 15), 0.55);
    // Inline code bg: subtle tint
    let code_bg_col = Color32::from_rgba_unmultiplied(90, 70, 30, 55);
    // Blockquote text: desaturated toward gray
    let quote_col = blend(default_color, Color32::from_rgb(130, 130, 130), 0.4);
    // Link text: blue tint
    let link_col = blend(default_color, Color32::from_rgb(50, 110, 220), 0.6);

    // Heading font sizes: H1..H6
    let h_sizes = [
        font_size * 1.90,
        font_size * 1.60,
        font_size * 1.35,
        font_size * 1.17,
        font_size * 1.07,
        font_size * 1.00,
    ];

    // ── Markdown parse ──
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    // Active style counters
    let mut heading_size: Option<f32> = None;
    let mut strong:    u32 = 0;
    let mut emph:      u32 = 0;
    let mut strike:    u32 = 0;
    let mut code_block       = false;
    let mut link_depth: u32  = 0;
    let mut quote_depth: u32 = 0;

    // Helper: write `dim` to bytes in s..e only if not already set by custom rules
    // (inline — we'll accept a small repeat rather than a closure borrow conflict)
    macro_rules! dim_range {
        ($s:expr, $e:expr) => {
            for i in ($s)..($e).min(len) {
                if fg[i].is_none() { fg[i] = Some(dim); }
            }
        };
    }

    for (event, range) in Parser::new_ext(text, opts).into_offset_iter() {
        let s = range.start.min(len);
        let e = range.end.min(len);

        match event {
            // ── Block: Headings ──
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = (level as usize).saturating_sub(1).min(5);
                heading_size = Some(h_sizes[lvl]);
                // Dim the leading `# ` series (hash_count + 1 space)
                let hashes = level as usize;
                for i in s..(s + hashes + 1).min(e).min(len) {
                    fg[i] = Some(dim);
                    sizes[i] = h_sizes[lvl];
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                heading_size = None;
            }

            // ── Block: Blockquote ──
            Event::Start(Tag::BlockQuote(_)) => { quote_depth += 1; }
            Event::End(TagEnd::BlockQuote(_)) => { quote_depth = quote_depth.saturating_sub(1); }

            // ── Block: Code block ──
            Event::Start(Tag::CodeBlock(_)) => {
                code_block = true;
                dim_range!(s, e);
            }
            Event::End(TagEnd::CodeBlock) => {
                code_block = false;
                dim_range!(s, e);
            }

            // ── Block: List item marker ──
            Event::Start(Tag::Item) => {
                // Highlight the bullet/number glyph(s)
                for i in s..(s + 3).min(e).min(len) {
                    fg[i] = Some(accent_col);
                }
            }

            // ── Inline: Strong (bold) ──
            Event::Start(Tag::Strong) => {
                strong += 1;
                dim_range!(s, e); // opening ** or __
            }
            Event::End(TagEnd::Strong) => {
                strong = strong.saturating_sub(1);
                dim_range!(s, e); // closing ** or __
            }

            // ── Inline: Emphasis (italic) ──
            Event::Start(Tag::Emphasis) => {
                emph += 1;
                dim_range!(s, e); // opening * or _
            }
            Event::End(TagEnd::Emphasis) => {
                emph = emph.saturating_sub(1);
                dim_range!(s, e); // closing * or _
            }

            // ── Inline: Strikethrough ──
            Event::Start(Tag::Strikethrough) => {
                strike += 1;
                dim_range!(s, e);
            }
            Event::End(TagEnd::Strikethrough) => {
                strike = strike.saturating_sub(1);
                dim_range!(s, e);
            }

            // ── Inline: Link ──
            Event::Start(Tag::Link { .. }) => {
                link_depth += 1;
                dim_range!(s, e); // whole span dims first; text overrides below
            }
            Event::End(TagEnd::Link) => {
                link_depth = link_depth.saturating_sub(1);
                dim_range!(s, e);
            }

            // ── Inline: Image ──
            Event::Start(Tag::Image { .. }) => { dim_range!(s, e); }
            Event::End(TagEnd::Image) => { dim_range!(s, e); }

            // ── Text content ──
            Event::Text(_) => {
                for i in s..e.min(len) {
                    // Heading takes priority
                    if let Some(hs) = heading_size {
                        sizes[i] = hs;
                        fg[i] = Some(accent_col);
                        continue;
                    }
                    // Fenced code block content
                    if code_block {
                        fg[i] = Some(code_fg_col);
                        bg[i] = Some(code_bg_col);
                        continue;
                    }
                    // Inline formatting — combinable
                    if emph > 0   { italics[i] = true; }
                    if strike > 0 { strikes[i] = true; }

                    // Foreground color (highest wins)
                    if link_depth > 0 {
                        fg[i] = Some(link_col);
                    } else if strong > 0 && emph > 0 {
                        fg[i] = Some(bold_ital_col);
                    } else if strong > 0 {
                        fg[i] = Some(bold_col);
                    } else if emph > 0 {
                        fg[i] = Some(ital_col);
                    } else if quote_depth > 0 {
                        if fg[i].is_none() { fg[i] = Some(quote_col); }
                    }
                    // strike with no other decoration: leave fg as default
                }
            }

            // ── Inline code span ──
            Event::Code(_) => {
                for i in s..e.min(len) {
                    fg[i] = Some(code_fg_col);
                    bg[i] = Some(code_bg_col);
                }
                // Make backtick delimiters dim
                if s < len { fg[s] = Some(dim); }
                let last = e.min(len).saturating_sub(1);
                if last > s { fg[last] = Some(dim); }
            }

            // ── Horizontal rule ──
            Event::Rule => { dim_range!(s, e); }

            // ── Task list checkbox ──
            Event::TaskListMarker(checked) => {
                let col = if checked { accent_col } else { dim };
                for i in s..e.min(len) { fg[i] = Some(col); }
            }

            _ => {}
        }
    }

    // ── Post-pass: blockquote '>' markers aren't emitted as text events ──
    // Scan lines manually and colour the leading '>' in accent.
    {
        let mut pos = 0;
        for line in text.split('\n') {
            let trimmed = line.trim_start();
            if trimmed.starts_with('>') {
                let gt = pos + (line.len() - trimmed.len());
                if gt < len { fg[gt] = Some(accent_col); }
            }
            pos += line.len() + 1;
        }
    }

    build_job_with_styles(text, &fg, &bg, &sizes, &strikes, &italics, &font_family, default_color)
}

// ──────────────────────────────────────────────────
// LayoutJob builder
// ──────────────────────────────────────────────────

fn build_job_with_styles(
    text: &str,
    fg: &[Option<Color32>], bg: &[Option<Color32>],
    sizes: &[f32], strikes: &[bool], italics: &[bool],
    font_family: &FontFamily, default_color: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let len = text.len();
    if len == 0 { return job; }

    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut span_start = 0;

    for idx in 0..chars.len() {
        let (bp, _) = chars[idx];
        let next_bp = if idx + 1 < chars.len() { chars[idx + 1].0 } else { len };

        let flush = if idx + 1 < chars.len() {
            let nb = chars[idx + 1].0;
            fg[nb] != fg[bp]
                || bg[nb] != bg[bp]
                || sizes[nb] != sizes[bp]
                || strikes[nb] != strikes[bp]
                || italics[nb] != italics[bp]
        } else {
            true
        };

        if flush {
            let span = &text[span_start..next_bp];
            let mut fmt = TextFormat {
                font_id: FontId::new(sizes[bp], font_family.clone()),
                color: fg[bp].unwrap_or(default_color),
                background: bg[bp].unwrap_or(Color32::TRANSPARENT),
                italics: italics[bp],
                ..Default::default()
            };
            if strikes[bp] {
                fmt.strikethrough = eframe::egui::Stroke::new(1.0, fmt.color);
            }
            job.append(span, 0.0, fmt);
            span_start = next_bp;
        }
    }
    job
}

// ── Document stats ──

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
