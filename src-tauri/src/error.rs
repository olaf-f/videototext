use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("settings I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("settings serialization failed: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("settings path resolution failed")]
    SettingsPathUnavailable,
    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),
    #[error("unsupported url scheme: {0}")]
    UnsupportedUrlScheme(String),
    #[error("url import request failed: {0}")]
    UrlImportRequest(String),
    #[error("url import request returned non-success status: {0}")]
    UrlImportHttpStatus(u16),
    #[error("url import response body is empty")]
    UrlImportEmptyBody,
    #[error("OCR asset directory missing: {kind} at {path}")]
    MissingOcrAssetDirectory { kind: &'static str, path: String },
    #[error("OCR asset file missing: {kind} at {path}")]
    MissingOcrAssetFile { kind: &'static str, path: String },
    #[error("image decode failed: {0}")]
    ImageDecode(#[from] image::ImageError),
    #[error("OCR execution task failed: {0}")]
    OcrTaskJoin(String),
    #[error("offline OCR runtime failed: {0}")]
    OcrRuntime(String),
    #[error("DeepSeek API key is not configured")]
    DeepSeekApiKeyUnavailable,
    #[error("DeepSeek request failed: {0}")]
    DeepSeekRequest(String),
    #[error("DeepSeek API returned an error: {0}")]
    DeepSeekApi(String),
    #[error("DeepSeek API returned no message content")]
    DeepSeekEmptyResponse,
    #[error("keyring operation failed: {0}")]
    Keyring(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("invalid export extension: {0}")]
    InvalidExportExtension(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
