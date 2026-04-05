use std::time::Duration;

use reqwest::header::CONTENT_TYPE;
use reqwest::Url;

use crate::error::{AppError, AppResult};

const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/webp",
    "image/gif",
    "image/bmp",
    "image/tiff",
    "image/heic",
    "image/heif",
];

#[derive(Debug, Clone)]
pub struct ImportedImageData {
    pub source_url: String,
    pub display_name: Option<String>,
    pub content_type: String,
    pub image_bytes: Vec<u8>,
}

pub fn parse_supported_http_url(raw_url: &str) -> AppResult<Url> {
    let url = Url::parse(raw_url).map_err(|error| AppError::UrlImportRequest(error.to_string()))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        _ => Err(AppError::UnsupportedUrlScheme(raw_url.to_string())),
    }
}

pub fn extract_display_name_from_url(raw_url: &str) -> Option<String> {
    let parsed = Url::parse(raw_url).ok()?;
    let segment = parsed.path_segments()?.next_back()?;
    let normalized = segment.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

pub fn normalize_image_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

pub fn validate_image_content_type(content_type: &str) -> AppResult<()> {
    let normalized = normalize_image_content_type(content_type);
    if ALLOWED_IMAGE_TYPES.contains(&normalized.as_str()) {
        return Ok(());
    }

    Err(AppError::UnsupportedContentType(normalized))
}

pub async fn import_image_from_url(raw_url: &str) -> AppResult<ImportedImageData> {
    let requested_url = parse_supported_http_url(raw_url)?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| AppError::UrlImportRequest(error.to_string()))?;

    let response = client
        .get(requested_url.clone())
        .send()
        .await
        .map_err(|error| AppError::UrlImportRequest(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::UrlImportHttpStatus(response.status().as_u16()));
    }

    let source_url = response.url().to_string();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(normalize_image_content_type)
        .unwrap_or_default();
    validate_image_content_type(&content_type)?;

    let image_bytes = response
        .bytes()
        .await
        .map_err(|error| AppError::UrlImportRequest(error.to_string()))?
        .to_vec();
    if image_bytes.is_empty() {
        return Err(AppError::UrlImportEmptyBody);
    }

    Ok(ImportedImageData {
        source_url: source_url.clone(),
        display_name: extract_display_name_from_url(&source_url)
            .or_else(|| extract_display_name_from_url(requested_url.as_str())),
        content_type,
        image_bytes,
    })
}
