use crate::parser;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ExportMode {
    CleanText,       // Strip labels, keep headings
    CleanFlat,       // Strip labels AND heading markers
    Annotated,       // Keep everything as-is
    TemplateReport,  // Validation report against template
}

pub(crate) fn export_text(text: &str, mode: ExportMode) -> String {
    match mode {
        ExportMode::CleanText => {
            let labels = parser::parse_semantic_labels(text);
            parser::strip_semantic_labels(text, &labels)
        }
        ExportMode::CleanFlat => {
            let labels = parser::parse_semantic_labels(text);
            let clean = parser::strip_semantic_labels(text, &labels);
            strip_heading_markers(&clean)
        }
        ExportMode::Annotated => text.to_string(),
        ExportMode::TemplateReport => {
            // Template report is handled separately with template context
            text.to_string()
        }
    }
}

pub(crate) fn export_template_report(text: &str, template: &parser::ZqTemplate) -> String {
    let results = parser::validate_document(text, template);
    let mut report = String::new();
    report.push_str(&format!("Template: {}\n", template.name));
    report.push_str(&format!("{}\n\n", template.description));
    report.push_str("--- Validation Results ---\n\n");

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    report.push_str(&format!("Passed: {} | Failed: {}\n\n", passed, failed));

    for r in &results {
        report.push_str(&format!("{} {}\n", r.message, if r.passed { "" } else { " ⚠" }));
    }
    report
}

/// Remove `#` heading markers from lines, keeping the title text.
fn strip_heading_markers(text: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            let title = trimmed[level..].trim();
            result.push_str(title);
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_heading_markers() {
        let text = "## 破题\n天下之事。\n### 子标题\n内容";
        let flat = strip_heading_markers(text);
        assert!(flat.contains("破题"));
        assert!(flat.contains("天下之事。"));
        assert!(!flat.contains("##"));
    }

    #[test]
    fn test_export_clean_text() {
        let text = "张三[角色]走进客栈[场景]。[伏笔:红伞]角落那把伞。";
        let clean = export_text(text, ExportMode::CleanText);
        assert_eq!(clean, "张三走进客栈。角落那把伞。");
    }

    #[test]
    fn test_export_clean_flat() {
        let text = "## 第一章\n张三[角色]走进客栈。";
        let flat = export_text(text, ExportMode::CleanFlat);
        assert_eq!(flat.trim(), "第一章\n张三走进客栈。");
    }
}
