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
    pub parent_category: Option<String>,
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

    // 3.2 Algorithm state
    let mut counters = Vec::new(); // count[depth] -> index 0 corresponds to depth 1
    let mut id_stack = Vec::new(); // identifier at each depth

    for mat in re.find_iter(text) {
        let start = mat.start();
        let end = mat.end();
        let inner = &text[start + open_len..end - close.len()];
        let line = text[..start].lines().count();

        if let Some(caps) = inner_re.captures(inner) {
            // 1. Parse depth and category
            let leading_dots = caps.get(1).map_or(0, |m| m.as_str().len());
            let raw_category = caps.get(2).map_or("", |m| m.as_str().trim()).to_string();
            let internal_dots = raw_category.chars().filter(|&c| c == '.').count();
            let depth = leading_dots + internal_dots + 1;
            let text_val = caps.get(3).map_or("", |m| m.as_str().trim()).to_string();
            let props_str = caps.get(4).map_or("", |m| m.as_str().trim());

            let properties: Vec<String> = if props_str.is_empty() {
                Vec::new()
            } else {
                props_str.split('|').map(|s| s.trim().to_string()).collect()
            };

            // 2. Reset deeper state
            if counters.len() > depth {
                counters.truncate(depth);
            }
            if id_stack.len() > depth {
                id_stack.truncate(depth);
            }
            // Ensure capacity
            while counters.len() < depth {
                counters.push(0);
            }
            while id_stack.len() < depth {
                id_stack.push(String::new());
            }

            // 3. Determine Identifier
            let category = if leading_dots > 0 {
                // Relative path: inherit from parent
                let parent_id = id_stack.get(depth - 2).cloned().unwrap_or_default();
                if !raw_category.is_empty() {
                    if parent_id.is_empty() {
                        raw_category.clone()
                    } else {
                        format!("{}.{}", parent_id, raw_category)
                    }
                } else {
                    counters[depth - 1] += 1;
                    if parent_id.is_empty() {
                        counters[depth - 1].to_string()
                    } else {
                        format!("{}.{}", parent_id, counters[depth - 1])
                    }
                }
            } else {
                // Absolute path: use verbatim or auto-index if empty
                if !raw_category.is_empty() {
                    raw_category.clone()
                } else {
                    counters[depth - 1] += 1;
                    counters[depth - 1].to_string()
                }
            };

            // 4. Update ID stack
            id_stack[depth - 1] = category.clone();

            labels.push(SemanticLabel {
                depth,
                category,
                parent_category: None,
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

/// Strip all text matching the semantic label pattern from a string.
/// Robust version using regex, useful for partial selections.
pub fn strip_all_labels_regex(text: &str) -> String {
    let open_esc = regex::escape(DEFAULT_OPEN);
    let close_esc = regex::escape(DEFAULT_CLOSE);
    let pattern = format!("{}[^{}{}]+{}", open_esc, open_esc, close_esc, close_esc);
    if let Ok(re) = Regex::new(&pattern) {
        re.replace_all(text, "").to_string()
    } else {
        text.to_string()
    }
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
    // Try exact match first
    if let Some(lt) = types.iter().find(|t| t.name == category) {
        return lt.color;
    }
    
    // Try parent match (e.g., "s5.1" -> try "s5")
    if let Some(dot_idx) = category.find('.') {
        let parent = &category[..dot_idx];
        if let Some(lt) = types.iter().find(|t| t.name == parent) {
            return lt.color;
        }
    }
    
    fallback
}
