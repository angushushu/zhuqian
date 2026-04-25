use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{Color32, FontId, FontFamily};
pub use zq_core::{
    StyledSpan, compute_stats, parse_markdown_to_spans,
    ZqTheme, DisplayPrefs, Language, BaseTheme, get_strings, LangStrings,
    extract_headings,
    deserialize_zq_file, serialize_zq_file,
    LabelType, SemanticLabel, TagCode, RelationCode,
    parse_semantic_labels, parse_semantic_labels_with_delim, group_labels_by_category,
    strip_semantic_labels, strip_all_labels_regex, get_label_color,
    ZqTemplate, validate_document, parse_template_directive,
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

        if span.is_hidden {
            let fmt = TextFormat {
                font_id: FontId::new(0.1, font_family.clone()),
                color: Color32::TRANSPARENT,
                ..Default::default()
            };
            job.append(text_segment, 0.0, fmt);
            continue;
        }

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
