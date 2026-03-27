use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{Color32, FontId, FontFamily};
pub use zq_core::{
    HighlightRule, StyledSpan, DocStats, compute_stats, parse_markdown_to_spans, 
    ZqTheme, DisplayPrefs, Language, BaseTheme, LangStrings, get_strings,
    HighlightRuleSet, extract_headings, deserialize_zq_file, serialize_zq_file,
    ZQ_META_PREFIX, ZQ_META_SUFFIX
};

pub fn render_to_job(
    text: &str,
    spans: &[StyledSpan],
    font_size: f32,
    font_family: FontFamily,
    default_color: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let len = text.len();
    if len == 0 { return job; }

    for span in spans {
        let s = span.start.min(len);
        let e = span.end.min(len);
        let text_segment = &text[s..e];
        
        let mut fmt = TextFormat {
            font_id: FontId::new(font_size * span.size_mult, font_family.clone()),
            color: span.fg.map(|c| Color32::from_rgb(c[0], c[1], c[2])).unwrap_or(default_color),
            background: span.bg.map(|c| Color32::from_rgb(c[0], c[1], c[2])).unwrap_or(Color32::TRANSPARENT),
            italics: span.italic,
            ..Default::default()
        };
        if span.strikethrough {
            fmt.strikethrough = eframe::egui::Stroke::new(1.0, fmt.color);
        }
        job.append(text_segment, 0.0, fmt);
    }
    job
}
