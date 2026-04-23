use eframe::egui;

#[derive(Clone)]
pub(crate) struct FontEntry {
    pub key: String,
    pub display: String,
}

/// Scan Windows Fonts dir, validate, extract font names, register families.
pub(crate) fn load_system_fonts(fonts: &mut egui::FontDefinitions) -> Vec<FontEntry> {
    let font_dir = "C:\\Windows\\Fonts";
    let mut loaded: Vec<FontEntry> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(font_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext != "ttf" && ext != "ttc" && ext != "otf" { continue; }

            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let stem_lower = stem.to_lowercase();
            if stem_lower.ends_with("bd") || stem_lower.ends_with("bi")
                || (stem_lower.ends_with('b') && stem_lower.len() > 3)
                || (stem_lower.ends_with('i') && stem_lower.len() > 3)
                || stem_lower.ends_with("it")
                || (stem_lower.ends_with('l') && stem_lower.len() > 3)
                || stem_lower.ends_with("li")
                || (stem_lower.ends_with('z') && stem_lower.len() > 3) {
                continue;
            }

            if let Ok(data) = std::fs::read(&path) {
                let face = match ttf_parser::Face::parse(&data, 0) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let display_name = face.names()
                    .into_iter()
                    .filter(|n| n.name_id == ttf_parser::name_id::FAMILY)
                    .find_map(|n| n.to_string())
                    .unwrap_or_else(|| stem.clone());

                let key = stem.clone();
                fonts.font_data.insert(key.clone(), egui::FontData::from_owned(data).into());
                fonts.families.insert(
                    egui::FontFamily::Name(key.clone().into()),
                    vec![key.clone()],
                );
                loaded.push(FontEntry { key, display: display_name });
            }
        }
    }
    loaded.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));
    loaded.dedup_by(|a, b| a.display.to_lowercase() == b.display.to_lowercase());
    loaded
}
