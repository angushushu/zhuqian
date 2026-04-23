use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ZqTemplate {
    pub name: String,
    pub description: String,
    pub expected_headings: Vec<TemplateHeading>,
    pub label_types: Vec<TemplateLabelType>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TemplateHeading {
    pub level: usize,
    pub title: Option<String>,  // None = any title allowed
    pub required: bool,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TemplateLabelType {
    pub name: String,
    pub color: [u8; 3],
    pub description: String,
    pub required: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ValidationResult {
    pub rule_id: String,
    pub passed: bool,
    pub message: String,
    pub line: Option<usize>,
}

/// Parse `@template:name` directive from the first line of text.
pub fn parse_template_directive(text: &str) -> Option<String> {
    let first_line = text.lines().next()?;
    let trimmed = first_line.trim();
    if let Some(rest) = trimmed.strip_prefix("@template:") {
        Some(rest.trim().to_string())
    } else {
        None
    }
}

/// Validate a document against a template.
pub fn validate_document(text: &str, template: &ZqTemplate) -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let headings = crate::extract_headings(text);
    let labels = crate::parse_semantic_labels(text);

    // Check required headings
    for th in &template.expected_headings {
        if !th.required { continue; }
        let found = headings.iter().any(|h| {
            if let Some(ref expected_title) = th.title {
                h.1 == th.level && h.2.contains(expected_title.as_str())
            } else {
                h.1 == th.level
            }
        });
        let title_desc = match th.title.as_deref() {
            Some(t) => t.to_string(),
            None => format!("H{}", th.level),
        };
        results.push(ValidationResult {
            rule_id: format!("heading_{}", title_desc),
            passed: found,
            message: if found {
                format!("✓ {} section present", title_desc)
            } else {
                format!("✗ Missing required section: {}", title_desc)
            },
            line: None,
        });
    }

    // Check required label types
    for lt in &template.label_types {
        if !lt.required { continue; }
        let found = labels.iter().any(|l| l.category == lt.name);
        results.push(ValidationResult {
            rule_id: format!("label_{}", lt.name),
            passed: found,
            message: if found {
                format!("✓ Label type '{}' used", lt.name)
            } else {
                format!("✗ Missing required label type: {}", lt.name)
            },
            line: None,
        });
    }

    results
}

/// Built-in templates as default options.
impl ZqTemplate {
    pub fn freeform() -> Self {
        Self {
            name: "freeform".into(),
            description: "No constraints — free writing".into(),
            expected_headings: vec![],
            label_types: vec![],
        }
    }

    pub fn baguwen() -> Self {
        Self {
            name: "八股文".into(),
            description: "传统八股文结构".into(),
            expected_headings: vec![
                TemplateHeading { level: 2, title: Some("破题".into()), required: true, description: "两句点明题意".into() },
                TemplateHeading { level: 2, title: Some("承题".into()), required: true, description: "承接引申".into() },
                TemplateHeading { level: 2, title: Some("起讲".into()), required: true, description: "开始议论".into() },
                TemplateHeading { level: 2, title: Some("入手".into()), required: true, description: "切入正题".into() },
                TemplateHeading { level: 2, title: Some("起股".into()), required: true, description: "第一对偶段".into() },
                TemplateHeading { level: 2, title: Some("中股".into()), required: true, description: "第二对偶段（核心）".into() },
                TemplateHeading { level: 2, title: Some("后股".into()), required: true, description: "第三对偶段".into() },
                TemplateHeading { level: 2, title: Some("束股".into()), required: true, description: "收束全篇".into() },
            ],
            label_types: vec![
                TemplateLabelType { name: "引用".into(), color: [100, 100, 200], description: "引用出处".into(), required: false },
                TemplateLabelType { name: "论点".into(), color: [200, 100, 100], description: "核心论点".into(), required: false },
                TemplateLabelType { name: "论据".into(), color: [100, 180, 100], description: "支持论据".into(), required: false },
                TemplateLabelType { name: "对仗上".into(), color: [180, 140, 60], description: "对偶上半句".into(), required: false },
                TemplateLabelType { name: "对仗下".into(), color: [180, 140, 60], description: "对偶下半句".into(), required: false },
                TemplateLabelType { name: "转折".into(), color: [180, 60, 180], description: "文意转折".into(), required: false },
                TemplateLabelType { name: "总结".into(), color: [60, 160, 160], description: "段落总结".into(), required: false },
            ],
        }
    }

    pub fn apa() -> Self {
        Self {
            name: "APA".into(),
            description: "APA format academic paper".into(),
            expected_headings: vec![
                TemplateHeading { level: 1, title: None, required: true, description: "Paper title".into() },
                TemplateHeading { level: 2, title: Some("Abstract".into()), required: true, description: "Summary (150-250 words)".into() },
                TemplateHeading { level: 2, title: Some("Introduction".into()), required: true, description: "Background and research question".into() },
                TemplateHeading { level: 2, title: Some("Method".into()), required: true, description: "Methodology".into() },
                TemplateHeading { level: 2, title: Some("Results".into()), required: true, description: "Findings".into() },
                TemplateHeading { level: 2, title: Some("Discussion".into()), required: true, description: "Interpretation".into() },
                TemplateHeading { level: 2, title: Some("References".into()), required: true, description: "Bibliography".into() },
            ],
            label_types: vec![
                TemplateLabelType { name: "cite".into(), color: [100, 100, 200], description: "Citation".into(), required: false },
                TemplateLabelType { name: "gap".into(), color: [200, 150, 50], description: "Research gap".into(), required: false },
                TemplateLabelType { name: "method".into(), color: [100, 180, 100], description: "Methodology note".into(), required: false },
                TemplateLabelType { name: "finding".into(), color: [200, 80, 80], description: "Key finding".into(), required: false },
                TemplateLabelType { name: "limit".into(), color: [180, 180, 60], description: "Limitation".into(), required: false },
                TemplateLabelType { name: "note".into(), color: [150, 150, 150], description: "Author note".into(), required: false },
            ],
        }
    }

    pub fn screenplay() -> Self {
        Self {
            name: "Screenplay".into(),
            description: "Film/TV screenplay".into(),
            expected_headings: vec![],
            label_types: vec![
                TemplateLabelType { name: "scene".into(), color: [180, 60, 60], description: "Scene heading".into(), required: false },
                TemplateLabelType { name: "action".into(), color: [100, 100, 100], description: "Action description".into(), required: false },
                TemplateLabelType { name: "dialog".into(), color: [60, 120, 180], description: "Dialogue".into(), required: false },
                TemplateLabelType { name: "paren".into(), color: [150, 150, 150], description: "Parenthetical".into(), required: false },
                TemplateLabelType { name: "transition".into(), color: [120, 60, 180], description: "Scene transition".into(), required: false },
                TemplateLabelType { name: "character".into(), color: [60, 160, 60], description: "Character name".into(), required: false },
            ],
        }
    }

    pub fn all_builtins() -> Vec<Self> {
        vec![Self::freeform(), Self::baguwen(), Self::apa(), Self::screenplay()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_directive() {
        assert_eq!(parse_template_directive("@template:八股文\nrest"), Some("八股文".to_string()));
        assert_eq!(parse_template_directive("@template:APA\n# Title"), Some("APA".to_string()));
        assert_eq!(parse_template_directive("# Just a title"), None);
        assert_eq!(parse_template_directive(""), None);
    }

    #[test]
    fn test_extract_headings_debug() {
        let headings = super::super::extract_headings("## 破题\n天下之事。\n## 承题\n故君子慎其所由。");
        for (ln, lv, title) in &headings {
            eprintln!("line={}, level={}, title='{}'", ln, lv, title);
        }
        assert!(!headings.is_empty());
        assert_eq!(headings[0].2, "破题");
    }

    #[test]
    fn test_validate_baguwen() {
        let template = ZqTemplate::baguwen();
        let text = "## 破题\n天下之事。\n## 承题\n故君子慎其所由。\n## 起讲\n盖闻圣人治天下。";
        let results = validate_document(text, &template);
        for r in &results {
            eprintln!("{}: {} ({})", r.rule_id, r.message, if r.passed { "PASS" } else { "FAIL" });
        }
        let passed = results.iter().filter(|r| r.passed).count();
        assert!(passed >= 3, "Expected at least 3 passed, got {}", passed);
    }

    #[test]
    fn test_validate_apa() {
        let template = ZqTemplate::apa();
        let text = "# My Paper\n## Abstract\nSummary.\n## Introduction\nBackground.\n## Method\nDetails.\n## Results\nData.\n## Discussion\nAnalysis.\n## References\nBibliography.";
        let results = validate_document(text, &template);
        // All 7 headings should be found, 6 label types (all optional)
        let heading_failed = results.iter()
            .filter(|r| !r.passed && r.rule_id.starts_with("heading_"))
            .count();
        assert_eq!(heading_failed, 0, "All headings should match");
    }
}
