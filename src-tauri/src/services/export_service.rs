use deunicode::deunicode;

use crate::error::{AppError, AppResult};

pub fn safe_export_name(value: &str) -> String {
    let transliterated = deunicode(value);
    let mut normalized = String::new();
    let mut last_was_separator = false;

    for ch in transliterated.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !normalized.is_empty() && !last_was_separator {
            normalized.push('-');
            last_was_separator = true;
        }
    }

    while normalized.ends_with('-') {
        normalized.pop();
    }

    if normalized.is_empty() {
        "export".into()
    } else {
        normalized
    }
}

pub fn normalize_export_filename(base_name: &str, extension: &str) -> AppResult<String> {
    let extension = normalize_extension(extension)?;
    Ok(format!("{}.{}", safe_export_name(base_name), extension))
}

fn normalize_extension(extension: &str) -> AppResult<String> {
    let extension = extension.trim().trim_start_matches('.').to_ascii_lowercase();
    if extension.is_empty() || !extension.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(AppError::InvalidExportExtension(extension));
    }

    Ok(extension)
}
