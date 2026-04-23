use regex::Regex;
use serde::{Serialize, Deserialize};

/// Preset color palette for auto-registering new label types.
pub const LABEL_PALETTE: [[u8; 3]; 12] = [
    [230, 80, 80],  [80, 180, 80],  [80, 120, 230], [230, 180, 50],
    [180, 80, 200], [80, 200, 200], [200, 120, 60],  [120, 200, 120],
    [200, 80, 140], [140, 140, 230], [200, 200, 80], [80, 160, 160],
];

/// Default open delimiter.
pub const DEFAULT_OPEN: &str = "[";
/// Default close delimiter.
pub const DEFAULT_CLOSE: &str = "]";

/// A user-defined label type with display properties.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LabelType {
    pub name: String,
    pub color: [u8; 3],
    pub description: String,
}

impl LabelType {
    pub fn note_type() -> Self {
        Self {
            name: "注".into(),
            color: [150, 150, 150],
            description: "写作备注".into(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SemanticLabel {
    pub depth: usize,
    pub category: String,
    pub text: String,
    pub properties: Vec<String>,
    pub line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub raw: String,
}

/// Parse semantic labels according to the rule:
/// [<层级前缀><类别>-<说明文本>|<属性1>|<属性2>|...]
pub fn parse_semantic_labels(text: &str) -> Vec<SemanticLabel> {
    parse_semantic_labels_with_delim(text, DEFAULT_OPEN, DEFAULT_CLOSE)
}

pub fn parse_semantic_labels_with_delim(text: &str, open: &str, close: &str) -> Vec<SemanticLabel> {
    let open_esc = regex::escape(open);
    let close_esc = regex::escape(close);
    let pattern = format!("{}[^{}{}]+{}", open_esc, open_esc, close_esc, close_esc);
    
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let inner_re = Regex::new(r"^(\.*)([^-\|]*?)(?:-([^\|]*))?(?:\|(.*))?$").unwrap();

    let mut labels = Vec::new();
    let open_len = open.len();

    let mut seq_by_depth = [0usize; 10];

    for mat in re.find_iter(text) {
        let start = mat.start();
        let end = mat.end();
        let inner = &text[start + open_len..end - close.len()];
        let line = text[..start].lines().count();

        if let Some(caps) = inner_re.captures(inner) {
            let depth = caps.get(1).map_or(0, |m| m.as_str().len()) + 1; // 0 dots = depth 1, 1 dot = depth 2
            let mut category = caps.get(2).map_or("", |m| m.as_str().trim()).to_string();
            let text_val = caps.get(3).map_or("", |m| m.as_str().trim()).to_string();
            let props_str = caps.get(4).map_or("", |m| m.as_str().trim());
            
            let properties: Vec<String> = if props_str.is_empty() {
                Vec::new()
            } else {
                props_str.split('|').map(|s| s.trim().to_string()).collect()
            };

            // Reset deeper levels
            if depth < 10 {
                for d in (depth + 1)..10 {
                    seq_by_depth[d] = 0;
                }
            }

            // Auto-sequence deduction if category is empty
            if category.is_empty() {
                let d_idx = depth.min(9);
                seq_by_depth[d_idx] += 1;
                category = format!("{}.", seq_by_depth[d_idx]);
            }

            labels.push(SemanticLabel {
                depth,
                category,
                text: text_val,
                properties,
                line: line + 1, // 1-based
                start_byte: start,
                end_byte: end,
                raw: inner.to_string(),
            });
        }
    }

    labels.sort_by_key(|l| l.start_byte);
    labels
}

pub fn strip_semantic_labels(text: &str, labels: &[SemanticLabel]) -> String {
    let mut result = text.to_string();
    let mut sorted: Vec<&SemanticLabel> = labels.iter().collect();
    sorted.sort_by(|a, b| b.start_byte.cmp(&a.start_byte));

    for label in sorted {
        let s = label.start_byte.min(result.len());
        let e = label.end_byte.min(result.len());
        if s < e {
            result.replace_range(s..e, "");
        }
    }
    result
}

pub fn group_labels_by_category(labels: &[SemanticLabel]) -> Vec<(String, Vec<&SemanticLabel>)> {
    let mut groups: Vec<(String, Vec<&SemanticLabel>)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();

    for label in labels {
        let key = label.category.clone();
        if let Some(pos) = seen.iter().position(|s| s == &key) {
            groups[pos].1.push(label);
        } else {
            seen.push(key.clone());
            groups.push((key, vec![label]));
        }
    }
    groups
}

pub fn auto_register_labels(labels: &[SemanticLabel], existing: &[LabelType]) -> Vec<LabelType> {
    let mut types = existing.to_vec();
    let mut next_color_idx = types.len() % LABEL_PALETTE.len();

    for label in labels {
        if !label.category.is_empty() && !label.category.ends_with('.') {
            if types.iter().any(|t| t.name == label.category) { continue; }
            let color = LABEL_PALETTE[next_color_idx];
            next_color_idx = (next_color_idx + 1) % LABEL_PALETTE.len();
            types.push(LabelType {
                name: label.category.clone(),
                color,
                description: String::new(),
            });
        }
    }
    types
}

pub fn get_label_color(category: &str, types: &[LabelType], fallback: [u8; 3]) -> [u8; 3] {
    if let Some(lt) = types.iter().find(|t| t.name == category) {
        return lt.color;
    }
    fallback
}
