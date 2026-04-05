use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::services::url_import::{
    import_image_from_url as import_image_from_remote_url, parse_supported_http_url,
    validate_image_content_type,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportImageFromUrlRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportImageFromUrlResponse {
    pub url: String,
    pub display_name: Option<String>,
    pub content_type: String,
    pub image_bytes: Vec<u8>,
}

#[tauri::command]
pub fn validate_image_url_content_type(url: String, content_type: String) -> AppResult<()> {
    parse_supported_http_url(&url)?;
    validate_image_content_type(&content_type)
}

#[tauri::command]
pub async fn import_image_from_url(
    request: ImportImageFromUrlRequest,
) -> AppResult<ImportImageFromUrlResponse> {
    let imported = import_image_from_remote_url(&request.url).await?;

    Ok(ImportImageFromUrlResponse {
        url: imported.source_url,
        display_name: imported.display_name,
        content_type: imported.content_type,
        image_bytes: imported.image_bytes,
    })
}
